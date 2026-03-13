//! Parsers for SEFAZ DistDFe (`retDistDFeInt`), Cadastro (`retConsCad`),
//! and NFC-e CSC administration (`retAdmCscNFCe`) responses.

use fiscal_core::FiscalError;
use fiscal_core::xml_utils::extract_xml_tag_value;

use super::helpers::{extract_all_tag_values, extract_inner_content, strip_soap_envelope};
use super::types::{CadastroResponse, CscResponse, CscToken, DistDFeResponse};

/// Parse a SEFAZ DistDFe response (`retDistDFeInt`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts `cStat`, `xMotivo`,
/// `ultNSU`, and `maxNSU`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element.
pub fn parse_dist_dfe_response(xml: &str) -> Result<DistDFeResponse, FiscalError> {
    let body = strip_soap_envelope(xml);

    let status_code = extract_xml_tag_value(&body, "cStat")
        .ok_or_else(|| FiscalError::XmlParsing("missing <cStat> in DistDFe response".into()))?;
    let status_message =
        extract_xml_tag_value(&body, "xMotivo").unwrap_or_else(|| "Unknown".into());
    let ult_nsu = extract_xml_tag_value(&body, "ultNSU");
    let max_nsu = extract_xml_tag_value(&body, "maxNSU");

    Ok(DistDFeResponse {
        status_code,
        status_message,
        ult_nsu,
        max_nsu,
        raw_xml: body,
    })
}

/// Parse a SEFAZ Cadastro response (`retConsCad`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts `cStat` and `xMotivo`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element.
pub fn parse_cadastro_response(xml: &str) -> Result<CadastroResponse, FiscalError> {
    let body = strip_soap_envelope(xml);

    // Try to narrow into <infCons> first
    let scope = extract_inner_content(&body, "infCons").unwrap_or(&body);

    let status_code = extract_xml_tag_value(scope, "cStat")
        .ok_or_else(|| FiscalError::XmlParsing("missing <cStat> in Cadastro response".into()))?;
    let status_message =
        extract_xml_tag_value(scope, "xMotivo").unwrap_or_else(|| "Unknown".into());

    Ok(CadastroResponse {
        status_code,
        status_message,
        raw_xml: body,
    })
}

