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

// ── Complement ────────────────────────────────────────────

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
