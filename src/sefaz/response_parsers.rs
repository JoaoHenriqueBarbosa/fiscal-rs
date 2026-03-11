use crate::xml_utils::extract_xml_tag_value;
use crate::FiscalError;

/// Parsed result of a SEFAZ NF-e authorization (`retEnviNFe`) response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationResponse {
    /// SEFAZ status code (`cStat`).
    pub status_code: String,
    /// Human-readable status message (`xMotivo`).
    pub status_message: String,
    /// Protocol number (`nProt`), present when the NF-e was processed.
    pub protocol_number: Option<String>,
    /// Raw `<protNFe>...</protNFe>` XML fragment for storage/attachment.
    pub protocol_xml: Option<String>,
    /// Timestamp when SEFAZ received/authorized the document (`dhRecbto`).
    pub authorized_at: Option<String>,
}

/// Parsed result of a SEFAZ service status (`retConsStatServ`) response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusResponse {
    /// SEFAZ status code (`cStat`).
    pub status_code: String,
    /// Human-readable status message (`xMotivo`).
    pub status_message: String,
    /// Average processing time in seconds (`tMed`).
    pub average_time: Option<String>,
}

/// Parsed result of a SEFAZ cancellation event (`retEvento`) response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancellationResponse {
    /// SEFAZ status code (`cStat`).
    pub status_code: String,
    /// Human-readable status message (`xMotivo`).
    pub status_message: String,
    /// Protocol number (`nProt`), present when the event was registered.
    pub protocol_number: Option<String>,
}

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

    // Try to find <protNFe> section first — it carries per-NF-e result
    if let Some(prot_xml) = extract_raw_tag(&body, "protNFe") {
        let inf_prot = extract_inner_content(&prot_xml, "infProt").unwrap_or(&prot_xml);

        let status_code = extract_xml_tag_value(inf_prot, "cStat")
            .ok_or_else(|| FiscalError::XmlParsing("missing <cStat> in <protNFe>".into()))?;
        let status_message =
            extract_xml_tag_value(inf_prot, "xMotivo").unwrap_or_default();
        let protocol_number = extract_xml_tag_value(inf_prot, "nProt");
        let authorized_at = extract_xml_tag_value(inf_prot, "dhRecbto");

        return Ok(AuthorizationResponse {
            status_code,
            status_message,
            protocol_number,
            protocol_xml: Some(prot_xml.to_string()),
            authorized_at,
        });
    }

    // Fallback: batch-level status from <retEnviNFe> directly
    let status_code = extract_xml_tag_value(&body, "cStat")
        .ok_or_else(|| FiscalError::XmlParsing("missing <cStat> in authorization response".into()))?;
    let status_message =
        extract_xml_tag_value(&body, "xMotivo").unwrap_or_else(|| "Unknown".into());

    Ok(AuthorizationResponse {
        status_code,
        status_message,
        protocol_number: None,
        protocol_xml: None,
        authorized_at: None,
    })
}

/// Parse a SEFAZ service status response (`retConsStatServ`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts `cStat`, `xMotivo`, and
/// optionally `tMed` (average processing time).
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element.
pub fn parse_status_response(xml: &str) -> Result<StatusResponse, FiscalError> {
    let body = strip_soap_envelope(xml);

    let status_code = extract_xml_tag_value(&body, "cStat")
        .ok_or_else(|| FiscalError::XmlParsing("missing <cStat> in status response".into()))?;
    let status_message =
        extract_xml_tag_value(&body, "xMotivo").unwrap_or_else(|| "Unknown".into());
    let average_time = extract_xml_tag_value(&body, "tMed");

    Ok(StatusResponse {
        status_code,
        status_message,
        average_time,
    })
}

/// Parse a SEFAZ cancellation event response (`retEvento`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts `cStat`, `xMotivo`, and
/// optionally `nProt` from `<infEvento>`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element.
pub fn parse_cancellation_response(xml: &str) -> Result<CancellationResponse, FiscalError> {
    let body = strip_soap_envelope(xml);

    // Try to narrow into <infEvento> first
    let scope = extract_inner_content(&body, "infEvento").unwrap_or(&body);

    let status_code = extract_xml_tag_value(scope, "cStat")
        .ok_or_else(|| FiscalError::XmlParsing("missing <cStat> in cancellation response".into()))?;
    let status_message =
        extract_xml_tag_value(scope, "xMotivo").unwrap_or_else(|| "Unknown".into());
    let protocol_number = extract_xml_tag_value(scope, "nProt");

    Ok(CancellationResponse {
        status_code,
        status_message,
        protocol_number,
    })
}

// ── Private helpers ─────────────────────────────────────────────────────────

