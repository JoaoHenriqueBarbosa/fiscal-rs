//! Tax section builders (ICMS, IPI, PIS, COFINS, ISSQN, II, IBS/CBS).

use super::helpers::*;
use super::parser::NFeParser;
use super::types::ItemBuild;

impl<'a> NFeParser<'a> {
    pub(super) fn build_imposto(&self, item: &ItemBuild) -> String {
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

    pub(super) fn build_icms(&self, item: &ItemBuild) -> String {
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

    pub(super) fn build_ipi(&self, item: &ItemBuild) -> String {
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

    pub(super) fn build_pis(&self, item: &ItemBuild) -> String {
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

    pub(super) fn build_pis_st(&self, item: &ItemBuild) -> String {
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

    pub(super) fn build_cofins(&self, item: &ItemBuild) -> String {
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

    pub(super) fn build_cofins_st(&self, item: &ItemBuild) -> String {
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

    pub(super) fn build_icms_ufdest(&self, d: &Fields) -> String {
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

    pub(super) fn build_ii(&self, d: &Fields) -> String {
        let mut c = Vec::new();
        add_child_dec(&mut c, "vBC", d.get("vBC").map(|s| s.as_str()), 2);
        add_child_dec(&mut c, "vDespAdu", d.get("vDespAdu").map(|s| s.as_str()), 2);
        add_child_dec(&mut c, "vII", d.get("vII").map(|s| s.as_str()), 2);
        add_child_dec(&mut c, "vIOF", d.get("vIOF").map(|s| s.as_str()), 2);
        xml_tag("II", &c.join(""))
    }

    pub(super) fn build_issqn(&self, d: &Fields) -> String {
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

    pub(super) fn build_imposto_devol(&self, d: &Fields) -> String {
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
