//! Types and XML generation for the Brazilian **MDF-e** (Manifesto Eletrônico
//! de Documentos Fiscais), fiscal document **model 58**, leiaute **3.00**.
//!
//! `fiscal-mdfe` mirrors the structure of [`fiscal_core`]: strongly-typed data
//! structures for each XML block, an access-key builder, and a string-based XML
//! builder that produces a schema-ordered `<MDFe>` document.
//!
//! All four transport modals are supported via `Modal`: road
//! (`infModal/rodo`), air (`aereo`), waterway (`aquav`), and rail (`ferrov`).
//!
//! # Example
//!
//! ```
//! use fiscal_mdfe::{build_mdfe_xml, types::*};
//! # fn demo(data: &MdfeBuildData) -> Result<(), fiscal_core::FiscalError> {
//! # let data: &MdfeBuildData = data;
//! let xml = build_mdfe_xml(data)?;
//! assert!(xml.contains("<MDFe"));
//! # Ok(()) }
//! ```

/// 44-digit MDF-e access key (chave de acesso) generation and parsing.
pub mod access_key;
/// String-based XML builder that assembles a complete `<MDFe>` document.
pub mod builder;
/// Enveloped XML-DSig signing of the `<infMDFe>` element (RSA-SHA1).
pub mod signing;
/// Public data structures for the MDF-e XML blocks.
pub mod types;
/// Structural validation of an MDF-e XML before transmission.
pub mod validate;

pub use access_key::{MdfeAccessKey, build_mdfe_access_key};
pub use builder::build_mdfe_xml;
pub use signing::{sign_mdfe_xml, sign_mdfe_xml_with_algorithm};
pub use validate::validate_mdfe_xml;

/// MDF-e XML namespace (`xmlns`).
pub const MDFE_NAMESPACE: &str = "http://www.portalfiscal.inf.br/mdfe";

/// MDF-e layout version emitted by this crate.
pub const MDFE_VERSION: &str = "3.00";

/// Fiscal document model for the MDF-e.
pub const MDFE_MODEL: &str = "58";
