//! Compile-time constants used throughout NF-e XML generation.
//!
//! Includes namespace URIs, algorithm identifiers, version strings, and the
//! [`payment_types`] submodule with `tPag` payment type codes.

/// NF-e XML namespace (`xmlns`).
pub const NFE_NAMESPACE: &str = "http://www.portalfiscal.inf.br/nfe";

/// XML Digital Signature namespace
pub const XMLDSIG_NAMESPACE: &str = "http://www.w3.org/2000/09/xmldsig#";

/// C14N canonicalization algorithm URI
pub const C14N_ALGORITHM: &str = "http://www.w3.org/TR/2001/REC-xml-c14n-20010315";

/// Enveloped signature transform URI
pub const ENVELOPED_SIGNATURE_TRANSFORM: &str =
    "http://www.w3.org/2000/09/xmldsig#enveloped-signature";

/// NF-e version (currently 4.00)
pub const NFE_VERSION: &str = "4.00";

/// SOAP envelope namespace for SEFAZ web service requests
pub const SOAP_ENVELOPE_NS: &str = "http://www.w3.org/2003/05/soap-envelope";

/// SEFAZ NF-e WSDL base namespace
pub const NFE_WSDL_NS: &str = "http://www.portalfiscal.inf.br/nfe/wsdl";

/// Payment type codes (`tPag`) defined by the NF-e specification.
///
/// Use these constants as the `method` field of [`crate::types::PaymentData`].
///
/// # Examples
///
/// ```
/// use fiscal_core::constants::payment_types;
/// use fiscal_core::types::PaymentData;
/// use fiscal_core::newtypes::Cents;
///
/// let payment = PaymentData::new(payment_types::PIX, Cents(10000));
/// assert_eq!(payment.method, "17");
/// ```
pub mod payment_types {
    /// Cash payment (`01`).
    pub const CASH: &str = "01";
    /// Bank cheque (`02`).
    pub const CHECK: &str = "02";
    /// Credit card (`03`).
    pub const CREDIT_CARD: &str = "03";
    /// Debit card (`04`).
    pub const DEBIT_CARD: &str = "04";
    /// Store credit / credit slip (`05`).
    pub const STORE_CREDIT: &str = "05";
    /// Meal / food voucher (`10`).
    pub const VOUCHER: &str = "10";
    /// Pix instant payment (`17`).
    pub const PIX: &str = "17";
    /// Other payment methods (`99`).
    pub const OTHER: &str = "99";
    /// No payment (used for NFC-e without a financial transaction) (`90`).
    pub const NONE: &str = "90";
}
