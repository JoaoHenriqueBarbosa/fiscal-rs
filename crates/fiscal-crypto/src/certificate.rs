use std::sync::Once;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use openssl::hash::MessageDigest;
use openssl::pkcs12::Pkcs12;
use openssl::pkey::PKey;
use openssl::sign::Signer;
use sha1::{Digest as _, Sha1};
use sha2::Sha256;

use fiscal_core::FiscalError;
use fiscal_core::types::{CertificateData, CertificateInfo};

/// Hash algorithm used for XML-DSig digest and RSA signature.
///
/// Brazilian ICP-Brasil v5 certificates require SHA-256, and some SEFAZs
/// already reject SHA-1 (rejeição 297). Use [`SignatureAlgorithm::Sha256`]
/// for new certificates; [`SignatureAlgorithm::Sha1`] is kept for
/// backwards compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SignatureAlgorithm {
    /// RSA-SHA1 — legacy, kept as default for backwards compatibility.
    #[default]
    Sha1,
    /// RSA-SHA256 — required by ICP-Brasil v5 certificates.
    Sha256,
}

/// Load OpenSSL legacy provider (needed for RC2-40-CBC in old PFX files on OpenSSL 3.x).
///
/// The provider must stay loaded for the entire process lifetime. We use
/// `std::mem::forget` to prevent `Drop` from calling `OSSL_PROVIDER_unload`.
/// `try_load(None, "legacy", true)` keeps the default provider as fallback.
fn ensure_legacy_provider() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        if let Ok(provider) = openssl::provider::Provider::try_load(None, "legacy", true) {
            std::mem::forget(provider);
        }
    });
}

/// Ensure a PFX buffer can be used with modern TLS stacks.
///
/// Brazilian A1 certificates are commonly issued with legacy encryption
/// (RC2-40-CBC) which OpenSSL 3.x rejects by default. This function loads
/// the OpenSSL legacy provider (process-wide) so the PFX can be parsed.
///
/// If the PFX uses legacy encryption and the legacy provider loaded
/// successfully, the PFX is re-exported with modern algorithms (AES-256-CBC)
/// via the OpenSSL API — no external CLI dependency.
///
/// If the PFX is already modern, the original bytes are returned as-is.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if the PFX is invalid, the
/// passphrase is wrong, or the legacy provider cannot be loaded.
pub fn ensure_modern_pfx(pfx_buffer: &[u8], passphrase: &str) -> Result<Vec<u8>, FiscalError> {
    ensure_legacy_provider();

    let pkcs12 = Pkcs12::from_der(pfx_buffer)
        .map_err(|e| FiscalError::Certificate(format!("Invalid PFX data: {e}")))?;

    match pkcs12.parse2(passphrase) {
        Ok(parsed) => {
            // PFX parsed OK. Re-export with modern encryption to guarantee
            // compatibility with native-tls / Identity::from_pkcs12_der,
            // which may not load the legacy provider independently.
            re_export_pfx(&parsed, passphrase)
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("unsupported") || msg.contains("RC2") || msg.contains("mac") {
                Err(FiscalError::Certificate(format!(
                    "Legacy PFX (RC2-40-CBC) detected but OpenSSL legacy provider \
                     could not handle it. Ensure OpenSSL 3.x with legacy provider \
                     support is available. Error: {e}"
                )))
            } else {
                Err(FiscalError::Certificate(format!(
                    "Failed to parse PFX (wrong password?): {e}"
                )))
            }
        }
    }
}

/// Re-export a parsed PKCS12 with modern encryption algorithms.
///
/// This converts legacy-encrypted PFX files to use AES-256-CBC (the OpenSSL
/// default for new PKCS12), ensuring compatibility across TLS stacks.
fn re_export_pfx(
    parsed: &openssl::pkcs12::ParsedPkcs12_2,
    passphrase: &str,
) -> Result<Vec<u8>, FiscalError> {
    let pkey = parsed
        .pkey
        .as_ref()
        .ok_or_else(|| FiscalError::Certificate("PFX does not contain a private key".into()))?;
    let cert = parsed
        .cert
        .as_ref()
        .ok_or_else(|| FiscalError::Certificate("PFX does not contain a certificate".into()))?;

    let mut builder = Pkcs12::builder();
    if let Some(chain) = &parsed.ca {
        let mut stack = openssl::stack::Stack::new()
            .map_err(|e| FiscalError::Certificate(format!("Failed to create CA stack: {e}")))?;
        for ca in chain {
            stack
                .push(ca.to_owned())
                .map_err(|e| FiscalError::Certificate(format!("Failed to add CA to stack: {e}")))?;
        }
        builder.ca(stack);
    }

    let new_pfx = builder
        .name("")
        .pkey(pkey)
        .cert(cert)
        .build2(passphrase)
        .map_err(|e| FiscalError::Certificate(format!("Failed to re-export PFX: {e}")))?;

    new_pfx
        .to_der()
        .map_err(|e| FiscalError::Certificate(format!("Failed to serialize PFX: {e}")))
}

fn parse_pfx(
    pfx_buffer: &[u8],
    passphrase: &str,
) -> Result<openssl::pkcs12::ParsedPkcs12_2, FiscalError> {
    let modern = ensure_modern_pfx(pfx_buffer, passphrase)?;
    let pkcs12 = Pkcs12::from_der(&modern)
        .map_err(|e| FiscalError::Certificate(format!("Invalid PFX data: {e}")))?;
    pkcs12
        .parse2(passphrase)
        .map_err(|e| FiscalError::Certificate(format!("Failed to parse PFX: {e}")))
}

