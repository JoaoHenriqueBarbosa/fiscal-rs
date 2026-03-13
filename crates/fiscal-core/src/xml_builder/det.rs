//! Build `<det>` (item detail) elements of the NF-e XML.

use crate::FiscalError;
use crate::format_utils::{format_cents, format_decimal};
use crate::newtypes::{Cents, Rate, Rate4};
use crate::tax_icms::{self, IcmsCsosn, IcmsCst, IcmsTotals, IcmsVariant};
use crate::tax_issqn;
use crate::tax_pis_cofins_ipi::{self, CofinsData, IiData, IpiData, PisData};
use crate::types::{
    CombData, InvoiceBuildData, InvoiceItemData, InvoiceModel, SefazEnvironment, TaxRegime,
};
use crate::xml_utils::{TagContent, tag};

/// Constant used when emitting NFC-e in homologation environment (first item only).
const HOMOLOGATION_XPROD: &str =
    "NOTA FISCAL EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL";

/// Result from building a single `<det>` element.
#[derive(Debug, Clone)]
pub struct DetResult {
    /// The serialised `<det>` XML string.
    pub xml: String,
    /// Accumulated ICMS totals contributed by this item.
    pub icms_totals: IcmsTotals,
    /// IPI value in cents contributed by this item.
    pub v_ipi: i64,
    /// PIS value in cents contributed by this item.
    pub v_pis: i64,
    /// COFINS value in cents contributed by this item.
    pub v_cofins: i64,
    /// II (import tax) value in cents contributed by this item.
    pub v_ii: i64,
    /// Freight value in cents for this item.
    pub v_frete: i64,
    /// Insurance value in cents for this item.
    pub v_seg: i64,
    /// Discount value in cents for this item.
    pub v_desc: i64,
    /// Other expenses value in cents for this item.
    pub v_outro: i64,
    /// Whether this item counts towards the invoice total (indTot).
    pub ind_tot: u8,
    /// Approximate total tax for this item (`vTotTrib`). Optional.
    pub v_tot_trib: i64,
    /// Whether this item uses ISSQN instead of ICMS.
    pub has_issqn: bool,
}

/// Map an invoice item's ICMS fields to the correct typed [`IcmsVariant`].
fn build_icms_variant(
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
            "102" | "103" | "300" | "400" => IcmsCsosn::Csosn102 {
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
            "202" | "203" => IcmsCsosn::Csosn202 {
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
                ind_deduz_deson: None,
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
                ind_deduz_deson: None,
            },
            "40" => IcmsCst::Cst40 {
                orig,
                v_icms_deson: item.icms_v_icms_deson,
                mot_des_icms: item.icms_mot_des_icms.map(|v| v.to_string()),
                ind_deduz_deson: None,
            },
            "41" => IcmsCst::Cst41 {
                orig,
                v_icms_deson: item.icms_v_icms_deson,
                mot_des_icms: item.icms_mot_des_icms.map(|v| v.to_string()),
                ind_deduz_deson: None,
            },
            "50" => IcmsCst::Cst50 {
                orig,
                v_icms_deson: item.icms_v_icms_deson,
                mot_des_icms: item.icms_mot_des_icms.map(|v| v.to_string()),
                ind_deduz_deson: None,
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
                ind_deduz_deson: None,
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
                ind_deduz_deson: None,
                v_icms_st_deson: None,
                mot_des_icms_st: None,
            },
            other => return Err(FiscalError::UnsupportedIcmsCst(other.to_string())),
        };
        Ok(cst.into())
    }
}

