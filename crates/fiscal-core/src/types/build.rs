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

/// Internal data bag assembled by [`InvoiceBuilder`] and consumed by sub-modules.
///
/// Not part of the public API — callers should use the builder.
#[derive(Debug, Clone)]
pub(crate) struct InvoiceBuildData {
    pub schema_version: SchemaVersion,
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
    pub only_ascii: bool,
    pub calculation_method: CalculationMethod,
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

/// Internal result of XML generation, consumed by the builder.
#[derive(Debug, Clone)]
pub(crate) struct InvoiceXmlResult {
    pub xml: String,
    pub access_key: String,
}
