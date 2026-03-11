//! Public data structures for NF-e / NFC-e documents.
//!
//! This module contains all the strongly-typed structs and enums used to
//! represent the data required to build a Brazilian electronic invoice (NF-e
//! model 55 or NFC-e model 65).  Every struct follows the builder pattern:
//! required fields are passed to `new(...)`, and optional fields are set via
//! chainable setter methods.
//!
//! # Key types
//!
//! | Type | Purpose |
//! |------|---------|
//! | [`IssuerData`] | Company/issuer identification and address |
//! | [`RecipientData`] | Buyer/recipient identification (optional for NFC-e under R$200) |
//! | [`InvoiceItemData`] | Line-item with product data and all applicable taxes |
//! | [`PaymentData`] | Payment method and amount |
//! | [`SefazEnvironment`] | Production vs. homologation environment selector |
//! | [`InvoiceModel`] | NF-e (55) vs. NFC-e (65) |
//! | [`EmissionType`] | Normal vs. contingency emission type |
//! | [`TaxRegime`] | Simples Nacional / Simples Excess / Normal regime |

use chrono::{DateTime, FixedOffset, NaiveDate};

use crate::newtypes::{Cents, IbgeCode, Rate, Rate4};

// ── Enums ────────────────────────────────────────────────────────────────────

/// NF-e model code: 55 = NF-e (business-to-business), 65 = NFC-e (consumer).
///
/// The value maps directly to the `<mod>` element inside `<ide>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// NF-e emission type (`tpEmis`) — normal or one of the contingency modes.
///
/// Values map directly to the `<tpEmis>` element in the `<ide>` group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EmissionType {
    /// `1` — Normal online emission via the primary SEFAZ authorizer.
    Normal = 1,
    /// `6` — SVC-AN contingency (Sefaz Virtual do Ambiente Nacional).
    SvcAn = 6,
    /// `7` — SVC-RS contingency (Sefaz Virtual do Rio Grande do Sul).
    SvcRs = 7,
    /// `9` — Offline contingency (NF-e or NFC-e issued without network access).
    Offline = 9,
}

impl EmissionType {
    /// Returns the numeric string representation (e.g. `"1"`, `"6"`, `"7"`, `"9"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "1",
            Self::SvcAn => "6",
            Self::SvcRs => "7",
            Self::Offline => "9",
        }
    }
}

/// Tax regime code (`CRT`) identifying the issuer's taxation framework.
///
/// Determines which ICMS CST/CSOSN codes are valid for the issuer and
/// maps to the `<CRT>` element inside `<emit>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContingencyType {
    /// SVC-AN — Sefaz Virtual do Ambiente Nacional (federal fallback).
    SvcAn,
    /// SVC-RS — Sefaz Virtual do Rio Grande do Sul (southern states fallback).
    SvcRs,
    /// Offline — document issued without any network access to SEFAZ.
    Offline,
}

impl ContingencyType {
    /// Returns the kebab-case string identifier (e.g. `"svc-an"`, `"svc-rs"`, `"offline"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SvcAn => "svc-an",
            Self::SvcRs => "svc-rs",
            Self::Offline => "offline",
        }
    }
}

/// NFC-e QR code generation version.
///
/// Version 2 (`V200`) is the current standard and requires a CSC token.
/// Version 3 (`V300`, introduced in NT 2025.001) drops the CSC requirement
/// for online mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum QrCodeVersion {
    /// Version 2 (`qrCodType=2`) — requires CSC token and CSC ID for SHA-1 HMAC.
    V200 = 200,
    /// Version 3 (`qrCodType=3`, NT 2025.001) — no CSC needed for online emission.
    V300 = 300,
}

// ── Data structures ──────────────────────────────────────────────────────────

/// PKCS#12 certificate data loaded from a PFX file.
///
/// Holds the PEM-encoded private key and certificate alongside the original
/// PFX buffer and passphrase so the certificate can be reused for multiple
/// signing operations without re-parsing.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CertificateData {
    /// PEM-encoded RSA private key (PKCS#8 format).
    pub private_key: String,
    /// PEM-encoded X.509 certificate.
    pub certificate: String,
    /// Raw PKCS#12/PFX binary buffer.
    pub pfx_buffer: Vec<u8>,
    /// Passphrase used to unlock the PFX file.
    pub passphrase: String,
}

impl CertificateData {
    /// Create a new `CertificateData` with all required fields.
    pub fn new(
        private_key: impl Into<String>,
        certificate: impl Into<String>,
        pfx_buffer: Vec<u8>,
        passphrase: impl Into<String>,
    ) -> Self {
        Self {
            private_key: private_key.into(),
            certificate: certificate.into(),
            pfx_buffer,
            passphrase: passphrase.into(),
        }
    }
}

/// Human-readable X.509 certificate metadata for display purposes.
///
/// Extracted from a PKCS#12 file without exposing the private key.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CertificateInfo {
    /// Common name (CN) from the certificate's subject field.
    pub common_name: String,
    /// Date from which the certificate is valid (`notBefore`).
    pub valid_from: NaiveDate,
    /// Expiry date of the certificate (`notAfter`).
    pub valid_until: NaiveDate,
    /// Hex-encoded X.509 serial number.
    pub serial_number: String,
    /// Common name (CN) of the certificate issuer.
    pub issuer: String,
}

impl CertificateInfo {
    /// Create a new `CertificateInfo` with all required fields.
    pub fn new(
        common_name: impl Into<String>,
        valid_from: NaiveDate,
        valid_until: NaiveDate,
        serial_number: impl Into<String>,
        issuer: impl Into<String>,
    ) -> Self {
        Self {
            common_name: common_name.into(),
            valid_from,
            valid_until,
            serial_number: serial_number.into(),
            issuer: issuer.into(),
        }
    }
}

/// Input parameters for building a 44-digit NF-e / NFC-e access key.
///
/// The access key (`chave de acesso`) uniquely identifies a Brazilian electronic
/// invoice and is computed from these components using a Verhoeff-like algorithm.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct AccessKeyParams {
    /// IBGE numeric state code (e.g. `"41"` for Paraná).
    pub state_code: IbgeCode,
    /// Emission year and month in `YYMM` format (e.g. `"2503"` for March 2025).
    pub year_month: String,
    /// CNPJ or CPF of the issuer (digits only, 14 or 11 characters).
    pub tax_id: String,
    /// Invoice model: [`InvoiceModel::Nfe`] (55) or [`InvoiceModel::Nfce`] (65).
    pub model: InvoiceModel,
    /// Series number (0–999).
    pub series: u32,
    /// Invoice number (`nNF`, 1–999 999 999).
    pub number: u32,
    /// Emission type used to set `tpEmis` in the key.
    pub emission_type: EmissionType,
    /// Random numeric code (`cNF`, 8 digits) for uniqueness.
    pub numeric_code: String,
}

impl AccessKeyParams {
    /// Create a new `AccessKeyParams` with all required fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        state_code: IbgeCode,
        year_month: impl Into<String>,
        tax_id: impl Into<String>,
        model: InvoiceModel,
        series: u32,
        number: u32,
        emission_type: EmissionType,
        numeric_code: impl Into<String>,
    ) -> Self {
        Self {
            state_code,
            year_month: year_month.into(),
            tax_id: tax_id.into(),
            model,
            series,
            number,
            emission_type,
            numeric_code: numeric_code.into(),
        }
    }
}

/// Issuer (emitente) identification and address data.
///
/// Required for every NF-e / NFC-e document. Built via [`IssuerData::new`];
/// optional fields (`trade_name`, `address_complement`) are set with chainable
/// methods.
///
/// # Examples
///
/// ```
/// use fiscal_core::types::{IssuerData, TaxRegime};
/// use fiscal_core::newtypes::IbgeCode;
///
/// let issuer = IssuerData::new(
///     "12345678000199",   // CNPJ
///     "123456789",        // state tax ID
///     "Minha Empresa Ltda",
///     TaxRegime::SimplesNacional,
///     "PR",
///     IbgeCode("4106852".into()),
///     "Curitiba",
///     "Rua das Flores",
///     "100",
///     "Centro",
///     "80010-010",
/// );
/// assert_eq!(issuer.state_code, "PR");
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct IssuerData {
    /// CNPJ or CPF of the issuer (digits only).
    pub tax_id: String,
    /// State tax registration (Inscrição Estadual).
    pub state_tax_id: String,
    /// Legal company name (`xNome`).
    pub company_name: String,
    /// Trading / fantasy name (`xFant`). Optional.
    pub trade_name: Option<String>,
    /// Tax regime code (`CRT`).
    pub tax_regime: TaxRegime,
    /// Two-letter state abbreviation (UF), e.g. `"PR"`.
    pub state_code: String,
    /// IBGE city code, e.g. `"4106852"` for Curitiba.
    pub city_code: IbgeCode,
    /// City name (`xMun`).
    pub city_name: String,
    /// Street name (`xLgr`).
    pub street: String,
    /// Street / building number (`nro`).
    pub street_number: String,
    /// Neighbourhood / district (`xBairro`).
    pub district: String,
    /// Brazilian postal code — 8 digits, no hyphen (`CEP`).
    pub zip_code: String,
    /// Address complement such as suite or floor (`xCpl`). Optional.
    pub address_complement: Option<String>,
}