/// Build a `<det nItem="N">` element for one invoice item.
pub(crate) fn build_det(
    item: &InvoiceItemData,
    data: &InvoiceBuildData,
) -> Result<DetResult, FiscalError> {
    let is_simples = matches!(
        data.issuer.tax_regime,
        TaxRegime::SimplesNacional | TaxRegime::SimplesExcess
    );

    let has_issqn = item.issqn.is_some();

    // Build ICMS (skipped when item has ISSQN)
    let mut icms_totals = IcmsTotals::default();
    let icms_xml = if has_issqn {
        String::new()
    } else {
        let icms_variant = build_icms_variant(item, is_simples)?;
        tax_icms::build_icms_xml(&icms_variant, &mut icms_totals)?
    };

    // Build ISSQN (optional — only when item.issqn is set)
    let issqn_xml = if let Some(ref issqn_data) = item.issqn {
        tax_issqn::build_issqn_xml(issqn_data)
    } else {
        String::new()
    };

    // Build PIS
    let pis_xml = tax_pis_cofins_ipi::build_pis_xml(&PisData {
        cst: item.pis_cst.clone(),
        v_bc: item.pis_v_bc.or(Some(Cents(0))),
        p_pis: item.pis_p_pis.or(Some(Rate4(0))),
        v_pis: item.pis_v_pis.or(Some(Cents(0))),
        q_bc_prod: item.pis_q_bc_prod,
        v_aliq_prod: item.pis_v_aliq_prod,
    });

    // Build COFINS
    let cofins_xml = tax_pis_cofins_ipi::build_cofins_xml(&CofinsData {
        cst: item.cofins_cst.clone(),
        v_bc: item.cofins_v_bc.or(Some(Cents(0))),
        p_cofins: item.cofins_p_cofins.or(Some(Rate4(0))),
        v_cofins: item.cofins_v_cofins.or(Some(Cents(0))),
        q_bc_prod: item.cofins_q_bc_prod,
        v_aliq_prod: item.cofins_v_aliq_prod,
    });

    // Build IPI (optional)
    let mut ipi_xml = String::new();
    let mut v_ipi = 0i64;
    if let Some(ref ipi_cst) = item.ipi_cst {
        ipi_xml = tax_pis_cofins_ipi::build_ipi_xml(&IpiData {
            cst: ipi_cst.clone(),
            c_enq: item.ipi_c_enq.clone().unwrap_or_else(|| "999".to_string()),
            v_bc: item.ipi_v_bc,
            p_ipi: item.ipi_p_ipi,
            v_ipi: item.ipi_v_ipi,
            q_unid: item.ipi_q_unid,
            v_unid: item.ipi_v_unid,
            ..IpiData::default()
        });
        v_ipi = item.ipi_v_ipi.map(|c| c.0).unwrap_or(0);
    }

    // Build II (optional)
    let mut ii_xml = String::new();
    let mut v_ii = 0i64;
    if let Some(ii_vbc) = item.ii_v_bc {
        ii_xml = tax_pis_cofins_ipi::build_ii_xml(&IiData {
            v_bc: ii_vbc,
            v_desp_adu: item.ii_v_desp_adu.unwrap_or(Cents(0)),
            v_ii: item.ii_v_ii.unwrap_or(Cents(0)),
            v_iof: item.ii_v_iof.unwrap_or(Cents(0)),
        });
        v_ii = item.ii_v_ii.map(|c| c.0).unwrap_or(0);
    }

    // Build prod options (rastro, veicProd, med, arma, comb, nRECOPI)
    let prod_options = build_prod_options(item);

    // Build det-level extras (infAdProd, obsItem, DFeReferenciado)
    let det_extras = build_det_extras(item);

    // Assemble imposto
    let mut imposto_children: Vec<String> = Vec::new();
    if !icms_xml.is_empty() {
        imposto_children.push(icms_xml);
    }
    if !ipi_xml.is_empty() {
        imposto_children.push(ipi_xml);
    }
    imposto_children.push(pis_xml);
    imposto_children.push(cofins_xml);
    if !ii_xml.is_empty() {
        imposto_children.push(ii_xml);
    }
    if !issqn_xml.is_empty() {
        imposto_children.push(issqn_xml);
    }

    // Assemble prod
    let fc2 = |c: i64| format_cents(c, 2);
    let fc10 = |c: i64| format_cents(c, 10);
    let fd4 = |v: f64| format_decimal(v, 4);

    let mut prod_children = vec![
        tag("cProd", &[], TagContent::Text(&item.product_code)),
        tag(
            "cEAN",
            &[],
            TagContent::Text(item.c_ean.as_deref().unwrap_or("SEM GTIN")),
        ),
        tag(
            "xProd",
            &[],
            TagContent::Text(
                // PHP substitutes xProd for item 1 of NFC-e in homologation
                if item.item_number == 1
                    && data.environment == SefazEnvironment::Homologation
                    && data.model == InvoiceModel::Nfce
                {
                    HOMOLOGATION_XPROD
                } else {
                    &item.description
                },
            ),
        ),
        tag("NCM", &[], TagContent::Text(&item.ncm)),
    ];
    if let Some(ref cest) = item.cest {
        prod_children.push(tag("CEST", &[], TagContent::Text(cest)));
        if let Some(ref ind) = item.cest_ind_escala {
            prod_children.push(tag("indEscala", &[], TagContent::Text(ind)));
        }
        if let Some(ref fab) = item.cest_cnpj_fab {
            prod_children.push(tag("CNPJFab", &[], TagContent::Text(fab)));
        }
    }
    if let Some(ref cb) = item.c_benef {
        prod_children.push(tag("cBenef", &[], TagContent::Text(cb)));
    }
    if let Some(ref ex) = item.extipi {
        prod_children.push(tag("EXTIPI", &[], TagContent::Text(ex)));
    }
    prod_children.extend([
        tag("CFOP", &[], TagContent::Text(&item.cfop)),
        tag("uCom", &[], TagContent::Text(&item.unit_of_measure)),
        tag("qCom", &[], TagContent::Text(&fd4(item.quantity))),
        tag("vUnCom", &[], TagContent::Text(&fc10(item.unit_price.0))),
        tag("vProd", &[], TagContent::Text(&fc2(item.total_price.0))),
        tag(
            "cEANTrib",
            &[],
            TagContent::Text(item.c_ean_trib.as_deref().unwrap_or("SEM GTIN")),
        ),
        tag("uTrib", &[], TagContent::Text(&item.unit_of_measure)),
        tag("qTrib", &[], TagContent::Text(&fd4(item.quantity))),
        tag("vUnTrib", &[], TagContent::Text(&fc10(item.unit_price.0))),
    ]);
    if let Some(v) = item.v_frete {
        prod_children.push(tag("vFrete", &[], TagContent::Text(&fc2(v.0))));
    }
    if let Some(v) = item.v_seg {
        prod_children.push(tag("vSeg", &[], TagContent::Text(&fc2(v.0))));
    }
    if let Some(v) = item.v_desc {
        prod_children.push(tag("vDesc", &[], TagContent::Text(&fc2(v.0))));
    }
    if let Some(v) = item.v_outro {
        prod_children.push(tag("vOutro", &[], TagContent::Text(&fc2(v.0))));
    }
    let ind_tot_str = match item.ind_tot {
        Some(v) => v.to_string(),
        None => "1".to_string(),
    };
    prod_children.push(tag("indTot", &[], TagContent::Text(&ind_tot_str)));
    if let Some(ref xped) = item.x_ped {
        prod_children.push(tag("xPed", &[], TagContent::Text(xped)));
    }
    if let Some(ref nip) = item.n_item_ped {
        prod_children.push(tag("nItemPed", &[], TagContent::Text(nip)));
    }
    if let Some(ref nfci) = item.n_fci {
        prod_children.push(tag("nFCI", &[], TagContent::Text(nfci)));
    }
    prod_children.extend(prod_options);

    // Assemble det
    let nitem = item.item_number.to_string();
    let mut det_children = vec![
        tag("prod", &[], TagContent::Children(prod_children)),
        tag("imposto", &[], TagContent::Children(imposto_children)),
    ];
    det_children.extend(det_extras);

    let xml = tag(
        "det",
        &[("nItem", &nitem)],
        TagContent::Children(det_children),
    );

    Ok(DetResult {
        xml,
        icms_totals,
        v_ipi,
        v_pis: item.pis_v_pis.map(|c| c.0).unwrap_or(0),
        v_cofins: item.cofins_v_cofins.map(|c| c.0).unwrap_or(0),
        v_ii,
        v_frete: item.v_frete.map(|c| c.0).unwrap_or(0),
        v_seg: item.v_seg.map(|c| c.0).unwrap_or(0),
        v_desc: item.v_desc.map(|c| c.0).unwrap_or(0),
        v_outro: item.v_outro.map(|c| c.0).unwrap_or(0),
        ind_tot: item.ind_tot.unwrap_or(1),
        v_tot_trib: item.v_tot_trib.map(|c| c.0).unwrap_or(0),
        has_issqn,
    })
}

