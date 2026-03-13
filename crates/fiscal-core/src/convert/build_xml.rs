//! Top-level XML orchestration for the TXT converter.

use super::parser::NFeParser;
use crate::constants::NFE_NAMESPACE;

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
}
