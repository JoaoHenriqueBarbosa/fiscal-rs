//! NFS-e Nacional — **DPS 1.01** (Declaração de Prestação de Serviço).
//!
//! Implementa o leiaute nacional publicado pela RFB/SEFIN
//! (`http://www.sped.fazenda.gov.br/nfse`). Diferente dos sistemas municipais
//! ABRASF (ver `fiscal-nfse-mun`), a NFS-e Nacional usa:
//!
//! - Transporte **REST** (não SOAP)
//! - Assinatura envelopada em `<infDPS>` via RSA-SHA-1 (mesmo padrão CT-e)
//! - Chave de **50 dígitos** (vs 44 do CT-e / 47 do MDF-e)
//! - Compressão gzip antes do envio ao SEFIN
//!
//! # Escopo
//!
//! `fiscal-nfse` cobre apenas o leiaute **nacional**. Para municípios com
//! sistemas próprios (São Paulo PMSP, Sorocaba DSF, etc.) use `fiscal-nfse-mun`.
//!
//! # Exemplo
//!
//! ```no_run
//! use fiscal_nfse::{build_dps_xml, types::*};
//! # fn demo(data: &DpsBuildData) {
//! let xml = build_dps_xml(data);
//! assert!(xml.contains("<DPS"));
//! # }
//! ```

/// String-based XML builder for the DPS document and events.
pub mod builder;
/// Public data structures for the DPS XML blocks.
pub mod types;

pub use builder::{build_dps_xml, build_nfse_cancelamento};
pub use types::DpsBuildData;

/// XML namespace for NFS-e Nacional (DPS).
pub const NFSE_NAMESPACE: &str = "http://www.sped.fazenda.gov.br/nfse";

/// DPS layout version implemented by this crate.
pub const NFSE_VERSION: &str = "1.01";
