//! XML builders para o elemento `<IBSCBS>` de cada item da NF-e.

use super::*;
use crate::xml_utils::{TagContent, tag};

// ── XML builders ────────────────────────────────────────────────────────

fn build_g_dif(prefix: &str, data: &GDifData) -> String {
    let _ = prefix;
    let mut c = vec![
        tag("pDif", &[], TagContent::Text(&data.p_dif)),
        tag("vDif", &[], TagContent::Text(&data.v_dif)),
    ];
    let _ = &mut c;
    tag("gDif", &[], TagContent::Children(c))
}

fn build_g_dev_trib(data: &GDevTribData) -> String {
    tag(
        "gDevTrib",
        &[],
        TagContent::Children(vec![tag(
            "vDevTrib",
            &[],
            TagContent::Text(&data.v_dev_trib),
        )]),
    )
}

fn build_g_red(data: &GRedData) -> String {
    tag(
        "gRed",
        &[],
        TagContent::Children(vec![
            tag("pRedAliq", &[], TagContent::Text(&data.p_red_aliq)),
            tag("pAliqEfet", &[], TagContent::Text(&data.p_aliq_efet)),
        ]),
    )
}

fn build_g_ibs_uf(data: &GIbsUfData) -> String {
    let mut children = vec![tag("pIBSUF", &[], TagContent::Text(&data.p_ibs_uf))];
    if let Some(ref dif) = data.g_dif {
        children.push(build_g_dif("IBSUF", dif));
    }
    if let Some(ref dev) = data.g_dev_trib {
        children.push(build_g_dev_trib(dev));
    }
    if let Some(ref red) = data.g_red {
        children.push(build_g_red(red));
    }
    children.push(tag("vIBSUF", &[], TagContent::Text(&data.v_ibs_uf)));
    tag("gIBSUF", &[], TagContent::Children(children))
}

fn build_g_ibs_mun(data: &GIbsMunData) -> String {
    let mut children = vec![tag("pIBSMun", &[], TagContent::Text(&data.p_ibs_mun))];
    if let Some(ref dif) = data.g_dif {
        children.push(build_g_dif("IBSMun", dif));
    }
    if let Some(ref dev) = data.g_dev_trib {
        children.push(build_g_dev_trib(dev));
    }
    if let Some(ref red) = data.g_red {
        children.push(build_g_red(red));
    }
    children.push(tag("vIBSMun", &[], TagContent::Text(&data.v_ibs_mun)));
    tag("gIBSMun", &[], TagContent::Children(children))
}

fn build_g_cbs(data: &GCbsData) -> String {
    let mut children = vec![tag("pCBS", &[], TagContent::Text(&data.p_cbs))];
    if let Some(ref dif) = data.g_dif {
        children.push(build_g_dif("CBS", dif));
    }
    if let Some(ref dev) = data.g_dev_trib {
        children.push(build_g_dev_trib(dev));
    }
    if let Some(ref red) = data.g_red {
        children.push(build_g_red(red));
    }
    children.push(tag("vCBS", &[], TagContent::Text(&data.v_cbs)));
    tag("gCBS", &[], TagContent::Children(children))
}

fn build_g_ibs_cbs(data: &GIbsCbsData) -> String {
    let mut children = vec![tag("vBC", &[], TagContent::Text(&data.v_bc))];
    children.push(build_g_ibs_uf(&data.g_ibs_uf));
    children.push(build_g_ibs_mun(&data.g_ibs_mun));
    // vIBS = provided or calculated placeholder
    let v_ibs_val = data.v_ibs.as_deref().unwrap_or("0.00");
    children.push(tag("vIBS", &[], TagContent::Text(v_ibs_val)));
    children.push(build_g_cbs(&data.g_cbs));
    tag("gIBSCBS", &[], TagContent::Children(children))
}

fn build_g_trib_regular(data: &GTribRegularData) -> String {
    tag(
        "gTribRegular",
        &[],
        TagContent::Children(vec![
            tag("CSTReg", &[], TagContent::Text(&data.cst_reg)),
            tag(
                "cClassTribReg",
                &[],
                TagContent::Text(&data.c_class_trib_reg),
            ),
            tag(
                "pAliqEfetRegIBSUF",
                &[],
                TagContent::Text(&data.p_aliq_efet_reg_ibs_uf),
            ),
            tag(
                "vTribRegIBSUF",
                &[],
                TagContent::Text(&data.v_trib_reg_ibs_uf),
            ),
            tag(
                "pAliqEfetRegIBSMun",
                &[],
                TagContent::Text(&data.p_aliq_efet_reg_ibs_mun),
            ),
            tag(
                "vTribRegIBSMun",
                &[],
                TagContent::Text(&data.v_trib_reg_ibs_mun),
            ),
            tag(
                "pAliqEfetRegCBS",
                &[],
                TagContent::Text(&data.p_aliq_efet_reg_cbs),
            ),
            tag("vTribRegCBS", &[], TagContent::Text(&data.v_trib_reg_cbs)),
        ]),
    )
}

