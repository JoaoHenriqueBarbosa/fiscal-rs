//! IDE (identification) section builder.

use super::helpers::*;
use super::parser::NFeParser;

impl<'a> NFeParser<'a> {
    pub(super) fn build_ide(&self) -> String {
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
}
