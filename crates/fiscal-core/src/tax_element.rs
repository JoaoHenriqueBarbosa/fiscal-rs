/// A single XML field: <name>value</name>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxField {
    pub name: String,
    pub value: String,
}

impl TaxField {
    /// Create a new XML field with the given tag name and text value.
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Structured representation of a tax XML element.
#[derive(Debug, Clone)]
pub struct TaxElement {
    /// Outer wrapper tag (e.g., "ICMS", "PIS", "IPI"). None = no wrapper.
    pub outer_tag: Option<String>,
    /// Fields at the outer level, before the variant (e.g., IPI's cEnq).
    pub outer_fields: Vec<TaxField>,
    /// The variant/inner tag (e.g., "ICMS00", "PISAliq", "IPITrib", "II").
    pub variant_tag: String,
    /// Fields inside the variant tag.
    pub fields: Vec<TaxField>,
}

/// Create an optional field (returns None if value is None)
pub fn optional_field(name: &str, value: Option<&str>) -> Option<TaxField> {
    value.map(|v| TaxField::new(name, v))
}

/// Create a required field (returns Err if value is None).
///
/// # Errors
///
/// Returns [`FiscalError::MissingRequiredField`] if `value` is `None`.
pub fn required_field(name: &str, value: Option<&str>) -> Result<TaxField, crate::FiscalError> {
    match value {
        Some(v) => Ok(TaxField::new(name, v)),
        None => Err(crate::FiscalError::MissingRequiredField {
            field: name.to_string(),
        }),
    }
}

/// Filter None entries from a TaxField option array
pub fn filter_fields(fields: Vec<Option<TaxField>>) -> Vec<TaxField> {
    fields.into_iter().flatten().collect()
}

/// Escape XML special characters in a value
fn escape_xml_value(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            c => result.push(c),
        }
    }
    result
}

/// Serialize a TaxField to XML: <name>value</name>
fn serialize_field(field: &TaxField) -> String {
    format!(
        "<{name}>{value}</{name}>",
        name = field.name,
        value = escape_xml_value(&field.value)
    )
}

/// Serialize a TaxElement to an XML string.
pub fn serialize_tax_element(element: &TaxElement) -> String {
    let inner_content: String = element.fields.iter().map(serialize_field).collect();
    let variant_xml = format!("<{tag}>{inner_content}</{tag}>", tag = element.variant_tag,);

    match &element.outer_tag {
        None => variant_xml,
        Some(outer) => {
            let outer_fields_xml: String =
                element.outer_fields.iter().map(serialize_field).collect();
            format!("<{outer}>{outer_fields_xml}{variant_xml}</{outer}>")
        }
    }
}
