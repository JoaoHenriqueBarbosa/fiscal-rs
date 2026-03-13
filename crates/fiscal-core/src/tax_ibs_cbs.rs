//! IBS/CBS (Imposto sobre Bens e Servicos / Contribuicao sobre Bens e Servicos) XML generation
//! for NF-e items -- PL_010 tax reform.
//!
//! This module provides all the data types and XML builders corresponding to
//! the PHP `TraitTagDetIBSCBS` methods:
//!
//! - [`IbsCbsData`] / [`build_ibs_cbs_xml`] -- `<IBSCBS>` element inside `<imposto>`
//! - Sub-groups: [`GIbsCbsData`], [`GIbsUfData`], [`GIbsMunData`], [`GCbsData`]
//! - [`GDifData`], [`GDevTribData`], [`GRedData`] -- optional sub-sub-groups
//! - [`GTribRegularData`] -- tributacao regular
//! - [`GTribCompraGovData`] -- compra governamental
//! - [`GIbsCbsMonoData`] -- monofasico (combustiveis)
//! - [`GTransfCredData`] -- transferencia de credito
//! - [`GCredPresIbsZfmData`] -- credito presumido ZFM
//! - [`GAjusteCompetData`] -- ajuste de competencia
//! - [`GEstornoCredData`] -- estorno de credito
//! - [`GCredPresOperData`] -- credito presumido por operacao

use crate::xml_utils::{TagContent, tag};

// ── Diferimento / Devolucao / Reducao sub-groups ──────────────────────────

/// Diferimento (deferment) data: `<gDif>` inside `<gIBSUF>`, `<gIBSMun>`, or `<gCBS>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GDifData {
    /// Percentual do diferimento (`pDif`), e.g. `"10.0000"`.
    pub p_dif: String,
    /// Valor do diferimento (`vDif`), e.g. `"5.00"`.
    pub v_dif: String,
}

impl GDifData {
    /// Create a new `GDifData`.
    pub fn new(p_dif: impl Into<String>, v_dif: impl Into<String>) -> Self {
        Self {
            p_dif: p_dif.into(),
            v_dif: v_dif.into(),
        }
    }
}

/// Devolucao de tributos data: `<gDevTrib>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GDevTribData {
    /// Valor do tributo devolvido (`vDevTrib`), e.g. `"3.00"`.
    pub v_dev_trib: String,
}

impl GDevTribData {
    /// Create a new `GDevTribData`.
    pub fn new(v_dev_trib: impl Into<String>) -> Self {
        Self {
            v_dev_trib: v_dev_trib.into(),
        }
    }
}

/// Reducao de aliquota data: `<gRed>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GRedData {
    /// Percentual da reducao de aliquota (`pRedAliq`), e.g. `"20.0000"`.
    pub p_red_aliq: String,
    /// Aliquota efetiva (`pAliqEfet`), e.g. `"15.0000"`.
    pub p_aliq_efet: String,
}

impl GRedData {
    /// Create a new `GRedData`.
    pub fn new(p_red_aliq: impl Into<String>, p_aliq_efet: impl Into<String>) -> Self {
        Self {
            p_red_aliq: p_red_aliq.into(),
            p_aliq_efet: p_aliq_efet.into(),
        }
    }
}

// ── gIBSUF / gIBSMun / gCBS sub-groups ───────────────────────────────────

/// IBS de competencia da UF: `<gIBSUF>` inside `<gIBSCBS>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GIbsUfData {
    /// Aliquota do IBS da UF (`pIBSUF`), e.g. `"18.0000"`.
    pub p_ibs_uf: String,
    /// Diferimento (`gDif`). Optional.
    pub g_dif: Option<GDifData>,
    /// Devolucao de tributos (`gDevTrib`). Optional.
    pub g_dev_trib: Option<GDevTribData>,
    /// Reducao de aliquota (`gRed`). Optional.
    pub g_red: Option<GRedData>,
    /// Valor do IBS da UF (`vIBSUF`), e.g. `"180.00"`.
    pub v_ibs_uf: String,
}

impl GIbsUfData {
    /// Create a new `GIbsUfData` with required fields.
    pub fn new(p_ibs_uf: impl Into<String>, v_ibs_uf: impl Into<String>) -> Self {
        Self {
            p_ibs_uf: p_ibs_uf.into(),
            v_ibs_uf: v_ibs_uf.into(),
            ..Default::default()
        }
    }
    /// Set diferimento.
    pub fn g_dif(mut self, v: GDifData) -> Self {
        self.g_dif = Some(v);
        self
    }
    /// Set devolucao de tributos.
    pub fn g_dev_trib(mut self, v: GDevTribData) -> Self {
        self.g_dev_trib = Some(v);
        self
    }
    /// Set reducao de aliquota.
    pub fn g_red(mut self, v: GRedData) -> Self {
        self.g_red = Some(v);
        self
    }
}

/// IBS de competencia do Municipio: `<gIBSMun>` inside `<gIBSCBS>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GIbsMunData {
    /// Aliquota do IBS do Municipio (`pIBSMun`), e.g. `"5.0000"`.
    pub p_ibs_mun: String,
    /// Diferimento (`gDif`). Optional.
    pub g_dif: Option<GDifData>,
    /// Devolucao de tributos (`gDevTrib`). Optional.
    pub g_dev_trib: Option<GDevTribData>,
    /// Reducao de aliquota (`gRed`). Optional.
    pub g_red: Option<GRedData>,
    /// Valor do IBS do Municipio (`vIBSMun`), e.g. `"50.00"`.
    pub v_ibs_mun: String,
}

impl GIbsMunData {
    /// Create a new `GIbsMunData` with required fields.
    pub fn new(p_ibs_mun: impl Into<String>, v_ibs_mun: impl Into<String>) -> Self {
        Self {
            p_ibs_mun: p_ibs_mun.into(),
            v_ibs_mun: v_ibs_mun.into(),
            ..Default::default()
        }
    }
    /// Set diferimento.
    pub fn g_dif(mut self, v: GDifData) -> Self {
        self.g_dif = Some(v);
        self
    }
    /// Set devolucao de tributos.
    pub fn g_dev_trib(mut self, v: GDevTribData) -> Self {
        self.g_dev_trib = Some(v);
        self
    }
    /// Set reducao de aliquota.
    pub fn g_red(mut self, v: GRedData) -> Self {
        self.g_red = Some(v);
        self
    }
}

/// CBS (Contribuicao sobre Bens e Servicos): `<gCBS>` inside `<gIBSCBS>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GCbsData {
    /// Aliquota da CBS (`pCBS`), e.g. `"9.0000"`.
    pub p_cbs: String,
    /// Diferimento (`gDif`). Optional.
    pub g_dif: Option<GDifData>,
    /// Devolucao de tributos (`gDevTrib`). Optional.
    pub g_dev_trib: Option<GDevTribData>,
    /// Reducao de aliquota (`gRed`). Optional.
    pub g_red: Option<GRedData>,
    /// Valor da CBS (`vCBS`), e.g. `"90.00"`.
    pub v_cbs: String,
}

impl GCbsData {
    /// Create a new `GCbsData` with required fields.
    pub fn new(p_cbs: impl Into<String>, v_cbs: impl Into<String>) -> Self {
        Self {
            p_cbs: p_cbs.into(),
            v_cbs: v_cbs.into(),
            ..Default::default()
        }
    }
    /// Set diferimento.
    pub fn g_dif(mut self, v: GDifData) -> Self {
        self.g_dif = Some(v);
        self
    }
    /// Set devolucao de tributos.
    pub fn g_dev_trib(mut self, v: GDevTribData) -> Self {
        self.g_dev_trib = Some(v);
        self
    }
    /// Set reducao de aliquota.
    pub fn g_red(mut self, v: GRedData) -> Self {
        self.g_red = Some(v);
        self
    }
}

// ── gIBSCBS (tributacao ad-valorem) ──────────────────────────────────────

