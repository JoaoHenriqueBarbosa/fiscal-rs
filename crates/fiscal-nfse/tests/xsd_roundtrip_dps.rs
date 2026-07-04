//! Gate: build an NFS-e DPS, sign it, validate against the official DPS_v1.01
//! XSD via fiscal-xsd.
use fiscal_crypto::certificate::{load_certificate, sign_dps_xml, sign_nfse_evento_xml};
use fiscal_nfse::types::*;
use fiscal_nfse::{build_dps_xml, build_nfse_cancelamento};

fn test_pfx() -> Vec<u8> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../..",
        "/tests/fixtures/certs/novo_cert_cnpj_06157250000116_senha_minhasenha.pfx"
    ))
    .expect("pfx")
}

fn sample() -> DpsBuildData {
    DpsBuildData {
        ide: IdeDps {
            tp_amb: "2".into(),
            dh_emi: "2026-06-06T10:00:00-03:00".parse().unwrap(),
            ver_aplic: "dfehub-1.0".into(),
            serie: "1".into(),
            n_dps: 1,
            d_compet: "2026-06-06".into(),
            tp_emit: "1".into(),
            c_loc_emi: "3550308".into(),
        },
        prest: Prestador {
            doc: Documento::Cnpj("12345678000190".into()),
            im: Some("123456".into()),
            x_nome: "PRESTADOR TESTE LTDA".into(),
            end: Some(EnderNac {
                x_lgr: "RUA A".into(),
                nro: "10".into(),
                x_cpl: None,
                x_bairro: "CENTRO".into(),
                c_mun: "3550308".into(),
                cep: "01001000".into(),
            }),
            fone: None,
            email: None,
            reg_trib: RegTrib {
                op_simp_nac: "1".into(),
                reg_esp_trib: "0".into(),
            },
        },
        toma: Some(Pessoa {
            doc: Documento::Cpf("34493536837".into()),
            im: None,
            x_nome: "FULANO DE TAL".into(),
            end: None,
            fone: None,
            email: None,
        }),
        serv: Servico {
            c_loc_prestacao: "3550308".into(),
            c_trib_nac: "010101".into(),
            c_trib_mun: None,
            x_desc_serv: "SERVICO DE TESTE".into(),
        },
        valores: Valores {
            v_serv: "100.00".into(),
            trib: Trib {
                trib_mun: TribMun {
                    trib_issqn: "1".into(),
                    p_aliq: Some("5.00".into()),
                    tp_ret_issqn: "1".into(),
                },
                trib_fed: None,
            },
        },
        ibscbs: None,
    }
}

#[test]
fn signed_dps_validates_against_official_xsd() {
    let xml = build_dps_xml(&sample());
    let cert = load_certificate(&test_pfx(), "minhasenha").expect("cert");
    let signed = sign_dps_xml(&xml, &cert.private_key, &cert.certificate).expect("sign");
    if let Err(errs) = fiscal_xsd::schemas::dps().validate(&signed) {
        panic!("DPS falhou no XSD oficial 1.01:\n{}", errs.join("\n"));
    }
}

#[test]
fn signed_dps_with_ibscbs_and_tribfed_validates() {
    let mut data = sample();
    data.valores.trib.trib_fed = Some(TribFed {
        piscofins: Some(PisCofins {
            cst: "01".into(),
            v_bc: Some("100.00".into()),
            p_aliq_pis: Some("0.65".into()),
            p_aliq_cofins: Some("3.00".into()),
            v_pis: Some("0.65".into()),
            v_cofins: Some("3.00".into()),
            tp_ret: Some("0".into()),
        }),
        v_ret_cp: None,
        v_ret_irrf: Some("1.50".into()),
        v_ret_csll: Some("1.00".into()),
    });
    data.ibscbs = Some(Ibscbs {
        fin_nfse: "0".into(),
        ind_final: Some("1".into()),
        c_ind_op: "100101".into(),
        ind_dest: "0".into(),
        cst: "000".into(),
        c_class_trib: "000001".into(),
        c_cred_pres: None,
    });
    let xml = build_dps_xml(&data);
    let cert = load_certificate(&test_pfx(), "minhasenha").expect("cert");
    let signed = sign_dps_xml(&xml, &cert.private_key, &cert.certificate).expect("sign");
    if let Err(errs) = fiscal_xsd::schemas::dps().validate(&signed) {
        panic!("DPS c/ IBSCBS+tribFed falhou no XSD:\n{}", errs.join("\n"));
    }
}

#[test]
fn signed_nfse_cancelamento_validates() {
    let xml = build_nfse_cancelamento(
        &"1".repeat(50),
        "12345678000190",
        "1",
        "Erro na emissao",
        "2",
        "2026-06-06T10:00:00-03:00",
    );
    let cert = load_certificate(&test_pfx(), "minhasenha").expect("cert");
    let signed = sign_nfse_evento_xml(&xml, &cert.private_key, &cert.certificate).expect("sign");
    if let Err(errs) = fiscal_xsd::schemas::nfse_evento().validate(&signed) {
        panic!("pedRegEvento falhou no XSD:\n{}", errs.join("\n"));
    }
}
