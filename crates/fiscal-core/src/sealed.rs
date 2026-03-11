//! Sealed trait infrastructure.
//!
//! Traits in this module use the sealed pattern to prevent external
//! implementations while keeping the trait itself public.

/// Private module that prevents external implementations of sealed traits.
pub(crate) mod private {
    /// Marker trait that seals a public trait.
    ///
    /// This trait is `pub(crate)` so only types within this crate can
    /// implement it, which transitively prevents external implementations
    /// of any trait that has `Sealed` as a supertrait.
    pub trait Sealed {}
}
