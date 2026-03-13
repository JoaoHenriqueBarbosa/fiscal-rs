//! Product-related code newtypes: [`Gtin`], [`Ncm`], [`Cfop`].

use std::fmt;

use crate::FiscalError;

// ── GTIN (barcode) ──────────────────────────────────────────────────────────

/// Validated GTIN barcode (GTIN-8, GTIN-12, GTIN-13, or GTIN-14).
///
/// The special value `"SEM GTIN"` is accepted — it is used extensively in
/// NF-e documents to indicate products that have no barcode.
///
/// Numeric values are validated via [`crate::gtin::is_valid_gtin`], which
/// checks both the digit count and the check digit.
///
/// `Display` renders the stored value as-is.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Gtin(pub String);

impl Gtin {
    /// Create a new `Gtin`, validating the barcode.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidGtin`] if the value is not a valid
    /// GTIN-8/12/13/14 or `"SEM GTIN"`.
    pub fn new(s: &str) -> Result<Self, FiscalError> {
        crate::gtin::is_valid_gtin(s)?;
        Ok(Self(s.to_string()))
    }

    /// Return the stored value as a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// `true` if this GTIN is the special `"SEM GTIN"` sentinel.
    #[inline]
    pub fn is_sem_gtin(&self) -> bool {
        self.0 == "SEM GTIN"
    }
}

impl fmt::Display for Gtin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ── NCM code ────────────────────────────────────────────────────────────────

/// Validated 8-digit NCM (Nomenclatura Comum do Mercosul) code.
///
/// Must be exactly 8 ASCII digits. The value `"00000000"` is valid and is
/// typically used for services.
///
/// `Display` renders the 8-digit string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ncm(pub String);

impl Ncm {
    /// Create a new `Ncm`, validating that it is exactly 8 ASCII digits.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidTaxData`] if the value is not exactly
    /// 8 ASCII digits.
    pub fn new(s: &str) -> Result<Self, FiscalError> {
        if s.len() != 8 || !s.bytes().all(|b| b.is_ascii_digit()) {
            return Err(FiscalError::InvalidTaxData(format!(
                "NCM must be exactly 8 digits, got \"{}\"",
                s
            )));
        }
        Ok(Self(s.to_string()))
    }

    /// Return the stored NCM code as a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Ncm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ── CFOP code ───────────────────────────────────────────────────────────────

/// Validated 4-digit CFOP (Codigo Fiscal de Operacoes e Prestacoes).
///
/// The first digit determines the operation direction:
/// - **1, 2, 3** — *entrada* (incoming goods / services)
/// - **5, 6, 7** — *saida* (outgoing goods / services)
///
/// `Display` renders the 4-digit string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cfop(pub String);

impl Cfop {
    /// Create a new `Cfop`, validating format and first-digit range.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidTaxData`] if the value is not exactly
    /// 4 ASCII digits or the first digit is not in 1..=7.
    pub fn new(s: &str) -> Result<Self, FiscalError> {
        if s.len() != 4 || !s.bytes().all(|b| b.is_ascii_digit()) {
            return Err(FiscalError::InvalidTaxData(format!(
                "CFOP must be exactly 4 digits, got \"{}\"",
                s
            )));
        }
        let first = s.as_bytes()[0] - b'0';
        if !(1..=7).contains(&first) {
            return Err(FiscalError::InvalidTaxData(format!(
                "CFOP first digit must be 1-7, got \"{}\"",
                s
            )));
        }
        Ok(Self(s.to_string()))
    }

    /// Return the stored CFOP code as a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// `true` if this is an *entrada* (incoming) operation (first digit 1, 2, or 3).
    #[inline]
    pub fn is_entrada(&self) -> bool {
        matches!(self.0.as_bytes()[0], b'1' | b'2' | b'3')
    }

    /// `true` if this is a *saida* (outgoing) operation (first digit 5, 6, or 7).
    #[inline]
    pub fn is_saida(&self) -> bool {
        matches!(self.0.as_bytes()[0], b'5' | b'6' | b'7')
    }
}

impl fmt::Display for Cfop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // -- Gtin -----------------------------------------------------------------

    #[test]
    fn gtin_valid_sem_gtin() {
        let g = Gtin::new("SEM GTIN").unwrap();
        assert_eq!(g.as_str(), "SEM GTIN");
        assert!(g.is_sem_gtin());
    }

