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

/// Replace characters that are valid in XML but rejected by SEFAZ.
///
/// This is a **SEFAZ-level** sanitisation function, distinct from [`escape_xml`].
/// While `escape_xml` performs standard XML entity encoding, this function
/// mirrors the PHP `Strings::replaceUnacceptableCharacters` from `sped-common`:
///
/// 1. Remove `<` and `>`.
/// 2. Replace `&` with ` & ` (space-padded).
/// 3. Remove single quotes (`'`) and double quotes (`"`).
/// 4. Collapse multiple consecutive whitespace characters into a single space.
/// 5. Encode the remaining `&` as `&amp;`.
/// 6. Remove carriage return (`\r`), tab (`\t`), and line feed (`\n`).
/// 7. Collapse multiple whitespace again (from normalize step).
/// 8. Remove ASCII control characters (`0x00`–`0x1F`, `0x7F`), except space.
/// 9. Trim leading and trailing whitespace.
///
/// The function is designed to be called on user-provided field values
/// (e.g. `xJust`, `xCorrecao`, `xPag`) before they are placed into the
/// NF-e XML, so that the SEFAZ web-service will not reject the document
/// because of forbidden characters.
///
/// # Examples
///
/// ```
/// use fiscal_core::xml_utils::replace_unacceptable_characters;
/// assert_eq!(
///     replace_unacceptable_characters("Tom & Jerry <cats>"),
///     "Tom &amp; Jerry cats"
/// );
/// assert_eq!(
///     replace_unacceptable_characters("  hello   world  "),
///     "hello world"
/// );
/// ```
pub fn replace_unacceptable_characters(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    // Step 1: Remove < and >
    let s = input.replace(['<', '>'], "");

    // Step 2: Replace & with " & " (space-padded)
    let s = s.replace('&', " & ");

    // Step 3-4: Remove single quotes and double quotes
    let s = s.replace(['\'', '"'], "");

    // Step 5: Collapse multiple whitespace into single space
    let s = collapse_whitespace(&s);

    // Step 6: Encode & as &amp; (the only entity that can remain after steps 1-4)
    let s = s.replace('&', "&amp;");

    // Step 7: Remove \r, \t, \n (normalize)
    let s = s.replace(['\r', '\t', '\n'], "");

    // Step 8: Collapse multiple whitespace again (normalize)
    let s = collapse_whitespace(&s);

    // Step 9: Remove control characters (0x00-0x1F except space 0x20, and 0x7F)
    let s: String = s
        .chars()
        .filter(|&c| !c.is_ascii_control() || c == ' ')
        .collect();

    // Step 10: Trim
    s.trim().to_string()
}

/// Collapse runs of whitespace characters into a single ASCII space.
///
/// Equivalent to the PHP `preg_replace('/(?:\s\s+)/', ' ', …)` pattern used
/// throughout `sped-common`.
fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_ws = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_ws {
                result.push(' ');
            }
            prev_ws = true;
        } else {
            result.push(ch);
            prev_ws = false;
        }
    }
    result
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

/// Remove characters that are invalid in XML 1.0 documents.
///
/// Per the XML 1.0 specification (Section 2.2), the only valid characters are:
///
/// - `#x9` (tab), `#xA` (line feed), `#xD` (carriage return)
/// - `#x20`–`#xD7FF`
/// - `#xE000`–`#xFFFD`
/// - `#x10000`–`#x10FFFF`
///
/// All other characters (control characters `\x00`–`\x08`, `\x0B`–`\x0C`,
/// `\x0E`–`\x1F`, surrogates `\xD800`–`\xDFFF`, `\xFFFE`–`\xFFFF`) are
/// stripped from the output.
///
/// This mirrors the character-level cleaning portion of the PHP
/// `Strings::normalize()` function in `sped-common`.
///
/// # Examples
///
/// ```
/// use fiscal_core::xml_utils::remove_invalid_xml_chars;
/// assert_eq!(remove_invalid_xml_chars("hello\x00world"), "helloworld");
/// assert_eq!(remove_invalid_xml_chars("tab\there"), "tab\there");
/// assert_eq!(remove_invalid_xml_chars("line\nfeed"), "line\nfeed");
/// ```
pub fn remove_invalid_xml_chars(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for ch in input.chars() {
        if is_valid_xml_char(ch) {
            result.push(ch);
        }
    }
    result
}

