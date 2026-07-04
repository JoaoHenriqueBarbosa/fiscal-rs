//! Provedores municipais. Cada um implementa [`MunicipalProvider`], plugando a
//! diferença sobre o modelo comum.
//!
//! - **Implementados**: São Paulo (PMSP), DSF (Sorocaba), SigISS (Caraguatatuba).
//! - **Stub (endpoint mapeado, emissão não implementada)**: Simpliss (Santana de Parnaíba).

#![cfg(feature = "client")]

use crate::error::{MunError, Result};
use crate::model::{EmitInput, EmitOutput};
use crate::provider::{MunicipalProvider, ProviderCtx};

/// URLs de webservice de um provedor por ambiente.
#[derive(Debug, Clone, Copy)]
pub struct Endpoints {
    pub homologacao: &'static str,
    pub producao: &'static str,
}

/// **DSF** — Sorocaba (ABRASF 2.03).
pub struct Dsf;
pub static DSF: Dsf = Dsf;
impl Dsf {
    pub const ENDPOINTS: Endpoints = Endpoints {
        homologacao: "https://homolsod.dsfweb.com.br/notafiscal-abrasfv203-ws/NotaFiscalSoap",
        producao: "https://notafiscal.sorocaba.sp.gov.br/notafiscal-abrasfv203-ws/NotaFiscalSoap",
    };
}
#[async_trait::async_trait]
impl MunicipalProvider for Dsf {
    fn nome(&self) -> &'static str {
        "DSF"
    }
    fn municipios(&self) -> &'static [&'static str] {
        &["3552205"]
    }
    async fn emitir(&self, input: &EmitInput, ctx: &ProviderCtx) -> Result<EmitOutput> {
        let endpoint = match ctx.ambiente {
            crate::model::Ambiente::Producao => Self::ENDPOINTS.producao,
            crate::model::Ambiente::Homologacao => Self::ENDPOINTS.homologacao,
        };
        // DSF abrasfv203: SOAPAction vazio.
        crate::abrasf::emit(input, ctx, endpoint, "").await
    }
}

/// **GINFES** — Guarulhos (ABRASF). Webservice exige mTLS (cert ICP-Brasil).
pub struct Ginfes;
pub static GINFES: Ginfes = Ginfes;
impl Ginfes {
    pub const ENDPOINTS: Endpoints = Endpoints {
        homologacao: "https://homologacao.ginfes.com.br/ServiceGinfesImpl",
        producao: "https://producao.ginfes.com.br/ServiceGinfesImpl",
    };
}
#[async_trait::async_trait]
impl MunicipalProvider for Ginfes {
    fn nome(&self) -> &'static str {
        "GINFES"
    }
    fn municipios(&self) -> &'static [&'static str] {
        &["3518800"]
    }
    async fn emitir(&self, input: &EmitInput, ctx: &ProviderCtx) -> Result<EmitOutput> {
        let endpoint = match ctx.ambiente {
            crate::model::Ambiente::Producao => Self::ENDPOINTS.producao,
            crate::model::Ambiente::Homologacao => Self::ENDPOINTS.homologacao,
        };
        crate::abrasf::emit(input, ctx, endpoint, "").await
    }
}

/// **SigISS** — Caraguatatuba (ABRASF 2.04). SOAPAction `nfs#GerarNfse`.
pub struct SigIss;
pub static SIGISS: SigIss = SigIss;
impl SigIss {
    pub const ENDPOINTS: Endpoints = Endpoints {
        homologacao: "https://testecaraguatatuba.meumunicipio.online/abrasf/ws/nfs",
        producao: "https://caraguatatuba.meumunicipio.online/abrasf/ws/nfs",
    };
}
#[async_trait::async_trait]
impl MunicipalProvider for SigIss {
    fn nome(&self) -> &'static str {
        "SigISS"
    }
    fn municipios(&self) -> &'static [&'static str] {
        &["3513801"]
    }
    async fn emitir(&self, input: &EmitInput, ctx: &ProviderCtx) -> Result<EmitOutput> {
        let endpoint = match ctx.ambiente {
            crate::model::Ambiente::Producao => Self::ENDPOINTS.producao,
            crate::model::Ambiente::Homologacao => Self::ENDPOINTS.homologacao,
        };
        crate::abrasf::emit(input, ctx, endpoint, "nfs#GerarNfse").await
    }
}

/// **São Paulo** (PMSP) — sistema próprio, RPS+lote, 2 assinaturas RSA.
pub struct SaoPaulo;
pub static SAOPAULO: SaoPaulo = SaoPaulo;
impl SaoPaulo {
    /// WS novo (layout v1+v2 com IBS/CBS). O antigo só suporta v1.
    pub const WS: &'static str = "https://nfews.prefeitura.sp.gov.br/lotenfe.asmx";
}
#[async_trait::async_trait]
impl MunicipalProvider for SaoPaulo {
    fn nome(&self) -> &'static str {
        "SAOPAULO"
    }
    fn municipios(&self) -> &'static [&'static str] {
        &["3550308"]
    }
    async fn emitir(&self, input: &EmitInput, ctx: &ProviderCtx) -> Result<EmitOutput> {
        crate::saopaulo::emit(input, ctx, Self::WS).await
    }
    async fn cancelar(
        &self,
        input: &crate::model::CancelInput,
        ctx: &ProviderCtx,
    ) -> Result<EmitOutput> {
        crate::saopaulo::cancelar(input, ctx, Self::WS).await
    }
    async fn consultar(&self, numero_nfse: &str, ctx: &ProviderCtx) -> Result<EmitOutput> {
        // numero_nfse no formato "numero|codigoVerificacao".
        let (num, cod) = numero_nfse.split_once('|').unwrap_or((numero_nfse, ""));
        crate::saopaulo::consultar(num, cod, ctx, Self::WS).await
    }
}

/// **Simpliss** — Santana de Parnaíba: layout **nacional (DPS 1.01)** em endpoint
/// municipal próprio (REST + mTLS, NÃO opera no Ambiente Nacional central).
/// Reusa o builder DPS do `fiscal-cte`; só muda a URL de POST (`/v2/nfsen`).
///
/// **Status: stub.** Endpoints mapeados; emissão pendente de implementação e
/// homologação. SpeedGov é um fornecedor distinto — este provedor é exclusivamente
/// a plataforma Simpliss (simplissweb.com.br).
pub struct Simpliss;
pub static SIMPLISS: Simpliss = Simpliss;
impl Simpliss {
    /// Base REST por ambiente; o caminho do recurso é `/v2/nfsen`.
    pub const ENDPOINTS: Endpoints = Endpoints {
        homologacao: "https://homologacaoabrasf.simplissweb.com.br/v2/nfsen",
        producao: "https://santanadeparnaiba.simplissweb.com.br/v2/nfsen",
    };
}
#[async_trait::async_trait]
impl MunicipalProvider for Simpliss {
    fn nome(&self) -> &'static str {
        "Simpliss"
    }
    fn municipios(&self) -> &'static [&'static str] {
        &["3547304"]
    }
    async fn emitir(&self, _input: &EmitInput, _ctx: &ProviderCtx) -> Result<EmitOutput> {
        // TODO: reusar o DPS nacional (fiscal-cte::build_dps_xml + sign_dps_xml) e
        // POST REST p/ Self::ENDPOINTS (mTLS). Endpoints já mapeados.
        Err(MunError::NaoImplementado(
            "Simpliss/nacional-municipal emitir",
        ))
    }
}