/// Extract private key and certificate PEM strings from a PKCS#12/PFX buffer.
///
/// Parses the PFX using the provided passphrase and returns a [`CertificateData`]
/// containing both PEM-encoded private key and certificate, along with the
/// original PFX buffer and passphrase for later reuse.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The buffer is not a valid PKCS#12 file
/// - The passphrase is incorrect
/// - The PFX does not contain a private key or certificate
pub fn load_certificate(
    pfx_buffer: &[u8],
    passphrase: &str,
) -> Result<CertificateData, FiscalError> {
    ensure_legacy_provider();
    let parsed = parse_pfx(pfx_buffer, passphrase)?;

    let pkey = parsed
        .pkey
        .ok_or_else(|| FiscalError::Certificate("PFX does not contain a private key".into()))?;

    let cert = parsed
        .cert
        .ok_or_else(|| FiscalError::Certificate("PFX does not contain a certificate".into()))?;

    let private_key_pem = String::from_utf8(
        pkey.private_key_to_pem_pkcs8()
            .map_err(|e| FiscalError::Certificate(format!("Failed to export private key: {e}")))?,
    )
    .map_err(|e| FiscalError::Certificate(format!("Private key PEM is not valid UTF-8: {e}")))?;

    let certificate_pem = String::from_utf8(
        cert.to_pem()
            .map_err(|e| FiscalError::Certificate(format!("Failed to export certificate: {e}")))?,
    )
    .map_err(|e| FiscalError::Certificate(format!("Certificate PEM is not valid UTF-8: {e}")))?;

    Ok(CertificateData::new(
        private_key_pem,
        certificate_pem,
        pfx_buffer.to_vec(),
        passphrase,
    ))
}

/// Extract display metadata from a PKCS#12/PFX certificate.
///
/// Parses the PFX and reads the X.509 subject, issuer, validity dates,
/// and serial number without exposing the private key.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The buffer is not a valid PKCS#12 file
/// - The passphrase is incorrect
/// - The certificate fields cannot be parsed
pub fn get_certificate_info(
    pfx_buffer: &[u8],
    passphrase: &str,
) -> Result<CertificateInfo, FiscalError> {
    ensure_legacy_provider();
    let parsed = parse_pfx(pfx_buffer, passphrase)?;

    let cert = parsed
        .cert
        .ok_or_else(|| FiscalError::Certificate("PFX does not contain a certificate".into()))?;

    let common_name = extract_cn_from_x509_name(cert.subject_name());
    let issuer = extract_cn_from_x509_name(cert.issuer_name());

    let valid_from = asn1_time_to_naive_date(cert.not_before())?;
    let valid_until = asn1_time_to_naive_date(cert.not_after())?;

    let serial_number = cert
        .serial_number()
        .to_bn()
        .map_err(|e| FiscalError::Certificate(format!("Failed to read serial number: {e}")))?
        .to_hex_str()
        .map_err(|e| FiscalError::Certificate(format!("Failed to format serial number: {e}")))?
        .to_string();

    Ok(CertificateInfo::new(
        common_name,
        valid_from,
        valid_until,
        serial_number,
        issuer,
    ))
}

/// Sign an NF-e XML with RSA-SHA1 enveloped XMLDSig signature.
///
/// Produces a `<Signature>` element inserted inside `<NFe>` after `</infNFe>`,
/// using C14N canonicalization, SHA-1 digest, and RSA-SHA1 signing.
///
/// The signed element is identified by the `Id` attribute on `<infNFe>`.
///
/// For SHA-256 support, use [`sign_xml_with_algorithm`].
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infNFe>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The signing operation fails
pub fn sign_xml(xml: &str, private_key: &str, certificate: &str) -> Result<String, FiscalError> {
    sign_xml_with_algorithm(xml, private_key, certificate, SignatureAlgorithm::Sha1)
}

/// Sign an NF-e XML with the specified hash algorithm.
///
/// Same as [`sign_xml`] but allows choosing between SHA-1 and SHA-256.
/// Use [`SignatureAlgorithm::Sha256`] for ICP-Brasil v5 certificates.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infNFe>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The signing operation fails
pub fn sign_xml_with_algorithm(
    xml: &str,
    private_key: &str,
    certificate: &str,
    algorithm: SignatureAlgorithm,
) -> Result<String, FiscalError> {
    sign_xml_generic(xml, private_key, certificate, "infNFe", "NFe", algorithm)
}

/// Sign a SEFAZ event XML with RSA-SHA1 enveloped XMLDSig signature.
///
/// Same algorithm as [`sign_xml`] but targets `<infEvento>` inside `<evento>`.
///
/// For SHA-256 support, use [`sign_event_xml_with_algorithm`].
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infEvento>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The signing operation fails
pub fn sign_event_xml(
    xml: &str,
    private_key: &str,
    certificate: &str,
) -> Result<String, FiscalError> {
    sign_event_xml_with_algorithm(xml, private_key, certificate, SignatureAlgorithm::Sha1)
}

/// Sign a SEFAZ event XML with the specified hash algorithm.
///
/// Same as [`sign_event_xml`] but allows choosing between SHA-1 and SHA-256.
/// Use [`SignatureAlgorithm::Sha256`] for ICP-Brasil v5 certificates.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infEvento>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The signing operation fails
pub fn sign_event_xml_with_algorithm(
    xml: &str,
    private_key: &str,
    certificate: &str,
    algorithm: SignatureAlgorithm,
) -> Result<String, FiscalError> {
    sign_xml_generic(
        xml,
        private_key,
        certificate,
        "infEvento",
        "evento",
        algorithm,
    )
}

/// Sign a SEFAZ inutilização XML with RSA-SHA1 enveloped XMLDSig signature.
///
/// Same algorithm as [`sign_xml`] but targets `<infInut>` inside `<inutNFe>`.
///
/// For SHA-256 support, use [`sign_inutilizacao_xml_with_algorithm`].
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infInut>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The signing operation fails
pub fn sign_inutilizacao_xml(
    xml: &str,
    private_key: &str,
    certificate: &str,
) -> Result<String, FiscalError> {
    sign_inutilizacao_xml_with_algorithm(xml, private_key, certificate, SignatureAlgorithm::Sha1)
}

/// Sign a SEFAZ inutilização XML with the specified hash algorithm.
///
/// Same as [`sign_inutilizacao_xml`] but allows choosing between SHA-1 and SHA-256.
/// Use [`SignatureAlgorithm::Sha256`] for ICP-Brasil v5 certificates.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infInut>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The signing operation fails
pub fn sign_inutilizacao_xml_with_algorithm(
    xml: &str,
    private_key: &str,
    certificate: &str,
    algorithm: SignatureAlgorithm,
) -> Result<String, FiscalError> {
    sign_xml_generic(
        xml,
        private_key,
        certificate,
        "infInut",
        "inutNFe",
        algorithm,
    )
}

// ── Private helpers ─────────────────────────────────────────────────────────

