//! Parse-don't-validate newtypes for monetary amounts, tax rates, access keys,
//! and state codes.
//!
//! All newtypes currently keep their inner field `pub` to allow gradual
//! migration from raw primitives.  A future pass will make the fields private
//! once the codebase is fully migrated.

mod access_key;
mod monetary;
mod product_codes;
mod state;
mod tax_id;

pub use access_key::AccessKey;
pub use monetary::{Cents, Rate, Rate4};
pub use product_codes::{Cfop, Gtin, Ncm};
pub use state::{IbgeCode, StateCode};
pub use tax_id::TaxId;