/// Check whether a character is valid in XML 1.0 documents.
///
/// Valid characters per the XML 1.0 spec:
/// `#x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]`
fn is_valid_xml_char(ch: char) -> bool {
    matches!(ch,
        '\u{09}' | '\u{0A}' | '\u{0D}' |
        '\u{20}'..='\u{D7FF}' |
        '\u{E000}'..='\u{FFFD}' |
        '\u{10000}'..='\u{10FFFF}'
    )
}

/// Clean an XML string by removing namespace artifacts, collapsing inter-tag
/// whitespace, and optionally stripping the `<?xml … ?>` declaration.
///
/// This is a direct port of the PHP `Strings::clearXmlString()` from
/// `sped-common`. It performs the following transformations:
///
/// 1. Removes the `xmlns:default="http://www.w3.org/2000/09/xmldsig#"` attribute.
/// 2. Removes the `standalone="no"` attribute.
/// 3. Removes `default:` namespace prefixes and `:default` suffixes.
/// 4. Strips `\n`, `\r`, and `\t` characters.
/// 5. Collapses whitespace between adjacent XML tags (`> <` becomes `><`).
/// 6. If `remove_encoding_tag` is `true`, removes the `<?xml … ?>` declaration.
///
/// # Examples
///
/// ```
/// use fiscal_core::xml_utils::clear_xml_string;
///
/// let xml = "<root>\n  <child>text</child>\n</root>";
/// assert_eq!(clear_xml_string(xml, false), "<root><child>text</child></root>");
///
/// let xml2 = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><root><a>1</a></root>";
/// assert_eq!(clear_xml_string(xml2, true), "<root><a>1</a></root>");
/// ```
pub fn clear_xml_string(input: &str, remove_encoding_tag: bool) -> String {
    // Remove namespace artifacts and control whitespace (matches PHP $aFind array)
    let mut result = input.to_string();

    let removals = [
        "xmlns:default=\"http://www.w3.org/2000/09/xmldsig#\"",
        " standalone=\"no\"",
        "default:",
        ":default",
        "\n",
        "\r",
        "\t",
    ];
    for pattern in &removals {
        result = result.replace(pattern, "");
    }

    // Collapse whitespace between tags: >   < becomes ><
    // This replicates: preg_replace('/(\>)\s*(\<)/m', '$1$2', $retXml)
    let mut collapsed = String::with_capacity(result.len());
    let mut chars = result.chars().peekable();
    while let Some(ch) = chars.next() {
        collapsed.push(ch);
        if ch == '>' {
            // Skip whitespace until we hit '<' or a non-whitespace char
            let mut ws_buf = String::new();
            while let Some(&next) = chars.peek() {
                if next.is_ascii_whitespace() {
                    ws_buf.push(next);
                    chars.next();
                } else {
                    break;
                }
            }
            // If the next char after whitespace is '<', drop the whitespace
            // Otherwise, keep it
            if let Some(&next) = chars.peek() {
                if next != '<' {
                    collapsed.push_str(&ws_buf);
                }
            } else {
                // End of string; preserve trailing whitespace
                collapsed.push_str(&ws_buf);
            }
        }
    }
    result = collapsed;

    // Optionally remove <?xml ... ?> declaration
    if remove_encoding_tag {
        result = delete_all_between(&result, "<?xml", "?>");
    }

    result
}

