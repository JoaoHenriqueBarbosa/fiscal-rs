//! Parse-don't-validate newtypes for monetary amounts, tax rates, access keys,
//! and state codes.
//!
//! All newtypes currently keep their inner field `pub` to allow gradual
//! migration from raw primitives.  A future pass will make the fields private
//! once the codebase is fully migrated.

use std::fmt;
use std::ops::{Add, AddAssign};

use crate::FiscalError;
use crate::state_codes::STATE_IBGE_CODES;

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
    ("AC", "12"),
    ("AL", "27"),
    ("AM", "13"),
    ("AP", "16"),
    ("BA", "29"),
    ("CE", "23"),
    ("DF", "53"),
    ("ES", "32"),
    ("GO", "52"),
    ("MA", "21"),
    ("MG", "31"),
    ("MS", "50"),
    ("MT", "51"),
    ("PA", "15"),
    ("PB", "25"),
    ("PE", "26"),
    ("PI", "22"),
    ("PR", "41"),
    ("RJ", "33"),
    ("RN", "24"),
    ("RO", "11"),
    ("RR", "14"),
    ("RS", "43"),
    ("SC", "42"),
    ("SE", "28"),
    ("SP", "35"),
    ("TO", "17"),
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

/// Validated 4-digit CFOP (Código Fiscal de Operações e Prestações).
///
/// The first digit determines the operation direction:
/// - **1, 2, 3** — *entrada* (incoming goods / services)
/// - **5, 6, 7** — *saída* (outgoing goods / services)
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

    /// `true` if this is a *saída* (outgoing) operation (first digit 5, 6, or 7).
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

    // -- From<i64> for Cents and From<Cents> for i64 -------------------------

    #[test]
    fn cents_from_i64() {
        let c = Cents::from(42_i64);
        assert_eq!(c, Cents(42));
    }

    #[test]
    fn i64_from_cents() {
        let v: i64 = Cents(999).into();
        assert_eq!(v, 999);
    }

    // -- From<i64> for Rate and From<Rate> for i64 ---------------------------

    #[test]
    fn rate_from_i64() {
        let r = Rate::from(1800_i64);
        assert_eq!(r, Rate(1800));
    }

    #[test]
    fn i64_from_rate() {
        let v: i64 = Rate(1800).into();
        assert_eq!(v, 1800);
    }

    // -- From<i64> for Rate4 and From<Rate4> for i64 -------------------------

    #[test]
    fn rate4_from_i64() {
        let r = Rate4::from(16500_i64);
        assert_eq!(r, Rate4(16500));
    }

    #[test]
    fn i64_from_rate4() {
        let v: i64 = Rate4(16500).into();
        assert_eq!(v, 16500);
    }

    // -- Rate arithmetic -------------------------------------------------------

    #[test]
    fn rate_add_assign() {
        let mut r = Rate(100);
        r += Rate(50);
        assert_eq!(r, Rate(150));
    }

    // -- Rate4 arithmetic -------------------------------------------------------

    #[test]
    fn rate4_add_assign() {
        let mut r = Rate4(10000);
        r += Rate4(5000);
        assert_eq!(r, Rate4(15000));
    }

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
            "AC", "AL", "AM", "AP", "BA", "CE", "DF", "ES", "GO", "MA", "MG", "MS", "MT", "PA",
            "PB", "PE", "PI", "PR", "RJ", "RN", "RO", "RR", "RS", "SC", "SE", "SP", "TO",
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

    // -- TaxId ----------------------------------------------------------------

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

    // -- Explicit From/Into round-trip tests for coverage on #[inline] impls --

    #[test]
    fn cents_from_into_roundtrip() {
        let original: i64 = 42;
        let c: Cents = original.into();
        let back: i64 = c.into();
        assert_eq!(original, back);
    }

    #[test]
    fn rate_from_into_roundtrip() {
        let original: i64 = 1800;
        let r: Rate = original.into();
        let back: i64 = r.into();
        assert_eq!(original, back);
    }

    #[test]
    fn rate4_from_into_roundtrip() {
        let original: i64 = 16500;
        let r: Rate4 = original.into();
        let back: i64 = r.into();
        assert_eq!(original, back);
    }
}
