//! Private XML helper functions for SOAP envelope stripping and tag extraction.

/// Strip SOAP envelope (`<soap:Body>` or `<soapenv:Body>`) if present.
///
/// Also strips common namespace prefixes (`nfe:`, `nfeResultMsg:`) that some
/// SEFAZ endpoints add, so downstream tag extraction works with plain names.
pub(super) fn strip_soap_envelope(xml: &str) -> String {
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
pub(super) fn find_closing_tag_pos(xml: &str, local_name: &str) -> Option<usize> {
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
pub(super) fn extract_all_tag_values(xml: &str, tag_name: &str) -> Vec<String> {
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

/// Extract all occurrences of a raw tag (from `<tag ...>` to `</tag>` inclusive),
/// returning each as a standalone `String`. Handles namespace-prefixed variants.
pub(super) fn extract_all_raw_tags(xml: &str, local_name: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut search_from = 0;

    while search_from < xml.len() {
        let remaining = &xml[search_from..];
        let start = match find_opening_tag_pos(remaining, local_name) {
            Some(pos) => pos,
            None => break,
        };
        let abs_start = search_from + start;
        let after_open = match xml[abs_start..].find('>') {
            Some(pos) => abs_start + pos + 1,
            None => break,
        };

        let inner = &xml[after_open..];
        let close_rel = match find_closing_tag_pos(inner, local_name) {
            Some(pos) => pos,
            None => break,
        };
        let close_tag_end = match inner[close_rel..].find('>') {
            Some(pos) => close_rel + pos + 1,
            None => break,
        };

        results.push(xml[abs_start..after_open + close_tag_end].to_string());
        search_from = after_open + close_tag_end;
    }

    results
}

/// Extract the raw content between the opening and closing of a tag,
/// including any namespace-prefixed variant. Returns a slice of the
/// original string covering from `<tag ...>` to `</tag>` inclusive.
pub(super) fn extract_raw_tag(xml: &str, local_name: &str) -> Option<String> {
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
pub(super) fn find_opening_tag_pos(xml: &str, local_name: &str) -> Option<usize> {
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
pub(super) fn extract_inner_content<'a>(xml: &'a str, local_name: &str) -> Option<&'a str> {
    let content_start = find_tag_content_start(xml, local_name)?;
    let rest = &xml[content_start..];
    let end = find_closing_tag_pos(rest, local_name)?;
    Some(&rest[..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── find_closing_tag_pos with namespace prefix ──────────────────

    #[test]
    fn find_closing_tag_pos_namespaced() {
        let xml = "<nfe:Body>content</nfe:Body>";
        let pos = find_closing_tag_pos(xml, "Body");
        assert!(pos.is_some());
        assert_eq!(pos.unwrap(), 17);
    }

    #[test]
    fn find_closing_tag_pos_plain() {
        let xml = "<Body>content</Body>";
        let pos = find_closing_tag_pos(xml, "Body");
        assert!(pos.is_some());
    }

    #[test]
    fn find_closing_tag_pos_not_found() {
        let xml = "<Body>no closing tag";
        let pos = find_closing_tag_pos(xml, "Body");
        assert!(pos.is_none());
    }

    // ── find_opening_tag_pos ─────────────────────────────────────────

    #[test]
    fn find_opening_tag_pos_namespaced() {
        let xml = "<nfe:retConsStatServ><nfe:cStat>107</nfe:cStat></nfe:retConsStatServ>";
        let pos = find_opening_tag_pos(xml, "retConsStatServ");
        assert_eq!(pos, Some(0));
    }

    #[test]
    fn find_opening_tag_pos_skips_closing_tags() {
        let xml = "</close><open>data</open>";
        let pos = find_opening_tag_pos(xml, "open");
        assert!(pos.is_some());
    }

    // ── extract_raw_tag ────────────────────────────────────────────────

    #[test]
    fn extract_raw_tag_namespaced() {
        let xml = r#"<nfe:protNFe versao="4.00"><nfe:infProt><nfe:cStat>100</nfe:cStat></nfe:infProt></nfe:protNFe>"#;
        let raw = extract_raw_tag(xml, "protNFe");
        assert!(raw.is_some());
        assert!(raw.unwrap().contains("protNFe"));
    }
}
