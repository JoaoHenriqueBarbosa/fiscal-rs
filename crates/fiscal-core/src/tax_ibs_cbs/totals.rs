//! Tipos e builders XML para totais IBS/CBS e IS: `<IBSCBSTot>` e `<ISTot>`.

use crate::xml_utils::{TagContent, tag};

// ── IBS/CBS Total data ──────────────────────────────────────────────────

/// Total do IBS/CBS: `<IBSCBSTot>` inside `<total>`.
///
/// Follows PHP `tagIBSCBSTot`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct IbsCbsTotData {
    /// Base de calculo total (`vBCIBSCBS`).
    pub v_bc_ibs_cbs: String,
    // gIBS
    /// Total IBS UF diferimento (`gIBSUF/vDif`).
    pub g_ibs_uf_v_dif: Option<String>,
    /// Total IBS UF devolucao (`gIBSUF/vDevTrib`).
    pub g_ibs_uf_v_dev_trib: Option<String>,
    /// Total IBS UF (`gIBSUF/vIBSUF`).
    pub g_ibs_uf_v_ibs_uf: Option<String>,
    /// Total IBS Mun diferimento (`gIBSMun/vDif`).
    pub g_ibs_mun_v_dif: Option<String>,
    /// Total IBS Mun devolucao (`gIBSMun/vDevTrib`).
    pub g_ibs_mun_v_dev_trib: Option<String>,
    /// Total IBS Mun (`gIBSMun/vIBSMun`).
    pub g_ibs_mun_v_ibs_mun: Option<String>,
    /// Total IBS (`gIBS/vIBS`).
    pub g_ibs_v_ibs: Option<String>,
    /// Total IBS credito presumido (`gIBS/vCredPres`).
    pub g_ibs_v_cred_pres: Option<String>,
    /// Total IBS credito presumido condicao suspensiva (`gIBS/vCredPresCondSus`).
    pub g_ibs_v_cred_pres_cond_sus: Option<String>,
    // gCBS
    /// Total CBS diferimento (`gCBS/vDif`).
    pub g_cbs_v_dif: Option<String>,
    /// Total CBS devolucao (`gCBS/vDevTrib`).
    pub g_cbs_v_dev_trib: Option<String>,
    /// Total CBS (`gCBS/vCBS`).
    pub g_cbs_v_cbs: Option<String>,
    /// Total CBS credito presumido (`gCBS/vCredPres`).
    pub g_cbs_v_cred_pres: Option<String>,
    /// Total CBS credito presumido condicao suspensiva (`gCBS/vCredPresCondSus`).
    pub g_cbs_v_cred_pres_cond_sus: Option<String>,
    // gMono
    /// Total IBS monofasico (`gMono/vIBSMono`).
    pub g_mono_v_ibs_mono: Option<String>,
    /// Total CBS monofasica (`gMono/vCBSMono`).
    pub g_mono_v_cbs_mono: Option<String>,
    /// Total IBS monofasico retencao (`gMono/vIBSMonoReten`).
    pub g_mono_v_ibs_mono_reten: Option<String>,
    /// Total CBS monofasica retencao (`gMono/vCBSMonoReten`).
    pub g_mono_v_cbs_mono_reten: Option<String>,
    /// Total IBS monofasico retido anteriormente (`gMono/vIBSMonoRet`).
    pub g_mono_v_ibs_mono_ret: Option<String>,
    /// Total CBS monofasica retida anteriormente (`gMono/vCBSMonoRet`).
    pub g_mono_v_cbs_mono_ret: Option<String>,
    // gEstornoCred
    /// Total IBS estornado (`gEstornoCred/vIBSEstCred`).
    pub g_estorno_cred_v_ibs_est_cred: Option<String>,
    /// Total CBS estornada (`gEstornoCred/vCBSEstCred`).
    pub g_estorno_cred_v_cbs_est_cred: Option<String>,
}

impl IbsCbsTotData {
    /// Create a new `IbsCbsTotData` with required BC.
    pub fn new(v_bc_ibs_cbs: impl Into<String>) -> Self {
        Self {
            v_bc_ibs_cbs: v_bc_ibs_cbs.into(),
            ..Default::default()
        }
    }
}

/// Total do IS (Imposto Seletivo): `<ISTot>` inside `<total>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct IsTotData {
    /// Valor total do IS (`vIS`).
    pub v_is: String,
}

impl IsTotData {
    /// Create a new `IsTotData`.
    pub fn new(v_is: impl Into<String>) -> Self {
        Self { v_is: v_is.into() }
    }
}

/// Build the `<ISTot>` XML element.
pub fn build_is_tot_xml(data: &IsTotData) -> String {
    tag(
        "ISTot",
        &[],
        TagContent::Children(vec![tag("vIS", &[], TagContent::Text(&data.v_is))]),
    )
}

