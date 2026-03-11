//! Typestate invoice builder for NF-e / NFC-e XML generation.
//!
//! ```text
//! InvoiceBuilder::new(issuer, env, model)   // Draft
//!     .series(1)
//!     .invoice_number(42)
//!     .add_item(item)
//!     .recipient(recipient)
//!     .payments(vec![payment])
//!     .build()?                              // Built
//!     .xml()                                 // &str
//! ```
//!
//! The typestate pattern ensures at compile time that `xml()` / `access_key()`
//! are only available after a successful `build()`.

use std::marker::PhantomData;

use chrono::{DateTime, FixedOffset};

use crate::FiscalError;
use crate::newtypes::Cents;
use crate::types::*;

// ── Typestate markers ────────────────────────────────────────────────────────

/// Marker: invoice is being assembled (setters available, no XML yet).
pub struct Draft;

/// Marker: invoice has been built (XML and access key available, no setters).
pub struct Built;

// ── Builder ──────────────────────────────────────────────────────────────────

/// Typestate builder for NF-e / NFC-e XML documents.
///
/// In the [`Draft`] state all setters are available.
/// Calling [`build()`](InvoiceBuilder::build) validates the data and
/// transitions to [`Built`], which exposes [`xml()`](InvoiceBuilder::xml)
/// and [`access_key()`](InvoiceBuilder::access_key).
pub struct InvoiceBuilder<State = Draft> {
    // Required from construction
    issuer: IssuerData,
    environment: SefazEnvironment,
    model: InvoiceModel,

    // Defaults provided, overridable
    series: u32,
    invoice_number: u32,
    emission_type: EmissionType,
    issued_at: DateTime<FixedOffset>,
    operation_nature: String,

    // Accumulated during Draft
    items: Vec<InvoiceItemData>,
    recipient: Option<RecipientData>,
    payments: Vec<PaymentData>,
    change_amount: Option<Cents>,
    payment_card_details: Option<Vec<PaymentCardDetail>>,
    contingency: Option<ContingencyData>,

    // IDE overrides
    operation_type: Option<u8>,
    purpose_code: Option<u8>,
    intermediary_indicator: Option<String>,
    emission_process: Option<String>,
    consumer_type: Option<String>,
    buyer_presence: Option<String>,
    print_format: Option<String>,
    references: Option<Vec<ReferenceDoc>>,

    // Optional groups
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

    // Present only after build
    result_xml: Option<String>,
    result_access_key: Option<String>,

    _state: PhantomData<State>,
}

// ── Draft methods (setters + build) ──────────────────────────────────────────

impl InvoiceBuilder<Draft> {
    /// Create a new builder in the [`Draft`] state.
    ///
    /// The three arguments are required; everything else has sensible defaults
    /// or is optional.
    pub fn new(issuer: IssuerData, environment: SefazEnvironment, model: InvoiceModel) -> Self {
        let now = chrono::Utc::now()
            .with_timezone(&FixedOffset::west_opt(3 * 3600).expect("valid offset"));

        Self {
            issuer,
            environment,
            model,
            series: 1,
            invoice_number: 1,
            emission_type: EmissionType::Normal,
            issued_at: now,
            operation_nature: "VENDA".to_string(),
            items: Vec::new(),
            recipient: None,
            payments: Vec::new(),
            change_amount: None,
            payment_card_details: None,
            contingency: None,
            operation_type: None,
            purpose_code: None,
            intermediary_indicator: None,
            emission_process: None,
            consumer_type: None,
            buyer_presence: None,
            print_format: None,
            references: None,
            transport: None,
            billing: None,
            withdrawal: None,
            delivery: None,
            authorized_xml: None,
            additional_info: None,
            intermediary: None,
            ret_trib: None,
            tech_responsible: None,
            purchase: None,
            export: None,
            result_xml: None,
            result_access_key: None,
            _state: PhantomData,
        }
    }

    // ── Chainable setters ────────────────────────────────────────────────

    /// Set the invoice series (default: 1).
    pub fn series(mut self, s: u32) -> Self {
        self.series = s;
        self
    }

