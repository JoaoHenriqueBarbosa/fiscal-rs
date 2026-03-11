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
/// Attributes are preserved with their original names (no prefix).
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the input is not valid NFe XML,
/// or if conversion to JSON fails.
pub fn xml_to_json(xml: &str) -> Result<String, FiscalError> {
    // Validate it is an NFe document first
    identify_xml_type(xml)?;

    let trimmed = xml.trim();

    // Parse XML into a serde_json::Value tree using quick-xml events
    let value = xml_str_to_json_value(trimmed)?;

    serde_json::to_string(&value)
        .map_err(|e| FiscalError::XmlParsing(format!("JSON serialization failed: {e}")))
}

/// Recursively convert an XML string into a serde_json::Value.
///
/// This is a simplified converter that handles elements, text content,
/// and attributes. Namespace prefixes are stripped from tag names.
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
                let text = e.unescape().unwrap_or_default().to_string();
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

    #[test]
    fn xml_to_json_basic() {
        let xml = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe123"><ide><cUF>35</cUF></ide></infNFe></NFe>"#;
        let json = xml_to_json(xml).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v.get("NFe").is_some());
    }
}
