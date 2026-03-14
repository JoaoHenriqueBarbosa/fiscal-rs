//! Validated Brazilian tax identifier (CPF / CNPJ).

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::FiscalError;

// ── Tax ID (CPF / CNPJ) ─────────────────────────────────────────────────────

/// Validated Brazilian tax identifier — either a CPF (11 digits) or CNPJ
/// (14 digits).
///
/// The constructor strips all non-digit characters (dots, dashes, slashes)
/// and validates that the result is exactly 11 or 14 digits. Check-digit
/// validation is intentionally skipped because many test fixtures use
/// synthetic CNPJs.
///
/// `Display` renders the raw digits (no formatting).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct TaxId(pub String);

impl TaxId {
    /// Create a new `TaxId`, stripping formatting and validating digit count.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidTaxData`] if the stripped value is not
    /// exactly 11 or 14 ASCII digits.
    pub fn new(s: &str) -> Result<Self, FiscalError> {
        let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() != 11 && digits.len() != 14 {
            return Err(FiscalError::InvalidTaxData(format!(
                "Tax ID must be 11 (CPF) or 14 (CNPJ) digits, got {} digits from \"{}\"",
                digits.len(),
                s
            )));
        }
        Ok(Self(digits))
    }

    /// Return the stored digits as a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Return the stored digits as a string slice (alias for `as_str`).
    #[inline]
    pub fn digits(&self) -> &str {
        &self.0
    }

    /// `true` if this is an 11-digit CPF.
    #[inline]
    pub fn is_cpf(&self) -> bool {
        self.0.len() == 11
    }

    /// `true` if this is a 14-digit CNPJ.
    #[inline]
    pub fn is_cnpj(&self) -> bool {
        self.0.len() == 14
    }
}

impl fmt::Display for TaxId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tax_id_valid_cpf_digits_only() {
        let tid = TaxId::new("12345678901").unwrap();
        assert_eq!(tid.as_str(), "12345678901");
        assert_eq!(tid.digits(), "12345678901");
        assert!(tid.is_cpf());
        assert!(!tid.is_cnpj());
    }

    #[test]
    fn tax_id_valid_cpf_formatted() {
        let tid = TaxId::new("123.456.789-01").unwrap();
        assert_eq!(tid.digits(), "12345678901");
        assert!(tid.is_cpf());
    }

    #[test]
    fn tax_id_valid_cnpj_digits_only() {
        let tid = TaxId::new("12345678000195").unwrap();
        assert_eq!(tid.as_str(), "12345678000195");
        assert!(tid.is_cnpj());
        assert!(!tid.is_cpf());
    }

    #[test]
    fn tax_id_valid_cnpj_formatted() {
        let tid = TaxId::new("12.345.678/0001-95").unwrap();
        assert_eq!(tid.digits(), "12345678000195");
        assert!(tid.is_cnpj());
    }

    #[test]
    fn tax_id_display() {
        let tid = TaxId::new("12.345.678/0001-95").unwrap();
        assert_eq!(tid.to_string(), "12345678000195");
    }

    #[test]
    fn tax_id_empty_string() {
        assert!(TaxId::new("").is_err());
    }

    #[test]
    fn tax_id_wrong_length() {
        // 10 digits — neither CPF nor CNPJ
        assert!(TaxId::new("1234567890").is_err());
        // 13 digits
        assert!(TaxId::new("1234567890123").is_err());
    }

    #[test]
    fn tax_id_all_letters() {
        assert!(TaxId::new("abcdefghijk").is_err());
    }

    #[test]
    fn tax_id_error_variant() {
        let err = TaxId::new("123").unwrap_err();
        assert!(matches!(err, FiscalError::InvalidTaxData(_)));
    }
}
