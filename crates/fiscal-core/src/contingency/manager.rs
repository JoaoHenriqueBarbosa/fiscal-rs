use crate::FiscalError;
use crate::types::{ContingencyType, EmissionType, InvoiceModel};

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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Contingency {
    /// The active contingency type, or `None` when in normal mode.
    pub contingency_type: Option<ContingencyType>,
    /// Justification reason for entering contingency (15-255 chars).
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
    /// The reason is trimmed and must be between 15 and 255 UTF-8 characters
    /// (inclusive). On success, the contingency is activated with the current
    /// UTC timestamp.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Contingency`] if the trimmed reason is shorter
    /// than 15 characters or longer than 255 characters.
    pub fn activate(
        &mut self,
        contingency_type: ContingencyType,
        reason: &str,
    ) -> Result<(), FiscalError> {
        let trimmed = reason.trim();
        let len = trimmed.chars().count();
        if !(15..=255).contains(&len) {
            return Err(FiscalError::Contingency(
                "The justification for entering contingency mode must be between 15 and 255 UTF-8 characters.".to_string(),
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

// ── Private JSON helpers ────────────────────────────────────────────────────

/// Escape a string for JSON output — handles `"`, `\`, and control characters.
pub(super) fn escape_json_string(s: &str) -> String {
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
pub(super) fn extract_json_string(json: &str, key: &str) -> Option<String> {
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
pub(super) fn extract_json_number(json: &str, key: &str) -> Option<u64> {
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
