//! RTC (Reforma Tributária do Consumo) event methods.

use fiscal_core::FiscalError;
use fiscal_core::types::SefazEnvironment;

use crate::request_builders::{self, RtcCredPresItem, RtcItem};
use crate::response_parsers::{self, CancellationResponse};
use crate::services::SefazService;

use super::{SefazClient, svrs_org_override};

impl SefazClient {
    // ── RTC (Reforma Tributaria) typed convenience methods ──────────────

    /// Send an RTC event via SVRS RecepcaoEvento.
    async fn send_rtc_event(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        request_xml: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let raw = self
            .send(SefazService::RecepcaoEvento, uf, environment, request_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// RTC: Informacao de pagamento integral (tpEvento=112110).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_info_pagto_integral(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_info_pagto_integral(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Aceite de debito na apuracao (tpEvento=211128).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_aceite_debito(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        ind_aceitacao: u8,
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_aceite_debito(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            ind_aceitacao,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Manifestacao transferencia credito IBS (tpEvento=212110).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_manif_transf_cred_ibs(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        ind_aceitacao: u8,
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_manif_transf_cred_ibs(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            ind_aceitacao,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Manifestacao transferencia credito CBS (tpEvento=212120).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_manif_transf_cred_cbs(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        ind_aceitacao: u8,
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_manif_transf_cred_cbs(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            ind_aceitacao,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Cancelamento de evento (tpEvento=110001).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_cancela_evento(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        tp_evento_aut: &str,
        n_prot_evento: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_cancela_evento(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            tp_evento_aut,
            n_prot_evento,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Atualizacao da data de previsao de entrega (tpEvento=112150).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_atualizacao_data_entrega(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        data_prevista: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_atualizacao_data_entrega(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            data_prevista,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Importacao via ZFM (tpEvento=112120).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_importacao_zfm(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        itens: &[RtcItem],
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_importacao_zfm(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            itens,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Roubo/perda em transporte pelo fornecedor (tpEvento=112130).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_roubo_perda_fornecedor(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        itens: &[RtcItem],
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_roubo_perda_fornecedor(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            itens,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Fornecimento nao realizado (tpEvento=112140).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_fornecimento_nao_realizado(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        itens: &[RtcItem],
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_fornecimento_nao_realizado(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            itens,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Solicitacao de apropriacao de credito presumido (tpEvento=211110).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_sol_aprop_cred_presumido(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        itens: &[RtcCredPresItem],
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_sol_aprop_cred_presumido(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            itens,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Destinacao de item para consumo pessoal (tpEvento=211120).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_destino_consumo_pessoal(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        tp_autor: u8,
        ver_aplic: &str,
        itens: &[RtcItem],
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_destino_consumo_pessoal(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            tp_autor,
            ver_aplic,
            itens,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Perecimento/roubo transporte adquirente (tpEvento=211124).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_roubo_perda_adquirente(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        itens: &[RtcItem],
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_roubo_perda_adquirente(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            itens,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Imobilizacao de item (tpEvento=211130).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_imobilizacao_item(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        itens: &[RtcItem],
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_imobilizacao_item(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            itens,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Apropriacao de credito combustivel (tpEvento=211140).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_apropriacao_credito_comb(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        itens: &[RtcItem],
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_apropriacao_credito_comb(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            itens,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Apropriacao de credito bens/servicos (tpEvento=211150).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_apropriacao_credito_bens(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        itens: &[RtcItem],
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_apropriacao_credito_bens(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            itens,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }
}
