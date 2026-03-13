//! XML building methods for NFeParser.

use super::helpers::*;
use super::types::ItemBuild;
use crate::constants::NFE_NAMESPACE;
use crate::xml_utils::escape_xml;
use super::parser::NFeParser;

impl<'a> NFeParser<'a> {
    pub(super) fn build_xml(&self) -> String {
        let mut parts = Vec::new();

        parts.push(format!(
            "<NFe xmlns=\"{NFE_NAMESPACE}\"><infNFe Id=\"{}\" versao=\"{}\">",
            self.inf_nfe_id, self.inf_nfe_versao
        ));

        parts.push(self.build_ide());
        parts.push(self.build_emit());
        parts.push(self.build_dest());

        if let Some(ref r) = self.retirada_fields {
            parts.push(self.build_local_section("retirada", r));
        }
        if let Some(ref e) = self.entrega_fields {
            parts.push(self.build_local_section("entrega", e));
        }
        for ax in &self.aut_xml_list {
            parts.push(self.build_aut_xml(ax));
        }

        for (i, item) in self.items.iter().enumerate() {
            parts.push(self.build_det(item, i + 1));
        }

        parts.push(self.build_total());
        parts.push(self.build_transp());

        if self.fat_fields.is_some() {
            parts.push(self.build_cobr());
        }

        parts.push(self.build_pag());

        if let Some(ref ii) = self.inf_intermed {
            parts.push(self.build_inf_intermed(ii));
        }

        if self.inf_adic_fields.contains_key("infAdFisco")
            || self.inf_adic_fields.contains_key("infCpl")
            || !self.obs_cont_list.is_empty()
            || !self.obs_fisco_list.is_empty()
            || !self.proc_ref_list.is_empty()
        {
            parts.push(self.build_inf_adic());
        }

        if let Some(ref exp) = self.exporta_fields {
            parts.push(self.build_exporta(exp));
        }
        if let Some(ref cmp) = self.compra_fields {
            parts.push(self.build_compra(cmp));
        }
        if let Some(ref cn) = self.cana_fields {
            parts.push(self.build_cana(cn));
        }
        if let Some(ref rt) = self.inf_resp_tec {
            parts.push(self.build_inf_resp_tec(rt));
        }
        if let Some(ref supl) = self.inf_nfe_supl {
            parts.push(self.build_inf_nfe_supl(supl));
        }

        parts.push("</infNFe></NFe>".into());
        parts.join("")
    }

    fn build_ide(&self) -> String {
        let d = &self.ide_data;
        let mut c = Vec::new();
        add_child(&mut c, "cUF", d.get("cUF").map(|s| s.as_str()));
        let cnf = d.get("cNF").map(|s| s.as_str()).unwrap_or("");
        let cnf_padded = format!("{:0>8}", cnf);
        add_child_str(&mut c, "cNF", &cnf_padded);
        add_child(&mut c, "natOp", d.get("natOp").map(|s| s.as_str()));
        add_child(&mut c, "mod", d.get("mod").map(|s| s.as_str()));
        add_child(&mut c, "serie", d.get("serie").map(|s| s.as_str()));
        add_child(&mut c, "nNF", d.get("nNF").map(|s| s.as_str()));
        add_child(&mut c, "dhEmi", d.get("dhEmi").map(|s| s.as_str()));
        if let Some(v) = d.get("dhSaiEnt") {
            if !v.is_empty() {
                add_child_str(&mut c, "dhSaiEnt", v);
            }
        }
        add_child(&mut c, "tpNF", d.get("tpNF").map(|s| s.as_str()));
        add_child(&mut c, "idDest", d.get("idDest").map(|s| s.as_str()));
        add_child(&mut c, "cMunFG", d.get("cMunFG").map(|s| s.as_str()));
        add_child(&mut c, "tpImp", d.get("tpImp").map(|s| s.as_str()));
        add_child(&mut c, "tpEmis", d.get("tpEmis").map(|s| s.as_str()));
        let cdv = d.get("cDV").map(|s| s.as_str()).unwrap_or("0");
        add_child_str(&mut c, "cDV", cdv);
        add_child(&mut c, "tpAmb", d.get("tpAmb").map(|s| s.as_str()));
        add_child(&mut c, "finNFe", d.get("finNFe").map(|s| s.as_str()));
        add_child(&mut c, "indFinal", d.get("indFinal").map(|s| s.as_str()));
        add_child(&mut c, "indPres", d.get("indPres").map(|s| s.as_str()));
        if let Some(v) = d.get("indIntermed") {
            if !v.is_empty() {
                add_child_str(&mut c, "indIntermed", v);
            }
        }
        add_child(&mut c, "procEmi", d.get("procEmi").map(|s| s.as_str()));
        add_child(&mut c, "verProc", d.get("verProc").map(|s| s.as_str()));

        for r in &self.nf_ref {
            c.push(xml_tag("NFref", &xml_tag("refNFe", r)));
        }
        for nf in &self.nf_ref_nf {
            let mut nc = Vec::new();
            if nf.get("IE").is_some() {
                // refNFP (BA10+BA13/BA14)
                add_child(&mut nc, "cUF", nf.get("cUF").map(|s| s.as_str()));
                add_child(&mut nc, "AAMM", nf.get("AAMM").map(|s| s.as_str()));
                if let Some(v) = nf.get("CNPJ") {
                    add_child_str(&mut nc, "CNPJ", v);
                }
                if let Some(v) = nf.get("CPF") {
                    add_child_str(&mut nc, "CPF", v);
                }
                add_child(&mut nc, "IE", nf.get("IE").map(|s| s.as_str()));
                add_child(&mut nc, "mod", nf.get("mod").map(|s| s.as_str()));
                add_child(&mut nc, "serie", nf.get("serie").map(|s| s.as_str()));
                add_child(&mut nc, "nNF", nf.get("nNF").map(|s| s.as_str()));
                c.push(xml_tag("NFref", &xml_tag("refNFP", &nc.join(""))));
            } else {
                // refNF (BA03)
                add_child(&mut nc, "cUF", nf.get("cUF").map(|s| s.as_str()));
                add_child(&mut nc, "AAMM", nf.get("AAMM").map(|s| s.as_str()));
                add_child(&mut nc, "CNPJ", nf.get("CNPJ").map(|s| s.as_str()));
                add_child(&mut nc, "mod", nf.get("mod").map(|s| s.as_str()));
                add_child(&mut nc, "serie", nf.get("serie").map(|s| s.as_str()));
                add_child(&mut nc, "nNF", nf.get("nNF").map(|s| s.as_str()));
                c.push(xml_tag("NFref", &xml_tag("refNF", &nc.join(""))));
            }
        }
        for cte in &self.nf_ref_cte {
            c.push(xml_tag("NFref", &xml_tag("refCTe", cte)));
        }
        for ecf in &self.nf_ref_ecf {
            let mut ec = Vec::new();
            add_child(&mut ec, "mod", ecf.get("mod").map(|s| s.as_str()));
            add_child(&mut ec, "nECF", ecf.get("nECF").map(|s| s.as_str()));
            add_child(&mut ec, "nCOO", ecf.get("nCOO").map(|s| s.as_str()));
            c.push(xml_tag("NFref", &xml_tag("refECF", &ec.join(""))));
        }

        if let (Some(dhcont), Some(xjust)) = (d.get("dhCont"), d.get("xJust")) {
            if !dhcont.is_empty() && !xjust.is_empty() {
                add_child_str(&mut c, "dhCont", dhcont);
                add_child_str(&mut c, "xJust", xjust);
            }
        }

        xml_tag("ide", &c.join(""))
    }

