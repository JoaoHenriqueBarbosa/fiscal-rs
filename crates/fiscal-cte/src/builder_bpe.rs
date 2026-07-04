//! String-based XML builder for the BP-e (model 63), leiaute 1.00.
//!
//! Block order (per `bpe_v1.00.xsd` / `TBPe`):
//!
//! ```text
//! BPe > infBPe[@Id] >
//!   ide, emit, comp?, infValorBPe, infViagem+, infPassagem, imp, pag+,
//!   autXML*, infRespTec?
//! ```

use fiscal_core::FiscalError;
use fiscal_core::xml_builder::access_key::{format_year_month, generate_numeric_code};
use fiscal_core::xml_utils::{TagContent, tag};

use crate::access_key::{CteAccessKeyParams, build_cte_access_key};
use crate::builder::{
    build_aut_xml, build_documento, build_endereco, build_icms, build_inf_resp_tec,
    format_datetime_cte,
};
use crate::types_bpe::*;
use crate::{BPE_MODEL, BPE_NAMESPACE, BPE_VERSION};

/// Build a complete unsigned `<BPe>` XML document from [`BpeBuildData`].
///
/// # Errors
///
/// [`FiscalError::XmlGeneration`] if the access key cannot be built.
pub fn build_bpe_xml(data: &BpeBuildData) -> Result<String, FiscalError> {
    let generated;
    let cbp = match data.numeric_code.as_deref() {
        Some(c) => c,
        None => {
            generated = generate_numeric_code();
            &generated
        }
    };
    let access_key = build_cte_access_key(&CteAccessKeyParams {
        model: BPE_MODEL,
        state_code: &data.ide.c_uf,
        year_month: format_year_month(&data.ide.dh_emi),
        tax_id: &data.emit_cnpj,
        series: data.ide.serie,
        number: data.ide.n_bp,
        emission_type: &data.ide.tp_emis,
        numeric_code: cbp,
    })?;
    let c_bp = &access_key.numeric_code;
    let c_dv = &access_key.key[43..44];

    let mut children = vec![build_ide_bpe(&data.ide, c_bp, c_dv)];
    children.push(build_emit_bpe(&data.emit));
    if let Some(c) = &data.comp {
        children.push(build_comp(c));
    }
    children.push(build_inf_passagem(&data.inf_passagem));
    for v in &data.inf_viagem {
        children.push(build_inf_viagem(v));
    }
    children.push(build_inf_valor(&data.inf_valor));
    children.push(build_bpe_imp(&data.imp));
    for p in &data.pag {
        children.push(build_pag(p));
    }
    children.extend(data.aut_xml.iter().map(build_aut_xml));
    if let Some(rt) = &data.inf_resp_tec {
        children.push(build_inf_resp_tec(rt));
    }

    let id_attr = format!("BPe{}", access_key.key);
    let inf_bpe = tag(
        "infBPe",
        &[("versao", BPE_VERSION), ("Id", &id_attr)],
        TagContent::Children(children),
    );
    Ok(tag(
        "BPe",
        &[("xmlns", BPE_NAMESPACE)],
        TagContent::Children(vec![inf_bpe]),
    ))
}

fn build_ide_bpe(ide: &IdeBpe, c_bp: &str, c_dv: &str) -> String {
    let serie = ide.serie.to_string();
    let n_bp = ide.n_bp.to_string();
    let dh_emi = format_datetime_cte(&ide.dh_emi, &ide.uf_ini);
    let mut c = vec![
        tag("cUF", &[], TagContent::Text(&ide.c_uf)),
        tag("tpAmb", &[], TagContent::Text(&ide.tp_amb)),
        tag("mod", &[], TagContent::Text(BPE_MODEL)),
        tag("serie", &[], TagContent::Text(&serie)),
        tag("nBP", &[], TagContent::Text(&n_bp)),
        tag("cBP", &[], TagContent::Text(c_bp)),
        tag("cDV", &[], TagContent::Text(c_dv)),
        tag("modal", &[], TagContent::Text(&ide.modal)),
        tag("dhEmi", &[], TagContent::Text(&dh_emi)),
        tag("tpEmis", &[], TagContent::Text(&ide.tp_emis)),
        tag(
            "verProc",
            &[],
            TagContent::Text(ide.ver_proc.as_deref().unwrap_or("fiscal-bpe 0.1.0")),
        ),
        tag("tpBPe", &[], TagContent::Text(&ide.tp_bpe)),
        tag("indPres", &[], TagContent::Text(&ide.ind_pres)),
        tag("UFIni", &[], TagContent::Text(&ide.uf_ini)),
        tag("cMunIni", &[], TagContent::Text(&ide.c_mun_ini)),
        tag("UFFim", &[], TagContent::Text(&ide.uf_fim)),
        tag("cMunFim", &[], TagContent::Text(&ide.c_mun_fim)),
    ];
    if let Some(v) = &ide.dh_cont {
        c.push(tag("dhCont", &[], TagContent::Text(v)));
    }
    if let Some(v) = &ide.x_just {
        c.push(tag("xJust", &[], TagContent::Text(v)));
    }
    tag("ide", &[], TagContent::Children(c))
}

