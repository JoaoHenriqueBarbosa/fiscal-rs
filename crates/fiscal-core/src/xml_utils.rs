//! Low-level XML building primitives used throughout the crate.
//!
//! These utilities are deliberately simple and allocation-efficient: they work
//! on `&str` slices and return owned `String`s, with no external XML library
//! dependency.

/// Escape special XML characters in text content and attribute values,
/// replacing `&`, `<`, `>`, `"`, and `'` with their XML entity equivalents.
///
/// # Examples
///
/// ```
/// use fiscal_core::xml_utils::escape_xml;
/// assert_eq!(escape_xml("Tom & Jerry <cats>"), "Tom &amp; Jerry &lt;cats&gt;");
/// ```
pub fn escape_xml(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            c => result.push(c),
        }
    }
    result
}

/// Extract the text content of the first occurrence of a simple XML tag in a
/// raw XML string.
///
/// Searches for `<tag_name>…</tag_name>` and returns the inner text.  Does not
/// handle namespaced tags, nested tags of the same name, or CDATA sections.
///
/// Returns `None` if the tag is absent.
///
/// # Examples
///
/// ```
/// use fiscal_core::xml_utils::extract_xml_tag_value;
/// let xml = "<root><cStat>100</cStat></root>";
/// assert_eq!(extract_xml_tag_value(xml, "cStat"), Some("100".to_string()));
/// assert_eq!(extract_xml_tag_value(xml, "missing"), None);
/// ```
pub fn extract_xml_tag_value(xml: &str, tag_name: &str) -> Option<String> {
    let open = format!("<{tag_name}>");
    let close = format!("</{tag_name}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml[start..end].to_string())
}

/// Build an XML tag with optional attributes and children.
///
/// If children is a string, it is escaped. If children is an array
/// of pre-built strings, they are concatenated as-is.
pub fn tag(name: &str, attrs: &[(&str, &str)], children: TagContent<'_>) -> String {
    let attr_str: String = attrs
        .iter()
        .map(|(k, v)| format!(" {k}=\"{}\"", escape_xml(v)))
        .collect();

    match children {
        TagContent::None => format!("<{name}{attr_str}></{name}>"),
        TagContent::Text(text) => {
            format!("<{name}{attr_str}>{}</{name}>", escape_xml(text))
        }
        TagContent::Children(kids) => {
            let inner: String = kids.into_iter().collect();
            format!("<{name}{attr_str}>{inner}</{name}>")
        }
    }
}

/// Content variants for the [`tag`] builder function.
///
/// Use [`TagContent::None`] for self-closing elements, [`TagContent::Text`]
/// for text nodes (automatically XML-escaped), and [`TagContent::Children`]
/// for pre-built child element strings.
#[non_exhaustive]
pub enum TagContent<'a> {
    /// Empty element: `<name></name>`.
    None,
    /// Text content (will be XML-escaped): `<name>text</name>`.
    Text(&'a str),
    /// Pre-built child elements concatenated verbatim: `<name><a/><b/></name>`.
    Children(Vec<String>),
}

impl<'a> From<&'a str> for TagContent<'a> {
    fn from(s: &'a str) -> Self {
        TagContent::Text(s)
    }
}

impl From<Vec<String>> for TagContent<'_> {
    fn from(v: Vec<String>) -> Self {
        TagContent::Children(v)
    }
}

impl From<String> for TagContent<'_> {
    fn from(s: String) -> Self {
        TagContent::Text(Box::leak(s.into_boxed_str()))
    }
}

