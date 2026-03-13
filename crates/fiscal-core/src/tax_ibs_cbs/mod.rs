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

mod build_item;
mod totals;
mod types_advalorem;
mod types_mono;
mod types_special;

pub use build_item::build_ibs_cbs_xml;
pub use totals::*;
pub use types_advalorem::*;
pub use types_mono::*;
pub use types_special::*;

#[cfg(test)]
mod tests;

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
