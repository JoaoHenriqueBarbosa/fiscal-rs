use crate::FiscalError;
use crate::newtypes::Rate;
use crate::tax_icms::{IcmsCsosn, IcmsCst, IcmsVariant};
use crate::types::InvoiceItemData;

/// Map an invoice item's ICMS fields to the correct typed [`IcmsVariant`].
pub(super) fn build_icms_variant(
    item: &InvoiceItemData,
    is_simples: bool,
) -> Result<IcmsVariant, FiscalError> {
    let orig = item.orig.clone().unwrap_or_else(|| "0".to_string());

    if is_simples {
        let csosn_code = if item.icms_cst.is_empty() {
            "102"
        } else {
            item.icms_cst.as_str()
        };

        let csosn = match csosn_code {
            "101" => IcmsCsosn::Csosn101 {
                orig,
                csosn: csosn_code.to_string(),
                p_cred_sn: item.icms_p_cred_sn.ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "pCredSN".to_string(),
                    }
                })?,
                v_cred_icms_sn: item.icms_v_cred_icms_sn.ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "vCredICMSSN".to_string(),
                    }
                })?,
            },
            "102" => IcmsCsosn::Csosn102 {
                orig,
                csosn: csosn_code.to_string(),
            },
            "103" => IcmsCsosn::Csosn103 {
                orig,
                csosn: csosn_code.to_string(),
            },
            "300" => IcmsCsosn::Csosn300 {
                orig,
                csosn: csosn_code.to_string(),
            },
            "400" => IcmsCsosn::Csosn400 {
                orig,
                csosn: csosn_code.to_string(),
            },
            "201" => IcmsCsosn::Csosn201 {
                orig,
                csosn: csosn_code.to_string(),
                mod_bc_st: item.icms_mod_bc_st.map(|v| v.to_string()).ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "modBCST".to_string(),
                    }
                })?,
                p_mva_st: item.icms_p_mva_st,
                p_red_bc_st: item.icms_red_bc_st,
                v_bc_st: item
                    .icms_v_bc_st
                    .ok_or_else(|| FiscalError::MissingRequiredField {
                        field: "vBCST".to_string(),
                    })?,
                p_icms_st: item.icms_p_icms_st.ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "pICMSST".to_string(),
                    }
                })?,
                v_icms_st: item.icms_v_icms_st.ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "vICMSST".to_string(),
                    }
                })?,
                v_bc_fcp_st: item.icms_v_bc_fcp_st,
                p_fcp_st: item.icms_p_fcp_st,
                v_fcp_st: item.icms_v_fcp_st,
                p_cred_sn: item.icms_p_cred_sn,
                v_cred_icms_sn: item.icms_v_cred_icms_sn,
            },
            "202" => IcmsCsosn::Csosn202 {
                orig,
                csosn: csosn_code.to_string(),
                mod_bc_st: item.icms_mod_bc_st.map(|v| v.to_string()).ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "modBCST".to_string(),
                    }
                })?,
                p_mva_st: item.icms_p_mva_st,
                p_red_bc_st: item.icms_red_bc_st,
                v_bc_st: item
                    .icms_v_bc_st
                    .ok_or_else(|| FiscalError::MissingRequiredField {
                        field: "vBCST".to_string(),
                    })?,
                p_icms_st: item.icms_p_icms_st.ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "pICMSST".to_string(),
                    }
                })?,
                v_icms_st: item.icms_v_icms_st.ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "vICMSST".to_string(),
                    }
                })?,
                v_bc_fcp_st: item.icms_v_bc_fcp_st,
                p_fcp_st: item.icms_p_fcp_st,
                v_fcp_st: item.icms_v_fcp_st,
            },
            "203" => IcmsCsosn::Csosn203 {
                orig,
                csosn: csosn_code.to_string(),
                mod_bc_st: item.icms_mod_bc_st.map(|v| v.to_string()).ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "modBCST".to_string(),
                    }
                })?,
                p_mva_st: item.icms_p_mva_st,
                p_red_bc_st: item.icms_red_bc_st,
                v_bc_st: item
                    .icms_v_bc_st
                    .ok_or_else(|| FiscalError::MissingRequiredField {
                        field: "vBCST".to_string(),
                    })?,
                p_icms_st: item.icms_p_icms_st.ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "pICMSST".to_string(),
                    }
                })?,
                v_icms_st: item.icms_v_icms_st.ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "vICMSST".to_string(),
                    }
                })?,
                v_bc_fcp_st: item.icms_v_bc_fcp_st,
                p_fcp_st: item.icms_p_fcp_st,
                v_fcp_st: item.icms_v_fcp_st,
            },
            "500" => IcmsCsosn::Csosn500 {
                orig,
                csosn: csosn_code.to_string(),
                v_bc_st_ret: None,
                p_st: None,
                v_icms_substituto: item.icms_v_icms_substituto,
                v_icms_st_ret: None,
                v_bc_fcp_st_ret: None,
                p_fcp_st_ret: None,
                v_fcp_st_ret: None,
                p_red_bc_efet: None,
                v_bc_efet: None,
                p_icms_efet: None,
                v_icms_efet: None,
            },
            "900" => IcmsCsosn::Csosn900 {
                orig,
                csosn: csosn_code.to_string(),
                mod_bc: item.icms_mod_bc.map(|v| v.to_string()),
                v_bc: Some(item.total_price),
                p_red_bc: item.icms_red_bc,
                p_icms: Some(item.icms_rate),
                v_icms: Some(item.icms_amount),
                mod_bc_st: item.icms_mod_bc_st.map(|v| v.to_string()),
                p_mva_st: item.icms_p_mva_st,
                p_red_bc_st: item.icms_red_bc_st,
                v_bc_st: item.icms_v_bc_st,
                p_icms_st: item.icms_p_icms_st,
                v_icms_st: item.icms_v_icms_st,
                v_bc_fcp_st: item.icms_v_bc_fcp_st,
                p_fcp_st: item.icms_p_fcp_st,
                v_fcp_st: item.icms_v_fcp_st,
                p_cred_sn: item.icms_p_cred_sn,
                v_cred_icms_sn: item.icms_v_cred_icms_sn,
            },
            other => return Err(FiscalError::UnsupportedIcmsCsosn(other.to_string())),
        };
        Ok(csosn.into())
    } else {
        let cst_code = item.icms_cst.as_str();
        let cst = match cst_code {
            "00" => IcmsCst::Cst00 {
                orig,
                mod_bc: item
                    .icms_mod_bc
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "3".to_string()),
                v_bc: item.total_price,
                p_icms: item.icms_rate,
                v_icms: item.icms_amount,
                p_fcp: item.icms_p_fcp,
                v_fcp: item.icms_v_fcp,
            },
            "10" => IcmsCst::Cst10 {
                orig,
                mod_bc: item
                    .icms_mod_bc
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "3".to_string()),
                v_bc: item.total_price,
                p_icms: item.icms_rate,
                v_icms: item.icms_amount,
                v_bc_fcp: item.icms_v_bc_fcp,
                p_fcp: item.icms_p_fcp,
                v_fcp: item.icms_v_fcp,
                mod_bc_st: item.icms_mod_bc_st.map(|v| v.to_string()).ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "modBCST".to_string(),
                    }
                })?,
                p_mva_st: item.icms_p_mva_st,
                p_red_bc_st: item.icms_red_bc_st,
                v_bc_st: item
                    .icms_v_bc_st
                    .ok_or_else(|| FiscalError::MissingRequiredField {
                        field: "vBCST".to_string(),
                    })?,
                p_icms_st: item.icms_p_icms_st.ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "pICMSST".to_string(),
                    }
                })?,
                v_icms_st: item.icms_v_icms_st.ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "vICMSST".to_string(),
                    }
                })?,
                v_bc_fcp_st: item.icms_v_bc_fcp_st,
                p_fcp_st: item.icms_p_fcp_st,
                v_fcp_st: item.icms_v_fcp_st,
                v_icms_st_deson: None,
                mot_des_icms_st: None,
            },
            "20" => IcmsCst::Cst20 {
                orig,
                mod_bc: item
                    .icms_mod_bc
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "3".to_string()),
                p_red_bc: item.icms_red_bc.unwrap_or(Rate(0)),
                v_bc: item.total_price,
                p_icms: item.icms_rate,
                v_icms: item.icms_amount,
                v_bc_fcp: item.icms_v_bc_fcp,
                p_fcp: item.icms_p_fcp,
                v_fcp: item.icms_v_fcp,
                v_icms_deson: item.icms_v_icms_deson,
                mot_des_icms: item.icms_mot_des_icms.map(|v| v.to_string()),
                ind_deduz_deson: item.icms_ind_deduz_deson.clone(),
            },
            "30" => IcmsCst::Cst30 {
                orig,
                mod_bc_st: item.icms_mod_bc_st.map(|v| v.to_string()).ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "modBCST".to_string(),
                    }
                })?,
                p_mva_st: item.icms_p_mva_st,
                p_red_bc_st: item.icms_red_bc_st,
                v_bc_st: item
                    .icms_v_bc_st
                    .ok_or_else(|| FiscalError::MissingRequiredField {
                        field: "vBCST".to_string(),
                    })?,
                p_icms_st: item.icms_p_icms_st.ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "pICMSST".to_string(),
                    }
                })?,
                v_icms_st: item.icms_v_icms_st.ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "vICMSST".to_string(),
                    }
                })?,
                v_bc_fcp_st: item.icms_v_bc_fcp_st,
                p_fcp_st: item.icms_p_fcp_st,
                v_fcp_st: item.icms_v_fcp_st,
                v_icms_deson: item.icms_v_icms_deson,
                mot_des_icms: item.icms_mot_des_icms.map(|v| v.to_string()),
                ind_deduz_deson: item.icms_ind_deduz_deson.clone(),
            },
            "40" => IcmsCst::Cst40 {
                orig,
                v_icms_deson: item.icms_v_icms_deson,
                mot_des_icms: item.icms_mot_des_icms.map(|v| v.to_string()),
                ind_deduz_deson: item.icms_ind_deduz_deson.clone(),
            },
            "41" => IcmsCst::Cst41 {
                orig,
                v_icms_deson: item.icms_v_icms_deson,
                mot_des_icms: item.icms_mot_des_icms.map(|v| v.to_string()),
                ind_deduz_deson: item.icms_ind_deduz_deson.clone(),
            },
            "50" => IcmsCst::Cst50 {
                orig,
                v_icms_deson: item.icms_v_icms_deson,
                mot_des_icms: item.icms_mot_des_icms.map(|v| v.to_string()),
                ind_deduz_deson: item.icms_ind_deduz_deson.clone(),
            },
            "51" => IcmsCst::Cst51 {
                orig,
                mod_bc: item.icms_mod_bc.map(|v| v.to_string()),
                p_red_bc: item.icms_red_bc,
                c_benef_rbc: None,
                v_bc: Some(item.total_price),
                p_icms: Some(item.icms_rate),
                v_icms_op: None,
                p_dif: None,
                v_icms_dif: None,
                v_icms: Some(item.icms_amount),
                v_bc_fcp: item.icms_v_bc_fcp,
                p_fcp: item.icms_p_fcp,
                v_fcp: item.icms_v_fcp,
                p_fcp_dif: None,
                v_fcp_dif: None,
                v_fcp_efet: None,
            },
            "60" => IcmsCst::Cst60 {
                orig,
                v_bc_st_ret: None,
                p_st: None,
                v_icms_substituto: item.icms_v_icms_substituto,
                v_icms_st_ret: None,
                v_bc_fcp_st_ret: None,
                p_fcp_st_ret: None,
                v_fcp_st_ret: None,
                p_red_bc_efet: None,
                v_bc_efet: None,
                p_icms_efet: None,
                v_icms_efet: None,
            },
            "70" => IcmsCst::Cst70 {
                orig,
                mod_bc: item
                    .icms_mod_bc
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "3".to_string()),
                p_red_bc: item.icms_red_bc.unwrap_or(Rate(0)),
                v_bc: item.total_price,
                p_icms: item.icms_rate,
                v_icms: item.icms_amount,
                v_bc_fcp: item.icms_v_bc_fcp,
                p_fcp: item.icms_p_fcp,
                v_fcp: item.icms_v_fcp,
                mod_bc_st: item.icms_mod_bc_st.map(|v| v.to_string()).ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "modBCST".to_string(),
                    }
                })?,
                p_mva_st: item.icms_p_mva_st,
                p_red_bc_st: item.icms_red_bc_st,
                v_bc_st: item
                    .icms_v_bc_st
                    .ok_or_else(|| FiscalError::MissingRequiredField {
                        field: "vBCST".to_string(),
                    })?,
                p_icms_st: item.icms_p_icms_st.ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "pICMSST".to_string(),
                    }
                })?,
                v_icms_st: item.icms_v_icms_st.ok_or_else(|| {
                    FiscalError::MissingRequiredField {
                        field: "vICMSST".to_string(),
                    }
                })?,
                v_bc_fcp_st: item.icms_v_bc_fcp_st,
                p_fcp_st: item.icms_p_fcp_st,
                v_fcp_st: item.icms_v_fcp_st,
                v_icms_deson: item.icms_v_icms_deson,
                mot_des_icms: item.icms_mot_des_icms.map(|v| v.to_string()),
                ind_deduz_deson: item.icms_ind_deduz_deson.clone(),
                v_icms_st_deson: None,
                mot_des_icms_st: None,
            },
            "90" => IcmsCst::Cst90 {
                orig,
                mod_bc: item.icms_mod_bc.map(|v| v.to_string()),
                v_bc: Some(item.total_price),
                p_red_bc: item.icms_red_bc,
                c_benef_rbc: None,
                p_icms: Some(item.icms_rate),
                v_icms_op: None,
                p_dif: None,
                v_icms_dif: None,
                v_icms: Some(item.icms_amount),
                v_bc_fcp: item.icms_v_bc_fcp,
                p_fcp: item.icms_p_fcp,
                v_fcp: item.icms_v_fcp,
                p_fcp_dif: None,
                v_fcp_dif: None,
                v_fcp_efet: None,
                mod_bc_st: item.icms_mod_bc_st.map(|v| v.to_string()),
                p_mva_st: item.icms_p_mva_st,
                p_red_bc_st: item.icms_red_bc_st,
                v_bc_st: item.icms_v_bc_st,
                p_icms_st: item.icms_p_icms_st,
                v_icms_st: item.icms_v_icms_st,
                v_bc_fcp_st: item.icms_v_bc_fcp_st,
                p_fcp_st: item.icms_p_fcp_st,
                v_fcp_st: item.icms_v_fcp_st,
                v_icms_deson: item.icms_v_icms_deson,
                mot_des_icms: item.icms_mot_des_icms.map(|v| v.to_string()),
                ind_deduz_deson: item.icms_ind_deduz_deson.clone(),
                v_icms_st_deson: None,
                mot_des_icms_st: None,
            },
            other => return Err(FiscalError::UnsupportedIcmsCst(other.to_string())),
        };
        Ok(cst.into())
    }
}
