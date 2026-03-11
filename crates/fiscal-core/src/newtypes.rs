//! Parse-don't-validate newtypes for monetary amounts, tax rates, access keys,
//! and state codes.
//!
//! All newtypes currently keep their inner field `pub` to allow gradual
//! migration from raw primitives.  A future pass will make the fields private
//! once the codebase is fully migrated.

use std::fmt;
use std::ops::{Add, AddAssign};

use crate::state_codes::STATE_IBGE_CODES;
use crate::FiscalError;

// ── Monetary amount ─────────────────────────────────────────────────────────

/// Monetary amount in cents (R$ 10.50 = `Cents(1050)`).
///
/// `Display` renders with 2 decimal places: `"10.50"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Cents(pub i64);

impl From<i64> for Cents {
    #[inline]
    fn from(v: i64) -> Self {
        Self(v)
    }
}

impl From<Cents> for i64 {
    #[inline]
    fn from(c: Cents) -> Self {
        c.0
    }
}

impl fmt::Display for Cents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let abs = self.0.unsigned_abs();
        let whole = abs / 100;
        let frac = abs % 100;
        if self.0 < 0 {
            write!(f, "-{whole}.{frac:02}")
        } else {
            write!(f, "{whole}.{frac:02}")
        }
    }
}

impl Add for Cents {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Cents {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

// ── Tax rate (hundredths) ───────────────────────────────────────────────────

/// Tax rate in hundredths of a percent (18% = `Rate(1800)`).
///
/// `Display` renders with 4 decimal places: `"18.0000"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Rate(pub i64);

impl From<i64> for Rate {
    #[inline]
    fn from(v: i64) -> Self {
        Self(v)
    }
}

impl From<Rate> for i64 {
    #[inline]
    fn from(r: Rate) -> Self {
        r.0
    }
}

impl fmt::Display for Rate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let abs = self.0.unsigned_abs();
        let whole = abs / 100;
        let frac = abs % 100;
        if self.0 < 0 {
            write!(f, "-{whole}.{frac:02}00")
        } else {
            write!(f, "{whole}.{frac:02}00")
        }
    }
}

impl Add for Rate {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Rate {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

// ── PIS/COFINS rate (×10 000) ───────────────────────────────────────────────

/// PIS/COFINS rate scaled by 10 000 (1.65% = `Rate4(16500)`).
///
/// `Display` renders with 4 decimal places: `"1.6500"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Rate4(pub i64);

impl From<i64> for Rate4 {
    #[inline]
    fn from(v: i64) -> Self {
        Self(v)
    }
}

impl From<Rate4> for i64 {
    #[inline]
    fn from(r: Rate4) -> Self {
        r.0
    }
}

impl fmt::Display for Rate4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let abs = self.0.unsigned_abs();
        let whole = abs / 10_000;
        let frac = abs % 10_000;
        if self.0 < 0 {
            write!(f, "-{whole}.{frac:04}")
        } else {
            write!(f, "{whole}.{frac:04}")
        }
    }
}

impl Add for Rate4 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Rate4 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

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
}

impl fmt::Display for AccessKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ── State code ──────────────────────────────────────────────────────────────

/// Two-letter Brazilian state abbreviation (UF), validated against
/// [`STATE_IBGE_CODES`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateCode(pub &'static str);

/// Static lookup table mapping UF abbreviation to IBGE code, used by
/// [`StateCode::new`] so we can hand out `&'static str` references
/// without going through the `LazyLock` `HashMap` at runtime.
static UF_TABLE: &[(&str, &str)] = &[
    ("AC", "12"), ("AL", "27"), ("AM", "13"), ("AP", "16"), ("BA", "29"),
    ("CE", "23"), ("DF", "53"), ("ES", "32"), ("GO", "52"), ("MA", "21"),
    ("MG", "31"), ("MS", "50"), ("MT", "51"), ("PA", "15"), ("PB", "25"),
    ("PE", "26"), ("PI", "22"), ("PR", "41"), ("RJ", "33"), ("RN", "24"),
    ("RO", "11"), ("RR", "14"), ("RS", "43"), ("SC", "42"), ("SE", "28"),
    ("SP", "35"), ("TO", "17"),
];

