//! Attach event protocol response to the event request (`<procEventoNFe>` wrapper).

use crate::FiscalError;
use crate::xml_utils::extract_xml_tag_value;

use super::helpers::{DEFAULT_VERSION, extract_attribute, extract_tag, join_xml};

/// Cancellation event type code (`110111`).
const EVT_CANCELA: &str = "110111";

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
/// - The `<idLote>` tag is missing from `request_xml` or `response_xml`
/// - The `idLote` values differ between request and response
///
/// Returns [`FiscalError::SefazRejection`] if the event status code
/// is not valid (135, 136, or 155 for cancellation only).
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

    // Validate event status FIRST (PHP validates cStat before idLote)
    let c_stat = extract_xml_tag_value(&ret_evento_content, "cStat").unwrap_or_default();
    let tp_evento = extract_xml_tag_value(&ret_evento_content, "tpEvento").unwrap_or_default();

    // Build the valid statuses list: 135, 136 always; 155 only for cancellation
    let mut valid_statuses: Vec<&str> = vec!["135", "136"];
    if tp_evento == EVT_CANCELA {
        valid_statuses.push("155");
    }

    if !valid_statuses.contains(&c_stat.as_str()) {
        let x_motivo = extract_xml_tag_value(&ret_evento_content, "xMotivo").unwrap_or_default();
        return Err(FiscalError::SefazRejection {
            code: c_stat,
            message: x_motivo,
        });
    }

    // Validate idLote is present in both request and response, then compare.
    // PHP addEnvEventoProtocol accesses ->nodeValue directly on idLote;
    // if the tag is absent, PHP throws a fatal error.
    let req_id_lote = extract_xml_tag_value(request_xml, "idLote")
        .ok_or_else(|| FiscalError::XmlParsing("idLote not found in request XML".into()))?;
    let ret_id_lote = extract_xml_tag_value(response_xml, "idLote")
        .ok_or_else(|| FiscalError::XmlParsing("idLote not found in response XML".into()))?;
    if req_id_lote != ret_id_lote {
        return Err(FiscalError::XmlParsing(
            "Os números de lote dos documentos são diferentes".into(),
        ));
    }

    Ok(join_xml(
        &evento_content,
        &ret_evento_content,
        "procEventoNFe",
        &version,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attach_event_protocol_empty_request() {
        let err = attach_event_protocol("", "<retEvento/>").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_event_protocol_empty_response() {
        let err = attach_event_protocol("<evento/>", "").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_event_protocol_missing_evento() {
        let err = attach_event_protocol(
            "<other/>",
            "<retEvento><infEvento><cStat>135</cStat></infEvento></retEvento>",
        )
        .unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_event_protocol_missing_ret_evento() {
        let err =
            attach_event_protocol(r#"<evento versao="1.00"><infEvento/></evento>"#, "<other/>")
                .unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_event_protocol_rejected_status() {
        let err = attach_event_protocol(
            r#"<evento versao="1.00"><infEvento/></evento>"#,
            r#"<retEvento><infEvento><cStat>999</cStat><xMotivo>Rejeitado</xMotivo></infEvento></retEvento>"#,
        )
        .unwrap_err();
        assert!(matches!(err, FiscalError::SefazRejection { .. }));
    }

    #[test]
    fn attach_event_protocol_success() {
        let request = concat!(
            r#"<envEvento><idLote>100</idLote>"#,
            r#"<evento versao="1.00"><infEvento Id="ID1234"/></evento>"#,
            r#"</envEvento>"#
        );
        let response = concat!(
            r#"<retEnvEvento><idLote>100</idLote>"#,
            r#"<retEvento><infEvento><cStat>135</cStat>"#,
            r#"<xMotivo>Evento registrado</xMotivo>"#,
            r#"</infEvento></retEvento></retEnvEvento>"#
        );
        let result = attach_event_protocol(request, response).unwrap();
        assert!(result.contains("<procEventoNFe"));
        assert!(result.contains("<evento"));
        assert!(result.contains("<retEvento>"));
    }

    // ── attach_event_protocol: idLote mismatch (lines 277-278) ──────────

    #[test]
    fn attach_event_protocol_id_lote_mismatch() {
        let request = concat!(
            r#"<envEvento><idLote>100</idLote>"#,
            r#"<evento versao="1.00"><infEvento>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></evento></envEvento>"#
        );
        let response = concat!(
            r#"<retEnvEvento><idLote>999</idLote>"#,
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat><xMotivo>OK</xMotivo>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></retEvento></retEnvEvento>"#
        );
        let err = attach_event_protocol(request, response).unwrap_err();
        match err {
            FiscalError::XmlParsing(msg) => {
                assert!(
                    msg.contains("lote"),
                    "Expected lote mismatch error, got: {msg}"
                );
            }
            other => panic!("Expected XmlParsing, got {:?}", other),
        }
    }

    // ── attach_event_protocol: cStat validated before idLote (PHP parity) ──

    #[test]
    fn attach_event_protocol_both_invalid_reports_cstat_first() {
        // When BOTH cStat is invalid AND idLote mismatches, the error
        // must be about cStat (SefazRejection), not about idLote,
        // matching PHP addEnvEventoProtocol validation order.
        let request = concat!(
            r#"<envEvento><idLote>100</idLote>"#,
            r#"<evento versao="1.00"><infEvento>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></evento></envEvento>"#
        );
        let response = concat!(
            r#"<retEnvEvento><idLote>999</idLote>"#,
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>573</cStat><xMotivo>Duplicidade de evento</xMotivo>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></retEvento></retEnvEvento>"#
        );
        let err = attach_event_protocol(request, response).unwrap_err();
        match err {
            FiscalError::SefazRejection { code, message } => {
                assert_eq!(code, "573");
                assert_eq!(message, "Duplicidade de evento");
            }
            other => panic!("Expected SefazRejection (cStat first), got {:?}", other),
        }
    }

    // ── attach_event_protocol: missing idLote ─────────────────────────

    #[test]
    fn attach_event_protocol_missing_id_lote_in_request() {
        let request = concat!(
            r#"<envEvento>"#,
            r#"<evento versao="1.00"><infEvento>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></evento></envEvento>"#
        );
        let response = concat!(
            r#"<retEnvEvento><idLote>100</idLote>"#,
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat><xMotivo>OK</xMotivo>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></retEvento></retEnvEvento>"#
        );
        let err = attach_event_protocol(request, response).unwrap_err();
        match err {
            FiscalError::XmlParsing(msg) => {
                assert_eq!(msg, "idLote not found in request XML");
            }
            other => panic!("Expected XmlParsing, got {:?}", other),
        }
    }

    #[test]
    fn attach_event_protocol_missing_id_lote_in_response() {
        let request = concat!(
            r#"<envEvento><idLote>100</idLote>"#,
            r#"<evento versao="1.00"><infEvento>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></evento></envEvento>"#
        );
        let response = concat!(
            r#"<retEnvEvento>"#,
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat><xMotivo>OK</xMotivo>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></retEvento></retEnvEvento>"#
        );
        let err = attach_event_protocol(request, response).unwrap_err();
        match err {
            FiscalError::XmlParsing(msg) => {
                assert_eq!(msg, "idLote not found in response XML");
            }
            other => panic!("Expected XmlParsing, got {:?}", other),
        }
    }

    #[test]
    fn attach_event_protocol_missing_id_lote_in_both() {
        let request = concat!(
            r#"<envEvento>"#,
            r#"<evento versao="1.00"><infEvento>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></evento></envEvento>"#
        );
        let response = concat!(
            r#"<retEnvEvento>"#,
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat><xMotivo>OK</xMotivo>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></retEvento></retEnvEvento>"#
        );
        let err = attach_event_protocol(request, response).unwrap_err();
        match err {
            FiscalError::XmlParsing(msg) => {
                assert_eq!(msg, "idLote not found in request XML");
            }
            other => panic!("Expected XmlParsing, got {:?}", other),
        }
    }
}
