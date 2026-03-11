//! Build `<det>` (item detail) elements of the NF-e XML.

use crate::format_utils::{format_cents, format_decimal};
use crate::newtypes::{Cents, Rate4};
use crate::tax_icms::{self, IcmsData, IcmsTotals};
use crate::tax_pis_cofins_ipi::{self, CofinsData, IiData, IpiData, PisData};
use crate::types::{InvoiceBuildData, InvoiceItemData, TaxRegime};
use crate::xml_utils::{tag, TagContent};
use crate::FiscalError;

/// Result from building a single `<det>` element.
#[derive(Debug, Clone)]
pub struct DetResult {
    pub xml: String,
    pub icms_totals: IcmsTotals,
    pub v_ipi: i64,
    pub v_pis: i64,
    pub v_cofins: i64,
    pub v_ii: i64,
}

/// Build a `<det nItem="N">` element for one invoice item.
pub fn build_det(
    item: &InvoiceItemData,
    data: &InvoiceBuildData,
) -> Result<DetResult, FiscalError> {
    let is_simples = matches!(
        data.issuer.tax_regime,
        TaxRegime::SimplesNacional | TaxRegime::SimplesExcess
    );

    // Build ICMS
    let icms_data = IcmsData {
        tax_regime: data.issuer.tax_regime as u8,
        orig: item.orig.clone().unwrap_or_else(|| "0".to_string()),
        cst: if is_simples { None } else { Some(item.icms_cst.clone()) },
        csosn: if is_simples {
            Some(if item.icms_cst.is_empty() { "102".to_string() } else { item.icms_cst.clone() })
        } else { None },
        mod_bc: item.icms_mod_bc.map(|v| v.to_string()),
        v_bc: Some(item.total_price),
        p_icms: Some(item.icms_rate),
        v_icms: Some(item.icms_amount),
        p_red_bc: item.icms_red_bc,
        mod_bc_st: item.icms_mod_bc_st.map(|v| v.to_string()),
        p_mva_st: item.icms_p_mva_st,
        p_red_bc_st: item.icms_red_bc_st,
        v_bc_st: item.icms_v_bc_st,
        p_icms_st: item.icms_p_icms_st,
        v_icms_st: item.icms_v_icms_st,
        v_icms_deson: item.icms_v_icms_deson,
        mot_des_icms: item.icms_mot_des_icms.map(|v| v.to_string()),
        p_fcp: item.icms_p_fcp,
        v_fcp: item.icms_v_fcp,
        v_bc_fcp: item.icms_v_bc_fcp,
        p_fcp_st: item.icms_p_fcp_st,
        v_fcp_st: item.icms_v_fcp_st,
        v_bc_fcp_st: item.icms_v_bc_fcp_st,
        p_cred_sn: item.icms_p_cred_sn,
        v_cred_icms_sn: item.icms_v_cred_icms_sn,
        v_icms_substituto: item.icms_v_icms_substituto,
        ..IcmsData::default()
    };
    let (icms_xml, icms_totals) = tax_icms::build_icms_xml(&icms_data)?;

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

    // Build prod options (rastro, veicProd, med, arma, nRECOPI)
    let prod_options = build_prod_options(item);

    // Build det-level extras (infAdProd, obsItem, DFeReferenciado)
    let det_extras = build_det_extras(item);

    // Assemble imposto
    let mut imposto_children: Vec<String> = vec![icms_xml];
    if !ipi_xml.is_empty() { imposto_children.push(ipi_xml); }
    imposto_children.push(pis_xml);
    imposto_children.push(cofins_xml);
    if !ii_xml.is_empty() { imposto_children.push(ii_xml); }

    // Assemble prod
    let fc2 = |c: i64| format_cents(c, 2);
    let fc10 = |c: i64| format_cents(c, 10);
    let fd4 = |v: f64| format_decimal(v, 4);

    let mut prod_children = vec![
        tag("cProd", &[], TagContent::Text(&item.product_code)),
        tag("cEAN", &[], TagContent::Text(item.c_ean.as_deref().unwrap_or("SEM GTIN"))),
        tag("xProd", &[], TagContent::Text(&item.description)),
        tag("NCM", &[], TagContent::Text(&item.ncm)),
    ];
    if let Some(ref cest) = item.cest {
        prod_children.push(tag("CEST", &[], TagContent::Text(cest)));
    }
    prod_children.extend([
        tag("CFOP", &[], TagContent::Text(&item.cfop)),
        tag("uCom", &[], TagContent::Text(&item.unit_of_measure)),
        tag("qCom", &[], TagContent::Text(&fd4(item.quantity))),
        tag("vUnCom", &[], TagContent::Text(&fc10(item.unit_price.0))),
        tag("vProd", &[], TagContent::Text(&fc2(item.total_price.0))),
        tag("cEANTrib", &[], TagContent::Text(item.c_ean_trib.as_deref().unwrap_or("SEM GTIN"))),
        tag("uTrib", &[], TagContent::Text(&item.unit_of_measure)),
        tag("qTrib", &[], TagContent::Text(&fd4(item.quantity))),
        tag("vUnTrib", &[], TagContent::Text(&fc10(item.unit_price.0))),
    ]);
    if let Some(v) = item.v_frete { prod_children.push(tag("vFrete", &[], TagContent::Text(&fc2(v.0)))); }
    if let Some(v) = item.v_seg { prod_children.push(tag("vSeg", &[], TagContent::Text(&fc2(v.0)))); }
    if let Some(v) = item.v_desc { prod_children.push(tag("vDesc", &[], TagContent::Text(&fc2(v.0)))); }
    if let Some(v) = item.v_outro { prod_children.push(tag("vOutro", &[], TagContent::Text(&fc2(v.0)))); }
    prod_children.push(tag("indTot", &[], TagContent::Text("1")));
    prod_children.extend(prod_options);

    // Assemble det
    let nitem = item.item_number.to_string();
    let mut det_children = vec![
        tag("prod", &[], TagContent::Children(prod_children)),
        tag("imposto", &[], TagContent::Children(imposto_children)),
    ];
    det_children.extend(det_extras);

    let xml = tag("det", &[("nItem", &nitem)], TagContent::Children(det_children));

    Ok(DetResult {
        xml,
        icms_totals,
        v_ipi,
        v_pis: item.pis_v_pis.map(|c| c.0).unwrap_or(0),
        v_cofins: item.cofins_v_cofins.map(|c| c.0).unwrap_or(0),
        v_ii,
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
        opts.push(tag("veicProd", &[], TagContent::Children(vec![
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
        ])));
    } else if let Some(ref m) = item.med {
        let mut med_children = Vec::new();
        if let Some(ref code) = m.c_prod_anvisa {
            med_children.push(tag("cProdANVISA", &[], TagContent::Text(code)));
        }
        if let Some(ref reason) = m.x_motivo_isencao {
            med_children.push(tag("xMotivoIsencao", &[], TagContent::Text(reason)));
        }
        med_children.push(tag("vPMC", &[], TagContent::Text(&format_cents(m.v_pmc.0, 2))));
        opts.push(tag("med", &[], TagContent::Children(med_children)));
    } else if let Some(ref arms) = item.arma {
        for a in arms.iter().take(500) {
            opts.push(tag("arma", &[], TagContent::Children(vec![
                tag("tpArma", &[], TagContent::Text(&a.tp_arma)),
                tag("nSerie", &[], TagContent::Text(&a.n_serie)),
                tag("nCano", &[], TagContent::Text(&a.n_cano)),
                tag("descr", &[], TagContent::Text(&a.descr)),
            ])));
        }
    } else if let Some(ref recopi) = item.n_recopi {
        if !recopi.is_empty() {
            opts.push(tag("nRECOPI", &[], TagContent::Text(recopi)));
        }
    }

    opts
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
                "obsCont", &[("xCampo", &cont.x_campo)],
                TagContent::Children(vec![
                    tag("xTexto", &[], TagContent::Text(&cont.x_texto)),
                ]),
            ));
        }
        if let Some(ref fisco) = obs.obs_fisco {
            obs_children.push(tag(
                "obsFisco", &[("xCampo", &fisco.x_campo)],
                TagContent::Children(vec![
                    tag("xTexto", &[], TagContent::Text(&fisco.x_texto)),
                ]),
            ));
        }
        extras.push(tag("obsItem", &[], TagContent::Children(obs_children)));
    }

    if let Some(ref dfe) = item.dfe_referenciado {
        let mut dfe_children = vec![
            tag("chaveAcesso", &[], TagContent::Text(&dfe.chave_acesso)),
        ];
        if let Some(ref n) = dfe.n_item {
            dfe_children.push(tag("nItem", &[], TagContent::Text(n)));
        }
        extras.push(tag("DFeReferenciado", &[], TagContent::Children(dfe_children)));
    }

    extras
}
