use crate::FiscalError;
use crate::constants::NFE_NAMESPACE;
use crate::status_codes::{VALID_PROTOCOL_STATUSES, sefaz_status};
use crate::xml_utils::extract_xml_tag_value;

/// NF-e version used in wrapper elements when no version is found.
const DEFAULT_VERSION: &str = "4.00";

// ── Public API ──────────────────────────────────────────────────────────────

/// Attach the SEFAZ authorization protocol to a signed NFe XML,
/// producing the `<nfeProc>` wrapper required for storage and DANFE.
///
/// The function extracts the `<NFe>` from `request_xml` and the matching
/// `<protNFe>` from `response_xml`, validates the protocol status, and
/// joins them into a single `<nfeProc>` document.
///
/// If the response contains multiple `<protNFe>` nodes (batch response),
/// the function attempts to match by digest value and access key. When no
/// exact match is found it falls back to the first available `<protNFe>`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if:
/// - Either input is empty
/// - The `<NFe>` tag is missing from `request_xml`
/// - No `<protNFe>` can be found in `response_xml`
///
/// Returns [`FiscalError::SefazRejection`] if the protocol status code
/// is not in [`VALID_PROTOCOL_STATUSES`].
pub fn attach_protocol(request_xml: &str, response_xml: &str) -> Result<String, FiscalError> {
    if request_xml.is_empty() {
        return Err(FiscalError::XmlParsing("Request XML (NFe) is empty".into()));
    }
    if response_xml.is_empty() {
        return Err(FiscalError::XmlParsing(
            "Response XML (protocol) is empty".into(),
        ));
    }

    let nfe_content = extract_tag(request_xml, "NFe")
        .ok_or_else(|| FiscalError::XmlParsing("Could not find <NFe> tag in request XML".into()))?;

    // Extract digest and access key from the NFe for matching
    let digest_nfe = extract_xml_tag_value(request_xml, "DigestValue");
    let access_key = extract_inf_nfe_id(request_xml);

    // Try to find a matching protNFe by digest + access key
    let mut matched_prot: Option<String> = None;

    let prot_nodes = extract_all_tags(response_xml, "protNFe");

    for prot in &prot_nodes {
        let dig_val = extract_xml_tag_value(prot, "digVal");
        let ch_nfe = extract_xml_tag_value(prot, "chNFe");

        if let (Some(dn), Some(dv)) = (&digest_nfe, &dig_val) {
            if let (Some(ak), Some(cn)) = (&access_key, &ch_nfe) {
                if dn == dv && ak == cn {
                    // Exact match — validate status
                    let c_stat = extract_xml_tag_value(prot, "cStat").unwrap_or_default();
                    if !VALID_PROTOCOL_STATUSES.contains(&c_stat.as_str()) {
                        let x_motivo = extract_xml_tag_value(prot, "xMotivo").unwrap_or_default();
                        return Err(FiscalError::SefazRejection {
                            code: c_stat,
                            message: x_motivo,
                        });
                    }
                    matched_prot = Some(prot.clone());
                    break;
                }
            }
        }
    }

    if matched_prot.is_none() {
        // Check if any protNFe had a digVal (but didn't match)
        let mut found_dig_val = false;
        for prot in &prot_nodes {
            if extract_xml_tag_value(prot, "digVal").is_some() {
                found_dig_val = true;
                break;
            }
        }

        if !prot_nodes.is_empty() && !found_dig_val {
            // digVal is null in the response — error 18 per PHP
            let first_prot = &prot_nodes[0];
            let c_stat = extract_xml_tag_value(first_prot, "cStat").unwrap_or_default();
            let x_motivo = extract_xml_tag_value(first_prot, "xMotivo").unwrap_or_default();
            let msg = format!("digVal ausente na resposta SEFAZ: [{c_stat}] {x_motivo}");
            return Err(FiscalError::SefazRejection {
                code: c_stat,
                message: msg,
            });
        }

        if found_dig_val {
            // digVal exists but didn't match our DigestValue — error 5 per PHP
            let key_info = access_key.as_deref().unwrap_or("unknown");
            return Err(FiscalError::XmlParsing(format!(
                "Os digest são diferentes [{key_info}]"
            )));
        }

        // No protNFe at all
        let single_prot = extract_tag(response_xml, "protNFe").ok_or_else(|| {
            FiscalError::XmlParsing("Could not find <protNFe> in response XML".into())
        })?;

        // Validate status on the fallback protNFe
        let c_stat = extract_xml_tag_value(&single_prot, "cStat").unwrap_or_default();
        if !VALID_PROTOCOL_STATUSES.contains(&c_stat.as_str()) {
            let x_motivo = extract_xml_tag_value(&single_prot, "xMotivo").unwrap_or_default();
            return Err(FiscalError::SefazRejection {
                code: c_stat,
                message: x_motivo,
            });
        }
        matched_prot = Some(single_prot);
    }

    let version = extract_attribute(&nfe_content, "infNFe", "versao")
        .unwrap_or_else(|| DEFAULT_VERSION.to_string());

    Ok(join_xml(
        &nfe_content,
        &matched_prot.unwrap(),
        "nfeProc",
        &version,
    ))
}

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

