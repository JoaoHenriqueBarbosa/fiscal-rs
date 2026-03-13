//! Build `<det>` (item detail) elements of the NF-e XML.

mod icms_variant;
mod prod;

#[cfg(test)]
mod tests;

use crate::FiscalError;
use crate::format_utils::{format_cents, format_decimal, format_rate4};
use crate::newtypes::{Cents, Rate4};
use crate::tax_ibs_cbs;
use crate::tax_icms::{self, IcmsTotals};
use crate::tax_is;
use crate::tax_issqn;
use crate::tax_pis_cofins_ipi::{self, CofinsData, IiData, IpiData, PisData};
use crate::types::{InvoiceBuildData, InvoiceItemData, InvoiceModel, SefazEnvironment, TaxRegime};
use crate::xml_utils::{TagContent, tag};

use icms_variant::build_icms_variant;
#[cfg(test)]
use prod::build_comb_xml;
use prod::{build_det_export_xml, build_det_extras, build_di_xml, build_prod_options};

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
    /// IPI devolution value in cents contributed by this item.
    pub v_ipi_devol: i64,
    /// PIS-ST value in cents contributed by this item (only when indSomaPISST = 1).
    pub v_pis_st: i64,
    /// COFINS-ST value in cents contributed by this item (only when indSomaCOFINSST = 1).
    pub v_cofins_st: i64,
    /// Whether this item has indDeduzDeson = 1 (desoneration deduction applies).
    pub ind_deduz_deson: bool,
    /// Whether this item uses ISSQN instead of ICMS.
    pub has_issqn: bool,
}

