use crate::format_utils::format_cents_or_none;
use crate::newtypes::{Cents, Rate};
use crate::tax_element::{
    filter_fields, optional_field, required_field, serialize_tax_element, TaxElement, TaxField,
};
use crate::FiscalError;

/// Accumulate a value into a totals field.
fn accum(current: Cents, value: Option<Cents>) -> Cents {
    current + value.unwrap_or(Cents(0))
}

/// Accumulate a raw i64 quantity into a totals field.
fn accum_raw(current: i64, value: Option<i64>) -> i64 {
    current + value.unwrap_or(0)
}

// ── Types ───────────────────────────────────────────────────────────────────

/// Unified input data for all ICMS variations.
/// Monetary fields use [`Cents`], rate fields use [`Rate`] (hundredths).
#[derive(Debug, Clone, Default)]
pub struct IcmsData {
    pub tax_regime: u8,
    pub orig: String,
    pub cst: Option<String>,
    pub csosn: Option<String>,
    pub mod_bc: Option<String>,
    pub v_bc: Option<Cents>,
    pub p_red_bc: Option<Rate>,
    pub p_icms: Option<Rate>,
    pub v_icms: Option<Cents>,
    pub v_bc_fcp: Option<Cents>,
    pub p_fcp: Option<Rate>,
    pub v_fcp: Option<Cents>,
    pub mod_bc_st: Option<String>,
    pub p_mva_st: Option<Rate>,
    pub p_red_bc_st: Option<Rate>,
    pub v_bc_st: Option<Cents>,
    pub p_icms_st: Option<Rate>,
    pub v_icms_st: Option<Cents>,
    pub v_bc_fcp_st: Option<Cents>,
    pub p_fcp_st: Option<Rate>,
    pub v_fcp_st: Option<Cents>,
    pub v_icms_deson: Option<Cents>,
    pub mot_des_icms: Option<String>,
    pub ind_deduz_deson: Option<String>,
    pub v_icms_st_deson: Option<Cents>,
    pub mot_des_icms_st: Option<String>,
    pub v_bc_st_ret: Option<Cents>,
    pub p_st: Option<Rate>,
    pub v_icms_substituto: Option<Cents>,
    pub v_icms_st_ret: Option<Cents>,
    pub v_bc_fcp_st_ret: Option<Cents>,
    pub p_fcp_st_ret: Option<Rate>,
    pub v_fcp_st_ret: Option<Cents>,
    pub p_red_bc_efet: Option<Rate>,
    pub v_bc_efet: Option<Cents>,
    pub p_icms_efet: Option<Rate>,
    pub v_icms_efet: Option<Cents>,
    pub v_icms_op: Option<Cents>,
    pub p_dif: Option<Rate>,
    pub v_icms_dif: Option<Cents>,
    pub p_fcp_dif: Option<Rate>,
    pub v_fcp_dif: Option<Cents>,
    pub v_fcp_efet: Option<Cents>,
    pub q_bc_mono: Option<i64>,
    pub ad_rem_icms: Option<Rate>,
    pub v_icms_mono: Option<Cents>,
    pub v_icms_mono_op: Option<Cents>,
    pub ad_rem_icms_reten: Option<Rate>,
    pub q_bc_mono_reten: Option<i64>,
    pub v_icms_mono_reten: Option<Cents>,
    pub v_icms_mono_dif: Option<Cents>,
    pub q_bc_mono_ret: Option<i64>,
    pub ad_rem_icms_ret: Option<Rate>,
    pub v_icms_mono_ret: Option<Cents>,
    pub p_red_ad_rem: Option<Rate>,
    pub mot_red_ad_rem: Option<String>,
    pub c_benef_rbc: Option<String>,
    pub p_cred_sn: Option<Rate>,
    pub v_cred_icms_sn: Option<Cents>,
    pub p_bc_op: Option<Rate>,
    pub uf_st: Option<String>,
    pub v_bc_st_dest: Option<Cents>,
    pub v_icms_st_dest: Option<Cents>,
    pub v_bc_uf_dest: Option<Cents>,
    pub v_bc_fcp_uf_dest: Option<Cents>,
    pub p_fcp_uf_dest: Option<Rate>,
    pub p_icms_uf_dest: Option<Rate>,
    pub p_icms_inter: Option<Rate>,
    pub p_icms_inter_part: Option<Rate>,
    pub v_fcp_uf_dest: Option<Cents>,
    pub v_icms_uf_dest: Option<Cents>,
    pub v_icms_uf_remet: Option<Cents>,
}

/// Accumulated ICMS totals across all items.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IcmsTotals {
    pub v_bc: Cents,
    pub v_icms: Cents,
    pub v_icms_deson: Cents,
    pub v_bc_st: Cents,
    pub v_st: Cents,
    pub v_fcp: Cents,
    pub v_fcp_st: Cents,
    pub v_fcp_st_ret: Cents,
    pub v_fcp_uf_dest: Cents,
    pub v_icms_uf_dest: Cents,
    pub v_icms_uf_remet: Cents,
    pub q_bc_mono: i64,
    pub v_icms_mono: Cents,
    pub q_bc_mono_reten: i64,
    pub v_icms_mono_reten: Cents,
    pub q_bc_mono_ret: i64,
    pub v_icms_mono_ret: Cents,
}

