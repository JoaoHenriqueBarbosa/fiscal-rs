//! Build the `<dest>` (recipient) group of the NF-e XML.

use super::tax_id::TaxId;
use crate::types::{InvoiceBuildData, InvoiceModel};
use crate::xml_utils::{TagContent, tag};

/// Build the `<dest>` element. Falls back to issuer address when
/// recipient address fields are empty (SEFAZ requires address for model 55).
pub(crate) fn build_dest(data: &InvoiceBuildData) -> Option<String> {
    let r = data.recipient.as_ref()?;

    let tid = TaxId::new(&r.tax_id);
    let padded = tid.padded();
    let tax_id_tag = tag(tid.tag_name(), &[], TagContent::Text(&padded));

    let is_nfce = data.model == InvoiceModel::Nfce;
    let iss = &data.issuer;

    let uf = r.state_code.as_deref().unwrap_or(&iss.state_code);

    let mut children = vec![tax_id_tag];

    if !r.name.is_empty() {
        children.push(tag("xNome", &[], TagContent::Text(&r.name)));
    }

    if !is_nfce {
        let street = r
            .street
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(&iss.street);
        let street_number = r
            .street_number
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(&iss.street_number);
        let district = r
            .district
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(&iss.district);
        let city_code = r
            .city_code
            .as_ref()
            .filter(|c| !c.0.is_empty())
            .map(|c| c.0.as_str())
            .unwrap_or(&iss.city_code.0);
        let city_name = r
            .city_name
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(&iss.city_name);
        let zip_code = r
            .zip_code
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(&iss.zip_code);

        let mut addr_children = vec![
            tag("xLgr", &[], TagContent::Text(street)),
            tag("nro", &[], TagContent::Text(street_number)),
        ];
        if let Some(ref cpl) = r.complement {
            addr_children.push(tag("xCpl", &[], TagContent::Text(cpl)));
        }
        addr_children.extend([
            tag("xBairro", &[], TagContent::Text(district)),
            tag("cMun", &[], TagContent::Text(city_code)),
            tag("xMun", &[], TagContent::Text(city_name)),
            tag("UF", &[], TagContent::Text(uf)),
            tag("CEP", &[], TagContent::Text(zip_code)),
            tag("cPais", &[], TagContent::Text("1058")),
            tag("xPais", &[], TagContent::Text("Brasil")),
        ]);

        children.push(tag("enderDest", &[], TagContent::Children(addr_children)));
    }

    children.push(tag(
        "indIEDest",
        &[],
        TagContent::Text(if r.state_tax_id.is_some() { "1" } else { "9" }),
    ));
    if let Some(ref ie) = r.state_tax_id {
        children.push(tag("IE", &[], TagContent::Text(ie)));
    }

    Some(tag("dest", &[], TagContent::Children(children)))
}
