//! SOAP 1.2 envelope construction for SEFAZ NF-e web services.
//!
//! This module is internal to the crate — [`crate::client::SefazClient`]
//! handles envelope wrapping automatically.

use fiscal_core::state_codes::get_state_code;
use fiscal_core::FiscalError;

use crate::services::ServiceMeta;

const SOAP_NS: &str = "http://www.w3.org/2003/05/soap-envelope";
const NFE_PORTAL: &str = "http://www.portalfiscal.inf.br/nfe";

/// Build the SOAP 1.2 envelope that wraps a SEFAZ request body.
///
/// The envelope follows the SEFAZ NF-e 4.00 specification:
///
/// ```xml
/// <soap12:Envelope xmlns:soap12="…/soap-envelope">
///   <soap12:Header>
///     <nfeCabecMsg xmlns="…/wsdl/{operation}">
///       <cUF>{state_ibge_code}</cUF>
///       <versaoDados>{version}</versaoDados>
///     </nfeCabecMsg>
///   </soap12:Header>
///   <soap12:Body>
///     <nfeDadosMsg xmlns="…/wsdl/{operation}">
///       {request_xml}
///     </nfeDadosMsg>
///   </soap12:Body>
/// </soap12:Envelope>
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
    let cuf = get_state_code(uf)?;
    let namespace = format!("{NFE_PORTAL}/wsdl/{}", meta.operation);

    let mut s = String::with_capacity(request_xml.len() + 600);

    // Envelope open
    s.push_str("<soap12:Envelope xmlns:soap12=\"");
    s.push_str(SOAP_NS);
    s.push_str("\">");

    // Header
    s.push_str("<soap12:Header>");
    s.push_str("<nfeCabecMsg xmlns=\"");
    s.push_str(&namespace);
    s.push_str("\"><cUF>");
    s.push_str(cuf);
    s.push_str("</cUF><versaoDados>");
    s.push_str(meta.version);
    s.push_str("</versaoDados></nfeCabecMsg>");
    s.push_str("</soap12:Header>");

    // Body
    s.push_str("<soap12:Body>");
    s.push_str("<nfeDadosMsg xmlns=\"");
    s.push_str(&namespace);
    s.push_str("\">");
    s.push_str(request_xml);
    s.push_str("</nfeDadosMsg>");
    s.push_str("</soap12:Body>");

    // Envelope close
    s.push_str("</soap12:Envelope>");

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
    fn envelope_contains_all_required_elements() {
        let meta = SefazService::StatusServico.meta();
        let body = "<consStatServ><tpAmb>2</tpAmb></consStatServ>";
        let envelope = build_envelope(body, "SP", &meta).unwrap();

        // Outer structure
        assert!(envelope.starts_with("<soap12:Envelope"));
        assert!(envelope.ends_with("</soap12:Envelope>"));
        assert!(envelope.contains("xmlns:soap12=\"http://www.w3.org/2003/05/soap-envelope\""));

        // Header with correct cUF for SP
        assert!(envelope.contains("<cUF>35</cUF>"));
        assert!(envelope.contains("<versaoDados>4.00</versaoDados>"));

        // Body wraps the request XML untouched
        assert!(envelope.contains("<nfeDadosMsg"));
        assert!(envelope.contains(body));
    }

    #[test]
    fn envelope_uses_correct_namespace_per_service() {
        let meta = SefazService::RecepcaoEvento.meta();
        let envelope = build_envelope("<evento/>", "RS", &meta).unwrap();

        let expected_ns = "http://www.portalfiscal.inf.br/nfe/wsdl/NFeRecepcaoEvento4";
        assert!(envelope.contains(expected_ns));
        assert!(envelope.contains("<cUF>43</cUF>")); // RS = 43
    }

    #[test]
    fn envelope_rejects_invalid_state() {
        let meta = SefazService::StatusServico.meta();
        let err = build_envelope("<test/>", "XX", &meta).unwrap_err();
        assert!(matches!(err, FiscalError::InvalidStateCode(_)));
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
