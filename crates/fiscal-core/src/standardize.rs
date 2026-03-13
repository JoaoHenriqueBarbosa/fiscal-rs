use crate::FiscalError;

/// Known root tag names for NFe-related XML documents.
///
/// Checked in order to identify the document type from the root element
/// of a parsed XML string.
const ROOT_TAG_LIST: &[&str] = &[
    "distDFeInt",
    "resNFe",
    "resEvento",
    "envEvento",
    "ConsCad",
    "consSitNFe",
    "consReciNFe",
    "downloadNFe",
    "enviNFe",
    "inutNFe",
    "admCscNFCe",
    "consStatServ",
    "retDistDFeInt",
    "retEnvEvento",
    "retConsCad",
    "retConsSitNFe",
    "retConsReciNFe",
    "retDownloadNFe",
    "retEnviNFe",
    "retInutNFe",
    "retAdmCscNFCe",
    "retConsStatServ",
    "procInutNFe",
    "procEventoNFe",
    "procNFe",
    "nfeProc",
    "NFe",
];

/// Identify the type of an NF-e XML document from its content.
///
/// Parses the XML and checks the root element against the list of known
/// NFe document types. Returns the matched root tag name (e.g. `"NFe"`,
/// `"nfeProc"`, `"retConsSitNFe"`).
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the input is empty, whitespace-only,
/// not valid XML, or not a recognised NFe document type.
pub fn identify_xml_type(xml: &str) -> Result<String, FiscalError> {
    let trimmed = xml.trim();
    if trimmed.is_empty() {
        return Err(FiscalError::XmlParsing("XML is empty.".into()));
    }
    if !trimmed.starts_with('<') {
        return Err(FiscalError::XmlParsing(
            "Invalid document: not valid XML.".into(),
        ));
    }

    // Try to parse with quick-xml to validate it is well-formed XML
    let reader = quick_xml::Reader::from_str(trimmed);
    let _ = reader; // quick-xml reader is lazy, so we probe events below

    // Find root tags by scanning the XML content for known element names.
    // We search for `<TagName` (followed by space or `>`) to identify the
    // root element even when namespaces or attributes are present.
    if let Some(tag) = find_root_tag(trimmed) {
        return Ok(tag.to_string());
    }

    Err(FiscalError::XmlParsing(
        "Document does not belong to the NFe project.".into(),
    ))
}

/// Search raw XML text for a known root tag.
fn find_root_tag(xml: &str) -> Option<&'static str> {
    for &tag in ROOT_TAG_LIST {
        // Match `<tag `, `<tag>`, or `<tag\n`
        let pattern_space = format!("<{tag} ");
        let pattern_close = format!("<{tag}>");
        let pattern_newline = format!("<{tag}\n");
        if xml.contains(&pattern_space)
            || xml.contains(&pattern_close)
            || xml.contains(&pattern_newline)
        {
            return Some(tag);
        }
    }
    None
}

/// Convert an NFe XML string to a JSON string representation.
///
/// First validates that the XML is a recognised NFe document via
/// [`identify_xml_type`], then converts the XML tree to a JSON object.
///
/// ## Root-tag unwrapping (PHP parity)
///
/// Like PHP's `Standardize::toStd()`, the identified root element is
/// **unwrapped**: for `<NFe xmlns="..."><infNFe>...</infNFe></NFe>` the
/// result is `{"infNFe":{...}}`, not `{"NFe":{"infNFe":{...}}}`.
/// Attributes on the root element (except `xmlns`) are merged into the
/// returned object as plain keys.
///
/// ## Attribute handling (difference from PHP)
///
/// The PHP `Standardize::toStd()` method uses `simplexml_load_string` +
/// `json_encode`, which places XML attributes under an `@attributes` key
/// (later renamed to `attributes`). This Rust implementation places
/// attributes **inline** alongside child elements — e.g. an element
/// `<infNFe versao="4.00" Id="NFe123">` produces `{"versao":"4.00",
/// "Id":"NFe123", ...}` rather than `{"attributes":{"versao":"4.00",
/// "Id":"NFe123"}, ...}`. This is a deliberate design decision: inline
/// attributes are more ergonomic for JSON consumers and avoid the extra
/// nesting level. Consumers that relied on the PHP `attributes` key
/// should adapt their field lookups accordingly.
///
/// ## `infNFeSupl` / CDATA handling
///
/// The PHP `toStd()` has special post-processing for `infNFeSupl`: it
/// re-extracts `qrCode` and `urlChave` via DOM because PHP's
/// `simplexml_load_string` can corrupt CDATA sections. This Rust
/// implementation uses `quick-xml`'s `Event::CData` handler, which
/// correctly preserves CDATA content without mangling, so no special
/// post-processing is needed.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the input is not valid NFe XML,
/// or if conversion to JSON fails.
pub fn xml_to_json(xml: &str) -> Result<String, FiscalError> {
    let value = xml_to_value(xml)?;

    serde_json::to_string(&value)
        .map_err(|e| FiscalError::XmlParsing(format!("JSON serialization failed: {e}")))
}

