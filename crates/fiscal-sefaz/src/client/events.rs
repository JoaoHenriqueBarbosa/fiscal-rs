//! Event, cancellation, EPEC, distribution, cadastro, and miscellaneous methods.

use fiscal_core::FiscalError;
use fiscal_core::types::SefazEnvironment;

use crate::request_builders;
use crate::response_parsers::{
    self, CadastroResponse, CancellationResponse, DistDFeResponse, StatusResponse,
};
use crate::services::SefazService;

use super::SefazClient;

impl SefazClient {
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
    /// `fiscal_crypto::sign_xml` before calling this method.
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
            .send_cadastro_raw(uf, environment, &request_xml)
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

    /// Check EPEC NFC-e service status (`EPECStatusServico`, SP only).
    ///
    /// Queries the operational status of the EPEC NFC-e service. This
    /// service exists only in SP (São Paulo) for model 65 (NFC-e),
    /// matching the PHP `sefazStatusEpecNfce()` method from
    /// `TraitEPECNfce`.
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation (must be `"SP"`).
    /// * `environment` — SEFAZ environment.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    pub async fn epec_nfce_status(
        &self,
        uf: &str,
        environment: SefazEnvironment,
    ) -> Result<StatusResponse, FiscalError> {
        let request_xml = request_builders::build_epec_nfce_status_request(uf, environment);
        let raw = self
            .send_model(
                SefazService::EpecNfceStatusServico,
                uf,
                environment,
                &request_xml,
                65,
            )
            .await?;
        response_parsers::parse_status_response(&raw)
    }

    /// Submit an EPEC event for NFC-e (`RecepcaoEPEC`, tpEvento=110140,
    /// SP only).
    ///
    /// The EPEC NFC-e event is sent to the state's `RecepcaoEPEC` endpoint
    /// (not Ambiente Nacional). This is only available in SP and differs
    /// from the standard EPEC: no `<vST>`, optional `<dest>`, and
    /// `cOrgao` is the state's IBGE code.
    ///
    /// Matches the PHP `sefazEpecNfce()` method from `TraitEPECNfce`.
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation (must be `"SP"`).
    /// * `epec_data` — Pre-extracted NFC-e data for the EPEC event.
    /// * `environment` — SEFAZ environment.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    pub async fn epec_nfce(
        &self,
        uf: &str,
        epec_data: &request_builders::EpecNfceData,
        environment: SefazEnvironment,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_epec_nfce_request(epec_data, environment);
        let raw = self
            .send_model(
                SefazService::RecepcaoEpecNfce,
                uf,
                environment,
                &request_xml,
                65,
            )
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
        // NT 2024.002: model 55 uses SVRS, model 65 uses state endpoint
        let effective_uf = if model == 55 { "SVRS" } else { uf };
        // NT 2024.002 / NT 2025.002: cOrgao=92 when sending via SVRS
        let org_override = if effective_uf == "SVRS" {
            Some("92")
        } else {
            None
        };
        let request_xml = request_builders::build_conciliacao_request(
            access_key,
            ver_aplic,
            det_pag,
            cancel,
            cancel_protocol,
            seq,
            environment,
            tax_id,
            org_override,
        );
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
}
