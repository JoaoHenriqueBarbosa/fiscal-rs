//! String-based XML builder for the NFS-e Nacional DPS (leiaute 1.01).
//!
//! Block order (per `DPS_v1.01.xsd` / `infDPS`):
//!
//! ```text
//! DPS > infDPS[@Id] >
//!   tpAmb, dhEmi, verAplic, serie, nDPS, dCompet, tpEmit, cLocEmi,
//!   prest, toma?, serv, valores
//! ```
//!
//! O `Id` é `DPS` + cLocEmi(7) + tpInsc(1) + inscrição(14) + serie(5) + nDPS(15)
//! = `DPS` + 42 dígitos (pattern `DPS[0-9]{42}`).

use fiscal_core::xml_utils::{TagContent, tag};

use crate::types::*;
use crate::{NFSE_NAMESPACE, NFSE_VERSION};

/// Build a complete unsigned `<DPS>` XML document from [`DpsBuildData`].
pub fn build_dps_xml(data: &DpsBuildData) -> String {
    let id = build_dps_id(&data.ide, &data.prest.doc);
    let dh_emi = format!("{}", data.ide.dh_emi.format("%Y-%m-%dT%H:%M:%S%:z"));
    let n_dps = data.ide.n_dps.to_string();

    let mut c = vec![
        tag("tpAmb", &[], TagContent::Text(&data.ide.tp_amb)),
        tag("dhEmi", &[], TagContent::Text(&dh_emi)),
        tag("verAplic", &[], TagContent::Text(&data.ide.ver_aplic)),
        tag("serie", &[], TagContent::Text(&data.ide.serie)),
        tag("nDPS", &[], TagContent::Text(&n_dps)),
        tag("dCompet", &[], TagContent::Text(&data.ide.d_compet)),
        tag("tpEmit", &[], TagContent::Text(&data.ide.tp_emit)),
        tag("cLocEmi", &[], TagContent::Text(&data.ide.c_loc_emi)),
        build_prest(&data.prest),
    ];
    if let Some(t) = &data.toma {
        c.push(build_pessoa("toma", t));
    }
    c.push(build_serv(&data.serv));
    c.push(build_valores(&data.valores));
    if let Some(ib) = &data.ibscbs {
        c.push(build_ibscbs(ib));
    }

    let inf = tag("infDPS", &[("Id", &id)], TagContent::Children(c));
    tag(
        "DPS",
        &[("xmlns", NFSE_NAMESPACE), ("versao", NFSE_VERSION)],
        TagContent::Children(vec![inf]),
    )
}

/// Build a NFS-e **cancelamento** event (`<pedRegEvento>`, tpEvento 101101).
///
/// `ch_nfse` é a chave de 50 dígitos; `c_motivo` o código (`1` erro de
/// emissão, `2` serviço não prestado, `9` outros); `x_desc` a descrição;
/// `tax_id` o CNPJ/CPF do autor. Retorna o XML não assinado.
pub fn build_nfse_cancelamento(
    ch_nfse: &str,
    tax_id: &str,
    c_motivo: &str,
    x_desc: &str,
    tp_amb: &str,
    dh_evento: &str,
) -> String {
    let id = format!("PRE{ch_nfse}101101");
    let autor = {
        let d: String = tax_id.chars().filter(|c| c.is_ascii_digit()).collect();
        if d.len() == 11 {
            tag("CPFAutor", &[], TagContent::Text(&d))
        } else {
            tag("CNPJAutor", &[], TagContent::Text(&d))
        }
    };
    let e = tag(
        "e101101",
        &[],
        TagContent::Children(vec![
            tag("xDesc", &[], TagContent::Text("Cancelamento de NFS-e")),
            tag("cMotivo", &[], TagContent::Text(c_motivo)),
            tag("xMotivo", &[], TagContent::Text(x_desc)),
        ]),
    );
    let inf = tag(
        "infPedReg",
        &[("Id", &id)],
        TagContent::Children(vec![
            tag("tpAmb", &[], TagContent::Text(tp_amb)),
            tag("verAplic", &[], TagContent::Text("dfehub-1.0")),
            tag("dhEvento", &[], TagContent::Text(dh_evento)),
            autor,
            tag("chNFSe", &[], TagContent::Text(ch_nfse)),
            e,
        ]),
    );
    tag(
        "pedRegEvento",
        &[("xmlns", NFSE_NAMESPACE), ("versao", NFSE_VERSION)],
        TagContent::Children(vec![inf]),
    )
}

