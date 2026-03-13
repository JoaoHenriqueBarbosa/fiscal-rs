//! ICMS tax computation and XML generation for NF-e / NFC-e documents.
//!
//! This module provides two main enum types:
//! - [`IcmsCst`] — for normal tax regime (Lucro Real / Lucro Presumido), covering
//!   CSTs 00, 02, 10, 15, 20, 30, 40, 41, 50, 51, 53, 60, 61, 70, and 90.
//! - [`IcmsCsosn`] — for Simples Nacional regime (CRT 1/2), covering CSOSNs
//!   101, 102, 103, 201, 202, 203, 300, 400, 500, and 900.
//!
//! Both are wrapped by the [`IcmsVariant`] enum, which is consumed by
//! [`build_icms_cst_xml`] / [`build_icms_csosn_xml`] to produce the `<ICMS>`
//! XML fragment and accumulate [`IcmsTotals`].
//!
//! There are also three auxiliary data structs for special ICMS groups:
//! [`IcmsPartData`] (partition), [`IcmsStData`] (ST repasse), and
//! [`IcmsUfDestData`] (interstate destination differential).

use crate::FiscalError;
use crate::format_utils::format_cents_or_none;
use crate::newtypes::{Cents, Rate};
use crate::tax_element::{
    TaxElement, TaxField, filter_fields, optional_field, required_field, serialize_tax_element,
};

/// Accumulate a value into a totals field.
fn accum(current: Cents, value: Option<Cents>) -> Cents {
    current + value.unwrap_or(Cents(0))
}

/// Accumulate a raw i64 quantity into a totals field.
fn accum_raw(current: i64, value: Option<i64>) -> i64 {
    current + value.unwrap_or(0)
}

// ── Types ───────────────────────────────────────────────────────────────────

/// Unified ICMS variant wrapping both normal-regime CSTs and Simples Nacional
/// CSOSNs. Pass one of these to [`build_icms_xml`] for compile-time-safe XML
/// generation.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum IcmsVariant {
    /// Normal tax regime (Lucro Real / Presumido).
    Cst(Box<IcmsCst>),
    /// Simples Nacional tax regime (CRT 1/2).
    Csosn(Box<IcmsCsosn>),
}

impl From<IcmsCst> for IcmsVariant {
    fn from(cst: IcmsCst) -> Self {
        Self::Cst(Box::new(cst))
    }
}

impl From<IcmsCsosn> for IcmsVariant {
    fn from(csosn: IcmsCsosn) -> Self {
        Self::Csosn(Box::new(csosn))
    }
}

/// Data for building the ICMSPart XML group (ICMS partition between states).
///
/// Used for interstate operations where the ICMS is split between origin and
/// destination states.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct IcmsPartData {
    /// Product origin code (`orig`).
    pub orig: String,
    /// ICMS CST code.
    pub cst: String,
    /// Base calculation modality (`modBC`).
    pub mod_bc: String,
    /// ICMS calculation base value (`vBC`).
    pub v_bc: Cents,
    /// Base reduction rate (`pRedBC`). Optional.
    pub p_red_bc: Option<Rate>,
    /// ICMS rate (`pICMS`).
    pub p_icms: Rate,
    /// ICMS value (`vICMS`).
    pub v_icms: Cents,
    /// ST base calculation modality (`modBCST`).
    pub mod_bc_st: String,
    /// ST added value margin (`pMVAST`). Optional.
    pub p_mva_st: Option<Rate>,
    /// ST base reduction rate (`pRedBCST`). Optional.
    pub p_red_bc_st: Option<Rate>,
    /// ST calculation base value (`vBCST`).
    pub v_bc_st: Cents,
    /// ST rate (`pICMSST`).
    pub p_icms_st: Rate,
    /// ST value (`vICMSST`).
    pub v_icms_st: Cents,
    /// FCP-ST calculation base (`vBCFCPST`). Optional.
    pub v_bc_fcp_st: Option<Cents>,
    /// FCP-ST rate (`pFCPST`). Optional.
    pub p_fcp_st: Option<Rate>,
    /// FCP-ST value (`vFCPST`). Optional.
    pub v_fcp_st: Option<Cents>,
    /// Partition percentage applied at origin state (`pBCOp`).
    pub p_bc_op: Rate,
    /// Destination state abbreviation for ST (`UFST`).
    pub uf_st: String,
    /// Desonerated ICMS value (`vICMSDeson`). Optional.
    pub v_icms_deson: Option<Cents>,
    /// Reason code for ICMS desoneration (`motDesICMS`). Optional.
    pub mot_des_icms: Option<String>,
    /// Indicator whether desoneration is deducted (`indDeduzDeson`). Optional.
    pub ind_deduz_deson: Option<String>,
}

impl IcmsPartData {
    /// Create a new `IcmsPartData` with all required fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        orig: impl Into<String>,
        cst: impl Into<String>,
        mod_bc: impl Into<String>,
        v_bc: Cents,
        p_icms: Rate,
        v_icms: Cents,
        mod_bc_st: impl Into<String>,
        v_bc_st: Cents,
        p_icms_st: Rate,
        v_icms_st: Cents,
        p_bc_op: Rate,
        uf_st: impl Into<String>,
    ) -> Self {
        Self {
            orig: orig.into(),
            cst: cst.into(),
            mod_bc: mod_bc.into(),
            v_bc,
            p_red_bc: None,
            p_icms,
            v_icms,
            mod_bc_st: mod_bc_st.into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st,
            p_icms_st,
            v_icms_st,
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
            p_bc_op,
            uf_st: uf_st.into(),
            v_icms_deson: None,
            mot_des_icms: None,
            ind_deduz_deson: None,
        }
    }
    /// Set the ICMS base reduction rate (`pRedBC`).
    pub fn p_red_bc(mut self, v: Rate) -> Self {
        self.p_red_bc = Some(v);
        self
    }
    /// Set the ST added value margin (`pMVAST`).
    pub fn p_mva_st(mut self, v: Rate) -> Self {
        self.p_mva_st = Some(v);
        self
    }
    /// Set the ST base reduction rate (`pRedBCST`).
    pub fn p_red_bc_st(mut self, v: Rate) -> Self {
        self.p_red_bc_st = Some(v);
        self
    }
    /// Set the FCP-ST calculation base (`vBCFCPST`).
    pub fn v_bc_fcp_st(mut self, v: Cents) -> Self {
        self.v_bc_fcp_st = Some(v);
        self
    }
    /// Set the FCP-ST rate (`pFCPST`).
    pub fn p_fcp_st(mut self, v: Rate) -> Self {
        self.p_fcp_st = Some(v);
        self
    }
    /// Set the FCP-ST value (`vFCPST`).
    pub fn v_fcp_st(mut self, v: Cents) -> Self {
        self.v_fcp_st = Some(v);
        self
    }
    /// Set the desonerated ICMS value (`vICMSDeson`).
    pub fn v_icms_deson(mut self, v: Cents) -> Self {
        self.v_icms_deson = Some(v);
        self
    }
    /// Set the ICMS desoneration reason code (`motDesICMS`).
    pub fn mot_des_icms(mut self, v: impl Into<String>) -> Self {
        self.mot_des_icms = Some(v.into());
        self
    }
    /// Set the desoneration deduction indicator (`indDeduzDeson`).
    pub fn ind_deduz_deson(mut self, v: impl Into<String>) -> Self {
        self.ind_deduz_deson = Some(v.into());
        self
    }
}

/// Data for building the ICMSST XML group (ST repasse).
///
/// Used for CST 41 or 60 operations with ST transfer (`repasse`) between states.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct IcmsStData {
    /// Product origin code (`orig`).
    pub orig: String,
    /// ICMS CST code.
    pub cst: String,
    /// ST retained calculation base (`vBCSTRet`).
    pub v_bc_st_ret: Cents,
    /// ST rate applied at retention (`pST`). Optional.
    pub p_st: Option<Rate>,
    /// ICMS value paid by the substitutor (`vICMSSubstituto`). Optional.
    pub v_icms_substituto: Option<Cents>,
    /// Retained ST ICMS value (`vICMSSTRet`).
    pub v_icms_st_ret: Cents,
    /// FCP-ST retained calculation base (`vBCFCPSTRet`). Optional.
    pub v_bc_fcp_st_ret: Option<Cents>,
    /// FCP-ST retained rate (`pFCPSTRet`). Optional.
    pub p_fcp_st_ret: Option<Rate>,
    /// FCP-ST retained value (`vFCPSTRet`). Optional.
    pub v_fcp_st_ret: Option<Cents>,
    /// ST calculation base for destination state (`vBCSTDest`).
    pub v_bc_st_dest: Cents,
    /// ICMS ST value for destination state (`vICMSSTDest`).
    pub v_icms_st_dest: Cents,
    /// Effective base reduction rate (`pRedBCEfet`). Optional.
    pub p_red_bc_efet: Option<Rate>,
    /// Effective calculation base (`vBCEfet`). Optional.
    pub v_bc_efet: Option<Cents>,
    /// Effective ICMS rate (`pICMSEfet`). Optional.
    pub p_icms_efet: Option<Rate>,
    /// Effective ICMS value (`vICMSEfet`). Optional.
    pub v_icms_efet: Option<Cents>,
}

impl IcmsStData {
    /// Create a new `IcmsStData` with required fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        orig: impl Into<String>,
        cst: impl Into<String>,
        v_bc_st_ret: Cents,
        v_icms_st_ret: Cents,
        v_bc_st_dest: Cents,
        v_icms_st_dest: Cents,
    ) -> Self {
        Self {
            orig: orig.into(),
            cst: cst.into(),
            v_bc_st_ret,
            p_st: None,
            v_icms_substituto: None,
            v_icms_st_ret,
            v_bc_fcp_st_ret: None,
            p_fcp_st_ret: None,
            v_fcp_st_ret: None,
            v_bc_st_dest,
            v_icms_st_dest,
            p_red_bc_efet: None,
            v_bc_efet: None,
            p_icms_efet: None,
            v_icms_efet: None,
        }
    }
    /// Set the ST rate at retention (`pST`).
    pub fn p_st(mut self, v: Rate) -> Self {
        self.p_st = Some(v);
        self
    }
    /// Set the ICMS value paid by the substitutor (`vICMSSubstituto`).
    pub fn v_icms_substituto(mut self, v: Cents) -> Self {
        self.v_icms_substituto = Some(v);
        self
    }
    /// Set the FCP-ST retained calculation base (`vBCFCPSTRet`).
    pub fn v_bc_fcp_st_ret(mut self, v: Cents) -> Self {
        self.v_bc_fcp_st_ret = Some(v);
        self
    }
    /// Set the FCP-ST retained rate (`pFCPSTRet`).
    pub fn p_fcp_st_ret(mut self, v: Rate) -> Self {
        self.p_fcp_st_ret = Some(v);
        self
    }
    /// Set the FCP-ST retained value (`vFCPSTRet`).
    pub fn v_fcp_st_ret(mut self, v: Cents) -> Self {
        self.v_fcp_st_ret = Some(v);
        self
    }
    /// Set the effective base reduction rate (`pRedBCEfet`).
    pub fn p_red_bc_efet(mut self, v: Rate) -> Self {
        self.p_red_bc_efet = Some(v);
        self
    }
    /// Set the effective calculation base (`vBCEfet`).
    pub fn v_bc_efet(mut self, v: Cents) -> Self {
        self.v_bc_efet = Some(v);
        self
    }
    /// Set the effective ICMS rate (`pICMSEfet`).
    pub fn p_icms_efet(mut self, v: Rate) -> Self {
        self.p_icms_efet = Some(v);
        self
    }
    /// Set the effective ICMS value (`vICMSEfet`).
    pub fn v_icms_efet(mut self, v: Cents) -> Self {
        self.v_icms_efet = Some(v);
        self
    }
}

/// Data for building the ICMSUFDest XML group (interstate destination).
///
/// Represents the ICMS differential (`DIFAL`) owed to the destination state
/// for interstate B2C operations (EC 87/2015).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct IcmsUfDestData {
    /// ICMS calculation base for destination state (`vBCUFDest`).
    pub v_bc_uf_dest: Cents,
    /// FCP calculation base for destination state (`vBCFCPUFDest`). Optional.
    pub v_bc_fcp_uf_dest: Option<Cents>,
    /// FCP rate for destination state (`pFCPUFDest`). Optional.
    pub p_fcp_uf_dest: Option<Rate>,
    /// Internal ICMS rate for destination state (`pICMSUFDest`).
    pub p_icms_uf_dest: Rate,
    /// Interstate ICMS rate (`pICMSInter`).
    pub p_icms_inter: Rate,
    /// FCP value for destination state (`vFCPUFDest`). Optional.
    pub v_fcp_uf_dest: Option<Cents>,
    /// ICMS value destined to destination state (`vICMSUFDest`).
    pub v_icms_uf_dest: Cents,
    /// ICMS value to be paid to origin state (`vICMSUFRemet`). Optional.
    pub v_icms_uf_remet: Option<Cents>,
}

impl IcmsUfDestData {
    /// Create a new `IcmsUfDestData` with required fields.
    pub fn new(
        v_bc_uf_dest: Cents,
        p_icms_uf_dest: Rate,
        p_icms_inter: Rate,
        v_icms_uf_dest: Cents,
    ) -> Self {
        Self {
            v_bc_uf_dest,
            v_bc_fcp_uf_dest: None,
            p_fcp_uf_dest: None,
            p_icms_uf_dest,
            p_icms_inter,
            v_fcp_uf_dest: None,
            v_icms_uf_dest,
            v_icms_uf_remet: None,
        }
    }
    /// Set the FCP base value for destination.
    pub fn v_bc_fcp_uf_dest(mut self, v: Cents) -> Self {
        self.v_bc_fcp_uf_dest = Some(v);
        self
    }
    /// Set the FCP rate for destination.
    pub fn p_fcp_uf_dest(mut self, v: Rate) -> Self {
        self.p_fcp_uf_dest = Some(v);
        self
    }
    /// Set the FCP value for destination.
    pub fn v_fcp_uf_dest(mut self, v: Cents) -> Self {
        self.v_fcp_uf_dest = Some(v);
        self
    }
    /// Set the ICMS value for origin state.
    pub fn v_icms_uf_remet(mut self, v: Cents) -> Self {
        self.v_icms_uf_remet = Some(v);
        self
    }
}