fn build_prod_options(item: &InvoiceItemData) -> Vec<String> {
    let mut opts = Vec::new();

    // rastro (batch tracking)
    if let Some(ref rastros) = item.rastro {
        for r in rastros.iter().take(500) {
            let mut rastro_children = vec![
                tag("nLote", &[], TagContent::Text(&r.n_lote)),
                tag("qLote", &[], TagContent::Text(&format_decimal(r.q_lote, 3))),
                tag("dFab", &[], TagContent::Text(&r.d_fab)),
                tag("dVal", &[], TagContent::Text(&r.d_val)),
            ];
            if let Some(ref agreg) = r.c_agreg {
                rastro_children.push(tag("cAgreg", &[], TagContent::Text(agreg)));
            }
            opts.push(tag("rastro", &[], TagContent::Children(rastro_children)));
        }
    }

    // CHOICE group: veicProd, med, arma, nRECOPI (mutually exclusive)
    if let Some(ref v) = item.veic_prod {
        opts.push(tag(
            "veicProd",
            &[],
            TagContent::Children(vec![
                tag("tpOp", &[], TagContent::Text(&v.tp_op)),
                tag("chassi", &[], TagContent::Text(&v.chassi)),
                tag("cCor", &[], TagContent::Text(&v.c_cor)),
                tag("xCor", &[], TagContent::Text(&v.x_cor)),
                tag("pot", &[], TagContent::Text(&v.pot)),
                tag("cilin", &[], TagContent::Text(&v.cilin)),
                tag("pesoL", &[], TagContent::Text(&v.peso_l)),
                tag("pesoB", &[], TagContent::Text(&v.peso_b)),
                tag("nSerie", &[], TagContent::Text(&v.n_serie)),
                tag("tpComb", &[], TagContent::Text(&v.tp_comb)),
                tag("nMotor", &[], TagContent::Text(&v.n_motor)),
                tag("CMT", &[], TagContent::Text(&v.cmt)),
                tag("dist", &[], TagContent::Text(&v.dist)),
                tag("anoMod", &[], TagContent::Text(&v.ano_mod)),
                tag("anoFab", &[], TagContent::Text(&v.ano_fab)),
                tag("tpPint", &[], TagContent::Text(&v.tp_pint)),
                tag("tpVeic", &[], TagContent::Text(&v.tp_veic)),
                tag("espVeic", &[], TagContent::Text(&v.esp_veic)),
                tag("VIN", &[], TagContent::Text(&v.vin)),
                tag("condVeic", &[], TagContent::Text(&v.cond_veic)),
                tag("cMod", &[], TagContent::Text(&v.c_mod)),
                tag("cCorDENATRAN", &[], TagContent::Text(&v.c_cor_denatran)),
                tag("lota", &[], TagContent::Text(&v.lota)),
                tag("tpRest", &[], TagContent::Text(&v.tp_rest)),
            ]),
        ));
    } else if let Some(ref m) = item.med {
        let mut med_children = Vec::new();
        if let Some(ref code) = m.c_prod_anvisa {
            med_children.push(tag("cProdANVISA", &[], TagContent::Text(code)));
        }
        if let Some(ref reason) = m.x_motivo_isencao {
            med_children.push(tag("xMotivoIsencao", &[], TagContent::Text(reason)));
        }
        med_children.push(tag(
            "vPMC",
            &[],
            TagContent::Text(&format_cents(m.v_pmc.0, 2)),
        ));
        opts.push(tag("med", &[], TagContent::Children(med_children)));
    } else if let Some(ref arms) = item.arma {
        for a in arms.iter().take(500) {
            opts.push(tag(
                "arma",
                &[],
                TagContent::Children(vec![
                    tag("tpArma", &[], TagContent::Text(&a.tp_arma)),
                    tag("nSerie", &[], TagContent::Text(&a.n_serie)),
                    tag("nCano", &[], TagContent::Text(&a.n_cano)),
                    tag("descr", &[], TagContent::Text(&a.descr)),
                ]),
            ));
        }
    } else if let Some(ref recopi) = item.n_recopi {
        if !recopi.is_empty() {
            opts.push(tag("nRECOPI", &[], TagContent::Text(recopi)));
        }
    }

    // comb — fuel product data (after the CHOICE group, per NF-e schema order)
    if let Some(ref comb) = item.comb {
        opts.push(build_comb_xml(comb));
    }

    opts
}