/// Create a zeroed-out ICMS totals.
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
        orig: String,
        mod_bc: String,
        v_bc: Cents,
        p_icms: Rate,
        v_icms: Cents,
        p_fcp: Option<Rate>,
        v_fcp: Option<Cents>,
    },
    /// CST 02 — Tributacao monofasica propria sobre combustiveis.
    Cst02 {
        orig: String,
        q_bc_mono: Option<i64>,
        ad_rem_icms: Rate,
        v_icms_mono: Cents,
    },
    /// CST 10 — Tributada e com cobranca do ICMS por substituicao tributaria.
    Cst10 {
        orig: String,
        mod_bc: String,
        v_bc: Cents,
        p_icms: Rate,
        v_icms: Cents,
        v_bc_fcp: Option<Cents>,
        p_fcp: Option<Rate>,
        v_fcp: Option<Cents>,
        mod_bc_st: String,
        p_mva_st: Option<Rate>,
        p_red_bc_st: Option<Rate>,
        v_bc_st: Cents,
        p_icms_st: Rate,
        v_icms_st: Cents,
        v_bc_fcp_st: Option<Cents>,
        p_fcp_st: Option<Rate>,
        v_fcp_st: Option<Cents>,
        v_icms_st_deson: Option<Cents>,
        mot_des_icms_st: Option<String>,
    },
    /// CST 15 — Tributacao monofasica propria e com responsabilidade pela
    /// retencao sobre combustiveis.
    Cst15 {
        orig: String,
        q_bc_mono: Option<i64>,
        ad_rem_icms: Rate,
        v_icms_mono: Cents,
        q_bc_mono_reten: Option<i64>,
        ad_rem_icms_reten: Rate,
        v_icms_mono_reten: Cents,
        p_red_ad_rem: Option<Rate>,
        mot_red_ad_rem: Option<String>,
    },
    /// CST 20 — Com reducao de base de calculo.
    Cst20 {
        orig: String,
        mod_bc: String,
        p_red_bc: Rate,
        v_bc: Cents,
        p_icms: Rate,
        v_icms: Cents,
        v_bc_fcp: Option<Cents>,
        p_fcp: Option<Rate>,
        v_fcp: Option<Cents>,
        v_icms_deson: Option<Cents>,
        mot_des_icms: Option<String>,
        ind_deduz_deson: Option<String>,
    },
    /// CST 30 — Isenta ou nao tributada e com cobranca do ICMS por ST.
    Cst30 {
        orig: String,
        mod_bc_st: String,
        p_mva_st: Option<Rate>,
        p_red_bc_st: Option<Rate>,
        v_bc_st: Cents,
        p_icms_st: Rate,
        v_icms_st: Cents,
        v_bc_fcp_st: Option<Cents>,
        p_fcp_st: Option<Rate>,
        v_fcp_st: Option<Cents>,
        v_icms_deson: Option<Cents>,
        mot_des_icms: Option<String>,
        ind_deduz_deson: Option<String>,
    },
    /// CST 40 — Isenta.
    Cst40 {
        orig: String,
        v_icms_deson: Option<Cents>,
        mot_des_icms: Option<String>,
        ind_deduz_deson: Option<String>,
    },
    /// CST 41 — Nao tributada.
    Cst41 {
        orig: String,
        v_icms_deson: Option<Cents>,
        mot_des_icms: Option<String>,
        ind_deduz_deson: Option<String>,
    },
    /// CST 50 — Suspensao.
    Cst50 {
        orig: String,
        v_icms_deson: Option<Cents>,
        mot_des_icms: Option<String>,
        ind_deduz_deson: Option<String>,
    },
    /// CST 51 — Diferimento.
    Cst51 {
        orig: String,
        mod_bc: Option<String>,
        p_red_bc: Option<Rate>,
        c_benef_rbc: Option<String>,
        v_bc: Option<Cents>,
        p_icms: Option<Rate>,
        v_icms_op: Option<Cents>,
        p_dif: Option<Rate>,
        v_icms_dif: Option<Cents>,
        v_icms: Option<Cents>,
        v_bc_fcp: Option<Cents>,
        p_fcp: Option<Rate>,
        v_fcp: Option<Cents>,
        p_fcp_dif: Option<Rate>,
        v_fcp_dif: Option<Cents>,
        v_fcp_efet: Option<Cents>,
    },
    /// CST 53 — Tributacao monofasica sobre combustiveis com recolhimento
    /// diferido.
    Cst53 {
        orig: String,
        q_bc_mono: Option<i64>,
        ad_rem_icms: Option<Rate>,
        v_icms_mono_op: Option<Cents>,
        p_dif: Option<Rate>,
        v_icms_mono_dif: Option<Cents>,
        v_icms_mono: Option<Cents>,
    },
    /// CST 60 — ICMS cobrado anteriormente por substituicao tributaria.
    Cst60 {
        orig: String,
        v_bc_st_ret: Option<Cents>,
        p_st: Option<Rate>,
        v_icms_substituto: Option<Cents>,
        v_icms_st_ret: Option<Cents>,
        v_bc_fcp_st_ret: Option<Cents>,
        p_fcp_st_ret: Option<Rate>,
        v_fcp_st_ret: Option<Cents>,
        p_red_bc_efet: Option<Rate>,
        v_bc_efet: Option<Cents>,
        p_icms_efet: Option<Rate>,
        v_icms_efet: Option<Cents>,
    },
    /// CST 61 — Tributacao monofasica sobre combustiveis cobrada anteriormente.
    Cst61 {
        orig: String,
        q_bc_mono_ret: Option<i64>,
        ad_rem_icms_ret: Rate,
        v_icms_mono_ret: Cents,
    },
    /// CST 70 — Reducao de base de calculo e cobranca do ICMS por ST.
    Cst70 {
        orig: String,
        mod_bc: String,
        p_red_bc: Rate,
        v_bc: Cents,
        p_icms: Rate,
        v_icms: Cents,
        v_bc_fcp: Option<Cents>,
        p_fcp: Option<Rate>,
        v_fcp: Option<Cents>,
        mod_bc_st: String,
        p_mva_st: Option<Rate>,
        p_red_bc_st: Option<Rate>,
        v_bc_st: Cents,
        p_icms_st: Rate,
        v_icms_st: Cents,
        v_bc_fcp_st: Option<Cents>,
        p_fcp_st: Option<Rate>,
        v_fcp_st: Option<Cents>,
        v_icms_deson: Option<Cents>,
        mot_des_icms: Option<String>,
        ind_deduz_deson: Option<String>,
        v_icms_st_deson: Option<Cents>,
        mot_des_icms_st: Option<String>,
    },
    /// CST 90 — Outros.
    Cst90 {
        orig: String,
        mod_bc: Option<String>,
        v_bc: Option<Cents>,
        p_red_bc: Option<Rate>,
        c_benef_rbc: Option<String>,
        p_icms: Option<Rate>,
        v_icms_op: Option<Cents>,
        p_dif: Option<Rate>,
        v_icms_dif: Option<Cents>,
        v_icms: Option<Cents>,
        v_bc_fcp: Option<Cents>,
        p_fcp: Option<Rate>,
        v_fcp: Option<Cents>,
        p_fcp_dif: Option<Rate>,
        v_fcp_dif: Option<Cents>,
        v_fcp_efet: Option<Cents>,
        mod_bc_st: Option<String>,
        p_mva_st: Option<Rate>,
        p_red_bc_st: Option<Rate>,
        v_bc_st: Option<Cents>,
        p_icms_st: Option<Rate>,
        v_icms_st: Option<Cents>,
        v_bc_fcp_st: Option<Cents>,
        p_fcp_st: Option<Rate>,
        v_fcp_st: Option<Cents>,
        v_icms_deson: Option<Cents>,
        mot_des_icms: Option<String>,
        ind_deduz_deson: Option<String>,
        v_icms_st_deson: Option<Cents>,
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

impl TryFrom<&IcmsData> for IcmsCst {
    type Error = FiscalError;

    /// Convert from the flat [`IcmsData`] struct into a typed [`IcmsCst`]
    /// variant.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::MissingRequiredField`] when a field required by
    /// the target CST variant is `None`, or
    /// [`FiscalError::UnsupportedIcmsCst`] for unrecognized CST codes.
    fn try_from(d: &IcmsData) -> Result<Self, FiscalError> {
        let cst = d.cst.as_deref().ok_or_else(|| FiscalError::MissingRequiredField {
            field: "CST".to_string(),
        })?;

        match cst {
            "00" => Ok(Self::Cst00 {
                orig: d.orig.clone(),
                mod_bc: d.mod_bc.clone().ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "modBC".to_string(),
                })?,
                v_bc: d.v_bc.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "vBC".to_string(),
                })?,
                p_icms: d.p_icms.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "pICMS".to_string(),
                })?,
                v_icms: d.v_icms.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "vICMS".to_string(),
                })?,
                p_fcp: d.p_fcp,
                v_fcp: d.v_fcp,
            }),

            "02" => Ok(Self::Cst02 {
                orig: d.orig.clone(),
                q_bc_mono: d.q_bc_mono,
                ad_rem_icms: d.ad_rem_icms.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "adRemICMS".to_string(),
                })?,
                v_icms_mono: d.v_icms_mono.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "vICMSMono".to_string(),
                })?,
            }),

            "10" => Ok(Self::Cst10 {
                orig: d.orig.clone(),
                mod_bc: d.mod_bc.clone().ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "modBC".to_string(),
                })?,
                v_bc: d.v_bc.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "vBC".to_string(),
                })?,
                p_icms: d.p_icms.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "pICMS".to_string(),
                })?,
                v_icms: d.v_icms.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "vICMS".to_string(),
                })?,
                v_bc_fcp: d.v_bc_fcp,
                p_fcp: d.p_fcp,
                v_fcp: d.v_fcp,
                mod_bc_st: d.mod_bc_st.clone().ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "modBCST".to_string(),
                })?,
                p_mva_st: d.p_mva_st,
                p_red_bc_st: d.p_red_bc_st,
                v_bc_st: d.v_bc_st.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "vBCST".to_string(),
                })?,
                p_icms_st: d.p_icms_st.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "pICMSST".to_string(),
                })?,
                v_icms_st: d.v_icms_st.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "vICMSST".to_string(),
                })?,
                v_bc_fcp_st: d.v_bc_fcp_st,
                p_fcp_st: d.p_fcp_st,
                v_fcp_st: d.v_fcp_st,
                v_icms_st_deson: d.v_icms_st_deson,
                mot_des_icms_st: d.mot_des_icms_st.clone(),
            }),

            "15" => Ok(Self::Cst15 {
                orig: d.orig.clone(),
                q_bc_mono: d.q_bc_mono,
                ad_rem_icms: d.ad_rem_icms.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "adRemICMS".to_string(),
                })?,
                v_icms_mono: d.v_icms_mono.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "vICMSMono".to_string(),
                })?,
                q_bc_mono_reten: d.q_bc_mono_reten,
                ad_rem_icms_reten: d.ad_rem_icms_reten.ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "adRemICMSReten".to_string(),
                    }
                })?,
                v_icms_mono_reten: d.v_icms_mono_reten.ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "vICMSMonoReten".to_string(),
                    }
                })?,
                p_red_ad_rem: d.p_red_ad_rem,
                mot_red_ad_rem: d.mot_red_ad_rem.clone(),
            }),

            "20" => Ok(Self::Cst20 {
                orig: d.orig.clone(),
                mod_bc: d.mod_bc.clone().ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "modBC".to_string(),
                })?,
                p_red_bc: d.p_red_bc.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "pRedBC".to_string(),
                })?,
                v_bc: d.v_bc.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "vBC".to_string(),
                })?,
                p_icms: d.p_icms.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "pICMS".to_string(),
                })?,
                v_icms: d.v_icms.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "vICMS".to_string(),
                })?,
                v_bc_fcp: d.v_bc_fcp,
                p_fcp: d.p_fcp,
                v_fcp: d.v_fcp,
                v_icms_deson: d.v_icms_deson,
                mot_des_icms: d.mot_des_icms.clone(),
                ind_deduz_deson: d.ind_deduz_deson.clone(),
            }),

            "30" => Ok(Self::Cst30 {
                orig: d.orig.clone(),
                mod_bc_st: d.mod_bc_st.clone().ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "modBCST".to_string(),
                })?,
                p_mva_st: d.p_mva_st,
                p_red_bc_st: d.p_red_bc_st,
                v_bc_st: d.v_bc_st.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "vBCST".to_string(),
                })?,
                p_icms_st: d.p_icms_st.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "pICMSST".to_string(),
                })?,
                v_icms_st: d.v_icms_st.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "vICMSST".to_string(),
                })?,
                v_bc_fcp_st: d.v_bc_fcp_st,
                p_fcp_st: d.p_fcp_st,
                v_fcp_st: d.v_fcp_st,
                v_icms_deson: d.v_icms_deson,
                mot_des_icms: d.mot_des_icms.clone(),
                ind_deduz_deson: d.ind_deduz_deson.clone(),
            }),

            "40" => Ok(Self::Cst40 {
                orig: d.orig.clone(),
                v_icms_deson: d.v_icms_deson,
                mot_des_icms: d.mot_des_icms.clone(),
                ind_deduz_deson: d.ind_deduz_deson.clone(),
            }),

            "41" => Ok(Self::Cst41 {
                orig: d.orig.clone(),
                v_icms_deson: d.v_icms_deson,
                mot_des_icms: d.mot_des_icms.clone(),
                ind_deduz_deson: d.ind_deduz_deson.clone(),
            }),

            "50" => Ok(Self::Cst50 {
                orig: d.orig.clone(),
                v_icms_deson: d.v_icms_deson,
                mot_des_icms: d.mot_des_icms.clone(),
                ind_deduz_deson: d.ind_deduz_deson.clone(),
            }),

            "51" => Ok(Self::Cst51 {
                orig: d.orig.clone(),
                mod_bc: d.mod_bc.clone(),
                p_red_bc: d.p_red_bc,
                c_benef_rbc: d.c_benef_rbc.clone(),
                v_bc: d.v_bc,
                p_icms: d.p_icms,
                v_icms_op: d.v_icms_op,
                p_dif: d.p_dif,
                v_icms_dif: d.v_icms_dif,
                v_icms: d.v_icms,
                v_bc_fcp: d.v_bc_fcp,
                p_fcp: d.p_fcp,
                v_fcp: d.v_fcp,
                p_fcp_dif: d.p_fcp_dif,
                v_fcp_dif: d.v_fcp_dif,
                v_fcp_efet: d.v_fcp_efet,
            }),

            "53" => Ok(Self::Cst53 {
                orig: d.orig.clone(),
                q_bc_mono: d.q_bc_mono,
                ad_rem_icms: d.ad_rem_icms,
                v_icms_mono_op: d.v_icms_mono_op,
                p_dif: d.p_dif,
                v_icms_mono_dif: d.v_icms_mono_dif,
                v_icms_mono: d.v_icms_mono,
            }),

            "60" => Ok(Self::Cst60 {
                orig: d.orig.clone(),
                v_bc_st_ret: d.v_bc_st_ret,
                p_st: d.p_st,
                v_icms_substituto: d.v_icms_substituto,
                v_icms_st_ret: d.v_icms_st_ret,
                v_bc_fcp_st_ret: d.v_bc_fcp_st_ret,
                p_fcp_st_ret: d.p_fcp_st_ret,
                v_fcp_st_ret: d.v_fcp_st_ret,
                p_red_bc_efet: d.p_red_bc_efet,
                v_bc_efet: d.v_bc_efet,
                p_icms_efet: d.p_icms_efet,
                v_icms_efet: d.v_icms_efet,
            }),

            "61" => Ok(Self::Cst61 {
                orig: d.orig.clone(),
                q_bc_mono_ret: d.q_bc_mono_ret,
                ad_rem_icms_ret: d.ad_rem_icms_ret.ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "adRemICMSRet".to_string(),
                    }
                })?,
                v_icms_mono_ret: d.v_icms_mono_ret.ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "vICMSMonoRet".to_string(),
                    }
                })?,
            }),

            "70" => Ok(Self::Cst70 {
                orig: d.orig.clone(),
                mod_bc: d.mod_bc.clone().ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "modBC".to_string(),
                })?,
                p_red_bc: d.p_red_bc.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "pRedBC".to_string(),
                })?,
                v_bc: d.v_bc.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "vBC".to_string(),
                })?,
                p_icms: d.p_icms.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "pICMS".to_string(),
                })?,
                v_icms: d.v_icms.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "vICMS".to_string(),
                })?,
                v_bc_fcp: d.v_bc_fcp,
                p_fcp: d.p_fcp,
                v_fcp: d.v_fcp,
                mod_bc_st: d.mod_bc_st.clone().ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "modBCST".to_string(),
                })?,
                p_mva_st: d.p_mva_st,
                p_red_bc_st: d.p_red_bc_st,
                v_bc_st: d.v_bc_st.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "vBCST".to_string(),
                })?,
                p_icms_st: d.p_icms_st.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "pICMSST".to_string(),
                })?,
                v_icms_st: d.v_icms_st.ok_or_else(|| FiscalError::MissingRequiredField {
                    field: "vICMSST".to_string(),
                })?,
                v_bc_fcp_st: d.v_bc_fcp_st,
                p_fcp_st: d.p_fcp_st,
                v_fcp_st: d.v_fcp_st,
                v_icms_deson: d.v_icms_deson,
                mot_des_icms: d.mot_des_icms.clone(),
                ind_deduz_deson: d.ind_deduz_deson.clone(),
                v_icms_st_deson: d.v_icms_st_deson,
                mot_des_icms_st: d.mot_des_icms_st.clone(),
            }),

            "90" => Ok(Self::Cst90 {
                orig: d.orig.clone(),
                mod_bc: d.mod_bc.clone(),
                v_bc: d.v_bc,
                p_red_bc: d.p_red_bc,
                c_benef_rbc: d.c_benef_rbc.clone(),
                p_icms: d.p_icms,
                v_icms_op: d.v_icms_op,
                p_dif: d.p_dif,
                v_icms_dif: d.v_icms_dif,
                v_icms: d.v_icms,
                v_bc_fcp: d.v_bc_fcp,
                p_fcp: d.p_fcp,
                v_fcp: d.v_fcp,
                p_fcp_dif: d.p_fcp_dif,
                v_fcp_dif: d.v_fcp_dif,
                v_fcp_efet: d.v_fcp_efet,
                mod_bc_st: d.mod_bc_st.clone(),
                p_mva_st: d.p_mva_st,
                p_red_bc_st: d.p_red_bc_st,
                v_bc_st: d.v_bc_st,
                p_icms_st: d.p_icms_st,
                v_icms_st: d.v_icms_st,
                v_bc_fcp_st: d.v_bc_fcp_st,
                p_fcp_st: d.p_fcp_st,
                v_fcp_st: d.v_fcp_st,
                v_icms_deson: d.v_icms_deson,
                mot_des_icms: d.mot_des_icms.clone(),
                ind_deduz_deson: d.ind_deduz_deson.clone(),
                v_icms_st_deson: d.v_icms_st_deson,
                mot_des_icms_st: d.mot_des_icms_st.clone(),
            }),

            other => Err(FiscalError::UnsupportedIcmsCst(other.to_string())),
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
            fields_opt.push(Some(TaxField::new("pICMSST", fc4(Some(*p_icms_st)).unwrap())));
            fields_opt.push(Some(TaxField::new("vICMSST", fc2(Some(*v_icms_st)).unwrap())));
            // FCP ST fields
            fields_opt.push(optional_field("vBCFCPST", fc2(*v_bc_fcp_st).as_deref()));
            fields_opt.push(optional_field("pFCPST", fc4(*p_fcp_st).as_deref()));
            fields_opt.push(optional_field("vFCPST", fc2(*v_fcp_st).as_deref()));
            // ST desoneration
            fields_opt.push(optional_field("vICMSSTDeson", fc2(*v_icms_st_deson).as_deref()));
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
                Some(TaxField::new("adRemICMSReten", fc4(Some(*ad_rem_icms_reten)).unwrap())),
                Some(TaxField::new("vICMSMonoReten", fc2(Some(*v_icms_mono_reten)).unwrap())),
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
            fields_opt.push(Some(TaxField::new("pICMSST", fc4(Some(*p_icms_st)).unwrap())));
            fields_opt.push(Some(TaxField::new("vICMSST", fc2(Some(*v_icms_st)).unwrap())));
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
                Some(TaxField::new("adRemICMSRet", fc4(Some(*ad_rem_icms_ret)).unwrap())),
                Some(TaxField::new("vICMSMonoRet", fc2(Some(*v_icms_mono_ret)).unwrap())),
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
            fields_opt.push(Some(TaxField::new("pICMSST", fc4(Some(*p_icms_st)).unwrap())));
            fields_opt.push(Some(TaxField::new("vICMSST", fc2(Some(*v_icms_st)).unwrap())));
            // FCP ST
            fields_opt.push(optional_field("vBCFCPST", fc2(*v_bc_fcp_st).as_deref()));
            fields_opt.push(optional_field("pFCPST", fc4(*p_fcp_st).as_deref()));
            fields_opt.push(optional_field("vFCPST", fc2(*v_fcp_st).as_deref()));
            // Desoneration
            fields_opt.push(optional_field("vICMSDeson", fc2(*v_icms_deson).as_deref()));
            fields_opt.push(optional_field("motDesICMS", mot_des_icms.as_deref()));
            fields_opt.push(optional_field("indDeduzDeson", ind_deduz_deson.as_deref()));
            // ST desoneration
            fields_opt.push(optional_field("vICMSSTDeson", fc2(*v_icms_st_deson).as_deref()));
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
            fields_opt.push(optional_field("vICMSSTDeson", fc2(*v_icms_st_deson).as_deref()));
            fields_opt.push(optional_field("motDesICMSST", mot_des_icms_st.as_deref()));
            Ok(("ICMS90".to_string(), filter_fields(fields_opt)))
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

