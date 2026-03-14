use napi::bindgen_prelude::Buffer;
use napi_derive::napi;

fn to_napi(e: impl std::fmt::Display) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}

fn to_json(v: &impl serde::Serialize) -> napi::Result<serde_json::Value> {
    serde_json::to_value(v).map_err(to_napi)
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
#[napi(ts_return_type = "Record<string, unknown>")]
pub fn load_certificate(pfx_buffer: Buffer, passphrase: String) -> napi::Result<serde_json::Value> {
    let result =
        fiscal_crypto::certificate::load_certificate(&pfx_buffer, &passphrase).map_err(to_napi)?;
    to_json(&result)
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
#[napi(ts_return_type = "Record<string, unknown>")]
pub fn get_certificate_info(
    pfx_buffer: Buffer,
    passphrase: String,
) -> napi::Result<serde_json::Value> {
    let result = fiscal_crypto::certificate::get_certificate_info(&pfx_buffer, &passphrase)
        .map_err(to_napi)?;
    to_json(&result)
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
#[napi]
pub fn sign_xml(xml: String, private_key: String, certificate: String) -> napi::Result<String> {
    fiscal_crypto::certificate::sign_xml(&xml, &private_key, &certificate).map_err(to_napi)
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
#[napi]
pub fn sign_xml_with_algorithm(
    xml: String,
    private_key: String,
    certificate: String,
    algorithm: String,
) -> napi::Result<String> {
    let algo = parse_algorithm(&algorithm)?;
    fiscal_crypto::certificate::sign_xml_with_algorithm(&xml, &private_key, &certificate, algo)
        .map_err(to_napi)
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
#[napi]
pub fn sign_event_xml(
    xml: String,
    private_key: String,
    certificate: String,
) -> napi::Result<String> {
    fiscal_crypto::certificate::sign_event_xml(&xml, &private_key, &certificate).map_err(to_napi)
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
#[napi]
pub fn sign_event_xml_with_algorithm(
    xml: String,
    private_key: String,
    certificate: String,
    algorithm: String,
) -> napi::Result<String> {
    let algo = parse_algorithm(&algorithm)?;
    fiscal_crypto::certificate::sign_event_xml_with_algorithm(
        &xml,
        &private_key,
        &certificate,
        algo,
    )
    .map_err(to_napi)
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
#[napi]
pub fn sign_inutilizacao_xml(
    xml: String,
    private_key: String,
    certificate: String,
) -> napi::Result<String> {
    fiscal_crypto::certificate::sign_inutilizacao_xml(&xml, &private_key, &certificate)
        .map_err(to_napi)
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
#[napi]
pub fn sign_inutilizacao_xml_with_algorithm(
    xml: String,
    private_key: String,
    certificate: String,
    algorithm: String,
) -> napi::Result<String> {
    let algo = parse_algorithm(&algorithm)?;
    fiscal_crypto::certificate::sign_inutilizacao_xml_with_algorithm(
        &xml,
        &private_key,
        &certificate,
        algo,
    )
    .map_err(to_napi)
}

fn parse_algorithm(s: &str) -> napi::Result<fiscal_crypto::SignatureAlgorithm> {
    match s.to_lowercase().as_str() {
        "sha1" => Ok(fiscal_crypto::SignatureAlgorithm::Sha1),
        "sha256" => Ok(fiscal_crypto::SignatureAlgorithm::Sha256),
        _ => Err(napi::Error::from_reason(format!(
            "Invalid algorithm: \"{s}\". Expected \"sha1\" or \"sha256\"."
        ))),
    }
}
