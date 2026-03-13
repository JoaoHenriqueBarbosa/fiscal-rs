//! Optional trailing sections (infAdic, exporta, compra, cana, etc).

use super::helpers::*;
use super::parser::NFeParser;
use crate::xml_utils::escape_xml;

impl<'a> NFeParser<'a> {
    pub(super) fn build_inf_adic(&self) -> String {
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

    pub(super) fn build_inf_intermed(&self, d: &Fields) -> String {
        let mut c = Vec::new();
        add_child(&mut c, "CNPJ", d.get("CNPJ").map(|s| s.as_str()));
        if let Some(v) = d.get("idCadIntTran") {
            if !v.is_empty() {
                add_child_str(&mut c, "idCadIntTran", v);
            }
        }
        xml_tag("infIntermed", &c.join(""))
    }

    pub(super) fn build_exporta(&self, d: &Fields) -> String {
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

    pub(super) fn build_compra(&self, d: &Fields) -> String {
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

    pub(super) fn build_cana(&self, d: &Fields) -> String {
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

    pub(super) fn build_inf_resp_tec(&self, d: &Fields) -> String {
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

    pub(super) fn build_inf_nfe_supl(&self, d: &Fields) -> String {
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
