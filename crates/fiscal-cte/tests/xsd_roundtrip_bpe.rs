//! Gate: build a BP-e (model 63), sign it, validate against the official
//! bpe_v1.00 XSD via fiscal-xsd.
use fiscal_crypto::certificate::load_certificate;
use fiscal_cte::types::{Endereco, Icms};
use fiscal_cte::types_bpe::*;
use fiscal_cte::{build_bpe_xml, sign_bpe_xml};

fn test_pfx() -> Vec<u8> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../..",
        "/tests/fixtures/certs/novo_cert_cnpj_06157250000116_senha_minhasenha.pfx"
    ))
    .expect("pfx")
}

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

fn sample() -> BpeBuildData {
    BpeBuildData {
        numeric_code: Some("00000001".into()),
        emit_cnpj: "12345678000190".into(),
        ide: IdeBpe {
            c_uf: "35".into(),
            tp_amb: "2".into(),
            serie: 1,
            n_bp: 1,
            modal: "1".into(),
            dh_emi: "2026-06-06T10:00:00-03:00".parse().unwrap(),
            tp_emis: "1".into(),
            ver_proc: None,
            tp_bpe: "0".into(),
            ind_pres: "1".into(),
            uf_ini: "SP".into(),
            c_mun_ini: "3550308".into(),
            uf_fim: "SP".into(),
            c_mun_fim: "3509502".into(),
            dh_cont: None,
            x_just: None,
        },
        emit: BpeEmit {
            cnpj: "12345678000190".into(),
            ie: "111111111111".into(),
            iest: None,
            x_nome: "VIACAO TESTE".into(),
            x_fant: None,
            im: None,
            crt: "3".into(),
            ender_emit: ender(),
            tar: Some("12345".into()),
        },
        comp: None,
        inf_valor: InfValorBpe {
            v_bp: "100.00".into(),
            v_desconto: "0.00".into(),
            v_pgto: "100.00".into(),
            v_troco: "0.00".into(),
            comp: vec![CompBpe {
                tp_comp: "01".into(),
                v_comp: "100.00".into(),
            }],
        },
        inf_viagem: vec![InfViagem {
            c_percurso: "SP".into(),
            x_percurso: "SAO PAULO-CAMPINAS".into(),
            tp_viagem: "00".into(),
            tp_serv: "1".into(),
            tp_acomodacao: "2".into(),
            tp_trecho: Some("1".into()),
            dh_viagem: "2026-06-07T08:00:00-03:00".into(),
            prefixo: None,
            poltrona: Some("12".into()),
            plataforma: None,
        }],
        inf_passagem: InfPassagem {
            c_loc_orig: "3550308".into(),
            x_loc_orig: "SAO PAULO".into(),
            c_loc_dest: "3509502".into(),
            x_loc_dest: "CAMPINAS".into(),
            dh_emb: "2026-06-07T08:00:00-03:00".into(),
            dh_validade: "2026-06-08T08:00:00-03:00".into(),
            passageiro: None,
        },
        imp: BpeImp {
            icms: Icms::IcmsSn { ind_sn: "1".into() },
            v_tot_trib: None,
        },
        pag: vec![Pagamento {
            t_pag: "01".into(),
            x_pag: None,
            v_pag: "100.00".into(),
        }],
        aut_xml: vec![],
        inf_resp_tec: None,
    }
}

#[test]
fn signed_bpe_validates_against_official_xsd() {
    let xml = build_bpe_xml(&sample()).unwrap();
    // infBPeSupl (qrCodBPe) é exigido antes da Signature — inserido pelo hub.
    let supl = "<infBPeSupl><qrCodBPe>https://dfe-portal.svrs.rs.gov.br/bpe/qrCode?chBPe=35260612345678000190630010000000011930555651&amp;tpAmb=2</qrCodBPe></infBPeSupl>";
    let pos = xml.rfind("</BPe>").unwrap();
    let xml = format!("{}{supl}{}", &xml[..pos], &xml[pos..]);
    let cert = load_certificate(&test_pfx(), "minhasenha").expect("cert");
    let signed = sign_bpe_xml(&xml, &cert.private_key, &cert.certificate).expect("sign");
    if let Err(errs) = fiscal_xsd::schemas::bpe().validate(&signed) {
        panic!("BP-e falhou no XSD oficial:\n{}", errs.join("\n"));
    }
}
