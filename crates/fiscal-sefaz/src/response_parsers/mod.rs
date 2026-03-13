//! SEFAZ XML response parsers and typed result structs.
//!
//! Each sub-module handles a specific SEFAZ response type. The public API
//! (structs + parse functions) is re-exported at this level so existing
//! `use crate::response_parsers::*` paths continue to work.

mod authorize;
mod consult;
mod event;
mod helpers;
mod inutilize;
mod misc;
mod status;
mod types;

// Re-export all public types
pub use types::{
    AuthorizationResponse, CadastroResponse, CancellationResponse, ConsultaReciboResponse,
    ConsultaSituacaoResponse, CscResponse, CscToken, DistDFeResponse, InutilizacaoResponse,
    ProtocolInfo, StatusResponse,
};

// Re-export all public parse functions
pub use authorize::parse_autorizacao_response;
pub use consult::{parse_consulta_recibo_response, parse_consulta_situacao_response};
pub use event::parse_cancellation_response;
pub use inutilize::parse_inutilizacao_response;
pub use misc::{parse_cadastro_response, parse_csc_response, parse_dist_dfe_response};
pub use status::parse_status_response;
