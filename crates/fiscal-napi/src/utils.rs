use napi_derive::napi;

// ── Standardize ─────────────────────────────────────────────────

/// Identify the type of an NF-e XML document from its content.
///
/// Parses the XML and checks the root element against the list of known
/// NFe document types. Returns the matched root tag name (e.g. `"NFe"`,
/// `"nfeProc"`, `"retConsSitNFe"`).
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the input is empty, whitespace-only,
/// not valid XML, or not a recognised NFe document type.
#[napi]
pub fn identify_xml_type(xml: String) -> napi::Result<String> {
    fiscal_core::standardize::identify_xml_type(&xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Convert an NFe XML string to a JSON string representation.
///
/// First validates that the XML is a recognised NFe document via
/// [`identify_xml_type`], then converts the XML tree to a JSON object.
///
/// ## Root-tag unwrapping (PHP parity)
///
/// Like PHP's `Standardize::toStd()`, the identified root element is
/// **unwrapped**: for `<NFe xmlns="..."><infNFe>...</infNFe></NFe>` the
/// result is `{"infNFe":{...}}`, not `{"NFe":{"infNFe":{...}}}`.
/// Attributes on the root element (except `xmlns`) are merged into the
/// returned object as plain keys.
///
/// ## Attribute handling (difference from PHP)
///
/// The PHP `Standardize::toStd()` method uses `simplexml_load_string` +
/// `json_encode`, which places XML attributes under an `@attributes` key
/// (later renamed to `attributes`). This Rust implementation places
/// attributes **inline** alongside child elements — e.g. an element
/// `<infNFe versao="4.00" Id="NFe123">` produces `{"versao":"4.00",
/// "Id":"NFe123", ...}` rather than `{"attributes":{"versao":"4.00",
/// "Id":"NFe123"}, ...}`. This is a deliberate design decision: inline
/// attributes are more ergonomic for JSON consumers and avoid the extra
/// nesting level. Consumers that relied on the PHP `attributes` key
/// should adapt their field lookups accordingly.
///
/// ## `infNFeSupl` / CDATA handling
///
/// The PHP `toStd()` has special post-processing for `infNFeSupl`: it
/// re-extracts `qrCode` and `urlChave` via DOM because PHP's
/// `simplexml_load_string` can corrupt CDATA sections. This Rust
/// implementation uses `quick-xml`'s `Event::CData` handler, which
/// correctly preserves CDATA content without mangling, so no special
/// post-processing is needed.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the input is not valid NFe XML,
/// or if conversion to JSON fails.
#[napi]
pub fn xml_to_json(xml: String) -> napi::Result<String> {
    fiscal_core::standardize::xml_to_json(&xml).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Convert an NFe XML string to a navigable [`serde_json::Value`] tree.
///
/// This is the Rust equivalent of PHP's `Standardize::toStd()`, which returns
/// a `stdClass` object. In Rust, [`serde_json::Value`] serves the same role:
/// it is a dynamically-typed, navigable tree that can be indexed with
/// `value["fieldName"]`.
///
/// The identified root element is **unwrapped** so the returned value
/// contains the children of the root tag directly, matching PHP behaviour.
/// For example, `<NFe xmlns="..."><infNFe>...</infNFe></NFe>` yields
/// `{"infNFe": {...}}`. Attributes on the root element (except `xmlns`)
/// are merged as plain keys.
///
/// # Example
///
/// ```rust,ignore
/// let value = xml_to_value(xml)?;
/// let cuf = &value["infNFe"]["ide"]["cUF"];
/// assert_eq!(cuf.as_str(), Some("35"));
/// ```
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the input is not valid NFe XML.
#[napi(ts_return_type = "Record<string, unknown>")]
pub fn xml_to_value(xml: String) -> napi::Result<serde_json::Value> {
    fiscal_core::standardize::xml_to_value(&xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Convert an NFe XML string to a [`serde_json::Map`] (equivalent to an
/// associative array / hash map).
///
/// This is the Rust equivalent of PHP's `Standardize::toArray()`, which returns
/// an associative array. In Rust, [`serde_json::Map<String, Value>`] is the
/// natural equivalent: an ordered map of string keys to dynamically-typed values.
///
/// Like [`xml_to_value`], the identified root element is unwrapped: the map
/// contains the *children* of the root tag, not the root tag itself.
///
/// # Example
///
/// ```rust,ignore
/// let map = xml_to_map(xml)?;
/// let inf_nfe = map.get("infNFe").unwrap();
/// ```
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the input is not valid NFe XML,
/// or if the top-level JSON value is not an object (should not happen for
/// well-formed NFe documents).
#[napi]
pub fn xml_to_map(xml: String) -> napi::Result<serde_json::Value> {
    let result = fiscal_core::standardize::xml_to_map(&xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}

// ── Convert ─────────────────────────────────────────────────────

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
#[napi]
pub fn txt_to_xml(txt: String, layout: String) -> napi::Result<String> {
    fiscal_core::convert::txt_to_xml(&txt, &layout)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
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
#[napi]
pub fn txt_to_xml_all(txt: String, layout: String) -> napi::Result<Vec<String>> {
    fiscal_core::convert::txt_to_xml_all(&txt, &layout)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
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
#[napi]
pub fn validate_txt(txt: String, layout: String) -> napi::Result<bool> {
    fiscal_core::convert::validate_txt(&txt, &layout)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

// ── QR Code ─────────────────────────────────────────────────────

/// Build the NFC-e urlChave tag content for consulting the NFe by access key.
///
/// Format: `url?p=key|env` or `url&p=key|env` if URL already contains `?`.
#[napi]
pub fn build_nfce_consult_url(
    url_chave: String,
    access_key: String,
    environment: String,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(fiscal_core::qrcode::build_nfce_consult_url(
        &url_chave,
        &access_key,
        sefaz_environment,
    ))
}

/// Insert QR Code and urlChave tags into a signed NFC-e XML.
///
/// Extracts fields from the XML (access key, environment, emission date/type,
/// totals, digest value, destination document), builds the QR Code URL, and
/// inserts an `<infNFeSupl>` element with `<qrCode>` and `<urlChave>` children
/// before the `<Signature` element.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if required XML tags are missing.
/// Returns [`FiscalError::MissingRequiredField`] if CSC token/ID is missing for v200.
/// Returns [`FiscalError::XmlGeneration`] if `<Signature` is not found in the XML.
#[napi]
pub fn put_qr_tag(params: serde_json::Value) -> napi::Result<String> {
    let __params = serde_json::from_value(params)
        .map_err(|e| napi::Error::from_reason(format!("Invalid params: {e}")))?;
    fiscal_core::qrcode::put_qr_tag(&__params).map_err(|e| napi::Error::from_reason(e.to_string()))
}

// ── Sanitize ────────────────────────────────────────────────────

/// Replace accented characters with their ASCII equivalents.
///
/// The replacement table matches the PHP `sped-common` `Strings::squashCharacters`
/// mapping exactly.  Characters that have no ASCII equivalent are left untouched
/// (they are still valid UTF-8 and will be XML-escaped normally).
///
/// # Examples
///
/// ```
/// use fiscal_core::sanitize::sanitize_to_ascii;
/// assert_eq!(sanitize_to_ascii("São Paulo"), "Sao Paulo");
/// assert_eq!(sanitize_to_ascii("ação"), "acao");
/// assert_eq!(sanitize_to_ascii("café"), "cafe");
/// ```
#[napi]
pub fn sanitize_to_ascii(input: String) -> napi::Result<String> {
    Ok(fiscal_core::sanitize::sanitize_to_ascii(&input))
}

/// Apply [`sanitize_to_ascii`] to the text content of an XML string, leaving
/// tag names, attribute names, and attribute values like namespaces/IDs intact.
///
/// The function walks through the XML and only transforms text that appears
/// between `>` and `<` (i.e., element text content).  Attribute values are
/// also sanitized (the text between quotes inside tags), except for well-known
/// structural attributes (`xmlns`, `Id`, `versao`).
///
/// This mirrors the PHP behaviour where `Strings::squashCharacters` is applied
/// to field values before they are placed into the DOM — the net effect is that
/// text nodes and most attribute values in the final XML have accents stripped.
#[napi]
pub fn sanitize_xml_text(xml: String) -> napi::Result<String> {
    Ok(fiscal_core::sanitize::sanitize_xml_text(&xml))
}

// ── GTIN ────────────────────────────────────────────────────────

/// Validate a GTIN-8/12/13/14 barcode number.
///
/// - Empty string and `"SEM GTIN"` are considered valid (exempt).
/// - Valid GTIN-8/12/13/14 with correct check digit returns `Ok(true)`.
/// - Non-numeric input or invalid check digit returns `Err`.
///
/// # Examples
///
/// ```
/// use fiscal_core::gtin::is_valid_gtin;
///
/// assert_eq!(is_valid_gtin(""), Ok(true));
/// assert_eq!(is_valid_gtin("SEM GTIN"), Ok(true));
/// assert!(is_valid_gtin("ABC").is_err());
/// ```
///
/// # Errors
///
/// Returns `Err` if:
/// - The input contains non-numeric characters
/// - The length is not 8, 12, 13, or 14
/// - The check digit is invalid
#[napi]
pub fn is_valid_gtin(gtin: String) -> napi::Result<bool> {
    fiscal_core::gtin::is_valid_gtin(&gtin).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Calculate the GTIN check digit using the standard algorithm.
///
/// Works for GTIN-8, GTIN-12, GTIN-13, and GTIN-14.
/// The input must include the check digit position (full barcode).
///
/// # Errors
///
/// Returns `Err` if the input contains non-digit characters.
#[napi]
pub fn calculate_check_digit(gtin: String) -> napi::Result<u8> {
    fiscal_core::gtin::calculate_check_digit(&gtin)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

// ── State Codes ─────────────────────────────────────────────────

/// Get the IBGE numeric code for a state abbreviation.
///
/// # Errors
///
/// Returns [`FiscalError::InvalidStateCode`] if `uf` is not a known
/// Brazilian state abbreviation.
#[napi]
pub fn get_state_code(uf: String) -> napi::Result<serde_json::Value> {
    let result = fiscal_core::state_codes::get_state_code(&uf)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Get the UF abbreviation for an IBGE numeric code.
///
/// # Errors
///
/// Returns [`FiscalError::InvalidStateCode`] if `code` is not a known
/// IBGE numeric state code.
#[napi]
pub fn get_state_by_code(code: String) -> napi::Result<serde_json::Value> {
    let result = fiscal_core::state_codes::get_state_by_code(&code)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}

// ── Timezone ────────────────────────────────────────────────────

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
#[napi]
pub fn timezone_for_uf(uf: String) -> napi::Result<Option<String>> {
    Ok(fiscal_core::timezone::timezone_for_uf(&uf).map(|s| s.to_string()))
}

// ── XML Utils ───────────────────────────────────────────────────

/// Escape special XML characters in text content and attribute values,
/// replacing `&`, `<`, `>`, `"`, and `'` with their XML entity equivalents.
///
/// # Examples
///
/// ```
/// use fiscal_core::xml_utils::escape_xml;
/// assert_eq!(escape_xml("Tom & Jerry <cats>"), "Tom &amp; Jerry &lt;cats&gt;");
/// ```
#[napi]
pub fn escape_xml(s: String) -> napi::Result<String> {
    Ok(fiscal_core::xml_utils::escape_xml(&s))
}

/// Extract the text content of the first occurrence of a simple XML tag in a
/// raw XML string.
///
/// Searches for `<tag_name>…</tag_name>` and returns the inner text.  Does not
/// handle namespaced tags, nested tags of the same name, or CDATA sections.
///
/// Returns `None` if the tag is absent.
///
/// # Examples
///
/// ```
/// use fiscal_core::xml_utils::extract_xml_tag_value;
/// let xml = "<root><cStat>100</cStat></root>";
/// assert_eq!(extract_xml_tag_value(xml, "cStat"), Some("100".to_string()));
/// assert_eq!(extract_xml_tag_value(xml, "missing"), None);
/// ```
#[napi]
pub fn extract_xml_tag_value(xml: String, tag_name: String) -> napi::Result<Option<String>> {
    Ok(fiscal_core::xml_utils::extract_xml_tag_value(
        &xml, &tag_name,
    ))
}

/// Pretty-print an XML string by adding indentation.
///
/// This is a lightweight formatter that does not parse XML semantically --
/// it works by splitting on `<` / `>` boundaries and inserting newlines and
/// indentation. Suitable for debugging/display purposes. Equivalent to the
/// PHP `FakePretty::prettyPrint` formatting behaviour (via DOMDocument::formatOutput).
///
/// # Examples
///
/// ```
/// use fiscal_core::xml_utils::pretty_print_xml;
/// let compact = "<root><child>text</child></root>";
/// let pretty = pretty_print_xml(compact);
/// assert!(pretty.contains("  <child>"));
/// ```
#[napi]
pub fn pretty_print_xml(xml: String) -> napi::Result<String> {
    Ok(fiscal_core::xml_utils::pretty_print_xml(&xml))
}

/// Replace characters that are valid in XML but rejected by SEFAZ.
///
/// This is a **SEFAZ-level** sanitisation function, distinct from [`escape_xml`].
/// While `escape_xml` performs standard XML entity encoding, this function
/// mirrors the PHP `Strings::replaceUnacceptableCharacters` from `sped-common`:
///
/// 1. Remove `<` and `>`.
/// 2. Replace `&` with ` & ` (space-padded).
/// 3. Remove single quotes (`'`) and double quotes (`"`).
/// 4. Collapse multiple consecutive whitespace characters into a single space.
/// 5. Encode the remaining `&` as `&amp;`.
/// 6. Remove carriage return (`\r`), tab (`\t`), and line feed (`\n`).
/// 7. Collapse multiple whitespace again (from normalize step).
/// 8. Remove ASCII control characters (`0x00`–`0x1F`, `0x7F`), except space.
/// 9. Trim leading and trailing whitespace.
///
/// The function is designed to be called on user-provided field values
/// (e.g. `xJust`, `xCorrecao`, `xPag`) before they are placed into the
/// NF-e XML, so that the SEFAZ web-service will not reject the document
/// because of forbidden characters.
///
/// # Examples
///
/// ```
/// use fiscal_core::xml_utils::replace_unacceptable_characters;
/// assert_eq!(
/// replace_unacceptable_characters("Tom & Jerry <cats>"),
/// "Tom &amp; Jerry cats"
/// );
/// assert_eq!(
/// replace_unacceptable_characters("  hello   world  "),
/// "hello world"
/// );
/// ```
#[napi]
pub fn replace_unacceptable_characters(input: String) -> napi::Result<String> {
    Ok(fiscal_core::xml_utils::replace_unacceptable_characters(
        &input,
    ))
}

/// Validate an NF-e XML string by checking for the presence of required tags.
///
/// This is a lightweight structural validator that checks for mandatory tags
/// in the NF-e/NFC-e XML. It does **not** perform full XSD schema validation
/// (which would require shipping XSD files and a full XML schema parser), but
/// covers the most common errors that would cause SEFAZ rejection.
///
/// Validated items:
/// - Required root structure (`<NFe>`, `<infNFe>`)
/// - Required `<ide>` fields (cUF, cNF, natOp, mod, serie, nNF, dhEmi, tpNF, etc.)
/// - Required `<emit>` fields (CNPJ/CPF, xNome, enderEmit, IE, CRT)
/// - Required `<det>` with at least one item
/// - Required `<total>` / `<ICMSTot>`
/// - Required `<transp>` and `<pag>`
/// - Access key format (44 digits)
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] with a description of all missing tags.
///
/// # Examples
///
/// ```
/// use fiscal_core::xml_utils::validate_xml;
/// let xml = "<NFe><infNFe>...</infNFe></NFe>";
/// // Will return an error listing all missing required tags
/// assert!(validate_xml(xml).is_err());
/// ```
#[napi]
pub fn validate_xml(xml: String) -> napi::Result<serde_json::Value> {
    let result = fiscal_core::xml_utils::validate_xml(&xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Remove characters that are invalid in XML 1.0 documents.
///
/// Per the XML 1.0 specification (Section 2.2), the only valid characters are:
///
/// - `#x9` (tab), `#xA` (line feed), `#xD` (carriage return)
/// - `#x20`–`#xD7FF`
/// - `#xE000`–`#xFFFD`
/// - `#x10000`–`#x10FFFF`
///
/// All other characters (control characters `\x00`–`\x08`, `\x0B`–`\x0C`,
/// `\x0E`–`\x1F`, surrogates `\xD800`–`\xDFFF`, `\xFFFE`–`\xFFFF`) are
/// stripped from the output.
///
/// This mirrors the character-level cleaning portion of the PHP
/// `Strings::normalize()` function in `sped-common`.
///
/// # Examples
///
/// ```
/// use fiscal_core::xml_utils::remove_invalid_xml_chars;
/// assert_eq!(remove_invalid_xml_chars("hello\x00world"), "helloworld");
/// assert_eq!(remove_invalid_xml_chars("tab\there"), "tab\there");
/// assert_eq!(remove_invalid_xml_chars("line\nfeed"), "line\nfeed");
/// ```
#[napi]
pub fn remove_invalid_xml_chars(input: String) -> napi::Result<String> {
    Ok(fiscal_core::xml_utils::remove_invalid_xml_chars(&input))
}

/// Clean an XML string by removing namespace artifacts, collapsing inter-tag
/// whitespace, and optionally stripping the `<?xml … ?>` declaration.
///
/// This is a direct port of the PHP `Strings::clearXmlString()` from
/// `sped-common`. It performs the following transformations:
///
/// 1. Removes the `xmlns:default="http://www.w3.org/2000/09/xmldsig#"` attribute.
/// 2. Removes the `standalone="no"` attribute.
/// 3. Removes `default:` namespace prefixes and `:default` suffixes.
/// 4. Strips `\n`, `\r`, and `\t` characters.
/// 5. Collapses whitespace between adjacent XML tags (`> <` becomes `><`).
/// 6. If `remove_encoding_tag` is `true`, removes the `<?xml … ?>` declaration.
///
/// # Examples
///
/// ```
/// use fiscal_core::xml_utils::clear_xml_string;
///
/// let xml = "<root>\n  <child>text</child>\n</root>";
/// assert_eq!(clear_xml_string(xml, false), "<root><child>text</child></root>");
///
/// let xml2 = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><root><a>1</a></root>";
/// assert_eq!(clear_xml_string(xml2, true), "<root><a>1</a></root>");
/// ```
#[napi]
pub fn clear_xml_string(input: String, remove_encoding_tag: bool) -> napi::Result<String> {
    Ok(fiscal_core::xml_utils::clear_xml_string(
        &input,
        remove_encoding_tag,
    ))
}

// ── Config ──────────────────────────────────────────────────────

/// Parse and validate a JSON configuration string.
///
/// This is the Rust equivalent of the PHP `Config::validate($content)` method.
/// It performs the following checks:
///
/// 1. The input must be valid JSON.
/// 2. The JSON root must be an object (not an array or scalar).
/// 3. All required fields must be present and non-null.
/// 4. `tpAmb` must be 1 or 2.
/// 5. `siglaUF` must be exactly 2 characters.
/// 6. `cnpj` must be 11 or 14 digits (CPF or CNPJ).
///
/// # Errors
///
/// Returns [`FiscalError::ConfigValidation`] if any validation rule fails.
#[napi]
pub fn validate_config(json: String) -> napi::Result<serde_json::Value> {
    let result = fiscal_core::config::validate_config(&json)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}

// ── Validate ────────────────────────────────────────────────────

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
#[napi]
pub fn is_valid_xml(content: String) -> napi::Result<bool> {
    Ok(fiscal_sefaz::validate::is_valid_xml(&content))
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
/// expected version.
/// 5. **Mandatory fields** — all required tags from `<ide>`, `<emit>`,
/// `<det>`, `<total>`, `<transp>`, and `<pag>` must exist.
/// 6. **Digital signature** — the `<Signature>` block must be present.
/// 7. **Access key** — the `Id` attribute on `<infNFe>` must contain a
/// valid 44-digit access key prefixed by `"NFe"`.
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
#[napi]
pub fn validate_nfe_xml(xml: String, version: String) -> napi::Result<serde_json::Value> {
    let result = fiscal_sefaz::validate::validate_nfe_xml(&xml, &version)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
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
#[napi]
pub fn extract_nfe_validation_data(nfe_xml: String) -> napi::Result<serde_json::Value> {
    let result = fiscal_sefaz::validate::extract_nfe_validation_data(&nfe_xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
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
#[napi]
pub fn validate_authorized_nfe(
    local_access_key: String,
    local_protocol: String,
    local_digest: String,
    sefaz_response_xml: String,
) -> napi::Result<serde_json::Value> {
    let result = fiscal_sefaz::validate::validate_authorized_nfe(
        &local_access_key,
        &local_protocol,
        &local_digest,
        &sefaz_response_xml,
    )
    .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
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
#[napi]
pub fn validate_request_xml(
    xml: String,
    version: String,
    method: String,
) -> napi::Result<serde_json::Value> {
    let result = fiscal_sefaz::validate::validate_request_xml(&xml, &version, &method)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}

// ── SEFAZ URLs ──────────────────────────────────────────────────

/// Get the SEFAZ service URL for a given state, environment, service name,
/// and invoice model (55 for NF-e, 65 for NFC-e).
///
/// The `service` parameter must be one of:
/// `"NfeStatusServico"`, `"NfeAutorizacao"`, `"NfeRetAutorizacao"`,
/// `"NfeConsultaProtocolo"`, `"NfeInutilizacao"`, `"RecepcaoEvento"`,
/// `"NfeConsultaCadastro"`, `"NfeDistribuicaoDFe"`, `"CscNFCe"`,
/// `"RecepcaoEPEC"` (SP NFC-e only), `"EPECStatusServico"` (SP NFC-e only),
/// `"NfeConsultaDest"` (AN only), `"NfeDownloadNF"` (AN only).
///
/// # Errors
///
/// Returns [`FiscalError::InvalidStateCode`] if `uf` is not a valid Brazilian
/// state abbreviation, or [`FiscalError::XmlGeneration`] if the service name
/// is unknown.
#[napi]
pub fn get_sefaz_url(uf: String, environment: String, service: String) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    fiscal_sefaz::urls::get_sefaz_url(&uf, sefaz_environment, &service)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Get the SEFAZ service URL for a specific invoice model.
///
/// Use model `55` for NF-e and `65` for NFC-e. NFC-e uses dedicated endpoints
/// for AM, GO, MG, MS, MT, PR, RS, SP; other states use SVRS NFC-e.
#[napi]
pub fn get_sefaz_url_for_model(
    uf: String,
    environment: String,
    service: String,
    model: u32,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    fiscal_sefaz::urls::get_sefaz_url_for_model(&uf, sefaz_environment, &service, model as u8)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Get the Ambiente Nacional (AN) service URL.
///
/// AN provides RecepcaoEvento, NfeDistribuicaoDFe, RecepcaoEPEC,
/// NfeConsultaDest, and NfeDownloadNF services.
#[napi]
pub fn get_an_url(environment: String, service: String) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    fiscal_sefaz::urls::get_an_url(sefaz_environment, &service)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Get the contingency authorizer for a given state (SVC-AN or SVC-RS).
///
/// Mapping follows the PHP sped-nfe Contingency.php:
/// - SVC-AN (SVCAN): AC, AL, AP, CE, DF, ES, MG, PA, PB, PI, RJ, RN, RO, RR, RS, SC, SE, SP, TO
/// - SVC-RS (SVCRS): AM, BA, GO, MA, MS, MT, PE, PR
#[napi]
pub fn get_state_contingency_authorizer(uf: String) -> napi::Result<Option<String>> {
    Ok(fiscal_sefaz::urls::get_state_contingency_authorizer(&uf).map(|s| s.to_string()))
}

/// Get the SEFAZ contingency service URL for a given state and environment.
///
/// Resolves the contingency authorizer (SVCAN or SVCRS) for the state and
/// returns the appropriate service URL.
///
/// # Errors
///
/// Returns [`FiscalError::InvalidStateCode`] if `uf` is not a valid Brazilian
/// state abbreviation or has no contingency mapping.
#[napi]
pub fn get_sefaz_contingency_url(
    uf: String,
    environment: String,
    service: String,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    fiscal_sefaz::urls::get_sefaz_contingency_url(&uf, sefaz_environment, &service)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

// ── Complement ──────────────────────────────────────────────────

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
#[napi]
pub fn attach_b2b(
    nfe_proc_xml: String,
    b2b_xml: String,
    tag_b2b: Option<String>,
) -> napi::Result<String> {
    fiscal_core::complement::attach_b2b(&nfe_proc_xml, &b2b_xml, tag_b2b.as_deref())
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

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
#[napi]
pub fn attach_cancellation(nfe_proc_xml: String, cancel_event_xml: String) -> napi::Result<String> {
    fiscal_core::complement::attach_cancellation(&nfe_proc_xml, &cancel_event_xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Attach an event protocol response to the event request,
/// producing the `<procEventoNFe>` wrapper.
///
/// Extracts `<evento>` from `request_xml` and `<retEvento>` from
/// `response_xml`, validates the event status, and joins them
/// into a `<procEventoNFe>` document.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if:
/// - Either input is empty
/// - The `<evento>` tag is missing from `request_xml`
/// - The `<retEvento>` tag is missing from `response_xml`
/// - The `<idLote>` tag is missing from `request_xml` or `response_xml`
/// - The `idLote` values differ between request and response
///
/// Returns [`FiscalError::SefazRejection`] if the event status code
/// is not valid (135, 136, or 155 for cancellation only).
#[napi]
pub fn attach_event_protocol(request_xml: String, response_xml: String) -> napi::Result<String> {
    fiscal_core::complement::attach_event_protocol(&request_xml, &response_xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Attach the SEFAZ inutilizacao response to the request,
/// producing the `<ProcInutNFe>` wrapper.
///
/// Extracts `<inutNFe>` from `request_xml` and `<retInutNFe>` from
/// `response_xml`, validates that the response status is `102` (voided),
/// and joins them into a `<ProcInutNFe>` document.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if:
/// - Either input is empty
/// - The `<inutNFe>` tag is missing from `request_xml`
/// - The `<retInutNFe>` tag is missing from `response_xml`
///
/// Returns [`FiscalError::SefazRejection`] if the response status is not `102`.
#[napi]
pub fn attach_inutilizacao(request_xml: String, response_xml: String) -> napi::Result<String> {
    fiscal_core::complement::attach_inutilizacao(&request_xml, &response_xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Detect the document type from raw XML and dispatch to the correct
/// protocol-attachment function.
///
/// This mirrors the PHP `Complements::toAuthorize()` method, which uses
/// `Standardize::whichIs()` internally. The detection logic checks for
/// the same root tags in the same priority order as the PHP implementation:
///
/// | Detected tag    | Dispatches to                  |
/// |-----------------|-------------------------------|
/// | `NFe`           | [`attach_protocol`]           |
/// | `envEvento`     | [`attach_event_protocol`]     |
/// | `inutNFe`       | [`attach_inutilizacao`]       |
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if:
/// - Either input is empty
/// - The request XML does not match any of the known document types
/// - The delegated function returns an error
#[napi]
pub fn to_authorize(request_xml: String, response_xml: String) -> napi::Result<String> {
    fiscal_core::complement::to_authorize(&request_xml, &response_xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Attach the SEFAZ authorization protocol to a signed NFe XML,
/// producing the `<nfeProc>` wrapper required for storage and DANFE.
///
/// The function extracts the `<NFe>` from `request_xml` and the matching
/// `<protNFe>` from `response_xml`, validates the protocol status, and
/// joins them into a single `<nfeProc>` document.
///
/// If the response contains multiple `<protNFe>` nodes (batch response),
/// the function attempts to match by digest value and access key. When no
/// exact match is found it falls back to the first available `<protNFe>`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if:
/// - Either input is empty
/// - The `<NFe>` tag is missing from `request_xml`
/// - No `<protNFe>` can be found in `response_xml`
///
/// Returns [`FiscalError::SefazRejection`] if the protocol status code
/// is not in [`VALID_PROTOCOL_STATUSES`].
#[napi]
pub fn attach_protocol(request_xml: String, response_xml: String) -> napi::Result<String> {
    fiscal_core::complement::attach_protocol(&request_xml, &response_xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

// ── Request Builders ────────────────────────────────────────────

/// Build a SEFAZ authorization request XML (`<enviNFe>`).
///
/// Wraps one or more signed NF-e XML documents in an `<enviNFe>` envelope
/// for submission to the SEFAZ authorization web service.
///
/// # Arguments
///
/// * `xml` - The signed NF-e XML (XML declaration is stripped automatically).
/// * `lot_id` - Lot identifier for the submission batch.
/// * `sync` - Whether to use synchronous processing (`indSinc=1`).
/// * `compressed` - Whether the XML content is gzip-compressed (flag only,
/// actual compression is handled at the transport layer).
///
/// # Panics
///
/// Panics if `xml` is empty.
///
/// # Errors
///
/// This function does not return `Result` errors but panics on invalid input.
#[napi]
pub fn build_autorizacao_request(
    xml: String,
    lot_id: String,
    sync: bool,
    _compressed: bool,
) -> napi::Result<String> {
    Ok(fiscal_sefaz::request_builders::build_autorizacao_request(
        &xml,
        &lot_id,
        sync,
        _compressed,
    ))
}

/// Build a SEFAZ service status request XML (`<consStatServ>`).
///
/// Queries the operational status of a SEFAZ web service for the given state.
///
/// # Panics
///
/// Panics if `uf` is not a valid Brazilian state code.
///
/// # Errors
///
/// This function panics on invalid state codes rather than returning `Result`.
#[napi]
pub fn build_status_request(uf: String, environment: String) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(fiscal_sefaz::request_builders::build_status_request(
        &uf,
        sefaz_environment,
    ))
}

/// Build a SEFAZ consultation request XML (`<consSitNFe>`) for an access key.
///
/// Queries the current status of an NF-e by its 44-digit access key.
///
/// # Panics
///
/// Panics if `access_key` is empty, not exactly 44 characters, or non-numeric.
///
/// # Errors
///
/// This function panics on invalid input rather than returning `Result`.
#[napi]
pub fn build_consulta_request(access_key: String, environment: String) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(fiscal_sefaz::request_builders::build_consulta_request(
        &access_key,
        sefaz_environment,
    ))
}

/// Build a SEFAZ receipt consultation request XML (`<consReciNFe>`).
///
/// Queries the processing result of a previously submitted batch by receipt number.
///
/// # Panics
///
/// Panics if `receipt` is empty.
///
/// # Errors
///
/// This function panics on invalid input rather than returning `Result`.
#[napi]
pub fn build_consulta_recibo_request(receipt: String, environment: String) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(fiscal_sefaz::request_builders::build_consulta_recibo_request(&receipt, sefaz_environment))
}

#[napi]
pub fn build_inutilizacao_request(
    year: u16,
    tax_id: String,
    model: String,
    series: u32,
    start_number: u32,
    end_number: u32,
    justification: String,
    environment: String,
    uf: String,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(fiscal_sefaz::request_builders::build_inutilizacao_request(
        year,
        &tax_id,
        &model,
        series,
        start_number,
        end_number,
        &justification,
        sefaz_environment,
        &uf,
    ))
}

/// Build a SEFAZ DistDFe (distribution) request XML (`<distDFeInt>`).
///
/// Queries the distribution of fiscal documents (DF-e) from the national
/// environment. Can search by last NSU, specific NSU, or access key.
///
/// # Arguments
///
/// * `uf` - State abbreviation of the interested party.
/// * `tax_id` - CNPJ or CPF of the interested party.
/// * `nsu` - Specific NSU or last NSU to query. If this is a 44-digit
/// all-numeric string, it is treated as an access key (`consChNFe`).
/// If `Some` with a 15-digit NSU, it uses `consNSU`.
/// If `None`, defaults to `distNSU` with `ultNSU=000000000000000`.
/// * `access_key` - Optional 44-digit access key for direct lookup.
/// * `environment` - SEFAZ environment.
///
/// # Panics
///
/// Panics if `uf` is not a valid Brazilian state code.
///
/// # Errors
///
/// This function panics on invalid state codes rather than returning `Result`.
#[napi]
pub fn build_dist_dfe_request(
    uf: String,
    tax_id: String,
    nsu: Option<String>,
    access_key: Option<String>,
    environment: String,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(fiscal_sefaz::request_builders::build_dist_dfe_request(
        &uf,
        &tax_id,
        nsu.as_deref(),
        access_key.as_deref(),
        sefaz_environment,
    ))
}

/// Build a SEFAZ cadastro (taxpayer registration) query XML (`<ConsCad>`).
///
/// Queries the SEFAZ taxpayer registry for a given state, searching by
/// CNPJ, IE (state tax ID), or CPF.
///
/// # Arguments
///
/// * `uf` - State abbreviation to query.
/// * `search_type` - One of `"CNPJ"`, `"IE"`, or `"CPF"`.
/// * `search_value` - The document number to search for.
///
/// # Errors
///
/// This function does not return `Result` errors.
#[napi]
pub fn build_cadastro_request(
    uf: String,
    search_type: String,
    search_value: String,
) -> napi::Result<String> {
    Ok(fiscal_sefaz::request_builders::build_cadastro_request(
        &uf,
        &search_type,
        &search_value,
    ))
}

/// Build a SEFAZ CSC (Código de Segurança do Contribuinte) request XML
/// (`<admCscNFCe>`).
///
/// Manages the CSC for NFC-e (model 65). Used exclusively with the
/// `CscNFCe` web service.
///
/// # Arguments
///
/// * `environment` — SEFAZ environment (production or homologation).
/// * `ind_op` — Operation type: 1=query active CSCs, 2=request new CSC,
/// 3=revoke active CSC.
/// * `cnpj` — Full CNPJ of the taxpayer (14 digits).
/// * `csc_id` — CSC identifier (required only for `ind_op=3`).
/// * `csc_code` — CSC code/value (required only for `ind_op=3`).
#[napi]
pub fn build_csc_request(
    environment: String,
    ind_op: u32,
    cnpj: String,
    csc_id: Option<String>,
    csc_code: Option<String>,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(fiscal_sefaz::request_builders::build_csc_request(
        sefaz_environment,
        ind_op as u8,
        &cnpj,
        csc_id.as_deref(),
        csc_code.as_deref(),
    ))
}

/// Extract [`EpecData`] from a signed NF-e XML string.
///
/// Parses the XML to extract all fields needed by the EPEC event. The
/// `ver_aplic_override` parameter, when `Some`, overrides the `<verProc>`
/// value from the XML (matching the PHP behavior where `$this->verAplic`
/// can override).
///
/// # Errors
///
/// Returns [`fiscal_core::FiscalError::XmlParsing`] if required tags are missing.
#[napi]
pub fn extract_epec_data(
    nfe_xml: String,
    ver_aplic_override: Option<String>,
) -> napi::Result<serde_json::Value> {
    let result =
        fiscal_sefaz::request_builders::extract_epec_data(&nfe_xml, ver_aplic_override.as_deref())
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Build a SEFAZ EPEC (Evento Prévio de Emissão em Contingência) event
/// request XML (`tpEvento=110140`).
///
/// The EPEC event is sent to the Ambiente Nacional (AN) with `cOrgao`
/// set to the IBGE code of the issuer's state. This matches the PHP
/// `Tools::sefazEPEC()` behavior.
///
/// # Arguments
///
/// * `epec_data` - Pre-extracted NF-e data (see [`extract_epec_data`]).
/// * `environment` - SEFAZ environment (production or homologation).
///
/// # Example
///
/// ```no_run
/// use fiscal_sefaz::request_builders::{build_epec_request, extract_epec_data};
/// use fiscal_core::types::SefazEnvironment;
///
/// let nfe_xml = "...signed NF-e XML...";
/// let data = extract_epec_data(nfe_xml, None).unwrap();
/// let request = build_epec_request(&data, SefazEnvironment::Homologation);
/// ```
#[napi]
pub fn build_epec_request(
    epec_data: serde_json::Value,
    environment: String,
) -> napi::Result<String> {
    let __epec_data = serde_json::from_value(epec_data)
        .map_err(|e| napi::Error::from_reason(format!("Invalid epec_data: {e}")))?;
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(fiscal_sefaz::request_builders::build_epec_request(
        &__epec_data,
        sefaz_environment,
    ))
}

/// Build a SEFAZ EPEC NFC-e status request XML (`<consStatServ>`).
///
/// Queries the operational status of the EPEC NFC-e service. This service
/// exists only for SP (São Paulo) and model 65 (NFC-e), matching the PHP
/// `sefazStatusEpecNfce()` method from `TraitEPECNfce`.
///
/// # Arguments
///
/// * `uf` - State abbreviation (must be `"SP"`).
/// * `environment` - SEFAZ environment (production or homologation).
///
/// # Panics
///
/// Panics if `uf` is not a valid Brazilian state code.
#[napi]
pub fn build_epec_nfce_status_request(uf: String, environment: String) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(fiscal_sefaz::request_builders::build_epec_nfce_status_request(&uf, sefaz_environment))
}

/// Extract [`EpecNfceData`] from a signed NFC-e XML string.
///
/// Parses the XML to extract all fields needed by the EPEC NFC-e event.
/// Unlike [`extract_epec_data`], the destination section is optional (NFC-e
/// can be issued without a recipient) and `vST` is not extracted.
///
/// # Errors
///
/// Returns [`fiscal_core::FiscalError::XmlParsing`] if required tags are missing.
#[napi]
pub fn extract_epec_nfce_data(
    nfce_xml: String,
    ver_aplic_override: Option<String>,
) -> napi::Result<serde_json::Value> {
    let result = fiscal_sefaz::request_builders::extract_epec_nfce_data(
        &nfce_xml,
        ver_aplic_override.as_deref(),
    )
    .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Build a SEFAZ EPEC NFC-e event request XML (`tpEvento=110140`).
///
/// Builds the complete `<envEvento>` wrapper for an EPEC event specific to
/// NFC-e (model 65). This is only available in SP and differs from the
/// standard EPEC in several ways:
///
/// - Sent to the state's `RecepcaoEPEC` endpoint (not Ambiente Nacional)
/// - `cOrgao` is the state's IBGE code (not 91)
/// - No `<vST>` field in the event detail
/// - The `<dest>` section is optional (NFC-e can have no recipient)
///
/// Matches the PHP `sefazEpecNfce()` method from `TraitEPECNfce`.
///
/// # Arguments
///
/// * `epec_data` - Pre-extracted NFC-e data (see [`extract_epec_nfce_data`]).
/// * `environment` - SEFAZ environment (production or homologation).
#[napi]
pub fn build_epec_nfce_request(
    epec_data: serde_json::Value,
    environment: String,
) -> napi::Result<String> {
    let __epec_data = serde_json::from_value(epec_data)
        .map_err(|e| napi::Error::from_reason(format!("Invalid epec_data: {e}")))?;
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(fiscal_sefaz::request_builders::build_epec_nfce_request(
        &__epec_data,
        sefaz_environment,
    ))
}

/// Build a SEFAZ cancel-prorrogacao (cancel ICMS extension) event request XML.
///
/// Creates an `<envEvento>` wrapper containing a cancelamento de pedido de
/// prorrogacao (`tpEvento=111502` for first term, `111503` for second term).
///
/// # Arguments
///
/// * `access_key` — 44-digit access key of the NF-e.
/// * `protocol` — authorization protocol number of the prorrogacao event.
/// * `second_term` — if `true`, uses the second-term event type (111503).
/// * `seq` — event sequence number.
/// * `environment` — SEFAZ environment.
/// * `tax_id` — CNPJ or CPF of the issuer.
#[napi]
pub fn build_cancel_prorrogacao_request(
    access_key: String,
    protocol: String,
    second_term: bool,
    seq: u32,
    environment: String,
    tax_id: String,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(
        fiscal_sefaz::request_builders::build_cancel_prorrogacao_request(
            &access_key,
            &protocol,
            second_term,
            seq,
            sefaz_environment,
            &tax_id,
        ),
    )
}

/// Build a SEFAZ cancellation event request XML.
///
/// Builds the complete `<envEvento>` wrapper containing a cancellation event
/// (`tpEvento=110111`) for a previously authorized NF-e.
///
/// # Arguments
///
/// * `access_key` - The 44-digit access key of the NF-e to cancel.
/// * `protocol` - The protocol number from the authorization response.
/// * `justification` - Justification text (minimum 15 characters).
/// * `seq` - Event sequence number (usually 1).
/// * `environment` - SEFAZ environment.
/// * `tax_id` - CNPJ or CPF of the issuer.
///
/// # Panics
///
/// Panics if `justification` is empty.
///
/// # Errors
///
/// This function panics on invalid input rather than returning `Result`.
#[napi]
pub fn build_cancela_request(
    access_key: String,
    protocol: String,
    justification: String,
    seq: u32,
    environment: String,
    tax_id: String,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(fiscal_sefaz::request_builders::build_cancela_request(
        &access_key,
        &protocol,
        &justification,
        seq,
        sefaz_environment,
        &tax_id,
    ))
}

/// Build a SEFAZ CCe (correction letter) event request XML.
///
/// Builds the complete `<envEvento>` wrapper containing a Carta de Correcao
/// (`tpEvento=110110`) for a previously authorized NF-e.
///
/// # Arguments
///
/// * `access_key` - The 44-digit access key of the NF-e to correct.
/// * `correction` - The correction text describing what is being changed.
/// * `seq` - Event sequence number (increments for each correction on the same NF-e).
/// * `environment` - SEFAZ environment.
/// * `tax_id` - CNPJ or CPF of the issuer.
///
/// # Panics
///
/// Panics if `correction` is empty.
///
/// # Errors
///
/// This function panics on invalid input rather than returning `Result`.
#[napi]
pub fn build_cce_request(
    access_key: String,
    correction: String,
    seq: u32,
    environment: String,
    tax_id: String,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(fiscal_sefaz::request_builders::build_cce_request(
        &access_key,
        &correction,
        seq,
        sefaz_environment,
        &tax_id,
    ))
}

/// Build a SEFAZ manifest (manifestacao do destinatario) event request XML.
///
/// Builds the complete `<envEvento>` wrapper containing a manifestation event.
/// Valid event types are:
/// - `"210200"` (Confirmacao da Operacao)
/// - `"210210"` (Ciencia da Operacao)
/// - `"210220"` (Desconhecimento da Operacao)
/// - `"210240"` (Operacao nao Realizada) - requires justification
///
/// # Arguments
///
/// * `access_key` - The 44-digit access key of the NF-e.
/// * `event_type` - Event type code as string (e.g. `"210210"`).
/// * `justification` - Required only for `"210240"` (operation not performed).
/// * `seq` - Event sequence number.
/// * `environment` - SEFAZ environment.
/// * `tax_id` - CNPJ or CPF of the recipient.
///
/// # Errors
///
/// This function does not return errors but may produce invalid XML if the
/// event type is not one of the valid manifest types.
#[napi]
pub fn build_manifesta_request(
    access_key: String,
    event_type: String,
    justification: Option<String>,
    seq: u32,
    environment: String,
    tax_id: String,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(fiscal_sefaz::request_builders::build_manifesta_request(
        &access_key,
        &event_type,
        justification.as_deref(),
        seq,
        sefaz_environment,
        &tax_id,
    ))
}

#[napi]
pub fn build_cancel_substituicao_request(
    access_key: String,
    ref_access_key: String,
    protocol: String,
    justification: String,
    ver_aplic: String,
    seq: u32,
    environment: String,
    tax_id: String,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(
        fiscal_sefaz::request_builders::build_cancel_substituicao_request(
            &access_key,
            &ref_access_key,
            &protocol,
            &justification,
            &ver_aplic,
            seq,
            sefaz_environment,
            &tax_id,
        ),
    )
}

#[napi]
pub fn build_ator_interessado_request(
    access_key: String,
    tp_autor: u32,
    ver_aplic: String,
    authorized_cnpj: Option<String>,
    authorized_cpf: Option<String>,
    tp_autorizacao: u32,
    issuer_uf: String,
    seq: u32,
    environment: String,
    tax_id: String,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(
        fiscal_sefaz::request_builders::build_ator_interessado_request(
            &access_key,
            tp_autor as u8,
            &ver_aplic,
            authorized_cnpj.as_deref(),
            authorized_cpf.as_deref(),
            tp_autorizacao as u8,
            &issuer_uf,
            seq,
            sefaz_environment,
            &tax_id,
        ),
    )
}

#[napi]
pub fn build_comprovante_entrega_request(
    access_key: String,
    ver_aplic: String,
    delivery_date: String,
    doc_number: String,
    name: String,
    lat: Option<String>,
    long: Option<String>,
    hash: String,
    hash_date: String,
    issuer_uf: String,
    seq: u32,
    environment: String,
    tax_id: String,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(
        fiscal_sefaz::request_builders::build_comprovante_entrega_request(
            &access_key,
            &ver_aplic,
            &delivery_date,
            &doc_number,
            &name,
            lat.as_deref(),
            long.as_deref(),
            &hash,
            &hash_date,
            &issuer_uf,
            seq,
            sefaz_environment,
            &tax_id,
        ),
    )
}

#[napi]
pub fn build_cancel_comprovante_entrega_request(
    access_key: String,
    ver_aplic: String,
    event_protocol: String,
    issuer_uf: String,
    seq: u32,
    environment: String,
    tax_id: String,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(
        fiscal_sefaz::request_builders::build_cancel_comprovante_entrega_request(
            &access_key,
            &ver_aplic,
            &event_protocol,
            &issuer_uf,
            seq,
            sefaz_environment,
            &tax_id,
        ),
    )
}

#[napi]
pub fn build_insucesso_entrega_request(
    access_key: String,
    ver_aplic: String,
    attempt_date: String,
    attempt_number: Option<u32>,
    reason_type: u32,
    reason_justification: Option<String>,
    lat: Option<String>,
    long: Option<String>,
    hash: String,
    hash_date: String,
    issuer_uf: String,
    seq: u32,
    environment: String,
    tax_id: String,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(
        fiscal_sefaz::request_builders::build_insucesso_entrega_request(
            &access_key,
            &ver_aplic,
            &attempt_date,
            attempt_number,
            reason_type as u8,
            reason_justification.as_deref(),
            lat.as_deref(),
            long.as_deref(),
            &hash,
            &hash_date,
            &issuer_uf,
            seq,
            sefaz_environment,
            &tax_id,
        ),
    )
}

#[napi]
pub fn build_cancel_insucesso_entrega_request(
    access_key: String,
    ver_aplic: String,
    event_protocol: String,
    issuer_uf: String,
    seq: u32,
    environment: String,
    tax_id: String,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(
        fiscal_sefaz::request_builders::build_cancel_insucesso_entrega_request(
            &access_key,
            &ver_aplic,
            &event_protocol,
            &issuer_uf,
            seq,
            sefaz_environment,
            &tax_id,
        ),
    )
}

/// Build RTC event: Informação de pagamento integral (tpEvento=112110).
#[napi]
pub fn build_rtc_info_pagto_integral(
    access_key: String,
    seq: u32,
    environment: String,
    tax_id: String,
    c_uf: String,
    ver_aplic: String,
    org_code_override: Option<String>,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(
        fiscal_sefaz::request_builders::build_rtc_info_pagto_integral(
            &access_key,
            seq,
            sefaz_environment,
            &tax_id,
            &c_uf,
            &ver_aplic,
            org_code_override.as_deref(),
        ),
    )
}

#[napi]
pub fn build_rtc_aceite_debito(
    access_key: String,
    seq: u32,
    environment: String,
    tax_id: String,
    c_uf: String,
    ver_aplic: String,
    ind_aceitacao: u32,
    org_code_override: Option<String>,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(fiscal_sefaz::request_builders::build_rtc_aceite_debito(
        &access_key,
        seq,
        sefaz_environment,
        &tax_id,
        &c_uf,
        &ver_aplic,
        ind_aceitacao as u8,
        org_code_override.as_deref(),
    ))
}

#[napi]
pub fn build_rtc_manif_transf_cred_ibs(
    access_key: String,
    seq: u32,
    environment: String,
    tax_id: String,
    c_uf: String,
    ver_aplic: String,
    ind_aceitacao: u32,
    org_code_override: Option<String>,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(
        fiscal_sefaz::request_builders::build_rtc_manif_transf_cred_ibs(
            &access_key,
            seq,
            sefaz_environment,
            &tax_id,
            &c_uf,
            &ver_aplic,
            ind_aceitacao as u8,
            org_code_override.as_deref(),
        ),
    )
}

#[napi]
pub fn build_rtc_manif_transf_cred_cbs(
    access_key: String,
    seq: u32,
    environment: String,
    tax_id: String,
    c_uf: String,
    ver_aplic: String,
    ind_aceitacao: u32,
    org_code_override: Option<String>,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(
        fiscal_sefaz::request_builders::build_rtc_manif_transf_cred_cbs(
            &access_key,
            seq,
            sefaz_environment,
            &tax_id,
            &c_uf,
            &ver_aplic,
            ind_aceitacao as u8,
            org_code_override.as_deref(),
        ),
    )
}

#[napi]
pub fn build_rtc_cancela_evento(
    access_key: String,
    seq: u32,
    environment: String,
    tax_id: String,
    c_uf: String,
    ver_aplic: String,
    tp_evento_aut: String,
    n_prot_evento: String,
    org_code_override: Option<String>,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(fiscal_sefaz::request_builders::build_rtc_cancela_evento(
        &access_key,
        seq,
        sefaz_environment,
        &tax_id,
        &c_uf,
        &ver_aplic,
        &tp_evento_aut,
        &n_prot_evento,
        org_code_override.as_deref(),
    ))
}

#[napi]
pub fn build_rtc_atualizacao_data_entrega(
    access_key: String,
    seq: u32,
    environment: String,
    tax_id: String,
    c_uf: String,
    ver_aplic: String,
    data_prevista: String,
    org_code_override: Option<String>,
) -> napi::Result<String> {
    let sefaz_environment = parse_sefaz_environment(&environment)?;
    Ok(
        fiscal_sefaz::request_builders::build_rtc_atualizacao_data_entrega(
            &access_key,
            seq,
            sefaz_environment,
            &tax_id,
            &c_uf,
            &ver_aplic,
            &data_prevista,
            org_code_override.as_deref(),
        ),
    )
}

// ── Response Parsers ────────────────────────────────────────────

/// Parse a SEFAZ authorization response (`retEnviNFe` / `nfeAutorizacaoLoteResult`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts the protocol information
/// from `<protNFe><infProt>` when present.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element at any level.
#[napi]
pub fn parse_autorizacao_response(xml: String) -> napi::Result<serde_json::Value> {
    let result = fiscal_sefaz::response_parsers::parse_autorizacao_response(&xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Parse a SEFAZ consulta recibo response (`retConsReciNFe`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts all batch-level fields
/// and individual `<protNFe>` protocol entries.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element.
#[napi]
pub fn parse_consulta_recibo_response(xml: String) -> napi::Result<serde_json::Value> {
    let result = fiscal_sefaz::response_parsers::parse_consulta_recibo_response(&xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Parse a SEFAZ consulta situação response (`retConsSitNFe`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts the situation fields,
/// optional `<protNFe>` and any `<retEvento>` entries.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element.
#[napi]
pub fn parse_consulta_situacao_response(xml: String) -> napi::Result<serde_json::Value> {
    let result = fiscal_sefaz::response_parsers::parse_consulta_situacao_response(&xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Parse a SEFAZ cancellation event response (`retEvento`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts `cStat`, `xMotivo`, and
/// optionally `nProt` from `<infEvento>`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element.
#[napi]
pub fn parse_cancellation_response(xml: String) -> napi::Result<serde_json::Value> {
    let result = fiscal_sefaz::response_parsers::parse_cancellation_response(&xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Parse a SEFAZ inutilização response (`retInutNFe`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts all fields from `<infInut>`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element.
#[napi]
pub fn parse_inutilizacao_response(xml: String) -> napi::Result<serde_json::Value> {
    let result = fiscal_sefaz::response_parsers::parse_inutilizacao_response(&xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Parse a SEFAZ DistDFe response (`retDistDFeInt`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts `cStat`, `xMotivo`,
/// `ultNSU`, and `maxNSU`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element.
#[napi]
pub fn parse_dist_dfe_response(xml: String) -> napi::Result<serde_json::Value> {
    let result = fiscal_sefaz::response_parsers::parse_dist_dfe_response(&xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Parse a SEFAZ Cadastro response (`retConsCad`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts `cStat` and `xMotivo`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element.
#[napi]
pub fn parse_cadastro_response(xml: String) -> napi::Result<serde_json::Value> {
    let result = fiscal_sefaz::response_parsers::parse_cadastro_response(&xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Parse a SEFAZ NFC-e CSC administration response (`retAdmCscNFCe`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts `tpAmb`, `indOp`, `cStat`,
/// `xMotivo`, and any `idCsc`/`CSC` token pairs from `<retInfCsc>`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element.
#[napi]
pub fn parse_csc_response(xml: String) -> napi::Result<serde_json::Value> {
    let result = fiscal_sefaz::response_parsers::parse_csc_response(&xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Parse a SEFAZ service status response (`retConsStatServ`).
///
/// The response may be wrapped in a SOAP envelope. The parser strips SOAP
/// wrappers and namespace prefixes, then extracts `cStat`, `xMotivo`, and
/// optionally `tMed` (average processing time).
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if the XML is malformed or does not
/// contain the expected `<cStat>` element.
#[napi]
pub fn parse_status_response(xml: String) -> napi::Result<serde_json::Value> {
    let result = fiscal_sefaz::response_parsers::parse_status_response(&xml)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}

// ── Enum Parsers ─────────────────────────────────────────────

fn parse_sefaz_environment(s: &str) -> napi::Result<fiscal_core::types::SefazEnvironment> {
    match s.to_lowercase().as_str() {
        "production" | "1" => Ok(fiscal_core::types::SefazEnvironment::Production),
        "homologation" | "2" => Ok(fiscal_core::types::SefazEnvironment::Homologation),
        _ => Err(napi::Error::from_reason(format!(
            "Invalid SefazEnvironment: \"{s}\""
        ))),
    }
}