/// Grupo de informacoes do IBS e CBS ad-valorem: `<gIBSCBS>` inside `<IBSCBS>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GIbsCbsData {
    /// Base de calculo (`vBC`), e.g. `"1000.00"`.
    pub v_bc: String,
    /// IBS UF sub-group.
    pub g_ibs_uf: GIbsUfData,
    /// IBS Municipal sub-group.
    pub g_ibs_mun: GIbsMunData,
    /// Valor total do IBS (`vIBS`). Optional override; if absent, calculated as vIBSUF + vIBSMun.
    pub v_ibs: Option<String>,
    /// CBS sub-group.
    pub g_cbs: GCbsData,
}

impl GIbsCbsData {
    /// Create a new `GIbsCbsData`.
    pub fn new(
        v_bc: impl Into<String>,
        g_ibs_uf: GIbsUfData,
        g_ibs_mun: GIbsMunData,
        g_cbs: GCbsData,
    ) -> Self {
        Self {
            v_bc: v_bc.into(),
            g_ibs_uf,
            g_ibs_mun,
            v_ibs: None,
            g_cbs,
        }
    }
    /// Override the total IBS value.
    pub fn v_ibs(mut self, v: impl Into<String>) -> Self {
        self.v_ibs = Some(v.into());
        self
    }
}

// ── Tributacao Regular ───────────────────────────────────────────────────

/// Grupo de informacoes da tributacao regular: `<gTribRegular>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GTribRegularData {
    /// CST da tributacao regular (`CSTReg`).
    pub cst_reg: String,
    /// Codigo de classificacao tributaria regular (`cClassTribReg`).
    pub c_class_trib_reg: String,
    /// Aliquota efetiva IBS UF (`pAliqEfetRegIBSUF`).
    pub p_aliq_efet_reg_ibs_uf: String,
    /// Valor IBS UF (`vTribRegIBSUF`).
    pub v_trib_reg_ibs_uf: String,
    /// Aliquota efetiva IBS Municipio (`pAliqEfetRegIBSMun`).
    pub p_aliq_efet_reg_ibs_mun: String,
    /// Valor IBS Municipio (`vTribRegIBSMun`).
    pub v_trib_reg_ibs_mun: String,
    /// Aliquota efetiva CBS (`pAliqEfetRegCBS`).
    pub p_aliq_efet_reg_cbs: String,
    /// Valor CBS (`vTribRegCBS`).
    pub v_trib_reg_cbs: String,
}

impl GTribRegularData {
    /// Create a new `GTribRegularData`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cst_reg: impl Into<String>,
        c_class_trib_reg: impl Into<String>,
        p_aliq_efet_reg_ibs_uf: impl Into<String>,
        v_trib_reg_ibs_uf: impl Into<String>,
        p_aliq_efet_reg_ibs_mun: impl Into<String>,
        v_trib_reg_ibs_mun: impl Into<String>,
        p_aliq_efet_reg_cbs: impl Into<String>,
        v_trib_reg_cbs: impl Into<String>,
    ) -> Self {
        Self {
            cst_reg: cst_reg.into(),
            c_class_trib_reg: c_class_trib_reg.into(),
            p_aliq_efet_reg_ibs_uf: p_aliq_efet_reg_ibs_uf.into(),
            v_trib_reg_ibs_uf: v_trib_reg_ibs_uf.into(),
            p_aliq_efet_reg_ibs_mun: p_aliq_efet_reg_ibs_mun.into(),
            v_trib_reg_ibs_mun: v_trib_reg_ibs_mun.into(),
            p_aliq_efet_reg_cbs: p_aliq_efet_reg_cbs.into(),
            v_trib_reg_cbs: v_trib_reg_cbs.into(),
        }
    }
}

// ── Tributacao Compra Governamental ──────────────────────────────────────

/// Grupo de tributacao em compras governamentais: `<gTribCompraGov>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GTribCompraGovData {
    /// Aliquota IBS UF (`pAliqIBSUF`).
    pub p_aliq_ibs_uf: String,
    /// Valor IBS UF (`vTribIBSUF`).
    pub v_trib_ibs_uf: String,
    /// Aliquota IBS Municipio (`pAliqIBSMun`).
    pub p_aliq_ibs_mun: String,
    /// Valor IBS Municipio (`vTribIBSMun`).
    pub v_trib_ibs_mun: String,
    /// Aliquota CBS (`pAliqCBS`).
    pub p_aliq_cbs: String,
    /// Valor CBS (`vTribCBS`).
    pub v_trib_cbs: String,
}

impl GTribCompraGovData {
    /// Create a new `GTribCompraGovData`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        p_aliq_ibs_uf: impl Into<String>,
        v_trib_ibs_uf: impl Into<String>,
        p_aliq_ibs_mun: impl Into<String>,
        v_trib_ibs_mun: impl Into<String>,
        p_aliq_cbs: impl Into<String>,
        v_trib_cbs: impl Into<String>,
    ) -> Self {
        Self {
            p_aliq_ibs_uf: p_aliq_ibs_uf.into(),
            v_trib_ibs_uf: v_trib_ibs_uf.into(),
            p_aliq_ibs_mun: p_aliq_ibs_mun.into(),
            v_trib_ibs_mun: v_trib_ibs_mun.into(),
            p_aliq_cbs: p_aliq_cbs.into(),
            v_trib_cbs: v_trib_cbs.into(),
        }
    }
}

// ── Monofasico ──────────────────────────────────────────────────────────

/// Monofasico padrao sub-group: `<gMonoPadrao>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GMonoPadraoData {
    /// Quantidade tributada (`qBCMono`).
    pub q_bc_mono: String,
    /// Aliquota ad rem IBS (`adRemIBS`).
    pub ad_rem_ibs: String,
    /// Aliquota ad rem CBS (`adRemCBS`).
    pub ad_rem_cbs: String,
    /// Valor IBS monofasico (`vIBSMono`).
    pub v_ibs_mono: String,
    /// Valor CBS monofasico (`vCBSMono`).
    pub v_cbs_mono: String,
}

impl GMonoPadraoData {
    /// Create a new `GMonoPadraoData`.
    pub fn new(
        q_bc_mono: impl Into<String>,
        ad_rem_ibs: impl Into<String>,
        ad_rem_cbs: impl Into<String>,
        v_ibs_mono: impl Into<String>,
        v_cbs_mono: impl Into<String>,
    ) -> Self {
        Self {
            q_bc_mono: q_bc_mono.into(),
            ad_rem_ibs: ad_rem_ibs.into(),
            ad_rem_cbs: ad_rem_cbs.into(),
            v_ibs_mono: v_ibs_mono.into(),
            v_cbs_mono: v_cbs_mono.into(),
        }
    }
}

/// Monofasico retencao sub-group: `<gMonoReten>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GMonoRetenData {
    pub q_bc_mono_reten: String,
    pub ad_rem_ibs_reten: String,
    pub v_ibs_mono_reten: String,
    pub ad_rem_cbs_reten: String,
    pub v_cbs_mono_reten: String,
}

impl GMonoRetenData {
    /// Create a new `GMonoRetenData`.
    pub fn new(
        q_bc_mono_reten: impl Into<String>,
        ad_rem_ibs_reten: impl Into<String>,
        v_ibs_mono_reten: impl Into<String>,
        ad_rem_cbs_reten: impl Into<String>,
        v_cbs_mono_reten: impl Into<String>,
    ) -> Self {
        Self {
            q_bc_mono_reten: q_bc_mono_reten.into(),
            ad_rem_ibs_reten: ad_rem_ibs_reten.into(),
            v_ibs_mono_reten: v_ibs_mono_reten.into(),
            ad_rem_cbs_reten: ad_rem_cbs_reten.into(),
            v_cbs_mono_reten: v_cbs_mono_reten.into(),
        }
    }
}

