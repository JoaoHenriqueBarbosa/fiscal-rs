//! PFX/PKCS#12 certificate loading, parsing, and info extraction.
//!
//! Pure-Rust implementation — no OpenSSL dependency.

use base64::Engine as _;
use fiscal_core::FiscalError;
use fiscal_core::types::{CertificateData, CertificateInfo};
use x509_cert::der::Decode as _;

use super::pkcs12_parser;

/// Hash algorithm used for the XML-DSig digest and RSA signature.
///
/// **For NF-e / NFC-e use [`SignatureAlgorithm::Sha1`] (the default).** The
/// `xmldsig-core` schema in the NF-e layout fixes the signature/digest
/// `Algorithm` to `rsa-sha1`/`sha1`; SHA-256 is rejected by SEFAZ with cStat
/// 225 — independent of the certificate version (an ICP-Brasil v5, SHA-256
/// *certificate*, still signs the NF-e XML with SHA-1). Reach for
/// [`SignatureAlgorithm::Sha256`] only where a service/document type
/// explicitly documents requiring it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SignatureAlgorithm {
    /// RSA-SHA1 — required by the NF-e/NFC-e XML-DSig schema; the default.
    #[default]
    Sha1,
    /// RSA-SHA256 — only for services/document types that explicitly require
    /// it. NOT valid for NF-e/NFC-e (schema fixes SHA-1 → cStat 225).
    Sha256,
}

/// Validate a PFX buffer and return the original bytes unchanged.
///
/// Since `fiscal-rs` uses a pure-Rust PKCS#12 parser that handles all
/// encryption schemes (legacy PBES1/RC2-40-CBC, PBES1/3DES-CBC, and modern
/// PBES2/AES-CBC) transparently, no re-encryption or "modernization" is
/// needed. This function validates that the PFX is parseable and the
/// passphrase is correct, and returns the original bytes as-is.
///
/// This function is kept for backward compatibility; new code can use
/// [`load_certificate`] directly, which performs the same validation.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if the PFX is invalid or the
/// passphrase is wrong.
pub fn ensure_modern_pfx(pfx_buffer: &[u8], passphrase: &str) -> Result<Vec<u8>, FiscalError> {
    let _parsed = pkcs12_parser::pkcs12_parse(pfx_buffer, passphrase)?;
    Ok(pfx_buffer.to_vec())
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
    let parsed = pkcs12_parser::pkcs12_parse(pfx_buffer, passphrase)?;

    // Convert private key DER to PEM using pkcs8 crate
    let private_key_pem = pkcs8_der_to_pem(&parsed.pkey)?;

    // Convert certificate DER to PEM using x509-cert
    let certificate_pem = x509_der_to_pem(&parsed.cert)?;

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
    let parsed = pkcs12_parser::pkcs12_parse(pfx_buffer, passphrase)?;

    // Parse X.509 certificate from DER
    let cert = x509_cert::Certificate::from_der(&parsed.cert)
        .map_err(|e| FiscalError::Certificate(format!("Failed to parse certificate: {e}")))?;

    let common_name = extract_cn_from_name(&cert.tbs_certificate.subject);
    let issuer = extract_cn_from_name(&cert.tbs_certificate.issuer);

    let valid_from = x509_time_to_naive_date(&cert.tbs_certificate.validity.not_before)?;
    let valid_until = x509_time_to_naive_date(&cert.tbs_certificate.validity.not_after)?;

    // Serial number — format as hex string
    let serial_number = cert
        .tbs_certificate
        .serial_number
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<String>();

    Ok(CertificateInfo::new(
        common_name,
        valid_from,
        valid_until,
        serial_number,
        issuer,
    ))
}

// ── Private helpers ─────────────────────────────────────────────────────────

/// Convert PKCS#8 DER bytes to PEM string.
fn pkcs8_der_to_pem(der: &[u8]) -> Result<String, FiscalError> {
    // Manual PEM encoding for PKCS#8 private key
    let b64 = base64::engine::general_purpose::STANDARD.encode(der);
    Ok(format!(
        "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----\n",
        wrap_base64(&b64)
    ))
}

/// Convert X.509 DER bytes to PEM string.
fn x509_der_to_pem(der: &[u8]) -> Result<String, FiscalError> {
    let b64 = base64::engine::general_purpose::STANDARD.encode(der);
    Ok(format!(
        "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n",
        wrap_base64(&b64)
    ))
}

/// Wrap base64 content at 64 characters per line.
fn wrap_base64(s: &str) -> String {
    s.as_bytes()
        .chunks(64)
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Extract the Common Name (CN) from an X.500 Name (RDN sequence).
fn extract_cn_from_name(name: &x509_cert::name::Name) -> String {
    // In x509-cert 0.2, Name wraps a Vec<RelativeDistinguishedName>
    for rdn in &name.0 {
        for attr in rdn.0.as_slice() {
            if attr.oid.to_string() == "2.5.4.3" {
                // Try to extract the string value from the attribute value bytes
                if let Ok(value) = std::str::from_utf8(attr.value.value()) {
                    return value.to_string();
                }
            }
        }
    }
    // Fallback: use the Debug representation
    format!("{name:?}")
}

/// Convert an x509-cert Time to a chrono NaiveDate.
fn x509_time_to_naive_date(time: &x509_cert::time::Time) -> Result<chrono::NaiveDate, FiscalError> {
    // Convert to SystemTime -> Unix timestamp -> NaiveDate
    let system_time = match time {
        x509_cert::time::Time::UtcTime(utc) => utc.to_system_time(),
        x509_cert::time::Time::GeneralTime(gt) => gt.to_system_time(),
    };

    // Use chrono to convert
    let dt: chrono::DateTime<chrono::Utc> = system_time.into();
    Ok(dt.date_naive())
}