impl StateCode {
    /// Create a new `StateCode`, validating that `uf` is a known Brazilian
    /// state abbreviation.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidStateCode`] if the abbreviation is
    /// not one of the 27 known Brazilian states.
    pub fn new(uf: &str) -> Result<Self, FiscalError> {
        // Validate via the canonical map
        if !STATE_IBGE_CODES.contains_key(uf) {
            return Err(FiscalError::InvalidStateCode(uf.to_string()));
        }
        // Return the matching &'static str from our table so the lifetime is 'static.
        let static_uf = UF_TABLE
            .iter()
            .find(|(u, _)| *u == uf)
            .map(|(u, _)| *u)
            .expect("UF validated above must exist in UF_TABLE");
        Ok(Self(static_uf))
    }

    /// Return the 2-digit IBGE numeric code for this state.
    pub fn ibge_code(&self) -> &'static str {
        UF_TABLE
            .iter()
            .find(|(u, _)| *u == self.0)
            .map(|(_, c)| *c)
            .expect("StateCode is always constructed from UF_TABLE")
    }
}

impl fmt::Display for StateCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

// ── IBGE code ───────────────────────────────────────────────────────────────

/// IBGE numeric state or city code (e.g. `"41"` for PR, `"4106852"` for
/// Cruzmaltina).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IbgeCode(pub String);

impl fmt::Display for IbgeCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // -- Cents -----------------------------------------------------------------

    #[test]
    fn cents_display() {
        assert_eq!(Cents(1050).to_string(), "10.50");
        assert_eq!(Cents(0).to_string(), "0.00");
        assert_eq!(Cents(1).to_string(), "0.01");
        assert_eq!(Cents(-350).to_string(), "-3.50");
    }

    #[test]
    fn cents_add() {
        assert_eq!(Cents(100) + Cents(250), Cents(350));
        let mut c = Cents(500);
        c += Cents(100);
        assert_eq!(c, Cents(600));
    }

    #[test]
    fn cents_conversion() {
        let c: Cents = 1050_i64.into();
        assert_eq!(c, Cents(1050));
        let v: i64 = c.into();
        assert_eq!(v, 1050);
    }

    // -- Rate -----------------------------------------------------------------

    #[test]
    fn rate_display() {
        assert_eq!(Rate(1800).to_string(), "18.0000");
        assert_eq!(Rate(0).to_string(), "0.0000");
        assert_eq!(Rate(750).to_string(), "7.5000");
        assert_eq!(Rate(-1800).to_string(), "-18.0000");
    }

    #[test]
    fn rate_add() {
        assert_eq!(Rate(1000) + Rate(800), Rate(1800));
        let mut r = Rate(500);
        r += Rate(300);
        assert_eq!(r, Rate(800));
    }

    // -- Rate4 ----------------------------------------------------------------

    #[test]
    fn rate4_display() {
        assert_eq!(Rate4(16500).to_string(), "1.6500");
        assert_eq!(Rate4(0).to_string(), "0.0000");
        assert_eq!(Rate4(76000).to_string(), "7.6000");
        assert_eq!(Rate4(-16500).to_string(), "-1.6500");
    }

    #[test]
    fn rate4_add() {
        assert_eq!(Rate4(16500) + Rate4(7600), Rate4(24100));
        let mut r = Rate4(10000);
        r += Rate4(5000);
        assert_eq!(r, Rate4(15000));
    }

    // -- AccessKey ------------------------------------------------------------

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

    // -- StateCode ------------------------------------------------------------

    #[test]
    fn state_code_valid() {
        let sc = StateCode::new("PR").unwrap();
        assert_eq!(sc.0, "PR");
        assert_eq!(sc.ibge_code(), "41");
        assert_eq!(sc.to_string(), "PR");
    }

    #[test]
    fn state_code_invalid() {
        assert!(StateCode::new("XX").is_err());
    }

    #[test]
    fn state_code_all_states() {
        let ufs = [
            "AC", "AL", "AM", "AP", "BA", "CE", "DF", "ES", "GO", "MA",
            "MG", "MS", "MT", "PA", "PB", "PE", "PI", "PR", "RJ", "RN",
            "RO", "RR", "RS", "SC", "SE", "SP", "TO",
        ];
        for uf in ufs {
            let sc = StateCode::new(uf).unwrap_or_else(|_| panic!("Failed for {uf}"));
            assert!(!sc.ibge_code().is_empty());
        }
    }

    // -- IbgeCode -------------------------------------------------------------

    #[test]
    fn ibge_code_display() {
        let code = IbgeCode("4106852".to_string());
        assert_eq!(code.to_string(), "4106852");
    }
}