// ── Domain field-block helpers ──────────────────────────────────────────────

/// FCP-ST: vBCFCPST, pFCPST, vFCPST
fn fcp_st_fields(d: &IcmsData) -> Vec<Option<TaxField>> {
    vec![
        optional_field("vBCFCPST", fc2(d.v_bc_fcp_st).as_deref()),
        optional_field("pFCPST", fc4(d.p_fcp_st).as_deref()),
        optional_field("vFCPST", fc2(d.v_fcp_st).as_deref()),
    ]
}

/// Collect ST fields: required fields cause error, optional skips return None
fn collect_st_fields(d: &IcmsData) -> Result<Vec<TaxField>, FiscalError> {
    let mut result = Vec::new();
    // modBCST - required
    result.push(required_field("modBCST", d.mod_bc_st.as_deref())?);
    // pMVAST - optional
    if let Some(f) = optional_field("pMVAST", fc4(d.p_mva_st).as_deref()) {
        result.push(f);
    }
    // pRedBCST - optional
    if let Some(f) = optional_field("pRedBCST", fc4(d.p_red_bc_st).as_deref()) {
        result.push(f);
    }
    // vBCST - required
    result.push(required_field("vBCST", fc2(d.v_bc_st).as_deref())?);
    // pICMSST - required
    result.push(required_field("pICMSST", fc4(d.p_icms_st).as_deref())?);
    // vICMSST - required
    result.push(required_field("vICMSST", fc2(d.v_icms_st).as_deref())?);
    Ok(result)
}