    fn build_emit(&self) -> String {
        let e = &self.emit_fields;
        let mut c = Vec::new();
        if let Some(v) = e.get("CNPJ") {
            add_child_str(&mut c, "CNPJ", v);
        }
        if let Some(v) = e.get("CPF") {
            add_child_str(&mut c, "CPF", v);
        }
        add_child(&mut c, "xNome", e.get("xNome").map(|s| s.as_str()));
        if let Some(v) = e.get("xFant") {
            if !v.is_empty() {
                add_child_str(&mut c, "xFant", v);
            }
        }

        // enderEmit
        let ee = &self.ender_emit_fields;
        let mut ec = Vec::new();
        add_child(&mut ec, "xLgr", ee.get("xLgr").map(|s| s.as_str()));
        add_child(&mut ec, "nro", ee.get("nro").map(|s| s.as_str()));
        if let Some(v) = ee.get("xCpl") {
            if !v.is_empty() {
                add_child_str(&mut ec, "xCpl", v);
            }
        }
        add_child(&mut ec, "xBairro", ee.get("xBairro").map(|s| s.as_str()));
        add_child(&mut ec, "cMun", ee.get("cMun").map(|s| s.as_str()));
        add_child(&mut ec, "xMun", ee.get("xMun").map(|s| s.as_str()));
        add_child(&mut ec, "UF", ee.get("UF").map(|s| s.as_str()));
        add_child(&mut ec, "CEP", ee.get("CEP").map(|s| s.as_str()));
        if let Some(v) = ee.get("cPais") {
            if !v.is_empty() {
                add_child_str(&mut ec, "cPais", v);
            }
        }
        if let Some(v) = ee.get("xPais") {
            if !v.is_empty() {
                add_child_str(&mut ec, "xPais", v);
            }
        }
        if let Some(v) = ee.get("fone") {
            if !v.is_empty() {
                add_child_str(&mut ec, "fone", v);
            }
        }
        c.push(xml_tag("enderEmit", &ec.join("")));

        if let Some(v) = e.get("IE") {
            if !v.is_empty() {
                add_child_str(&mut c, "IE", v);
            }
        }
        if let Some(v) = e.get("IEST") {
            if !v.is_empty() {
                add_child_str(&mut c, "IEST", v);
            }
        }
        if let Some(v) = e.get("IM") {
            if !v.is_empty() {
                add_child_str(&mut c, "IM", v);
            }
        }
        if let Some(v) = e.get("CNAE") {
            if !v.is_empty() {
                add_child_str(&mut c, "CNAE", v);
            }
        }
        add_child(&mut c, "CRT", e.get("CRT").map(|s| s.as_str()));

        xml_tag("emit", &c.join(""))
    }

