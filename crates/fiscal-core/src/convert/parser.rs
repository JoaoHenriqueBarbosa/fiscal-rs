//! NFeParser: state machine that accumulates TXT entities into XML.

use super::helpers::*;
use std::collections::HashMap;
use super::types::{DiEntry, ItemBuild};

pub(super) struct NFeParser<'a> {
    structure: &'a HashMap<&'a str, &'a str>,
    pub(super) base_layout: String,

    // Header
    pub(super) inf_nfe_id: String,
    pub(super) inf_nfe_versao: String,

    // Collected data
    pub(super) ide_data: Fields,
    pub(super) g_compra_gov: Option<Fields>,
    pub(super) nf_ref: Vec<String>,
    pub(super) nf_ref_nf: Vec<Fields>,
    pub(super) nf_ref_nfp_pending: Option<Fields>,
    pub(super) nf_ref_cte: Vec<String>,
    pub(super) nf_ref_ecf: Vec<Fields>,
    bb02_ref_nfe: Vec<String>,
    pub(super) emit_fields: Fields,
    pub(super) ender_emit_fields: Fields,
    pub(super) dest_fields: Fields,
    pub(super) ender_dest_fields: Fields,
    pub(super) retirada_fields: Option<Fields>,
    pub(super) entrega_fields: Option<Fields>,
    pub(super) aut_xml_list: Vec<Fields>,
    pub(super) aut_xml_pending: Option<Fields>,
    pub(super) items: Vec<ItemBuild>,
    pub(super) totals_fields: Fields,
    pub(super) issqn_tot_fields: Option<Fields>,
    pub(super) ret_trib_fields: Option<Fields>,
    pub(super) ibs_cbs_tot_fields: Option<Fields>,
    pub(super) transp_fields: Fields,
    pub(super) ret_transp: Option<Fields>,
    pub(super) transporta_fields: Option<Fields>,
    pub(super) veic_transp: Option<Fields>,
    pub(super) reboque_list: Vec<Fields>,
    pub(super) vagao: Option<String>,
    pub(super) balsa: Option<String>,
    pub(super) volumes: Vec<Fields>,
    pub(super) cur_vol_lacres: Vec<String>,
    pub(super) fat_fields: Option<Fields>,
    pub(super) dup_items: Vec<Fields>,
    pub(super) pag_fields: Option<Fields>,
    pub(super) det_pag_list: Vec<Fields>,
    pub(super) inf_intermed: Option<Fields>,
    pub(super) inf_adic_fields: Fields,
    pub(super) obs_cont_list: Vec<Fields>,
    pub(super) obs_fisco_list: Vec<Fields>,
    pub(super) proc_ref_list: Vec<Fields>,
    pub(super) exporta_fields: Option<Fields>,
    pub(super) compra_fields: Option<Fields>,
    pub(super) cana_fields: Option<Fields>,
    pub(super) cana_for_dia: Vec<Fields>,
    pub(super) cana_deduc: Vec<Fields>,
    pub(super) inf_resp_tec: Option<Fields>,
    pub(super) inf_nfe_supl: Option<Fields>,

    // Current item accumulation
    pub(super) current_item_num: usize,
    pub(super) cur_inf_ad_prod: String,
    pub(super) cur_prod: Fields,
    pub(super) cur_cest: Option<Fields>,
    pub(super) cur_g_cred: Option<Fields>,
    pub(super) cur_nve_list: Vec<String>,
    pub(super) cur_di_list: Vec<DiEntry>,
    pub(super) cur_det_export_list: Vec<Fields>,
    pub(super) cur_rastro_list: Vec<Fields>,
    pub(super) cur_veic_prod: Option<Fields>,
    pub(super) cur_med: Option<Fields>,
    pub(super) cur_arma_list: Vec<Fields>,
    pub(super) cur_comb: Option<Fields>,
    pub(super) cur_comb_cide: Option<Fields>,
    pub(super) cur_encerrante: Option<Fields>,
    pub(super) cur_recopi: Option<String>,
    pub(super) cur_v_tot_trib: String,
    pub(super) cur_icms_tag: String,
    pub(super) cur_icms_data: Option<Fields>,
    pub(super) cur_icms_ufdest: Option<Fields>,
    pub(super) cur_ipi_header: Option<Fields>,
    pub(super) cur_ipi_cst: String,
    pub(super) cur_ipi_v_ipi: String,
    pub(super) cur_ipi_v_bc: String,
    pub(super) cur_ipi_p_ipi: String,
    pub(super) cur_ipi_q_unid: String,
    pub(super) cur_ipi_v_unid: String,
    pub(super) cur_ii: Option<Fields>,
    pub(super) cur_pis_cst: String,
    pub(super) cur_pis_v_bc: String,
    pub(super) cur_pis_p_pis: String,
    pub(super) cur_pis_v_pis: String,
    pub(super) cur_pis_q_bc_prod: String,
    pub(super) cur_pis_v_aliq_prod: String,
    pub(super) cur_pis_st: Option<Fields>,
    pub(super) cur_pis_st_v_bc: String,
    pub(super) cur_pis_st_p_pis: String,
    pub(super) cur_pis_st_q_bc_prod: String,
    pub(super) cur_pis_st_v_aliq_prod: String,
    pub(super) cur_pis_st_v_pis: String,
    pub(super) cur_cofins_cst: String,
    pub(super) cur_cofins_v_bc: String,
    pub(super) cur_cofins_p_cofins: String,
    pub(super) cur_cofins_v_cofins: String,
    pub(super) cur_cofins_q_bc_prod: String,
    pub(super) cur_cofins_v_aliq_prod: String,
    pub(super) cur_cofins_st: Option<Fields>,
    pub(super) cur_cofins_st_v_bc: String,
    pub(super) cur_cofins_st_p_cofins: String,
    pub(super) cur_cofins_st_q_bc_prod: String,
    pub(super) cur_cofins_st_v_aliq_prod: String,
    pub(super) cur_cofins_st_v_cofins: String,
    pub(super) cur_issqn: Option<Fields>,
    pub(super) cur_imposto_devol: Option<Fields>,
}