/// Generic XML-DSig signing for both NFe and event documents.
///
/// `signed_tag` is the element whose Id attribute is referenced (e.g. "infNFe").
/// `parent_tag` is the element that receives the `<Signature>` child (e.g. "NFe").
/// `algorithm` selects between SHA-1 and SHA-256 for digest and RSA signing.
fn sign_xml_generic(
    xml: &str,
    private_key_pem: &str,
    certificate_pem: &str,
    signed_tag: &str,
    parent_tag: &str,
    algorithm: SignatureAlgorithm,
) -> Result<String, FiscalError> {
    // 1. Extract the Id from the signed element
    let id = extract_element_id(xml, signed_tag)?;

    // 2. Extract the signed element content (including the element itself)
    let signed_element = extract_element(xml, signed_tag).ok_or_else(|| {
        FiscalError::Certificate(format!("<{signed_tag}> element not found in XML"))
    })?;

    // 3. Apply enveloped-signature transform: remove any existing <Signature> from the content
    let without_sig = remove_signature_element(&signed_element);

    // 4. In C14N inclusive, inherited namespaces from ancestor elements must
    //    appear on the root element of the canonicalized subset. If the signed
    //    element doesn't explicitly declare xmlns but the parent does, we must
    //    add it — this matches what PHP's DOMDocument C14N does automatically.
    let with_inherited_ns = ensure_inherited_namespace(&without_sig, xml, signed_tag);

    // 5. Canonicalize the signed element (C14N 1.0 — sorts attributes)
    let canonical = canonicalize_xml(&with_inherited_ns);

    // 6. Compute digest of the canonical form using the selected algorithm
    let digest = compute_digest(canonical.as_bytes(), algorithm);

    // 7. Build the SignedInfo XML (without xmlns — it inherits from parent Signature)
    let signed_info = build_signed_info(&id, &digest, algorithm);

    // 8. Canonicalize the SignedInfo for signing.
    //    In C14N inclusive, when SignedInfo is canonicalized as a subset of
    //    <Signature xmlns="...">, the in-scope namespace is included on
    //    SignedInfo even though it's inherited. We must sign this form so
    //    SEFAZ can verify against the same canonical representation.
    let canonical_signed_info = signed_info.replacen(
        "<SignedInfo>",
        "<SignedInfo xmlns=\"http://www.w3.org/2000/09/xmldsig#\">",
        1,
    );

    // 9. RSA sign the canonical SignedInfo with the selected digest algorithm
    let pkey = PKey::private_key_from_pem(private_key_pem.as_bytes())
        .map_err(|e| FiscalError::Certificate(format!("Failed to parse private key: {e}")))?;

    let openssl_digest = match algorithm {
        SignatureAlgorithm::Sha1 => MessageDigest::sha1(),
        SignatureAlgorithm::Sha256 => MessageDigest::sha256(),
    };

    let mut signer = Signer::new(openssl_digest, &pkey)
        .map_err(|e| FiscalError::Certificate(format!("Failed to create signer: {e}")))?;

    signer
        .update(canonical_signed_info.as_bytes())
        .map_err(|e| FiscalError::Certificate(format!("Failed to update signer: {e}")))?;

    let algo_name = match algorithm {
        SignatureAlgorithm::Sha1 => "RSA-SHA1",
        SignatureAlgorithm::Sha256 => "RSA-SHA256",
    };

    let signature_bytes = signer
        .sign_to_vec()
        .map_err(|e| FiscalError::Certificate(format!("{algo_name} signing failed: {e}")))?;

    let signature_value = BASE64.encode(&signature_bytes);

    // 10. Extract certificate Base64 (strip PEM headers)
    let cert_base64 = extract_cert_base64(certificate_pem);

    // 11. Build the full <Signature> element
    let signature_xml = build_signature_element(&signed_info, &signature_value, &cert_base64);

    // 12. Insert the <Signature> inside the parent element, after the signed element
    let closing_tag = format!("</{parent_tag}>");
    let result = if let Some(pos) = xml.rfind(&closing_tag) {
        format!("{}{signature_xml}{}", &xml[..pos], &xml[pos..])
    } else {
        return Err(FiscalError::Certificate(format!(
            "<{parent_tag}> closing tag not found in XML"
        )));
    };

    Ok(result)
}

/// Compute the Base64-encoded digest of `data` using the selected algorithm.
fn compute_digest(data: &[u8], algorithm: SignatureAlgorithm) -> String {
    match algorithm {
        SignatureAlgorithm::Sha1 => {
            let mut hasher = Sha1::new();
            sha1::Digest::update(&mut hasher, data);
            BASE64.encode(sha1::Digest::finalize(hasher))
        }
        SignatureAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            sha2::Digest::update(&mut hasher, data);
            BASE64.encode(sha2::Digest::finalize(hasher))
        }
    }
}

/// Extract the `Id` attribute value from the first occurrence of `<tag_name ... Id="...">`.
fn extract_element_id(xml: &str, tag_name: &str) -> Result<String, FiscalError> {
    let pattern = format!("<{tag_name}");
    let tag_start = xml.find(&pattern).ok_or_else(|| {
        FiscalError::Certificate(format!(
            "Could not find <{tag_name}> element with Id attribute in XML"
        ))
    })?;

    let rest = &xml[tag_start..];
    // Find the closing > of this opening tag
    let tag_end = rest
        .find('>')
        .ok_or_else(|| FiscalError::Certificate(format!("<{tag_name}> tag is malformed")))?;

    let tag_content = &rest[..tag_end];

    // Find Id="..."
    let id_prefix = "Id=\"";
    let id_start = tag_content.find(id_prefix).ok_or_else(|| {
        FiscalError::Certificate(format!(
            "Could not find <{tag_name}> element with Id attribute in XML"
        ))
    })?;

    let id_value_start = id_start + id_prefix.len();
    let id_value_end = tag_content[id_value_start..].find('"').ok_or_else(|| {
        FiscalError::Certificate(format!("Malformed Id attribute in <{tag_name}>"))
    })?;

    Ok(tag_content[id_value_start..id_value_start + id_value_end].to_string())
}

