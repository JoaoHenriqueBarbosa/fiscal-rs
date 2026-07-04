//! Types and XML generation for the Brazilian **CT-e** model 57, leiaute 4.00.
//!
//! Mirrors the structure of [`fiscal_core`] / `fiscal_mdfe`: one strongly
//! typed struct per `<CTe>/infCte` block, in schema order. The first release
//! targets **CT-e Normal** with the **road** modal (`infModal/rodo`); rarely
//! used optional blocks (docAnt, cobr, veicNovos, fluxo, Entrega, …) are
//! omitted for now and can be added without breaking existing callers.
//!
//! Field order in each struct deliberately matches the XSD `xs:sequence` so the
//! string builder can serialize top-to-bottom.

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use ts_rs::TS;

// ── top-level build data ─────────────────────────────────────────────────────

/// Everything required to build a complete CT-e Normal `<CTe>` document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct CteBuildData {
    /// `<ide>` — identification block.
    pub ide: Ide,
    /// `<compl>` — optional complementary information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compl: Option<Compl>,
    /// `<emit>` — issuer (transporter).
    pub emit: Emit,
    /// `<rem>` — sender (remetente).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rem: Option<Party>,
    /// `<exped>` — dispatcher (expedidor).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exped: Option<Party>,
    /// `<receb>` — receiver (recebedor).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receb: Option<Party>,
    /// `<dest>` — recipient (destinatário).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dest: Option<Party>,
    /// `<vPrest>` — service price block.
    pub v_prest: VPrest,
    /// `<imp>` — taxes (ICMS).
    pub imp: Imp,
    /// `<infCTeNorm>` — normal CT-e payload (cargo, documents, modal). Usado
    /// para CT-e Normal (tpCTe 0) e Substituto (tpCTe 3).
    pub inf_cte_norm: InfCteNorm,
    /// `<infCteComp>` — chaves do(s) CT-e complementado(s). Quando **não vazio**,
    /// o documento é um **Complementar** (tpCTe 1): emite-se `infCteComp` no lugar
    /// de `infCTeNorm`, e os valores complementares vão em `vPrest`/`imp`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inf_cte_comp: Vec<String>,
    /// `<autXML>` — parties authorized to download the XML.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aut_xml: Vec<AutXml>,
    /// `<infRespTec>` — technical responsible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inf_resp_tec: Option<InfRespTec>,
    /// Issuer CNPJ — used for the access key (not emitted directly here).
    pub emit_cnpj: String,
    /// Optional explicit 8-digit `cCT` random code. When `None`, a code is
    /// generated at build time. Provided mainly for deterministic tests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numeric_code: Option<String>,
}

// ── ide ──────────────────────────────────────────────────────────────────────

/// `<ide>` — CT-e identification block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Ide {
    /// `cUF` — issuer state IBGE code (2 digits).
    pub c_uf: String,
    /// `CFOP` — fiscal operation code (4 digits).
    pub cfop: String,
    /// `natOp` — operation nature description.
    pub nat_op: String,
    /// `serie` — document series.
    pub serie: u32,
    /// `nCT` — sequential document number.
    pub n_ct: u32,
    /// `dhEmi` — emission timestamp.
    pub dh_emi: chrono::DateTime<chrono::FixedOffset>,
    /// `tpImp` — DACTE layout: `1` = portrait, `2` = landscape.
    pub tp_imp: String,
    /// `tpEmis` — emission type: `1` = normal, others = contingency.
    pub tp_emis: String,
    /// `tpAmb` — environment: `1` = production, `2` = homologation.
    pub tp_amb: String,
    /// `tpCTe` — `0` Normal, `1` Complemento, `2` Anulação, `3` Substituto.
    pub tp_cte: String,
    /// `procEmi` — emission process code (usually `0`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proc_emi: Option<String>,
    /// `verProc` — emitting-application version string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ver_proc: Option<String>,
    /// `indGlobalizado` — globalized flag (`1`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ind_globalizado: Option<String>,
    /// `cMunEnv` — emission municipality IBGE code (7 digits).
    pub c_mun_env: String,
    /// `xMunEnv` — emission municipality name.
    pub x_mun_env: String,
    /// `UFEnv` — emission municipality state.
    pub uf_env: String,
    /// `modal` — transport modal: `01` road, `02` air, `03` waterway,
    /// `04` rail, `05` pipeline, `06` multimodal.
    pub modal: String,
    /// `tpServ` — `0` Normal, `1` Subcontratação, `2` Redespacho,
    /// `3` Redespacho Intermediário, `4` Serviço Vinculado a Multimodal.
    pub tp_serv: String,
    /// `cMunIni` — service start municipality IBGE code.
    pub c_mun_ini: String,
    /// `xMunIni` — service start municipality name.
    pub x_mun_ini: String,
    /// `UFIni` — service start state.
    pub uf_ini: String,
    /// `cMunFim` — service end municipality IBGE code.
    pub c_mun_fim: String,
    /// `xMunFim` — service end municipality name.
    pub x_mun_fim: String,
    /// `UFFim` — service end state.
    pub uf_fim: String,
    /// `retira` — pickup at issuer: `0` = no, `1` = yes.
    pub retira: String,
    /// `xDetRetira` — pickup details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_det_retira: Option<String>,
    /// `indIEToma` — taker IE indicator: `1` contributor, `2` exempt,
    /// `9` non-contributor.
    pub ind_ie_toma: String,
    /// Taker (tomador) — either a reference to an existing party (`toma3`) or
    /// a fully described "other" taker (`toma4`).
    pub toma: Tomador,
}

