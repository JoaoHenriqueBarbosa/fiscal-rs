//! Ad-valorem data types: diferimento, devolução, redução, IBS UF/Mun, CBS,
//! gIBSCBS, tributação regular e compra governamental.

use serde::{Deserialize, Serialize};

/// Diferimento (deferment) data: `<gDif>` inside `<gIBSUF>`, `<gIBSMun>`, or `<gCBS>`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
