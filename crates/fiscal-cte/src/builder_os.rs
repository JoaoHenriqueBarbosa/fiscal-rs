//! String-based XML builder for the CT-e OS (model 67), leiaute 4.00.
//!
//! Block order (per `cteOS_v4.00.xsd`):
//!
//! ```text
//! CTeOS > infCte[@Id] >
//!   ide, compl?, emit, toma, vPrest, imp,
//!   infCTeNorm{ infServico, infDocRef*, seg*, infModal{rodoOS} },
//!   autXML*, infRespTec?
//! ```
//!
//! Shared blocks (emit, compl, vPrest, imp, autXML, infRespTec, endereço) are
//! reused from [`crate::builder`]. The returned XML is **unsigned** and carries
//! no `<infCTeSupl>` — the QR supplement and signature are added by the hub.

use fiscal_core::FiscalError;
use fiscal_core::xml_builder::access_key::{format_year_month, generate_numeric_code};
use fiscal_core::xml_utils::{TagContent, tag};

use crate::access_key::{CteAccessKeyParams, build_cte_access_key};
use crate::builder::{
    build_aut_xml, build_compl, build_documento, build_emit, build_endereco, build_imp,
    build_inf_resp_tec, build_vprest, format_datetime_cte,
};
use crate::types_os::*;
use crate::{CTE_NAMESPACE, CTE_VERSION, CTEOS_MODEL};

/// Build a complete unsigned `<CTeOS>` XML document from [`CteOsBuildData`].
///
/// # Errors
///
/// [`FiscalError::XmlGeneration`] if the access key cannot be built.
pub fn build_cteos_xml(data: &CteOsBuildData) -> Result<String, FiscalError> {
    let generated;
    let cct = match data.numeric_code.as_deref() {
        Some(c) => c,
        None => {
            generated = generate_numeric_code();
            &generated
        }
    };
    let access_key = build_cte_access_key(&CteAccessKeyParams {
        model: CTEOS_MODEL,
        state_code: &data.ide.c_uf,
        year_month: format_year_month(&data.ide.dh_emi),
        tax_id: &data.emit_cnpj,
        series: data.ide.serie,
        number: data.ide.n_ct,
        emission_type: &data.ide.tp_emis,
        numeric_code: cct,
    })?;

    let c_ct = &access_key.numeric_code;
    let c_dv = &access_key.key[43..44];

    let mut children = vec![build_ide_os(&data.ide, c_ct, c_dv)];
    if let Some(compl) = &data.compl {
        children.push(build_compl(compl));
    }
    children.push(build_emit(&data.emit));
    children.push(build_toma(&data.toma));
    children.push(build_vprest(&data.v_prest));
    children.push(build_imp(&data.imp));
    children.push(build_inf_cte_norm_os(&data.inf_cte_norm));
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
    Ok(tag(
        "CTeOS",
        &[("xmlns", CTE_NAMESPACE), ("versao", CTE_VERSION)],
        TagContent::Children(vec![inf_cte]),
    ))
}

fn build_ide_os(ide: &IdeOs, c_ct: &str, c_dv: &str) -> String {
    let serie = ide.serie.to_string();
    let n_ct = ide.n_ct.to_string();
    let dh_emi = format_datetime_cte(&ide.dh_emi, &ide.uf_env);

    let mut c = vec![
        tag("cUF", &[], TagContent::Text(&ide.c_uf)),
        tag("cCT", &[], TagContent::Text(c_ct)),
        tag("CFOP", &[], TagContent::Text(&ide.cfop)),
        tag("natOp", &[], TagContent::Text(&ide.nat_op)),
        tag("mod", &[], TagContent::Text(CTEOS_MODEL)),
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
        tag("cMunEnv", &[], TagContent::Text(&ide.c_mun_env)),
        tag("xMunEnv", &[], TagContent::Text(&ide.x_mun_env)),
        tag("UFEnv", &[], TagContent::Text(&ide.uf_env)),
        tag("modal", &[], TagContent::Text(&ide.modal)),
        tag("tpServ", &[], TagContent::Text(&ide.tp_serv)),
        tag("indIEToma", &[], TagContent::Text(&ide.ind_ie_toma)),
        tag("cMunIni", &[], TagContent::Text(&ide.c_mun_ini)),
        tag("xMunIni", &[], TagContent::Text(&ide.x_mun_ini)),
        tag("UFIni", &[], TagContent::Text(&ide.uf_ini)),
        tag("cMunFim", &[], TagContent::Text(&ide.c_mun_fim)),
        tag("xMunFim", &[], TagContent::Text(&ide.x_mun_fim)),
        tag("UFFim", &[], TagContent::Text(&ide.uf_fim)),
    ];

    for uf in &ide.inf_percurso {
        c.push(tag(
            "infPercurso",
            &[],
            TagContent::Children(vec![tag("UFPer", &[], TagContent::Text(uf))]),
        ));
    }
    if let Some(dh) = &ide.dh_cont {
        c.push(tag("dhCont", &[], TagContent::Text(dh)));
    }
    if let Some(xj) = &ide.x_just {
        c.push(tag("xJust", &[], TagContent::Text(xj)));
    }

    tag("ide", &[], TagContent::Children(c))
}

