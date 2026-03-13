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

/// Cancellation event type code (`110111`).
const EVT_CANCELA: &str = "110111";
/// Cancellation by substitution event type code (`110112`).
const EVT_CANCELA_SUBSTITUICAO: &str = "110112";

/// Valid status codes for cancellation event matching.
///
/// - `135` — Event registered and linked
/// - `136` — Event registered but not linked
/// - `155` — Already cancelled (late)
const VALID_CANCEL_STATUSES: &[&str] = &["135", "136", "155"];

/// Attach a cancellation event response to an authorized `<nfeProc>` XML,
/// marking the NF-e as locally cancelled.
///
/// This mirrors the PHP `Complements::cancelRegister()` method. The function
/// searches the `cancel_event_xml` for `<retEvento>` elements whose:
/// - `cStat` is in `[135, 136, 155]` (valid cancellation statuses)
/// - `tpEvento` is `110111` (cancellation) or `110112` (cancellation by substitution)
/// - `chNFe` matches the access key in the authorized NF-e's `<protNFe>`
///
/// When a matching `<retEvento>` is found, it is appended inside the
/// `<nfeProc>` element (before the closing `</nfeProc>` tag).
///
/// If no matching cancellation event is found, the original NF-e XML is
/// returned unchanged (same behavior as the PHP implementation).
///
/// # Arguments
///
/// * `nfe_proc_xml` - The authorized NF-e XML containing `<nfeProc>` with `<protNFe>`.
/// * `cancel_event_xml` - The SEFAZ cancellation event response XML containing `<retEvento>`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if:
/// - The `nfe_proc_xml` does not contain `<protNFe>` (not an authorized NF-e)
/// - The `<protNFe>` does not contain `<chNFe>`
pub fn attach_cancellation(
    nfe_proc_xml: &str,
    cancel_event_xml: &str,
) -> Result<String, FiscalError> {
    // Validate the NF-e has a protNFe with a chNFe
    let prot_nfe = extract_tag(nfe_proc_xml, "protNFe").ok_or_else(|| {
        FiscalError::XmlParsing(
            "Could not find <protNFe> in NF-e XML — is this an authorized NF-e?".into(),
        )
    })?;

    let ch_nfe = extract_xml_tag_value(&prot_nfe, "chNFe")
        .ok_or_else(|| FiscalError::XmlParsing("Could not find <chNFe> inside <protNFe>".into()))?;

    // Search for matching retEvento in the cancellation XML
    let ret_eventos = extract_all_tags(cancel_event_xml, "retEvento");

    for ret_evento in &ret_eventos {
        let c_stat = match extract_xml_tag_value(ret_evento, "cStat") {
            Some(v) => v,
            None => continue,
        };
        let tp_evento = match extract_xml_tag_value(ret_evento, "tpEvento") {
            Some(v) => v,
            None => continue,
        };
        let ch_nfe_evento = match extract_xml_tag_value(ret_evento, "chNFe") {
            Some(v) => v,
            None => continue,
        };

        if VALID_CANCEL_STATUSES.contains(&c_stat.as_str())
            && (tp_evento == EVT_CANCELA || tp_evento == EVT_CANCELA_SUBSTITUICAO)
            && ch_nfe_evento == ch_nfe
        {
            // Insert the retEvento before </nfeProc>
            let close_tag = "</nfeProc>";
            if let Some(pos) = nfe_proc_xml.rfind(close_tag) {
                let mut result = String::with_capacity(nfe_proc_xml.len() + ret_evento.len());
                result.push_str(&nfe_proc_xml[..pos]);
                result.push_str(ret_evento);
                result.push_str(close_tag);
                return Ok(result);
            }
            // If no </nfeProc>, just append to the end (best effort)
            break;
        }
    }

    // No matching cancellation event found — return original XML unchanged
    Ok(nfe_proc_xml.to_string())
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

    // ── attach_cancellation tests ─────────────────────────────────────

    #[test]
    fn attach_cancellation_appends_matching_ret_evento() {
        let nfe_proc = concat!(
            r#"<?xml version="1.0" encoding="UTF-8"?>"#,
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe><infNFe versao="4.00" Id="NFe35260112345678000199550010000000011123456780">"#,
            r#"<ide/></infNFe></NFe>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat><nProt>135220000009921</nProt>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEnvEvento><retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat>"#,
            r#"<xMotivo>Evento registrado e vinculado a NF-e</xMotivo>"#,
            r#"<tpEvento>110111</tpEvento>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento></retEnvEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();

        // Must contain the retEvento inside nfeProc
        assert!(
            result.contains("<retEvento"),
            "Result should contain <retEvento>"
        );
        assert!(
            result.contains("<tpEvento>110111</tpEvento>"),
            "Result should contain cancellation event type"
        );
        // The retEvento should appear before </nfeProc>
        let ret_pos = result.find("<retEvento").unwrap();
        let close_pos = result.rfind("</nfeProc>").unwrap();
        assert!(ret_pos < close_pos, "retEvento should be before </nfeProc>");
        // Original content should be preserved
        assert!(result.contains("<protNFe"));
        assert!(result.contains("<NFe>"));
    }

    #[test]
    fn attach_cancellation_ignores_non_matching_ch_nfe() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat>"#,
            r#"<tpEvento>110111</tpEvento>"#,
            r#"<chNFe>99999999999999999999999999999999999999999999</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();
        // Should return original unchanged — no matching chNFe
        assert_eq!(result, nfe_proc);
    }

    #[test]
    fn attach_cancellation_ignores_wrong_tp_evento() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat>"#,
            r#"<tpEvento>110110</tpEvento>"#, // CCe, not cancellation
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();
        // Should return original unchanged — wrong tpEvento
        assert_eq!(result, nfe_proc);
    }

    #[test]
    fn attach_cancellation_ignores_rejected_status() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>573</cStat>"#, // Rejected status
            r#"<tpEvento>110111</tpEvento>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();
        // Should return original unchanged — rejected status
        assert_eq!(result, nfe_proc);
    }

    #[test]
    fn attach_cancellation_accepts_status_155() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>155</cStat>"#,
            r#"<tpEvento>110111</tpEvento>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();
        assert!(result.contains("<retEvento"));
    }

    #[test]
    fn attach_cancellation_accepts_substituicao_110112() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat>"#,
            r#"<tpEvento>110112</tpEvento>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();
        assert!(
            result.contains("<tpEvento>110112</tpEvento>"),
            "Should accept cancellation by substitution"
        );
    }

    #[test]
    fn attach_cancellation_rejects_missing_prot_nfe() {
        let nfe_xml = "<NFe><infNFe/></NFe>";
        let cancel_xml = "<retEvento/>";
        let err = attach_cancellation(nfe_xml, cancel_xml).unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_cancellation_rejects_missing_ch_nfe_in_prot() {
        let nfe_proc = concat!(
            r#"<nfeProc><protNFe versao="4.00"><infProt>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe></nfeProc>"#
        );
        let cancel_xml = "<retEvento/>";
        let err = attach_cancellation(nfe_proc, cancel_xml).unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_cancellation_picks_first_matching_from_multiple_ret_eventos() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEnvEvento>"#,
            // First: wrong chNFe
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat><tpEvento>110111</tpEvento>"#,
            r#"<chNFe>99999999999999999999999999999999999999999999</chNFe>"#,
            r#"<nProt>111111111111111</nProt>"#,
            r#"</infEvento></retEvento>"#,
            // Second: correct match
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat><tpEvento>110111</tpEvento>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>222222222222222</nProt>"#,
            r#"</infEvento></retEvento>"#,
            r#"</retEnvEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();
        assert!(result.contains("<nProt>222222222222222</nProt>"));
        // Should only have one retEvento (the matching one)
        assert_eq!(result.matches("<retEvento").count(), 1);
    }
}