/// Accumulated ICMS totals across all NF-e items.
///
/// This struct is filled incrementally by [`build_icms_xml`] /
/// [`build_icms_cst_xml`] / [`build_icms_csosn_xml`] as each item is
/// processed, then passed to the XML builder to generate the `<ICMSTot>`
/// element. Start with [`IcmsTotals::new`] (or [`create_icms_totals`]) and
/// use [`merge_icms_totals`] when accumulating per-item sub-totals.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct IcmsTotals {
    /// Total ICMS calculation base (`vBC`).
    pub v_bc: Cents,
    /// Total ICMS value (`vICMS`).
    pub v_icms: Cents,
    /// Total desonerated ICMS value (`vICMSDeson`).
    pub v_icms_deson: Cents,
    /// Total ST calculation base (`vBCST`).
    pub v_bc_st: Cents,
    /// Total ST ICMS value (`vST`).
    pub v_st: Cents,
    /// Total FCP value (`vFCP`).
    pub v_fcp: Cents,
    /// Total FCP-ST value (`vFCPST`).
    pub v_fcp_st: Cents,
    /// Total retained FCP-ST value (`vFCPSTRet`).
    pub v_fcp_st_ret: Cents,
    /// Total FCP value for destination state (`vFCPUFDest`).
    pub v_fcp_uf_dest: Cents,
    /// Total ICMS value for destination state (`vICMSUFDest`).
    pub v_icms_uf_dest: Cents,
    /// Total ICMS value for origin state / remitter (`vICMSUFRemet`).
    pub v_icms_uf_remet: Cents,
    /// Total monophasic calculation base quantity (`qBCMono`).
    pub q_bc_mono: i64,
    /// Total monophasic ICMS value (`vICMSMono`).
    pub v_icms_mono: Cents,
    /// Total monophasic retained calculation base quantity (`qBCMonoReten`).
    pub q_bc_mono_reten: i64,
    /// Total monophasic retained ICMS value (`vICMSMonoReten`).
    pub v_icms_mono_reten: Cents,
    /// Total monophasic previously-collected calculation base quantity (`qBCMonoRet`).
    pub q_bc_mono_ret: i64,
    /// Total monophasic previously-collected ICMS value (`vICMSMonoRet`).
    pub v_icms_mono_ret: Cents,
    /// Whether ICMS desoneration should be deducted from vNF (`indDeduzDeson`).
    ///
    /// Mirrors the PHP class-level `$this->indDeduzDeson` flag.  When any item
    /// sets `indDeduzDeson=1`, this flag becomes `true` and the accumulated
    /// `v_icms_deson` is subtracted from vNF.
    pub ind_deduz_deson: bool,
}

impl IcmsTotals {
    /// Create a new zeroed-out `IcmsTotals`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Set the total ICMS calculation base (`vBC`).
    pub fn v_bc(mut self, v: Cents) -> Self {
        self.v_bc = v;
        self
    }
    /// Set the total ICMS value (`vICMS`).
    pub fn v_icms(mut self, v: Cents) -> Self {
        self.v_icms = v;
        self
    }
    /// Set the total desonerated ICMS value (`vICMSDeson`).
    pub fn v_icms_deson(mut self, v: Cents) -> Self {
        self.v_icms_deson = v;
        self
    }
    /// Set the total ST calculation base (`vBCST`).
    pub fn v_bc_st(mut self, v: Cents) -> Self {
        self.v_bc_st = v;
        self
    }
    /// Set the total ST ICMS value (`vST`).
    pub fn v_st(mut self, v: Cents) -> Self {
        self.v_st = v;
        self
    }
    /// Set the total FCP value (`vFCP`).
    pub fn v_fcp(mut self, v: Cents) -> Self {
        self.v_fcp = v;
        self
    }
    /// Set the total FCP-ST value (`vFCPST`).
    pub fn v_fcp_st(mut self, v: Cents) -> Self {
        self.v_fcp_st = v;
        self
    }
    /// Set the total retained FCP-ST value (`vFCPSTRet`).
    pub fn v_fcp_st_ret(mut self, v: Cents) -> Self {
        self.v_fcp_st_ret = v;
        self
    }
    /// Set the total FCP value for destination state (`vFCPUFDest`).
    pub fn v_fcp_uf_dest(mut self, v: Cents) -> Self {
        self.v_fcp_uf_dest = v;
        self
    }
    /// Set the total ICMS value for destination state (`vICMSUFDest`).
    pub fn v_icms_uf_dest(mut self, v: Cents) -> Self {
        self.v_icms_uf_dest = v;
        self
    }
    /// Set the total ICMS value for origin state (`vICMSUFRemet`).
    pub fn v_icms_uf_remet(mut self, v: Cents) -> Self {
        self.v_icms_uf_remet = v;
        self
    }
    /// Set the total monophasic ICMS value (`vICMSMono`).
    pub fn v_icms_mono(mut self, v: Cents) -> Self {
        self.v_icms_mono = v;
        self
    }
    /// Set the total monophasic retained ICMS value (`vICMSMonoReten`).
    pub fn v_icms_mono_reten(mut self, v: Cents) -> Self {
        self.v_icms_mono_reten = v;
        self
    }
    /// Set the total monophasic previously-collected ICMS value (`vICMSMonoRet`).
    pub fn v_icms_mono_ret(mut self, v: Cents) -> Self {
        self.v_icms_mono_ret = v;
        self
    }
    /// Set the desoneration deduction indicator (`indDeduzDeson`).
    ///
    /// When `true`, the accumulated `v_icms_deson` is subtracted from vNF.
    pub fn ind_deduz_deson(mut self, v: bool) -> Self {
        self.ind_deduz_deson = v;
        self
    }
}

/// Create a zeroed-out [`IcmsTotals`] accumulator.
///
/// Equivalent to `IcmsTotals::new()`. Provided as a free function for
/// ergonomic use in XML builder pipelines.
///
/// # Examples
///
/// ```
/// use fiscal_core::tax_icms::create_icms_totals;
/// let totals = create_icms_totals();
/// use fiscal_core::newtypes::Cents;
/// assert_eq!(totals.v_bc, Cents(0));
/// ```
pub fn create_icms_totals() -> IcmsTotals {
    IcmsTotals::default()
}

// ── IcmsCst enum (normal regime) ────────────────────────────────────────────