/// `<toma3>` / `<toma4>` — who pays for the transport service.
// `Toma4` carrega o tomador completo (endereço, nome, doc…) e é o caso comum;
// `Toma3` é só um código. Boxear a variante grande complicaria o pattern-match
// em todo o builder sem ganho real — preferimos manter a ergonomia.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Tomador {
    /// `toma3` — the taker is one of the declared parties.
    /// `toma`: `0` rem, `1` exped, `2` receb, `3` dest.
    Toma3 {
        /// `toma` code (`0`–`3`).
        toma: String,
    },
    /// `toma4` — the taker is a separate party, fully described.
    Toma4 {
        /// `toma` code (always `4`).
        #[serde(default = "toma4_code")]
        toma: String,
        /// `CNPJ` or `CPF`.
        doc: Documento,
        /// `IE` — state registration.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ie: Option<String>,
        /// `xNome` — name.
        x_nome: String,
        /// `xFant` — trade name.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        x_fant: Option<String>,
        /// `fone` — phone.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fone: Option<String>,
        /// `enderToma` — address.
        ender_toma: Endereco,
        /// `email`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        email: Option<String>,
    },
}

fn toma4_code() -> String {
    "4".to_string()
}

/// CNPJ or CPF — most CT-e party blocks accept either.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
#[serde(rename_all = "UPPERCASE")]
pub enum Documento {
    /// `CNPJ` (14 digits).
    Cnpj(String),
    /// `CPF` (11 digits).
    Cpf(String),
}

// ── compl ────────────────────────────────────────────────────────────────────

/// `<compl>` — complementary information (subset: observations).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Compl {
    /// `xCaracAd` — additional characteristics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_carac_ad: Option<String>,
    /// `xCaracSer` — service characteristics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_carac_ser: Option<String>,
    /// `xEmi` — issuer operator name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_emi: Option<String>,
    /// `xObs` — free-form observations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_obs: Option<String>,
    /// `ObsCont` — taxpayer observation fields (xCampo/xTexto).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obs_cont: Vec<ObsCampo>,
    /// `ObsFisco` — fisco observation fields.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obs_fisco: Vec<ObsCampo>,
}

/// `ObsCont` / `ObsFisco` — a named observation field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct ObsCampo {
    /// `xCampo` — field name attribute.
    pub x_campo: String,
    /// `xTexto` — field value.
    pub x_texto: String,
}

// ── emit ─────────────────────────────────────────────────────────────────────

