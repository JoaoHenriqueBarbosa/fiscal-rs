//! Monofásico (combustíveis) data types: padrão, retenção, retido anteriormente,
//! diferimento e grupo monofásico completo.

use serde::{Deserialize, Serialize};

// ── Monofasico ──────────────────────────────────────────────────────────

/// Monofasico padrao sub-group: `<gMonoPadrao>`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
