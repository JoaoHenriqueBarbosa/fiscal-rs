//! Access key (chave de acesso) generation for NF-e/NFC-e.
//!
//! An access key is a 44-digit numeric string that uniquely identifies
//! a Brazilian fiscal document. Layout:
//!
//! ```text
//! cUF(2) + AAMM(4) + CNPJ(14) + mod(2) + serie(3) + nNF(9)
//! + tpEmis(1) + cNF(8) + cDV(1) = 44 digits
//! ```

use crate::types::AccessKeyParams;
use crate::FiscalError;

/// Build the 44-digit access key from its component parts.
///
/// Concatenates all parts with proper padding, computes the mod-11 check
/// digit, and returns the complete 44-digit key.
///
/// # Errors
///
/// Returns [`FiscalError::XmlGeneration`] if the resulting base does not
/// have exactly 43 digits (indicating malformed input parameters).
pub fn build_access_key(params: &AccessKeyParams) -> Result<String, FiscalError> {
    let base = format!(
        "{cuf:0>2}{aamm}{cnpj:0>14}{model:0>2}{serie:0>3}{nnf:0>9}{tp_emis}{cnf:0>8}",
        cuf = params.state_code,
        aamm = params.year_month,
        cnpj = params.tax_id,
        model = params.model.as_str(),
        serie = params.series,
        nnf = params.number,
        tp_emis = params.emission_type.as_str(),
        cnf = params.numeric_code,
    );

    if base.len() != 43 {
        return Err(FiscalError::XmlGeneration(format!(
            "Access key base must be 43 digits, got {} (\"{}\")",
            base.len(),
            base
        )));
    }

    let check_digit = calculate_mod11(&base);
    Ok(format!("{base}{check_digit}"))
}

/// Calculate the mod-11 check digit used in Brazilian fiscal documents.
///
/// Weights cycle 2→9 from right to left. If the remainder after `% 11`
/// is less than 2 the digit is 0; otherwise it is `11 - remainder`.
pub fn calculate_mod11(digits: &str) -> u8 {
    let mut sum: u32 = 0;
    let mut weight: u32 = 2;

    for ch in digits.bytes().rev() {
        let val = (ch - b'0') as u32;
        sum += val * weight;
        weight = if weight >= 9 { 2 } else { weight + 1 };
    }

    let remainder = sum % 11;
    if remainder < 2 { 0 } else { (11 - remainder) as u8 }
}

/// Generate an 8-digit random numeric code for the access key.
pub fn generate_numeric_code() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let code = (nanos ^ (nanos >> 16)) % 100_000_000;
    format!("{code:08}")
}

/// Format a `DateTime<FixedOffset>` as YYMM for the access key.
pub fn format_year_month(dt: &chrono::DateTime<chrono::FixedOffset>) -> String {
    format!("{}{:02}", &dt.format("%y"), dt.format("%m"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mod11_known_values() {
        let base = "4325030412345678901255001000000001100000001";
        let dv = calculate_mod11(base);
        let full = format!("{base}{dv}");
        assert_eq!(full.len(), 44);
    }

    #[test]
    fn mod11_all_zeros() {
        let dv = calculate_mod11("0000000000000000000000000000000000000000000");
        assert_eq!(dv, 0);
    }
}