/// Attach an event protocol response to the event request,
/// producing the `<procEventoNFe>` wrapper.
///
/// Extracts `<evento>` from `request_xml` and `<retEvento>` from
/// `response_xml`, validates the event status, and joins them
/// into a `<procEventoNFe>` document.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if:
/// - Either input is empty
/// - The `<evento>` tag is missing from `request_xml`
/// - The `<retEvento>` tag is missing from `response_xml`
/// - The `<idLote>` tag is missing from `request_xml` or `response_xml`
/// - The `idLote` values differ between request and response
///
/// Returns [`FiscalError::SefazRejection`] if the event status code
/// is not valid (135, 136, or 155 for cancellation only).
pub fn attach_event_protocol(request_xml: &str, response_xml: &str) -> Result<String, FiscalError> {
    if request_xml.is_empty() {
        return Err(FiscalError::XmlParsing("Event request XML is empty".into()));
    }
    if response_xml.is_empty() {
        return Err(FiscalError::XmlParsing(
            "Event response XML is empty".into(),
        ));
    }

    let evento_content = extract_tag(request_xml, "evento").ok_or_else(|| {
        FiscalError::XmlParsing("Could not find <evento> tag in request XML".into())
    })?;

    let ret_evento_content = extract_tag(response_xml, "retEvento").ok_or_else(|| {
        FiscalError::XmlParsing("Could not find <retEvento> tag in response XML".into())
    })?;

    // Get version from the evento tag
    let version = extract_attribute(&evento_content, "evento", "versao")
        .unwrap_or_else(|| DEFAULT_VERSION.to_string());

    // Validate event status (PHP validates status before idLote)
    let c_stat = extract_xml_tag_value(&ret_evento_content, "cStat").unwrap_or_default();
    let tp_evento = extract_xml_tag_value(&ret_evento_content, "tpEvento").unwrap_or_default();

    // Build the valid statuses list: 135, 136 always; 155 only for cancellation
    let mut valid_statuses: Vec<&str> = vec!["135", "136"];
    if tp_evento == EVT_CANCELA {
        valid_statuses.push("155");
    }

    if !valid_statuses.contains(&c_stat.as_str()) {
        let x_motivo = extract_xml_tag_value(&ret_evento_content, "xMotivo").unwrap_or_default();
        return Err(FiscalError::SefazRejection {
            code: c_stat,
            message: x_motivo,
        });
    }

    // Validate idLote is present in both request and response, then compare.
    // PHP addEnvEventoProtocol accesses ->nodeValue directly on idLote;
    // if the tag is absent, PHP throws a fatal error.
    let req_id_lote = extract_xml_tag_value(request_xml, "idLote")
        .ok_or_else(|| FiscalError::XmlParsing("idLote not found in request XML".into()))?;
    let ret_id_lote = extract_xml_tag_value(response_xml, "idLote")
        .ok_or_else(|| FiscalError::XmlParsing("idLote not found in response XML".into()))?;
    if req_id_lote != ret_id_lote {
        return Err(FiscalError::XmlParsing(
            "Os números de lote dos documentos são diferentes".into(),
        ));
    }

    Ok(join_xml(
        &evento_content,
        &ret_evento_content,
        "procEventoNFe",
        &version,
    ))
}

/// Attach a B2B financial tag to an authorized `<nfeProc>` XML,
/// wrapping both in a `<nfeProcB2B>` element.
///
/// # Arguments
///
/// * `nfe_proc_xml` - The authorized nfeProc XML.
/// * `b2b_xml` - The B2B financial XML (must contain the `tag_b2b` element).
/// * `tag_b2b` - Optional B2B tag name; defaults to `"NFeB2BFin"`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if:
/// - The `nfe_proc_xml` does not contain `<nfeProc>`
/// - The `b2b_xml` does not contain the expected B2B tag
/// - Either tag cannot be extracted
pub fn attach_b2b(
    nfe_proc_xml: &str,
    b2b_xml: &str,
    tag_b2b: Option<&str>,
) -> Result<String, FiscalError> {
    let tag_name = tag_b2b.unwrap_or("NFeB2BFin");

    if !nfe_proc_xml.contains("<nfeProc") {
        return Err(FiscalError::XmlParsing(
            "XML does not contain <nfeProc> — is this an authorized NFe?".into(),
        ));
    }

    let open_check = format!("<{tag_name}");
    if !b2b_xml.contains(&open_check) {
        return Err(FiscalError::XmlParsing(format!(
            "B2B XML does not contain <{tag_name}> tag"
        )));
    }

    let nfe_proc_content = extract_tag(nfe_proc_xml, "nfeProc")
        .ok_or_else(|| FiscalError::XmlParsing("Could not extract <nfeProc> from XML".into()))?;

    let b2b_content = extract_tag(b2b_xml, tag_name).ok_or_else(|| {
        FiscalError::XmlParsing(format!("Could not extract <{tag_name}> from B2B XML"))
    })?;

    let raw = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
         <nfeProcB2B>{nfe_proc_content}{b2b_content}</nfeProcB2B>"
    );

    // PHP Complements::b2bTag line 79 does:
    //   str_replace(array("\n", "\r", "\s"), '', $nfeb2bXML)
    // This removes newlines/carriage-returns (and the literal "\s" which is
    // a PHP quirk — "\s" inside single quotes is just the characters \ and s,
    // but that string never appears in XML anyway).
    let cleaned = strip_newlines(&raw);
    Ok(cleaned)
}

