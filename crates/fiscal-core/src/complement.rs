use crate::FiscalError;
use crate::constants::NFE_NAMESPACE;
use crate::status_codes::{VALID_EVENT_STATUSES, VALID_PROTOCOL_STATUSES, sefaz_status};
use crate::xml_utils::extract_xml_tag_value;

/// NF-e version used in wrapper elements when no version is found.
const DEFAULT_VERSION: &str = "4.00";

// ── Public API ──────────────────────────────────────────────────────────────

/// Attach the SEFAZ authorization protocol to a signed NFe XML,
/// producing the `<nfeProc>` wrapper required for storage and DANFE.
///
/// The function extracts the `<NFe>` from `request_xml` and the matching
/// `<protNFe>` from `response_xml`, validates the protocol status, and
/// joins them into a single `<nfeProc>` document.
///
/// If the response contains multiple `<protNFe>` nodes (batch response),
/// the function attempts to match by digest value and access key. When no
/// exact match is found it falls back to the first available `<protNFe>`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if:
/// - Either input is empty
/// - The `<NFe>` tag is missing from `request_xml`
/// - No `<protNFe>` can be found in `response_xml`
///
/// Returns [`FiscalError::SefazRejection`] if the protocol status code
/// is not in [`VALID_PROTOCOL_STATUSES`].
pub fn attach_protocol(request_xml: &str, response_xml: &str) -> Result<String, FiscalError> {
    if request_xml.is_empty() {
        return Err(FiscalError::XmlParsing("Request XML (NFe) is empty".into()));
    }
    if response_xml.is_empty() {
        return Err(FiscalError::XmlParsing(
            "Response XML (protocol) is empty".into(),
        ));
    }

    let nfe_content = extract_tag(request_xml, "NFe")
        .ok_or_else(|| FiscalError::XmlParsing("Could not find <NFe> tag in request XML".into()))?;

    // Extract digest and access key from the NFe for matching
    let digest_nfe = extract_xml_tag_value(request_xml, "DigestValue");
    let access_key = extract_inf_nfe_id(request_xml);

    // Try to find a matching protNFe by digest + access key
    let mut matched_prot: Option<String> = None;

    let prot_nodes = extract_all_tags(response_xml, "protNFe");

    for prot in &prot_nodes {
        let dig_val = extract_xml_tag_value(prot, "digVal");
        let ch_nfe = extract_xml_tag_value(prot, "chNFe");

        if let (Some(dn), Some(dv)) = (&digest_nfe, &dig_val) {
            if let (Some(ak), Some(cn)) = (&access_key, &ch_nfe) {
                if dn == dv && ak == cn {
                    // Exact match — validate status
                    let c_stat = extract_xml_tag_value(prot, "cStat").unwrap_or_default();
                    if !VALID_PROTOCOL_STATUSES.contains(&c_stat.as_str()) {
                        let x_motivo = extract_xml_tag_value(prot, "xMotivo").unwrap_or_default();
                        return Err(FiscalError::SefazRejection {
                            code: c_stat,
                            message: x_motivo,
                        });
                    }
                    matched_prot = Some(prot.clone());
                    break;
                }
            }
        }
    }

    if matched_prot.is_none() {
        // Fallback: use first available protNFe
        let single_prot = extract_tag(response_xml, "protNFe").ok_or_else(|| {
            FiscalError::XmlParsing("Could not find <protNFe> in response XML".into())
        })?;

        // Validate status on the fallback protNFe
        let c_stat = extract_xml_tag_value(&single_prot, "cStat").unwrap_or_default();
        if !VALID_PROTOCOL_STATUSES.contains(&c_stat.as_str()) {
            let x_motivo = extract_xml_tag_value(&single_prot, "xMotivo").unwrap_or_default();
            return Err(FiscalError::SefazRejection {
                code: c_stat,
                message: x_motivo,
            });
        }
        matched_prot = Some(single_prot);
    }

    let version = extract_attribute(&nfe_content, "infNFe", "versao")
        .unwrap_or_else(|| DEFAULT_VERSION.to_string());

    Ok(join_xml(
        &nfe_content,
        &matched_prot.unwrap(),
        "nfeProc",
        &version,
    ))
}