fn build_g_trib_compra_gov(data: &GTribCompraGovData) -> String {
    tag(
        "gTribCompraGov",
        &[],
        TagContent::Children(vec![
            tag("pAliqIBSUF", &[], TagContent::Text(&data.p_aliq_ibs_uf)),
            tag("vTribIBSUF", &[], TagContent::Text(&data.v_trib_ibs_uf)),
            tag("pAliqIBSMun", &[], TagContent::Text(&data.p_aliq_ibs_mun)),
            tag("vTribIBSMun", &[], TagContent::Text(&data.v_trib_ibs_mun)),
            tag("pAliqCBS", &[], TagContent::Text(&data.p_aliq_cbs)),
            tag("vTribCBS", &[], TagContent::Text(&data.v_trib_cbs)),
        ]),
    )
}

fn build_g_ibs_cbs_mono(data: &GIbsCbsMonoData) -> String {
    let mut children: Vec<String> = Vec::new();
    if let Some(ref p) = data.g_mono_padrao {
        children.push(tag(
            "gMonoPadrao",
            &[],
            TagContent::Children(vec![
                tag("qBCMono", &[], TagContent::Text(&p.q_bc_mono)),
                tag("adRemIBS", &[], TagContent::Text(&p.ad_rem_ibs)),
                tag("adRemCBS", &[], TagContent::Text(&p.ad_rem_cbs)),
                tag("vIBSMono", &[], TagContent::Text(&p.v_ibs_mono)),
                tag("vCBSMono", &[], TagContent::Text(&p.v_cbs_mono)),
            ]),
        ));
    }
    if let Some(ref r) = data.g_mono_reten {
        children.push(tag(
            "gMonoReten",
            &[],
            TagContent::Children(vec![
                tag("qBCMonoReten", &[], TagContent::Text(&r.q_bc_mono_reten)),
                tag("adRemIBSReten", &[], TagContent::Text(&r.ad_rem_ibs_reten)),
                tag("vIBSMonoReten", &[], TagContent::Text(&r.v_ibs_mono_reten)),
                tag("adRemCBSReten", &[], TagContent::Text(&r.ad_rem_cbs_reten)),
                tag("vCBSMonoReten", &[], TagContent::Text(&r.v_cbs_mono_reten)),
            ]),
        ));
    }
    if let Some(ref r) = data.g_mono_ret {
        children.push(tag(
            "gMonoRet",
            &[],
            TagContent::Children(vec![
                tag("qBCMonoRet", &[], TagContent::Text(&r.q_bc_mono_ret)),
                tag("adRemIBSRet", &[], TagContent::Text(&r.ad_rem_ibs_ret)),
                tag("vIBSMonoRet", &[], TagContent::Text(&r.v_ibs_mono_ret)),
                tag("adRemCBSRet", &[], TagContent::Text(&r.ad_rem_cbs_ret)),
                tag("vCBSMonoRet", &[], TagContent::Text(&r.v_cbs_mono_ret)),
            ]),
        ));
    }
    if let Some(ref d) = data.g_mono_dif {
        children.push(tag(
            "gMonoDif",
            &[],
            TagContent::Children(vec![
                tag("pDifIBS", &[], TagContent::Text(&d.p_dif_ibs)),
                tag("vIBSMonoDif", &[], TagContent::Text(&d.v_ibs_mono_dif)),
                tag("pDifCBS", &[], TagContent::Text(&d.p_dif_cbs)),
                tag("vCBSMonoDif", &[], TagContent::Text(&d.v_cbs_mono_dif)),
            ]),
        ));
    }
    children.push(tag(
        "vTotIBSMonoItem",
        &[],
        TagContent::Text(&data.v_tot_ibs_mono_item),
    ));
    children.push(tag(
        "vTotCBSMonoItem",
        &[],
        TagContent::Text(&data.v_tot_cbs_mono_item),
    ));
    tag("gIBSCBSMono", &[], TagContent::Children(children))
}

fn build_g_transf_cred(data: &GTransfCredData) -> String {
    tag(
        "gTransfCred",
        &[],
        TagContent::Children(vec![
            tag("vIBS", &[], TagContent::Text(&data.v_ibs)),
            tag("vCBS", &[], TagContent::Text(&data.v_cbs)),
        ]),
    )
}

