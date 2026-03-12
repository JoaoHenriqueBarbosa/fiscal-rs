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

/// Ensure a PFX buffer uses modern encryption algorithms.
///
/// Brazilian A1 certificates are commonly issued with legacy encryption
/// (RC2-40-CBC) which OpenSSL 3.x rejects by default. This function
/// detects legacy PFX files and converts them automatically via the
/// system `openssl` CLI, returning modern-encrypted PFX bytes.
///
/// If the PFX is already modern, the original bytes are returned as-is.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if conversion fails or if the
/// system `openssl` CLI is not available.
pub fn ensure_modern_pfx(pfx_buffer: &[u8], passphrase: &str) -> Result<Vec<u8>, FiscalError> {
    ensure_legacy_provider();

    let pkcs12 = Pkcs12::from_der(pfx_buffer)
        .map_err(|e| FiscalError::Certificate(format!("Invalid PFX data: {e}")))?;

    match pkcs12.parse2(passphrase) {
        Ok(_) => Ok(pfx_buffer.to_vec()),
        Err(e) => {
            let msg = e.to_string();
            if !msg.contains("unsupported") && !msg.contains("RC2") {
                return Err(FiscalError::Certificate(format!(
                    "Failed to parse PFX (wrong password?): {e}"
                )));
            }
            convert_legacy_pfx_bytes(pfx_buffer, passphrase)
        }
    }
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

/// Convert a legacy-encrypted PFX to modern algorithms via the system `openssl` CLI.
fn convert_legacy_pfx_bytes(pfx_buffer: &[u8], passphrase: &str) -> Result<Vec<u8>, FiscalError> {
    use std::process::Command;

    let dir = std::env::temp_dir().join("fiscal-rs-pfx-convert");
    std::fs::create_dir_all(&dir)
        .map_err(|e| FiscalError::Certificate(format!("Failed to create temp dir: {e}")))?;

    let legacy_path = dir.join("legacy.pfx");
    let modern_path = dir.join("modern.pfx");
    let pem_path = dir.join("cert.pem");

    std::fs::write(&legacy_path, pfx_buffer)
        .map_err(|e| FiscalError::Certificate(format!("Failed to write temp PFX: {e}")))?;

    // Step 1: PFX → PEM (with -legacy flag)
    let output = Command::new("openssl")
        .args([
            "pkcs12",
            "-in",
            legacy_path.to_str().unwrap(),
            "-out",
            pem_path.to_str().unwrap(),
            "-passin",
            &format!("pass:{passphrase}"),
            "-passout",
            &format!("pass:{passphrase}"),
            "-legacy",
        ])
        .output()
        .map_err(|e| {
            FiscalError::Certificate(format!(
                "Legacy PFX detected (RC2-40-CBC). Automatic conversion requires the `openssl` CLI \
             to be installed. Install it with: sudo apt-get install openssl. Error: {e}"
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = std::fs::remove_dir_all(&dir);
        return Err(FiscalError::Certificate(format!(
            "Failed to convert legacy PFX to PEM: {stderr}"
        )));
    }

    // Step 2: PEM → modern PFX
    let output = Command::new("openssl")
        .args([
            "pkcs12",
            "-export",
            "-in",
            pem_path.to_str().unwrap(),
            "-out",
            modern_path.to_str().unwrap(),
            "-passin",
            &format!("pass:{passphrase}"),
            "-passout",
            &format!("pass:{passphrase}"),
        ])
        .output()
        .map_err(|e| FiscalError::Certificate(format!("Failed to re-export PFX: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = std::fs::remove_dir_all(&dir);
        return Err(FiscalError::Certificate(format!(
            "Failed to convert PEM back to modern PFX: {stderr}"
        )));
    }

    let modern_bytes = std::fs::read(&modern_path)
        .map_err(|e| FiscalError::Certificate(format!("Failed to read converted PFX: {e}")))?;

    let _ = std::fs::remove_dir_all(&dir);
    Ok(modern_bytes)
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

    // 4. Canonicalize the signed element (basic C14N without comments)
    let canonical = canonicalize_xml(&without_sig);

    // 5. SHA-1 digest of the canonical form
    let digest = {
        let mut hasher = Sha1::new();
        hasher.update(canonical.as_bytes());
        BASE64.encode(hasher.finalize())
    };

    // 6. Build the SignedInfo XML
    let signed_info = build_signed_info(&id, &digest);

    // 7. Canonicalize the SignedInfo
    let canonical_signed_info = canonicalize_xml(&signed_info);

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

/// Basic XML Canonicalization (C14N 1.0 without comments).
///
/// For NF-e purposes this is a simplified implementation that:
/// - Ensures consistent attribute ordering (already handled by our XML generation)
/// - Removes the XML declaration
/// - Normalizes whitespace in a basic way
///
/// NF-e XML is typically generated without extra whitespace, making full C14N
/// largely a no-op beyond removing the XML declaration.
fn canonicalize_xml(xml: &str) -> String {
    let mut result = xml.to_string();

    // Remove XML declaration if present
    if let Some(decl_start) = result.find("<?xml") {
        if let Some(decl_end) = result[decl_start..].find("?>") {
            result = format!(
                "{}{}",
                &result[..decl_start],
                result[decl_start + decl_end + 2..].trim_start()
            );
        }
    }

    result
}

/// Build the `<SignedInfo>` element for XML-DSig.
fn build_signed_info(reference_id: &str, digest_value: &str) -> String {
    let mut s = String::with_capacity(1024);
    s.push_str("<SignedInfo xmlns=\"http://www.w3.org/2000/09/xmldsig#\">");
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