    fn build_dest(&self) -> String {
        let d = &self.dest_fields;
        let mut c = Vec::new();
        if let Some(v) = d.get("CNPJ") {
            add_child_str(&mut c, "CNPJ", v);
        }
        if let Some(v) = d.get("CPF") {
            add_child_str(&mut c, "CPF", v);
        }
        if let Some(v) = d.get("idEstrangeiro") {
            add_child_str(&mut c, "idEstrangeiro", v);
        }
        add_child(&mut c, "xNome", d.get("xNome").map(|s| s.as_str()));

        // enderDest
        let ee = &self.ender_dest_fields;
        if ee.get("xLgr").is_some() {
            let mut ec = Vec::new();
            add_child(&mut ec, "xLgr", ee.get("xLgr").map(|s| s.as_str()));
            add_child(&mut ec, "nro", ee.get("nro").map(|s| s.as_str()));
            if let Some(v) = ee.get("xCpl") {
                if !v.is_empty() {
                    add_child_str(&mut ec, "xCpl", v);
                }
            }
            add_child(&mut ec, "xBairro", ee.get("xBairro").map(|s| s.as_str()));
            add_child(&mut ec, "cMun", ee.get("cMun").map(|s| s.as_str()));
            add_child(&mut ec, "xMun", ee.get("xMun").map(|s| s.as_str()));
            add_child(&mut ec, "UF", ee.get("UF").map(|s| s.as_str()));
            add_child(&mut ec, "CEP", ee.get("CEP").map(|s| s.as_str()));
            if let Some(v) = ee.get("cPais") {
                if !v.is_empty() {
                    add_child_str(&mut ec, "cPais", v);
                }
            }
            if let Some(v) = ee.get("xPais") {
                if !v.is_empty() {
                    add_child_str(&mut ec, "xPais", v);
                }
            }
            if let Some(v) = ee.get("fone") {
                if !v.is_empty() {
                    add_child_str(&mut ec, "fone", v);
                }
            }
            c.push(xml_tag("enderDest", &ec.join("")));
        }

        if let Some(v) = d.get("indIEDest") {
            if !v.is_empty() {
                add_child_str(&mut c, "indIEDest", v);
            }
        }
        if let Some(v) = d.get("IE") {
            if !v.is_empty() {
                add_child_str(&mut c, "IE", v);
            }
        }
        if let Some(v) = d.get("ISUF") {
            if !v.is_empty() {
                add_child_str(&mut c, "ISUF", v);
            }
        }
        if let Some(v) = d.get("IM") {
            if !v.is_empty() {
                add_child_str(&mut c, "IM", v);
            }
        }
        if let Some(v) = d.get("email") {
            if !v.is_empty() {
                add_child_str(&mut c, "email", v);
            }
        }

        xml_tag("dest", &c.join(""))
    }

