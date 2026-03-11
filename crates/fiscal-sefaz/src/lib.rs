//! SEFAZ web-service integration for NF-e / NFC-e authorization and events.
//!
//! This crate provides:
//!
//! - [`urls`] — endpoint URL resolution for all SEFAZ web services by state and environment.
//! - [`request_builders`] — XML request builders for authorization, cancellation, events, etc.
//! - [`response_parsers`] — SEFAZ XML response parsing and typed result structs.
//! - [`services`] — SOAP metadata (method, operation, version) for each web service.
//! - [`client`] — async SEFAZ client with mTLS authentication (feature-gated).
//!
//! The [`status_codes`] module from `fiscal-core` is re-exported for convenience.

/// XML request builders for SEFAZ web services.
pub mod request_builders;
/// SEFAZ XML response parsers and typed result structs.
pub mod response_parsers;
/// SOAP metadata for each NF-e/NFC-e web service operation.
pub mod services;
/// SEFAZ endpoint URL resolution by UF and environment.
pub mod urls;

mod soap;

/// Async SEFAZ web service client with mTLS authentication.
///
/// Requires the `client` feature (enabled by default).
#[cfg(feature = "client")]
pub mod client;

// Re-export status_codes from fiscal-core for convenience
pub use fiscal_core::status_codes;
