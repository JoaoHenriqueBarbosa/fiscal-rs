// ── Enums ────────────────────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};

/// NF-e model code: 55 = NF-e (business-to-business), 65 = NFC-e (consumer).
///
/// The value maps directly to the `<mod>` element inside `<ide>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub enum InvoiceModel {
    /// Model 55 — NF-e for business operations and B2B transactions.
    Nfe = 55,
    /// Model 65 — NFC-e for consumer-facing retail sales (Nota Fiscal de Consumidor Eletrônica).
    Nfce = 65,
}

impl InvoiceModel {
    /// Returns the numeric string representation (`"55"` or `"65"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Nfe => "55",
            Self::Nfce => "65",
        }
    }
}

impl std::fmt::Display for InvoiceModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

/// SEFAZ submission environment: production (`tpAmb=1`) or homologation (`tpAmb=2`).
///
/// Use [`Homologation`](SefazEnvironment::Homologation) during development and
/// testing; switch to [`Production`](SefazEnvironment::Production) only when
/// issuing real fiscal documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub enum SefazEnvironment {
    /// Production environment — documents have legal fiscal validity.
    Production = 1,
    /// Homologation environment — for testing only; documents have no fiscal validity.
    Homologation = 2,
}

impl SefazEnvironment {
    /// Returns the numeric string representation (`"1"` or `"2"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Production => "1",
            Self::Homologation => "2",
        }
    }
}

/// NF-e schema version selector.
///
/// Controls which XML schema layout the builder emits:
///
/// - [`PL009`](SchemaVersion::PL009) — standard NF-e schema (PL_009_V4), without tax-reform
///   tags. This is the default for backward compatibility.
/// - [`PL010`](SchemaVersion::PL010) — tax-reform schema (PL_010+), adds IBS, CBS, IS,
///   `gCompraGov`, `gPagAntecipado`, `agropecuario`, and per-item `vItem` tags.
///
/// When [`PL009`](SchemaVersion::PL009) is selected the builder silently omits
/// all PL_010-exclusive tags even if data is provided, matching the behaviour
/// of the PHP `sped-nfe` `$schema` property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub enum SchemaVersion {
    /// PL_009_V4 — standard schema without tax-reform tags (default).
    #[default]
    PL009,
    /// PL_010 — tax-reform schema with IBS/CBS, IS, gCompraGov, gPagAntecipado, agropecuario.
    PL010,
}

impl SchemaVersion {
    /// Returns `true` when the schema is PL_010 or later.
    pub fn is_pl010(&self) -> bool {
        matches!(self, Self::PL010)
    }
}

/// NF-e emission type (`tpEmis`) — normal or one of the contingency modes.
///
/// Values map directly to the `<tpEmis>` element in the `<ide>` group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub enum EmissionType {
    /// `1` — Normal online emission via the primary SEFAZ authorizer.
    Normal = 1,
    /// `2` — FS-IA contingency (Formulário de Segurança — Impressor Autônomo).
    FsIa = 2,
    /// `4` — EPEC contingency (Evento Prévio de Emissão em Contingência).
    Epec = 4,
    /// `5` — FS-DA contingency (Formulário de Segurança — Documento Auxiliar).
    FsDa = 5,
    /// `6` — SVC-AN contingency (Sefaz Virtual do Ambiente Nacional).
    SvcAn = 6,
    /// `7` — SVC-RS contingency (Sefaz Virtual do Rio Grande do Sul).
    SvcRs = 7,
    /// `9` — Offline contingency (NF-e or NFC-e issued without network access).
    Offline = 9,
}

impl EmissionType {
    /// Returns the numeric string representation (e.g. `"1"`, `"4"`, `"6"`, `"7"`, `"9"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "1",
            Self::FsIa => "2",
            Self::Epec => "4",
            Self::FsDa => "5",
            Self::SvcAn => "6",
            Self::SvcRs => "7",
            Self::Offline => "9",
        }
    }
}

/// Method used to calculate the automatic NF-e totals (`vNF` and `vItem`).
///
/// Mirrors the PHP `sped-nfe` constants `METHOD_CALCULATION_V1` and
/// `METHOD_CALCULATION_V2`.
///
/// - **V1** — calculates from pre-accumulated item values (struct fields).
/// - **V2** — calculates from the already-built XML tags (re-parsing).
///
/// In the current Rust implementation both methods produce the same result
/// because values are accumulated from the same source.  The enum exists for
/// API parity with PHP and to allow future divergence.
///
/// The default is [`V2`](CalculationMethod::V2), matching PHP's default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub enum CalculationMethod {
    /// Calculate totals from accumulated struct values.
    V1 = 1,
    /// Calculate totals from built XML tags (default).
    #[default]
    V2 = 2,
}

