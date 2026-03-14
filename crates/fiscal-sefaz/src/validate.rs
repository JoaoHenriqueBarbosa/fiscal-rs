//! NF-e XML validation utilities.
//!
//! This module provides two layers of validation mirroring the PHP `sped-nfe`
//! library:
//!
//! - **Pre-send validation** ([`validate_nfe_xml`]) — structural checks on an
//!   NF-e XML before it is submitted to SEFAZ.  Equivalent to the PHP
//!   `Validator::isValid()` / `Tools::isValid()` which validates against XSD
//!   schemas. Since shipping full XSD files is impractical in a Rust library,
//!   this performs comprehensive structural validation: well-formedness, correct
//!   root element, namespaces, required fields, and access key format.
//!
//! - **Post-authorization validation** ([`SefazClient::sefaz_validate`]) —
//!   queries SEFAZ by access key and verifies that the local protocol number,
//!   digest value, and access key match the SEFAZ records.  Equivalent to the
//!   PHP `Tools::sefazValidate()`.
//!
//! # Examples
//!
//! ```
//! use fiscal_sefaz::validate::{validate_nfe_xml, is_valid_xml};
//!
//! // Check if a string is valid XML
//! assert!(is_valid_xml("<root><child/></root>"));
//! assert!(!is_valid_xml("not xml"));
//!
//! // Validate NF-e structure (will fail because the XML is incomplete)
//! let result = validate_nfe_xml("<NFe/>", "4.00");
//! assert!(result.is_err());
//! ```

use fiscal_core::FiscalError;
use fiscal_core::xml_utils::extract_xml_tag_value;

/// Check whether a string is well-formed XML.
///
/// Mirrors the PHP `Validator::isXML()` method. Returns `false` for empty
/// strings, HTML documents, and malformed XML.
///
/// # Examples
///
/// ```
/// use fiscal_sefaz::validate::is_valid_xml;
///
/// assert!(is_valid_xml("<root><child>text</child></root>"));
/// assert!(!is_valid_xml(""));
/// assert!(!is_valid_xml("<!DOCTYPE html><html></html>"));
/// assert!(!is_valid_xml("not xml at all"));
/// ```
pub fn is_valid_xml(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Reject HTML documents (same as PHP)
    let lower = trimmed.to_lowercase();
    if lower.contains("<!doctype html>") || lower.contains("</html>") {
        return false;
    }

    // XML must start with '<' (optionally preceded by a BOM)
    let effective = trimmed.trim_start_matches('\u{feff}');
    if !effective.starts_with('<') {
        return false;
    }

    // Try to parse as XML using quick-xml, tracking open/close balance
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
                    return false; // more closes than opens
                }
                depth -= 1;
            }
            Ok(Event::Empty(_)) => {
                had_element = true;
            }
            Ok(Event::Eof) => {
                // Valid only if we saw at least one element and all tags are closed
                return had_element && depth == 0;
            }
            Err(_) => return false,
            _ => {}
        }
    }
}