impl<'a> NFeParser<'a> {
    pub(super) fn new(version: &str, layout: &str, structure: &'a HashMap<&'a str, &'a str>) -> Self {
        let _ = version;
        Self {
            structure,
            base_layout: layout.to_uppercase(),
            inf_nfe_id: String::new(),
            inf_nfe_versao: "4.00".into(),
            ide_data: Fields::new(),
            g_compra_gov: None,
            nf_ref: Vec::new(),
            nf_ref_nf: Vec::new(),
            nf_ref_nfp_pending: None,
            nf_ref_cte: Vec::new(),
            nf_ref_ecf: Vec::new(),
            bb02_ref_nfe: Vec::new(),
            emit_fields: Fields::new(),
            ender_emit_fields: Fields::new(),
            dest_fields: Fields::new(),
            ender_dest_fields: Fields::new(),
            retirada_fields: None,
            entrega_fields: None,
            aut_xml_list: Vec::new(),
            aut_xml_pending: None,
            items: Vec::new(),
            totals_fields: Fields::new(),
            issqn_tot_fields: None,
            ret_trib_fields: None,
            ibs_cbs_tot_fields: None,
            transp_fields: Fields::new(),
            ret_transp: None,
            transporta_fields: None,
            veic_transp: None,
            reboque_list: Vec::new(),
            vagao: None,
            balsa: None,
            volumes: Vec::new(),
            cur_vol_lacres: Vec::new(),
            fat_fields: None,
            dup_items: Vec::new(),
            pag_fields: None,
            det_pag_list: Vec::new(),
            inf_intermed: None,
            inf_adic_fields: Fields::new(),
            obs_cont_list: Vec::new(),
            obs_fisco_list: Vec::new(),
            proc_ref_list: Vec::new(),
            exporta_fields: None,
            compra_fields: None,
            cana_fields: None,
            cana_for_dia: Vec::new(),
            cana_deduc: Vec::new(),
            inf_resp_tec: None,
            inf_nfe_supl: None,
            current_item_num: 0,
            cur_inf_ad_prod: String::new(),
            cur_prod: Fields::new(),
            cur_cest: None,
            cur_g_cred: None,
            cur_nve_list: Vec::new(),
            cur_di_list: Vec::new(),
            cur_det_export_list: Vec::new(),
            cur_rastro_list: Vec::new(),
            cur_veic_prod: None,
            cur_med: None,
            cur_arma_list: Vec::new(),
            cur_comb: None,
            cur_comb_cide: None,
            cur_encerrante: None,
            cur_recopi: None,
            cur_v_tot_trib: String::new(),
            cur_icms_tag: String::new(),
            cur_icms_data: None,
            cur_icms_ufdest: None,
            cur_ipi_header: None,
            cur_ipi_cst: String::new(),
            cur_ipi_v_ipi: String::new(),
            cur_ipi_v_bc: String::new(),
            cur_ipi_p_ipi: String::new(),
            cur_ipi_q_unid: String::new(),
            cur_ipi_v_unid: String::new(),
            cur_ii: None,
            cur_pis_cst: String::new(),
            cur_pis_v_bc: String::new(),
            cur_pis_p_pis: String::new(),
            cur_pis_v_pis: String::new(),
            cur_pis_q_bc_prod: String::new(),
            cur_pis_v_aliq_prod: String::new(),
            cur_pis_st: None,
            cur_pis_st_v_bc: String::new(),
            cur_pis_st_p_pis: String::new(),
            cur_pis_st_q_bc_prod: String::new(),
            cur_pis_st_v_aliq_prod: String::new(),
            cur_pis_st_v_pis: String::new(),
            cur_cofins_cst: String::new(),
            cur_cofins_v_bc: String::new(),
            cur_cofins_p_cofins: String::new(),
            cur_cofins_v_cofins: String::new(),
            cur_cofins_q_bc_prod: String::new(),
            cur_cofins_v_aliq_prod: String::new(),
            cur_cofins_st: None,
            cur_cofins_st_v_bc: String::new(),
            cur_cofins_st_p_cofins: String::new(),
            cur_cofins_st_q_bc_prod: String::new(),
            cur_cofins_st_v_aliq_prod: String::new(),
            cur_cofins_st_v_cofins: String::new(),
            cur_issqn: None,
            cur_imposto_devol: None,
        }
    }

