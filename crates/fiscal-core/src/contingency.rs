use crate::FiscalError;
use crate::newtypes::IbgeCode;
use crate::types::{AccessKeyParams, ContingencyType, EmissionType, InvoiceModel};
use crate::xml_builder::access_key::build_access_key;
use crate::xml_utils::extract_xml_tag_value;

/// Contingency manager for NF-e emission.
///
/// Manages activation and deactivation of contingency mode, used when the
/// primary SEFAZ authorizer is unavailable. Supports all contingency types
/// defined in the NF-e specification: SVC-AN, SVC-RS, EPEC, FS-DA, FS-IA,
/// and offline.
///
/// # JSON persistence
///
/// The state can be serialized to / deserialized from a compact JSON string
/// using [`to_json`](Contingency::to_json) and [`load`](Contingency::load),
/// matching the PHP `Contingency::__toString()` format:
///
/// ```json
/// {"motive":"reason","timestamp":1480700623,"type":"SVCAN","tpEmis":6}
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Contingency {
    /// The active contingency type, or `None` when in normal mode.
    pub contingency_type: Option<ContingencyType>,
    /// Justification reason for entering contingency (15-256 chars).
    pub reason: Option<String>,
    /// ISO-8601 timestamp when contingency was activated.
    pub activated_at: Option<String>,
    /// Unix timestamp (seconds since epoch) of activation.
    pub timestamp: u64,
}

impl Contingency {
    /// Create a new contingency manager with no active contingency (normal mode).
    pub fn new() -> Self {
        Self {
            contingency_type: None,
            reason: None,
            activated_at: None,
            timestamp: 0,
        }
    }

    /// Returns `true` when a contingency mode is currently active.
    pub fn is_active(&self) -> bool {
        self.contingency_type.is_some()
    }

    /// Activate contingency mode with the given type and justification reason.
    ///
    /// The reason is trimmed and must be between 15 and 256 UTF-8 characters
    /// (inclusive). On success, the contingency is activated with the current
    /// UTC timestamp.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Contingency`] if the trimmed reason is shorter
    /// than 15 characters or longer than 256 characters.
    pub fn activate(
        &mut self,
        contingency_type: ContingencyType,
        reason: &str,
    ) -> Result<(), FiscalError> {
        let trimmed = reason.trim();
        let len = trimmed.chars().count();
        if !(15..=256).contains(&len) {
            return Err(FiscalError::Contingency(
                "The justification for entering contingency mode must be between 15 and 256 UTF-8 characters.".to_string(),
            ));
        }

        // Use current UTC timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.contingency_type = Some(contingency_type);
        self.reason = Some(trimmed.to_string());
        self.timestamp = now;
        self.activated_at = Some(
            chrono::DateTime::from_timestamp(now as i64, 0)
                .unwrap_or_default()
                .to_rfc3339(),
        );
        Ok(())
    }

    /// Deactivate contingency mode, resetting to normal emission.
    pub fn deactivate(&mut self) {
        self.contingency_type = None;
        self.reason = None;
        self.activated_at = None;
        self.timestamp = 0;
    }

    /// Load contingency state from a JSON string.
    ///
    /// Expected JSON format (matching PHP `Contingency`):
    /// ```json
    /// {"motive":"reason","timestamp":1480700623,"type":"SVCAN","tpEmis":6}
    /// ```
    ///
    /// Accepts all contingency type strings: `SVCAN`, `SVC-AN`, `SVCRS`,
    /// `SVC-RS`, `EPEC`, `FSDA`, `FS-DA`, `FSIA`, `FS-IA`, `OFFLINE`,
    /// and their lowercase equivalents.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Contingency`] if the JSON cannot be parsed or
    /// contains an unrecognized contingency type.
    pub fn load(json: &str) -> Result<Self, FiscalError> {
        // Manual JSON parsing to avoid adding serde_json as a runtime dependency.
        let motive = extract_json_string(json, "motive")
            .ok_or_else(|| FiscalError::Contingency("Missing 'motive' in JSON".to_string()))?;
        let timestamp = extract_json_number(json, "timestamp")
            .ok_or_else(|| FiscalError::Contingency("Missing 'timestamp' in JSON".to_string()))?;
        let type_str = extract_json_string(json, "type")
            .ok_or_else(|| FiscalError::Contingency("Missing 'type' in JSON".to_string()))?;
        let tp_emis = extract_json_number(json, "tpEmis")
            .ok_or_else(|| FiscalError::Contingency("Missing 'tpEmis' in JSON".to_string()))?;

        let contingency_type = ContingencyType::from_type_str(&type_str);

        // Validate that, if a type is given, it is recognized
        if !type_str.is_empty() && contingency_type.is_none() {
            return Err(FiscalError::Contingency(format!(
                "Unrecognized contingency type: {type_str}"
            )));
        }

        let _ = tp_emis; // Validated through contingency_type mapping

        Ok(Self {
            contingency_type,
            reason: if motive.is_empty() {
                None
            } else {
                Some(motive)
            },
            activated_at: if timestamp > 0 {
                Some(
                    chrono::DateTime::from_timestamp(timestamp as i64, 0)
                        .unwrap_or_default()
                        .to_rfc3339(),
                )
            } else {
                None
            },
            timestamp,
        })
    }

