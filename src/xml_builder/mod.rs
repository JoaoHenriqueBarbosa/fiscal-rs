//! NF-e/NFC-e XML builder module.
//!
//! Provides [`InvoiceBuilder`] — a typestate builder that enforces the
//! invoice lifecycle at compile time:
//!
//! ```text
//! InvoiceBuilder::new(issuer, env, model)   // Draft
//!     .series(1)
//!     .invoice_number(42)
//!     .add_item(item)
//!     .recipient(recipient)
//!     .payments(vec![payment])
//!     .build()?                              // Built
//!     .xml()                                 // &str
//! ```

pub mod access_key;
pub mod tax_id;
pub mod ide;
pub mod emit;
pub mod dest;
pub mod det;
pub mod total;
pub mod transp;
pub mod pag;
pub mod optional;
mod builder;

pub use builder::{Draft, Built, InvoiceBuilder};
pub use access_key::build_access_key;

use crate::constants::{NFE_NAMESPACE, NFE_VERSION};
use crate::newtypes::IbgeCode;
use crate::state_codes::STATE_IBGE_CODES;
use crate::tax_icms::{create_icms_totals, merge_icms_totals};
use crate::types::{AccessKeyParams, InvoiceBuildData, InvoiceXmlResult};
use crate::xml_utils::{tag, TagContent};
use crate::FiscalError;

/// Internal XML generation from a fully populated [`InvoiceBuildData`].
///
/// Called by [`InvoiceBuilder::build`]; not part of the public API.
fn generate_xml(data: &InvoiceBuildData) -> Result<InvoiceXmlResult, FiscalError> {
    let state_ibge = STATE_IBGE_CODES
        .get(data.issuer.state_code.as_str())
        .copied()
        .ok_or_else(|| FiscalError::InvalidStateCode(data.issuer.state_code.clone()))?;

    let numeric_code = access_key::generate_numeric_code();
    let year_month = access_key::format_year_month(&data.issued_at);

    let ak_params = AccessKeyParams {
        state_code: IbgeCode(state_ibge.to_string()),
        year_month,
        tax_id: data.issuer.tax_id.clone(),
        model: data.model,
        series: data.series,
        number: data.number,
        emission_type: data.emission_type,
        numeric_code: numeric_code.clone(),
    };

    let access_key = build_access_key(&ak_params)?;
    let inf_nfe_id = format!("NFe{access_key}");

    // Build items and accumulate tax totals
    let mut icms_totals = create_icms_totals();
    let mut total_products: i64 = 0;
    let mut total_ipi: i64 = 0;
    let mut total_pis: i64 = 0;
    let mut total_cofins: i64 = 0;
    let mut total_ii: i64 = 0;

    let mut det_elements = Vec::with_capacity(data.items.len());
    for item in &data.items {
        total_products += item.total_price.0;
        let det_result = det::build_det(item, data)?;
        merge_icms_totals(&mut icms_totals, &det_result.icms_totals);
        total_ipi += det_result.v_ipi;
        total_pis += det_result.v_pis;
        total_cofins += det_result.v_cofins;
        total_ii += det_result.v_ii;
        det_elements.push(det_result.xml);
    }

    // Assemble infNFe children in schema order
    let mut inf_children = vec![
        ide::build_ide(data, state_ibge, &numeric_code, &access_key),
        emit::build_emit(data),
    ];

    if let Some(dest_xml) = dest::build_dest(data) {
        inf_children.push(dest_xml);
    }

    if let Some(ref w) = data.withdrawal {
        inf_children.push(optional::build_withdrawal(w));
    }
    if let Some(ref d) = data.delivery {
        inf_children.push(optional::build_delivery(d));
    }
    if let Some(ref auths) = data.authorized_xml {
        for a in auths {
            inf_children.push(optional::build_aut_xml(a));
        }
    }

    inf_children.extend(det_elements);

    inf_children.push(total::build_total(
        total_products,
        &icms_totals,
        &total::OtherTotals {
            v_ipi: total_ipi,
            v_pis: total_pis,
            v_cofins: total_cofins,
            v_ii: total_ii,
        },
        data.ret_trib.as_ref(),
    ));

    inf_children.push(transp::build_transp(data));

    if let Some(ref billing) = data.billing {
        inf_children.push(optional::build_cobr(billing));
    }

    inf_children.push(pag::build_pag(
        &data.payments,
        data.change_amount,
        data.payment_card_details.as_deref(),
    ));

    if let Some(ref intermed) = data.intermediary {
        inf_children.push(optional::build_intermediary(intermed));
    }

    let inf_adic = optional::build_inf_adic(data);
    if !inf_adic.is_empty() {
        inf_children.push(inf_adic);
    }

    if let Some(ref exp) = data.export {
        inf_children.push(optional::build_export(exp));
    }
    if let Some(ref purchase) = data.purchase {
        inf_children.push(optional::build_purchase(purchase));
    }
    if let Some(ref tech) = data.tech_responsible {
        inf_children.push(optional::build_tech_responsible(tech));
    }

    let inf_nfe = tag(
        "infNFe",
        &[("xmlns", NFE_NAMESPACE), ("versao", NFE_VERSION), ("Id", &inf_nfe_id)],
        TagContent::Children(inf_children),
    );

    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>{}",
        tag("NFe", &[("xmlns", NFE_NAMESPACE)], TagContent::Children(vec![inf_nfe])),
    );

    Ok(InvoiceXmlResult { xml, access_key })
}
