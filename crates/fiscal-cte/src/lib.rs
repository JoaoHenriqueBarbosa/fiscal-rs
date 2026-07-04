//! Types and XML generation for the Brazilian **CT-e** (Conhecimento de
//! Transporte Eletrônico), fiscal document **model 57**, leiaute **4.00**,
//! and related transport documents:
//!
//! - **CT-e OS** (model 67) — Outros Serviços
//! - **GTV-e** (model 64) — Guia de Transporte de Valores
//! - **BP-e** (model 63) — Bilhete de Passagem Eletrônico
//!
//! For the NFS-e Nacional (DPS 1.01), see the separate `fiscal-nfse` crate.
//!
//! # Example
//!
//! ```no_run
//! use fiscal_cte::{build_cte_xml, types::*};
//! # fn demo(data: &CteBuildData) -> Result<(), fiscal_core::FiscalError> {
//! # let data: &CteBuildData = data;
//! let xml = build_cte_xml(data)?;
//! assert!(xml.contains("<CTe"));
//! # Ok(()) }
//! ```

/// 44-digit CT-e access key (chave de acesso) generation and parsing.
pub mod access_key;
/// String-based XML builder that assembles a complete `<CTe>` document.
pub mod builder;
/// String-based XML builder for the BP-e (`<BPe>`, model 63).
pub mod builder_bpe;
/// String-based XML builder for the GTV-e (`<GTVe>`, model 64).
pub mod builder_gtve;
/// String-based XML builder for the CT-e OS (`<CTeOS>`, model 67).
pub mod builder_os;
/// Enveloped XML-DSig signing of the `<infCte>` element (RSA-SHA1).
pub mod signing;
/// Public data structures for the CT-e XML blocks.
pub mod types;
/// Public data structures for the BP-e (model 63) XML blocks.
pub mod types_bpe;
/// Public data structures for the GTV-e (model 64) XML blocks.
pub mod types_gtve;
/// Public data structures for the CT-e OS (model 67) XML blocks.
pub mod types_os;
/// Structural validation of a CT-e XML before transmission.
pub mod validate;

pub use access_key::{CteAccessKey, build_cte_access_key};
pub use builder::build_cte_xml;
pub use builder_bpe::build_bpe_xml;
pub use builder_gtve::build_gtve_xml;
pub use builder_os::build_cteos_xml;
pub use signing::{
    sign_bpe_xml, sign_bpe_xml_with_algorithm, sign_cte_xml, sign_cte_xml_with_algorithm,
    sign_cteos_xml, sign_cteos_xml_with_algorithm, sign_dps_xml, sign_dps_xml_with_algorithm,
    sign_gtve_xml, sign_gtve_xml_with_algorithm, sign_nfse_evento_xml,
};
pub use types::CteBuildData;
pub use types_bpe::BpeBuildData;
pub use types_gtve::GtveBuildData;
pub use types_os::CteOsBuildData;
pub use validate::validate_cte_xml;

/// XML namespace for CT-e documents.
pub const CTE_NAMESPACE: &str = "http://www.portalfiscal.inf.br/cte";

/// CT-e layout version implemented by this crate.
pub const CTE_VERSION: &str = "4.00";

/// Fiscal document model number for CT-e.
pub const CTE_MODEL: &str = "57";

/// Fiscal document model number for CT-e OS (Outros Serviços).
pub const CTEOS_MODEL: &str = "67";

/// Fiscal document model number for GTV-e (Guia de Transporte de Valores).
pub const CTEGTVE_MODEL: &str = "64";

/// Fiscal document model number for BP-e (Bilhete de Passagem eletrônico).
pub const BPE_MODEL: &str = "63";
/// XML namespace for BP-e documents.
pub const BPE_NAMESPACE: &str = "http://www.portalfiscal.inf.br/bpe";
/// BP-e layout version.
pub const BPE_VERSION: &str = "1.00";
