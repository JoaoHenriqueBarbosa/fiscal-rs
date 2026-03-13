//! Async SEFAZ web service client with mTLS authentication.
//!
//! [`SefazClient`] wraps a [`reqwest::Client`] pre-configured with the
//! emitter's A1 digital certificate (PFX/PKCS#12) for mutual TLS.
//!
//! ```rust,no_run
//! # async fn example() -> Result<(), fiscal_core::FiscalError> {
//! use fiscal_core::types::SefazEnvironment;
//! use fiscal_sefaz::client::SefazClient;
//! use fiscal_sefaz::request_builders::build_status_request;
//! use fiscal_sefaz::services::SefazService;
//!
//! let pfx = std::fs::read("cert.pfx").unwrap();
//! let client = SefazClient::new(&pfx, "my_password")?;
//!
//! // Typed convenience method
//! let status = client.status("SP", SefazEnvironment::Homologation).await?;
//! println!("SEFAZ status: {} — {}", status.status_code, status.status_message);
//!
//! // Or low-level send for any service
//! let request_xml = build_status_request("RS", SefazEnvironment::Production);
//! let raw_xml = client
//!     .send(SefazService::StatusServico, "RS", SefazEnvironment::Production, &request_xml)
//!     .await?;
//! # Ok(())
//! # }
//! ```

mod authorize;
mod delivery;
mod events;
mod rtc;

use std::fmt;
use std::time::Duration;

use reqwest::{Client, Identity};

use fiscal_core::FiscalError;
use fiscal_core::types::SefazEnvironment;

use crate::request_builders;
use crate::services::SefazService;
use crate::soap;
use crate::urls::{get_an_url, get_sefaz_url_for_model};

/// Default timeout for connecting to a SEFAZ endpoint.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// Default timeout for the full request/response cycle.
///
/// SEFAZ authorization can be slow; 90 s accommodates peak-hour latency.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(90);

/// NT 2024.002 / NT 2025.002: when sending conciliação or RTC events via SVRS,
/// `cOrgao` must be set to 92 instead of deriving from the access key.
fn svrs_org_override(uf: &str) -> Option<&'static str> {
    if uf == "SVRS" { Some("92") } else { None }
}

/// Async SEFAZ web service client with mTLS authentication.
///
/// Holds a pre-configured [`reqwest::Client`] with the emitter's digital
/// certificate identity. The client itself is stateless — UF and environment
/// are passed per-call so a single client can serve multiple states.
///
/// # Construction
///
/// Use [`SefazClient::new`] with the raw PFX bytes and passphrase.
/// The certificate is loaded once and reused for all requests.
pub struct SefazClient {
    http: Client,
}

impl fmt::Debug for SefazClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SefazClient")
            .field("http", &"reqwest::Client { .. }")
            .finish()
    }
}

impl SefazClient {
    /// Create a new client from a PKCS#12 (PFX) certificate buffer.
    ///
    /// The PFX is parsed and installed as the TLS client identity for all
    /// subsequent requests. TLS 1.2 is enforced as required by SEFAZ.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Certificate`] if:
    /// - The PFX buffer is invalid or the passphrase is wrong
    /// - The underlying TLS stack rejects the certificate
    ///
    /// Returns [`FiscalError::Network`] if the HTTP client cannot be built.
    pub fn new(pfx_buffer: &[u8], passphrase: &str) -> Result<Self, FiscalError> {
        let modern_pfx = fiscal_crypto::certificate::ensure_modern_pfx(pfx_buffer, passphrase)?;
        let identity = Identity::from_pkcs12_der(&modern_pfx, passphrase)
            .map_err(|e| FiscalError::Certificate(format!("Failed to load PFX identity: {e}")))?;

        let http = Client::builder()
            .use_native_tls()
            .identity(identity)
            .danger_accept_invalid_certs(true)
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|e| FiscalError::Network(format!("Failed to build HTTP client: {e}")))?;

