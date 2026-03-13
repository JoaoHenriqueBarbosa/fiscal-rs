//! XML Canonicalization (C14N 1.0) and XML element helpers.

use fiscal_core::FiscalError;

/// Extract the `Id` attribute value from the first occurrence of `<tag_name ... Id="...">`.
pub(super) fn extract_element_id(xml: &str, tag_name: &str) -> Result<String, FiscalError> {
    let pattern = format!("<{tag_name}");
    let tag_start = xml.find(&pattern).ok_or_else(|| {
        FiscalError::Certificate(format!(
            "Could not find <{tag_name}> element with Id attribute in XML"
        ))
    })?;

    let rest = &xml[tag_start..];
    // Find the closing > of this opening tag
    let tag_end = rest
        .find('>')
        .ok_or_else(|| FiscalError::Certificate(format!("<{tag_name}> tag is malformed")))?;

    let tag_content = &rest[..tag_end];

    // Find Id="..."
    let id_prefix = "Id=\"";
    let id_start = tag_content.find(id_prefix).ok_or_else(|| {
        FiscalError::Certificate(format!(
            "Could not find <{tag_name}> element with Id attribute in XML"
        ))
    })?;

    let id_value_start = id_start + id_prefix.len();
    let id_value_end = tag_content[id_value_start..].find('"').ok_or_else(|| {
        FiscalError::Certificate(format!("Malformed Id attribute in <{tag_name}>"))
    })?;

    Ok(tag_content[id_value_start..id_value_start + id_value_end].to_string())
}

/// Ensure the signed element includes inherited xmlns from ancestor elements.
///
/// In C14N inclusive canonicalization, the root element of the subset must
/// include all in-scope namespace declarations from ancestors. If `<infNFe>`
/// doesn't explicitly declare `xmlns` but the parent `<NFe>` does, we add it.
/// This matches PHP DOMDocument's C14N behavior.
pub(super) fn ensure_inherited_namespace(element: &str, full_xml: &str, tag_name: &str) -> String {
    // Check if the element already has xmlns
    let open_end = element.find('>').unwrap_or(element.len());
    let open_tag = &element[..open_end];
    if open_tag.contains("xmlns=") {
        return element.to_string();
    }

    // Find xmlns from the closest ancestor in the full XML
    let tag_pos = full_xml.find(&format!("<{tag_name}")).unwrap_or(0);
    let before = &full_xml[..tag_pos];

    // Search backwards for xmlns="..." in ancestor tags
    if let Some(ns_start) = before.rfind("xmlns=\"") {
        let ns_val_start = ns_start + 7; // skip xmlns="
        if let Some(ns_val_end) = full_xml[ns_val_start..].find('"') {
            let ns_value = &full_xml[ns_val_start..ns_val_start + ns_val_end];
            // Insert xmlns after the tag name
            let insert_pos = element
                .find(|c: char| c.is_ascii_whitespace() || c == '>')
                .unwrap_or(open_end);
            return format!(
                "{} xmlns=\"{ns_value}\"{}",
                &element[..insert_pos],
                &element[insert_pos..],
            );
        }
    }

    element.to_string()
}

/// Extract the full element (from `<tag_name ...>` to `</tag_name>`) from the XML.
pub(super) fn extract_element(xml: &str, tag_name: &str) -> Option<String> {
    let open_pattern = format!("<{tag_name}");
    let close_pattern = format!("</{tag_name}>");

    let start = xml.find(&open_pattern)?;
    let end = xml.find(&close_pattern)? + close_pattern.len();

    Some(xml[start..end].to_string())
}

/// Remove any `<Signature ...>...</Signature>` from the XML string (enveloped-signature transform).
pub(super) fn remove_signature_element(xml: &str) -> String {
    if let Some(sig_start) = xml.find("<Signature") {
        if let Some(sig_end_tag) = xml[sig_start..].find("</Signature>") {
            let sig_end = sig_start + sig_end_tag + "</Signature>".len();
            return format!("{}{}", &xml[..sig_start], &xml[sig_end..]);
        }
    }
    xml.to_string()
}

