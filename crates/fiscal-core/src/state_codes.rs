//! Brazilian state IBGE code lookup tables and helper functions.
//!
//! Two static maps are provided:
//! - [`STATE_IBGE_CODES`] — two-letter UF abbreviation → IBGE numeric `cUF` code.
//! - [`IBGE_TO_UF`] — IBGE numeric code → two-letter UF abbreviation (reverse).
//!
//! Use [`get_state_code`] and [`get_state_by_code`] for ergonomic access
//! with proper error handling.

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::FiscalError;

/// Lazy-initialised map from two-letter UF abbreviation to IBGE numeric state code (`cUF`).
///
/// Contains all 26 Brazilian states plus the Federal District (DF).
///
/// # Examples
///
/// ```
/// use fiscal_core::state_codes::STATE_IBGE_CODES;
/// assert_eq!(STATE_IBGE_CODES.get("PR"), Some(&"41"));
/// assert_eq!(STATE_IBGE_CODES.get("SP"), Some(&"35"));
/// ```
pub static STATE_IBGE_CODES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("AC", "12"),
        ("AL", "27"),
        ("AP", "16"),
        ("AM", "13"),
        ("BA", "29"),
        ("CE", "23"),
        ("DF", "53"),
        ("ES", "32"),
        ("GO", "52"),
        ("MA", "21"),
        ("MT", "51"),
        ("MS", "50"),
        ("MG", "31"),
        ("PA", "15"),
        ("PB", "25"),
        ("PR", "41"),
        ("PE", "26"),
        ("PI", "22"),
        ("RJ", "33"),
        ("RN", "24"),
        ("RS", "43"),
        ("RO", "11"),
        ("RR", "14"),
        ("SC", "42"),
        ("SP", "35"),
        ("SE", "28"),
        ("TO", "17"),
        // Special codes (Ambiente Nacional, SEFAZ Virtual)
        ("AN", "91"),
        ("SVRS", "92"),
    ])
});

/// Lazy-initialised reverse map from IBGE numeric state code to two-letter UF abbreviation.
///
/// # Examples
///
/// ```
/// use fiscal_core::state_codes::IBGE_TO_UF;
/// assert_eq!(IBGE_TO_UF.get("41"), Some(&"PR"));
/// ```
pub static IBGE_TO_UF: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    STATE_IBGE_CODES
        .iter()
        .map(|(&uf, &code)| (code, uf))
        .collect()
});

/// Get the IBGE numeric code for a state abbreviation.
///
/// # Errors
///
/// Returns [`FiscalError::InvalidStateCode`] if `uf` is not a known
/// Brazilian state abbreviation.
pub fn get_state_code(uf: &str) -> Result<&'static str, FiscalError> {
    STATE_IBGE_CODES
        .get(uf)
        .copied()
        .ok_or_else(|| FiscalError::InvalidStateCode(uf.to_string()))
}

/// Get the UF abbreviation for an IBGE numeric code.
///
/// # Errors
///
/// Returns [`FiscalError::InvalidStateCode`] if `code` is not a known
/// IBGE numeric state code.
pub fn get_state_by_code(code: &str) -> Result<&'static str, FiscalError> {
    IBGE_TO_UF
        .get(code)
        .copied()
        .ok_or_else(|| FiscalError::InvalidStateCode(code.to_string()))
}