/// Monofasico retido anteriormente: `<gMonoRet>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GMonoRetData {
    pub q_bc_mono_ret: String,
    pub ad_rem_ibs_ret: String,
    pub v_ibs_mono_ret: String,
    pub ad_rem_cbs_ret: String,
    pub v_cbs_mono_ret: String,
}

impl GMonoRetData {
    /// Create a new `GMonoRetData`.
    pub fn new(
        q_bc_mono_ret: impl Into<String>,
        ad_rem_ibs_ret: impl Into<String>,
        v_ibs_mono_ret: impl Into<String>,
        ad_rem_cbs_ret: impl Into<String>,
        v_cbs_mono_ret: impl Into<String>,
    ) -> Self {
        Self {
            q_bc_mono_ret: q_bc_mono_ret.into(),
            ad_rem_ibs_ret: ad_rem_ibs_ret.into(),
            v_ibs_mono_ret: v_ibs_mono_ret.into(),
            ad_rem_cbs_ret: ad_rem_cbs_ret.into(),
            v_cbs_mono_ret: v_cbs_mono_ret.into(),
        }
    }
}

/// Monofasico diferimento: `<gMonoDif>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GMonoDifData {
    pub p_dif_ibs: String,
    pub v_ibs_mono_dif: String,
    pub p_dif_cbs: String,
    pub v_cbs_mono_dif: String,
}

impl GMonoDifData {
    /// Create a new `GMonoDifData`.
    pub fn new(
        p_dif_ibs: impl Into<String>,
        v_ibs_mono_dif: impl Into<String>,
        p_dif_cbs: impl Into<String>,
        v_cbs_mono_dif: impl Into<String>,
    ) -> Self {
        Self {
            p_dif_ibs: p_dif_ibs.into(),
            v_ibs_mono_dif: v_ibs_mono_dif.into(),
            p_dif_cbs: p_dif_cbs.into(),
            v_cbs_mono_dif: v_cbs_mono_dif.into(),
        }
    }
}

/// Grupo monofasico completo: `<gIBSCBSMono>` inside `<IBSCBS>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GIbsCbsMonoData {
    /// Monofasico padrao. Optional.
    pub g_mono_padrao: Option<GMonoPadraoData>,
    /// Monofasico retencao. Optional.
    pub g_mono_reten: Option<GMonoRetenData>,
    /// Monofasico retido anteriormente. Optional.
    pub g_mono_ret: Option<GMonoRetData>,
    /// Monofasico diferimento. Optional.
    pub g_mono_dif: Option<GMonoDifData>,
    /// Total IBS monofasico do item (`vTotIBSMonoItem`).
    pub v_tot_ibs_mono_item: String,
    /// Total CBS monofasica do item (`vTotCBSMonoItem`).
    pub v_tot_cbs_mono_item: String,
}

impl GIbsCbsMonoData {
    /// Create a new `GIbsCbsMonoData` with totals.
    pub fn new(
        v_tot_ibs_mono_item: impl Into<String>,
        v_tot_cbs_mono_item: impl Into<String>,
    ) -> Self {
        Self {
            v_tot_ibs_mono_item: v_tot_ibs_mono_item.into(),
            v_tot_cbs_mono_item: v_tot_cbs_mono_item.into(),
            ..Default::default()
        }
    }
    /// Set monofasico padrao.
    pub fn g_mono_padrao(mut self, v: GMonoPadraoData) -> Self {
        self.g_mono_padrao = Some(v);
        self
    }
    /// Set monofasico retencao.
    pub fn g_mono_reten(mut self, v: GMonoRetenData) -> Self {
        self.g_mono_reten = Some(v);
        self
    }
    /// Set monofasico retido anteriormente.
    pub fn g_mono_ret(mut self, v: GMonoRetData) -> Self {
        self.g_mono_ret = Some(v);
        self
    }
    /// Set monofasico diferimento.
    pub fn g_mono_dif(mut self, v: GMonoDifData) -> Self {
        self.g_mono_dif = Some(v);
        self
    }
}

// ── Transferencia de Credito ────────────────────────────────────────────

/// Grupo de transferencia de creditos: `<gTransfCred>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GTransfCredData {
    /// Valor do IBS a ser transferido (`vIBS`).
    pub v_ibs: String,
    /// Valor da CBS a ser transferida (`vCBS`).
    pub v_cbs: String,
}

impl GTransfCredData {
    /// Create a new `GTransfCredData`.
    pub fn new(v_ibs: impl Into<String>, v_cbs: impl Into<String>) -> Self {
        Self {
            v_ibs: v_ibs.into(),
            v_cbs: v_cbs.into(),
        }
    }
}

// ── Credito Presumido ZFM ───────────────────────────────────────────────

/// Grupo de credito presumido IBS com ZFM: `<gCredPresIBSZFM>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GCredPresIbsZfmData {
    /// Competencia de apuracao (`competApur`), e.g. `"2025-06"`. Optional.
    pub compet_apur: Option<String>,
    /// Tipo de credito presumido ZFM (`tpCredPresIBSZFM`).
    pub tp_cred_pres_ibs_zfm: String,
    /// Valor do credito presumido ZFM (`vCredPresIBSZFM`).
    pub v_cred_pres_ibs_zfm: String,
}

impl GCredPresIbsZfmData {
    /// Create a new `GCredPresIbsZfmData`.
    pub fn new(
        tp_cred_pres_ibs_zfm: impl Into<String>,
        v_cred_pres_ibs_zfm: impl Into<String>,
    ) -> Self {
        Self {
            tp_cred_pres_ibs_zfm: tp_cred_pres_ibs_zfm.into(),
            v_cred_pres_ibs_zfm: v_cred_pres_ibs_zfm.into(),
            compet_apur: None,
        }
    }
    /// Set competencia de apuracao.
    pub fn compet_apur(mut self, v: impl Into<String>) -> Self {
        self.compet_apur = Some(v.into());
        self
    }
}

// ── Ajuste de Competencia ───────────────────────────────────────────────

/// Grupo de ajuste de competencia: `<gAjusteCompet>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GAjusteCompetData {
    /// Competencia de apuracao (`competApur`), e.g. `"2025-06"`.
    pub compet_apur: String,
    /// Valor do IBS (`vIBS`).
    pub v_ibs: String,
    /// Valor da CBS (`vCBS`).
    pub v_cbs: String,
}

impl GAjusteCompetData {
    /// Create a new `GAjusteCompetData`.
    pub fn new(
        compet_apur: impl Into<String>,
        v_ibs: impl Into<String>,
        v_cbs: impl Into<String>,
    ) -> Self {
        Self {
            compet_apur: compet_apur.into(),
            v_ibs: v_ibs.into(),
            v_cbs: v_cbs.into(),
        }
    }
}

// ── Estorno de Credito ──────────────────────────────────────────────────

/// Grupo de estorno de credito: `<gEstornoCred>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GEstornoCredData {
    /// Valor do IBS estornado (`vIBSEstCred`).
    pub v_ibs_est_cred: String,
    /// Valor da CBS estornada (`vCBSEstCred`).
    pub v_cbs_est_cred: String,
}

impl GEstornoCredData {
    /// Create a new `GEstornoCredData`.
    pub fn new(v_ibs_est_cred: impl Into<String>, v_cbs_est_cred: impl Into<String>) -> Self {
        Self {
            v_ibs_est_cred: v_ibs_est_cred.into(),
            v_cbs_est_cred: v_cbs_est_cred.into(),
        }
    }
}

// ── Credito Presumido por Operacao ──────────────────────────────────────

/// IBS credito presumido sub-group: `<gIBSCredPres>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GIbsCredPresData {
    /// Percentual do credito presumido (`pCredPres`).
    pub p_cred_pres: String,
    /// Valor do credito presumido (`vCredPres`). Optional -- choice with `vCredPresCondSus`.
    pub v_cred_pres: Option<String>,
    /// Valor do credito presumido em condicao suspensiva (`vCredPresCondSus`). Optional.
    pub v_cred_pres_cond_sus: Option<String>,
}

