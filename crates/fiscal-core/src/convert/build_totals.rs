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
        xml_tag("total", &xml_tag("ICMSTot", &c.join("")))
    }

    pub(super) fn build_transp(&self) -> String {
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
