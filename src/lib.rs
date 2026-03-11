// Facade crate: re-exports all sub-crates so existing `use fiscal::...` imports work.

pub use fiscal_core::*;

/// Certificate handling and XML signing.
pub mod certificate {
    pub use fiscal_crypto::certificate::*;
}

/// SEFAZ URLs, request builders, response parsers, and status codes.
pub mod sefaz {
    pub use fiscal_sefaz::*;
}
