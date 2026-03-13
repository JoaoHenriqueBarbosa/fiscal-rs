//! IcmsTotals accumulator and merge functions.

use crate::newtypes::Cents;

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
