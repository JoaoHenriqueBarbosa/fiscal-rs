//! CT-e 44-digit access key (chave de acesso) generation.
//!
//! Layout (leiaute 4.00):
//!
//! ```text
//! cUF(2) + AAMM(4) + CNPJ(14) + mod57(2) + serie(3) + nCT(9)
//! + tpEmis(1) + cCT(8) + cDV(1) = 44 digits
//! ```
//!
//! The mod-11 check digit (`cDV`) reuses [`fiscal_core::xml_builder::access_key::calculate_mod11`],
//! the same algorithm used for NF-e/NFC-e/MDF-e keys.

use fiscal_core::FiscalError;
use fiscal_core::xml_builder::access_key::{
    calculate_mod11, format_year_month, generate_numeric_code,
};

use crate::CTE_MODEL;
use crate::types::Ide;

/// Parameters required to build a CT-e access key.
#[derive(Debug, Clone)]
pub struct CteAccessKeyParams<'a> {
    /// `mod` — document model: `"57"` (CT-e) or `"67"` (CT-e OS).
    pub model: &'a str,
    /// `cUF` — issuer state IBGE code (2 digits).
    pub state_code: &'a str,
    /// `AAMM` — emission year/month (4 digits, `YYMM`).
    pub year_month: String,
    /// `CNPJ` — issuer CNPJ (14 digits).
    pub tax_id: &'a str,
    /// `serie` — document series.
    pub series: u32,
    /// `nCT` — document number.
    pub number: u32,
    /// `tpEmis` — emission type (1 digit).
    pub emission_type: &'a str,
    /// `cCT` — random numeric code (8 digits).
    pub numeric_code: &'a str,
}

/// A validated 44-digit CT-e access key together with its component parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CteAccessKey {
    /// The full 44-digit key.
    pub key: String,
    /// The 8-digit `cCT` numeric code used to build the key.
    pub numeric_code: String,
    /// The single `cDV` check digit.
    pub check_digit: u8,
}

impl CteAccessKey {
    /// Return the key as a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.key
    }
}

/// Build the 44-digit CT-e access key from its component parts.
///
/// Concatenates the parts with proper padding, computes the mod-11 check
/// digit, and returns the [`CteAccessKey`].
///
/// # Errors
///
/// Returns [`FiscalError::XmlGeneration`] if the assembled base is not exactly
/// 43 digits (indicating malformed input parameters).
pub fn build_cte_access_key(params: &CteAccessKeyParams<'_>) -> Result<CteAccessKey, FiscalError> {
    let base = format!(
        "{cuf:0>2}{aamm}{cnpj:0>14}{model:0>2}{serie:0>3}{nct:0>9}{tp_emis}{cct:0>8}",
        cuf = params.state_code,
        aamm = params.year_month,
        cnpj = params.tax_id,
        model = params.model,
        serie = params.series,
        nct = params.number,
        tp_emis = params.emission_type,
        cct = params.numeric_code,
    );

    if base.len() != 43 || !base.bytes().all(|b| b.is_ascii_digit()) {
        return Err(FiscalError::XmlGeneration(format!(
            "CT-e access key base must be 43 digits, got {} (\"{}\")",
            base.len(),
            base
        )));
    }

    let check_digit = calculate_mod11(&base);
    Ok(CteAccessKey {
        key: format!("{base}{check_digit}"),
        numeric_code: format!("{:0>8}", params.numeric_code),
        check_digit,
    })
}

/// Build the CT-e access key directly from an `Ide` block.
///
/// Uses `ide.dh_emi` for the `AAMM`, and `numeric_code` for `cCT` when given;
/// otherwise a code is generated.
///
/// # Errors
///
/// Propagates [`build_cte_access_key`] failures.
pub fn build_cte_access_key_from_ide(
    ide: &Ide,
    tax_id: &str,
    numeric_code: Option<&str>,
) -> Result<CteAccessKey, FiscalError> {
    build_cte_access_key_from_ide_model(ide, CTE_MODEL, tax_id, numeric_code)
}

/// Like [`build_cte_access_key_from_ide`] but for an explicit model (`"57"` or
/// `"67"` for CT-e OS).
pub fn build_cte_access_key_from_ide_model(
    ide: &Ide,
    model: &str,
    tax_id: &str,
    numeric_code: Option<&str>,
) -> Result<CteAccessKey, FiscalError> {
    let generated;
    let cct = match numeric_code {
        Some(c) => c,
        None => {
            generated = generate_numeric_code();
            &generated
        }
    };

    build_cte_access_key(&CteAccessKeyParams {
        model,
        state_code: &ide.c_uf,
        year_month: format_year_month(&ide.dh_emi),
        tax_id,
        series: ide.serie,
        number: ide.n_ct,
        emission_type: &ide.tp_emis,
        numeric_code: cct,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_44_digit_key_with_correct_layout() {
        let params = CteAccessKeyParams {
            model: "57",
            state_code: "43",
            year_month: "2506".to_string(),
            tax_id: "12345678000190",
            series: 1,
            number: 123,
            emission_type: "1",
            numeric_code: "00000001",
        };
        let ak = build_cte_access_key(&params).unwrap();
        assert_eq!(ak.key.len(), 44);
        assert_eq!(&ak.key[0..2], "43"); // cUF
        assert_eq!(&ak.key[2..6], "2506"); // AAMM
        assert_eq!(&ak.key[6..20], "12345678000190"); // CNPJ
        assert_eq!(&ak.key[20..22], "57"); // mod
        assert_eq!(&ak.key[22..25], "001"); // serie
        assert_eq!(&ak.key[25..34], "000000123"); // nCT
        assert_eq!(&ak.key[34..35], "1"); // tpEmis
        assert_eq!(&ak.key[35..43], "00000001"); // cCT
        assert_eq!(ak.key.as_bytes()[43] - b'0', calculate_mod11(&ak.key[..43]));
    }

    #[test]
    fn check_digit_matches_mod11() {
        let base = "4325061234567800019057001000000123100000001";
        assert_eq!(base.len(), 43);
        let params = CteAccessKeyParams {
            model: "57",
            state_code: "43",
            year_month: "2506".to_string(),
            tax_id: "12345678000190",
            series: 1,
            number: 123,
            emission_type: "1",
            numeric_code: "00000001",
        };
        let ak = build_cte_access_key(&params).unwrap();
        assert_eq!(ak.check_digit, calculate_mod11(base));
        assert_eq!(ak.key, format!("{base}{}", ak.check_digit));
    }

    #[test]
    fn rejects_non_numeric_tax_id() {
        let params = CteAccessKeyParams {
            model: "57",
            state_code: "43",
            year_month: "2506".to_string(),
            tax_id: "1234567800019X",
            series: 1,
            number: 123,
            emission_type: "1",
            numeric_code: "00000001",
        };
        assert!(build_cte_access_key(&params).is_err());
    }
}