/// ICMS CST variant for normal tax regime (Lucro Real / Presumido).
///
/// Each variant carries **only** the fields that are valid for that CST,
/// giving compile-time safety instead of runtime string matching against a
/// flat struct full of `Option`s.
///
/// Simples Nacional / CSOSN variants are **not** included here (see R7).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum IcmsCst {
    /// CST 00 — Tributada integralmente.
    Cst00 {
        /// Product origin code (`orig`).
        orig: String,
        /// Base calculation modality (`modBC`).
        mod_bc: String,
        /// Calculation base value (`vBC`).
        v_bc: Cents,
        /// ICMS rate (`pICMS`).
        p_icms: Rate,
        /// ICMS value (`vICMS`).
        v_icms: Cents,
        /// FCP rate (`pFCP`). Optional.
        p_fcp: Option<Rate>,
        /// FCP value (`vFCP`). Optional.
        v_fcp: Option<Cents>,
    },
    /// CST 02 — Tributacao monofasica propria sobre combustiveis.
    Cst02 {
        /// Product origin code (`orig`).
        orig: String,
        /// Monophasic calculation base quantity (`qBCMono`). Optional.
        q_bc_mono: Option<i64>,
        /// Monophasic ad-rem ICMS rate (`adRemICMS`).
        ad_rem_icms: Rate,
        /// Monophasic ICMS value (`vICMSMono`).
        v_icms_mono: Cents,
    },
    /// CST 10 — Tributada e com cobranca do ICMS por substituicao tributaria.
    Cst10 {
        /// Product origin code (`orig`).
        orig: String,
        /// Base calculation modality (`modBC`).
        mod_bc: String,
        /// ICMS calculation base value (`vBC`).
        v_bc: Cents,
        /// ICMS rate (`pICMS`).
        p_icms: Rate,
        /// ICMS value (`vICMS`).
        v_icms: Cents,
        /// FCP calculation base (`vBCFCP`). Optional.
        v_bc_fcp: Option<Cents>,
        /// FCP rate (`pFCP`). Optional.
        p_fcp: Option<Rate>,
        /// FCP value (`vFCP`). Optional.
        v_fcp: Option<Cents>,
        /// ST base calculation modality (`modBCST`).
        mod_bc_st: String,
        /// ST added value margin (`pMVAST`). Optional.
        p_mva_st: Option<Rate>,
        /// ST base reduction rate (`pRedBCST`). Optional.
        p_red_bc_st: Option<Rate>,
        /// ST calculation base value (`vBCST`).
        v_bc_st: Cents,
        /// ST rate (`pICMSST`).
        p_icms_st: Rate,
        /// ST ICMS value (`vICMSST`).
        v_icms_st: Cents,
        /// FCP-ST calculation base (`vBCFCPST`). Optional.
        v_bc_fcp_st: Option<Cents>,
        /// FCP-ST rate (`pFCPST`). Optional.
        p_fcp_st: Option<Rate>,
        /// FCP-ST value (`vFCPST`). Optional.
        v_fcp_st: Option<Cents>,
        /// ST desonerated ICMS value (`vICMSSTDeson`). Optional.
        v_icms_st_deson: Option<Cents>,
        /// ST desoneration reason code (`motDesICMSST`). Optional.
        mot_des_icms_st: Option<String>,
    },
    /// CST 15 — Tributacao monofasica propria e com responsabilidade pela
    /// retencao sobre combustiveis.
    Cst15 {
        /// Product origin code (`orig`).
        orig: String,
        /// Monophasic calculation base quantity (`qBCMono`). Optional.
        q_bc_mono: Option<i64>,
        /// Monophasic ad-rem ICMS rate (`adRemICMS`).
        ad_rem_icms: Rate,
        /// Monophasic ICMS value (`vICMSMono`).
        v_icms_mono: Cents,
        /// Retained monophasic calculation base quantity (`qBCMonoReten`). Optional.
        q_bc_mono_reten: Option<i64>,
        /// Retained monophasic ad-rem ICMS rate (`adRemICMSReten`).
        ad_rem_icms_reten: Rate,
        /// Retained monophasic ICMS value (`vICMSMonoReten`).
        v_icms_mono_reten: Cents,
        /// Ad-rem reduction rate (`pRedAdRem`). Optional.
        p_red_ad_rem: Option<Rate>,
        /// Ad-rem reduction reason (`motRedAdRem`). Required when `p_red_ad_rem` is set.
        mot_red_ad_rem: Option<String>,
    },
    /// CST 20 — Com reducao de base de calculo.
    Cst20 {
        /// Product origin code (`orig`).
        orig: String,
        /// Base calculation modality (`modBC`).
        mod_bc: String,
        /// Base reduction rate (`pRedBC`).
        p_red_bc: Rate,
        /// Calculation base value (`vBC`).
        v_bc: Cents,
        /// ICMS rate (`pICMS`).
        p_icms: Rate,
        /// ICMS value (`vICMS`).
        v_icms: Cents,
        /// FCP calculation base (`vBCFCP`). Optional.
        v_bc_fcp: Option<Cents>,
        /// FCP rate (`pFCP`). Optional.
        p_fcp: Option<Rate>,
        /// FCP value (`vFCP`). Optional.
        v_fcp: Option<Cents>,
        /// Desonerated ICMS value (`vICMSDeson`). Optional.
        v_icms_deson: Option<Cents>,
        /// Desoneration reason code (`motDesICMS`). Optional.
        mot_des_icms: Option<String>,
        /// Desoneration deduction indicator (`indDeduzDeson`). Optional.
        ind_deduz_deson: Option<String>,
    },
    /// CST 30 — Isenta ou nao tributada e com cobranca do ICMS por ST.
    Cst30 {
        /// Product origin code (`orig`).
        orig: String,
        /// ST base calculation modality (`modBCST`).
        mod_bc_st: String,
        /// ST added value margin (`pMVAST`). Optional.
        p_mva_st: Option<Rate>,
        /// ST base reduction rate (`pRedBCST`). Optional.
        p_red_bc_st: Option<Rate>,
        /// ST calculation base value (`vBCST`).
        v_bc_st: Cents,
        /// ST rate (`pICMSST`).
        p_icms_st: Rate,
        /// ST ICMS value (`vICMSST`).
        v_icms_st: Cents,
        /// FCP-ST calculation base (`vBCFCPST`). Optional.
        v_bc_fcp_st: Option<Cents>,
        /// FCP-ST rate (`pFCPST`). Optional.
        p_fcp_st: Option<Rate>,
        /// FCP-ST value (`vFCPST`). Optional.
        v_fcp_st: Option<Cents>,
        /// Desonerated ICMS value (`vICMSDeson`). Optional.
        v_icms_deson: Option<Cents>,
        /// Desoneration reason code (`motDesICMS`). Optional.
        mot_des_icms: Option<String>,
        /// Desoneration deduction indicator (`indDeduzDeson`). Optional.
        ind_deduz_deson: Option<String>,
    },
    /// CST 40 — Isenta.
    Cst40 {
        /// Product origin code (`orig`).
        orig: String,
        /// Desonerated ICMS value (`vICMSDeson`). Optional.
        v_icms_deson: Option<Cents>,
        /// Desoneration reason code (`motDesICMS`). Optional.
        mot_des_icms: Option<String>,
        /// Desoneration deduction indicator (`indDeduzDeson`). Optional.
        ind_deduz_deson: Option<String>,
    },
    /// CST 41 — Nao tributada.
    Cst41 {
        /// Product origin code (`orig`).
        orig: String,
        /// Desonerated ICMS value (`vICMSDeson`). Optional.
        v_icms_deson: Option<Cents>,
        /// Desoneration reason code (`motDesICMS`). Optional.
        mot_des_icms: Option<String>,
        /// Desoneration deduction indicator (`indDeduzDeson`). Optional.
        ind_deduz_deson: Option<String>,
    },
    /// CST 50 — Suspensao.
    Cst50 {
        /// Product origin code (`orig`).
        orig: String,
        /// Desonerated ICMS value (`vICMSDeson`). Optional.
        v_icms_deson: Option<Cents>,
        /// Desoneration reason code (`motDesICMS`). Optional.
        mot_des_icms: Option<String>,
        /// Desoneration deduction indicator (`indDeduzDeson`). Optional.
        ind_deduz_deson: Option<String>,
    },
    /// CST 51 — Diferimento.
    Cst51 {
        /// Product origin code (`orig`).
        orig: String,
        /// Base calculation modality (`modBC`). Optional.
        mod_bc: Option<String>,
        /// Base reduction rate (`pRedBC`). Optional.
        p_red_bc: Option<Rate>,
        /// Fiscal benefit code for base reduction (`cBenefRBC`). Optional.
        c_benef_rbc: Option<String>,
        /// Calculation base value (`vBC`). Optional.
        v_bc: Option<Cents>,
        /// ICMS rate (`pICMS`). Optional.
        p_icms: Option<Rate>,
        /// ICMS value before deferral (`vICMSOp`). Optional.
        v_icms_op: Option<Cents>,
        /// Deferral percentage (`pDif`). Optional.
        p_dif: Option<Rate>,
        /// Deferred ICMS value (`vICMSDif`). Optional.
        v_icms_dif: Option<Cents>,
        /// ICMS value payable after deferral (`vICMS`). Optional.
        v_icms: Option<Cents>,
        /// FCP calculation base (`vBCFCP`). Optional.
        v_bc_fcp: Option<Cents>,
        /// FCP rate (`pFCP`). Optional.
        p_fcp: Option<Rate>,
        /// FCP value (`vFCP`). Optional.
        v_fcp: Option<Cents>,
        /// FCP deferral rate (`pFCPDif`). Optional.
        p_fcp_dif: Option<Rate>,
        /// FCP deferred value (`vFCPDif`). Optional.
        v_fcp_dif: Option<Cents>,
        /// FCP effective value after deferral (`vFCPEfet`). Optional.
        v_fcp_efet: Option<Cents>,
    },
    /// CST 53 — Tributacao monofasica sobre combustiveis com recolhimento
    /// diferido.
    Cst53 {
        /// Product origin code (`orig`).
        orig: String,
        /// Monophasic calculation base quantity (`qBCMono`). Optional.
        q_bc_mono: Option<i64>,
        /// Monophasic ad-rem ICMS rate (`adRemICMS`). Optional.
        ad_rem_icms: Option<Rate>,
        /// Monophasic ICMS value before deferral (`vICMSMonoOp`). Optional.
        v_icms_mono_op: Option<Cents>,
        /// Deferral percentage (`pDif`). Optional.
        p_dif: Option<Rate>,
        /// Deferred monophasic ICMS value (`vICMSMonoDif`). Optional.
        v_icms_mono_dif: Option<Cents>,
        /// Monophasic ICMS value payable after deferral (`vICMSMono`). Optional.
        v_icms_mono: Option<Cents>,
    },
    /// CST 60 — ICMS cobrado anteriormente por substituicao tributaria.
    Cst60 {
        /// Product origin code (`orig`).
        orig: String,
        /// ST retained calculation base (`vBCSTRet`). Optional.
        v_bc_st_ret: Option<Cents>,
        /// ST rate at retention (`pST`). Optional.
        p_st: Option<Rate>,
        /// ICMS value paid by the substitutor (`vICMSSubstituto`). Optional.
        v_icms_substituto: Option<Cents>,
        /// Retained ST ICMS value (`vICMSSTRet`). Optional.
        v_icms_st_ret: Option<Cents>,
        /// FCP-ST retained calculation base (`vBCFCPSTRet`). Optional.
        v_bc_fcp_st_ret: Option<Cents>,
        /// FCP-ST retained rate (`pFCPSTRet`). Optional.
        p_fcp_st_ret: Option<Rate>,
        /// FCP-ST retained value (`vFCPSTRet`). Optional.
        v_fcp_st_ret: Option<Cents>,
        /// Effective base reduction rate (`pRedBCEfet`). Optional.
        p_red_bc_efet: Option<Rate>,
        /// Effective calculation base (`vBCEfet`). Optional.
        v_bc_efet: Option<Cents>,
        /// Effective ICMS rate (`pICMSEfet`). Optional.
        p_icms_efet: Option<Rate>,
        /// Effective ICMS value (`vICMSEfet`). Optional.
        v_icms_efet: Option<Cents>,
    },
    /// CST 61 — Tributacao monofasica sobre combustiveis cobrada anteriormente.
    Cst61 {
        /// Product origin code (`orig`).
        orig: String,
        /// Monophasic previously-collected calculation base quantity (`qBCMonoRet`). Optional.
        q_bc_mono_ret: Option<i64>,
        /// Monophasic previously-collected ad-rem ICMS rate (`adRemICMSRet`).
        ad_rem_icms_ret: Rate,
        /// Monophasic previously-collected ICMS value (`vICMSMonoRet`).
        v_icms_mono_ret: Cents,
    },
    /// CST 70 — Reducao de base de calculo e cobranca do ICMS por ST.
    Cst70 {
        /// Product origin code (`orig`).
        orig: String,
        /// Base calculation modality (`modBC`).
        mod_bc: String,
        /// Base reduction rate (`pRedBC`).
        p_red_bc: Rate,
        /// ICMS calculation base value (`vBC`).
        v_bc: Cents,
        /// ICMS rate (`pICMS`).
        p_icms: Rate,
        /// ICMS value (`vICMS`).
        v_icms: Cents,
        /// FCP calculation base (`vBCFCP`). Optional.
        v_bc_fcp: Option<Cents>,
        /// FCP rate (`pFCP`). Optional.
        p_fcp: Option<Rate>,
        /// FCP value (`vFCP`). Optional.
        v_fcp: Option<Cents>,
        /// ST base calculation modality (`modBCST`).
        mod_bc_st: String,
        /// ST added value margin (`pMVAST`). Optional.
        p_mva_st: Option<Rate>,
        /// ST base reduction rate (`pRedBCST`). Optional.
        p_red_bc_st: Option<Rate>,
        /// ST calculation base value (`vBCST`).
        v_bc_st: Cents,
        /// ST rate (`pICMSST`).
        p_icms_st: Rate,
        /// ST ICMS value (`vICMSST`).
        v_icms_st: Cents,
        /// FCP-ST calculation base (`vBCFCPST`). Optional.
        v_bc_fcp_st: Option<Cents>,
        /// FCP-ST rate (`pFCPST`). Optional.
        p_fcp_st: Option<Rate>,
        /// FCP-ST value (`vFCPST`). Optional.
        v_fcp_st: Option<Cents>,
        /// Desonerated ICMS value (`vICMSDeson`). Optional.
        v_icms_deson: Option<Cents>,
        /// Desoneration reason code (`motDesICMS`). Optional.
        mot_des_icms: Option<String>,
        /// Desoneration deduction indicator (`indDeduzDeson`). Optional.
        ind_deduz_deson: Option<String>,
        /// ST desonerated ICMS value (`vICMSSTDeson`). Optional.
        v_icms_st_deson: Option<Cents>,
        /// ST desoneration reason code (`motDesICMSST`). Optional.
        mot_des_icms_st: Option<String>,
    },
    /// CST 90 — Outros.
    Cst90 {
        /// Product origin code (`orig`).
        orig: String,
        /// Base calculation modality (`modBC`). Optional.
        mod_bc: Option<String>,
        /// ICMS calculation base value (`vBC`). Optional.
        v_bc: Option<Cents>,
        /// Base reduction rate (`pRedBC`). Optional.
        p_red_bc: Option<Rate>,
        /// Fiscal benefit code for base reduction (`cBenefRBC`). Optional.
        c_benef_rbc: Option<String>,
        /// ICMS rate (`pICMS`). Optional.
        p_icms: Option<Rate>,
        /// ICMS value before deferral (`vICMSOp`). Optional.
        v_icms_op: Option<Cents>,
        /// Deferral percentage (`pDif`). Optional.
        p_dif: Option<Rate>,
        /// Deferred ICMS value (`vICMSDif`). Optional.
        v_icms_dif: Option<Cents>,
        /// ICMS value (`vICMS`). Optional.
        v_icms: Option<Cents>,
        /// FCP calculation base (`vBCFCP`). Optional.
        v_bc_fcp: Option<Cents>,
        /// FCP rate (`pFCP`). Optional.
        p_fcp: Option<Rate>,
        /// FCP value (`vFCP`). Optional.
        v_fcp: Option<Cents>,
        /// FCP deferral rate (`pFCPDif`). Optional.
        p_fcp_dif: Option<Rate>,
        /// FCP deferred value (`vFCPDif`). Optional.
        v_fcp_dif: Option<Cents>,
        /// FCP effective value (`vFCPEfet`). Optional.
        v_fcp_efet: Option<Cents>,
        /// ST base calculation modality (`modBCST`). Optional.
        mod_bc_st: Option<String>,
        /// ST added value margin (`pMVAST`). Optional.
        p_mva_st: Option<Rate>,
        /// ST base reduction rate (`pRedBCST`). Optional.
        p_red_bc_st: Option<Rate>,
        /// ST calculation base value (`vBCST`). Optional.
        v_bc_st: Option<Cents>,
        /// ST rate (`pICMSST`). Optional.
        p_icms_st: Option<Rate>,
        /// ST ICMS value (`vICMSST`). Optional.
        v_icms_st: Option<Cents>,
        /// FCP-ST calculation base (`vBCFCPST`). Optional.
        v_bc_fcp_st: Option<Cents>,
        /// FCP-ST rate (`pFCPST`). Optional.
        p_fcp_st: Option<Rate>,
        /// FCP-ST value (`vFCPST`). Optional.
        v_fcp_st: Option<Cents>,
        /// Desonerated ICMS value (`vICMSDeson`). Optional.
        v_icms_deson: Option<Cents>,
        /// Desoneration reason code (`motDesICMS`). Optional.
        mot_des_icms: Option<String>,
        /// Desoneration deduction indicator (`indDeduzDeson`). Optional.
        ind_deduz_deson: Option<String>,
        /// ST desonerated ICMS value (`vICMSSTDeson`). Optional.
        v_icms_st_deson: Option<Cents>,
        /// ST desoneration reason code (`motDesICMSST`). Optional.
        mot_des_icms_st: Option<String>,
    },
}

impl IcmsCst {
    /// Return the two-character CST code for this variant.
    pub fn cst_code(&self) -> &str {
        match self {
            Self::Cst00 { .. } => "00",
            Self::Cst02 { .. } => "02",
            Self::Cst10 { .. } => "10",
            Self::Cst15 { .. } => "15",
            Self::Cst20 { .. } => "20",
            Self::Cst30 { .. } => "30",
            Self::Cst40 { .. } => "40",
            Self::Cst41 { .. } => "41",
            Self::Cst50 { .. } => "50",
            Self::Cst51 { .. } => "51",
            Self::Cst53 { .. } => "53",
            Self::Cst60 { .. } => "60",
            Self::Cst61 { .. } => "61",
            Self::Cst70 { .. } => "70",
            Self::Cst90 { .. } => "90",
        }
    }
}

