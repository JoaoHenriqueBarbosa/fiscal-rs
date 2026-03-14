use napi_derive::napi;

// ── Standardize ─────────────────────────────────────────────────────────────

/// Identify the type of an NF-e XML document from its content.
///
/// Returns the matched root tag name (e.g. "NFe", "nfeProc", "retConsSitNFe").
#[napi]
pub fn identify_xml_type(xml: String) -> napi::Result<String> {
    fiscal_core::standardize::identify_xml_type(&xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Convert NF-e XML to a JSON string.
#[napi]
pub fn xml_to_json(xml: String) -> napi::Result<String> {
    fiscal_core::standardize::xml_to_json(&xml).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Convert NF-e XML to a JavaScript value (parsed JSON object).
#[napi(ts_return_type = "Record<string, unknown>")]
pub fn xml_to_value(xml: String) -> napi::Result<serde_json::Value> {
    fiscal_core::standardize::xml_to_value(&xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

// ── Convert ─────────────────────────────────────────────────────────────────

/// Convert a SPED TXT file to NF-e XML.
#[napi]
pub fn txt_to_xml(txt: String, layout: String) -> napi::Result<String> {
    fiscal_core::convert::txt_to_xml(&txt, &layout)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Convert a SPED TXT file to multiple NF-e XMLs (one per document).
#[napi]
pub fn txt_to_xml_all(txt: String, layout: String) -> napi::Result<Vec<String>> {
    fiscal_core::convert::txt_to_xml_all(&txt, &layout)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Validate the structure of a SPED TXT file.
#[napi]
pub fn validate_txt(txt: String, layout: String) -> napi::Result<bool> {
    fiscal_core::convert::validate_txt(&txt, &layout)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

// ── Complement ──────────────────────────────────────────────────────────────

/// Attach SEFAZ authorization protocol to a signed NF-e XML.
///
/// Combines the signed request XML with the SEFAZ response to produce
/// the authorized `nfeProc` document.
#[napi]
pub fn to_authorize(request_xml: String, response_xml: String) -> napi::Result<String> {
    fiscal_core::complement::to_authorize(&request_xml, &response_xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Attach protocol to a signed NF-e XML (procNFe).
#[napi]
pub fn attach_protocol(request_xml: String, response_xml: String) -> napi::Result<String> {
    fiscal_core::complement::attach_protocol(&request_xml, &response_xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Attach event protocol to a signed event XML (procEventoNFe).
#[napi]
pub fn attach_event_protocol(request_xml: String, response_xml: String) -> napi::Result<String> {
    fiscal_core::complement::attach_event_protocol(&request_xml, &response_xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Attach inutilização protocol (procInutNFe).
#[napi]
pub fn attach_inutilizacao(request_xml: String, response_xml: String) -> napi::Result<String> {
    fiscal_core::complement::attach_inutilizacao(&request_xml, &response_xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Attach cancellation event to an authorized NF-e (nfeProc + procEventoNFe).
#[napi]
pub fn attach_cancellation(nfe_proc_xml: String, cancel_event_xml: String) -> napi::Result<String> {
    fiscal_core::complement::attach_cancellation(&nfe_proc_xml, &cancel_event_xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}
