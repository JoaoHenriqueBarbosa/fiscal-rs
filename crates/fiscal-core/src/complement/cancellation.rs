//! Attach cancellation event response to an authorized `<nfeProc>` XML.

use crate::FiscalError;
use crate::xml_utils::extract_xml_tag_value;

use super::helpers::{dom_reserialize_nfe_proc, extract_all_tags, extract_tag};

/// Cancellation event type code (`110111`).
const EVT_CANCELA: &str = "110111";
/// Cancellation by substitution event type code (`110112`).
const EVT_CANCELA_SUBSTITUICAO: &str = "110112";

/// Valid status codes for cancellation event matching.
///
/// - `135` — Event registered and linked
/// - `136` — Event registered but not linked
/// - `155` — Already cancelled (late)
const VALID_CANCEL_STATUSES: &[&str] = &["135", "136", "155"];

/// Attach a cancellation event response to an authorized `<nfeProc>` XML,
/// marking the NF-e as locally cancelled.
///
/// This mirrors the PHP `Complements::cancelRegister()` method. The function
/// searches the `cancel_event_xml` for `<retEvento>` elements whose:
/// - `cStat` is in `[135, 136, 155]` (valid cancellation statuses)
/// - `tpEvento` is `110111` (cancellation) or `110112` (cancellation by substitution)
/// - `chNFe` matches the access key in the authorized NF-e's `<protNFe>`
///
/// When a matching `<retEvento>` is found, it is appended inside the
/// `<nfeProc>` element (before the closing `</nfeProc>` tag).
///
/// If no matching cancellation event is found, the original NF-e XML is
/// returned unchanged (same behavior as the PHP implementation).
///
/// # Arguments
///
/// * `nfe_proc_xml` - The authorized NF-e XML containing `<nfeProc>` with `<protNFe>`.
/// * `cancel_event_xml` - The SEFAZ cancellation event response XML containing `<retEvento>`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if:
/// - The `nfe_proc_xml` does not contain `<protNFe>` (not an authorized NF-e)
/// - The `<protNFe>` does not contain `<chNFe>`
pub fn attach_cancellation(
    nfe_proc_xml: &str,
    cancel_event_xml: &str,
) -> Result<String, FiscalError> {
    // Validate the NF-e has a protNFe with a chNFe
    let prot_nfe = extract_tag(nfe_proc_xml, "protNFe").ok_or_else(|| {
        FiscalError::XmlParsing(
            "Could not find <protNFe> in NF-e XML — is this an authorized NF-e?".into(),
        )
    })?;

    let ch_nfe = extract_xml_tag_value(&prot_nfe, "chNFe")
        .ok_or_else(|| FiscalError::XmlParsing("Could not find <chNFe> inside <protNFe>".into()))?;

    // Search for matching retEvento in the cancellation XML
    let ret_eventos = extract_all_tags(cancel_event_xml, "retEvento");

    let mut matched_ret_evento: Option<&str> = None;

    for ret_evento in &ret_eventos {
        let c_stat = match extract_xml_tag_value(ret_evento, "cStat") {
            Some(v) => v,
            None => continue,
        };
        let tp_evento = match extract_xml_tag_value(ret_evento, "tpEvento") {
            Some(v) => v,
            None => continue,
        };
        let ch_nfe_evento = match extract_xml_tag_value(ret_evento, "chNFe") {
            Some(v) => v,
            None => continue,
        };

        if VALID_CANCEL_STATUSES.contains(&c_stat.as_str())
            && (tp_evento == EVT_CANCELA || tp_evento == EVT_CANCELA_SUBSTITUICAO)
            && ch_nfe_evento == ch_nfe
        {
            matched_ret_evento = Some(ret_evento.as_str());
            break;
        }
    }

    // Re-serialize via DOM-like logic to match PHP DOMDocument::saveXML().
    // PHP always re-serializes even when no match is found — reordering
    // xmlns before versao and emitting an XML declaration + newline.
    dom_reserialize_nfe_proc(nfe_proc_xml, matched_ret_evento)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── attach_cancellation tests ─────────────────────────────────────

    #[test]
    fn attach_cancellation_appends_matching_ret_evento() {
        let nfe_proc = concat!(
            r#"<?xml version="1.0" encoding="UTF-8"?>"#,
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe><infNFe versao="4.00" Id="NFe35260112345678000199550010000000011123456780">"#,
            r#"<ide/></infNFe></NFe>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat><nProt>135220000009921</nProt>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEnvEvento><retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat>"#,
            r#"<xMotivo>Evento registrado e vinculado a NF-e</xMotivo>"#,
            r#"<tpEvento>110111</tpEvento>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento></retEnvEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();

        // Must contain the retEvento inside nfeProc
        assert!(
            result.contains("<retEvento"),
            "Result should contain <retEvento>"
        );
        assert!(
            result.contains("<tpEvento>110111</tpEvento>"),
            "Result should contain cancellation event type"
        );
        // The retEvento should appear before </nfeProc>
        let ret_pos = result.find("<retEvento").unwrap();
        let close_pos = result.rfind("</nfeProc>").unwrap();
        assert!(ret_pos < close_pos, "retEvento should be before </nfeProc>");
        // Original content should be preserved
        assert!(result.contains("<protNFe"));
        assert!(result.contains("<NFe>"));
    }

    #[test]
    fn attach_cancellation_ignores_non_matching_ch_nfe() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat>"#,
            r#"<tpEvento>110111</tpEvento>"#,
            r#"<chNFe>99999999999999999999999999999999999999999999</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();
        // No matching chNFe — should NOT contain retEvento, but still
        // re-serialized like PHP (declaration + xmlns before versao)
        assert!(
            !result.contains("<retEvento"),
            "Should not contain retEvento"
        );
        assert!(
            result.starts_with("<?xml version=\"1.0\"?>\n"),
            "Should have XML declaration (no encoding since input had none)"
        );
        assert!(
            result
                .contains(r#"<nfeProc xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00">"#),
            "Should reorder xmlns before versao"
        );
    }

    #[test]
    fn attach_cancellation_ignores_wrong_tp_evento() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat>"#,
            r#"<tpEvento>110110</tpEvento>"#, // CCe, not cancellation
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();
        // Wrong tpEvento — should NOT contain retEvento
        assert!(
            !result.contains("<retEvento"),
            "Should not contain retEvento"
        );
        assert!(
            result.starts_with("<?xml version=\"1.0\"?>\n"),
            "Should have XML declaration"
        );
    }

    #[test]
    fn attach_cancellation_ignores_rejected_status() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>573</cStat>"#, // Rejected status
            r#"<tpEvento>110111</tpEvento>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();
        // Rejected status — should NOT contain retEvento
        assert!(
            !result.contains("<retEvento"),
            "Should not contain retEvento"
        );
        assert!(
            result.starts_with("<?xml version=\"1.0\"?>\n"),
            "Should have XML declaration"
        );
    }

    #[test]
    fn attach_cancellation_accepts_status_155() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>155</cStat>"#,
            r#"<tpEvento>110111</tpEvento>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();
        assert!(result.contains("<retEvento"));
    }

    #[test]
    fn attach_cancellation_accepts_substituicao_110112() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat>"#,
            r#"<tpEvento>110112</tpEvento>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();
        assert!(
            result.contains("<tpEvento>110112</tpEvento>"),
            "Should accept cancellation by substitution"
        );
    }

    #[test]
    fn attach_cancellation_rejects_missing_prot_nfe() {
        let nfe_xml = "<NFe><infNFe/></NFe>";
        let cancel_xml = "<retEvento/>";
        let err = attach_cancellation(nfe_xml, cancel_xml).unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_cancellation_rejects_missing_ch_nfe_in_prot() {
        let nfe_proc = concat!(
            r#"<nfeProc><protNFe versao="4.00"><infProt>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe></nfeProc>"#
        );
        let cancel_xml = "<retEvento/>";
        let err = attach_cancellation(nfe_proc, cancel_xml).unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    /// Byte-for-byte parity test: Rust output must match what PHP
    /// `Complements::cancelRegister()` produces for the same inputs.
    ///
    /// PHP uses `DOMDocument::saveXML()` which:
    /// 1. Emits `<?xml version="1.0" encoding="UTF-8"?>` + `\n`
    /// 2. Reorders `xmlns` before `versao` on `<nfeProc>`
    /// 3. Appends `<retEvento>` as last child of `<nfeProc>`
    #[test]
    fn attach_cancellation_parity_with_php() {
        // Input: authorized nfeProc (as produced by join_xml / PHP join())
        let nfe_proc = concat!(
            r#"<?xml version="1.0" encoding="UTF-8"?>"#,
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe><infNFe versao="4.00" Id="NFe35260112345678000199550010000000011123456780">"#,
            r#"<ide/></infNFe></NFe>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<digVal>abc</digVal>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"<xMotivo>Autorizado</xMotivo>"#,
            r#"<nProt>135220000009921</nProt>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEnvEvento><retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat>"#,
            r#"<xMotivo>Evento registrado e vinculado a NF-e</xMotivo>"#,
            r#"<tpEvento>110111</tpEvento>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento></retEnvEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();

        // Expected output from PHP DOMDocument::saveXML():
        // - declaration with encoding="UTF-8" followed by \n
        // - <nfeProc xmlns="..." versao="..."> (xmlns first)
        // - inner content unchanged
        // - <retEvento> appended before </nfeProc>
        let expected = concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            r#"<nfeProc xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00">"#,
            r#"<NFe><infNFe versao="4.00" Id="NFe35260112345678000199550010000000011123456780">"#,
            r#"<ide/></infNFe></NFe>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<digVal>abc</digVal>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"<xMotivo>Autorizado</xMotivo>"#,
            r#"<nProt>135220000009921</nProt>"#,
            r#"</infProt></protNFe>"#,
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat>"#,
            r#"<xMotivo>Evento registrado e vinculado a NF-e</xMotivo>"#,
            r#"<tpEvento>110111</tpEvento>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento>"#,
            "</nfeProc>\n"
        );

        assert_eq!(result, expected);
    }

    /// Parity test for no-match case: PHP still re-serializes through saveXML().
    #[test]
    fn attach_cancellation_no_match_still_reserializes_like_php() {
        let nfe_proc = concat!(
            r#"<?xml version="1.0" encoding="UTF-8"?>"#,
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEnvEvento><retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat>"#,
            r#"<tpEvento>110111</tpEvento>"#,
            r#"<chNFe>99999999999999999999999999999999999999999999</chNFe>"#,
            r#"<nProt>135220000009999</nProt>"#,
            r#"</infEvento></retEvento></retEnvEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();

        // PHP DOMDocument::saveXML() output (no retEvento appended):
        let expected = concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            r#"<nfeProc xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            "</nfeProc>\n"
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn attach_cancellation_picks_first_matching_from_multiple_ret_eventos() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe/>"#,
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"</infProt></protNFe>"#,
            r#"</nfeProc>"#
        );

        let cancel_xml = concat!(
            r#"<retEnvEvento>"#,
            // First: wrong chNFe
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat><tpEvento>110111</tpEvento>"#,
            r#"<chNFe>99999999999999999999999999999999999999999999</chNFe>"#,
            r#"<nProt>111111111111111</nProt>"#,
            r#"</infEvento></retEvento>"#,
            // Second: correct match
            r#"<retEvento versao="1.00"><infEvento>"#,
            r#"<cStat>135</cStat><tpEvento>110111</tpEvento>"#,
            r#"<chNFe>35260112345678000199550010000000011123456780</chNFe>"#,
            r#"<nProt>222222222222222</nProt>"#,
            r#"</infEvento></retEvento>"#,
            r#"</retEnvEvento>"#
        );

        let result = attach_cancellation(nfe_proc, cancel_xml).unwrap();
        assert!(result.contains("<nProt>222222222222222</nProt>"));
        // Should only have one retEvento (the matching one)
        assert_eq!(result.matches("<retEvento").count(), 1);
    }
}