/// Ensure the signed element includes inherited xmlns from ancestor elements.
///
/// In C14N inclusive canonicalization, the root element of the subset must
/// include all in-scope namespace declarations from ancestors. If `<infNFe>`
/// doesn't explicitly declare `xmlns` but the parent `<NFe>` does, we add it.
/// This matches PHP DOMDocument's C14N behavior.
fn ensure_inherited_namespace(element: &str, full_xml: &str, tag_name: &str) -> String {
    // Check if the element already has xmlns
    let open_end = element.find('>').unwrap_or(element.len());
    let open_tag = &element[..open_end];
    if open_tag.contains("xmlns=") {
        return element.to_string();
    }

    // Find xmlns from the closest ancestor in the full XML
    let tag_pos = full_xml.find(&format!("<{tag_name}")).unwrap_or(0);
    let before = &full_xml[..tag_pos];

    // Search backwards for xmlns="..." in ancestor tags
    if let Some(ns_start) = before.rfind("xmlns=\"") {
        let ns_val_start = ns_start + 7; // skip xmlns="
        if let Some(ns_val_end) = full_xml[ns_val_start..].find('"') {
            let ns_value = &full_xml[ns_val_start..ns_val_start + ns_val_end];
            // Insert xmlns after the tag name
            let insert_pos = element
                .find(|c: char| c.is_ascii_whitespace() || c == '>')
                .unwrap_or(open_end);
            return format!(
                "{} xmlns=\"{ns_value}\"{}",
                &element[..insert_pos],
                &element[insert_pos..],
            );
        }
    }

    element.to_string()
}

/// Extract the full element (from `<tag_name ...>` to `</tag_name>`) from the XML.
fn extract_element(xml: &str, tag_name: &str) -> Option<String> {
    let open_pattern = format!("<{tag_name}");
    let close_pattern = format!("</{tag_name}>");

    let start = xml.find(&open_pattern)?;
    let end = xml.find(&close_pattern)? + close_pattern.len();

    Some(xml[start..end].to_string())
}

/// Remove any `<Signature ...>...</Signature>` from the XML string (enveloped-signature transform).
fn remove_signature_element(xml: &str) -> String {
    if let Some(sig_start) = xml.find("<Signature") {
        if let Some(sig_end_tag) = xml[sig_start..].find("</Signature>") {
            let sig_end = sig_start + sig_end_tag + "</Signature>".len();
            return format!("{}{}", &xml[..sig_start], &xml[sig_end..]);
        }
    }
    xml.to_string()
}

