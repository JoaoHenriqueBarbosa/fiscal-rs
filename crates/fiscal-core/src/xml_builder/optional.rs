//! Build optional XML groups: cobr, infAdic, infIntermed, exporta, compra,
//! infRespTec, retirada, entrega, autXML.

use super::emit::build_address_fields;
use super::tax_id::TaxId;
use crate::format_utils::format_cents;
use crate::types::*;
use crate::xml_utils::{TagContent, tag};
use base64::Engine as _;

/// Build `<cobr>` (billing) element.
pub fn build_cobr(billing: &BillingData) -> String {
    let fc2 = |c: i64| format_cents(c, 2);
    let mut children = Vec::new();

    if let Some(ref inv) = billing.invoice {
        let mut fat_children = vec![
            tag("nFat", &[], TagContent::Text(&inv.number)),
            tag("vOrig", &[], TagContent::Text(&fc2(inv.original_value.0))),
        ];
        if let Some(disc) = inv.discount_value {
            fat_children.push(tag("vDesc", &[], TagContent::Text(&fc2(disc.0))));
        }
        fat_children.push(tag("vLiq", &[], TagContent::Text(&fc2(inv.net_value.0))));
        children.push(tag("fat", &[], TagContent::Children(fat_children)));
    }

    if let Some(ref installments) = billing.installments {
        for inst in installments {
            children.push(tag(
                "dup",
                &[],
                TagContent::Children(vec![
                    tag("nDup", &[], TagContent::Text(&inst.number)),
                    tag("dVenc", &[], TagContent::Text(&inst.due_date)),
                    tag("vDup", &[], TagContent::Text(&fc2(inst.value.0))),
                ]),
            ));
        }
    }

    tag("cobr", &[], TagContent::Children(children))
}

/// Build `<infAdic>` (additional info) element.
pub(crate) fn build_inf_adic(data: &InvoiceBuildData) -> String {
    let mut notes: Vec<String> = Vec::new();

    if let Some(ref cont) = data.contingency {
        notes.push(format!(
            "Emitida em contingencia ({}). Motivo: {}",
            cont.contingency_type.as_str(),
            cont.reason
        ));
    }

    // PHP does NOT auto-add homologation note to infAdic — removed to match PHP

    let add_info = data.additional_info.as_ref();
    let has_additional = add_info.is_some_and(|a| {
        a.taxpayer_note.is_some()
            || a.tax_authority_note.is_some()
            || a.contributor_obs.as_ref().is_some_and(|v| !v.is_empty())
            || a.fiscal_obs.as_ref().is_some_and(|v| !v.is_empty())
            || a.process_refs.as_ref().is_some_and(|v| !v.is_empty())
    });

    if notes.is_empty() && !has_additional {
        return String::new();
    }

    let mut children = Vec::new();

    // infAdFisco before infCpl per schema
    if let Some(note) = add_info.and_then(|a| a.tax_authority_note.as_ref()) {
        children.push(tag("infAdFisco", &[], TagContent::Text(note)));
    }

    // Merge contingency/env notes with taxpayer note
    if let Some(tn) = add_info.and_then(|a| a.taxpayer_note.as_ref()) {
        notes.push(tn.to_string());
    }
    if !notes.is_empty() {
        children.push(tag("infCpl", &[], TagContent::Text(&notes.join("; "))));
    }

    // obsCont (max 10)
    if let Some(obs_list) = add_info.and_then(|a| a.contributor_obs.as_ref()) {
        for obs in obs_list.iter().take(10) {
            children.push(tag(
                "obsCont",
                &[("xCampo", &obs.field)],
                TagContent::Children(vec![tag("xTexto", &[], TagContent::Text(&obs.text))]),
            ));
        }
    }

    // obsFisco (max 10)
    if let Some(obs_list) = add_info.and_then(|a| a.fiscal_obs.as_ref()) {
        for obs in obs_list.iter().take(10) {
            children.push(tag(
                "obsFisco",
                &[("xCampo", &obs.field)],
                TagContent::Children(vec![tag("xTexto", &[], TagContent::Text(&obs.text))]),
            ));
        }
    }

    // procRef (max 100)
    if let Some(procs) = add_info.and_then(|a| a.process_refs.as_ref()) {
        for p in procs.iter().take(100) {
            children.push(tag(
                "procRef",
                &[],
                TagContent::Children(vec![
                    tag("nProc", &[], TagContent::Text(&p.number)),
                    tag("indProc", &[], TagContent::Text(&p.origin)),
                ]),
            ));
        }
    }

    tag("infAdic", &[], TagContent::Children(children))
}

/// Build `<infIntermed>` element.
pub fn build_intermediary(intermed: &IntermediaryData) -> String {
    let mut children = vec![tag("CNPJ", &[], TagContent::Text(&intermed.tax_id))];
    if let Some(ref id) = intermed.id_cad_int_tran {
        children.push(tag("idCadIntTran", &[], TagContent::Text(id)));
    }
    tag("infIntermed", &[], TagContent::Children(children))
}