impl GIbsCredPresData {
    /// Create with vCredPres.
    pub fn with_cred_pres(p_cred_pres: impl Into<String>, v_cred_pres: impl Into<String>) -> Self {
        Self {
            p_cred_pres: p_cred_pres.into(),
            v_cred_pres: Some(v_cred_pres.into()),
            v_cred_pres_cond_sus: None,
        }
    }
    /// Create with vCredPresCondSus.
    pub fn with_cred_pres_cond_sus(
        p_cred_pres: impl Into<String>,
        v_cred_pres_cond_sus: impl Into<String>,
    ) -> Self {
        Self {
            p_cred_pres: p_cred_pres.into(),
            v_cred_pres: None,
            v_cred_pres_cond_sus: Some(v_cred_pres_cond_sus.into()),
        }
    }
}

/// CBS credito presumido sub-group: `<gCBSCredPres>`.
pub type GCbsCredPresData = GIbsCredPresData;

/// Grupo de credito presumido por operacao: `<gCredPresOper>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GCredPresOperData {
    /// Base de calculo do credito presumido (`vBCCredPres`).
    pub v_bc_cred_pres: String,
    /// Codigo de classificacao do credito presumido (`cCredPres`).
    pub c_cred_pres: String,
    /// IBS credito presumido. Optional.
    pub g_ibs_cred_pres: Option<GIbsCredPresData>,
    /// CBS credito presumido. Optional.
    pub g_cbs_cred_pres: Option<GCbsCredPresData>,
}

impl GCredPresOperData {
    /// Create a new `GCredPresOperData`.
    pub fn new(v_bc_cred_pres: impl Into<String>, c_cred_pres: impl Into<String>) -> Self {
        Self {
            v_bc_cred_pres: v_bc_cred_pres.into(),
            c_cred_pres: c_cred_pres.into(),
            g_ibs_cred_pres: None,
            g_cbs_cred_pres: None,
        }
    }
    /// Set IBS credito presumido.
    pub fn g_ibs_cred_pres(mut self, v: GIbsCredPresData) -> Self {
        self.g_ibs_cred_pres = Some(v);
        self
    }
    /// Set CBS credito presumido.
    pub fn g_cbs_cred_pres(mut self, v: GCbsCredPresData) -> Self {
        self.g_cbs_cred_pres = Some(v);
        self
    }
}

// ── Main IBS/CBS data ───────────────────────────────────────────────────

/// Complete IBS/CBS data for a single invoice item: `<IBSCBS>` inside `<imposto>`.
///
/// Follows the PHP `tagIBSCBS` + all appended sub-groups.
/// `gIBSCBS` and `gIBSCBSMono` are mutually exclusive (choice).
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct IbsCbsData {
    /// CST code (`CST`).
    pub cst: String,
    /// Codigo de classificacao tributaria (`cClassTrib`).
    pub c_class_trib: String,
    /// Indicador de doacao (`indDoacao`). When `true`, emits `<indDoacao>1</indDoacao>`.
    pub ind_doacao: bool,
    /// Tributacao ad-valorem (`gIBSCBS`). Optional -- choice with `g_ibs_cbs_mono`.
    pub g_ibs_cbs: Option<GIbsCbsData>,
    /// Tributacao regular (`gTribRegular`). Optional.
    pub g_trib_regular: Option<GTribRegularData>,
    /// Tributacao compra governamental (`gTribCompraGov`). Optional.
    pub g_trib_compra_gov: Option<GTribCompraGovData>,
    /// Monofasico (`gIBSCBSMono`). Optional -- choice with `g_ibs_cbs`.
    pub g_ibs_cbs_mono: Option<GIbsCbsMonoData>,
    /// Transferencia de credito (`gTransfCred`). Optional.
    pub g_transf_cred: Option<GTransfCredData>,
    /// Credito presumido ZFM (`gCredPresIBSZFM`). Optional.
    pub g_cred_pres_ibs_zfm: Option<GCredPresIbsZfmData>,
    /// Ajuste de competencia (`gAjusteCompet`). Optional.
    pub g_ajuste_compet: Option<GAjusteCompetData>,
    /// Estorno de credito (`gEstornoCred`). Optional.
    pub g_estorno_cred: Option<GEstornoCredData>,
    /// Credito presumido por operacao (`gCredPresOper`). Optional.
    pub g_cred_pres_oper: Option<GCredPresOperData>,
}

impl IbsCbsData {
    /// Create a new `IbsCbsData` with required CST and classification code.
    pub fn new(cst: impl Into<String>, c_class_trib: impl Into<String>) -> Self {
        Self {
            cst: cst.into(),
            c_class_trib: c_class_trib.into(),
            ..Default::default()
        }
    }
    /// Set indicador de doacao.
    pub fn ind_doacao(mut self, v: bool) -> Self {
        self.ind_doacao = v;
        self
    }
    /// Set tributacao ad-valorem.
    pub fn g_ibs_cbs(mut self, v: GIbsCbsData) -> Self {
        self.g_ibs_cbs = Some(v);
        self
    }
    /// Set tributacao regular.
    pub fn g_trib_regular(mut self, v: GTribRegularData) -> Self {
        self.g_trib_regular = Some(v);
        self
    }
    /// Set tributacao compra governamental.
    pub fn g_trib_compra_gov(mut self, v: GTribCompraGovData) -> Self {
        self.g_trib_compra_gov = Some(v);
        self
    }
    /// Set monofasico.
    pub fn g_ibs_cbs_mono(mut self, v: GIbsCbsMonoData) -> Self {
        self.g_ibs_cbs_mono = Some(v);
        self
    }
    /// Set transferencia de credito.
    pub fn g_transf_cred(mut self, v: GTransfCredData) -> Self {
        self.g_transf_cred = Some(v);
        self
    }
    /// Set credito presumido ZFM.
    pub fn g_cred_pres_ibs_zfm(mut self, v: GCredPresIbsZfmData) -> Self {
        self.g_cred_pres_ibs_zfm = Some(v);
        self
    }
    /// Set ajuste de competencia.
    pub fn g_ajuste_compet(mut self, v: GAjusteCompetData) -> Self {
        self.g_ajuste_compet = Some(v);
        self
    }
    /// Set estorno de credito.
    pub fn g_estorno_cred(mut self, v: GEstornoCredData) -> Self {
        self.g_estorno_cred = Some(v);
        self
    }
    /// Set credito presumido por operacao.
    pub fn g_cred_pres_oper(mut self, v: GCredPresOperData) -> Self {
        self.g_cred_pres_oper = Some(v);
        self
    }
}

// ── XML builders ────────────────────────────────────────────────────────

fn build_g_dif(prefix: &str, data: &GDifData) -> String {
    let _ = prefix;
    let mut c = vec![
        tag("pDif", &[], TagContent::Text(&data.p_dif)),
        tag("vDif", &[], TagContent::Text(&data.v_dif)),
    ];
    let _ = &mut c;
    tag("gDif", &[], TagContent::Children(c))
}

fn build_g_dev_trib(data: &GDevTribData) -> String {
    tag(
        "gDevTrib",
        &[],
        TagContent::Children(vec![tag(
            "vDevTrib",
            &[],
            TagContent::Text(&data.v_dev_trib),
        )]),
    )
}

fn build_g_red(data: &GRedData) -> String {
    tag(
        "gRed",
        &[],
        TagContent::Children(vec![
            tag("pRedAliq", &[], TagContent::Text(&data.p_red_aliq)),
            tag("pAliqEfet", &[], TagContent::Text(&data.p_aliq_efet)),
        ]),
    )
}