/// Pretty-print an XML string by adding indentation.
///
/// This is a lightweight formatter that does not parse XML semantically --
/// it works by splitting on `<` / `>` boundaries and inserting newlines and
/// indentation. Suitable for debugging/display purposes. Equivalent to the
/// PHP `FakePretty::prettyPrint` formatting behaviour (via DOMDocument::formatOutput).
///
/// # Examples
///
/// ```
/// use fiscal_core::xml_utils::pretty_print_xml;
/// let compact = "<root><child>text</child></root>";
/// let pretty = pretty_print_xml(compact);
/// assert!(pretty.contains("  <child>"));
/// ```
pub fn pretty_print_xml(xml: &str) -> String {
    // Tokenise into tags and text segments
    let mut tokens: Vec<XmlToken> = Vec::new();
    let mut pos = 0;
    let bytes = xml.as_bytes();

    while pos < bytes.len() {
        if bytes[pos] == b'<' {
            // Find end of tag
            let end = xml[pos..]
                .find('>')
                .map(|i| pos + i + 1)
                .unwrap_or(bytes.len());
            tokens.push(XmlToken::Tag(xml[pos..end].to_string()));
            pos = end;
        } else {
            // Text until next '<'
            let end = xml[pos..].find('<').map(|i| pos + i).unwrap_or(bytes.len());
            let text = &xml[pos..end];
            if !text.trim().is_empty() {
                tokens.push(XmlToken::Text(text.trim().to_string()));
            }
            pos = end;
        }
    }

    // Now render with indentation
    let indent = "  ";
    let mut result = String::with_capacity(xml.len() * 2);
    let mut depth: usize = 0;

    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            XmlToken::Tag(t) if t.starts_with("<?") => {
                // XML declaration
                result.push_str(t);
                result.push('\n');
            }
            XmlToken::Tag(t) if t.starts_with("</") => {
                // Closing tag
                depth = depth.saturating_sub(1);
                for _ in 0..depth {
                    result.push_str(indent);
                }
                result.push_str(t);
                result.push('\n');
            }
            XmlToken::Tag(t) if t.ends_with("/>") => {
                // Self-closing tag
                for _ in 0..depth {
                    result.push_str(indent);
                }
                result.push_str(t);
                result.push('\n');
            }
            XmlToken::Tag(t) => {
                // Opening tag -- check if next token is Text followed by closing tag
                if i + 2 < tokens.len() {
                    if let (XmlToken::Text(text), XmlToken::Tag(close)) =
                        (&tokens[i + 1], &tokens[i + 2])
                    {
                        if close.starts_with("</") {
                            // Inline text element: <tag>text</tag>
                            for _ in 0..depth {
                                result.push_str(indent);
                            }
                            result.push_str(t);
                            result.push_str(text);
                            result.push_str(close);
                            result.push('\n');
                            i += 3;
                            continue;
                        }
                    }
                }
                for _ in 0..depth {
                    result.push_str(indent);
                }
                result.push_str(t);
                result.push('\n');
                depth += 1;
            }
            XmlToken::Text(t) => {
                // Standalone text (unusual)
                for _ in 0..depth {
                    result.push_str(indent);
                }
                result.push_str(t);
                result.push('\n');
            }
        }
        i += 1;
    }

    // Remove trailing newline
    while result.ends_with('\n') {
        result.pop();
    }
    result
}

/// Internal token type for XML pretty-printing.
enum XmlToken {
    Tag(String),
    Text(String),
}

