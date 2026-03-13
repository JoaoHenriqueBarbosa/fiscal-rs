//! Functions for attaching SEFAZ authorization protocols to signed XML documents.
//!
//! Each submodule handles one type of protocol attachment:
//! - [`protocol`] — NFe authorization (`<nfeProc>`)
//! - [`inutilizacao`] — Number voiding (`<ProcInutNFe>`)
//! - [`event`] — Event protocol (`<procEventoNFe>`)
//! - [`b2b`] — B2B financial wrapper (`<nfeProcB2B>`)
//! - [`cancellation`] — Cancellation event attachment
//! - [`helpers`] — Internal XML parsing utilities

mod b2b;
mod cancellation;
mod event;
mod helpers;
mod inutilizacao;
mod protocol;

pub use b2b::attach_b2b;
pub use cancellation::attach_cancellation;
pub use event::attach_event_protocol;
pub use inutilizacao::attach_inutilizacao;
pub use protocol::attach_protocol;

use crate::FiscalError;
use helpers::contains_xml_tag;

// ── Unified routing (mirrors PHP Complements::toAuthorize) ──────────────────

/// Detect the document type from raw XML and dispatch to the correct
/// protocol-attachment function.
///
/// This mirrors the PHP `Complements::toAuthorize()` method, which uses
/// `Standardize::whichIs()` internally. The detection logic checks for
/// the same root tags in the same priority order as the PHP implementation:
///
/// | Detected tag    | Dispatches to                  |
/// |-----------------|-------------------------------|
/// | `NFe`           | [`attach_protocol`]           |
/// | `envEvento`     | [`attach_event_protocol`]     |
/// | `inutNFe`       | [`attach_inutilizacao`]       |
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if:
/// - Either input is empty
/// - The request XML does not match any of the known document types
/// - The delegated function returns an error
pub fn to_authorize(request_xml: &str, response_xml: &str) -> Result<String, FiscalError> {
    if request_xml.is_empty() {
        return Err(FiscalError::XmlParsing(
            "Erro ao protocolar: o XML a protocolar está vazio.".into(),
        ));
    }
    if response_xml.is_empty() {
        return Err(FiscalError::XmlParsing(
            "Erro ao protocolar: o retorno da SEFAZ está vazio.".into(),
        ));
    }

    // Detect using the same tag order as PHP Standardize::whichIs() + the
    // ucfirst() / if-check in toAuthorize().
    // PHP checks: whichIs() returns the root tag name from rootTagList,
    // then toAuthorize() accepts only "NFe", "EnvEvento", "InutNFe".
    // We search for these tags in the XML content:
    if contains_xml_tag(request_xml, "NFe") {
        attach_protocol(request_xml, response_xml)
    } else if contains_xml_tag(request_xml, "envEvento") {
        attach_event_protocol(request_xml, response_xml)
    } else if contains_xml_tag(request_xml, "inutNFe") {
        attach_inutilizacao(request_xml, response_xml)
    } else {
        Err(FiscalError::XmlParsing(
            "Tipo de documento não reconhecido para protocolação".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_authorize_empty_request_returns_error() {
        let err = to_authorize("", "<retEnviNFe/>").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn to_authorize_empty_response_returns_error() {
        let err = to_authorize("<NFe/>", "").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn to_authorize_unrecognized_document_returns_error() {
        let err = to_authorize("<other>data</other>", "<response/>").unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("não reconhecido"),
            "should mention unrecognized type: {msg}"
        );
    }

    // ── to_authorize: NFe path (line 428) ───────────────────────────────

    #[test]
    fn to_authorize_dispatches_nfe() {
        let request = concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe35260112345678000199550010000000011123456780">"#,
            r#"<DigestValue>abc</DigestValue>"#,
            r#"</infNFe></NFe>"#
        );
        let response = concat!(
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<cStat>100</cStat><xMotivo>OK</xMotivo>"#,
            r#"<digVal>abc</digVal>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"</infProt></protNFe>"#
        );
        let result = to_authorize(request, response).unwrap();
        assert!(result.contains("<nfeProc"));
    }

    // ── to_authorize: envEvento path (line 430) ─────────────────────────

    #[test]
    fn to_authorize_dispatches_env_evento() {
        let request = concat!(
            r#"<envEvento><idLote>1</idLote>"#,
            r#"<evento versao="1.00"><infEvento>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></evento></envEvento>"#
        );
        let response = concat!(
            r#"<retEnvEvento><idLote>1</idLote>"#,
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat><xMotivo>OK</xMotivo>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></retEvento></retEnvEvento>"#
        );
        let result = to_authorize(request, response).unwrap();
        assert!(result.contains("<procEventoNFe"));
    }

    // ── to_authorize: inutNFe path (line 432) ───────────────────────────

    #[test]
    fn to_authorize_dispatches_inut_nfe() {
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
            r#"<CNPJ>12345678000199</CNPJ><mod>55</mod><serie>1</serie>"#,
            r#"<nNFIni>1</nNFIni><nNFFin>10</nNFFin>"#,
            r#"</infInut></retInutNFe>"#
        );
        let result = to_authorize(request, response).unwrap();
        assert!(result.contains("<ProcInutNFe"));
    }
}
