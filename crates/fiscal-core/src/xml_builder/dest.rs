//! Build the `<dest>` (recipient) group of the NF-e XML.

use super::tax_id::TaxId;
use crate::types::{InvoiceBuildData, InvoiceModel, SefazEnvironment};
use crate::xml_utils::{TagContent, tag};

/// Constant used when emitting in homologation environment.
const HOMOLOGATION_NAME: &str = "NF-E EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL";

/// Build the `<dest>` element. Falls back to issuer address when
/// recipient address fields are empty (SEFAZ requires address for model 55).
pub(crate) fn build_dest(data: &InvoiceBuildData) -> Option<String> {
    let r = data.recipient.as_ref()?;

    let is_nfce = data.model == InvoiceModel::Nfce;
    let is_homologation = data.environment == SefazEnvironment::Homologation;
    let iss = &data.issuer;
    let uf = r.state_code.as_deref().unwrap_or(&iss.state_code);

    // Tax ID: CNPJ (14 digits), CPF (11 digits), or idEstrangeiro
    let tid = TaxId::new(&r.tax_id);
    let is_foreign = !r.tax_id.chars().all(|c| c.is_ascii_digit())
        || (r.tax_id.len() != 11 && r.tax_id.len() != 14);

    let mut children = Vec::new();

    if is_foreign {
        // Foreign recipient: use idEstrangeiro
        children.push(tag("idEstrangeiro", &[], TagContent::Text(&r.tax_id)));
    } else {
        let padded = tid.padded();
        children.push(tag(tid.tag_name(), &[], TagContent::Text(&padded)));
    }

    // xNome: substitute in homologation
    if is_homologation && (!r.name.is_empty() || is_nfce) {
        children.push(tag("xNome", &[], TagContent::Text(HOMOLOGATION_NAME)));
    } else if !r.name.is_empty() {
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
        let country_code = r.country_code.as_deref().unwrap_or("1058");
        let country_name = r.country_name.as_deref().unwrap_or("Brasil");

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
            tag("cPais", &[], TagContent::Text(country_code)),
            tag("xPais", &[], TagContent::Text(country_name)),
        ]);
        if let Some(ref fone) = r.phone {
            addr_children.push(tag("fone", &[], TagContent::Text(fone)));
        }

        children.push(tag("enderDest", &[], TagContent::Children(addr_children)));
    }

    // indIEDest: PHP forces "9" for NFC-e; user can override via ind_ie_dest field
    let ind_ie_dest = if is_nfce {
        "9".to_string()
    } else if let Some(ref override_val) = r.ind_ie_dest {
        override_val.clone()
    } else if r.state_tax_id.is_some() {
        "1".to_string()
    } else {
        "9".to_string()
    };
    children.push(tag("indIEDest", &[], TagContent::Text(&ind_ie_dest)));

    // IE: only when not NFC-e and state_tax_id is present
    if !is_nfce {
        if let Some(ref ie) = r.state_tax_id {
            children.push(tag("IE", &[], TagContent::Text(ie)));
        }
    }

    // ISUF (SUFRAMA)
    if let Some(ref isuf) = r.isuf {
        children.push(tag("ISUF", &[], TagContent::Text(isuf)));
    }

    // IM (Inscrição Municipal)
    if let Some(ref im) = r.im {
        children.push(tag("IM", &[], TagContent::Text(im)));
    }

    // email
    if let Some(ref email) = r.email {
        children.push(tag("email", &[], TagContent::Text(email)));
    }

    Some(tag("dest", &[], TagContent::Children(children)))
}