    /// Serialize the contingency state to a JSON string.
    ///
    /// Produces the same format as the PHP `Contingency::__toString()`:
    /// ```json
    /// {"motive":"reason","timestamp":1480700623,"type":"SVCAN","tpEmis":6}
    /// ```
    ///
    /// When deactivated, produces:
    /// ```json
    /// {"motive":"","timestamp":0,"type":"","tpEmis":1}
    /// ```
    pub fn to_json(&self) -> String {
        let motive = self.reason.as_deref().unwrap_or("");
        let type_str = self
            .contingency_type
            .map(|ct| ct.to_type_str())
            .unwrap_or("");
        let tp_emis = self.emission_type();
        format!(
            r#"{{"motive":"{}","timestamp":{},"type":"{}","tpEmis":{}}}"#,
            escape_json_string(motive),
            self.timestamp,
            type_str,
            tp_emis
        )
    }

    /// Get the emission type code for the current contingency state.
    ///
    /// Returns `1` (normal) if no contingency is active, or the corresponding
    /// `tpEmis` code: `2` (FS-IA), `4` (EPEC), `5` (FS-DA), `6` (SVC-AN),
    /// `7` (SVC-RS), `9` (offline).
    pub fn emission_type(&self) -> u8 {
        match self.contingency_type {
            Some(ct) => ct.tp_emis(),
            None => 1,
        }
    }

    /// Get the [`EmissionType`] enum for the current contingency state.
    pub fn emission_type_enum(&self) -> EmissionType {
        match self.contingency_type {
            Some(ContingencyType::SvcAn) => EmissionType::SvcAn,
            Some(ContingencyType::SvcRs) => EmissionType::SvcRs,
            Some(ContingencyType::Epec) => EmissionType::Epec,
            Some(ContingencyType::FsDa) => EmissionType::FsDa,
            Some(ContingencyType::FsIa) => EmissionType::FsIa,
            Some(ContingencyType::Offline) => EmissionType::Offline,
            None => EmissionType::Normal,
        }
    }

    /// Check whether the current contingency mode has a dedicated web service.
    ///
    /// Only SVC-AN and SVC-RS have their own SEFAZ web services. Other types
    /// (EPEC, FS-DA, FS-IA, offline) do not have their own web services for
    /// NF-e authorization and will return an error if used with `sefazAutorizacao`.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Contingency`] if:
    /// - The document is model 65 (NFC-e) and an SVC contingency is active
    ///   (NFC-e does not support SVC-AN/SVC-RS).
    /// - The active contingency type does not have a dedicated web service
    ///   (EPEC, FS-DA, FS-IA, offline).
    pub fn check_web_service_availability(&self, model: InvoiceModel) -> Result<(), FiscalError> {
        let ct = match self.contingency_type {
            Some(ct) => ct,
            None => return Ok(()),
        };

        if model == InvoiceModel::Nfce
            && matches!(ct, ContingencyType::SvcAn | ContingencyType::SvcRs)
        {
            return Err(FiscalError::Contingency(
                "Não existe serviço para contingência SVCRS ou SVCAN para NFCe (modelo 65)."
                    .to_string(),
            ));
        }

        if !matches!(ct, ContingencyType::SvcAn | ContingencyType::SvcRs) {
            return Err(FiscalError::Contingency(format!(
                "Esse modo de contingência [{}] não possui webservice próprio, portanto não haverão envios.",
                ct.to_type_str()
            )));
        }

        Ok(())
    }
}

impl Default for Contingency {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Display for Contingency {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.to_json())
    }
}

