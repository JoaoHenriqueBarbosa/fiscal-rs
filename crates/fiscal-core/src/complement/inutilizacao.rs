//! Attach SEFAZ inutilizacao response to the request (`<ProcInutNFe>` wrapper).

use crate::FiscalError;
use crate::status_codes::sefaz_status;
use crate::xml_utils::extract_xml_tag_value;

use super::helpers::{DEFAULT_VERSION, extract_attribute, extract_tag, join_xml};

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

    // Cross-validate request vs response fields (like PHP addInutNFeProtocol)
    let ret_version = extract_attribute(&ret_inut_content, "retInutNFe", "versao")
        .unwrap_or_else(|| DEFAULT_VERSION.to_string());

    // Determine whether the request uses CNPJ or CPF
    let cpf_or_cnpj_tag = if extract_xml_tag_value(&inut_content, "CNPJ").is_some() {
        "CNPJ"
    } else {
        "CPF"
    };

    let field_pairs: &[(&str, &str, &str)] = &[("versao", &version, &ret_version)];
    for &(name, req_val, ret_val) in field_pairs {
        if req_val != ret_val {
            return Err(FiscalError::XmlParsing(format!(
                "Inutilização: {name} diverge entre request ({req_val}) e response ({ret_val})"
            )));
        }
    }

    let tag_pairs: &[&str] = &[
        "tpAmb",
        "cUF",
        "ano",
        cpf_or_cnpj_tag,
        "mod",
        "serie",
        "nNFIni",
        "nNFFin",
    ];
    for tag_name in tag_pairs {
        let req_val = extract_xml_tag_value(&inut_content, tag_name).unwrap_or_default();
        let ret_val = extract_xml_tag_value(&ret_inut_content, tag_name).unwrap_or_default();
        if req_val != ret_val {
            return Err(FiscalError::XmlParsing(format!(
                "Inutilização: <{tag_name}> diverge entre request ({req_val}) e response ({ret_val})"
            )));
        }
    }

    Ok(join_xml(
        &inut_content,
        &ret_inut_content,
        "ProcInutNFe",
        &version,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attach_inutilizacao_empty_request() {
        let err = attach_inutilizacao("", "<retInutNFe/>").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_inutilizacao_empty_response() {
        let err = attach_inutilizacao("<inutNFe/>", "").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_inutilizacao_missing_inut_tag() {
        let err = attach_inutilizacao("<other/>", "<retInutNFe><cStat>102</cStat></retInutNFe>")
            .unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_inutilizacao_missing_ret_tag() {
        let err = attach_inutilizacao(r#"<inutNFe versao="4.00"><data/></inutNFe>"#, "<other/>")
            .unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_inutilizacao_rejected_status() {
        let err = attach_inutilizacao(
            r#"<inutNFe versao="4.00"><data/></inutNFe>"#,
            r#"<retInutNFe><cStat>999</cStat><xMotivo>Erro</xMotivo></retInutNFe>"#,
        )
        .unwrap_err();
        assert!(matches!(err, FiscalError::SefazRejection { .. }));
    }

    #[test]
    fn attach_inutilizacao_success() {
        let result = attach_inutilizacao(
            r#"<inutNFe versao="4.00"><infInut/></inutNFe>"#,
            r#"<retInutNFe><cStat>102</cStat><xMotivo>Inutilizacao de numero homologado</xMotivo></retInutNFe>"#,
        )
        .unwrap();
        assert!(result.contains("<ProcInutNFe"));
        assert!(result.contains("<inutNFe"));
        assert!(result.contains("<retInutNFe>"));
    }

    // ── attach_inutilizacao: version mismatch (line 197) ────────────────

    #[test]
    fn attach_inutilizacao_version_mismatch() {
        let request = concat!(
            r#"<inutNFe versao="4.00"><infInut>"#,
            r#"<tpAmb>2</tpAmb><cUF>35</cUF><ano>26</ano>"#,
            r#"<CNPJ>12345678000199</CNPJ><mod>55</mod><serie>1</serie>"#,
            r#"<nNFIni>1</nNFIni><nNFFin>10</nNFFin>"#,
            r#"</infInut></inutNFe>"#
        );
        let response = concat!(
            r#"<retInutNFe versao="3.10"><infInut>"#,
            r#"<cStat>102</cStat><xMotivo>Inutilizacao homologada</xMotivo>"#,
            r#"<tpAmb>2</tpAmb><cUF>35</cUF><ano>26</ano>"#,
            r#"<CNPJ>12345678000199</CNPJ><mod>55</mod><serie>1</serie>"#,
            r#"<nNFIni>1</nNFIni><nNFFin>10</nNFFin>"#,
            r#"</infInut></retInutNFe>"#
        );
        let err = attach_inutilizacao(request, response).unwrap_err();
        match err {
            FiscalError::XmlParsing(msg) => {
                assert!(
                    msg.contains("versao"),
                    "Expected version mismatch error, got: {msg}"
                );
            }
            other => panic!("Expected XmlParsing, got {:?}", other),
        }
    }

    // ── attach_inutilizacao: tag mismatch (line 217) ────────────────────

    #[test]
    fn attach_inutilizacao_tag_value_mismatch() {
        let request = concat!(
            r#"<inutNFe versao="4.00"><infInut>"#,
            r#"<tpAmb>2</tpAmb><cUF>35</cUF><ano>26</ano>"#,
            r#"<CNPJ>12345678000199</CNPJ><mod>55</mod><serie>1</serie>"#,
            r#"<nNFIni>1</nNFIni><nNFFin>10</nNFFin>"#,
            r#"</infInut></inutNFe>"#
        );
        let response = concat!(
            r#"<retInutNFe versao="4.00"><infInut>"#,
            r#"<cStat>102</cStat><xMotivo>Inutilizacao homologada</xMotivo>"#,
            r#"<tpAmb>2</tpAmb><cUF>35</cUF><ano>26</ano>"#,
            r#"<CNPJ>12345678000199</CNPJ><mod>55</mod><serie>2</serie>"#,
            r#"<nNFIni>1</nNFIni><nNFFin>10</nNFFin>"#,
            r#"</infInut></retInutNFe>"#
        );
        let err = attach_inutilizacao(request, response).unwrap_err();
        match err {
            FiscalError::XmlParsing(msg) => {
                assert!(
                    msg.contains("serie"),
                    "Expected serie mismatch error, got: {msg}"
                );
            }
            other => panic!("Expected XmlParsing, got {:?}", other),
        }
    }
}
