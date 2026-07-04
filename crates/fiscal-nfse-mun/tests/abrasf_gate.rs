//! Gate: monta um GerarNfseEnvio ABRASF 2.03 e valida contra o XSD oficial
//! (extraído do WSDL DSF).

use fiscal_nfse_mun::abrasf::build_gerar_nfse;
use fiscal_nfse_mun::model::*;

fn sample() -> EmitInput {
    EmitInput {
        emitente: Emitente {
            cnpj: "18885949000181".into(),
            im: Some("123456".into()),
            razao_social: "CENTRE SOLUCOES LTDA".into(),
            c_mun: "3552205".into(),
            uf: "SP".into(),
            endereco: None,
            optante_simples: true,
        },
        rps: Rps {
            numero: 1,
            serie: "1".into(),
            tipo: 1,
            data_emissao: "2026-06-06T10:00:00-03:00".into(),
            tomador: Tomador {
                doc: Some("34493536837".into()),
                razao_social: Some("FULANO DE TAL".into()),
                email: Some("fulano@ex.com".into()),
                endereco: None,
                im: None,
            },
            servico: Servico {
                valor_centavos: 10000,
                valor_deducoes_centavos: 0,
                aliquota_iss: Some("2.00".into()),
                iss_retido: false,
                item_lista_servico: "1.01".into(),
                cod_tributacao_municipio: None,
                cnae: Some("6201500".into()),
                discriminacao: "SERVICO DE TESTE DFEHUB".into(),
                c_mun_prestacao: None,
                nbs: None,
                c_class_trib: None,
                c_ind_op: None,
            },
            natureza_operacao: None,
            regime_especial_tributacao: None,
            incentivador_cultural: false,
            intermediario: None,
        },
    }
}

#[test]
fn gerar_nfse_valida_no_xsd_abrasf() {
    let xml = build_gerar_nfse(&sample()).expect("build");
    if let Err(errs) = fiscal_xsd::schemas::abrasf_gerar_nfse().validate(&xml) {
        panic!(
            "GerarNfseEnvio falhou no XSD ABRASF 2.03:\n{}",
            errs.join("\n")
        );
    }
}

fn test_pfx() -> Vec<u8> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../..",
        "/tests/fixtures/certs/novo_cert_cnpj_06157250000116_senha_minhasenha.pfx"
    ))
    .expect("pfx")
}

#[test]
fn gerar_nfse_assinado_valida_no_xsd() {
    let xml = build_gerar_nfse(&sample()).expect("build");
    let cert =
        fiscal_crypto::certificate::load_certificate(&test_pfx(), "minhasenha").expect("cert");
    let signed =
        fiscal_crypto::certificate::sign_abrasf_xml(&xml, &cert.private_key, &cert.certificate)
            .expect("sign");
    assert!(signed.contains("<Signature"), "Signature ausente");
    // Signature é minOccurs=0 no schema → o doc assinado também valida.
    if let Err(errs) = fiscal_xsd::schemas::abrasf_gerar_nfse().validate(&signed) {
        panic!(
            "GerarNfseEnvio assinado falhou no XSD:\n{}",
            errs.join("\n")
        );
    }
}
