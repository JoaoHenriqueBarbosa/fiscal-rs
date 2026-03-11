/// Escape special XML characters in text content and attribute values
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

/// Extract text content of a simple XML tag from a raw XML string
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

/// Content variants for the tag() builder
#[non_exhaustive]
pub enum TagContent<'a> {
    None,
    Text(&'a str),
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
