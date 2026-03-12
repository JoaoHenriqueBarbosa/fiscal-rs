use std::sync::Once;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use openssl::hash::MessageDigest;
use openssl::pkcs12::Pkcs12;
use openssl::pkey::PKey;
use openssl::sign::Signer;
use sha1::{Digest, Sha1};

use fiscal_core::FiscalError;
use fiscal_core::types::{CertificateData, CertificateInfo};

/// Load OpenSSL legacy provider (needed for RC2-40-CBC in old PFX files on OpenSSL 3.x).
fn ensure_legacy_provider() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = openssl::provider::Provider::try_load(None, "legacy", true);
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
/// using C14N exclusive canonicalization, SHA-1 digest, and RSA-SHA1 signing.
///
/// The signed element is identified by the `Id` attribute on `<infNFe>`.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infNFe>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The RSA-SHA1 signing operation fails
pub fn sign_xml(xml: &str, private_key: &str, certificate: &str) -> Result<String, FiscalError> {
    sign_xml_generic(xml, private_key, certificate, "infNFe", "NFe")
}

/// Sign a SEFAZ event XML with RSA-SHA1 enveloped XMLDSig signature.
///
/// Same algorithm as [`sign_xml`] but targets `<infEvento>` inside `<evento>`.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infEvento>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The RSA-SHA1 signing operation fails
pub fn sign_event_xml(
    xml: &str,
    private_key: &str,
    certificate: &str,
) -> Result<String, FiscalError> {
    sign_xml_generic(xml, private_key, certificate, "infEvento", "evento")
}

// ── Private helpers ─────────────────────────────────────────────────────────

/// Generic XML-DSig signing for both NFe and event documents.
///
/// `signed_tag` is the element whose Id attribute is referenced (e.g. "infNFe").
/// `parent_tag` is the element that receives the `<Signature>` child (e.g. "NFe").
fn sign_xml_generic(
    xml: &str,
    private_key_pem: &str,
    certificate_pem: &str,
    signed_tag: &str,
    parent_tag: &str,
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

    // 5. SHA-1 digest of the canonical form
    let digest = {
        let mut hasher = Sha1::new();
        hasher.update(canonical.as_bytes());
        BASE64.encode(hasher.finalize())
    };

    // 6. Build the SignedInfo XML (without xmlns — it inherits from parent Signature)
    let signed_info = build_signed_info(&id, &digest);

    // 7. Canonicalize the SignedInfo for signing.
    //    In C14N inclusive, when SignedInfo is canonicalized as a subset of
    //    <Signature xmlns="...">, the in-scope namespace is included on
    //    SignedInfo even though it's inherited. We must sign this form so
    //    SEFAZ can verify against the same canonical representation.
    let canonical_signed_info = signed_info.replacen(
        "<SignedInfo>",
        "<SignedInfo xmlns=\"http://www.w3.org/2000/09/xmldsig#\">",
        1,
    );

    // 8. RSA-SHA1 sign the canonical SignedInfo
    let pkey = PKey::private_key_from_pem(private_key_pem.as_bytes())
        .map_err(|e| FiscalError::Certificate(format!("Failed to parse private key: {e}")))?;

    let mut signer = Signer::new(MessageDigest::sha1(), &pkey)
        .map_err(|e| FiscalError::Certificate(format!("Failed to create signer: {e}")))?;

    signer
        .update(canonical_signed_info.as_bytes())
        .map_err(|e| FiscalError::Certificate(format!("Failed to update signer: {e}")))?;

    let signature_bytes = signer
        .sign_to_vec()
        .map_err(|e| FiscalError::Certificate(format!("RSA-SHA1 signing failed: {e}")))?;

    let signature_value = BASE64.encode(&signature_bytes);

    // 9. Extract certificate Base64 (strip PEM headers)
    let cert_base64 = extract_cert_base64(certificate_pem);

    // 10. Build the full <Signature> element
    let signature_xml = build_signature_element(&signed_info, &signature_value, &cert_base64);

    // 11. Insert the <Signature> inside the parent element, after the signed element
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
fn build_signed_info(reference_id: &str, digest_value: &str) -> String {
    let mut s = String::with_capacity(1024);
    s.push_str("<SignedInfo>");
    s.push_str("<CanonicalizationMethod Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"></CanonicalizationMethod>");
    s.push_str("<SignatureMethod Algorithm=\"http://www.w3.org/2000/09/xmldsig#rsa-sha1\"></SignatureMethod>");
    s.push_str("<Reference URI=\"#");
    s.push_str(reference_id);
    s.push_str("\">");
    s.push_str("<Transforms>");
    s.push_str("<Transform Algorithm=\"http://www.w3.org/2000/09/xmldsig#enveloped-signature\"></Transform>");
    s.push_str(
        "<Transform Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"></Transform>",
    );
    s.push_str("</Transforms>");
    s.push_str(
        "<DigestMethod Algorithm=\"http://www.w3.org/2000/09/xmldsig#sha1\"></DigestMethod>",
    );
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