/// Build the `<comb>` element for fuel products.
///
/// Follows the PHP sped-nfe `tagcomb` / `tagencerrante` / `tagorigComb`
/// structure exactly: cProdANP, descANP, pGLP, pGNn, pGNi, vPart, CODIF,
/// qTemp, UFCons, CIDE, encerrante, pBio, origComb[].
fn build_comb_xml(comb: &CombData) -> String {
    let mut children = vec![
        tag("cProdANP", &[], TagContent::Text(&comb.c_prod_anp)),
        tag("descANP", &[], TagContent::Text(&comb.desc_anp)),
    ];

    if let Some(ref v) = comb.p_glp {
        children.push(tag("pGLP", &[], TagContent::Text(v)));
    }
    if let Some(ref v) = comb.p_gn_n {
        children.push(tag("pGNn", &[], TagContent::Text(v)));
    }
    if let Some(ref v) = comb.p_gn_i {
        children.push(tag("pGNi", &[], TagContent::Text(v)));
    }
    if let Some(ref v) = comb.v_part {
        children.push(tag("vPart", &[], TagContent::Text(v)));
    }
    if let Some(ref v) = comb.codif {
        children.push(tag("CODIF", &[], TagContent::Text(v)));
    }
    if let Some(ref v) = comb.q_temp {
        children.push(tag("qTemp", &[], TagContent::Text(v)));
    }

    children.push(tag("UFCons", &[], TagContent::Text(&comb.uf_cons)));

    // CIDE (conditional — only when qBCProd is present)
    if let Some(ref cide) = comb.cide {
        let cide_children = vec![
            tag("qBCProd", &[], TagContent::Text(&cide.q_bc_prod)),
            tag("vAliqProd", &[], TagContent::Text(&cide.v_aliq_prod)),
            tag("vCIDE", &[], TagContent::Text(&cide.v_cide)),
        ];
        children.push(tag("CIDE", &[], TagContent::Children(cide_children)));
    }

    // encerrante
    if let Some(ref enc) = comb.encerrante {
        let mut enc_children = vec![tag("nBico", &[], TagContent::Text(&enc.n_bico))];
        if let Some(ref bomba) = enc.n_bomba {
            enc_children.push(tag("nBomba", &[], TagContent::Text(bomba)));
        }
        enc_children.push(tag("nTanque", &[], TagContent::Text(&enc.n_tanque)));
        enc_children.push(tag("vEncIni", &[], TagContent::Text(&enc.v_enc_ini)));
        enc_children.push(tag("vEncFin", &[], TagContent::Text(&enc.v_enc_fin)));
        children.push(tag("encerrante", &[], TagContent::Children(enc_children)));
    }

    // pBio
    if let Some(ref v) = comb.p_bio {
        children.push(tag("pBio", &[], TagContent::Text(v)));
    }

    // origComb (may be multiple)
    if let Some(ref origins) = comb.orig_comb {
        for orig in origins {
            let orig_children = vec![
                tag("indImport", &[], TagContent::Text(&orig.ind_import)),
                tag("cUFOrig", &[], TagContent::Text(&orig.c_uf_orig)),
                tag("pOrig", &[], TagContent::Text(&orig.p_orig)),
            ];
            children.push(tag("origComb", &[], TagContent::Children(orig_children)));
        }
    }

    tag("comb", &[], TagContent::Children(children))
}

