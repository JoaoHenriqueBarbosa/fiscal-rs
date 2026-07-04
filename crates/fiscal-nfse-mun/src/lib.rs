//! # fiscal-nfse-mun
//!
//! NFS-e **municipal** — cidades que ainda não delegam emissão ao Sistema
//! Nacional (SP, Guarulhos, Sorocaba, Caraguatatuba, Santana de Parnaíba, ...).
//!
//! Design: um **modelo comum** ([`model`]) + uma **interface padrão**
//! ([`provider::MunicipalProvider`]). Cada município pluga a diferença num
//! provedor; o [`registry`] roteia IBGE → provedor. Transporte (SOAP/REST +
//! mTLS) atrás da feature `client`.
//!
//! Famílias de provedor:
//! - **ABRASF 2.x** — DSF (Sorocaba), GINFES (Guarulhos), SigISS (Caraguatatuba)
//! - **Próprio** — SÃO PAULO (PMSP, RPS+lote, 2 assinaturas RSA)
//! - **Nacional-em-endpoint-municipal** — SpeedGov (Santana de Parnaíba), reusa DPS

pub mod abrasf;
pub mod error;
pub mod model;
pub mod provider;
pub mod saopaulo;

#[cfg(feature = "client")]
pub mod providers;
#[cfg(feature = "client")]
pub mod registry;

pub use error::{MunError, Result};
pub use model::*;
pub use provider::ProviderCtx;

#[cfg(feature = "client")]
pub use provider::MunicipalProvider;
#[cfg(feature = "client")]
pub use registry::{is_municipal, national_layout_endpoint, resolve};