/// Get the default contingency type (SVC-AN or SVC-RS) for a given Brazilian state.
///
/// Each state has a pre-assigned SVC authorizer:
/// - **SVC-RS** (8 states): AM, BA, GO, MA, MS, MT, PE, PR
/// - **SVC-AN** (19 states): all others (AC, AL, AP, CE, DF, ES, MG, PA, PB,
///   PI, RJ, RN, RO, RR, RS, SC, SE, SP, TO)
///
/// # Panics
///
/// Panics if `uf` is not a valid 2-letter Brazilian state abbreviation.
pub fn contingency_for_state(uf: &str) -> ContingencyType {
    match uf {
        "AM" | "BA" | "GO" | "MA" | "MS" | "MT" | "PE" | "PR" => ContingencyType::SvcRs,
        "AC" | "AL" | "AP" | "CE" | "DF" | "ES" | "MG" | "PA" | "PB" | "PI" | "RJ" | "RN"
        | "RO" | "RR" | "RS" | "SC" | "SE" | "SP" | "TO" => ContingencyType::SvcAn,
        _ => panic!("Unknown state abbreviation: {uf}"),
    }
}

/// Get the default contingency type (SVC-AN or SVC-RS) for a given Brazilian state.
///
/// Same as [`contingency_for_state`] but returns a `Result` instead of panicking.
///
/// # Errors
///
/// Returns [`FiscalError::InvalidStateCode`] if `uf` is not a valid 2-letter
/// Brazilian state abbreviation.
pub fn try_contingency_for_state(uf: &str) -> Result<ContingencyType, FiscalError> {
    match uf {
        "AM" | "BA" | "GO" | "MA" | "MS" | "MT" | "PE" | "PR" => Ok(ContingencyType::SvcRs),
        "AC" | "AL" | "AP" | "CE" | "DF" | "ES" | "MG" | "PA" | "PB" | "PI" | "RJ" | "RN"
        | "RO" | "RR" | "RS" | "SC" | "SE" | "SP" | "TO" => Ok(ContingencyType::SvcAn),
        _ => Err(FiscalError::InvalidStateCode(uf.to_string())),
    }
}