    #[test]
    fn gtin_valid_empty() {
        // Empty string is accepted by is_valid_gtin
        let g = Gtin::new("").unwrap();
        assert_eq!(g.as_str(), "");
        assert!(!g.is_sem_gtin());
    }

    #[test]
    fn gtin_valid_13_digit() {
        // EAN-13 barcode: 7891000315507
        let g = Gtin::new("7891000315507").unwrap();
        assert_eq!(g.as_str(), "7891000315507");
        assert!(!g.is_sem_gtin());
    }

    #[test]
    fn gtin_display() {
        let g = Gtin::new("SEM GTIN").unwrap();
        assert_eq!(g.to_string(), "SEM GTIN");
    }

    #[test]
    fn gtin_invalid_non_numeric() {
        assert!(Gtin::new("ABC12345").is_err());
    }

    #[test]
    fn gtin_invalid_length() {
        // 10 digits — not a valid GTIN length
        assert!(Gtin::new("1234567890").is_err());
    }

    #[test]
    fn gtin_invalid_check_digit() {
        // 7891000315508 — wrong check digit (8 instead of 7)
        assert!(Gtin::new("7891000315508").is_err());
    }

    #[test]
    fn gtin_error_variant() {
        let err = Gtin::new("INVALID").unwrap_err();
        assert!(matches!(err, FiscalError::InvalidGtin(_)));
    }

    // -- Ncm ------------------------------------------------------------------

    #[test]
    fn ncm_valid() {
        let n = Ncm::new("22021000").unwrap();
        assert_eq!(n.as_str(), "22021000");
    }

    #[test]
    fn ncm_valid_services() {
        let n = Ncm::new("00000000").unwrap();
        assert_eq!(n.as_str(), "00000000");
    }

    #[test]
    fn ncm_display() {
        let n = Ncm::new("22021000").unwrap();
        assert_eq!(n.to_string(), "22021000");
    }

    #[test]
    fn ncm_too_short() {
        assert!(Ncm::new("2202100").is_err());
    }

    #[test]
    fn ncm_too_long() {
        assert!(Ncm::new("220210001").is_err());
    }

    #[test]
    fn ncm_non_digits() {
        assert!(Ncm::new("2202100A").is_err());
    }

    #[test]
    fn ncm_empty() {
        assert!(Ncm::new("").is_err());
    }

    #[test]
    fn ncm_error_variant() {
        let err = Ncm::new("bad").unwrap_err();
        assert!(matches!(err, FiscalError::InvalidTaxData(_)));
    }

    // -- Cfop -----------------------------------------------------------------

    #[test]
    fn cfop_valid_entrada() {
        for code in &["1102", "2102", "3102"] {
            let c = Cfop::new(code).unwrap();
            assert!(c.is_entrada());
            assert!(!c.is_saida());
        }
    }

    #[test]
    fn cfop_valid_saida() {
        for code in &["5102", "6102", "7102"] {
            let c = Cfop::new(code).unwrap();
            assert!(c.is_saida());
            assert!(!c.is_entrada());
        }
    }

    #[test]
    fn cfop_valid_digit_4() {
        // First digit 4 is valid (1-7 range) but neither entrada nor saida
        let c = Cfop::new("4102").unwrap();
        assert!(!c.is_entrada());
        assert!(!c.is_saida());
    }

    #[test]
    fn cfop_display() {
        let c = Cfop::new("5102").unwrap();
        assert_eq!(c.to_string(), "5102");
    }

    #[test]
    fn cfop_as_str() {
        let c = Cfop::new("5102").unwrap();
        assert_eq!(c.as_str(), "5102");
    }

    #[test]
    fn cfop_too_short() {
        assert!(Cfop::new("510").is_err());
    }

    #[test]
    fn cfop_too_long() {
        assert!(Cfop::new("51020").is_err());
    }

    #[test]
    fn cfop_non_digits() {
        assert!(Cfop::new("51A2").is_err());
    }

    #[test]
    fn cfop_first_digit_zero() {
        assert!(Cfop::new("0102").is_err());
    }

    #[test]
    fn cfop_first_digit_eight() {
        assert!(Cfop::new("8102").is_err());
    }

    #[test]
    fn cfop_first_digit_nine() {
        assert!(Cfop::new("9102").is_err());
    }

    #[test]
    fn cfop_empty() {
        assert!(Cfop::new("").is_err());
    }

    #[test]
    fn cfop_error_variant() {
        let err = Cfop::new("bad").unwrap_err();
        assert!(matches!(err, FiscalError::InvalidTaxData(_)));
    }
}
