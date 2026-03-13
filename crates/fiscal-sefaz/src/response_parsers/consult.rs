//! Parsers for SEFAZ consulta recibo (`retConsReciNFe`) and consulta
//! situação (`retConsSitNFe`) responses.

use fiscal_core::FiscalError;
use fiscal_core::xml_utils::extract_xml_tag_value;

use super::helpers::{
    extract_all_raw_tags, extract_inner_content, extract_raw_tag, strip_soap_envelope,
};
use super::types::{ConsultaReciboResponse, ConsultaSituacaoResponse, ProtocolInfo};

/// Parse a SEFAZ consulta recibo response (`retConsReciNFe`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts all batch-level fields
/// and individual `<protNFe>` protocol entries.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element.
pub fn parse_consulta_recibo_response(xml: &str) -> Result<ConsultaReciboResponse, FiscalError> {
    let body = strip_soap_envelope(xml);

    let c_stat = extract_xml_tag_value(&body, "cStat").ok_or_else(|| {
        FiscalError::XmlParsing("missing <cStat> in consulta recibo response".into())
    })?;
    let x_motivo = extract_xml_tag_value(&body, "xMotivo").unwrap_or_default();
    let tp_amb = extract_xml_tag_value(&body, "tpAmb").unwrap_or_default();
    let ver_aplic = extract_xml_tag_value(&body, "verAplic").unwrap_or_default();
    let n_rec = extract_xml_tag_value(&body, "nRec").unwrap_or_default();
    let c_uf = extract_xml_tag_value(&body, "cUF").unwrap_or_default();

    // Collect all <protNFe> entries
    let prot_xmls = extract_all_raw_tags(&body, "protNFe");
    let mut protocols = Vec::new();
    for prot_xml in &prot_xmls {
        let inf_prot = extract_inner_content(prot_xml, "infProt").unwrap_or(prot_xml);
        let prot_c_stat = match extract_xml_tag_value(inf_prot, "cStat") {
            Some(v) => v,
            None => continue,
        };
        protocols.push(ProtocolInfo {
            tp_amb: extract_xml_tag_value(inf_prot, "tpAmb").unwrap_or_default(),
            ver_aplic: extract_xml_tag_value(inf_prot, "verAplic").unwrap_or_default(),
            ch_nfe: extract_xml_tag_value(inf_prot, "chNFe").unwrap_or_default(),
            dh_recbto: extract_xml_tag_value(inf_prot, "dhRecbto"),
            n_prot: extract_xml_tag_value(inf_prot, "nProt"),
            dig_val: extract_xml_tag_value(inf_prot, "digVal"),
            c_stat: prot_c_stat,
            x_motivo: extract_xml_tag_value(inf_prot, "xMotivo").unwrap_or_default(),
        });
    }

    Ok(ConsultaReciboResponse {
        tp_amb,
        ver_aplic,
        n_rec,
        c_stat,
        x_motivo,
        c_uf,
        protocols,
    })
}

