//! State-related newtypes: [`StateCode`] and [`IbgeCode`].

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::FiscalError;
use crate::state_codes::STATE_IBGE_CODES;

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
}