/// Tax regime code (`CRT`) identifying the issuer's taxation framework.
///
/// Determines which ICMS CST/CSOSN codes are valid for the issuer and
/// maps to the `<CRT>` element inside `<emit>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub enum TaxRegime {
    /// `CRT=1` — Simples Nacional (small businesses, unified tax collection).
    SimplesNacional = 1,
    /// `CRT=2` — Simples Nacional with ICMS excess (revenue above Simples threshold).
    SimplesExcess = 2,
    /// `CRT=3` — Normal regime (Lucro Real or Lucro Presumido).
    Normal = 3,
}

/// Contingency type used when the primary SEFAZ authorizer is unavailable.
///
/// Each Brazilian state is pre-assigned to either SVC-AN or SVC-RS; see
/// [`crate::contingency::contingency_for_state`] for the mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub enum ContingencyType {
    /// SVC-AN — Sefaz Virtual do Ambiente Nacional (federal fallback).
    SvcAn,
    /// SVC-RS — Sefaz Virtual do Rio Grande do Sul (southern states fallback).
    SvcRs,
    /// EPEC — Evento Prévio de Emissão em Contingência (tpEmis=4).
    Epec,
    /// FS-DA — Formulário de Segurança — Documento Auxiliar (tpEmis=5).
    FsDa,
    /// FS-IA — Formulário de Segurança — Impressor Autônomo (tpEmis=2).
    FsIa,
    /// Offline — document issued without any network access to SEFAZ (tpEmis=9).
    Offline,
}

impl ContingencyType {
    /// Returns the kebab-case string identifier (e.g. `"svc-an"`, `"svc-rs"`, `"offline"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SvcAn => "svc-an",
            Self::SvcRs => "svc-rs",
            Self::Epec => "epec",
            Self::FsDa => "fs-da",
            Self::FsIa => "fs-ia",
            Self::Offline => "offline",
        }
    }

    /// Returns the uppercase type string used in PHP JSON serialization
    /// (e.g. `"SVCAN"`, `"SVCRS"`, `"EPEC"`, `"FSDA"`, `"FSIA"`).
    pub fn to_type_str(&self) -> &'static str {
        match self {
            Self::SvcAn => "SVCAN",
            Self::SvcRs => "SVCRS",
            Self::Epec => "EPEC",
            Self::FsDa => "FSDA",
            Self::FsIa => "FSIA",
            Self::Offline => "OFFLINE",
        }
    }

    /// Returns the emission type code (`tpEmis`) for this contingency type.
    pub fn tp_emis(&self) -> u8 {
        match self {
            Self::SvcAn => 6,
            Self::SvcRs => 7,
            Self::Epec => 4,
            Self::FsDa => 5,
            Self::FsIa => 2,
            Self::Offline => 9,
        }
    }

    /// Parse a contingency type from its uppercase type string.
    ///
    /// Accepts all common representations: `SVCAN`, `SVC-AN`, `svc-an`,
    /// `SVCRS`, `SVC-RS`, `svc-rs`, `EPEC`, `epec`, `FSDA`, `FS-DA`,
    /// `fs-da`, `FSIA`, `FS-IA`, `fs-ia`, `OFFLINE`, `offline`.
    ///
    /// Returns `None` for empty strings (meaning deactivated).
    pub fn from_type_str(s: &str) -> Option<Self> {
        match s {
            "SVCAN" | "SVC-AN" | "svc-an" => Some(Self::SvcAn),
            "SVCRS" | "SVC-RS" | "svc-rs" => Some(Self::SvcRs),
            "EPEC" | "epec" => Some(Self::Epec),
            "FSDA" | "FS-DA" | "fs-da" => Some(Self::FsDa),
            "FSIA" | "FS-IA" | "fs-ia" => Some(Self::FsIa),
            "OFFLINE" | "offline" => Some(Self::Offline),
            "" => None,
            _ => None,
        }
    }

    /// Create a contingency type from its `tpEmis` code.
    ///
    /// Returns `None` for `1` (normal emission) or unknown codes.
    pub fn from_tp_emis(tp_emis: u8) -> Option<Self> {
        match tp_emis {
            2 => Some(Self::FsIa),
            4 => Some(Self::Epec),
            5 => Some(Self::FsDa),
            6 => Some(Self::SvcAn),
            7 => Some(Self::SvcRs),
            9 => Some(Self::Offline),
            _ => None,
        }
    }
}

impl core::fmt::Display for ContingencyType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.to_type_str())
    }
}

impl core::str::FromStr for ContingencyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_type_str(s).ok_or_else(|| format!("Unrecognized contingency type: {s}"))
    }
}

/// NFC-e QR code generation version.
///
/// Version 2 (`V200`) is the current standard and requires a CSC token.
/// Version 3 (`V300`, introduced in NT 2025.001) drops the CSC requirement
/// for online mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub enum QrCodeVersion {
    /// Version 2 (`qrCodType=2`) — requires CSC token and CSC ID for SHA-1 HMAC.
    V200 = 200,
    /// Version 3 (`qrCodType=3`, NT 2025.001) — no CSC needed for online emission.
    V300 = 300,
}
