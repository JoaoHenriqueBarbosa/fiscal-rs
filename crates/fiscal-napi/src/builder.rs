use napi_derive::napi;

/// Build an NF-e/NFC-e XML from a configuration object.
///
/// Accepts the full invoice data as a single JSON object (matching
/// `InvoiceBuildData` fields in camelCase) and returns
/// `{ xml: string, accessKey: string }`.
#[napi(ts_return_type = "{ xml: string; accessKey: string }")]
pub fn build_invoice(config: serde_json::Value) -> napi::Result<serde_json::Value> {
    let data: fiscal_core::types::InvoiceBuildData = serde_json::from_value(config)
        .map_err(|e| napi::Error::from_reason(format!("Invalid config: {e}")))?;

    let result = fiscal_core::xml_builder::build_from_data(&data)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Build and sign an NF-e/NFC-e XML in one step.
///
/// Same as `buildInvoice` but also signs the XML using the provided
/// PEM-encoded private key and certificate.
#[napi(ts_return_type = "{ xml: string; signedXml: string; accessKey: string }")]
pub fn build_and_sign_invoice(
    config: serde_json::Value,
    private_key: String,
    certificate: String,
) -> napi::Result<serde_json::Value> {
    let data: fiscal_core::types::InvoiceBuildData = serde_json::from_value(config)
        .map_err(|e| napi::Error::from_reason(format!("Invalid config: {e}")))?;

    let result = fiscal_core::xml_builder::build_from_data(&data)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    let signed_xml = fiscal_crypto::certificate::sign_xml(&result.xml, &private_key, &certificate)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    Ok(serde_json::json!({
        "xml": result.xml,
        "signedXml": signed_xml,
        "accessKey": result.access_key,
    }))
}
