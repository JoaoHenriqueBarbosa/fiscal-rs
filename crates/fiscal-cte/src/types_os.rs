//! Types for the CT-e OS (Outros Serviços, model 67), leiaute 4.00.
//!
//! Reuses the shared CT-e blocks (`Emit`, `Endereco`, `VPrest`, `Imp`, `Compl`,
//! `AutXml`, `InfRespTec`, `Documento`) from [`crate::types`]; only the
//! OS-specific top-level shape is declared here: a single `<toma>` (taker), an
//! `<infServico>` instead of cargo/documents, referenced documents
//! (`<infDocRef>`), insurance (`<seg>`) and the road-OS modal (`<rodoOS>`).

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use ts_rs::TS;

use crate::types::{AutXml, Compl, Documento, Emit, Endereco, Imp, InfRespTec, VPrest};

/// Root build data for a CT-e OS document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct CteOsBuildData {
    /// `cCT` numeric code (8 digits); generated when `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numeric_code: Option<String>,
    /// Issuer CNPJ — used to build the access key.
    pub emit_cnpj: String,
    pub ide: IdeOs,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compl: Option<Compl>,
    pub emit: Emit,
    pub toma: TomaOs,
    pub v_prest: VPrest,
    pub imp: Imp,
    pub inf_cte_norm: InfCteNormOs,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aut_xml: Vec<AutXml>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inf_resp_tec: Option<InfRespTec>,
}

/// `<ide>` for CT-e OS — no toma3/toma4 (the taker is the standalone `<toma>`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct IdeOs {
    pub c_uf: String,
    pub cfop: String,
    pub nat_op: String,
    pub serie: u32,
    pub n_ct: u32,
    pub dh_emi: chrono::DateTime<chrono::FixedOffset>,
    pub tp_imp: String,
    pub tp_emis: String,
    pub tp_amb: String,
    /// `tpCTe` — `0` Normal, `1` Complemento, `3` Substituto.
    pub tp_cte: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proc_emi: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ver_proc: Option<String>,
    pub c_mun_env: String,
    pub x_mun_env: String,
    pub uf_env: String,
    /// `modal` — `01` rodoviário (único suportado no OS por ora).
    pub modal: String,
    /// `tpServ` — OS: `6` transporte de pessoas, `7` transporte de valores,
    /// `8` excesso de bagagem.
    pub tp_serv: String,
    pub ind_ie_toma: String,
    pub c_mun_ini: String,
    pub x_mun_ini: String,
    pub uf_ini: String,
    pub c_mun_fim: String,
    pub x_mun_fim: String,
    pub uf_fim: String,
    /// `infPercurso` — UFs intermediárias do percurso (na ordem).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inf_percurso: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dh_cont: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_just: Option<String>,
}

/// `<toma>` — the single taker of the OS service.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct TomaOs {
    pub doc: Documento,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ie: Option<String>,
    pub x_nome: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_fant: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fone: Option<String>,
    pub ender_toma: Endereco,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// `<infCTeNorm>` for CT-e OS.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfCteNormOs {
    pub inf_servico: InfServico,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inf_doc_ref: Vec<InfDocRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub seg: Vec<Seg>,
    pub inf_modal: InfModalOs,
}

/// `<infServico>` — description of the service and total quantity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfServico {
    pub x_desc_serv: String,
    /// `infQ/qCarga` — total quantity transported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub q_carga: Option<String>,
}

/// `<infDocRef>` — a referenced document (não NF-e).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfDocRef {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_doc: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serie: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subserie: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub d_emi: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_doc: Option<String>,
    /// `chBPe` — chave de um BP-e referenciado (alternativa aos campos acima).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ch_bpe: Option<String>,
}

/// `<seg>` — insurance group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Seg {
    /// `respSeg` — `4` emitente, `5` tomador.
    pub resp_seg: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_seg: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_apol: Option<String>,
}

/// `<infModal>` para OS — por ora apenas o modal rodoviário (`rodoOS`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfModalOs {
    /// `versaoModal` (`"4.00"`).
    pub versao_modal: String,
    pub rodo_os: RodoOs,
}

/// `<rodoOS>` — road modal for OS.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct RodoOs {
    /// `TAF` — Termo de Autorização de Fretamento.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taf: Option<String>,
    /// `NroRegEstadual` — registro estadual (alternativa ao TAF).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nro_reg_estadual: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub veic: Option<VeicOs>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inf_fretamento: Option<InfFretamento>,
}

/// `<veic>` — veículo do rodoOS.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct VeicOs {
    pub placa: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub renavam: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prop: Option<PropOs>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uf: Option<String>,
}

/// `<prop>` — proprietário do veículo (quando não é o emitente).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct PropOs {
    pub doc: Documento,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_nome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ie: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uf: Option<String>,
    /// `tpProp` — `0` TAC Agregado, `1` TAC Independente, `2` Outros.
    pub tp_prop: String,
}

/// `<infFretamento>` — dados de fretamento.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfFretamento {
    /// `tpFretamento` — `1` eventual, `2` contínuo.
    pub tp_fretamento: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dh_viagem: Option<String>,
}
