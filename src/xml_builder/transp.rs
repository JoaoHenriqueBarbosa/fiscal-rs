//! Build the `<transp>` (transport) group of the NF-e XML.

use crate::format_utils::{format_cents, format_decimal};
use crate::types::InvoiceBuildData;
use crate::xml_utils::{tag, TagContent};
use super::tax_id::TaxId;

/// Build the `<transp>` element with carrier, vehicle, volumes, etc.
pub fn build_transp(data: &InvoiceBuildData) -> String {
    let Some(ref t) = data.transport else {
        return tag("transp", &[], TagContent::Children(vec![
            tag("modFrete", &[], TagContent::Text("9")),
        ]));
    };

    let mut children = vec![
        tag("modFrete", &[], TagContent::Text(&t.freight_mode)),
    ];

    // retTransp (before transporta per schema)
    if let Some(ref r) = t.retained_icms {
        children.push(tag("retTransp", &[], TagContent::Children(vec![
            tag("vServ", &[], TagContent::Text(&format_cents(r.v_bc_ret.0, 2))),
            tag("vBCRet", &[], TagContent::Text(&format_cents(r.v_bc_ret.0, 2))),
            tag("pICMSRet", &[], TagContent::Text(&format_decimal(r.p_icms_ret.0 as f64 / 100.0, 4))),
            tag("vICMSRet", &[], TagContent::Text(&format_cents(r.v_icms_ret.0, 2))),
            tag("CFOP", &[], TagContent::Text(&r.cfop)),
            tag("cMunFG", &[], TagContent::Text(&r.city_code.0)),
        ])));
    }

    // transporta (carrier)
    if let Some(ref c) = t.carrier {
        let mut carrier_children = Vec::new();
        if let Some(ref tid) = c.tax_id {
            let t = TaxId::new(tid);
            let padded = t.padded();
            carrier_children.push(tag(t.tag_name(), &[], TagContent::Text(&padded)));
        }
        if let Some(ref name) = c.name {
            carrier_children.push(tag("xNome", &[], TagContent::Text(name)));
        }
        if let Some(ref ie) = c.state_tax_id {
            carrier_children.push(tag("IE", &[], TagContent::Text(ie)));
        }
        if let Some(ref addr) = c.address {
            carrier_children.push(tag("xEnder", &[], TagContent::Text(addr)));
        }
        if let Some(ref uf) = c.state_code {
            carrier_children.push(tag("UF", &[], TagContent::Text(uf)));
        }
        children.push(tag("transporta", &[], TagContent::Children(carrier_children)));
    }

    // veicTransp
    if let Some(ref v) = t.vehicle {
        let mut veic_children = vec![
            tag("placa", &[], TagContent::Text(&v.plate)),
            tag("UF", &[], TagContent::Text(&v.state_code)),
        ];
        if let Some(ref rntc) = v.rntc {
            veic_children.push(tag("RNTC", &[], TagContent::Text(rntc)));
        }
        children.push(tag("veicTransp", &[], TagContent::Children(veic_children)));
    }

    // reboque (trailers)
    if let Some(ref trailers) = t.trailers {
        for trailer in trailers {
            let mut trl_children = vec![
                tag("placa", &[], TagContent::Text(&trailer.plate)),
                tag("UF", &[], TagContent::Text(&trailer.state_code)),
            ];
            if let Some(ref rntc) = trailer.rntc {
                trl_children.push(tag("RNTC", &[], TagContent::Text(rntc)));
            }
            children.push(tag("reboque", &[], TagContent::Children(trl_children)));
        }
    }

    // vol (volumes)
    if let Some(ref volumes) = t.volumes {
        for vol in volumes {
            let mut vol_children = Vec::new();
            if let Some(qty) = vol.quantity {
                vol_children.push(tag("qVol", &[], TagContent::Text(&qty.to_string())));
            }
            if let Some(ref sp) = vol.species {
                vol_children.push(tag("esp", &[], TagContent::Text(sp)));
            }
            if let Some(ref brand) = vol.brand {
                vol_children.push(tag("marca", &[], TagContent::Text(brand)));
            }
            if let Some(ref num) = vol.number {
                vol_children.push(tag("nVol", &[], TagContent::Text(num)));
            }
            if let Some(w) = vol.net_weight {
                vol_children.push(tag("pesoL", &[], TagContent::Text(&format_decimal(w, 3))));
            }
            if let Some(w) = vol.gross_weight {
                vol_children.push(tag("pesoB", &[], TagContent::Text(&format_decimal(w, 3))));
            }
            if let Some(ref seals) = vol.seals {
                for seal in seals {
                    vol_children.push(tag("lacres", &[], TagContent::Children(vec![
                        tag("nLacre", &[], TagContent::Text(seal)),
                    ])));
                }
            }
            children.push(tag("vol", &[], TagContent::Children(vol_children)));
        }
    }

    tag("transp", &[], TagContent::Children(children))
}