fn build_g_ibs_uf(data: &GIbsUfData) -> String {
    let mut children = vec![tag("pIBSUF", &[], TagContent::Text(&data.p_ibs_uf))];
    if let Some(ref dif) = data.g_dif {
        children.push(build_g_dif("IBSUF", dif));
    }
    if let Some(ref dev) = data.g_dev_trib {
        children.push(build_g_dev_trib(dev));
    }
    if let Some(ref red) = data.g_red {
        children.push(build_g_red(red));
    }
    children.push(tag("vIBSUF", &[], TagContent::Text(&data.v_ibs_uf)));
    tag("gIBSUF", &[], TagContent::Children(children))
}

fn build_g_ibs_mun(data: &GIbsMunData) -> String {
    let mut children = vec![tag("pIBSMun", &[], TagContent::Text(&data.p_ibs_mun))];
    if let Some(ref dif) = data.g_dif {
        children.push(build_g_dif("IBSMun", dif));
    }
    if let Some(ref dev) = data.g_dev_trib {
        children.push(build_g_dev_trib(dev));
    }
    if let Some(ref red) = data.g_red {
        children.push(build_g_red(red));
    }
    children.push(tag("vIBSMun", &[], TagContent::Text(&data.v_ibs_mun)));
    tag("gIBSMun", &[], TagContent::Children(children))
}

fn build_g_cbs(data: &GCbsData) -> String {
    let mut children = vec![tag("pCBS", &[], TagContent::Text(&data.p_cbs))];
    if let Some(ref dif) = data.g_dif {
        children.push(build_g_dif("CBS", dif));
    }
    if let Some(ref dev) = data.g_dev_trib {
        children.push(build_g_dev_trib(dev));
    }
    if let Some(ref red) = data.g_red {
        children.push(build_g_red(red));
    }
    children.push(tag("vCBS", &[], TagContent::Text(&data.v_cbs)));
    tag("gCBS", &[], TagContent::Children(children))
}

fn build_g_ibs_cbs(data: &GIbsCbsData) -> String {
    let mut children = vec![tag("vBC", &[], TagContent::Text(&data.v_bc))];
    children.push(build_g_ibs_uf(&data.g_ibs_uf));
    children.push(build_g_ibs_mun(&data.g_ibs_mun));
    // vIBS = provided or calculated placeholder
    let v_ibs_val = data.v_ibs.as_deref().unwrap_or("0.00");
    children.push(tag("vIBS", &[], TagContent::Text(v_ibs_val)));
    children.push(build_g_cbs(&data.g_cbs));
    tag("gIBSCBS", &[], TagContent::Children(children))
}

fn build_g_trib_regular(data: &GTribRegularData) -> String {
    tag(
        "gTribRegular",
        &[],
        TagContent::Children(vec![
            tag("CSTReg", &[], TagContent::Text(&data.cst_reg)),
            tag(
                "cClassTribReg",
                &[],
                TagContent::Text(&data.c_class_trib_reg),
            ),
            tag(
                "pAliqEfetRegIBSUF",
                &[],
                TagContent::Text(&data.p_aliq_efet_reg_ibs_uf),
            ),
            tag(
                "vTribRegIBSUF",
                &[],
                TagContent::Text(&data.v_trib_reg_ibs_uf),
            ),
            tag(
                "pAliqEfetRegIBSMun",
                &[],
                TagContent::Text(&data.p_aliq_efet_reg_ibs_mun),
            ),
            tag(
                "vTribRegIBSMun",
                &[],
                TagContent::Text(&data.v_trib_reg_ibs_mun),
            ),
            tag(
                "pAliqEfetRegCBS",
                &[],
                TagContent::Text(&data.p_aliq_efet_reg_cbs),
            ),
            tag("vTribRegCBS", &[], TagContent::Text(&data.v_trib_reg_cbs)),
        ]),
    )
}

fn build_g_trib_compra_gov(data: &GTribCompraGovData) -> String {
    tag(
        "gTribCompraGov",
        &[],
        TagContent::Children(vec![
            tag("pAliqIBSUF", &[], TagContent::Text(&data.p_aliq_ibs_uf)),
            tag("vTribIBSUF", &[], TagContent::Text(&data.v_trib_ibs_uf)),
            tag("pAliqIBSMun", &[], TagContent::Text(&data.p_aliq_ibs_mun)),
            tag("vTribIBSMun", &[], TagContent::Text(&data.v_trib_ibs_mun)),
            tag("pAliqCBS", &[], TagContent::Text(&data.p_aliq_cbs)),
            tag("vTribCBS", &[], TagContent::Text(&data.v_trib_cbs)),
        ]),
    )
}

fn build_g_ibs_cbs_mono(data: &GIbsCbsMonoData) -> String {
    let mut children: Vec<String> = Vec::new();
    if let Some(ref p) = data.g_mono_padrao {
        children.push(tag(
            "gMonoPadrao",
            &[],
            TagContent::Children(vec![
                tag("qBCMono", &[], TagContent::Text(&p.q_bc_mono)),
                tag("adRemIBS", &[], TagContent::Text(&p.ad_rem_ibs)),
                tag("adRemCBS", &[], TagContent::Text(&p.ad_rem_cbs)),
                tag("vIBSMono", &[], TagContent::Text(&p.v_ibs_mono)),
                tag("vCBSMono", &[], TagContent::Text(&p.v_cbs_mono)),
            ]),
        ));
    }
    if let Some(ref r) = data.g_mono_reten {
        children.push(tag(
            "gMonoReten",
            &[],
            TagContent::Children(vec![
                tag("qBCMonoReten", &[], TagContent::Text(&r.q_bc_mono_reten)),
                tag("adRemIBSReten", &[], TagContent::Text(&r.ad_rem_ibs_reten)),
                tag("vIBSMonoReten", &[], TagContent::Text(&r.v_ibs_mono_reten)),
                tag("adRemCBSReten", &[], TagContent::Text(&r.ad_rem_cbs_reten)),
                tag("vCBSMonoReten", &[], TagContent::Text(&r.v_cbs_mono_reten)),
            ]),
        ));
    }
    if let Some(ref r) = data.g_mono_ret {
        children.push(tag(
            "gMonoRet",
            &[],
            TagContent::Children(vec![
                tag("qBCMonoRet", &[], TagContent::Text(&r.q_bc_mono_ret)),
                tag("adRemIBSRet", &[], TagContent::Text(&r.ad_rem_ibs_ret)),
                tag("vIBSMonoRet", &[], TagContent::Text(&r.v_ibs_mono_ret)),
                tag("adRemCBSRet", &[], TagContent::Text(&r.ad_rem_cbs_ret)),
                tag("vCBSMonoRet", &[], TagContent::Text(&r.v_cbs_mono_ret)),
            ]),
        ));
    }
    if let Some(ref d) = data.g_mono_dif {
        children.push(tag(
            "gMonoDif",
            &[],
            TagContent::Children(vec![
                tag("pDifIBS", &[], TagContent::Text(&d.p_dif_ibs)),
                tag("vIBSMonoDif", &[], TagContent::Text(&d.v_ibs_mono_dif)),
                tag("pDifCBS", &[], TagContent::Text(&d.p_dif_cbs)),
                tag("vCBSMonoDif", &[], TagContent::Text(&d.v_cbs_mono_dif)),
            ]),
        ));
    }
    children.push(tag(
        "vTotIBSMonoItem",
        &[],
        TagContent::Text(&data.v_tot_ibs_mono_item),
    ));
    children.push(tag(
        "vTotCBSMonoItem",
        &[],
        TagContent::Text(&data.v_tot_cbs_mono_item),
    ));
    tag("gIBSCBSMono", &[], TagContent::Children(children))
}

fn build_g_transf_cred(data: &GTransfCredData) -> String {
    tag(
        "gTransfCred",
        &[],
        TagContent::Children(vec![
            tag("vIBS", &[], TagContent::Text(&data.v_ibs)),
            tag("vCBS", &[], TagContent::Text(&data.v_cbs)),
        ]),
    )
}

