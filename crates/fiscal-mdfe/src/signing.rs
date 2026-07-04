//! Digital signing of MDF-e (model 58) XML documents.
//!
//! The MDF-e uses the **same** enveloped XML-DSig as the NF-e — RSA-SHA1 with
//! a SHA-1 digest over `<infMDFe>`, with the `<Signature>` inserted as a child
//! of `<MDFe>`. SEFAZ rejects SHA-256 for the MDF-e, so SHA-1 is the default.
//!
//! The signing mechanics are reused verbatim from [`fiscal_crypto`]; this
//! module re-exports the MDF-e entry points so callers depend only on
//! `fiscal-mdfe`.

pub use fiscal_crypto::certificate::SignatureAlgorithm;
pub use fiscal_crypto::certificate::{sign_mdfe_xml, sign_mdfe_xml_with_algorithm};
