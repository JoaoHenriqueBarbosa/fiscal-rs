//! XML builder for IcmsCst variants.

use crate::FiscalError;
use crate::tax_element::{TaxField, filter_fields, optional_field, required_field};

use super::cst::IcmsCst;
use super::totals::IcmsTotals;
use super::{accum, accum_raw, fc2, fc4, fc4_raw};

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
