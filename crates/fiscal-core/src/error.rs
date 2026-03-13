//! Error types for all fiscal operations.

use thiserror::Error;

/// The primary error type returned by all public functions in `fiscal-core`,
/// `fiscal-crypto`, and `fiscal-sefaz`.
///
/// All variants are `#[non_exhaustive]` so new error cases can be added in
/// future releases without breaking downstream `match` arms.
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum FiscalError {
    /// A tax data field contains an illegal value (e.g. malformed CNPJ, wrong
    /// field length, invalid access-key format).
    #[error("Invalid tax data: {0}")]
    InvalidTaxData(String),

    /// An ICMS CST code is not recognised by the current tax computation module.
    #[error("Unsupported ICMS CST: {0}")]
    UnsupportedIcmsCst(String),

    /// An ICMS CSOSN code is not recognised by the current tax computation module.
    #[error("Unsupported ICMS CSOSN: {0}")]
    UnsupportedIcmsCsosn(String),

    /// A required XML field is absent. The `field` member names the missing tag.
    #[error("Required tax field \"{field}\" is missing")]
    MissingRequiredField {
        /// Name of the required XML tag or parameter that was absent.
        field: String,
    },

    /// A GTIN barcode value is malformed or has an invalid check digit.
    #[error("Invalid GTIN: {0}")]
    InvalidGtin(String),

    /// An error occurred while building or serialising an XML document.
    #[error("XML generation failed: {0}")]
    XmlGeneration(String),

    /// An error occurred while parsing an XML document.
    #[error("XML parsing failed: {0}")]
    XmlParsing(String),

    /// SEFAZ returned a rejection status code for a submitted document or event.
    #[error("SEFAZ rejected: [{code}] {message}")]
    SefazRejection {
        /// SEFAZ status code (`cStat`), e.g. `"301"`.
        code: String,
        /// Human-readable rejection message (`xMotivo`).
        message: String,
    },

    /// An error occurred while loading, parsing, or using a PKCS#12 certificate.
    #[error("Certificate error: {0}")]
    Certificate(String),

    /// A two-letter state abbreviation (UF) or IBGE numeric code was not found
    /// in the lookup table.
    #[error("Invalid state code: {0}")]
    InvalidStateCode(String),

    /// An error occurred while activating, loading, or applying contingency mode.
    #[error("Contingency error: {0}")]
    Contingency(String),

    /// A TXT document is structurally invalid or references an unsupported layout.
    #[error("Invalid TXT: {0}")]
    InvalidTxt(String),

    /// The supplied document is not of the expected type (e.g. not a valid
    /// NFe TXT header).
    #[error("Wrong document: {0}")]
    WrongDocument(String),

    /// An HTTP or network-level error occurred during SEFAZ communication.
    #[error("Network error: {0}")]
    Network(String),

    /// A configuration JSON is invalid or missing required fields.
    #[error("Config validation error: {0}")]
    ConfigValidation(String),
}