    /// Set the invoice number (default: 1).
    pub fn invoice_number(mut self, n: u32) -> Self {
        self.invoice_number = n;
        self
    }

    /// Set the emission type (default: [`EmissionType::Normal`]).
    pub fn emission_type(mut self, et: EmissionType) -> Self {
        self.emission_type = et;
        self
    }

    /// Set the emission date/time (default: now in UTC-3).
    pub fn issued_at(mut self, dt: DateTime<FixedOffset>) -> Self {
        self.issued_at = dt;
        self
    }

    /// Set the operation nature (default: `"VENDA"`).
    pub fn operation_nature(mut self, n: impl Into<String>) -> Self {
        self.operation_nature = n.into();
        self
    }

    /// Add one item to the invoice.
    pub fn add_item(mut self, item: InvoiceItemData) -> Self {
        self.items.push(item);
        self
    }

    /// Set all items at once (replaces any previously added items).
    pub fn items(mut self, items: Vec<InvoiceItemData>) -> Self {
        self.items = items;
        self
    }

    /// Set the recipient (optional for NFC-e under R$200).
    pub fn recipient(mut self, r: RecipientData) -> Self {
        self.recipient = Some(r);
        self
    }

    /// Set the payment list.
    pub fn payments(mut self, p: Vec<PaymentData>) -> Self {
        self.payments = p;
        self
    }

    /// Set the change amount (vTroco).
    pub fn change_amount(mut self, c: Cents) -> Self {
        self.change_amount = Some(c);
        self
    }

    /// Set card payment details.
    pub fn payment_card_details(mut self, d: Vec<PaymentCardDetail>) -> Self {
        self.payment_card_details = Some(d);
        self
    }

    /// Set contingency data.
    pub fn contingency(mut self, c: ContingencyData) -> Self {
        self.contingency = Some(c);
        self
    }

    /// Override the operation type (tpNF, default: 1).
    pub fn operation_type(mut self, v: u8) -> Self {
        self.operation_type = Some(v);
        self
    }

    /// Override the invoice purpose code (finNFe, default: 1).
    pub fn purpose_code(mut self, v: u8) -> Self {
        self.purpose_code = Some(v);
        self
    }

    /// Set the intermediary indicator (indIntermed).
    pub fn intermediary_indicator(mut self, v: impl Into<String>) -> Self {
        self.intermediary_indicator = Some(v.into());
        self
    }

    /// Set the emission process (procEmi).
    pub fn emission_process(mut self, v: impl Into<String>) -> Self {
        self.emission_process = Some(v.into());
        self
    }

    /// Set the consumer type (indFinal).
    pub fn consumer_type(mut self, v: impl Into<String>) -> Self {
        self.consumer_type = Some(v.into());
        self
    }

    /// Set the buyer presence indicator (indPres).
    pub fn buyer_presence(mut self, v: impl Into<String>) -> Self {
        self.buyer_presence = Some(v.into());
        self
    }

    /// Set the DANFE print format (tpImp).
    pub fn print_format(mut self, v: impl Into<String>) -> Self {
        self.print_format = Some(v.into());
        self
    }

    /// Set referenced documents (NFref).
    pub fn references(mut self, refs: Vec<ReferenceDoc>) -> Self {
        self.references = Some(refs);
        self
    }

    /// Set transport data.
    pub fn transport(mut self, t: TransportData) -> Self {
        self.transport = Some(t);
        self
    }

    /// Set billing data (cobr).
    pub fn billing(mut self, b: BillingData) -> Self {
        self.billing = Some(b);
        self
    }

    /// Set the withdrawal/pickup location (retirada).
    pub fn withdrawal(mut self, w: LocationData) -> Self {
        self.withdrawal = Some(w);
        self
    }

    /// Set the delivery location (entrega).
    pub fn delivery(mut self, d: LocationData) -> Self {
        self.delivery = Some(d);
        self
    }

    /// Set authorized XML downloaders (autXML).
    pub fn authorized_xml(mut self, a: Vec<AuthorizedXml>) -> Self {
        self.authorized_xml = Some(a);
        self
    }