/// Parse a SEFAZ consulta situação response (`retConsSitNFe`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts the situation fields,
/// optional `<protNFe>` and any `<retEvento>` entries.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element.
pub fn parse_consulta_situacao_response(
    xml: &str,
) -> Result<ConsultaSituacaoResponse, FiscalError> {
    let body = strip_soap_envelope(xml);

    let c_stat = extract_xml_tag_value(&body, "cStat").ok_or_else(|| {
        FiscalError::XmlParsing("missing <cStat> in consulta situação response".into())
    })?;
    let x_motivo = extract_xml_tag_value(&body, "xMotivo").unwrap_or_default();
    let tp_amb = extract_xml_tag_value(&body, "tpAmb").unwrap_or_default();
    let ver_aplic = extract_xml_tag_value(&body, "verAplic").unwrap_or_default();
    let c_uf = extract_xml_tag_value(&body, "cUF").unwrap_or_default();
    let ch_nfe = extract_xml_tag_value(&body, "chNFe");

    let protocol_xml = extract_raw_tag(&body, "protNFe");

    // Collect all <retEvento> entries
    let event_xmls = extract_all_raw_tags(&body, "retEvento");

    Ok(ConsultaSituacaoResponse {
        tp_amb,
        ver_aplic,
        c_stat,
        x_motivo,
        c_uf,
        ch_nfe,
        protocol_xml,
        event_xmls,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_consulta_recibo_response ──────────────────────────────

    #[test]
    fn parses_consulta_recibo_response() {
        let xml = concat!(
            r#"<retConsReciNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00">"#,
            "<tpAmb>2</tpAmb>",
            "<verAplic>SP_NFE_PL009_V4</verAplic>",
            "<nRec>351000000012345</nRec>",
            "<cStat>104</cStat>",
            "<xMotivo>Lote processado</xMotivo>",
            "<cUF>35</cUF>",
            r#"<protNFe versao="4.00"><infProt>"#,
            "<tpAmb>2</tpAmb>",
            "<verAplic>SP_NFE_PL009_V4</verAplic>",
            "<chNFe>35260112345678000199550010000000011123456780</chNFe>",
            "<dhRecbto>2024-06-01T14:30:00-03:00</dhRecbto>",
            "<nProt>135240000054321</nProt>",
            "<digVal>dGVzdGU=</digVal>",
            "<cStat>100</cStat>",
            "<xMotivo>Autorizado o uso da NF-e</xMotivo>",
            "</infProt></protNFe>",
            "</retConsReciNFe>"
        );
        let resp = parse_consulta_recibo_response(xml).unwrap();
        assert_eq!(resp.tp_amb, "2");
        assert_eq!(resp.ver_aplic, "SP_NFE_PL009_V4");
        assert_eq!(resp.n_rec, "351000000012345");
        assert_eq!(resp.c_stat, "104");
        assert_eq!(resp.x_motivo, "Lote processado");
        assert_eq!(resp.c_uf, "35");
        assert_eq!(resp.protocols.len(), 1);
        assert_eq!(
            resp.protocols[0].ch_nfe,
            "35260112345678000199550010000000011123456780"
        );
        assert_eq!(resp.protocols[0].c_stat, "100");
        assert_eq!(resp.protocols[0].n_prot.as_deref(), Some("135240000054321"));
        assert_eq!(resp.protocols[0].dig_val.as_deref(), Some("dGVzdGU="));
    }

    #[test]
    fn parses_consulta_recibo_response_multiple_protocols() {
        let xml = concat!(
            r#"<retConsReciNFe versao="4.00">"#,
            "<tpAmb>2</tpAmb>",
            "<verAplic>SP</verAplic>",
            "<nRec>351000000099999</nRec>",
            "<cStat>104</cStat>",
            "<xMotivo>Lote processado</xMotivo>",
            "<cUF>35</cUF>",
            r#"<protNFe versao="4.00"><infProt>"#,
            "<chNFe>11111111111111111111111111111111111111111111</chNFe>",
            "<cStat>100</cStat><xMotivo>OK</xMotivo>",
            "</infProt></protNFe>",
            r#"<protNFe versao="4.00"><infProt>"#,
            "<chNFe>22222222222222222222222222222222222222222222</chNFe>",
            "<cStat>100</cStat><xMotivo>OK</xMotivo>",
            "</infProt></protNFe>",
            "</retConsReciNFe>"
        );
        let resp = parse_consulta_recibo_response(xml).unwrap();
        assert_eq!(resp.protocols.len(), 2);
        assert_eq!(
            resp.protocols[0].ch_nfe,
            "11111111111111111111111111111111111111111111"
        );
        assert_eq!(
            resp.protocols[1].ch_nfe,
            "22222222222222222222222222222222222222222222"
        );
    }

    #[test]
    fn consulta_recibo_rejects_malformed_xml() {
        let err = parse_consulta_recibo_response("<garbage>nothing</garbage>").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    // ── parse_consulta_situacao_response ────────────────────────────

    #[test]
    fn parses_consulta_situacao_response() {
        let xml = concat!(
            r#"<retConsSitNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00">"#,
            "<tpAmb>2</tpAmb>",
            "<verAplic>SP_NFE_PL009_V4</verAplic>",
            "<cStat>100</cStat>",
            "<xMotivo>Autorizado o uso da NF-e</xMotivo>",
            "<cUF>35</cUF>",
            "<chNFe>35260112345678000199550010000000011123456780</chNFe>",
            r#"<protNFe versao="4.00"><infProt>"#,
            "<cStat>100</cStat>",
            "<xMotivo>Autorizado o uso da NF-e</xMotivo>",
            "<nProt>135240000054321</nProt>",
            "</infProt></protNFe>",
            "</retConsSitNFe>"
        );
        let resp = parse_consulta_situacao_response(xml).unwrap();
        assert_eq!(resp.tp_amb, "2");
        assert_eq!(resp.c_stat, "100");
        assert_eq!(
            resp.ch_nfe.as_deref(),
            Some("35260112345678000199550010000000011123456780")
        );
        assert!(resp.protocol_xml.is_some());
        assert!(resp.protocol_xml.as_ref().unwrap().contains("<protNFe"));
        assert!(resp.event_xmls.is_empty());
    }

    #[test]
    fn parses_consulta_situacao_response_with_events() {
        let xml = concat!(
            r#"<retConsSitNFe versao="4.00">"#,
            "<tpAmb>2</tpAmb>",
            "<verAplic>SP</verAplic>",
            "<cStat>100</cStat>",
            "<xMotivo>Autorizado</xMotivo>",
            "<cUF>35</cUF>",
            "<chNFe>35260112345678000199550010000000011123456780</chNFe>",
            r#"<protNFe versao="4.00"><infProt>"#,
            "<cStat>100</cStat><xMotivo>OK</xMotivo>",
            "</infProt></protNFe>",
            r#"<retEvento versao="1.00"><infEvento>"#,
            "<cStat>135</cStat><tpEvento>110111</tpEvento>",
            "</infEvento></retEvento>",
            "</retConsSitNFe>"
        );
        let resp = parse_consulta_situacao_response(xml).unwrap();
        assert!(resp.protocol_xml.is_some());
        assert_eq!(resp.event_xmls.len(), 1);
        assert!(resp.event_xmls[0].contains("<retEvento"));
    }

    #[test]
    fn consulta_situacao_rejects_malformed_xml() {
        let err = parse_consulta_situacao_response("<garbage>nothing</garbage>").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }
}