/// Desoneration: vICMSDeson, motDesICMS, indDeduzDeson
fn desoneration_fields(d: &IcmsData) -> Vec<Option<TaxField>> {
    vec![
        optional_field("vICMSDeson", fc2(d.v_icms_deson).as_deref()),
        optional_field("motDesICMS", d.mot_des_icms.as_deref()),
        optional_field("indDeduzDeson", d.ind_deduz_deson.as_deref()),
    ]
}

// ── Main builder ────────────────────────────────────────────────────────────

/// Build ICMS XML string.
///
/// Dispatches by tax_regime and CST/CSOSN to produce the correct variant tag
/// and accumulate totals. Returns `(xml_string, totals)`.
///
/// # Errors
///
/// Returns [`FiscalError::MissingRequiredField`] if a required tax field
/// (CST, CSOSN, or any inner field like `modBC`, `vBC`, etc.) is `None`.
/// Returns [`FiscalError::UnsupportedIcmsCst`] or
/// [`FiscalError::UnsupportedIcmsCsosn`] for unrecognized CST/CSOSN codes.
pub fn build_icms_xml(data: &IcmsData) -> Result<(String, IcmsTotals), FiscalError> {
    let mut totals = create_icms_totals();

    let (variant_tag, fields) = if data.tax_regime == 1 || data.tax_regime == 2 {
        let csosn = data.csosn.as_deref().ok_or_else(|| {
            FiscalError::MissingRequiredField {
                field: "CSOSN".to_string(),
            }
        })?;
        // Generic SN totals
        totals.v_fcp_st = accum(totals.v_fcp_st, data.v_fcp_st);
        totals.v_fcp_st_ret = accum(totals.v_fcp_st_ret, data.v_fcp_st_ret);
        calculate_csosn(data, &mut totals, csosn)?
    } else {
        let cst = data.cst.as_deref().ok_or_else(|| {
            FiscalError::MissingRequiredField {
                field: "CST".to_string(),
            }
        })?;
        calculate_cst(data, &mut totals, cst)?
    };

    let element = TaxElement {
        outer_tag: Some("ICMS".to_string()),
        outer_fields: vec![],
        variant_tag,
        fields,
    };

    Ok((serialize_tax_element(&element), totals))
}