    /// Set additional info (infAdic).
    pub fn additional_info(mut self, a: AdditionalInfo) -> Self {
        self.additional_info = Some(a);
        self
    }

    /// Set intermediary data (infIntermed).
    pub fn intermediary(mut self, i: IntermediaryData) -> Self {
        self.intermediary = Some(i);
        self
    }

    /// Set retained taxes (retTrib).
    pub fn ret_trib(mut self, r: RetTribData) -> Self {
        self.ret_trib = Some(r);
        self
    }

    /// Set tech responsible (infRespTec).
    pub fn tech_responsible(mut self, t: TechResponsibleData) -> Self {
        self.tech_responsible = Some(t);
        self
    }

    /// Set purchase data (compra).
    pub fn purchase(mut self, p: PurchaseData) -> Self {
        self.purchase = Some(p);
        self
    }

    /// Set export data (exporta).
    pub fn export(mut self, e: ExportData) -> Self {
        self.export = Some(e);
        self
    }

    /// Validate and build the XML, transitioning to [`Built`].
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError`] if:
    /// - The issuer state code is unknown
    /// - Tax data is invalid
    pub fn build(self) -> Result<InvoiceBuilder<Built>, FiscalError> {
        let data = InvoiceBuildData {
            model: self.model,
            series: self.series,
            number: self.invoice_number,
            emission_type: self.emission_type,
            environment: self.environment,
            issued_at: self.issued_at,
            operation_nature: self.operation_nature,
            issuer: self.issuer,
            recipient: self.recipient,
            items: self.items,
            payments: self.payments,
            change_amount: self.change_amount,
            payment_card_details: self.payment_card_details,
            contingency: self.contingency,
            operation_type: self.operation_type,
            purpose_code: self.purpose_code,
            intermediary_indicator: self.intermediary_indicator,
            emission_process: self.emission_process,
            consumer_type: self.consumer_type,
            buyer_presence: self.buyer_presence,
            print_format: self.print_format,
            references: self.references,
            transport: self.transport,
            billing: self.billing,
            withdrawal: self.withdrawal,
            delivery: self.delivery,
            authorized_xml: self.authorized_xml,
            additional_info: self.additional_info,
            intermediary: self.intermediary,
            ret_trib: self.ret_trib,
            tech_responsible: self.tech_responsible,
            purchase: self.purchase,
            export: self.export,
        };

        let result = super::generate_xml(&data)?;

        Ok(InvoiceBuilder {
            issuer: data.issuer,
            environment: data.environment,
            model: data.model,
            series: data.series,
            invoice_number: data.number,
            emission_type: data.emission_type,
            issued_at: data.issued_at,
            operation_nature: data.operation_nature,
            items: data.items,
            recipient: data.recipient,
            payments: data.payments,
            change_amount: data.change_amount,
            payment_card_details: data.payment_card_details,
            contingency: data.contingency,
            operation_type: data.operation_type,
            purpose_code: data.purpose_code,
            intermediary_indicator: data.intermediary_indicator,
            emission_process: data.emission_process,
            consumer_type: data.consumer_type,
            buyer_presence: data.buyer_presence,
            print_format: data.print_format,
            references: data.references,
            transport: data.transport,
            billing: data.billing,
            withdrawal: data.withdrawal,
            delivery: data.delivery,
            authorized_xml: data.authorized_xml,
            additional_info: data.additional_info,
            intermediary: data.intermediary,
            ret_trib: data.ret_trib,
            tech_responsible: data.tech_responsible,
            purchase: data.purchase,
            export: data.export,
            result_xml: Some(result.xml),
            result_access_key: Some(result.access_key),
            _state: PhantomData,
        })
    }
}

// ── Built methods (accessors) ────────────────────────────────────────────────

impl InvoiceBuilder<Built> {
    /// The unsigned XML string.
    pub fn xml(&self) -> &str {
        self.result_xml
            .as_deref()
            .expect("Built state always has XML")
    }

    /// The 44-digit access key.
    pub fn access_key(&self) -> &str {
        self.result_access_key
            .as_deref()
            .expect("Built state always has access key")
    }
}
