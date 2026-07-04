//! Structural validation for CT-e (model 57) XML before sending to SEFAZ.
//!
//! Mirrors `fiscal_mdfe::validate`: the `fiscal-rs` workspace deliberately
//! does **not** link libxml here — full XSD validation lives in `fiscal-xsd`
//! (opt-in) and in the hub. This performs comprehensive **structural** checks
//! that catch the same classes of error before transmission.

use fiscal_core::FiscalError;

use crate::{CTE_NAMESPACE, CTE_VERSION};

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

    let mut depth: i64 = 0;
    let mut had_element = false;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(_)) => {
                depth += 1;
                had_element = true;
            }
            Ok(Event::End(_)) => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            Ok(Event::Empty(_)) => had_element = true,
            Ok(Event::Eof) => break,
            Err(_) => return false,
            _ => {}
        }
        buf.clear();
    }
    had_element && depth == 0
}

/// Validate the structure of a CT-e XML document before sending to SEFAZ.
///
/// Structural (not XSD) validation: well-formedness, the `<CTe>`/`<infCte>`
/// root, the CT-e namespace, the `versao` attribute, the `Id` (`"CTe"` + 44
/// digits), the mandatory `<ide>`/`<emit>`/`<vPrest>`/`<imp>`/`<infCTeNorm>`
/// blocks, and a `<Signature>` element.
///
/// # Errors
///
/// Returns `FiscalError::XmlParsing` listing every failure found, separated
/// by `"; "`.
pub fn validate_cte_xml(xml: &str) -> Result<(), FiscalError> {
    if xml.trim().is_empty() {
        return Err(FiscalError::XmlParsing(
            "Validação CT-e: a string do CT-e está vazia".to_string(),
        ));
    }
    if !is_valid_xml(xml) {
        return Err(FiscalError::XmlParsing(
            "A string passada não é um XML válido".to_string(),
        ));
    }

    let mut errors: Vec<String> = Vec::new();

    if !xml.contains("<CTe") {
        errors.push("Elemento raiz <CTe> ausente".to_string());
    }
    if !xml.contains("<infCte") {
        errors.push("Elemento <infCte> ausente".to_string());
    }
    if !xml.contains(CTE_NAMESPACE) {
        errors.push(format!("Namespace CT-e ausente ({CTE_NAMESPACE})"));
    }

    // versao + Id on <infCte>
    if let Some(pos) = xml.find("<infCte") {
        let after = &xml[pos..];
        match find_attr(after, "versao") {
            Some(v) if v == CTE_VERSION => {}
            Some(v) => errors.push(format!(
                "Versão do XML ({v}) não corresponde à esperada ({CTE_VERSION})"
            )),
            None => errors.push("Atributo versao ausente em <infCte>".to_string()),
        }
        match find_attr(after, "Id") {
            Some(id) => {
                let valid = id.len() == 47
                    && id.starts_with("CTe")
                    && id[3..].bytes().all(|b| b.is_ascii_digit());
                if !valid {
                    errors.push(format!(
                        "Id de <infCte> inválido ({id}): esperado \"CTe\" + 44 dígitos"
                    ));
                }
            }
            None => errors.push("Atributo Id ausente em <infCte>".to_string()),
        }
    }

    for (tagname, label) in [
        ("<ide", "<ide>"),
        ("<emit", "<emit>"),
        ("<vPrest", "<vPrest>"),
        ("<imp", "<imp>"),
    ] {
        if !xml.contains(tagname) {
            errors.push(format!("Bloco obrigatório {label} ausente"));
        }
    }

    // CT-e Normal/Substituto usa <infCTeNorm>; Complementar (tpCTe=1) usa
    // <infCteComp>. Pelo menos um dos dois deve estar presente.
    if !xml.contains("<infCTeNorm") && !xml.contains("<infCteComp") {
        errors.push(
            "Bloco <infCTeNorm> (normal/substituto) ou <infCteComp> (complementar) ausente"
                .to_string(),
        );
    }

    if !xml.contains("<Signature") {
        errors.push("Assinatura digital <Signature> ausente".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(FiscalError::XmlParsing(format!(
            "Validação estrutural do CT-e falhou: {}",
            errors.join("; ")
        )))
    }
}

/// Extract the value of an unquoted-name `attr="value"` occurrence in `s`.
fn find_attr<'a>(s: &'a str, attr: &str) -> Option<&'a str> {
    let needle = format!("{attr}=\"");
    let start = s.find(&needle)? + needle.len();
    let end = s[start..].find('"')? + start;
    Some(&s[start..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty() {
        assert!(validate_cte_xml("").is_err());
    }

    #[test]
    fn rejects_non_xml() {
        assert!(validate_cte_xml("not xml").is_err());
    }

    #[test]
    fn detects_missing_blocks() {
        let xml = format!(
            "<CTe xmlns=\"{CTE_NAMESPACE}\"><infCte versao=\"{CTE_VERSION}\" Id=\"CTe{}\"></infCte></CTe>",
            "1".repeat(44)
        );
        let err = validate_cte_xml(&xml).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("<ide>"));
        assert!(msg.contains("<Signature>"));
    }
}