impl IssuerData {
    /// Create a new `IssuerData` with all required fields.
    /// Optional fields (`trade_name`, `address_complement`) default to `None`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tax_id: impl Into<String>,
        state_tax_id: impl Into<String>,
        company_name: impl Into<String>,
        tax_regime: TaxRegime,
        state_code: impl Into<String>,
        city_code: IbgeCode,
        city_name: impl Into<String>,
        street: impl Into<String>,
        street_number: impl Into<String>,
        district: impl Into<String>,
        zip_code: impl Into<String>,
    ) -> Self {
        Self {
            tax_id: tax_id.into(),
            state_tax_id: state_tax_id.into(),
            company_name: company_name.into(),
            trade_name: None,
            tax_regime,
            state_code: state_code.into(),
            city_code,
            city_name: city_name.into(),
            street: street.into(),
            street_number: street_number.into(),
            district: district.into(),
            zip_code: zip_code.into(),
            address_complement: None,
        }
    }

    /// Set the trade name.
    pub fn trade_name(mut self, name: impl Into<String>) -> Self {
        self.trade_name = Some(name.into());
        self
    }

    /// Set the address complement.
    pub fn address_complement(mut self, complement: impl Into<String>) -> Self {
        self.address_complement = Some(complement.into());
        self
    }
}

/// Recipient (destinatário) identification and optional address data.
///
/// For NFC-e issued to anonymous consumers under R$200 the recipient may be
/// omitted entirely. For other documents, at minimum `tax_id` and `name` are
/// required; the full address is optional.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct RecipientData {
    /// CNPJ, CPF, or foreign ID of the recipient (digits only).
    pub tax_id: String,
    /// Recipient legal or individual name (`xNome`).
    pub name: String,
    /// Two-letter state abbreviation (UF), e.g. `"PR"`.
    /// `None` when the recipient's state is unknown or absent.
    pub state_code: Option<String>,
    /// State tax registration (IE) of the recipient.
    pub state_tax_id: Option<String>,
    /// Street name (`xLgr`).
    pub street: Option<String>,
    /// Street / building number (`nro`).
    pub street_number: Option<String>,
    /// Neighbourhood / district (`xBairro`).
    pub district: Option<String>,
    /// IBGE city code, e.g. `"4106852"` for Curitiba.
    pub city_code: Option<IbgeCode>,
    /// City name (`xMun`).
    pub city_name: Option<String>,
    /// Brazilian postal code — 8 digits, no hyphen (`CEP`).
    pub zip_code: Option<String>,
    /// Address complement (`xCpl`).
    pub complement: Option<String>,
}

impl RecipientData {
    /// Create a new `RecipientData` with the two required fields.
    /// All optional fields default to `None`.
    pub fn new(tax_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            tax_id: tax_id.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set the state code (UF).
    pub fn state_code(mut self, code: impl Into<String>) -> Self {
        self.state_code = Some(code.into());
        self
    }

    /// Set the state tax ID (IE).
    pub fn state_tax_id(mut self, id: impl Into<String>) -> Self {
        self.state_tax_id = Some(id.into());
        self
    }

    /// Set the street.
    pub fn street(mut self, street: impl Into<String>) -> Self {
        self.street = Some(street.into());
        self
    }

    /// Set the street number.
    pub fn street_number(mut self, number: impl Into<String>) -> Self {
        self.street_number = Some(number.into());
        self
    }

    /// Set the district.
    pub fn district(mut self, district: impl Into<String>) -> Self {
        self.district = Some(district.into());
        self
    }

    /// Set the city code (IBGE).
    pub fn city_code(mut self, code: IbgeCode) -> Self {
        self.city_code = Some(code);
        self
    }

    /// Set the city name.
    pub fn city_name(mut self, name: impl Into<String>) -> Self {
        self.city_name = Some(name.into());
        self
    }

    /// Set the zip code.
    pub fn zip_code(mut self, zip: impl Into<String>) -> Self {
        self.zip_code = Some(zip.into());
        self
    }

    /// Set the address complement.
    pub fn complement(mut self, complement: impl Into<String>) -> Self {
        self.complement = Some(complement.into());
        self
    }
}

/// Contingency activation data embedded in an NF-e when the primary SEFAZ
/// authorizer is unavailable.
///
/// When present, the XML builder inserts `<dhCont>` and `<xJust>` into `<ide>`.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ContingencyData {
    /// Which contingency mode is active.
    pub contingency_type: ContingencyType,
    /// Human-readable justification for entering contingency (15–256 chars).
    pub reason: String,
    /// Timestamp when contingency mode was activated.
    pub at: DateTime<FixedOffset>,
}

impl ContingencyData {
    /// Create a new `ContingencyData` with all required fields.
    pub fn new(
        contingency_type: ContingencyType,
        reason: impl Into<String>,
        at: DateTime<FixedOffset>,
    ) -> Self {
        Self {
            contingency_type,
            reason: reason.into(),
            at,
        }
    }
}

/// Payment method and amount for a single payment entry (`<detPag>`).
///
/// Use the payment type codes from [`crate::constants::payment_types`] for
/// the `method` field (e.g. `"01"` for cash, `"17"` for Pix).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PaymentData {
    /// Payment type code (`tPag`), e.g. `"01"` (cash) or `"03"` (credit card).
    pub method: String,
    /// Amount paid in this payment entry.
    pub amount: Cents,
}

impl PaymentData {
    /// Create a new `PaymentData`.
    pub fn new(method: impl Into<String>, amount: Cents) -> Self {
        Self {
            method: method.into(),
            amount,
        }
    }
}

/// Optional credit/debit card details attached to a payment entry (`<card>`).
///
/// All fields are optional; set only the ones available from the payment terminal.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct PaymentCardDetail {
    /// Integration type code (`tpIntegra`): `"1"` (integrated) or `"2"` (non-integrated).
    pub integ_type: Option<String>,
    /// CNPJ of the card acquirer (`CNPJ`).
    pub card_tax_id: Option<String>,
    /// Card brand code (`tBand`), e.g. `"01"` (Visa), `"02"` (Mastercard).
    pub card_brand: Option<String>,
    /// Authorization code from the acquirer (`cAut`).
    pub auth_code: Option<String>,
}

impl PaymentCardDetail {
    /// Create a new `PaymentCardDetail` with all fields set to `None`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the integration type.
    pub fn integ_type(mut self, v: impl Into<String>) -> Self {
        self.integ_type = Some(v.into());
        self
    }

    /// Set the card tax ID (CNPJ).
    pub fn card_tax_id(mut self, v: impl Into<String>) -> Self {
        self.card_tax_id = Some(v.into());
        self
    }

    /// Set the card brand.
    pub fn card_brand(mut self, v: impl Into<String>) -> Self {
        self.card_brand = Some(v.into());
        self
    }

    /// Set the authorization code.
    pub fn auth_code(mut self, v: impl Into<String>) -> Self {
        self.auth_code = Some(v.into());
        self
    }
}

/// Referenced fiscal document types that may appear in the `<NFref>` section.
///
/// Each variant represents a different class of referenced document.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ReferenceDoc {
    /// Reference to another NF-e by its 44-digit access key.
    Nfe {
        /// 44-digit access key of the referenced NF-e.
        access_key: String,
    },
    /// Reference to a paper NF (model 1 or 1A).
    Nf {
        /// IBGE numeric state code (e.g. `"41"` for PR).
        state_code: IbgeCode,
        /// Year and month in `YYMM` format.
        year_month: String,
        /// CNPJ of the issuer.
        tax_id: String,
        /// Document model (e.g. `"01"`).
        model: String,
        /// Series number.
        series: String,
        /// Document number.
        number: String,
    },
    /// Reference to a paper NF from a rural producer (NFP).
    Nfp {
        /// IBGE numeric state code (e.g. `"41"` for PR).
        state_code: IbgeCode,
        /// Year and month in `YYMM` format.
        year_month: String,
        /// CPF or CNPJ of the issuer.
        tax_id: String,
        /// Document model.
        model: String,
        /// Series number.
        series: String,
        /// Document number.
        number: String,
    },
    /// Reference to a CT-e by its 44-digit access key.
    Cte {
        /// 44-digit access key of the referenced CT-e.
        access_key: String,
    },
    /// Reference to an ECF fiscal receipt.
    Ecf {
        /// ECF model code.
        model: String,
        /// ECF sequential number.
        ecf_number: String,
        /// COO (Contador de Ordem de Operação) number.
        coo_number: String,
    },
}

/// Transport section (`<transp>`) data for an NF-e document.
///
/// The freight mode is required; all other fields are optional.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct TransportData {
    /// Freight modality code (`modFrete`): `"0"` (issuer) through `"9"` (no freight).
    pub freight_mode: String,
    /// Carrier identification (transportadora).
    pub carrier: Option<CarrierData>,
    /// Main transport vehicle.
    pub vehicle: Option<VehicleData>,
    /// Trailer vehicles (reboque).
    pub trailers: Option<Vec<VehicleData>>,
    /// List of transported volumes (`vol`).
    pub volumes: Option<Vec<VolumeData>>,
    /// ICMS retained on transport services (`retTransp`).
    pub retained_icms: Option<RetainedIcmsTransp>,
}

