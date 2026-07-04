//! Interface padrão de provedor municipal.
//!
//! A ideia: o **modelo comum** ([`crate::model`]) é único; cada município pluga
//! a diferença implementando [`MunicipalProvider`]. A orquestração de transporte
//! (SOAP/REST + mTLS) fica em cada provedor porque varia muito (ABRASF é SOAP,
//! SpeedGov/Santana é REST com layout nacional, SP é SOAP próprio).

#[cfg(feature = "client")]
use crate::error::Result;
use crate::model::Ambiente;
#[cfg(feature = "client")]
use crate::model::{CancelInput, EmitInput, EmitOutput};

/// Contexto de execução: ambiente + certificado do tenant (PFX/DER + senha)
/// para mTLS e assinatura.
#[derive(Clone)]
pub struct ProviderCtx {
    pub ambiente: Ambiente,
    pub pfx_der: Vec<u8>,
    pub senha: String,
    /// Versão de layout do provedor (SP: 1 = legado, 2 = reforma). Default 1.
    pub versao: u8,
    /// Inscrição Municipal (CCM) do prestador — necessária p/ cancelar/consultar SP.
    pub inscricao_municipal: Option<String>,
    /// CNPJ do prestador (remetente).
    pub cnpj: Option<String>,
}

impl std::fmt::Debug for ProviderCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderCtx")
            .field("ambiente", &self.ambiente)
            .field("pfx_der", &format!("<{} bytes>", self.pfx_der.len()))
            .finish()
    }
}

#[cfg(feature = "client")]
impl ProviderCtx {
    /// Constrói um `reqwest::Client` com identidade PKCS#12 (mTLS) a partir do PFX.
    pub fn http_client(&self) -> Result<reqwest::Client> {
        use crate::error::MunError;
        let identity = reqwest::Identity::from_pkcs12_der(&self.pfx_der, &self.senha)
            .map_err(|e| MunError::Transporte(format!("identidade PKCS#12: {e}")))?;
        reqwest::Client::builder()
            .identity(identity)
            .build()
            .map_err(|e| MunError::Transporte(format!("cliente HTTP: {e}")))
    }
}

/// Interface padrão que **todo** provedor municipal implementa. A diferença
/// entre municípios é plugada aqui; o hub fala sempre com este trait.
#[cfg(feature = "client")]
#[async_trait::async_trait]
pub trait MunicipalProvider: Send + Sync {
    /// Nome do provedor (ex.: "DSF", "GINFES", "SAOPAULO", "SpeedGov").
    fn nome(&self) -> &'static str;

    /// Códigos IBGE (7 dígitos) atendidos por este provedor.
    fn municipios(&self) -> &'static [&'static str];

    /// Emite uma NFS-e a partir do RPS.
    async fn emitir(&self, input: &EmitInput, ctx: &ProviderCtx) -> Result<EmitOutput>;

    /// Consulta uma NFS-e (por número/protocolo) — default: não implementado.
    async fn consultar(&self, _numero_nfse: &str, _ctx: &ProviderCtx) -> Result<EmitOutput> {
        Err(crate::error::MunError::NaoImplementado("consultar"))
    }

    /// Cancela uma NFS-e — default: não implementado.
    async fn cancelar(&self, _input: &CancelInput, _ctx: &ProviderCtx) -> Result<EmitOutput> {
        Err(crate::error::MunError::NaoImplementado("cancelar"))
    }
}
