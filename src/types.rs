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
pub struct CertificateData {
    pub private_key: String,
    pub certificate: String,
    pub pfx_buffer: Vec<u8>,
    pub passphrase: String,
}

/// Certificate info for display purposes
#[derive(Debug, Clone)]
pub struct CertificateInfo {
    pub common_name: String,
    pub valid_from: NaiveDate,
    pub valid_until: NaiveDate,
    pub serial_number: String,
    pub issuer: String,
}

/// Access key components for NF-e/NFC-e
#[derive(Debug, Clone)]
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

/// Issuer data (from fiscal settings)
#[derive(Debug, Clone)]
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

/// Recipient data (optional for NFC-e under R$200)
#[derive(Debug, Clone, Default)]
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

/// Contingency data for fallback emission
#[derive(Debug, Clone)]
pub struct ContingencyData {
    pub contingency_type: ContingencyType,
    pub reason: String,
    pub at: DateTime<FixedOffset>,
}

/// Payment data for invoice
#[derive(Debug, Clone)]
pub struct PaymentData {
    pub method: String,
    pub amount: Cents,
}

/// Payment card details
#[derive(Debug, Clone, Default)]
pub struct PaymentCardDetail {
    pub integ_type: Option<String>,
    pub card_tax_id: Option<String>,
    pub card_brand: Option<String>,
    pub auth_code: Option<String>,
}

/// Referenced document types
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ReferenceDoc {
    Nfe { access_key: String },
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
    Cte { access_key: String },
    Ecf {
        model: String,
        ecf_number: String,
        coo_number: String,
    },
}

/// Transport data
#[derive(Debug, Clone, Default)]
pub struct TransportData {
    pub freight_mode: String,
    pub carrier: Option<CarrierData>,
    pub vehicle: Option<VehicleData>,
    pub trailers: Option<Vec<VehicleData>>,
    pub volumes: Option<Vec<VolumeData>>,
    pub retained_icms: Option<RetainedIcmsTransp>,
}

/// Carrier (transportadora) identification for freight transport.
#[derive(Debug, Clone, Default)]
pub struct CarrierData {
    pub tax_id: Option<String>,
    pub name: Option<String>,
    pub state_tax_id: Option<String>,
    pub state_code: Option<String>,
    pub address: Option<String>,
}

/// Vehicle data for transport (veicTransp / reboque).
#[derive(Debug, Clone)]
pub struct VehicleData {
    pub plate: String,
    pub state_code: String,
    pub rntc: Option<String>,
}

/// Volume data for transported goods (vol).
#[derive(Debug, Clone, Default)]
pub struct VolumeData {
    pub quantity: Option<u32>,
    pub species: Option<String>,
    pub brand: Option<String>,
    pub number: Option<String>,
    pub net_weight: Option<f64>,
    pub gross_weight: Option<f64>,
    pub seals: Option<Vec<String>>,
}

/// Retained ICMS on transport services (retTransp).
#[derive(Debug, Clone)]
pub struct RetainedIcmsTransp {
    pub v_bc_ret: Cents,
    pub p_icms_ret: Rate,
    pub v_icms_ret: Cents,
    pub cfop: String,
    /// IBGE city code for the transport tax event municipality.
    pub city_code: IbgeCode,
}

/// Billing data (cobr)
#[derive(Debug, Clone, Default)]
pub struct BillingData {
    pub invoice: Option<BillingInvoice>,
    pub installments: Option<Vec<Installment>>,
}

/// Billing invoice header (fat) with original, discount, and net values.
#[derive(Debug, Clone)]
pub struct BillingInvoice {
    pub number: String,
    pub original_value: Cents,
    pub discount_value: Option<Cents>,
    pub net_value: Cents,
}

/// Billing installment (dup) with number, due date, and value.
#[derive(Debug, Clone)]
pub struct Installment {
    pub number: String,
    pub due_date: String,
    pub value: Cents,
}

/// Withdrawal/pickup or delivery location
#[derive(Debug, Clone)]
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