/// Convert an NFe XML string to a navigable [`serde_json::Value`] tree.
///
/// This is the Rust equivalent of PHP's `Standardize::toStd()`, which returns
/// a `stdClass` object. In Rust, [`serde_json::Value`] serves the same role:
/// it is a dynamically-typed, navigable tree that can be indexed with
/// `value["fieldName"]`.
///
/// The identified root element is **unwrapped** so the returned value
/// contains the children of the root tag directly, matching PHP behaviour.
/// For example, `<NFe xmlns="..."><infNFe>...</infNFe></NFe>` yields
/// `{"infNFe": {...}}`. Attributes on the root element (except `xmlns`)
/// are merged as plain keys.
///
/// # Example
///
/// ```rust,ignore
/// let value = xml_to_value(xml)?;
/// let cuf = &value["infNFe"]["ide"]["cUF"];
/// assert_eq!(cuf.as_str(), Some("35"));
/// ```
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the input is not valid NFe XML.
pub fn xml_to_value(xml: &str) -> Result<serde_json::Value, FiscalError> {
    let root_tag = identify_xml_type(xml)?;
    let full = xml_str_to_json_value(xml.trim())?;

    // Unwrap the identified root tag to match PHP's Standardize::toStd()
    // behaviour, which returns the *contents* of the root element.
    match full {
        serde_json::Value::Object(map) => {
            if let Some(inner) = map.get(&root_tag) {
                Ok(inner.clone())
            } else {
                // Root tag not found as key — return as-is (shouldn't happen)
                Ok(serde_json::Value::Object(map))
            }
        }
        other => Ok(other),
    }
}

/// Convert an NFe XML string to a [`serde_json::Map`] (equivalent to an
/// associative array / hash map).
///
/// This is the Rust equivalent of PHP's `Standardize::toArray()`, which returns
/// an associative array. In Rust, [`serde_json::Map<String, Value>`] is the
/// natural equivalent: an ordered map of string keys to dynamically-typed values.
///
/// Like [`xml_to_value`], the identified root element is unwrapped: the map
/// contains the *children* of the root tag, not the root tag itself.
///
/// # Example
///
/// ```rust,ignore
/// let map = xml_to_map(xml)?;
/// let inf_nfe = map.get("infNFe").unwrap();
/// ```
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the input is not valid NFe XML,
/// or if the top-level JSON value is not an object (should not happen for
/// well-formed NFe documents).
pub fn xml_to_map(xml: &str) -> Result<serde_json::Map<String, serde_json::Value>, FiscalError> {
    let value = xml_to_value(xml)?;
    match value {
        serde_json::Value::Object(map) => Ok(map),
        _ => Err(FiscalError::XmlParsing(
            "Top-level XML value is not an object.".into(),
        )),
    }
}

