//! String-based XML builder for the GTV-e (model 64), leiaute 4.00.
//!
//! Block order (per `GTVe_v4.00.xsd` / `TGTVe` in `cteTiposBasico`):
//!
//! ```text
//! GTVe > infCte[@Id] >
//!   ide{..., toma}, compl?, emit, rem, dest, origem?, destino?,
//!   detGTV{ infEspecie+, qCarga, infVeiculo+ }, autXML*, infRespTec?
//! ```

use fiscal_core::FiscalError;
use fiscal_core::xml_builder::access_key::{format_year_month, generate_numeric_code};
use fiscal_core::xml_utils::{TagContent, tag};

use crate::access_key::{CteAccessKeyParams, build_cte_access_key};
use crate::builder::{
    build_aut_xml, build_compl, build_documento, build_endereco, build_inf_resp_tec, build_party,
    format_datetime_cte,
};
use crate::types_gtve::*;
use crate::{CTE_NAMESPACE, CTE_VERSION, CTEGTVE_MODEL};

/// Build a complete unsigned `<GTVe>` XML document from [`GtveBuildData`].
///
/// # Errors
///
/// [`FiscalError::XmlGeneration`] if the access key cannot be built.
pub fn build_gtve_xml(data: &GtveBuildData) -> Result<String, FiscalError> {
    let generated;
    let cct = match data.numeric_code.as_deref() {
        Some(c) => c,
        None => {
            generated = generate_numeric_code();
            &generated
        }
    };
    let access_key = build_cte_access_key(&CteAccessKeyParams {
        model: CTEGTVE_MODEL,
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

    let mut children = vec![build_ide_gtve(&data.ide, c_ct, c_dv)];
    if let Some(compl) = &data.compl {
        children.push(build_compl(compl));
    }
    children.push(build_emit_gtve(&data.emit));
    children.push(build_party("rem", "enderReme", &data.rem, false, false));
    children.push(build_party("dest", "enderDest", &data.dest, false, true));
    if let Some(o) = &data.origem {
        children.push(build_endereco("origem", o, true));
    }
    if let Some(d) = &data.destino {
        children.push(build_endereco("destino", d, true));
    }
    children.push(build_det_gtv(&data.det_gtv));
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
        "GTVe",
        &[("xmlns", CTE_NAMESPACE), ("versao", CTE_VERSION)],
        TagContent::Children(vec![inf_cte]),
    ))
}

fn build_ide_gtve(ide: &IdeGtve, c_ct: &str, c_dv: &str) -> String {
    let serie = ide.serie.to_string();
    let n_ct = ide.n_ct.to_string();
    let dh_emi = format_datetime_cte(&ide.dh_emi, &ide.uf_env);

    let mut c = vec![
        tag("cUF", &[], TagContent::Text(&ide.c_uf)),
        tag("cCT", &[], TagContent::Text(c_ct)),
        tag("CFOP", &[], TagContent::Text(&ide.cfop)),
        tag("natOp", &[], TagContent::Text(&ide.nat_op)),
        tag("mod", &[], TagContent::Text(CTEGTVE_MODEL)),
        tag("serie", &[], TagContent::Text(&serie)),
        tag("nCT", &[], TagContent::Text(&n_ct)),
        tag("dhEmi", &[], TagContent::Text(&dh_emi)),
        tag("tpImp", &[], TagContent::Text(&ide.tp_imp)),
        tag("tpEmis", &[], TagContent::Text(&ide.tp_emis)),
        tag("cDV", &[], TagContent::Text(c_dv)),
        tag("tpAmb", &[], TagContent::Text(&ide.tp_amb)),
        tag("tpCTe", &[], TagContent::Text(&ide.tp_cte)),
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
    ];
    c.push(tag(
        "dhSaidaOrig",
        &[],
        TagContent::Text(&ide.dh_saida_orig),
    ));
    c.push(tag(
        "dhChegadaDest",
        &[],
        TagContent::Text(&ide.dh_chegada_dest),
    ));
    c.push(build_toma_gtve(&ide.toma));
    tag("ide", &[], TagContent::Children(c))
}

/// Tomador da GTV-e como `<tomaTerceiro>` (toma=4, tomador detalhado) — irmão de
/// `<toma>` no ide. Ordem: toma, CNPJ/CPF, IE?, xNome, xFant?, fone?, enderToma,
/// email?.
fn build_toma_gtve(t: &crate::types_os::TomaOs) -> String {
    let mut tc = vec![
        tag("toma", &[], TagContent::Text("4")),
        build_documento(&t.doc),
    ];
    if let Some(ie) = &t.ie {
        tc.push(tag("IE", &[], TagContent::Text(ie)));
    }
    tc.push(tag("xNome", &[], TagContent::Text(&t.x_nome)));
    if let Some(xf) = &t.x_fant {
        tc.push(tag("xFant", &[], TagContent::Text(xf)));
    }
    if let Some(f) = &t.fone {
        tc.push(tag("fone", &[], TagContent::Text(f)));
    }
    tc.push(build_endereco("enderToma", &t.ender_toma, false));
    if let Some(e) = &t.email {
        tc.push(tag("email", &[], TagContent::Text(e)));
    }
    tag("tomaTerceiro", &[], TagContent::Children(tc))
}

