//! MDF-e 44-digit access key (chave de acesso) generation.
//!
//! Layout (leiaute 3.00):
//!
//! ```text
//! cUF(2) + AAMM(4) + CNPJ(14) + mod58(2) + serie(3) + nMDF(9)
//! + tpEmis(1) + cMDF(8) + cDV(1) = 44 digits
//! ```
//!
//! The mod-11 check digit (`cDV`) reuses [`fiscal_core::xml_builder::access_key::calculate_mod11`],
//! the same algorithm used for NF-e/NFC-e keys.

use fiscal_core::FiscalError;
use fiscal_core::xml_builder::access_key::{
    calculate_mod11, format_year_month, generate_numeric_code,
};

use crate::MDFE_MODEL;
use crate::types::Ide;

/// Parameters required to build an MDF-e access key.
#[derive(Debug, Clone)]
pub struct MdfeAccessKeyParams<'a> {
    /// `cUF` — issuer state IBGE code (2 digits).
    pub state_code: &'a str,
    /// `AAMM` — emission year/month (4 digits, `YYMM`).
    pub year_month: String,
    /// `CNPJ` — issuer CNPJ (14 digits).
    pub tax_id: &'a str,
    /// `serie` — document series.
    pub series: u32,
    /// `nMDF` — document number.
    pub number: u32,
    /// `tpEmis` — emission type (1 digit).
    pub emission_type: &'a str,
    /// `cMDF` — random numeric code (8 digits).
    pub numeric_code: &'a str,
}

/// A validated 44-digit MDF-e access key together with its component parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MdfeAccessKey {
    /// The full 44-digit key.
    pub key: String,
    /// The 8-digit `cMDF` numeric code used to build the key.
    pub numeric_code: String,
    /// The single `cDV` check digit.
    pub check_digit: u8,
}

impl MdfeAccessKey {
    /// Return the key as a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.key
    }
}

/// Build the 44-digit MDF-e access key from its component parts.
///
/// Concatenates the parts with proper padding, computes the mod-11 check
/// digit, and returns the [`MdfeAccessKey`].
///
/// # Errors
///
/// Returns [`FiscalError::XmlGeneration`] if the assembled base is not exactly
/// 43 digits (indicating malformed input parameters).
pub fn build_mdfe_access_key(
    params: &MdfeAccessKeyParams<'_>,
) -> Result<MdfeAccessKey, FiscalError> {
    let base = format!(
        "{cuf:0>2}{aamm}{cnpj:0>14}{model:0>2}{serie:0>3}{nmdf:0>9}{tp_emis}{cmdf:0>8}",
        cuf = params.state_code,
        aamm = params.year_month,
        cnpj = params.tax_id,
        model = MDFE_MODEL,
        serie = params.series,
        nmdf = params.number,
        tp_emis = params.emission_type,
        cmdf = params.numeric_code,
    );

    if base.len() != 43 || !base.bytes().all(|b| b.is_ascii_digit()) {
        return Err(FiscalError::XmlGeneration(format!(
            "MDF-e access key base must be 43 digits, got {} (\"{}\")",
            base.len(),
            base
        )));
    }

    let check_digit = calculate_mod11(&base);
    Ok(MdfeAccessKey {
        key: format!("{base}{check_digit}"),
        numeric_code: format!("{:0>8}", params.numeric_code),
        check_digit,
    })
}

/// Build the MDF-e access key directly from an `Ide` block.
///
/// Uses `ide.dh_emi` for the `AAMM`, and `numeric_code` for `cMDF` when given;
/// otherwise a code is generated.
///
/// # Errors
///
/// Propagates [`build_mdfe_access_key`] failures.
pub fn build_mdfe_access_key_from_ide(
    ide: &Ide,
    tax_id: &str,
    numeric_code: Option<&str>,
) -> Result<MdfeAccessKey, FiscalError> {
    let generated;
    let cmdf = match numeric_code {
        Some(c) => c,
        None => {
            generated = generate_numeric_code();
            &generated
        }
    };

    build_mdfe_access_key(&MdfeAccessKeyParams {
        state_code: &ide.c_uf,
        year_month: format_year_month(&ide.dh_emi),
        tax_id,
        series: ide.serie,
        number: ide.n_mdf,
        emission_type: &ide.tp_emis,
        numeric_code: cmdf,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_44_digit_key_with_correct_layout() {
        let params = MdfeAccessKeyParams {
            state_code: "43",
            year_month: "2506".to_string(),
            tax_id: "12345678000190",
            series: 1,
            number: 123,
            emission_type: "1",
            numeric_code: "00000001",
        };
        let ak = build_mdfe_access_key(&params).unwrap();
        assert_eq!(ak.key.len(), 44);
        assert_eq!(&ak.key[0..2], "43"); // cUF
        assert_eq!(&ak.key[2..6], "2506"); // AAMM
        assert_eq!(&ak.key[6..20], "12345678000190"); // CNPJ
        assert_eq!(&ak.key[20..22], "58"); // mod
        assert_eq!(&ak.key[22..25], "001"); // serie
        assert_eq!(&ak.key[25..34], "000000123"); // nMDF
        assert_eq!(&ak.key[34..35], "1"); // tpEmis
        assert_eq!(&ak.key[35..43], "00000001"); // cMDF
        // cDV is the recomputed mod-11 of the first 43 digits.
        assert_eq!(ak.key.as_bytes()[43] - b'0', calculate_mod11(&ak.key[..43]));
    }

    #[test]
    fn check_digit_matches_mod11() {
        let base = "4325061234567800019058001000000123100000001";
        assert_eq!(base.len(), 43);
        let params = MdfeAccessKeyParams {
            state_code: "43",
            year_month: "2506".to_string(),
            tax_id: "12345678000190",
            series: 1,
            number: 123,
            emission_type: "1",
            numeric_code: "00000001",
        };
        let ak = build_mdfe_access_key(&params).unwrap();
        assert_eq!(ak.check_digit, calculate_mod11(base));
        assert_eq!(ak.key, format!("{base}{}", ak.check_digit));
    }

    #[test]
    fn rejects_non_numeric_tax_id() {
        let params = MdfeAccessKeyParams {
            state_code: "43",
            year_month: "2506".to_string(),
            tax_id: "1234567800019X",
            series: 1,
            number: 123,
            emission_type: "1",
            numeric_code: "00000001",
        };
        assert!(build_mdfe_access_key(&params).is_err());
    }
}