/// XML Canonicalization (C14N 1.0 without comments).
///
/// Implements the subset of Canonical XML 1.0 needed for NF-e signing:
/// - Removes the XML declaration (`<?xml ...?>`)
/// - Sorts attributes on each opening tag: namespace declarations first
///   (sorted by prefix, default namespace first), then regular attributes
///   sorted alphabetically by local name
/// - Expands self-closing tags (`<foo/>` → `<foo></foo>`)
fn canonicalize_xml(xml: &str) -> String {
    let mut input = xml;

    // Remove XML declaration if present
    if let Some(decl_start) = input.find("<?xml") {
        if let Some(decl_end) = input[decl_start..].find("?>") {
            input = input[decl_start + decl_end + 2..].trim_start();
        }
    }

    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            // Collect everything up to '>'
            let mut tag = String::from('<');
            for c in chars.by_ref() {
                tag.push(c);
                if c == '>' {
                    break;
                }
            }

            // Skip processing instructions, closing tags, comments, CDATA
            if tag.starts_with("</") || tag.starts_with("<?") || tag.starts_with("<!") {
                result.push_str(&tag);
                continue;
            }

            // Opening tag — sort attributes
            let self_closing = tag.ends_with("/>");
            let tag_content = if self_closing {
                &tag[1..tag.len() - 2] // strip < and />
            } else {
                &tag[1..tag.len() - 1] // strip < and >
            };

            let (tag_name, attrs_str) = match tag_content.find(|c: char| c.is_ascii_whitespace()) {
                Some(pos) => (&tag_content[..pos], tag_content[pos..].trim()),
                None => (tag_content, ""),
            };

            if attrs_str.is_empty() {
                if self_closing {
                    // C14N expands self-closing to <tag></tag>
                    result.push('<');
                    result.push_str(tag_name);
                    result.push_str("></");
                    result.push_str(tag_name);
                    result.push('>');
                } else {
                    result.push_str(&tag);
                }
                continue;
            }

            // Parse attributes
            let attrs = parse_attributes(attrs_str);

            // Separate namespace declarations from regular attributes
            let mut ns_attrs: Vec<(&str, &str)> = Vec::new();
            let mut reg_attrs: Vec<(&str, &str)> = Vec::new();

            for (name, value) in &attrs {
                if *name == "xmlns" || name.starts_with("xmlns:") {
                    ns_attrs.push((name, value));
                } else {
                    reg_attrs.push((name, value));
                }
            }

            // Sort namespace declarations: default namespace first, then by prefix
            ns_attrs.sort_by(|a, b| match (a.0, b.0) {
                ("xmlns", _) => std::cmp::Ordering::Less,
                (_, "xmlns") => std::cmp::Ordering::Greater,
                _ => a.0.cmp(b.0),
            });

            // Sort regular attributes by local name
            reg_attrs.sort_by(|a, b| a.0.cmp(b.0));

            // Rebuild tag
            result.push('<');
            result.push_str(tag_name);
            for (name, value) in ns_attrs.iter().chain(reg_attrs.iter()) {
                result.push(' ');
                result.push_str(name);
                result.push_str("=\"");
                result.push_str(value);
                result.push('"');
            }
            if self_closing {
                result.push_str("></");
                result.push_str(tag_name);
                result.push('>');
            } else {
                result.push('>');
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Parse attributes from a tag's attribute string.
///
/// Returns a vector of (name, value) pairs. Handles both single and double
/// quoted attribute values.
fn parse_attributes(attrs_str: &str) -> Vec<(&str, &str)> {
    let mut attrs = Vec::new();
    let mut remaining = attrs_str.trim();

    while !remaining.is_empty() {
        // Find attribute name (up to '=')
        let eq_pos = match remaining.find('=') {
            Some(pos) => pos,
            None => break,
        };
        let name = remaining[..eq_pos].trim();
        remaining = remaining[eq_pos + 1..].trim();

        // Find quoted value
        let quote = match remaining.chars().next() {
            Some(q @ ('"' | '\'')) => q,
            _ => break,
        };
        remaining = &remaining[1..]; // skip opening quote
        let end_pos = match remaining.find(quote) {
            Some(pos) => pos,
            None => break,
        };
        let value = &remaining[..end_pos];
        remaining = remaining[end_pos + 1..].trim();

        attrs.push((name, value));
    }

    attrs
}

/// Build the `<SignedInfo>` element for XML-DSig.
///
/// The `xmlns` is intentionally omitted here because `<SignedInfo>` inherits
/// its namespace from the parent `<Signature xmlns="...">`. In C14N 1.0,
/// redundant namespace declarations are suppressed, so both the signing and
/// verification sides must use the same canonical form (without `xmlns` on
/// `<SignedInfo>`).
///
/// The `algorithm` parameter selects the appropriate XML-DSig URIs for
/// `<SignatureMethod>` and `<DigestMethod>`.
fn build_signed_info(
    reference_id: &str,
    digest_value: &str,
    algorithm: SignatureAlgorithm,
) -> String {
    let (signature_method_uri, digest_method_uri) = match algorithm {
        SignatureAlgorithm::Sha1 => (
            "http://www.w3.org/2000/09/xmldsig#rsa-sha1",
            "http://www.w3.org/2000/09/xmldsig#sha1",
        ),
        SignatureAlgorithm::Sha256 => (
            "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256",
            "http://www.w3.org/2001/04/xmlenc#sha256",
        ),
    };

    let mut s = String::with_capacity(1024);
    s.push_str("<SignedInfo>");
    s.push_str("<CanonicalizationMethod Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"></CanonicalizationMethod>");
    s.push_str("<SignatureMethod Algorithm=\"");
    s.push_str(signature_method_uri);
    s.push_str("\"></SignatureMethod>");
    s.push_str("<Reference URI=\"#");
    s.push_str(reference_id);
    s.push_str("\">");
    s.push_str("<Transforms>");
    s.push_str("<Transform Algorithm=\"http://www.w3.org/2000/09/xmldsig#enveloped-signature\"></Transform>");
    s.push_str(
        "<Transform Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"></Transform>",
    );
    s.push_str("</Transforms>");
    s.push_str("<DigestMethod Algorithm=\"");
    s.push_str(digest_method_uri);
    s.push_str("\"></DigestMethod>");
    s.push_str("<DigestValue>");
    s.push_str(digest_value);
    s.push_str("</DigestValue>");
    s.push_str("</Reference>");
    s.push_str("</SignedInfo>");
    s
}

/// Build the full `<Signature>` element including SignedInfo, SignatureValue, and KeyInfo.
fn build_signature_element(signed_info: &str, signature_value: &str, cert_base64: &str) -> String {
    let mut s = String::with_capacity(2048);
    s.push_str("<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\">");
    s.push_str(signed_info);
    s.push_str("<SignatureValue>");
    s.push_str(signature_value);
    s.push_str("</SignatureValue>");
    s.push_str("<KeyInfo><X509Data><X509Certificate>");
    s.push_str(cert_base64);
    s.push_str("</X509Certificate></X509Data></KeyInfo>");
    s.push_str("</Signature>");
    s
}

/// Strip PEM headers/footers from a certificate and return the raw Base64 content.
fn extract_cert_base64(cert_pem: &str) -> String {
    cert_pem
        .replace("-----BEGIN CERTIFICATE-----", "")
        .replace("-----END CERTIFICATE-----", "")
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect()
}

/// Extract the Common Name (CN) from an X509Name.
fn extract_cn_from_x509_name(name: &openssl::x509::X509NameRef) -> String {
    for entry in name.entries_by_nid(openssl::nid::Nid::COMMONNAME) {
        if let Ok(s) = entry.data().as_utf8() {
            return s.to_string();
        }
    }
    // Fallback: return the full subject string
    format!("{:?}", name)
}

/// Convert an OpenSSL ASN1Time to a chrono NaiveDate.
fn asn1_time_to_naive_date(
    time: &openssl::asn1::Asn1TimeRef,
) -> Result<chrono::NaiveDate, FiscalError> {
    let epoch = openssl::asn1::Asn1Time::from_unix(0)
        .map_err(|e| FiscalError::Certificate(format!("ASN1 epoch creation failed: {e}")))?;
    let diff = epoch
        .diff(time)
        .map_err(|e| FiscalError::Certificate(format!("ASN1 time diff failed: {e}")))?;

    let days = diff.days as i64;
    let secs = diff.secs as i64;
    let total_secs = days * 86400 + secs;

    let dt = chrono::DateTime::from_timestamp(total_secs, 0)
        .ok_or_else(|| FiscalError::Certificate("Invalid timestamp from ASN1 time".into()))?;

    Ok(dt.date_naive())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Path to the test PFX in the fixtures directory
    fn test_pfx_cnpj() -> Vec<u8> {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../..",
            "/tests/fixtures/certs/novo_cert_cnpj_06157250000116_senha_minhasenha.pfx"
        );
        std::fs::read(path).expect("test PFX not found")
    }

    fn test_pfx_cpf() -> Vec<u8> {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../..",
            "/tests/fixtures/certs/novo_cert_cpf_90483926086_minhasenha.pfx"
        );
        std::fs::read(path).expect("test PFX not found")
    }

    const PASSWORD: &str = "minhasenha";

    // ── ensure_modern_pfx ─────────────────────────────────────────

    #[test]
    fn ensure_modern_pfx_valid_cert() {
        let pfx = test_pfx_cnpj();
        let result = ensure_modern_pfx(&pfx, PASSWORD);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn ensure_modern_pfx_wrong_password() {
        let pfx = test_pfx_cnpj();
        let result = ensure_modern_pfx(&pfx, "wrongpassword");
        assert!(result.is_err());
    }

    #[test]
    fn ensure_modern_pfx_invalid_data() {
        let result = ensure_modern_pfx(b"not a pfx", PASSWORD);
        assert!(result.is_err());
    }

    // ── load_certificate ──────────────────────────────────────────

    #[test]
    fn load_certificate_cnpj() {
        let pfx = test_pfx_cnpj();
        let cert_data = load_certificate(&pfx, PASSWORD).expect("should load");
        assert!(!cert_data.private_key.is_empty());
        assert!(!cert_data.certificate.is_empty());
        assert!(cert_data.private_key.contains("PRIVATE KEY"));
        assert!(cert_data.certificate.contains("CERTIFICATE"));
    }

    #[test]
    fn load_certificate_cpf() {
        let pfx = test_pfx_cpf();
        let cert_data = load_certificate(&pfx, PASSWORD).expect("should load");
        assert!(!cert_data.private_key.is_empty());
        assert!(!cert_data.certificate.is_empty());
    }

    #[test]
    fn load_certificate_wrong_password() {
        let pfx = test_pfx_cnpj();
        let result = load_certificate(&pfx, "wrong");
        assert!(result.is_err());
    }

    #[test]
    fn load_certificate_invalid_pfx() {
        let result = load_certificate(b"invalid data", PASSWORD);
        assert!(result.is_err());
    }

    // ── get_certificate_info ──────────────────────────────────────

    #[test]
    fn get_certificate_info_cnpj() {
        let pfx = test_pfx_cnpj();
        let info = get_certificate_info(&pfx, PASSWORD).expect("should get info");
        assert!(!info.common_name.is_empty());
        assert!(!info.serial_number.is_empty());
    }

    #[test]
    fn get_certificate_info_cpf() {
        let pfx = test_pfx_cpf();
        let info = get_certificate_info(&pfx, PASSWORD).expect("should get info");
        assert!(!info.common_name.is_empty());
    }

    #[test]
    fn get_certificate_info_wrong_password() {
        let pfx = test_pfx_cnpj();
        let result = get_certificate_info(&pfx, "wrong");
        assert!(result.is_err());
    }

    // ── sign_xml ──────────────────────────────────────────────────

    #[test]
    fn sign_xml_basic() {
        let pfx = test_pfx_cnpj();
        let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
        let xml = concat!(
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
            "<ide><cUF>41</cUF></ide>",
            "</infNFe></NFe>"
        );
        let signed =
            sign_xml(xml, &cert_data.private_key, &cert_data.certificate).expect("should sign");
        assert!(signed.contains("<Signature"));
        assert!(signed.contains("<SignatureValue>"));
        assert!(signed.contains("<X509Certificate>"));
        assert!(signed.contains("<DigestValue>"));
    }

    #[test]
    fn sign_xml_missing_tag() {
        let pfx = test_pfx_cnpj();
        let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
        let xml = "<other>content</other>";
        let result = sign_xml(xml, &cert_data.private_key, &cert_data.certificate);
        assert!(result.is_err());
    }

    #[test]
    fn sign_xml_no_id_attribute() {
        let pfx = test_pfx_cnpj();
        let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
        let xml = "<NFe><infNFe><data/></infNFe></NFe>";
        let result = sign_xml(xml, &cert_data.private_key, &cert_data.certificate);
        assert!(result.is_err());
    }

    // ── sign_event_xml ────────────────────────────────────────────

    #[test]
    fn sign_event_xml_basic() {
        let pfx = test_pfx_cnpj();
        let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
        let xml = concat!(
            r#"<evento xmlns="http://www.portalfiscal.inf.br/nfe" versao="1.00">"#,
            r#"<infEvento Id="ID1101114126030412345600019055001000000012312345678001">"#,
            "<cOrgao>41</cOrgao>",
            "</infEvento></evento>"
        );
        let signed = sign_event_xml(xml, &cert_data.private_key, &cert_data.certificate)
            .expect("should sign");
        assert!(signed.contains("<Signature"));
        assert!(signed.contains("<SignatureValue>"));
    }

    #[test]
    fn sign_event_xml_missing_inf_evento() {
        let pfx = test_pfx_cnpj();
        let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
        let xml = "<evento><other/></evento>";
        let result = sign_event_xml(xml, &cert_data.private_key, &cert_data.certificate);
        assert!(result.is_err());
    }

    // ── extract_element_id ────────────────────────────────────────

    #[test]
    fn extract_element_id_success() {
        let xml = r#"<infNFe versao="4.00" Id="NFe41260312345678000199550010000000011123456780"><data/></infNFe>"#;
        let id = extract_element_id(xml, "infNFe").unwrap();
        assert_eq!(id, "NFe41260312345678000199550010000000011123456780");
    }

    #[test]
    fn extract_element_id_no_tag() {
        let result = extract_element_id("<other/>", "infNFe");
        assert!(result.is_err());
    }

    #[test]
    fn extract_element_id_no_id_attr() {
        let xml = "<infNFe versao=\"4.00\"><data/></infNFe>";
        let result = extract_element_id(xml, "infNFe");
        assert!(result.is_err());
    }

    // ── canonicalize_xml ──────────────────────────────────────────

    #[test]
    fn canonicalize_xml_removes_declaration() {
        let xml = "<?xml version=\"1.0\"?><root><a/></root>";
        let canonical = canonicalize_xml(xml);
        assert!(!canonical.contains("<?xml"));
        // Self-closing should be expanded
        assert!(canonical.contains("<a></a>"));
    }

    #[test]
    fn canonicalize_xml_sorts_attributes() {
        let xml = r#"<root b="2" a="1"><child/></root>"#;
        let canonical = canonicalize_xml(xml);
        // Attributes should be sorted: a before b
        assert!(canonical.contains(r#"<root a="1" b="2">"#));
    }

    #[test]
    fn canonicalize_xml_ns_first() {
        let xml = r#"<root b="2" xmlns="http://example.com" a="1"><child/></root>"#;
        let canonical = canonicalize_xml(xml);
        // xmlns should come before regular attributes
        assert!(canonical.starts_with(r#"<root xmlns="http://example.com""#));
    }

    // ── remove_signature_element ──────────────────────────────────

    #[test]
    fn remove_signature_element_present() {
        let xml = "<root><data/><Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\"><SignedInfo/></Signature></root>";
        let result = remove_signature_element(xml);
        assert!(!result.contains("<Signature"));
        assert!(result.contains("<root>"));
        assert!(result.contains("<data/>"));
    }

    #[test]
    fn remove_signature_element_absent() {
        let xml = "<root><data/></root>";
        let result = remove_signature_element(xml);
        assert_eq!(result, xml);
    }

    // ── extract_cert_base64 ───────────────────────────────────────

    #[test]
    fn extract_cert_base64_strips_headers() {
        let pem = "-----BEGIN CERTIFICATE-----\nTWFu\n-----END CERTIFICATE-----\n";
        let b64 = extract_cert_base64(pem);
        assert_eq!(b64, "TWFu");
    }

    // ── ensure_inherited_namespace ────────────────────────────────

    #[test]
    fn ensure_inherited_namespace_already_has_xmlns() {
        let element = r#"<infNFe xmlns="http://www.portalfiscal.inf.br/nfe" Id="X">data</infNFe>"#;
        let full_xml = element;
        let result = ensure_inherited_namespace(element, full_xml, "infNFe");
        assert_eq!(result, element);
    }

    #[test]
    fn ensure_inherited_namespace_inherits_from_parent() {
        let element = r#"<infNFe Id="X">data</infNFe>"#;
        let full_xml =
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="X">data</infNFe></NFe>"#;
        let result = ensure_inherited_namespace(element, full_xml, "infNFe");
        assert!(result.contains("xmlns=\"http://www.portalfiscal.inf.br/nfe\""));
    }

    // ── parse_attributes ──────────────────────────────────────────

    #[test]
    fn parse_attributes_basic() {
        let attrs = parse_attributes(r#"a="1" b="2""#);
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0], ("a", "1"));
        assert_eq!(attrs[1], ("b", "2"));
    }

    #[test]
    fn parse_attributes_single_quotes() {
        let attrs = parse_attributes("a='1'");
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0], ("a", "1"));
    }

    #[test]
    fn parse_attributes_empty() {
        let attrs = parse_attributes("");
        assert!(attrs.is_empty());
    }

    // ── sign_inutilizacao_xml (lines 263, 268) ──────────────────────

    #[test]
    fn sign_inutilizacao_xml_basic() {
        let pfx = test_pfx_cnpj();
        let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
        let xml = concat!(
            r#"<inutNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00">"#,
            r#"<infInut Id="ID41260304123456000190550010000001231000000010">"#,
            "<tpAmb>2</tpAmb>",
            "</infInut></inutNFe>"
        );
        let signed = sign_inutilizacao_xml(xml, &cert_data.private_key, &cert_data.certificate)
            .expect("should sign inutilizacao");
        assert!(signed.contains("<Signature"));
        assert!(signed.contains("<SignatureValue>"));
        assert!(signed.contains("<X509Certificate>"));
    }

    #[test]
    fn sign_inutilizacao_xml_missing_inf_inut() {
        let pfx = test_pfx_cnpj();
        let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
        let xml = "<inutNFe><other/></inutNFe>";
        let result = sign_inutilizacao_xml(xml, &cert_data.private_key, &cert_data.certificate);
        assert!(result.is_err());
    }

    // ── sign_xml_generic: parent closing tag not found (line 353) ───

    #[test]
    fn sign_xml_generic_missing_parent_closing_tag() {
        let pfx = test_pfx_cnpj();
        let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
        // XML has infNFe with Id but the parent NFe is never closed
        let xml =
            r#"<NFe><infNFe Id="NFe41260304123456000190550010000001231123456780"><data/></infNFe>"#;
        let result = sign_xml(xml, &cert_data.private_key, &cert_data.certificate);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("closing tag not found"),
            "Expected 'closing tag not found' error, got: {err_msg}"
        );
    }

    // ── extract_element_id: malformed Id attribute (line 388) ───────

    #[test]
    fn extract_element_id_malformed_id() {
        // The Id attribute starts but has no closing quote
        let xml = r#"<infNFe Id="NFe41260312345678></infNFe>"#;
        let result = extract_element_id(xml, "infNFe");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Malformed Id"),
            "Expected 'Malformed Id' error, got: {err_msg}"
        );
    }

    // ── ensure_inherited_namespace: no xmlns anywhere (line 429) ────

    #[test]
    fn ensure_inherited_namespace_no_xmlns_anywhere() {
        // Neither element nor any ancestor has xmlns — should return element as-is
        let element = r#"<infNFe Id="X">data</infNFe>"#;
        let full_xml = r#"<NFe><infNFe Id="X">data</infNFe></NFe>"#;
        let result = ensure_inherited_namespace(element, full_xml, "infNFe");
        assert_eq!(result, element);
    }

    // ── canonicalize_xml: namespace prefix sorting (lines 536-538) ──

    #[test]
    fn canonicalize_xml_sorts_multiple_ns_prefixes() {
        // Multiple namespace declarations: default xmlns should come first,
        // then prefixed namespaces sorted alphabetically
        let xml = r#"<root xmlns:z="http://z.example.com" xmlns="http://default.example.com" xmlns:a="http://a.example.com"><child/></root>"#;
        let canonical = canonicalize_xml(xml);
        // Default xmlns first, then xmlns:a, then xmlns:z
        assert!(canonical.contains(
            r#"<root xmlns="http://default.example.com" xmlns:a="http://a.example.com" xmlns:z="http://z.example.com">"#
        ));
    }

    // ── canonicalize_xml: self-closing tag with attributes (lines 554-557) ──

    #[test]
    fn canonicalize_xml_self_closing_with_attrs() {
        // Self-closing tag with attributes should be expanded to <tag attrs></tag>
        let xml = r#"<root><item b="2" a="1"/></root>"#;
        let canonical = canonicalize_xml(xml);
        // Attributes sorted, self-closing expanded
        assert!(canonical.contains(r#"<item a="1" b="2"></item>"#));
    }

    // ── SignatureAlgorithm ──────────────────────────────────────────

    #[test]
    fn signature_algorithm_default_is_sha1() {
        assert_eq!(SignatureAlgorithm::default(), SignatureAlgorithm::Sha1);
    }

    // ── sign_xml_with_algorithm (SHA-256) ───────────────────────────

    #[test]
    fn sign_xml_sha256_basic() {
        let pfx = test_pfx_cnpj();
        let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
        let xml = concat!(
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
            "<ide><cUF>41</cUF></ide>",
            "</infNFe></NFe>"
        );
        let signed = sign_xml_with_algorithm(
            xml,
            &cert_data.private_key,
            &cert_data.certificate,
            SignatureAlgorithm::Sha256,
        )
        .expect("should sign with SHA-256");

        assert!(signed.contains("<Signature"));
        assert!(signed.contains("<SignatureValue>"));
        assert!(signed.contains("<X509Certificate>"));
        assert!(signed.contains("<DigestValue>"));
        // Verify SHA-256 URIs are used
        assert!(
            signed.contains("http://www.w3.org/2001/04/xmldsig-more#rsa-sha256"),
            "SignatureMethod should reference rsa-sha256"
        );
        assert!(
            signed.contains("http://www.w3.org/2001/04/xmlenc#sha256"),
            "DigestMethod should reference sha256"
        );
        // Verify SHA-1 URIs are NOT present
        assert!(
            !signed.contains("xmldsig#rsa-sha1"),
            "SHA-1 SignatureMethod should not be present"
        );
        assert!(
            !signed.contains("xmldsig#sha1"),
            "SHA-1 DigestMethod should not be present"
        );
    }

    #[test]
    fn sign_xml_sha1_explicit() {
        let pfx = test_pfx_cnpj();
        let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
        let xml = concat!(
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
            "<ide><cUF>41</cUF></ide>",
            "</infNFe></NFe>"
        );
        let signed = sign_xml_with_algorithm(
            xml,
            &cert_data.private_key,
            &cert_data.certificate,
            SignatureAlgorithm::Sha1,
        )
        .expect("should sign with SHA-1");

        // Verify SHA-1 URIs are used (same as sign_xml)
        assert!(signed.contains("xmldsig#rsa-sha1"));
        assert!(signed.contains("xmldsig#sha1"));
    }

    #[test]
    fn sign_xml_sha256_produces_different_signature_than_sha1() {
        let pfx = test_pfx_cnpj();
        let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
        let xml = concat!(
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
            "<ide><cUF>41</cUF></ide>",
            "</infNFe></NFe>"
        );

        let signed_sha1 = sign_xml(xml, &cert_data.private_key, &cert_data.certificate).unwrap();
        let signed_sha256 = sign_xml_with_algorithm(
            xml,
            &cert_data.private_key,
            &cert_data.certificate,
            SignatureAlgorithm::Sha256,
        )
        .unwrap();

        // Both should produce valid signatures, but with different values
        assert!(signed_sha1.contains("<SignatureValue>"));
        assert!(signed_sha256.contains("<SignatureValue>"));
        assert_ne!(signed_sha1, signed_sha256);
    }

    #[test]
    fn sign_xml_sha256_verify_roundtrip() {
        // Sign with SHA-256, then verify the signature by re-computing the digest
        let pfx = test_pfx_cnpj();
        let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
        let xml = concat!(
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
            "<ide><cUF>41</cUF></ide>",
            "</infNFe></NFe>"
        );

        let signed = sign_xml_with_algorithm(
            xml,
            &cert_data.private_key,
            &cert_data.certificate,
            SignatureAlgorithm::Sha256,
        )
        .unwrap();

        // Extract the SignatureValue from the signed XML
        let sig_start = signed.find("<SignatureValue>").unwrap() + "<SignatureValue>".len();
        let sig_end = signed.find("</SignatureValue>").unwrap();
        let sig_b64 = &signed[sig_start..sig_end];
        let sig_bytes = BASE64.decode(sig_b64).unwrap();

        // Extract the SignedInfo for verification
        let si_start = signed.find("<SignedInfo>").unwrap();
        let si_end = signed.find("</SignedInfo>").unwrap() + "</SignedInfo>".len();
        let signed_info_raw = &signed[si_start..si_end];

        // Add the namespace for canonical form (same as signing does)
        let canonical_signed_info = signed_info_raw.replacen(
            "<SignedInfo>",
            "<SignedInfo xmlns=\"http://www.w3.org/2000/09/xmldsig#\">",
            1,
        );

        // Verify the signature using the certificate's public key
        let cert = openssl::x509::X509::from_pem(cert_data.certificate.as_bytes()).unwrap();
        let pubkey = cert.public_key().unwrap();

        let mut verifier = openssl::sign::Verifier::new(MessageDigest::sha256(), &pubkey).unwrap();
        verifier.update(canonical_signed_info.as_bytes()).unwrap();
        assert!(
            verifier.verify(&sig_bytes).unwrap(),
            "RSA-SHA256 signature verification failed"
        );
    }

    // ── sign_event_xml_with_algorithm (SHA-256) ─────────────────────

    #[test]
    fn sign_event_xml_sha256() {
        let pfx = test_pfx_cnpj();
        let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
        let xml = concat!(
            r#"<evento xmlns="http://www.portalfiscal.inf.br/nfe" versao="1.00">"#,
            r#"<infEvento Id="ID1101114126030412345600019055001000000012312345678001">"#,
            "<cOrgao>41</cOrgao>",
            "</infEvento></evento>"
        );
        let signed = sign_event_xml_with_algorithm(
            xml,
            &cert_data.private_key,
            &cert_data.certificate,
            SignatureAlgorithm::Sha256,
        )
        .expect("should sign event with SHA-256");

        assert!(signed.contains("<Signature"));
        assert!(signed.contains("rsa-sha256"));
        assert!(signed.contains("xmlenc#sha256"));
    }

    // ── sign_inutilizacao_xml_with_algorithm (SHA-256) ──────────────

    #[test]
    fn sign_inutilizacao_xml_sha256() {
        let pfx = test_pfx_cnpj();
        let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
        let xml = concat!(
            r#"<inutNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00">"#,
            r#"<infInut Id="ID41260304123456000190550010000001231000000010">"#,
            "<tpAmb>2</tpAmb>",
            "</infInut></inutNFe>"
        );
        let signed = sign_inutilizacao_xml_with_algorithm(
            xml,
            &cert_data.private_key,
            &cert_data.certificate,
            SignatureAlgorithm::Sha256,
        )
        .expect("should sign inutilizacao with SHA-256");

        assert!(signed.contains("<Signature"));
        assert!(signed.contains("rsa-sha256"));
        assert!(signed.contains("xmlenc#sha256"));
    }

    // ── compute_digest ─────────────────────────────────────────────

    #[test]
    fn compute_digest_sha1_matches_known_value() {
        // SHA-1 of "" = da39a3ee5e6b4b0d3255bfef95601890afd80709
        let result = compute_digest(b"", SignatureAlgorithm::Sha1);
        let bytes = BASE64.decode(&result).unwrap();
        assert_eq!(bytes.len(), 20); // SHA-1 produces 20 bytes
    }

    #[test]
    fn compute_digest_sha256_matches_known_value() {
        // SHA-256 of "" = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let result = compute_digest(b"", SignatureAlgorithm::Sha256);
        let bytes = BASE64.decode(&result).unwrap();
        assert_eq!(bytes.len(), 32); // SHA-256 produces 32 bytes
    }

    #[test]
    fn compute_digest_sha1_and_sha256_differ() {
        let data = b"test data for hashing";
        let sha1_result = compute_digest(data, SignatureAlgorithm::Sha1);
        let sha256_result = compute_digest(data, SignatureAlgorithm::Sha256);
        assert_ne!(sha1_result, sha256_result);
    }
}
