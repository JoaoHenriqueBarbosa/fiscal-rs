use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum FiscalError {
    #[error("Invalid tax data: {0}")]
    InvalidTaxData(String),

    #[error("Unsupported ICMS CST: {0}")]
    UnsupportedIcmsCst(String),

    #[error("Unsupported ICMS CSOSN: {0}")]
    UnsupportedIcmsCsosn(String),

    #[error("Required tax field \"{field}\" is missing")]
    MissingRequiredField { field: String },

    #[error("Invalid GTIN: {0}")]
    InvalidGtin(String),

    #[error("XML generation failed: {0}")]
    XmlGeneration(String),

    #[error("XML parsing failed: {0}")]
    XmlParsing(String),

    #[error("SEFAZ rejected: [{code}] {message}")]
    SefazRejection { code: String, message: String },

    #[error("Certificate error: {0}")]
    Certificate(String),

    #[error("Invalid state code: {0}")]
    InvalidStateCode(String),

    #[error("Contingency error: {0}")]
    Contingency(String),

    #[error("Invalid TXT: {0}")]
    InvalidTxt(String),

    #[error("Wrong document: {0}")]
    WrongDocument(String),

    #[error("Network error: {0}")]
    Network(String),
}
