//! Parser for SEFAZ cancellation event (`retEvento`) responses.

use fiscal_core::FiscalError;
use fiscal_core::xml_utils::extract_xml_tag_value;

use super::helpers::{extract_inner_content, strip_soap_envelope};
use super::types::CancellationResponse;

/// Parse a SEFAZ cancellation event response (`retEvento`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts `cStat`, `xMotivo`, and
/// optionally `nProt` from `<infEvento>`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element.
pub fn parse_cancellation_response(xml: &str) -> Result<CancellationResponse, FiscalError> {
    let body = strip_soap_envelope(xml);

    // Try to narrow into <infEvento> first
    let scope = extract_inner_content(&body, "infEvento").unwrap_or(&body);

    let status_code = extract_xml_tag_value(scope, "cStat").ok_or_else(|| {
        FiscalError::XmlParsing("missing <cStat> in cancellation response".into())
    })?;
    let status_message =
        extract_xml_tag_value(scope, "xMotivo").unwrap_or_else(|| "Unknown".into());
    let protocol_number = extract_xml_tag_value(scope, "nProt");

    Ok(CancellationResponse {
        status_code,
        status_message,
        protocol_number,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_cancellation_response ─────────────────────────────────

    #[test]
    fn parses_cancellation_response() {
        let xml = concat!(
            "<retEvento><infEvento>",
            "<cStat>135</cStat>",
            "<xMotivo>Evento registrado e vinculado a NF-e</xMotivo>",
            "<nProt>135220000009999</nProt>",
            "</infEvento></retEvento>"
        );
        let resp = parse_cancellation_response(xml).unwrap();
        assert_eq!(resp.status_code, "135");
        assert_eq!(resp.status_message, "Evento registrado e vinculado a NF-e");
        assert_eq!(resp.protocol_number.as_deref(), Some("135220000009999"));
    }

    #[test]
    fn cancellation_rejects_malformed_xml() {
        let err = parse_cancellation_response("<retEvento>nothing</retEvento>").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn parses_soap_wrapped_cancellation() {
        let xml = r#"<soap:Envelope>
            <soap:Body>
                <nfe:retEvento xmlns:nfe="http://www.portalfiscal.inf.br/nfe">
                    <nfe:infEvento>
                        <nfe:cStat>135</nfe:cStat>
                        <nfe:xMotivo>Evento registrado</nfe:xMotivo>
                        <nfe:nProt>141240000099999</nfe:nProt>
                    </nfe:infEvento>
                </nfe:retEvento>
            </soap:Body>
        </soap:Envelope>"#;
        let resp = parse_cancellation_response(xml).unwrap();
        assert_eq!(resp.status_code, "135");
        assert_eq!(resp.protocol_number.as_deref(), Some("141240000099999"));
    }
}