/// Build the ICMS XML fragment and accumulate totals from a typed [`IcmsCst`]
/// variant.
///
/// This is the compile-time-safe counterpart of the original
/// [`build_icms_xml`] code path for normal-regime CSTs. It can be used
/// directly by new code that already has an `IcmsCst`, or indirectly via the
/// unchanged [`build_icms_xml`] public API (which converts internally).
///
/// # Errors
///
/// Returns [`FiscalError`] if XML field serialization fails (should not happen
/// when the enum is correctly constructed).
pub fn build_icms_cst_xml(
    cst: &IcmsCst,
    totals: &mut IcmsTotals,
) -> Result<(String, Vec<TaxField>), FiscalError> {
    match cst {
        IcmsCst::Cst00 {
            orig,
            mod_bc,
            v_bc,
            p_icms,
            v_icms,
            p_fcp,
            v_fcp,
        } => {
            totals.v_bc = accum(totals.v_bc, Some(*v_bc));
            totals.v_icms = accum(totals.v_icms, Some(*v_icms));
            totals.v_fcp = accum(totals.v_fcp, *v_fcp);
            let fields = filter_fields(vec![
                Some(TaxField::new("orig", orig.as_str())),
                Some(TaxField::new("CST", "00")),
                Some(TaxField::new("modBC", mod_bc.as_str())),
                Some(TaxField::new("vBC", fc2(Some(*v_bc)).unwrap())),
                Some(TaxField::new("pICMS", fc4(Some(*p_icms)).unwrap())),
                Some(TaxField::new("vICMS", fc2(Some(*v_icms)).unwrap())),
                optional_field("pFCP", fc4(*p_fcp).as_deref()),
                optional_field("vFCP", fc2(*v_fcp).as_deref()),
            ]);
            Ok(("ICMS00".to_string(), fields))
        }

        IcmsCst::Cst02 {
            orig,
            q_bc_mono,
            ad_rem_icms,
            v_icms_mono,
        } => {
            totals.q_bc_mono = accum_raw(totals.q_bc_mono, *q_bc_mono);
            totals.v_icms_mono = accum(totals.v_icms_mono, Some(*v_icms_mono));
            let fields = filter_fields(vec![
                Some(TaxField::new("orig", orig.as_str())),
                Some(TaxField::new("CST", "02")),
                optional_field("qBCMono", fc4_raw(*q_bc_mono).as_deref()),
                Some(TaxField::new("adRemICMS", fc4(Some(*ad_rem_icms)).unwrap())),
                Some(TaxField::new("vICMSMono", fc2(Some(*v_icms_mono)).unwrap())),
            ]);
            Ok(("ICMS02".to_string(), fields))
        }

        IcmsCst::Cst10 {
            orig,
            mod_bc,
            v_bc,
            p_icms,
            v_icms,
            v_bc_fcp,
            p_fcp,
            v_fcp,
            mod_bc_st,
            p_mva_st,
            p_red_bc_st,
            v_bc_st,
            p_icms_st,
            v_icms_st,
            v_bc_fcp_st,
            p_fcp_st,
            v_fcp_st,
            v_icms_st_deson,
            mot_des_icms_st,
        } => {
            totals.v_bc = accum(totals.v_bc, Some(*v_bc));
            totals.v_icms = accum(totals.v_icms, Some(*v_icms));
            totals.v_bc_st = accum(totals.v_bc_st, Some(*v_bc_st));
            totals.v_st = accum(totals.v_st, Some(*v_icms_st));
            totals.v_fcp_st = accum(totals.v_fcp_st, *v_fcp_st);
            totals.v_fcp = accum(totals.v_fcp, *v_fcp);
            let mut fields_opt: Vec<Option<TaxField>> = vec![
                Some(TaxField::new("orig", orig.as_str())),
                Some(TaxField::new("CST", "10")),
                Some(TaxField::new("modBC", mod_bc.as_str())),
                Some(TaxField::new("vBC", fc2(Some(*v_bc)).unwrap())),
                Some(TaxField::new("pICMS", fc4(Some(*p_icms)).unwrap())),
                Some(TaxField::new("vICMS", fc2(Some(*v_icms)).unwrap())),
            ];
            // FCP fields
            fields_opt.push(optional_field("vBCFCP", fc2(*v_bc_fcp).as_deref()));
            fields_opt.push(optional_field("pFCP", fc4(*p_fcp).as_deref()));
            fields_opt.push(optional_field("vFCP", fc2(*v_fcp).as_deref()));
            // ST fields
            fields_opt.push(Some(TaxField::new("modBCST", mod_bc_st.as_str())));
            if let Some(v) = p_mva_st {
                fields_opt.push(Some(TaxField::new("pMVAST", fc4(Some(*v)).unwrap())));
            }
            if let Some(v) = p_red_bc_st {
                fields_opt.push(Some(TaxField::new("pRedBCST", fc4(Some(*v)).unwrap())));
            }
            fields_opt.push(Some(TaxField::new("vBCST", fc2(Some(*v_bc_st)).unwrap())));
            fields_opt.push(Some(TaxField::new(
                "pICMSST",
                fc4(Some(*p_icms_st)).unwrap(),
            )));
            fields_opt.push(Some(TaxField::new(
                "vICMSST",
                fc2(Some(*v_icms_st)).unwrap(),
            )));
            // FCP ST fields
            fields_opt.push(optional_field("vBCFCPST", fc2(*v_bc_fcp_st).as_deref()));
            fields_opt.push(optional_field("pFCPST", fc4(*p_fcp_st).as_deref()));
            fields_opt.push(optional_field("vFCPST", fc2(*v_fcp_st).as_deref()));
            // ST desoneration
            fields_opt.push(optional_field(
                "vICMSSTDeson",
                fc2(*v_icms_st_deson).as_deref(),
            ));
            fields_opt.push(optional_field("motDesICMSST", mot_des_icms_st.as_deref()));
            Ok(("ICMS10".to_string(), filter_fields(fields_opt)))
        }

        IcmsCst::Cst15 {
            orig,
            q_bc_mono,
            ad_rem_icms,
            v_icms_mono,
            q_bc_mono_reten,
            ad_rem_icms_reten,
            v_icms_mono_reten,
            p_red_ad_rem,
            mot_red_ad_rem,
        } => {
            totals.q_bc_mono = accum_raw(totals.q_bc_mono, *q_bc_mono);
            totals.v_icms_mono = accum(totals.v_icms_mono, Some(*v_icms_mono));
            totals.q_bc_mono_reten = accum_raw(totals.q_bc_mono_reten, *q_bc_mono_reten);
            totals.v_icms_mono_reten = accum(totals.v_icms_mono_reten, Some(*v_icms_mono_reten));
            let mut fields = filter_fields(vec![
                Some(TaxField::new("orig", orig.as_str())),
                Some(TaxField::new("CST", "15")),
                optional_field("qBCMono", fc4_raw(*q_bc_mono).as_deref()),
                Some(TaxField::new("adRemICMS", fc4(Some(*ad_rem_icms)).unwrap())),
                Some(TaxField::new("vICMSMono", fc2(Some(*v_icms_mono)).unwrap())),
                optional_field("qBCMonoReten", fc4_raw(*q_bc_mono_reten).as_deref()),
                Some(TaxField::new(
                    "adRemICMSReten",
                    fc4(Some(*ad_rem_icms_reten)).unwrap(),
                )),
                Some(TaxField::new(
                    "vICMSMonoReten",
                    fc2(Some(*v_icms_mono_reten)).unwrap(),
                )),
            ]);
            if p_red_ad_rem.is_some() {
                fields.push(TaxField::new("pRedAdRem", fc4(*p_red_ad_rem).unwrap()));
                fields.push(required_field("motRedAdRem", mot_red_ad_rem.as_deref())?);
            }
            Ok(("ICMS15".to_string(), fields))
        }

        IcmsCst::Cst20 {
            orig,
            mod_bc,
            p_red_bc,
            v_bc,
            p_icms,
            v_icms,
            v_bc_fcp,
            p_fcp,
            v_fcp,
            v_icms_deson,
            mot_des_icms,
            ind_deduz_deson,
        } => {
            totals.v_icms_deson = accum(totals.v_icms_deson, *v_icms_deson);
            if ind_deduz_deson.as_deref() == Some("1") {
                totals.ind_deduz_deson = true;
            }
            totals.v_bc = accum(totals.v_bc, Some(*v_bc));
            totals.v_icms = accum(totals.v_icms, Some(*v_icms));
            totals.v_fcp = accum(totals.v_fcp, *v_fcp);
            let mut fields_opt: Vec<Option<TaxField>> = vec![
                Some(TaxField::new("orig", orig.as_str())),
                Some(TaxField::new("CST", "20")),
                Some(TaxField::new("modBC", mod_bc.as_str())),
                Some(TaxField::new("pRedBC", fc4(Some(*p_red_bc)).unwrap())),
                Some(TaxField::new("vBC", fc2(Some(*v_bc)).unwrap())),
                Some(TaxField::new("pICMS", fc4(Some(*p_icms)).unwrap())),
                Some(TaxField::new("vICMS", fc2(Some(*v_icms)).unwrap())),
            ];
            // FCP fields
            fields_opt.push(optional_field("vBCFCP", fc2(*v_bc_fcp).as_deref()));
            fields_opt.push(optional_field("pFCP", fc4(*p_fcp).as_deref()));
            fields_opt.push(optional_field("vFCP", fc2(*v_fcp).as_deref()));
            // Desoneration
            fields_opt.push(optional_field("vICMSDeson", fc2(*v_icms_deson).as_deref()));
            fields_opt.push(optional_field("motDesICMS", mot_des_icms.as_deref()));
            fields_opt.push(optional_field("indDeduzDeson", ind_deduz_deson.as_deref()));
            Ok(("ICMS20".to_string(), filter_fields(fields_opt)))
        }

        IcmsCst::Cst30 {
            orig,
            mod_bc_st,
            p_mva_st,
            p_red_bc_st,
            v_bc_st,
            p_icms_st,
            v_icms_st,
            v_bc_fcp_st,
            p_fcp_st,
            v_fcp_st,
            v_icms_deson,
            mot_des_icms,
            ind_deduz_deson,
        } => {
            totals.v_icms_deson = accum(totals.v_icms_deson, *v_icms_deson);
            if ind_deduz_deson.as_deref() == Some("1") {
                totals.ind_deduz_deson = true;
            }
            totals.v_bc_st = accum(totals.v_bc_st, Some(*v_bc_st));
            totals.v_st = accum(totals.v_st, Some(*v_icms_st));
            totals.v_fcp_st = accum(totals.v_fcp_st, *v_fcp_st);
            let mut fields_opt: Vec<Option<TaxField>> = vec![
                Some(TaxField::new("orig", orig.as_str())),
                Some(TaxField::new("CST", "30")),
            ];
            // ST fields
            fields_opt.push(Some(TaxField::new("modBCST", mod_bc_st.as_str())));
            if let Some(v) = p_mva_st {
                fields_opt.push(Some(TaxField::new("pMVAST", fc4(Some(*v)).unwrap())));
            }
            if let Some(v) = p_red_bc_st {
                fields_opt.push(Some(TaxField::new("pRedBCST", fc4(Some(*v)).unwrap())));
            }
            fields_opt.push(Some(TaxField::new("vBCST", fc2(Some(*v_bc_st)).unwrap())));
            fields_opt.push(Some(TaxField::new(
                "pICMSST",
                fc4(Some(*p_icms_st)).unwrap(),
            )));
            fields_opt.push(Some(TaxField::new(
                "vICMSST",
                fc2(Some(*v_icms_st)).unwrap(),
            )));
            // FCP ST
            fields_opt.push(optional_field("vBCFCPST", fc2(*v_bc_fcp_st).as_deref()));
            fields_opt.push(optional_field("pFCPST", fc4(*p_fcp_st).as_deref()));
            fields_opt.push(optional_field("vFCPST", fc2(*v_fcp_st).as_deref()));
            // Desoneration
            fields_opt.push(optional_field("vICMSDeson", fc2(*v_icms_deson).as_deref()));
            fields_opt.push(optional_field("motDesICMS", mot_des_icms.as_deref()));
            fields_opt.push(optional_field("indDeduzDeson", ind_deduz_deson.as_deref()));
            Ok(("ICMS30".to_string(), filter_fields(fields_opt)))
        }

        IcmsCst::Cst40 {
            orig,
            v_icms_deson,
            mot_des_icms,
            ind_deduz_deson,
        }
        | IcmsCst::Cst41 {
            orig,
            v_icms_deson,
            mot_des_icms,
            ind_deduz_deson,
        }
        | IcmsCst::Cst50 {
            orig,
            v_icms_deson,
            mot_des_icms,
            ind_deduz_deson,
        } => {
            totals.v_icms_deson = accum(totals.v_icms_deson, *v_icms_deson);
            if ind_deduz_deson.as_deref() == Some("1") {
                totals.ind_deduz_deson = true;
            }
            let mut fields_opt: Vec<Option<TaxField>> = vec![
                Some(TaxField::new("orig", orig.as_str())),
                Some(TaxField::new("CST", cst.cst_code())),
            ];
            // Desoneration
            fields_opt.push(optional_field("vICMSDeson", fc2(*v_icms_deson).as_deref()));
            fields_opt.push(optional_field("motDesICMS", mot_des_icms.as_deref()));
            fields_opt.push(optional_field("indDeduzDeson", ind_deduz_deson.as_deref()));
            Ok(("ICMS40".to_string(), filter_fields(fields_opt)))
        }

        IcmsCst::Cst51 {
            orig,
            mod_bc,
            p_red_bc,
            c_benef_rbc,
            v_bc,
            p_icms,
            v_icms_op,
            p_dif,
            v_icms_dif,
            v_icms,
            v_bc_fcp,
            p_fcp,
            v_fcp,
            p_fcp_dif,
            v_fcp_dif,
            v_fcp_efet,
        } => {
            totals.v_bc = accum(totals.v_bc, *v_bc);
            totals.v_icms = accum(totals.v_icms, *v_icms);
            totals.v_fcp = accum(totals.v_fcp, *v_fcp);
            let fields_opt: Vec<Option<TaxField>> = vec![
                Some(TaxField::new("orig", orig.as_str())),
                Some(TaxField::new("CST", "51")),
                optional_field("modBC", mod_bc.as_deref()),
                optional_field("pRedBC", fc4(*p_red_bc).as_deref()),
                optional_field("cBenefRBC", c_benef_rbc.as_deref()),
                optional_field("vBC", fc2(*v_bc).as_deref()),
                optional_field("pICMS", fc4(*p_icms).as_deref()),
                optional_field("vICMSOp", fc2(*v_icms_op).as_deref()),
                optional_field("pDif", fc4(*p_dif).as_deref()),
                optional_field("vICMSDif", fc2(*v_icms_dif).as_deref()),
                optional_field("vICMS", fc2(*v_icms).as_deref()),
                optional_field("vBCFCP", fc2(*v_bc_fcp).as_deref()),
                optional_field("pFCP", fc4(*p_fcp).as_deref()),
                optional_field("vFCP", fc2(*v_fcp).as_deref()),
                optional_field("pFCPDif", fc4(*p_fcp_dif).as_deref()),
                optional_field("vFCPDif", fc2(*v_fcp_dif).as_deref()),
                optional_field("vFCPEfet", fc2(*v_fcp_efet).as_deref()),
            ];
            Ok(("ICMS51".to_string(), filter_fields(fields_opt)))
        }

        IcmsCst::Cst53 {
            orig,
            q_bc_mono,
            ad_rem_icms,
            v_icms_mono_op,
            p_dif,
            v_icms_mono_dif,
            v_icms_mono,
        } => {
            totals.q_bc_mono = accum_raw(totals.q_bc_mono, *q_bc_mono);
            totals.v_icms_mono = accum(totals.v_icms_mono, *v_icms_mono);
            totals.q_bc_mono_reten = accum_raw(totals.q_bc_mono_reten, None);
            totals.v_icms_mono_reten = accum(totals.v_icms_mono_reten, None);
            let fields_opt: Vec<Option<TaxField>> = vec![
                Some(TaxField::new("orig", orig.as_str())),
                Some(TaxField::new("CST", "53")),
                optional_field("qBCMono", fc4_raw(*q_bc_mono).as_deref()),
                optional_field("adRemICMS", fc4(*ad_rem_icms).as_deref()),
                optional_field("vICMSMonoOp", fc2(*v_icms_mono_op).as_deref()),
                optional_field("pDif", fc4(*p_dif).as_deref()),
                optional_field("vICMSMonoDif", fc2(*v_icms_mono_dif).as_deref()),
                optional_field("vICMSMono", fc2(*v_icms_mono).as_deref()),
            ];
            Ok(("ICMS53".to_string(), filter_fields(fields_opt)))
        }

        IcmsCst::Cst60 {
            orig,
            v_bc_st_ret,
            p_st,
            v_icms_substituto,
            v_icms_st_ret,
            v_bc_fcp_st_ret,
            p_fcp_st_ret,
            v_fcp_st_ret,
            p_red_bc_efet,
            v_bc_efet,
            p_icms_efet,
            v_icms_efet,
        } => {
            totals.v_fcp_st_ret = accum(totals.v_fcp_st_ret, *v_fcp_st_ret);
            let fields_opt: Vec<Option<TaxField>> = vec![
                Some(TaxField::new("orig", orig.as_str())),
                Some(TaxField::new("CST", "60")),
                optional_field("vBCSTRet", fc2(*v_bc_st_ret).as_deref()),
                optional_field("pST", fc4(*p_st).as_deref()),
                optional_field("vICMSSubstituto", fc2(*v_icms_substituto).as_deref()),
                optional_field("vICMSSTRet", fc2(*v_icms_st_ret).as_deref()),
                optional_field("vBCFCPSTRet", fc2(*v_bc_fcp_st_ret).as_deref()),
                optional_field("pFCPSTRet", fc4(*p_fcp_st_ret).as_deref()),
                optional_field("vFCPSTRet", fc2(*v_fcp_st_ret).as_deref()),
                optional_field("pRedBCEfet", fc4(*p_red_bc_efet).as_deref()),
                optional_field("vBCEfet", fc2(*v_bc_efet).as_deref()),
                optional_field("pICMSEfet", fc4(*p_icms_efet).as_deref()),
                optional_field("vICMSEfet", fc2(*v_icms_efet).as_deref()),
            ];
            Ok(("ICMS60".to_string(), filter_fields(fields_opt)))
        }

        IcmsCst::Cst61 {
            orig,
            q_bc_mono_ret,
            ad_rem_icms_ret,
            v_icms_mono_ret,
        } => {
            totals.q_bc_mono_ret = accum_raw(totals.q_bc_mono_ret, *q_bc_mono_ret);
            totals.v_icms_mono_ret = accum(totals.v_icms_mono_ret, Some(*v_icms_mono_ret));
            let fields = filter_fields(vec![
                Some(TaxField::new("orig", orig.as_str())),
                Some(TaxField::new("CST", "61")),
                optional_field("qBCMonoRet", fc4_raw(*q_bc_mono_ret).as_deref()),
                Some(TaxField::new(
                    "adRemICMSRet",
                    fc4(Some(*ad_rem_icms_ret)).unwrap(),
                )),
                Some(TaxField::new(
                    "vICMSMonoRet",
                    fc2(Some(*v_icms_mono_ret)).unwrap(),
                )),
            ]);
            Ok(("ICMS61".to_string(), fields))
        }

        IcmsCst::Cst70 {
            orig,
            mod_bc,
            p_red_bc,
            v_bc,
            p_icms,
            v_icms,
            v_bc_fcp,
            p_fcp,
            v_fcp,
            mod_bc_st,
            p_mva_st,
            p_red_bc_st,
            v_bc_st,
            p_icms_st,
            v_icms_st,
            v_bc_fcp_st,
            p_fcp_st,
            v_fcp_st,
            v_icms_deson,
            mot_des_icms,
            ind_deduz_deson,
            v_icms_st_deson,
            mot_des_icms_st,
        } => {
            totals.v_icms_deson = accum(totals.v_icms_deson, *v_icms_deson);
            if ind_deduz_deson.as_deref() == Some("1") {
                totals.ind_deduz_deson = true;
            }
            totals.v_bc = accum(totals.v_bc, Some(*v_bc));
            totals.v_icms = accum(totals.v_icms, Some(*v_icms));
            totals.v_bc_st = accum(totals.v_bc_st, Some(*v_bc_st));
            totals.v_st = accum(totals.v_st, Some(*v_icms_st));
            totals.v_fcp_st = accum(totals.v_fcp_st, *v_fcp_st);
            totals.v_fcp = accum(totals.v_fcp, *v_fcp);
            let mut fields_opt: Vec<Option<TaxField>> = vec![
                Some(TaxField::new("orig", orig.as_str())),
                Some(TaxField::new("CST", "70")),
                Some(TaxField::new("modBC", mod_bc.as_str())),
                Some(TaxField::new("pRedBC", fc4(Some(*p_red_bc)).unwrap())),
                Some(TaxField::new("vBC", fc2(Some(*v_bc)).unwrap())),
                Some(TaxField::new("pICMS", fc4(Some(*p_icms)).unwrap())),
                Some(TaxField::new("vICMS", fc2(Some(*v_icms)).unwrap())),
            ];
            // FCP
            fields_opt.push(optional_field("vBCFCP", fc2(*v_bc_fcp).as_deref()));
            fields_opt.push(optional_field("pFCP", fc4(*p_fcp).as_deref()));
            fields_opt.push(optional_field("vFCP", fc2(*v_fcp).as_deref()));
            // ST
            fields_opt.push(Some(TaxField::new("modBCST", mod_bc_st.as_str())));
            if let Some(v) = p_mva_st {
                fields_opt.push(Some(TaxField::new("pMVAST", fc4(Some(*v)).unwrap())));
            }
            if let Some(v) = p_red_bc_st {
                fields_opt.push(Some(TaxField::new("pRedBCST", fc4(Some(*v)).unwrap())));
            }
            fields_opt.push(Some(TaxField::new("vBCST", fc2(Some(*v_bc_st)).unwrap())));
            fields_opt.push(Some(TaxField::new(
                "pICMSST",
                fc4(Some(*p_icms_st)).unwrap(),
            )));
            fields_opt.push(Some(TaxField::new(
                "vICMSST",
                fc2(Some(*v_icms_st)).unwrap(),
            )));
            // FCP ST
            fields_opt.push(optional_field("vBCFCPST", fc2(*v_bc_fcp_st).as_deref()));
            fields_opt.push(optional_field("pFCPST", fc4(*p_fcp_st).as_deref()));
            fields_opt.push(optional_field("vFCPST", fc2(*v_fcp_st).as_deref()));
            // Desoneration
            fields_opt.push(optional_field("vICMSDeson", fc2(*v_icms_deson).as_deref()));
            fields_opt.push(optional_field("motDesICMS", mot_des_icms.as_deref()));
            fields_opt.push(optional_field("indDeduzDeson", ind_deduz_deson.as_deref()));
            // ST desoneration
            fields_opt.push(optional_field(
                "vICMSSTDeson",
                fc2(*v_icms_st_deson).as_deref(),
            ));
            fields_opt.push(optional_field("motDesICMSST", mot_des_icms_st.as_deref()));
            Ok(("ICMS70".to_string(), filter_fields(fields_opt)))
        }

        IcmsCst::Cst90 {
            orig,
            mod_bc,
            v_bc,
            p_red_bc,
            c_benef_rbc,
            p_icms,
            v_icms_op,
            p_dif,
            v_icms_dif,
            v_icms,
            v_bc_fcp,
            p_fcp,
            v_fcp,
            p_fcp_dif,
            v_fcp_dif,
            v_fcp_efet,
            mod_bc_st,
            p_mva_st,
            p_red_bc_st,
            v_bc_st,
            p_icms_st,
            v_icms_st,
            v_bc_fcp_st,
            p_fcp_st,
            v_fcp_st,
            v_icms_deson,
            mot_des_icms,
            ind_deduz_deson,
            v_icms_st_deson,
            mot_des_icms_st,
        } => {
            totals.v_icms_deson = accum(totals.v_icms_deson, *v_icms_deson);
            if ind_deduz_deson.as_deref() == Some("1") {
                totals.ind_deduz_deson = true;
            }
            totals.v_bc = accum(totals.v_bc, *v_bc);
            totals.v_icms = accum(totals.v_icms, *v_icms);
            totals.v_bc_st = accum(totals.v_bc_st, *v_bc_st);
            totals.v_st = accum(totals.v_st, *v_icms_st);
            totals.v_fcp_st = accum(totals.v_fcp_st, *v_fcp_st);
            totals.v_fcp = accum(totals.v_fcp, *v_fcp);
            let mut fields_opt: Vec<Option<TaxField>> = vec![
                Some(TaxField::new("orig", orig.as_str())),
                Some(TaxField::new("CST", "90")),
                optional_field("modBC", mod_bc.as_deref()),
                optional_field("vBC", fc2(*v_bc).as_deref()),
                optional_field("pRedBC", fc4(*p_red_bc).as_deref()),
                optional_field("cBenefRBC", c_benef_rbc.as_deref()),
                optional_field("pICMS", fc4(*p_icms).as_deref()),
                optional_field("vICMSOp", fc2(*v_icms_op).as_deref()),
                optional_field("pDif", fc4(*p_dif).as_deref()),
                optional_field("vICMSDif", fc2(*v_icms_dif).as_deref()),
                optional_field("vICMS", fc2(*v_icms).as_deref()),
            ];
            // FCP
            fields_opt.push(optional_field("vBCFCP", fc2(*v_bc_fcp).as_deref()));
            fields_opt.push(optional_field("pFCP", fc4(*p_fcp).as_deref()));
            fields_opt.push(optional_field("vFCP", fc2(*v_fcp).as_deref()));
            // FCP deferral
            fields_opt.push(optional_field("pFCPDif", fc4(*p_fcp_dif).as_deref()));
            fields_opt.push(optional_field("vFCPDif", fc2(*v_fcp_dif).as_deref()));
            fields_opt.push(optional_field("vFCPEfet", fc2(*v_fcp_efet).as_deref()));
            // ST (all optional for CST 90)
            fields_opt.push(optional_field("modBCST", mod_bc_st.as_deref()));
            fields_opt.push(optional_field("pMVAST", fc4(*p_mva_st).as_deref()));
            fields_opt.push(optional_field("pRedBCST", fc4(*p_red_bc_st).as_deref()));
            fields_opt.push(optional_field("vBCST", fc2(*v_bc_st).as_deref()));
            fields_opt.push(optional_field("pICMSST", fc4(*p_icms_st).as_deref()));
            fields_opt.push(optional_field("vICMSST", fc2(*v_icms_st).as_deref()));
            // FCP ST
            fields_opt.push(optional_field("vBCFCPST", fc2(*v_bc_fcp_st).as_deref()));
            fields_opt.push(optional_field("pFCPST", fc4(*p_fcp_st).as_deref()));
            fields_opt.push(optional_field("vFCPST", fc2(*v_fcp_st).as_deref()));
            // Desoneration
            fields_opt.push(optional_field("vICMSDeson", fc2(*v_icms_deson).as_deref()));
            fields_opt.push(optional_field("motDesICMS", mot_des_icms.as_deref()));
            fields_opt.push(optional_field("indDeduzDeson", ind_deduz_deson.as_deref()));
            // ST desoneration
            fields_opt.push(optional_field(
                "vICMSSTDeson",
                fc2(*v_icms_st_deson).as_deref(),
            ));
            fields_opt.push(optional_field("motDesICMSST", mot_des_icms_st.as_deref()));
            Ok(("ICMS90".to_string(), filter_fields(fields_opt)))
        }
    }
}