impl TransportData {
    /// Create a new `TransportData` with the required freight mode.
    pub fn new(freight_mode: impl Into<String>) -> Self {
        Self {
            freight_mode: freight_mode.into(),
            ..Default::default()
        }
    }

    /// Set the carrier data.
    pub fn carrier(mut self, carrier: CarrierData) -> Self {
        self.carrier = Some(carrier);
        self
    }

    /// Set the vehicle data.
    pub fn vehicle(mut self, vehicle: VehicleData) -> Self {
        self.vehicle = Some(vehicle);
        self
    }

    /// Set the trailers.
    pub fn trailers(mut self, trailers: Vec<VehicleData>) -> Self {
        self.trailers = Some(trailers);
        self
    }

    /// Set the volumes.
    pub fn volumes(mut self, volumes: Vec<VolumeData>) -> Self {
        self.volumes = Some(volumes);
        self
    }

    /// Set the retained ICMS on transport.
    pub fn retained_icms(mut self, retained: RetainedIcmsTransp) -> Self {
        self.retained_icms = Some(retained);
        self
    }
}

/// Carrier (transportadora) identification for freight transport.
///
/// All fields are optional to accommodate scenarios where only partial
/// carrier information is available.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct CarrierData {
    /// CNPJ or CPF of the carrier.
    pub tax_id: Option<String>,
    /// Legal name of the carrier (`xNome`).
    pub name: Option<String>,
    /// State tax registration (IE) of the carrier.
    pub state_tax_id: Option<String>,
    /// Two-letter state code (UF) of the carrier.
    pub state_code: Option<String>,
    /// Full address string of the carrier (`xEnder`).
    pub address: Option<String>,
}

impl CarrierData {
    /// Create a new empty `CarrierData`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the tax ID.
    pub fn tax_id(mut self, v: impl Into<String>) -> Self {
        self.tax_id = Some(v.into());
        self
    }

    /// Set the name.
    pub fn name(mut self, v: impl Into<String>) -> Self {
        self.name = Some(v.into());
        self
    }

    /// Set the state tax ID.
    pub fn state_tax_id(mut self, v: impl Into<String>) -> Self {
        self.state_tax_id = Some(v.into());
        self
    }

    /// Set the state code.
    pub fn state_code(mut self, v: impl Into<String>) -> Self {
        self.state_code = Some(v.into());
        self
    }

    /// Set the address.
    pub fn address(mut self, v: impl Into<String>) -> Self {
        self.address = Some(v.into());
        self
    }
}

/// Vehicle identification for transport (`veicTransp`) or trailers (`reboque`).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct VehicleData {
    /// Vehicle licence plate (`placa`).
    pub plate: String,
    /// State (UF) where the vehicle is registered.
    pub state_code: String,
    /// ANTT registration code (`RNTC`). Optional.
    pub rntc: Option<String>,
}

impl VehicleData {
    /// Create a new `VehicleData` with required fields.
    pub fn new(plate: impl Into<String>, state_code: impl Into<String>) -> Self {
        Self {
            plate: plate.into(),
            state_code: state_code.into(),
            rntc: None,
        }
    }

    /// Set the RNTC code.
    pub fn rntc(mut self, rntc: impl Into<String>) -> Self {
        self.rntc = Some(rntc.into());
        self
    }
}

/// A single transported volume (`<vol>`) with optional identification and weights.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct VolumeData {
    /// Number of volumes (`qVol`).
    pub quantity: Option<u32>,
    /// Species / type of packaging (`esp`), e.g. `"CAIXA"`.
    pub species: Option<String>,
    /// Brand on the packaging (`marca`).
    pub brand: Option<String>,
    /// Volume number / identifier (`nVol`).
    pub number: Option<String>,
    /// Net weight in kilograms (`pesoL`).
    pub net_weight: Option<f64>,
    /// Gross weight in kilograms (`pesoB`).
    pub gross_weight: Option<f64>,
    /// List of seal numbers (`lacres`).
    pub seals: Option<Vec<String>>,
}

impl VolumeData {
    /// Create a new empty `VolumeData`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the quantity.
    pub fn quantity(mut self, v: u32) -> Self {
        self.quantity = Some(v);
        self
    }
    /// Set the species.
    pub fn species(mut self, v: impl Into<String>) -> Self {
        self.species = Some(v.into());
        self
    }
    /// Set the brand.
    pub fn brand(mut self, v: impl Into<String>) -> Self {
        self.brand = Some(v.into());
        self
    }
    /// Set the number.
    pub fn number(mut self, v: impl Into<String>) -> Self {
        self.number = Some(v.into());
        self
    }
    /// Set the net weight.
    pub fn net_weight(mut self, v: f64) -> Self {
        self.net_weight = Some(v);
        self
    }
    /// Set the gross weight.
    pub fn gross_weight(mut self, v: f64) -> Self {
        self.gross_weight = Some(v);
        self
    }
    /// Set the seals.
    pub fn seals(mut self, v: Vec<String>) -> Self {
        self.seals = Some(v);
        self
    }
}

/// ICMS retained on transport services (`<retTransp>`).
///
/// Applicable when the carrier is subject to ICMS withholding.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RetainedIcmsTransp {
    /// ICMS calculation base for the retained amount (`vBCRet`).
    pub v_bc_ret: Cents,
    /// ICMS rate applied to the retained amount (`pICMSRet`).
    pub p_icms_ret: Rate,
    /// Retained ICMS value (`vICMSRet`).
    pub v_icms_ret: Cents,
    /// CFOP code applicable to the transport service.
    pub cfop: String,
    /// IBGE city code of the municipality where the tax event occurred.
    pub city_code: IbgeCode,
}

impl RetainedIcmsTransp {
    /// Create a new `RetainedIcmsTransp` with all required fields.
    pub fn new(
        v_bc_ret: Cents,
        p_icms_ret: Rate,
        v_icms_ret: Cents,
        cfop: impl Into<String>,
        city_code: IbgeCode,
    ) -> Self {
        Self {
            v_bc_ret,
            p_icms_ret,
            v_icms_ret,
            cfop: cfop.into(),
            city_code,
        }
    }
}

/// Billing section (`<cobr>`) with optional invoice header and installments.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct BillingData {
    /// Billing invoice summary (`<fat>`).
    pub invoice: Option<BillingInvoice>,
    /// Individual billing installments (`<dup>`).
    pub installments: Option<Vec<Installment>>,
}

impl BillingData {
    /// Create a new empty `BillingData`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the billing invoice header.
    pub fn invoice(mut self, inv: BillingInvoice) -> Self {
        self.invoice = Some(inv);
        self
    }

    /// Set the installments.
    pub fn installments(mut self, inst: Vec<Installment>) -> Self {
        self.installments = Some(inst);
        self
    }
}

/// Billing invoice summary (`<fat>`) with original, discount, and net values.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct BillingInvoice {
    /// Invoice / bill number (`nFat`).
    pub number: String,
    /// Original invoice value before discounts (`vOrig`).
    pub original_value: Cents,
    /// Discount amount (`vDesc`). Optional.
    pub discount_value: Option<Cents>,
    /// Net invoice value after discounts (`vLiq`).
    pub net_value: Cents,
}

impl BillingInvoice {
    /// Create a new `BillingInvoice` with required fields.
    pub fn new(number: impl Into<String>, original_value: Cents, net_value: Cents) -> Self {
        Self {
            number: number.into(),
            original_value,
            discount_value: None,
            net_value,
        }
    }

    /// Set the discount value.
    pub fn discount_value(mut self, v: Cents) -> Self {
        self.discount_value = Some(v);
        self
    }
}

/// A single billing installment (`<dup>`).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Installment {
    /// Installment number (`nDup`), e.g. `"001"`.
    pub number: String,
    /// Due date (`dVenc`) in `YYYY-MM-DD` format.
    pub due_date: String,
    /// Instalment amount (`vDup`).
    pub value: Cents,
}

impl Installment {
    /// Create a new `Installment`.
    pub fn new(number: impl Into<String>, due_date: impl Into<String>, value: Cents) -> Self {
        Self {
            number: number.into(),
            due_date: due_date.into(),
            value,
        }
    }
}

/// Address data for pickup (`retirada`) or delivery (`entrega`) locations.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct LocationData {
    /// CNPJ or CPF of the location owner.
    pub tax_id: String,
    /// Name of the location (`xNome`). Optional.
    pub name: Option<String>,
    /// Street name (`xLgr`).
    pub street: String,
    /// Street / building number (`nro`).
    pub number: String,
    /// Address complement (`xCpl`). Optional.
    pub complement: Option<String>,
    /// Neighbourhood / district (`xBairro`).
    pub district: String,
    /// IBGE city code, e.g. `"4106852"` for Curitiba.
    pub city_code: IbgeCode,
    /// City name (`xMun`).
    pub city_name: String,
    /// Two-letter state abbreviation (UF), e.g. `"PR"`.
    pub state_code: String,
    /// Postal code (`CEP`). Optional.
    pub zip_code: Option<String>,
}

