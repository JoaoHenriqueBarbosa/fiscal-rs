//! Build the `<pag>` (payment) group of the NF-e XML.

use crate::constants::payment_types;
use crate::format_utils::format_cents;
use crate::newtypes::Cents;
use crate::types::{PaymentCardDetail, PaymentData};
use crate::xml_utils::{TagContent, tag};

/// Build the `<pag>` element with payment methods and optional change.
pub fn build_pag(
    payments: &[PaymentData],
    change_amount: Option<Cents>,
    card_details: Option<&[PaymentCardDetail]>,
) -> String {
    if payments.is_empty() {
        return tag(
            "pag",
            &[],
            TagContent::Children(vec![tag(
                "detPag",
                &[],
                TagContent::Children(vec![
                    tag("tPag", &[], TagContent::Text(payment_types::NONE)),
                    tag("vPag", &[], TagContent::Text("0.00")),
                ]),
            )]),
        );
    }

    let fc2 = |c: i64| format_cents(c, 2);
    let mut det_pag_elements: Vec<String> = payments
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let mut det_children = Vec::new();

            // indPag (before tPag per PHP schema)
            if let Some(ref ind) = p.ind_pag {
                det_children.push(tag("indPag", &[], TagContent::Text(ind)));
            }

            det_children.push(tag("tPag", &[], TagContent::Text(&p.method)));

            // xPag
            if let Some(ref xpag) = p.x_pag {
                det_children.push(tag("xPag", &[], TagContent::Text(xpag)));
            }

            det_children.push(tag("vPag", &[], TagContent::Text(&fc2(p.amount.0))));

            // dPag
            if let Some(ref dpag) = p.d_pag {
                det_children.push(tag("dPag", &[], TagContent::Text(dpag)));
            }

            // CNPJPag / UFPag (NT 2023.004)
            if let Some(ref cnpj) = p.cnpj_pag {
                det_children.push(tag("CNPJPag", &[], TagContent::Text(cnpj)));
            }
            if let Some(ref uf) = p.uf_pag {
                det_children.push(tag("UFPag", &[], TagContent::Text(uf)));
            }

            // Card details
            if let Some(cards) = card_details {
                if let Some(card) = cards.get(i) {
                    if let Some(ref integ) = card.integ_type {
                        let mut card_children =
                            vec![tag("tpIntegra", &[], TagContent::Text(integ))];
                        if let Some(ref tid) = card.card_tax_id {
                            card_children.push(tag("CNPJ", &[], TagContent::Text(tid)));
                        }
                        if let Some(ref brand) = card.card_brand {
                            card_children.push(tag("tBand", &[], TagContent::Text(brand)));
                        }
                        if let Some(ref auth) = card.auth_code {
                            card_children.push(tag("cAut", &[], TagContent::Text(auth)));
                        }
                        det_children.push(tag("card", &[], TagContent::Children(card_children)));
                    }
                }
            }

            tag("detPag", &[], TagContent::Children(det_children))
        })
        .collect();

    // vTroco after all detPag elements
    if let Some(change) = change_amount {
        if change.0 > 0 {
            det_pag_elements.push(tag("vTroco", &[], TagContent::Text(&fc2(change.0))));
        }
    }

    tag("pag", &[], TagContent::Children(det_pag_elements))
}
