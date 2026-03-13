//! Core types, tax computation, and XML generation for Brazilian fiscal documents.
//!
//! `fiscal-core` is the foundation crate of the `fiscal-rs` workspace. It contains:
//!
//! - **Data types** ([`types`], [`newtypes`]) — strongly-typed structs and newtypes for
//!   NF-e / NFC-e document data, validated at construction time.
//! - **Tax computation** ([`tax_icms`], [`tax_pis_cofins_ipi`], [`tax_issqn`], [`tax_is`]) —
//!   build XML fragments for each Brazilian tax group and accumulate invoice totals.
//! - **XML builder** ([`xml_builder`]) — typestate builder for assembling a complete,
//!   schema-compliant NF-e / NFC-e XML document.
//! - **Protocol helpers** ([`complement`], [`contingency`], [`qrcode`]) — attach SEFAZ
//!   authorization protocols, manage contingency mode, and build NFC-e QR codes.
//! - **Utilities** ([`format_utils`], [`xml_utils`], [`gtin`], [`state_codes`],
//!   [`status_codes`], [`constants`]) — formatting, XML escaping, GTIN validation,
//!   IBGE state code lookups, and SEFAZ status-code constants.
//! - **Conversion** ([`convert`], [`standardize`]) — TXT-to-XML converter and
//!   XML type identification / JSON conversion.
//! - **Traits** ([`traits`]) — sealed `TaxCalculation` and `XmlSerializable` traits.

pub mod error;
pub use error::FiscalError;

/// Functions for attaching SEFAZ authorization protocols to signed XML documents.
pub mod complement;
/// Compile-time constants: namespaces, algorithm URIs, and payment type codes.
pub mod constants;
/// Contingency mode manager for NF-e fallback emission.
pub mod contingency;
/// SPED TXT-to-XML converter for NF-e documents.
pub mod convert;
/// Formatting helpers for monetary amounts, rates, and decimal numbers.
pub mod format_utils;
/// GTIN (barcode) validation and check-digit calculation.
pub mod gtin;
/// Parse-don't-validate newtypes for monetary amounts, tax rates, access keys, and state codes.
pub mod newtypes;
/// NFC-e QR code URL builder and XML injection.
pub mod qrcode;
/// ASCII sanitization for XML text content (replaces accented characters).
pub mod sanitize;
/// NF-e XML document type identification and XML-to-JSON conversion.
pub mod standardize;
/// Brazilian state IBGE code lookup tables and helpers.
pub mod state_codes;
/// SEFAZ status code constants and valid-status sets.
pub mod status_codes;
/// Internal tax element types used by tax computation modules.
pub mod tax_element;
/// IBS/CBS (Imposto sobre Bens e Servicos / Contribuicao sobre Bens e Servicos) XML generation.
pub mod tax_ibs_cbs;
/// ICMS tax computation and XML generation (CST and CSOSN variants).
pub mod tax_icms;
/// IS (Imposto Seletivo) XML generation.
pub mod tax_is;
/// ISSQN (municipal service tax) XML generation.
pub mod tax_issqn;
/// PIS, COFINS, IPI, and II tax computation and XML generation.
pub mod tax_pis_cofins_ipi;
/// Timezone lookup by Brazilian state (UF).
pub mod timezone;
/// Public data structures for NF-e / NFC-e documents.
pub mod types;
/// Typestate XML builder for NF-e / NFC-e documents.
pub mod xml_builder;
/// XML building primitives: `tag`, `escape_xml`, and `extract_xml_tag_value`.
pub mod xml_utils;

/// Sealed trait infrastructure for preventing external implementations.
pub mod sealed;
/// Sealed public traits: [`TaxCalculation`](traits::TaxCalculation) and [`XmlSerializable`](traits::XmlSerializable).
pub mod traits;
