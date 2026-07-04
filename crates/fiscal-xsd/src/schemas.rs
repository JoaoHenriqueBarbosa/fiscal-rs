//! Embedded official SEFAZ schema bundles, behind cargo features so a binary
//! only carries the schemas it actually validates against.
//!
//! Enable the `mdfe` feature to embed the MDF-e 3.00 schemas and use
//! `mdfe()`.

#[cfg(feature = "mdfe")]
mod mdfe_300;

#[cfg(feature = "mdfe")]
pub use mdfe_300::mdfe;

#[cfg(feature = "nfe")]
mod nfe_400;

#[cfg(feature = "nfe")]
pub use nfe_400::nfe_lote;

#[cfg(feature = "cte")]
mod cte_400;

#[cfg(feature = "cte")]
pub use cte_400::{cte, cteos, gtve};

#[cfg(feature = "bpe")]
mod bpe_100;

#[cfg(feature = "bpe")]
pub use bpe_100::bpe;

#[cfg(feature = "nfse")]
mod nfse_101;

#[cfg(feature = "nfse")]
pub use nfse_101::{dps, nfse_evento};

#[cfg(feature = "abrasf")]
mod abrasf_203;

#[cfg(feature = "abrasf")]
pub use abrasf_203::abrasf_gerar_nfse;

#[cfg(feature = "saopaulo")]
mod saopaulo_v01;

#[cfg(feature = "saopaulo")]
pub use saopaulo_v01::sp_lote_rps;

#[cfg(feature = "saopaulo")]
mod saopaulo_v02;

#[cfg(feature = "saopaulo")]
pub use saopaulo_v02::sp_lote_rps_v2;
