//! TXT-to-XML converter for NF-e/NFC-e.
//!
//! Converts pipe-delimited TXT files (official SEFAZ layout) into NF-e XML.

mod builder;
mod helpers;
mod parser;
mod structures;
mod types;

use std::collections::HashMap;

use crate::FiscalError;

use structures::*;
use helpers::validate_txt_lines;
use parser::NFeParser;


// ── TXT structures ──────────────────────────────────────────────────────────

/// Return the TXT field structure map for a given version string and layout.
///
/// # Errors
///
/// Returns [`FiscalError::InvalidTxt`] if the version/layout combination
/// is not supported.
pub(super) fn get_structure(
    version: &str,
    layout: &str,
) -> Result<HashMap<&'static str, &'static str>, FiscalError> {
    let ver: u32 = version.replace('.', "").parse().unwrap_or(0);
    let lay = layout.to_uppercase();

    if ver == 310 {
        return Ok(structure_310());
    }
    if ver == 400 {
        if lay == "SEBRAE" {
            return Ok(structure_400_sebrae());
        }
        if lay == "LOCAL_V12" {
            return Ok(structure_400_v12());
        }
        if lay == "LOCAL_V13" {
            return Ok(structure_400_v13());
        }
        return Ok(structure_400());
    }

    Err(FiscalError::InvalidTxt(format!(
        "Structure definition for TXT layout version {version} ({layout}) was not found."
    )))
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Convert SPED TXT format to NF-e XML (first invoice only).
///
/// Convenience wrapper around [`txt_to_xml_all`] that returns only the first
/// invoice XML. Use this when you know the TXT contains a single NF-e.
///
/// # Errors
///
/// Returns [`FiscalError::InvalidTxt`] if the TXT is empty, not a valid
/// NOTAFISCAL document, has structural errors, or if the access key is
/// malformed. Returns [`FiscalError::WrongDocument`] if the document header
/// is missing.
pub fn txt_to_xml(txt: &str, layout: &str) -> Result<String, FiscalError> {
    let mut xmls = txt_to_xml_all(txt, layout)?;
    // txt_to_xml_all guarantees at least one element on success.
    Ok(xmls.swap_remove(0))
}

/// Convert SPED TXT format to NF-e XML for **all** invoices in the file.
///
/// Parses the pipe-delimited TXT representation of one or more NF-e invoices
/// and produces a `Vec<String>` containing the XML for each invoice, in the
/// same order they appear in the TXT. Supports layouts:
/// `"local"`, `"local_v12"`, `"local_v13"`, `"sebrae"`.
///
/// This mirrors the PHP `Convert::parse()` / `toXml()` behaviour which
/// returns an array of XML strings — one per nota fiscal.
///
/// # Errors
///
/// Returns [`FiscalError::InvalidTxt`] if the TXT is empty, not a valid
/// NOTAFISCAL document, has structural errors, or if the access key is
/// malformed. Returns [`FiscalError::WrongDocument`] if the document header
/// is missing or the declared invoice count does not match.
pub fn txt_to_xml_all(txt: &str, layout: &str) -> Result<Vec<String>, FiscalError> {
    let txt = txt.trim();
    if txt.is_empty() {
        return Err(FiscalError::WrongDocument("Empty document".into()));
    }

    let lines: Vec<&str> = txt.lines().collect();
    let first_fields: Vec<&str> = lines[0].split('|').collect();
    if first_fields[0] != "NOTAFISCAL" {
        return Err(FiscalError::WrongDocument(
            "Wrong document: not a valid NFe TXT".into(),
        ));
    }

    let declared_count: usize = first_fields
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let rest: Vec<&str> = lines[1..].to_vec();

    // Slice invoices
    let invoices = slice_invoices(&rest, declared_count);
    if invoices.len() != declared_count {
        return Err(FiscalError::WrongDocument(format!(
            "Number of NFe declared ({declared_count}) does not match found ({})",
            invoices.len()
        )));
    }

    let norm_layout = normalize_layout(layout);
    let mut xmls = Vec::with_capacity(declared_count);

    for invoice in &invoices {
        let version = extract_layout_version(invoice)?;

        // Validate
        let errors = validate_txt_lines(invoice, &norm_layout);
        if !errors.is_empty() {
            return Err(FiscalError::InvalidTxt(errors.join("\n")));
        }

        // Parse
        let structure = get_structure(&version, &norm_layout)?;
        let mut parser = NFeParser::new(&version, &norm_layout, &structure);
        parser.parse(invoice);

        // Validate access key
        if !parser.inf_nfe_id.is_empty() {
            let key = parser
                .inf_nfe_id
                .strip_prefix("NFe")
                .unwrap_or(&parser.inf_nfe_id);
            if !key.is_empty() && key.len() != 44 {
                return Err(FiscalError::InvalidTxt(format!(
                    "A chave informada est\u{e1} incorreta [{}]",
                    parser.inf_nfe_id
                )));
            }
        }

        xmls.push(parser.build_xml());
    }

    Ok(xmls)
}

/// Validate TXT format structure without converting to XML.
///
/// Returns `Ok(true)` if the TXT passes structural validation, or
/// `Ok(false)` / `Err` if validation errors are found.
///
/// # Errors
///
/// Returns [`FiscalError::WrongDocument`] if the document header is missing
/// or empty.
pub fn validate_txt(txt: &str, layout: &str) -> Result<bool, FiscalError> {
    let txt = txt.replace(['\r', '\t'], "");
    let txt = txt.trim();
    if txt.is_empty() {
        return Err(FiscalError::WrongDocument("Empty document".into()));
    }

    let lines: Vec<&str> = txt.lines().collect();
    let first_fields: Vec<&str> = lines[0].split('|').collect();
    if first_fields[0] != "NOTAFISCAL" {
        return Err(FiscalError::WrongDocument(
            "Wrong document: not a valid NFe TXT".into(),
        ));
    }

    let rest: Vec<&str> = lines[1..].to_vec();
    let norm_layout = normalize_layout(layout);
    let errors = validate_txt_lines(&rest, &norm_layout);

    if errors.is_empty() {
        Ok(true)
    } else {
        Ok(false)
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn normalize_layout(layout: &str) -> String {
    let up = layout.to_uppercase();
    match up.as_str() {
        "LOCAL" | "LOCAL_V12" | "LOCAL_V13" | "SEBRAE" => up,
        _ => "LOCAL_V12".to_string(),
    }
}

fn slice_invoices<'a>(rest: &[&'a str], declared: usize) -> Vec<Vec<&'a str>> {
    if declared <= 1 {
        return vec![rest.to_vec()];
    }

    let mut markers: Vec<(usize, usize)> = Vec::new();
    for (i, line) in rest.iter().enumerate() {
        if line.starts_with("A|") {
            if let Some(last) = markers.last_mut() {
                last.1 = i;
            }
            markers.push((i, 0));
        }
    }
    if let Some(last) = markers.last_mut() {
        last.1 = rest.len();
    }

    markers.iter().map(|(s, e)| rest[*s..*e].to_vec()).collect()
}

fn extract_layout_version(invoice: &[&str]) -> Result<String, FiscalError> {
    for line in invoice {
        let fields: Vec<&str> = line.split('|').collect();
        if fields[0] == "A" {
            return Ok(fields.get(1).unwrap_or(&"4.00").to_string());
        }
    }
    Err(FiscalError::InvalidTxt(
        "No 'A' entity found in invoice".into(),
    ))
}
