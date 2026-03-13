//! IcmsCsosn enum — Simples Nacional (CRT 1/2) + XML builder.

use crate::FiscalError;
use crate::newtypes::{Cents, Rate};
use crate::tax_element::{TaxField, filter_fields, optional_field};

use super::totals::IcmsTotals;
use super::{accum, fc2, fc4};

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