/// Validate the structure of an NF-e XML document before sending to SEFAZ.
///
/// This is the Rust equivalent of the PHP `Validator::isValid()` +
/// `Tools::isValid()` chain. Since full XSD schema validation requires
/// shipping `.xsd` files and a schema-aware XML parser, this function
/// performs **comprehensive structural validation** that catches the same
/// classes of errors:
///
/// 1. **Well-formedness** — the XML must parse without errors.
/// 2. **Root element** — `<NFe>` must be present.
/// 3. **Namespace** — the NF-e namespace must be declared.
/// 4. **Version** — the `versao` attribute on `<infNFe>` must match the
///    expected version.
/// 5. **Mandatory fields** — all required tags from `<ide>`, `<emit>`,
///    `<det>`, `<total>`, `<transp>`, and `<pag>` must exist.
/// 6. **Digital signature** — the `<Signature>` block must be present.
/// 7. **Access key** — the `Id` attribute on `<infNFe>` must contain a
///    valid 44-digit access key prefixed by `"NFe"`.
///
/// # Arguments
///
/// * `xml` — The complete NF-e XML string (signed, ready to send).
/// * `version` — Expected NF-e schema version, e.g. `"4.00"`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if:
/// - The XML string is empty or not well-formed.
/// - Any required structural element is missing or incorrect.
///
/// The error message lists all validation failures found, separated by `"; "`.
///
/// # Examples
///
/// ```
/// use fiscal_sefaz::validate::validate_nfe_xml;
///
/// // Minimal example — will fail because required tags are missing
/// let result = validate_nfe_xml("<NFe><infNFe/></NFe>", "4.00");
/// assert!(result.is_err());
/// ```
pub fn validate_nfe_xml(xml: &str, version: &str) -> Result<(), FiscalError> {
    if xml.trim().is_empty() {
        return Err(FiscalError::XmlParsing(
            "Validação NF-e: a string da NF-e está vazia".to_string(),
        ));
    }

    if !is_valid_xml(xml) {
        return Err(FiscalError::XmlParsing(
            "A string passada não é um XML válido".to_string(),
        ));
    }

    let mut errors: Vec<String> = Vec::new();

    // --- Root structure ---
    if !xml.contains("<NFe") {
        errors.push("Elemento raiz <NFe> ausente".to_string());
    }
    if !xml.contains("<infNFe") {
        errors.push("Elemento <infNFe> ausente".to_string());
    }

    // --- Namespace ---
    if !xml.contains("http://www.portalfiscal.inf.br/nfe") {
        errors.push("Namespace NF-e ausente (http://www.portalfiscal.inf.br/nfe)".to_string());
    }

    // --- Version attribute ---
    if let Some(pos) = xml.find("<infNFe") {
        let after = &xml[pos..];
        if let Some(versao_pos) = after.find("versao=\"") {
            let ver_start = versao_pos + 8;
            if let Some(ver_end) = after[ver_start..].find('"') {
                let found_version = &after[ver_start..ver_start + ver_end];
                if found_version != version {
                    errors.push(format!(
                        "Versão do XML ({found_version}) não corresponde à versão esperada ({version})"
                    ));
                }
            }
        } else {
            errors.push("Atributo versao ausente em <infNFe>".to_string());
        }
    }

    // --- Access key format (Id="NFe" + 44 digits) ---
    if let Some(id_start) = xml.find("Id=\"NFe") {
        let after_id = &xml[id_start + 7..]; // skip `Id="NFe`
        if let Some(quote_end) = after_id.find('"') {
            let key = &after_id[..quote_end];
            if key.len() != 44 || !key.chars().all(|c| c.is_ascii_digit()) {
                errors.push(format!(
                    "Chave de acesso inválida: esperado 44 dígitos, encontrado '{key}'"
                ));
            }
        }
    } else if xml.contains("<infNFe") {
        errors.push("Atributo Id com chave de acesso ausente em <infNFe>".to_string());
    }

    // --- IDE required tags ---
    let ide_tags = [
        "cUF", "cNF", "natOp", "mod", "serie", "nNF", "dhEmi", "tpNF", "idDest", "cMunFG", "tpImp",
        "tpEmis", "cDV", "tpAmb", "finNFe", "indFinal", "indPres", "procEmi", "verProc",
    ];
    for tag_name in &ide_tags {
        if extract_xml_tag_value(xml, tag_name).is_none() {
            errors.push(format!("Tag obrigatória <{tag_name}> ausente em <ide>"));
        }
    }

    // --- EMIT required tags ---
    let emit_required = ["xNome", "IE", "CRT"];
    for tag_name in &emit_required {
        if extract_xml_tag_value(xml, tag_name).is_none() {
            errors.push(format!("Tag obrigatória <{tag_name}> ausente em <emit>"));
        }
    }
    // CNPJ or CPF
    if extract_xml_tag_value(xml, "CNPJ").is_none() && extract_xml_tag_value(xml, "CPF").is_none() {
        errors.push("Tag <CNPJ> ou <CPF> ausente em <emit>".to_string());
    }

    // --- Required blocks ---
    if !xml.contains("<enderEmit") {
        errors.push("Bloco <enderEmit> ausente".to_string());
    }
    if !xml.contains("<det ") && !xml.contains("<det>") {
        errors.push("Nenhum item <det> encontrado".to_string());
    }
    if !xml.contains("<total") {
        errors.push("Bloco <total> ausente".to_string());
    }
    if !xml.contains("<ICMSTot") {
        errors.push("Bloco <ICMSTot> ausente".to_string());
    }
    if !xml.contains("<transp") {
        errors.push("Bloco <transp> ausente".to_string());
    }
    if !xml.contains("<pag") {
        errors.push("Bloco <pag> ausente".to_string());
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

/// Result of a SEFAZ post-authorization validation.
///
/// Returned by [`validate_authorized_nfe`] after comparing local document
/// data against the SEFAZ response.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    /// Whether all three fields (protocol, digest, access key) match.
    pub is_valid: bool,
    /// The 44-digit access key extracted from the local XML.
    pub access_key: String,
    /// The protocol number from the local XML.
    pub local_protocol: String,
    /// The protocol number returned by SEFAZ.
    pub sefaz_protocol: String,
    /// The digest value from the local XML signature.
    pub local_digest: String,
    /// The digest value returned by SEFAZ.
    pub sefaz_digest: String,
}

/// Extract validation data from an authorized NF-e XML for comparison
/// against the SEFAZ response.
///
/// This is the offline portion of the PHP `sefazValidate()` method. It
/// extracts the access key, protocol number, and digest value from the
/// local authorized XML.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is empty or missing
/// required elements (`infNFe`, `nProt`, `DigestValue`).
pub fn extract_nfe_validation_data(nfe_xml: &str) -> Result<(String, String, String), FiscalError> {
    if nfe_xml.trim().is_empty() {
        return Err(FiscalError::XmlParsing(
            "Validação NF-e: a string da NF-e está vazia".to_string(),
        ));
    }

    // Extract access key from infNFe Id attribute
    let access_key = extract_access_key(nfe_xml).ok_or_else(|| {
        FiscalError::XmlParsing(
            "Chave de acesso não encontrada no atributo Id de <infNFe>".to_string(),
        )
    })?;

    // Extract protocol number
    let protocol = extract_xml_tag_value(nfe_xml, "nProt")
        .ok_or_else(|| FiscalError::XmlParsing("Tag <nProt> não encontrada no XML".to_string()))?;

    // Extract digest value from digital signature
    let digest = extract_xml_tag_value(nfe_xml, "DigestValue").ok_or_else(|| {
        FiscalError::XmlParsing("Tag <DigestValue> não encontrada no XML".to_string())
    })?;

    Ok((access_key, protocol, digest))
}

/// Validate an authorized NF-e by comparing local data against a SEFAZ
/// consultation response.
///
/// This is the offline comparison portion of the PHP `sefazValidate()`
/// method. Call [`extract_nfe_validation_data`] to get the local values,
/// then use [`SefazClient::consult`](crate::client::SefazClient::consult)
/// to query SEFAZ, and finally pass both to this function.
///
/// The validation checks three conditions (all must match for `is_valid`
/// to be `true`):
///
/// 1. Protocol number (`nProt`) matches
/// 2. Digest value (`digVal`/`DigestValue`) matches
/// 3. Access key (`chNFe`) matches
///
/// # Arguments
///
/// * `local_access_key` — 44-digit access key from the local XML.
/// * `local_protocol` — Protocol number from the local XML.
/// * `local_digest` — DigestValue from the local XML signature.
/// * `sefaz_response_xml` — Raw XML response from SEFAZ consultation.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the SEFAZ response is missing
/// the `<protNFe>` / `<infProt>` structure, or if required fields are
/// absent.
pub fn validate_authorized_nfe(
    local_access_key: &str,
    local_protocol: &str,
    local_digest: &str,
    sefaz_response_xml: &str,
) -> Result<ValidationResult, FiscalError> {
    // Check for protNFe in the response
    if !sefaz_response_xml.contains("<protNFe") && !sefaz_response_xml.contains("protNFe>") {
        // Try to extract xMotivo for a descriptive error
        if let Some(motivo) = extract_xml_tag_value(sefaz_response_xml, "xMotivo") {
            return Err(FiscalError::XmlParsing(format!("Validação NF-e: {motivo}")));
        }
        return Err(FiscalError::XmlParsing(
            "O documento de resposta não contém o node \"protNFe\"".to_string(),
        ));
    }

    // Extract SEFAZ values from infProt
    let sefaz_digest =
        extract_xml_tag_value(sefaz_response_xml, "digVal").unwrap_or_else(|| "000".to_string());
    let sefaz_key = extract_xml_tag_value(sefaz_response_xml, "chNFe").ok_or_else(|| {
        FiscalError::XmlParsing("Tag <chNFe> não encontrada na resposta da SEFAZ".to_string())
    })?;
    let sefaz_protocol = extract_xml_tag_value(sefaz_response_xml, "nProt").ok_or_else(|| {
        FiscalError::XmlParsing("Tag <nProt> não encontrada na resposta da SEFAZ".to_string())
    })?;

    let is_valid = local_protocol == sefaz_protocol
        && local_digest == sefaz_digest
        && local_access_key == sefaz_key;

    Ok(ValidationResult {
        is_valid,
        access_key: local_access_key.to_string(),
        local_protocol: local_protocol.to_string(),
        sefaz_protocol,
        local_digest: local_digest.to_string(),
        sefaz_digest,
    })
}

/// Extract the 44-digit access key from an NF-e XML `infNFe` Id attribute.
///
/// Looks for `Id="NFe<44 digits>"` and returns just the 44-digit portion.
fn extract_access_key(xml: &str) -> Option<String> {
    let id_start = xml.find("Id=\"NFe")?;
    let after_id = &xml[id_start + 7..]; // skip `Id="NFe`
    let quote_end = after_id.find('"')?;
    let key = &after_id[..quote_end];
    if key.len() == 44 && key.chars().all(|c| c.is_ascii_digit()) {
        Some(key.to_string())
    } else {
        None
    }
}

/// Validate the structure of a SEFAZ request XML envelope.
///
/// This is a simpler validation for request envelopes (like `<enviNFe>`,
/// `<consSitNFe>`, etc.) that checks well-formedness and the presence of
/// the NF-e namespace. Mirrors the PHP `Tools::isValid()` method that
/// validates request XML against method-specific XSD schemas.
///
/// # Arguments
///
/// * `xml` — The request XML string.
/// * `version` — Expected schema version (e.g. `"4.00"`).
/// * `method` — The request method name (e.g. `"enviNFe"`, `"consSitNFe"`).
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is not well-formed or
/// is missing the expected root element.
pub fn validate_request_xml(xml: &str, version: &str, method: &str) -> Result<(), FiscalError> {
    if xml.trim().is_empty() {
        return Err(FiscalError::XmlParsing(format!(
            "Validação {method}: XML vazio"
        )));
    }

    if !is_valid_xml(xml) {
        return Err(FiscalError::XmlParsing(format!(
            "Validação {method}: a string não é um XML válido"
        )));
    }

    let mut errors: Vec<String> = Vec::new();

    // Check root element matches the method
    if !xml.contains(&format!("<{method}")) {
        errors.push(format!("Elemento raiz <{method}> ausente"));
    }

    // Check namespace
    if !xml.contains("http://www.portalfiscal.inf.br/nfe") {
        errors.push("Namespace NF-e ausente (http://www.portalfiscal.inf.br/nfe)".to_string());
    }

    // Check version attribute
    if let Some(pos) = xml.find(&format!("<{method}")) {
        let after = &xml[pos..];
        if let Some(ver_pos) = after.find("versao=\"") {
            let ver_start = ver_pos + 8;
            if let Some(ver_end) = after[ver_start..].find('"') {
                let found = &after[ver_start..ver_start + ver_end];
                if found != version {
                    errors.push(format!(
                        "Versão do XML ({found}) não corresponde à versão esperada ({version})"
                    ));
                }
            }
        }
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

    // ── is_valid_xml ──────────────────────────────────────────────

    #[test]
    fn is_valid_xml_simple() {
        assert!(is_valid_xml("<root><child>text</child></root>"));
    }

    #[test]
    fn is_valid_xml_with_declaration() {
        assert!(is_valid_xml(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><root/>"
        ));
    }

    #[test]
    fn is_valid_xml_empty() {
        assert!(!is_valid_xml(""));
        assert!(!is_valid_xml("   "));
    }

    #[test]
    fn is_valid_xml_html_rejected() {
        assert!(!is_valid_xml("<!DOCTYPE html><html><body></body></html>"));
        assert!(!is_valid_xml("<div>text</div></html>"));
    }

    #[test]
    fn is_valid_xml_malformed() {
        assert!(!is_valid_xml("not xml at all"));
        assert!(!is_valid_xml("<root><unclosed>"));
    }

    // ── validate_nfe_xml ──────────────────────────────────────────

    #[test]
    fn validate_nfe_xml_empty_string() {
        let err = validate_nfe_xml("", "4.00").unwrap_err();
        assert!(err.to_string().contains("vazia"));
    }

    #[test]
    fn validate_nfe_xml_not_xml() {
        let err = validate_nfe_xml("not xml", "4.00").unwrap_err();
        assert!(err.to_string().contains("não é um XML"));
    }

    #[test]
    fn validate_nfe_xml_missing_root() {
        let err = validate_nfe_xml("<root/>", "4.00").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("NFe"));
    }

    #[test]
    fn validate_nfe_xml_wrong_version() {
        let xml = concat!(
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<infNFe versao="3.10" Id="NFe41260304123456000190550010000001231123456780">"#,
            "<ide><cUF>41</cUF><cNF>12345678</cNF><natOp>VENDA</natOp>",
            "<mod>55</mod><serie>1</serie><nNF>123</nNF>",
            "<dhEmi>2026-03-11T10:30:00-03:00</dhEmi>",
            "<tpNF>1</tpNF><idDest>1</idDest><cMunFG>4106902</cMunFG>",
            "<tpImp>1</tpImp><tpEmis>1</tpEmis><cDV>0</cDV>",
            "<tpAmb>2</tpAmb><finNFe>1</finNFe><indFinal>1</indFinal>",
            "<indPres>1</indPres><procEmi>0</procEmi><verProc>1.0</verProc></ide>",
            "<emit><CNPJ>04123456000190</CNPJ><xNome>Test</xNome>",
            "<enderEmit><xLgr>Rua</xLgr></enderEmit>",
            "<IE>9012345678</IE><CRT>3</CRT></emit>",
            r#"<det nItem="1"><prod><cProd>001</cProd></prod></det>"#,
            "<total><ICMSTot><vNF>150.00</vNF></ICMSTot></total>",
            "<transp><modFrete>9</modFrete></transp>",
            "<pag><detPag><tPag>01</tPag><vPag>150.00</vPag></detPag></pag>",
            "<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\">",
            "<SignedInfo/><SignatureValue/></Signature>",
            "</infNFe></NFe>",
        );
        let err = validate_nfe_xml(xml, "4.00").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("3.10"));
        assert!(msg.contains("4.00"));
    }

    #[test]
    fn validate_nfe_xml_valid_complete() {
        let xml = concat!(
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
            "<ide><cUF>41</cUF><cNF>12345678</cNF><natOp>VENDA</natOp>",
            "<mod>55</mod><serie>1</serie><nNF>123</nNF>",
            "<dhEmi>2026-03-11T10:30:00-03:00</dhEmi>",
            "<tpNF>1</tpNF><idDest>1</idDest><cMunFG>4106902</cMunFG>",
            "<tpImp>1</tpImp><tpEmis>1</tpEmis><cDV>0</cDV>",
            "<tpAmb>2</tpAmb><finNFe>1</finNFe><indFinal>1</indFinal>",
            "<indPres>1</indPres><procEmi>0</procEmi><verProc>1.0</verProc></ide>",
            "<emit><CNPJ>04123456000190</CNPJ><xNome>Test</xNome>",
            "<enderEmit><xLgr>Rua</xLgr></enderEmit>",
            "<IE>9012345678</IE><CRT>3</CRT></emit>",
            r#"<det nItem="1"><prod><cProd>001</cProd></prod></det>"#,
            "<total><ICMSTot><vNF>150.00</vNF></ICMSTot></total>",
            "<transp><modFrete>9</modFrete></transp>",
            "<pag><detPag><tPag>01</tPag><vPag>150.00</vPag></detPag></pag>",
            "<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\">",
            "<SignedInfo/><SignatureValue/></Signature>",
            "</infNFe></NFe>",
        );
        assert!(validate_nfe_xml(xml, "4.00").is_ok());
    }

    #[test]
    fn validate_nfe_xml_missing_signature() {
        let xml = concat!(
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
            "<ide><cUF>41</cUF><cNF>12345678</cNF><natOp>VENDA</natOp>",
            "<mod>55</mod><serie>1</serie><nNF>123</nNF>",
            "<dhEmi>2026-03-11T10:30:00-03:00</dhEmi>",
            "<tpNF>1</tpNF><idDest>1</idDest><cMunFG>4106902</cMunFG>",
            "<tpImp>1</tpImp><tpEmis>1</tpEmis><cDV>0</cDV>",
            "<tpAmb>2</tpAmb><finNFe>1</finNFe><indFinal>1</indFinal>",
            "<indPres>1</indPres><procEmi>0</procEmi><verProc>1.0</verProc></ide>",
            "<emit><CNPJ>04123456000190</CNPJ><xNome>Test</xNome>",
            "<enderEmit><xLgr>Rua</xLgr></enderEmit>",
            "<IE>9012345678</IE><CRT>3</CRT></emit>",
            r#"<det nItem="1"><prod><cProd>001</cProd></prod></det>"#,
            "<total><ICMSTot><vNF>150.00</vNF></ICMSTot></total>",
            "<transp><modFrete>9</modFrete></transp>",
            "<pag><detPag><tPag>01</tPag><vPag>150.00</vPag></detPag></pag>",
            "</infNFe></NFe>",
        );
        let err = validate_nfe_xml(xml, "4.00").unwrap_err();
        assert!(err.to_string().contains("Signature"));
    }

    #[test]
    fn validate_nfe_xml_invalid_access_key() {
        let xml = concat!(
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<infNFe versao="4.00" Id="NFe123">"#,
            "<ide><cUF>41</cUF><cNF>12345678</cNF><natOp>VENDA</natOp>",
            "<mod>55</mod><serie>1</serie><nNF>123</nNF>",
            "<dhEmi>2026-03-11T10:30:00-03:00</dhEmi>",
            "<tpNF>1</tpNF><idDest>1</idDest><cMunFG>4106902</cMunFG>",
            "<tpImp>1</tpImp><tpEmis>1</tpEmis><cDV>0</cDV>",
            "<tpAmb>2</tpAmb><finNFe>1</finNFe><indFinal>1</indFinal>",
            "<indPres>1</indPres><procEmi>0</procEmi><verProc>1.0</verProc></ide>",
            "<emit><CNPJ>04123456000190</CNPJ><xNome>Test</xNome>",
            "<enderEmit><xLgr>Rua</xLgr></enderEmit>",
            "<IE>9012345678</IE><CRT>3</CRT></emit>",
            r#"<det nItem="1"><prod><cProd>001</cProd></prod></det>"#,
            "<total><ICMSTot><vNF>150.00</vNF></ICMSTot></total>",
            "<transp><modFrete>9</modFrete></transp>",
            "<pag><detPag><tPag>01</tPag><vPag>150.00</vPag></detPag></pag>",
            "<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\">",
            "<SignedInfo/><SignatureValue/></Signature>",
            "</infNFe></NFe>",
        );
        let err = validate_nfe_xml(xml, "4.00").unwrap_err();
        assert!(err.to_string().contains("Chave de acesso"));
    }

    #[test]
    fn validate_nfe_xml_missing_namespace() {
        let xml = concat!(
            "<NFe>",
            r#"<infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
            "<ide><cUF>41</cUF><cNF>12345678</cNF><natOp>VENDA</natOp>",
            "<mod>55</mod><serie>1</serie><nNF>123</nNF>",
            "<dhEmi>2026-03-11T10:30:00-03:00</dhEmi>",
            "<tpNF>1</tpNF><idDest>1</idDest><cMunFG>4106902</cMunFG>",
            "<tpImp>1</tpImp><tpEmis>1</tpEmis><cDV>0</cDV>",
            "<tpAmb>2</tpAmb><finNFe>1</finNFe><indFinal>1</indFinal>",
            "<indPres>1</indPres><procEmi>0</procEmi><verProc>1.0</verProc></ide>",
            "<emit><CNPJ>04123456000190</CNPJ><xNome>Test</xNome>",
            "<enderEmit><xLgr>Rua</xLgr></enderEmit>",
            "<IE>9012345678</IE><CRT>3</CRT></emit>",
            r#"<det nItem="1"><prod><cProd>001</cProd></prod></det>"#,
            "<total><ICMSTot><vNF>150.00</vNF></ICMSTot></total>",
            "<transp><modFrete>9</modFrete></transp>",
            "<pag><detPag><tPag>01</tPag><vPag>150.00</vPag></detPag></pag>",
            "<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\">",
            "<SignedInfo/><SignatureValue/></Signature>",
            "</infNFe></NFe>",
        );
        let err = validate_nfe_xml(xml, "4.00").unwrap_err();
        assert!(err.to_string().contains("Namespace"));
    }

    #[test]
    fn validate_nfe_xml_multiple_errors_collected() {
        // Minimal XML that is valid XML but missing most NF-e structure
        let xml = "<root><something>value</something></root>";
        let err = validate_nfe_xml(xml, "4.00").unwrap_err();
        let msg = err.to_string();
        // Should have multiple errors separated by "; "
        assert!(msg.contains("; "));
        assert!(msg.contains("NFe"));
        assert!(msg.contains("infNFe"));
    }

    #[test]
    fn validate_nfe_xml_cpf_accepted() {
        // Use CPF instead of CNPJ — should pass CNPJ/CPF check
        let xml = concat!(
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
            "<ide><cUF>41</cUF><cNF>12345678</cNF><natOp>VENDA</natOp>",
            "<mod>55</mod><serie>1</serie><nNF>123</nNF>",
            "<dhEmi>2026-03-11T10:30:00-03:00</dhEmi>",
            "<tpNF>1</tpNF><idDest>1</idDest><cMunFG>4106902</cMunFG>",
            "<tpImp>1</tpImp><tpEmis>1</tpEmis><cDV>0</cDV>",
            "<tpAmb>2</tpAmb><finNFe>1</finNFe><indFinal>1</indFinal>",
            "<indPres>1</indPres><procEmi>0</procEmi><verProc>1.0</verProc></ide>",
            "<emit><CPF>12345678901</CPF><xNome>Test PF</xNome>",
            "<enderEmit><xLgr>Rua</xLgr></enderEmit>",
            "<IE>9012345678</IE><CRT>1</CRT></emit>",
            r#"<det nItem="1"><prod><cProd>001</cProd></prod></det>"#,
            "<total><ICMSTot><vNF>150.00</vNF></ICMSTot></total>",
            "<transp><modFrete>9</modFrete></transp>",
            "<pag><detPag><tPag>01</tPag><vPag>150.00</vPag></detPag></pag>",
            "<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\">",
            "<SignedInfo/><SignatureValue/></Signature>",
            "</infNFe></NFe>",
        );
        assert!(validate_nfe_xml(xml, "4.00").is_ok());
    }

    // ── extract_nfe_validation_data ───────────────────────────────

    #[test]
    fn extract_validation_data_success() {
        let xml = concat!(
            r#"<nfeProc><NFe><infNFe Id="NFe41260304123456000190550010000001231123456780">"#,
            "</infNFe></NFe>",
            "<protNFe><infProt>",
            "<nProt>141260000123456</nProt>",
            "<DigestValue>abc123==</DigestValue>",
            "</infProt></protNFe></nfeProc>",
        );
        let (key, prot, digest) = extract_nfe_validation_data(xml).expect("should parse");
        assert_eq!(key, "41260304123456000190550010000001231123456780");
        assert_eq!(prot, "141260000123456");
        assert_eq!(digest, "abc123==");
    }

    #[test]
    fn extract_validation_data_empty() {
        let err = extract_nfe_validation_data("").unwrap_err();
        assert!(err.to_string().contains("vazia"));
    }

    #[test]
    fn extract_validation_data_missing_key() {
        let xml = "<nfeProc><nProt>123</nProt><DigestValue>abc</DigestValue></nfeProc>";
        let err = extract_nfe_validation_data(xml).unwrap_err();
        assert!(err.to_string().contains("Chave de acesso"));
    }

    // ── validate_authorized_nfe ───────────────────────────────────

    #[test]
    fn validate_authorized_nfe_match() {
        let sefaz_response = concat!(
            "<retConsSitNFe>",
            "<protNFe><infProt>",
            "<chNFe>41260304123456000190550010000001231123456780</chNFe>",
            "<nProt>141260000123456</nProt>",
            "<digVal>abc123==</digVal>",
            "</infProt></protNFe>",
            "</retConsSitNFe>",
        );
        let result = validate_authorized_nfe(
            "41260304123456000190550010000001231123456780",
            "141260000123456",
            "abc123==",
            sefaz_response,
        )
        .expect("should succeed");
        assert!(result.is_valid);
        assert_eq!(
            result.access_key,
            "41260304123456000190550010000001231123456780"
        );
    }

    #[test]
    fn validate_authorized_nfe_mismatch_protocol() {
        let sefaz_response = concat!(
            "<retConsSitNFe>",
            "<protNFe><infProt>",
            "<chNFe>41260304123456000190550010000001231123456780</chNFe>",
            "<nProt>999999999999999</nProt>",
            "<digVal>abc123==</digVal>",
            "</infProt></protNFe>",
            "</retConsSitNFe>",
        );
        let result = validate_authorized_nfe(
            "41260304123456000190550010000001231123456780",
            "141260000123456",
            "abc123==",
            sefaz_response,
        )
        .expect("should succeed");
        assert!(!result.is_valid);
        assert_ne!(result.local_protocol, result.sefaz_protocol);
    }

    #[test]
    fn validate_authorized_nfe_mismatch_digest() {
        let sefaz_response = concat!(
            "<retConsSitNFe>",
            "<protNFe><infProt>",
            "<chNFe>41260304123456000190550010000001231123456780</chNFe>",
            "<nProt>141260000123456</nProt>",
            "<digVal>DIFFERENT==</digVal>",
            "</infProt></protNFe>",
            "</retConsSitNFe>",
        );
        let result = validate_authorized_nfe(
            "41260304123456000190550010000001231123456780",
            "141260000123456",
            "abc123==",
            sefaz_response,
        )
        .expect("should succeed");
        assert!(!result.is_valid);
    }

    #[test]
    fn validate_authorized_nfe_no_prot() {
        let sefaz_response = "<retConsSitNFe><cStat>217</cStat><xMotivo>NF-e nao consta na base</xMotivo></retConsSitNFe>";
        let err = validate_authorized_nfe(
            "41260304123456000190550010000001231123456780",
            "141260000123456",
            "abc123==",
            sefaz_response,
        )
        .unwrap_err();
        assert!(err.to_string().contains("NF-e nao consta na base"));
    }

    #[test]
    fn validate_authorized_nfe_no_prot_no_motivo() {
        let sefaz_response = "<retConsSitNFe><cStat>999</cStat></retConsSitNFe>";
        let err = validate_authorized_nfe(
            "41260304123456000190550010000001231123456780",
            "141260000123456",
            "abc123==",
            sefaz_response,
        )
        .unwrap_err();
        assert!(err.to_string().contains("protNFe"));
    }

    #[test]
    fn validate_authorized_nfe_missing_digest_uses_default() {
        // When SEFAZ response has no digVal, PHP uses "000"
        let sefaz_response = concat!(
            "<retConsSitNFe>",
            "<protNFe><infProt>",
            "<chNFe>41260304123456000190550010000001231123456780</chNFe>",
            "<nProt>141260000123456</nProt>",
            "</infProt></protNFe>",
            "</retConsSitNFe>",
        );
        let result = validate_authorized_nfe(
            "41260304123456000190550010000001231123456780",
            "141260000123456",
            "000",
            sefaz_response,
        )
        .expect("should succeed");
        assert!(result.is_valid);
        assert_eq!(result.sefaz_digest, "000");
    }

    // ── validate_request_xml ──────────────────────────────────────

    #[test]
    fn validate_request_xml_valid() {
        let xml = r#"<enviNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><idLote>1</idLote></enviNFe>"#;
        assert!(validate_request_xml(xml, "4.00", "enviNFe").is_ok());
    }

    #[test]
    fn validate_request_xml_empty() {
        let err = validate_request_xml("", "4.00", "enviNFe").unwrap_err();
        assert!(err.to_string().contains("vazio"));
    }

    #[test]
    fn validate_request_xml_wrong_root() {
        let xml =
            r#"<wrong xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><data/></wrong>"#;
        let err = validate_request_xml(xml, "4.00", "enviNFe").unwrap_err();
        assert!(err.to_string().contains("enviNFe"));
    }

    #[test]
    fn validate_request_xml_wrong_version() {
        let xml = r#"<enviNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="3.10"><idLote>1</idLote></enviNFe>"#;
        let err = validate_request_xml(xml, "4.00", "enviNFe").unwrap_err();
        assert!(err.to_string().contains("3.10"));
    }

    // ── extract_access_key ────────────────────────────────────────

    #[test]
    fn extract_access_key_valid() {
        let xml =
            r#"<infNFe Id="NFe41260304123456000190550010000001231123456780">content</infNFe>"#;
        let key = extract_access_key(xml);
        assert_eq!(
            key.as_deref(),
            Some("41260304123456000190550010000001231123456780")
        );
    }

    #[test]
    fn extract_access_key_invalid_length() {
        let xml = r#"<infNFe Id="NFe123">content</infNFe>"#;
        assert_eq!(extract_access_key(xml), None);
    }

    #[test]
    fn extract_access_key_not_present() {
        let xml = "<infNFe>content</infNFe>";
        assert_eq!(extract_access_key(xml), None);
    }

    // ── is_valid_xml edge cases ───────────────────────────────────

    #[test]
    fn is_valid_xml_more_closes_than_opens() {
        assert!(!is_valid_xml("</root>"));
    }

    #[test]
    fn is_valid_xml_bom_prefix() {
        // BOM + valid XML
        let xml = "\u{feff}<root><child/></root>";
        assert!(is_valid_xml(xml));
    }

    #[test]
    fn is_valid_xml_not_starting_with_lt() {
        assert!(!is_valid_xml("text <root/>"));
    }

    #[test]
    fn is_valid_xml_empty_element() {
        assert!(is_valid_xml("<root/>"));
    }

    // ── validate_nfe_xml edge cases ─────────────────────────────────

    #[test]
    fn validate_nfe_xml_missing_versao_attribute() {
        let xml = concat!(
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<infNFe Id="NFe41260304123456000190550010000001231123456780">"#,
            "<ide><cUF>41</cUF><cNF>12345678</cNF><natOp>VENDA</natOp>",
            "<mod>55</mod><serie>1</serie><nNF>123</nNF>",
            "<dhEmi>2026-03-11T10:30:00-03:00</dhEmi>",
            "<tpNF>1</tpNF><idDest>1</idDest><cMunFG>4106902</cMunFG>",
            "<tpImp>1</tpImp><tpEmis>1</tpEmis><cDV>0</cDV>",
            "<tpAmb>2</tpAmb><finNFe>1</finNFe><indFinal>1</indFinal>",
            "<indPres>1</indPres><procEmi>0</procEmi><verProc>1.0</verProc></ide>",
            "<emit><CNPJ>04123456000190</CNPJ><xNome>Test</xNome>",
            "<enderEmit><xLgr>Rua</xLgr></enderEmit>",
            "<IE>9012345678</IE><CRT>3</CRT></emit>",
            r#"<det nItem="1"><prod><cProd>001</cProd></prod></det>"#,
            "<total><ICMSTot><vNF>150.00</vNF></ICMSTot></total>",
            "<transp><modFrete>9</modFrete></transp>",
            "<pag><detPag><tPag>01</tPag><vPag>150.00</vPag></detPag></pag>",
            "<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\">",
            "<SignedInfo/><SignatureValue/></Signature>",
            "</infNFe></NFe>",
        );
        let err = validate_nfe_xml(xml, "4.00").unwrap_err();
        assert!(err.to_string().contains("versao"));
    }

    #[test]
    fn validate_nfe_xml_missing_id_attribute() {
        let xml = concat!(
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<infNFe versao="4.00">"#,
            "<ide><cUF>41</cUF><cNF>12345678</cNF><natOp>VENDA</natOp>",
            "<mod>55</mod><serie>1</serie><nNF>123</nNF>",
            "<dhEmi>2026-03-11T10:30:00-03:00</dhEmi>",
            "<tpNF>1</tpNF><idDest>1</idDest><cMunFG>4106902</cMunFG>",
            "<tpImp>1</tpImp><tpEmis>1</tpEmis><cDV>0</cDV>",
            "<tpAmb>2</tpAmb><finNFe>1</finNFe><indFinal>1</indFinal>",
            "<indPres>1</indPres><procEmi>0</procEmi><verProc>1.0</verProc></ide>",
            "<emit><CNPJ>04123456000190</CNPJ><xNome>Test</xNome>",
            "<enderEmit><xLgr>Rua</xLgr></enderEmit>",
            "<IE>9012345678</IE><CRT>3</CRT></emit>",
            r#"<det nItem="1"><prod><cProd>001</cProd></prod></det>"#,
            "<total><ICMSTot><vNF>150.00</vNF></ICMSTot></total>",
            "<transp><modFrete>9</modFrete></transp>",
            "<pag><detPag><tPag>01</tPag><vPag>150.00</vPag></detPag></pag>",
            "<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\">",
            "<SignedInfo/><SignatureValue/></Signature>",
            "</infNFe></NFe>",
        );
        let err = validate_nfe_xml(xml, "4.00").unwrap_err();
        assert!(err.to_string().contains("chave de acesso"));
    }

    // ── validate_request_xml edge cases ──────────────────────────────

    #[test]
    fn validate_request_xml_not_valid_xml() {
        let err = validate_request_xml("not xml", "4.00", "enviNFe").unwrap_err();
        assert!(err.to_string().contains("não é um XML"));
    }

    #[test]
    fn validate_request_xml_missing_namespace() {
        let xml = r#"<enviNFe versao="4.00"><idLote>1</idLote></enviNFe>"#;
        let err = validate_request_xml(xml, "4.00", "enviNFe").unwrap_err();
        assert!(err.to_string().contains("Namespace"));
    }

    // ── validate_authorized_nfe edge cases ──────────────────────────

    #[test]
    fn validate_authorized_nfe_missing_ch_nfe() {
        let sefaz_response = concat!(
            "<retConsSitNFe>",
            "<protNFe><infProt>",
            "<nProt>141260000123456</nProt>",
            "<digVal>abc123==</digVal>",
            "</infProt></protNFe>",
            "</retConsSitNFe>",
        );
        let err = validate_authorized_nfe(
            "41260304123456000190550010000001231123456780",
            "141260000123456",
            "abc123==",
            sefaz_response,
        )
        .unwrap_err();
        assert!(err.to_string().contains("chNFe"));
    }

    #[test]
    fn validate_authorized_nfe_missing_nprot() {
        let sefaz_response = concat!(
            "<retConsSitNFe>",
            "<protNFe><infProt>",
            "<chNFe>41260304123456000190550010000001231123456780</chNFe>",
            "<digVal>abc123==</digVal>",
            "</infProt></protNFe>",
            "</retConsSitNFe>",
        );
        let err = validate_authorized_nfe(
            "41260304123456000190550010000001231123456780",
            "141260000123456",
            "abc123==",
            sefaz_response,
        )
        .unwrap_err();
        assert!(err.to_string().contains("nProt"));
    }

    // ── extract_nfe_validation_data edge cases ──────────────────────

    #[test]
    fn extract_validation_data_missing_nprot() {
        let xml = concat!(
            r#"<nfeProc><NFe><infNFe Id="NFe41260304123456000190550010000001231123456780">"#,
            "</infNFe></NFe>",
            "<protNFe><infProt>",
            "<DigestValue>abc123==</DigestValue>",
            "</infProt></protNFe></nfeProc>",
        );
        let err = extract_nfe_validation_data(xml).unwrap_err();
        assert!(err.to_string().contains("nProt"));
    }

    #[test]
    fn extract_validation_data_missing_digest() {
        let xml = concat!(
            r#"<nfeProc><NFe><infNFe Id="NFe41260304123456000190550010000001231123456780">"#,
            "</infNFe></NFe>",
            "<protNFe><infProt>",
            "<nProt>141260000123456</nProt>",
            "</infProt></protNFe></nfeProc>",
        );
        let err = extract_nfe_validation_data(xml).unwrap_err();
        assert!(err.to_string().contains("DigestValue"));
    }
}
