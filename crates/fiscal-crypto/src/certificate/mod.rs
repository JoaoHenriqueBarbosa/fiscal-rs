//! Digital certificate loading and XML signing for Brazilian fiscal documents.
//!
//! This module is split into:
//! - `pfx` — PFX/PKCS#12 loading, parsing, and certificate info extraction
//! - `sign` — XML-DSig signing for NF-e, CT-e family, MDF-e, events, inutilização
//! - `c14n` — XML Canonicalization (C14N 1.0) and element helpers

mod c14n;
mod pfx;
mod sign;

#[cfg(test)]
mod tests;

pub use pfx::{SignatureAlgorithm, ensure_modern_pfx, get_certificate_info, load_certificate};
pub use sign::{
    sign_bpe_xml, sign_bpe_xml_with_algorithm, sign_cte_event_xml,
    sign_cte_event_xml_with_algorithm, sign_cte_xml, sign_cte_xml_with_algorithm, sign_cteos_xml,
    sign_cteos_xml_with_algorithm, sign_dps_xml, sign_dps_xml_with_algorithm, sign_event_xml,
    sign_event_xml_with_algorithm, sign_gtve_xml, sign_gtve_xml_with_algorithm,
    sign_inutilizacao_xml, sign_inutilizacao_xml_with_algorithm, sign_mdfe_xml,
    sign_mdfe_xml_with_algorithm, sign_nfse_evento_xml, sign_xml, sign_xml_with_algorithm,
};
