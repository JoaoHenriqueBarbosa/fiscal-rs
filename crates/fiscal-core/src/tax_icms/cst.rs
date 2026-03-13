//! IcmsCst enum — normal tax regime (Lucro Real / Presumido).

use crate::newtypes::{Cents, Rate};

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
