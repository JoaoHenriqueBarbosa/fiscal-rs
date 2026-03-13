//! Parser for SEFAZ service status (`retConsStatServ`) responses.

use fiscal_core::FiscalError;
use fiscal_core::xml_utils::extract_xml_tag_value;

use super::helpers::strip_soap_envelope;
use super::types::StatusResponse;

/// Parse a SEFAZ service status response (`retConsStatServ`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts `cStat`, `xMotivo`, and
/// optionally `tMed` (average processing time).
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element.
pub fn parse_status_response(xml: &str) -> Result<StatusResponse, FiscalError> {
    let body = strip_soap_envelope(xml);

    let status_code = extract_xml_tag_value(&body, "cStat")
        .ok_or_else(|| FiscalError::XmlParsing("missing <cStat> in status response".into()))?;
    let status_message =
        extract_xml_tag_value(&body, "xMotivo").unwrap_or_else(|| "Unknown".into());
    let average_time = extract_xml_tag_value(&body, "tMed");

    Ok(StatusResponse {
        status_code,
        status_message,
        average_time,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_status_response() {
        let xml = "<retConsStatServ><cStat>107</cStat>\
                    <xMotivo>Servico em Operacao</xMotivo>\
                    <tMed>1</tMed></retConsStatServ>";
        let resp = parse_status_response(xml).unwrap();
        assert_eq!(resp.status_code, "107");
        assert_eq!(resp.status_message, "Servico em Operacao");
        assert_eq!(resp.average_time.as_deref(), Some("1"));
    }

    #[test]
    fn parses_status_response_without_tmed() {
        let xml = "<retConsStatServ><cStat>107</cStat>\
                    <xMotivo>Servico em Operacao</xMotivo></retConsStatServ>";
        let resp = parse_status_response(xml).unwrap();
        assert_eq!(resp.status_code, "107");
        assert_eq!(resp.average_time, None);
    }

    #[test]
    fn parses_soap_wrapped_status_response() {
        let xml = r#"<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
            <soap:Body>
                <nfeResultMsg:nfeStatusServicoNF2Result xmlns:nfeResultMsg="http://www.portalfiscal.inf.br/nfe/wsdl/NFeStatusServico4">
                    <nfe:retConsStatServ xmlns:nfe="http://www.portalfiscal.inf.br/nfe">
                        <nfe:cStat>107</nfe:cStat>
                        <nfe:xMotivo>Servico em Operacao</nfe:xMotivo>
                        <nfe:tMed>2</nfe:tMed>
                    </nfe:retConsStatServ>
                </nfeResultMsg:nfeStatusServicoNF2Result>
            </soap:Body>
        </soap:Envelope>"#;
        let resp = parse_status_response(xml).unwrap();
        assert_eq!(resp.status_code, "107");
        assert_eq!(resp.status_message, "Servico em Operacao");
        assert_eq!(resp.average_time.as_deref(), Some("2"));
    }

    #[test]
    fn status_response_rejects_malformed_xml() {
        let xml = "<retConsStatServ><xMotivo>ok</xMotivo></retConsStatServ>";
        let err = parse_status_response(xml).unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }
}
