/// NF-e XML namespace
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

/// Payment type codes (tPag) by name
pub mod payment_types {
    pub const CASH: &str = "01";
    pub const CHECK: &str = "02";
    pub const CREDIT_CARD: &str = "03";
    pub const DEBIT_CARD: &str = "04";
    pub const STORE_CREDIT: &str = "05";
    pub const VOUCHER: &str = "10";
    pub const PIX: &str = "17";
    pub const OTHER: &str = "99";
    pub const NONE: &str = "90";
}
