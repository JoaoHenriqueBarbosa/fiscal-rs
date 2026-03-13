//! Authorization, consultation, and batch submission methods.

use fiscal_core::FiscalError;
use fiscal_core::types::SefazEnvironment;

use crate::request_builders;
use crate::response_parsers::{self, AuthorizationResponse, StatusResponse};
use crate::services::SefazService;

use super::SefazClient;

impl SefazClient {
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
    /// The `signed_xml` must already be signed with `fiscal_crypto::sign_xml`.
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
    /// Same as [`SefazClient::authorize`] but compresses the XML with gzip and uses
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
    /// Same as [`SefazClient::authorize`] but routes to the NFC-e endpoint.
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
    /// Same as [`SefazClient::authorize_nfce`] but compresses the XML with gzip.
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

    /// Submit multiple signed NF-e documents as an asynchronous batch
    /// (`NFeAutorizacao4`, `indSinc=0`).
    ///
    /// Matches the PHP `sefazEnviaLote()` method from `Tools.php`.
    /// The batch can contain 1 to 50 NF-e documents. SEFAZ returns a
    /// receipt number (`nRec`) in [`AuthorizationResponse::receipt_number`]
    /// that should be polled via [`SefazClient::consult_receipt`] to obtain
    /// the individual processing results.
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation of the issuer.
    /// * `environment` — SEFAZ environment (production or homologation).
    /// * `signed_xmls` — Slice of signed NF-e XML strings (1 to 50).
    /// * `lot_id` — Lot identifier for the submission batch.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidTaxData`] if the batch is empty or
    /// exceeds 50 documents.
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    pub async fn authorize_batch(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        signed_xmls: &[String],
        lot_id: &str,
    ) -> Result<AuthorizationResponse, FiscalError> {
        self.authorize_batch_opts(uf, environment, signed_xmls, lot_id, false, 55)
            .await
    }

    /// Submit multiple signed NF-e documents as an asynchronous batch
    /// with gzip compression.
    ///
    /// Same as [`authorize_batch`](SefazClient::authorize_batch) but
    /// compresses the XML with gzip and uses `<nfeDadosMsgZip>` in the
    /// SOAP envelope.
    pub async fn authorize_batch_compressed(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        signed_xmls: &[String],
        lot_id: &str,
    ) -> Result<AuthorizationResponse, FiscalError> {
        self.authorize_batch_opts(uf, environment, signed_xmls, lot_id, true, 55)
            .await
    }

    /// Submit multiple signed NFC-e (model 65) documents as an asynchronous
    /// batch (`NFeAutorizacao4`, `indSinc=0`).
    ///
    /// Same as [`authorize_batch`](SefazClient::authorize_batch) but routes
    /// to the NFC-e endpoint.
    pub async fn authorize_batch_nfce(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        signed_xmls: &[String],
        lot_id: &str,
    ) -> Result<AuthorizationResponse, FiscalError> {
        self.authorize_batch_opts(uf, environment, signed_xmls, lot_id, false, 65)
            .await
    }

    /// Internal: submit batch authorization with optional compression and
    /// model selection.
    async fn authorize_batch_opts(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        signed_xmls: &[String],
        lot_id: &str,
        compress: bool,
        model: u8,
    ) -> Result<AuthorizationResponse, FiscalError> {
        let refs: Vec<&str> = signed_xmls.iter().map(|s| s.as_str()).collect();
        let request_xml = request_builders::build_autorizacao_batch_request(&refs, lot_id, 0)?;
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
}
