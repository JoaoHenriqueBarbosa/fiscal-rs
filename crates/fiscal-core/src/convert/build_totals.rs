//! Total, transport, billing, and payment builders.

use super::helpers::*;
use super::parser::NFeParser;

impl<'a> NFeParser<'a> {
    pub(super) fn build_total(&self) -> String {
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
        let mut total_parts = vec![xml_tag("ICMSTot", &c.join(""))];

        // ISSQNtot (W17)
        if let Some(ref it) = self.issqn_tot_fields {
            let mut ic = Vec::new();
            for &f in &[
                "vServ",
                "vBC",
                "vISS",
                "vPIS",
                "vCOFINS",
                "dCompet",
                "vDeducao",
                "vOutro",
                "vDescIncond",
                "vDescCond",
                "vISSRet",
                "cRegTrib",
            ] {
                if let Some(v) = it.get(f) {
                    if !v.is_empty() {
                        if f == "dCompet" || f == "cRegTrib" {
                            add_child_str(&mut ic, f, v);
                        } else {
                            add_child_str_dec(&mut ic, f, v, 2);
                        }
                    }
                }
            }
            if !ic.is_empty() {
                total_parts.push(xml_tag("ISSQNtot", &ic.join("")));
            }
        }

        // retTrib (W23)
        if let Some(ref rt) = self.ret_trib_fields {
            let mut rc = Vec::new();
            for &f in &[
                "vRetPIS",
                "vRetCOFINS",
                "vRetCSLL",
                "vBCIRRF",
                "vIRRF",
                "vBCRetPrev",
                "vRetPrev",
            ] {
                if let Some(v) = rt.get(f) {
                    if !v.is_empty() {
                        add_child_str_dec(&mut rc, f, v, 2);
                    }
                }
            }
            if !rc.is_empty() {
                total_parts.push(xml_tag("retTrib", &rc.join("")));
            }
        }

        xml_tag("total", &total_parts.join(""))
    }

    pub(super) fn build_transp(&self) -> String {
        let mut c = Vec::new();
        add_child(
            &mut c,
            "modFrete",
            self.transp_fields.get("modFrete").map(|s| s.as_str()),
        );

        // retTransp (X11)
        if let Some(ref rt) = self.ret_transp {
            let mut rc = Vec::new();
            for &f in &["vServ", "vBCRet", "pICMSRet", "vICMSRet"] {
                if let Some(v) = rt.get(f) {
                    if !v.is_empty() {
                        add_child_str_dec(&mut rc, f, v, 2);
                    }
                }
            }
            add_child(&mut rc, "CFOP", rt.get("CFOP").map(|s| s.as_str()));
            add_child(&mut rc, "cMunFG", rt.get("cMunFG").map(|s| s.as_str()));
            c.push(xml_tag("retTransp", &rc.join("")));
        }

        if let Some(t) = &self.transporta_fields {
            let mut tc = Vec::new();
            if let Some(v) = t.get("CNPJ") {
                add_child_str(&mut tc, "CNPJ", v);
            }
            if let Some(v) = t.get("CPF") {
                add_child_str(&mut tc, "CPF", v);
            }
            add_child(&mut tc, "xNome", t.get("xNome").map(|s| s.as_str()));
            for &f in &["IE", "xEnder", "xMun", "UF"] {
                if let Some(v) = t.get(f) {
                    if !v.is_empty() {
                        add_child_str(&mut tc, f, v);
                    }
                }
            }
            c.push(xml_tag("transporta", &tc.join("")));
        }

        // veicTransp (X18)
        if let Some(ref vt) = self.veic_transp {
            let mut vc = Vec::new();
            add_child(&mut vc, "placa", vt.get("placa").map(|s| s.as_str()));
            add_child(&mut vc, "UF", vt.get("UF").map(|s| s.as_str()));
            if let Some(v) = vt.get("RNTC") {
                if !v.is_empty() {
                    add_child_str(&mut vc, "RNTC", v);
                }
            }
            c.push(xml_tag("veicTransp", &vc.join("")));
        }

        // reboque (X22)
        for reb in &self.reboque_list {
            let mut rc = Vec::new();
            add_child(&mut rc, "placa", reb.get("placa").map(|s| s.as_str()));
            add_child(&mut rc, "UF", reb.get("UF").map(|s| s.as_str()));
            if let Some(v) = reb.get("RNTC") {
                if !v.is_empty() {
                    add_child_str(&mut rc, "RNTC", v);
                }
            }
            c.push(xml_tag("reboque", &rc.join("")));
        }

        // vagao (X25A)
        if let Some(ref v) = self.vagao {
            c.push(xml_tag("vagao", v));
        }
        // balsa (X25B)
        if let Some(ref v) = self.balsa {
            c.push(xml_tag("balsa", v));
        }

        for vol in &self.volumes {
            let mut vc = Vec::new();
            for &f in &["qVol", "esp", "marca", "nVol"] {
                if let Some(v) = vol.get(f) {
                    if !v.is_empty() {
                        add_child_str(&mut vc, f, v);
                    }
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
            // lacres inside vol
            if let Some(lacres) = vol.get("_lacres") {
                for lac in lacres.split(',') {
                    if !lac.is_empty() {
                        vc.push(xml_tag("lacres", &xml_tag("nLacre", lac)));
                    }
                }
            }
            c.push(xml_tag("vol", &vc.join("")));
        }

        xml_tag("transp", &c.join(""))
    }

    pub(super) fn build_cobr(&self) -> String {
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

    pub(super) fn build_pag(&self) -> String {
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
}
