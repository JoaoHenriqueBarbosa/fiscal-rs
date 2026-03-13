use fiscal_core::constants::NFE_NAMESPACE;
use fiscal_core::types::SefazEnvironment;

use super::event_core::{build_event_id, event_description};

/// Generate a CNPJ XML tag from a tax ID string.
///
/// If the tax ID is 11 digits it is treated as CPF; otherwise CNPJ.
pub(super) fn tax_id_xml_tag(tax_id: &str) -> String {
    let digits: String = tax_id.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 11 {
        format!("<CPF>{digits}</CPF>")
    } else {
        format!("<CNPJ>{digits}</CNPJ>")
    }
}

/// Extract a named section from XML (e.g., `<emit>...</emit>`).
///
/// Returns the full content between the opening and closing tags, inclusive.
pub(super) fn extract_section(xml: &str, tag_name: &str) -> Option<String> {
    let open = format!("<{tag_name}");
    let close = format!("</{tag_name}>");

    let start = xml.find(&open)?;
    // Verify delimiter
    let after_open = start + open.len();
    if after_open < xml.len() {
        let next = xml.as_bytes()[after_open];
        if next != b' ' && next != b'>' && next != b'/' && next != b'\n' && next != b'\t' {
            return None;
        }
    }

    let end = xml[start..].find(&close)? + start + close.len();
    Some(xml[start..end].to_string())
}

/// This is the core event builder used by cancellation, CCe, manifestation,
/// and other event-type request builders.
///
/// When `org_code_override` is `Some`, the provided value is used as `cOrgao`
/// instead of deriving it from the access key. This is needed for manifestation
/// events which must use code 91 (Ambiente Nacional).
pub(super) fn build_event_xml(
    access_key: &str,
    event_type: u32,
    seq: u32,
    tax_id: &str,
    environment: SefazEnvironment,
    additional_tags: &str,
) -> String {
    build_event_xml_with_org(
        access_key,
        event_type,
        seq,
        tax_id,
        environment,
        additional_tags,
        None,
    )
}

/// Build a generic SEFAZ event XML with an optional `cOrgao` override.
pub(super) fn build_event_xml_with_org(
    access_key: &str,
    event_type: u32,
    seq: u32,
    tax_id: &str,
    environment: SefazEnvironment,
    additional_tags: &str,
    org_code_override: Option<&str>,
) -> String {
    let event_id = build_event_id(event_type, access_key, seq);
    let desc_evento = event_description(event_type);
    let tax_id_tag = tax_id_xml_tag(tax_id);
    let tp_amb = environment.as_str();
    // cOrgao: use override when provided (e.g. "91" for manifestacao),
    // otherwise derive from the first 2 digits of the access key.
    let org_code_owned;
    let org_code = match org_code_override {
        Some(code) => code,
        None => {
            org_code_owned = access_key[..2].to_string();
            &org_code_owned
        }
    };
    // Use current timestamp as lot ID (milliseconds)
    let lot_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().to_string())
        .unwrap_or_else(|_| "1".to_string());
    // Event datetime with BRT offset (-03:00)
    let dh_evento = chrono::Utc::now()
        .with_timezone(&chrono::FixedOffset::west_opt(3 * 3600).unwrap())
        .format("%Y-%m-%dT%H:%M:%S%:z")
        .to_string();

    format!(
        "<envEvento xmlns=\"{NFE_NAMESPACE}\" versao=\"1.00\"><idLote>{lot_id}</idLote><evento xmlns=\"{NFE_NAMESPACE}\" versao=\"1.00\"><infEvento Id=\"{event_id}\"><cOrgao>{org_code}</cOrgao><tpAmb>{tp_amb}</tpAmb>{tax_id_tag}<chNFe>{access_key}</chNFe><dhEvento>{dh_evento}</dhEvento><tpEvento>{event_type}</tpEvento><nSeqEvento>{seq}</nSeqEvento><verEvento>1.00</verEvento><detEvento versao=\"1.00\"><descEvento>{desc_evento}</descEvento>{additional_tags}</detEvento></infEvento></evento></envEvento>"
    )
}

/// Validate that an access key is exactly 44 numeric digits.
pub(super) fn validate_access_key(access_key: &str) {
    assert!(!access_key.is_empty(), "Access key is required");
    assert!(
        access_key.len() == 44,
        "Invalid access key: must be exactly 44 digits, got {}",
        access_key.len()
    );
    assert!(
        access_key.chars().all(|c| c.is_ascii_digit()),
        "Invalid access key: must contain only digits"
    );
}

/// Strip XML declaration (`<?xml ... ?>`) from a string.
pub(super) fn strip_xml_declaration(xml: &str) -> &str {
    if let Some(start) = xml.find("<?xml") {
        if let Some(end) = xml[start..].find("?>") {
            let after = &xml[start + end + 2..];
            return after.trim_start();
        }
    }
    xml
}