/// `DPS` + cLocEmi(7) + tpInsc(1) + inscrição(14) + serie(5) + nDPS(15).
///
/// `tpInsc` segue o manual NFS-e Nacional: **1 = CPF, 2 = CNPJ**
/// (inverso da intuição — divergência causa rejeição E0004).
fn build_dps_id(ide: &IdeDps, doc: &Documento) -> String {
    let (tp_insc, insc) = match doc {
        Documento::Cnpj(v) => ("2", v.clone()),
        Documento::Cpf(v) => ("1", v.clone()),
    };
    format!(
        "DPS{:0>7}{tp_insc}{:0>14}{:0>5}{:0>15}",
        ide.c_loc_emi, insc, ide.serie, ide.n_dps
    )
}

fn build_doc(doc: &Documento) -> String {
    match doc {
        Documento::Cnpj(v) => tag("CNPJ", &[], TagContent::Text(v)),
        Documento::Cpf(v) => tag("CPF", &[], TagContent::Text(v)),
    }
}

fn build_ender(e: &EnderNac) -> String {
    let end_nac = tag(
        "endNac",
        &[],
        TagContent::Children(vec![
            tag("cMun", &[], TagContent::Text(&e.c_mun)),
            tag("CEP", &[], TagContent::Text(&e.cep)),
        ]),
    );
    let mut c = vec![
        end_nac,
        tag("xLgr", &[], TagContent::Text(&e.x_lgr)),
        tag("nro", &[], TagContent::Text(&e.nro)),
    ];
    if let Some(v) = &e.x_cpl {
        c.push(tag("xCpl", &[], TagContent::Text(v)));
    }
    c.push(tag("xBairro", &[], TagContent::Text(&e.x_bairro)));
    tag("end", &[], TagContent::Children(c))
}

fn build_prest(p: &Prestador) -> String {
    let mut c = vec![build_doc(&p.doc)];
    if let Some(v) = &p.im {
        c.push(tag("IM", &[], TagContent::Text(v)));
    }
    c.push(tag("xNome", &[], TagContent::Text(&p.x_nome)));
    if let Some(e) = &p.end {
        c.push(build_ender(e));
    }
    if let Some(v) = &p.fone {
        c.push(tag("fone", &[], TagContent::Text(v)));
    }
    if let Some(v) = &p.email {
        c.push(tag("email", &[], TagContent::Text(v)));
    }
    c.push(tag(
        "regTrib",
        &[],
        TagContent::Children(vec![
            tag("opSimpNac", &[], TagContent::Text(&p.reg_trib.op_simp_nac)),
            tag(
                "regEspTrib",
                &[],
                TagContent::Text(&p.reg_trib.reg_esp_trib),
            ),
        ]),
    ));
    tag("prest", &[], TagContent::Children(c))
}

fn build_pessoa(tag_name: &str, p: &Pessoa) -> String {
    let mut c = vec![build_doc(&p.doc)];
    if let Some(v) = &p.im {
        c.push(tag("IM", &[], TagContent::Text(v)));
    }
    c.push(tag("xNome", &[], TagContent::Text(&p.x_nome)));
    if let Some(e) = &p.end {
        c.push(build_ender(e));
    }
    if let Some(v) = &p.fone {
        c.push(tag("fone", &[], TagContent::Text(v)));
    }
    if let Some(v) = &p.email {
        c.push(tag("email", &[], TagContent::Text(v)));
    }
    tag(tag_name, &[], TagContent::Children(c))
}

fn build_serv(s: &Servico) -> String {
    let loc = tag(
        "locPrest",
        &[],
        TagContent::Children(vec![tag(
            "cLocPrestacao",
            &[],
            TagContent::Text(&s.c_loc_prestacao),
        )]),
    );
    let mut cserv = vec![tag("cTribNac", &[], TagContent::Text(&s.c_trib_nac))];
    if let Some(v) = &s.c_trib_mun {
        cserv.push(tag("cTribMun", &[], TagContent::Text(v)));
    }
    cserv.push(tag("xDescServ", &[], TagContent::Text(&s.x_desc_serv)));
    tag(
        "serv",
        &[],
        TagContent::Children(vec![loc, tag("cServ", &[], TagContent::Children(cserv))]),
    )
}