impl LocationData {
    /// Create a new `LocationData` with required fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tax_id: impl Into<String>,
        street: impl Into<String>,
        number: impl Into<String>,
        district: impl Into<String>,
        city_code: IbgeCode,
        city_name: impl Into<String>,
        state_code: impl Into<String>,
    ) -> Self {
        Self {
            tax_id: tax_id.into(),
            name: None,
            street: street.into(),
            number: number.into(),
            complement: None,
            district: district.into(),
            city_code,
            city_name: city_name.into(),
            state_code: state_code.into(),
            zip_code: None,
        }
    }

    /// Set the name.
    pub fn name(mut self, v: impl Into<String>) -> Self {
        self.name = Some(v.into());
        self
    }
    /// Set the complement.
    pub fn complement(mut self, v: impl Into<String>) -> Self {
        self.complement = Some(v.into());
        self
    }
    /// Set the zip code.
    pub fn zip_code(mut self, v: impl Into<String>) -> Self {
        self.zip_code = Some(v.into());
        self
    }
}

/// Additional information section (`<infAdic>`) for freeform notes and observations.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct AdditionalInfo {
    /// Free-text note for the taxpayer (`infCpl`), printed on the DANFE.
    pub taxpayer_note: Option<String>,
    /// Note for the tax authority (`infAdFisco`), not printed on the DANFE.
    pub tax_authority_note: Option<String>,
    /// Contributor observations (`obsCont`).
    pub contributor_obs: Option<Vec<FieldText>>,
    /// Fiscal observations (`obsFisco`).
    pub fiscal_obs: Option<Vec<FieldText>>,
    /// References to administrative or judicial processes (`procRef`).
    pub process_refs: Option<Vec<ProcessRef>>,
}

impl AdditionalInfo {
    /// Create a new empty `AdditionalInfo`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the taxpayer note.
    pub fn taxpayer_note(mut self, v: impl Into<String>) -> Self {
        self.taxpayer_note = Some(v.into());
        self
    }
    /// Set the tax authority note.
    pub fn tax_authority_note(mut self, v: impl Into<String>) -> Self {
        self.tax_authority_note = Some(v.into());
        self
    }
    /// Set the contributor observations.
    pub fn contributor_obs(mut self, v: Vec<FieldText>) -> Self {
        self.contributor_obs = Some(v);
        self
    }
    /// Set the fiscal observations.
    pub fn fiscal_obs(mut self, v: Vec<FieldText>) -> Self {
        self.fiscal_obs = Some(v);
        self
    }
    /// Set the process references.
    pub fn process_refs(mut self, v: Vec<ProcessRef>) -> Self {
        self.process_refs = Some(v);
        self
    }
}

/// A field-name / text-value pair used in contributor and fiscal observations.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FieldText {
    /// Field identifier (`xCampo`), max 20 characters.
    pub field: String,
    /// Text value (`xTexto`), max 60 characters.
    pub text: String,
}

impl FieldText {
    /// Create a new `FieldText`.
    pub fn new(field: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            text: text.into(),
        }
    }
}

/// Reference to an administrative or judicial process (`<procRef>`).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ProcessRef {
    /// Process number (`nProc`).
    pub number: String,
    /// Process origin code (`indProc`): `"0"` (SEFAZ) through `"9"` (others).
    pub origin: String,
}

impl ProcessRef {
    /// Create a new `ProcessRef`.
    pub fn new(number: impl Into<String>, origin: impl Into<String>) -> Self {
        Self {
            number: number.into(),
            origin: origin.into(),
        }
    }
}

/// Intermediary entity data (`<infIntermed>`) for marketplace transactions.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct IntermediaryData {
    /// CNPJ of the intermediary platform.
    pub tax_id: String,
    /// Platform's internal identifier for the transaction (`idCadIntTran`). Optional.
    pub id_cad_int_tran: Option<String>,
}

impl IntermediaryData {
    /// Create a new `IntermediaryData`.
    pub fn new(tax_id: impl Into<String>) -> Self {
        Self {
            tax_id: tax_id.into(),
            id_cad_int_tran: None,
        }
    }

    /// Set the intermediary transaction registration ID.
    pub fn id_cad_int_tran(mut self, v: impl Into<String>) -> Self {
        self.id_cad_int_tran = Some(v.into());
        self
    }
}

/// Technical responsible entity (`<infRespTec>`) — the company that developed the
/// software used to issue the NF-e.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TechResponsibleData {
    /// CNPJ of the responsible software company (`CNPJ`).
    pub tax_id: String,
    /// Name of the technical contact person (`xContato`).
    pub contact: String,
    /// Contact email address (`email`).
    pub email: String,
    /// Contact phone number (`fone`). Optional.
    pub phone: Option<String>,
}

impl TechResponsibleData {
    /// Create a new `TechResponsibleData` with required fields.
    pub fn new(
        tax_id: impl Into<String>,
        contact: impl Into<String>,
        email: impl Into<String>,
    ) -> Self {
        Self {
            tax_id: tax_id.into(),
            contact: contact.into(),
            email: email.into(),
            phone: None,
        }
    }

    /// Set the phone number.
    pub fn phone(mut self, v: impl Into<String>) -> Self {
        self.phone = Some(v.into());
        self
    }
}

/// Purchase references (`<compra>`) linking the NF-e to a purchase order or contract.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct PurchaseData {
    /// Purchase order number (`xPed`). Optional.
    pub order_number: Option<String>,
    /// Contract number (`xCont`). Optional.
    pub contract_number: Option<String>,
    /// Purchase note / tender number (`xNEmp`). Optional.
    pub purchase_note: Option<String>,
}

impl PurchaseData {
    /// Create a new empty `PurchaseData`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the order number.
    pub fn order_number(mut self, v: impl Into<String>) -> Self {
        self.order_number = Some(v.into());
        self
    }
    /// Set the contract number.
    pub fn contract_number(mut self, v: impl Into<String>) -> Self {
        self.contract_number = Some(v.into());
        self
    }
    /// Set the purchase note.
    pub fn purchase_note(mut self, v: impl Into<String>) -> Self {
        self.purchase_note = Some(v.into());
        self
    }
}

/// Export information (`<exporta>`) for NF-e documents covering international exports.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ExportData {
    /// UF of the exit point from Brazil (`UFSaidaPais`).
    pub exit_state: String,
    /// Name of the export location / port (`xLocExporta`).
    pub export_location: String,
    /// Name of the dispatch/customs location (`xLocDespacho`). Optional.
    pub dispatch_location: Option<String>,
}

impl ExportData {
    /// Create a new `ExportData` with required fields.
    pub fn new(exit_state: impl Into<String>, export_location: impl Into<String>) -> Self {
        Self {
            exit_state: exit_state.into(),
            export_location: export_location.into(),
            dispatch_location: None,
        }
    }

    /// Set the dispatch location.
    pub fn dispatch_location(mut self, v: impl Into<String>) -> Self {
        self.dispatch_location = Some(v.into());
        self
    }
}

/// Retained federal taxes (`<retTrib>`) withheld at source.
///
/// All fields are optional; include only those applicable to the transaction.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct RetTribData {
    /// Retained PIS value (`vRetPIS`).
    pub v_ret_pis: Option<Cents>,
    /// Retained COFINS value (`vRetCOFINS`).
    pub v_ret_cofins: Option<Cents>,
    /// Retained CSLL value (`vRetCSLL`).
    pub v_ret_csll: Option<Cents>,
    /// IRRF calculation base (`vBCIRRF`).
    pub v_bc_irrf: Option<Cents>,
    /// Retained IRRF value (`vIRRF`).
    pub v_irrf: Option<Cents>,
    /// Social security (INSS) calculation base (`vBCRetPrev`).
    pub v_bc_ret_prev: Option<Cents>,
    /// Retained social security contribution (`vRetPrev`).
    pub v_ret_prev: Option<Cents>,
}

impl RetTribData {
    /// Create a new empty `RetTribData`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the retained PIS value.
    pub fn v_ret_pis(mut self, v: Cents) -> Self {
        self.v_ret_pis = Some(v);
        self
    }
    /// Set the retained COFINS value.
    pub fn v_ret_cofins(mut self, v: Cents) -> Self {
        self.v_ret_cofins = Some(v);
        self
    }
    /// Set the retained CSLL value.
    pub fn v_ret_csll(mut self, v: Cents) -> Self {
        self.v_ret_csll = Some(v);
        self
    }
    /// Set the IRRF base calculation value.
    pub fn v_bc_irrf(mut self, v: Cents) -> Self {
        self.v_bc_irrf = Some(v);
        self
    }
    /// Set the IRRF value.
    pub fn v_irrf(mut self, v: Cents) -> Self {
        self.v_irrf = Some(v);
        self
    }
    /// Set the social security base calculation value.
    pub fn v_bc_ret_prev(mut self, v: Cents) -> Self {
        self.v_bc_ret_prev = Some(v);
        self
    }
    /// Set the retained social security value.
    pub fn v_ret_prev(mut self, v: Cents) -> Self {
        self.v_ret_prev = Some(v);
        self
    }
}

