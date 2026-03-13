//! Build optional XML groups: cobr, infAdic, infIntermed, exporta, compra,
//! infRespTec, retirada, entrega, autXML.

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
            let mut proc_children = vec![
                tag("nProc", &[], TagContent::Text(&p.number)),
                tag("indProc", &[], TagContent::Text(&p.origin)),
            ];
            if let Some(tp_ato) = &p.tp_ato {
                proc_children.push(tag("tpAto", &[], TagContent::Text(tp_ato)));
            }
            children.push(tag("procRef", &[], TagContent::Children(proc_children)));
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
///
/// Ordem das tags conforme PHP sped-nfe `tagretirada()`:
/// CPF|CNPJ, xNome, xLgr, nro, xCpl, xBairro, cMun, xMun, UF, CEP,
/// cPais, xPais, fone, email, IE.
pub fn build_withdrawal(w: &LocationData) -> String {
    tag(
        "retirada",
        &[],
        TagContent::Children(build_location_children(w)),
    )
}

/// Build `<entrega>` (delivery) element.
///
/// Ordem das tags conforme PHP sped-nfe `tagentrega()`:
/// CPF|CNPJ, xNome, xLgr, nro, xCpl, xBairro, cMun, xMun, UF, CEP,
/// cPais, xPais, fone, email, IE.
pub fn build_delivery(d: &LocationData) -> String {
    tag(
        "entrega",
        &[],
        TagContent::Children(build_location_children(d)),
    )
}

/// Constrói a lista de tags-filhas comuns a `<retirada>` e `<entrega>`,
/// respeitando a ordem do schema NFe / PHP sped-nfe.
fn build_location_children(loc: &LocationData) -> Vec<String> {
    let tid = TaxId::new(&loc.tax_id);
    let padded = tid.padded();
    let mut children = vec![tag(tid.tag_name(), &[], TagContent::Text(&padded))];
    if let Some(ref name) = loc.name {
        children.push(tag("xNome", &[], TagContent::Text(name)));
    }
    children.push(tag("xLgr", &[], TagContent::Text(&loc.street)));
    children.push(tag("nro", &[], TagContent::Text(&loc.number)));
    if let Some(ref cpl) = loc.complement {
        children.push(tag("xCpl", &[], TagContent::Text(cpl)));
    }
    children.push(tag("xBairro", &[], TagContent::Text(&loc.district)));
    children.push(tag("cMun", &[], TagContent::Text(&loc.city_code.0)));
    children.push(tag("xMun", &[], TagContent::Text(&loc.city_name)));
    children.push(tag("UF", &[], TagContent::Text(&loc.state_code)));
    if let Some(ref cep) = loc.zip_code {
        children.push(tag("CEP", &[], TagContent::Text(cep)));
    }
    if let Some(ref c_pais) = loc.c_pais {
        children.push(tag("cPais", &[], TagContent::Text(c_pais)));
    }
    if let Some(ref x_pais) = loc.x_pais {
        children.push(tag("xPais", &[], TagContent::Text(x_pais)));
    }
    if let Some(ref fone) = loc.fone {
        children.push(tag("fone", &[], TagContent::Text(fone)));
    }
    if let Some(ref email) = loc.email {
        children.push(tag("email", &[], TagContent::Text(email)));
    }
    if let Some(ref ie) = loc.ie {
        children.push(tag("IE", &[], TagContent::Text(ie)));
    }
    children
}

