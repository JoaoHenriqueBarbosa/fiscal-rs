use chrono::{DateTime, FixedOffset, NaiveDate};

use crate::newtypes::{Cents, IbgeCode, Rate, Rate4};

// ── Enums ────────────────────────────────────────────────────────────────────

/// NF-e model: 55 = NF-e (B2B), 65 = NFC-e (consumer)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum InvoiceModel {
    Nfe = 55,
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

/// SEFAZ environment: 1 = production, 2 = homologation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SefazEnvironment {
    Production = 1,
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

/// Emission type (tpEmis)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EmissionType {
    Normal = 1,
    SvcAn = 6,
    SvcRs = 7,
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

/// Tax regime (CRT): 1=Simples Nacional, 2=Simples excess, 3=Normal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TaxRegime {
    SimplesNacional = 1,
    SimplesExcess = 2,
    Normal = 3,
}

/// Contingency type for NF-e emission fallback
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContingencyType {
    SvcAn,
    SvcRs,
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

/// QR Code version: 200 (v2) or 300 (v3, NT 2025.001)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum QrCodeVersion {
    V200 = 200,
    V300 = 300,
}

// ── Data structures ──────────────────────────────────────────────────────────

/// Certificate loaded from PFX file
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CertificateData {
    pub private_key: String,
    pub certificate: String,
    pub pfx_buffer: Vec<u8>,
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

/// Certificate info for display purposes
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CertificateInfo {
    pub common_name: String,
    pub valid_from: NaiveDate,
    pub valid_until: NaiveDate,
    pub serial_number: String,
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

/// Access key components for NF-e/NFC-e
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct AccessKeyParams {
    /// IBGE numeric state code (e.g. "41" for PR).
    pub state_code: IbgeCode,
    pub year_month: String,
    pub tax_id: String,
    pub model: InvoiceModel,
    pub series: u32,
    pub number: u32,
    pub emission_type: EmissionType,
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

/// Issuer data (from fiscal settings)
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct IssuerData {
    pub tax_id: String,
    pub state_tax_id: String,
    pub company_name: String,
    pub trade_name: Option<String>,
    pub tax_regime: TaxRegime,
    /// Two-letter state abbreviation (UF), e.g. "PR".
    pub state_code: String,
    /// IBGE city code, e.g. "4106852".
    pub city_code: IbgeCode,
    pub city_name: String,
    pub street: String,
    pub street_number: String,
    pub district: String,
    pub zip_code: String,
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

/// Recipient data (optional for NFC-e under R$200)
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct RecipientData {
    pub tax_id: String,
    pub name: String,
    /// Two-letter state abbreviation (UF), e.g. "PR". Kept as Option<String>
    /// because recipients can be from any state or absent entirely.
    pub state_code: Option<String>,
    pub state_tax_id: Option<String>,
    pub street: Option<String>,
    pub street_number: Option<String>,
    pub district: Option<String>,
    /// IBGE city code, e.g. "4106852".
    pub city_code: Option<IbgeCode>,
    pub city_name: Option<String>,
    pub zip_code: Option<String>,
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

/// Contingency data for fallback emission
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ContingencyData {
    pub contingency_type: ContingencyType,
    pub reason: String,
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

/// Payment data for invoice
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PaymentData {
    pub method: String,
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

/// Payment card details
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct PaymentCardDetail {
    pub integ_type: Option<String>,
    pub card_tax_id: Option<String>,
    pub card_brand: Option<String>,
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

/// Referenced document types
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ReferenceDoc {
    Nfe {
        access_key: String,
    },
    Nf {
        /// IBGE numeric state code (e.g. "41" for PR).
        state_code: IbgeCode,
        year_month: String,
        tax_id: String,
        model: String,
        series: String,
        number: String,
    },
    Nfp {
        /// IBGE numeric state code (e.g. "41" for PR).
        state_code: IbgeCode,
        year_month: String,
        tax_id: String,
        model: String,
        series: String,
        number: String,
    },
    Cte {
        access_key: String,
    },
    Ecf {
        model: String,
        ecf_number: String,
        coo_number: String,
    },
}

/// Transport data
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct TransportData {
    pub freight_mode: String,
    pub carrier: Option<CarrierData>,
    pub vehicle: Option<VehicleData>,
    pub trailers: Option<Vec<VehicleData>>,
    pub volumes: Option<Vec<VolumeData>>,
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
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct CarrierData {
    pub tax_id: Option<String>,
    pub name: Option<String>,
    pub state_tax_id: Option<String>,
    pub state_code: Option<String>,
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

/// Vehicle data for transport (veicTransp / reboque).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct VehicleData {
    pub plate: String,
    pub state_code: String,
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

/// Volume data for transported goods (vol).
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct VolumeData {
    pub quantity: Option<u32>,
    pub species: Option<String>,
    pub brand: Option<String>,
    pub number: Option<String>,
    pub net_weight: Option<f64>,
    pub gross_weight: Option<f64>,
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

/// Retained ICMS on transport services (retTransp).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RetainedIcmsTransp {
    pub v_bc_ret: Cents,
    pub p_icms_ret: Rate,
    pub v_icms_ret: Cents,
    pub cfop: String,
    /// IBGE city code for the transport tax event municipality.
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

/// Billing data (cobr)
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct BillingData {
    pub invoice: Option<BillingInvoice>,
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

/// Billing invoice header (fat) with original, discount, and net values.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct BillingInvoice {
    pub number: String,
    pub original_value: Cents,
    pub discount_value: Option<Cents>,
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

/// Billing installment (dup) with number, due date, and value.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Installment {
    pub number: String,
    pub due_date: String,
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

/// Withdrawal/pickup or delivery location
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct LocationData {
    pub tax_id: String,
    pub name: Option<String>,
    pub street: String,
    pub number: String,
    pub complement: Option<String>,
    pub district: String,
    /// IBGE city code, e.g. "4106852".
    pub city_code: IbgeCode,
    pub city_name: String,
    /// Two-letter state abbreviation (UF), e.g. "PR".
    pub state_code: String,
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

/// Additional info (infAdic)
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct AdditionalInfo {
    pub taxpayer_note: Option<String>,
    pub tax_authority_note: Option<String>,
    pub contributor_obs: Option<Vec<FieldText>>,
    pub fiscal_obs: Option<Vec<FieldText>>,
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

/// Generic field/text pair used in contributor and fiscal observations.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FieldText {
    pub field: String,
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

/// Referenced administrative or judicial process (procRef).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ProcessRef {
    pub number: String,
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

/// Intermediary info (infIntermed)
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct IntermediaryData {
    pub tax_id: String,
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

/// Tech responsible (infRespTec)
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TechResponsibleData {
    pub tax_id: String,
    pub contact: String,
    pub email: String,
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

/// Purchase data (compra)
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct PurchaseData {
    pub order_number: Option<String>,
    pub contract_number: Option<String>,
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

/// Export data (exporta)
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ExportData {
    pub exit_state: String,
    pub export_location: String,
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

/// Retained taxes (retTrib)
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct RetTribData {
    pub v_ret_pis: Option<Cents>,
    pub v_ret_cofins: Option<Cents>,
    pub v_ret_csll: Option<Cents>,
    pub v_bc_irrf: Option<Cents>,
    pub v_irrf: Option<Cents>,
    pub v_bc_ret_prev: Option<Cents>,
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

/// Batch tracking (rastro)
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RastroData {
    pub n_lote: String,
    pub q_lote: f64,
    pub d_fab: String,
    pub d_val: String,
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

/// Vehicle details (veicProd)
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct VeicProdData {
    pub tp_op: String,
    pub chassi: String,
    pub c_cor: String,
    pub x_cor: String,
    pub pot: String,
    pub cilin: String,
    pub peso_l: String,
    pub peso_b: String,
    pub n_serie: String,
    pub tp_comb: String,
    pub n_motor: String,
    pub cmt: String,
    pub dist: String,
    pub ano_mod: String,
    pub ano_fab: String,
    pub tp_pint: String,
    pub tp_veic: String,
    pub esp_veic: String,
    pub vin: String,
    pub cond_veic: String,
    pub c_mod: String,
    pub c_cor_denatran: String,
    pub lota: String,
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

/// Medicine details (med)
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct MedData {
    pub c_prod_anvisa: Option<String>,
    pub x_motivo_isencao: Option<String>,
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

/// Weapon details (arma)
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ArmaData {
    pub tp_arma: String,
    pub n_serie: String,
    pub n_cano: String,
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

/// Per-item observations (obsItem)
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ObsItemData {
    pub obs_cont: Option<ObsField>,
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

/// Per-item observation field (obsCont / obsFisco) with field name and text.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ObsField {
    pub x_campo: String,
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

/// Referenced DFe per item
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DFeReferenciadoData {
    pub chave_acesso: String,
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

/// Invoice item data
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct InvoiceItemData {
    pub item_number: u32,
    pub product_code: String,
    pub description: String,
    pub ncm: String,
    pub cfop: String,
    pub unit_of_measure: String,
    pub quantity: f64,
    pub unit_price: Cents,
    pub total_price: Cents,
    pub c_ean: Option<String>,
    pub c_ean_trib: Option<String>,
    pub cest: Option<String>,
    pub v_frete: Option<Cents>,
    pub v_seg: Option<Cents>,
    pub v_desc: Option<Cents>,
    pub v_outro: Option<Cents>,
    pub orig: Option<String>,
    // ICMS
    pub icms_cst: String,
    pub icms_rate: Rate,
    pub icms_amount: Cents,
    pub icms_mod_bc: Option<i64>,
    pub icms_red_bc: Option<Rate>,
    pub icms_mod_bc_st: Option<i64>,
    pub icms_p_mva_st: Option<Rate>,
    pub icms_red_bc_st: Option<Rate>,
    pub icms_v_bc_st: Option<Cents>,
    pub icms_p_icms_st: Option<Rate>,
    pub icms_v_icms_st: Option<Cents>,
    pub icms_v_icms_deson: Option<Cents>,
    pub icms_mot_des_icms: Option<i64>,
    pub icms_p_fcp: Option<Rate>,
    pub icms_v_fcp: Option<Cents>,
    pub icms_v_bc_fcp: Option<Cents>,
    pub icms_p_fcp_st: Option<Rate>,
    pub icms_v_fcp_st: Option<Cents>,
    pub icms_v_bc_fcp_st: Option<Cents>,
    pub icms_p_cred_sn: Option<Rate>,
    pub icms_v_cred_icms_sn: Option<Cents>,
    pub icms_v_icms_substituto: Option<Cents>,
    // PIS
    pub pis_cst: String,
    pub pis_v_bc: Option<Cents>,
    pub pis_p_pis: Option<Rate4>,
    pub pis_v_pis: Option<Cents>,
    pub pis_q_bc_prod: Option<i64>,
    pub pis_v_aliq_prod: Option<i64>,
    // COFINS
    pub cofins_cst: String,
    pub cofins_v_bc: Option<Cents>,
    pub cofins_p_cofins: Option<Rate4>,
    pub cofins_v_cofins: Option<Cents>,
    pub cofins_q_bc_prod: Option<i64>,
    pub cofins_v_aliq_prod: Option<i64>,
    // IPI
    pub ipi_cst: Option<String>,
    pub ipi_c_enq: Option<String>,
    pub ipi_v_bc: Option<Cents>,
    pub ipi_p_ipi: Option<Rate>,
    pub ipi_v_ipi: Option<Cents>,
    pub ipi_q_unid: Option<i64>,
    pub ipi_v_unid: Option<i64>,
    // II
    pub ii_v_bc: Option<Cents>,
    pub ii_v_desp_adu: Option<Cents>,
    pub ii_v_ii: Option<Cents>,
    pub ii_v_iof: Option<Cents>,
    // Product options
    pub rastro: Option<Vec<RastroData>>,
    pub veic_prod: Option<VeicProdData>,
    pub med: Option<MedData>,
    pub arma: Option<Vec<ArmaData>>,
    pub n_recopi: Option<String>,
    pub inf_ad_prod: Option<String>,
    pub obs_item: Option<ObsItemData>,
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

/// Third-party entity authorized to download the XML (autXML).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct AuthorizedXml {
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

/// Parameters for building an NFC-e QR Code URL
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct NfceQrCodeParams {
    pub access_key: String,
    pub version: QrCodeVersion,
    pub environment: SefazEnvironment,
    pub emission_type: EmissionType,
    pub qr_code_base_url: String,
    pub csc_token: Option<String>,
    pub csc_id: Option<String>,
    pub issued_at: Option<String>,
    pub total_value: Option<String>,
    pub total_icms: Option<String>,
    pub digest_value: Option<String>,
    pub dest_document: Option<String>,
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

/// Parameters for inserting QR Code into NFC-e XML
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PutQRTagParams {
    pub xml: String,
    pub csc_token: String,
    pub csc_id: String,
    pub version: String,
    pub qr_code_base_url: String,
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