// ── IcmsCsosn enum (Simples Nacional) ────────────────────────────────────────

/// ICMS CSOSN variant for Simples Nacional tax regime (CRT 1/2).
///
/// Each variant carries **only** the fields that are valid for that CSOSN,
/// giving compile-time safety instead of runtime string matching against a
/// flat struct full of `Option`s.
///
/// Normal regime CSTs use [`IcmsCst`] instead.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum IcmsCsosn {
    /// CSOSN 101 — Tributada pelo Simples Nacional com permissao de credito.
    Csosn101 {
        /// Product origin code (`orig`).
        orig: String,
        /// CSOSN code, always `"101"`.
        csosn: String,
        /// Simples Nacional credit rate (`pCredSN`).
        p_cred_sn: Rate,
        /// Simples Nacional credit value (`vCredICMSSN`).
        v_cred_icms_sn: Cents,
    },
    /// CSOSN 102 — Tributada pelo Simples Nacional sem permissão de crédito.
    Csosn102 {
        /// Product origin code (`orig`). May be empty when CRT=4.
        orig: String,
        /// CSOSN code, always `"102"`.
        csosn: String,
    },
    /// CSOSN 103 — Isenção do ICMS no Simples Nacional para faixa de receita
    /// bruta.
    Csosn103 {
        /// Product origin code (`orig`).
        orig: String,
        /// CSOSN code, always `"103"`.
        csosn: String,
    },
    /// CSOSN 300 — Imune.
    Csosn300 {
        /// Product origin code (`orig`). May be empty.
        orig: String,
        /// CSOSN code, always `"300"`.
        csosn: String,
    },
    /// CSOSN 400 — Não tributada pelo Simples Nacional.
    Csosn400 {
        /// Product origin code (`orig`). May be empty.
        orig: String,
        /// CSOSN code, always `"400"`.
        csosn: String,
    },
    /// CSOSN 201 — Tributada com permissao de credito e com cobranca do ICMS
    /// por ST.
    Csosn201 {
        /// Product origin code (`orig`).
        orig: String,
        /// CSOSN code, always `"201"`.
        csosn: String,
        /// ST base calculation modality (`modBCST`).
        mod_bc_st: String,
        /// ST added value margin (`pMVAST`). Optional.
        p_mva_st: Option<Rate>,
        /// ST base reduction rate (`pRedBCST`). Optional.
        p_red_bc_st: Option<Rate>,
        /// ST calculation base value (`vBCST`).
        v_bc_st: Cents,
        /// ST rate (`pICMSST`).
        p_icms_st: Rate,
        /// ST ICMS value (`vICMSST`).
        v_icms_st: Cents,
        /// FCP-ST calculation base (`vBCFCPST`). Optional.
        v_bc_fcp_st: Option<Cents>,
        /// FCP-ST rate (`pFCPST`). Optional.
        p_fcp_st: Option<Rate>,
        /// FCP-ST value (`vFCPST`). Optional.
        v_fcp_st: Option<Cents>,
        /// Simples Nacional credit rate (`pCredSN`). Optional.
        p_cred_sn: Option<Rate>,
        /// Simples Nacional credit value (`vCredICMSSN`). Optional.
        v_cred_icms_sn: Option<Cents>,
    },
    /// CSOSN 202 — Tributada sem permissão de crédito e com cobrança do
    /// ICMS por ST.
    Csosn202 {
        /// Product origin code (`orig`).
        orig: String,
        /// CSOSN code, always `"202"`.
        csosn: String,
        /// ST base calculation modality (`modBCST`).
        mod_bc_st: String,
        /// ST added value margin (`pMVAST`). Optional.
        p_mva_st: Option<Rate>,
        /// ST base reduction rate (`pRedBCST`). Optional.
        p_red_bc_st: Option<Rate>,
        /// ST calculation base value (`vBCST`).
        v_bc_st: Cents,
        /// ST rate (`pICMSST`).
        p_icms_st: Rate,
        /// ST ICMS value (`vICMSST`).
        v_icms_st: Cents,
        /// FCP-ST calculation base (`vBCFCPST`). Optional.
        v_bc_fcp_st: Option<Cents>,
        /// FCP-ST rate (`pFCPST`). Optional.
        p_fcp_st: Option<Rate>,
        /// FCP-ST value (`vFCPST`). Optional.
        v_fcp_st: Option<Cents>,
    },
    /// CSOSN 203 — Isenção do ICMS no Simples Nacional para faixa de receita
    /// bruta e com cobrança do ICMS por ST.
    Csosn203 {
        /// Product origin code (`orig`).
        orig: String,
        /// CSOSN code, always `"203"`.
        csosn: String,
        /// ST base calculation modality (`modBCST`).
        mod_bc_st: String,
        /// ST added value margin (`pMVAST`). Optional.
        p_mva_st: Option<Rate>,
        /// ST base reduction rate (`pRedBCST`). Optional.
        p_red_bc_st: Option<Rate>,
        /// ST calculation base value (`vBCST`).
        v_bc_st: Cents,
        /// ST rate (`pICMSST`).
        p_icms_st: Rate,
        /// ST ICMS value (`vICMSST`).
        v_icms_st: Cents,
        /// FCP-ST calculation base (`vBCFCPST`). Optional.
        v_bc_fcp_st: Option<Cents>,
        /// FCP-ST rate (`pFCPST`). Optional.
        p_fcp_st: Option<Rate>,
        /// FCP-ST value (`vFCPST`). Optional.
        v_fcp_st: Option<Cents>,
    },
    /// CSOSN 500 — ICMS cobrado anteriormente por ST ou por antecipacao.
    Csosn500 {
        /// Product origin code (`orig`).
        orig: String,
        /// CSOSN code, always `"500"`.
        csosn: String,
        /// ST retained calculation base (`vBCSTRet`). Optional.
        v_bc_st_ret: Option<Cents>,
        /// ST rate at retention (`pST`). Optional.
        p_st: Option<Rate>,
        /// ICMS value paid by the substitutor (`vICMSSubstituto`). Optional.
        v_icms_substituto: Option<Cents>,
        /// Retained ST ICMS value (`vICMSSTRet`). Optional.
        v_icms_st_ret: Option<Cents>,
        /// FCP-ST retained calculation base (`vBCFCPSTRet`). Optional.
        v_bc_fcp_st_ret: Option<Cents>,
        /// FCP-ST retained rate (`pFCPSTRet`). Optional.
        p_fcp_st_ret: Option<Rate>,
        /// FCP-ST retained value (`vFCPSTRet`). Optional.
        v_fcp_st_ret: Option<Cents>,
        /// Effective base reduction rate (`pRedBCEfet`). Optional.
        p_red_bc_efet: Option<Rate>,
        /// Effective calculation base (`vBCEfet`). Optional.
        v_bc_efet: Option<Cents>,
        /// Effective ICMS rate (`pICMSEfet`). Optional.
        p_icms_efet: Option<Rate>,
        /// Effective ICMS value (`vICMSEfet`). Optional.
        v_icms_efet: Option<Cents>,
    },
    /// CSOSN 900 — Outros.
    Csosn900 {
        /// Product origin code (`orig`). May be empty.
        orig: String,
        /// CSOSN code, always `"900"`.
        csosn: String,
        /// Base calculation modality (`modBC`). Optional.
        mod_bc: Option<String>,
        /// ICMS calculation base value (`vBC`). Optional.
        v_bc: Option<Cents>,
        /// Base reduction rate (`pRedBC`). Optional.
        p_red_bc: Option<Rate>,
        /// ICMS rate (`pICMS`). Optional.
        p_icms: Option<Rate>,
        /// ICMS value (`vICMS`). Optional.
        v_icms: Option<Cents>,
        /// ST base calculation modality (`modBCST`). Optional.
        mod_bc_st: Option<String>,
        /// ST added value margin (`pMVAST`). Optional.
        p_mva_st: Option<Rate>,
        /// ST base reduction rate (`pRedBCST`). Optional.
        p_red_bc_st: Option<Rate>,
        /// ST calculation base value (`vBCST`). Optional.
        v_bc_st: Option<Cents>,
        /// ST rate (`pICMSST`). Optional.
        p_icms_st: Option<Rate>,
        /// ST ICMS value (`vICMSST`). Optional.
        v_icms_st: Option<Cents>,
        /// FCP-ST calculation base (`vBCFCPST`). Optional.
        v_bc_fcp_st: Option<Cents>,
        /// FCP-ST rate (`pFCPST`). Optional.
        p_fcp_st: Option<Rate>,
        /// FCP-ST value (`vFCPST`). Optional.
        v_fcp_st: Option<Cents>,
        /// Simples Nacional credit rate (`pCredSN`). Optional.
        p_cred_sn: Option<Rate>,
        /// Simples Nacional credit value (`vCredICMSSN`). Optional.
        v_cred_icms_sn: Option<Cents>,
    },
}

