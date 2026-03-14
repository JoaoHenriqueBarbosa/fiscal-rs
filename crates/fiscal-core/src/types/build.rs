use super::{
    AdditionalInfo, AgropecuarioData, BillingData, CalculationMethod, CanaData, CompraGovData,
    ContingencyData, EmissionType, ExportData, IntermediaryData, InvoiceItemData, InvoiceModel,
    IssqnTotData, IssuerData, LocationData, PagAntecipadoData, PaymentCardDetail, PaymentData,
    PurchaseData, RecipientData, ReferenceDoc, RetTribData, SchemaVersion, SefazEnvironment,
    TechResponsibleData, TransportData,
};
use crate::newtypes::Cents;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

/// Complete data for building an NF-e/NFC-e XML document.
///
/// This struct contains all the fields needed to generate a fiscal document.
/// It can be used directly via [`crate::xml_builder::build_from_data`] for
/// FFI/binding scenarios where the typestate builder is not practical, or
/// indirectly via [`crate::xml_builder::InvoiceBuilder`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct InvoiceBuildData {
    #[serde(default)]
    pub schema_version: SchemaVersion,
    pub model: InvoiceModel,
    #[serde(default = "default_one")]
    pub series: u32,
    #[serde(default = "default_one")]
    pub number: u32,
    #[serde(default)]
    pub emission_type: EmissionType,
    pub environment: SefazEnvironment,
    #[serde(default = "default_now_br")]
    pub issued_at: DateTime<FixedOffset>,
    #[serde(default = "default_operation_nature")]
    pub operation_nature: String,
    pub issuer: IssuerData,
    pub recipient: Option<RecipientData>,
    #[serde(default)]
    pub items: Vec<InvoiceItemData>,
    #[serde(default)]
    pub payments: Vec<PaymentData>,
    pub change_amount: Option<Cents>,
    pub payment_card_details: Option<Vec<PaymentCardDetail>>,
    pub contingency: Option<ContingencyData>,
    pub exit_at: Option<DateTime<FixedOffset>>,
    // IDE overrides
    pub operation_type: Option<u8>,
    pub purpose_code: Option<u8>,
    pub destination_indicator: Option<String>,
    pub intermediary_indicator: Option<String>,
    pub emission_process: Option<String>,
    pub consumer_type: Option<String>,
    pub buyer_presence: Option<String>,
    pub print_format: Option<String>,
    pub ver_proc: Option<String>,
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
    pub issqn_tot: Option<IssqnTotData>,
    pub cana: Option<CanaData>,
    pub agropecuario: Option<AgropecuarioData>,
    pub compra_gov: Option<CompraGovData>,
    pub pag_antecipado: Option<PagAntecipadoData>,
    pub is_tot: Option<crate::tax_ibs_cbs::IsTotData>,
    pub ibs_cbs_tot: Option<crate::tax_ibs_cbs::IbsCbsTotData>,
    // vNFTot override (PL_010 only)
    pub v_nf_tot_override: Option<crate::newtypes::Cents>,
    // ASCII sanitization
    #[serde(default)]
    pub only_ascii: bool,
    #[serde(default)]
    pub calculation_method: CalculationMethod,
}

fn default_one() -> u32 {
    1
}

fn default_operation_nature() -> String {
    "VENDA".to_string()
}

fn default_now_br() -> DateTime<FixedOffset> {
    chrono::Utc::now().with_timezone(&FixedOffset::west_opt(3 * 3600).expect("valid offset"))
}

/// Third-party entity authorised to download the NF-e XML from the SEFAZ portal (`<autXML>`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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

/// Result of XML generation: the unsigned XML and its 44-digit access key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct InvoiceXmlResult {
    /// The unsigned NF-e/NFC-e XML document.
    pub xml: String,
    /// The 44-digit access key (`chave de acesso`).
    pub access_key: String,
}
