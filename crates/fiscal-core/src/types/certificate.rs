use super::{EmissionType, InvoiceModel};
use crate::newtypes::IbgeCode;
use chrono::NaiveDate;

// ── Data structures ──────────────────────────────────────────────────────────

/// PKCS#12 certificate data loaded from a PFX file.
///
/// Holds the PEM-encoded private key and certificate alongside the original
/// PFX buffer and passphrase so the certificate can be reused for multiple
/// signing operations without re-parsing.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CertificateData {
    /// PEM-encoded RSA private key (PKCS#8 format).
    pub private_key: String,
    /// PEM-encoded X.509 certificate.
    pub certificate: String,
    /// Raw PKCS#12/PFX binary buffer.
    pub pfx_buffer: Vec<u8>,
    /// Passphrase used to unlock the PFX file.
    pub passphrase: String,
}

impl CertificateData {
    /// Create a new `CertificateData` with all required fields.
    pub fn new(
        private_key: impl Into<String>,
        certificate: impl Into<String>,
        pfx_buffer: Vec<u8>,
        passphrase: impl Into<String>,
    ) -> Self {
        Self {
            private_key: private_key.into(),
            certificate: certificate.into(),
            pfx_buffer,
            passphrase: passphrase.into(),
        }
    }
}

/// Human-readable X.509 certificate metadata for display purposes.
///
/// Extracted from a PKCS#12 file without exposing the private key.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CertificateInfo {
    /// Common name (CN) from the certificate's subject field.
    pub common_name: String,
    /// Date from which the certificate is valid (`notBefore`).
    pub valid_from: NaiveDate,
    /// Expiry date of the certificate (`notAfter`).
    pub valid_until: NaiveDate,
    /// Hex-encoded X.509 serial number.
    pub serial_number: String,
    /// Common name (CN) of the certificate issuer.
    pub issuer: String,
}

impl CertificateInfo {
    /// Create a new `CertificateInfo` with all required fields.
    pub fn new(
        common_name: impl Into<String>,
        valid_from: NaiveDate,
        valid_until: NaiveDate,
        serial_number: impl Into<String>,
        issuer: impl Into<String>,
    ) -> Self {
        Self {
            common_name: common_name.into(),
            valid_from,
            valid_until,
            serial_number: serial_number.into(),
            issuer: issuer.into(),
        }
    }
}

/// Input parameters for building a 44-digit NF-e / NFC-e access key.
///
/// The access key (`chave de acesso`) uniquely identifies a Brazilian electronic
/// invoice and is computed from these components using a Verhoeff-like algorithm.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct AccessKeyParams {
    /// IBGE numeric state code (e.g. `"41"` for Paraná).
    pub state_code: IbgeCode,
    /// Emission year and month in `YYMM` format (e.g. `"2503"` for March 2025).
    pub year_month: String,
    /// CNPJ or CPF of the issuer (digits only, 14 or 11 characters).
    pub tax_id: String,
    /// Invoice model: [`InvoiceModel::Nfe`] (55) or [`InvoiceModel::Nfce`] (65).
    pub model: InvoiceModel,
    /// Series number (0–999).
    pub series: u32,
    /// Invoice number (`nNF`, 1–999 999 999).
    pub number: u32,
    /// Emission type used to set `tpEmis` in the key.
    pub emission_type: EmissionType,
    /// Random numeric code (`cNF`, 8 digits) for uniqueness.
    pub numeric_code: String,
}

impl AccessKeyParams {
    /// Create a new `AccessKeyParams` with all required fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        state_code: IbgeCode,
        year_month: impl Into<String>,
        tax_id: impl Into<String>,
        model: InvoiceModel,
        series: u32,
        number: u32,
        emission_type: EmissionType,
        numeric_code: impl Into<String>,
    ) -> Self {
        Self {
            state_code,
            year_month: year_month.into(),
            tax_id: tax_id.into(),
            model,
            series,
            number,
            emission_type,
            numeric_code: numeric_code.into(),
        }
    }
}