/// Attach the SEFAZ inutilizacao response to the request,
/// producing the `<ProcInutNFe>` wrapper.
///
/// Extracts `<inutNFe>` from `request_xml` and `<retInutNFe>` from
/// `response_xml`, validates that the response status is `102` (voided),
/// and joins them into a `<ProcInutNFe>` document.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if:
/// - Either input is empty
/// - The `<inutNFe>` tag is missing from `request_xml`
/// - The `<retInutNFe>` tag is missing from `response_xml`
///
/// Returns [`FiscalError::SefazRejection`] if the response status is not `102`.
pub fn attach_inutilizacao(request_xml: &str, response_xml: &str) -> Result<String, FiscalError> {
    if request_xml.is_empty() {
        return Err(FiscalError::XmlParsing(
            "Inutilizacao request XML is empty".into(),
        ));
    }
    if response_xml.is_empty() {
        return Err(FiscalError::XmlParsing(
            "Inutilizacao response XML is empty".into(),
        ));
    }

    let inut_content = extract_tag(request_xml, "inutNFe").ok_or_else(|| {
        FiscalError::XmlParsing("Could not find <inutNFe> tag in request XML".into())
    })?;

    let ret_inut_content = extract_tag(response_xml, "retInutNFe").ok_or_else(|| {
        FiscalError::XmlParsing("Could not find <retInutNFe> tag in response XML".into())
    })?;

    // Validate the response status — must be 102 (voided)
    let c_stat = extract_xml_tag_value(&ret_inut_content, "cStat").unwrap_or_default();
    if c_stat != sefaz_status::VOIDED {
        let x_motivo = extract_xml_tag_value(&ret_inut_content, "xMotivo").unwrap_or_default();
        return Err(FiscalError::SefazRejection {
            code: c_stat,
            message: x_motivo,
        });
    }

    // Get version from the inutNFe request tag
    let version = extract_attribute(&inut_content, "inutNFe", "versao")
        .unwrap_or_else(|| DEFAULT_VERSION.to_string());

    Ok(join_xml(
        &inut_content,
        &ret_inut_content,
        "ProcInutNFe",
        &version,
    ))
}

/// Attach an event protocol response to the event request,
/// producing the `<procEventoNFe>` wrapper.
///
/// Extracts `<evento>` from `request_xml` and `<retEvento>` from
/// `response_xml`, validates the event status, and joins them
/// into a `<procEventoNFe>` document.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if:
/// - Either input is empty
/// - The `<evento>` tag is missing from `request_xml`
/// - The `<retEvento>` tag is missing from `response_xml`
///
/// Returns [`FiscalError::SefazRejection`] if the event status code
/// is not in [`VALID_EVENT_STATUSES`].
pub fn attach_event_protocol(request_xml: &str, response_xml: &str) -> Result<String, FiscalError> {
    if request_xml.is_empty() {
        return Err(FiscalError::XmlParsing("Event request XML is empty".into()));
    }
    if response_xml.is_empty() {
        return Err(FiscalError::XmlParsing(
            "Event response XML is empty".into(),
        ));
    }

    let evento_content = extract_tag(request_xml, "evento").ok_or_else(|| {
        FiscalError::XmlParsing("Could not find <evento> tag in request XML".into())
    })?;

    let ret_evento_content = extract_tag(response_xml, "retEvento").ok_or_else(|| {
        FiscalError::XmlParsing("Could not find <retEvento> tag in response XML".into())
    })?;

    // Get version from the evento tag
    let version = extract_attribute(&evento_content, "evento", "versao")
        .unwrap_or_else(|| DEFAULT_VERSION.to_string());

    // Validate event status
    let c_stat = extract_xml_tag_value(&ret_evento_content, "cStat").unwrap_or_default();
    if !VALID_EVENT_STATUSES.contains(&c_stat.as_str()) {
        let x_motivo = extract_xml_tag_value(&ret_evento_content, "xMotivo").unwrap_or_default();
        return Err(FiscalError::SefazRejection {
            code: c_stat,
            message: x_motivo,
        });
    }

    Ok(join_xml(
        &evento_content,
        &ret_evento_content,
        "procEventoNFe",
        &version,
    ))
}

