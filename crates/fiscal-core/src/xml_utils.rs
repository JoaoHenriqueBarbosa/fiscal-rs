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