/// `<emit>` da GTV-e — igual ao CT-e mas **sem `<CRT>`** (TGTVe não o possui).
fn build_emit_gtve(e: &crate::types::Emit) -> String {
    let mut c = vec![build_documento(&e.doc)];
    if let Some(ie) = &e.ie {
        c.push(tag("IE", &[], TagContent::Text(ie)));
    }
    if let Some(iest) = &e.iest {
        c.push(tag("IEST", &[], TagContent::Text(iest)));
    }
    c.push(tag("xNome", &[], TagContent::Text(&e.x_nome)));
    if let Some(xf) = &e.x_fant {
        c.push(tag("xFant", &[], TagContent::Text(xf)));
    }
    c.push(build_endereco("enderEmit", &e.ender_emit, true));
    tag("emit", &[], TagContent::Children(c))
}

fn build_det_gtv(d: &DetGtv) -> String {
    let mut c = Vec::new();
    for e in &d.inf_especie {
        let mut ec = vec![
            tag("tpEspecie", &[], TagContent::Text(&e.tp_especie)),
            tag("vEspecie", &[], TagContent::Text(&e.v_especie)),
        ];
        if let Some(v) = &e.tp_numerario {
            ec.push(tag("tpNumerario", &[], TagContent::Text(v)));
        }
        if let Some(v) = &e.x_moeda_estr {
            ec.push(tag("xMoedaEstr", &[], TagContent::Text(v)));
        }
        c.push(tag("infEspecie", &[], TagContent::Children(ec)));
    }
    c.push(tag("qCarga", &[], TagContent::Text(&d.q_carga)));
    for v in &d.inf_veiculo {
        let mut vc = vec![
            tag("placa", &[], TagContent::Text(&v.placa)),
            tag("UF", &[], TagContent::Text(&v.uf)),
        ];
        if let Some(r) = &v.rntrc {
            vc.push(tag("RNTRC", &[], TagContent::Text(r)));
        }
        c.push(tag("infVeiculo", &[], TagContent::Children(vc)));
    }
    tag("detGTV", &[], TagContent::Children(c))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Documento, Emit, Endereco, Party};
    use crate::types_os::TomaOs;

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

    fn party(nome: &str) -> Party {
        Party {
            doc: Documento::Cnpj("11222333000181".into()),
            ie: Some("111111111".into()),
            x_nome: nome.into(),
            x_fant: None,
            fone: None,
            isuf: None,
            ender: ender(),
            email: None,
        }
    }

    #[test]
    fn builds_well_formed_gtve() {
        let data = GtveBuildData {
            numeric_code: Some("00000001".into()),
            emit_cnpj: "12345678000190".into(),
            ide: IdeGtve {
                c_uf: "35".into(),
                cfop: "5359".into(),
                nat_op: "TRANSPORTE DE VALORES".into(),
                serie: 1,
                n_ct: 1,
                dh_emi: "2026-06-06T10:00:00-03:00".parse().unwrap(),
                tp_imp: "1".into(),
                tp_emis: "1".into(),
                tp_amb: "2".into(),
                tp_cte: "4".into(),
                ver_proc: None,
                c_mun_env: "3550308".into(),
                x_mun_env: "SAO PAULO".into(),
                uf_env: "SP".into(),
                modal: "01".into(),
                tp_serv: "9".into(),
                ind_ie_toma: "9".into(),
                dh_saida_orig: "2026-06-06T09:00:00-03:00".into(),
                dh_chegada_dest: "2026-06-06T18:00:00-03:00".into(),
                toma: TomaOs {
                    doc: Documento::Cpf("34493536837".into()),
                    ie: None,
                    x_nome: "FULANO".into(),
                    x_fant: None,
                    fone: None,
                    ender_toma: ender(),
                    email: None,
                },
            },
            compl: None,
            emit: Emit {
                doc: Documento::Cnpj("12345678000190".into()),
                ie: Some("111111111111".into()),
                iest: None,
                x_nome: "TRANSP VALORES".into(),
                x_fant: None,
                ender_emit: ender(),
                crt: "3".into(),
            },
            rem: party("REMETENTE"),
            dest: party("DESTINATARIO"),
            origem: Some(ender()),
            destino: Some(ender()),
            det_gtv: DetGtv {
                inf_especie: vec![InfEspecie {
                    tp_especie: "1".into(),
                    v_especie: "1000.00".into(),
                    tp_numerario: Some("1".into()),
                    x_moeda_estr: None,
                }],
                q_carga: "1000.0000".into(),
                inf_veiculo: vec![InfVeiculoGtv {
                    placa: "ABC1234".into(),
                    uf: "SP".into(),
                    rntrc: Some("12345678".into()),
                }],
            },
            aut_xml: vec![],
            inf_resp_tec: None,
        };
        let xml = build_gtve_xml(&data).unwrap();
        assert!(xml.starts_with("<GTVe"));
        assert!(xml.contains("<mod>64</mod>"));
        assert!(xml.contains("<detGTV><infEspecie><tpEspecie>1</tpEspecie>"));
        assert!(xml.contains("<infVeiculo><placa>ABC1234</placa>"));
        let id = xml
            .split("Id=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        assert_eq!(&id[3..][20..22], "64");
    }
}