/// Remove `\n` and `\r` characters from a string.
///
/// Mirrors the PHP `str_replace(array("\n", "\r", "\s"), '', ...)` call
/// in `Complements::b2bTag`. The `\s` in PHP single-quoted strings is the
/// literal two-character sequence `\s`, not a regex; we replicate by also
/// removing it just in case, though it should never appear in valid XML.
fn strip_newlines(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\n' || c == '\r' {
            continue;
        }
        if c == '\\' {
            if let Some(&'s') = chars.peek() {
                chars.next(); // consume the 's'
                continue;
            }
        }
        result.push(c);
    }
    result
}

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

/// Check if an XML string contains a given tag (with proper delimiter check).
fn contains_xml_tag(xml: &str, tag_name: &str) -> bool {
    let pattern = format!("<{tag_name}");
    for (i, _) in xml.match_indices(&pattern) {
        let after = i + pattern.len();
        if after >= xml.len() {
            return true;
        }
        let next = xml.as_bytes()[after];
        if next == b' '
            || next == b'>'
            || next == b'/'
            || next == b'\n'
            || next == b'\r'
            || next == b'\t'
        {
            return true;
        }
    }
    false
}

/// Cancellation event type code (`110111`).
const EVT_CANCELA: &str = "110111";
/// Cancellation by substitution event type code (`110112`).
const EVT_CANCELA_SUBSTITUICAO: &str = "110112";

/// Valid status codes for cancellation event matching.
///
/// - `135` — Event registered and linked
/// - `136` — Event registered but not linked
/// - `155` — Already cancelled (late)
const VALID_CANCEL_STATUSES: &[&str] = &["135", "136", "155"];

/// Attach a cancellation event response to an authorized `<nfeProc>` XML,
/// marking the NF-e as locally cancelled.
///
/// This mirrors the PHP `Complements::cancelRegister()` method. The function
/// searches the `cancel_event_xml` for `<retEvento>` elements whose:
/// - `cStat` is in `[135, 136, 155]` (valid cancellation statuses)
/// - `tpEvento` is `110111` (cancellation) or `110112` (cancellation by substitution)
/// - `chNFe` matches the access key in the authorized NF-e's `<protNFe>`
///
/// When a matching `<retEvento>` is found, it is appended inside the
/// `<nfeProc>` element (before the closing `</nfeProc>` tag).
///
/// If no matching cancellation event is found, the original NF-e XML is
/// returned unchanged (same behavior as the PHP implementation).
///
/// # Arguments
///
/// * `nfe_proc_xml` - The authorized NF-e XML containing `<nfeProc>` with `<protNFe>`.
/// * `cancel_event_xml` - The SEFAZ cancellation event response XML containing `<retEvento>`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if:
/// - The `nfe_proc_xml` does not contain `<protNFe>` (not an authorized NF-e)
/// - The `<protNFe>` does not contain `<chNFe>`
pub fn attach_cancellation(
    nfe_proc_xml: &str,
    cancel_event_xml: &str,
) -> Result<String, FiscalError> {
    // Validate the NF-e has a protNFe with a chNFe
    let prot_nfe = extract_tag(nfe_proc_xml, "protNFe").ok_or_else(|| {
        FiscalError::XmlParsing(
            "Could not find <protNFe> in NF-e XML — is this an authorized NF-e?".into(),
        )
    })?;

    let ch_nfe = extract_xml_tag_value(&prot_nfe, "chNFe")
        .ok_or_else(|| FiscalError::XmlParsing("Could not find <chNFe> inside <protNFe>".into()))?;

    // Search for matching retEvento in the cancellation XML
    let ret_eventos = extract_all_tags(cancel_event_xml, "retEvento");

    for ret_evento in &ret_eventos {
        let c_stat = match extract_xml_tag_value(ret_evento, "cStat") {
            Some(v) => v,
            None => continue,
        };
        let tp_evento = match extract_xml_tag_value(ret_evento, "tpEvento") {
            Some(v) => v,
            None => continue,
        };
        let ch_nfe_evento = match extract_xml_tag_value(ret_evento, "chNFe") {
            Some(v) => v,
            None => continue,
        };

        if VALID_CANCEL_STATUSES.contains(&c_stat.as_str())
            && (tp_evento == EVT_CANCELA || tp_evento == EVT_CANCELA_SUBSTITUICAO)
            && ch_nfe_evento == ch_nfe
        {
            // Insert the retEvento before </nfeProc>
            let close_tag = "</nfeProc>";
            if let Some(pos) = nfe_proc_xml.rfind(close_tag) {
                let mut result = String::with_capacity(nfe_proc_xml.len() + ret_evento.len());
                result.push_str(&nfe_proc_xml[..pos]);
                result.push_str(ret_evento);
                result.push_str(close_tag);
                return Ok(result);
            }
            // If no </nfeProc>, just append to the end (best effort)
            break;
        }
    }

    // No matching cancellation event found — return original XML unchanged
    Ok(nfe_proc_xml.to_string())
}

