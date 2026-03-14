use napi_derive::napi;
use serde::Deserialize;

use fiscal_core::newtypes::Cents;
use fiscal_core::types::*;
use fiscal_core::xml_builder::InvoiceBuilder;

/// Build an NF-e/NFC-e XML from a configuration object.
///
/// Accepts the full invoice data as a single JSON object and returns
/// `{ xml: string, accessKey: string }`.
#[napi(ts_return_type = "{ xml: string; accessKey: string }")]
pub fn build_invoice(config: serde_json::Value) -> napi::Result<serde_json::Value> {
    let cfg: BuildInvoiceConfig = serde_json::from_value(config)
        .map_err(|e| napi::Error::from_reason(format!("Invalid config: {e}")))?;

    let mut builder = InvoiceBuilder::new(cfg.issuer, cfg.environment, cfg.model);

    if let Some(v) = cfg.series {
        builder = builder.series(v);
    }
    if let Some(v) = cfg.invoice_number {
        builder = builder.invoice_number(v);
    }
    if let Some(v) = cfg.emission_type {
        builder = builder.emission_type(v);
    }
    if let Some(v) = cfg.schema_version {
        builder = builder.schema_version(v);
    }
    if let Some(ref v) = cfg.issued_at {
        let dt = chrono::DateTime::parse_from_rfc3339(v)
            .map_err(|e| napi::Error::from_reason(format!("Invalid issued_at: {e}")))?;
        builder = builder.issued_at(dt);
    }
    if let Some(v) = cfg.operation_nature {
        builder = builder.operation_nature(v);
    }
    if let Some(v) = cfg.add_item {
        builder = builder.add_item(v);
    }
    builder = builder.items(cfg.items);
    if let Some(v) = cfg.recipient {
        builder = builder.recipient(v);
    }
    builder = builder.payments(cfg.payments);
    if let Some(v) = cfg.change_amount {
        builder = builder.change_amount(v);
    }
    if let Some(v) = cfg.payment_card_details {
        builder = builder.payment_card_details(v);
    }
    if let Some(v) = cfg.contingency {
        builder = builder.contingency(v);
    }
    if let Some(ref v) = cfg.exit_at {
        let dt = chrono::DateTime::parse_from_rfc3339(v)
            .map_err(|e| napi::Error::from_reason(format!("Invalid exit_at: {e}")))?;
        builder = builder.exit_at(dt);
    }
    if let Some(v) = cfg.operation_type {
        builder = builder.operation_type(v);
    }
    if let Some(v) = cfg.purpose_code {
        builder = builder.purpose_code(v);
    }
    if let Some(v) = cfg.intermediary_indicator {
        builder = builder.intermediary_indicator(v);
    }
    if let Some(v) = cfg.emission_process {
        builder = builder.emission_process(v);
    }
    if let Some(v) = cfg.consumer_type {
        builder = builder.consumer_type(v);
    }
    if let Some(v) = cfg.buyer_presence {
        builder = builder.buyer_presence(v);
    }
    if let Some(v) = cfg.print_format {
        builder = builder.print_format(v);
    }
    if let Some(v) = cfg.destination_indicator {
        builder = builder.destination_indicator(v);
    }
    if let Some(v) = cfg.ver_proc {
        builder = builder.ver_proc(v);
    }
    if let Some(v) = cfg.references {
        builder = builder.references(v);
    }
    if let Some(v) = cfg.transport {
        builder = builder.transport(v);
    }
    if let Some(v) = cfg.billing {
        builder = builder.billing(v);
    }
    if let Some(v) = cfg.withdrawal {
        builder = builder.withdrawal(v);
    }
    if let Some(v) = cfg.delivery {
        builder = builder.delivery(v);
    }
    if let Some(v) = cfg.authorized_xml {
        builder = builder.authorized_xml(v);
    }
    if let Some(v) = cfg.additional_info {
        builder = builder.additional_info(v);
    }
    if let Some(v) = cfg.intermediary {
        builder = builder.intermediary(v);
    }
    if let Some(v) = cfg.ret_trib {
        builder = builder.ret_trib(v);
    }
    if let Some(v) = cfg.tech_responsible {
        builder = builder.tech_responsible(v);
    }
    if let Some(v) = cfg.purchase {
        builder = builder.purchase(v);
    }
    if let Some(v) = cfg.export {
        builder = builder.export(v);
    }
    if let Some(v) = cfg.issqn_tot {
        builder = builder.issqn_tot(v);
    }
    if let Some(v) = cfg.cana {
        builder = builder.cana(v);
    }
    if let Some(v) = cfg.agropecuario {
        builder = builder.agropecuario(v);
    }
    if let Some(v) = cfg.compra_gov {
        builder = builder.compra_gov(v);
    }
    if let Some(v) = cfg.pag_antecipado {
        builder = builder.pag_antecipado(v);
    }
    if let Some(v) = cfg.is_tot {
        builder = builder.is_tot(v);
    }
    if let Some(v) = cfg.ibs_cbs_tot {
        builder = builder.ibs_cbs_tot(v);
    }
    if let Some(v) = cfg.only_ascii {
        builder = builder.only_ascii(v);
    }
    if let Some(v) = cfg.calculation_method {
        builder = builder.calculation_method(v);
    }
    if let Some(v) = cfg.v_nf_tot_override {
        builder = builder.v_nf_tot_override(v);
    }

    let built = builder
        .build()
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    Ok(serde_json::json!({
        "xml": built.xml(),
        "accessKey": built.access_key(),
    }))
}

