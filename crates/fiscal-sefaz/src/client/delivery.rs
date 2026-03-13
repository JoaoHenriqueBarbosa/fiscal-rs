//! Delivery receipt, delivery failure, prorrogação, and actor registration methods.

use fiscal_core::FiscalError;
use fiscal_core::types::SefazEnvironment;

use crate::request_builders;
use crate::response_parsers::{self, CancellationResponse};
use crate::services::SefazService;

use super::SefazClient;

impl SefazClient {
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
}