    pub(super) fn parse(&mut self, invoice: &[&str]) {
        for line in invoice {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let fields: Vec<&str> = trimmed.split('|').collect();
            let ref_upper = fields[0].to_uppercase();
            let struct_def = match self.structure.get(ref_upper.as_str()) {
                Some(d) => *d,
                None => continue,
            };
            let std = fields_to_std(&fields, struct_def);
            self.handle(&ref_upper, &std);
        }
        self.finalize_current_item();
    }

    fn handle(&mut self, ref_name: &str, std: &Fields) {
        match ref_name {
            "A" => {
                self.inf_nfe_versao = std.get("versao").cloned().unwrap_or_else(|| "4.00".into());
                self.inf_nfe_id = std.get("Id").cloned().unwrap_or_default();
            }
            "B" => {
                self.ide_data = std.clone();
            }
            "BA" | "BB" => {}
            "BA02" => {
                if let Some(v) = std.get("refNFe") {
                    self.nf_ref.push(v.clone());
                }
            }
            "BA03" => {
                self.nf_ref_nf.push(std.clone());
            }
            "BA10" => {
                self.nf_ref_nfp_pending = Some(std.clone());
            }
            "BA13" => {
                if let Some(ref mut nfp) = self.nf_ref_nfp_pending {
                    if let Some(v) = std.get("CNPJ") {
                        nfp.insert("CNPJ".into(), v.clone());
                    }
                }
                if let Some(nfp) = self.nf_ref_nfp_pending.take() {
                    self.nf_ref_nf.push(nfp);
                }
            }
            "BA14" => {
                if let Some(ref mut nfp) = self.nf_ref_nfp_pending {
                    if let Some(v) = std.get("CPF") {
                        nfp.insert("CPF".into(), v.clone());
                    }
                }
                if let Some(nfp) = self.nf_ref_nfp_pending.take() {
                    self.nf_ref_nf.push(nfp);
                }
            }
            "BA19" => {
                if let Some(v) = std.get("refCTe") {
                    self.nf_ref_cte.push(v.clone());
                }
            }
            "BA20" => {
                self.nf_ref_ecf.push(std.clone());
            }
            "BB02" | "BB01" => {
                if let Some(v) = std.get("refNFe") {
                    self.bb02_ref_nfe.push(v.clone());
                }
            }
            "C" => {
                self.emit_fields = std.clone();
            }
            "C02" => {
                if let Some(v) = std.get("CNPJ") {
                    self.emit_fields.insert("CNPJ".into(), v.clone());
                }
            }
            "C02A" => {
                if let Some(v) = std.get("CPF") {
                    self.emit_fields.insert("CPF".into(), v.clone());
                }
            }
            "C05" => {
                self.ender_emit_fields = std.clone();
            }
            "D" => {}
            "E" => {
                self.dest_fields = std.clone();
            }
            "E02" => {
                if let Some(v) = std.get("CNPJ") {
                    self.dest_fields.insert("CNPJ".into(), v.clone());
                }
            }
            "E03" => {
                if let Some(v) = std.get("CPF") {
                    self.dest_fields.insert("CPF".into(), v.clone());
                }
            }
            "E03A" => {
                let v = std.get("idEstrangeiro").cloned().unwrap_or_default();
                self.dest_fields.insert("idEstrangeiro".into(), v);
            }
            "E05" => {
                self.ender_dest_fields = std.clone();
            }
            "F" => {
                self.retirada_fields = Some(std.clone());
            }
            "F02" => {
                if let Some(ref mut r) = self.retirada_fields {
                    if let Some(v) = std.get("CNPJ") {
                        r.insert("CNPJ".into(), v.clone());
                    }
                }
            }
            "F02A" => {
                if let Some(ref mut r) = self.retirada_fields {
                    if let Some(v) = std.get("CPF") {
                        r.insert("CPF".into(), v.clone());
                    }
                }
            }
            "F02B" => {
                if let Some(ref mut r) = self.retirada_fields {
                    if let Some(v) = std.get("xNome") {
                        r.insert("xNome".into(), v.clone());
                    }
                }
            }
            "G" => {
                self.entrega_fields = Some(std.clone());
            }
            "G02" => {
                if let Some(ref mut e) = self.entrega_fields {
                    if let Some(v) = std.get("CNPJ") {
                        e.insert("CNPJ".into(), v.clone());
                    }
                }
            }
            "G02A" => {
                if let Some(ref mut e) = self.entrega_fields {
                    if let Some(v) = std.get("CPF") {
                        e.insert("CPF".into(), v.clone());
                    }
                }
            }
            "G02B" => {
                if let Some(ref mut e) = self.entrega_fields {
                    if let Some(v) = std.get("xNome") {
                        e.insert("xNome".into(), v.clone());
                    }
                }
            }
            "GA" => {
                self.aut_xml_pending = Some(Fields::new());
            }
            "GA02" => {
                if let Some(ref mut ax) = self.aut_xml_pending {
                    if let Some(v) = std.get("CNPJ") {
                        ax.insert("CNPJ".into(), v.clone());
                    }
                }
                if let Some(ax) = self.aut_xml_pending.take() {
                    self.aut_xml_list.push(ax);
                }
            }
            "GA03" => {
                if let Some(ref mut ax) = self.aut_xml_pending {
                    if let Some(v) = std.get("CPF") {
                        ax.insert("CPF".into(), v.clone());
                    }
                }
                if let Some(ax) = self.aut_xml_pending.take() {
                    self.aut_xml_list.push(ax);
                }
            }
            "H" => {
                self.finalize_current_item();
                self.current_item_num = std.get("item").and_then(|s| s.parse().ok()).unwrap_or(0);
                self.cur_prod = Fields::new();
                self.cur_cest = None;
                self.cur_g_cred = None;
                self.cur_v_tot_trib = String::new();
                self.cur_icms_tag = String::new();
                self.cur_icms_data = None;
                self.cur_ipi_header = None;
                self.cur_ipi_cst = String::new();
                self.cur_ipi_v_ipi = String::new();
                self.cur_ipi_v_bc = String::new();
                self.cur_ipi_p_ipi = String::new();
                self.cur_pis_cst = String::new();
                self.cur_pis_v_bc = String::new();
                self.cur_pis_p_pis = String::new();
                self.cur_pis_v_pis = String::new();
                self.cur_cofins_cst = String::new();
                self.cur_cofins_v_bc = String::new();
                self.cur_cofins_p_cofins = String::new();
                self.cur_cofins_v_cofins = String::new();
            }
            "I" => {
                self.cur_prod = std.clone();
            }
            "I05C" => {
                self.cur_cest = Some(std.clone());
            }
            "I05A" => {
                if let Some(v) = std.get("NVE") {
                    self.cur_nve_list.push(v.clone());
                }
            }
            "I05G" => {
                self.cur_g_cred = Some(std.clone());
            }
            "I18" => {
                self.cur_di_list.push(DiEntry {
                    fields: std.clone(),
                    adi_list: Vec::new(),
                });
            }
            "I25" => {
                if let Some(last_di) = self.cur_di_list.last_mut() {
                    last_di.adi_list.push(std.clone());
                }
            }
            "I50" => {
                self.cur_det_export_list.push(std.clone());
            }
            "I52" => {
                if let Some(last) = self.cur_det_export_list.last_mut() {
                    for (k, v) in std.iter() {
                        if !v.is_empty() {
                            last.insert(k.clone(), v.clone());
                        }
                    }
                }
            }
            "I80" => {
                self.cur_rastro_list.push(std.clone());
            }
            "JA" => {
                self.cur_veic_prod = Some(std.clone());
            }
            "K" => {
                self.cur_med = Some(std.clone());
            }
            "L" => {
                self.cur_arma_list.push(std.clone());
            }
            "LA" => {
                self.cur_comb = Some(std.clone());
            }
            "LA07" => {
                self.cur_comb_cide = Some(std.clone());
            }
            "LA11" => {
                self.cur_encerrante = Some(std.clone());
            }
            "LB" => {
                self.cur_recopi = std.get("nRECOPI").cloned();
            }
            "M" => {
                self.cur_v_tot_trib = std.get("vTotTrib").cloned().unwrap_or_default();
            }
            "N" => {}
            "N02" | "N03" | "N04" | "N05" | "N06" | "N07" | "N08" | "N09" | "N10" => {
                self.cur_icms_tag = ref_name.to_string();
                self.cur_icms_data = Some(std.clone());
            }
            "N10A" | "N10B" | "N10C" | "N10D" | "N10E" | "N10F" | "N10G" | "N10H" => {
                self.cur_icms_tag = ref_name.to_string();
                self.cur_icms_data = Some(std.clone());
            }
            "NA" => {
                self.cur_icms_ufdest = Some(std.clone());
            }
            "O" => {
                self.cur_ipi_header = Some(std.clone());
            }
            "O07" => {
                self.cur_ipi_cst = std.get("CST").cloned().unwrap_or_default();
                self.cur_ipi_v_ipi = std.get("vIPI").cloned().unwrap_or_default();
            }
            "O08" => {
                self.cur_ipi_cst = std.get("CST").cloned().unwrap_or_default();
            }
            "O10" => {
                self.cur_ipi_v_bc = std.get("vBC").cloned().unwrap_or_default();
                self.cur_ipi_p_ipi = std.get("pIPI").cloned().unwrap_or_default();
            }
            "O11" => {
                self.cur_ipi_q_unid = std.get("qUnid").cloned().unwrap_or_default();
                self.cur_ipi_v_unid = std.get("vUnid").cloned().unwrap_or_default();
            }
            "P" => {
                self.cur_ii = Some(std.clone());
            }
            "Q" => {}
            "Q02" => {
                self.cur_pis_cst = std.get("CST").cloned().unwrap_or_default();
                self.cur_pis_v_bc = std.get("vBC").cloned().unwrap_or_default();
                self.cur_pis_p_pis = std.get("pPIS").cloned().unwrap_or_default();
                self.cur_pis_v_pis = std.get("vPIS").cloned().unwrap_or_default();
            }
            "Q03" => {
                self.cur_pis_cst = std.get("CST").cloned().unwrap_or_default();
                self.cur_pis_v_pis = std.get("vPIS").cloned().unwrap_or_default();
                self.cur_pis_q_bc_prod = std.get("qBCProd").cloned().unwrap_or_default();
                self.cur_pis_v_aliq_prod = std.get("vAliqProd").cloned().unwrap_or_default();
            }
            "Q04" => {
                self.cur_pis_cst = std.get("CST").cloned().unwrap_or_default();
            }
            "Q05" => {
                self.cur_pis_cst = std.get("CST").cloned().unwrap_or_default();
                self.cur_pis_v_pis = std.get("vPIS").cloned().unwrap_or_default();
            }
            "Q07" => {
                self.cur_pis_v_bc = std.get("vBC").cloned().unwrap_or_default();
                self.cur_pis_p_pis = std.get("pPIS").cloned().unwrap_or_default();
            }
            "Q10" => {
                self.cur_pis_q_bc_prod = std.get("qBCProd").cloned().unwrap_or_default();
                self.cur_pis_v_aliq_prod = std.get("vAliqProd").cloned().unwrap_or_default();
            }
            "R" => {
                self.cur_pis_st = Some(std.clone());
                self.cur_pis_st_v_pis = std.get("vPIS").cloned().unwrap_or_default();
                self.cur_pis_st_v_bc = String::new();
                self.cur_pis_st_p_pis = String::new();
                self.cur_pis_st_q_bc_prod = String::new();
                self.cur_pis_st_v_aliq_prod = String::new();
            }
            "R02" => {
                self.cur_pis_st_v_bc = std.get("vBC").cloned().unwrap_or_default();
                self.cur_pis_st_p_pis = std.get("pPIS").cloned().unwrap_or_default();
            }
            "R04" => {
                self.cur_pis_st_q_bc_prod = std.get("qBCProd").cloned().unwrap_or_default();
                self.cur_pis_st_v_aliq_prod = std.get("vAliqProd").cloned().unwrap_or_default();
                if let Some(v) = std.get("vPIS") {
                    self.cur_pis_st_v_pis = v.clone();
                }
            }
            "S" => {}
            "S02" => {
                self.cur_cofins_cst = std.get("CST").cloned().unwrap_or_default();
                self.cur_cofins_v_bc = std.get("vBC").cloned().unwrap_or_default();
                self.cur_cofins_p_cofins = std.get("pCOFINS").cloned().unwrap_or_default();
                self.cur_cofins_v_cofins = std.get("vCOFINS").cloned().unwrap_or_default();
            }
            "S03" => {
                self.cur_cofins_cst = std.get("CST").cloned().unwrap_or_default();
                self.cur_cofins_v_cofins = std.get("vCOFINS").cloned().unwrap_or_default();
                self.cur_cofins_q_bc_prod = std.get("qBCProd").cloned().unwrap_or_default();
                self.cur_cofins_v_aliq_prod = std.get("vAliqProd").cloned().unwrap_or_default();
            }
            "S04" => {
                self.cur_cofins_cst = std.get("CST").cloned().unwrap_or_default();
            }
            "S05" => {
                self.cur_cofins_cst = std.get("CST").cloned().unwrap_or_default();
                self.cur_cofins_v_cofins = std.get("vCOFINS").cloned().unwrap_or_default();
            }
            "S07" => {
                self.cur_cofins_v_bc = std.get("vBC").cloned().unwrap_or_default();
                self.cur_cofins_p_cofins = std.get("pCOFINS").cloned().unwrap_or_default();
            }
            "S09" => {
                self.cur_cofins_q_bc_prod = std.get("qBCProd").cloned().unwrap_or_default();
                self.cur_cofins_v_aliq_prod = std.get("vAliqProd").cloned().unwrap_or_default();
            }
            "T" => {
                self.cur_cofins_st = Some(std.clone());
                self.cur_cofins_st_v_cofins = std.get("vCOFINS").cloned().unwrap_or_default();
                self.cur_cofins_st_v_bc = String::new();
                self.cur_cofins_st_p_cofins = String::new();
                self.cur_cofins_st_q_bc_prod = String::new();
                self.cur_cofins_st_v_aliq_prod = String::new();
            }
            "T02" => {
                self.cur_cofins_st_v_bc = std.get("vBC").cloned().unwrap_or_default();
                self.cur_cofins_st_p_cofins = std.get("pCOFINS").cloned().unwrap_or_default();
            }
            "T04" => {
                self.cur_cofins_st_q_bc_prod = std.get("qBCProd").cloned().unwrap_or_default();
                self.cur_cofins_st_v_aliq_prod = std.get("vAliqProd").cloned().unwrap_or_default();
            }
            "U" => {
                self.cur_issqn = Some(std.clone());
            }
            "UA" => {
                self.cur_imposto_devol = Some(std.clone());
            }
            "UB" | "UB01" | "UB12" | "UB73" | "UB78" => {}
            "W" => {
                self.finalize_current_item();
            }
            "W02" => {
                self.totals_fields = std.clone();
            }
            "W03" => {
                self.ibs_cbs_tot_fields = Some(std.clone());
            }
            "W04C" | "W04E" | "W04G" => {}
            "W17" => {
                self.issqn_tot_fields = Some(std.clone());
            }
            "W23" => {
                self.ret_trib_fields = Some(std.clone());
            }
            "X" => {
                self.transp_fields = std.clone();
            }
            "X03" => {
                self.transporta_fields = Some(std.clone());
            }
            "X04" => {
                if let Some(ref mut t) = self.transporta_fields {
                    if let Some(v) = std.get("CNPJ") {
                        t.insert("CNPJ".into(), v.clone());
                    }
                }
            }
            "X05" => {
                if let Some(ref mut t) = self.transporta_fields {
                    if let Some(v) = std.get("CPF") {
                        t.insert("CPF".into(), v.clone());
                    }
                }
            }
            "X11" => {
                self.ret_transp = Some(std.clone());
            }
            "X18" => {
                self.veic_transp = Some(std.clone());
            }
            "X22" => {
                self.reboque_list.push(std.clone());
            }
            "X25A" => {
                self.vagao = std.get("vagao").cloned();
            }
            "X25B" => {
                self.balsa = std.get("balsa").cloned();
            }
            "X26" => {
                if let Some(last_vol) = self.volumes.last_mut() {
                    if !self.cur_vol_lacres.is_empty() {
                        let joined = self.cur_vol_lacres.join(",");
                        last_vol.insert("_lacres".into(), joined);
                        self.cur_vol_lacres.clear();
                    }
                }
                self.volumes.push(std.clone());
            }
            "X33" => {
                self.cur_vol_lacres
                    .push(std.get("nLacre").cloned().unwrap_or_default());
            }
            "Y" => {
                self.pag_fields = Some(std.clone());
            }
            "Y02" => {
                self.fat_fields = Some(std.clone());
            }
            "Y07" => {
                self.dup_items.push(std.clone());
            }
            "YA" => {
                if self.base_layout == "SEBRAE" {
                    self.pag_fields = Some(std.clone());
                } else {
                    self.det_pag_list.push(std.clone());
                }
            }
            "YA01" => {
                self.det_pag_list.push(std.clone());
            }
            "YA04" => {
                if let Some(last) = self.det_pag_list.last_mut() {
                    for (k, v) in std.iter() {
                        if !v.is_empty() {
                            last.insert(k.clone(), v.clone());
                        }
                    }
                }
            }
            "YB" => {
                self.inf_intermed = Some(std.clone());
            }
            "Z" => {
                self.inf_adic_fields = std.clone();
            }
            "Z04" => {
                self.obs_cont_list.push(std.clone());
            }
            "Z07" => {
                self.obs_fisco_list.push(std.clone());
            }
            "Z10" => {
                self.proc_ref_list.push(std.clone());
            }
            "ZA" => {
                self.exporta_fields = Some(std.clone());
            }
            "ZB" => {
                self.compra_fields = Some(std.clone());
            }
            "ZC" => {
                self.cana_fields = Some(std.clone());
            }
            "ZC04" => {
                self.cana_for_dia.push(std.clone());
            }
            "ZC10" => {
                self.cana_deduc.push(std.clone());
            }
            "ZD" => {
                self.inf_resp_tec = Some(std.clone());
            }
            "ZX01" => {
                self.inf_nfe_supl = Some(std.clone());
            }
            _ => {}
        }
    }