/// `<emit>` — issuer (transporter).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Emit {
    /// `CNPJ` or `CPF`.
    pub doc: Documento,
    /// `IE` — state registration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ie: Option<String>,
    /// `IEST` — substitute-tax state registration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iest: Option<String>,
    /// `xNome` — corporate name.
    pub x_nome: String,
    /// `xFant` — trade name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_fant: Option<String>,
    /// `enderEmit` — issuer address.
    pub ender_emit: Endereco,
    /// `CRT` — tax regime: `1` Simples, `2` Simples excesso, `3` Normal.
    pub crt: String,
}

// ── parties (rem/exped/receb/dest) ───────────────────────────────────────────

/// A generic CT-e party (sender, dispatcher, receiver, recipient).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Party {
    /// `CNPJ` or `CPF`.
    pub doc: Documento,
    /// `IE` — state registration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ie: Option<String>,
    /// `xNome` — name.
    pub x_nome: String,
    /// `xFant` — trade name (sender only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_fant: Option<String>,
    /// `fone` — phone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fone: Option<String>,
    /// `ISUF` — SUFRAMA registration (recipient only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isuf: Option<String>,
    /// Address (`enderReme`/`enderExped`/`enderReceb`/`enderDest`).
    pub ender: Endereco,
    /// `email`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// A CT-e address (`TEndereco` / `TEndeEmi`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Endereco {
    /// `xLgr` — street.
    pub x_lgr: String,
    /// `nro` — number.
    pub nro: String,
    /// `xCpl` — complement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_cpl: Option<String>,
    /// `xBairro` — neighbourhood.
    pub x_bairro: String,
    /// `cMun` — IBGE municipality code (7 digits).
    pub c_mun: String,
    /// `xMun` — municipality name.
    pub x_mun: String,
    /// `CEP` — postal code (8 digits).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cep: Option<String>,
    /// `UF` — state abbreviation (`EX` for abroad).
    pub uf: String,
    /// `cPais` — country IBGE code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub c_pais: Option<String>,
    /// `xPais` — country name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_pais: Option<String>,
    /// `fone` — phone (issuer address only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fone: Option<String>,
}

// ── vPrest ───────────────────────────────────────────────────────────────────

/// `<vPrest>` — service price.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct VPrest {
    /// `vTPrest` — total service value.
    pub v_t_prest: String,
    /// `vRec` — value to be received.
    pub v_rec: String,
    /// `Comp` — price components (name + value).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comp: Vec<Componente>,
}

/// `<Comp>` — a single price component.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Componente {
    /// `xNome` — component name.
    pub x_nome: String,
    /// `vComp` — component value.
    pub v_comp: String,
}

// ── imp / ICMS ───────────────────────────────────────────────────────────────

/// `<imp>` — tax block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Imp {
    /// `ICMS` — ICMS group.
    pub icms: Icms,
    /// `vTotTrib` — approximate total taxes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_tot_trib: Option<String>,
    /// `infAdFisco` — additional fisco information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inf_ad_fisco: Option<String>,
}

/// `<ICMS>` — the ICMS taxation group (`TImp` choice).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
#[serde(tag = "cst")]
pub enum Icms {
    /// `ICMS00` — full taxation (CST 00).
    #[serde(rename = "00")]
    Icms00 {
        /// `vBC` — calculation base.
        v_bc: String,
        /// `pICMS` — rate.
        p_icms: String,
        /// `vICMS` — tax value.
        v_icms: String,
    },
    /// `ICMS20` — base reduction (CST 20).
    #[serde(rename = "20")]
    Icms20 {
        /// `pRedBC` — base reduction percentage.
        p_red_bc: String,
        /// `vBC` — calculation base.
        v_bc: String,
        /// `pICMS` — rate.
        p_icms: String,
        /// `vICMS` — tax value.
        v_icms: String,
    },
    /// `ICMS45` — exempt/non-taxed/deferred (CST 40, 41, 51).
    #[serde(rename = "45")]
    Icms45 {
        /// CST code (`40`, `41`, or `51`).
        #[serde(rename = "CST")]
        cst_code: String,
    },
    /// `ICMS90` — others (CST 90).
    #[serde(rename = "90")]
    Icms90 {
        /// `pRedBC` — base reduction.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        p_red_bc: Option<String>,
        /// `vBC` — calculation base.
        v_bc: String,
        /// `pICMS` — rate.
        p_icms: String,
        /// `vICMS` — tax value.
        v_icms: String,
        /// `vCred` — credit value.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        v_cred: Option<String>,
    },
    /// `ICMSSN` — Simples Nacional (CST 90, `indSN=1`).
    #[serde(rename = "SN")]
    IcmsSn {
        /// `indSN` — always `1`.
        #[serde(default = "ind_sn_default")]
        ind_sn: String,
    },
}

