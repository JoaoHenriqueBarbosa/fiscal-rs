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

mod additional;
mod billing;
mod build;
mod certificate;
mod enums;
mod issuer;
mod item;
mod optional;
mod payment;
mod product;
mod qrcode_params;
mod recipient;
mod retained;
mod totals;
mod transport;

// ── Re-exports: enums ──────────────────────────────────────────────────────
pub use enums::{
    CalculationMethod, ContingencyType, EmissionType, InvoiceModel, QrCodeVersion, SchemaVersion,
    SefazEnvironment, TaxRegime,
};

// ── Re-exports: certificate ────────────────────────────────────────────────
pub use certificate::{AccessKeyParams, CertificateData, CertificateInfo};

// ── Re-exports: issuer ─────────────────────────────────────────────────────
pub use issuer::IssuerData;

// ── Re-exports: recipient ──────────────────────────────────────────────────
pub use recipient::{ContingencyData, RecipientData};

// ── Re-exports: payment ────────────────────────────────────────────────────
pub use payment::{PaymentCardDetail, PaymentData, ReferenceDoc};

// ── Re-exports: transport ──────────────────────────────────────────────────
pub use transport::{CarrierData, RetainedIcmsTransp, TransportData, VehicleData, VolumeData};

// ── Re-exports: billing ────────────────────────────────────────────────────
pub use billing::{BillingData, BillingInvoice, Installment};

// ── Re-exports: additional ─────────────────────────────────────────────────
pub use additional::{
    AdditionalInfo, ExportData, FieldText, IntermediaryData, LocationData, ProcessRef,
    PurchaseData, TechResponsibleData,
};

// ── Re-exports: retained ───────────────────────────────────────────────────
pub use retained::{GCredData, RetTribData};

// ── Re-exports: product ────────────────────────────────────────────────────
pub use product::{
    AdiData, ArmaData, CideData, CombData, DFeReferenciadoData, DetExportData, DiData,
    EncerranteData, ImpostoDevolData, MedData, ObsField, ObsItemData, OrigCombData, RastroData,
    VeicProdData,
};

// ── Re-exports: totals ─────────────────────────────────────────────────────
pub use totals::IssqnTotData;

// ── Re-exports: item ───────────────────────────────────────────────────────
pub use item::InvoiceItemData;

// ── Re-exports: build ──────────────────────────────────────────────────────
pub use build::AuthorizedXml;
pub(crate) use build::{InvoiceBuildData, InvoiceXmlResult};

// ── Re-exports: qrcode_params ──────────────────────────────────────────────
pub use qrcode_params::{NfceQrCodeParams, PutQRTagParams};

// ── Re-exports: optional ───────────────────────────────────────────────────
pub use optional::{
    AgropecuarioData, AgropecuarioDefensivoData, AgropecuarioGuiaData, CanaData, CompraGovData,
    DeducData, ForDiaData, PagAntecipadoData,
};