pub(crate) fn build_toma(t: &TomaOs) -> String {
    let mut c = vec![build_documento(&t.doc)];
    if let Some(ie) = &t.ie {
        c.push(tag("IE", &[], TagContent::Text(ie)));
    }
    c.push(tag("xNome", &[], TagContent::Text(&t.x_nome)));
    if let Some(xf) = &t.x_fant {
        c.push(tag("xFant", &[], TagContent::Text(xf)));
    }
    if let Some(f) = &t.fone {
        c.push(tag("fone", &[], TagContent::Text(f)));
    }
    c.push(build_endereco("enderToma", &t.ender_toma, false));
    if let Some(e) = &t.email {
        c.push(tag("email", &[], TagContent::Text(e)));
    }
    tag("toma", &[], TagContent::Children(c))
}

fn build_inf_cte_norm_os(n: &InfCteNormOs) -> String {
    let mut c = vec![build_inf_servico(&n.inf_servico)];
    for d in &n.inf_doc_ref {
        c.push(build_inf_doc_ref(d));
    }
    for s in &n.seg {
        c.push(build_seg(s));
    }
    c.push(build_inf_modal_os(&n.inf_modal));
    tag("infCTeNorm", &[], TagContent::Children(c))
}

fn build_inf_servico(s: &InfServico) -> String {
    let mut c = vec![tag("xDescServ", &[], TagContent::Text(&s.x_desc_serv))];
    if let Some(q) = &s.q_carga {
        c.push(tag(
            "infQ",
            &[],
            TagContent::Children(vec![tag("qCarga", &[], TagContent::Text(q))]),
        ));
    }
    tag("infServico", &[], TagContent::Children(c))
}

fn build_inf_doc_ref(d: &InfDocRef) -> String {
    let mut c = Vec::new();
    if let Some(ch) = &d.ch_bpe {
        c.push(tag("chBPe", &[], TagContent::Text(ch)));
    } else {
        if let Some(v) = &d.n_doc {
            c.push(tag("nDoc", &[], TagContent::Text(v)));
        }
        if let Some(v) = &d.serie {
            c.push(tag("serie", &[], TagContent::Text(v)));
        }
        if let Some(v) = &d.subserie {
            c.push(tag("subserie", &[], TagContent::Text(v)));
        }
        if let Some(v) = &d.d_emi {
            c.push(tag("dEmi", &[], TagContent::Text(v)));
        }
        if let Some(v) = &d.v_doc {
            c.push(tag("vDoc", &[], TagContent::Text(v)));
        }
    }
    tag("infDocRef", &[], TagContent::Children(c))
}

fn build_seg(s: &Seg) -> String {
    let mut c = vec![tag("respSeg", &[], TagContent::Text(&s.resp_seg))];
    if let Some(v) = &s.x_seg {
        c.push(tag("xSeg", &[], TagContent::Text(v)));
    }
    if let Some(v) = &s.n_apol {
        c.push(tag("nApol", &[], TagContent::Text(v)));
    }
    tag("seg", &[], TagContent::Children(c))
}

fn build_inf_modal_os(m: &InfModalOs) -> String {
    tag(
        "infModal",
        &[("versaoModal", &m.versao_modal)],
        TagContent::Children(vec![build_rodo_os(&m.rodo_os)]),
    )
}

