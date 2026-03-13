//! PFX/PKCS#12 certificate loading, parsing, and info extraction.

use std::sync::Once;

use openssl::pkcs12::Pkcs12;

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
