use napi::bindgen_prelude::Buffer;
use napi_derive::napi;

fn to_napi(e: impl std::fmt::Display) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}

fn to_json(v: &impl serde::Serialize) -> napi::Result<serde_json::Value> {
    serde_json::to_value(v).map_err(to_napi)
}

/// Load a PKCS#12 (PFX) certificate and return the PEM-encoded
/// private key and certificate.
#[napi(
    ts_return_type = "{ privateKey: string; certificate: string; pfxBuffer: number[]; passphrase: string }"
)]
pub fn load_certificate(pfx_buffer: Buffer, passphrase: String) -> napi::Result<serde_json::Value> {
    let cert_data =
        fiscal_crypto::certificate::load_certificate(&pfx_buffer, &passphrase).map_err(to_napi)?;
    to_json(&cert_data)
}

/// Get certificate metadata (common name, validity dates, serial, issuer).
#[napi(
    ts_return_type = "{ commonName: string; validFrom: string; validUntil: string; serialNumber: string; issuer: string }"
)]
pub fn get_certificate_info(
    pfx_buffer: Buffer,
    passphrase: String,
) -> napi::Result<serde_json::Value> {
    let info = fiscal_crypto::certificate::get_certificate_info(&pfx_buffer, &passphrase)
        .map_err(to_napi)?;
    to_json(&info)
}

/// Sign an NF-e XML using RSA-SHA1 (default algorithm).
#[napi]
pub fn sign_xml(xml: String, private_key: String, certificate: String) -> napi::Result<String> {
    fiscal_crypto::certificate::sign_xml(&xml, &private_key, &certificate).map_err(to_napi)
}

/// Sign an NF-e XML using a specific algorithm ("sha1" or "sha256").
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

/// Sign an event XML using RSA-SHA1 (default algorithm).
#[napi]
pub fn sign_event_xml(
    xml: String,
    private_key: String,
    certificate: String,
) -> napi::Result<String> {
    fiscal_crypto::certificate::sign_event_xml(&xml, &private_key, &certificate).map_err(to_napi)
}

/// Sign an event XML using a specific algorithm ("sha1" or "sha256").
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

/// Sign an inutilização XML using RSA-SHA1 (default algorithm).
#[napi]
pub fn sign_inutilizacao_xml(
    xml: String,
    private_key: String,
    certificate: String,
) -> napi::Result<String> {
    fiscal_crypto::certificate::sign_inutilizacao_xml(&xml, &private_key, &certificate)
        .map_err(to_napi)
}

/// Sign an inutilização XML using a specific algorithm ("sha1" or "sha256").
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