fn build_g_cred_pres_ibs_zfm(data: &GCredPresIbsZfmData) -> String {
    let mut children: Vec<String> = Vec::new();
    if let Some(ref ca) = data.compet_apur {
        children.push(tag("competApur", &[], TagContent::Text(ca)));
    }
    children.push(tag(
        "tpCredPresIBSZFM",
        &[],
        TagContent::Text(&data.tp_cred_pres_ibs_zfm),
    ));
    children.push(tag(
        "vCredPresIBSZFM",
        &[],
        TagContent::Text(&data.v_cred_pres_ibs_zfm),
    ));
    tag("gCredPresIBSZFM", &[], TagContent::Children(children))
}

fn build_g_ajuste_compet(data: &GAjusteCompetData) -> String {
    tag(
        "gAjusteCompet",
        &[],
        TagContent::Children(vec![
            tag("competApur", &[], TagContent::Text(&data.compet_apur)),
            tag("vIBS", &[], TagContent::Text(&data.v_ibs)),
            tag("vCBS", &[], TagContent::Text(&data.v_cbs)),
        ]),
    )
}

fn build_g_estorno_cred(data: &GEstornoCredData) -> String {
    tag(
        "gEstornoCred",
        &[],
        TagContent::Children(vec![
            tag("vIBSEstCred", &[], TagContent::Text(&data.v_ibs_est_cred)),
            tag("vCBSEstCred", &[], TagContent::Text(&data.v_cbs_est_cred)),
        ]),
    )
}

fn build_cred_pres_sub(tag_name: &str, data: &GIbsCredPresData) -> String {
    let mut children = vec![tag("pCredPres", &[], TagContent::Text(&data.p_cred_pres))];
    if let Some(ref v) = data.v_cred_pres {
        children.push(tag("vCredPres", &[], TagContent::Text(v)));
    } else if let Some(ref v) = data.v_cred_pres_cond_sus {
        children.push(tag("vCredPresCondSus", &[], TagContent::Text(v)));
    }
    tag(tag_name, &[], TagContent::Children(children))
}

fn build_g_cred_pres_oper(data: &GCredPresOperData) -> String {
    let mut children = vec![
        tag("vBCCredPres", &[], TagContent::Text(&data.v_bc_cred_pres)),
        tag("cCredPres", &[], TagContent::Text(&data.c_cred_pres)),
    ];
    if let Some(ref ibs) = data.g_ibs_cred_pres {
        children.push(build_cred_pres_sub("gIBSCredPres", ibs));
    }
    if let Some(ref cbs) = data.g_cbs_cred_pres {
        children.push(build_cred_pres_sub("gCBSCredPres", cbs));
    }
    tag("gCredPresOper", &[], TagContent::Children(children))
}

/// Build the complete `<IBSCBS>` XML string for an invoice item.
pub fn build_ibs_cbs_xml(data: &IbsCbsData) -> String {
    let mut children = vec![
        tag("CST", &[], TagContent::Text(&data.cst)),
        tag("cClassTrib", &[], TagContent::Text(&data.c_class_trib)),
    ];
    if data.ind_doacao {
        children.push(tag("indDoacao", &[], TagContent::Text("1")));
    }
    // gIBSCBS (ad-valorem) -- choice with gIBSCBSMono
    if let Some(ref g) = data.g_ibs_cbs {
        children.push(build_g_ibs_cbs(g));
    }
    // gTribRegular -- appended inside IBSCBS after gIBSCBS
    if let Some(ref g) = data.g_trib_regular {
        children.push(build_g_trib_regular(g));
    }
    // gTribCompraGov
    if let Some(ref g) = data.g_trib_compra_gov {
        children.push(build_g_trib_compra_gov(g));
    }
    // gIBSCBSMono -- choice with gIBSCBS
    if let Some(ref g) = data.g_ibs_cbs_mono {
        children.push(build_g_ibs_cbs_mono(g));
    }
    // gTransfCred
    if let Some(ref g) = data.g_transf_cred {
        children.push(build_g_transf_cred(g));
    }
    // gCredPresIBSZFM
    if let Some(ref g) = data.g_cred_pres_ibs_zfm {
        children.push(build_g_cred_pres_ibs_zfm(g));
    }
    // gAjusteCompet
    if let Some(ref g) = data.g_ajuste_compet {
        children.push(build_g_ajuste_compet(g));
    }
    // gEstornoCred
    if let Some(ref g) = data.g_estorno_cred {
        children.push(build_g_estorno_cred(g));
    }
    // gCredPresOper
    if let Some(ref g) = data.g_cred_pres_oper {
        children.push(build_g_cred_pres_oper(g));
    }
    tag("IBSCBS", &[], TagContent::Children(children))
}

// ── IBS/CBS Total data ──────────────────────────────────────────────────

/// Total do IBS/CBS: `<IBSCBSTot>` inside `<total>`.
///
/// Follows PHP `tagIBSCBSTot`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct IbsCbsTotData {
    /// Base de calculo total (`vBCIBSCBS`).
    pub v_bc_ibs_cbs: String,
    // gIBS
    /// Total IBS UF diferimento (`gIBSUF/vDif`).
    pub g_ibs_uf_v_dif: Option<String>,
    /// Total IBS UF devolucao (`gIBSUF/vDevTrib`).
    pub g_ibs_uf_v_dev_trib: Option<String>,
    /// Total IBS UF (`gIBSUF/vIBSUF`).
    pub g_ibs_uf_v_ibs_uf: Option<String>,
    /// Total IBS Mun diferimento (`gIBSMun/vDif`).
    pub g_ibs_mun_v_dif: Option<String>,
    /// Total IBS Mun devolucao (`gIBSMun/vDevTrib`).
    pub g_ibs_mun_v_dev_trib: Option<String>,
    /// Total IBS Mun (`gIBSMun/vIBSMun`).
    pub g_ibs_mun_v_ibs_mun: Option<String>,
    /// Total IBS (`gIBS/vIBS`).
    pub g_ibs_v_ibs: Option<String>,
    /// Total IBS credito presumido (`gIBS/vCredPres`).
    pub g_ibs_v_cred_pres: Option<String>,
    /// Total IBS credito presumido condicao suspensiva (`gIBS/vCredPresCondSus`).
    pub g_ibs_v_cred_pres_cond_sus: Option<String>,
    // gCBS
    /// Total CBS diferimento (`gCBS/vDif`).
    pub g_cbs_v_dif: Option<String>,
    /// Total CBS devolucao (`gCBS/vDevTrib`).
    pub g_cbs_v_dev_trib: Option<String>,
    /// Total CBS (`gCBS/vCBS`).
    pub g_cbs_v_cbs: Option<String>,
    /// Total CBS credito presumido (`gCBS/vCredPres`).
    pub g_cbs_v_cred_pres: Option<String>,
    /// Total CBS credito presumido condicao suspensiva (`gCBS/vCredPresCondSus`).
    pub g_cbs_v_cred_pres_cond_sus: Option<String>,
    // gMono
    /// Total IBS monofasico (`gMono/vIBSMono`).
    pub g_mono_v_ibs_mono: Option<String>,
    /// Total CBS monofasica (`gMono/vCBSMono`).
    pub g_mono_v_cbs_mono: Option<String>,
    /// Total IBS monofasico retencao (`gMono/vIBSMonoReten`).
    pub g_mono_v_ibs_mono_reten: Option<String>,
    /// Total CBS monofasica retencao (`gMono/vCBSMonoReten`).
    pub g_mono_v_cbs_mono_reten: Option<String>,
    /// Total IBS monofasico retido anteriormente (`gMono/vIBSMonoRet`).
    pub g_mono_v_ibs_mono_ret: Option<String>,
    /// Total CBS monofasica retida anteriormente (`gMono/vCBSMonoRet`).
    pub g_mono_v_cbs_mono_ret: Option<String>,
    // gEstornoCred
    /// Total IBS estornado (`gEstornoCred/vIBSEstCred`).
    pub g_estorno_cred_v_ibs_est_cred: Option<String>,
    /// Total CBS estornada (`gEstornoCred/vCBSEstCred`).
    pub g_estorno_cred_v_cbs_est_cred: Option<String>,
}