/// Parse a SEFAZ NFC-e CSC administration response (`retAdmCscNFCe`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts `tpAmb`, `indOp`, `cStat`,
/// `xMotivo`, and any `idCsc`/`CSC` token pairs from `<retInfCsc>`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element.
pub fn parse_csc_response(xml: &str) -> Result<CscResponse, FiscalError> {
    let body = strip_soap_envelope(xml);

    // Try to narrow into <retInfCsc> first
    let scope = extract_inner_content(&body, "retInfCsc").unwrap_or(&body);

    let c_stat = extract_xml_tag_value(scope, "cStat")
        .ok_or_else(|| FiscalError::XmlParsing("missing <cStat> in CSC response".into()))?;
    let x_motivo = extract_xml_tag_value(scope, "xMotivo").unwrap_or_default();
    let tp_amb = extract_xml_tag_value(scope, "tpAmb").unwrap_or_default();
    let ind_op = extract_xml_tag_value(scope, "indOp").unwrap_or_default();

    // Collect all <idCsc>/<CSC> pairs
    let ids = extract_all_tag_values(scope, "idCsc");
    let cscs = extract_all_tag_values(scope, "CSC");

    let tokens: Vec<CscToken> = ids
        .into_iter()
        .zip(cscs)
        .map(|(id_csc, csc)| CscToken { id_csc, csc })
        .collect();

    Ok(CscResponse {
        tp_amb,
        ind_op,
        c_stat,
        x_motivo,
        tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_dist_dfe_response ───────────────────────────────────────

    #[test]
    fn parses_dist_dfe_response() {
        let xml = concat!(
            "<retDistDFeInt><cStat>137</cStat>",
            "<xMotivo>Nenhum documento localizado</xMotivo>",
            "<ultNSU>000000000000000</ultNSU>",
            "<maxNSU>000000000012345</maxNSU>",
            "</retDistDFeInt>"
        );
        let resp = parse_dist_dfe_response(xml).unwrap();
        assert_eq!(resp.status_code, "137");
        assert_eq!(resp.status_message, "Nenhum documento localizado");
        assert_eq!(resp.ult_nsu.as_deref(), Some("000000000000000"));
        assert_eq!(resp.max_nsu.as_deref(), Some("000000000012345"));
    }

    #[test]
    fn dist_dfe_response_rejects_malformed_xml() {
        let err = parse_dist_dfe_response("<garbage>nothing</garbage>").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    // ── parse_cadastro_response ───────────────────────────────────────

    #[test]
    fn parses_cadastro_response() {
        let xml = concat!(
            "<retConsCad><infCons>",
            "<cStat>111</cStat>",
            "<xMotivo>Consulta cadastro com uma ocorrencia</xMotivo>",
            "</infCons></retConsCad>"
        );
        let resp = parse_cadastro_response(xml).unwrap();
        assert_eq!(resp.status_code, "111");
        assert_eq!(resp.status_message, "Consulta cadastro com uma ocorrencia");
    }

    #[test]
    fn cadastro_response_rejects_malformed_xml() {
        let err = parse_cadastro_response("<garbage>nothing</garbage>").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    // ── parse_csc_response ──────────────────────────────────────────

    #[test]
    fn parses_csc_response_consulta_ind_op_1() {
        let xml = concat!(
            r#"<retAdmCscNFCe versao="1.00">"#,
            "<retInfCsc>",
            "<tpAmb>2</tpAmb>",
            "<indOp>1</indOp>",
            "<cStat>150</cStat>",
            "<xMotivo>Consulta CSC efetivada</xMotivo>",
            "<idCsc>000001</idCsc>",
            "<CSC>AAAA-BBBB-CCCC-DDDD-1111</CSC>",
            "<idCsc>000002</idCsc>",
            "<CSC>EEEE-FFFF-GGGG-HHHH-2222</CSC>",
            "</retInfCsc>",
            "</retAdmCscNFCe>"
        );
        let resp = parse_csc_response(xml).unwrap();
        assert_eq!(resp.tp_amb, "2");
        assert_eq!(resp.ind_op, "1");
        assert_eq!(resp.c_stat, "150");
        assert_eq!(resp.x_motivo, "Consulta CSC efetivada");
        assert_eq!(resp.tokens.len(), 2);
        assert_eq!(resp.tokens[0].id_csc, "000001");
        assert_eq!(resp.tokens[0].csc, "AAAA-BBBB-CCCC-DDDD-1111");
        assert_eq!(resp.tokens[1].id_csc, "000002");
        assert_eq!(resp.tokens[1].csc, "EEEE-FFFF-GGGG-HHHH-2222");
    }

    #[test]
    fn parses_csc_response_novo_ind_op_2() {
        let xml = concat!(
            r#"<retAdmCscNFCe versao="1.00">"#,
            "<retInfCsc>",
            "<tpAmb>1</tpAmb>",
            "<indOp>2</indOp>",
            "<cStat>151</cStat>",
            "<xMotivo>Novo CSC gerado com sucesso</xMotivo>",
            "<idCsc>000003</idCsc>",
            "<CSC>ZZZZ-YYYY-XXXX-WWWW-3333</CSC>",
            "</retInfCsc>",
            "</retAdmCscNFCe>"
        );
        let resp = parse_csc_response(xml).unwrap();
        assert_eq!(resp.tp_amb, "1");
        assert_eq!(resp.ind_op, "2");
        assert_eq!(resp.c_stat, "151");
        assert_eq!(resp.x_motivo, "Novo CSC gerado com sucesso");
        assert_eq!(resp.tokens.len(), 1);
        assert_eq!(resp.tokens[0].id_csc, "000003");
        assert_eq!(resp.tokens[0].csc, "ZZZZ-YYYY-XXXX-WWWW-3333");
    }

    #[test]
    fn parses_csc_response_revogar_ind_op_3() {
        let xml = concat!(
            r#"<retAdmCscNFCe versao="1.00">"#,
            "<retInfCsc>",
            "<tpAmb>2</tpAmb>",
            "<indOp>3</indOp>",
            "<cStat>152</cStat>",
            "<xMotivo>CSC revogado com sucesso</xMotivo>",
            "</retInfCsc>",
            "</retAdmCscNFCe>"
        );
        let resp = parse_csc_response(xml).unwrap();
        assert_eq!(resp.tp_amb, "2");
        assert_eq!(resp.ind_op, "3");
        assert_eq!(resp.c_stat, "152");
        assert_eq!(resp.x_motivo, "CSC revogado com sucesso");
        assert_eq!(resp.tokens.len(), 0);
    }

    #[test]
    fn parses_soap_wrapped_csc_response() {
        let xml = r#"<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
            <soap:Body>
                <nfe:retAdmCscNFCe xmlns:nfe="http://www.portalfiscal.inf.br/nfe" versao="1.00">
                    <nfe:retInfCsc>
                        <nfe:tpAmb>2</nfe:tpAmb>
                        <nfe:indOp>1</nfe:indOp>
                        <nfe:cStat>150</nfe:cStat>
                        <nfe:xMotivo>Consulta CSC efetivada</nfe:xMotivo>
                        <nfe:idCsc>000001</nfe:idCsc>
                        <nfe:CSC>SOAP-TOKEN-1111</nfe:CSC>
                    </nfe:retInfCsc>
                </nfe:retAdmCscNFCe>
            </soap:Body>
        </soap:Envelope>"#;
        let resp = parse_csc_response(xml).unwrap();
        assert_eq!(resp.c_stat, "150");
        assert_eq!(resp.ind_op, "1");
        assert_eq!(resp.tokens.len(), 1);
        assert_eq!(resp.tokens[0].id_csc, "000001");
        assert_eq!(resp.tokens[0].csc, "SOAP-TOKEN-1111");
    }

    #[test]
    fn csc_response_rejects_malformed_xml() {
        let err = parse_csc_response("<garbage>nothing</garbage>").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }
}