/// Build the ICMSPart XML group (partition between states).
///
/// Used inside `<ICMS>` for CST 10 or 90 with interstate partition.
///
/// # Errors
///
/// Returns [`FiscalError::MissingRequiredField`] if any required field
/// (e.g. `orig`, `CST`, `modBC`, `vBC`, `pICMS`, `vICMS`, `modBCST`,
/// `vBCST`, `pICMSST`, `vICMSST`, `pBCOp`, `UFST`) is `None`.
pub fn build_icms_part_xml(data: &IcmsData) -> Result<(String, IcmsTotals), FiscalError> {
    let mut totals = create_icms_totals();
    totals.v_bc = accum(totals.v_bc, data.v_bc);
    totals.v_icms = accum(totals.v_icms, data.v_icms);
    totals.v_bc_st = accum(totals.v_bc_st, data.v_bc_st);
    totals.v_st = accum(totals.v_st, data.v_icms_st);

    let mut fields_opt: Vec<Option<TaxField>> = vec![
        Some(required_field("orig", Some(data.orig.as_str()))?),
        Some(required_field("CST", data.cst.as_deref())?),
        Some(required_field("modBC", data.mod_bc.as_deref())?),
        Some(required_field("vBC", fc2(data.v_bc).as_deref())?),
        optional_field("pRedBC", fc4(data.p_red_bc).as_deref()),
        Some(required_field("pICMS", fc4(data.p_icms).as_deref())?),
        Some(required_field("vICMS", fc2(data.v_icms).as_deref())?),
    ];

    // ST fields
    let st = collect_st_fields(data)?;
    for f in st {
        fields_opt.push(Some(f));
    }

    // FCP ST fields
    fields_opt.extend(fcp_st_fields(data));

    // pBCOp, UFST
    fields_opt.push(Some(required_field(
        "pBCOp",
        fc4(data.p_bc_op).as_deref(),
    )?));
    fields_opt.push(Some(required_field("UFST", data.uf_st.as_deref())?));

    // Desoneration
    fields_opt.extend(desoneration_fields(data));

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
/// Returns [`FiscalError::MissingRequiredField`] if any required field
/// (e.g. `orig`, `CST`, `vBCSTRet`, `vICMSSTRet`, `vBCSTDest`,
/// `vICMSSTDest`) is `None`.
pub fn build_icms_st_xml(data: &IcmsData) -> Result<(String, IcmsTotals), FiscalError> {
    let mut totals = create_icms_totals();
    totals.v_fcp_st_ret = accum(totals.v_fcp_st_ret, data.v_fcp_st_ret);

    let fields_opt: Vec<Option<TaxField>> = vec![
        Some(required_field("orig", Some(data.orig.as_str()))?),
        Some(required_field("CST", data.cst.as_deref())?),
        Some(required_field("vBCSTRet", fc2(data.v_bc_st_ret).as_deref())?),
        optional_field("pST", fc4(data.p_st).as_deref()),
        optional_field("vICMSSubstituto", fc2(data.v_icms_substituto).as_deref()),
        Some(required_field(
            "vICMSSTRet",
            fc2(data.v_icms_st_ret).as_deref(),
        )?),
        optional_field("vBCFCPSTRet", fc2(data.v_bc_fcp_st_ret).as_deref()),
        optional_field("pFCPSTRet", fc4(data.p_fcp_st_ret).as_deref()),
        optional_field("vFCPSTRet", fc2(data.v_fcp_st_ret).as_deref()),
        Some(required_field(
            "vBCSTDest",
            fc2(data.v_bc_st_dest).as_deref(),
        )?),
        Some(required_field(
            "vICMSSTDest",
            fc2(data.v_icms_st_dest).as_deref(),
        )?),
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
/// Returns [`FiscalError::MissingRequiredField`] if any required field
/// (e.g. `vBCUFDest`, `pICMSUFDest`, `pICMSInter`, `vICMSUFDest`,
/// `vICMSUFRemet`) is `None`.
pub fn build_icms_uf_dest_xml(data: &IcmsData) -> Result<(String, IcmsTotals), FiscalError> {
    let mut totals = create_icms_totals();
    totals.v_icms_uf_dest = accum(totals.v_icms_uf_dest, data.v_icms_uf_dest);
    totals.v_fcp_uf_dest = accum(totals.v_fcp_uf_dest, data.v_fcp_uf_dest);
    totals.v_icms_uf_remet = accum(totals.v_icms_uf_remet, data.v_icms_uf_remet);

    let fields_opt: Vec<Option<TaxField>> = vec![
        Some(required_field(
            "vBCUFDest",
            fc2(data.v_bc_uf_dest).as_deref(),
        )?),
        optional_field("vBCFCPUFDest", fc2(data.v_bc_fcp_uf_dest).as_deref()),
        optional_field("pFCPUFDest", fc4(data.p_fcp_uf_dest).as_deref()),
        Some(required_field(
            "pICMSUFDest",
            fc4(data.p_icms_uf_dest).as_deref(),
        )?),
        Some(required_field(
            "pICMSInter",
            fc4(data.p_icms_inter).as_deref(),
        )?),
        Some(TaxField::new("pICMSInterPart", "100.0000")),
        optional_field("vFCPUFDest", fc2(data.v_fcp_uf_dest).as_deref()),
        Some(required_field(
            "vICMSUFDest",
            fc2(data.v_icms_uf_dest).as_deref(),
        )?),
        Some(required_field(
            "vICMSUFRemet",
            fc2(Some(data.v_icms_uf_remet.unwrap_or(Cents(0)))).as_deref(),
        )?),
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

/// Merge item-level ICMS totals into an accumulator.
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
}

// ── CST builders (regime Normal) ────────────────────────────────────────────

fn calculate_cst(
    data: &IcmsData,
    totals: &mut IcmsTotals,
    _cst: &str,
) -> Result<(String, Vec<TaxField>), FiscalError> {
    // Convert the flat IcmsData into a typed IcmsCst variant, then delegate
    // to build_icms_cst_xml. This gives us one canonical code path for
    // XML generation while keeping the old public API intact.
    let typed = IcmsCst::try_from(data)?;
    build_icms_cst_xml(&typed, totals)
}

// ── CSOSN builders (Simples Nacional) ───────────────────────────────────────

fn calculate_csosn(
    data: &IcmsData,
    totals: &mut IcmsTotals,
    csosn: &str,
) -> Result<(String, Vec<TaxField>), FiscalError> {
    match csosn {
        "101" => calc_csosn_101(data, totals),
        "102" | "103" | "300" | "400" => calc_csosn_102(data, totals),
        "201" => calc_csosn_201(data, totals),
        "202" | "203" => calc_csosn_202(data, totals),
        "500" => calc_csosn_500(data, totals),
        "900" => calc_csosn_900(data, totals),
        _ => Err(FiscalError::UnsupportedIcmsCsosn(csosn.to_string())),
    }
}

/// CSOSN 101 - Tributada pelo Simples Nacional com permissao de credito
fn calc_csosn_101(d: &IcmsData, _t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    let fields = filter_fields(vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CSOSN", d.csosn.as_deref())?),
        Some(required_field("pCredSN", fc4(d.p_cred_sn).as_deref())?),
        Some(required_field(
            "vCredICMSSN",
            fc2(d.v_cred_icms_sn).as_deref(),
        )?),
    ]);

    Ok(("ICMSSN101".to_string(), fields))
}

/// CSOSN 102/103/300/400 - Tributada sem permissao de credito / Imune / Nao tributada
fn calc_csosn_102(d: &IcmsData, _t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    let orig = if d.orig.is_empty() { None } else { Some(d.orig.as_str()) };
    let fields = filter_fields(vec![
        optional_field("orig", orig), // may be null for CRT=4
        Some(required_field("CSOSN", d.csosn.as_deref())?),
    ]);

    Ok(("ICMSSN102".to_string(), fields))
}

/// CSOSN 201 - Tributada com permissao de credito e com cobranca do ICMS por ST
fn calc_csosn_201(d: &IcmsData, t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    t.v_bc_st = accum(t.v_bc_st, d.v_bc_st);
    t.v_st = accum(t.v_st, d.v_icms_st);

    let mut fields_opt: Vec<Option<TaxField>> = vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CSOSN", d.csosn.as_deref())?),
    ];

    // ST fields
    let st = collect_st_fields(d)?;
    for f in st {
        fields_opt.push(Some(f));
    }
    // FCP ST fields
    fields_opt.extend(fcp_st_fields(d));
    // SN credit fields
    fields_opt.push(optional_field("pCredSN", fc4(d.p_cred_sn).as_deref()));
    fields_opt.push(optional_field(
        "vCredICMSSN",
        fc2(d.v_cred_icms_sn).as_deref(),
    ));

    Ok(("ICMSSN201".to_string(), filter_fields(fields_opt)))
}

/// CSOSN 202/203 - Tributada sem permissao de credito e com cobranca do ICMS por ST
fn calc_csosn_202(d: &IcmsData, t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    t.v_bc_st = accum(t.v_bc_st, d.v_bc_st);
    t.v_st = accum(t.v_st, d.v_icms_st);

    let mut fields_opt: Vec<Option<TaxField>> = vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CSOSN", d.csosn.as_deref())?),
    ];

    // ST fields
    let st = collect_st_fields(d)?;
    for f in st {
        fields_opt.push(Some(f));
    }
    // FCP ST fields
    fields_opt.extend(fcp_st_fields(d));

    Ok(("ICMSSN202".to_string(), filter_fields(fields_opt)))
}

/// CSOSN 500 - ICMS cobrado anteriormente por ST ou por antecipacao
fn calc_csosn_500(d: &IcmsData, _t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    let fields = filter_fields(vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CSOSN", d.csosn.as_deref())?),
        optional_field("vBCSTRet", fc2(d.v_bc_st_ret).as_deref()),
        optional_field("pST", fc4(d.p_st).as_deref()),
        optional_field("vICMSSubstituto", fc2(d.v_icms_substituto).as_deref()),
        optional_field("vICMSSTRet", fc2(d.v_icms_st_ret).as_deref()),
        optional_field("vBCFCPSTRet", fc2(d.v_bc_fcp_st_ret).as_deref()),
        optional_field("pFCPSTRet", fc4(d.p_fcp_st_ret).as_deref()),
        optional_field("vFCPSTRet", fc2(d.v_fcp_st_ret).as_deref()),
        optional_field("pRedBCEfet", fc4(d.p_red_bc_efet).as_deref()),
        optional_field("vBCEfet", fc2(d.v_bc_efet).as_deref()),
        optional_field("pICMSEfet", fc4(d.p_icms_efet).as_deref()),
        optional_field("vICMSEfet", fc2(d.v_icms_efet).as_deref()),
    ]);

    Ok(("ICMSSN500".to_string(), fields))
}

