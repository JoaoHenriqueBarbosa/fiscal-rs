//! Tipos especiais: transferência de crédito, crédito presumido ZFM,
//! ajuste de competência, estorno de crédito, crédito presumido por operação.

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
