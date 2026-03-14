use napi_derive::napi;

use fiscal_core::contingency::Contingency as RustContingency;
use fiscal_core::types::ContingencyType;

fn to_napi(e: impl std::fmt::Display) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}

// ── Contingency class ───────────────────────────────────────────────────────

/// Contingency manager for NF-e emission.
///
/// Manages activation/deactivation of contingency mode when the primary
/// SEFAZ authorizer is unavailable.
#[napi]
pub struct ContingencyManager {
    inner: RustContingency,
}

#[napi]
impl ContingencyManager {
    /// Create a new contingency manager in normal mode (no active contingency).
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RustContingency::new(),
        }
    }

    /// Load contingency state from a JSON string.
    ///
    /// Expected format: `{"motive":"reason","timestamp":1480700623,"type":"SVCAN","tpEmis":6}`
    #[napi(factory)]
    pub fn load(json: String) -> napi::Result<Self> {
        let inner = RustContingency::load(&json).map_err(to_napi)?;
        Ok(Self { inner })
    }

    /// Returns `true` when a contingency mode is currently active.
    #[napi(getter)]
    pub fn is_active(&self) -> bool {
        self.inner.is_active()
    }

    /// Activate contingency mode.
    ///
    /// `contingencyType`: one of "svcAn", "svcRs", "epec", "fsDa", "fsIa", "offline"
    /// `reason`: justification (15-255 characters)
    #[napi]
    pub fn activate(&mut self, contingency_type: String, reason: String) -> napi::Result<()> {
        let ct = parse_contingency_type(&contingency_type)?;
        self.inner.activate(ct, &reason).map_err(to_napi)
    }

    /// Deactivate contingency mode, returning to normal emission.
    #[napi]
    pub fn deactivate(&mut self) {
        self.inner.deactivate();
    }

    /// Serialize the contingency state to a JSON string.
    #[napi]
    pub fn to_json(&self) -> String {
        self.inner.to_json()
    }

    /// Get the emission type code for the current state.
    /// Returns 1 (normal) when no contingency is active.
    #[napi(getter)]
    pub fn emission_type(&self) -> u32 {
        self.inner.emission_type() as u32
    }

    /// Check if the current contingency mode has a dedicated SEFAZ web service.
    ///
    /// `model`: "nfe" or "nfce"
    #[napi]
    pub fn check_web_service_availability(&self, model: String) -> napi::Result<()> {
        let m = match model.to_lowercase().as_str() {
            "nfe" | "55" => fiscal_core::types::InvoiceModel::Nfe,
            "nfce" | "65" => fiscal_core::types::InvoiceModel::Nfce,
            _ => {
                return Err(napi::Error::from_reason(format!(
                    "Invalid model: \"{model}\". Expected \"nfe\" or \"nfce\"."
                )));
            }
        };
        self.inner
            .check_web_service_availability(m)
            .map_err(to_napi)
    }

    /// Adjust an NF-e XML for the current contingency mode.
    ///
    /// Modifies tpEmis, inserts dhCont/xJust, recalculates access key.
    /// Returns the XML unchanged if no contingency is active.
    #[napi]
    pub fn adjust_xml(&self, xml: String) -> napi::Result<String> {
        fiscal_core::contingency::adjust_nfe_contingency(&xml, &self.inner).map_err(to_napi)
    }
}

// ── Standalone functions ────────────────────────────────────────────────────

/// Get the default contingency type (SVC-AN or SVC-RS) for a Brazilian state.
///
/// Returns "svcRs" or "svcAn".
#[napi]
pub fn contingency_for_state(uf: String) -> napi::Result<String> {
    let ct = fiscal_core::contingency::try_contingency_for_state(&uf).map_err(to_napi)?;
    Ok(match ct {
        ContingencyType::SvcAn => "svcAn".to_string(),
        ContingencyType::SvcRs => "svcRs".to_string(),
        ContingencyType::Epec => "epec".to_string(),
        ContingencyType::FsDa => "fsDa".to_string(),
        ContingencyType::FsIa => "fsIa".to_string(),
        ContingencyType::Offline => "offline".to_string(),
        _ => "unknown".to_string(),
    })
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn parse_contingency_type(s: &str) -> napi::Result<ContingencyType> {
    match s.to_lowercase().as_str() {
        "svcan" | "svc-an" | "svcAn" => Ok(ContingencyType::SvcAn),
        "svcrs" | "svc-rs" | "svcRs" => Ok(ContingencyType::SvcRs),
        "epec" => Ok(ContingencyType::Epec),
        "fsda" | "fs-da" | "fsDa" => Ok(ContingencyType::FsDa),
        "fsia" | "fs-ia" | "fsIa" => Ok(ContingencyType::FsIa),
        "offline" => Ok(ContingencyType::Offline),
        _ => Err(napi::Error::from_reason(format!(
            "Invalid contingency type: \"{s}\""
        ))),
    }
}