fn ind_sn_default() -> String {
    "1".to_string()
}

// ── infCTeNorm ───────────────────────────────────────────────────────────────

/// `<infCTeNorm>` — normal CT-e payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfCteNorm {
    /// `infCarga` — cargo information.
    pub inf_carga: InfCarga,
    /// `infDoc` — transported documents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inf_doc: Option<InfDoc>,
    /// `infModal` — modal-specific block (road for now).
    pub inf_modal: InfModal,
    /// `infCteSub` — informação do CT-e substituído (tpCTe 3 Substituto).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inf_cte_sub: Option<InfCteSub>,
    /// `<seg>` — informações de seguro da carga (0..N).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub seg: Vec<SegCte>,
}

/// `<seg>` — informações de seguro da carga (RCTRC obrigatório - Lei 11.442/07).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct SegCte {
    /// `respSeg` — responsável pelo seguro: `4` emitente, `5` tomador.
    pub resp_seg: String,
    /// `xSeg` — nome da seguradora.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_seg: Option<String>,
    /// `CNPJ` — CNPJ da seguradora.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cnpj_seg: Option<String>,
    /// `nApol` — número da apólice (obrigatório pela Lei 11.442/07).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_apol: Option<String>,
}

/// `<infCteSub>` — substituição (CT-e Substituto, tpCTe 3).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfCteSub {
    /// `chCte` — chave do CT-e a ser substituído.
    pub ch_cte: String,
    /// `indAlteraToma` — `1` quando o tomador foi alterado na substituição.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ind_altera_toma: Option<String>,
}

/// `<infCarga>` — cargo information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfCarga {
    /// `vCarga` — total cargo value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_carga: Option<String>,
    /// `proPred` — predominant product.
    pub pro_pred: String,
    /// `xOutCat` — other cargo category.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_out_cat: Option<String>,
    /// `infQ` — quantity measures (at least one).
    pub inf_q: Vec<InfQ>,
    /// `vCargaAverb` — insured cargo value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_carga_averb: Option<String>,
}

/// `<infQ>` — a cargo quantity measure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfQ {
    /// `cUnid` — unit code: `00` m³, `01` kg, `02` ton, `03` unit, `04` litres,
    /// `05` MMBTU.
    pub c_unid: String,
    /// `tpMed` — measure type description.
    pub tp_med: String,
    /// `qCarga` — quantity.
    pub q_carga: String,
}

/// `<infDoc>` — transported documents (subset: linked NF-e keys).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfDoc {
    /// `infNFe` — linked NF-e access keys.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inf_nfe: Vec<InfNfe>,
    /// `infOutros` — outros documentos (declaração, NFC-e, CF-e/SAT, outros).
    /// O schema é um *choice*: ou `infNFe`, ou `infOutros` (não misturados).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inf_outros: Vec<InfOutros>,
}

/// `<infOutros>` — documento transportado que não é NF-e.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfOutros {
    /// `tpDoc` — `00` Declaração, `10` Dutoviário, `59` CF-e/SAT, `60` NFC-e,
    /// `99` Outros.
    pub tp_doc: String,
    /// `descOutros` — descrição (obrigatório quando `tpDoc=99`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desc_outros: Option<String>,
    /// `nDoc` — número do documento.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_doc: Option<String>,
    /// `dEmi` — data de emissão (`AAAA-MM-DD`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub d_emi: Option<String>,
    /// `vDocFisc` — valor do documento.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_doc_fisc: Option<String>,
}

/// `<infNFe>` — a linked NF-e.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfNfe {
    /// `chave` — 44-digit NF-e access key.
    pub chave: String,
    /// `dPrev` — expected delivery date (`AAAA-MM-DD`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub d_prev: Option<String>,
}

