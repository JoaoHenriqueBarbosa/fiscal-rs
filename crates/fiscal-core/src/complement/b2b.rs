//! Attach B2B financial tag to an authorized `<nfeProc>` XML (`<nfeProcB2B>` wrapper).

use crate::FiscalError;

use super::helpers::{extract_tag, normalize_nfe_proc_attrs, strip_newlines};

/// Attach a B2B financial tag to an authorized `<nfeProc>` XML,
/// wrapping both in a `<nfeProcB2B>` element.
///
/// # Arguments
///
/// * `nfe_proc_xml` - The authorized nfeProc XML.
/// * `b2b_xml` - The B2B financial XML (must contain the `tag_b2b` element).
/// * `tag_b2b` - Optional B2B tag name; defaults to `"NFeB2BFin"`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if:
/// - The `nfe_proc_xml` does not contain `<nfeProc>`
/// - The `b2b_xml` does not contain the expected B2B tag
/// - Either tag cannot be extracted
pub fn attach_b2b(
    nfe_proc_xml: &str,
    b2b_xml: &str,
    tag_b2b: Option<&str>,
) -> Result<String, FiscalError> {
    let tag_name = tag_b2b.unwrap_or("NFeB2BFin");

    if !nfe_proc_xml.contains("<nfeProc") {
        return Err(FiscalError::XmlParsing(
            "XML does not contain <nfeProc> — is this an authorized NFe?".into(),
        ));
    }

    let open_check = format!("<{tag_name}");
    if !b2b_xml.contains(&open_check) {
        return Err(FiscalError::XmlParsing(format!(
            "B2B XML does not contain <{tag_name}> tag"
        )));
    }

    let nfe_proc_content = extract_tag(nfe_proc_xml, "nfeProc")
        .ok_or_else(|| FiscalError::XmlParsing("Could not extract <nfeProc> from XML".into()))?;

    // PHP DOMDocument re-serializes <nfeProc> with xmlns before versao
    // (DOM canonical attribute ordering). We must match this behavior.
    let nfe_proc_normalized = normalize_nfe_proc_attrs(&nfe_proc_content);

    let b2b_content = extract_tag(b2b_xml, tag_name).ok_or_else(|| {
        FiscalError::XmlParsing(format!("Could not extract <{tag_name}> from B2B XML"))
    })?;

    let raw = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
         <nfeProcB2B>{nfe_proc_normalized}{b2b_content}</nfeProcB2B>"
    );

    // PHP Complements::b2bTag line 79 does:
    //   str_replace(array("\n", "\r", "\s"), '', $nfeb2bXML)
    // This removes newlines/carriage-returns (and the literal "\s" which is
    // a PHP quirk — "\s" inside single quotes is just the characters \ and s,
    // but that string never appears in XML anyway).
    let cleaned = strip_newlines(&raw);
    Ok(cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── attach_b2b tests ────────────────────────────────────────────

    #[test]
    fn attach_b2b_no_nfe_proc() {
        let err = attach_b2b("<NFe/>", "<NFeB2BFin>data</NFeB2BFin>", None).unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_b2b_no_b2b_tag() {
        let err = attach_b2b("<nfeProc><NFe/></nfeProc>", "<other>data</other>", None).unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_b2b_extract_failure() {
        // nfeProc without closing tag won't extract
        let err = attach_b2b("<nfeProc><NFe/>", "<NFeB2BFin>data</NFeB2BFin>", None).unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_b2b_success() {
        let result = attach_b2b(
            "<nfeProc><NFe/><protNFe/></nfeProc>",
            "<NFeB2BFin><tag>data</tag></NFeB2BFin>",
            None,
        )
        .unwrap();
        assert!(result.contains("<nfeProcB2B>"));
        assert!(result.contains("<nfeProc>"));
        assert!(result.contains("<NFeB2BFin>"));
    }

    #[test]
    fn attach_b2b_custom_tag() {
        let result = attach_b2b(
            "<nfeProc><NFe/><protNFe/></nfeProc>",
            "<CustomB2B><tag>data</tag></CustomB2B>",
            Some("CustomB2B"),
        )
        .unwrap();
        assert!(result.contains("<CustomB2B>"));
    }

    // ── attach_b2b whitespace stripping tests ───────────────────────────

    #[test]
    fn attach_b2b_strips_newlines() {
        let nfe_proc = "<nfeProc versao=\"4.00\">\n<NFe/>\n<protNFe/>\n</nfeProc>";
        let b2b = "<NFeB2BFin>\n<data>test</data>\n</NFeB2BFin>";
        let result = attach_b2b(nfe_proc, b2b, None).unwrap();
        assert!(!result.contains('\n'), "Result should not contain newlines");
        assert!(
            !result.contains('\r'),
            "Result should not contain carriage returns"
        );
        assert!(result.contains("<nfeProcB2B>"));
        assert!(result.contains("<NFeB2BFin>"));
    }

    #[test]
    fn attach_b2b_strips_carriage_returns() {
        let nfe_proc = "<nfeProc versao=\"4.00\">\r\n<NFe/>\r\n</nfeProc>";
        let b2b = "<NFeB2BFin><data>test</data></NFeB2BFin>";
        let result = attach_b2b(nfe_proc, b2b, None).unwrap();
        assert!(!result.contains('\r'));
        assert!(!result.contains('\n'));
    }

    // ── attach_b2b: extract_tag for b2b content (line 348) ──────────────

    #[test]
    fn attach_b2b_extract_tag_coverage() {
        let nfe_proc = concat!(
            r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe><infNFe/></NFe><protNFe><infProt/></protNFe>"#,
            r#"</nfeProc>"#
        );
        let b2b = r#"<NFeB2BFin versao="1.00"><dados>value</dados></NFeB2BFin>"#;
        let result = attach_b2b(nfe_proc, b2b, None).unwrap();
        assert!(result.contains("<nfeProcB2B>"));
        assert!(result.contains("<dados>value</dados>"));
    }
}
