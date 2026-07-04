//! Types for the GTV-e (Guia de Transporte de Valores eletrônica, model 64),
//! leiaute 4.00.
//!
//! Reuses shared blocks (`Emit`, `Party`, `Endereco`, `Compl`, `AutXml`,
//! `InfRespTec`, `Documento`) and the single-`toma` block from
//! [`crate::types_os::TomaOs`]. GTV-e carries value-species details (`detGTV`)
//! and vehicles instead of cargo.

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use ts_rs::TS;

use crate::types::{AutXml, Compl, Emit, Endereco, InfRespTec, Party};
use crate::types_os::TomaOs;

/// Root build data for a GTV-e document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct GtveBuildData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numeric_code: Option<String>,
    pub emit_cnpj: String,
    pub ide: IdeGtve,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compl: Option<Compl>,
    pub emit: Emit,
    pub rem: Party,
    pub dest: Party,
    /// `origem` — endereço de origem do serviço (opcional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origem: Option<Endereco>,
    /// `destino` — endereço de destino do serviço (opcional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destino: Option<Endereco>,
    pub det_gtv: DetGtv,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aut_xml: Vec<AutXml>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inf_resp_tec: Option<InfRespTec>,
}

/// `<ide>` for GTV-e.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct IdeGtve {
    pub c_uf: String,
    pub cfop: String,
    pub nat_op: String,
    pub serie: u32,
    pub n_ct: u32,
    pub dh_emi: chrono::DateTime<chrono::FixedOffset>,
    pub tp_imp: String,
    pub tp_emis: String,
    pub tp_amb: String,
    pub tp_cte: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ver_proc: Option<String>,
    pub c_mun_env: String,
    pub x_mun_env: String,
    pub uf_env: String,
    pub modal: String,
    pub tp_serv: String,
    pub ind_ie_toma: String,
    /// `dhSaidaOrig` — data/hora de saída da origem (obrigatório na GTV-e).
    pub dh_saida_orig: String,
    /// `dhChegadaDest` — data/hora de chegada no destino (obrigatório).
    pub dh_chegada_dest: String,
    pub toma: TomaOs,
}

/// `<detGTV>` — detalhamento dos valores transportados.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct DetGtv {
    pub inf_especie: Vec<InfEspecie>,
    /// `qCarga` — quantidade/valor total da carga (valores).
    pub q_carga: String,
    pub inf_veiculo: Vec<InfVeiculoGtv>,
}

/// `<infEspecie>` — espécie de valor transportado.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfEspecie {
    /// `tpEspecie` — `1` Moeda/Dinheiro, `2` Cheque, `3` Moeda estrangeira,
    /// `4` Outros.
    pub tp_especie: String,
    /// `vEspecie` — valor da espécie.
    pub v_especie: String,
    /// `tpNumerario` — `1` Nacional, `2` Estrangeiro (quando aplicável).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tp_numerario: Option<String>,
    /// `xMoedaEstr` — moeda estrangeira (quando `tpNumerario=2`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_moeda_estr: Option<String>,
}

/// `<infVeiculo>` — veículo de transporte de valores.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfVeiculoGtv {
    pub placa: String,
    pub uf: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rntrc: Option<String>,
}
