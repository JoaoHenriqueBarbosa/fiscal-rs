//! Structural validation for MDF-e (model 58) XML before sending to SEFAZ.
//!
//! Mirrors the approach of `fiscal_sefaz`'s `validate_nfe_xml`: the
//! `fiscal-rs` workspace deliberately does **not** ship XSD schemas or link
//! libxml — full XSD validation lives in the hub (which already has libxml2 +
//! the official schemas). This performs comprehensive **structural** checks
//! that catch the same classes of error before transmission.

use fiscal_core::FiscalError;
use fiscal_core::xml_utils::extract_xml_tag_value;

use crate::{MDFE_NAMESPACE, MDFE_VERSION};

/// Check whether a string is well-formed XML (balanced tags, at least one
/// element, not HTML).
pub fn is_valid_xml(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return false;
    }

    let lower = trimmed.to_lowercase();
    if lower.contains("<!doctype html>") || lower.contains("</html>") {
        return false;
    }

    let effective = trimmed.trim_start_matches('\u{feff}');
    if !effective.starts_with('<') {
        return false;
    }

    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(trimmed);
    reader.config_mut().trim_text(true);

    let mut depth: usize = 0;
    let mut had_element = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(_)) => {
                depth += 1;
                had_element = true;
            }
            Ok(Event::End(_)) => {
                if depth == 0 {
                    return false;
                }
                depth -= 1;
            }
            Ok(Event::Empty(_)) => {
                had_element = true;
            }
            Ok(Event::Eof) => {
                return had_element && depth == 0;
            }
            Err(_) => return false,
            _ => {}
        }
    }
}

/// Validate the structure of an MDF-e XML document before sending to SEFAZ.
///
/// This is the MDF-e analogue of `fiscal_sefaz::validate::validate_nfe_xml`.
/// It performs structural validation rather than XSD validation:
///
/// 1. **Well-formedness** — the XML must parse without errors.
/// 2. **Root elements** — `<MDFe>` and `<infMDFe>` must be present.
/// 3. **Namespace** — the MDF-e namespace must be declared.
/// 4. **Version** — the `versao` attribute on `<infMDFe>` must be `3.00`.
/// 5. **Access key** — `Id` on `<infMDFe>` must be `"MDFe"` + 44 digits.
/// 6. **Mandatory fields** — required `<ide>` / `<emit>` tags and the
///    `<infModal>`, `<infDoc>`, `<tot>` blocks.
/// 7. **Digital signature** — a `<Signature>` block must be present.
///
/// # Arguments
///
/// * `xml` — the complete MDF-e XML string (signed, ready to send).
///
/// # Errors
///
/// Returns `FiscalError::XmlParsing` if the XML is empty, not well-formed,
/// or any required structural element is missing or incorrect. The message
/// lists every failure found, separated by `"; "`.
pub fn validate_mdfe_xml(xml: &str) -> Result<(), FiscalError> {
    if xml.trim().is_empty() {
        return Err(FiscalError::XmlParsing(
            "Validação MDF-e: a string do MDF-e está vazia".to_string(),
        ));
    }

    if !is_valid_xml(xml) {
        return Err(FiscalError::XmlParsing(
            "A string passada não é um XML válido".to_string(),
        ));
    }

    let mut errors: Vec<String> = Vec::new();

    // --- Root structure ---
    if !xml.contains("<MDFe") {
        errors.push("Elemento raiz <MDFe> ausente".to_string());
    }
    if !xml.contains("<infMDFe") {
        errors.push("Elemento <infMDFe> ausente".to_string());
    }

    // --- Namespace ---
    if !xml.contains(MDFE_NAMESPACE) {
        errors.push(format!("Namespace MDF-e ausente ({MDFE_NAMESPACE})"));
    }

    // --- Version attribute on <infMDFe> ---
    if let Some(pos) = xml.find("<infMDFe") {
        let after = &xml[pos..];
        if let Some(versao_pos) = after.find("versao=\"") {
            let ver_start = versao_pos + 8;
            if let Some(ver_end) = after[ver_start..].find('"') {
                let found_version = &after[ver_start..ver_start + ver_end];
                if found_version != MDFE_VERSION {
                    errors.push(format!(
                        "Versão do XML ({found_version}) não corresponde à versão esperada ({MDFE_VERSION})"
                    ));
                }
            }
        } else {
            errors.push("Atributo versao ausente em <infMDFe>".to_string());
        }
    }

    // --- Access key format (Id="MDFe" + 44 digits) ---
    if let Some(id_start) = xml.find("Id=\"MDFe") {
        let after_id = &xml[id_start + 8..]; // skip `Id="MDFe`
        if let Some(quote_end) = after_id.find('"') {
            let key = &after_id[..quote_end];
            if key.len() != 44 || !key.chars().all(|c| c.is_ascii_digit()) {
                errors.push(format!(
                    "Chave de acesso inválida: esperado 44 dígitos, encontrado '{key}'"
                ));
            }
        }
    } else if xml.contains("<infMDFe") {
        errors.push("Atributo Id com chave de acesso ausente em <infMDFe>".to_string());
    }

    // --- IDE required tags ---
    let ide_tags = [
        "cUF", "tpAmb", "tpEmit", "mod", "serie", "nMDF", "cMDF", "cDV", "modal", "dhEmi",
        "tpEmis", "procEmi", "verProc", "UFIni", "UFFim",
    ];
    for tag_name in &ide_tags {
        if extract_xml_tag_value(xml, tag_name).is_none() {
            errors.push(format!("Tag obrigatória <{tag_name}> ausente em <ide>"));
        }
    }

    // --- EMIT required tags ---
    if extract_xml_tag_value(xml, "xNome").is_none() {
        errors.push("Tag obrigatória <xNome> ausente em <emit>".to_string());
    }
    if extract_xml_tag_value(xml, "CNPJ").is_none() && extract_xml_tag_value(xml, "CPF").is_none() {
        errors.push("Tag <CNPJ> ou <CPF> ausente em <emit>".to_string());
    }
    if !xml.contains("<enderEmit") {
        errors.push("Bloco <enderEmit> ausente".to_string());
    }

    // --- Required blocks ---
    if !xml.contains("<infModal") {
        errors.push("Bloco <infModal> ausente".to_string());
    }
    if !xml.contains("<infDoc") {
        errors.push("Bloco <infDoc> ausente".to_string());
    }
    if !xml.contains("<tot") {
        errors.push("Bloco <tot> ausente".to_string());
    }

    // --- Digital signature ---
    if !xml.contains("<Signature") {
        errors.push("Assinatura digital <Signature> ausente".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(FiscalError::XmlParsing(format!(
            "Este XML não é válido. {}",
            errors.join("; ")
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_is_invalid() {
        assert!(validate_mdfe_xml("").is_err());
        assert!(!is_valid_xml(""));
    }

    #[test]
    fn well_formed_detection() {
        assert!(is_valid_xml("<a><b>x</b></a>"));
        assert!(!is_valid_xml("<a><b></a>"));
        assert!(!is_valid_xml("not xml"));
        assert!(!is_valid_xml("<!DOCTYPE html><html></html>"));
    }

    #[test]
    fn incomplete_mdfe_reports_missing_blocks() {
        let err = validate_mdfe_xml("<MDFe><infMDFe/></MDFe>").unwrap_err();
        let msg = err.to_string();
        // A bare skeleton must flag missing namespace, signature, and ide tags.
        assert!(msg.contains("Namespace"));
        assert!(msg.contains("<Signature>"));
        assert!(msg.contains("<infModal>"));
    }
}
