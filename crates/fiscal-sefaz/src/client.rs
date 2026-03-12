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

use std::fmt;
use std::time::Duration;

use reqwest::{Client, Identity};

use fiscal_core::FiscalError;
use fiscal_core::types::SefazEnvironment;

use crate::request_builders;
use crate::response_parsers::{self, AuthorizationResponse, CancellationResponse, StatusResponse};
use crate::services::SefazService;
use crate::soap;
use crate::urls::get_sefaz_url_for_model;

/// Default timeout for connecting to a SEFAZ endpoint.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// Default timeout for the full request/response cycle.
///
/// SEFAZ authorization can be slow; 90 s accommodates peak-hour latency.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(90);

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
        self.send_model(service, uf, environment, request_xml, 55).await
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

    // ── Typed convenience methods ────────────────────────────────────────

    /// Check SEFAZ operational status (`NFeStatusServico4`).
    ///
    /// Builds the `<consStatServ>` request, sends it, and parses the
    /// `<retConsStatServ>` response.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidStateCode`] if `uf` is invalid.
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    pub async fn status(
        &self,
        uf: &str,
        environment: SefazEnvironment,
    ) -> Result<StatusResponse, FiscalError> {
        let request_xml = request_builders::build_status_request(uf, environment);
        let raw = self
            .send(SefazService::StatusServico, uf, environment, &request_xml)
            .await?;
        response_parsers::parse_status_response(&raw)
    }

    /// Submit a signed NF-e for authorization (`NFeAutorizacao4`).
    ///
    /// The `signed_xml` must already be signed with [`fiscal_crypto::sign_xml`].
    /// Uses synchronous processing (`indSinc=1`) for a single document.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    pub async fn authorize(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        signed_xml: &str,
        lot_id: &str,
    ) -> Result<AuthorizationResponse, FiscalError> {
        let request_xml =
            request_builders::build_autorizacao_request(signed_xml, lot_id, true, false);
        let raw = self
            .send(SefazService::Autorizacao, uf, environment, &request_xml)
            .await?;
        response_parsers::parse_autorizacao_response(&raw)
    }

    /// Submit a signed NFC-e (model 65) for authorization.
    ///
    /// Same as [`authorize`] but routes to the NFC-e endpoint.
    pub async fn authorize_nfce(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        signed_xml: &str,
        lot_id: &str,
    ) -> Result<AuthorizationResponse, FiscalError> {
        let request_xml =
            request_builders::build_autorizacao_request(signed_xml, lot_id, true, false);
        let raw = self
            .send_model(SefazService::Autorizacao, uf, environment, &request_xml, 65)
            .await?;
        response_parsers::parse_autorizacao_response(&raw)
    }

    /// Query batch processing result by receipt number (`NFeRetAutorizacao4`).
    ///
    /// Used after asynchronous authorization (`indSinc=0`) to poll for
    /// the result.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    pub async fn consult_receipt(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        receipt: &str,
    ) -> Result<AuthorizationResponse, FiscalError> {
        let request_xml = request_builders::build_consulta_recibo_request(receipt, environment);
        let raw = self
            .send(SefazService::RetAutorizacao, uf, environment, &request_xml)
            .await?;
        response_parsers::parse_autorizacao_response(&raw)
    }

    /// Consult an NF-e by its 44-digit access key (`NFeConsultaProtocolo4`).
    ///
    /// Returns the current status, protocol number, and authorization
    /// timestamp for an existing NF-e.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    pub async fn consult(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
    ) -> Result<AuthorizationResponse, FiscalError> {
        let request_xml = request_builders::build_consulta_request(access_key, environment);
        let raw = self
            .send(
                SefazService::ConsultaProtocolo,
                uf,
                environment,
                &request_xml,
            )
            .await?;
        response_parsers::parse_autorizacao_response(&raw)
    }

    /// Cancel a previously authorized NF-e (`RecepcaoEvento4`, tpEvento=110111).
    ///
    /// # Arguments
    ///
    /// * `access_key` — 44-digit access key of the NF-e to cancel.
    /// * `protocol` — protocol number from the authorization response.
    /// * `justification` — reason for cancellation (min 15 characters).
    /// * `tax_id` — CNPJ or CPF of the issuer.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    pub async fn cancel(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        protocol: &str,
        justification: &str,
        tax_id: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_cancela_request(
            access_key,
            protocol,
            justification,
            1,
            environment,
            tax_id,
        );
        let raw = self
            .send(SefazService::RecepcaoEvento, uf, environment, &request_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// Send a Carta de Correcao / CCe (`RecepcaoEvento4`, tpEvento=110110).
    ///
    /// # Arguments
    ///
    /// * `access_key` — 44-digit access key of the NF-e to correct.
    /// * `correction` — correction text describing the change.
    /// * `seq` — event sequence number (increments per correction on same NF-e).
    /// * `tax_id` — CNPJ or CPF of the issuer.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    pub async fn cce(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        correction: &str,
        seq: u32,
        tax_id: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml =
            request_builders::build_cce_request(access_key, correction, seq, environment, tax_id);
        let raw = self
            .send(SefazService::RecepcaoEvento, uf, environment, &request_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// Submit an inutilizacao request to void unused number ranges
    /// (`NFeInutilizacao4`).
    ///
    /// The signed `<inutNFe>` XML must be pre-built via
    /// [`request_builders::build_inutilizacao_request`] and signed with
    /// [`fiscal_crypto::sign_xml`] before calling this method.
    ///
    /// Returns the raw SEFAZ response XML. Parse it with
    /// [`response_parsers`] or extract fields manually.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    pub async fn inutilize(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        signed_inut_xml: &str,
    ) -> Result<String, FiscalError> {
        self.send(SefazService::Inutilizacao, uf, environment, signed_inut_xml)
            .await
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