// ── Internal helpers ────────────────────────────────────────────────────────

/// Join two XML fragments into a versioned namespace wrapper element.
///
/// Produces:
/// ```xml
/// <?xml version="1.0" encoding="UTF-8"?>
/// <{node_name} versao="{version}" xmlns="{NFE_NAMESPACE}">
///   {first}{second}
/// </{node_name}>
/// ```
fn join_xml(first: &str, second: &str, node_name: &str, version: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
         <{node_name} versao=\"{version}\" xmlns=\"{NFE_NAMESPACE}\">\
         {first}{second}</{node_name}>"
    )
}

/// Extract a complete XML tag (outermost match) including attributes and
/// all nested content. Uses `lastIndexOf`-style search for the closing tag
/// to handle nested tags of the same name.
///
/// Returns `None` if either the opening or closing tag is not found.
fn extract_tag(xml: &str, tag_name: &str) -> Option<String> {
    // Find the opening tag: <tagName followed by whitespace, >, or /
    let open_pattern = format!("<{tag_name}");
    let start = xml.find(&open_pattern)?;

    // Verify that the character after `<tagName` is a valid delimiter
    // (space, >, /) to avoid matching tags like `<tagNameExtra>`
    let after_open = start + open_pattern.len();
    if after_open < xml.len() {
        let next_char = xml.as_bytes()[after_open];
        if next_char != b' '
            && next_char != b'>'
            && next_char != b'/'
            && next_char != b'\n'
            && next_char != b'\r'
            && next_char != b'\t'
        {
            return None;
        }
    }

    let close_tag = format!("</{tag_name}>");
    let close_index = xml.rfind(&close_tag)?;

    Some(xml[start..close_index + close_tag.len()].to_string())
}

/// Extract all occurrences of a tag from XML. Finds each non-overlapping
/// `<tagName ...>...</tagName>` in the source string.
fn extract_all_tags(xml: &str, tag_name: &str) -> Vec<String> {
    let mut results = Vec::new();
    let open_pattern = format!("<{tag_name}");
    let close_tag = format!("</{tag_name}>");
    let mut search_from = 0;

    while search_from < xml.len() {
        let start = match xml[search_from..].find(&open_pattern) {
            Some(pos) => search_from + pos,
            None => break,
        };

        // Verify delimiter after tag name
        let after_open = start + open_pattern.len();
        if after_open < xml.len() {
            let next_char = xml.as_bytes()[after_open];
            if next_char != b' '
                && next_char != b'>'
                && next_char != b'/'
                && next_char != b'\n'
                && next_char != b'\r'
                && next_char != b'\t'
            {
                search_from = after_open;
                continue;
            }
        }

        let end = match xml[start..].find(&close_tag) {
            Some(pos) => start + pos + close_tag.len(),
            None => break,
        };

        results.push(xml[start..end].to_string());
        search_from = end;
    }

    results
}

/// Extract an XML attribute value from a tag. Searches for the tag opening
/// then finds `attr="value"` within it.
fn extract_attribute(xml: &str, tag_name: &str, attr_name: &str) -> Option<String> {
    let open = format!("<{tag_name}");
    let start = xml.find(&open)?;

    // Find the end of the opening tag
    let tag_end = xml[start..].find('>')? + start;
    let tag_header = &xml[start..tag_end];

    // Find attr="value" pattern
    let attr_pattern = format!("{attr_name}=\"");
    let attr_start = tag_header.find(&attr_pattern)? + attr_pattern.len();
    let attr_end = tag_header[attr_start..].find('"')? + attr_start;

    Some(tag_header[attr_start..attr_end].to_string())
}

