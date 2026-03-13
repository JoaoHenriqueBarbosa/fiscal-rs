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
use crate::response_parsers::{
    self, AuthorizationResponse, CadastroResponse, CancellationResponse, DistDFeResponse,
    StatusResponse,
};
use crate::services::SefazService;
use crate::soap;
use crate::urls::{get_an_url, get_sefaz_url_for_model};

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
        self.authorize_opts(uf, environment, signed_xml, lot_id, false, 55)
            .await
    }

    /// Submit a signed NF-e for authorization with gzip compression.
    ///
    /// Same as [`authorize`] but compresses the XML with gzip and uses
    /// `<nfeDadosMsgZip>` in the SOAP envelope, matching PHP sped-nfe's
    /// `$compactar = true` mode.
    pub async fn authorize_compressed(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        signed_xml: &str,
        lot_id: &str,
    ) -> Result<AuthorizationResponse, FiscalError> {
        self.authorize_opts(uf, environment, signed_xml, lot_id, true, 55)
            .await
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
        self.authorize_opts(uf, environment, signed_xml, lot_id, false, 65)
            .await
    }

    /// Submit a signed NFC-e (model 65) for authorization with gzip compression.
    ///
    /// Same as [`authorize_nfce`] but compresses the XML with gzip.
    pub async fn authorize_nfce_compressed(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        signed_xml: &str,
        lot_id: &str,
    ) -> Result<AuthorizationResponse, FiscalError> {
        self.authorize_opts(uf, environment, signed_xml, lot_id, true, 65)
            .await
    }

    /// Internal: submit authorization with optional compression and model selection.
    async fn authorize_opts(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        signed_xml: &str,
        lot_id: &str,
        compress: bool,
        model: u8,
    ) -> Result<AuthorizationResponse, FiscalError> {
        let request_xml =
            request_builders::build_autorizacao_request(signed_xml, lot_id, true, compress);
        let raw = if compress {
            self.send_model_compressed(
                SefazService::Autorizacao,
                uf,
                environment,
                &request_xml,
                model,
            )
            .await?
        } else {
            self.send_model(
                SefazService::Autorizacao,
                uf,
                environment,
                &request_xml,
                model,
            )
            .await?
        };
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

    /// Submit a manifestacao do destinatario event (`RecepcaoEvento4`).
    ///
    /// Routes to the Ambiente Nacional (AN) endpoint. Valid event types:
    /// - `"210200"` — Confirmacao da Operacao
    /// - `"210210"` — Ciencia da Operacao
    /// - `"210220"` — Desconhecimento da Operacao
    /// - `"210240"` — Operacao nao Realizada (requires justification)
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    pub async fn manifest(
        &self,
        environment: SefazEnvironment,
        access_key: &str,
        event_type: &str,
        justification: Option<&str>,
        seq: u32,
        tax_id: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_manifesta_request(
            access_key,
            event_type,
            justification,
            seq,
            environment,
            tax_id,
        );
        let raw = self
            .send_an(SefazService::RecepcaoEvento, environment, &request_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// Query the distribution of fiscal documents (DF-e) from the national
    /// environment (`NFeDistribuicaoDFe`).
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation of the interested party.
    /// * `tax_id` — CNPJ or CPF of the interested party.
    /// * `nsu` — Optional specific NSU or last NSU to query.
    /// * `access_key` — Optional 44-digit access key for direct lookup.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    pub async fn dist_dfe(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        tax_id: &str,
        nsu: Option<&str>,
        access_key: Option<&str>,
    ) -> Result<DistDFeResponse, FiscalError> {
        let request_xml =
            request_builders::build_dist_dfe_request(uf, tax_id, nsu, access_key, environment);
        let raw = self.send_dist_dfe_raw(environment, &request_xml).await?;
        response_parsers::parse_dist_dfe_response(&raw)
    }

    /// Query the SEFAZ taxpayer registry (`CadConsultaCadastro4`).
    ///
    /// # Arguments
    ///
    /// * `uf` — State to query.
    /// * `search_type` — One of `"CNPJ"`, `"IE"`, or `"CPF"`.
    /// * `search_value` — The document number to search for.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    pub async fn cadastro(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        search_type: &str,
        search_value: &str,
    ) -> Result<CadastroResponse, FiscalError> {
        let request_xml = request_builders::build_cadastro_request(uf, search_type, search_value);
        let raw = self
            .send(
                SefazService::ConsultaCadastro,
                uf,
                environment,
                &request_xml,
            )
            .await?;
        response_parsers::parse_cadastro_response(&raw)
    }

    /// Submit an EPEC (Evento Prévio de Emissão em Contingência) event
    /// (`RecepcaoEvento4`, tpEvento=110140).
    ///
    /// The EPEC event is sent to the Ambiente Nacional (AN) endpoint.
    /// The NF-e XML data is extracted via [`request_builders::extract_epec_data`]
    /// and assembled into an EPEC event request.
    ///
    /// # Arguments
    ///
    /// * `epec_data` — Pre-extracted NF-e data for the EPEC event.
    /// * `environment` — SEFAZ environment (production or homologation).
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    pub async fn epec(
        &self,
        epec_data: &request_builders::EpecData,
        environment: SefazEnvironment,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_epec_request(epec_data, environment);
        let raw = self
            .send_an(SefazService::RecepcaoEvento, environment, &request_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// Cancel an NFC-e by substitution (`RecepcaoEvento4`, tpEvento=110112).
    ///
    /// Used exclusively for NFC-e (model 65). The event is sent to
    /// `RecepcaoEvento` of the issuing state.
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation of the issuer.
    /// * `environment` — SEFAZ environment.
    /// * `access_key` — 44-digit access key of the NFC-e being cancelled.
    /// * `ref_access_key` — 44-digit access key of the replacement NFC-e.
    /// * `protocol` — Authorization protocol of the original NFC-e.
    /// * `justification` — Reason for cancellation.
    /// * `ver_aplic` — Version of the issuing application.
    /// * `tax_id` — CNPJ or CPF of the issuer.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[allow(clippy::too_many_arguments)]
    pub async fn cancel_substituicao(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        ref_access_key: &str,
        protocol: &str,
        justification: &str,
        ver_aplic: &str,
        tax_id: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_cancel_substituicao_request(
            access_key,
            ref_access_key,
            protocol,
            justification,
            ver_aplic,
            1,
            environment,
            tax_id,
        );
        let raw = self
            .send_model(
                SefazService::RecepcaoEvento,
                uf,
                environment,
                &request_xml,
                65,
            )
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// Register an interested actor for an NF-e (`RecepcaoEvento4`,
    /// tpEvento=110150).
    ///
    /// Authorizes a transporter to access the NF-e. Sent to
    /// Ambiente Nacional (AN).
    ///
    /// # Arguments
    ///
    /// * `environment` — SEFAZ environment.
    /// * `access_key` — 44-digit access key of the NF-e.
    /// * `tp_autor` — Author type (1=emitente, 2=destinatario, 3=transportador).
    /// * `ver_aplic` — Version of the issuing application.
    /// * `authorized_cnpj` — Optional CNPJ to authorize.
    /// * `authorized_cpf` — Optional CPF to authorize.
    /// * `tp_autorizacao` — Authorization type (0=no subcontract, 1=allowed).
    /// * `issuer_uf` — UF of the issuer.
    /// * `seq` — Event sequence number.
    /// * `tax_id` — CNPJ or CPF of the event sender.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[allow(clippy::too_many_arguments)]
    pub async fn ator_interessado(
        &self,
        environment: SefazEnvironment,
        access_key: &str,
        tp_autor: u8,
        ver_aplic: &str,
        authorized_cnpj: Option<&str>,
        authorized_cpf: Option<&str>,
        tp_autorizacao: u8,
        issuer_uf: &str,
        seq: u32,
        tax_id: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_ator_interessado_request(
            access_key,
            tp_autor,
            ver_aplic,
            authorized_cnpj,
            authorized_cpf,
            tp_autorizacao,
            issuer_uf,
            seq,
            environment,
            tax_id,
        );
        let raw = self
            .send_an(SefazService::RecepcaoEvento, environment, &request_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// Register a delivery receipt for an NF-e (`RecepcaoEvento4`,
    /// tpEvento=110130).
    ///
    /// Records proof of delivery. Sent to Ambiente Nacional (AN).
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[allow(clippy::too_many_arguments)]
    pub async fn comprovante_entrega(
        &self,
        environment: SefazEnvironment,
        access_key: &str,
        ver_aplic: &str,
        delivery_date: &str,
        doc_number: &str,
        name: &str,
        lat: Option<&str>,
        long: Option<&str>,
        hash: &str,
        hash_date: &str,
        issuer_uf: &str,
        seq: u32,
        tax_id: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_comprovante_entrega_request(
            access_key,
            ver_aplic,
            delivery_date,
            doc_number,
            name,
            lat,
            long,
            hash,
            hash_date,
            issuer_uf,
            seq,
            environment,
            tax_id,
        );
        let raw = self
            .send_an(SefazService::RecepcaoEvento, environment, &request_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// Cancel a delivery receipt event (`RecepcaoEvento4`,
    /// tpEvento=110131).
    ///
    /// Cancels a previously registered delivery receipt. Sent to AN.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[allow(clippy::too_many_arguments)]
    pub async fn cancel_comprovante_entrega(
        &self,
        environment: SefazEnvironment,
        access_key: &str,
        ver_aplic: &str,
        event_protocol: &str,
        issuer_uf: &str,
        seq: u32,
        tax_id: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_cancel_comprovante_entrega_request(
            access_key,
            ver_aplic,
            event_protocol,
            issuer_uf,
            seq,
            environment,
            tax_id,
        );
        let raw = self
            .send_an(SefazService::RecepcaoEvento, environment, &request_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// Register a delivery failure event (`RecepcaoEvento4`,
    /// tpEvento=110192).
    ///
    /// Records a failed delivery attempt. Sent to Ambiente Nacional (AN).
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[allow(clippy::too_many_arguments)]
    pub async fn insucesso_entrega(
        &self,
        environment: SefazEnvironment,
        access_key: &str,
        ver_aplic: &str,
        attempt_date: &str,
        attempt_number: Option<u32>,
        reason_type: u8,
        reason_justification: Option<&str>,
        lat: Option<&str>,
        long: Option<&str>,
        hash: &str,
        hash_date: &str,
        issuer_uf: &str,
        seq: u32,
        tax_id: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_insucesso_entrega_request(
            access_key,
            ver_aplic,
            attempt_date,
            attempt_number,
            reason_type,
            reason_justification,
            lat,
            long,
            hash,
            hash_date,
            issuer_uf,
            seq,
            environment,
            tax_id,
        );
        let raw = self
            .send_an(SefazService::RecepcaoEvento, environment, &request_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// Cancel a delivery failure event (`RecepcaoEvento4`,
    /// tpEvento=110193).
    ///
    /// Cancels a previously registered delivery failure. Sent to AN.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[allow(clippy::too_many_arguments)]
    pub async fn cancel_insucesso_entrega(
        &self,
        environment: SefazEnvironment,
        access_key: &str,
        ver_aplic: &str,
        event_protocol: &str,
        issuer_uf: &str,
        seq: u32,
        tax_id: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_cancel_insucesso_entrega_request(
            access_key,
            ver_aplic,
            event_protocol,
            issuer_uf,
            seq,
            environment,
            tax_id,
        );
        let raw = self
            .send_an(SefazService::RecepcaoEvento, environment, &request_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// Submit a pedido de prorrogacao ICMS event (`RecepcaoEvento4`,
    /// tpEvento=111500 or 111501).
    ///
    /// Used for NF-e of consignment for industrialization with ICMS suspension
    /// in interstate operations. First term uses 111500, second term uses 111501.
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation of the issuer.
    /// * `environment` — SEFAZ environment.
    /// * `access_key` — 44-digit access key of the NF-e.
    /// * `protocol` — Authorization protocol of the original NF-e.
    /// * `items` — Items and quantities for the prorrogacao request.
    /// * `second_term` — If `true`, sends 2nd-term event (111501).
    /// * `seq` — Event sequence number.
    /// * `tax_id` — CNPJ or CPF of the issuer.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[allow(clippy::too_many_arguments)]
    pub async fn prorrogacao(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        protocol: &str,
        items: &[request_builders::ProrrogacaoItem],
        second_term: bool,
        seq: u32,
        tax_id: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_prorrogacao_request(
            access_key,
            protocol,
            items,
            second_term,
            seq,
            environment,
            tax_id,
        );
        let raw = self
            .send(SefazService::RecepcaoEvento, uf, environment, &request_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// Cancel a pedido de prorrogacao ICMS event (`RecepcaoEvento4`,
    /// tpEvento=111502 or 111503).
    ///
    /// First term uses 111502, second term uses 111503.
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation of the issuer.
    /// * `environment` — SEFAZ environment.
    /// * `access_key` — 44-digit access key of the NF-e.
    /// * `protocol` — Authorization protocol of the prorrogacao event.
    /// * `second_term` — If `true`, sends 2nd-term cancellation (111503).
    /// * `seq` — Event sequence number.
    /// * `tax_id` — CNPJ or CPF of the issuer.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[allow(clippy::too_many_arguments)]
    pub async fn cancel_prorrogacao(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        protocol: &str,
        second_term: bool,
        seq: u32,
        tax_id: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_cancel_prorrogacao_request(
            access_key,
            protocol,
            second_term,
            seq,
            environment,
            tax_id,
        );
        let raw = self
            .send(SefazService::RecepcaoEvento, uf, environment, &request_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// Download an NF-e by its 44-digit access key (`NFeDistribuicaoDFe`).
    ///
    /// A convenience wrapper around [`dist_dfe`](Self::dist_dfe) that passes
    /// the access key directly. Matches the PHP `sefazDownload()` method.
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation of the interested party.
    /// * `environment` — SEFAZ environment.
    /// * `tax_id` — CNPJ or CPF of the interested party.
    /// * `access_key` — 44-digit access key of the NF-e to download.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    pub async fn download(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        tax_id: &str,
        access_key: &str,
    ) -> Result<DistDFeResponse, FiscalError> {
        self.dist_dfe(uf, environment, tax_id, None, Some(access_key))
            .await
    }

    /// Manage CSC (Código de Segurança do Contribuinte) for NFC-e
    /// (`CscNFCe`).
    ///
    /// Matches the PHP `sefazCsc()` method. Used exclusively with NFC-e
    /// (model 65).
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation of the issuer.
    /// * `environment` — SEFAZ environment.
    /// * `ind_op` — Operation type: 1=query, 2=request new, 3=revoke.
    /// * `cnpj` — Full CNPJ of the taxpayer (14 digits).
    /// * `csc_id` — CSC identifier (required only for `ind_op=3`).
    /// * `csc_code` — CSC code/value (required only for `ind_op=3`).
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    pub async fn csc(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        ind_op: u8,
        cnpj: &str,
        csc_id: Option<&str>,
        csc_code: Option<&str>,
    ) -> Result<String, FiscalError> {
        let request_xml =
            request_builders::build_csc_request(environment, ind_op, cnpj, csc_id, csc_code);
        self.send_model(SefazService::CscNFCe, uf, environment, &request_xml, 65)
            .await
    }

    /// Send multiple events in a single batch (`RecepcaoEvento4`).
    ///
    /// Matches the PHP `sefazEventoLote()` method. The batch can contain
    /// up to 20 events. EPEC events are automatically skipped.
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation or `"AN"` for Ambiente Nacional.
    /// * `environment` — SEFAZ environment.
    /// * `events` — Slice of [`request_builders::EventItem`] structs.
    /// * `lot_id` — Optional lot identifier (auto-generated if `None`).
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    pub async fn event_batch(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        events: &[request_builders::EventItem],
        lot_id: Option<&str>,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml =
            request_builders::build_event_batch_request(uf, events, lot_id, environment);
        let raw = if uf == "AN" {
            self.send_an(SefazService::RecepcaoEvento, environment, &request_xml)
                .await?
        } else {
            self.send(SefazService::RecepcaoEvento, uf, environment, &request_xml)
                .await?
        };
        response_parsers::parse_cancellation_response(&raw)
    }

    /// Send a batch of manifestação do destinatário events.
    ///
    /// Matches the PHP `sefazManifestaLote()` method. Valid event types:
    /// - `210200` — Confirmação da Operação
    /// - `210210` — Ciência da Operação
    /// - `210220` — Desconhecimento da Operação
    /// - `210240` — Operação não Realizada (requires justification in
    ///   `additional_tags`)
    ///
    /// All events are sent to the Ambiente Nacional (AN) endpoint.
    ///
    /// # Arguments
    ///
    /// * `environment` — SEFAZ environment.
    /// * `events` — Slice of [`request_builders::EventItem`] structs.
    /// * `lot_id` — Optional lot identifier (auto-generated if `None`).
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    pub async fn manifest_batch(
        &self,
        environment: SefazEnvironment,
        events: &[request_builders::EventItem],
        lot_id: Option<&str>,
    ) -> Result<CancellationResponse, FiscalError> {
        self.event_batch("AN", environment, events, lot_id).await
    }

    /// Submit a conciliação financeira event (`RecepcaoEvento4`,
    /// tpEvento=110750 or 110751 for cancellation).
    ///
    /// Matches the PHP `sefazConciliacao()` method.
    ///
    /// Per NT 2024.002, for NF-e (model 55) the event is sent to SVRS;
    /// for NFC-e (model 65) it goes to the state's own endpoint.
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation of the issuer.
    /// * `environment` — SEFAZ environment.
    /// * `model` — Invoice model (55 = NF-e, 65 = NFC-e).
    /// * `access_key` — 44-digit access key.
    /// * `ver_aplic` — Application version string.
    /// * `det_pag` — Payment details for the reconciliation.
    /// * `cancel` — If `true`, sends cancellation event (110751).
    /// * `cancel_protocol` — Protocol of the event being cancelled.
    /// * `seq` — Event sequence number.
    /// * `tax_id` — CNPJ or CPF of the issuer.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[allow(clippy::too_many_arguments)]
    pub async fn conciliacao(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        model: u8,
        access_key: &str,
        ver_aplic: &str,
        det_pag: &[request_builders::ConciliacaoDetPag],
        cancel: bool,
        cancel_protocol: Option<&str>,
        seq: u32,
        tax_id: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_conciliacao_request(
            access_key,
            ver_aplic,
            det_pag,
            cancel,
            cancel_protocol,
            seq,
            environment,
            tax_id,
        );
        // NT 2024.002: model 55 uses SVRS, model 65 uses state endpoint
        let effective_uf = if model == 55 { "SVRS" } else { uf };
        let raw = self
            .send(
                SefazService::RecepcaoEvento,
                effective_uf,
                environment,
                &request_xml,
            )
            .await?;
        response_parsers::parse_cancellation_response(&raw)
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
