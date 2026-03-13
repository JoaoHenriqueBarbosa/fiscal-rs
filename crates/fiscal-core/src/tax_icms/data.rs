//! ICMS data structures: IcmsPartData, IcmsStData, IcmsUfDestData.

use crate::newtypes::{Cents, Rate};

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
