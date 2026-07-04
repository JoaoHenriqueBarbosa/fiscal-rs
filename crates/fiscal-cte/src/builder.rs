//! String-based XML builder for the CT-e (model 57), leiaute 4.00.
//!
//! [`build_cte_xml`] assembles a complete `<CTe>` document for the **road**
//! modal in the exact block order required by the XSD:
//!
//! ```text
//! CTe > infCte[@Id] >
//!   ide, compl?, emit, rem?, exped?, receb?, dest?, vPrest, imp,
//!   infCTeNorm{ infCarga, infDoc?, infModal{rodo} }, autXML*, infRespTec?
//! ```
//!
//! The returned XML is **unsigned** and carries no `<infCTeSupl>` (qrCodCTe) —
//! the QR-code supplement and the signature are inserted by the hub before
//! transmission, mirroring the MDF-e flow.

use fiscal_core::FiscalError;
use fiscal_core::xml_utils::{TagContent, tag};

use crate::access_key::build_cte_access_key_from_ide;
use crate::types::*;
use crate::{CTE_MODEL, CTE_NAMESPACE, CTE_VERSION};

/// Build a complete unsigned `<CTe>` XML document from [`CteBuildData`].
///
/// Generates the 44-digit access key, derives `cCT`/`cDV` from it, and emits
/// every block in schema order.
///
/// # Errors
///
/// - [`FiscalError::XmlGeneration`] if the access key cannot be built.
pub fn build_cte_xml(data: &CteBuildData) -> Result<String, FiscalError> {
    let access_key =
        build_cte_access_key_from_ide(&data.ide, &data.emit_cnpj, data.numeric_code.as_deref())?;

    let c_ct = &access_key.numeric_code;
    let c_dv = &access_key.key[43..44];

    let mut children = vec![build_ide(&data.ide, c_ct, c_dv)];
    if let Some(compl) = &data.compl {
        children.push(build_compl(compl));
    }
    children.push(build_emit(&data.emit));
    if let Some(p) = &data.rem {
        children.push(build_party("rem", "enderReme", p, true, false));
    }
    if let Some(p) = &data.exped {
        children.push(build_party("exped", "enderExped", p, false, false));
    }
    if let Some(p) = &data.receb {
        children.push(build_party("receb", "enderReceb", p, false, false));
    }
    if let Some(p) = &data.dest {
        children.push(build_party("dest", "enderDest", p, false, true));
    }
    children.push(build_vprest(&data.v_prest));
    children.push(build_imp(&data.imp));
    // Choice: Complementar (tpCTe 1) emite infCteComp; senão infCTeNorm.
    if data.inf_cte_comp.is_empty() {
        children.push(build_inf_cte_norm(&data.inf_cte_norm));
    } else {
        for ch in &data.inf_cte_comp {
            children.push(tag(
                "infCteComp",
                &[],
                TagContent::Children(vec![tag("chCTe", &[], TagContent::Text(ch))]),
            ));
        }
    }
    children.extend(data.aut_xml.iter().map(build_aut_xml));
    if let Some(rt) = &data.inf_resp_tec {
        children.push(build_inf_resp_tec(rt));
    }

    let id_attr = format!("CTe{}", access_key.key);
    let inf_cte = tag(
        "infCte",
        &[("versao", CTE_VERSION), ("Id", &id_attr)],
        TagContent::Children(children),
    );

    let cte = tag(
        "CTe",
        &[("xmlns", CTE_NAMESPACE)],
        TagContent::Children(vec![inf_cte]),
    );

    Ok(cte)
}

