//! Item detail and product builders.

use super::helpers::*;
use super::parser::NFeParser;
use super::types::ItemBuild;

impl<'a> NFeParser<'a> {
    pub(super) fn build_det(&self, item: &ItemBuild, n_item: usize) -> String {
        let mut c = Vec::new();
        c.push(self.build_prod(item));
        c.push(self.build_imposto(item));
        if let Some(ref dev) = item.imposto_devol {
            c.push(self.build_imposto_devol(dev));
        }
        if !item.inf_ad_prod.is_empty() {
            add_child_str(&mut c, "infAdProd", &item.inf_ad_prod);
        }
        format!("<det nItem=\"{n_item}\">{}</det>", c.join(""))
    }

    pub(super) fn build_prod(&self, item: &ItemBuild) -> String {
        let p = &item.prod;
        let mut c = Vec::new();
        add_child(&mut c, "cProd", p.get("cProd").map(|s| s.as_str()));
        add_child_str(
            &mut c,
            "cEAN",
            p.get("cEAN").map(|s| s.as_str()).unwrap_or("SEM GTIN"),
        );
        add_child(&mut c, "xProd", p.get("xProd").map(|s| s.as_str()));
        add_child(&mut c, "NCM", p.get("NCM").map(|s| s.as_str()));
        if let Some(cest) = &item.cest {
            if let Some(v) = cest.get("CEST") {
                add_child_str(&mut c, "CEST", v);
            }
        }
        if let Some(v) = p.get("cBenef") {
            if !v.is_empty() {
                add_child_str(&mut c, "cBenef", v);
            }
        }
        // gCred — must come between cBenef and EXTIPI per XSD sequence
        if let Some(gc) = &item.g_cred {
            let mut gcc = Vec::new();
            add_child(
                &mut gcc,
                "cCredPresumido",
                gc.get("cCredPresumido").map(|s| s.as_str()),
            );
            add_child_dec(
                &mut gcc,
                "pCredPresumido",
                gc.get("pCredPresumido").map(|s| s.as_str()),
                4,
            );
            add_child_dec(
                &mut gcc,
                "vCredPresumido",
                gc.get("vCredPresumido").map(|s| s.as_str()),
                2,
            );
            c.push(xml_tag("gCred", &gcc.join("")));
        }
        if let Some(v) = p.get("EXTIPI") {
            if !v.is_empty() {
                add_child_str(&mut c, "EXTIPI", v);
            }
        }
        add_child(&mut c, "CFOP", p.get("CFOP").map(|s| s.as_str()));
        add_child(&mut c, "uCom", p.get("uCom").map(|s| s.as_str()));
        add_child_dec(&mut c, "qCom", p.get("qCom").map(|s| s.as_str()), 4);
        add_child_dec(&mut c, "vUnCom", p.get("vUnCom").map(|s| s.as_str()), 10);
        add_child_dec(&mut c, "vProd", p.get("vProd").map(|s| s.as_str()), 2);
        add_child_str(
            &mut c,
            "cEANTrib",
            p.get("cEANTrib").map(|s| s.as_str()).unwrap_or("SEM GTIN"),
        );
        add_child(&mut c, "uTrib", p.get("uTrib").map(|s| s.as_str()));
        add_child_dec(&mut c, "qTrib", p.get("qTrib").map(|s| s.as_str()), 4);
        add_child_dec(&mut c, "vUnTrib", p.get("vUnTrib").map(|s| s.as_str()), 10);
        if let Some(v) = p.get("vFrete") {
            if !v.is_empty() {
                add_child_str_dec(&mut c, "vFrete", v, 2);
            }
        }
        if let Some(v) = p.get("vSeg") {
            if !v.is_empty() {
                add_child_str_dec(&mut c, "vSeg", v, 2);
            }
        }
        if let Some(v) = p.get("vDesc") {
            if !v.is_empty() {
                add_child_str_dec(&mut c, "vDesc", v, 2);
            }
        }
        if let Some(v) = p.get("vOutro") {
            if !v.is_empty() {
                add_child_str_dec(&mut c, "vOutro", v, 2);
            }
        }
        let ind_tot = p.get("indTot").map(|s| s.as_str()).unwrap_or("1");
        add_child_str(&mut c, "indTot", ind_tot);
        if let Some(v) = p.get("xPed") {
            if !v.is_empty() {
                add_child_str(&mut c, "xPed", v);
            }
        }
        if let Some(v) = p.get("nItemPed") {
            if !v.is_empty() {
                add_child_str(&mut c, "nItemPed", v);
            }
        }
        if let Some(v) = p.get("nFCI") {
            if !v.is_empty() {
                add_child_str(&mut c, "nFCI", v);
            }
        }
        // DI/adi
        for di in &item.di_list {
            let mut dc = Vec::new();
            let d = &di.fields;
            for &f in &[
                "nDI",
                "dDI",
                "xLocDesemb",
                "UFDesemb",
                "dDesemb",
                "tpViaTransp",
            ] {
                add_child(&mut dc, f, d.get(f).map(|s| s.as_str()));
            }
            if let Some(v) = d.get("vAFRMM") {
                if !v.is_empty() {
                    add_child_str_dec(&mut dc, "vAFRMM", v, 2);
                }
            }
            add_child(
                &mut dc,
                "tpIntermedio",
                d.get("tpIntermedio").map(|s| s.as_str()),
            );
            if let Some(v) = d.get("CNPJ") {
                if !v.is_empty() {
                    add_child_str(&mut dc, "CNPJ", v);
                }
            }
            if let Some(v) = d.get("UFTerceiro") {
                if !v.is_empty() {
                    add_child_str(&mut dc, "UFTerceiro", v);
                }
            }
            add_child(
                &mut dc,
                "cExportador",
                d.get("cExportador").map(|s| s.as_str()),
            );
            for adi in &di.adi_list {
                let mut ac = Vec::new();
                add_child(&mut ac, "nAdicao", adi.get("nAdicao").map(|s| s.as_str()));
                add_child(
                    &mut ac,
                    "nSeqAdic",
                    adi.get("nSeqAdic")
                        .or_else(|| adi.get("nSeqAdicC"))
                        .map(|s| s.as_str()),
                );
                add_child(
                    &mut ac,
                    "cFabricante",
                    adi.get("cFabricante").map(|s| s.as_str()),
                );
                if let Some(v) = adi.get("vDescDI") {
                    if !v.is_empty() {
                        add_child_str_dec(&mut ac, "vDescDI", v, 2);
                    }
                }
                if let Some(v) = adi.get("nDraw") {
                    if !v.is_empty() {
                        add_child_str(&mut ac, "nDraw", v);
                    }
                }
                dc.push(xml_tag("adi", &ac.join("")));
            }
            c.push(xml_tag("DI", &dc.join("")));
        }
        // detExport
        for exp in &item.det_export_list {
            let mut ec = Vec::new();
            if let Some(v) = exp.get("nDraw") {
                if !v.is_empty() {
                    add_child_str(&mut ec, "nDraw", v);
                }
            }
            if exp.get("nRE").is_some() || exp.get("chNFe").is_some() {
                let mut ic = Vec::new();
                add_child(&mut ic, "nRE", exp.get("nRE").map(|s| s.as_str()));
                add_child(&mut ic, "chNFe", exp.get("chNFe").map(|s| s.as_str()));
                add_child_dec(
                    &mut ic,
                    "qExport",
                    exp.get("qExport").map(|s| s.as_str()),
                    4,
                );
                ec.push(xml_tag("exportInd", &ic.join("")));
            }
            c.push(xml_tag("detExport", &ec.join("")));
        }
        // rastro
        for r in &item.rastro_list {
            let mut rc = Vec::new();
            add_child(&mut rc, "nLote", r.get("nLote").map(|s| s.as_str()));
            add_child_dec(&mut rc, "qLote", r.get("qLote").map(|s| s.as_str()), 3);
            add_child(&mut rc, "dFab", r.get("dFab").map(|s| s.as_str()));
            add_child(&mut rc, "dVal", r.get("dVal").map(|s| s.as_str()));
            if let Some(v) = r.get("cAgreg") {
                if !v.is_empty() {
                    add_child_str(&mut rc, "cAgreg", v);
                }
            }
            c.push(xml_tag("rastro", &rc.join("")));
        }
        // veicProd
        if let Some(ref vp) = item.veic_prod {
            let mut vc = Vec::new();
            for &f in &[
                "tpOp",
                "chassi",
                "cCor",
                "xCor",
                "pot",
                "cilin",
                "pesoL",
                "pesoB",
                "nSerie",
                "tpComb",
                "nMotor",
                "CMT",
                "dist",
                "anoMod",
                "anoFab",
                "tpPint",
                "tpVeic",
                "espVeic",
                "VIN",
                "condVeic",
                "cMod",
                "cCorDENATRAN",
                "lota",
                "tpRest",
            ] {
                add_child(&mut vc, f, vp.get(f).map(|s| s.as_str()));
            }
            c.push(xml_tag("veicProd", &vc.join("")));
        }
        // med
        if let Some(ref m) = item.med {
            let mut mc = Vec::new();
            add_child(
                &mut mc,
                "cProdANVISA",
                m.get("cProdANVISA").map(|s| s.as_str()),
            );
            if let Some(v) = m.get("xMotivoIsencao") {
                if !v.is_empty() {
                    add_child_str(&mut mc, "xMotivoIsencao", v);
                }
            }
            if let Some(v) = m.get("vPMC") {
                if !v.is_empty() {
                    add_child_str_dec(&mut mc, "vPMC", v, 2);
                }
            }
            c.push(xml_tag("med", &mc.join("")));
        }
        // arma
        for a in &item.arma_list {
            let mut ac = Vec::new();
            for &f in &["tpArma", "nSerie", "nCano", "descr"] {
                add_child(&mut ac, f, a.get(f).map(|s| s.as_str()));
            }
            c.push(xml_tag("arma", &ac.join("")));
        }
        // comb + CIDE + encerrante
        if let Some(ref cb) = item.comb {
            let mut cc = Vec::new();
            add_child(&mut cc, "cProdANP", cb.get("cProdANP").map(|s| s.as_str()));
            if let Some(v) = cb.get("descANP") {
                if !v.is_empty() {
                    add_child_str(&mut cc, "descANP", v);
                }
            }
            for &f in &["pGLP", "pGNn", "pGNi", "vPart"] {
                if let Some(v) = cb.get(f) {
                    if !v.is_empty() {
                        add_child_str_dec(&mut cc, f, v, 4);
                    }
                }
            }
            if let Some(v) = cb.get("CODIF") {
                if !v.is_empty() {
                    add_child_str(&mut cc, "CODIF", v);
                }
            }
            if let Some(v) = cb.get("qTemp") {
                if !v.is_empty() {
                    add_child_str_dec(&mut cc, "qTemp", v, 4);
                }
            }
            add_child(&mut cc, "UFCons", cb.get("UFCons").map(|s| s.as_str()));
            if let Some(ref cide) = item.comb_cide {
                let mut cdc = Vec::new();
                add_child_dec(
                    &mut cdc,
                    "qBCProd",
                    cide.get("qBCProd").map(|s| s.as_str()),
                    4,
                );
                add_child_dec(
                    &mut cdc,
                    "vAliqProd",
                    cide.get("vAliqProd").map(|s| s.as_str()),
                    4,
                );
                add_child_dec(&mut cdc, "vCIDE", cide.get("vCIDE").map(|s| s.as_str()), 2);
                cc.push(xml_tag("CIDE", &cdc.join("")));
            }
            if let Some(ref enc) = item.encerrante {
                let mut ec = Vec::new();
                add_child(&mut ec, "nBico", enc.get("nBico").map(|s| s.as_str()));
                if let Some(v) = enc.get("nBomba") {
                    if !v.is_empty() {
                        add_child_str(&mut ec, "nBomba", v);
                    }
                }
                if let Some(v) = enc.get("nTanque") {
                    if !v.is_empty() {
                        add_child_str(&mut ec, "nTanque", v);
                    }
                }
                add_child_dec(
                    &mut ec,
                    "vEncIni",
                    enc.get("vEncIni").map(|s| s.as_str()),
                    3,
                );
                add_child_dec(
                    &mut ec,
                    "vEncFin",
                    enc.get("vEncFin").map(|s| s.as_str()),
                    3,
                );
                cc.push(xml_tag("encerrante", &ec.join("")));
            }
            c.push(xml_tag("comb", &cc.join("")));
        }
        // NVE
        for nve in &item.nve_list {
            add_child_str(&mut c, "NVE", nve);
        }
        // RECOPI
        if let Some(ref r) = item.recopi {
            add_child_str(&mut c, "nRECOPI", r);
        }

        xml_tag("prod", &c.join(""))
    }
}
