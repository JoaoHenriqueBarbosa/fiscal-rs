//! Modelo **comum** de NFS-e municipal — independente de provedor.
//!
//! Cada provedor (ABRASF/DSF/GINFES/SigISS/SP/SpeedGov) recebe este modelo e
//! produz o XML/JSON específico. A interface padrão fica em [`crate::provider`].

use serde::{Deserialize, Serialize};

/// Ambiente de transmissão.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ambiente {
    Producao,
    Homologacao,
}

impl Ambiente {
    pub fn from_tp_amb(tp: i16) -> Self {
        if tp == 1 {
            Ambiente::Producao
        } else {
            Ambiente::Homologacao
        }
    }
}

/// Endereço genérico.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endereco {
    pub logradouro: String,
    pub numero: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub complemento: Option<String>,
    pub bairro: String,
    /// Código IBGE do município (7 dígitos).
    pub c_mun: String,
    pub uf: String,
    pub cep: String,
}

/// Prestador (emitente) do serviço.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Emitente {
    pub cnpj: String,
    /// Inscrição municipal (obrigatória na maioria dos municípios próprios/ABRASF).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub im: Option<String>,
    pub razao_social: String,
    /// Código IBGE do município emissor.
    pub c_mun: String,
    pub uf: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endereco: Option<Endereco>,
    /// CRT / regime (`1` Simples, ...). Usado para opção pelo Simples.
    #[serde(default)]
    pub optante_simples: bool,
}

/// Tomador do serviço (opcional em alguns municípios).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Tomador {
    /// CNPJ ou CPF (só dígitos).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub razao_social: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endereco: Option<Endereco>,
    /// Inscrição municipal do tomador (raro).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub im: Option<String>,
}

/// Dados do serviço prestado.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Servico {
    /// Valor do serviço em centavos.
    pub valor_centavos: i64,
    /// Valor das deduções em centavos (SP). Default 0.
    #[serde(default)]
    pub valor_deducoes_centavos: i64,
    /// Alíquota do ISS (%), ex.: "2.00". Opcional (alguns calculam).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aliquota_iss: Option<String>,
    /// ISS retido pelo tomador?
    #[serde(default)]
    pub iss_retido: bool,
    /// Item da lista de serviços (LC 116) — ex.: "1.01" ou "0101".
    pub item_lista_servico: String,
    /// Código de tributação do município (cTribMun / CodigoTributacaoMunicipio).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cod_tributacao_municipio: Option<String>,
    /// CNAE (alguns municípios exigem).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cnae: Option<String>,
    /// Discriminação/descrição do serviço.
    pub discriminacao: String,
    /// Município da prestação (IBGE). Default = município do emitente.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub c_mun_prestacao: Option<String>,
    // --- Campos da reforma (SP v2 / IBS-CBS) ---
    /// `NBS` — código da Nomenclatura Brasileira de Serviços (9 dígitos).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nbs: Option<String>,
    /// `cClassTrib` — classificação tributária IBS/CBS (6 dígitos).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub c_class_trib: Option<String>,
    /// `cIndOp` — código indicador da operação (6 dígitos).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub c_ind_op: Option<String>,
}

/// RPS (Recibo Provisório de Serviços) — a base de quase todos os municípios.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rps {
    pub numero: i64,
    pub serie: String,
    /// Tipo do RPS (`1` RPS, `2` Nota Conjugada, `3` Cupom).
    #[serde(default = "tipo_rps_default")]
    pub tipo: u8,
    /// Data/hora de emissão (ISO 8601).
    pub data_emissao: String,
    pub tomador: Tomador,
    pub servico: Servico,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub natureza_operacao: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regime_especial_tributacao: Option<String>,
    #[serde(default)]
    pub incentivador_cultural: bool,
    /// Intermediário do serviço (exigido por SP na assinatura do RPS — campos 13/14/15).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intermediario: Option<Intermediario>,
}

/// Intermediário do serviço (SP).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intermediario {
    /// CNPJ/CPF (só dígitos).
    pub doc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub im: Option<String>,
    #[serde(default)]
    pub iss_retido: bool,
}

fn tipo_rps_default() -> u8 {
    1
}

/// Entrada de emissão: emitente + RPS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitInput {
    pub emitente: Emitente,
    pub rps: Rps,
}

/// Entrada de cancelamento.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelInput {
    pub numero_nfse: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codigo_verificacao: Option<String>,
    pub motivo: String,
    /// Código do motivo (ABRASF: 1 erro, 2 serviço não prestado, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codigo_motivo: Option<String>,
}

/// Situação resultante.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Autorizado,
    Rejeitado,
    Processando,
    Cancelado,
}

/// Resultado de emissão/consulta/cancelamento.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitOutput {
    pub status: Status,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numero_nfse: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codigo_verificacao: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocolo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_emissao: Option<String>,
    /// XML da NFS-e (quando autorizada) ou do retorno.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub xml: Option<String>,
    /// Motivo da rejeição (lista de erros concatenada).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motivo: Option<String>,
    /// Link de consulta/visualização da nota no portal da **prefeitura**, quando
    /// o provedor o retorna (ex.: ABRASF `<Url>`). `None` p/ provedores sem URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    /// Corpo bruto do retorno (auditoria).
    pub raw: String,
}