/// Batch/lot tracking data (`<rastro>`) for traceability of perishable or regulated goods.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RastroData {
    /// Batch / lot number (`nLote`).
    pub n_lote: String,
    /// Quantity in this batch (`qLote`).
    pub q_lote: f64,
    /// Manufacturing / production date (`dFab`) in `YYYY-MM-DD` format.
    pub d_fab: String,
    /// Expiry / validation date (`dVal`) in `YYYY-MM-DD` format.
    pub d_val: String,
    /// Aggregate code (`cAgreg`). Optional.
    pub c_agreg: Option<String>,
}

impl RastroData {
    /// Create a new `RastroData` with required fields.
    pub fn new(
        n_lote: impl Into<String>,
        q_lote: f64,
        d_fab: impl Into<String>,
        d_val: impl Into<String>,
    ) -> Self {
        Self {
            n_lote: n_lote.into(),
            q_lote,
            d_fab: d_fab.into(),
            d_val: d_val.into(),
            c_agreg: None,
        }
    }

    /// Set the aggregate code.
    pub fn c_agreg(mut self, v: impl Into<String>) -> Self {
        self.c_agreg = Some(v.into());
        self
    }
}

/// Vehicle product details (`<veicProd>`) for NF-e documents covering automotive sales.
///
/// All fields are required as mandated by DENATRAN / SEFAZ vehicle invoicing rules.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct VeicProdData {
    /// Type of operation (`tpOp`): `"1"` (sale to end consumer), `"2"` (sell to reseller), `"3"` (other).
    pub tp_op: String,
    /// Chassis number (`chassi`), 17 characters.
    pub chassi: String,
    /// DENATRAN colour code (`cCor`).
    pub c_cor: String,
    /// Colour description (`xCor`).
    pub x_cor: String,
    /// Engine power in CV (`pot`).
    pub pot: String,
    /// Engine displacement in cm³ (`cilin`).
    pub cilin: String,
    /// Net weight in kg (`pesoL`).
    pub peso_l: String,
    /// Gross weight in kg (`pesoB`).
    pub peso_b: String,
    /// Vehicle serial number (`nSerie`).
    pub n_serie: String,
    /// Fuel type code (`tpComb`).
    pub tp_comb: String,
    /// Engine number (`nMotor`).
    pub n_motor: String,
    /// Maximum towing capacity in kg (`CMT`).
    pub cmt: String,
    /// Wheelbase in mm (`dist`).
    pub dist: String,
    /// Model year (`anoMod`).
    pub ano_mod: String,
    /// Manufacturing year (`anoFab`).
    pub ano_fab: String,
    /// Paint type code (`tpPint`).
    pub tp_pint: String,
    /// Vehicle type code (`tpVeic`).
    pub tp_veic: String,
    /// Vehicle species code (`espVeic`).
    pub esp_veic: String,
    /// VIN condition (`VIN`): `"R"` (regular) or `"N"` (non-regular).
    pub vin: String,
    /// Vehicle condition (`condVeic`): `"1"` (new) or `"2"` (used).
    pub cond_veic: String,
    /// DENATRAN vehicle model code (`cMod`).
    pub c_mod: String,
    /// DENATRAN colour code (extended) (`cCorDENATRAN`).
    pub c_cor_denatran: String,
    /// Passenger capacity (`lota`).
    pub lota: String,
    /// Vehicle restriction code (`tpRest`).
    pub tp_rest: String,
}

impl VeicProdData {
    /// Create a new `VeicProdData` with all required fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tp_op: impl Into<String>,
        chassi: impl Into<String>,
        c_cor: impl Into<String>,
        x_cor: impl Into<String>,
        pot: impl Into<String>,
        cilin: impl Into<String>,
        peso_l: impl Into<String>,
        peso_b: impl Into<String>,
        n_serie: impl Into<String>,
        tp_comb: impl Into<String>,
        n_motor: impl Into<String>,
        cmt: impl Into<String>,
        dist: impl Into<String>,
        ano_mod: impl Into<String>,
        ano_fab: impl Into<String>,
        tp_pint: impl Into<String>,
        tp_veic: impl Into<String>,
        esp_veic: impl Into<String>,
        vin: impl Into<String>,
        cond_veic: impl Into<String>,
        c_mod: impl Into<String>,
        c_cor_denatran: impl Into<String>,
        lota: impl Into<String>,
        tp_rest: impl Into<String>,
    ) -> Self {
        Self {
            tp_op: tp_op.into(),
            chassi: chassi.into(),
            c_cor: c_cor.into(),
            x_cor: x_cor.into(),
            pot: pot.into(),
            cilin: cilin.into(),
            peso_l: peso_l.into(),
            peso_b: peso_b.into(),
            n_serie: n_serie.into(),
            tp_comb: tp_comb.into(),
            n_motor: n_motor.into(),
            cmt: cmt.into(),
            dist: dist.into(),
            ano_mod: ano_mod.into(),
            ano_fab: ano_fab.into(),
            tp_pint: tp_pint.into(),
            tp_veic: tp_veic.into(),
            esp_veic: esp_veic.into(),
            vin: vin.into(),
            cond_veic: cond_veic.into(),
            c_mod: c_mod.into(),
            c_cor_denatran: c_cor_denatran.into(),
            lota: lota.into(),
            tp_rest: tp_rest.into(),
        }
    }
}

/// Medicine / pharmaceutical product details (`<med>`).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct MedData {
    /// ANVISA product registry code (`cProdANVISA`). Optional (use `"isento"` when exempt).
    pub c_prod_anvisa: Option<String>,
    /// Exemption reason when `cProdANVISA` is absent (`xMotivoIsencao`). Optional.
    pub x_motivo_isencao: Option<String>,
    /// Maximum consumer price (`vPMC`) in the applicable state.
    pub v_pmc: Cents,
}

impl MedData {
    /// Create a new `MedData` with the required PMC value.
    pub fn new(v_pmc: Cents) -> Self {
        Self {
            c_prod_anvisa: None,
            x_motivo_isencao: None,
            v_pmc,
        }
    }

    /// Set the ANVISA product code.
    pub fn c_prod_anvisa(mut self, v: impl Into<String>) -> Self {
        self.c_prod_anvisa = Some(v.into());
        self
    }
    /// Set the exemption reason.
    pub fn x_motivo_isencao(mut self, v: impl Into<String>) -> Self {
        self.x_motivo_isencao = Some(v.into());
        self
    }
}

/// Firearm / weapon product details (`<arma>`).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ArmaData {
    /// Weapon type code (`tpArma`): `"0"` (allowed use) or `"1"` (restricted use).
    pub tp_arma: String,
    /// Weapon serial number (`nSerie`).
    pub n_serie: String,
    /// Barrel number (`nCano`).
    pub n_cano: String,
    /// Weapon description (`descr`).
    pub descr: String,
}

impl ArmaData {
    /// Create a new `ArmaData` with all required fields.
    pub fn new(
        tp_arma: impl Into<String>,
        n_serie: impl Into<String>,
        n_cano: impl Into<String>,
        descr: impl Into<String>,
    ) -> Self {
        Self {
            tp_arma: tp_arma.into(),
            n_serie: n_serie.into(),
            n_cano: n_cano.into(),
            descr: descr.into(),
        }
    }
}

/// Per-item observation fields (`<obsItem>`).
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ObsItemData {
    /// Contributor observation (`obsCont`). Optional.
    pub obs_cont: Option<ObsField>,
    /// Fiscal observation (`obsFisco`). Optional.
    pub obs_fisco: Option<ObsField>,
}

impl ObsItemData {
    /// Create a new empty `ObsItemData`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the contributor observation.
    pub fn obs_cont(mut self, v: ObsField) -> Self {
        self.obs_cont = Some(v);
        self
    }
    /// Set the fiscal observation.
    pub fn obs_fisco(mut self, v: ObsField) -> Self {
        self.obs_fisco = Some(v);
        self
    }
}

/// A single per-item observation field (`obsCont` or `obsFisco`).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ObsField {
    /// Field identifier (`xCampo`), max 20 characters.
    pub x_campo: String,
    /// Text content (`xTexto`), max 60 characters.
    pub x_texto: String,
}

impl ObsField {
    /// Create a new `ObsField`.
    pub fn new(x_campo: impl Into<String>, x_texto: impl Into<String>) -> Self {
        Self {
            x_campo: x_campo.into(),
            x_texto: x_texto.into(),
        }
    }
}

/// A referenced digital fiscal document (DFe) linked to an invoice item (`<DFeRef>`).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DFeReferenciadoData {
    /// 44-digit access key of the referenced DFe.
    pub chave_acesso: String,
    /// Item number within the referenced DFe (`nItemRef`). Optional.
    pub n_item: Option<String>,
}

impl DFeReferenciadoData {
    /// Create a new `DFeReferenciadoData`.
    pub fn new(chave_acesso: impl Into<String>) -> Self {
        Self {
            chave_acesso: chave_acesso.into(),
            n_item: None,
        }
    }

    /// Set the item number.
    pub fn n_item(mut self, v: impl Into<String>) -> Self {
        self.n_item = Some(v.into());
        self
    }
}