/// Attach a B2B financial tag to an authorized `<nfeProc>` XML,
/// wrapping both in a `<nfeProcB2B>` element.
///
/// # Arguments
///
/// * `nfe_proc_xml` - The authorized nfeProc XML.
/// * `b2b_xml` - The B2B financial XML (must contain the `tag_b2b` element).
/// * `tag_b2b` - Optional B2B tag name; defaults to `"NFeB2BFin"`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if:
/// - The `nfe_proc_xml` does not contain `<nfeProc>`
/// - The `b2b_xml` does not contain the expected B2B tag
/// - Either tag cannot be extracted
pub fn attach_b2b(
    nfe_proc_xml: &str,
    b2b_xml: &str,
    tag_b2b: Option<&str>,
) -> Result<String, FiscalError> {
    let tag_name = tag_b2b.unwrap_or("NFeB2BFin");

    if !nfe_proc_xml.contains("<nfeProc") {
        return Err(FiscalError::XmlParsing(
            "XML does not contain <nfeProc> — is this an authorized NFe?".into(),
        ));
    }

    let open_check = format!("<{tag_name}");
    if !b2b_xml.contains(&open_check) {
        return Err(FiscalError::XmlParsing(format!(
            "B2B XML does not contain <{tag_name}> tag"
        )));
    }

    let nfe_proc_content = extract_tag(nfe_proc_xml, "nfeProc")
        .ok_or_else(|| FiscalError::XmlParsing("Could not extract <nfeProc> from XML".into()))?;

    let b2b_content = extract_tag(b2b_xml, tag_name).ok_or_else(|| {
        FiscalError::XmlParsing(format!("Could not extract <{tag_name}> from B2B XML"))
    })?;

    Ok(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
         <nfeProcB2B>{nfe_proc_content}{b2b_content}</nfeProcB2B>"
    ))
}

// ── Internal helpers ────────────────────────────────────────────────────────

/// Join two XML fragments into a versioned namespace wrapper element.
///
/// Produces:
/// ```xml
/// <?xml version="1.0" encoding="UTF-8"?>
/// <{node_name} versao="{version}" xmlns="{NFE_NAMESPACE}">
///   {first}{second}
/// </{node_name}>
/// ```
fn join_xml(first: &str, second: &str, node_name: &str, version: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
         <{node_name} versao=\"{version}\" xmlns=\"{NFE_NAMESPACE}\">\
         {first}{second}</{node_name}>"
    )
}

/// Extract a complete XML tag (outermost match) including attributes and
/// all nested content. Uses `lastIndexOf`-style search for the closing tag
/// to handle nested tags of the same name.
///
/// Returns `None` if either the opening or closing tag is not found.
fn extract_tag(xml: &str, tag_name: &str) -> Option<String> {
    // Find the opening tag: <tagName followed by whitespace, >, or /
    let open_pattern = format!("<{tag_name}");
    let start = xml.find(&open_pattern)?;

    // Verify that the character after `<tagName` is a valid delimiter
    // (space, >, /) to avoid matching tags like `<tagNameExtra>`
    let after_open = start + open_pattern.len();
    if after_open < xml.len() {
        let next_char = xml.as_bytes()[after_open];
        if next_char != b' '
            && next_char != b'>'
            && next_char != b'/'
            && next_char != b'\n'
            && next_char != b'\r'
            && next_char != b'\t'
        {
            return None;
        }
    }

    let close_tag = format!("</{tag_name}>");
    let close_index = xml.rfind(&close_tag)?;

    Some(xml[start..close_index + close_tag.len()].to_string())
}

