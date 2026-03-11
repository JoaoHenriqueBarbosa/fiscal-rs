//! TaxId value object for CPF/CNPJ detection and XML tag generation.
//!
//! A CPF has at most 11 digits; a CNPJ has 14. This module encapsulates
//! the detection logic and zero-padding rules used throughout the XML builder.

use crate::xml_utils::TagContent;
use crate::xml_utils::tag;

/// Determines whether a tax identifier is CPF or CNPJ based on length,
/// and provides XML tag generation helpers.
#[derive(Debug, Clone)]
pub struct TaxId<'a> {
    value: &'a str,
}

impl<'a> TaxId<'a> {
    /// Wrap a raw tax identifier string for CPF/CNPJ detection.
    pub fn new(value: &'a str) -> Self {
        Self { value }
    }

    /// A CPF has at most 11 digits.
    pub fn is_cpf(&self) -> bool {
        self.value.len() <= 11
    }

    /// Returns `"CPF"` or `"CNPJ"` — ready to use as an XML tag name.
    pub fn tag_name(&self) -> &'static str {
        if self.is_cpf() { "CPF" } else { "CNPJ" }
    }

    /// Zero-padded to 11 (CPF) or 14 (CNPJ) digits.
    pub fn padded(&self) -> String {
        let width = if self.is_cpf() { 11 } else { 14 };
        format!("{:0>width$}", self.value, width = width)
    }

    /// Build the XML tag: `<CPF>00012345678</CPF>` or `<CNPJ>…</CNPJ>`.
    pub fn to_xml_tag(&self) -> String {
        tag(
            self.tag_name(),
            &[],
            TagContent::Text(&self.padded_static()),
        )
    }

    // Internal: return padded as owned String (for borrowing into tag)
    fn padded_static(&self) -> String {
        self.padded()
    }
}
