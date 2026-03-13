//! Build the `<emit>` (issuer/emitter) group of the NF-e XML.

use super::tax_id::TaxId;
use crate::types::InvoiceBuildData;
use crate::xml_utils::{TagContent, tag};

/// Build the `<emit>` element with issuer data and address.
pub(crate) fn build_emit(data: &InvoiceBuildData) -> String {
    let iss = &data.issuer;
    let tid = TaxId::new(&iss.tax_id);

    let mut children = vec![
        tag(tid.tag_name(), &[], TagContent::Text(&iss.tax_id)),
        tag("xNome", &[], TagContent::Text(&iss.company_name)),
    ];

    if let Some(ref trade_name) = iss.trade_name {
        children.push(tag("xFant", &[], TagContent::Text(trade_name)));
    }

    children.push(tag(
        "enderEmit",
        &[],
        TagContent::Children(build_address_fields(
            &iss.street,
            &iss.street_number,
            iss.address_complement.as_deref(),
            &iss.district,
            &iss.city_code.0,
            &iss.city_name,
            &iss.state_code,
            Some(&iss.zip_code),
            true,
            iss.phone.as_deref(),
        )),
    ));
    children.push(tag("IE", &[], TagContent::Text(&iss.state_tax_id)));

    if let Some(ref iest) = iss.iest {
        children.push(tag("IEST", &[], TagContent::Text(iest)));
    }
    if let Some(ref im) = iss.im {
        children.push(tag("IM", &[], TagContent::Text(im)));
        if let Some(ref cnae) = iss.cnae {
            children.push(tag("CNAE", &[], TagContent::Text(cnae)));
        }
    }

    children.push(tag(
        "CRT",
        &[],
        TagContent::Text(&(iss.tax_regime as u8).to_string()),
    ));

    tag("emit", &[], TagContent::Children(children))
}

/// Build address child tags (xLgr … fone), reused for emit/dest/retirada/entrega.
#[allow(clippy::too_many_arguments)]
pub fn build_address_fields(
    street: &str,
    number: &str,
    complement: Option<&str>,
    district: &str,
    city_code: &str,
    city_name: &str,
    state_code: &str,
    zip_code: Option<&str>,
    include_country: bool,
    phone: Option<&str>,
) -> Vec<String> {
    let mut fields = vec![
        tag("xLgr", &[], TagContent::Text(street)),
        tag("nro", &[], TagContent::Text(number)),
    ];
    if let Some(cpl) = complement {
        fields.push(tag("xCpl", &[], TagContent::Text(cpl)));
    }
    fields.extend([
        tag("xBairro", &[], TagContent::Text(district)),
        tag("cMun", &[], TagContent::Text(city_code)),
        tag("xMun", &[], TagContent::Text(city_name)),
        tag("UF", &[], TagContent::Text(state_code)),
    ]);
    if let Some(cep) = zip_code {
        fields.push(tag("CEP", &[], TagContent::Text(cep)));
    }
    if include_country {
        fields.push(tag("cPais", &[], TagContent::Text("1058")));
        fields.push(tag("xPais", &[], TagContent::Text("Brasil")));
    }
    if let Some(fone) = phone {
        fields.push(tag("fone", &[], TagContent::Text(fone)));
    }
    fields
}