/// Extract all occurrences of a tag from XML. Finds each non-overlapping
/// `<tagName ...>...</tagName>` in the source string.
fn extract_all_tags(xml: &str, tag_name: &str) -> Vec<String> {
    let mut results = Vec::new();
    let open_pattern = format!("<{tag_name}");
    let close_tag = format!("</{tag_name}>");
    let mut search_from = 0;

    while search_from < xml.len() {
        let start = match xml[search_from..].find(&open_pattern) {
            Some(pos) => search_from + pos,
            None => break,
        };

        // Verify delimiter after tag name
        let after_open = start + open_pattern.len();
        if after_open < xml.len() {
            let next_char = xml.as_bytes()[after_open];
            if next_char != b' '
                && next_char != b'>'
                && next_char != b'/'
                && next_char != b'\n'
                && next_char != b'\r'
                && next_char != b'\t'
            {
                search_from = after_open;
                continue;
            }
        }

        let end = match xml[start..].find(&close_tag) {
            Some(pos) => start + pos + close_tag.len(),
            None => break,
        };

        results.push(xml[start..end].to_string());
        search_from = end;
    }

    results
}

/// Extract an XML attribute value from a tag. Searches for the tag opening
/// then finds `attr="value"` within it.
fn extract_attribute(xml: &str, tag_name: &str, attr_name: &str) -> Option<String> {
    let open = format!("<{tag_name}");
    let start = xml.find(&open)?;

    // Find the end of the opening tag
    let tag_end = xml[start..].find('>')? + start;
    let tag_header = &xml[start..tag_end];

    // Find attr="value" pattern
    let attr_pattern = format!("{attr_name}=\"");
    let attr_start = tag_header.find(&attr_pattern)? + attr_pattern.len();
    let attr_end = tag_header[attr_start..].find('"')? + attr_start;

    Some(tag_header[attr_start..attr_end].to_string())
}

/// Extract the access key from an `<infNFe Id="NFe...">` attribute.
/// Returns the 44-digit key (without the "NFe" prefix).
fn extract_inf_nfe_id(xml: &str) -> Option<String> {
    let attr_val = extract_attribute(xml, "infNFe", "Id")?;
    Some(
        attr_val
            .strip_prefix("NFe")
            .unwrap_or(&attr_val)
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_tag_finds_outermost_match() {
        let xml = r#"<root><NFe versao="4.00"><inner/></NFe></root>"#;
        let result = extract_tag(xml, "NFe").unwrap();
        assert!(result.starts_with("<NFe"));
        assert!(result.ends_with("</NFe>"));
        assert!(result.contains("<inner/>"));
    }

    #[test]
    fn extract_tag_returns_none_for_missing_tag() {
        let xml = "<root><other/></root>";
        assert!(extract_tag(xml, "NFe").is_none());
    }

    #[test]
    fn extract_tag_does_not_match_prefix() {
        let xml = "<root><NFeExtra>data</NFeExtra></root>";
        assert!(extract_tag(xml, "NFe").is_none());
    }

    #[test]
    fn extract_attribute_works() {
        let xml = r#"<infNFe versao="4.00" Id="NFe12345">"#;
        assert_eq!(
            extract_attribute(xml, "infNFe", "versao"),
            Some("4.00".to_string())
        );
        assert_eq!(
            extract_attribute(xml, "infNFe", "Id"),
            Some("NFe12345".to_string())
        );
    }

    #[test]
    fn extract_all_tags_finds_multiple() {
        let xml = r#"<root><item>1</item><item>2</item><item>3</item></root>"#;
        let items = extract_all_tags(xml, "item");
        assert_eq!(items.len(), 3);
        assert!(items[0].contains("1"));
        assert!(items[2].contains("3"));
    }

    #[test]
    fn join_xml_produces_correct_wrapper() {
        let result = join_xml("<A/>", "<B/>", "wrapper", "4.00");
        assert!(result.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(result.contains("<wrapper versao=\"4.00\""));
        assert!(result.contains(&format!("xmlns=\"{NFE_NAMESPACE}\"")));
        assert!(result.ends_with("</wrapper>"));
    }

    #[test]
    fn extract_inf_nfe_id_strips_prefix() {
        let xml = r#"<NFe><infNFe versao="4.00" Id="NFe35260112345678000199650010000000011123456780"></infNFe></NFe>"#;
        let key = extract_inf_nfe_id(xml).unwrap();
        assert_eq!(key, "35260112345678000199650010000000011123456780");
    }
}
