//! SOAP 1.2 envelope construction for SEFAZ NF-e web services.
//!
//! This module is internal to the crate — [`crate::client::SefazClient`]
//! handles envelope wrapping automatically.

use fiscal_core::FiscalError;
use fiscal_core::state_codes::get_state_code;

use crate::services::ServiceMeta;

const SOAP_NS: &str = "http://www.w3.org/2003/05/soap-envelope";
const NFE_PORTAL: &str = "http://www.portalfiscal.inf.br/nfe";

/// Build the SOAP 1.2 envelope that wraps a SEFAZ request body.
///
/// The envelope matches PHP sped-nfe format exactly:
///
/// ```xml
/// <soap:Envelope xmlns:xsi="…" xmlns:xsd="…" xmlns:soap="…/soap-envelope">
///   <soap:Body>
///     <nfeDadosMsg xmlns="…/wsdl/{operation}">
///       {request_xml}
///     </nfeDadosMsg>
///   </soap:Body>
/// </soap:Envelope>
/// ```
///
/// # Errors
///
/// Returns [`FiscalError::InvalidStateCode`] if `uf` is not a valid
/// Brazilian state abbreviation.
pub(crate) fn build_envelope(
    request_xml: &str,
    uf: &str,
    meta: &ServiceMeta,
) -> Result<String, FiscalError> {
    let _cuf = get_state_code(uf)?; // validates state code
    let namespace = format!("{NFE_PORTAL}/wsdl/{}", meta.operation);

    let mut s = String::with_capacity(request_xml.len() + 400);

    // Envelope open — matches PHP sped-nfe exactly:
    // <soap:Envelope xmlns:xsi="..." xmlns:xsd="..." xmlns:soap="...">
    s.push_str("<soap:Envelope xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soap=\"");
    s.push_str(SOAP_NS);
    s.push_str("\">");

    // No <soap:Header> — PHP omits it entirely in NF-e 4.00

    // Body
    s.push_str("<soap:Body>");
    s.push_str("<nfeDadosMsg xmlns=\"");
    s.push_str(&namespace);
    s.push_str("\">");
    s.push_str(request_xml);
    s.push_str("</nfeDadosMsg>");
    s.push_str("</soap:Body>");

    // Envelope close
    s.push_str("</soap:Envelope>");

    Ok(s)
}

/// Build the SOAP 1.2 envelope for DistDFe with the special wrapper.
///
/// The DistDFe service requires an extra `<nfeDistDFeInteresse>` wrapper
/// around `<nfeDadosMsg>`, matching the PHP sped-nfe format:
///
/// ```xml
/// <soap:Envelope ...>
///   <soap:Body>
///     <nfeDistDFeInteresse xmlns="…/wsdl/NFeDistribuicaoDFe">
///       <nfeDadosMsg xmlns="…/wsdl/NFeDistribuicaoDFe">
///         {request_xml}
///       </nfeDadosMsg>
///     </nfeDistDFeInteresse>
///   </soap:Body>
/// </soap:Envelope>
/// ```
///
/// # Errors
///
/// Returns [`FiscalError::InvalidStateCode`] if `uf` is not a valid
/// Brazilian state abbreviation or special code (e.g. `AN`).
pub(crate) fn build_envelope_dist_dfe(
    request_xml: &str,
    uf: &str,
    meta: &ServiceMeta,
) -> Result<String, FiscalError> {
    let _cuf = get_state_code(uf)?; // validates state code
    let namespace = format!("{NFE_PORTAL}/wsdl/{}", meta.operation);

    let mut s = String::with_capacity(request_xml.len() + 600);

    // Envelope open
    s.push_str("<soap:Envelope xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soap=\"");
    s.push_str(SOAP_NS);
    s.push_str("\">");

    // Body with nfeDistDFeInteresse wrapper
    s.push_str("<soap:Body>");
    s.push_str("<nfeDistDFeInteresse xmlns=\"");
    s.push_str(&namespace);
    s.push_str("\">");
    s.push_str("<nfeDadosMsg xmlns=\"");
    s.push_str(&namespace);
    s.push_str("\">");
    s.push_str(request_xml);
    s.push_str("</nfeDadosMsg>");
    s.push_str("</nfeDistDFeInteresse>");
    s.push_str("</soap:Body>");

    // Envelope close
    s.push_str("</soap:Envelope>");

    Ok(s)
}