/// Complete data for a single invoice line item (`<det>`), including product
/// identification, pricing, and all applicable taxes.
///
/// Required fields are supplied via [`InvoiceItemData::new`]; optional fields
/// (shipping, discounts, extended tax fields, specialised product data) are set
/// via chainable setter methods.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct InvoiceItemData {
    /// Sequential item number (`nItem`, 1-based).
    pub item_number: u32,
    /// Issuer's internal product code (`cProd`).
    pub product_code: String,
    /// Product or service description (`xProd`).
    pub description: String,
    /// NCM (Nomenclatura Comum do MERCOSUL) classification code.
    pub ncm: String,
    /// CFOP operation code (4 digits).
    pub cfop: String,
    /// Commercial unit of measure (`uCom`), e.g. `"UN"`, `"KG"`.
    pub unit_of_measure: String,
    /// Quantity in commercial units (`qCom`).
    pub quantity: f64,
    /// Commercial unit price (`vUnCom`).
    pub unit_price: Cents,
    /// Total item value (`vProd = qCom × vUnCom`).
    pub total_price: Cents,
    /// GTIN / EAN barcode for commercial units (`cEAN`). `None` = no barcode.
    pub c_ean: Option<String>,
    /// GTIN / EAN barcode for the taxation unit (`cEANTrib`). `None` = no barcode.
    pub c_ean_trib: Option<String>,
    /// CEST code for ST-subject products (`CEST`). Optional.
    pub cest: Option<String>,
    /// Freight value allocated to this item (`vFrete`). Optional.
    pub v_frete: Option<Cents>,
    /// Insurance value allocated to this item (`vSeg`). Optional.
    pub v_seg: Option<Cents>,
    /// Discount value for this item (`vDesc`). Optional.
    pub v_desc: Option<Cents>,
    /// Other costs allocated to this item (`vOutro`). Optional.
    pub v_outro: Option<Cents>,
    /// Product origin code (`orig`), e.g. `"0"` (domestic). Optional.
    pub orig: Option<String>,
    // ── ICMS ────────────────────────────────────────────────────────────────
    /// ICMS CST or CSOSN code (2–3 digits).
    pub icms_cst: String,
    /// ICMS rate applied to this item (`pICMS`).
    pub icms_rate: Rate,
    /// ICMS value for this item (`vICMS`).
    pub icms_amount: Cents,
    /// ICMS base calculation modality (`modBC`). Optional.
    pub icms_mod_bc: Option<i64>,
    /// ICMS base reduction rate (`pRedBC`). Optional.
    pub icms_red_bc: Option<Rate>,
    /// ICMS ST base calculation modality (`modBCST`). Optional.
    pub icms_mod_bc_st: Option<i64>,
    /// ICMS ST added value margin (`pMVAST`). Optional.
    pub icms_p_mva_st: Option<Rate>,
    /// ICMS ST base reduction rate (`pRedBCST`). Optional.
    pub icms_red_bc_st: Option<Rate>,
    /// ICMS ST calculation base value (`vBCST`). Optional.
    pub icms_v_bc_st: Option<Cents>,
    /// ICMS ST rate (`pICMSST`). Optional.
    pub icms_p_icms_st: Option<Rate>,
    /// ICMS ST value (`vICMSST`). Optional.
    pub icms_v_icms_st: Option<Cents>,
    /// Desonerated ICMS value (`vICMSDeson`). Optional.
    pub icms_v_icms_deson: Option<Cents>,
    /// Reason code for ICMS desoneration (`motDesICMS`). Optional.
    pub icms_mot_des_icms: Option<i64>,
    /// FCP rate (`pFCP`). Optional.
    pub icms_p_fcp: Option<Rate>,
    /// FCP value (`vFCP`). Optional.
    pub icms_v_fcp: Option<Cents>,
    /// FCP calculation base (`vBCFCP`). Optional.
    pub icms_v_bc_fcp: Option<Cents>,
    /// FCP-ST rate (`pFCPST`). Optional.
    pub icms_p_fcp_st: Option<Rate>,
    /// FCP-ST value (`vFCPST`). Optional.
    pub icms_v_fcp_st: Option<Cents>,
    /// FCP-ST calculation base (`vBCFCPST`). Optional.
    pub icms_v_bc_fcp_st: Option<Cents>,
    /// Simples Nacional ICMS credit rate (`pCredSN`). Optional.
    pub icms_p_cred_sn: Option<Rate>,
    /// Simples Nacional ICMS credit value (`vCredICMSSN`). Optional.
    pub icms_v_cred_icms_sn: Option<Cents>,
    /// ICMS substitute value (`vICMSSubstituto`). Optional.
    pub icms_v_icms_substituto: Option<Cents>,
    // ── PIS ─────────────────────────────────────────────────────────────────
    /// PIS CST code (2 digits).
    pub pis_cst: String,
    /// PIS calculation base value (`vBCPIS`). Optional.
    pub pis_v_bc: Option<Cents>,
    /// PIS rate (`pPIS`). Optional.
    pub pis_p_pis: Option<Rate4>,
    /// PIS value (`vPIS`). Optional.
    pub pis_v_pis: Option<Cents>,
    /// PIS quantity base (`qBCProd`). Optional.
    pub pis_q_bc_prod: Option<i64>,
    /// PIS unit value (`vAliqProd`) for quantity-based calculation. Optional.
    pub pis_v_aliq_prod: Option<i64>,
    // ── COFINS ──────────────────────────────────────────────────────────────
    /// COFINS CST code (2 digits).
    pub cofins_cst: String,
    /// COFINS calculation base value (`vBCCOFINS`). Optional.
    pub cofins_v_bc: Option<Cents>,
    /// COFINS rate (`pCOFINS`). Optional.
    pub cofins_p_cofins: Option<Rate4>,
    /// COFINS value (`vCOFINS`). Optional.
    pub cofins_v_cofins: Option<Cents>,
    /// COFINS quantity base (`qBCProd`). Optional.
    pub cofins_q_bc_prod: Option<i64>,
    /// COFINS unit value (`vAliqProd`) for quantity-based calculation. Optional.
    pub cofins_v_aliq_prod: Option<i64>,
    // ── IPI ─────────────────────────────────────────────────────────────────
    /// IPI CST code. Optional (only for industrialised products).
    pub ipi_cst: Option<String>,
    /// IPI enquadramento (classification) code (`cEnq`). Optional.
    pub ipi_c_enq: Option<String>,
    /// IPI calculation base (`vBCIPI`). Optional.
    pub ipi_v_bc: Option<Cents>,
    /// IPI rate (`pIPI`). Optional.
    pub ipi_p_ipi: Option<Rate>,
    /// IPI value (`vIPI`). Optional.
    pub ipi_v_ipi: Option<Cents>,
    /// IPI quantity base (`qUnid`). Optional.
    pub ipi_q_unid: Option<i64>,
    /// IPI unit value (`vUnid`). Optional.
    pub ipi_v_unid: Option<i64>,
    // ── II (Import Duty) ─────────────────────────────────────────────────────
    /// Import duty (II) calculation base (`vBCII`). Optional.
    pub ii_v_bc: Option<Cents>,
    /// Customs clearance expenses (`vDespAdu`). Optional.
    pub ii_v_desp_adu: Option<Cents>,
    /// Import duty value (`vII`). Optional.
    pub ii_v_ii: Option<Cents>,
    /// IOF (financial operations tax) for imports (`vIOF`). Optional.
    pub ii_v_iof: Option<Cents>,
    // ── Specialised product data ─────────────────────────────────────────────
    /// Batch / lot traceability records (`rastro`). Optional.
    pub rastro: Option<Vec<RastroData>>,
    /// Vehicle product details (`veicProd`). Optional.
    pub veic_prod: Option<VeicProdData>,
    /// Medicine / pharmaceutical product details (`med`). Optional.
    pub med: Option<MedData>,
    /// Firearm / weapon details (`arma`). Optional.
    pub arma: Option<Vec<ArmaData>>,
    /// RECOPI number for paper / printing sector products. Optional.
    pub n_recopi: Option<String>,
    /// Additional product information printed on the DANFE (`infAdProd`). Optional.
    pub inf_ad_prod: Option<String>,
    /// Per-item observations (`obsItem`). Optional.
    pub obs_item: Option<ObsItemData>,
    /// Referenced digital fiscal document for this item (`DFeRef`). Optional.
    pub dfe_referenciado: Option<DFeReferenciadoData>,
}