/// Build `<infRespTec>` element.
///
/// When both `csrt` and `csrt_id` are present, generates `<idCSRT>` and
/// `<hashCSRT>` tags. The hash follows the PHP sped-nfe algorithm:
/// `base64(sha1(CSRT + chNFe, raw_binary))`.
pub fn build_tech_responsible(tech: &TechResponsibleData) -> String {
    build_tech_responsible_with_key(tech, "")
}

/// Build `<infRespTec>` element with access key for CSRT hash computation.
///
/// When both `csrt` and `csrt_id` are present on `tech`, generates `<idCSRT>` and
/// `<hashCSRT>` tags. The hash follows the PHP sped-nfe algorithm:
/// `base64(sha1(CSRT + chNFe, raw_binary))`.
pub fn build_tech_responsible_with_key(tech: &TechResponsibleData, access_key: &str) -> String {
    let mut children = vec![
        tag("CNPJ", &[], TagContent::Text(&tech.tax_id)),
        tag("xContato", &[], TagContent::Text(&tech.contact)),
        tag("email", &[], TagContent::Text(&tech.email)),
    ];
    if let Some(ref phone) = tech.phone {
        children.push(tag("fone", &[], TagContent::Text(phone)));
    }
    if let (Some(csrt), Some(csrt_id)) = (&tech.csrt, &tech.csrt_id) {
        if !access_key.is_empty() {
            children.push(tag("idCSRT", &[], TagContent::Text(csrt_id)));
            let hash = compute_hash_csrt(csrt, access_key);
            children.push(tag("hashCSRT", &[], TagContent::Text(&hash)));
        }
    }
    tag("infRespTec", &[], TagContent::Children(children))
}

/// Compute hashCSRT as defined by the SEFAZ specification.
///
/// Algorithm: `base64(sha1(CSRT + chNFe))` — matching PHP's
/// `base64_encode(sha1($CSRT . $this->chNFe, true))`.
fn compute_hash_csrt(csrt: &str, access_key: &str) -> String {
    use sha1::{Digest, Sha1};
    let combined = format!("{csrt}{access_key}");
    let mut hasher = Sha1::new();
    hasher.update(combined.as_bytes());
    let raw_hash = hasher.finalize();
    base64::engine::general_purpose::STANDARD.encode(raw_hash)
}

/// Build `<compra>` (purchase) element.
pub fn build_purchase(purchase: &PurchaseData) -> String {
    let mut children = Vec::new();
    if let Some(ref note) = purchase.purchase_note {
        children.push(tag("xNEmp", &[], TagContent::Text(note)));
    }
    if let Some(ref order) = purchase.order_number {
        children.push(tag("xPed", &[], TagContent::Text(order)));
    }
    if let Some(ref contract) = purchase.contract_number {
        children.push(tag("xCont", &[], TagContent::Text(contract)));
    }
    tag("compra", &[], TagContent::Children(children))
}

/// Build `<exporta>` element.
pub fn build_export(exp: &ExportData) -> String {
    let mut children = vec![
        tag("UFSaidaPais", &[], TagContent::Text(&exp.exit_state)),
        tag("xLocExporta", &[], TagContent::Text(&exp.export_location)),
    ];
    if let Some(ref dispatch) = exp.dispatch_location {
        children.push(tag("xLocDespacho", &[], TagContent::Text(dispatch)));
    }
    tag("exporta", &[], TagContent::Children(children))
}

/// Build `<retirada>` (withdrawal) element.
pub fn build_withdrawal(w: &LocationData) -> String {
    let tid = TaxId::new(&w.tax_id);
    let padded = tid.padded();
    let mut children = vec![tag(tid.tag_name(), &[], TagContent::Text(&padded))];
    if let Some(ref name) = w.name {
        children.push(tag("xNome", &[], TagContent::Text(name)));
    }
    children.extend(build_address_fields(
        &w.street,
        &w.number,
        w.complement.as_deref(),
        &w.district,
        &w.city_code.0,
        &w.city_name,
        &w.state_code,
        w.zip_code.as_deref(),
        false,
        None,
    ));
    tag("retirada", &[], TagContent::Children(children))
}

/// Build `<entrega>` (delivery) element.
pub fn build_delivery(d: &LocationData) -> String {
    let tid = TaxId::new(&d.tax_id);
    let padded = tid.padded();
    let mut children = vec![tag(tid.tag_name(), &[], TagContent::Text(&padded))];
    if let Some(ref name) = d.name {
        children.push(tag("xNome", &[], TagContent::Text(name)));
    }
    children.extend(build_address_fields(
        &d.street,
        &d.number,
        d.complement.as_deref(),
        &d.district,
        &d.city_code.0,
        &d.city_name,
        &d.state_code,
        d.zip_code.as_deref(),
        false,
        None,
    ));
    tag("entrega", &[], TagContent::Children(children))
}

/// Build `<autXML>` element.
pub fn build_aut_xml(entry: &AuthorizedXml) -> String {
    let tid = TaxId::new(&entry.tax_id);
    let padded = tid.padded();
    tag(
        "autXML",
        &[],
        TagContent::Children(vec![tag(tid.tag_name(), &[], TagContent::Text(&padded))]),
    )
}