impl IcmsCsosn {
    /// Return the CSOSN code string for this variant (e.g. "101", "202").
    pub fn csosn_code(&self) -> &str {
        match self {
            Self::Csosn101 { csosn, .. } => csosn.as_str(),
            Self::Csosn102 { csosn, .. } => csosn.as_str(),
            Self::Csosn103 { csosn, .. } => csosn.as_str(),
            Self::Csosn201 { csosn, .. } => csosn.as_str(),
            Self::Csosn202 { csosn, .. } => csosn.as_str(),
            Self::Csosn203 { csosn, .. } => csosn.as_str(),
            Self::Csosn300 { csosn, .. } => csosn.as_str(),
            Self::Csosn400 { csosn, .. } => csosn.as_str(),
            Self::Csosn500 { csosn, .. } => csosn.as_str(),
            Self::Csosn900 { csosn, .. } => csosn.as_str(),
        }
    }
}

/// Build the ICMS XML fragment and accumulate totals from a typed
/// [`IcmsCsosn`] variant.
///
/// This is the compile-time-safe counterpart of the original
/// [`build_icms_xml`] code path for Simples Nacional CSOSNs. It can be used
/// directly by new code that already has an `IcmsCsosn`, or indirectly via
/// the unchanged [`build_icms_xml`] public API (which converts internally).
///
/// # Errors
///
/// Returns [`FiscalError`] if XML field serialization fails (should not happen
/// when the enum is correctly constructed).
pub fn build_icms_csosn_xml(
    csosn: &IcmsCsosn,
    totals: &mut IcmsTotals,
) -> Result<(String, Vec<TaxField>), FiscalError> {
    match csosn {
        IcmsCsosn::Csosn101 {
            orig,
            csosn,
            p_cred_sn,
            v_cred_icms_sn,
        } => {
            let fields = filter_fields(vec![
                Some(TaxField::new("orig", orig.as_str())),
                Some(TaxField::new("CSOSN", csosn.as_str())),
                Some(TaxField::new("pCredSN", fc4(Some(*p_cred_sn)).unwrap())),
                Some(TaxField::new(
                    "vCredICMSSN",
                    fc2(Some(*v_cred_icms_sn)).unwrap(),
                )),
            ]);
            Ok(("ICMSSN101".to_string(), fields))
        }

        IcmsCsosn::Csosn102 { orig, csosn }
        | IcmsCsosn::Csosn103 { orig, csosn }
        | IcmsCsosn::Csosn300 { orig, csosn }
        | IcmsCsosn::Csosn400 { orig, csosn } => {
            let orig_val = if orig.is_empty() {
                None
            } else {
                Some(orig.as_str())
            };
            let fields = filter_fields(vec![
                optional_field("orig", orig_val),
                Some(TaxField::new("CSOSN", csosn.as_str())),
            ]);
            Ok(("ICMSSN102".to_string(), fields))
        }

        IcmsCsosn::Csosn201 {
            orig,
            csosn,
            mod_bc_st,
            p_mva_st,
            p_red_bc_st,
            v_bc_st,
            p_icms_st,
            v_icms_st,
            v_bc_fcp_st,
            p_fcp_st,
            v_fcp_st,
            p_cred_sn,
            v_cred_icms_sn,
        } => {
            totals.v_bc_st = accum(totals.v_bc_st, Some(*v_bc_st));
            totals.v_st = accum(totals.v_st, Some(*v_icms_st));

            let mut fields_opt: Vec<Option<TaxField>> = vec![
                Some(TaxField::new("orig", orig.as_str())),
                Some(TaxField::new("CSOSN", csosn.as_str())),
            ];
            // ST fields
            fields_opt.push(Some(TaxField::new("modBCST", mod_bc_st.as_str())));
            if let Some(v) = p_mva_st {
                fields_opt.push(Some(TaxField::new("pMVAST", fc4(Some(*v)).unwrap())));
            }
            if let Some(v) = p_red_bc_st {
                fields_opt.push(Some(TaxField::new("pRedBCST", fc4(Some(*v)).unwrap())));
            }
            fields_opt.push(Some(TaxField::new("vBCST", fc2(Some(*v_bc_st)).unwrap())));
            fields_opt.push(Some(TaxField::new(
                "pICMSST",
                fc4(Some(*p_icms_st)).unwrap(),
            )));
            fields_opt.push(Some(TaxField::new(
                "vICMSST",
                fc2(Some(*v_icms_st)).unwrap(),
            )));
            // FCP ST fields
            fields_opt.push(optional_field("vBCFCPST", fc2(*v_bc_fcp_st).as_deref()));
            fields_opt.push(optional_field("pFCPST", fc4(*p_fcp_st).as_deref()));
            fields_opt.push(optional_field("vFCPST", fc2(*v_fcp_st).as_deref()));
            // SN credit fields
            fields_opt.push(optional_field("pCredSN", fc4(*p_cred_sn).as_deref()));
            fields_opt.push(optional_field(
                "vCredICMSSN",
                fc2(*v_cred_icms_sn).as_deref(),
            ));
            Ok(("ICMSSN201".to_string(), filter_fields(fields_opt)))
        }

        IcmsCsosn::Csosn202 {
            orig,
            csosn,
            mod_bc_st,
            p_mva_st,
            p_red_bc_st,
            v_bc_st,
            p_icms_st,
            v_icms_st,
            v_bc_fcp_st,
            p_fcp_st,
            v_fcp_st,
        }
        | IcmsCsosn::Csosn203 {
            orig,
            csosn,
            mod_bc_st,
            p_mva_st,
            p_red_bc_st,
            v_bc_st,
            p_icms_st,
            v_icms_st,
            v_bc_fcp_st,
            p_fcp_st,
            v_fcp_st,
        } => {
            totals.v_bc_st = accum(totals.v_bc_st, Some(*v_bc_st));
            totals.v_st = accum(totals.v_st, Some(*v_icms_st));

            let mut fields_opt: Vec<Option<TaxField>> = vec![
                Some(TaxField::new("orig", orig.as_str())),
                Some(TaxField::new("CSOSN", csosn.as_str())),
            ];
            // ST fields
            fields_opt.push(Some(TaxField::new("modBCST", mod_bc_st.as_str())));
            if let Some(v) = p_mva_st {
                fields_opt.push(Some(TaxField::new("pMVAST", fc4(Some(*v)).unwrap())));
            }
            if let Some(v) = p_red_bc_st {
                fields_opt.push(Some(TaxField::new("pRedBCST", fc4(Some(*v)).unwrap())));
            }
            fields_opt.push(Some(TaxField::new("vBCST", fc2(Some(*v_bc_st)).unwrap())));
            fields_opt.push(Some(TaxField::new(
                "pICMSST",
                fc4(Some(*p_icms_st)).unwrap(),
            )));
            fields_opt.push(Some(TaxField::new(
                "vICMSST",
                fc2(Some(*v_icms_st)).unwrap(),
            )));
            // FCP ST fields
            fields_opt.push(optional_field("vBCFCPST", fc2(*v_bc_fcp_st).as_deref()));
            fields_opt.push(optional_field("pFCPST", fc4(*p_fcp_st).as_deref()));
            fields_opt.push(optional_field("vFCPST", fc2(*v_fcp_st).as_deref()));
            Ok(("ICMSSN202".to_string(), filter_fields(fields_opt)))
        }

        IcmsCsosn::Csosn500 {
            orig,
            csosn,
            v_bc_st_ret,
            p_st,
            v_icms_substituto,
            v_icms_st_ret,
            v_bc_fcp_st_ret,
            p_fcp_st_ret,
            v_fcp_st_ret,
            p_red_bc_efet,
            v_bc_efet,
            p_icms_efet,
            v_icms_efet,
        } => {
            let fields = filter_fields(vec![
                Some(TaxField::new("orig", orig.as_str())),
                Some(TaxField::new("CSOSN", csosn.as_str())),
                optional_field("vBCSTRet", fc2(*v_bc_st_ret).as_deref()),
                optional_field("pST", fc4(*p_st).as_deref()),
                optional_field("vICMSSubstituto", fc2(*v_icms_substituto).as_deref()),
                optional_field("vICMSSTRet", fc2(*v_icms_st_ret).as_deref()),
                optional_field("vBCFCPSTRet", fc2(*v_bc_fcp_st_ret).as_deref()),
                optional_field("pFCPSTRet", fc4(*p_fcp_st_ret).as_deref()),
                optional_field("vFCPSTRet", fc2(*v_fcp_st_ret).as_deref()),
                optional_field("pRedBCEfet", fc4(*p_red_bc_efet).as_deref()),
                optional_field("vBCEfet", fc2(*v_bc_efet).as_deref()),
                optional_field("pICMSEfet", fc4(*p_icms_efet).as_deref()),
                optional_field("vICMSEfet", fc2(*v_icms_efet).as_deref()),
            ]);
            Ok(("ICMSSN500".to_string(), fields))
        }

        IcmsCsosn::Csosn900 {
            orig,
            csosn,
            mod_bc,
            v_bc,
            p_red_bc,
            p_icms,
            v_icms,
            mod_bc_st,
            p_mva_st,
            p_red_bc_st,
            v_bc_st,
            p_icms_st,
            v_icms_st,
            v_bc_fcp_st,
            p_fcp_st,
            v_fcp_st,
            p_cred_sn,
            v_cred_icms_sn,
        } => {
            totals.v_bc = accum(totals.v_bc, *v_bc);
            totals.v_icms = accum(totals.v_icms, *v_icms);
            totals.v_bc_st = accum(totals.v_bc_st, *v_bc_st);
            totals.v_st = accum(totals.v_st, *v_icms_st);

            let orig_val = if orig.is_empty() {
                None
            } else {
                Some(orig.as_str())
            };
            let mut fields_opt: Vec<Option<TaxField>> = vec![
                optional_field("orig", orig_val),
                Some(TaxField::new("CSOSN", csosn.as_str())),
                optional_field("modBC", mod_bc.as_deref()),
                optional_field("vBC", fc2(*v_bc).as_deref()),
                optional_field("pRedBC", fc4(*p_red_bc).as_deref()),
                optional_field("pICMS", fc4(*p_icms).as_deref()),
                optional_field("vICMS", fc2(*v_icms).as_deref()),
                // ST fields are all optional for CSOSN 900
                optional_field("modBCST", mod_bc_st.as_deref()),
                optional_field("pMVAST", fc4(*p_mva_st).as_deref()),
                optional_field("pRedBCST", fc4(*p_red_bc_st).as_deref()),
                optional_field("vBCST", fc2(*v_bc_st).as_deref()),
                optional_field("pICMSST", fc4(*p_icms_st).as_deref()),
                optional_field("vICMSST", fc2(*v_icms_st).as_deref()),
            ];
            // FCP ST fields
            fields_opt.push(optional_field("vBCFCPST", fc2(*v_bc_fcp_st).as_deref()));
            fields_opt.push(optional_field("pFCPST", fc4(*p_fcp_st).as_deref()));
            fields_opt.push(optional_field("vFCPST", fc2(*v_fcp_st).as_deref()));
            // SN credit fields
            fields_opt.push(optional_field("pCredSN", fc4(*p_cred_sn).as_deref()));
            fields_opt.push(optional_field(
                "vCredICMSSN",
                fc2(*v_cred_icms_sn).as_deref(),
            ));
            Ok(("ICMSSN900".to_string(), filter_fields(fields_opt)))
        }
    }
}

