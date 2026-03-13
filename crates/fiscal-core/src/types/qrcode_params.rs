use super::{EmissionType, QrCodeVersion, SefazEnvironment};

/// Parameters for building an NFC-e QR Code URL.
///
/// Pass to [`crate::qrcode::build_nfce_qr_code_url`].
#[non_exhaustive]
pub struct NfceQrCodeParams {
    /// 44-digit NFC-e access key.
    pub access_key: String,
    /// QR Code generation version.
    pub version: QrCodeVersion,
    /// Submission environment (production or homologation).
    pub environment: SefazEnvironment,
    /// Emission type (normal or contingency).
    pub emission_type: EmissionType,
    /// Base URL from the state's NFC-e portal (without `?p=`).
    pub qr_code_base_url: String,
    /// CSC (Consumer Security Code) token. Required for `V200`.
    pub csc_token: Option<String>,
    /// CSC identifier number. Required for `V200`.
    pub csc_id: Option<String>,
    /// Emission date-time string (`dhEmi`). Required for offline `V200`.
    pub issued_at: Option<String>,
    /// Total NFC-e value string (2 decimal places). Required for offline `V200`.
    pub total_value: Option<String>,
    /// Total ICMS value string. Optional.
    pub total_icms: Option<String>,
    /// SHA-1 digest value from the XML signature. Required for offline `V200`.
    pub digest_value: Option<String>,
    /// CNPJ, CPF, or foreign ID of the destination. Optional.
    pub dest_document: Option<String>,
    /// Destination ID type indicator (`1`=CNPJ, `2`=CPF, `3`=foreign). Optional.
    pub dest_id_type: Option<String>,
    /// RSA signing function for v300 offline QR Code.
    ///
    /// Receives the data to sign as bytes and must return the raw signature.
    /// Only required for v300 offline emission (`tpEmis=9`).
    #[cfg_attr(not(doc), allow(clippy::type_complexity))]
    pub sign_fn: Option<Box<dyn Fn(&[u8]) -> Result<Vec<u8>, crate::FiscalError> + Send + Sync>>,
}

impl std::fmt::Debug for NfceQrCodeParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NfceQrCodeParams")
            .field("access_key", &self.access_key)
            .field("version", &self.version)
            .field("environment", &self.environment)
            .field("emission_type", &self.emission_type)
            .field("qr_code_base_url", &self.qr_code_base_url)
            .field("csc_token", &self.csc_token)
            .field("csc_id", &self.csc_id)
            .field("issued_at", &self.issued_at)
            .field("total_value", &self.total_value)
            .field("total_icms", &self.total_icms)
            .field("digest_value", &self.digest_value)
            .field("dest_document", &self.dest_document)
            .field("dest_id_type", &self.dest_id_type)
            .field("sign_fn", &self.sign_fn.as_ref().map(|_| "<fn>"))
            .finish()
    }
}

impl NfceQrCodeParams {
    /// Create a new `NfceQrCodeParams` with required fields.
    pub fn new(
        access_key: impl Into<String>,
        version: QrCodeVersion,
        environment: SefazEnvironment,
        emission_type: EmissionType,
        qr_code_base_url: impl Into<String>,
    ) -> Self {
        Self {
            access_key: access_key.into(),
            version,
            environment,
            emission_type,
            qr_code_base_url: qr_code_base_url.into(),
            csc_token: None,
            csc_id: None,
            issued_at: None,
            total_value: None,
            total_icms: None,
            digest_value: None,
            dest_document: None,
            dest_id_type: None,
            sign_fn: None,
        }
    }

    /// Set the CSC token.
    pub fn csc_token(mut self, v: impl Into<String>) -> Self {
        self.csc_token = Some(v.into());
        self
    }
    /// Set the CSC ID.
    pub fn csc_id(mut self, v: impl Into<String>) -> Self {
        self.csc_id = Some(v.into());
        self
    }
    /// Set the issued at date.
    pub fn issued_at(mut self, v: impl Into<String>) -> Self {
        self.issued_at = Some(v.into());
        self
    }
    /// Set the total value.
    pub fn total_value(mut self, v: impl Into<String>) -> Self {
        self.total_value = Some(v.into());
        self
    }
    /// Set the total ICMS.
    pub fn total_icms(mut self, v: impl Into<String>) -> Self {
        self.total_icms = Some(v.into());
        self
    }
    /// Set the digest value.
    pub fn digest_value(mut self, v: impl Into<String>) -> Self {
        self.digest_value = Some(v.into());
        self
    }
    /// Set the destination document.
    pub fn dest_document(mut self, v: impl Into<String>) -> Self {
        self.dest_document = Some(v.into());
        self
    }
    /// Set the destination ID type.
    pub fn dest_id_type(mut self, v: impl Into<String>) -> Self {
        self.dest_id_type = Some(v.into());
        self
    }

    /// Set the RSA signing function for v300 offline QR Code.
    ///
    /// The function receives the data string as bytes and must return the raw
    /// RSA signature bytes. The caller must use the same private key from the
    /// digital certificate used to sign the NF-e.
    pub fn sign_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(&[u8]) -> Result<Vec<u8>, crate::FiscalError> + Send + Sync + 'static,
    {
        self.sign_fn = Some(Box::new(f));
        self
    }
}

/// Parameters for inserting the `<infNFeSupl>` QR Code block into a signed NFC-e XML.
///
/// Pass to [`crate::qrcode::put_qr_tag`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PutQRTagParams {
    /// The signed NFC-e XML string.
    pub xml: String,
    /// CSC (Consumer Security Code) token value.
    pub csc_token: String,
    /// CSC identifier number (as a string).
    pub csc_id: String,
    /// QR Code version string: `"200"` or `"300"`.
    pub version: String,
    /// Base QR Code URL for the issuer's state.
    pub qr_code_base_url: String,
    /// Base URL for the consumer consultation link (`urlChave`).
    pub url_chave: String,
}

impl PutQRTagParams {
    /// Create a new `PutQRTagParams` with all required fields.
    pub fn new(
        xml: impl Into<String>,
        csc_token: impl Into<String>,
        csc_id: impl Into<String>,
        version: impl Into<String>,
        qr_code_base_url: impl Into<String>,
        url_chave: impl Into<String>,
    ) -> Self {
        Self {
            xml: xml.into(),
            csc_token: csc_token.into(),
            csc_id: csc_id.into(),
            version: version.into(),
            qr_code_base_url: qr_code_base_url.into(),
            url_chave: url_chave.into(),
        }
    }
}
