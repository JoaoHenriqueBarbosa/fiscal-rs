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

/// Build the SOAP 1.2 envelope with `<nfeCabecMsg>` inside `<soap:Header>`.
///
/// Used by services that still require the legacy SOAP header, notably
/// `ConsultaCadastro` (version 2.00).  The header contains `<cUF>` and
/// `<versaoDados>` and matches the PHP sped-nfe / sped-common format:
///
/// ```xml
/// <soap:Envelope xmlns:xsi="…" xmlns:xsd="…" xmlns:soap="…/soap-envelope">
///   <soap:Header>
///     <nfeCabecMsg xmlns="…/wsdl/{operation}">
///       <cUF>{cuf}</cUF>
///       <versaoDados>{version}</versaoDados>
///     </nfeCabecMsg>
///   </soap:Header>
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
pub(crate) fn build_envelope_with_header(
    request_xml: &str,
    uf: &str,
    meta: &ServiceMeta,
) -> Result<String, FiscalError> {
    let cuf = get_state_code(uf)?;
    let namespace = format!("{NFE_PORTAL}/wsdl/{}", meta.operation);

    let mut s = String::with_capacity(request_xml.len() + 600);

    // Envelope open — matches PHP sped-nfe exactly
    s.push_str("<soap:Envelope xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soap=\"");
    s.push_str(SOAP_NS);
    s.push_str("\">");

    // Header — matches PHP sped-common mountSoapHeaders():
    // <soap:Header><nfeCabecMsg xmlns="{ns}"><cUF>{cuf}</cUF><versaoDados>{ver}</versaoDados></nfeCabecMsg></soap:Header>
    s.push_str("<soap:Header>");
    s.push_str("<nfeCabecMsg xmlns=\"");
    s.push_str(&namespace);
    s.push_str("\">");
    s.push_str("<cUF>");
    let cuf_str = cuf.to_string();
    s.push_str(&cuf_str);
    s.push_str("</cUF>");
    s.push_str("<versaoDados>");
    s.push_str(meta.version);
    s.push_str("</versaoDados>");
    s.push_str("</nfeCabecMsg>");
    s.push_str("</soap:Header>");

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

/// Build the SOAP 1.2 envelope with gzip-compressed body (`nfeDadosMsgZip`).
///
/// Matches PHP sped-nfe behavior: `base64(gzencode($request, 9, FORCE_GZIP))`.
/// The request XML is gzip-compressed (level 9) and base64-encoded, then
/// wrapped in `<nfeDadosMsgZip>` instead of `<nfeDadosMsg>`.
///
/// # Errors
///
/// Returns [`FiscalError::InvalidStateCode`] if `uf` is not a valid
/// Brazilian state abbreviation.
///
/// Returns [`FiscalError::XmlParsing`] if gzip compression fails.
pub(crate) fn build_envelope_compressed(
    request_xml: &str,
    uf: &str,
    meta: &ServiceMeta,
) -> Result<String, FiscalError> {
    let _cuf = get_state_code(uf)?; // validates state code
    let namespace = format!("{NFE_PORTAL}/wsdl/{}", meta.operation);

    // Gzip compress and base64 encode (matching PHP: gzencode level 9)
    use base64::Engine as _;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(request_xml.as_bytes())
        .map_err(|e| FiscalError::XmlParsing(format!("Gzip compression failed: {e}")))?;
    let compressed = encoder
        .finish()
        .map_err(|e| FiscalError::XmlParsing(format!("Gzip compression failed: {e}")))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&compressed);

    let mut s = String::with_capacity(b64.len() + 400);

    // Envelope open
    s.push_str("<soap:Envelope xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soap=\"");
    s.push_str(SOAP_NS);
    s.push_str("\">");

    // Body with nfeDadosMsgZip
    s.push_str("<soap:Body>");
    s.push_str("<nfeDadosMsgZip xmlns=\"");
    s.push_str(&namespace);
    s.push_str("\">");
    s.push_str(&b64);
    s.push_str("</nfeDadosMsgZip>");
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

    #[test]
    fn compressed_envelope_uses_nfe_dados_msg_zip() {
        let meta = SefazService::Autorizacao.meta();
        let body = "<enviNFe><idLote>1</idLote></enviNFe>";
        let envelope = build_envelope_compressed(body, "SP", &meta).unwrap();

        // Must use nfeDadosMsgZip instead of nfeDadosMsg
        assert!(
            envelope.contains("<nfeDadosMsgZip"),
            "Compressed envelope must use nfeDadosMsgZip"
        );
        assert!(
            envelope.contains("</nfeDadosMsgZip>"),
            "Compressed envelope must close nfeDadosMsgZip"
        );
        assert!(
            !envelope.contains("<nfeDadosMsg "),
            "Compressed envelope must NOT contain nfeDadosMsg (uncompressed)"
        );

        // Must have the correct SOAP structure
        assert!(envelope.starts_with("<soap:Envelope"));
        assert!(envelope.ends_with("</soap:Envelope>"));
        assert!(envelope.contains("<soap:Body>"));
        assert!(envelope.contains("</soap:Body>"));

        // Must contain the namespace
        let expected_ns = "http://www.portalfiscal.inf.br/nfe/wsdl/NFeAutorizacao4";
        assert!(
            envelope.contains(expected_ns),
            "Compressed envelope must contain the service namespace"
        );
    }

    #[test]
    fn compressed_envelope_content_is_valid_base64_gzip() {
        use base64::Engine as _;
        use flate2::read::GzDecoder;
        use std::io::Read;

        let meta = SefazService::Autorizacao.meta();
        let body = "<enviNFe><idLote>42</idLote><indSinc>1</indSinc></enviNFe>";
        let envelope = build_envelope_compressed(body, "SP", &meta).unwrap();

        // Extract the base64 content from nfeDadosMsgZip
        let start_tag =
            "<nfeDadosMsgZip xmlns=\"http://www.portalfiscal.inf.br/nfe/wsdl/NFeAutorizacao4\">";
        let start = envelope.find(start_tag).unwrap() + start_tag.len();
        let end = envelope.find("</nfeDadosMsgZip>").unwrap();
        let b64_content = &envelope[start..end];

        // Decode base64
        let compressed = base64::engine::general_purpose::STANDARD
            .decode(b64_content)
            .expect("Content must be valid base64");

        // Decompress gzip
        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut decompressed = String::new();
        decoder
            .read_to_string(&mut decompressed)
            .expect("Content must be valid gzip");

        // Verify the decompressed content matches the original
        assert_eq!(
            decompressed, body,
            "Decompressed content must match original XML"
        );
    }

    #[test]
    fn compressed_envelope_rejects_invalid_state() {
        let meta = SefazService::Autorizacao.meta();
        let err = build_envelope_compressed("<test/>", "XX", &meta).unwrap_err();
        assert!(matches!(err, FiscalError::InvalidStateCode(_)));
    }

    // ── nfeCabecMsg header tests ────────────────────────────────────

    #[test]
    fn envelope_with_header_has_nfe_cabec_msg() {
        let meta = SefazService::ConsultaCadastro.meta();
        let body = "<ConsCad><infCons><xServ>CONS-CAD</xServ></infCons></ConsCad>";
        let envelope = build_envelope_with_header(body, "SP", &meta).unwrap();

        // Must contain soap:Header with nfeCabecMsg
        assert!(envelope.contains("<soap:Header>"), "Must have soap:Header");
        assert!(
            envelope.contains("</soap:Header>"),
            "Must close soap:Header"
        );
        assert!(
            envelope.contains("<nfeCabecMsg"),
            "Must have nfeCabecMsg element"
        );
        assert!(
            envelope.contains("<cUF>35</cUF>"),
            "Must contain cUF for SP (35)"
        );
        assert!(
            envelope.contains("<versaoDados>2.00</versaoDados>"),
            "Must contain versaoDados matching service version"
        );
    }

    #[test]
    fn envelope_with_header_exact_string_matches_php() {
        let meta = SefazService::ConsultaCadastro.meta();
        let body = "<ConsCad xmlns=\"http://www.portalfiscal.inf.br/nfe\" versao=\"2.00\"><infCons><xServ>CONS-CAD</xServ><UF>SP</UF><CNPJ>11222333000181</CNPJ></infCons></ConsCad>";
        let envelope = build_envelope_with_header(body, "SP", &meta).unwrap();

        let expected = concat!(
            "<soap:Envelope xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" ",
            "xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" ",
            "xmlns:soap=\"http://www.w3.org/2003/05/soap-envelope\">",
            "<soap:Header>",
            "<nfeCabecMsg xmlns=\"http://www.portalfiscal.inf.br/nfe/wsdl/CadConsultaCadastro4\">",
            "<cUF>35</cUF>",
            "<versaoDados>2.00</versaoDados>",
            "</nfeCabecMsg>",
            "</soap:Header>",
            "<soap:Body>",
            "<nfeDadosMsg xmlns=\"http://www.portalfiscal.inf.br/nfe/wsdl/CadConsultaCadastro4\">",
            "<ConsCad xmlns=\"http://www.portalfiscal.inf.br/nfe\" versao=\"2.00\">",
            "<infCons><xServ>CONS-CAD</xServ><UF>SP</UF><CNPJ>11222333000181</CNPJ></infCons>",
            "</ConsCad>",
            "</nfeDadosMsg>",
            "</soap:Body>",
            "</soap:Envelope>",
        );
        assert_eq!(
            envelope, expected,
            "Envelope with header must match PHP sped-nfe format exactly"
        );
    }

    #[test]
    fn envelope_with_header_preserves_body_untouched() {
        let meta = SefazService::ConsultaCadastro.meta();
        let body = "<ConsCad><infCons><xServ>CONS-CAD</xServ><UF>MT</UF><CNPJ>99888777000166</CNPJ></infCons></ConsCad>";
        let envelope = build_envelope_with_header(body, "MT", &meta).unwrap();

        // Body must be preserved exactly
        assert!(envelope.contains(body));
        // MT = cUF 51
        assert!(
            envelope.contains("<cUF>51</cUF>"),
            "Must contain cUF for MT (51)"
        );
    }

    #[test]
    fn envelope_with_header_rejects_invalid_state() {
        let meta = SefazService::ConsultaCadastro.meta();
        let err = build_envelope_with_header("<test/>", "XX", &meta).unwrap_err();
        assert!(matches!(err, FiscalError::InvalidStateCode(_)));
    }

    #[test]
    fn envelope_with_header_header_before_body() {
        let meta = SefazService::ConsultaCadastro.meta();
        let body = "<ConsCad/>";
        let envelope = build_envelope_with_header(body, "RS", &meta).unwrap();

        let header_start = envelope
            .find("<soap:Header>")
            .expect("must have soap:Header");
        let header_end = envelope
            .find("</soap:Header>")
            .expect("must close soap:Header");
        let body_start = envelope.find("<soap:Body>").expect("must have soap:Body");

        assert!(
            header_start < header_end,
            "Header must open before it closes"
        );
        assert!(
            header_end < body_start,
            "Header must close before Body opens"
        );
    }

    #[test]
    fn regular_envelope_still_omits_header() {
        // Verify that the standard build_envelope still has no header
        let meta = SefazService::StatusServico.meta();
        let body = "<consStatServ/>";
        let envelope = build_envelope(body, "SP", &meta).unwrap();
        assert!(
            !envelope.contains("<soap:Header"),
            "Regular envelope must not have soap:Header"
        );
        assert!(
            !envelope.contains("nfeCabecMsg"),
            "Regular envelope must not have nfeCabecMsg"
        );
    }
}
