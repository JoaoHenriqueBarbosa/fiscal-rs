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
}
