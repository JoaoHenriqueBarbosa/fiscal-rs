//! String-based XML builder for the MDF-e (model 58), leiaute 3.00.
//!
//! [`build_mdfe_xml`] assembles a complete `<MDFe>` document — for any of the
//! four transport modals (road, air, waterway, rail) — in the exact block
//! order required by the XSD:
//!
//! ```text
//! MDFe > infMDFe[@Id] > ide, emit, infModal, infDoc, tot, infAdic?
//! ```
//!
//! The reusable XML primitives (`tag`, `TagContent`) come from
//! [`fiscal_core::xml_utils`].

use fiscal_core::FiscalError;
use fiscal_core::xml_utils::{TagContent, tag};

use crate::access_key::build_mdfe_access_key_from_ide;
use crate::types::*;
use crate::{MDFE_MODEL, MDFE_NAMESPACE, MDFE_VERSION};

/// Build a complete `<MDFe>` XML document from `MdfeBuildData`.
///
/// Emits the modal block carried by [`MdfeBuildData::modal`] (road, air,
/// waterway, or rail). Generates the 44-digit access key, derives `cMDF`/`cDV`
/// from it, and emits every block in schema order. The returned XML is
/// **unsigned**; signing and the `<MDFeProc>` envelope belong to later phases.
///
/// # Errors
///
/// - [`FiscalError::XmlGeneration`] if the access key cannot be built.
pub fn build_mdfe_xml(data: &MdfeBuildData) -> Result<String, FiscalError> {
    let access_key =
        build_mdfe_access_key_from_ide(&data.ide, &data.emit.cnpj, data.numeric_code.as_deref())?;

    let c_mdf = &access_key.numeric_code;
    let c_dv = &access_key.key[43..44];

    let inf_mdfe_children = vec![
        build_ide(&data.ide, c_mdf, c_dv),
        build_emit(&data.emit),
        build_inf_modal(&data.modal)?,
        build_inf_doc(&data.inf_doc),
        build_tot(&data.tot),
    ]
    .into_iter()
    .chain(data.inf_adic.as_ref().map(build_inf_adic))
    .collect::<Vec<_>>();

    let id_attr = format!("MDFe{}", access_key.key);
    let inf_mdfe = tag(
        "infMDFe",
        &[("versao", MDFE_VERSION), ("Id", &id_attr)],
        TagContent::Children(inf_mdfe_children),
    );

    let mdfe = tag(
        "MDFe",
        &[("xmlns", MDFE_NAMESPACE)],
        TagContent::Children(vec![inf_mdfe]),
    );

    Ok(mdfe)
}