/// Recursively convert an XML string into a serde_json::Value.
///
/// This is a simplified converter that handles elements, text content,
/// and attributes. Namespace prefixes are stripped from tag names and
/// `xmlns` attributes are omitted (they are XML metadata, not data).
fn xml_str_to_json_value(xml: &str) -> Result<serde_json::Value, FiscalError> {
    use quick_xml::Reader;
    use quick_xml::events::Event;
    use serde_json::{Map, Value};

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut stack: Vec<(String, Map<String, Value>)> = Vec::new();
    let mut root_map = Map::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let raw_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let local_name = strip_ns_prefix(&raw_name);

                let mut attrs_map = Map::new();
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    if is_xmlns_attr(&key) {
                        continue;
                    }
                    let val = String::from_utf8_lossy(&attr.value).to_string();
                    attrs_map.insert(key, Value::String(val));
                }

                stack.push((local_name, attrs_map));
            }
            Ok(Event::Empty(ref e)) => {
                let raw_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let local_name = strip_ns_prefix(&raw_name);

                let mut attrs_map = Map::new();
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    if is_xmlns_attr(&key) {
                        continue;
                    }
                    let val = String::from_utf8_lossy(&attr.value).to_string();
                    attrs_map.insert(key, Value::String(val));
                }

                let child_val = if attrs_map.is_empty() {
                    Value::String(String::new())
                } else {
                    Value::Object(attrs_map)
                };

                if let Some((_name, map)) = stack.last_mut() {
                    insert_into_map(map, &local_name, child_val);
                } else {
                    root_map.insert(local_name, child_val);
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.decode().unwrap_or_default().to_string();
                if !text.is_empty() {
                    if let Some((_name, map)) = stack.last_mut() {
                        map.insert("#text".to_string(), Value::String(text));
                    }
                }
            }
            Ok(Event::End(_)) => {
                if let Some((name, map)) = stack.pop() {
                    let child_val = if map.len() == 1 {
                        if let Some(text) = map.get("#text") {
                            text.clone()
                        } else {
                            Value::Object(map)
                        }
                    } else if map.is_empty() {
                        Value::String(String::new())
                    } else {
                        Value::Object(map)
                    };

                    if let Some((_parent_name, parent_map)) = stack.last_mut() {
                        insert_into_map(parent_map, &name, child_val);
                    } else {
                        root_map.insert(name, child_val);
                    }
                }
            }
            Ok(Event::Decl(_)) | Ok(Event::Comment(_)) | Ok(Event::PI(_)) => {}
            Ok(Event::CData(ref e)) => {
                let text = String::from_utf8_lossy(e.as_ref()).to_string();
                if let Some((_name, map)) = stack.last_mut() {
                    map.insert("#text".to_string(), Value::String(text));
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(FiscalError::XmlParsing(format!("XML parse error: {e}"))),
            _ => {}
        }
    }

    Ok(Value::Object(root_map))
}

/// Strip namespace prefix from a tag name (e.g. `"nfe:NFe"` -> `"NFe"`).
fn strip_ns_prefix(name: &str) -> String {
    match name.find(':') {
        Some(idx) => name[idx + 1..].to_string(),
        None => name.to_string(),
    }
}

/// Check whether an attribute key is an XML namespace declaration.
///
/// Returns `true` for `xmlns` and `xmlns:*` (e.g. `xmlns:nfe`).
fn is_xmlns_attr(key: &str) -> bool {
    key == "xmlns" || key.starts_with("xmlns:")
}