fn build_valores(v: &Valores) -> String {
    let v_serv_prest = tag(
        "vServPrest",
        &[],
        TagContent::Children(vec![tag("vServ", &[], TagContent::Text(&v.v_serv))]),
    );
    let mut trib_mun = vec![tag(
        "tribISSQN",
        &[],
        TagContent::Text(&v.trib.trib_mun.trib_issqn),
    )];
    trib_mun.push(tag(
        "tpRetISSQN",
        &[],
        TagContent::Text(&v.trib.trib_mun.tp_ret_issqn),
    ));
    if let Some(a) = &v.trib.trib_mun.p_aliq {
        trib_mun.push(tag("pAliq", &[], TagContent::Text(a)));
    }
    let mut trib_children = vec![tag("tribMun", &[], TagContent::Children(trib_mun))];
    if let Some(tf) = &v.trib.trib_fed {
        trib_children.push(build_trib_fed(tf));
    }
    trib_children.push(tag(
        "totTrib",
        &[],
        TagContent::Children(vec![tag("indTotTrib", &[], TagContent::Text("0"))]),
    ));
    let trib = tag("trib", &[], TagContent::Children(trib_children));
    tag(
        "valores",
        &[],
        TagContent::Children(vec![v_serv_prest, trib]),
    )
}

fn build_trib_fed(tf: &TribFed) -> String {
    let mut c = Vec::new();
    if let Some(pc) = &tf.piscofins {
        let mut pcc = vec![tag("CST", &[], TagContent::Text(&pc.cst))];
        if let Some(x) = &pc.v_bc {
            pcc.push(tag("vBCPisCofins", &[], TagContent::Text(x)));
        }
        if let Some(x) = &pc.p_aliq_pis {
            pcc.push(tag("pAliqPis", &[], TagContent::Text(x)));
        }
        if let Some(x) = &pc.p_aliq_cofins {
            pcc.push(tag("pAliqCofins", &[], TagContent::Text(x)));
        }
        if let Some(x) = &pc.v_pis {
            pcc.push(tag("vPis", &[], TagContent::Text(x)));
        }
        if let Some(x) = &pc.v_cofins {
            pcc.push(tag("vCofins", &[], TagContent::Text(x)));
        }
        if let Some(x) = &pc.tp_ret {
            pcc.push(tag("tpRetPisCofins", &[], TagContent::Text(x)));
        }
        c.push(tag("piscofins", &[], TagContent::Children(pcc)));
    }
    if let Some(x) = &tf.v_ret_cp {
        c.push(tag("vRetCP", &[], TagContent::Text(x)));
    }
    if let Some(x) = &tf.v_ret_irrf {
        c.push(tag("vRetIRRF", &[], TagContent::Text(x)));
    }
    if let Some(x) = &tf.v_ret_csll {
        c.push(tag("vRetCSLL", &[], TagContent::Text(x)));
    }
    tag("tribFed", &[], TagContent::Children(c))
}

fn build_ibscbs(ib: &Ibscbs) -> String {
    let mut c = vec![tag("finNFSe", &[], TagContent::Text(&ib.fin_nfse))];
    if let Some(x) = &ib.ind_final {
        c.push(tag("indFinal", &[], TagContent::Text(x)));
    }
    c.push(tag("cIndOp", &[], TagContent::Text(&ib.c_ind_op)));
    c.push(tag("indDest", &[], TagContent::Text(&ib.ind_dest)));

    let mut sit = vec![
        tag("CST", &[], TagContent::Text(&ib.cst)),
        tag("cClassTrib", &[], TagContent::Text(&ib.c_class_trib)),
    ];
    if let Some(x) = &ib.c_cred_pres {
        sit.push(tag("cCredPres", &[], TagContent::Text(x)));
    }
    let g_ibscbs = tag("gIBSCBS", &[], TagContent::Children(sit));
    let trib = tag("trib", &[], TagContent::Children(vec![g_ibscbs]));
    let valores = tag("valores", &[], TagContent::Children(vec![trib]));
    c.push(valores);

    tag("IBSCBS", &[], TagContent::Children(c))
}