/// Build the SOAP Action URI for an HTTP `Content-Type` header.
///
/// Format: `http://www.portalfiscal.inf.br/nfe/wsdl/{operation}/{method}`
pub(crate) fn build_action(meta: &ServiceMeta) -> String {
    format!("{NFE_PORTAL}/wsdl/{}/{}", meta.operation, meta.method)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::SefazService;

    #[test]
    fn envelope_matches_php_sped_nfe_format() {
        let meta = SefazService::StatusServico.meta();
        let body = "<consStatServ><tpAmb>2</tpAmb></consStatServ>";
        let envelope = build_envelope(body, "SP", &meta).unwrap();

        // Must use soap: prefix (not soap12:) — matches PHP sped-nfe
        assert!(envelope.starts_with("<soap:Envelope"));
        assert!(envelope.ends_with("</soap:Envelope>"));

        // Must include xsi, xsd, and soap namespace declarations — matches PHP
        assert!(envelope.contains("xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\""));
        assert!(envelope.contains("xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\""));
        assert!(envelope.contains("xmlns:soap=\"http://www.w3.org/2003/05/soap-envelope\""));

        // No <soap:Header> at all — PHP omits it in NF-e 4.00
        assert!(
            !envelope.contains("<soap:Header"),
            "Header must be omitted like PHP"
        );

        // Body wraps the request XML untouched
        assert!(envelope.contains("<soap:Body>"));
        assert!(envelope.contains("</soap:Body>"));
        assert!(envelope.contains("<nfeDadosMsg"));
        assert!(envelope.contains(body));
    }

    #[test]
    fn envelope_uses_correct_namespace_per_service() {
        let meta = SefazService::RecepcaoEvento.meta();
        let envelope = build_envelope("<evento/>", "RS", &meta).unwrap();

        let expected_ns = "http://www.portalfiscal.inf.br/nfe/wsdl/NFeRecepcaoEvento4";
        assert!(envelope.contains(expected_ns));
    }

    #[test]
    fn envelope_rejects_invalid_state() {
        let meta = SefazService::StatusServico.meta();
        let err = build_envelope("<test/>", "XX", &meta).unwrap_err();
        assert!(matches!(err, FiscalError::InvalidStateCode(_)));
    }

    #[test]
    fn envelope_exact_string_matches_php() {
        // Verify the exact envelope string to catch any subtle formatting differences
        let meta = SefazService::StatusServico.meta();
        let body = "<consStatServ xmlns=\"http://www.portalfiscal.inf.br/nfe\" versao=\"4.00\"><tpAmb>2</tpAmb><cUF>35</cUF><xServ>STATUS</xServ></consStatServ>";
        let envelope = build_envelope(body, "SP", &meta).unwrap();

        let expected = concat!(
            "<soap:Envelope xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" ",
            "xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" ",
            "xmlns:soap=\"http://www.w3.org/2003/05/soap-envelope\">",
            "<soap:Body>",
            "<nfeDadosMsg xmlns=\"http://www.portalfiscal.inf.br/nfe/wsdl/NFeStatusServico4\">",
            "<consStatServ xmlns=\"http://www.portalfiscal.inf.br/nfe\" versao=\"4.00\">",
            "<tpAmb>2</tpAmb><cUF>35</cUF><xServ>STATUS</xServ></consStatServ>",
            "</nfeDadosMsg>",
            "</soap:Body>",
            "</soap:Envelope>",
        );
        assert_eq!(
            envelope, expected,
            "Envelope must match PHP sped-nfe format exactly"
        );
    }

    #[test]
    fn action_uri_format() {
        let meta = SefazService::StatusServico.meta();
        assert_eq!(
            build_action(&meta),
            "http://www.portalfiscal.inf.br/nfe/wsdl/NFeStatusServico4/nfeStatusServicoNF"
        );

        let meta = SefazService::Autorizacao.meta();
        assert_eq!(
            build_action(&meta),
            "http://www.portalfiscal.inf.br/nfe/wsdl/NFeAutorizacao4/nfeAutorizacaoLote"
        );
    }

    #[test]
    fn dist_dfe_envelope_has_nfe_dist_dfe_interesse_wrapper() {
        let meta = SefazService::DistribuicaoDFe.meta();
        let body = "<distDFeInt><tpAmb>2</tpAmb></distDFeInt>";
        let envelope = build_envelope_dist_dfe(body, "AN", &meta).unwrap();

        // Must contain the extra nfeDistDFeInteresse wrapper
        assert!(
            envelope.contains("<nfeDistDFeInteresse xmlns=\"http://www.portalfiscal.inf.br/nfe/wsdl/NFeDistribuicaoDFe\">"),
            "Must have nfeDistDFeInteresse wrapper"
        );
        assert!(
            envelope.contains("</nfeDistDFeInteresse>"),
            "Must close nfeDistDFeInteresse"
        );

        // nfeDadosMsg must be inside nfeDistDFeInteresse
        let interesse_start = envelope.find("<nfeDistDFeInteresse").unwrap();
        let dados_start = envelope.find("<nfeDadosMsg").unwrap();
        let dados_end = envelope.find("</nfeDadosMsg>").unwrap();
        let interesse_end = envelope.find("</nfeDistDFeInteresse>").unwrap();

        assert!(
            interesse_start < dados_start,
            "nfeDistDFeInteresse must come before nfeDadosMsg"
        );
        assert!(
            dados_end < interesse_end,
            "nfeDadosMsg must close before nfeDistDFeInteresse"
        );

        // The inner content must be preserved
        assert!(envelope.contains(body));
    }

    #[test]
    fn dist_dfe_envelope_exact_structure() {
        let meta = SefazService::DistribuicaoDFe.meta();
        let body = "<distDFeInt>test</distDFeInt>";
        let envelope = build_envelope_dist_dfe(body, "AN", &meta).unwrap();

        let expected = concat!(
            "<soap:Envelope xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" ",
            "xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" ",
            "xmlns:soap=\"http://www.w3.org/2003/05/soap-envelope\">",
            "<soap:Body>",
            "<nfeDistDFeInteresse xmlns=\"http://www.portalfiscal.inf.br/nfe/wsdl/NFeDistribuicaoDFe\">",
            "<nfeDadosMsg xmlns=\"http://www.portalfiscal.inf.br/nfe/wsdl/NFeDistribuicaoDFe\">",
            "<distDFeInt>test</distDFeInt>",
            "</nfeDadosMsg>",
            "</nfeDistDFeInteresse>",
            "</soap:Body>",
            "</soap:Envelope>",
        );
        assert_eq!(
            envelope, expected,
            "DistDFe envelope must match PHP sped-nfe format exactly"
        );
    }
}
