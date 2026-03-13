//! Internal XML helper functions for extracting, parsing, and assembling XML fragments.

use crate::FiscalError;
use crate::constants::NFE_NAMESPACE;

pub(super) const DEFAULT_VERSION: &str = "4.00";

/// Remove `\n` and `\r` characters from a string.
///
/// Mirrors the PHP `str_replace(array("\n", "\r", "\s"), '', ...)` call
/// in `Complements::b2bTag`. The `\s` in PHP single-quoted strings is the
/// literal two-character sequence `\s`, not a regex; we replicate by also
/// removing it just in case, though it should never appear in valid XML.
pub(super) fn strip_newlines(s: &str) -> String {
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

/// Normalize the opening `<nfeProc>` tag so that `xmlns` comes before `versao`,
/// matching PHP DOMDocument canonical attribute ordering.
///
/// If the tag already has `xmlns` before `versao`, or if either attribute is
/// missing, the string is returned unchanged.
pub(super) fn normalize_nfe_proc_attrs(nfe_proc_xml: &str) -> String {
    let xmlns = extract_attribute(nfe_proc_xml, "nfeProc", "xmlns");
    let versao = extract_attribute(nfe_proc_xml, "nfeProc", "versao");

    if let (Some(xmlns_val), Some(versao_val)) = (xmlns, versao) {
        // Find the opening tag range
        let open_pattern = "<nfeProc";
        if let Some(start) = nfe_proc_xml.find(open_pattern) {
            if let Some(gt_offset) = nfe_proc_xml[start..].find('>') {
                let gt_pos = start + gt_offset;
                let old_opening = &nfe_proc_xml[start..=gt_pos];
                let new_opening =
                    format!("<nfeProc xmlns=\"{xmlns_val}\" versao=\"{versao_val}\">");
                if old_opening != new_opening {
                    return nfe_proc_xml.replacen(old_opening, &new_opening, 1);
                }
            }
        }
    }

    nfe_proc_xml.to_string()
}

/// Re-serialize an `<nfeProc>` XML in a way that matches PHP
/// `DOMDocument::saveXML()` output.
///
/// PHP DOM re-serialization does the following:
///
/// 1. Emits `<?xml version="1.0" encoding="..."?>` followed by a newline (`\n`).
///    The encoding is preserved from the original XML declaration; if absent,
///    the declaration is emitted without an encoding attribute.
/// 2. Reorders attributes on `<nfeProc>`: `xmlns` comes before `versao`
///    (DOM canonical ordering puts namespace declarations first).
/// 3. All other element content is preserved as-is.
///
/// If `extra_child` is `Some`, it is appended as the last child inside
/// `<nfeProc>` (before `</nfeProc>`).
pub(super) fn dom_reserialize_nfe_proc(
    nfe_proc_xml: &str,
    extra_child: Option<&str>,
) -> Result<String, FiscalError> {
    // 1. Extract the encoding from the original XML declaration (if any).
    let encoding = extract_xml_declaration_encoding(nfe_proc_xml);

    // 2. Build the XML declaration exactly as PHP DOMDocument::saveXML() does.
    let xml_decl = match &encoding {
        Some(enc) => format!("<?xml version=\"1.0\" encoding=\"{enc}\"?>"),
        None => "<?xml version=\"1.0\"?>".to_string(),
    };

    // 3. Extract the <nfeProc> tag attributes and body content.
    //    We need to rewrite the opening tag with xmlns before versao.
    let nfe_proc_full = extract_tag(nfe_proc_xml, "nfeProc")
        .ok_or_else(|| FiscalError::XmlParsing("Could not find <nfeProc> in NF-e XML".into()))?;

    // Extract versao and xmlns from the <nfeProc> opening tag
    let versao = extract_attribute(&nfe_proc_full, "nfeProc", "versao")
        .unwrap_or_else(|| DEFAULT_VERSION.to_string());
    let xmlns = extract_attribute(&nfe_proc_full, "nfeProc", "xmlns")
        .unwrap_or_else(|| NFE_NAMESPACE.to_string());

    // Extract the inner content of <nfeProc> (everything between the opening
    // and closing tags).
    let inner = extract_tag_inner_content(&nfe_proc_full, "nfeProc").ok_or_else(|| {
        FiscalError::XmlParsing("Could not extract <nfeProc> inner content".into())
    })?;

    // 4. Reassemble with PHP attribute order: xmlns first, then versao.
    let mut result = String::with_capacity(
        xml_decl.len() + 1 + 60 + inner.len() + extra_child.map_or(0, |c| c.len()) + 12,
    );
    result.push_str(&xml_decl);
    result.push('\n');
    result.push_str(&format!("<nfeProc xmlns=\"{xmlns}\" versao=\"{versao}\">"));
    result.push_str(inner);
    if let Some(child) = extra_child {
        result.push_str(child);
    }
    result.push_str("</nfeProc>\n");

    Ok(result)
}

/// Extract the `encoding` value from an XML declaration, if present.
///
/// Given `<?xml version="1.0" encoding="UTF-8"?>`, returns `Some("UTF-8")`.
/// Given `<?xml version="1.0"?>` or no declaration, returns `None`.
pub(super) fn extract_xml_declaration_encoding(xml: &str) -> Option<String> {
    let decl_start = xml.find("<?xml ")?;
    let decl_end = xml[decl_start..].find("?>")? + decl_start;
    let decl = &xml[decl_start..decl_end + 2];

    let enc_pat = "encoding=\"";
    let enc_start = decl.find(enc_pat)? + enc_pat.len();
    let enc_end = decl[enc_start..].find('"')? + enc_start;
    Some(decl[enc_start..enc_end].to_string())
}

/// Extract the inner content of an XML tag (everything between the end of the
/// opening tag `>` and the start of the closing tag `</tagName>`).
pub(super) fn extract_tag_inner_content<'a>(xml: &'a str, tag_name: &str) -> Option<&'a str> {
    let open_pattern = format!("<{tag_name}");
    let start = xml.find(&open_pattern)?;

    // Find end of opening tag
    let gt_pos = xml[start..].find('>')? + start;

    let close_tag = format!("</{tag_name}>");
    let close_pos = xml.rfind(&close_tag)?;

    if gt_pos + 1 > close_pos {
        return Some("");
    }

    Some(&xml[gt_pos + 1..close_pos])
}