/// Remove the first occurrence of text delimited by `beginning` and `end`
/// (inclusive of the delimiters).
///
/// Port of PHP `Strings::deleteAllBetween()`.
fn delete_all_between(input: &str, beginning: &str, end: &str) -> String {
    let begin_pos = match input.find(beginning) {
        Some(p) => p,
        None => return input.to_string(),
    };
    let after_begin = begin_pos + beginning.len();
    let end_pos = match input[after_begin..].find(end) {
        Some(p) => after_begin + p + end.len(),
        None => return input.to_string(),
    };
    let mut result = String::with_capacity(input.len() - (end_pos - begin_pos));
    result.push_str(&input[..begin_pos]);
    result.push_str(&input[end_pos..]);
    result
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

    // ── remove_invalid_xml_chars tests ──────────────────────────────────

    #[test]
    fn remove_invalid_xml_chars_preserves_valid_text() {
        assert_eq!(remove_invalid_xml_chars("Hello, World!"), "Hello, World!");
    }

    #[test]
    fn remove_invalid_xml_chars_preserves_tab_lf_cr() {
        // \x09 (tab), \x0A (line feed), \x0D (carriage return) are valid
        assert_eq!(
            remove_invalid_xml_chars("a\x09b\x0Ac\x0Dd"),
            "a\x09b\x0Ac\x0Dd"
        );
    }

    #[test]
    fn remove_invalid_xml_chars_strips_null_and_low_controls() {
        // \x00 through \x08 are invalid
        assert_eq!(
            remove_invalid_xml_chars("\x00\x01\x02\x03\x04\x05\x06\x07\x08hello"),
            "hello"
        );
    }

    #[test]
    fn remove_invalid_xml_chars_strips_0b_0c() {
        // \x0B (vertical tab) and \x0C (form feed) are invalid
        assert_eq!(remove_invalid_xml_chars("a\x0Bb\x0Cc"), "abc");
    }

    #[test]
    fn remove_invalid_xml_chars_strips_0e_to_1f() {
        // \x0E through \x1F are invalid
        let mut input = String::from("ok");
        for byte in 0x0Eu8..=0x1F {
            input.push(byte as char);
        }
        input.push_str("end");
        assert_eq!(remove_invalid_xml_chars(&input), "okend");
    }

    #[test]
    fn remove_invalid_xml_chars_strips_del() {
        // DEL (\x7F) is invalid — it falls outside the valid range
        // (it's > \x1F but not in \x20..=\xD7FF since \x7F is a control char,
        //  however by codepoint it IS in \x20..=\xD7FF so XML 1.0 actually
        //  allows it as a valid character).
        // Wait — XML 1.0 valid range includes #x20-#xD7FF, and \x7F = U+007F
        // is within that range. So DEL is technically valid in XML 1.0.
        // Our implementation follows the spec exactly.
        assert_eq!(remove_invalid_xml_chars("a\x7Fb"), "a\x7Fb");
    }

    #[test]
    fn remove_invalid_xml_chars_strips_fffe_ffff() {
        // U+FFFE and U+FFFF are invalid
        let input = format!("a{}b{}c", '\u{FFFE}', '\u{FFFF}');
        assert_eq!(remove_invalid_xml_chars(&input), "abc");
    }

    #[test]
    fn remove_invalid_xml_chars_preserves_bmp_and_supplementary() {
        // Valid BMP characters (accented, CJK, etc.)
        assert_eq!(
            remove_invalid_xml_chars("café résumé 日本語"),
            "café résumé 日本語"
        );
        // Valid supplementary plane characters (emoji, etc.)
        let input = "hello \u{1F600} world"; // U+1F600 is valid (in #x10000-#x10FFFF)
        assert_eq!(remove_invalid_xml_chars(input), input);
    }

    #[test]
    fn remove_invalid_xml_chars_preserves_private_use_area() {
        // U+E000-U+FFFD is valid
        let input = "a\u{E000}b\u{FFFD}c";
        assert_eq!(remove_invalid_xml_chars(input), input);
    }

    #[test]
    fn remove_invalid_xml_chars_empty_string() {
        assert_eq!(remove_invalid_xml_chars(""), "");
    }

    #[test]
    fn remove_invalid_xml_chars_all_invalid() {
        assert_eq!(remove_invalid_xml_chars("\x00\x01\x02\x03"), "");
    }

    #[test]
    fn remove_invalid_xml_chars_mixed_xml_content() {
        let input = "<tag>val\x00ue with \x0Bcontrol\x1F chars</tag>";
        assert_eq!(
            remove_invalid_xml_chars(input),
            "<tag>value with control chars</tag>"
        );
    }

    // ── clear_xml_string tests ──────────────────────────────────────────

    #[test]
    fn clear_xml_string_removes_whitespace_between_tags() {
        let xml = "<root>\n  <child>text</child>\n</root>";
        assert_eq!(
            clear_xml_string(xml, false),
            "<root><child>text</child></root>"
        );
    }

    #[test]
    fn clear_xml_string_removes_tabs_cr_lf() {
        let xml = "<a>\t<b>\r\n<c>val</c>\n</b>\n</a>";
        assert_eq!(clear_xml_string(xml, false), "<a><b><c>val</c></b></a>");
    }

    #[test]
    fn clear_xml_string_removes_default_namespace() {
        // Note: removing the xmlns attribute leaves a trailing space before '>',
        // matching PHP str_replace behaviour exactly.
        let xml = "<Signature xmlns:default=\"http://www.w3.org/2000/09/xmldsig#\"><default:SignedInfo>data</default:SignedInfo></Signature>";
        assert_eq!(
            clear_xml_string(xml, false),
            "<Signature ><SignedInfo>data</SignedInfo></Signature>"
        );
    }

    #[test]
    fn clear_xml_string_removes_standalone_no() {
        let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?><root/>";
        assert_eq!(
            clear_xml_string(xml, false),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><root/>"
        );
    }

    #[test]
    fn clear_xml_string_removes_encoding_tag() {
        let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><root><a>1</a></root>";
        assert_eq!(clear_xml_string(xml, true), "<root><a>1</a></root>");
    }

    #[test]
    fn clear_xml_string_preserves_without_encoding_tag() {
        let xml = "<?xml version=\"1.0\"?><root><a>1</a></root>";
        assert_eq!(
            clear_xml_string(xml, false),
            "<?xml version=\"1.0\"?><root><a>1</a></root>"
        );
    }

    #[test]
    fn clear_xml_string_no_encoding_tag_present() {
        let xml = "<root><a>1</a></root>";
        assert_eq!(clear_xml_string(xml, true), "<root><a>1</a></root>");
    }

    #[test]
    fn clear_xml_string_empty_input() {
        assert_eq!(clear_xml_string("", false), "");
        assert_eq!(clear_xml_string("", true), "");
    }

    #[test]
    fn clear_xml_string_preserves_text_content_spaces() {
        // Spaces inside text content (not between tags) should be preserved
        let xml = "<tag>hello world</tag>";
        assert_eq!(clear_xml_string(xml, false), "<tag>hello world</tag>");
    }

    #[test]
    fn clear_xml_string_collapses_multiple_spaces_between_tags() {
        let xml = "<a>   <b>text</b>   </a>";
        assert_eq!(clear_xml_string(xml, false), "<a><b>text</b></a>");
    }

    #[test]
    fn clear_xml_string_removes_colon_default_suffix() {
        let xml = "<Signature:default><data/></Signature:default>";
        assert_eq!(
            clear_xml_string(xml, false),
            "<Signature><data/></Signature>"
        );
    }

    // ----- replace_unacceptable_characters tests -----

    #[test]
    fn replace_unacceptable_empty() {
        assert_eq!(replace_unacceptable_characters(""), "");
    }

    #[test]
    fn replace_unacceptable_plain_text() {
        assert_eq!(
            replace_unacceptable_characters("Venda de mercadorias"),
            "Venda de mercadorias"
        );
    }

    #[test]
    fn replace_unacceptable_removes_angle_brackets() {
        assert_eq!(replace_unacceptable_characters("foo<bar>baz"), "foobarbaz");
    }

    #[test]
    fn replace_unacceptable_ampersand_encoding() {
        assert_eq!(replace_unacceptable_characters("A&B"), "A &amp; B");
    }

    #[test]
    fn replace_unacceptable_removes_quotes() {
        assert_eq!(
            replace_unacceptable_characters(r#"It's a "test""#),
            "Its a test"
        );
    }

    #[test]
    fn replace_unacceptable_collapses_whitespace() {
        assert_eq!(
            replace_unacceptable_characters("hello    world"),
            "hello world"
        );
    }

    #[test]
    fn replace_unacceptable_trims() {
        assert_eq!(replace_unacceptable_characters("  hello  "), "hello");
    }

    #[test]
    fn replace_unacceptable_removes_control_chars() {
        assert_eq!(
            replace_unacceptable_characters("abc\x00\x01\x02def"),
            "abcdef"
        );
    }

    #[test]
    fn replace_unacceptable_removes_cr_lf_tab() {
        assert_eq!(
            replace_unacceptable_characters("line1\r\n\tline2"),
            "line1 line2"
        );
    }

    #[test]
    fn replace_unacceptable_combined() {
        assert_eq!(
            replace_unacceptable_characters(
                "  Cancelamento <por>  erro & \"duplicidade\"  na emissão\t\n  "
            ),
            "Cancelamento por erro &amp; duplicidade na emissão"
        );
    }

    #[test]
    fn replace_unacceptable_ampersand_already_spaced() {
        assert_eq!(replace_unacceptable_characters("A & B"), "A &amp; B");
    }

    #[test]
    fn replace_unacceptable_multiple_ampersands() {
        assert_eq!(
            replace_unacceptable_characters("A&B&C"),
            "A &amp; B &amp; C"
        );
    }

    #[test]
    fn replace_unacceptable_preserves_accented_chars() {
        assert_eq!(
            replace_unacceptable_characters("São Paulo — café"),
            "São Paulo — café"
        );
    }

    #[test]
    fn replace_unacceptable_only_special_chars() {
        assert_eq!(replace_unacceptable_characters("<>\"'"), "");
    }

    #[test]
    fn replace_unacceptable_del_char() {
        assert_eq!(replace_unacceptable_characters("abc\x7Fdef"), "abcdef");
    }
}