    fn build_det(&self, item: &ItemBuild, n_item: usize) -> String {
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

    fn build_prod(&self, item: &ItemBuild) -> String {
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

    fn build_imposto(&self, item: &ItemBuild) -> String {
        let mut c = Vec::new();
        if !item.v_tot_trib.is_empty() {
            add_child_str_dec(&mut c, "vTotTrib", &item.v_tot_trib, 2);
        }
        if item.icms_data.is_some() {
            c.push(self.build_icms(item));
        }
        if let Some(ref ufdest) = item.icms_ufdest {
            c.push(self.build_icms_ufdest(ufdest));
        }
        if item.ipi_header.is_some() || !item.ipi_cst.is_empty() {
            c.push(self.build_ipi(item));
        }
        if let Some(ref ii) = item.ii {
            c.push(self.build_ii(ii));
        }
        if !item.pis_cst.is_empty() {
            c.push(self.build_pis(item));
        }
        if item.pis_st.is_some() {
            c.push(self.build_pis_st(item));
        }
        if !item.cofins_cst.is_empty() {
            c.push(self.build_cofins(item));
        }
        if item.cofins_st.is_some() {
            c.push(self.build_cofins_st(item));
        }
        if let Some(ref issqn) = item.issqn {
            c.push(self.build_issqn(issqn));
        }
        xml_tag("imposto", &c.join(""))
    }

    fn build_icms(&self, item: &ItemBuild) -> String {
        let d = match &item.icms_data {
            Some(d) => d,
            None => return String::new(),
        };
        let cst = d
            .get("CST")
            .or_else(|| d.get("CSOSN"))
            .map(|s| s.as_str())
            .unwrap_or("");
        let group_tag = icms_group_tag(cst);

        let mut ic = Vec::new();
        add_child(&mut ic, "orig", d.get("orig").map(|s| s.as_str()));
        if let Some(v) = d.get("CST") {
            add_child_str(&mut ic, "CST", v);
        }
        if let Some(v) = d.get("CSOSN") {
            add_child_str(&mut ic, "CSOSN", v);
        }
        for &field in &[
            "modBC",
            "pRedBC",
            "vBC",
            "pICMS",
            "vICMS",
            "vICMSOp",
            "pDif",
            "vICMSDif",
            "vBCFCP",
            "pFCP",
            "vFCP",
            "modBCST",
            "pMVAST",
            "pRedBCST",
            "vBCST",
            "pICMSST",
            "vICMSST",
            "vBCFCPST",
            "pFCPST",
            "vFCPST",
            "vICMSDeson",
            "motDesICMS",
            "vBCSTRet",
            "pST",
            "vICMSSTRet",
            "vICMSSubstituto",
            "vBCFCPSTRet",
            "pFCPSTRet",
            "vFCPSTRet",
            "pRedBCEfet",
            "vBCEfet",
            "pICMSEfet",
            "vICMSEfet",
            "pBCOp",
            "UFST",
            "pCredSN",
            "vCredICMSSN",
            "indDeduzDeson",
        ] {
            if let Some(v) = d.get(field) {
                if !v.is_empty() {
                    let places = icms_field_decimal_places(field);
                    if places > 0 {
                        add_child_str_dec(&mut ic, field, v, places);
                    } else {
                        add_child_str(&mut ic, field, v);
                    }
                }
            }
        }

        xml_tag("ICMS", &xml_tag(&group_tag, &ic.join("")))
    }

    fn build_ipi(&self, item: &ItemBuild) -> String {
        let mut c = Vec::new();
        if let Some(h) = &item.ipi_header {
            if let Some(v) = h.get("qSelo") {
                if !v.is_empty() {
                    add_child_str(&mut c, "qSelo", v);
                }
            }
            if let Some(v) = h.get("cEnq") {
                if !v.is_empty() {
                    add_child_str(&mut c, "cEnq", v);
                }
            }
        }

        let cst = &item.ipi_cst;
        let trib_csts = ["00", "49", "50", "99"];
        if trib_csts.contains(&cst.as_str()) {
            let mut tc = Vec::new();
            add_child_str(&mut tc, "CST", cst);
            if !item.ipi_v_bc.is_empty() {
                add_child_str_dec(&mut tc, "vBC", &item.ipi_v_bc, 2);
            }
            if !item.ipi_p_ipi.is_empty() {
                add_child_str_dec(&mut tc, "pIPI", &item.ipi_p_ipi, 4);
            }
            if !item.ipi_q_unid.is_empty() {
                add_child_str_dec(&mut tc, "qUnid", &item.ipi_q_unid, 4);
            }
            if !item.ipi_v_unid.is_empty() {
                add_child_str_dec(&mut tc, "vUnid", &item.ipi_v_unid, 4);
            }
            if !item.ipi_v_ipi.is_empty() {
                add_child_str_dec(&mut tc, "vIPI", &item.ipi_v_ipi, 2);
            }
            c.push(xml_tag("IPITrib", &tc.join("")));
        } else if !cst.is_empty() {
            c.push(xml_tag("IPINT", &format!("<CST>{cst}</CST>")));
        }

        xml_tag("IPI", &c.join(""))
    }

    fn build_pis(&self, item: &ItemBuild) -> String {
        let cst = &item.pis_cst;
        let nt_csts = ["04", "05", "06", "07", "08", "09"];
        if nt_csts.contains(&cst.as_str()) {
            return xml_tag("PIS", &xml_tag("PISNT", &format!("<CST>{cst}</CST>")));
        }
        let aliq_csts = ["01", "02"];
        let inner_tag = if aliq_csts.contains(&cst.as_str()) {
            "PISAliq"
        } else if cst == "03" {
            "PISQtde"
        } else {
            "PISOutr"
        };
        let mut c = Vec::new();
        add_child_str(&mut c, "CST", cst);
        if !item.pis_v_bc.is_empty() {
            add_child_str_dec(&mut c, "vBC", &item.pis_v_bc, 2);
        }
        if !item.pis_p_pis.is_empty() {
            add_child_str_dec(&mut c, "pPIS", &item.pis_p_pis, 4);
        }
        if !item.pis_q_bc_prod.is_empty() {
            add_child_str_dec(&mut c, "qBCProd", &item.pis_q_bc_prod, 4);
        }
        if !item.pis_v_aliq_prod.is_empty() {
            add_child_str_dec(&mut c, "vAliqProd", &item.pis_v_aliq_prod, 4);
        }
        if !item.pis_v_pis.is_empty() {
            add_child_str_dec(&mut c, "vPIS", &item.pis_v_pis, 2);
        }
        xml_tag("PIS", &xml_tag(inner_tag, &c.join("")))
    }

    fn build_pis_st(&self, item: &ItemBuild) -> String {
        let mut c = Vec::new();
        if !item.pis_st_v_bc.is_empty() {
            add_child_str_dec(&mut c, "vBC", &item.pis_st_v_bc, 2);
        }
        if !item.pis_st_p_pis.is_empty() {
            add_child_str_dec(&mut c, "pPIS", &item.pis_st_p_pis, 4);
        }
        if !item.pis_st_q_bc_prod.is_empty() {
            add_child_str_dec(&mut c, "qBCProd", &item.pis_st_q_bc_prod, 4);
        }
        if !item.pis_st_v_aliq_prod.is_empty() {
            add_child_str_dec(&mut c, "vAliqProd", &item.pis_st_v_aliq_prod, 4);
        }
        if !item.pis_st_v_pis.is_empty() {
            add_child_str_dec(&mut c, "vPIS", &item.pis_st_v_pis, 2);
        }
        xml_tag("PISST", &c.join(""))
    }

    fn build_cofins(&self, item: &ItemBuild) -> String {
        let cst = &item.cofins_cst;
        let nt_csts = ["04", "05", "06", "07", "08", "09"];
        if nt_csts.contains(&cst.as_str()) {
            return xml_tag("COFINS", &xml_tag("COFINSNT", &format!("<CST>{cst}</CST>")));
        }
        let aliq_csts = ["01", "02"];
        let inner_tag = if aliq_csts.contains(&cst.as_str()) {
            "COFINSAliq"
        } else if cst == "03" {
            "COFINSQtde"
        } else {
            "COFINSOutr"
        };
        let mut c = Vec::new();
        add_child_str(&mut c, "CST", cst);
        if !item.cofins_v_bc.is_empty() {
            add_child_str_dec(&mut c, "vBC", &item.cofins_v_bc, 2);
        }
        if !item.cofins_p_cofins.is_empty() {
            add_child_str_dec(&mut c, "pCOFINS", &item.cofins_p_cofins, 4);
        }
        if !item.cofins_q_bc_prod.is_empty() {
            add_child_str_dec(&mut c, "qBCProd", &item.cofins_q_bc_prod, 4);
        }
        if !item.cofins_v_aliq_prod.is_empty() {
            add_child_str_dec(&mut c, "vAliqProd", &item.cofins_v_aliq_prod, 4);
        }
        if !item.cofins_v_cofins.is_empty() {
            add_child_str_dec(&mut c, "vCOFINS", &item.cofins_v_cofins, 2);
        }
        xml_tag("COFINS", &xml_tag(inner_tag, &c.join("")))
    }

    fn build_cofins_st(&self, item: &ItemBuild) -> String {
        let mut c = Vec::new();
        if !item.cofins_st_v_bc.is_empty() {
            add_child_str_dec(&mut c, "vBC", &item.cofins_st_v_bc, 2);
        }
        if !item.cofins_st_p_cofins.is_empty() {
            add_child_str_dec(&mut c, "pCOFINS", &item.cofins_st_p_cofins, 4);
        }
        if !item.cofins_st_q_bc_prod.is_empty() {
            add_child_str_dec(&mut c, "qBCProd", &item.cofins_st_q_bc_prod, 4);
        }
        if !item.cofins_st_v_aliq_prod.is_empty() {
            add_child_str_dec(&mut c, "vAliqProd", &item.cofins_st_v_aliq_prod, 4);
        }
        if !item.cofins_st_v_cofins.is_empty() {
            add_child_str_dec(&mut c, "vCOFINS", &item.cofins_st_v_cofins, 2);
        }
        xml_tag("COFINSST", &c.join(""))
    }

    fn build_total(&self) -> String {
        let t = &self.totals_fields;
        let mut c = Vec::new();
        for &field in &[
            "vBC",
            "vICMS",
            "vICMSDeson",
            "vFCP",
            "vBCST",
            "vST",
            "vFCPST",
            "vFCPSTRet",
            "vProd",
            "vFrete",
            "vSeg",
            "vDesc",
            "vII",
            "vIPI",
            "vIPIDevol",
            "vPIS",
            "vCOFINS",
            "vOutro",
            "vNF",
        ] {
            add_child_dec(&mut c, field, t.get(field).map(|s| s.as_str()), 2);
        }
        if let Some(v) = t.get("vTotTrib") {
            if !v.is_empty() {
                add_child_str_dec(&mut c, "vTotTrib", v, 2);
            }
        }
        xml_tag("total", &xml_tag("ICMSTot", &c.join("")))
    }

    fn build_transp(&self) -> String {
        let mut c = Vec::new();
        add_child(
            &mut c,
            "modFrete",
            self.transp_fields.get("modFrete").map(|s| s.as_str()),
        );

        if let Some(t) = &self.transporta_fields {
            let mut tc = Vec::new();
            if let Some(v) = t.get("CNPJ") {
                add_child_str(&mut tc, "CNPJ", v);
            }
            if let Some(v) = t.get("CPF") {
                add_child_str(&mut tc, "CPF", v);
            }
            add_child(&mut tc, "xNome", t.get("xNome").map(|s| s.as_str()));
            if let Some(v) = t.get("IE") {
                if !v.is_empty() {
                    add_child_str(&mut tc, "IE", v);
                }
            }
            if let Some(v) = t.get("xEnder") {
                if !v.is_empty() {
                    add_child_str(&mut tc, "xEnder", v);
                }
            }
            if let Some(v) = t.get("xMun") {
                if !v.is_empty() {
                    add_child_str(&mut tc, "xMun", v);
                }
            }
            if let Some(v) = t.get("UF") {
                if !v.is_empty() {
                    add_child_str(&mut tc, "UF", v);
                }
            }
            c.push(xml_tag("transporta", &tc.join("")));
        }

        for vol in &self.volumes {
            let mut vc = Vec::new();
            if let Some(v) = vol.get("qVol") {
                if !v.is_empty() {
                    add_child_str(&mut vc, "qVol", v);
                }
            }
            if let Some(v) = vol.get("esp") {
                if !v.is_empty() {
                    add_child_str(&mut vc, "esp", v);
                }
            }
            if let Some(v) = vol.get("marca") {
                if !v.is_empty() {
                    add_child_str(&mut vc, "marca", v);
                }
            }
            if let Some(v) = vol.get("nVol") {
                if !v.is_empty() {
                    add_child_str(&mut vc, "nVol", v);
                }
            }
            if let Some(v) = vol.get("pesoL") {
                if !v.is_empty() {
                    add_child_str_dec(&mut vc, "pesoL", v, 3);
                }
            }
            if let Some(v) = vol.get("pesoB") {
                if !v.is_empty() {
                    add_child_str_dec(&mut vc, "pesoB", v, 3);
                }
            }
            c.push(xml_tag("vol", &vc.join("")));
        }

        xml_tag("transp", &c.join(""))
    }

    fn build_cobr(&self) -> String {
        let mut c = Vec::new();
        if let Some(f) = &self.fat_fields {
            let mut fc = Vec::new();
            add_child(&mut fc, "nFat", f.get("nFat").map(|s| s.as_str()));
            add_child_dec(&mut fc, "vOrig", f.get("vOrig").map(|s| s.as_str()), 2);
            add_child_dec(&mut fc, "vDesc", f.get("vDesc").map(|s| s.as_str()), 2);
            add_child_dec(&mut fc, "vLiq", f.get("vLiq").map(|s| s.as_str()), 2);
            c.push(xml_tag("fat", &fc.join("")));
        }
        for dup in &self.dup_items {
            let mut dc = Vec::new();
            add_child(&mut dc, "nDup", dup.get("nDup").map(|s| s.as_str()));
            add_child(&mut dc, "dVenc", dup.get("dVenc").map(|s| s.as_str()));
            add_child_dec(&mut dc, "vDup", dup.get("vDup").map(|s| s.as_str()), 2);
            c.push(xml_tag("dup", &dc.join("")));
        }
        xml_tag("cobr", &c.join(""))
    }

    fn build_pag(&self) -> String {
        let mut c = Vec::new();
        for dp in &self.det_pag_list {
            let mut dc = Vec::new();
            if let Some(v) = dp.get("indPag") {
                if !v.is_empty() {
                    add_child_str(&mut dc, "indPag", v);
                }
            }
            add_child(&mut dc, "tPag", dp.get("tPag").map(|s| s.as_str()));
            if let Some(v) = dp.get("xPag") {
                if !v.is_empty() {
                    add_child_str(&mut dc, "xPag", v);
                }
            }
            add_child_dec(&mut dc, "vPag", dp.get("vPag").map(|s| s.as_str()), 2);
            if let Some(v) = dp.get("dPag") {
                if !v.is_empty() {
                    add_child_str(&mut dc, "dPag", v);
                }
            }
            if let Some(v) = dp.get("CNPJPag") {
                if !v.is_empty() {
                    add_child_str(&mut dc, "CNPJPag", v);
                }
            }
            if let Some(v) = dp.get("UFPag") {
                if !v.is_empty() {
                    add_child_str(&mut dc, "UFPag", v);
                }
            }
            // Card group – only generate <card> for valid XSD values (1 or 2).
            // tpIntegra=0 is not part of the NF-e enumeration; PHP's empty("0")
            // also suppresses it, so we replicate that behaviour here.
            if let Some(tp) = dp.get("tpIntegra") {
                if tp == "1" || tp == "2" {
                    let mut cc = Vec::new();
                    add_child_str(&mut cc, "tpIntegra", tp);
                    if let Some(v) = dp.get("CNPJ") {
                        if !v.is_empty() {
                            add_child_str(&mut cc, "CNPJ", v);
                        }
                    }
                    if let Some(v) = dp.get("tBand") {
                        if !v.is_empty() {
                            add_child_str(&mut cc, "tBand", v);
                        }
                    }
                    if let Some(v) = dp.get("cAut") {
                        if !v.is_empty() {
                            add_child_str(&mut cc, "cAut", v);
                        }
                    }
                    if let Some(v) = dp.get("CNPJReceb") {
                        if !v.is_empty() {
                            add_child_str(&mut cc, "CNPJReceb", v);
                        }
                    }
                    if let Some(v) = dp.get("idTermPag") {
                        if !v.is_empty() {
                            add_child_str(&mut cc, "idTermPag", v);
                        }
                    }
                    dc.push(xml_tag("card", &cc.join("")));
                }
            }
            c.push(xml_tag("detPag", &dc.join("")));
        }
        if let Some(pf) = &self.pag_fields {
            if let Some(v) = pf.get("vTroco") {
                if !v.is_empty() {
                    add_child_str_dec(&mut c, "vTroco", v, 2);
                }
            }
        }
        xml_tag("pag", &c.join(""))
    }

    fn build_inf_adic(&self) -> String {
        let mut c = Vec::new();
        if let Some(v) = self.inf_adic_fields.get("infAdFisco") {
            if !v.is_empty() {
                add_child_str(&mut c, "infAdFisco", v);
            }
        }
        if let Some(v) = self.inf_adic_fields.get("infCpl") {
            if !v.is_empty() {
                add_child_str(&mut c, "infCpl", v);
            }
        }
        xml_tag("infAdic", &c.join(""))
    }

    fn build_local_section(&self, tag_name: &str, f: &Fields) -> String {
        let mut c = Vec::new();
        if let Some(v) = f.get("CNPJ") {
            add_child_str(&mut c, "CNPJ", v);
        }
        if let Some(v) = f.get("CPF") {
            add_child_str(&mut c, "CPF", v);
        }
        if let Some(v) = f.get("xNome") {
            if !v.is_empty() {
                add_child_str(&mut c, "xNome", v);
            }
        }
        add_child(&mut c, "xLgr", f.get("xLgr").map(|s| s.as_str()));
        add_child(&mut c, "nro", f.get("nro").map(|s| s.as_str()));
        if let Some(v) = f.get("xCpl") {
            if !v.is_empty() {
                add_child_str(&mut c, "xCpl", v);
            }
        }
        add_child(&mut c, "xBairro", f.get("xBairro").map(|s| s.as_str()));
        add_child(&mut c, "cMun", f.get("cMun").map(|s| s.as_str()));
        add_child(&mut c, "xMun", f.get("xMun").map(|s| s.as_str()));
        add_child(&mut c, "UF", f.get("UF").map(|s| s.as_str()));
        if let Some(v) = f.get("CEP") {
            if !v.is_empty() {
                add_child_str(&mut c, "CEP", v);
            }
        }
        if let Some(v) = f.get("cPais") {
            if !v.is_empty() {
                add_child_str(&mut c, "cPais", v);
            }
        }
        if let Some(v) = f.get("xPais") {
            if !v.is_empty() {
                add_child_str(&mut c, "xPais", v);
            }
        }
        if let Some(v) = f.get("fone") {
            if !v.is_empty() {
                add_child_str(&mut c, "fone", v);
            }
        }
        if let Some(v) = f.get("email") {
            if !v.is_empty() {
                add_child_str(&mut c, "email", v);
            }
        }
        if let Some(v) = f.get("IE") {
            if !v.is_empty() {
                add_child_str(&mut c, "IE", v);
            }
        }
        xml_tag(tag_name, &c.join(""))
    }

    fn build_aut_xml(&self, ax: &Fields) -> String {
        let mut c = Vec::new();
        if let Some(v) = ax.get("CNPJ") {
            add_child_str(&mut c, "CNPJ", v);
        }
        if let Some(v) = ax.get("CPF") {
            add_child_str(&mut c, "CPF", v);
        }
        xml_tag("autXML", &c.join(""))
    }

    fn build_icms_ufdest(&self, d: &Fields) -> String {
        let mut c = Vec::new();
        add_child_dec(
            &mut c,
            "vBCUFDest",
            d.get("vBCUFDest").map(|s| s.as_str()),
            2,
        );
        add_child_dec(
            &mut c,
            "vBCFCPUFDest",
            d.get("vBCFCPUFDest").map(|s| s.as_str()),
            2,
        );
        add_child_dec(
            &mut c,
            "pFCPUFDest",
            d.get("pFCPUFDest").map(|s| s.as_str()),
            4,
        );
        add_child_dec(
            &mut c,
            "pICMSUFDest",
            d.get("pICMSUFDest").map(|s| s.as_str()),
            4,
        );
        add_child_dec(
            &mut c,
            "pICMSInter",
            d.get("pICMSInter").map(|s| s.as_str()),
            4,
        );
        add_child_dec(
            &mut c,
            "pICMSInterPart",
            d.get("pICMSInterPart").map(|s| s.as_str()),
            4,
        );
        add_child_dec(
            &mut c,
            "vFCPUFDest",
            d.get("vFCPUFDest").map(|s| s.as_str()),
            2,
        );
        add_child_dec(
            &mut c,
            "vICMSUFDest",
            d.get("vICMSUFDest").map(|s| s.as_str()),
            2,
        );
        add_child_dec(
            &mut c,
            "vICMSUFRemet",
            d.get("vICMSUFRemet").map(|s| s.as_str()),
            2,
        );
        xml_tag("ICMSUFDest", &c.join(""))
    }

    fn build_ii(&self, d: &Fields) -> String {
        let mut c = Vec::new();
        add_child_dec(&mut c, "vBC", d.get("vBC").map(|s| s.as_str()), 2);
        add_child_dec(&mut c, "vDespAdu", d.get("vDespAdu").map(|s| s.as_str()), 2);
        add_child_dec(&mut c, "vII", d.get("vII").map(|s| s.as_str()), 2);
        add_child_dec(&mut c, "vIOF", d.get("vIOF").map(|s| s.as_str()), 2);
        xml_tag("II", &c.join(""))
    }

    fn build_issqn(&self, d: &Fields) -> String {
        let mut c = Vec::new();
        add_child_dec(&mut c, "vBC", d.get("vBC").map(|s| s.as_str()), 2);
        add_child_dec(&mut c, "vAliq", d.get("vAliq").map(|s| s.as_str()), 4);
        add_child_dec(&mut c, "vISSQN", d.get("vISSQN").map(|s| s.as_str()), 2);
        add_child(&mut c, "cMunFG", d.get("cMunFG").map(|s| s.as_str()));
        add_child(&mut c, "cListServ", d.get("cListServ").map(|s| s.as_str()));
        for &f in &["vDeducao", "vOutro", "vDescIncond", "vDescCond", "vISSRet"] {
            if let Some(v) = d.get(f) {
                if !v.is_empty() {
                    add_child_str_dec(&mut c, f, v, 2);
                }
            }
        }
        add_child(&mut c, "indISS", d.get("indISS").map(|s| s.as_str()));
        for &f in &["cServico", "cMun", "cPais", "nProcesso"] {
            if let Some(v) = d.get(f) {
                if !v.is_empty() {
                    add_child_str(&mut c, f, v);
                }
            }
        }
        add_child(
            &mut c,
            "indIncentivo",
            d.get("indIncentivo").map(|s| s.as_str()),
        );
        xml_tag("ISSQN", &c.join(""))
    }

    fn build_imposto_devol(&self, d: &Fields) -> String {
        let mut c = Vec::new();
        add_child_dec(&mut c, "pDevol", d.get("pDevol").map(|s| s.as_str()), 2);
        if let Some(v) = d.get("vIPIDevol") {
            if !v.is_empty() {
                let mut ic = Vec::new();
                add_child_str_dec(&mut ic, "vIPIDevol", v, 2);
                c.push(xml_tag("IPI", &ic.join("")));
            }
        }
        xml_tag("impostoDevol", &c.join(""))
    }

    fn build_inf_intermed(&self, d: &Fields) -> String {
        let mut c = Vec::new();
        add_child(&mut c, "CNPJ", d.get("CNPJ").map(|s| s.as_str()));
        if let Some(v) = d.get("idCadIntTran") {
            if !v.is_empty() {
                add_child_str(&mut c, "idCadIntTran", v);
            }
        }
        xml_tag("infIntermed", &c.join(""))
    }

    fn build_exporta(&self, d: &Fields) -> String {
        let mut c = Vec::new();
        add_child(
            &mut c,
            "UFSaidaPais",
            d.get("UFSaidaPais").map(|s| s.as_str()),
        );
        add_child(
            &mut c,
            "xLocExporta",
            d.get("xLocExporta").map(|s| s.as_str()),
        );
        if let Some(v) = d.get("xLocDespacho") {
            if !v.is_empty() {
                add_child_str(&mut c, "xLocDespacho", v);
            }
        }
        xml_tag("exporta", &c.join(""))
    }

    fn build_compra(&self, d: &Fields) -> String {
        let mut c = Vec::new();
        for &f in &["xNEmp", "xPed", "xCont"] {
            if let Some(v) = d.get(f) {
                if !v.is_empty() {
                    add_child_str(&mut c, f, v);
                }
            }
        }
        xml_tag("compra", &c.join(""))
    }

    fn build_cana(&self, d: &Fields) -> String {
        let mut c = Vec::new();
        add_child(&mut c, "safra", d.get("safra").map(|s| s.as_str()));
        add_child(&mut c, "ref", d.get("ref").map(|s| s.as_str()));
        for fd in &self.cana_for_dia {
            let mut fc = Vec::new();
            add_child(&mut fc, "dia", fd.get("dia").map(|s| s.as_str()));
            add_child_dec(&mut fc, "qtde", fd.get("qtde").map(|s| s.as_str()), 10);
            c.push(xml_tag("forDia", &fc.join("")));
        }
        add_child_dec(&mut c, "qTotMes", d.get("qTotMes").map(|s| s.as_str()), 10);
        add_child_dec(&mut c, "qTotAnt", d.get("qTotAnt").map(|s| s.as_str()), 10);
        add_child_dec(&mut c, "qTotGer", d.get("qTotGer").map(|s| s.as_str()), 10);
        for ded in &self.cana_deduc {
            let mut dc = Vec::new();
            add_child(&mut dc, "xDed", ded.get("xDed").map(|s| s.as_str()));
            add_child_dec(&mut dc, "vDed", ded.get("vDed").map(|s| s.as_str()), 2);
            c.push(xml_tag("deduc", &dc.join("")));
        }
        add_child_dec(&mut c, "vFor", d.get("vFor").map(|s| s.as_str()), 2);
        add_child_dec(&mut c, "vTotDed", d.get("vTotDed").map(|s| s.as_str()), 2);
        add_child_dec(&mut c, "vLiqFor", d.get("vLiqFor").map(|s| s.as_str()), 2);
        xml_tag("cana", &c.join(""))
    }

    fn build_inf_resp_tec(&self, d: &Fields) -> String {
        let mut c = Vec::new();
        add_child(&mut c, "CNPJ", d.get("CNPJ").map(|s| s.as_str()));
        add_child(&mut c, "xContato", d.get("xContato").map(|s| s.as_str()));
        add_child(&mut c, "email", d.get("email").map(|s| s.as_str()));
        add_child(&mut c, "fone", d.get("fone").map(|s| s.as_str()));
        for &f in &["CSRT", "idCSRT"] {
            if let Some(v) = d.get(f) {
                if !v.is_empty() {
                    add_child_str(&mut c, f, v);
                }
            }
        }
        xml_tag("infRespTec", &c.join(""))
    }

    fn build_inf_nfe_supl(&self, d: &Fields) -> String {
        let mut c = Vec::new();
        if let Some(v) = d.get("qrcode") {
            c.push(format!("<qrCode>{}</qrCode>", escape_xml(v)));
        }
        if let Some(v) = d.get("urlChave") {
            c.push(format!("<urlChave>{}</urlChave>", escape_xml(v)));
        }
        xml_tag("infNFeSupl", &c.join(""))
    }
}

fn icms_group_tag(cst: &str) -> String {
    match cst {
        "00" => "ICMS00".into(),
        "10" => "ICMS10".into(),
        "20" => "ICMS20".into(),
        "30" => "ICMS30".into(),
        "40" | "41" | "50" => "ICMS40".into(),
        "51" => "ICMS51".into(),
        "60" => "ICMS60".into(),
        "70" => "ICMS70".into(),
        "90" => "ICMS90".into(),
        // Simples Nacional CSOSN
        "101" => "ICMSSN101".into(),
        "102" | "103" | "300" | "400" => "ICMSSN102".into(),
        "201" => "ICMSSN201".into(),
        "202" | "203" => "ICMSSN202".into(),
        "500" => "ICMSSN500".into(),
        "900" => "ICMSSN900".into(),
        _ => format!("ICMS{cst}"),
    }
}

fn icms_field_decimal_places(field: &str) -> usize {
    match field {
        // Percentage fields → 4 decimal places
        "pRedBC" | "pICMS" | "pDif" | "pFCP" | "pMVAST" | "pRedBCST" | "pICMSST" | "pFCPST"
        | "pST" | "pFCPSTRet" | "pRedBCEfet" | "pICMSEfet" | "pBCOp" | "pCredSN" => 4,
        // Value fields → 2 decimal places
        "vBC" | "vICMS" | "vICMSOp" | "vICMSDif" | "vBCFCP" | "vFCP" | "vBCST" | "vICMSST"
        | "vBCFCPST" | "vFCPST" | "vICMSDeson" | "vBCSTRet" | "vICMSSTRet" | "vICMSSubstituto"
        | "vBCFCPSTRet" | "vFCPSTRet" | "vBCEfet" | "vICMSEfet" | "vCredICMSSN" => 2,
        // Non-decimal fields
        _ => 0,
    }
}