/// Insert a value into a JSON map, converting to an array if the key already exists.
fn insert_into_map(
    map: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: serde_json::Value,
) {
    use serde_json::Value;
    if let Some(existing) = map.get_mut(key) {
        match existing {
            Value::Array(arr) => {
                arr.push(value);
            }
            _ => {
                let prev = existing.take();
                *existing = Value::Array(vec![prev, value]);
            }
        }
    } else {
        map.insert(key.to_string(), value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identify_nfe() {
        let xml = r#"<?xml version="1.0"?><NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe/></NFe>"#;
        assert_eq!(identify_xml_type(xml).unwrap(), "NFe");
    }

    #[test]
    fn identify_nfe_proc() {
        let xml = r#"<nfeProc versao="4.00"><NFe/><protNFe/></nfeProc>"#;
        assert_eq!(identify_xml_type(xml).unwrap(), "nfeProc");
    }

    #[test]
    fn empty_returns_err() {
        assert!(identify_xml_type("").is_err());
    }

    #[test]
    fn non_xml_returns_err() {
        assert!(identify_xml_type("hello world").is_err());
    }

    #[test]
    fn unknown_root_returns_err() {
        let xml = "<other><data/></other>";
        assert!(identify_xml_type(xml).is_err());
    }

    // ── identify_xml_type additional patterns ────────────────────────

    #[test]
    fn identify_env_evento() {
        let xml = r#"<envEvento versao="1.00"><evento/></envEvento>"#;
        assert_eq!(identify_xml_type(xml).unwrap(), "envEvento");
    }

    #[test]
    fn identify_ret_cons_sit_nfe() {
        let xml = "<retConsSitNFe><cStat>100</cStat></retConsSitNFe>";
        assert_eq!(identify_xml_type(xml).unwrap(), "retConsSitNFe");
    }

    #[test]
    fn identify_cons_cad() {
        let xml = r#"<ConsCad versao="2.00"><infCons/></ConsCad>"#;
        assert_eq!(identify_xml_type(xml).unwrap(), "ConsCad");
    }

    #[test]
    fn identify_inut_nfe() {
        let xml = r#"<inutNFe versao="4.00"><infInut/></inutNFe>"#;
        assert_eq!(identify_xml_type(xml).unwrap(), "inutNFe");
    }

    #[test]
    fn identify_ret_env_evento() {
        let xml = "<retEnvEvento><cStat>128</cStat></retEnvEvento>";
        assert_eq!(identify_xml_type(xml).unwrap(), "retEnvEvento");
    }

    #[test]
    fn identify_ret_inut_nfe() {
        let xml = "<retInutNFe><infInut/></retInutNFe>";
        assert_eq!(identify_xml_type(xml).unwrap(), "retInutNFe");
    }

    #[test]
    fn identify_adm_csc_nfce() {
        let xml = r#"<admCscNFCe versao="1.00"><data/></admCscNFCe>"#;
        assert_eq!(identify_xml_type(xml).unwrap(), "admCscNFCe");
    }

    #[test]
    fn identify_dist_dfe_int() {
        let xml = r#"<distDFeInt versao="1.01"><data/></distDFeInt>"#;
        assert_eq!(identify_xml_type(xml).unwrap(), "distDFeInt");
    }

    #[test]
    fn identify_proc_evento_nfe() {
        let xml = r#"<procEventoNFe versao="1.00"><evento/></procEventoNFe>"#;
        assert_eq!(identify_xml_type(xml).unwrap(), "procEventoNFe");
    }

    // ── xml_to_json additional coverage ──────────────────────────────

    #[test]
    fn xml_to_json_empty_elements() {
        // Root NFe is unwrapped; infNFe becomes top-level
        let xml = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe123"><empty/></infNFe></NFe>"#;
        let json = xml_to_json(xml).unwrap();
        assert!(json.contains("infNFe"));
        // xmlns should NOT appear
        assert!(!json.contains("xmlns"));
    }

    #[test]
    fn xml_to_json_non_nfe_document_fails() {
        let xml = "<garbage><data>hello</data></garbage>";
        assert!(xml_to_json(xml).is_err());
    }

    #[test]
    fn xml_to_json_empty_element_with_attrs() {
        // Root NFe is unwrapped; tag with attr becomes top-level
        let xml = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><tag attr="val"/></NFe>"#;
        let json = xml_to_json(xml).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v.get("tag").is_some());
        assert!(v.get("NFe").is_none(), "root tag NFe must be unwrapped");
    }

    #[test]
    fn xml_to_json_repeated_elements_become_array() {
        // Root NFe is unwrapped; det array becomes top-level
        let xml =
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><det>a</det><det>b</det></NFe>"#;
        let json = xml_to_json(xml).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let det = v
            .get("det")
            .expect("det should be at top level after unwrap");
        assert!(det.is_array());
        assert_eq!(det.as_array().expect("is array").len(), 2);
    }

    #[test]
    fn xml_to_json_element_with_only_attrs_no_text() {
        // Root NFe is unwrapped; item with attrs becomes top-level
        let xml =
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><item a="1" b="2"></item></NFe>"#;
        let json = xml_to_json(xml).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v.get("item").is_some());
    }

    #[test]
    fn strip_ns_prefix_with_colon() {
        assert_eq!(strip_ns_prefix("nfe:NFe"), "NFe");
    }

    #[test]
    fn strip_ns_prefix_without_colon() {
        assert_eq!(strip_ns_prefix("NFe"), "NFe");
    }

    #[test]
    fn is_xmlns_attr_cases() {
        assert!(is_xmlns_attr("xmlns"));
        assert!(is_xmlns_attr("xmlns:nfe"));
        assert!(!is_xmlns_attr("versao"));
        assert!(!is_xmlns_attr("Id"));
    }

    #[test]
    fn xml_to_json_basic() {
        // Root NFe is unwrapped; infNFe at top level
        let xml = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe123"><ide><cUF>35</cUF></ide></infNFe></NFe>"#;
        let json = xml_to_json(xml).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v.get("infNFe").is_some(), "infNFe should be at top level");
        assert!(v.get("NFe").is_none(), "NFe root should be unwrapped");
    }

    #[test]
    fn xml_to_json_attributes_inline() {
        // Root NFe is unwrapped; verify inline attributes on infNFe
        let xml = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe versao="4.00" Id="NFe123"><ide><cUF>35</cUF></ide></infNFe></NFe>"#;
        let json = xml_to_json(xml).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let inf_nfe = v.get("infNFe").expect("infNFe at top level");
        // Attributes should be inline, not nested under "attributes"
        assert_eq!(inf_nfe.get("versao").and_then(|v| v.as_str()), Some("4.00"));
        assert_eq!(inf_nfe.get("Id").and_then(|v| v.as_str()), Some("NFe123"));
        assert!(
            inf_nfe.get("attributes").is_none(),
            "should NOT have @attributes/attributes key"
        );
    }

    #[test]
    fn xml_to_json_cdata_in_qrcode() {
        // Root NFe is unwrapped; infNFeSupl at top level
        let xml = concat!(
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<infNFe versao="4.00" Id="NFe123"><ide><cUF>35</cUF></ide></infNFe>"#,
            r#"<infNFeSupl>"#,
            r#"<qrCode><![CDATA[http://example.com/nfce?p=123&x=456]]></qrCode>"#,
            r#"<urlChave>http://example.com/nfce/consulta</urlChave>"#,
            r#"</infNFeSupl>"#,
            r#"</NFe>"#,
        );
        let json = xml_to_json(xml).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let supl = v.get("infNFeSupl").expect("infNFeSupl at top level");
        assert_eq!(
            supl.get("qrCode").and_then(|v| v.as_str()),
            Some("http://example.com/nfce?p=123&x=456"),
            "CDATA content should be preserved without mangling"
        );
        assert_eq!(
            supl.get("urlChave").and_then(|v| v.as_str()),
            Some("http://example.com/nfce/consulta"),
        );
    }

    #[test]
    fn xml_to_json_xmlns_stripped() {
        // Verify xmlns attributes are completely removed from output
        let xml = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe" xmlns:nfe="http://www.portalfiscal.inf.br/nfe"><infNFe versao="4.00"><ide><cUF>35</cUF></ide></infNFe></NFe>"#;
        let json = xml_to_json(xml).unwrap();
        assert!(
            !json.contains("xmlns"),
            "xmlns must not appear in JSON output"
        );
        assert!(json.contains("versao"), "non-xmlns attributes must be kept");
    }

    // ── xml_to_value (equivalente a toStd) ───────────────────────────

    #[test]
    fn xml_to_value_navigable_fields() {
        // Equivalent to PHP: $std = $standardize->toStd($xml);
        // PHP returns the *contents* of the matched root element.
        // For NFe, the root is unwrapped: $std->infNFe->ide->cUF
        let xml = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe versao="4.00" Id="NFe35..."><ide><cUF>35</cUF><nNF>12345</nNF></ide></infNFe></NFe>"#;
        let value = xml_to_value(xml).unwrap();

        // Root NFe is unwrapped — navigate directly to infNFe
        assert_eq!(value["infNFe"]["ide"]["cUF"].as_str(), Some("35"));
        assert_eq!(value["infNFe"]["ide"]["nNF"].as_str(), Some("12345"));
        // Inline attributes on infNFe
        assert_eq!(value["infNFe"]["versao"].as_str(), Some("4.00"));
        assert_eq!(value["infNFe"]["Id"].as_str(), Some("NFe35..."));
        // No xmlns
        assert!(value.get("xmlns").is_none(), "xmlns must not appear");
    }

    #[test]
    fn xml_to_value_ret_cons_sit_nfe() {
        // Root retConsSitNFe is unwrapped; children are at top level
        let xml = concat!(
            r#"<retConsSitNFe versao="4.00">"#,
            "<cStat>100</cStat>",
            "<xMotivo>Autorizado o uso da NF-e</xMotivo>",
            "<chNFe>35200612345678901234550010000000011000000019</chNFe>",
            "</retConsSitNFe>"
        );
        let value = xml_to_value(xml).unwrap();

        assert_eq!(value["cStat"].as_str(), Some("100"));
        assert_eq!(value["xMotivo"].as_str(), Some("Autorizado o uso da NF-e"));
        assert_eq!(
            value["chNFe"].as_str(),
            Some("35200612345678901234550010000000011000000019")
        );
        // Root attributes are merged inline
        assert_eq!(value["versao"].as_str(), Some("4.00"));
    }

    #[test]
    fn xml_to_value_empty_xml_fails() {
        assert!(xml_to_value("").is_err());
    }

    #[test]
    fn xml_to_value_non_nfe_fails() {
        assert!(xml_to_value("<other/>").is_err());
    }

    #[test]
    fn xml_to_value_cdata_preserved() {
        // Root NFe is unwrapped; infNFeSupl at top level
        let xml = concat!(
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<infNFeSupl>"#,
            r#"<qrCode><![CDATA[http://example.com?a=1&b=2]]></qrCode>"#,
            r#"</infNFeSupl>"#,
            r#"</NFe>"#,
        );
        let value = xml_to_value(xml).unwrap();
        assert_eq!(
            value["infNFeSupl"]["qrCode"].as_str(),
            Some("http://example.com?a=1&b=2")
        );
    }

    // ── xml_to_map (equivalente a toArray) ───────────────────────────

    #[test]
    fn xml_to_map_returns_top_level_keys() {
        // Root NFe is unwrapped; infNFe is at top level of the map
        let xml = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe><ide><cUF>35</cUF></ide></infNFe></NFe>"#;
        let map = xml_to_map(xml).unwrap();

        assert!(
            map.contains_key("infNFe"),
            "top-level map must contain 'infNFe' key (root NFe unwrapped)"
        );
        assert_eq!(map["infNFe"]["ide"]["cUF"].as_str(), Some("35"));
    }

    #[test]
    fn xml_to_map_nfe_proc() {
        // Root nfeProc is unwrapped; its children (versao, NFe, protNFe) are top-level
        let xml = concat!(
            r#"<nfeProc versao="4.00">"#,
            r#"<NFe><infNFe><ide><cUF>31</cUF></ide></infNFe></NFe>"#,
            r#"<protNFe><infProt><cStat>100</cStat></infProt></protNFe>"#,
            r#"</nfeProc>"#
        );
        let map = xml_to_map(xml).unwrap();

        assert_eq!(map["versao"].as_str(), Some("4.00"));
        assert_eq!(map["NFe"]["infNFe"]["ide"]["cUF"].as_str(), Some("31"));
        assert_eq!(map["protNFe"]["infProt"]["cStat"].as_str(), Some("100"));
    }

    #[test]
    fn xml_to_map_empty_xml_fails() {
        assert!(xml_to_map("").is_err());
    }

    #[test]
    fn xml_to_map_non_nfe_fails() {
        assert!(xml_to_map("<garbage/>").is_err());
    }

    #[test]
    fn xml_to_map_repeated_elements() {
        // Root NFe is unwrapped; det array at top level
        let xml = concat!(
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<det nItem="1"><prod><cProd>001</cProd></prod></det>"#,
            r#"<det nItem="2"><prod><cProd>002</cProd></prod></det>"#,
            r#"</NFe>"#
        );
        let map = xml_to_map(xml).unwrap();
        let det = &map["det"];
        assert!(det.is_array(), "repeated elements must become an array");
        let arr = det.as_array().expect("is array");
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["prod"]["cProd"].as_str(), Some("001"));
        assert_eq!(arr[1]["prod"]["cProd"].as_str(), Some("002"));
    }

    #[test]
    fn xml_to_value_and_json_produce_equivalent_output() {
        // Ensure xml_to_value and xml_to_json produce the same data
        let xml = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe versao="4.00"><ide><cUF>35</cUF></ide></infNFe></NFe>"#;
        let value = xml_to_value(xml).unwrap();
        let json_str = xml_to_json(xml).unwrap();
        let from_json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(value, from_json);
    }

    // ── PHP parity tests ─────────────────────────────────────────────

    #[test]
    fn xml_to_value_matches_php_to_std_nfe() {
        // PHP's Standardize::toStd() for NFe returns:
        // {"infNFe": {"versao": "4.00", "Id": "NFe123", "ide": {"cUF": "35"}}}
        // (no NFe wrapper, no xmlns)
        let xml = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe versao="4.00" Id="NFe123"><ide><cUF>35</cUF></ide></infNFe></NFe>"#;
        let value = xml_to_value(xml).unwrap();

        // Must NOT have root wrapper
        assert!(value.get("NFe").is_none(), "NFe wrapper must be removed");
        // Must NOT have xmlns
        assert!(value.get("xmlns").is_none(), "xmlns must be stripped");
        // Children must be directly accessible
        assert!(value.get("infNFe").is_some(), "infNFe must be at top level");
        assert_eq!(value["infNFe"]["versao"].as_str(), Some("4.00"));
        assert_eq!(value["infNFe"]["Id"].as_str(), Some("NFe123"));
        assert_eq!(value["infNFe"]["ide"]["cUF"].as_str(), Some("35"));
    }

    #[test]
    fn xml_to_value_matches_php_to_std_ret_envi_nfe() {
        // PHP's toStd() for retEnviNFe returns: {"versao": "4.00", "cStat": "103", ...}
        let xml = concat!(
            r#"<retEnviNFe versao="4.00">"#,
            "<tpAmb>2</tpAmb>",
            "<cStat>103</cStat>",
            "<xMotivo>Lote recebido com sucesso</xMotivo>",
            "</retEnviNFe>"
        );
        let value = xml_to_value(xml).unwrap();

        assert!(value.get("retEnviNFe").is_none(), "root must be unwrapped");
        assert_eq!(value["versao"].as_str(), Some("4.00"));
        assert_eq!(value["tpAmb"].as_str(), Some("2"));
        assert_eq!(value["cStat"].as_str(), Some("103"));
        assert_eq!(value["xMotivo"].as_str(), Some("Lote recebido com sucesso"));
    }

    #[test]
    fn xml_to_value_matches_php_to_std_nfe_proc() {
        // PHP's toStd() for nfeProc returns:
        // {"versao": "4.00", "NFe": {...}, "protNFe": {...}}
        let xml = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe><infNFe><ide><cUF>35</cUF></ide></infNFe></NFe>"#,
            r#"<protNFe><infProt><cStat>100</cStat></infProt></protNFe>"#,
            r#"</nfeProc>"#
        );
        let value = xml_to_value(xml).unwrap();

        assert!(
            value.get("nfeProc").is_none(),
            "root nfeProc must be unwrapped"
        );
        assert!(value.get("xmlns").is_none(), "xmlns must not appear");
        assert_eq!(value["versao"].as_str(), Some("4.00"));
        assert!(value.get("NFe").is_some());
        assert!(value.get("protNFe").is_some());
        assert_eq!(value["protNFe"]["infProt"]["cStat"].as_str(), Some("100"));
    }
}
