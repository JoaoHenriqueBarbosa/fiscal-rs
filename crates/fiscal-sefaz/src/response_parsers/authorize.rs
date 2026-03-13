//! Parser for SEFAZ authorization (`retEnviNFe`) responses.

use fiscal_core::FiscalError;
use fiscal_core::xml_utils::extract_xml_tag_value;

use super::helpers::{extract_inner_content, extract_raw_tag, strip_soap_envelope};
use super::types::AuthorizationResponse;

/// Parse a SEFAZ authorization response (`retEnviNFe` / `nfeAutorizacaoLoteResult`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts the protocol information
/// from `<protNFe><infProt>` when present.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element at any level.
pub fn parse_autorizacao_response(xml: &str) -> Result<AuthorizationResponse, FiscalError> {
    let body = strip_soap_envelope(xml);

    // Extract receipt number (nRec) at the batch level — present for
    // asynchronous submissions (indSinc=0).
    let receipt_number = extract_xml_tag_value(&body, "nRec");

    // Try to find <protNFe> section first — it carries per-NF-e result
    if let Some(prot_xml) = extract_raw_tag(&body, "protNFe") {
        let inf_prot = extract_inner_content(&prot_xml, "infProt").unwrap_or(&prot_xml);

        let status_code = extract_xml_tag_value(inf_prot, "cStat")
            .ok_or_else(|| FiscalError::XmlParsing("missing <cStat> in <protNFe>".into()))?;
        let status_message = extract_xml_tag_value(inf_prot, "xMotivo").unwrap_or_default();
        let protocol_number = extract_xml_tag_value(inf_prot, "nProt");
        let authorized_at = extract_xml_tag_value(inf_prot, "dhRecbto");

        return Ok(AuthorizationResponse {
            status_code,
            status_message,
            protocol_number,
            protocol_xml: Some(prot_xml.to_string()),
            authorized_at,
            receipt_number,
        });
    }

    // Fallback: batch-level status from <retEnviNFe> directly
    let status_code = extract_xml_tag_value(&body, "cStat").ok_or_else(|| {
        FiscalError::XmlParsing("missing <cStat> in authorization response".into())
    })?;
    let status_message =
        extract_xml_tag_value(&body, "xMotivo").unwrap_or_else(|| "Unknown".into());

    Ok(AuthorizationResponse {
        status_code,
        status_message,
        protocol_number: None,
        protocol_xml: None,
        authorized_at: None,
        receipt_number,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_autorizacao_response ──────────────────────────────────

    #[test]
    fn parses_authorization_with_protocol() {
        let xml = concat!(
            "<retEnviNFe><cStat>104</cStat>",
            r#"<protNFe versao="4.00"><infProt>"#,
            "<cStat>100</cStat>",
            "<xMotivo>Autorizado o uso da NF-e</xMotivo>",
            "<nProt>135220000009921</nProt>",
            "<dhRecbto>2024-05-31T12:00:00-03:00</dhRecbto>",
            "</infProt></protNFe></retEnviNFe>"
        );
        let resp = parse_autorizacao_response(xml).unwrap();
        assert_eq!(resp.status_code, "100");
        assert_eq!(resp.status_message, "Autorizado o uso da NF-e");
        assert_eq!(resp.protocol_number.as_deref(), Some("135220000009921"));
        assert_eq!(
            resp.authorized_at.as_deref(),
            Some("2024-05-31T12:00:00-03:00")
        );
        assert!(resp.protocol_xml.is_some());
        assert!(resp.protocol_xml.as_ref().unwrap().contains("<protNFe"));
        assert!(resp.protocol_xml.as_ref().unwrap().contains("</protNFe>"));
    }

    #[test]
    fn parses_authorization_batch_level_only() {
        let xml = "<retEnviNFe><cStat>105</cStat>\
                    <xMotivo>Lote em processamento</xMotivo></retEnviNFe>";
        let resp = parse_autorizacao_response(xml).unwrap();
        assert_eq!(resp.status_code, "105");
        assert_eq!(resp.status_message, "Lote em processamento");
        assert!(resp.protocol_number.is_none());
        assert!(resp.protocol_xml.is_none());
    }

    #[test]
    fn parses_soap_wrapped_authorization() {
        let xml = r#"<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
            <soap:Body>
                <nfeResultMsg:nfeAutorizacaoLoteResult xmlns:nfeResultMsg="http://www.portalfiscal.inf.br/nfe/wsdl/NFeAutorizacao4">
                    <nfe:retEnviNFe xmlns:nfe="http://www.portalfiscal.inf.br/nfe">
                        <nfe:cStat>104</nfe:cStat>
                        <nfe:protNFe versao="4.00">
                            <nfe:infProt>
                                <nfe:cStat>100</nfe:cStat>
                                <nfe:xMotivo>Autorizado o uso da NF-e</nfe:xMotivo>
                                <nfe:nProt>141240000012345</nfe:nProt>
                                <nfe:dhRecbto>2024-06-15T10:30:00-03:00</nfe:dhRecbto>
                            </nfe:infProt>
                        </nfe:protNFe>
                    </nfe:retEnviNFe>
                </nfeResultMsg:nfeAutorizacaoLoteResult>
            </soap:Body>
        </soap:Envelope>"#;
        let resp = parse_autorizacao_response(xml).unwrap();
        assert_eq!(resp.status_code, "100");
        assert_eq!(resp.protocol_number.as_deref(), Some("141240000012345"));
    }

    #[test]
    fn authorization_rejects_malformed_xml() {
        let err = parse_autorizacao_response("<garbage>no cstat</garbage>").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn parses_authorization_async_receipt_number() {
        let xml = concat!(
            "<retEnviNFe><cStat>103</cStat>",
            "<xMotivo>Lote recebido com sucesso</xMotivo>",
            "<nRec>351000000012345</nRec>",
            "</retEnviNFe>"
        );
        let resp = parse_autorizacao_response(xml).unwrap();
        assert_eq!(resp.status_code, "103");
        assert_eq!(resp.status_message, "Lote recebido com sucesso");
        assert_eq!(resp.receipt_number.as_deref(), Some("351000000012345"));
        assert!(resp.protocol_number.is_none());
    }

    #[test]
    fn parses_authorization_sync_no_receipt() {
        let xml = concat!(
            "<retEnviNFe><cStat>104</cStat>",
            r#"<protNFe versao="4.00"><infProt>"#,
            "<cStat>100</cStat>",
            "<xMotivo>Autorizado o uso da NF-e</xMotivo>",
            "<nProt>135220000009921</nProt>",
            "<dhRecbto>2024-05-31T12:00:00-03:00</dhRecbto>",
            "</infProt></protNFe></retEnviNFe>"
        );
        let resp = parse_autorizacao_response(xml).unwrap();
        assert_eq!(resp.status_code, "100");
        // Sync responses typically don't have nRec
        assert!(resp.receipt_number.is_none());
    }
}
