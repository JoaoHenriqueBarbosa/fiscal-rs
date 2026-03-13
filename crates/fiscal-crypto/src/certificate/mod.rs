//! Digital certificate loading and XML signing for Brazilian fiscal documents.
//!
//! This module is split into:
//! - [`pfx`] — PFX/PKCS#12 loading, parsing, and certificate info extraction
//! - [`sign`] — XML-DSig signing for NF-e, events, and inutilizacao
//! - [`c14n`] — XML Canonicalization (C14N 1.0) and element helpers

mod c14n;
mod pfx;
mod sign;

#[cfg(test)]
mod tests;

pub use pfx::{SignatureAlgorithm, ensure_modern_pfx, get_certificate_info, load_certificate};
pub use sign::{
    sign_event_xml, sign_event_xml_with_algorithm, sign_inutilizacao_xml,
    sign_inutilizacao_xml_with_algorithm, sign_xml, sign_xml_with_algorithm,
};