/// Additional info (infAdic)
#[derive(Debug, Clone, Default)]
pub struct AdditionalInfo {
    pub taxpayer_note: Option<String>,
    pub tax_authority_note: Option<String>,
    pub contributor_obs: Option<Vec<FieldText>>,
    pub fiscal_obs: Option<Vec<FieldText>>,
    pub process_refs: Option<Vec<ProcessRef>>,
}

/// Generic field/text pair used in contributor and fiscal observations.
#[derive(Debug, Clone)]
pub struct FieldText {
    pub field: String,
    pub text: String,
}

/// Referenced administrative or judicial process (procRef).
#[derive(Debug, Clone)]
pub struct ProcessRef {
    pub number: String,
    pub origin: String,
}

/// Intermediary info (infIntermed)
#[derive(Debug, Clone)]
pub struct IntermediaryData {
    pub tax_id: String,
    pub id_cad_int_tran: Option<String>,
}

/// Tech responsible (infRespTec)
#[derive(Debug, Clone)]
pub struct TechResponsibleData {
    pub tax_id: String,
    pub contact: String,
    pub email: String,
    pub phone: Option<String>,
}

/// Purchase data (compra)
#[derive(Debug, Clone, Default)]
pub struct PurchaseData {
    pub order_number: Option<String>,
    pub contract_number: Option<String>,
    pub purchase_note: Option<String>,
}

/// Export data (exporta)
#[derive(Debug, Clone)]
pub struct ExportData {
    pub exit_state: String,
    pub export_location: String,
    pub dispatch_location: Option<String>,
}

/// Retained taxes (retTrib)
#[derive(Debug, Clone, Default)]
pub struct RetTribData {
    pub v_ret_pis: Option<Cents>,
    pub v_ret_cofins: Option<Cents>,
    pub v_ret_csll: Option<Cents>,
    pub v_bc_irrf: Option<Cents>,
    pub v_irrf: Option<Cents>,
    pub v_bc_ret_prev: Option<Cents>,
    pub v_ret_prev: Option<Cents>,
}

/// Batch tracking (rastro)
#[derive(Debug, Clone)]
pub struct RastroData {
    pub n_lote: String,
    pub q_lote: f64,
    pub d_fab: String,
    pub d_val: String,
    pub c_agreg: Option<String>,
}

/// Vehicle details (veicProd)
#[derive(Debug, Clone)]
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

/// Medicine details (med)
#[derive(Debug, Clone)]
pub struct MedData {
    pub c_prod_anvisa: Option<String>,
    pub x_motivo_isencao: Option<String>,
    pub v_pmc: Cents,
}

/// Weapon details (arma)
#[derive(Debug, Clone)]
pub struct ArmaData {
    pub tp_arma: String,
    pub n_serie: String,
    pub n_cano: String,
    pub descr: String,
}

/// Per-item observations (obsItem)
#[derive(Debug, Clone, Default)]
pub struct ObsItemData {
    pub obs_cont: Option<ObsField>,
    pub obs_fisco: Option<ObsField>,
}

/// Per-item observation field (obsCont / obsFisco) with field name and text.
#[derive(Debug, Clone)]
pub struct ObsField {
    pub x_campo: String,
    pub x_texto: String,
}

/// Referenced DFe per item
#[derive(Debug, Clone)]
pub struct DFeReferenciadoData {
    pub chave_acesso: String,
    pub n_item: Option<String>,
}

/// Invoice item data
#[derive(Debug, Clone)]
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

/// Data needed to build an invoice XML
#[derive(Debug, Clone)]
pub struct InvoiceBuildData {
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
pub struct AuthorizedXml {
    pub tax_id: String,
}

/// Result of building an invoice XML
#[derive(Debug, Clone)]
pub struct InvoiceXmlResult {
    pub xml: String,
    pub access_key: String,
}

/// Parameters for building an NFC-e QR Code URL
#[derive(Debug, Clone)]
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

/// Parameters for inserting QR Code into NFC-e XML
#[derive(Debug, Clone)]
pub struct PutQRTagParams {
    pub xml: String,
    pub csc_token: String,
    pub csc_id: String,
    pub version: String,
    pub qr_code_base_url: String,
    pub url_chave: String,
}