// ── Helper: format field values ─────────────────────────────────────────────

/// Format a monetary [`Cents`] value (2 decimal places) returning `Option<String>`.
fn fc2(v: Option<Cents>) -> Option<String> {
    format_cents_or_none(v.map(|c| c.0), 2)
}

/// Format a [`Rate`] value (4 decimal places) returning `Option<String>`.
fn fc4(v: Option<Rate>) -> Option<String> {
    format_cents_or_none(v.map(|r| r.0), 4)
}

/// Format a raw i64 quantity (4 decimal places) returning `Option<String>`.
fn fc4_raw(v: Option<i64>) -> Option<String> {
    format_cents_or_none(v, 4)
}

// ── Main builders ───────────────────────────────────────────────────────────

/// Build ICMS XML string from a typed [`IcmsVariant`].
///
/// Delegates to [`build_icms_cst_xml`] or [`build_icms_csosn_xml`] depending
/// on the variant, then wraps the result in an `<ICMS>` element and
/// accumulates totals.
///
/// # Errors
///
/// Returns [`FiscalError`] if XML field serialization fails (should not happen
/// when the enum is correctly constructed).
pub fn build_icms_xml(
    variant: &IcmsVariant,
    totals: &mut IcmsTotals,
) -> Result<String, FiscalError> {
    let (variant_tag, fields) = match variant {
        IcmsVariant::Cst(cst) => build_icms_cst_xml(cst, totals)?,
        IcmsVariant::Csosn(csosn) => build_icms_csosn_xml(csosn, totals)?,
    };

    let element = TaxElement {
        outer_tag: Some("ICMS".to_string()),
        outer_fields: vec![],
        variant_tag,
        fields,
    };

    Ok(serialize_tax_element(&element))
}

/// Build the ICMSPart XML group (partition between states).
///
/// Used inside `<ICMS>` for CST 10 or 90 with interstate partition.
///
/// # Errors
///
/// Returns [`FiscalError::MissingRequiredField`] if any required field is
/// missing in the data.
pub fn build_icms_part_xml(data: &IcmsPartData) -> Result<(String, IcmsTotals), FiscalError> {
    let mut totals = create_icms_totals();
    totals.v_bc = accum(totals.v_bc, Some(data.v_bc));
    totals.v_icms = accum(totals.v_icms, Some(data.v_icms));
    totals.v_bc_st = accum(totals.v_bc_st, Some(data.v_bc_st));
    totals.v_st = accum(totals.v_st, Some(data.v_icms_st));
    if data.ind_deduz_deson.as_deref() == Some("1") {
        totals.ind_deduz_deson = true;
    }

    let mut fields_opt: Vec<Option<TaxField>> = vec![
        Some(TaxField::new("orig", data.orig.as_str())),
        Some(TaxField::new("CST", data.cst.as_str())),
        Some(TaxField::new("modBC", data.mod_bc.as_str())),
        Some(TaxField::new("vBC", fc2(Some(data.v_bc)).unwrap())),
        optional_field("pRedBC", fc4(data.p_red_bc).as_deref()),
        Some(TaxField::new("pICMS", fc4(Some(data.p_icms)).unwrap())),
        Some(TaxField::new("vICMS", fc2(Some(data.v_icms)).unwrap())),
    ];

    // ST fields
    fields_opt.push(Some(TaxField::new("modBCST", data.mod_bc_st.as_str())));
    if let Some(v) = data.p_mva_st {
        fields_opt.push(Some(TaxField::new("pMVAST", fc4(Some(v)).unwrap())));
    }
    if let Some(v) = data.p_red_bc_st {
        fields_opt.push(Some(TaxField::new("pRedBCST", fc4(Some(v)).unwrap())));
    }
    fields_opt.push(Some(TaxField::new(
        "vBCST",
        fc2(Some(data.v_bc_st)).unwrap(),
    )));
    fields_opt.push(Some(TaxField::new(
        "pICMSST",
        fc4(Some(data.p_icms_st)).unwrap(),
    )));
    fields_opt.push(Some(TaxField::new(
        "vICMSST",
        fc2(Some(data.v_icms_st)).unwrap(),
    )));

    // FCP ST fields
    fields_opt.push(optional_field("vBCFCPST", fc2(data.v_bc_fcp_st).as_deref()));
    fields_opt.push(optional_field("pFCPST", fc4(data.p_fcp_st).as_deref()));
    fields_opt.push(optional_field("vFCPST", fc2(data.v_fcp_st).as_deref()));

    // pBCOp, UFST
    fields_opt.push(Some(TaxField::new(
        "pBCOp",
        fc4(Some(data.p_bc_op)).unwrap(),
    )));
    fields_opt.push(Some(TaxField::new("UFST", data.uf_st.as_str())));

    // Desoneration
    fields_opt.push(optional_field(
        "vICMSDeson",
        fc2(data.v_icms_deson).as_deref(),
    ));
    fields_opt.push(optional_field("motDesICMS", data.mot_des_icms.as_deref()));
    fields_opt.push(optional_field(
        "indDeduzDeson",
        data.ind_deduz_deson.as_deref(),
    ));

    let fields = filter_fields(fields_opt);

    let element = TaxElement {
        outer_tag: Some("ICMS".to_string()),
        outer_fields: vec![],
        variant_tag: "ICMSPart".to_string(),
        fields,
    };

    Ok((serialize_tax_element(&element), totals))
}

/// Build the ICMSST XML group (ST repasse).
///
/// Used inside `<ICMS>` for CST 41 or 60 with interstate ST repasse.
///
/// # Errors
///
/// Returns [`FiscalError`] if serialization fails.
pub fn build_icms_st_xml(data: &IcmsStData) -> Result<(String, IcmsTotals), FiscalError> {
    let mut totals = create_icms_totals();
    totals.v_fcp_st_ret = accum(totals.v_fcp_st_ret, data.v_fcp_st_ret);

    let fields_opt: Vec<Option<TaxField>> = vec![
        Some(TaxField::new("orig", data.orig.as_str())),
        Some(TaxField::new("CST", data.cst.as_str())),
        Some(TaxField::new(
            "vBCSTRet",
            fc2(Some(data.v_bc_st_ret)).unwrap(),
        )),
        optional_field("pST", fc4(data.p_st).as_deref()),
        optional_field("vICMSSubstituto", fc2(data.v_icms_substituto).as_deref()),
        Some(TaxField::new(
            "vICMSSTRet",
            fc2(Some(data.v_icms_st_ret)).unwrap(),
        )),
        optional_field("vBCFCPSTRet", fc2(data.v_bc_fcp_st_ret).as_deref()),
        optional_field("pFCPSTRet", fc4(data.p_fcp_st_ret).as_deref()),
        optional_field("vFCPSTRet", fc2(data.v_fcp_st_ret).as_deref()),
        Some(TaxField::new(
            "vBCSTDest",
            fc2(Some(data.v_bc_st_dest)).unwrap(),
        )),
        Some(TaxField::new(
            "vICMSSTDest",
            fc2(Some(data.v_icms_st_dest)).unwrap(),
        )),
        optional_field("pRedBCEfet", fc4(data.p_red_bc_efet).as_deref()),
        optional_field("vBCEfet", fc2(data.v_bc_efet).as_deref()),
        optional_field("pICMSEfet", fc4(data.p_icms_efet).as_deref()),
        optional_field("vICMSEfet", fc2(data.v_icms_efet).as_deref()),
    ];

    let fields = filter_fields(fields_opt);

    let element = TaxElement {
        outer_tag: Some("ICMS".to_string()),
        outer_fields: vec![],
        variant_tag: "ICMSST".to_string(),
        fields,
    };

    Ok((serialize_tax_element(&element), totals))
}

/// Build the ICMSUFDest XML group (interstate destination).
///
/// This is a sibling of `<ICMS>`, placed directly inside `<imposto>`.
///
/// # Errors
///
/// Returns [`FiscalError`] if serialization fails.
pub fn build_icms_uf_dest_xml(data: &IcmsUfDestData) -> Result<(String, IcmsTotals), FiscalError> {
    let mut totals = create_icms_totals();
    totals.v_icms_uf_dest = accum(totals.v_icms_uf_dest, Some(data.v_icms_uf_dest));
    totals.v_fcp_uf_dest = accum(totals.v_fcp_uf_dest, data.v_fcp_uf_dest);
    totals.v_icms_uf_remet = accum(totals.v_icms_uf_remet, data.v_icms_uf_remet);

    let fields_opt: Vec<Option<TaxField>> = vec![
        Some(TaxField::new(
            "vBCUFDest",
            fc2(Some(data.v_bc_uf_dest)).unwrap(),
        )),
        optional_field("vBCFCPUFDest", fc2(data.v_bc_fcp_uf_dest).as_deref()),
        optional_field("pFCPUFDest", fc4(data.p_fcp_uf_dest).as_deref()),
        Some(TaxField::new(
            "pICMSUFDest",
            fc4(Some(data.p_icms_uf_dest)).unwrap(),
        )),
        Some(TaxField::new(
            "pICMSInter",
            fc4(Some(data.p_icms_inter)).unwrap(),
        )),
        Some(TaxField::new("pICMSInterPart", "100.0000")),
        optional_field("vFCPUFDest", fc2(data.v_fcp_uf_dest).as_deref()),
        Some(TaxField::new(
            "vICMSUFDest",
            fc2(Some(data.v_icms_uf_dest)).unwrap(),
        )),
        Some(TaxField::new(
            "vICMSUFRemet",
            fc2(Some(data.v_icms_uf_remet.unwrap_or(Cents(0)))).unwrap(),
        )),
    ];

    let fields = filter_fields(fields_opt);

    let element = TaxElement {
        outer_tag: None,
        outer_fields: vec![],
        variant_tag: "ICMSUFDest".to_string(),
        fields,
    };

    Ok((serialize_tax_element(&element), totals))
}