/// Format a datetime as MDF-e ISO 8601 with the Brazil offset for `uf` (no UTC `Z`).
///
/// Converts the instant to the UF's fixed offset **before** formatting, so that
/// the wall-clock time in the output matches the target timezone. Discarding the
/// offset without converting would silently shift the represented instant.
fn format_datetime_mdfe(dt: &chrono::DateTime<chrono::FixedOffset>, uf: &str) -> String {
    let secs = match uf {
        "AC" => 5 * 3600,
        "AM" | "RO" | "RR" | "MT" | "MS" => 4 * 3600,
        _ => 3 * 3600,
    };
    let offset = chrono::FixedOffset::west_opt(secs).expect("static offset is valid");
    let local = dt.with_timezone(&offset);
    local.format("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

/// Choose `CNPJ` or `CPF` tag by tax-id length (14 → CNPJ, otherwise CPF).
fn tax_id_tag(tax_id: &str) -> &'static str {
    if tax_id.chars().filter(|c| c.is_ascii_digit()).count() == 14 {
        "CNPJ"
    } else {
        "CPF"
    }
}

// ── ide ──────────────────────────────────────────────────────────────────────

fn build_ide(ide: &Ide, c_mdf: &str, c_dv: &str) -> String {
    let serie = ide.serie.to_string();
    let n_mdf = ide.n_mdf.to_string();
    let dh_emi = format_datetime_mdfe(&ide.dh_emi, &ide.uf_ini);

    let mut children = vec![
        tag("cUF", &[], TagContent::Text(&ide.c_uf)),
        tag("tpAmb", &[], TagContent::Text(&ide.tp_amb)),
        tag("tpEmit", &[], TagContent::Text(&ide.tp_emit)),
        tag("mod", &[], TagContent::Text(MDFE_MODEL)),
        tag("serie", &[], TagContent::Text(&serie)),
        tag("nMDF", &[], TagContent::Text(&n_mdf)),
        tag("cMDF", &[], TagContent::Text(c_mdf)),
        tag("cDV", &[], TagContent::Text(c_dv)),
        tag("modal", &[], TagContent::Text(&ide.modal)),
        tag("dhEmi", &[], TagContent::Text(&dh_emi)),
        tag("tpEmis", &[], TagContent::Text(&ide.tp_emis)),
        tag(
            "procEmi",
            &[],
            TagContent::Text(ide.proc_emi.as_deref().unwrap_or("0")),
        ),
        tag(
            "verProc",
            &[],
            TagContent::Text(ide.ver_proc.as_deref().unwrap_or("fiscal-mdfe 0.1.0")),
        ),
        tag("UFIni", &[], TagContent::Text(&ide.uf_ini)),
        tag("UFFim", &[], TagContent::Text(&ide.uf_fim)),
    ];

    children.extend(ide.inf_mun_carrega.iter().map(|m| {
        tag(
            "infMunCarrega",
            &[],
            TagContent::Children(vec![
                tag("cMunCarrega", &[], TagContent::Text(&m.c_mun)),
                tag("xMunCarrega", &[], TagContent::Text(&m.x_mun)),
            ]),
        )
    }));

    children.extend(ide.inf_percurso.iter().map(|uf| {
        tag(
            "infPercurso",
            &[],
            TagContent::Children(vec![tag("UFPer", &[], TagContent::Text(uf))]),
        )
    }));

    if let Some(dt) = ide.dh_ini_viagem {
        let v = format_datetime_mdfe(&dt, &ide.uf_ini);
        children.push(tag("dhIniViagem", &[], TagContent::Text(&v)));
    }

    tag("ide", &[], TagContent::Children(children))
}

// ── emit ─────────────────────────────────────────────────────────────────────

fn build_emit(emit: &Emit) -> String {
    let mut children = vec![tag("CNPJ", &[], TagContent::Text(&emit.cnpj))];
    if let Some(ie) = &emit.ie {
        children.push(tag("IE", &[], TagContent::Text(ie)));
    }
    children.push(tag("xNome", &[], TagContent::Text(&emit.x_nome)));
    if let Some(xf) = &emit.x_fant {
        children.push(tag("xFant", &[], TagContent::Text(xf)));
    }
    children.push(build_ender_emit(&emit.ender_emit));

    tag("emit", &[], TagContent::Children(children))
}

fn build_ender_emit(e: &EnderEmit) -> String {
    let mut children = vec![
        tag("xLgr", &[], TagContent::Text(&e.x_lgr)),
        tag("nro", &[], TagContent::Text(&e.nro)),
    ];
    if let Some(c) = &e.x_cpl {
        children.push(tag("xCpl", &[], TagContent::Text(c)));
    }
    children.extend([
        tag("xBairro", &[], TagContent::Text(&e.x_bairro)),
        tag("cMun", &[], TagContent::Text(&e.c_mun)),
        tag("xMun", &[], TagContent::Text(&e.x_mun)),
        tag("CEP", &[], TagContent::Text(&e.cep)),
        tag("UF", &[], TagContent::Text(&e.uf)),
    ]);
    if let Some(f) = &e.fone {
        children.push(tag("fone", &[], TagContent::Text(f)));
    }
    if let Some(em) = &e.email {
        children.push(tag("email", &[], TagContent::Text(em)));
    }
    tag("enderEmit", &[], TagContent::Children(children))
}

// ── infModal ─────────────────────────────────────────────────────────────────

fn build_inf_modal(modal: &Modal) -> Result<String, FiscalError> {
    let modal_xml = match modal {
        Modal::Rodo(r) => build_rodo(r),
        Modal::Aereo(a) => build_aereo(a),
        Modal::Aquav(a) => build_aquav(a),
        Modal::Ferrov(f) => build_ferrov(f),
    };

    Ok(tag(
        "infModal",
        &[("versaoModal", MDFE_VERSION)],
        TagContent::Children(vec![modal_xml]),
    ))
}

fn build_aereo(a: &Aereo) -> String {
    tag(
        "aereo",
        &[],
        TagContent::Children(vec![
            tag("nac", &[], TagContent::Text(&a.nac)),
            tag("matr", &[], TagContent::Text(&a.matr)),
            tag("nVoo", &[], TagContent::Text(&a.n_voo)),
            tag("cAerEmb", &[], TagContent::Text(&a.c_aer_emb)),
            tag("cAerDes", &[], TagContent::Text(&a.c_aer_des)),
            tag("dVoo", &[], TagContent::Text(&a.d_voo)),
        ]),
    )
}

fn build_aquav(a: &Aquav) -> String {
    let mut children = vec![
        tag("irin", &[], TagContent::Text(&a.irin)),
        tag("tpEmb", &[], TagContent::Text(&a.tp_emb)),
        tag("cEmbar", &[], TagContent::Text(&a.c_embar)),
        tag("xEmbar", &[], TagContent::Text(&a.x_embar)),
        tag("nViag", &[], TagContent::Text(&a.n_viag)),
        tag("cPrtEmb", &[], TagContent::Text(&a.c_prt_emb)),
        tag("cPrtDest", &[], TagContent::Text(&a.c_prt_dest)),
    ];
    if let Some(p) = &a.prt_trans {
        children.push(tag("prtTrans", &[], TagContent::Text(p)));
    }
    if let Some(t) = &a.tp_nav {
        children.push(tag("tpNav", &[], TagContent::Text(t)));
    }
    children.extend(a.inf_term_carreg.iter().map(|t| {
        tag(
            "infTermCarreg",
            &[],
            TagContent::Children(vec![
                tag("cTermCarreg", &[], TagContent::Text(&t.c_term_carreg)),
                tag("xTermCarreg", &[], TagContent::Text(&t.x_term_carreg)),
            ]),
        )
    }));
    children.extend(a.inf_term_descarreg.iter().map(|t| {
        tag(
            "infTermDescarreg",
            &[],
            TagContent::Children(vec![
                tag("cTermDescarreg", &[], TagContent::Text(&t.c_term_descarreg)),
                tag("xTermDescarreg", &[], TagContent::Text(&t.x_term_descarreg)),
            ]),
        )
    }));
    children.extend(a.inf_emb_comb.iter().map(|e| {
        tag(
            "infEmbComb",
            &[],
            TagContent::Children(vec![
                tag("cEmbComb", &[], TagContent::Text(&e.c_emb_comb)),
                tag("xBalsa", &[], TagContent::Text(&e.x_balsa)),
            ]),
        )
    }));
    children.extend(a.inf_unid_carga_vazia.iter().map(|u| {
        tag(
            "infUnidCargaVazia",
            &[],
            TagContent::Children(vec![
                tag(
                    "idUnidCargaVazia",
                    &[],
                    TagContent::Text(&u.id_unid_carga_vazia),
                ),
                tag(
                    "tpUnidCargaVazia",
                    &[],
                    TagContent::Text(&u.tp_unid_carga_vazia),
                ),
            ]),
        )
    }));
    children.extend(a.inf_unid_transp_vazia.iter().map(|u| {
        tag(
            "infUnidTranspVazia",
            &[],
            TagContent::Children(vec![
                tag(
                    "idUnidTranspVazia",
                    &[],
                    TagContent::Text(&u.id_unid_transp_vazia),
                ),
                tag(
                    "tpUnidTranspVazia",
                    &[],
                    TagContent::Text(&u.tp_unid_transp_vazia),
                ),
            ]),
        )
    }));
    if let Some(m) = &a.mmsi {
        children.push(tag("MMSI", &[], TagContent::Text(m)));
    }

    tag("aquav", &[], TagContent::Children(children))
}

fn build_ferrov(f: &Ferrov) -> String {
    let mut trem_children = vec![tag("xPref", &[], TagContent::Text(&f.trem.x_pref))];
    if let Some(dh) = &f.trem.dh_trem {
        trem_children.push(tag("dhTrem", &[], TagContent::Text(dh)));
    }
    trem_children.extend([
        tag("xOri", &[], TagContent::Text(&f.trem.x_ori)),
        tag("xDest", &[], TagContent::Text(&f.trem.x_dest)),
        tag("qVag", &[], TagContent::Text(&f.trem.q_vag)),
    ]);

    let mut children = vec![tag("trem", &[], TagContent::Children(trem_children))];
    children.extend(f.vag.iter().map(|v| {
        let mut vag_children = vec![
            tag("pesoBC", &[], TagContent::Text(&v.peso_bc)),
            tag("pesoR", &[], TagContent::Text(&v.peso_r)),
        ];
        if let Some(t) = &v.tp_vag {
            vag_children.push(tag("tpVag", &[], TagContent::Text(t)));
        }
        vag_children.extend([
            tag("serie", &[], TagContent::Text(&v.serie)),
            tag("nVag", &[], TagContent::Text(&v.n_vag)),
        ]);
        if let Some(s) = &v.n_seq {
            vag_children.push(tag("nSeq", &[], TagContent::Text(s)));
        }
        vag_children.push(tag("TU", &[], TagContent::Text(&v.tu)));
        tag("vag", &[], TagContent::Children(vag_children))
    }));

    tag("ferrov", &[], TagContent::Children(children))
}

fn build_rodo(rodo: &Rodo) -> String {
    let mut children = Vec::new();
    if let Some(antt) = &rodo.inf_antt {
        children.push(build_inf_antt(antt));
    }
    children.push(build_veic_tracao(&rodo.veic_tracao));
    children.extend(rodo.veic_reboque.iter().map(build_veic_reboque));

    tag("rodo", &[], TagContent::Children(children))
}

fn build_inf_antt(antt: &InfAntt) -> String {
    let mut children = Vec::new();
    if let Some(rntrc) = &antt.rntrc {
        children.push(tag("RNTRC", &[], TagContent::Text(rntrc)));
    }
    children.extend(antt.inf_ciot.iter().map(|c| {
        tag(
            "infCIOT",
            &[],
            TagContent::Children(vec![
                tag("CIOT", &[], TagContent::Text(&c.ciot)),
                tag(tax_id_tag(&c.tax_id), &[], TagContent::Text(&c.tax_id)),
            ]),
        )
    }));
    children.extend(antt.vale_ped.iter().map(|vp| {
        let mut vc = vec![
            tag("CNPJForn", &[], TagContent::Text(&vp.cnpj_forn)),
            tag("nCompra", &[], TagContent::Text(&vp.n_compra)),
        ];
        if let Some(v) = &vp.v_vale_ped {
            vc.push(tag("vValePed", &[], TagContent::Text(v)));
        }
        vc.push(tag("tpValePed", &[], TagContent::Text(&vp.tp_vale_ped)));
        vc.push(tag(
            "categCombVeic",
            &[],
            TagContent::Text(&vp.categ_comb_veic),
        ));
        tag("valePed", &[], TagContent::Children(vc))
    }));
    tag("infANTT", &[], TagContent::Children(children))
}

fn build_prop(p: &Prop) -> String {
    let mut children = vec![tag(tax_id_tag(&p.tax_id), &[], TagContent::Text(&p.tax_id))];
    if let Some(r) = &p.rntrc {
        children.push(tag("RNTRC", &[], TagContent::Text(r)));
    }
    children.push(tag("xNome", &[], TagContent::Text(&p.x_nome)));
    if let Some(ie) = &p.ie {
        children.push(tag("IE", &[], TagContent::Text(ie)));
    }
    children.extend([
        tag("UF", &[], TagContent::Text(&p.uf)),
        tag("tpProp", &[], TagContent::Text(&p.tp_prop)),
    ]);
    tag("prop", &[], TagContent::Children(children))
}

fn build_veic_tracao(v: &VeicTracao) -> String {
    let tara = v.tara.to_string();
    let mut children = Vec::new();
    if let Some(ci) = &v.c_int {
        children.push(tag("cInt", &[], TagContent::Text(ci)));
    }
    children.push(tag("placa", &[], TagContent::Text(&v.placa)));
    if let Some(r) = &v.renavam {
        children.push(tag("RENAVAM", &[], TagContent::Text(r)));
    }
    children.push(tag("tara", &[], TagContent::Text(&tara)));
    if let Some(c) = v.cap_kg {
        children.push(tag("capKG", &[], TagContent::Text(&c.to_string())));
    }
    if let Some(c) = v.cap_m3 {
        children.push(tag("capM3", &[], TagContent::Text(&c.to_string())));
    }
    if let Some(prop) = &v.prop {
        children.push(build_prop(prop));
    }
    children.extend(v.condutor.iter().map(|c| {
        tag(
            "condutor",
            &[],
            TagContent::Children(vec![
                tag("xNome", &[], TagContent::Text(&c.x_nome)),
                tag("CPF", &[], TagContent::Text(&c.cpf)),
            ]),
        )
    }));
    children.push(tag("tpRod", &[], TagContent::Text(&v.tp_rod)));
    children.push(tag("tpCar", &[], TagContent::Text(&v.tp_car)));
    if let Some(uf) = &v.uf {
        children.push(tag("UF", &[], TagContent::Text(uf)));
    }
    tag("veicTracao", &[], TagContent::Children(children))
}

fn build_veic_reboque(v: &VeicReboque) -> String {
    let tara = v.tara.to_string();
    let mut children = Vec::new();
    if let Some(ci) = &v.c_int {
        children.push(tag("cInt", &[], TagContent::Text(ci)));
    }
    children.push(tag("placa", &[], TagContent::Text(&v.placa)));
    if let Some(r) = &v.renavam {
        children.push(tag("RENAVAM", &[], TagContent::Text(r)));
    }
    children.push(tag("tara", &[], TagContent::Text(&tara)));
    if let Some(c) = v.cap_kg {
        children.push(tag("capKG", &[], TagContent::Text(&c.to_string())));
    }
    if let Some(c) = v.cap_m3 {
        children.push(tag("capM3", &[], TagContent::Text(&c.to_string())));
    }
    children.push(tag("tpCar", &[], TagContent::Text(&v.tp_car)));
    if let Some(prop) = &v.prop {
        children.push(build_prop(prop));
    }
    if let Some(uf) = &v.uf {
        children.push(tag("UF", &[], TagContent::Text(uf)));
    }
    tag("veicReboque", &[], TagContent::Children(children))
}

// ── infDoc ───────────────────────────────────────────────────────────────────

fn build_inf_doc(doc: &InfDoc) -> String {
    let children: Vec<String> = doc
        .inf_mun_descarga
        .iter()
        .map(|m| {
            let mut kids = vec![
                tag("cMunDescarga", &[], TagContent::Text(&m.c_mun)),
                tag("xMunDescarga", &[], TagContent::Text(&m.x_mun)),
            ];
            kids.extend(m.inf_nfe.iter().map(|ch| {
                tag(
                    "infNFe",
                    &[],
                    TagContent::Children(vec![tag("chNFe", &[], TagContent::Text(ch))]),
                )
            }));
            kids.extend(m.inf_cte.iter().map(|ch| {
                tag(
                    "infCTe",
                    &[],
                    TagContent::Children(vec![tag("chCTe", &[], TagContent::Text(ch))]),
                )
            }));
            kids.extend(m.inf_mdfe.iter().map(|ch| {
                tag(
                    "infMDFeTransp",
                    &[],
                    TagContent::Children(vec![tag("chMDFe", &[], TagContent::Text(ch))]),
                )
            }));
            tag("infMunDescarga", &[], TagContent::Children(kids))
        })
        .collect();

    tag("infDoc", &[], TagContent::Children(children))
}

// ── tot ──────────────────────────────────────────────────────────────────────

fn build_tot(tot: &Tot) -> String {
    let mut children = Vec::new();
    if let Some(q) = tot.q_cte {
        children.push(tag("qCTe", &[], TagContent::Text(&q.to_string())));
    }
    if let Some(q) = tot.q_nfe {
        children.push(tag("qNFe", &[], TagContent::Text(&q.to_string())));
    }
    if let Some(q) = tot.q_mdfe {
        children.push(tag("qMDFe", &[], TagContent::Text(&q.to_string())));
    }
    children.push(tag(
        "vCarga",
        &[],
        TagContent::Text(&format!("{:.2}", tot.v_carga)),
    ));
    children.push(tag("cUnid", &[], TagContent::Text(&tot.c_unid)));
    children.push(tag(
        "qCarga",
        &[],
        TagContent::Text(&format!("{:.4}", tot.q_carga)),
    ));
    tag("tot", &[], TagContent::Children(children))
}

// ── infAdic ──────────────────────────────────────────────────────────────────

fn build_inf_adic(adic: &InfAdic) -> String {
    let mut children = Vec::new();
    if let Some(f) = &adic.inf_ad_fisco {
        children.push(tag("infAdFisco", &[], TagContent::Text(f)));
    }
    if let Some(c) = &adic.inf_cpl {
        children.push(tag("infCpl", &[], TagContent::Text(c)));
    }
    tag("infAdic", &[], TagContent::Children(children))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, TimeZone};

    #[test]
    fn format_datetime_preserves_instant_across_uf_offsets() {
        // 2026-06-04T09:30:00-03:00 passed with UF="AM" (-04:00):
        // should output 08:30:00-04:00 (same instant, different wall-clock), NOT 09:30:00-04:00.
        let brt = FixedOffset::west_opt(3 * 3600).unwrap();
        let dt = brt.with_ymd_and_hms(2026, 6, 4, 9, 30, 0).unwrap();

        let out = format_datetime_mdfe(&dt, "AM");
        assert_eq!(
            out, "2026-06-04T08:30:00-04:00",
            "instant must be preserved when converting to AM offset"
        );
    }

    #[test]
    fn format_datetime_same_zone_unchanged() {
        // UF in BRT zone: wall-clock unchanged.
        let brt = FixedOffset::west_opt(3 * 3600).unwrap();
        let dt = brt.with_ymd_and_hms(2026, 6, 4, 12, 0, 0).unwrap();

        let out = format_datetime_mdfe(&dt, "SP");
        assert_eq!(out, "2026-06-04T12:00:00-03:00");
    }

    #[test]
    fn format_datetime_ac_offset() {
        let brt = FixedOffset::west_opt(3 * 3600).unwrap();
        let dt = brt.with_ymd_and_hms(2026, 6, 4, 10, 0, 0).unwrap();

        let out = format_datetime_mdfe(&dt, "AC");
        assert_eq!(out, "2026-06-04T08:00:00-05:00");
    }
}