/// Build the `<IBSCBSTot>` XML element.
pub fn build_ibs_cbs_tot_xml(data: &IbsCbsTotData) -> String {
    let mut children = vec![tag("vBCIBSCBS", &[], TagContent::Text(&data.v_bc_ibs_cbs))];

    // gIBS block
    if let Some(ref v_ibs) = data.g_ibs_v_ibs {
        let mut g_ibs_children: Vec<String> = Vec::new();
        // gIBSUF
        let g_ibs_uf_children: Vec<String> = vec![
            tag(
                "vDif",
                &[],
                TagContent::Text(data.g_ibs_uf_v_dif.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vDevTrib",
                &[],
                TagContent::Text(data.g_ibs_uf_v_dev_trib.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vIBSUF",
                &[],
                TagContent::Text(data.g_ibs_uf_v_ibs_uf.as_deref().unwrap_or("0.00")),
            ),
        ];
        g_ibs_children.push(tag("gIBSUF", &[], TagContent::Children(g_ibs_uf_children)));
        // gIBSMun
        let g_ibs_mun_children: Vec<String> = vec![
            tag(
                "vDif",
                &[],
                TagContent::Text(data.g_ibs_mun_v_dif.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vDevTrib",
                &[],
                TagContent::Text(data.g_ibs_mun_v_dev_trib.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vIBSMun",
                &[],
                TagContent::Text(data.g_ibs_mun_v_ibs_mun.as_deref().unwrap_or("0.00")),
            ),
        ];
        g_ibs_children.push(tag(
            "gIBSMun",
            &[],
            TagContent::Children(g_ibs_mun_children),
        ));
        g_ibs_children.push(tag("vIBS", &[], TagContent::Text(v_ibs)));
        g_ibs_children.push(tag(
            "vCredPres",
            &[],
            TagContent::Text(data.g_ibs_v_cred_pres.as_deref().unwrap_or("0.00")),
        ));
        g_ibs_children.push(tag(
            "vCredPresCondSus",
            &[],
            TagContent::Text(data.g_ibs_v_cred_pres_cond_sus.as_deref().unwrap_or("0.00")),
        ));
        children.push(tag("gIBS", &[], TagContent::Children(g_ibs_children)));
    }

    // gCBS block
    if let Some(ref v_cbs) = data.g_cbs_v_cbs {
        let g_cbs_children: Vec<String> = vec![
            tag(
                "vDif",
                &[],
                TagContent::Text(data.g_cbs_v_dif.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vDevTrib",
                &[],
                TagContent::Text(data.g_cbs_v_dev_trib.as_deref().unwrap_or("0.00")),
            ),
            tag("vCBS", &[], TagContent::Text(v_cbs)),
            tag(
                "vCredPres",
                &[],
                TagContent::Text(data.g_cbs_v_cred_pres.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vCredPresCondSus",
                &[],
                TagContent::Text(data.g_cbs_v_cred_pres_cond_sus.as_deref().unwrap_or("0.00")),
            ),
        ];
        children.push(tag("gCBS", &[], TagContent::Children(g_cbs_children)));
    }

    // gMono block
    if let Some(ref v) = data.g_mono_v_ibs_mono {
        let g_mono_children = vec![
            tag("vIBSMono", &[], TagContent::Text(v)),
            tag(
                "vCBSMono",
                &[],
                TagContent::Text(data.g_mono_v_cbs_mono.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vIBSMonoReten",
                &[],
                TagContent::Text(data.g_mono_v_ibs_mono_reten.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vCBSMonoReten",
                &[],
                TagContent::Text(data.g_mono_v_cbs_mono_reten.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vIBSMonoRet",
                &[],
                TagContent::Text(data.g_mono_v_ibs_mono_ret.as_deref().unwrap_or("0.00")),
            ),
            tag(
                "vCBSMonoRet",
                &[],
                TagContent::Text(data.g_mono_v_cbs_mono_ret.as_deref().unwrap_or("0.00")),
            ),
        ];
        children.push(tag("gMono", &[], TagContent::Children(g_mono_children)));
    }

    // gEstornoCred block
    let has_est_ibs = data
        .g_estorno_cred_v_ibs_est_cred
        .as_ref()
        .is_some_and(|v| !v.is_empty());
    let has_est_cbs = data
        .g_estorno_cred_v_cbs_est_cred
        .as_ref()
        .is_some_and(|v| !v.is_empty());
    if has_est_ibs || has_est_cbs {
        let g_est_children = vec![
            tag(
                "vIBSEstCred",
                &[],
                TagContent::Text(
                    data.g_estorno_cred_v_ibs_est_cred
                        .as_deref()
                        .unwrap_or("0.00"),
                ),
            ),
            tag(
                "vCBSEstCred",
                &[],
                TagContent::Text(
                    data.g_estorno_cred_v_cbs_est_cred
                        .as_deref()
                        .unwrap_or("0.00"),
                ),
            ),
        ];
        children.push(tag(
            "gEstornoCred",
            &[],
            TagContent::Children(g_est_children),
        ));
    }

    tag("IBSCBSTot", &[], TagContent::Children(children))
}
