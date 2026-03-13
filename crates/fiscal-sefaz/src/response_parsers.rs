use fiscal_core::FiscalError;
use fiscal_core::xml_utils::extract_xml_tag_value;

/// Parsed result of a SEFAZ NF-e authorization (`retEnviNFe`) response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
pub struct CancellationResponse {
    /// SEFAZ status code (`cStat`).
    pub status_code: String,
    /// Human-readable status message (`xMotivo`).
    pub status_message: String,
    /// Protocol number (`nProt`), present when the event was registered.
    pub protocol_number: Option<String>,
}

/// Parsed result of a SEFAZ DistDFe (`retDistDFeInt`) response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DistDFeResponse {
    /// SEFAZ status code (`cStat`).
    pub status_code: String,
    /// Human-readable status message (`xMotivo`).
    pub status_message: String,
    /// Last NSU returned (`ultNSU`).
    pub ult_nsu: Option<String>,
    /// Maximum NSU available (`maxNSU`).
    pub max_nsu: Option<String>,
    /// Raw XML of individual `<docZip>` or `<loteDistDFeInt>` entries.
    pub raw_xml: String,
}

/// Parsed result of a SEFAZ inutilização (`retInutNFe`) response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct InutilizacaoResponse {
    /// Environment type (`tpAmb`): 1 = Produção, 2 = Homologação.
    pub tp_amb: String,
    /// SEFAZ application version (`verAplic`).
    pub ver_aplic: String,
    /// SEFAZ status code (`cStat`).
    pub c_stat: String,
    /// Human-readable status message (`xMotivo`).
    pub x_motivo: String,
    /// UF code (`cUF`).
    pub c_uf: String,
    /// Year of the inutilização (`ano`).
    pub ano: String,
    /// CNPJ of the emitter.
    pub cnpj: String,
    /// Fiscal document model (`mod`).
    pub modelo: String,
    /// Series number (`serie`).
    pub serie: String,
    /// Initial NF-e number (`nNFIni`).
    pub n_nf_ini: String,
    /// Final NF-e number (`nNFFin`).
    pub n_nf_fin: String,
    /// Timestamp when SEFAZ received the request (`dhRecbto`).
    pub dh_recbto: Option<String>,
    /// Protocol number (`nProt`).
    pub n_prot: Option<String>,
}

/// A single CSC token (id + secret) from the NFC-e CSC administration response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CscToken {
    /// CSC identifier (`idCsc`).
    pub id_csc: String,
    /// CSC secret value (`CSC`).
    pub csc: String,
}

/// Parsed result of a SEFAZ NFC-e CSC administration (`retAdmCscNFCe`) response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CscResponse {
    /// Environment type (`tpAmb`): 1 = production, 2 = homologation.
    pub tp_amb: String,
    /// Operation indicator (`indOp`): 1 = consulta, 2 = novo, 3 = revogar.
    pub ind_op: String,
    /// SEFAZ status code (`cStat`).
    pub c_stat: String,
    /// Human-readable status message (`xMotivo`).
    pub x_motivo: String,
    /// Active CSC tokens (`idCsc` + `CSC` pairs), present for indOp 1 or 2.
    pub tokens: Vec<CscToken>,
}

/// Parsed result of a SEFAZ Cadastro (`retConsCad`) response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CadastroResponse {
    /// SEFAZ status code (`cStat`).
    pub status_code: String,
    /// Human-readable status message (`xMotivo`).
    pub status_message: String,
    /// Raw inner XML of `<infCons>` for detailed parsing.
    pub raw_xml: String,
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
        let status_message = extract_xml_tag_value(inf_prot, "xMotivo").unwrap_or_default();
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
    let status_code = extract_xml_tag_value(&body, "cStat").ok_or_else(|| {
        FiscalError::XmlParsing("missing <cStat> in authorization response".into())
    })?;
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

    let status_code = extract_xml_tag_value(scope, "cStat").ok_or_else(|| {
        FiscalError::XmlParsing("missing <cStat> in cancellation response".into())
    })?;
    let status_message =
        extract_xml_tag_value(scope, "xMotivo").unwrap_or_else(|| "Unknown".into());
    let protocol_number = extract_xml_tag_value(scope, "nProt");

    Ok(CancellationResponse {
        status_code,
        status_message,
        protocol_number,
    })
}

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
        modelo,
        serie,
        n_nf_ini,
        n_nf_fin,
        dh_recbto,
        n_prot,
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

/// Extract all occurrences of a simple XML tag's text content.
///
/// Searches for every `<tag_name>…</tag_name>` pair and returns the inner
/// text of each occurrence. Does not handle namespaced tags or CDATA sections.
fn extract_all_tag_values(xml: &str, tag_name: &str) -> Vec<String> {
    let open = format!("<{tag_name}>");
    let close = format!("</{tag_name}>");
    let mut results = Vec::new();
    let mut search_from = 0;

    while let Some(start_rel) = xml[search_from..].find(&open) {
        let content_start = search_from + start_rel + open.len();
        if let Some(end_rel) = xml[content_start..].find(&close) {
            results.push(xml[content_start..content_start + end_rel].to_string());
            search_from = content_start + end_rel + close.len();
        } else {
            break;
        }
    }

    results
}

/// Extract the raw content between the opening and closing of a tag,
/// including any namespace-prefixed variant. Returns a slice of the
/// original string covering from `<tag ...>` to `</tag>` inclusive.
fn extract_raw_tag(xml: &str, local_name: &str) -> Option<String> {
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
        assert_eq!(resp.status_message, "Evento registrado e vinculado a NF-e");
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