/// Merge item-level ICMS totals into a running accumulator.
///
/// All monetary fields in `source` are added to the corresponding fields of
/// `target`. Call this after each item's ICMS XML has been generated via
/// [`build_icms_part_xml`] or [`build_icms_st_xml`] (which return their own
/// per-item sub-totals) to keep a running document total.
pub fn merge_icms_totals(target: &mut IcmsTotals, source: &IcmsTotals) {
    target.v_bc += source.v_bc;
    target.v_icms += source.v_icms;
    target.v_icms_deson += source.v_icms_deson;
    target.v_bc_st += source.v_bc_st;
    target.v_st += source.v_st;
    target.v_fcp += source.v_fcp;
    target.v_fcp_st += source.v_fcp_st;
    target.v_fcp_st_ret += source.v_fcp_st_ret;
    target.v_fcp_uf_dest += source.v_fcp_uf_dest;
    target.v_icms_uf_dest += source.v_icms_uf_dest;
    target.v_icms_uf_remet += source.v_icms_uf_remet;
    target.q_bc_mono += source.q_bc_mono;
    target.v_icms_mono += source.v_icms_mono;
    target.q_bc_mono_reten += source.q_bc_mono_reten;
    target.v_icms_mono_reten += source.v_icms_mono_reten;
    target.q_bc_mono_ret += source.q_bc_mono_ret;
    target.v_icms_mono_ret += source.v_icms_mono_ret;
    // PHP uses "last one wins" for indDeduzDeson — if any source sets it, propagate.
    if source.ind_deduz_deson {
        target.ind_deduz_deson = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::newtypes::{Cents, Rate};

    // ── IcmsTotals builder methods ──────────────────────────────────

    #[test]
    fn icms_totals_builder_v_icms_uf_remet() {
        let t = IcmsTotals::new().v_icms_uf_remet(Cents(500));
        assert_eq!(t.v_icms_uf_remet, Cents(500));
    }

    #[test]
    fn icms_totals_builder_v_icms_mono() {
        let t = IcmsTotals::new().v_icms_mono(Cents(300));
        assert_eq!(t.v_icms_mono, Cents(300));
    }

    #[test]
    fn icms_totals_builder_v_icms_mono_reten() {
        let t = IcmsTotals::new().v_icms_mono_reten(Cents(200));
        assert_eq!(t.v_icms_mono_reten, Cents(200));
    }

    #[test]
    fn icms_totals_builder_v_icms_mono_ret() {
        let t = IcmsTotals::new().v_icms_mono_ret(Cents(100));
        assert_eq!(t.v_icms_mono_ret, Cents(100));
    }

    #[test]
    fn icms_totals_builder_ind_deduz_deson() {
        let t = IcmsTotals::new().ind_deduz_deson(true);
        assert!(t.ind_deduz_deson);
        let t2 = IcmsTotals::new().ind_deduz_deson(false);
        assert!(!t2.ind_deduz_deson);
    }

    // ── IcmsCst::cst_code() for uncovered variants ─────────────────

    #[test]
    fn cst_code_cst30() {
        let cst = IcmsCst::Cst30 {
            orig: "0".into(),
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(1000),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(180),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
            v_icms_deson: None,
            mot_des_icms: None,
            ind_deduz_deson: None,
        };
        assert_eq!(cst.cst_code(), "30");
    }

    #[test]
    fn cst_code_cst51() {
        let cst = IcmsCst::Cst51 {
            orig: "0".into(),
            mod_bc: None,
            p_red_bc: None,
            c_benef_rbc: None,
            v_bc: None,
            p_icms: None,
            v_icms_op: None,
            p_dif: None,
            v_icms_dif: None,
            v_icms: None,
            v_bc_fcp: None,
            p_fcp: None,
            v_fcp: None,
            p_fcp_dif: None,
            v_fcp_dif: None,
            v_fcp_efet: None,
        };
        assert_eq!(cst.cst_code(), "51");
    }

    #[test]
    fn cst_code_cst53() {
        let cst = IcmsCst::Cst53 {
            orig: "0".into(),
            q_bc_mono: None,
            ad_rem_icms: None,
            v_icms_mono_op: None,
            p_dif: None,
            v_icms_mono_dif: None,
            v_icms_mono: None,
        };
        assert_eq!(cst.cst_code(), "53");
    }

    #[test]
    fn cst_code_cst61() {
        let cst = IcmsCst::Cst61 {
            orig: "0".into(),
            q_bc_mono_ret: None,
            ad_rem_icms_ret: Rate(100),
            v_icms_mono_ret: Cents(50),
        };
        assert_eq!(cst.cst_code(), "61");
    }

    // ── IcmsCsosn::csosn_code() for uncovered variants ─────────────

    #[test]
    fn csosn_code_102() {
        let c = IcmsCsosn::Csosn102 {
            orig: "0".into(),
            csosn: "102".into(),
        };
        assert_eq!(c.csosn_code(), "102");
    }

    #[test]
    fn csosn_code_103() {
        let c = IcmsCsosn::Csosn103 {
            orig: "0".into(),
            csosn: "103".into(),
        };
        assert_eq!(c.csosn_code(), "103");
    }

    #[test]
    fn csosn_code_201() {
        let c = IcmsCsosn::Csosn201 {
            orig: "0".into(),
            csosn: "201".into(),
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(0),
            p_icms_st: Rate(0),
            v_icms_st: Cents(0),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
            p_cred_sn: None,
            v_cred_icms_sn: None,
        };
        assert_eq!(c.csosn_code(), "201");
    }

    #[test]
    fn csosn_code_202() {
        let c = IcmsCsosn::Csosn202 {
            orig: "0".into(),
            csosn: "202".into(),
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(0),
            p_icms_st: Rate(0),
            v_icms_st: Cents(0),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
        };
        assert_eq!(c.csosn_code(), "202");
    }

    #[test]
    fn csosn_code_203() {
        let c = IcmsCsosn::Csosn203 {
            orig: "0".into(),
            csosn: "203".into(),
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(0),
            p_icms_st: Rate(0),
            v_icms_st: Cents(0),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
        };
        assert_eq!(c.csosn_code(), "203");
    }

    #[test]
    fn csosn_code_300() {
        let c = IcmsCsosn::Csosn300 {
            orig: "0".into(),
            csosn: "300".into(),
        };
        assert_eq!(c.csosn_code(), "300");
    }

    #[test]
    fn csosn_code_400() {
        let c = IcmsCsosn::Csosn400 {
            orig: "0".into(),
            csosn: "400".into(),
        };
        assert_eq!(c.csosn_code(), "400");
    }

    #[test]
    fn csosn_code_500() {
        let c = IcmsCsosn::Csosn500 {
            orig: "0".into(),
            csosn: "500".into(),
            v_bc_st_ret: None,
            p_st: None,
            v_icms_substituto: None,
            v_icms_st_ret: None,
            v_bc_fcp_st_ret: None,
            p_fcp_st_ret: None,
            v_fcp_st_ret: None,
            p_red_bc_efet: None,
            v_bc_efet: None,
            p_icms_efet: None,
            v_icms_efet: None,
        };
        assert_eq!(c.csosn_code(), "500");
    }

    #[test]
    fn csosn_code_900() {
        let c = IcmsCsosn::Csosn900 {
            orig: "0".into(),
            csosn: "900".into(),
            mod_bc: None,
            v_bc: None,
            p_red_bc: None,
            p_icms: None,
            v_icms: None,
            mod_bc_st: None,
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: None,
            p_icms_st: None,
            v_icms_st: None,
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
            p_cred_sn: None,
            v_cred_icms_sn: None,
        };
        assert_eq!(c.csosn_code(), "900");
    }

    // ── empty orig in Csosn102/103/300/400 ──────────────────────────

    #[test]
    fn csosn102_empty_orig_omits_orig_field() {
        let c = IcmsCsosn::Csosn102 {
            orig: String::new(),
            csosn: "102".into(),
        };
        let mut totals = IcmsTotals::new();
        let (tag, fields) = build_icms_csosn_xml(&c, &mut totals).unwrap();
        assert_eq!(tag, "ICMSSN102");
        // When orig is empty, the orig field should not be present
        assert!(fields.iter().all(|f| f.name != "orig"));
    }

    // ── merge_icms_totals with ind_deduz_deson=true ─────────────────

    #[test]
    fn merge_icms_totals_propagates_ind_deduz_deson() {
        let mut target = IcmsTotals::new();
        assert!(!target.ind_deduz_deson);

        let source = IcmsTotals::new()
            .v_bc(Cents(1000))
            .v_icms(Cents(180))
            .v_icms_mono(Cents(50))
            .v_icms_mono_reten(Cents(25))
            .v_icms_mono_ret(Cents(10))
            .v_icms_uf_remet(Cents(30))
            .ind_deduz_deson(true);

        merge_icms_totals(&mut target, &source);
        assert!(target.ind_deduz_deson);
        assert_eq!(target.v_bc, Cents(1000));
        assert_eq!(target.v_icms, Cents(180));
        assert_eq!(target.v_icms_mono, Cents(50));
        assert_eq!(target.v_icms_mono_reten, Cents(25));
        assert_eq!(target.v_icms_mono_ret, Cents(10));
        assert_eq!(target.v_icms_uf_remet, Cents(30));
    }

    #[test]
    fn merge_icms_totals_does_not_set_false_on_target() {
        let mut target = IcmsTotals::new().ind_deduz_deson(true);
        let source = IcmsTotals::new(); // ind_deduz_deson = false
        merge_icms_totals(&mut target, &source);
        // target should remain true
        assert!(target.ind_deduz_deson);
    }

    // ── IcmsCst::cst_code() for previously uncovered variants ────────────

    #[test]
    fn cst_code_cst00() {
        let cst = IcmsCst::Cst00 {
            orig: "0".into(),
            mod_bc: "3".into(),
            v_bc: Cents(10000),
            p_icms: Rate(1800),
            v_icms: Cents(1800),
            p_fcp: None,
            v_fcp: None,
        };
        assert_eq!(cst.cst_code(), "00");
    }

    #[test]
    fn cst_code_cst02() {
        let cst = IcmsCst::Cst02 {
            orig: "0".into(),
            q_bc_mono: None,
            ad_rem_icms: Rate(100),
            v_icms_mono: Cents(50),
        };
        assert_eq!(cst.cst_code(), "02");
    }

    #[test]
    fn cst_code_cst10() {
        let cst = IcmsCst::Cst10 {
            orig: "0".into(),
            mod_bc: "3".into(),
            v_bc: Cents(10000),
            p_icms: Rate(1800),
            v_icms: Cents(1800),
            v_bc_fcp: None,
            p_fcp: None,
            v_fcp: None,
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(10000),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(1800),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
            v_icms_st_deson: None,
            mot_des_icms_st: None,
        };
        assert_eq!(cst.cst_code(), "10");
    }

    #[test]
    fn cst_code_cst15() {
        let cst = IcmsCst::Cst15 {
            orig: "0".into(),
            q_bc_mono: None,
            ad_rem_icms: Rate(100),
            v_icms_mono: Cents(50),
            q_bc_mono_reten: None,
            ad_rem_icms_reten: Rate(80),
            v_icms_mono_reten: Cents(40),
            p_red_ad_rem: None,
            mot_red_ad_rem: None,
        };
        assert_eq!(cst.cst_code(), "15");
    }

    #[test]
    fn cst_code_cst20() {
        let cst = IcmsCst::Cst20 {
            orig: "0".into(),
            mod_bc: "3".into(),
            p_red_bc: Rate(5000),
            v_bc: Cents(5000),
            p_icms: Rate(1800),
            v_icms: Cents(900),
            v_bc_fcp: None,
            p_fcp: None,
            v_fcp: None,
            v_icms_deson: None,
            mot_des_icms: None,
            ind_deduz_deson: None,
        };
        assert_eq!(cst.cst_code(), "20");
    }

    #[test]
    fn cst_code_cst60() {
        let cst = IcmsCst::Cst60 {
            orig: "0".into(),
            v_bc_st_ret: None,
            p_st: None,
            v_icms_substituto: None,
            v_icms_st_ret: None,
            v_bc_fcp_st_ret: None,
            p_fcp_st_ret: None,
            v_fcp_st_ret: None,
            p_red_bc_efet: None,
            v_bc_efet: None,
            p_icms_efet: None,
            v_icms_efet: None,
        };
        assert_eq!(cst.cst_code(), "60");
    }

    #[test]
    fn cst_code_cst70() {
        let cst = IcmsCst::Cst70 {
            orig: "0".into(),
            mod_bc: "3".into(),
            p_red_bc: Rate(5000),
            v_bc: Cents(5000),
            p_icms: Rate(1800),
            v_icms: Cents(900),
            v_bc_fcp: None,
            p_fcp: None,
            v_fcp: None,
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(5000),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(900),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
            v_icms_deson: None,
            mot_des_icms: None,
            ind_deduz_deson: None,
            v_icms_st_deson: None,
            mot_des_icms_st: None,
        };
        assert_eq!(cst.cst_code(), "70");
    }

    #[test]
    fn cst_code_cst90() {
        let cst = IcmsCst::Cst90 {
            orig: "0".into(),
            mod_bc: None,
            v_bc: None,
            p_red_bc: None,
            c_benef_rbc: None,
            p_icms: None,
            v_icms_op: None,
            p_dif: None,
            v_icms_dif: None,
            v_icms: None,
            v_bc_fcp: None,
            p_fcp: None,
            v_fcp: None,
            p_fcp_dif: None,
            v_fcp_dif: None,
            v_fcp_efet: None,
            mod_bc_st: None,
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: None,
            p_icms_st: None,
            v_icms_st: None,
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
            v_icms_deson: None,
            mot_des_icms: None,
            ind_deduz_deson: None,
            v_icms_st_deson: None,
            mot_des_icms_st: None,
        };
        assert_eq!(cst.cst_code(), "90");
    }

    // ── IcmsCsosn::csosn_code() for Csosn101 (line 1752) ────────────────

    #[test]
    fn csosn_code_101() {
        let c = IcmsCsosn::Csosn101 {
            orig: "0".into(),
            csosn: "101".into(),
            p_cred_sn: Rate(150),
            v_cred_icms_sn: Cents(30),
        };
        assert_eq!(c.csosn_code(), "101");
    }
}