/// Validate an NF-e XML string by checking for the presence of required tags.
///
/// This is a lightweight structural validator that checks for mandatory tags
/// in the NF-e/NFC-e XML. It does **not** perform full XSD schema validation
/// (which would require shipping XSD files and a full XML schema parser), but
/// covers the most common errors that would cause SEFAZ rejection.
///
/// Validated items:
/// - Required root structure (`<NFe>`, `<infNFe>`)
/// - Required `<ide>` fields (cUF, cNF, natOp, mod, serie, nNF, dhEmi, tpNF, etc.)
/// - Required `<emit>` fields (CNPJ/CPF, xNome, enderEmit, IE, CRT)
/// - Required `<det>` with at least one item
/// - Required `<total>` / `<ICMSTot>`
/// - Required `<transp>` and `<pag>`
/// - Access key format (44 digits)
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] with a description of all missing tags.
///
/// # Examples
///
/// ```
/// use fiscal_core::xml_utils::validate_xml;
/// let xml = "<NFe><infNFe>...</infNFe></NFe>";
/// // Will return an error listing all missing required tags
/// assert!(validate_xml(xml).is_err());
/// ```
pub fn validate_xml(xml: &str) -> Result<(), crate::FiscalError> {
    let mut errors: Vec<String> = Vec::new();

    // Check root structure
    let required_structure = [
        ("NFe", "Elemento raiz <NFe> ausente"),
        ("infNFe", "Elemento <infNFe> ausente"),
    ];
    for (tag_name, msg) in &required_structure {
        if !xml.contains(&format!("<{tag_name}")) {
            errors.push(msg.to_string());
        }
    }

    // Check IDE required tags
    let ide_tags = [
        "cUF", "cNF", "natOp", "mod", "serie", "nNF", "dhEmi", "tpNF", "idDest", "cMunFG", "tpImp",
        "tpEmis", "cDV", "tpAmb", "finNFe", "indFinal", "indPres", "procEmi", "verProc",
    ];
    for tag_name in &ide_tags {
        if extract_xml_tag_value(xml, tag_name).is_none() {
            errors.push(format!("Tag obrigatória <{tag_name}> ausente em <ide>"));
        }
    }

    // Check emit required tags
    let emit_required = ["xNome", "IE", "CRT"];
    for tag_name in &emit_required {
        if extract_xml_tag_value(xml, tag_name).is_none() {
            errors.push(format!("Tag obrigatória <{tag_name}> ausente em <emit>"));
        }
    }
    // CNPJ or CPF must be present
    if extract_xml_tag_value(xml, "CNPJ").is_none() && extract_xml_tag_value(xml, "CPF").is_none() {
        errors.push("Tag <CNPJ> ou <CPF> ausente em <emit>".to_string());
    }

    // Check required blocks
    let required_blocks = [
        ("enderEmit", "Bloco <enderEmit> ausente"),
        ("det ", "Nenhum item <det> encontrado"),
        ("total", "Bloco <total> ausente"),
        ("ICMSTot", "Bloco <ICMSTot> ausente"),
        ("transp", "Bloco <transp> ausente"),
        ("pag", "Bloco <pag> ausente"),
    ];
    for (fragment, msg) in &required_blocks {
        if !xml.contains(&format!("<{fragment}")) {
            errors.push(msg.to_string());
        }
    }

    // Validate access key format (44 digits) from infNFe Id attribute
    if let Some(id_start) = xml.find("Id=\"NFe") {
        let after_id = &xml[id_start + 7..];
        if let Some(quote_end) = after_id.find('"') {
            let key = &after_id[..quote_end];
            if key.len() != 44 || !key.chars().all(|c| c.is_ascii_digit()) {
                errors.push(format!(
                    "Chave de acesso inválida: esperado 44 dígitos, encontrado '{key}'"
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(crate::FiscalError::XmlParsing(errors.join("; ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pretty_print_simple_xml() {
        let compact = "<root><child>text</child></root>";
        let pretty = pretty_print_xml(compact);
        assert!(pretty.contains("<root>"));
        assert!(pretty.contains("  <child>text</child>"));
        assert!(pretty.contains("</root>"));
    }

    #[test]
    fn pretty_print_nested_xml() {
        let compact = "<a><b><c>val</c></b></a>";
        let pretty = pretty_print_xml(compact);
        let lines: Vec<&str> = pretty.lines().collect();
        assert_eq!(lines[0], "<a>");
        assert_eq!(lines[1], "  <b>");
        assert_eq!(lines[2], "    <c>val</c>");
        assert_eq!(lines[3], "  </b>");
        assert_eq!(lines[4], "</a>");
    }

    #[test]
    fn pretty_print_with_declaration() {
        let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><root><a>1</a></root>";
        let pretty = pretty_print_xml(xml);
        assert!(pretty.starts_with("<?xml"));
        assert!(pretty.contains("  <a>1</a>"));
    }

    #[test]
    fn pretty_print_empty_input() {
        let pretty = pretty_print_xml("");
        assert_eq!(pretty, "");
    }

    #[test]
    fn validate_xml_valid_nfe() {
        let xml = concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
            "<ide><cUF>41</cUF><cNF>12345678</cNF><natOp>VENDA</natOp>",
            "<mod>55</mod><serie>1</serie><nNF>123</nNF>",
            "<dhEmi>2026-03-11T10:30:00-03:00</dhEmi>",
            "<tpNF>1</tpNF><idDest>1</idDest><cMunFG>4106902</cMunFG>",
            "<tpImp>1</tpImp><tpEmis>1</tpEmis><cDV>0</cDV>",
            "<tpAmb>2</tpAmb><finNFe>1</finNFe><indFinal>1</indFinal>",
            "<indPres>1</indPres><procEmi>0</procEmi><verProc>1.0</verProc></ide>",
            "<emit><CNPJ>04123456000190</CNPJ><xNome>Test</xNome>",
            "<enderEmit><xLgr>Rua</xLgr></enderEmit>",
            "<IE>9012345678</IE><CRT>3</CRT></emit>",
            "<det nItem=\"1\"><prod><cProd>001</cProd></prod></det>",
            "<total><ICMSTot><vNF>150.00</vNF></ICMSTot></total>",
            "<transp><modFrete>9</modFrete></transp>",
            "<pag><detPag><tPag>01</tPag><vPag>150.00</vPag></detPag></pag>",
            "</infNFe></NFe>",
        );
        assert!(validate_xml(xml).is_ok());
    }

    #[test]
    fn validate_xml_missing_tags() {
        let xml = "<root><something>val</something></root>";
        let err = validate_xml(xml).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("NFe"));
        assert!(msg.contains("infNFe"));
    }

    #[test]
    fn validate_xml_invalid_access_key() {
        let xml = concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe123">"#,
            "<ide><cUF>41</cUF><cNF>12345678</cNF><natOp>VENDA</natOp>",
            "<mod>55</mod><serie>1</serie><nNF>123</nNF>",
            "<dhEmi>2026-03-11T10:30:00-03:00</dhEmi>",
            "<tpNF>1</tpNF><idDest>1</idDest><cMunFG>4106902</cMunFG>",
            "<tpImp>1</tpImp><tpEmis>1</tpEmis><cDV>0</cDV>",
            "<tpAmb>2</tpAmb><finNFe>1</finNFe><indFinal>1</indFinal>",
            "<indPres>1</indPres><procEmi>0</procEmi><verProc>1.0</verProc></ide>",
            "<emit><CNPJ>04123456000190</CNPJ><xNome>Test</xNome>",
            "<enderEmit><xLgr>Rua</xLgr></enderEmit>",
            "<IE>9012345678</IE><CRT>3</CRT></emit>",
            "<det nItem=\"1\"><prod><cProd>001</cProd></prod></det>",
            "<total><ICMSTot><vNF>150.00</vNF></ICMSTot></total>",
            "<transp><modFrete>9</modFrete></transp>",
            "<pag><detPag><tPag>01</tPag><vPag>150.00</vPag></detPag></pag>",
            "</infNFe></NFe>",
        );
        let err = validate_xml(xml).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Chave de acesso"));
    }
}
