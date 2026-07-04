//! Types for the NFS-e Nacional DPS (Declaração de Prestação de Serviço),
//! leiaute **1.01** (RTC — com grupo IBS/CBS da reforma tributária).
//!
//! Namespace `http://www.sped.fazenda.gov.br/nfse`, transporte REST.
//! O emitente monta a `<DPS>`, assina `<infDPS>` e envia ao SEFIN Nacional,
//! que devolve a NFS-e com chave de 50 dígitos.

use serde::{Deserialize, Serialize};

/// CNPJ ou CPF do emitente/tomador.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Documento {
    /// CNPJ (14 dígitos, sem pontuação).
    Cnpj(String),
    /// CPF (11 dígitos, sem pontuação).
    Cpf(String),
}

/// Root build data for a DPS document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpsBuildData {
    pub ide: IdeDps,
    pub prest: Prestador,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toma: Option<Pessoa>,
    pub serv: Servico,
    pub valores: Valores,
    /// Grupo `IBSCBS` da reforma tributária (irmão de `valores` em `infDPS`).
    /// Opcional na transição; obrigatório quando o RTC entrar em vigor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ibscbs: Option<Ibscbs>,
}

/// `<infDPS>` identification block.
///
/// O `Id` (`DPS` + 42 dígitos) é derivado de cLocEmi + tpInsc + inscrição +
/// serie + nDPS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeDps {
    /// `tpAmb` — `1` Produção, `2` Homologação.
    pub tp_amb: String,
    pub dh_emi: chrono::DateTime<chrono::FixedOffset>,
    #[serde(default = "ver_aplic_default")]
    pub ver_aplic: String,
    pub serie: String,
    pub n_dps: u64,
    /// `dCompet` — competência (AAAA-MM-DD).
    pub d_compet: String,
    /// `tpEmit` — `1` Prestador, `2` Tomador, `3` Intermediário.
    pub tp_emit: String,
    /// `cLocEmi` — código IBGE do município emitente (7 dígitos).
    pub c_loc_emi: String,
}

fn ver_aplic_default() -> String {
    "dfehub-1.0".into()
}

/// `<prest>` — prestador do serviço.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prestador {
    pub doc: Documento,
    /// `IM` — inscrição municipal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub im: Option<String>,
    pub x_nome: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<EnderNac>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    pub reg_trib: RegTrib,
}

/// `<regTrib>` — regime tributário do prestador.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegTrib {
    /// `opSimpNac` — `1` não optante, `2` MEI, `3` ME/EPP.
    pub op_simp_nac: String,
    /// `regEspTrib` — regime especial de tributação (`0` nenhum, ...).
    pub reg_esp_trib: String,
}

/// `<toma>` / `<interm>` — tomador ou intermediário.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pessoa {
    pub doc: Documento,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub im: Option<String>,
    pub x_nome: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<EnderNac>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// `<endNac>` — endereço nacional.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnderNac {
    pub x_lgr: String,
    pub nro: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_cpl: Option<String>,
    pub x_bairro: String,
    /// `cMun` — código IBGE do município (7 dígitos).
    pub c_mun: String,
    pub cep: String,
}

/// `<serv>` — dados do serviço prestado.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Servico {
    /// `cLocPrestacao` — município de prestação (IBGE).
    pub c_loc_prestacao: String,
    /// `cTribNac` — código de tributação nacional.
    pub c_trib_nac: String,
    /// `cTribMun` — código de tributação municipal (opcional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub c_trib_mun: Option<String>,
    pub x_desc_serv: String,
}

/// `<valores>` — valores e tributos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Valores {
    /// `vServ` — valor do serviço.
    pub v_serv: String,
    pub trib: Trib,
}

/// `<trib>` — tributação (ISSQN municipal + federal).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trib {
    pub trib_mun: TribMun,
    /// `tribFed` — PIS/COFINS/IR/CSLL (opcional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trib_fed: Option<TribFed>,
}

/// `<tribMun>` — ISSQN.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TribMun {
    /// `tribISSQN` — `1` operação tributável, `2` imunidade, `3` exportação,
    /// `4` não incidência.
    pub trib_issqn: String,
    /// `pAliq` — alíquota ISSQN (%). Obrigatória quando tribISSQN=1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p_aliq: Option<String>,
    /// `tpRetISSQN` — `1` não retido, `2` retido pelo tomador, `3` intermediário.
    pub tp_ret_issqn: String,
}

/// `<tribFed>` — tributos federais (PIS/COFINS + retenções CP/IRRF/CSLL).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TribFed {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub piscofins: Option<PisCofins>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_ret_cp: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_ret_irrf: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_ret_csll: Option<String>,
}

/// `<piscofins>` — grupo PIS/COFINS dentro de `tribFed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PisCofins {
    /// `CST` do PIS/COFINS (2 dígitos).
    pub cst: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_bc: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p_aliq_pis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p_aliq_cofins: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_pis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_cofins: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tp_ret: Option<String>,
}

/// `<IBSCBS>` (TCRTCInfoIBSCBS) — declaração de IBS/CBS no nível de `infDPS`.
///
/// A SEFIN calcula alíquotas e valores; a DPS apenas declara CST, classificação
/// e contexto.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ibscbs {
    /// `finNFSe` — finalidade de emissão (`0` normal, ...).
    pub fin_nfse: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ind_final: Option<String>,
    /// `cIndOp` — código indicador da operação (6 dígitos).
    pub c_ind_op: String,
    /// `indDest` — a respeito do destinatário (`0`/`1`/`2`).
    pub ind_dest: String,
    /// `CST` do IBS/CBS (3 dígitos).
    pub cst: String,
    /// `cClassTrib` — classificação tributária IBS/CBS (6 dígitos).
    pub c_class_trib: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub c_cred_pres: Option<String>,
}