fn build_rodo_os(r: &RodoOs) -> String {
    let mut c = Vec::new();
    if let Some(taf) = &r.taf {
        c.push(tag("TAF", &[], TagContent::Text(taf)));
    }
    if let Some(nre) = &r.nro_reg_estadual {
        c.push(tag("NroRegEstadual", &[], TagContent::Text(nre)));
    }
    if let Some(v) = &r.veic {
        let mut vc = vec![tag("placa", &[], TagContent::Text(&v.placa))];
        if let Some(rv) = &v.renavam {
            vc.push(tag("RENAVAM", &[], TagContent::Text(rv)));
        }
        if let Some(p) = &v.prop {
            let mut pc = vec![build_documento(&p.doc)];
            if let Some(x) = &p.x_nome {
                pc.push(tag("xNome", &[], TagContent::Text(x)));
            }
            if let Some(ie) = &p.ie {
                pc.push(tag("IE", &[], TagContent::Text(ie)));
            }
            if let Some(uf) = &p.uf {
                pc.push(tag("UF", &[], TagContent::Text(uf)));
            }
            pc.push(tag("tpProp", &[], TagContent::Text(&p.tp_prop)));
            vc.push(tag("prop", &[], TagContent::Children(pc)));
        }
        if let Some(uf) = &v.uf {
            vc.push(tag("UF", &[], TagContent::Text(uf)));
        }
        c.push(tag("veic", &[], TagContent::Children(vc)));
    }
    if let Some(f) = &r.inf_fretamento {
        let mut fc = vec![tag("tpFretamento", &[], TagContent::Text(&f.tp_fretamento))];
        if let Some(dh) = &f.dh_viagem {
            fc.push(tag("dhViagem", &[], TagContent::Text(dh)));
        }
        c.push(tag("infFretamento", &[], TagContent::Children(fc)));
    }
    tag("rodoOS", &[], TagContent::Children(c))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Documento, Emit, Endereco, Icms, Imp, VPrest};

    fn ender() -> Endereco {
        Endereco {
            x_lgr: "RUA A".into(),
            nro: "10".into(),
            x_cpl: None,
            x_bairro: "CENTRO".into(),
            c_mun: "3550308".into(),
            x_mun: "SAO PAULO".into(),
            cep: Some("01001000".into()),
            uf: "SP".into(),
            c_pais: Some("1058".into()),
            x_pais: Some("BRASIL".into()),
            fone: None,
        }
    }

    fn sample() -> CteOsBuildData {
        CteOsBuildData {
            numeric_code: Some("00000001".into()),
            emit_cnpj: "12345678000190".into(),
            ide: IdeOs {
                c_uf: "35".into(),
                cfop: "5357".into(),
                nat_op: "TRANSPORTE DE PESSOAS".into(),
                serie: 1,
                n_ct: 1,
                dh_emi: "2026-06-06T10:00:00-03:00".parse().unwrap(),
                tp_imp: "1".into(),
                tp_emis: "1".into(),
                tp_amb: "2".into(),
                tp_cte: "0".into(),
                proc_emi: None,
                ver_proc: None,
                c_mun_env: "3550308".into(),
                x_mun_env: "SAO PAULO".into(),
                uf_env: "SP".into(),
                modal: "01".into(),
                tp_serv: "6".into(),
                ind_ie_toma: "9".into(),
                c_mun_ini: "3550308".into(),
                x_mun_ini: "SAO PAULO".into(),
                uf_ini: "SP".into(),
                c_mun_fim: "3509502".into(),
                x_mun_fim: "CAMPINAS".into(),
                uf_fim: "SP".into(),
                inf_percurso: vec![],
                dh_cont: None,
                x_just: None,
            },
            compl: None,
            emit: Emit {
                doc: Documento::Cnpj("12345678000190".into()),
                ie: Some("111111111111".into()),
                iest: None,
                x_nome: "TRANSP TESTE".into(),
                x_fant: None,
                ender_emit: ender(),
                crt: "3".into(),
            },
            toma: TomaOs {
                doc: Documento::Cpf("34493536837".into()),
                ie: None,
                x_nome: "FULANO".into(),
                x_fant: None,
                fone: None,
                ender_toma: ender(),
                email: None,
            },
            v_prest: VPrest {
                v_t_prest: "100.00".into(),
                v_rec: "100.00".into(),
                comp: vec![],
            },
            imp: Imp {
                icms: Icms::IcmsSn { ind_sn: "1".into() },
                v_tot_trib: None,
                inf_ad_fisco: None,
            },
            inf_cte_norm: InfCteNormOs {
                inf_servico: InfServico {
                    x_desc_serv: "TRANSPORTE DE PESSOAS".into(),
                    q_carga: None,
                },
                inf_doc_ref: vec![InfDocRef {
                    n_doc: Some("123".into()),
                    serie: None,
                    subserie: None,
                    d_emi: Some("2026-06-06".into()),
                    v_doc: Some("100.00".into()),
                    ch_bpe: None,
                }],
                seg: vec![],
                inf_modal: InfModalOs {
                    versao_modal: "4.00".into(),
                    rodo_os: RodoOs {
                        taf: Some("123456789".into()),
                        nro_reg_estadual: None,
                        veic: None,
                        inf_fretamento: None,
                    },
                },
            },
            aut_xml: vec![],
            inf_resp_tec: None,
        }
    }

    #[test]
    fn builds_well_formed_cteos() {
        let xml = build_cteos_xml(&sample()).unwrap();
        assert!(xml.starts_with("<CTeOS"));
        assert!(xml.contains("<mod>67</mod>"));
        assert!(xml.contains("<toma>"));
        assert!(xml.contains("<infServico><xDescServ>"));
        assert!(xml.contains("<rodoOS><TAF>123456789</TAF>"));
        assert!(xml.contains("<infDocRef><nDoc>123</nDoc>"));
        // Id = "CTe" + 44-digit key, with model 67 at positions 20-22.
        let id = xml
            .split("Id=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        assert_eq!(&id[3..][20..22], "67");
    }
}