        Ok(Self { http })
    }

    // ── Low-level ────────────────────────────────────────────────────────

    /// Send a raw request XML to a SEFAZ service and return the raw
    /// response XML.
    ///
    /// This is the escape hatch for services that do not (yet) have a
    /// typed convenience method. The SOAP envelope is built automatically.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidStateCode`] if `uf` is not a valid
    /// Brazilian state abbreviation.
    ///
    /// Returns [`FiscalError::Network`] if the HTTP request fails
    /// (connection refused, timeout, TLS handshake failure, non-2xx
    /// status, etc.).
    pub async fn send(
        &self,
        service: SefazService,
        uf: &str,
        environment: SefazEnvironment,
        request_xml: &str,
    ) -> Result<String, FiscalError> {
        self.send_model(service, uf, environment, request_xml, 55)
            .await
    }

    /// Send a raw request XML to a SEFAZ service for a specific invoice model
    /// (55 = NF-e, 65 = NFC-e) and return the raw response XML.
    pub async fn send_model(
        &self,
        service: SefazService,
        uf: &str,
        environment: SefazEnvironment,
        request_xml: &str,
        model: u8,
    ) -> Result<String, FiscalError> {
        let url = get_sefaz_url_for_model(uf, environment, service.url_key(), model)?;
        let meta = service.meta();
        let envelope = soap::build_envelope(request_xml, uf, &meta)?;
        let action = soap::build_action(&meta);

        let content_type = format!("application/soap+xml;charset=utf-8;action=\"{action}\"");

        let response = self
            .http
            .post(&url)
            .header("Content-Type", &content_type)
            .body(envelope)
            .send()
            .await
            .map_err(|e| FiscalError::Network(format!("{e}")))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| FiscalError::Network(format!("Failed to read response body: {e}")))?;

        if !status.is_success() {
            return Err(FiscalError::Network(format!(
                "SEFAZ returned HTTP {status}: {body}"
            )));
        }

        Ok(body)
    }

    // ── Validation ────────────────────────────────────────────────────

    /// Validate an authorized NF-e against SEFAZ records.
    ///
    /// Extracts the access key, protocol number, and digest value from the
    /// local authorized NF-e XML, then queries SEFAZ via
    /// [`consult`](Self::consult) and compares:
    ///
    /// 1. Protocol number (`nProt`)
    /// 2. Digest value (`digVal` / `DigestValue`)
    /// 3. Access key (`chNFe`)
    ///
    /// Returns a [`ValidationResult`](crate::validate::ValidationResult)
    /// with `is_valid = true` only when all three match.
    ///
    /// Mirrors the PHP `Tools::sefazValidate()` method.
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation.
    /// * `nfe_xml` — The complete authorized NF-e XML (with `<protNFe>` attached).
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::XmlParsing`] if the local XML is missing
    /// required elements.
    /// Returns [`FiscalError::Network`] on SEFAZ communication failure.
    pub async fn sefaz_validate(
        &self,
        uf: &str,
        nfe_xml: &str,
    ) -> Result<crate::validate::ValidationResult, FiscalError> {
        let (access_key, protocol, digest) = crate::validate::extract_nfe_validation_data(nfe_xml)?;

        // Determine environment from tpAmb in the XML
        let tp_amb =
            fiscal_core::xml_utils::extract_xml_tag_value(nfe_xml, "tpAmb").ok_or_else(|| {
                FiscalError::XmlParsing("Tag <tpAmb> não encontrada no XML".to_string())
            })?;
        let environment = if tp_amb == "1" {
            SefazEnvironment::Production
        } else {
            SefazEnvironment::Homologation
        };

        // Query SEFAZ by access key
        let request_xml = request_builders::build_consulta_request(&access_key, environment);
        let raw = self
            .send(
                SefazService::ConsultaProtocolo,
                uf,
                environment,
                &request_xml,
            )
            .await?;

        // Compare local vs SEFAZ data
        crate::validate::validate_authorized_nfe(&access_key, &protocol, &digest, &raw)
    }

    // ── Internal helpers ────────────────────────────────────────────────

    /// Send a raw request XML to a SEFAZ service using gzip compression.
    ///
    /// The request XML is gzip-compressed and base64-encoded in the SOAP
    /// envelope, using `<nfeDadosMsgZip>` instead of `<nfeDadosMsg>`.
    async fn send_model_compressed(
        &self,
        service: SefazService,
        uf: &str,
        environment: SefazEnvironment,
        request_xml: &str,
        model: u8,
    ) -> Result<String, FiscalError> {
        let url = get_sefaz_url_for_model(uf, environment, service.url_key(), model)?;
        let meta = service.meta();
        let envelope = soap::build_envelope_compressed(request_xml, uf, &meta)?;
        let action = soap::build_action(&meta);

        let content_type = format!("application/soap+xml;charset=utf-8;action=\"{action}\"");

        let response = self
            .http
            .post(&url)
            .header("Content-Type", &content_type)
            .body(envelope)
            .send()
            .await
            .map_err(|e| FiscalError::Network(format!("{e}")))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| FiscalError::Network(format!("Failed to read response body: {e}")))?;

        if !status.is_success() {
            return Err(FiscalError::Network(format!(
                "SEFAZ returned HTTP {status}: {body}"
            )));
        }

        Ok(body)
    }

    /// Send a request to the Ambiente Nacional (AN) endpoint.
    ///
    /// AN provides RecepcaoEvento (manifestacao) and DistDFe services.
    async fn send_an(
        &self,
        service: SefazService,
        environment: SefazEnvironment,
        request_xml: &str,
    ) -> Result<String, FiscalError> {
        let url = get_an_url(environment, service.url_key())?;
        let meta = service.meta();
        // AN is not a real state code, but we use it for envelope building
        let envelope = soap::build_envelope(request_xml, "AN", &meta)?;
        let action = soap::build_action(&meta);

        let content_type = format!("application/soap+xml;charset=utf-8;action=\"{action}\"");

        let response = self
            .http
            .post(&url)
            .header("Content-Type", &content_type)
            .body(envelope)
            .send()
            .await
            .map_err(|e| FiscalError::Network(format!("{e}")))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| FiscalError::Network(format!("Failed to read response body: {e}")))?;

        if !status.is_success() {
            return Err(FiscalError::Network(format!(
                "SEFAZ returned HTTP {status}: {body}"
            )));
        }

        Ok(body)
    }

    /// Send a ConsultaCadastro request with the `<nfeCabecMsg>` SOAP header.
    ///
    /// Uses `build_envelope_with_header` which adds the legacy
    /// `<soap:Header><nfeCabecMsg>` block required by this v2.00 service.
    async fn send_cadastro_raw(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        request_xml: &str,
    ) -> Result<String, FiscalError> {
        let service = SefazService::ConsultaCadastro;
        let url = get_sefaz_url_for_model(uf, environment, service.url_key(), 55)?;
        let meta = service.meta();
        // PHP sped-nfe wraps the request in an extra <nfeDadosMsg> for MT
        // before the SOAP envelope body (Tools.php ~line 324-326)
        let mt_wrapped;
        let effective_xml = if uf.eq_ignore_ascii_case("MT") {
            mt_wrapped = format!("<nfeDadosMsg>{request_xml}</nfeDadosMsg>");
            &mt_wrapped
        } else {
            request_xml
        };
        let envelope = soap::build_envelope_with_header(effective_xml, uf, &meta)?;
        let action = soap::build_action(&meta);

        let content_type = format!("application/soap+xml;charset=utf-8;action=\"{action}\"");

        let response = self
            .http
            .post(&url)
            .header("Content-Type", &content_type)
            .body(envelope)
            .send()
            .await
            .map_err(|e| FiscalError::Network(format!("{e}")))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| FiscalError::Network(format!("Failed to read response body: {e}")))?;

        if !status.is_success() {
            return Err(FiscalError::Network(format!(
                "SEFAZ returned HTTP {status}: {body}"
            )));
        }

        Ok(body)
    }

    /// Send a DistDFe request with the special SOAP wrapper.
    ///
    /// Uses `build_envelope_dist_dfe` which adds the extra
    /// `<nfeDistDFeInteresse>` wrapper required by this service.
    async fn send_dist_dfe_raw(
        &self,
        environment: SefazEnvironment,
        request_xml: &str,
    ) -> Result<String, FiscalError> {
        let service = SefazService::DistribuicaoDFe;
        let url = get_an_url(environment, service.url_key())?;
        let meta = service.meta();
        let envelope = soap::build_envelope_dist_dfe(request_xml, "AN", &meta)?;
        let action = soap::build_action(&meta);

        let content_type = format!("application/soap+xml;charset=utf-8;action=\"{action}\"");

        let response = self
            .http
            .post(&url)
            .header("Content-Type", &content_type)
            .body(envelope)
            .send()
            .await
            .map_err(|e| FiscalError::Network(format!("{e}")))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| FiscalError::Network(format!("Failed to read response body: {e}")))?;

        if !status.is_success() {
            return Err(FiscalError::Network(format!(
                "SEFAZ returned HTTP {status}: {body}"
            )));
        }

        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: Integration tests against real SEFAZ endpoints belong in
    // `tests/` with `#[ignore]` and require a valid certificate.
    // The tests here verify construction error paths only.

    #[test]
    fn rejects_invalid_pfx_buffer() {
        let err = SefazClient::new(b"not a pfx", "password").unwrap_err();
        assert!(
            matches!(err, FiscalError::Certificate(_)),
            "expected Certificate error, got: {err}"
        );
    }

    #[test]
    fn rejects_empty_pfx_buffer() {
        let err = SefazClient::new(&[], "").unwrap_err();
        assert!(matches!(err, FiscalError::Certificate(_)));
    }
}