/// XML Canonicalization (C14N 1.0 without comments).
///
/// Implements the subset of Canonical XML 1.0 needed for NF-e signing:
/// - Removes the XML declaration (`<?xml ...?>`)
/// - Sorts attributes on each opening tag: namespace declarations first
///   (sorted by prefix, default namespace first), then regular attributes
///   sorted alphabetically by local name
/// - Expands self-closing tags (`<foo/>` → `<foo></foo>`)
pub(super) fn canonicalize_xml(xml: &str) -> String {
    let mut input = xml;

    // Remove XML declaration if present
    if let Some(decl_start) = input.find("<?xml") {
        if let Some(decl_end) = input[decl_start..].find("?>") {
            input = input[decl_start + decl_end + 2..].trim_start();
        }
    }

    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            // Collect everything up to '>'
            let mut tag = String::from('<');
            for c in chars.by_ref() {
                tag.push(c);
                if c == '>' {
                    break;
                }
            }

            // Skip processing instructions, closing tags, comments, CDATA
            if tag.starts_with("</") || tag.starts_with("<?") || tag.starts_with("<!") {
                result.push_str(&tag);
                continue;
            }

            // Opening tag — sort attributes
            let self_closing = tag.ends_with("/>");
            let tag_content = if self_closing {
                &tag[1..tag.len() - 2] // strip < and />
            } else {
                &tag[1..tag.len() - 1] // strip < and >
            };

            let (tag_name, attrs_str) = match tag_content.find(|c: char| c.is_ascii_whitespace()) {
                Some(pos) => (&tag_content[..pos], tag_content[pos..].trim()),
                None => (tag_content, ""),
            };

            if attrs_str.is_empty() {
                if self_closing {
                    // C14N expands self-closing to <tag></tag>
                    result.push('<');
                    result.push_str(tag_name);
                    result.push_str("></");
                    result.push_str(tag_name);
                    result.push('>');
                } else {
                    result.push_str(&tag);
                }
                continue;
            }

            // Parse attributes
            let attrs = parse_attributes(attrs_str);

            // Separate namespace declarations from regular attributes
            let mut ns_attrs: Vec<(&str, &str)> = Vec::new();
            let mut reg_attrs: Vec<(&str, &str)> = Vec::new();

            for (name, value) in &attrs {
                if *name == "xmlns" || name.starts_with("xmlns:") {
                    ns_attrs.push((name, value));
                } else {
                    reg_attrs.push((name, value));
                }
            }

            // Sort namespace declarations: default namespace first, then by prefix
            ns_attrs.sort_by(|a, b| match (a.0, b.0) {
                ("xmlns", _) => std::cmp::Ordering::Less,
                (_, "xmlns") => std::cmp::Ordering::Greater,
                _ => a.0.cmp(b.0),
            });

            // Sort regular attributes by local name
            reg_attrs.sort_by(|a, b| a.0.cmp(b.0));

            // Rebuild tag
            result.push('<');
            result.push_str(tag_name);
            for (name, value) in ns_attrs.iter().chain(reg_attrs.iter()) {
                result.push(' ');
                result.push_str(name);
                result.push_str("=\"");
                result.push_str(value);
                result.push('"');
            }
            if self_closing {
                result.push_str("></");
                result.push_str(tag_name);
                result.push('>');
            } else {
                result.push('>');
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Parse attributes from a tag's attribute string.
///
/// Returns a vector of (name, value) pairs. Handles both single and double
/// quoted attribute values.
pub(super) fn parse_attributes(attrs_str: &str) -> Vec<(&str, &str)> {
    let mut attrs = Vec::new();
    let mut remaining = attrs_str.trim();

    while !remaining.is_empty() {
        // Find attribute name (up to '=')
        let eq_pos = match remaining.find('=') {
            Some(pos) => pos,
            None => break,
        };
        let name = remaining[..eq_pos].trim();
        remaining = remaining[eq_pos + 1..].trim();

        // Find quoted value
        let quote = match remaining.chars().next() {
            Some(q @ ('"' | '\'')) => q,
            _ => break,
        };
        remaining = &remaining[1..]; // skip opening quote
        let end_pos = match remaining.find(quote) {
            Some(pos) => pos,
            None => break,
        };
        let value = &remaining[..end_pos];
        remaining = remaining[end_pos + 1..].trim();

        attrs.push((name, value));
    }

    attrs
}