/// Strip SOAP envelope (`<soap:Body>` or `<soapenv:Body>`) if present.
///
/// Also strips common namespace prefixes (`nfe:`, `nfeResultMsg:`) that some
/// SEFAZ endpoints add, so downstream tag extraction works with plain names.
fn strip_soap_envelope(xml: &str) -> String {
    let mut s = xml.to_string();

    // Strip SOAP Body wrapper — look for the innermost Body content.
    // Handles `<soap:Body>`, `<soapenv:Body>`, `<S:Body>`, etc.
    if let Some(body_start) = find_tag_content_start(&s, "Body") {
        if let Some(body_end) = find_closing_tag_pos(&s[body_start..], "Body") {
            s = s[body_start..body_start + body_end].to_string();
        }
    }

    // Remove common namespace prefixes so extract_xml_tag_value works
    // with plain tag names like <cStat> instead of <nfe:cStat>.
    s = remove_ns_prefix(&s, "nfe:");
    s = remove_ns_prefix(&s, "nfeResultMsg:");

    s
}

/// Find the byte offset where the content of a tag starts (after `>`),
/// searching for any namespace-prefixed variant of the tag name.
///
/// For `<soap:Body attr="x">content</soap:Body>`, returns the offset
/// pointing to the start of `content`.
fn find_tag_content_start(xml: &str, local_name: &str) -> Option<usize> {
    // Look for `<...:{local_name}` or `<{local_name}`
    let mut search_from = 0;
    while search_from < xml.len() {
        let lt_pos = xml[search_from..].find('<')? + search_from;
        let gt_pos = xml[lt_pos..].find('>')? + lt_pos;
        let tag_slice = &xml[lt_pos + 1..gt_pos];

        // Skip closing tags and processing instructions
        if tag_slice.starts_with('/') || tag_slice.starts_with('?') {
            search_from = gt_pos + 1;
            continue;
        }

        // Extract the tag name (before any space/attribute)
        let tag_name = tag_slice.split_whitespace().next().unwrap_or(tag_slice);

        // Check if the local part matches (with or without namespace prefix)
        let local_part = if let Some((_prefix, local)) = tag_name.split_once(':') {
            local
        } else {
            tag_name
        };

        if local_part == local_name {
            return Some(gt_pos + 1);
        }

        search_from = gt_pos + 1;
    }

    None
}

/// Find the position of the closing tag `</...:local_name>` or `</local_name>`
/// relative to the start of the given slice.
fn find_closing_tag_pos(xml: &str, local_name: &str) -> Option<usize> {
    // Search for `</{anything}:{local_name}>` or `</{local_name}>`
    let pattern_plain = format!("</{local_name}>");
    if let Some(pos) = xml.find(&pattern_plain) {
        return Some(pos);
    }

    // Search with namespace prefix: `</xxx:{local_name}>`
    let mut search_from = 0;
    while search_from < xml.len() {
        let close_start = xml[search_from..].find("</")? + search_from;
        let close_end = xml[close_start..].find('>')? + close_start;
        let tag_name = &xml[close_start + 2..close_end];
        let local_part = if let Some((_prefix, local)) = tag_name.split_once(':') {
            local
        } else {
            tag_name
        };
        if local_part == local_name {
            return Some(close_start);
        }
        search_from = close_end + 1;
    }

    None
}

/// Remove a namespace prefix from all opening and closing tags.
///
/// E.g. `remove_ns_prefix(xml, "nfe:")` turns `<nfe:cStat>` into `<cStat>`
/// and `</nfe:cStat>` into `</cStat>`.
fn remove_ns_prefix(xml: &str, prefix: &str) -> String {
    let open = format!("<{prefix}");
    let close = format!("</{prefix}");
    xml.replace(&open, "<").replace(&close, "</")
}

/// Extract the raw content between the opening and closing of a tag,
/// including any namespace-prefixed variant. Returns a slice of the
/// original string covering from `<tag ...>` to `</tag>` inclusive.
fn extract_raw_tag<'a>(xml: &'a str, local_name: &str) -> Option<String> {
    // Find opening tag
    let start = find_opening_tag_pos(xml, local_name)?;
    let after_start = xml[start..].find('>')? + start + 1;

    // Find matching closing tag from after the opening tag
    let inner = &xml[after_start..];
    let close_rel = find_closing_tag_pos(inner, local_name)?;
    let close_tag_end = inner[close_rel..].find('>')? + close_rel + 1;

    Some(xml[start..after_start + close_tag_end].to_string())
}