/// Build and sign an NF-e/NFC-e XML in one step.
#[napi(ts_return_type = "{ xml: string; signedXml: string; accessKey: string }")]
pub fn build_and_sign_invoice(
    config: serde_json::Value,
    private_key: String,
    certificate: String,
) -> napi::Result<serde_json::Value> {
    let result = build_invoice(config)?;
    let xml = result["xml"].as_str().unwrap();
    let access_key = result["accessKey"].as_str().unwrap().to_string();

    let signed_xml = fiscal_crypto::certificate::sign_xml(xml, &private_key, &certificate)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    Ok(serde_json::json!({
        "xml": xml,
        "signedXml": signed_xml,
        "accessKey": access_key,
    }))
}

// ── Config (auto-generated from InvoiceBuilder<Draft> setters) ──

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuildInvoiceConfig {
    // Required
    issuer: IssuerData,
    environment: SefazEnvironment,
    model: InvoiceModel,
    series: Option<u32>,
    invoice_number: Option<u32>,
    emission_type: Option<EmissionType>,
    schema_version: Option<SchemaVersion>,
    /// ISO 8601 string
    issued_at: Option<String>,
    operation_nature: Option<String>,
    add_item: Option<InvoiceItemData>,
    items: Vec<InvoiceItemData>,
    recipient: Option<RecipientData>,
    payments: Vec<PaymentData>,
    change_amount: Option<Cents>,
    payment_card_details: Option<Vec<PaymentCardDetail>>,
    contingency: Option<ContingencyData>,
    /// ISO 8601 string
    exit_at: Option<String>,
    operation_type: Option<u8>,
    purpose_code: Option<u8>,
    intermediary_indicator: Option<String>,
    emission_process: Option<String>,
    consumer_type: Option<String>,
    buyer_presence: Option<String>,
    print_format: Option<String>,
    destination_indicator: Option<String>,
    ver_proc: Option<String>,
    references: Option<Vec<ReferenceDoc>>,
    transport: Option<TransportData>,
    billing: Option<BillingData>,
    withdrawal: Option<LocationData>,
    delivery: Option<LocationData>,
    authorized_xml: Option<Vec<AuthorizedXml>>,
    additional_info: Option<AdditionalInfo>,
    intermediary: Option<IntermediaryData>,
    ret_trib: Option<RetTribData>,
    tech_responsible: Option<TechResponsibleData>,
    purchase: Option<PurchaseData>,
    export: Option<ExportData>,
    issqn_tot: Option<IssqnTotData>,
    cana: Option<CanaData>,
    agropecuario: Option<AgropecuarioData>,
    compra_gov: Option<CompraGovData>,
    pag_antecipado: Option<PagAntecipadoData>,
    is_tot: Option<fiscal_core::tax_ibs_cbs::IsTotData>,
    ibs_cbs_tot: Option<fiscal_core::tax_ibs_cbs::IbsCbsTotData>,
    only_ascii: Option<bool>,
    calculation_method: Option<fiscal_core::types::CalculationMethod>,
    v_nf_tot_override: Option<Cents>,
}
