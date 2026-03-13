//! Timezone lookup by Brazilian state (UF).
//!
//! Maps each of the 27 Brazilian states (UFs) to its IANA timezone string,
//! mirroring the PHP `NFePHP\Common\TimeZoneByUF` class.

/// Returns the IANA timezone string for a given Brazilian state abbreviation (UF).
///
/// The input is case-insensitive. Returns `None` if the UF is not recognized.
///
/// # Examples
///
/// ```
/// use fiscal_core::timezone::timezone_for_uf;
///
/// assert_eq!(timezone_for_uf("SP"), Some("America/Sao_Paulo"));
/// assert_eq!(timezone_for_uf("am"), Some("America/Manaus"));
/// assert_eq!(timezone_for_uf("XX"), None);
/// ```
pub fn timezone_for_uf(uf: &str) -> Option<&'static str> {
    // Normalize to uppercase for case-insensitive matching.
    // We avoid heap allocation by matching on the two-char slice directly.
    if uf.len() != 2 {
        return None;
    }

    let bytes = uf.as_bytes();
    let upper = [bytes[0].to_ascii_uppercase(), bytes[1].to_ascii_uppercase()];

    match &upper {
        b"AC" => Some("America/Rio_Branco"),
        b"AL" => Some("America/Maceio"),
        b"AM" => Some("America/Manaus"),
        b"AP" => Some("America/Belem"),
        b"BA" => Some("America/Bahia"),
        b"CE" => Some("America/Fortaleza"),
        b"DF" => Some("America/Sao_Paulo"),
        b"ES" => Some("America/Sao_Paulo"),
        b"GO" => Some("America/Sao_Paulo"),
        b"MA" => Some("America/Fortaleza"),
        b"MG" => Some("America/Sao_Paulo"),
        b"MS" => Some("America/Campo_Grande"),
        b"MT" => Some("America/Cuiaba"),
        b"PA" => Some("America/Belem"),
        b"PB" => Some("America/Fortaleza"),
        b"PE" => Some("America/Recife"),
        b"PI" => Some("America/Fortaleza"),
        b"PR" => Some("America/Sao_Paulo"),
        b"RJ" => Some("America/Sao_Paulo"),
        b"RN" => Some("America/Fortaleza"),
        b"RO" => Some("America/Porto_Velho"),
        b"RR" => Some("America/Boa_Vista"),
        b"RS" => Some("America/Sao_Paulo"),
        b"SC" => Some("America/Sao_Paulo"),
        b"SE" => Some("America/Maceio"),
        b"SP" => Some("America/Sao_Paulo"),
        b"TO" => Some("America/Araguaina"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_ufs() {
        assert_eq!(timezone_for_uf("AC"), Some("America/Rio_Branco"));
        assert_eq!(timezone_for_uf("AL"), Some("America/Maceio"));
        assert_eq!(timezone_for_uf("AM"), Some("America/Manaus"));
        assert_eq!(timezone_for_uf("AP"), Some("America/Belem"));
        assert_eq!(timezone_for_uf("BA"), Some("America/Bahia"));
        assert_eq!(timezone_for_uf("CE"), Some("America/Fortaleza"));
        assert_eq!(timezone_for_uf("DF"), Some("America/Sao_Paulo"));
        assert_eq!(timezone_for_uf("ES"), Some("America/Sao_Paulo"));
        assert_eq!(timezone_for_uf("GO"), Some("America/Sao_Paulo"));
        assert_eq!(timezone_for_uf("MA"), Some("America/Fortaleza"));
        assert_eq!(timezone_for_uf("MG"), Some("America/Sao_Paulo"));
        assert_eq!(timezone_for_uf("MS"), Some("America/Campo_Grande"));
        assert_eq!(timezone_for_uf("MT"), Some("America/Cuiaba"));
        assert_eq!(timezone_for_uf("PA"), Some("America/Belem"));
        assert_eq!(timezone_for_uf("PB"), Some("America/Fortaleza"));
        assert_eq!(timezone_for_uf("PE"), Some("America/Recife"));
        assert_eq!(timezone_for_uf("PI"), Some("America/Fortaleza"));
        assert_eq!(timezone_for_uf("PR"), Some("America/Sao_Paulo"));
        assert_eq!(timezone_for_uf("RJ"), Some("America/Sao_Paulo"));
        assert_eq!(timezone_for_uf("RN"), Some("America/Fortaleza"));
        assert_eq!(timezone_for_uf("RO"), Some("America/Porto_Velho"));
        assert_eq!(timezone_for_uf("RR"), Some("America/Boa_Vista"));
        assert_eq!(timezone_for_uf("RS"), Some("America/Sao_Paulo"));
        assert_eq!(timezone_for_uf("SC"), Some("America/Sao_Paulo"));
        assert_eq!(timezone_for_uf("SE"), Some("America/Maceio"));
        assert_eq!(timezone_for_uf("SP"), Some("America/Sao_Paulo"));
        assert_eq!(timezone_for_uf("TO"), Some("America/Araguaina"));
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(timezone_for_uf("sp"), Some("America/Sao_Paulo"));
        assert_eq!(timezone_for_uf("Sp"), Some("America/Sao_Paulo"));
        assert_eq!(timezone_for_uf("am"), Some("America/Manaus"));
        assert_eq!(timezone_for_uf("Am"), Some("America/Manaus"));
    }

    #[test]
    fn test_invalid_uf() {
        assert_eq!(timezone_for_uf("XX"), None);
        assert_eq!(timezone_for_uf(""), None);
        assert_eq!(timezone_for_uf("A"), None);
        assert_eq!(timezone_for_uf("ABC"), None);
    }

    #[test]
    fn test_all_27_ufs_covered() {
        let all_ufs = [
            "AC", "AL", "AM", "AP", "BA", "CE", "DF", "ES", "GO", "MA", "MG", "MS", "MT", "PA",
            "PB", "PE", "PI", "PR", "RJ", "RN", "RO", "RR", "RS", "SC", "SE", "SP", "TO",
        ];
        assert_eq!(all_ufs.len(), 27);
        for uf in &all_ufs {
            assert!(
                timezone_for_uf(uf).is_some(),
                "UF {} should have a timezone mapping",
                uf
            );
        }
    }
}
