//! Validated 44-digit NF-e / NFC-e access key.

use std::fmt;

use crate::FiscalError;
use crate::state_codes::get_state_by_code;

// ── Access key ──────────────────────────────────────────────────────────────

/// Validated 44-digit NF-e / NFC-e access key.
///
/// Layout:
/// ```text
/// cUF(2) + AAMM(4) + CNPJ(14) + mod(2) + serie(3) + nNF(9)
/// + tpEmis(1) + cNF(8) + cDV(1) = 44 digits
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccessKey(pub String);

impl AccessKey {
    /// Create a new `AccessKey`, validating that it is exactly 44 ASCII digits.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidTaxData`] if the key is not exactly
    /// 44 ASCII digits.
    pub fn new(key: &str) -> Result<Self, FiscalError> {
        if key.len() != 44 || !key.bytes().all(|b| b.is_ascii_digit()) {
            return Err(FiscalError::InvalidTaxData(format!(
                "Access key must be exactly 44 digits, got \"{}\"",
                key
            )));
        }
        Ok(Self(key.to_string()))
    }

    /// Return the inner key as a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// IBGE state code (positions 0..2).
    #[inline]
    pub fn state_code(&self) -> &str {
        &self.0[0..2]
    }

    /// Year and month — YYMM (positions 2..6).
    #[inline]
    pub fn year_month(&self) -> &str {
        &self.0[2..6]
    }

    /// CNPJ / CPF of the issuer (positions 6..20).
    #[inline]
    pub fn tax_id(&self) -> &str {
        &self.0[6..20]
    }

    /// Invoice model — `"55"` (NF-e) or `"65"` (NFC-e) (positions 20..22).
    #[inline]
    pub fn model(&self) -> &str {
        &self.0[20..22]
    }

    /// Series number (positions 22..25).
    #[inline]
    pub fn series(&self) -> &str {
        &self.0[22..25]
    }

    /// Invoice number (positions 25..34).
    #[inline]
    pub fn number(&self) -> &str {
        &self.0[25..34]
    }

    /// Emission type — `tpEmis` (position 34).
    #[inline]
    pub fn emission_type(&self) -> &str {
        &self.0[34..35]
    }

    /// Numeric code (positions 35..43).
    #[inline]
    pub fn numeric_code(&self) -> &str {
        &self.0[35..43]
    }

    /// Check digit (position 43).
    #[inline]
    pub fn check_digit(&self) -> &str {
        &self.0[43..44]
    }

    /// Validate that this key's cUF (first 2 digits) matches the expected UF.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidTaxData`] if the cUF in the key does not
    /// correspond to the given UF abbreviation.
    pub fn validate_uf(&self, expected_uf: &str) -> Result<(), FiscalError> {
        let uf = get_state_by_code(self.state_code())?;
        if !uf.eq_ignore_ascii_case(expected_uf) {
            return Err(FiscalError::InvalidTaxData(format!(
                "Access key {} does not belong to UF {}, got {}",
                self.0, expected_uf, uf
            )));
        }
        Ok(())
    }
}

impl fmt::Display for AccessKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn access_key_valid() {
        let key = "43250304123456789012550010000000011000000010";
        let ak = AccessKey::new(key).unwrap();
        assert_eq!(ak.as_str(), key);
        assert_eq!(ak.state_code(), "43");
        assert_eq!(ak.year_month(), "2503");
        assert_eq!(ak.tax_id(), "04123456789012");
        assert_eq!(ak.model(), "55");
        assert_eq!(ak.series(), "001");
        assert_eq!(ak.number(), "000000001");
        assert_eq!(ak.emission_type(), "1");
        assert_eq!(ak.numeric_code(), "00000001");
        assert_eq!(ak.check_digit(), "0");
        assert_eq!(ak.to_string(), key);
    }

    #[test]
    fn access_key_too_short() {
        assert!(AccessKey::new("1234").is_err());
    }

    #[test]
    fn access_key_non_digit() {
        let bad = "4325030412345678901255001000000001100000001X";
        assert!(AccessKey::new(bad).is_err());
    }

    #[test]
    fn validate_uf_match() {
        // 43 = RS
        let key = "43250304123456789012550010000000011000000010";
        let ak = AccessKey::new(key).unwrap();
        assert!(ak.validate_uf("RS").is_ok());
    }

    #[test]
    fn validate_uf_mismatch() {
        // 43 = RS, not SP
        let key = "43250304123456789012550010000000011000000010";
        let ak = AccessKey::new(key).unwrap();
        assert!(ak.validate_uf("SP").is_err());
    }
}