/// Format a datetime as CT-e ISO 8601 with explicit Brazil offset (no UTC `Z`).
///
/// Converts the instant to the UF's local timezone before formatting so that
/// the AAMM in the access key matches the `dhEmi` field even across midnight
/// or month-end crossings (same fix applied to MDF-e in fiscal-mdfe).
pub(crate) fn format_datetime_cte(dt: &chrono::DateTime<chrono::FixedOffset>, uf: &str) -> String {
    let secs = match uf {
        "AC" => 5 * 3600,
        "AM" | "RO" | "RR" | "MT" | "MS" => 4 * 3600,
        _ => 3 * 3600,
    };
    let offset = chrono::FixedOffset::west_opt(secs).expect("static offset is valid");
    let local = dt.with_timezone(&offset);
    local.format("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

#[cfg(test)]
mod datetime_tests {
    use chrono::{FixedOffset, TimeZone};

    use super::format_datetime_cte;

    #[test]
    fn am_converts_brt_to_amt() {
        // 10:30 BRT (UTC-3) = 09:30 AMT (UTC-4)
        let brt = FixedOffset::west_opt(3 * 3600).unwrap();
        let dt = brt.with_ymd_and_hms(2026, 1, 15, 10, 30, 0).unwrap();
        let out = format_datetime_cte(&dt, "AM");
        assert_eq!(out, "2026-01-15T09:30:00-04:00");
    }

    #[test]
    fn sp_stays_brt() {
        let brt = FixedOffset::west_opt(3 * 3600).unwrap();
        let dt = brt.with_ymd_and_hms(2026, 6, 10, 14, 0, 0).unwrap();
        let out = format_datetime_cte(&dt, "SP");
        assert_eq!(out, "2026-06-10T14:00:00-03:00");
    }

    #[test]
    fn ac_converts_brt_to_act() {
        // 10:00 BRT (UTC-3) = 08:00 ACT (UTC-5)
        let brt = FixedOffset::west_opt(3 * 3600).unwrap();
        let dt = brt.with_ymd_and_hms(2026, 3, 1, 10, 0, 0).unwrap();
        let out = format_datetime_cte(&dt, "AC");
        assert_eq!(out, "2026-03-01T08:00:00-05:00");
    }
}

/// Emit a `<CNPJ>` or `<CPF>` element from a [`Documento`].
pub(crate) fn build_documento(doc: &Documento) -> String {
    match doc {
        Documento::Cnpj(v) => tag("CNPJ", &[], TagContent::Text(v)),
        Documento::Cpf(v) => tag("CPF", &[], TagContent::Text(v)),
    }
}

// ── ide ──────────────────────────────────────────────────────────────────────

fn build_ide(ide: &Ide, c_ct: &str, c_dv: &str) -> String {
    let serie = ide.serie.to_string();
    let n_ct = ide.n_ct.to_string();
    let dh_emi = format_datetime_cte(&ide.dh_emi, &ide.uf_env);

    let mut children = vec![
        tag("cUF", &[], TagContent::Text(&ide.c_uf)),
        tag("cCT", &[], TagContent::Text(c_ct)),
        tag("CFOP", &[], TagContent::Text(&ide.cfop)),
        tag("natOp", &[], TagContent::Text(&ide.nat_op)),
        tag("mod", &[], TagContent::Text(CTE_MODEL)),
        tag("serie", &[], TagContent::Text(&serie)),
        tag("nCT", &[], TagContent::Text(&n_ct)),
        tag("dhEmi", &[], TagContent::Text(&dh_emi)),
        tag("tpImp", &[], TagContent::Text(&ide.tp_imp)),
        tag("tpEmis", &[], TagContent::Text(&ide.tp_emis)),
        tag("cDV", &[], TagContent::Text(c_dv)),
        tag("tpAmb", &[], TagContent::Text(&ide.tp_amb)),
        tag("tpCTe", &[], TagContent::Text(&ide.tp_cte)),
        tag(
            "procEmi",
            &[],
            TagContent::Text(ide.proc_emi.as_deref().unwrap_or("0")),
        ),
        tag(
            "verProc",
            &[],
            TagContent::Text(ide.ver_proc.as_deref().unwrap_or("fiscal-cte 0.1.0")),
        ),
    ];

    if let Some(ig) = &ide.ind_globalizado {
        children.push(tag("indGlobalizado", &[], TagContent::Text(ig)));
    }

    children.extend([
        tag("cMunEnv", &[], TagContent::Text(&ide.c_mun_env)),
        tag("xMunEnv", &[], TagContent::Text(&ide.x_mun_env)),
        tag("UFEnv", &[], TagContent::Text(&ide.uf_env)),
        tag("modal", &[], TagContent::Text(&ide.modal)),
        tag("tpServ", &[], TagContent::Text(&ide.tp_serv)),
        tag("cMunIni", &[], TagContent::Text(&ide.c_mun_ini)),
        tag("xMunIni", &[], TagContent::Text(&ide.x_mun_ini)),
        tag("UFIni", &[], TagContent::Text(&ide.uf_ini)),
        tag("cMunFim", &[], TagContent::Text(&ide.c_mun_fim)),
        tag("xMunFim", &[], TagContent::Text(&ide.x_mun_fim)),
        tag("UFFim", &[], TagContent::Text(&ide.uf_fim)),
        tag("retira", &[], TagContent::Text(&ide.retira)),
    ]);

    if let Some(xd) = &ide.x_det_retira {
        children.push(tag("xDetRetira", &[], TagContent::Text(xd)));
    }

    children.push(tag("indIEToma", &[], TagContent::Text(&ide.ind_ie_toma)));
    children.push(build_tomador(&ide.toma));

    tag("ide", &[], TagContent::Children(children))
}

fn build_tomador(toma: &Tomador) -> String {
    match toma {
        Tomador::Toma3 { toma } => tag(
            "toma3",
            &[],
            TagContent::Children(vec![tag("toma", &[], TagContent::Text(toma))]),
        ),
        Tomador::Toma4 {
            toma,
            doc,
            ie,
            x_nome,
            x_fant,
            fone,
            ender_toma,
            email,
        } => {
            let mut c = vec![
                tag("toma", &[], TagContent::Text(toma)),
                build_documento(doc),
            ];
            if let Some(ie) = ie {
                c.push(tag("IE", &[], TagContent::Text(ie)));
            }
            c.push(tag("xNome", &[], TagContent::Text(x_nome)));
            if let Some(xf) = x_fant {
                c.push(tag("xFant", &[], TagContent::Text(xf)));
            }
            if let Some(f) = fone {
                c.push(tag("fone", &[], TagContent::Text(f)));
            }
            c.push(build_endereco("enderToma", ender_toma, false));
            if let Some(e) = email {
                c.push(tag("email", &[], TagContent::Text(e)));
            }
            tag("toma4", &[], TagContent::Children(c))
        }
    }
}

// ── compl ────────────────────────────────────────────────────────────────────

pub(crate) fn build_compl(compl: &Compl) -> String {
    let mut c = Vec::new();
    if let Some(v) = &compl.x_carac_ad {
        c.push(tag("xCaracAd", &[], TagContent::Text(v)));
    }
    if let Some(v) = &compl.x_carac_ser {
        c.push(tag("xCaracSer", &[], TagContent::Text(v)));
    }
    if let Some(v) = &compl.x_emi {
        c.push(tag("xEmi", &[], TagContent::Text(v)));
    }
    if let Some(v) = &compl.x_obs {
        c.push(tag("xObs", &[], TagContent::Text(v)));
    }
    for o in &compl.obs_cont {
        c.push(tag(
            "ObsCont",
            &[("xCampo", &o.x_campo)],
            TagContent::Children(vec![tag("xTexto", &[], TagContent::Text(&o.x_texto))]),
        ));
    }
    for o in &compl.obs_fisco {
        c.push(tag(
            "ObsFisco",
            &[("xCampo", &o.x_campo)],
            TagContent::Children(vec![tag("xTexto", &[], TagContent::Text(&o.x_texto))]),
        ));
    }
    tag("compl", &[], TagContent::Children(c))
}

// ── emit ─────────────────────────────────────────────────────────────────────

pub(crate) fn build_emit(emit: &Emit) -> String {
    let mut c = vec![build_documento(&emit.doc)];
    if let Some(ie) = &emit.ie {
        c.push(tag("IE", &[], TagContent::Text(ie)));
    }
    if let Some(iest) = &emit.iest {
        c.push(tag("IEST", &[], TagContent::Text(iest)));
    }
    c.push(tag("xNome", &[], TagContent::Text(&emit.x_nome)));
    if let Some(xf) = &emit.x_fant {
        c.push(tag("xFant", &[], TagContent::Text(xf)));
    }
    c.push(build_endereco("enderEmit", &emit.ender_emit, true));
    c.push(tag("CRT", &[], TagContent::Text(&emit.crt)));
    tag("emit", &[], TagContent::Children(c))
}

// ── parties ──────────────────────────────────────────────────────────────────

pub(crate) fn build_party(
    tag_name: &str,
    ender_tag: &str,
    p: &Party,
    include_xfant: bool,
    include_isuf: bool,
) -> String {
    let mut c = vec![build_documento(&p.doc)];
    if let Some(ie) = &p.ie {
        c.push(tag("IE", &[], TagContent::Text(ie)));
    }
    c.push(tag("xNome", &[], TagContent::Text(&p.x_nome)));
    if include_xfant {
        if let Some(xf) = &p.x_fant {
            c.push(tag("xFant", &[], TagContent::Text(xf)));
        }
    }
    if let Some(f) = &p.fone {
        c.push(tag("fone", &[], TagContent::Text(f)));
    }
    if include_isuf {
        if let Some(isuf) = &p.isuf {
            c.push(tag("ISUF", &[], TagContent::Text(isuf)));
        }
    }
    c.push(build_endereco(ender_tag, &p.ender, false));
    if let Some(e) = &p.email {
        c.push(tag("email", &[], TagContent::Text(e)));
    }
    tag(tag_name, &[], TagContent::Children(c))
}

/// Build an address block. `is_emit` selects the `TEndeEmi` shape (trailing
/// `fone`) versus the party `TEndereco` shape (trailing `cPais`/`xPais`).
pub(crate) fn build_endereco(tag_name: &str, e: &Endereco, is_emit: bool) -> String {
    let mut c = vec![
        tag("xLgr", &[], TagContent::Text(&e.x_lgr)),
        tag("nro", &[], TagContent::Text(&e.nro)),
    ];
    if let Some(v) = &e.x_cpl {
        c.push(tag("xCpl", &[], TagContent::Text(v)));
    }
    c.push(tag("xBairro", &[], TagContent::Text(&e.x_bairro)));
    c.push(tag("cMun", &[], TagContent::Text(&e.c_mun)));
    c.push(tag("xMun", &[], TagContent::Text(&e.x_mun)));
    if let Some(cep) = &e.cep {
        c.push(tag("CEP", &[], TagContent::Text(cep)));
    }
    c.push(tag("UF", &[], TagContent::Text(&e.uf)));
    if is_emit {
        if let Some(f) = &e.fone {
            c.push(tag("fone", &[], TagContent::Text(f)));
        }
    } else {
        if let Some(cp) = &e.c_pais {
            c.push(tag("cPais", &[], TagContent::Text(cp)));
        }
        if let Some(xp) = &e.x_pais {
            c.push(tag("xPais", &[], TagContent::Text(xp)));
        }
    }
    tag(tag_name, &[], TagContent::Children(c))
}

// ── vPrest ───────────────────────────────────────────────────────────────────

pub(crate) fn build_vprest(v: &VPrest) -> String {
    let mut c = vec![
        tag("vTPrest", &[], TagContent::Text(&v.v_t_prest)),
        tag("vRec", &[], TagContent::Text(&v.v_rec)),
    ];
    for comp in &v.comp {
        c.push(tag(
            "Comp",
            &[],
            TagContent::Children(vec![
                tag("xNome", &[], TagContent::Text(&comp.x_nome)),
                tag("vComp", &[], TagContent::Text(&comp.v_comp)),
            ]),
        ));
    }
    tag("vPrest", &[], TagContent::Children(c))
}

// ── imp / ICMS ───────────────────────────────────────────────────────────────

pub(crate) fn build_imp(imp: &Imp) -> String {
    let mut c = vec![build_icms(&imp.icms)];
    if let Some(v) = &imp.v_tot_trib {
        c.push(tag("vTotTrib", &[], TagContent::Text(v)));
    }
    if let Some(v) = &imp.inf_ad_fisco {
        c.push(tag("infAdFisco", &[], TagContent::Text(v)));
    }
    tag("imp", &[], TagContent::Children(c))
}

pub(crate) fn build_icms(icms: &Icms) -> String {
    let inner = match icms {
        Icms::Icms00 {
            v_bc,
            p_icms,
            v_icms,
        } => tag(
            "ICMS00",
            &[],
            TagContent::Children(vec![
                tag("CST", &[], TagContent::Text("00")),
                tag("vBC", &[], TagContent::Text(v_bc)),
                tag("pICMS", &[], TagContent::Text(p_icms)),
                tag("vICMS", &[], TagContent::Text(v_icms)),
            ]),
        ),
        Icms::Icms20 {
            p_red_bc,
            v_bc,
            p_icms,
            v_icms,
        } => tag(
            "ICMS20",
            &[],
            TagContent::Children(vec![
                tag("CST", &[], TagContent::Text("20")),
                tag("pRedBC", &[], TagContent::Text(p_red_bc)),
                tag("vBC", &[], TagContent::Text(v_bc)),
                tag("pICMS", &[], TagContent::Text(p_icms)),
                tag("vICMS", &[], TagContent::Text(v_icms)),
            ]),
        ),
        Icms::Icms45 { cst_code } => tag(
            "ICMS45",
            &[],
            TagContent::Children(vec![tag("CST", &[], TagContent::Text(cst_code))]),
        ),
        Icms::Icms90 {
            p_red_bc,
            v_bc,
            p_icms,
            v_icms,
            v_cred,
        } => {
            let mut cc = vec![tag("CST", &[], TagContent::Text("90"))];
            if let Some(p) = p_red_bc {
                cc.push(tag("pRedBC", &[], TagContent::Text(p)));
            }
            cc.push(tag("vBC", &[], TagContent::Text(v_bc)));
            cc.push(tag("pICMS", &[], TagContent::Text(p_icms)));
            cc.push(tag("vICMS", &[], TagContent::Text(v_icms)));
            if let Some(v) = v_cred {
                cc.push(tag("vCred", &[], TagContent::Text(v)));
            }
            tag("ICMS90", &[], TagContent::Children(cc))
        }
        Icms::IcmsSn { ind_sn } => tag(
            "ICMSSN",
            &[],
            TagContent::Children(vec![
                tag("CST", &[], TagContent::Text("90")),
                tag("indSN", &[], TagContent::Text(ind_sn)),
            ]),
        ),
    };
    tag("ICMS", &[], TagContent::Children(vec![inner]))
}

// ── infCTeNorm ───────────────────────────────────────────────────────────────

fn build_inf_cte_norm(n: &InfCteNorm) -> String {
    let mut c = vec![build_inf_carga(&n.inf_carga)];
    if let Some(d) = &n.inf_doc {
        c.push(build_inf_doc(d));
    }
    c.push(build_inf_modal(&n.inf_modal));
    if let Some(sub) = &n.inf_cte_sub {
        let mut sc = vec![tag("chCte", &[], TagContent::Text(&sub.ch_cte))];
        if let Some(ia) = &sub.ind_altera_toma {
            sc.push(tag("indAlteraToma", &[], TagContent::Text(ia)));
        }
        c.push(tag("infCteSub", &[], TagContent::Children(sc)));
    }
    for s in &n.seg {
        let mut sc = vec![tag("respSeg", &[], TagContent::Text(&s.resp_seg))];
        if let Some(v) = &s.x_seg {
            sc.push(tag("xSeg", &[], TagContent::Text(v)));
        }
        if let Some(v) = &s.cnpj_seg {
            sc.push(tag("CNPJ", &[], TagContent::Text(v)));
        }
        if let Some(v) = &s.n_apol {
            sc.push(tag("nApol", &[], TagContent::Text(v)));
        }
        c.push(tag("seg", &[], TagContent::Children(sc)));
    }
    tag("infCTeNorm", &[], TagContent::Children(c))
}

fn build_inf_carga(carga: &InfCarga) -> String {
    let mut c = Vec::new();
    if let Some(v) = &carga.v_carga {
        c.push(tag("vCarga", &[], TagContent::Text(v)));
    }
    c.push(tag("proPred", &[], TagContent::Text(&carga.pro_pred)));
    if let Some(v) = &carga.x_out_cat {
        c.push(tag("xOutCat", &[], TagContent::Text(v)));
    }
    for q in &carga.inf_q {
        c.push(tag(
            "infQ",
            &[],
            TagContent::Children(vec![
                tag("cUnid", &[], TagContent::Text(&q.c_unid)),
                tag("tpMed", &[], TagContent::Text(&q.tp_med)),
                tag("qCarga", &[], TagContent::Text(&q.q_carga)),
            ]),
        ));
    }
    if let Some(v) = &carga.v_carga_averb {
        c.push(tag("vCargaAverb", &[], TagContent::Text(v)));
    }
    tag("infCarga", &[], TagContent::Children(c))
}

fn build_inf_doc(d: &InfDoc) -> String {
    // `infDoc` é um `<xs:choice>`: ou todos `infNFe`, ou todos `infOutros` (não
    // misturados). Prefere NF-e quando há; senão emite os outros documentos.
    let mut c = Vec::new();
    if !d.inf_nfe.is_empty() {
        for nfe in &d.inf_nfe {
            let mut nc = vec![tag("chave", &[], TagContent::Text(&nfe.chave))];
            if let Some(dp) = &nfe.d_prev {
                nc.push(tag("dPrev", &[], TagContent::Text(dp)));
            }
            c.push(tag("infNFe", &[], TagContent::Children(nc)));
        }
    } else {
        for o in &d.inf_outros {
            let mut oc = vec![tag("tpDoc", &[], TagContent::Text(&o.tp_doc))];
            if let Some(v) = &o.desc_outros {
                oc.push(tag("descOutros", &[], TagContent::Text(v)));
            }
            if let Some(v) = &o.n_doc {
                oc.push(tag("nDoc", &[], TagContent::Text(v)));
            }
            if let Some(v) = &o.d_emi {
                oc.push(tag("dEmi", &[], TagContent::Text(v)));
            }
            if let Some(v) = &o.v_doc_fisc {
                oc.push(tag("vDocFisc", &[], TagContent::Text(v)));
            }
            c.push(tag("infOutros", &[], TagContent::Children(oc)));
        }
    }
    tag("infDoc", &[], TagContent::Children(c))
}

fn build_rodo_cte(r: &RodoCte) -> String {
    let mut children = vec![tag("RNTRC", &[], TagContent::Text(&r.rntrc))];
    // infANTT somente quando há CIOT ou valePed
    if !r.inf_ciot.is_empty() || !r.vale_ped.is_empty() {
        let mut antt = Vec::new();
        for c in &r.inf_ciot {
            antt.push(tag(
                "infCIOT",
                &[],
                TagContent::Children(vec![
                    tag("CIOT", &[], TagContent::Text(&c.ciot)),
                    tag(
                        if c.tax_id.len() == 14 { "CNPJ" } else { "CPF" },
                        &[],
                        TagContent::Text(&c.tax_id),
                    ),
                ]),
            ));
        }
        for vp in &r.vale_ped {
            let mut vc = vec![
                tag("CNPJForn", &[], TagContent::Text(&vp.cnpj_forn)),
                tag("nCompra", &[], TagContent::Text(&vp.n_compra)),
            ];
            if let Some(v) = &vp.v_vale_ped {
                vc.push(tag("vValePed", &[], TagContent::Text(v)));
            }
            antt.push(tag("valePed", &[], TagContent::Children(vc)));
        }
        children.push(tag("infANTT", &[], TagContent::Children(antt)));
    }
    tag("rodo", &[], TagContent::Children(children))
}

fn build_inf_modal(m: &InfModal) -> String {
    let modal = match &m.modal {
        Modal::Rodo(rodo) => build_rodo_cte(rodo),
        Modal::Aereo {
            d_prev_aereo,
            x_dime,
            tarifa_cl,
            tarifa_v_tar,
            n_minu,
        } => {
            let mut c = Vec::new();
            if let Some(v) = n_minu {
                c.push(tag("nMinu", &[], TagContent::Text(v)));
            }
            c.push(tag("dPrevAereo", &[], TagContent::Text(d_prev_aereo)));
            let mut nat = Vec::new();
            if let Some(v) = x_dime {
                nat.push(tag("xDime", &[], TagContent::Text(v)));
            }
            c.push(tag("natCarga", &[], TagContent::Children(nat)));
            c.push(tag(
                "tarifa",
                &[],
                TagContent::Children(vec![
                    tag("CL", &[], TagContent::Text(tarifa_cl)),
                    tag("vTar", &[], TagContent::Text(tarifa_v_tar)),
                ]),
            ));
            tag("aereo", &[], TagContent::Children(c))
        }
        Modal::Aquav {
            v_prest,
            v_afrmm,
            x_navio,
            direc,
            irin,
            n_viag,
        } => {
            let mut c = vec![
                tag("vPrest", &[], TagContent::Text(v_prest)),
                tag("vAFRMM", &[], TagContent::Text(v_afrmm)),
                tag("xNavio", &[], TagContent::Text(x_navio)),
            ];
            if let Some(v) = n_viag {
                c.push(tag("nViag", &[], TagContent::Text(v)));
            }
            c.push(tag("direc", &[], TagContent::Text(direc)));
            c.push(tag("irin", &[], TagContent::Text(irin)));
            tag("aquav", &[], TagContent::Children(c))
        }
        Modal::Ferrov { tp_traf, fluxo } => tag(
            "ferrov",
            &[],
            TagContent::Children(vec![
                tag("tpTraf", &[], TagContent::Text(tp_traf)),
                tag("fluxo", &[], TagContent::Text(fluxo)),
            ]),
        ),
        Modal::Duto {
            d_ini,
            d_fim,
            v_tar,
        } => {
            let mut c = Vec::new();
            if let Some(v) = v_tar {
                c.push(tag("vTar", &[], TagContent::Text(v)));
            }
            c.push(tag("dIni", &[], TagContent::Text(d_ini)));
            c.push(tag("dFim", &[], TagContent::Text(d_fim)));
            tag("duto", &[], TagContent::Children(c))
        }
        Modal::Multimodal {
            cotm,
            ind_negociavel,
        } => tag(
            "multimodal",
            &[],
            TagContent::Children(vec![
                tag("COTM", &[], TagContent::Text(cotm)),
                tag("indNegociavel", &[], TagContent::Text(ind_negociavel)),
            ]),
        ),
    };
    tag(
        "infModal",
        &[("versaoModal", &m.versao_modal)],
        TagContent::Children(vec![modal]),
    )
}

// ── autXML / infRespTec ──────────────────────────────────────────────────────

pub(crate) fn build_aut_xml(a: &AutXml) -> String {
    tag(
        "autXML",
        &[],
        TagContent::Children(vec![build_documento(&a.doc)]),
    )
}

pub(crate) fn build_inf_resp_tec(rt: &InfRespTec) -> String {
    tag(
        "infRespTec",
        &[],
        TagContent::Children(vec![
            tag("CNPJ", &[], TagContent::Text(&rt.cnpj)),
            tag("xContato", &[], TagContent::Text(&rt.x_contato)),
            tag("email", &[], TagContent::Text(&rt.email)),
            tag("fone", &[], TagContent::Text(&rt.fone)),
        ]),
    )
}