impl IbsCbsTotData {
    /// Create a new `IbsCbsTotData` with required BC.
    pub fn new(v_bc_ibs_cbs: impl Into<String>) -> Self {
        Self {
            v_bc_ibs_cbs: v_bc_ibs_cbs.into(),
            ..Default::default()
        }
    }
}

/// Total do IS (Imposto Seletivo): `<ISTot>` inside `<total>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct IsTotData {
    /// Valor total do IS (`vIS`).
    pub v_is: String,
}

impl IsTotData {
    /// Create a new `IsTotData`.
    pub fn new(v_is: impl Into<String>) -> Self {
        Self { v_is: v_is.into() }
    }
}

/// Build the `<ISTot>` XML element.
pub fn build_is_tot_xml(data: &IsTotData) -> String {
    tag(
        "ISTot",
        &[],
        TagContent::Children(vec![tag("vIS", &[], TagContent::Text(&data.v_is))]),
    )
}

/// Build the `<IBSCBSTot>` XML element.
pub fn build_ibs_cbs_tot_xml(data: &IbsCbsTotData) -> String {
    let mut children = vec![tag("vBCIBSCBS", &[], TagContent::Text(&data.v_bc_ibs_cbs))];

    // gIBS block
    if let Some(ref v_ibs) = data.g_ibs_v_ibs {
        let mut g_ibs_children: Vec<String> = Vec::new();
        // gIBSUF
        let g_ibs_uf_children: Vec<String> = vec![
            tag(
                "vDif",
                &[],
                TagContent::Text(data.g_ibs_uf_v_dif.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vDevTrib",
                &[],
                TagContent::Text(data.g_ibs_uf_v_dev_trib.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vIBSUF",
                &[],
                TagContent::Text(data.g_ibs_uf_v_ibs_uf.as_deref().unwrap_or("0.00")),
            ),
        ];
        g_ibs_children.push(tag("gIBSUF", &[], TagContent::Children(g_ibs_uf_children)));
        // gIBSMun
        let g_ibs_mun_children: Vec<String> = vec![
            tag(
                "vDif",
                &[],
                TagContent::Text(data.g_ibs_mun_v_dif.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vDevTrib",
                &[],
                TagContent::Text(data.g_ibs_mun_v_dev_trib.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vIBSMun",
                &[],
                TagContent::Text(data.g_ibs_mun_v_ibs_mun.as_deref().unwrap_or("0.00")),
            ),
        ];
        g_ibs_children.push(tag(
            "gIBSMun",
            &[],
            TagContent::Children(g_ibs_mun_children),
        ));
        g_ibs_children.push(tag("vIBS", &[], TagContent::Text(v_ibs)));
        g_ibs_children.push(tag(
            "vCredPres",
            &[],
            TagContent::Text(data.g_ibs_v_cred_pres.as_deref().unwrap_or("0.00")),
        ));
        g_ibs_children.push(tag(
            "vCredPresCondSus",
            &[],
            TagContent::Text(data.g_ibs_v_cred_pres_cond_sus.as_deref().unwrap_or("0.00")),
        ));
        children.push(tag("gIBS", &[], TagContent::Children(g_ibs_children)));
    }

    // gCBS block
    if let Some(ref v_cbs) = data.g_cbs_v_cbs {
        let g_cbs_children: Vec<String> = vec![
            tag(
                "vDif",
                &[],
                TagContent::Text(data.g_cbs_v_dif.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vDevTrib",
                &[],
                TagContent::Text(data.g_cbs_v_dev_trib.as_deref().unwrap_or("0.00")),
            ),
            tag("vCBS", &[], TagContent::Text(v_cbs)),
            tag(
                "vCredPres",
                &[],
                TagContent::Text(data.g_cbs_v_cred_pres.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vCredPresCondSus",
                &[],
                TagContent::Text(data.g_cbs_v_cred_pres_cond_sus.as_deref().unwrap_or("0.00")),
            ),
        ];
        children.push(tag("gCBS", &[], TagContent::Children(g_cbs_children)));
    }

    // gMono block
    if let Some(ref v) = data.g_mono_v_ibs_mono {
        let g_mono_children = vec![
            tag("vIBSMono", &[], TagContent::Text(v)),
            tag(
                "vCBSMono",
                &[],
                TagContent::Text(data.g_mono_v_cbs_mono.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vIBSMonoReten",
                &[],
                TagContent::Text(data.g_mono_v_ibs_mono_reten.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vCBSMonoReten",
                &[],
                TagContent::Text(data.g_mono_v_cbs_mono_reten.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vIBSMonoRet",
                &[],
                TagContent::Text(data.g_mono_v_ibs_mono_ret.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vCBSMonoRet",
                &[],
                TagContent::Text(data.g_mono_v_cbs_mono_ret.as_deref().unwrap_or("0.00")),
            ),
        ];
        children.push(tag("gMono", &[], TagContent::Children(g_mono_children)));
    }

    // gEstornoCred block
    let has_est_ibs = data
        .g_estorno_cred_v_ibs_est_cred
        .as_ref()
        .is_some_and(|v| !v.is_empty());
    let has_est_cbs = data
        .g_estorno_cred_v_cbs_est_cred
        .as_ref()
        .is_some_and(|v| !v.is_empty());
    if has_est_ibs || has_est_cbs {
        let g_est_children = vec![
            tag(
                "vIBSEstCred",
                &[],
                TagContent::Text(
                    data.g_estorno_cred_v_ibs_est_cred
                        .as_deref()
                        .unwrap_or("0.00"),
                ),
            ),
            tag(
                "vCBSEstCred",
                &[],
                TagContent::Text(
                    data.g_estorno_cred_v_cbs_est_cred
                        .as_deref()
                        .unwrap_or("0.00"),
                ),
            ),
        ];
        children.push(tag(
            "gEstornoCred",
            &[],
            TagContent::Children(g_est_children),
        ));
    }

    tag("IBSCBSTot", &[], TagContent::Children(children))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_ibs_cbs_xml() {
        let data = IbsCbsData::new("00", "12345678");
        let xml = build_ibs_cbs_xml(&data);
        assert!(xml.contains("<IBSCBS>"));
        assert!(xml.contains("<CST>00</CST>"));
        assert!(xml.contains("<cClassTrib>12345678</cClassTrib>"));
        assert!(xml.contains("</IBSCBS>"));
        assert!(!xml.contains("<indDoacao>"));
    }

    #[test]
    fn ibs_cbs_with_ad_valorem() {
        let g_ibs_uf = GIbsUfData::new("18.0000", "180.00");
        let g_ibs_mun = GIbsMunData::new("5.0000", "50.00");
        let g_cbs = GCbsData::new("9.0000", "90.00");
        let g = GIbsCbsData::new("1000.00", g_ibs_uf, g_ibs_mun, g_cbs).v_ibs("230.00");
        let data = IbsCbsData::new("00", "12345678")
            .ind_doacao(true)
            .g_ibs_cbs(g);
        let xml = build_ibs_cbs_xml(&data);
        assert!(xml.contains("<indDoacao>1</indDoacao>"));
        assert!(xml.contains("<gIBSCBS>"));
        assert!(xml.contains("<vBC>1000.00</vBC>"));
        assert!(xml.contains("<pIBSUF>18.0000</pIBSUF>"));
        assert!(xml.contains("<vIBSUF>180.00</vIBSUF>"));
        assert!(xml.contains("<pIBSMun>5.0000</pIBSMun>"));
        assert!(xml.contains("<vIBSMun>50.00</vIBSMun>"));
        assert!(xml.contains("<vIBS>230.00</vIBS>"));
        assert!(xml.contains("<pCBS>9.0000</pCBS>"));
        assert!(xml.contains("<vCBS>90.00</vCBS>"));
        assert!(xml.contains("</gIBSCBS>"));
    }

    #[test]
    fn ibs_cbs_with_diferimento() {
        let g_ibs_uf =
            GIbsUfData::new("18.0000", "162.00").g_dif(GDifData::new("10.0000", "18.00"));
        let g_ibs_mun = GIbsMunData::new("5.0000", "50.00");
        let g_cbs = GCbsData::new("9.0000", "90.00");
        let g = GIbsCbsData::new("1000.00", g_ibs_uf, g_ibs_mun, g_cbs).v_ibs("212.00");
        let data = IbsCbsData::new("00", "12345678").g_ibs_cbs(g);
        let xml = build_ibs_cbs_xml(&data);
        assert!(xml.contains("<gDif><pDif>10.0000</pDif><vDif>18.00</vDif></gDif>"));
    }

    #[test]
    fn ibs_cbs_monofasico() {
        let mono = GIbsCbsMonoData::new("15.00", "10.00").g_mono_padrao(GMonoPadraoData::new(
            "100.0000", "0.1500", "0.1000", "15.00", "10.00",
        ));
        let data = IbsCbsData::new("02", "87654321").g_ibs_cbs_mono(mono);
        let xml = build_ibs_cbs_xml(&data);
        assert!(xml.contains("<gIBSCBSMono>"));
        assert!(xml.contains("<gMonoPadrao>"));
        assert!(xml.contains("<qBCMono>100.0000</qBCMono>"));
        assert!(xml.contains("<vTotIBSMonoItem>15.00</vTotIBSMonoItem>"));
        assert!(xml.contains("<vTotCBSMonoItem>10.00</vTotCBSMonoItem>"));
    }

    #[test]
    fn ibs_cbs_transf_cred() {
        let data =
            IbsCbsData::new("03", "11111111").g_transf_cred(GTransfCredData::new("50.00", "30.00"));
        let xml = build_ibs_cbs_xml(&data);
        assert!(xml.contains("<gTransfCred>"));
        assert!(xml.contains("<vIBS>50.00</vIBS>"));
        assert!(xml.contains("<vCBS>30.00</vCBS>"));
    }

    #[test]
    fn ibs_cbs_estorno_cred() {
        let data = IbsCbsData::new("04", "22222222")
            .g_estorno_cred(GEstornoCredData::new("10.00", "5.00"));
        let xml = build_ibs_cbs_xml(&data);
        assert!(xml.contains("<gEstornoCred>"));
        assert!(xml.contains("<vIBSEstCred>10.00</vIBSEstCred>"));
        assert!(xml.contains("<vCBSEstCred>5.00</vCBSEstCred>"));
    }

    #[test]
    fn ibs_cbs_cred_pres_oper() {
        let data = IbsCbsData::new("05", "33333333").g_cred_pres_oper(
            GCredPresOperData::new("500.00", "01")
                .g_ibs_cred_pres(GIbsCredPresData::with_cred_pres("2.5000", "12.50"))
                .g_cbs_cred_pres(GCbsCredPresData::with_cred_pres_cond_sus("1.5000", "7.50")),
        );
        let xml = build_ibs_cbs_xml(&data);
        assert!(xml.contains("<gCredPresOper>"));
        assert!(xml.contains("<vBCCredPres>500.00</vBCCredPres>"));
        assert!(xml.contains("<gIBSCredPres>"));
        assert!(xml.contains("<vCredPres>12.50</vCredPres>"));
        assert!(xml.contains("<gCBSCredPres>"));
        assert!(xml.contains("<vCredPresCondSus>7.50</vCredPresCondSus>"));
    }

    #[test]
    fn is_tot_xml() {
        let data = IsTotData::new("15.00");
        let xml = build_is_tot_xml(&data);
        assert_eq!(xml, "<ISTot><vIS>15.00</vIS></ISTot>");
    }

    #[test]
    fn ibs_cbs_tot_minimal() {
        let data = IbsCbsTotData::new("1000.00");
        let xml = build_ibs_cbs_tot_xml(&data);
        assert!(xml.contains("<IBSCBSTot>"));
        assert!(xml.contains("<vBCIBSCBS>1000.00</vBCIBSCBS>"));
        // No gIBS/gCBS/gMono without data
        assert!(!xml.contains("<gIBS>"));
        assert!(!xml.contains("<gCBS>"));
        assert!(!xml.contains("<gMono>"));
    }

    #[test]
    fn ibs_cbs_tot_with_gibs_and_gcbs() {
        let mut data = IbsCbsTotData::new("1000.00");
        data.g_ibs_v_ibs = Some("230.00".into());
        data.g_ibs_uf_v_ibs_uf = Some("180.00".into());
        data.g_ibs_mun_v_ibs_mun = Some("50.00".into());
        data.g_cbs_v_cbs = Some("90.00".into());
        let xml = build_ibs_cbs_tot_xml(&data);
        assert!(xml.contains("<gIBS>"));
        assert!(xml.contains("<vIBSUF>180.00</vIBSUF>"));
        assert!(xml.contains("<vIBSMun>50.00</vIBSMun>"));
        assert!(xml.contains("<vIBS>230.00</vIBS>"));
        assert!(xml.contains("<gCBS>"));
        assert!(xml.contains("<vCBS>90.00</vCBS>"));
    }

    #[test]
    fn trib_regular_xml() {
        let data = IbsCbsData::new("00", "12345678").g_trib_regular(GTribRegularData::new(
            "01", "99999999", "18.0000", "180.00", "5.0000", "50.00", "9.0000", "90.00",
        ));
        let xml = build_ibs_cbs_xml(&data);
        assert!(xml.contains("<gTribRegular>"));
        assert!(xml.contains("<CSTReg>01</CSTReg>"));
        assert!(xml.contains("<pAliqEfetRegCBS>9.0000</pAliqEfetRegCBS>"));
        assert!(xml.contains("<vTribRegCBS>90.00</vTribRegCBS>"));
    }

    #[test]
    fn trib_compra_gov_xml() {
        let data = IbsCbsData::new("00", "12345678").g_trib_compra_gov(GTribCompraGovData::new(
            "18.0000", "180.00", "5.0000", "50.00", "9.0000", "90.00",
        ));
        let xml = build_ibs_cbs_xml(&data);
        assert!(xml.contains("<gTribCompraGov>"));
        assert!(xml.contains("<pAliqIBSUF>18.0000</pAliqIBSUF>"));
        assert!(xml.contains("<vTribCBS>90.00</vTribCBS>"));
    }

    #[test]
    fn cred_pres_ibs_zfm_xml() {
        let data = IbsCbsData::new("00", "12345678")
            .g_cred_pres_ibs_zfm(GCredPresIbsZfmData::new("1", "100.00").compet_apur("2025-06"));
        let xml = build_ibs_cbs_xml(&data);
        assert!(xml.contains("<gCredPresIBSZFM>"));
        assert!(xml.contains("<competApur>2025-06</competApur>"));
        assert!(xml.contains("<tpCredPresIBSZFM>1</tpCredPresIBSZFM>"));
        assert!(xml.contains("<vCredPresIBSZFM>100.00</vCredPresIBSZFM>"));
    }

    #[test]
    fn ajuste_compet_xml() {
        let data = IbsCbsData::new("00", "12345678")
            .g_ajuste_compet(GAjusteCompetData::new("2025-06", "50.00", "30.00"));
        let xml = build_ibs_cbs_xml(&data);
        assert!(xml.contains("<gAjusteCompet>"));
        assert!(xml.contains("<competApur>2025-06</competApur>"));
        assert!(xml.contains("<vIBS>50.00</vIBS>"));
        assert!(xml.contains("<vCBS>30.00</vCBS>"));
    }
}