/// Extract the access key from an `<infNFe Id="NFe...">` attribute.
/// Returns the 44-digit key (without the "NFe" prefix).
fn extract_inf_nfe_id(xml: &str) -> Option<String> {
    let attr_val = extract_attribute(xml, "infNFe", "Id")?;
    Some(
        attr_val
            .strip_prefix("NFe")
            .unwrap_or(&attr_val)
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_tag_finds_outermost_match() {
        let xml = r#"<root><NFe versao="4.00"><inner/></NFe></root>"#;
        let result = extract_tag(xml, "NFe").unwrap();
        assert!(result.starts_with("<NFe"));
        assert!(result.ends_with("</NFe>"));
        assert!(result.contains("<inner/>"));
    }

    #[test]
    fn extract_tag_returns_none_for_missing_tag() {
        let xml = "<root><other/></root>";
        assert!(extract_tag(xml, "NFe").is_none());
    }

    #[test]
    fn extract_tag_does_not_match_prefix() {
        let xml = "<root><NFeExtra>data</NFeExtra></root>";
        assert!(extract_tag(xml, "NFe").is_none());
    }

    #[test]
    fn extract_attribute_works() {
        let xml = r#"<infNFe versao="4.00" Id="NFe12345">"#;
        assert_eq!(
            extract_attribute(xml, "infNFe", "versao"),
            Some("4.00".to_string())
        );
        assert_eq!(
            extract_attribute(xml, "infNFe", "Id"),
            Some("NFe12345".to_string())
        );
    }

    #[test]
    fn extract_all_tags_finds_multiple() {
        let xml = r#"<root><item>1</item><item>2</item><item>3</item></root>"#;
        let items = extract_all_tags(xml, "item");
        assert_eq!(items.len(), 3);
        assert!(items[0].contains("1"));
        assert!(items[2].contains("3"));
    }

    #[test]
    fn join_xml_produces_correct_wrapper() {
        let result = join_xml("<A/>", "<B/>", "wrapper", "4.00");
        assert!(result.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(result.contains("<wrapper versao=\"4.00\""));
        assert!(result.contains(&format!("xmlns=\"{NFE_NAMESPACE}\"")));
        assert!(result.ends_with("</wrapper>"));
    }

    #[test]
    fn extract_inf_nfe_id_strips_prefix() {
        let xml = r#"<NFe><infNFe versao="4.00" Id="NFe35260112345678000199650010000000011123456780"></infNFe></NFe>"#;
        let key = extract_inf_nfe_id(xml).unwrap();
        assert_eq!(key, "35260112345678000199650010000000011123456780");
    }

    // ── attach_cancellation tests ─────────────────────────────────────

    #[test]
    fn attach_cancellation_appends_matching_ret_evento() {
        let nfe_proc = concat!(
            r#"<?xml version="1.0" encoding="UTF-8"?>"#,
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe><infNFe versao="4.00" Id="NFe35260112345678000199550010000000011123456780">"#,
            r#"<ide/></infNFe></NFe>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat><nProt>135220000009921</nProt>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEnvEvento><retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat>"#,
            r#"<xMotivo>Evento registrado e vinculado a NF-e</xMotivo>"#,
            r#"<tpEvento>110111</tpEvento>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento></retEnvEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();

        // Must contain the retEvento inside nfeProc
        assert!(
            result.contains("<retEvento"),
            "Result should contain <retEvento>"
        );
        assert!(
            result.contains("<tpEvento>110111</tpEvento>"),
            "Result should contain cancellation event type"
        );
        // The retEvento should appear before </nfeProc>
        let ret_pos = result.find("<retEvento").unwrap();
        let close_pos = result.rfind("</nfeProc>").unwrap();
        assert!(ret_pos < close_pos, "retEvento should be before </nfeProc>");
        // Original content should be preserved
        assert!(result.contains("<protNFe"));
        assert!(result.contains("<NFe>"));
    }

    #[test]
    fn attach_cancellation_ignores_non_matching_ch_nfe() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat>"#,
            r#"<tpEvento>110111</tpEvento>"#,
            r#"<chNFe>99999999999999999999999999999999999999999999</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();
        // Should return original unchanged — no matching chNFe
        assert_eq!(result, nfe_proc);
    }

    #[test]
    fn attach_cancellation_ignores_wrong_tp_evento() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat>"#,
            r#"<tpEvento>110110</tpEvento>"#, // CCe, not cancellation
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();
        // Should return original unchanged — wrong tpEvento
        assert_eq!(result, nfe_proc);
    }

    #[test]
    fn attach_cancellation_ignores_rejected_status() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>573</cStat>"#, // Rejected status
            r#"<tpEvento>110111</tpEvento>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();
        // Should return original unchanged — rejected status
        assert_eq!(result, nfe_proc);
    }

    #[test]
    fn attach_cancellation_accepts_status_155() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>155</cStat>"#,
            r#"<tpEvento>110111</tpEvento>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();
        assert!(result.contains("<retEvento"));
    }

    #[test]
    fn attach_cancellation_accepts_substituicao_110112() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat>"#,
            r#"<tpEvento>110112</tpEvento>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();
        assert!(
            result.contains("<tpEvento>110112</tpEvento>"),
            "Should accept cancellation by substitution"
        );
    }

    #[test]
    fn attach_cancellation_rejects_missing_prot_nfe() {
        let nfe_xml = "<NFe><infNFe/></NFe>";
        let cancel_xml = "<retEvento/>";
        let err = attach_cancellation(nfe_xml, cancel_xml).unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_cancellation_rejects_missing_ch_nfe_in_prot() {
        let nfe_proc = concat!(
            r#"<nfeProc><protNFe versao="4.00"><infProt>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe></nfeProc>"#
        );
        let cancel_xml = "<retEvento/>";
        let err = attach_cancellation(nfe_proc, cancel_xml).unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    // ── attach_protocol tests ─────────────────────────────────────

    #[test]
    fn attach_protocol_empty_request_xml() {
        let err = attach_protocol("", "<protNFe/>").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_protocol_empty_response_xml() {
        let err = attach_protocol("<NFe/>", "").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_protocol_matching_digest_and_key() {
        let request = concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe35260112345678000199650010000000011123456780">"#,
            r#"<ide/></infNFe>"#,
            r#"<Signature><SignedInfo/><SignatureValue/>"#,
            r#"<KeyInfo><DigestValue>abc123</DigestValue></KeyInfo></Signature>"#,
            r#"</NFe>"#
        );
        let response = concat!(
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<digVal>abc123</digVal>"#,
            r#"<chNFe>35260112345678000199650010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"<xMotivo>Autorizado</xMotivo>"#,
            r#"</infProt></protNFe>"#
        );
        let result = attach_protocol(request, response).unwrap();
        assert!(result.contains("<nfeProc"));
        assert!(result.contains("</nfeProc>"));
        assert!(result.contains("<NFe>"));
        assert!(result.contains("<protNFe"));
    }

    #[test]
    fn attach_protocol_rejected_status_in_exact_match() {
        let request = concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe35260112345678000199650010000000011123456780">"#,
            r#"<ide/></infNFe>"#,
            r#"<Signature><SignedInfo/><SignatureValue/>"#,
            r#"<KeyInfo><DigestValue>abc123</DigestValue></KeyInfo></Signature>"#,
            r#"</NFe>"#
        );
        let response = concat!(
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<digVal>abc123</digVal>"#,
            r#"<chNFe>35260112345678000199650010000000011123456780</chNFe>"#,
            r#"<cStat>999</cStat>"#,
            r#"<xMotivo>Rejeitada</xMotivo>"#,
            r#"</infProt></protNFe>"#
        );
        let err = attach_protocol(request, response).unwrap_err();
        assert!(matches!(err, FiscalError::SefazRejection { .. }));
    }

    #[test]
    fn attach_protocol_fallback_rejected_status() {
        // No digest match, falls back to first protNFe which is rejected
        let request = concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe35260112345678000199650010000000011123456780">"#,
            r#"<ide/></infNFe></NFe>"#
        );
        let response = concat!(
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<cStat>999</cStat>"#,
            r#"<xMotivo>Rejeitada</xMotivo>"#,
            r#"</infProt></protNFe>"#
        );
        let err = attach_protocol(request, response).unwrap_err();
        assert!(matches!(err, FiscalError::SefazRejection { .. }));
    }

    // ── attach_inutilizacao tests ───────────────────────────────────

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

    // ── attach_event_protocol tests ─────────────────────────────────

    #[test]
    fn attach_event_protocol_empty_request() {
        let err = attach_event_protocol("", "<retEvento/>").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_event_protocol_empty_response() {
        let err = attach_event_protocol("<evento/>", "").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_event_protocol_missing_evento() {
        let err = attach_event_protocol(
            "<other/>",
            "<retEvento><infEvento><cStat>135</cStat></infEvento></retEvento>",
        )
        .unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_event_protocol_missing_ret_evento() {
        let err =
            attach_event_protocol(r#"<evento versao="1.00"><infEvento/></evento>"#, "<other/>")
                .unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_event_protocol_rejected_status() {
        let err = attach_event_protocol(
            r#"<evento versao="1.00"><infEvento/></evento>"#,
            r#"<retEvento><infEvento><cStat>999</cStat><xMotivo>Rejeitado</xMotivo></infEvento></retEvento>"#,
        )
        .unwrap_err();
        assert!(matches!(err, FiscalError::SefazRejection { .. }));
    }

    #[test]
    fn attach_event_protocol_success() {
        let request = concat!(
            r#"<envEvento><idLote>100</idLote>"#,
            r#"<evento versao="1.00"><infEvento Id="ID1234"/></evento>"#,
            r#"</envEvento>"#
        );
        let response = concat!(
            r#"<retEnvEvento><idLote>100</idLote>"#,
            r#"<retEvento><infEvento><cStat>135</cStat>"#,
            r#"<xMotivo>Evento registrado</xMotivo>"#,
            r#"</infEvento></retEvento></retEnvEvento>"#
        );
        let result = attach_event_protocol(request, response).unwrap();
        assert!(result.contains("<procEventoNFe"));
        assert!(result.contains("<evento"));
        assert!(result.contains("<retEvento>"));
    }

    // ── attach_b2b tests ────────────────────────────────────────────

    #[test]
    fn attach_b2b_no_nfe_proc() {
        let err = attach_b2b("<NFe/>", "<NFeB2BFin>data</NFeB2BFin>", None).unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_b2b_no_b2b_tag() {
        let err = attach_b2b("<nfeProc><NFe/></nfeProc>", "<other>data</other>", None).unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_b2b_extract_failure() {
        // nfeProc without closing tag won't extract
        let err = attach_b2b("<nfeProc><NFe/>", "<NFeB2BFin>data</NFeB2BFin>", None).unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_b2b_success() {
        let result = attach_b2b(
            "<nfeProc><NFe/><protNFe/></nfeProc>",
            "<NFeB2BFin><tag>data</tag></NFeB2BFin>",
            None,
        )
        .unwrap();
        assert!(result.contains("<nfeProcB2B>"));
        assert!(result.contains("<nfeProc>"));
        assert!(result.contains("<NFeB2BFin>"));
    }

    #[test]
    fn attach_b2b_custom_tag() {
        let result = attach_b2b(
            "<nfeProc><NFe/><protNFe/></nfeProc>",
            "<CustomB2B><tag>data</tag></CustomB2B>",
            Some("CustomB2B"),
        )
        .unwrap();
        assert!(result.contains("<CustomB2B>"));
    }

    // ── extract_all_tags delimiter check ─────────────────────────────

    #[test]
    fn extract_all_tags_skips_prefix_match() {
        // "protNFeExtra" should NOT be matched when looking for "protNFe"
        let xml = "<root><protNFeExtra>bad</protNFeExtra><protNFe>good</protNFe></root>";
        let results = extract_all_tags(xml, "protNFe");
        assert_eq!(results.len(), 1);
        assert!(results[0].contains("good"));
    }

    #[test]
    fn attach_cancellation_picks_first_matching_from_multiple_ret_eventos() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEnvEvento>"#,
            // First: wrong chNFe
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat><tpEvento>110111</tpEvento>"#,
            r#"<chNFe>99999999999999999999999999999999999999999999</chNFe>"#,
            r#"<nProt>111111111111111</nProt>"#,
            r#"</infEvento></retEvento>"#,
            // Second: correct match
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat><tpEvento>110111</tpEvento>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>222222222222222</nProt>"#,
            r#"</infEvento></retEvento>"#,
            r#"</retEnvEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();
        assert!(result.contains("<nProt>222222222222222</nProt>"));
        // Should only have one retEvento (the matching one)
        assert_eq!(result.matches("<retEvento").count(), 1);
    }

    // ── to_authorize routing tests ──────────────────────────────────────

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

    #[test]
    fn contains_xml_tag_basic() {
        assert!(contains_xml_tag("<NFe versao=\"4.00\">", "NFe"));
        assert!(contains_xml_tag("<NFe>", "NFe"));
        assert!(contains_xml_tag("<NFe/>", "NFe"));
        assert!(!contains_xml_tag("<NFeExtra>", "NFe"));
        assert!(contains_xml_tag("<envEvento versao=\"1.00\">", "envEvento"));
        assert!(contains_xml_tag("<inutNFe versao=\"4.00\">", "inutNFe"));
    }

    // ── attach_b2b whitespace stripping tests ───────────────────────────

    #[test]
    fn attach_b2b_strips_newlines() {
        let nfe_proc = "<nfeProc versao=\"4.00\">\n<NFe/>\n<protNFe/>\n</nfeProc>";
        let b2b = "<NFeB2BFin>\n<data>test</data>\n</NFeB2BFin>";
        let result = attach_b2b(nfe_proc, b2b, None).unwrap();
        assert!(!result.contains('\n'), "Result should not contain newlines");
        assert!(
            !result.contains('\r'),
            "Result should not contain carriage returns"
        );
        assert!(result.contains("<nfeProcB2B>"));
        assert!(result.contains("<NFeB2BFin>"));
    }

    #[test]
    fn attach_b2b_strips_carriage_returns() {
        let nfe_proc = "<nfeProc versao=\"4.00\">\r\n<NFe/>\r\n</nfeProc>";
        let b2b = "<NFeB2BFin><data>test</data></NFeB2BFin>";
        let result = attach_b2b(nfe_proc, b2b, None).unwrap();
        assert!(!result.contains('\r'));
        assert!(!result.contains('\n'));
    }

    // ── attach_protocol: fallback protNFe with invalid cStat (lines 112-116) ──

    #[test]
    fn attach_protocol_fallback_prot_invalid_status() {
        // Request with NFe, digest, access key
        let request = concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe35260112345678000199550010000000011123456780">"#,
            r#"<DigestValue>abc123</DigestValue>"#,
            r#"</infNFe></NFe>"#
        );
        // Response with single protNFe that has NO digVal (trigger fallback),
        // but status is invalid
        let response = concat!(
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<cStat>999</cStat>"#,
            r#"<xMotivo>Rejeitado</xMotivo>"#,
            r#"</infProt></protNFe>"#
        );
        let err = attach_protocol(request, response).unwrap_err();
        match err {
            FiscalError::SefazRejection { code, .. } => assert_eq!(code, "999"),
            other => panic!("Expected SefazRejection, got {:?}", other),
        }
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

    // ── attach_event_protocol: idLote mismatch (lines 277-278) ──────────

    #[test]
    fn attach_event_protocol_id_lote_mismatch() {
        let request = concat!(
            r#"<envEvento><idLote>100</idLote>"#,
            r#"<evento versao="1.00"><infEvento>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></evento></envEvento>"#
        );
        let response = concat!(
            r#"<retEnvEvento><idLote>999</idLote>"#,
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat><xMotivo>OK</xMotivo>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></retEvento></retEnvEvento>"#
        );
        let err = attach_event_protocol(request, response).unwrap_err();
        match err {
            FiscalError::XmlParsing(msg) => {
                assert!(
                    msg.contains("lote"),
                    "Expected lote mismatch error, got: {msg}"
                );
            }
            other => panic!("Expected XmlParsing, got {:?}", other),
        }
    }

    // ── attach_event_protocol: missing idLote ─────────────────────────

    #[test]
    fn attach_event_protocol_missing_id_lote_in_request() {
        let request = concat!(
            r#"<envEvento>"#,
            r#"<evento versao="1.00"><infEvento>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></evento></envEvento>"#
        );
        let response = concat!(
            r#"<retEnvEvento><idLote>100</idLote>"#,
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat><xMotivo>OK</xMotivo>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></retEvento></retEnvEvento>"#
        );
        let err = attach_event_protocol(request, response).unwrap_err();
        match err {
            FiscalError::XmlParsing(msg) => {
                assert_eq!(msg, "idLote not found in request XML");
            }
            other => panic!("Expected XmlParsing, got {:?}", other),
        }
    }

    #[test]
    fn attach_event_protocol_missing_id_lote_in_response() {
        let request = concat!(
            r#"<envEvento><idLote>100</idLote>"#,
            r#"<evento versao="1.00"><infEvento>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></evento></envEvento>"#
        );
        let response = concat!(
            r#"<retEnvEvento>"#,
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat><xMotivo>OK</xMotivo>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></retEvento></retEnvEvento>"#
        );
        let err = attach_event_protocol(request, response).unwrap_err();
        match err {
            FiscalError::XmlParsing(msg) => {
                assert_eq!(msg, "idLote not found in response XML");
            }
            other => panic!("Expected XmlParsing, got {:?}", other),
        }
    }

    #[test]
    fn attach_event_protocol_missing_id_lote_in_both() {
        let request = concat!(
            r#"<envEvento>"#,
            r#"<evento versao="1.00"><infEvento>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></evento></envEvento>"#
        );
        let response = concat!(
            r#"<retEnvEvento>"#,
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat><xMotivo>OK</xMotivo>"#,
            r#"<tpEvento>110110</tpEvento>"#,
            r#"</infEvento></retEvento></retEnvEvento>"#
        );
        let err = attach_event_protocol(request, response).unwrap_err();
        match err {
            FiscalError::XmlParsing(msg) => {
                assert_eq!(msg, "idLote not found in request XML");
            }
            other => panic!("Expected XmlParsing, got {:?}", other),
        }
    }

    // ── attach_b2b: extract_tag for b2b content (line 348) ──────────────

    #[test]
    fn attach_b2b_extract_tag_coverage() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe><infNFe/></NFe><protNFe><infProt/></protNFe>"#,
            r#"</nfeProc>"#
        );
        let b2b = r#"<NFeB2BFin versao="1.00"><dados>value</dados></NFeB2BFin>"#;
        let result = attach_b2b(nfe_proc, b2b, None).unwrap();
        assert!(result.contains("<nfeProcB2B>"));
        assert!(result.contains("<dados>value</dados>"));
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

    // ── contains_xml_tag: tag at very end of string (line 446) ──────────

    #[test]
    fn contains_xml_tag_at_end_of_string() {
        // Tag pattern at the very end, after >= xml.len() → true
        assert!(contains_xml_tag("<NFe", "NFe"));
    }

    // ── strip_newlines helper tests ─────────────────────────────────────

    #[test]
    fn strip_newlines_removes_newlines_and_cr() {
        assert_eq!(strip_newlines("a\nb\rc\r\nd"), "abcd");
    }

    #[test]
    fn strip_newlines_removes_literal_backslash_s() {
        assert_eq!(strip_newlines("abc\\sdef"), "abcdef");
    }

    #[test]
    fn strip_newlines_preserves_normal_content() {
        assert_eq!(strip_newlines("<tag>value</tag>"), "<tag>value</tag>");
    }
}
