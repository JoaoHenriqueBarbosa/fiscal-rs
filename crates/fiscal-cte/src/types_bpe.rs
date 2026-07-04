//! Types for the BP-e (Bilhete de Passagem eletrônico, model 63), leiaute 1.00.
//!
//! BP-e is a **separate** fiscal document (namespace `.../bpe`), not a CT-e
//! variant — but the 44-digit access-key layout matches, so the key builder is
//! reused with model `63`. Shared `Endereco`/`Documento`/`Icms`/`AutXml`/
//! `InfRespTec` are reused from [`crate::types`].

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use ts_rs::TS;

use crate::types::{AutXml, Documento, Endereco, Icms, InfRespTec};

/// Root build data for a BP-e document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct BpeBuildData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numeric_code: Option<String>,
    pub emit_cnpj: String,
    pub ide: IdeBpe,
    pub emit: BpeEmit,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comp: Option<Comprador>,
    pub inf_valor: InfValorBpe,
    pub inf_viagem: Vec<InfViagem>,
    pub inf_passagem: InfPassagem,
    pub imp: BpeImp,
    pub pag: Vec<Pagamento>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aut_xml: Vec<AutXml>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inf_resp_tec: Option<InfRespTec>,
}

/// `<ide>` for BP-e.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct IdeBpe {
    pub c_uf: String,
    pub tp_amb: String,
    pub serie: u32,
    pub n_bp: u32,
    /// `modal` — `1` Rodoviário, `2` Aquaviário, `3` Ferroviário.
    pub modal: String,
    pub dh_emi: chrono::DateTime<chrono::FixedOffset>,
    pub tp_emis: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ver_proc: Option<String>,
    /// `tpBPe` — `0` Normal, `1` BP-e de substituição.
    pub tp_bpe: String,
    /// `indPres` — `1` presencial, etc.
    pub ind_pres: String,
    pub uf_ini: String,
    pub c_mun_ini: String,
    pub uf_fim: String,
    pub c_mun_fim: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dh_cont: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_just: Option<String>,
}

/// `<emit>` for BP-e (carries IM?/CRT + TAR).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct BpeEmit {
    pub cnpj: String,
    pub ie: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iest: Option<String>,
    pub x_nome: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_fant: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub im: Option<String>,
    pub crt: String,
    pub ender_emit: Endereco,
    /// `TAR` — número de registro na ANTT (autorização de fretamento/linha).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tar: Option<String>,
}

/// `<comp>` — comprador (passageiro pagante), opcional.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Comprador {
    pub x_nome: String,
    pub doc: Documento,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ie: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ender_comp: Option<Endereco>,
}

/// `<infValorBPe>` — valores do bilhete.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfValorBpe {
    pub v_bp: String,
    /// `vDesconto` — obrigatório (use "0.00" quando não houver desconto).
    pub v_desconto: String,
    pub v_pgto: String,
    /// `vTroco` — obrigatório (use "0.00").
    pub v_troco: String,
    /// `Comp` — componentes do valor (≥1). tpComp `00` Tarifa, `01` Pedágio,
    /// `02` Taxa de embarque, `03` Seguro, `04` Outros.
    pub comp: Vec<CompBpe>,
}

/// `<Comp>` — componente do valor do BP-e.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct CompBpe {
    pub tp_comp: String,
    pub v_comp: String,
}

/// `<infViagem>` — dados da viagem.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfViagem {
    /// `cPercurso` — código do percurso. `xPercurso` — descrição.
    pub c_percurso: String,
    pub x_percurso: String,
    /// `tpViagem` — `00` Regular/Normal, `01` Viagem Extra, etc.
    pub tp_viagem: String,
    /// `tpServ` — `1` Regular, `2` Extra, `3` Fretamento, ...
    pub tp_serv: String,
    /// `tpAcomodacao` — `1` Leito, `2` Semileito, `3` Cama, `4` Executivo, etc.
    pub tp_acomodacao: String,
    /// `tpTrecho` — `1` Origem/Destino, `2` Conexão, `3` Escala.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tp_trecho: Option<String>,
    /// `dhViagem` — data/hora da viagem.
    pub dh_viagem: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefixo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poltrona: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plataforma: Option<String>,
}

/// `<infPassagem>` — origem/destino da passagem.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfPassagem {
    pub c_loc_orig: String,
    pub x_loc_orig: String,
    pub c_loc_dest: String,
    pub x_loc_dest: String,
    /// `dhEmb` — data/hora de embarque (obrigatório).
    pub dh_emb: String,
    /// `dhValidade` — validade do bilhete (obrigatório).
    pub dh_validade: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passageiro: Option<Passageiro>,
}

/// `<infPassageiro>` — dados do passageiro (opcional).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Passageiro {
    pub x_nome: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpf: Option<String>,
    /// `tpDoc` — `1` RG, `2` CNH, etc.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tp_doc: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_doc: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fone: Option<String>,
}

/// `<imp>` — impostos do BP-e.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct BpeImp {
    pub icms: Icms,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_tot_trib: Option<String>,
}

/// `<pag>` — forma de pagamento.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Pagamento {
    /// `tPag` — `01` Dinheiro, `03` Cartão crédito, `04` Cartão débito, `05`
    /// Crédito loja, `15` Boleto, `17` PIX, `99` Outros.
    pub t_pag: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_pag: Option<String>,
    pub v_pag: String,
}