/// Adjust an NF-e XML string for contingency mode.
///
/// Modifies the XML to:
/// 1. Replace the `<tpEmis>` value with the contingency emission type
/// 2. Insert `<dhCont>` (contingency datetime) and `<xJust>` (reason) inside `<ide>`
/// 3. Recalculate the access key and check digit
///
/// If the contingency is not active (no type set), returns the XML unchanged.
/// If the XML already has a non-normal `<tpEmis>` (not `1`), returns unchanged.
///
/// # Errors
///
/// Returns [`FiscalError::Contingency`] if the XML belongs to an NFC-e (model 65),
/// since SVC-AN/SVC-RS contingency does not apply to NFC-e documents.
///
/// Returns [`FiscalError::XmlParsing`] if required XML tags cannot be found.
pub fn adjust_nfe_contingency(xml: &str, contingency: &Contingency) -> Result<String, FiscalError> {
    // If no contingency is active, return XML unchanged
    if contingency.contingency_type.is_none() {
        return Ok(xml.to_string());
    }

    // Remove XML signature if present
    let mut xml = remove_signature(xml);

    // Check model - must be NF-e (55), not NFC-e (65)
    let model = extract_xml_tag_value(&xml, "mod").unwrap_or_default();
    if model == "65" {
        return Err(FiscalError::Contingency(
            "The XML belongs to a model 65 document (NFC-e), incorrect for SVCAN or SVCRS contingency.".to_string(),
        ));
    }

    // Check if already in contingency mode
    let current_tp_emis = extract_xml_tag_value(&xml, "tpEmis").unwrap_or_default();
    if current_tp_emis != "1" {
        // Already configured for contingency, return as-is
        return Ok(xml);
    }

    // Extract fields for access key recalculation
    let c_uf = extract_xml_tag_value(&xml, "cUF").unwrap_or_default();
    let c_nf = extract_xml_tag_value(&xml, "cNF").unwrap_or_default();
    let n_nf = extract_xml_tag_value(&xml, "nNF").unwrap_or_default();
    let serie = extract_xml_tag_value(&xml, "serie").unwrap_or_default();
    let dh_emi = extract_xml_tag_value(&xml, "dhEmi").unwrap_or_default();

    // Extract emitter CNPJ or CPF from <emit> block
    let emit_doc = extract_emitter_doc(&xml);

    // Parse emission date for year/month
    let (year, month) = parse_year_month(&dh_emi);

    // Format contingency datetime with timezone from dhEmi
    let tz_offset = extract_tz_offset(&dh_emi);
    let dth_cont = format_timestamp_with_offset(contingency.timestamp, &tz_offset);

    let reason = contingency.reason.as_deref().unwrap_or("").trim();
    let tp_emis = contingency.emission_type();

    // Replace tpEmis value
    xml = xml.replacen(
        &format!("<tpEmis>{current_tp_emis}</tpEmis>"),
        &format!("<tpEmis>{tp_emis}</tpEmis>"),
        1,
    );

    // Insert dhCont
    if xml.contains("<dhCont>") {
        // Replace existing dhCont
        let re_start = xml.find("<dhCont>").unwrap();
        let re_end = xml.find("</dhCont>").unwrap() + "</dhCont>".len();
        xml = format!(
            "{}<dhCont>{dth_cont}</dhCont>{}",
            &xml[..re_start],
            &xml[re_end..]
        );
    } else if xml.contains("<NFref>") {
        xml = xml.replacen("<NFref>", &format!("<dhCont>{dth_cont}</dhCont><NFref>"), 1);
    } else {
        xml = xml.replacen("</ide>", &format!("<dhCont>{dth_cont}</dhCont></ide>"), 1);
    }

    // Insert xJust
    if xml.contains("<xJust>") {
        // Replace existing xJust
        let re_start = xml.find("<xJust>").unwrap();
        let re_end = xml.find("</xJust>").unwrap() + "</xJust>".len();
        xml = format!(
            "{}<xJust>{reason}</xJust>{}",
            &xml[..re_start],
            &xml[re_end..]
        );
    } else if xml.contains("<NFref>") {
        xml = xml.replacen("<NFref>", &format!("<xJust>{reason}</xJust><NFref>"), 1);
    } else {
        xml = xml.replacen("</ide>", &format!("<xJust>{reason}</xJust></ide>"), 1);
    }

    // Recalculate access key
    let model_enum = match model.as_str() {
        "65" => InvoiceModel::Nfce,
        _ => InvoiceModel::Nfe,
    };
    let emission_type_enum = contingency.emission_type_enum();

    let new_key = build_access_key(&AccessKeyParams {
        state_code: IbgeCode(c_uf),
        year_month: format!("{year}{month}"),
        tax_id: emit_doc,
        model: model_enum,
        series: serie.parse().unwrap_or(0),
        number: n_nf.parse().unwrap_or(0),
        emission_type: emission_type_enum,
        numeric_code: c_nf,
    })?;

    // Update cDV (check digit is last char of access key)
    let new_cdv = &new_key[new_key.len() - 1..];
    // Replace <cDV> tag
    if let Some(start) = xml.find("<cDV>") {
        if let Some(end) = xml[start..].find("</cDV>") {
            let full_end = start + end + "</cDV>".len();
            xml = format!("{}<cDV>{new_cdv}</cDV>{}", &xml[..start], &xml[full_end..]);
        }
    }

    // Update infNFe Id attribute
    // Match pattern: Id="NFeXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
    if let Some(id_start) = xml.find("Id=\"NFe") {
        let after_nfe = id_start + 7; // past Id="NFe
        // Find the closing quote — the key is 44 digits
        if xml.len() >= after_nfe + 44 {
            let id_end = after_nfe + 44;
            xml = format!("{}NFe{new_key}{}", &xml[..after_nfe], &xml[id_end..]);
        }
    }

    Ok(xml)
}

// ── Private helpers ─────────────────────────────────────────────────────────

/// Remove XML digital signature block if present.
fn remove_signature(xml: &str) -> String {
    // Remove <Signature xmlns...>...</Signature>
    if let Some(start) = xml.find("<Signature") {
        if let Some(end) = xml.find("</Signature>") {
            let full_end = end + "</Signature>".len();
            return format!("{}{}", xml[..start].trim_end(), &xml[full_end..])
                .trim()
                .to_string();
        }
    }
    xml.to_string()
}

/// Extract the emitter's CNPJ or CPF from the <emit> block.
fn extract_emitter_doc(xml: &str) -> String {
    if let Some(emit_start) = xml.find("<emit>") {
        if let Some(emit_end) = xml.find("</emit>") {
            let emit_block = &xml[emit_start..emit_end];
            // Try CNPJ first
            if let Some(cnpj) = extract_inner(emit_block, "CNPJ") {
                return cnpj;
            }
            // Then CPF
            if let Some(cpf) = extract_inner(emit_block, "CPF") {
                return cpf;
            }
        }
    }
    String::new()
}