/// Build a `<det nItem="N">` element for one invoice item.
pub(crate) fn build_det(
    item: &InvoiceItemData,
    data: &InvoiceBuildData,
) -> Result<DetResult, FiscalError> {
    // Validate NVE: up to 8 per item
    if item.nve.len() > 8 {
        return Err(FiscalError::InvalidTaxData(format!(
            "Item {}: NVE limited to 8 entries, got {}",
            item.item_number,
            item.nve.len()
        )));
    }

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

    // Build PIS-ST (optional)
    let mut v_pis_st = 0i64;
    if let Some(ref pis_st_data) = item.pis_st {
        // Accumulate only when indSomaPISST == 1 (matches PHP)
        if pis_st_data.ind_soma_pis_st == Some(1) {
            v_pis_st = pis_st_data.v_pis.0;
        }
    }

    // Build COFINS-ST (optional)
    let mut v_cofins_st = 0i64;
    if let Some(ref cofins_st_data) = item.cofins_st {
        // Accumulate only when indSomaCOFINSST == 1 (matches PHP)
        if cofins_st_data.ind_soma_cofins_st == Some(1) {
            v_cofins_st = cofins_st_data.v_cofins.0;
        }
    }

    // Detect indDeduzDeson from item ICMS data
    let item_ind_deduz_deson = item
        .icms_ind_deduz_deson
        .as_deref()
        .map(|v| v == "1")
        .unwrap_or(false);

    // Build det-level extras (infAdProd, obsItem, vItem, DFeReferenciado)
    // Deferred to after imposto assembly so we have access to computed values.

    // Assemble imposto
    let mut imposto_children: Vec<String> = Vec::new();
    // vTotTrib: emitted as first child of <imposto> when > 0 (matches PHP tagimposto)
    if let Some(ref v) = item.v_tot_trib {
        if v.0 > 0 {
            imposto_children.push(tag(
                "vTotTrib",
                &[],
                TagContent::Text(&format_cents(v.0, 2)),
            ));
        }
    }
    if !icms_xml.is_empty() {
        imposto_children.push(icms_xml);
    }
    if !ipi_xml.is_empty() {
        imposto_children.push(ipi_xml);
    }
    // PIS or PISST (mutually exclusive per PHP sped-nfe)
    if let Some(ref pis_st) = item.pis_st {
        imposto_children.push(tax_pis_cofins_ipi::build_pis_st_xml(pis_st));
    } else {
        imposto_children.push(pis_xml);
    }
    // COFINS or COFINSST (mutually exclusive per PHP sped-nfe)
    if let Some(ref cofins_st) = item.cofins_st {
        imposto_children.push(tax_pis_cofins_ipi::build_cofins_st_xml(cofins_st));
    } else {
        imposto_children.push(cofins_xml);
    }
    if !ii_xml.is_empty() {
        imposto_children.push(ii_xml);
    }
    if !issqn_xml.is_empty() {
        imposto_children.push(issqn_xml);
    }

    // Build IS (Imposto Seletivo) -- optional, inside <imposto>
    // Only emitted when schema is PL_010 or later (matching PHP: $this->schema > 9)
    if data.schema_version.is_pl010() {
        if let Some(ref is_data) = item.is_data {
            imposto_children.push(tax_is::build_is_xml(is_data));
        }

        // Build IBS/CBS -- optional, inside <imposto>
        if let Some(ref ibs_cbs_data) = item.ibs_cbs {
            imposto_children.push(tax_ibs_cbs::build_ibs_cbs_xml(ibs_cbs_data));
        }
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
    ];
    if let Some(ref cb) = item.c_barra {
        prod_children.push(tag("cBarra", &[], TagContent::Text(cb)));
    }
    prod_children.push(tag(
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
    ));
    prod_children.push(tag("NCM", &[], TagContent::Text(&item.ncm)));
    for nve_code in &item.nve {
        prod_children.push(tag("NVE", &[], TagContent::Text(nve_code)));
    }
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
    // tpCredPresIBSZFM — PL_010 only, after cBenef, before gCred (NT 2025.002)
    if data.schema_version.is_pl010() {
        if let Some(ref tp) = item.tp_cred_pres_ibs_zfm {
            prod_children.push(tag("tpCredPresIBSZFM", &[], TagContent::Text(tp)));
        }
    }
    // gCred (crédito presumido ICMS) — up to 4 per item, inside <prod>
    for gc in item.g_cred.iter().take(4) {
        let p_str = format_rate4(gc.p_cred_presumido.0);
        let mut gc_children = vec![
            tag(
                "cCredPresumido",
                &[],
                TagContent::Text(&gc.c_cred_presumido),
            ),
            tag("pCredPresumido", &[], TagContent::Text(&p_str)),
        ];
        if let Some(v) = gc.v_cred_presumido {
            let v_str = format_cents(v.0, 2);
            gc_children.push(tag("vCredPresumido", &[], TagContent::Text(&v_str)));
        }
        prod_children.push(tag("gCred", &[], TagContent::Children(gc_children)));
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
    ]);
    if let Some(ref cbt) = item.c_barra_trib {
        prod_children.push(tag("cBarraTrib", &[], TagContent::Text(cbt)));
    }
    let u_trib = item
        .taxable_unit
        .as_deref()
        .unwrap_or(&item.unit_of_measure);
    let q_trib = item.taxable_quantity.unwrap_or(item.quantity);
    let v_un_trib = item
        .taxable_unit_price
        .map(|c| c.0)
        .unwrap_or(item.unit_price.0);
    prod_children.extend([
        tag("uTrib", &[], TagContent::Text(u_trib)),
        tag("qTrib", &[], TagContent::Text(&fd4(q_trib))),
        tag("vUnTrib", &[], TagContent::Text(&fc10(v_un_trib))),
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
    if item.ind_bem_movel_usado == Some(true) {
        prod_children.push(tag("indBemMovelUsado", &[], TagContent::Text("1")));
    }
    // DI (Declaração de Importação) — after indTot, before detExport
    if let Some(ref dis) = item.di {
        for di in dis.iter().take(100) {
            prod_children.push(build_di_xml(di));
        }
    }
    // detExport — after DI, before xPed
    if let Some(ref exports) = item.det_export {
        for dex in exports.iter().take(500) {
            prod_children.push(build_det_export_xml(dex));
        }
    }
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

    // impostoDevol (after imposto, before infAdProd)
    let mut v_ipi_devol = 0i64;
    let imposto_devol_xml = if let Some(ref devol) = item.imposto_devol {
        v_ipi_devol = devol.v_ipi_devol.0;
        let p_devol_str = format_decimal(devol.p_devol.0 as f64 / 100.0, 2);
        let v_ipi_devol_str = format_cents(devol.v_ipi_devol.0, 2);
        tag(
            "impostoDevol",
            &[],
            TagContent::Children(vec![
                tag("pDevol", &[], TagContent::Text(&p_devol_str)),
                tag(
                    "IPI",
                    &[],
                    TagContent::Children(vec![tag(
                        "vIPIDevol",
                        &[],
                        TagContent::Text(&v_ipi_devol_str),
                    )]),
                ),
            ]),
        )
    } else {
        String::new()
    };

    // Compute vItem for PL_010 — matching PHP calculateTtensValues2
    // Emitted inside <det> when schema >= PL_010 and at least one item has IBS/CBS data.
    let v_item_xml =
        if data.schema_version.is_pl010() && data.items.iter().any(|i| i.ibs_cbs.is_some()) {
            let v_item_cents = if let Some(ref explicit) = item.v_item {
                // User-supplied vItem takes precedence (matches PHP: $this->aVItem[$item]['vItem'])
                explicit.0
            } else {
                // Auto-calculate (matches PHP calculateTtensValues2)
                let v_prod = item.total_price.0;
                let v_desc = item.v_desc.map(|c| c.0).unwrap_or(0);
                let v_icms_deson = if item_ind_deduz_deson {
                    item.icms_v_icms_deson.map(|c| c.0).unwrap_or(0)
                } else {
                    0
                };
                let v_icms_st = icms_totals.v_st.0;
                let v_icms_mono_reten = icms_totals.v_icms_mono_reten.0;
                let v_fcp_st = icms_totals.v_fcp_st.0;
                let v_frete = item.v_frete.map(|c| c.0).unwrap_or(0);
                let v_seg = item.v_seg.map(|c| c.0).unwrap_or(0);
                let v_outro = item.v_outro.map(|c| c.0).unwrap_or(0);

                v_prod - v_desc - v_icms_deson
                    + v_icms_st
                    + v_icms_mono_reten
                    + v_fcp_st
                    + v_frete
                    + v_seg
                    + v_outro
                    + v_ii
                    + v_ipi
                    + v_ipi_devol
                    + v_pis_st
                    + v_cofins_st
            };
            let v_item_str = format_cents(v_item_cents, 2);
            tag("vItem", &[], TagContent::Text(&v_item_str))
        } else {
            String::new()
        };

    // Build det-level extras (infAdProd, obsItem, vItem, DFeReferenciado)
    let det_extras = build_det_extras(item, &v_item_xml);

    // Assemble det
    let nitem = item.item_number.to_string();
    let mut det_children = vec![
        tag("prod", &[], TagContent::Children(prod_children)),
        tag("imposto", &[], TagContent::Children(imposto_children)),
    ];
    if !imposto_devol_xml.is_empty() {
        det_children.push(imposto_devol_xml);
    }
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
        v_ipi_devol,
        v_pis_st,
        v_cofins_st,
        ind_deduz_deson: item_ind_deduz_deson,
        has_issqn,
    })
}