fn build_g_cred_pres_ibs_zfm(data: &GCredPresIbsZfmData) -> String {
    let mut children: Vec<String> = Vec::new();
    if let Some(ref ca) = data.compet_apur {
        children.push(tag("competApur", &[], TagContent::Text(ca)));
    }
    children.push(tag(
        "tpCredPresIBSZFM",
        &[],
        TagContent::Text(&data.tp_cred_pres_ibs_zfm),
    ));
    children.push(tag(
        "vCredPresIBSZFM",
        &[],
        TagContent::Text(&data.v_cred_pres_ibs_zfm),
    ));
    tag("gCredPresIBSZFM", &[], TagContent::Children(children))
}

fn build_g_ajuste_compet(data: &GAjusteCompetData) -> String {
    tag(
        "gAjusteCompet",
        &[],
        TagContent::Children(vec![
            tag("competApur", &[], TagContent::Text(&data.compet_apur)),
            tag("vIBS", &[], TagContent::Text(&data.v_ibs)),
            tag("vCBS", &[], TagContent::Text(&data.v_cbs)),
        ]),
    )
}

fn build_g_estorno_cred(data: &GEstornoCredData) -> String {
    tag(
        "gEstornoCred",
        &[],
        TagContent::Children(vec![
            tag("vIBSEstCred", &[], TagContent::Text(&data.v_ibs_est_cred)),
            tag("vCBSEstCred", &[], TagContent::Text(&data.v_cbs_est_cred)),
        ]),
    )
}

fn build_cred_pres_sub(tag_name: &str, data: &GIbsCredPresData) -> String {
    let mut children = vec![tag("pCredPres", &[], TagContent::Text(&data.p_cred_pres))];
    if let Some(ref v) = data.v_cred_pres {
        children.push(tag("vCredPres", &[], TagContent::Text(v)));
    } else if let Some(ref v) = data.v_cred_pres_cond_sus {
        children.push(tag("vCredPresCondSus", &[], TagContent::Text(v)));
    }
    tag(tag_name, &[], TagContent::Children(children))
}

fn build_g_cred_pres_oper(data: &GCredPresOperData) -> String {
    let mut children = vec![
        tag("vBCCredPres", &[], TagContent::Text(&data.v_bc_cred_pres)),
        tag("cCredPres", &[], TagContent::Text(&data.c_cred_pres)),
    ];
    if let Some(ref ibs) = data.g_ibs_cred_pres {
        children.push(build_cred_pres_sub("gIBSCredPres", ibs));
    }
    if let Some(ref cbs) = data.g_cbs_cred_pres {
        children.push(build_cred_pres_sub("gCBSCredPres", cbs));
    }
    tag("gCredPresOper", &[], TagContent::Children(children))
}

/// Build the complete `<IBSCBS>` XML string for an invoice item.
pub fn build_ibs_cbs_xml(data: &IbsCbsData) -> String {
    let mut children = vec![
        tag("CST", &[], TagContent::Text(&data.cst)),
        tag("cClassTrib", &[], TagContent::Text(&data.c_class_trib)),
    ];
    if data.ind_doacao {
        children.push(tag("indDoacao", &[], TagContent::Text("1")));
    }
    // gIBSCBS (ad-valorem) -- choice with gIBSCBSMono
    if let Some(ref g) = data.g_ibs_cbs {
        children.push(build_g_ibs_cbs(g));
    }
    // gTribRegular -- appended inside IBSCBS after gIBSCBS
    if let Some(ref g) = data.g_trib_regular {
        children.push(build_g_trib_regular(g));
    }
    // gTribCompraGov
    if let Some(ref g) = data.g_trib_compra_gov {
        children.push(build_g_trib_compra_gov(g));
    }
    // gIBSCBSMono -- choice with gIBSCBS
    if let Some(ref g) = data.g_ibs_cbs_mono {
        children.push(build_g_ibs_cbs_mono(g));
    }
    // gTransfCred
    if let Some(ref g) = data.g_transf_cred {
        children.push(build_g_transf_cred(g));
    }
    // gCredPresIBSZFM
    if let Some(ref g) = data.g_cred_pres_ibs_zfm {
        children.push(build_g_cred_pres_ibs_zfm(g));
    }
    // gAjusteCompet
    if let Some(ref g) = data.g_ajuste_compet {
        children.push(build_g_ajuste_compet(g));
    }
    // gEstornoCred
    if let Some(ref g) = data.g_estorno_cred {
        children.push(build_g_estorno_cred(g));
    }
    // gCredPresOper
    if let Some(ref g) = data.g_cred_pres_oper {
        children.push(build_g_cred_pres_oper(g));
    }
    tag("IBSCBS", &[], TagContent::Children(children))
}