/// Extract inner text from a simple XML tag.
fn extract_inner(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml[start..end].to_string())
}

/// Parse YY and MM from an ISO datetime string like "2018-09-25T00:00:00-03:00".
fn parse_year_month(dh_emi: &str) -> (String, String) {
    if dh_emi.len() >= 7 {
        let year = &dh_emi[2..4]; // "18"
        let month = &dh_emi[5..7]; // "09"
        (year.to_string(), month.to_string())
    } else {
        ("00".to_string(), "00".to_string())
    }
}

/// Extract timezone offset from an ISO datetime string.
/// Returns something like "-03:00". Defaults to "-03:00" if not found.
fn extract_tz_offset(dh_emi: &str) -> String {
    // Look for +HH:MM or -HH:MM at the end
    if dh_emi.len() >= 6 {
        let tail = &dh_emi[dh_emi.len() - 6..];
        if (tail.starts_with('+') || tail.starts_with('-')) && tail.as_bytes()[3] == b':' {
            return tail.to_string();
        }
    }
    "-03:00".to_string()
}

/// Format a unix timestamp as ISO datetime with a given timezone offset.
fn format_timestamp_with_offset(timestamp: u64, offset: &str) -> String {
    // Parse offset to get total seconds
    let offset_seconds = parse_offset_seconds(offset);

    // Create a chrono FixedOffset and format
    if let Some(fo) = chrono::FixedOffset::east_opt(offset_seconds) {
        if let Some(dt) = chrono::DateTime::from_timestamp(timestamp as i64, 0) {
            let local = dt.with_timezone(&fo);
            return local.format("%Y-%m-%dT%H:%M:%S").to_string() + offset;
        }
    }

    // Fallback: just format as UTC
    format!("1970-01-01T00:00:00{offset}")
}

/// Parse a timezone offset string like "-03:00" into total seconds.
fn parse_offset_seconds(offset: &str) -> i32 {
    if offset.len() < 6 {
        return 0;
    }
    let sign: i32 = if offset.starts_with('-') { -1 } else { 1 };
    let hours: i32 = offset[1..3].parse().unwrap_or(0);
    let minutes: i32 = offset[4..6].parse().unwrap_or(0);
    sign * (hours * 3600 + minutes * 60)
}

/// Escape a string for JSON output — handles `"`, `\`, and control characters.
fn escape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                // \uXXXX for other control chars
                for unit in c.encode_utf16(&mut [0; 2]) {
                    out.push_str(&format!("\\u{unit:04x}"));
                }
            }
            _ => out.push(c),
        }
    }
    out
}

/// Extract a string value from a simple JSON object by key.
/// E.g., from `{"key":"value"}` extracts "value" for key "key".
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let search = format!("\"{key}\"");
    let idx = json.find(&search)?;
    let after_key = idx + search.len();
    // Skip whitespace and colon
    let rest = json[after_key..].trim_start();
    let rest = rest.strip_prefix(':')?;
    let rest = rest.trim_start();

    if let Some(content) = rest.strip_prefix('"') {
        // String value
        let end = content.find('"')?;
        Some(content[..end].to_string())
    } else {
        None
    }
}