fn build_emit_bpe(e: &BpeEmit) -> String {
    let mut c = vec![
        tag("CNPJ", &[], TagContent::Text(&e.cnpj)),
        tag("IE", &[], TagContent::Text(&e.ie)),
    ];
    if let Some(v) = &e.iest {
        c.push(tag("IEST", &[], TagContent::Text(v)));
    }
    c.push(tag("xNome", &[], TagContent::Text(&e.x_nome)));
    if let Some(v) = &e.x_fant {
        c.push(tag("xFant", &[], TagContent::Text(v)));
    }
    if let Some(v) = &e.im {
        c.push(tag("IM", &[], TagContent::Text(v)));
    }
    c.push(tag("CRT", &[], TagContent::Text(&e.crt)));
    c.push(build_endereco("enderEmit", &e.ender_emit, true));
    if let Some(v) = &e.tar {
        c.push(tag("TAR", &[], TagContent::Text(v)));
    }
    tag("emit", &[], TagContent::Children(c))
}

fn build_comp(c: &Comprador) -> String {
    let mut ch = vec![
        tag("xNome", &[], TagContent::Text(&c.x_nome)),
        build_documento(&c.doc),
    ];
    if let Some(v) = &c.ie {
        ch.push(tag("IE", &[], TagContent::Text(v)));
    }
    if let Some(e) = &c.ender_comp {
        ch.push(build_endereco("enderComp", e, false));
    }
    tag("comp", &[], TagContent::Children(ch))
}

fn build_inf_valor(v: &InfValorBpe) -> String {
    let mut c = vec![
        tag("vBP", &[], TagContent::Text(&v.v_bp)),
        tag("vDesconto", &[], TagContent::Text(&v.v_desconto)),
        tag("vPgto", &[], TagContent::Text(&v.v_pgto)),
    ];
    c.push(tag("vTroco", &[], TagContent::Text(&v.v_troco)));
    for comp in &v.comp {
        c.push(tag(
            "Comp",
            &[],
            TagContent::Children(vec![
                tag("tpComp", &[], TagContent::Text(&comp.tp_comp)),
                tag("vComp", &[], TagContent::Text(&comp.v_comp)),
            ]),
        ));
    }
    tag("infValorBPe", &[], TagContent::Children(c))
}

fn build_inf_viagem(v: &InfViagem) -> String {
    let mut c = vec![
        tag("cPercurso", &[], TagContent::Text(&v.c_percurso)),
        tag("xPercurso", &[], TagContent::Text(&v.x_percurso)),
    ];
    c.push(tag("tpViagem", &[], TagContent::Text(&v.tp_viagem)));
    c.push(tag("tpServ", &[], TagContent::Text(&v.tp_serv)));
    c.push(tag("tpAcomodacao", &[], TagContent::Text(&v.tp_acomodacao)));
    if let Some(t) = &v.tp_trecho {
        c.push(tag("tpTrecho", &[], TagContent::Text(t)));
    }
    c.push(tag("dhViagem", &[], TagContent::Text(&v.dh_viagem)));
    if let Some(p) = &v.prefixo {
        c.push(tag("prefixo", &[], TagContent::Text(p)));
    }
    if let Some(p) = &v.poltrona {
        c.push(tag("poltrona", &[], TagContent::Text(p)));
    }
    if let Some(p) = &v.plataforma {
        c.push(tag("plataforma", &[], TagContent::Text(p)));
    }
    tag("infViagem", &[], TagContent::Children(c))
}

fn build_inf_passagem(p: &InfPassagem) -> String {
    let mut c = vec![
        tag("cLocOrig", &[], TagContent::Text(&p.c_loc_orig)),
        tag("xLocOrig", &[], TagContent::Text(&p.x_loc_orig)),
        tag("cLocDest", &[], TagContent::Text(&p.c_loc_dest)),
        tag("xLocDest", &[], TagContent::Text(&p.x_loc_dest)),
    ];
    c.push(tag("dhEmb", &[], TagContent::Text(&p.dh_emb)));
    c.push(tag("dhValidade", &[], TagContent::Text(&p.dh_validade)));
    if let Some(pa) = &p.passageiro {
        let mut pc = vec![tag("xNome", &[], TagContent::Text(&pa.x_nome))];
        if let Some(v) = &pa.cpf {
            pc.push(tag("CPF", &[], TagContent::Text(v)));
        }
        if let Some(v) = &pa.tp_doc {
            pc.push(tag("tpDoc", &[], TagContent::Text(v)));
        }
        if let Some(v) = &pa.n_doc {
            pc.push(tag("nDoc", &[], TagContent::Text(v)));
        }
        if let Some(v) = &pa.fone {
            pc.push(tag("fone", &[], TagContent::Text(v)));
        }
        c.push(tag("infPassageiro", &[], TagContent::Children(pc)));
    }
    tag("infPassagem", &[], TagContent::Children(c))
}

fn build_bpe_imp(imp: &BpeImp) -> String {
    let mut c = vec![build_icms(&imp.icms)];
    if let Some(v) = &imp.v_tot_trib {
        c.push(tag("vTotTrib", &[], TagContent::Text(v)));
    }
    tag("imp", &[], TagContent::Children(c))
}

fn build_pag(p: &Pagamento) -> String {
    let mut c = vec![tag("tPag", &[], TagContent::Text(&p.t_pag))];
    if let Some(v) = &p.x_pag {
        c.push(tag("xPag", &[], TagContent::Text(v)));
    }
    c.push(tag("vPag", &[], TagContent::Text(&p.v_pag)));
    tag("pag", &[], TagContent::Children(c))
}
