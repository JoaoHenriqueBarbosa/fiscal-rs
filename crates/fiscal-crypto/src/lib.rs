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
//! By default, XML signing uses RSA-SHA1, which is **mandatory for NF-e /
//! NFC-e (and their events / inutilização)**: the `xmldsig-core` schema
//! bundled in the NF-e layout *fixes* the `SignatureMethod`/`DigestMethod`
//! `Algorithm` attributes to `rsa-sha1` / `sha1`. A SHA-256 signature is
//! therefore rejected by SEFAZ with **cStat 225 ("Falha no Schema XML")** —
//! verified live against SEFAZ-SP Homologação. This is independent of the
//! certificate's own signature algorithm: an ICP-Brasil v5 certificate (whose
//! certificate is itself SHA-256-signed) still produces a SHA-1 XML-DSig for
//! NF-e and is accepted (cStat 100).
//!
//! Use [`SignatureAlgorithm::Sha256`] (via the `*_with_algorithm` functions)
//! only for document types / services that explicitly document requiring it —
//! NOT for NF-e/NFC-e, where SHA-256 is a schema violation.

/// Digital certificate loading, management, and XML signing.
pub mod certificate;

pub use certificate::SignatureAlgorithm;
