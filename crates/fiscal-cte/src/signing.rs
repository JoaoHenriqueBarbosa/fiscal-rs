//! Digital signing of CT-e (model 57) XML documents.
//!
//! The CT-e uses the **same** enveloped XML-DSig as the NF-e/MDF-e — RSA-SHA1
//! with a SHA-1 digest over `<infCte>`, with the `<Signature>` inserted as a
//! child of `<CTe>`. SEFAZ rejects SHA-256 for the CT-e, so SHA-1 is the
//! default.
//!
//! The signing mechanics are reused verbatim from [`fiscal_crypto`]; this
//! module re-exports the CT-e entry points so callers depend only on
//! `fiscal-cte`.

pub use fiscal_crypto::certificate::SignatureAlgorithm;
pub use fiscal_crypto::certificate::{
    sign_bpe_xml, sign_bpe_xml_with_algorithm, sign_cte_xml, sign_cte_xml_with_algorithm,
    sign_cteos_xml, sign_cteos_xml_with_algorithm, sign_dps_xml, sign_dps_xml_with_algorithm,
    sign_gtve_xml, sign_gtve_xml_with_algorithm, sign_nfse_evento_xml,
};