/// Build `<cana>` (sugarcane supply) element.
///
/// XML schema order (Grupo ZC01):
/// ```xml
/// <cana>
///   <safra>...</safra>
///   <ref>...</ref>
///   <forDia dia="N"><qtde>...</qtde></forDia>  <!-- up to 31 -->
///   <qTotMes>...</qTotMes>
///   <qTotAnt>...</qTotAnt>
///   <qTotGer>...</qTotGer>
///   <deduc><xDed>...</xDed><vDed>...</vDed></deduc>  <!-- up to 10 -->
///   <vFor>...</vFor>
///   <vTotDed>...</vTotDed>
///   <vLiqFor>...</vLiqFor>
/// </cana>
/// ```
pub fn build_cana(cana: &CanaData) -> String {
    let fc2 = |c: i64| format_cents(c, 2);
    let fc10 = |c: i64| format_cents(c, 10);

    let mut children = vec![
        tag("safra", &[], TagContent::Text(&cana.safra)),
        tag("ref", &[], TagContent::Text(&cana.referencia)),
    ];

    // forDia entries (up to 31, one per day)
    for fd in cana.for_dia.iter().take(31) {
        let dia_str = fd.dia.to_string();
        children.push(tag(
            "forDia",
            &[("dia", &dia_str)],
            TagContent::Children(vec![tag("qtde", &[], TagContent::Text(&fc10(fd.qtde.0)))]),
        ));
    }

    children.push(tag(
        "qTotMes",
        &[],
        TagContent::Text(&fc10(cana.q_tot_mes.0)),
    ));
    children.push(tag(
        "qTotAnt",
        &[],
        TagContent::Text(&fc10(cana.q_tot_ant.0)),
    ));
    children.push(tag(
        "qTotGer",
        &[],
        TagContent::Text(&fc10(cana.q_tot_ger.0)),
    ));

    // deduc entries (up to 10)
    if let Some(ref deducs) = cana.deducoes {
        for d in deducs.iter().take(10) {
            children.push(tag(
                "deduc",
                &[],
                TagContent::Children(vec![
                    tag("xDed", &[], TagContent::Text(&d.x_ded)),
                    tag("vDed", &[], TagContent::Text(&fc2(d.v_ded.0))),
                ]),
            ));
        }
    }

    children.push(tag("vFor", &[], TagContent::Text(&fc2(cana.v_for.0))));
    children.push(tag(
        "vTotDed",
        &[],
        TagContent::Text(&fc2(cana.v_tot_ded.0)),
    ));
    children.push(tag(
        "vLiqFor",
        &[],
        TagContent::Text(&fc2(cana.v_liq_for.0)),
    ));

    tag("cana", &[], TagContent::Children(children))
}

/// Build `<agropecuario>` element (Grupo ZF01).
///
/// Contains either a `<guiaTransito>` or up to 20 `<defensivo>` entries.
/// Positioned inside `<infNFe>` after `<infRespTec>`.
pub fn build_agropecuario(data: &AgropecuarioData) -> String {
    let children = match data {
        AgropecuarioData::Guia(guia) => {
            let mut kids = vec![tag("tpGuia", &[], TagContent::Text(&guia.tp_guia))];
            if let Some(ref uf) = guia.uf_guia {
                kids.push(tag("UFGuia", &[], TagContent::Text(uf)));
            }
            if let Some(ref serie) = guia.serie_guia {
                kids.push(tag("serieGuia", &[], TagContent::Text(serie)));
            }
            kids.push(tag("nGuia", &[], TagContent::Text(&guia.n_guia)));
            vec![tag("guiaTransito", &[], TagContent::Children(kids))]
        }
        AgropecuarioData::Defensivos(defs) => defs
            .iter()
            .take(20)
            .map(|d| {
                tag(
                    "defensivo",
                    &[],
                    TagContent::Children(vec![
                        tag("nReceituario", &[], TagContent::Text(&d.n_receituario)),
                        tag("CPFRespTec", &[], TagContent::Text(&d.cpf_resp_tec)),
                    ]),
                )
            })
            .collect(),
    };

    tag("agropecuario", &[], TagContent::Children(children))
}