/// Join two XML fragments into a versioned namespace wrapper element.
///
/// Produces:
/// ```xml
/// <?xml version="1.0" encoding="UTF-8"?>
/// <{node_name} versao="{version}" xmlns="{NFE_NAMESPACE}">
///   {first}{second}
/// </{node_name}>
/// ```
pub(super) fn join_xml(first: &str, second: &str, node_name: &str, version: &str) -> String {
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
pub(super) fn extract_tag(xml: &str, tag_name: &str) -> Option<String> {
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
pub(super) fn extract_all_tags(xml: &str, tag_name: &str) -> Vec<String> {
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
pub(super) fn extract_attribute(xml: &str, tag_name: &str, attr_name: &str) -> Option<String> {
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

/// Check if an XML string contains a given tag (with proper delimiter check).
pub(super) fn contains_xml_tag(xml: &str, tag_name: &str) -> bool {
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

/// Extract the access key from an `<infNFe Id="NFe...">` attribute.
/// Returns the 44-digit key (without the "NFe" prefix).
pub(super) fn extract_inf_nfe_id(xml: &str) -> Option<String> {
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

    #[test]
    fn extract_xml_declaration_encoding_works() {
        assert_eq!(
            extract_xml_declaration_encoding(r#"<?xml version="1.0" encoding="UTF-8"?><root/>"#),
            Some("UTF-8".to_string())
        );
        assert_eq!(
            extract_xml_declaration_encoding(r#"<?xml version="1.0" encoding="utf-8"?><root/>"#),
            Some("utf-8".to_string())
        );
        assert_eq!(
            extract_xml_declaration_encoding(r#"<?xml version="1.0"?><root/>"#),
            None
        );
        assert_eq!(extract_xml_declaration_encoding(r#"<root/>"#), None);
    }

    #[test]
    fn extract_tag_inner_content_works() {
        assert_eq!(
            extract_tag_inner_content(r#"<root attr="val">inner content</root>"#, "root"),
            Some("inner content")
        );
        assert_eq!(
            extract_tag_inner_content(r#"<root></root>"#, "root"),
            Some("")
        );
    }

    #[test]
    fn extract_all_tags_skips_prefix_match() {
        // "protNFeExtra" should NOT be matched when looking for "protNFe"
        let xml = "<root><protNFeExtra>bad</protNFeExtra><protNFe>good</protNFe></root>";
        let results = extract_all_tags(xml, "protNFe");
        assert_eq!(results.len(), 1);
        assert!(results[0].contains("good"));
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

    #[test]
    fn contains_xml_tag_at_end_of_string() {
        // Tag pattern at the very end, after >= xml.len() → true
        assert!(contains_xml_tag("<NFe", "NFe"));
    }

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