impl InvoiceItemData {
    /// Create a new `InvoiceItemData` with required fields.
    /// All optional fields default to `None` or zero.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        item_number: u32,
        product_code: impl Into<String>,
        description: impl Into<String>,
        ncm: impl Into<String>,
        cfop: impl Into<String>,
        unit_of_measure: impl Into<String>,
        quantity: f64,
        unit_price: Cents,
        total_price: Cents,
        icms_cst: impl Into<String>,
        icms_rate: Rate,
        icms_amount: Cents,
        pis_cst: impl Into<String>,
        cofins_cst: impl Into<String>,
    ) -> Self {
        Self {
            item_number,
            product_code: product_code.into(),
            description: description.into(),
            ncm: ncm.into(),
            cfop: cfop.into(),
            unit_of_measure: unit_of_measure.into(),
            quantity,
            unit_price,
            total_price,
            c_ean: None,
            c_ean_trib: None,
            cest: None,
            v_frete: None,
            v_seg: None,
            v_desc: None,
            v_outro: None,
            orig: None,
            icms_cst: icms_cst.into(),
            icms_rate,
            icms_amount,
            icms_mod_bc: None,
            icms_red_bc: None,
            icms_mod_bc_st: None,
            icms_p_mva_st: None,
            icms_red_bc_st: None,
            icms_v_bc_st: None,
            icms_p_icms_st: None,
            icms_v_icms_st: None,
            icms_v_icms_deson: None,
            icms_mot_des_icms: None,
            icms_p_fcp: None,
            icms_v_fcp: None,
            icms_v_bc_fcp: None,
            icms_p_fcp_st: None,
            icms_v_fcp_st: None,
            icms_v_bc_fcp_st: None,
            icms_p_cred_sn: None,
            icms_v_cred_icms_sn: None,
            icms_v_icms_substituto: None,
            pis_cst: pis_cst.into(),
            pis_v_bc: None,
            pis_p_pis: None,
            pis_v_pis: None,
            pis_q_bc_prod: None,
            pis_v_aliq_prod: None,
            cofins_cst: cofins_cst.into(),
            cofins_v_bc: None,
            cofins_p_cofins: None,
            cofins_v_cofins: None,
            cofins_q_bc_prod: None,
            cofins_v_aliq_prod: None,
            ipi_cst: None,
            ipi_c_enq: None,
            ipi_v_bc: None,
            ipi_p_ipi: None,
            ipi_v_ipi: None,
            ipi_q_unid: None,
            ipi_v_unid: None,
            ii_v_bc: None,
            ii_v_desp_adu: None,
            ii_v_ii: None,
            ii_v_iof: None,
            rastro: None,
            veic_prod: None,
            med: None,
            arma: None,
            n_recopi: None,
            inf_ad_prod: None,
            obs_item: None,
            dfe_referenciado: None,
        }
    }

    // Chainable setters for optional fields
    /// Set the EAN code.
    pub fn c_ean(mut self, v: impl Into<String>) -> Self {
        self.c_ean = Some(v.into());
        self
    }
    /// Set the tributary EAN code.
    pub fn c_ean_trib(mut self, v: impl Into<String>) -> Self {
        self.c_ean_trib = Some(v.into());
        self
    }
    /// Set the CEST code.
    pub fn cest(mut self, v: impl Into<String>) -> Self {
        self.cest = Some(v.into());
        self
    }
    /// Set the freight value.
    pub fn v_frete(mut self, v: Cents) -> Self {
        self.v_frete = Some(v);
        self
    }
    /// Set the insurance value.
    pub fn v_seg(mut self, v: Cents) -> Self {
        self.v_seg = Some(v);
        self
    }
    /// Set the discount value.
    pub fn v_desc(mut self, v: Cents) -> Self {
        self.v_desc = Some(v);
        self
    }
    /// Set the "other" value.
    pub fn v_outro(mut self, v: Cents) -> Self {
        self.v_outro = Some(v);
        self
    }
    /// Set the origin code.
    pub fn orig(mut self, v: impl Into<String>) -> Self {
        self.orig = Some(v.into());
        self
    }
    /// Set the ICMS base calculation modality.
    pub fn icms_mod_bc(mut self, v: i64) -> Self {
        self.icms_mod_bc = Some(v);
        self
    }
    /// Set the ICMS base reduction rate.
    pub fn icms_red_bc(mut self, v: Rate) -> Self {
        self.icms_red_bc = Some(v);
        self
    }
    /// Set the ICMS ST base calculation modality.
    pub fn icms_mod_bc_st(mut self, v: i64) -> Self {
        self.icms_mod_bc_st = Some(v);
        self
    }
    /// Set the ICMS ST MVA rate.
    pub fn icms_p_mva_st(mut self, v: Rate) -> Self {
        self.icms_p_mva_st = Some(v);
        self
    }
    /// Set the ICMS ST base reduction rate.
    pub fn icms_red_bc_st(mut self, v: Rate) -> Self {
        self.icms_red_bc_st = Some(v);
        self
    }
    /// Set the ICMS ST base value.
    pub fn icms_v_bc_st(mut self, v: Cents) -> Self {
        self.icms_v_bc_st = Some(v);
        self
    }
    /// Set the ICMS ST rate.
    pub fn icms_p_icms_st(mut self, v: Rate) -> Self {
        self.icms_p_icms_st = Some(v);
        self
    }
    /// Set the ICMS ST value.
    pub fn icms_v_icms_st(mut self, v: Cents) -> Self {
        self.icms_v_icms_st = Some(v);
        self
    }
    /// Set the desonerated ICMS value.
    pub fn icms_v_icms_deson(mut self, v: Cents) -> Self {
        self.icms_v_icms_deson = Some(v);
        self
    }
    /// Set the ICMS desonerating motive.
    pub fn icms_mot_des_icms(mut self, v: i64) -> Self {
        self.icms_mot_des_icms = Some(v);
        self
    }
    /// Set the FCP rate.
    pub fn icms_p_fcp(mut self, v: Rate) -> Self {
        self.icms_p_fcp = Some(v);
        self
    }
    /// Set the FCP value.
    pub fn icms_v_fcp(mut self, v: Cents) -> Self {
        self.icms_v_fcp = Some(v);
        self
    }
    /// Set the FCP base value.
    pub fn icms_v_bc_fcp(mut self, v: Cents) -> Self {
        self.icms_v_bc_fcp = Some(v);
        self
    }
    /// Set the FCP ST rate.
    pub fn icms_p_fcp_st(mut self, v: Rate) -> Self {
        self.icms_p_fcp_st = Some(v);
        self
    }
    /// Set the FCP ST value.
    pub fn icms_v_fcp_st(mut self, v: Cents) -> Self {
        self.icms_v_fcp_st = Some(v);
        self
    }
    /// Set the FCP ST base value.
    pub fn icms_v_bc_fcp_st(mut self, v: Cents) -> Self {
        self.icms_v_bc_fcp_st = Some(v);
        self
    }
    /// Set the Simples Nacional credit rate.
    pub fn icms_p_cred_sn(mut self, v: Rate) -> Self {
        self.icms_p_cred_sn = Some(v);
        self
    }
    /// Set the Simples Nacional credit ICMS value.
    pub fn icms_v_cred_icms_sn(mut self, v: Cents) -> Self {
        self.icms_v_cred_icms_sn = Some(v);
        self
    }
    /// Set the ICMS substitute value.
    pub fn icms_v_icms_substituto(mut self, v: Cents) -> Self {
        self.icms_v_icms_substituto = Some(v);
        self
    }
    /// Set the PIS base value.
    pub fn pis_v_bc(mut self, v: Cents) -> Self {
        self.pis_v_bc = Some(v);
        self
    }
    /// Set the PIS rate.
    pub fn pis_p_pis(mut self, v: Rate4) -> Self {
        self.pis_p_pis = Some(v);
        self
    }
    /// Set the PIS value.
    pub fn pis_v_pis(mut self, v: Cents) -> Self {
        self.pis_v_pis = Some(v);
        self
    }
    /// Set the PIS quantity base.
    pub fn pis_q_bc_prod(mut self, v: i64) -> Self {
        self.pis_q_bc_prod = Some(v);
        self
    }
    /// Set the PIS quantity rate.
    pub fn pis_v_aliq_prod(mut self, v: i64) -> Self {
        self.pis_v_aliq_prod = Some(v);
        self
    }
    /// Set the COFINS base value.
    pub fn cofins_v_bc(mut self, v: Cents) -> Self {
        self.cofins_v_bc = Some(v);
        self
    }
    /// Set the COFINS rate.
    pub fn cofins_p_cofins(mut self, v: Rate4) -> Self {
        self.cofins_p_cofins = Some(v);
        self
    }
    /// Set the COFINS value.
    pub fn cofins_v_cofins(mut self, v: Cents) -> Self {
        self.cofins_v_cofins = Some(v);
        self
    }
    /// Set the COFINS quantity base.
    pub fn cofins_q_bc_prod(mut self, v: i64) -> Self {
        self.cofins_q_bc_prod = Some(v);
        self
    }
    /// Set the COFINS quantity rate.
    pub fn cofins_v_aliq_prod(mut self, v: i64) -> Self {
        self.cofins_v_aliq_prod = Some(v);
        self
    }
    /// Set the IPI CST.
    pub fn ipi_cst(mut self, v: impl Into<String>) -> Self {
        self.ipi_cst = Some(v.into());
        self
    }
    /// Set the IPI enquadramento code.
    pub fn ipi_c_enq(mut self, v: impl Into<String>) -> Self {
        self.ipi_c_enq = Some(v.into());
        self
    }
    /// Set the IPI base value.
    pub fn ipi_v_bc(mut self, v: Cents) -> Self {
        self.ipi_v_bc = Some(v);
        self
    }
    /// Set the IPI rate.
    pub fn ipi_p_ipi(mut self, v: Rate) -> Self {
        self.ipi_p_ipi = Some(v);
        self
    }
    /// Set the IPI value.
    pub fn ipi_v_ipi(mut self, v: Cents) -> Self {
        self.ipi_v_ipi = Some(v);
        self
    }
    /// Set the IPI quantity.
    pub fn ipi_q_unid(mut self, v: i64) -> Self {
        self.ipi_q_unid = Some(v);
        self
    }
    /// Set the IPI unit value.
    pub fn ipi_v_unid(mut self, v: i64) -> Self {
        self.ipi_v_unid = Some(v);
        self
    }
    /// Set the II base value.
    pub fn ii_v_bc(mut self, v: Cents) -> Self {
        self.ii_v_bc = Some(v);
        self
    }
    /// Set the II customs expenses.
    pub fn ii_v_desp_adu(mut self, v: Cents) -> Self {
        self.ii_v_desp_adu = Some(v);
        self
    }
    /// Set the II value.
    pub fn ii_v_ii(mut self, v: Cents) -> Self {
        self.ii_v_ii = Some(v);
        self
    }
    /// Set the II IOF value.
    pub fn ii_v_iof(mut self, v: Cents) -> Self {
        self.ii_v_iof = Some(v);
        self
    }
    /// Set batch tracking data.
    pub fn rastro(mut self, v: Vec<RastroData>) -> Self {
        self.rastro = Some(v);
        self
    }
    /// Set vehicle product data.
    pub fn veic_prod(mut self, v: VeicProdData) -> Self {
        self.veic_prod = Some(v);
        self
    }
    /// Set medicine data.
    pub fn med(mut self, v: MedData) -> Self {
        self.med = Some(v);
        self
    }
    /// Set weapon data.
    pub fn arma(mut self, v: Vec<ArmaData>) -> Self {
        self.arma = Some(v);
        self
    }
    /// Set RECOPI number.
    pub fn n_recopi(mut self, v: impl Into<String>) -> Self {
        self.n_recopi = Some(v.into());
        self
    }
    /// Set additional product info.
    pub fn inf_ad_prod(mut self, v: impl Into<String>) -> Self {
        self.inf_ad_prod = Some(v.into());
        self
    }
    /// Set per-item observation data.
    pub fn obs_item(mut self, v: ObsItemData) -> Self {
        self.obs_item = Some(v);
        self
    }
    /// Set referenced DFe data.
    pub fn dfe_referenciado(mut self, v: DFeReferenciadoData) -> Self {
        self.dfe_referenciado = Some(v);
        self
    }
}