/// Find the byte offset of the opening `<` for a tag with the given local name.
fn find_opening_tag_pos(xml: &str, local_name: &str) -> Option<usize> {
    let mut search_from = 0;
    while search_from < xml.len() {
        let lt_pos = xml[search_from..].find('<')? + search_from;
        let gt_pos = xml[lt_pos..].find('>')? + lt_pos;
        let tag_slice = &xml[lt_pos + 1..gt_pos];

        if tag_slice.starts_with('/') || tag_slice.starts_with('?') {
            search_from = gt_pos + 1;
            continue;
        }

        let tag_name = tag_slice.split_whitespace().next().unwrap_or(tag_slice);
        let local_part = if let Some((_prefix, local)) = tag_name.split_once(':') {
            local
        } else {
            tag_name
        };

        if local_part == local_name {
            return Some(lt_pos);
        }

        search_from = gt_pos + 1;
    }

    None
}

/// Extract the inner text content of a tag, returning a slice into the
/// original string. Only finds the first occurrence.
fn extract_inner_content<'a>(xml: &'a str, local_name: &str) -> Option<&'a str> {
    let content_start = find_tag_content_start(xml, local_name)?;
    let rest = &xml[content_start..];
    let end = find_closing_tag_pos(rest, local_name)?;
    Some(&rest[..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_status_response ───────────────────────────────────────

    #[test]
    fn parses_plain_status_response() {
        let xml = "<retConsStatServ><cStat>107</cStat>\
                    <xMotivo>Servico em Operacao</xMotivo>\
                    <tMed>1</tMed></retConsStatServ>";
        let resp = parse_status_response(xml).unwrap();
        assert_eq!(resp.status_code, "107");
        assert_eq!(resp.status_message, "Servico em Operacao");
        assert_eq!(resp.average_time.as_deref(), Some("1"));
    }

    #[test]
    fn parses_status_response_without_tmed() {
        let xml = "<retConsStatServ><cStat>107</cStat>\
                    <xMotivo>Servico em Operacao</xMotivo></retConsStatServ>";
        let resp = parse_status_response(xml).unwrap();
        assert_eq!(resp.status_code, "107");
        assert_eq!(resp.average_time, None);
    }

    #[test]
    fn parses_soap_wrapped_status_response() {
        let xml = r#"<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
            <soap:Body>
                <nfeResultMsg:nfeStatusServicoNF2Result xmlns:nfeResultMsg="http://www.portalfiscal.inf.br/nfe/wsdl/NFeStatusServico4">
                    <nfe:retConsStatServ xmlns:nfe="http://www.portalfiscal.inf.br/nfe">
                        <nfe:cStat>107</nfe:cStat>
                        <nfe:xMotivo>Servico em Operacao</nfe:xMotivo>
                        <nfe:tMed>2</nfe:tMed>
                    </nfe:retConsStatServ>
                </nfeResultMsg:nfeStatusServicoNF2Result>
            </soap:Body>
        </soap:Envelope>"#;
        let resp = parse_status_response(xml).unwrap();
        assert_eq!(resp.status_code, "107");
        assert_eq!(resp.status_message, "Servico em Operacao");
        assert_eq!(resp.average_time.as_deref(), Some("2"));
    }

    #[test]
    fn status_response_rejects_malformed_xml() {
        let xml = "<retConsStatServ><xMotivo>ok</xMotivo></retConsStatServ>";
        let err = parse_status_response(xml).unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

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

    // ── parse_cancellation_response ─────────────────────────────────

    #[test]
    fn parses_cancellation_response() {
        let xml = concat!(
            "<retEvento><infEvento>",
            "<cStat>135</cStat>",
            "<xMotivo>Evento registrado e vinculado a NF-e</xMotivo>",
            "<nProt>135220000009999</nProt>",
            "</infEvento></retEvento>"
        );
        let resp = parse_cancellation_response(xml).unwrap();
        assert_eq!(resp.status_code, "135");
        assert_eq!(
            resp.status_message,
            "Evento registrado e vinculado a NF-e"
        );
        assert_eq!(resp.protocol_number.as_deref(), Some("135220000009999"));
    }

    #[test]
    fn cancellation_rejects_malformed_xml() {
        let err = parse_cancellation_response("<retEvento>nothing</retEvento>").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn parses_soap_wrapped_cancellation() {
        let xml = r#"<soap:Envelope>
            <soap:Body>
                <nfe:retEvento xmlns:nfe="http://www.portalfiscal.inf.br/nfe">
                    <nfe:infEvento>
                        <nfe:cStat>135</nfe:cStat>
                        <nfe:xMotivo>Evento registrado</nfe:xMotivo>
                        <nfe:nProt>141240000099999</nfe:nProt>
                    </nfe:infEvento>
                </nfe:retEvento>
            </soap:Body>
        </soap:Envelope>"#;
        let resp = parse_cancellation_response(xml).unwrap();
        assert_eq!(resp.status_code, "135");
        assert_eq!(resp.protocol_number.as_deref(), Some("141240000099999"));
    }
}