fn build_det_extras(item: &InvoiceItemData) -> Vec<String> {
    let mut extras = Vec::new();

    if let Some(ref info) = item.inf_ad_prod {
        extras.push(tag("infAdProd", &[], TagContent::Text(info)));
    }

    if let Some(ref obs) = item.obs_item {
        let mut obs_children = Vec::new();
        if let Some(ref cont) = obs.obs_cont {
            obs_children.push(tag(
                "obsCont",
                &[("xCampo", &cont.x_campo)],
                TagContent::Children(vec![tag("xTexto", &[], TagContent::Text(&cont.x_texto))]),
            ));
        }
        if let Some(ref fisco) = obs.obs_fisco {
            obs_children.push(tag(
                "obsFisco",
                &[("xCampo", &fisco.x_campo)],
                TagContent::Children(vec![tag("xTexto", &[], TagContent::Text(&fisco.x_texto))]),
            ));
        }
        extras.push(tag("obsItem", &[], TagContent::Children(obs_children)));
    }

    if let Some(ref dfe) = item.dfe_referenciado {
        let mut dfe_children = vec![tag("chaveAcesso", &[], TagContent::Text(&dfe.chave_acesso))];
        if let Some(ref n) = dfe.n_item {
            dfe_children.push(tag("nItem", &[], TagContent::Text(n)));
        }
        extras.push(tag(
            "DFeReferenciado",
            &[],
            TagContent::Children(dfe_children),
        ));
    }

    extras
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::newtypes::{Cents, IbgeCode, Rate};
    use crate::tax_issqn::IssqnData as TaxIssqnData;
    use crate::types::{
        CideData, CombData, EncerranteData, InvoiceItemData, InvoiceModel, IssuerData,
        OrigCombData, SefazEnvironment, TaxRegime,
    };

    fn sample_build_data() -> InvoiceBuildData {
        let issuer = IssuerData::new(
            "12345678000199",
            "123456789",
            "Test Company",
            TaxRegime::SimplesNacional,
            "SP",
            IbgeCode("3550308".to_string()),
            "Sao Paulo",
            "Av Paulista",
            "1000",
            "Bela Vista",
            "01310100",
        );

        InvoiceBuildData {
            model: InvoiceModel::Nfe,
            series: 1,
            number: 1,
            emission_type: crate::types::EmissionType::Normal,
            environment: SefazEnvironment::Homologation,
            issued_at: chrono::Utc::now()
                .with_timezone(&chrono::FixedOffset::west_opt(3 * 3600).expect("valid offset")),
            operation_nature: "VENDA".to_string(),
            issuer,
            recipient: None,
            items: Vec::new(),
            payments: Vec::new(),
            change_amount: None,
            payment_card_details: None,
            contingency: None,
            operation_type: None,
            purpose_code: None,
            intermediary_indicator: None,
            emission_process: None,
            consumer_type: None,
            buyer_presence: None,
            print_format: None,
            references: None,
            transport: None,
            billing: None,
            withdrawal: None,
            delivery: None,
            authorized_xml: None,
            additional_info: None,
            intermediary: None,
            ret_trib: None,
            tech_responsible: None,
            purchase: None,
            export: None,
            issqn_tot: None,
        }
    }

    fn sample_item() -> InvoiceItemData {
        InvoiceItemData::new(
            1,
            "001",
            "Gasolina Comum",
            "27101259",
            "5102",
            "LT",
            50.0,
            Cents(599),
            Cents(29950),
            "102",
            Rate(0),
            Cents(0),
            "99",
            "99",
        )
    }

    // ── Combustíveis ────────────────────────────────────────────────────────

    #[test]
    fn comb_minimal_produces_correct_xml() {
        let comb = CombData::new("210203001", "GLP", "SP");
        let xml = build_comb_xml(&comb);

        assert_eq!(
            xml,
            "<comb>\
                <cProdANP>210203001</cProdANP>\
                <descANP>GLP</descANP>\
                <UFCons>SP</UFCons>\
            </comb>"
        );
    }

    #[test]
    fn comb_with_glp_percentages() {
        let comb = CombData::new("210203001", "GLP", "SP")
            .p_glp("60.0000")
            .p_gn_n("25.0000")
            .p_gn_i("15.0000")
            .v_part("3.50");

        let xml = build_comb_xml(&comb);

        assert_eq!(
            xml,
            "<comb>\
                <cProdANP>210203001</cProdANP>\
                <descANP>GLP</descANP>\
                <pGLP>60.0000</pGLP>\
                <pGNn>25.0000</pGNn>\
                <pGNi>15.0000</pGNi>\
                <vPart>3.50</vPart>\
                <UFCons>SP</UFCons>\
            </comb>"
        );
    }

    #[test]
    fn comb_with_codif_and_qtemp() {
        let comb = CombData::new("320102001", "GASOLINA COMUM", "PR")
            .codif("123456789")
            .q_temp("1000.0000");

        let xml = build_comb_xml(&comb);

        assert_eq!(
            xml,
            "<comb>\
                <cProdANP>320102001</cProdANP>\
                <descANP>GASOLINA COMUM</descANP>\
                <CODIF>123456789</CODIF>\
                <qTemp>1000.0000</qTemp>\
                <UFCons>PR</UFCons>\
            </comb>"
        );
    }

    #[test]
    fn comb_with_cide() {
        let cide = CideData::new("1000.0000", "0.0700", "70.00");
        let comb = CombData::new("320102001", "GASOLINA COMUM", "SP").cide(cide);

        let xml = build_comb_xml(&comb);

        assert_eq!(
            xml,
            "<comb>\
                <cProdANP>320102001</cProdANP>\
                <descANP>GASOLINA COMUM</descANP>\
                <UFCons>SP</UFCons>\
                <CIDE>\
                    <qBCProd>1000.0000</qBCProd>\
                    <vAliqProd>0.0700</vAliqProd>\
                    <vCIDE>70.00</vCIDE>\
                </CIDE>\
            </comb>"
        );
    }

    #[test]
    fn comb_with_encerrante() {
        let enc = EncerranteData::new("1", "1", "1234.567", "1284.567").n_bomba("2");
        let comb = CombData::new("320102001", "GASOLINA COMUM", "SP").encerrante(enc);

        let xml = build_comb_xml(&comb);

        assert_eq!(
            xml,
            "<comb>\
                <cProdANP>320102001</cProdANP>\
                <descANP>GASOLINA COMUM</descANP>\
                <UFCons>SP</UFCons>\
                <encerrante>\
                    <nBico>1</nBico>\
                    <nBomba>2</nBomba>\
                    <nTanque>1</nTanque>\
                    <vEncIni>1234.567</vEncIni>\
                    <vEncFin>1284.567</vEncFin>\
                </encerrante>\
            </comb>"
        );
    }

    #[test]
    fn comb_encerrante_without_bomba() {
        let enc = EncerranteData::new("3", "2", "5000.000", "5050.000");
        let comb = CombData::new("320102001", "GASOLINA COMUM", "RJ").encerrante(enc);

        let xml = build_comb_xml(&comb);

        assert_eq!(
            xml,
            "<comb>\
                <cProdANP>320102001</cProdANP>\
                <descANP>GASOLINA COMUM</descANP>\
                <UFCons>RJ</UFCons>\
                <encerrante>\
                    <nBico>3</nBico>\
                    <nTanque>2</nTanque>\
                    <vEncIni>5000.000</vEncIni>\
                    <vEncFin>5050.000</vEncFin>\
                </encerrante>\
            </comb>"
        );
    }

    #[test]
    fn comb_with_pbio() {
        let comb = CombData::new("810102001", "OLEO DIESEL B S10", "SP").p_bio("15.0000");

        let xml = build_comb_xml(&comb);

        assert_eq!(
            xml,
            "<comb>\
                <cProdANP>810102001</cProdANP>\
                <descANP>OLEO DIESEL B S10</descANP>\
                <UFCons>SP</UFCons>\
                <pBio>15.0000</pBio>\
            </comb>"
        );
    }

    #[test]
    fn comb_with_orig_comb_single() {
        let orig = OrigCombData::new("0", "35", "100.0000");
        let comb = CombData::new("320102001", "GASOLINA COMUM", "SP").orig_comb(vec![orig]);

        let xml = build_comb_xml(&comb);

        assert_eq!(
            xml,
            "<comb>\
                <cProdANP>320102001</cProdANP>\
                <descANP>GASOLINA COMUM</descANP>\
                <UFCons>SP</UFCons>\
                <origComb>\
                    <indImport>0</indImport>\
                    <cUFOrig>35</cUFOrig>\
                    <pOrig>100.0000</pOrig>\
                </origComb>\
            </comb>"
        );
    }

    #[test]
    fn comb_with_orig_comb_multiple() {
        let orig1 = OrigCombData::new("0", "35", "70.0000");
        let orig2 = OrigCombData::new("1", "99", "30.0000");
        let comb = CombData::new("320102001", "GASOLINA COMUM", "SP").orig_comb(vec![orig1, orig2]);

        let xml = build_comb_xml(&comb);

        assert_eq!(
            xml,
            "<comb>\
                <cProdANP>320102001</cProdANP>\
                <descANP>GASOLINA COMUM</descANP>\
                <UFCons>SP</UFCons>\
                <origComb>\
                    <indImport>0</indImport>\
                    <cUFOrig>35</cUFOrig>\
                    <pOrig>70.0000</pOrig>\
                </origComb>\
                <origComb>\
                    <indImport>1</indImport>\
                    <cUFOrig>99</cUFOrig>\
                    <pOrig>30.0000</pOrig>\
                </origComb>\
            </comb>"
        );
    }

    #[test]
    fn comb_full_with_all_fields() {
        let cide = CideData::new("500.0000", "0.0700", "35.00");
        let enc = EncerranteData::new("1", "1", "10000.000", "10050.000").n_bomba("1");
        let orig = OrigCombData::new("0", "35", "100.0000");

        let comb = CombData::new("210203001", "GLP", "SP")
            .p_glp("60.0000")
            .p_gn_n("25.0000")
            .p_gn_i("15.0000")
            .v_part("3.50")
            .codif("999888777")
            .q_temp("500.0000")
            .cide(cide)
            .encerrante(enc)
            .p_bio("12.0000")
            .orig_comb(vec![orig]);

        let xml = build_comb_xml(&comb);

        assert_eq!(
            xml,
            "<comb>\
                <cProdANP>210203001</cProdANP>\
                <descANP>GLP</descANP>\
                <pGLP>60.0000</pGLP>\
                <pGNn>25.0000</pGNn>\
                <pGNi>15.0000</pGNi>\
                <vPart>3.50</vPart>\
                <CODIF>999888777</CODIF>\
                <qTemp>500.0000</qTemp>\
                <UFCons>SP</UFCons>\
                <CIDE>\
                    <qBCProd>500.0000</qBCProd>\
                    <vAliqProd>0.0700</vAliqProd>\
                    <vCIDE>35.00</vCIDE>\
                </CIDE>\
                <encerrante>\
                    <nBico>1</nBico>\
                    <nBomba>1</nBomba>\
                    <nTanque>1</nTanque>\
                    <vEncIni>10000.000</vEncIni>\
                    <vEncFin>10050.000</vEncFin>\
                </encerrante>\
                <pBio>12.0000</pBio>\
                <origComb>\
                    <indImport>0</indImport>\
                    <cUFOrig>35</cUFOrig>\
                    <pOrig>100.0000</pOrig>\
                </origComb>\
            </comb>"
        );
    }

    #[test]
    fn comb_in_det_xml() {
        let comb = CombData::new("320102001", "GASOLINA COMUM", "SP");
        let item = sample_item().comb(comb);
        let data = sample_build_data();
        let result = build_det(&item, &data).expect("build_det should succeed");

        // <comb> appears inside <prod>
        let prod_start = result.xml.find("<prod>").expect("<prod> must exist");
        let prod_end = result.xml.find("</prod>").expect("</prod> must exist");
        let prod_section = &result.xml[prod_start..prod_end];

        assert!(prod_section.contains("<comb>"));
        assert!(prod_section.contains("<cProdANP>320102001</cProdANP>"));
        assert!(prod_section.contains("<descANP>GASOLINA COMUM</descANP>"));
        assert!(prod_section.contains("<UFCons>SP</UFCons>"));
        assert!(prod_section.contains("</comb>"));
    }

    // ── ISSQN ───────────────────────────────────────────────────────────────

    #[test]
    fn issqn_item_produces_issqn_tag_not_icms() {
        let issqn_data = TaxIssqnData::new(10000, 500, 500, "3550308", "14.01")
            .ind_iss("1")
            .ind_incentivo("2");
        let item = sample_item().issqn(issqn_data);
        let data = sample_build_data();
        let result = build_det(&item, &data).expect("build_det should succeed");

        // ISSQN tag present inside <imposto>
        assert!(result.xml.contains("<ISSQN>"));
        assert!(result.xml.contains("<vBC>100.00</vBC>"));
        assert!(result.xml.contains("<vAliq>5.0000</vAliq>"));
        assert!(result.xml.contains("<vISSQN>5.00</vISSQN>"));
        assert!(result.xml.contains("<cMunFG>3550308</cMunFG>"));
        assert!(result.xml.contains("<cListServ>14.01</cListServ>"));
        assert!(result.xml.contains("<indISS>1</indISS>"));
        assert!(result.xml.contains("<indIncentivo>2</indIncentivo>"));
        assert!(result.xml.contains("</ISSQN>"));

        // ICMS should NOT be present for ISSQN items
        assert!(!result.xml.contains("<ICMS>"));
        assert!(!result.xml.contains("</ICMS>"));
        assert!(result.has_issqn);
    }

    #[test]
    fn issqn_item_with_all_optional_fields() {
        let issqn_data = TaxIssqnData::new(20000, 300, 600, "3304557", "07.02")
            .v_deducao(1000)
            .v_outro(500)
            .v_desc_incond(200)
            .v_desc_cond(100)
            .v_iss_ret(300)
            .ind_iss("1")
            .c_servico("1234")
            .c_mun("3304557")
            .c_pais("1058")
            .n_processo("ABC123")
            .ind_incentivo("1");

        let item = sample_item().issqn(issqn_data);
        let data = sample_build_data();
        let result = build_det(&item, &data).expect("build_det should succeed");

        assert!(result.xml.contains("<vBC>200.00</vBC>"));
        assert!(result.xml.contains("<vAliq>3.0000</vAliq>"));
        assert!(result.xml.contains("<vISSQN>6.00</vISSQN>"));
        assert!(result.xml.contains("<vDeducao>10.00</vDeducao>"));
        assert!(result.xml.contains("<vOutro>5.00</vOutro>"));
        assert!(result.xml.contains("<vDescIncond>2.00</vDescIncond>"));
        assert!(result.xml.contains("<vDescCond>1.00</vDescCond>"));
        assert!(result.xml.contains("<vISSRet>3.00</vISSRet>"));
        assert!(result.xml.contains("<cServico>1234</cServico>"));
        assert!(result.xml.contains("<cMun>3304557</cMun>"));
        assert!(result.xml.contains("<cPais>1058</cPais>"));
        assert!(result.xml.contains("<nProcesso>ABC123</nProcesso>"));
        assert!(result.xml.contains("<indIncentivo>1</indIncentivo>"));
        assert!(result.has_issqn);
    }

    #[test]
    fn non_issqn_item_has_icms_and_no_issqn() {
        let item = sample_item();
        let data = sample_build_data();
        let result = build_det(&item, &data).expect("build_det should succeed");

        assert!(result.xml.contains("<ICMS"));
        assert!(!result.xml.contains("<ISSQN>"));
        assert!(!result.has_issqn);
    }
}