/// CSOSN 900 - Outros
fn calc_csosn_900(d: &IcmsData, t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    t.v_bc = accum(t.v_bc, d.v_bc);
    t.v_icms = accum(t.v_icms, d.v_icms);
    t.v_bc_st = accum(t.v_bc_st, d.v_bc_st);
    t.v_st = accum(t.v_st, d.v_icms_st);

    let orig = if d.orig.is_empty() { None } else { Some(d.orig.as_str()) };
    let mut fields_opt: Vec<Option<TaxField>> = vec![
        optional_field("orig", orig), // may be null for CRT=4
        Some(required_field("CSOSN", d.csosn.as_deref())?),
        optional_field("modBC", d.mod_bc.as_deref()),
        optional_field("vBC", fc2(d.v_bc).as_deref()),
        optional_field("pRedBC", fc4(d.p_red_bc).as_deref()),
        optional_field("pICMS", fc4(d.p_icms).as_deref()),
        optional_field("vICMS", fc2(d.v_icms).as_deref()),
        // ST fields are all optional for CSOSN 900
        optional_field("modBCST", d.mod_bc_st.as_deref()),
        optional_field("pMVAST", fc4(d.p_mva_st).as_deref()),
        optional_field("pRedBCST", fc4(d.p_red_bc_st).as_deref()),
        optional_field("vBCST", fc2(d.v_bc_st).as_deref()),
        optional_field("pICMSST", fc4(d.p_icms_st).as_deref()),
        optional_field("vICMSST", fc2(d.v_icms_st).as_deref()),
    ];

    // FCP ST fields
    fields_opt.extend(fcp_st_fields(d));
    // SN credit fields
    fields_opt.push(optional_field("pCredSN", fc4(d.p_cred_sn).as_deref()));
    fields_opt.push(optional_field(
        "vCredICMSSN",
        fc2(d.v_cred_icms_sn).as_deref(),
    ));

    Ok(("ICMSSN900".to_string(), filter_fields(fields_opt)))
}