/// `<infModal>` — modal-specific block. Suporta rodoviário + não-rodoviários.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfModal {
    /// `@versaoModal` — modal layout version (`4.00`).
    #[serde(default = "modal_version")]
    pub versao_modal: String,
    /// Modal específico (rodo/aéreo/aquav/ferrov/duto/multimodal).
    pub modal: Modal,
}

fn modal_version() -> String {
    "4.00".to_string()
}

/// Modal de transporte do CT-e. A tag emitida segue a variante.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
#[serde(tag = "tipo", rename_all = "lowercase")]
pub enum Modal {
    /// `<rodo>` — rodoviário.
    Rodo(RodoCte),
    /// `<aereo>` — aéreo.
    Aereo {
        /// `dPrevAereo` — data prevista da entrega (AAAA-MM-DD).
        d_prev_aereo: String,
        /// `natCarga/xDime` — dimensões (AxLxC cm), opcional.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        x_dime: Option<String>,
        /// `tarifa/CL` — classe da tarifa.
        tarifa_cl: String,
        /// `tarifa/vTar` — valor da tarifa.
        tarifa_v_tar: String,
        /// `nMinu` — número da minuta, opcional.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        n_minu: Option<String>,
    },
    /// `<aquav>` — aquaviário.
    Aquav {
        v_prest: String,
        /// `vAFRMM` — Adicional ao Frete p/ Renovação da Marinha Mercante.
        v_afrmm: String,
        x_navio: String,
        /// `direc` — sentido: N, S, L, O.
        direc: String,
        /// `irin` — Identificação do Registro Irin da embarcação.
        irin: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        n_viag: Option<String>,
    },
    /// `<ferrov>` — ferroviário.
    Ferrov {
        /// `tpTraf` — `0` próprio, `1` mútuo.
        tp_traf: String,
        /// `fluxo` — número do fluxo ferroviário.
        fluxo: String,
    },
    /// `<duto>` — dutoviário.
    Duto {
        /// `dIni` — data de início (AAAA-MM-DD).
        d_ini: String,
        /// `dFim` — data de fim (AAAA-MM-DD).
        d_fim: String,
        /// `vTar` — valor da tarifa, opcional.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        v_tar: Option<String>,
    },
    /// `<multimodal>` — multimodal.
    Multimodal {
        /// `COTM` — número do Certificado do Operador de Transporte Multimodal.
        cotm: String,
        /// `indNegociavel` — `0` não negociável, `1` negociável.
        ind_negociavel: String,
    },
}

/// Modal rodoviário CT-e com campos ANTT.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct RodoCte {
    /// `RNTRC` — registro ANTT do transportador (8 dígitos ou `ISENTO`).
    pub rntrc: String,
    /// `infCIOT` — CIOTs vinculados (0..N). Obrigatório desde 24/05/2026 (Res. 6.078).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inf_ciot: Vec<InfCiotCte>,
    /// `valePed` — vale pedágio obrigatório (0..N). Lei 10.209/2001.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vale_ped: Vec<ValePedCte>,
}

/// `infCIOT` — entrada de CIOT no CT-e.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfCiotCte {
    /// `CIOT` — código de 12 dígitos.
    pub ciot: String,
    /// CNPJ ou CPF do responsável (tag escolhida pelo tamanho).
    pub tax_id: String,
}

/// `valePed` no CT-e modal rodoviário (`<infANTT>`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct ValePedCte {
    /// `CNPJForn` — CNPJ do fornecedor habilitado VPO (FVPO).
    pub cnpj_forn: String,
    /// `nCompra` — IDVPO gerado pela ANTT.
    pub n_compra: String,
    /// `vValePed` — valor do vale pedágio em reais.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_vale_ped: Option<String>,
}

// ── autXML / infRespTec ──────────────────────────────────────────────────────

/// `<autXML>` — a party authorized to download the XML.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct AutXml {
    /// `CNPJ` or `CPF`.
    pub doc: Documento,
}

/// `<infRespTec>` — technical responsible (`TRespTec`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfRespTec {
    /// `CNPJ`.
    pub cnpj: String,
    /// `xContato` — contact name.
    pub x_contato: String,
    /// `email`.
    pub email: String,
    /// `fone`.
    pub fone: String,
}
