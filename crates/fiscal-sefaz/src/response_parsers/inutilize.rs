//! Parser for SEFAZ inutilização (`retInutNFe`) responses.

use fiscal_core::FiscalError;
use fiscal_core::xml_utils::extract_xml_tag_value;

use super::helpers::{extract_inner_content, strip_soap_envelope};
use super::types::InutilizacaoResponse;

/// Parse a SEFAZ inutilização response (`retInutNFe`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts all fields from `<infInut>`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element.
pub fn parse_inutilizacao_response(xml: &str) -> Result<InutilizacaoResponse, FiscalError> {
    let body = strip_soap_envelope(xml);

    // Try to narrow into <infInut> first
    let scope = extract_inner_content(&body, "infInut").unwrap_or(&body);

    let c_stat = extract_xml_tag_value(scope, "cStat").ok_or_else(|| {
        FiscalError::XmlParsing("missing <cStat> in inutilização response".into())
    })?;
    let x_motivo = extract_xml_tag_value(scope, "xMotivo").unwrap_or_else(|| "Unknown".into());
    let tp_amb = extract_xml_tag_value(scope, "tpAmb").unwrap_or_default();
    let ver_aplic = extract_xml_tag_value(scope, "verAplic").unwrap_or_default();
    let c_uf = extract_xml_tag_value(scope, "cUF").unwrap_or_default();
    let ano = extract_xml_tag_value(scope, "ano").unwrap_or_default();
    let cnpj = extract_xml_tag_value(scope, "CNPJ").unwrap_or_default();
    let cpf = extract_xml_tag_value(scope, "CPF");
    let modelo = extract_xml_tag_value(scope, "mod").unwrap_or_default();
    let serie = extract_xml_tag_value(scope, "serie").unwrap_or_default();
    let n_nf_ini = extract_xml_tag_value(scope, "nNFIni").unwrap_or_default();
    let n_nf_fin = extract_xml_tag_value(scope, "nNFFin").unwrap_or_default();
    let dh_recbto = extract_xml_tag_value(scope, "dhRecbto");
    let n_prot = extract_xml_tag_value(scope, "nProt");

    Ok(InutilizacaoResponse {
        tp_amb,
        ver_aplic,
        c_stat,
        x_motivo,
        c_uf,
        ano,
        cnpj,
        cpf,
        modelo,
        serie,
        n_nf_ini,
        n_nf_fin,
        dh_recbto,
        n_prot,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_inutilizacao_response ─────────────────────────────────

    #[test]
    fn parses_inutilizacao_response() {
        let xml = concat!(
            r#"<retInutNFe versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            "<infInut>",
            "<tpAmb>2</tpAmb>",
            "<verAplic>SP_NFE_PL009_V4</verAplic>",
            "<cStat>102</cStat>",
            "<xMotivo>Inutilizacao de numero homologado</xMotivo>",
            "<cUF>35</cUF>",
            "<ano>24</ano>",
            "<CNPJ>11222333000181</CNPJ>",
            "<mod>55</mod>",
            "<serie>1</serie>",
            "<nNFIni>100</nNFIni>",
            "<nNFFin>110</nNFFin>",
            "<dhRecbto>2024-06-01T14:30:00-03:00</dhRecbto>",
            "<nProt>135240000054321</nProt>",
            "</infInut>",
            "</retInutNFe>"
        );
        let resp = parse_inutilizacao_response(xml).unwrap();
        assert_eq!(resp.tp_amb, "2");
        assert_eq!(resp.ver_aplic, "SP_NFE_PL009_V4");
        assert_eq!(resp.c_stat, "102");
        assert_eq!(resp.x_motivo, "Inutilizacao de numero homologado");
        assert_eq!(resp.c_uf, "35");
        assert_eq!(resp.ano, "24");
        assert_eq!(resp.cnpj, "11222333000181");
        assert_eq!(resp.modelo, "55");
        assert_eq!(resp.serie, "1");
        assert_eq!(resp.n_nf_ini, "100");
        assert_eq!(resp.n_nf_fin, "110");
        assert_eq!(resp.dh_recbto.as_deref(), Some("2024-06-01T14:30:00-03:00"));
        assert_eq!(resp.n_prot.as_deref(), Some("135240000054321"));
    }

    #[test]
    fn parses_inutilizacao_response_without_optional_fields() {
        let xml = concat!(
            "<retInutNFe><infInut>",
            "<tpAmb>1</tpAmb>",
            "<verAplic>SVRS202406</verAplic>",
            "<cStat>102</cStat>",
            "<xMotivo>Inutilizacao de numero homologado</xMotivo>",
            "<cUF>43</cUF>",
            "<ano>24</ano>",
            "<CNPJ>99888777000166</CNPJ>",
            "<mod>65</mod>",
            "<serie>2</serie>",
            "<nNFIni>50</nNFIni>",
            "<nNFFin>60</nNFFin>",
            "</infInut></retInutNFe>"
        );
        let resp = parse_inutilizacao_response(xml).unwrap();
        assert_eq!(resp.tp_amb, "1");
        assert_eq!(resp.c_stat, "102");
        assert_eq!(resp.c_uf, "43");
        assert_eq!(resp.modelo, "65");
        assert_eq!(resp.n_nf_ini, "50");
        assert_eq!(resp.n_nf_fin, "60");
        assert_eq!(resp.dh_recbto, None);
        assert_eq!(resp.n_prot, None);
    }

    #[test]
    fn parses_soap_wrapped_inutilizacao_response() {
        let xml = r#"<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
            <soap:Body>
                <nfeResultMsg:nfeInutilizacaoNF2Result xmlns:nfeResultMsg="http://www.portalfiscal.inf.br/nfe/wsdl/NFeInutilizacao4">
                    <nfe:retInutNFe xmlns:nfe="http://www.portalfiscal.inf.br/nfe" versao="4.00">
                        <nfe:infInut>
                            <nfe:tpAmb>2</nfe:tpAmb>
                            <nfe:verAplic>SP_NFE_PL009_V4</nfe:verAplic>
                            <nfe:cStat>102</nfe:cStat>
                            <nfe:xMotivo>Inutilizacao de numero homologado</nfe:xMotivo>
                            <nfe:cUF>35</nfe:cUF>
                            <nfe:ano>24</nfe:ano>
                            <nfe:CNPJ>11222333000181</nfe:CNPJ>
                            <nfe:mod>55</nfe:mod>
                            <nfe:serie>1</nfe:serie>
                            <nfe:nNFIni>200</nfe:nNFIni>
                            <nfe:nNFFin>250</nfe:nNFFin>
                            <nfe:dhRecbto>2024-07-10T09:15:00-03:00</nfe:dhRecbto>
                            <nfe:nProt>135240000067890</nfe:nProt>
                        </nfe:infInut>
                    </nfe:retInutNFe>
                </nfeResultMsg:nfeInutilizacaoNF2Result>
            </soap:Body>
        </soap:Envelope>"#;
        let resp = parse_inutilizacao_response(xml).unwrap();
        assert_eq!(resp.tp_amb, "2");
        assert_eq!(resp.c_stat, "102");
        assert_eq!(resp.x_motivo, "Inutilizacao de numero homologado");
        assert_eq!(resp.c_uf, "35");
        assert_eq!(resp.cnpj, "11222333000181");
        assert_eq!(resp.n_nf_ini, "200");
        assert_eq!(resp.n_nf_fin, "250");
        assert_eq!(resp.n_prot.as_deref(), Some("135240000067890"));
    }

    #[test]
    fn inutilizacao_rejects_malformed_xml() {
        let err = parse_inutilizacao_response("<retInutNFe>nothing</retInutNFe>").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    // ── parse_inutilizacao_response with CPF ────────────────────────

    #[test]
    fn parses_inutilizacao_response_with_cpf() {
        let xml = concat!(
            "<retInutNFe><infInut>",
            "<tpAmb>2</tpAmb>",
            "<verAplic>SVRS202406</verAplic>",
            "<cStat>102</cStat>",
            "<xMotivo>Inutilizacao de numero homologado</xMotivo>",
            "<cUF>51</cUF>",
            "<ano>24</ano>",
            "<CPF>12345678901</CPF>",
            "<mod>55</mod>",
            "<serie>1</serie>",
            "<nNFIni>1</nNFIni>",
            "<nNFFin>10</nNFFin>",
            "</infInut></retInutNFe>"
        );
        let resp = parse_inutilizacao_response(xml).unwrap();
        assert_eq!(resp.cpf.as_deref(), Some("12345678901"));
        assert_eq!(resp.cnpj, ""); // No CNPJ in this case
    }
}