/// Internal data bag assembled by [`InvoiceBuilder`] and consumed by sub-modules.
///
/// Not part of the public API — callers should use the builder.
#[derive(Debug, Clone)]
pub(crate) struct InvoiceBuildData {
    pub model: InvoiceModel,
    pub series: u32,
    pub number: u32,
    pub emission_type: EmissionType,
    pub environment: SefazEnvironment,
    pub issued_at: DateTime<FixedOffset>,
    pub operation_nature: String,
    pub issuer: IssuerData,
    pub recipient: Option<RecipientData>,
    pub items: Vec<InvoiceItemData>,
    pub payments: Vec<PaymentData>,
    pub change_amount: Option<Cents>,
    pub payment_card_details: Option<Vec<PaymentCardDetail>>,
    pub contingency: Option<ContingencyData>,
    // IDE overrides
    pub operation_type: Option<u8>,
    pub purpose_code: Option<u8>,
    pub intermediary_indicator: Option<String>,
    pub emission_process: Option<String>,
    pub consumer_type: Option<String>,
    pub buyer_presence: Option<String>,
    pub print_format: Option<String>,
    pub references: Option<Vec<ReferenceDoc>>,
    // Optional groups
    pub transport: Option<TransportData>,
    pub billing: Option<BillingData>,
    pub withdrawal: Option<LocationData>,
    pub delivery: Option<LocationData>,
    pub authorized_xml: Option<Vec<AuthorizedXml>>,
    pub additional_info: Option<AdditionalInfo>,
    pub intermediary: Option<IntermediaryData>,
    pub ret_trib: Option<RetTribData>,
    pub tech_responsible: Option<TechResponsibleData>,
    pub purchase: Option<PurchaseData>,
    pub export: Option<ExportData>,
}

/// Third-party entity authorised to download the NF-e XML from the SEFAZ portal (`<autXML>`).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct AuthorizedXml {
    /// CNPJ or CPF of the authorised entity.
    pub tax_id: String,
}

impl AuthorizedXml {
    /// Create a new `AuthorizedXml`.
    pub fn new(tax_id: impl Into<String>) -> Self {
        Self {
            tax_id: tax_id.into(),
        }
    }
}

/// Internal result of XML generation, consumed by the builder.
#[derive(Debug, Clone)]
pub(crate) struct InvoiceXmlResult {
    pub xml: String,
    pub access_key: String,
}

/// Parameters for building an NFC-e QR Code URL.
///
/// Pass to [`crate::qrcode::build_nfce_qr_code_url`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct NfceQrCodeParams {
    /// 44-digit NFC-e access key.
    pub access_key: String,
    /// QR Code generation version.
    pub version: QrCodeVersion,
    /// Submission environment (production or homologation).
    pub environment: SefazEnvironment,
    /// Emission type (normal or contingency).
    pub emission_type: EmissionType,
    /// Base URL from the state's NFC-e portal (without `?p=`).
    pub qr_code_base_url: String,
    /// CSC (Consumer Security Code) token. Required for `V200`.
    pub csc_token: Option<String>,
    /// CSC identifier number. Required for `V200`.
    pub csc_id: Option<String>,
    /// Emission date-time string (`dhEmi`). Required for offline `V200`.
    pub issued_at: Option<String>,
    /// Total NFC-e value string (2 decimal places). Required for offline `V200`.
    pub total_value: Option<String>,
    /// Total ICMS value string. Optional.
    pub total_icms: Option<String>,
    /// SHA-1 digest value from the XML signature. Required for offline `V200`.
    pub digest_value: Option<String>,
    /// CNPJ, CPF, or foreign ID of the destination. Optional.
    pub dest_document: Option<String>,
    /// Destination ID type indicator. Optional.
    pub dest_id_type: Option<String>,
}

impl NfceQrCodeParams {
    /// Create a new `NfceQrCodeParams` with required fields.
    pub fn new(
        access_key: impl Into<String>,
        version: QrCodeVersion,
        environment: SefazEnvironment,
        emission_type: EmissionType,
        qr_code_base_url: impl Into<String>,
    ) -> Self {
        Self {
            access_key: access_key.into(),
            version,
            environment,
            emission_type,
            qr_code_base_url: qr_code_base_url.into(),
            csc_token: None,
            csc_id: None,
            issued_at: None,
            total_value: None,
            total_icms: None,
            digest_value: None,
            dest_document: None,
            dest_id_type: None,
        }
    }

    /// Set the CSC token.
    pub fn csc_token(mut self, v: impl Into<String>) -> Self {
        self.csc_token = Some(v.into());
        self
    }
    /// Set the CSC ID.
    pub fn csc_id(mut self, v: impl Into<String>) -> Self {
        self.csc_id = Some(v.into());
        self
    }
    /// Set the issued at date.
    pub fn issued_at(mut self, v: impl Into<String>) -> Self {
        self.issued_at = Some(v.into());
        self
    }
    /// Set the total value.
    pub fn total_value(mut self, v: impl Into<String>) -> Self {
        self.total_value = Some(v.into());
        self
    }
    /// Set the total ICMS.
    pub fn total_icms(mut self, v: impl Into<String>) -> Self {
        self.total_icms = Some(v.into());
        self
    }
    /// Set the digest value.
    pub fn digest_value(mut self, v: impl Into<String>) -> Self {
        self.digest_value = Some(v.into());
        self
    }
    /// Set the destination document.
    pub fn dest_document(mut self, v: impl Into<String>) -> Self {
        self.dest_document = Some(v.into());
        self
    }
    /// Set the destination ID type.
    pub fn dest_id_type(mut self, v: impl Into<String>) -> Self {
        self.dest_id_type = Some(v.into());
        self
    }
}

/// Parameters for inserting the `<infNFeSupl>` QR Code block into a signed NFC-e XML.
///
/// Pass to [`crate::qrcode::put_qr_tag`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PutQRTagParams {
    /// The signed NFC-e XML string.
    pub xml: String,
    /// CSC (Consumer Security Code) token value.
    pub csc_token: String,
    /// CSC identifier number (as a string).
    pub csc_id: String,
    /// QR Code version string: `"200"` or `"300"`.
    pub version: String,
    /// Base QR Code URL for the issuer's state.
    pub qr_code_base_url: String,
    /// Base URL for the consumer consultation link (`urlChave`).
    pub url_chave: String,
}

impl PutQRTagParams {
    /// Create a new `PutQRTagParams` with all required fields.
    pub fn new(
        xml: impl Into<String>,
        csc_token: impl Into<String>,
        csc_id: impl Into<String>,
        version: impl Into<String>,
        qr_code_base_url: impl Into<String>,
        url_chave: impl Into<String>,
    ) -> Self {
        Self {
            xml: xml.into(),
            csc_token: csc_token.into(),
            csc_id: csc_id.into(),
            version: version.into(),
            qr_code_base_url: qr_code_base_url.into(),
            url_chave: url_chave.into(),
        }
    }
}