/// Build `<gCompraGov>` element (Grupo B31, PL_010+).
///
/// Placed inside `<ide>` after `<NFref>` elements.
pub fn build_compra_gov(data: &CompraGovData) -> String {
    tag(
        "gCompraGov",
        &[],
        TagContent::Children(vec![
            tag("tpEnteGov", &[], TagContent::Text(&data.tp_ente_gov)),
            tag("pRedutor", &[], TagContent::Text(&data.p_redutor)),
            tag("tpOperGov", &[], TagContent::Text(&data.tp_oper_gov)),
        ]),
    )
}

/// Build `<gPagAntecipado>` element (Grupo B34, PL_010+).
///
/// Placed inside `<ide>` after `<gCompraGov>`.
pub fn build_pag_antecipado(data: &PagAntecipadoData) -> String {
    let children: Vec<String> = data
        .ref_nfe
        .iter()
        .map(|key| tag("refNFe", &[], TagContent::Text(key)))
        .collect();
    tag("gPagAntecipado", &[], TagContent::Children(children))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::newtypes::Cents;

    #[test]
    fn build_cana_minimal_without_deducoes() {
        let cana = CanaData::new(
            "2025/2026",
            "03/2026",
            vec![
                ForDiaData::new(1, Cents(1000000)),
                ForDiaData::new(2, Cents(1500000)),
            ],
            Cents(2500000), // qTotMes
            Cents(5000000), // qTotAnt
            Cents(7500000), // qTotGer
            Cents(150000),  // vFor = 1500.00
            Cents(0),       // vTotDed = 0.00
            Cents(150000),  // vLiqFor = 1500.00
        );

        let xml = build_cana(&cana);

        assert_eq!(
            xml,
            "<cana>\
                <safra>2025/2026</safra>\
                <ref>03/2026</ref>\
                <forDia dia=\"1\"><qtde>10000.0000000000</qtde></forDia>\
                <forDia dia=\"2\"><qtde>15000.0000000000</qtde></forDia>\
                <qTotMes>25000.0000000000</qTotMes>\
                <qTotAnt>50000.0000000000</qTotAnt>\
                <qTotGer>75000.0000000000</qTotGer>\
                <vFor>1500.00</vFor>\
                <vTotDed>0.00</vTotDed>\
                <vLiqFor>1500.00</vLiqFor>\
            </cana>"
        );
    }

    #[test]
    fn build_cana_with_deducoes() {
        let cana = CanaData::new(
            "2024/2025",
            "06/2025",
            vec![ForDiaData::new(15, Cents(2000000))],
            Cents(2000000),  // qTotMes
            Cents(10000000), // qTotAnt
            Cents(12000000), // qTotGer
            Cents(500000),   // vFor = 5000.00
            Cents(50000),    // vTotDed = 500.00
            Cents(450000),   // vLiqFor = 4500.00
        )
        .deducoes(vec![
            DeducData::new("TAXA PRODUCAO", Cents(30000)),
            DeducData::new("FUNRURAL", Cents(20000)),
        ]);

        let xml = build_cana(&cana);

        assert!(xml.contains("<safra>2024/2025</safra>"));
        assert!(xml.contains("<ref>06/2025</ref>"));
        assert!(xml.contains("<forDia dia=\"15\"><qtde>20000.0000000000</qtde></forDia>"));
        assert!(xml.contains("<qTotMes>20000.0000000000</qTotMes>"));
        assert!(xml.contains("<qTotAnt>100000.0000000000</qTotAnt>"));
        assert!(xml.contains("<qTotGer>120000.0000000000</qTotGer>"));
        assert!(xml.contains("<deduc><xDed>TAXA PRODUCAO</xDed><vDed>300.00</vDed></deduc>"));
        assert!(xml.contains("<deduc><xDed>FUNRURAL</xDed><vDed>200.00</vDed></deduc>"));
        assert!(xml.contains("<vFor>5000.00</vFor>"));
        assert!(xml.contains("<vTotDed>500.00</vTotDed>"));
        assert!(xml.contains("<vLiqFor>4500.00</vLiqFor>"));

        // Verify order: deduc comes before vFor
        let deduc_pos = xml.find("<deduc>").expect("deduc must be present");
        let vfor_pos = xml.find("<vFor>").expect("vFor must be present");
        assert!(
            deduc_pos < vfor_pos,
            "deduc must come before vFor in the XML"
        );

        // Verify order: forDia comes before qTotMes
        let fordia_pos = xml.find("<forDia").expect("forDia must be present");
        let qtotmes_pos = xml.find("<qTotMes>").expect("qTotMes must be present");
        assert!(
            fordia_pos < qtotmes_pos,
            "forDia must come before qTotMes in the XML"
        );
    }

    #[test]
    fn build_cana_limits_fordia_to_31() {
        let mut for_dia = Vec::new();
        for day in 1..=35 {
            for_dia.push(ForDiaData::new(day, Cents(100000)));
        }

        let cana = CanaData::new(
            "2025/2026",
            "01/2026",
            for_dia,
            Cents(0),
            Cents(0),
            Cents(0),
            Cents(0),
            Cents(0),
            Cents(0),
        );

        let xml = build_cana(&cana);

        // Count forDia occurrences — should be capped at 31
        let count = xml.matches("<forDia").count();
        assert_eq!(count, 31, "forDia entries must be capped at 31");
    }

    #[test]
    fn build_cana_limits_deduc_to_10() {
        let mut deducs = Vec::new();
        for i in 1..=15 {
            deducs.push(DeducData::new(format!("DEDUC {i}"), Cents(1000)));
        }

        let cana = CanaData::new(
            "2025/2026",
            "01/2026",
            vec![ForDiaData::new(1, Cents(100000))],
            Cents(0),
            Cents(0),
            Cents(0),
            Cents(0),
            Cents(0),
            Cents(0),
        )
        .deducoes(deducs);

        let xml = build_cana(&cana);

        // Count deduc occurrences — should be capped at 10
        let count = xml.matches("<deduc>").count();
        assert_eq!(count, 10, "deduc entries must be capped at 10");
    }

    // ── Agropecuário tests ──────────────────────────────────────────────

    #[test]
    fn build_agropecuario_guia() {
        let guia = AgropecuarioGuiaData::new("1", "12345")
            .uf_guia("SP")
            .serie_guia("A");
        let data = AgropecuarioData::Guia(guia);
        let xml = build_agropecuario(&data);

        assert_eq!(
            xml,
            "<agropecuario>\
                <guiaTransito>\
                    <tpGuia>1</tpGuia>\
                    <UFGuia>SP</UFGuia>\
                    <serieGuia>A</serieGuia>\
                    <nGuia>12345</nGuia>\
                </guiaTransito>\
            </agropecuario>"
        );
    }

    #[test]
    fn build_agropecuario_guia_minimal() {
        let guia = AgropecuarioGuiaData::new("2", "99999");
        let data = AgropecuarioData::Guia(guia);
        let xml = build_agropecuario(&data);

        assert_eq!(
            xml,
            "<agropecuario>\
                <guiaTransito>\
                    <tpGuia>2</tpGuia>\
                    <nGuia>99999</nGuia>\
                </guiaTransito>\
            </agropecuario>"
        );
    }

    #[test]
    fn build_agropecuario_defensivos() {
        let defs = vec![
            AgropecuarioDefensivoData::new("REC001", "12345678901"),
            AgropecuarioDefensivoData::new("REC002", "98765432109"),
        ];
        let data = AgropecuarioData::Defensivos(defs);
        let xml = build_agropecuario(&data);

        assert_eq!(
            xml,
            "<agropecuario>\
                <defensivo>\
                    <nReceituario>REC001</nReceituario>\
                    <CPFRespTec>12345678901</CPFRespTec>\
                </defensivo>\
                <defensivo>\
                    <nReceituario>REC002</nReceituario>\
                    <CPFRespTec>98765432109</CPFRespTec>\
                </defensivo>\
            </agropecuario>"
        );
    }

    #[test]
    fn build_agropecuario_defensivos_capped_at_20() {
        let defs: Vec<AgropecuarioDefensivoData> = (0..25)
            .map(|i| AgropecuarioDefensivoData::new(format!("REC{i:03}"), "12345678901"))
            .collect();
        let data = AgropecuarioData::Defensivos(defs);
        let xml = build_agropecuario(&data);

        let count = xml.matches("<defensivo>").count();
        assert_eq!(count, 20, "defensivo entries must be capped at 20");
    }

    // ── Compra Governamental tests ──────────────────────────────────────

    #[test]
    fn build_compra_gov_all_fields() {
        let cg = CompraGovData::new("1", "10.5000", "2");
        let xml = build_compra_gov(&cg);

        assert_eq!(
            xml,
            "<gCompraGov>\
                <tpEnteGov>1</tpEnteGov>\
                <pRedutor>10.5000</pRedutor>\
                <tpOperGov>2</tpOperGov>\
            </gCompraGov>"
        );
    }

    // ── Pagamento Antecipado tests ──────────────────────────────────────

    #[test]
    fn build_pag_antecipado_single_ref() {
        let pa = PagAntecipadoData::new(vec![
            "41260304123456000190550010000001231123456780".to_string(),
        ]);
        let xml = build_pag_antecipado(&pa);

        assert_eq!(
            xml,
            "<gPagAntecipado>\
                <refNFe>41260304123456000190550010000001231123456780</refNFe>\
            </gPagAntecipado>"
        );
    }

    #[test]
    fn build_pag_antecipado_multiple_refs() {
        let pa = PagAntecipadoData::new(vec![
            "41260304123456000190550010000001231123456780".to_string(),
            "41260304123456000190550010000001241123456781".to_string(),
        ]);
        let xml = build_pag_antecipado(&pa);

        assert!(xml.contains("<gPagAntecipado>"));
        assert_eq!(xml.matches("<refNFe>").count(), 2);
    }

    // ── Retirada / Entrega tests ──────────────────────────────────────

    #[test]
    fn build_withdrawal_all_fields_cnpj() {
        let loc = LocationData::new(
            "12345678000199",
            "Rua das Flores",
            "100",
            "Centro",
            IbgeCode("3550308".to_string()),
            "São Paulo",
            "SP",
        )
        .name("Empresa Teste")
        .complement("Sala 5")
        .zip_code("01001000")
        .c_pais("1058")
        .x_pais("Brasil")
        .fone("1155551234")
        .email("teste@empresa.com")
        .ie("123456789");

        let xml = build_withdrawal(&loc);

        assert_eq!(
            xml,
            "<retirada>\
                <CNPJ>12345678000199</CNPJ>\
                <xNome>Empresa Teste</xNome>\
                <xLgr>Rua das Flores</xLgr>\
                <nro>100</nro>\
                <xCpl>Sala 5</xCpl>\
                <xBairro>Centro</xBairro>\
                <cMun>3550308</cMun>\
                <xMun>São Paulo</xMun>\
                <UF>SP</UF>\
                <CEP>01001000</CEP>\
                <cPais>1058</cPais>\
                <xPais>Brasil</xPais>\
                <fone>1155551234</fone>\
                <email>teste@empresa.com</email>\
                <IE>123456789</IE>\
            </retirada>"
        );
    }

    #[test]
    fn build_delivery_all_fields_cpf() {
        let loc = LocationData::new(
            "12345678901",
            "Av. Brasil",
            "200",
            "Bela Vista",
            IbgeCode("3304557".to_string()),
            "Rio de Janeiro",
            "RJ",
        )
        .name("João da Silva")
        .complement("Apto 301")
        .zip_code("20040020")
        .c_pais("1058")
        .x_pais("Brasil")
        .fone("2199998888")
        .email("joao@email.com")
        .ie("ISENTO");

        let xml = build_delivery(&loc);

        assert_eq!(
            xml,
            "<entrega>\
                <CPF>12345678901</CPF>\
                <xNome>João da Silva</xNome>\
                <xLgr>Av. Brasil</xLgr>\
                <nro>200</nro>\
                <xCpl>Apto 301</xCpl>\
                <xBairro>Bela Vista</xBairro>\
                <cMun>3304557</cMun>\
                <xMun>Rio de Janeiro</xMun>\
                <UF>RJ</UF>\
                <CEP>20040020</CEP>\
                <cPais>1058</cPais>\
                <xPais>Brasil</xPais>\
                <fone>2199998888</fone>\
                <email>joao@email.com</email>\
                <IE>ISENTO</IE>\
            </entrega>"
        );
    }

    #[test]
    fn build_withdrawal_minimal_only_required() {
        let loc = LocationData::new(
            "12345678000199",
            "Rua A",
            "1",
            "Centro",
            IbgeCode("3550308".to_string()),
            "São Paulo",
            "SP",
        );

        let xml = build_withdrawal(&loc);

        assert_eq!(
            xml,
            "<retirada>\
                <CNPJ>12345678000199</CNPJ>\
                <xLgr>Rua A</xLgr>\
                <nro>1</nro>\
                <xBairro>Centro</xBairro>\
                <cMun>3550308</cMun>\
                <xMun>São Paulo</xMun>\
                <UF>SP</UF>\
            </retirada>"
        );
    }

    #[test]
    fn build_delivery_partial_new_fields() {
        let loc = LocationData::new(
            "98765432000111",
            "Rua B",
            "50",
            "Jardim",
            IbgeCode("4106902".to_string()),
            "Curitiba",
            "PR",
        )
        .fone("4133334444")
        .email("contato@loja.com");

        let xml = build_delivery(&loc);

        // Verifica que fone e email aparecem mas cPais, xPais e IE não
        assert_eq!(
            xml,
            "<entrega>\
                <CNPJ>98765432000111</CNPJ>\
                <xLgr>Rua B</xLgr>\
                <nro>50</nro>\
                <xBairro>Jardim</xBairro>\
                <cMun>4106902</cMun>\
                <xMun>Curitiba</xMun>\
                <UF>PR</UF>\
                <fone>4133334444</fone>\
                <email>contato@loja.com</email>\
            </entrega>"
        );
    }

    #[test]
    fn build_withdrawal_tag_order_matches_php() {
        // Verifica que a ordem das tags confere com o PHP sped-nfe
        let loc = LocationData::new(
            "12345678000199",
            "Rua X",
            "10",
            "Bairro Y",
            IbgeCode("1100015".to_string()),
            "Porto Velho",
            "RO",
        )
        .name("Empresa")
        .complement("Loja 1")
        .zip_code("76801000")
        .c_pais("1058")
        .x_pais("Brasil")
        .fone("6932221111")
        .email("a@b.com")
        .ie("1234");

        let xml = build_withdrawal(&loc);

        // Ordem PHP: CNPJ, xNome, xLgr, nro, xCpl, xBairro, cMun, xMun, UF, CEP, cPais, xPais, fone, email, IE
        let tags_in_order = [
            "<CNPJ>",
            "<xNome>",
            "<xLgr>",
            "<nro>",
            "<xCpl>",
            "<xBairro>",
            "<cMun>",
            "<xMun>",
            "<UF>",
            "<CEP>",
            "<cPais>",
            "<xPais>",
            "<fone>",
            "<email>",
            "<IE>",
        ];
        let mut last_pos = 0;
        for tag_name in &tags_in_order {
            let pos = xml
                .find(tag_name)
                .unwrap_or_else(|| panic!("tag {tag_name} não encontrada no XML"));
            assert!(
                pos >= last_pos,
                "tag {tag_name} está fora de ordem (pos {pos} < {last_pos})"
            );
            last_pos = pos;
        }
    }
}