/// Extract a numeric value from a simple JSON object by key.
/// E.g., from `{"key":123}` extracts 123 for key "key".
fn extract_json_number(json: &str, key: &str) -> Option<u64> {
    let search = format!("\"{key}\"");
    let idx = json.find(&search)?;
    let after_key = idx + search.len();
    let rest = json[after_key..].trim_start();
    let rest = rest.strip_prefix(':')?;
    let rest = rest.trim_start();

    // Read digits
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    if end == 0 {
        return None;
    }
    rest[..end].parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_contingency_is_inactive() {
        let c = Contingency::new();
        assert!(c.contingency_type.is_none());
        assert!(!c.is_active());
        assert_eq!(c.emission_type(), 1);
    }

    #[test]
    fn default_is_inactive() {
        let c = Contingency::default();
        assert!(c.contingency_type.is_none());
        assert!(!c.is_active());
    }

    #[test]
    fn activate_sets_fields() {
        let mut c = Contingency::new();
        c.activate(
            ContingencyType::SvcAn,
            "A valid reason for contingency mode activation",
        )
        .unwrap();
        assert_eq!(c.contingency_type, Some(ContingencyType::SvcAn));
        assert_eq!(c.emission_type(), 6);
        assert!(c.is_active());
        assert!(c.reason.is_some());
        assert!(c.activated_at.is_some());
    }

    #[test]
    fn activate_svc_rs() {
        let mut c = Contingency::new();
        c.activate(
            ContingencyType::SvcRs,
            "A valid reason for contingency mode activation",
        )
        .unwrap();
        assert_eq!(c.emission_type(), 7);
        assert_eq!(c.emission_type_enum(), EmissionType::SvcRs);
    }

    #[test]
    fn activate_offline() {
        let mut c = Contingency::new();
        c.activate(
            ContingencyType::Offline,
            "A valid reason for contingency mode activation",
        )
        .unwrap();
        assert_eq!(c.emission_type(), 9);
        assert_eq!(c.emission_type_enum(), EmissionType::Offline);
    }

    #[test]
    fn activate_epec() {
        let mut c = Contingency::new();
        c.activate(
            ContingencyType::Epec,
            "A valid reason for contingency mode activation",
        )
        .unwrap();
        assert_eq!(c.emission_type(), 4);
        assert_eq!(c.emission_type_enum(), EmissionType::Epec);
    }

    #[test]
    fn activate_fs_da() {
        let mut c = Contingency::new();
        c.activate(
            ContingencyType::FsDa,
            "A valid reason for contingency mode activation",
        )
        .unwrap();
        assert_eq!(c.emission_type(), 5);
        assert_eq!(c.emission_type_enum(), EmissionType::FsDa);
    }

    #[test]
    fn activate_fs_ia() {
        let mut c = Contingency::new();
        c.activate(
            ContingencyType::FsIa,
            "A valid reason for contingency mode activation",
        )
        .unwrap();
        assert_eq!(c.emission_type(), 2);
        assert_eq!(c.emission_type_enum(), EmissionType::FsIa);
    }

    #[test]
    fn activate_rejects_short_reason() {
        let mut c = Contingency::new();
        let result = c.activate(ContingencyType::SvcAn, "Short");
        assert!(result.is_err());
    }

    #[test]
    fn activate_rejects_long_reason() {
        let mut c = Contingency::new();
        let motive = "A".repeat(257);
        let result = c.activate(ContingencyType::SvcAn, &motive);
        assert!(result.is_err());
    }

    #[test]
    fn deactivate_clears_state() {
        let mut c = Contingency::new();
        c.activate(
            ContingencyType::SvcAn,
            "A valid reason for contingency mode activation",
        )
        .unwrap();
        c.deactivate();
        assert!(c.contingency_type.is_none());
        assert!(!c.is_active());
        assert_eq!(c.emission_type(), 1);
        assert_eq!(c.emission_type_enum(), EmissionType::Normal);
    }

    #[test]
    fn load_from_json() {
        let json =
            r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"SVCAN","tpEmis":6}"#;
        let c = Contingency::load(json).unwrap();
        assert_eq!(c.contingency_type, Some(ContingencyType::SvcAn));
        assert_eq!(c.emission_type(), 6);
        assert_eq!(c.reason.as_deref(), Some("Testes Unitarios"));
        assert!(c.is_active());
    }

    #[test]
    fn load_svc_rs_from_json() {
        let json =
            r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"SVCRS","tpEmis":7}"#;
        let c = Contingency::load(json).unwrap();
        assert_eq!(c.contingency_type, Some(ContingencyType::SvcRs));
        assert_eq!(c.emission_type(), 7);
    }

    #[test]
    fn load_epec_from_json() {
        let json =
            r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"EPEC","tpEmis":4}"#;
        let c = Contingency::load(json).unwrap();
        assert_eq!(c.contingency_type, Some(ContingencyType::Epec));
        assert_eq!(c.emission_type(), 4);
    }

    #[test]
    fn load_fs_da_from_json() {
        let json =
            r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"FSDA","tpEmis":5}"#;
        let c = Contingency::load(json).unwrap();
        assert_eq!(c.contingency_type, Some(ContingencyType::FsDa));
        assert_eq!(c.emission_type(), 5);
    }

    #[test]
    fn load_fs_ia_from_json() {
        let json =
            r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"FSIA","tpEmis":2}"#;
        let c = Contingency::load(json).unwrap();
        assert_eq!(c.contingency_type, Some(ContingencyType::FsIa));
        assert_eq!(c.emission_type(), 2);
    }

    #[test]
    fn load_offline_from_json() {
        let json =
            r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"OFFLINE","tpEmis":9}"#;
        let c = Contingency::load(json).unwrap();
        assert_eq!(c.contingency_type, Some(ContingencyType::Offline));
        assert_eq!(c.emission_type(), 9);
    }

    #[test]
    fn load_deactivated_from_json() {
        let json = r#"{"motive":"","timestamp":0,"type":"","tpEmis":1}"#;
        let c = Contingency::load(json).unwrap();
        assert!(c.contingency_type.is_none());
        assert!(!c.is_active());
        assert_eq!(c.emission_type(), 1);
    }

    #[test]
    fn load_rejects_unknown_type() {
        let json =
            r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"UNKNOWN","tpEmis":1}"#;
        let result = Contingency::load(json);
        assert!(result.is_err());
    }

    #[test]
    fn to_json_activated() {
        let json =
            r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"SVCAN","tpEmis":6}"#;
        let c = Contingency::load(json).unwrap();
        assert_eq!(c.to_json(), json);
    }

    #[test]
    fn to_json_deactivated() {
        let c = Contingency::new();
        assert_eq!(
            c.to_json(),
            r#"{"motive":"","timestamp":0,"type":"","tpEmis":1}"#
        );
    }

    #[test]
    fn to_json_roundtrip() {
        let json =
            r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"SVCRS","tpEmis":7}"#;
        let c = Contingency::load(json).unwrap();
        let output = c.to_json();
        assert_eq!(output, json);
        // Load again and verify
        let c2 = Contingency::load(&output).unwrap();
        assert_eq!(c2.contingency_type, c.contingency_type);
        assert_eq!(c2.reason, c.reason);
        assert_eq!(c2.timestamp, c.timestamp);
    }

    #[test]
    fn deactivate_produces_correct_json() {
        let json =
            r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"SVCAN","tpEmis":6}"#;
        let mut c = Contingency::load(json).unwrap();
        c.deactivate();
        assert_eq!(
            c.to_json(),
            r#"{"motive":"","timestamp":0,"type":"","tpEmis":1}"#
        );
    }

    #[test]
    fn display_matches_to_json() {
        let json =
            r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"SVCAN","tpEmis":6}"#;
        let c = Contingency::load(json).unwrap();
        assert_eq!(format!("{c}"), c.to_json());
    }

    #[test]
    fn extract_json_string_works() {
        let json = r#"{"motive":"hello world","type":"SVCAN"}"#;
        assert_eq!(
            extract_json_string(json, "motive"),
            Some("hello world".to_string())
        );
        assert_eq!(extract_json_string(json, "type"), Some("SVCAN".to_string()));
    }

    #[test]
    fn extract_json_number_works() {
        let json = r#"{"timestamp":1480700623,"tpEmis":6}"#;
        assert_eq!(extract_json_number(json, "timestamp"), Some(1480700623));
        assert_eq!(extract_json_number(json, "tpEmis"), Some(6));
    }

    #[test]
    fn format_timestamp_with_offset_formats_correctly() {
        // 1480700623 = 2016-12-02T17:43:43Z = 2016-12-02T14:43:43-03:00
        let result = format_timestamp_with_offset(1480700623, "-03:00");
        assert_eq!(result, "2016-12-02T14:43:43-03:00");
    }

    #[test]
    fn contingency_for_state_sp() {
        assert_eq!(contingency_for_state("SP").as_str(), "svc-an");
    }

    #[test]
    fn contingency_for_state_am() {
        assert_eq!(contingency_for_state("AM").as_str(), "svc-rs");
    }

    #[test]
    fn try_contingency_for_state_valid() {
        assert_eq!(
            try_contingency_for_state("SP").unwrap(),
            ContingencyType::SvcAn
        );
        assert_eq!(
            try_contingency_for_state("AM").unwrap(),
            ContingencyType::SvcRs
        );
    }

    #[test]
    fn try_contingency_for_state_invalid() {
        assert!(try_contingency_for_state("XX").is_err());
    }

    #[test]
    fn check_web_service_nfe_svc_an_ok() {
        let mut c = Contingency::new();
        c.activate(
            ContingencyType::SvcAn,
            "A valid reason for contingency mode activation",
        )
        .unwrap();
        assert!(c.check_web_service_availability(InvoiceModel::Nfe).is_ok());
    }

    #[test]
    fn check_web_service_nfe_svc_rs_ok() {
        let mut c = Contingency::new();
        c.activate(
            ContingencyType::SvcRs,
            "A valid reason for contingency mode activation",
        )
        .unwrap();
        assert!(c.check_web_service_availability(InvoiceModel::Nfe).is_ok());
    }

    #[test]
    fn check_web_service_nfce_svc_fails() {
        let mut c = Contingency::new();
        c.activate(
            ContingencyType::SvcAn,
            "A valid reason for contingency mode activation",
        )
        .unwrap();
        assert!(
            c.check_web_service_availability(InvoiceModel::Nfce)
                .is_err()
        );
    }

    #[test]
    fn check_web_service_epec_no_webservice() {
        let mut c = Contingency::new();
        c.activate(
            ContingencyType::Epec,
            "A valid reason for contingency mode activation",
        )
        .unwrap();
        let err = c
            .check_web_service_availability(InvoiceModel::Nfe)
            .unwrap_err();
        assert!(err.to_string().contains("EPEC"));
    }

    #[test]
    fn check_web_service_normal_mode_ok() {
        let c = Contingency::new();
        assert!(c.check_web_service_availability(InvoiceModel::Nfe).is_ok());
        assert!(c.check_web_service_availability(InvoiceModel::Nfce).is_ok());
    }

    #[test]
    fn contingency_type_display() {
        assert_eq!(format!("{}", ContingencyType::SvcAn), "SVCAN");
        assert_eq!(format!("{}", ContingencyType::SvcRs), "SVCRS");
        assert_eq!(format!("{}", ContingencyType::Epec), "EPEC");
        assert_eq!(format!("{}", ContingencyType::FsDa), "FSDA");
        assert_eq!(format!("{}", ContingencyType::FsIa), "FSIA");
        assert_eq!(format!("{}", ContingencyType::Offline), "OFFLINE");
    }

    #[test]
    fn contingency_type_from_str() {
        assert_eq!(
            "SVCAN".parse::<ContingencyType>().unwrap(),
            ContingencyType::SvcAn
        );
        assert_eq!(
            "SVC-AN".parse::<ContingencyType>().unwrap(),
            ContingencyType::SvcAn
        );
        assert_eq!(
            "SVCRS".parse::<ContingencyType>().unwrap(),
            ContingencyType::SvcRs
        );
        assert_eq!(
            "EPEC".parse::<ContingencyType>().unwrap(),
            ContingencyType::Epec
        );
        assert_eq!(
            "FSDA".parse::<ContingencyType>().unwrap(),
            ContingencyType::FsDa
        );
        assert_eq!(
            "FSIA".parse::<ContingencyType>().unwrap(),
            ContingencyType::FsIa
        );
        assert_eq!(
            "OFFLINE".parse::<ContingencyType>().unwrap(),
            ContingencyType::Offline
        );
        assert!("UNKNOWN".parse::<ContingencyType>().is_err());
    }

    #[test]
    fn contingency_type_from_tp_emis() {
        assert_eq!(
            ContingencyType::from_tp_emis(2),
            Some(ContingencyType::FsIa)
        );
        assert_eq!(
            ContingencyType::from_tp_emis(4),
            Some(ContingencyType::Epec)
        );
        assert_eq!(
            ContingencyType::from_tp_emis(5),
            Some(ContingencyType::FsDa)
        );
        assert_eq!(
            ContingencyType::from_tp_emis(6),
            Some(ContingencyType::SvcAn)
        );
        assert_eq!(
            ContingencyType::from_tp_emis(7),
            Some(ContingencyType::SvcRs)
        );
        assert_eq!(
            ContingencyType::from_tp_emis(9),
            Some(ContingencyType::Offline)
        );
        assert_eq!(ContingencyType::from_tp_emis(1), None);
        assert_eq!(ContingencyType::from_tp_emis(0), None);
        assert_eq!(ContingencyType::from_tp_emis(3), None);
    }

    #[test]
    fn escape_json_string_basic() {
        assert_eq!(escape_json_string("hello"), "hello");
        assert_eq!(escape_json_string(r#"say "hi""#), r#"say \"hi\""#);
        assert_eq!(escape_json_string("a\\b"), "a\\\\b");
    }

    #[test]
    fn all_27_states_have_mapping() {
        let states = [
            "AC", "AL", "AM", "AP", "BA", "CE", "DF", "ES", "GO", "MA", "MG", "MS", "MT", "PA",
            "PB", "PE", "PI", "PR", "RJ", "RN", "RO", "RR", "RS", "SC", "SE", "SP", "TO",
        ];
        for uf in states {
            let ct = contingency_for_state(uf);
            assert!(
                ct == ContingencyType::SvcAn || ct == ContingencyType::SvcRs,
                "State {uf} should map to SVC-AN or SVC-RS"
            );
        }
    }
}
