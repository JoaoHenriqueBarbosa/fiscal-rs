use crate::format_utils::{format_cents, format_decimal};
use crate::types::{CombData, InvoiceItemData};
use crate::xml_utils::{TagContent, tag};

pub(super) fn build_prod_options(item: &InvoiceItemData) -> Vec<String> {
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

/// Build a single `<DI>` element with its nested `<adi>` children.
pub(super) fn build_di_xml(di: &crate::types::DiData) -> String {
    let mut children = vec![
        tag("nDI", &[], TagContent::Text(&di.n_di)),
        tag("dDI", &[], TagContent::Text(&di.d_di)),
        tag("xLocDesemb", &[], TagContent::Text(&di.x_loc_desemb)),
        tag("UFDesemb", &[], TagContent::Text(&di.uf_desemb)),
        tag("dDesemb", &[], TagContent::Text(&di.d_desemb)),
        tag("tpViaTransp", &[], TagContent::Text(&di.tp_via_transp)),
    ];
    if let Some(ref v) = di.v_afrmm {
        children.push(tag("vAFRMM", &[], TagContent::Text(&format_cents(v.0, 2))));
    }
    children.push(tag(
        "tpIntermedio",
        &[],
        TagContent::Text(&di.tp_intermedio),
    ));
    if let Some(ref cnpj) = di.cnpj {
        children.push(tag("CNPJ", &[], TagContent::Text(cnpj)));
    } else if let Some(ref cpf) = di.cpf {
        children.push(tag("CPF", &[], TagContent::Text(cpf)));
    }
    if let Some(ref uf) = di.uf_terceiro {
        children.push(tag("UFTerceiro", &[], TagContent::Text(uf)));
    }
    children.push(tag("cExportador", &[], TagContent::Text(&di.c_exportador)));
    // adi children (up to 999 per DI)
    for adi in di.adi.iter().take(999) {
        let mut adi_children = Vec::new();
        if let Some(ref n) = adi.n_adicao {
            adi_children.push(tag("nAdicao", &[], TagContent::Text(n)));
        }
        adi_children.push(tag("nSeqAdic", &[], TagContent::Text(&adi.n_seq_adic)));
        adi_children.push(tag("cFabricante", &[], TagContent::Text(&adi.c_fabricante)));
        if let Some(ref v) = adi.v_desc_di {
            adi_children.push(tag("vDescDI", &[], TagContent::Text(&format_cents(v.0, 2))));
        }
        if let Some(ref n) = adi.n_draw {
            adi_children.push(tag("nDraw", &[], TagContent::Text(n)));
        }
        children.push(tag("adi", &[], TagContent::Children(adi_children)));
    }
    tag("DI", &[], TagContent::Children(children))
}

/// Build a single `<detExport>` element with optional `<exportInd>`.
pub(super) fn build_det_export_xml(dex: &crate::types::DetExportData) -> String {
    let mut children = Vec::new();
    if let Some(ref n) = dex.n_draw {
        children.push(tag("nDraw", &[], TagContent::Text(n)));
    }
    if dex.n_re.is_some() || dex.ch_nfe.is_some() || dex.q_export.is_some() {
        let mut exp_ind_children = Vec::new();
        if let Some(ref n) = dex.n_re {
            exp_ind_children.push(tag("nRE", &[], TagContent::Text(n)));
        }
        if let Some(ref ch) = dex.ch_nfe {
            exp_ind_children.push(tag("chNFe", &[], TagContent::Text(ch)));
        }
        if let Some(q) = dex.q_export {
            exp_ind_children.push(tag("qExport", &[], TagContent::Text(&format_decimal(q, 4))));
        }
        children.push(tag(
            "exportInd",
            &[],
            TagContent::Children(exp_ind_children),
        ));
    }
    tag("detExport", &[], TagContent::Children(children))
}

/// Build the `<comb>` element for fuel products.
///
/// Follows the PHP sped-nfe `tagcomb` / `tagencerrante` / `tagorigComb`
/// structure exactly: cProdANP, descANP, pGLP, pGNn, pGNi, vPart, CODIF,
/// qTemp, UFCons, CIDE, encerrante, pBio, origComb[].
pub(super) fn build_comb_xml(comb: &CombData) -> String {
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

pub(super) fn build_det_extras(item: &InvoiceItemData, v_item_xml: &str) -> Vec<String> {
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

    // vItem — PL_010 only, after obsItem, before DFeReferenciado
    if !v_item_xml.is_empty() {
        extras.push(v_item_xml.to_string());
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
