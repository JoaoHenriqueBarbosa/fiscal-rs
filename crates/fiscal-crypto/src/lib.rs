//! Digital-certificate loading and XML signing for Brazilian fiscal documents.
//!
//! `fiscal-crypto` provides two capabilities:
//! - **Certificate management** — load a PKCS#12 `.pfx` file and extract the
//!   signing key plus X.509 certificate chain ([`certificate`] module).
//! - **XML signing** — produce `<Signature>` elements conforming to the
//!   NF-e / NFC-e XML-DSIG specification, using the loaded certificate.
//!
//! # Signature algorithms
//!
//! By default, XML signing uses RSA-SHA1 for backwards compatibility.
//! For ICP-Brasil v5 certificates or SEFAZs that reject SHA-1
//! (rejeição 297), use [`SignatureAlgorithm::Sha256`] with the
//! `*_with_algorithm` signing functions.

/// Digital certificate loading, management, and XML signing.
pub mod certificate;

pub use certificate::SignatureAlgorithm;
