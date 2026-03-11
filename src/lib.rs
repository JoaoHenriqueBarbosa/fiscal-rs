//! Facade crate that re-exports all `fiscal-rs` sub-crates under a single `fiscal` namespace.
//!
//! All items from [`fiscal_core`], [`fiscal_crypto`] (under [`certificate`]), and
//! [`fiscal_sefaz`] (under [`sefaz`]) are available from this crate so that
//! existing `use fiscal::...` imports continue to work unchanged.

pub use fiscal_core::*;

/// Certificate handling and XML signing.
pub mod certificate {
    pub use fiscal_crypto::certificate::*;
}

/// SEFAZ URLs, request builders, response parsers, and status codes.
pub mod sefaz {
    pub use fiscal_sefaz::*;
}
