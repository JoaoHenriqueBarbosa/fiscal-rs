//! Emitter, recipient, withdrawal, delivery, and authorized XML builders.

use super::helpers::*;
use super::parser::NFeParser;

impl<'a> NFeParser<'a> {
    pub(super) fn build_emit(&self) -> String {
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

    pub(super) fn build_dest(&self) -> String {
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

    pub(super) fn build_local_section(&self, tag_name: &str, f: &Fields) -> String {
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

    pub(super) fn build_aut_xml(&self, ax: &Fields) -> String {
        let mut c = Vec::new();
        if let Some(v) = ax.get("CNPJ") {
            add_child_str(&mut c, "CNPJ", v);
        }
        if let Some(v) = ax.get("CPF") {
            add_child_str(&mut c, "CPF", v);
        }
        xml_tag("autXML", &c.join(""))
    }
}