    fn finalize_current_item(&mut self) {
        if !self.cur_prod.contains_key("cProd") && !self.cur_prod.contains_key("xProd") {
            return;
        }
        // Flush pending lacres for the last volume
        if let Some(last_vol) = self.volumes.last_mut() {
            if !self.cur_vol_lacres.is_empty() {
                let joined = self.cur_vol_lacres.join(",");
                last_vol.insert("_lacres".into(), joined);
                self.cur_vol_lacres.clear();
            }
        }
        self.items.push(ItemBuild {
            n_item: self.current_item_num,
            inf_ad_prod: std::mem::take(&mut self.cur_inf_ad_prod),
            prod: std::mem::take(&mut self.cur_prod),
            cest: self.cur_cest.take(),
            g_cred: self.cur_g_cred.take(),
            nve_list: std::mem::take(&mut self.cur_nve_list),
            di_list: std::mem::take(&mut self.cur_di_list),
            det_export_list: std::mem::take(&mut self.cur_det_export_list),
            rastro_list: std::mem::take(&mut self.cur_rastro_list),
            veic_prod: self.cur_veic_prod.take(),
            med: self.cur_med.take(),
            arma_list: std::mem::take(&mut self.cur_arma_list),
            comb: self.cur_comb.take(),
            comb_cide: self.cur_comb_cide.take(),
            encerrante: self.cur_encerrante.take(),
            recopi: self.cur_recopi.take(),
            v_tot_trib: std::mem::take(&mut self.cur_v_tot_trib),
            icms_tag: std::mem::take(&mut self.cur_icms_tag),
            icms_data: self.cur_icms_data.take(),
            icms_ufdest: self.cur_icms_ufdest.take(),
            ipi_header: self.cur_ipi_header.take(),
            ipi_cst: std::mem::take(&mut self.cur_ipi_cst),
            ipi_v_ipi: std::mem::take(&mut self.cur_ipi_v_ipi),
            ipi_v_bc: std::mem::take(&mut self.cur_ipi_v_bc),
            ipi_p_ipi: std::mem::take(&mut self.cur_ipi_p_ipi),
            ipi_q_unid: std::mem::take(&mut self.cur_ipi_q_unid),
            ipi_v_unid: std::mem::take(&mut self.cur_ipi_v_unid),
            ii: self.cur_ii.take(),
            pis_cst: std::mem::take(&mut self.cur_pis_cst),
            pis_v_bc: std::mem::take(&mut self.cur_pis_v_bc),
            pis_p_pis: std::mem::take(&mut self.cur_pis_p_pis),
            pis_v_pis: std::mem::take(&mut self.cur_pis_v_pis),
            pis_q_bc_prod: std::mem::take(&mut self.cur_pis_q_bc_prod),
            pis_v_aliq_prod: std::mem::take(&mut self.cur_pis_v_aliq_prod),
            pis_st: self.cur_pis_st.take(),
            pis_st_v_bc: std::mem::take(&mut self.cur_pis_st_v_bc),
            pis_st_p_pis: std::mem::take(&mut self.cur_pis_st_p_pis),
            pis_st_q_bc_prod: std::mem::take(&mut self.cur_pis_st_q_bc_prod),
            pis_st_v_aliq_prod: std::mem::take(&mut self.cur_pis_st_v_aliq_prod),
            pis_st_v_pis: std::mem::take(&mut self.cur_pis_st_v_pis),
            cofins_cst: std::mem::take(&mut self.cur_cofins_cst),
            cofins_v_bc: std::mem::take(&mut self.cur_cofins_v_bc),
            cofins_p_cofins: std::mem::take(&mut self.cur_cofins_p_cofins),
            cofins_v_cofins: std::mem::take(&mut self.cur_cofins_v_cofins),
            cofins_q_bc_prod: std::mem::take(&mut self.cur_cofins_q_bc_prod),
            cofins_v_aliq_prod: std::mem::take(&mut self.cur_cofins_v_aliq_prod),
            cofins_st: self.cur_cofins_st.take(),
            cofins_st_v_bc: std::mem::take(&mut self.cur_cofins_st_v_bc),
            cofins_st_p_cofins: std::mem::take(&mut self.cur_cofins_st_p_cofins),
            cofins_st_q_bc_prod: std::mem::take(&mut self.cur_cofins_st_q_bc_prod),
            cofins_st_v_aliq_prod: std::mem::take(&mut self.cur_cofins_st_v_aliq_prod),
            cofins_st_v_cofins: std::mem::take(&mut self.cur_cofins_st_v_cofins),
            issqn: self.cur_issqn.take(),
            imposto_devol: self.cur_imposto_devol.take(),
        });
    }
}
