//! Gate: monta o PedidoEnvioLoteRPS (SP), assina RPS + lote, valida no XSD oficial SP v01.

use fiscal_nfse_mun::model::*;
use fiscal_nfse_mun::saopaulo::{SP_LOTE_ROOT, assinatura_string, build_lote_rps};

fn test_pfx() -> Vec<u8> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../..",
        "/tests/fixtures/certs/novo_cert_cnpj_06157250000116_senha_minhasenha.pfx"
    ))
    .expect("pfx")
}

fn sample() -> EmitInput {
    EmitInput {
        emitente: Emitente {
            cnpj: "06157250000116".into(),
            im: Some("12345678".into()),
            razao_social: "CENTRE LTDA".into(),
            c_mun: "3550308".into(),
            uf: "SP".into(),
            endereco: None,
            optante_simples: false,
        },
        rps: Rps {
            numero: 1,
            serie: "1".into(),
            tipo: 1,
            data_emissao: "2026-06-06T10:00:00-03:00".into(),
            tomador: Tomador {
                doc: Some("11222333000181".into()),
                razao_social: Some("TOMADOR LTDA".into()),
                email: None,
                endereco: None,
                im: None,
            },
            servico: Servico {
                valor_centavos: 10000,
                valor_deducoes_centavos: 0,
                aliquota_iss: Some("2.00".into()),
                iss_retido: false,
                item_lista_servico: "1.01".into(),
                cod_tributacao_municipio: Some("02916".into()),
                cnae: None,
                discriminacao: "SERVICO DE TESTE".into(),
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
fn lote_assinado_valida_no_xsd_sp() {
    let cert =
        fiscal_crypto::certificate::load_certificate(&test_pfx(), "minhasenha").expect("cert");
    let assinatura = fiscal_crypto::certificate::rsa_sha1_base64(
        assinatura_string(&sample()).as_bytes(),
        &cert.private_key,
    )
    .expect("ass rps");
    let lote = build_lote_rps(&sample(), &assinatura);
    let signed = fiscal_crypto::certificate::sign_sp_lote_xml(
        &lote,
        SP_LOTE_ROOT,
        &cert.private_key,
        &cert.certificate,
    )
    .expect("sign lote");
    if let Err(errs) = fiscal_xsd::schemas::sp_lote_rps().validate(&signed) {
        panic!(
            "PedidoEnvioLoteRPS falhou no XSD SP v01:\n{}",
            errs.join("\n")
        );
    }
}

#[test]
fn lote_v2_assinado_valida_no_xsd_sp() {
    use fiscal_nfse_mun::saopaulo::{assinatura_string_v2, build_lote_rps_v2};
    let cert =
        fiscal_crypto::certificate::load_certificate(&test_pfx(), "minhasenha").expect("cert");
    let assinatura = fiscal_crypto::certificate::rsa_sha1_base64(
        assinatura_string_v2(&sample()).as_bytes(),
        &cert.private_key,
    )
    .expect("ass rps");
    let lote = build_lote_rps_v2(&sample(), &assinatura);
    let signed = fiscal_crypto::certificate::sign_sp_lote_xml(
        &lote,
        SP_LOTE_ROOT,
        &cert.private_key,
        &cert.certificate,
    )
    .expect("sign lote");
    if let Err(errs) = fiscal_xsd::schemas::sp_lote_rps_v2().validate(&signed) {
        panic!(
            "PedidoEnvioLoteRPS v2 falhou no XSD SP v02:\n{}",
            errs.join("\n")
        );
    }
}

/// v2 com intermediário: a assinatura anexa a cauda dele, então o RPS tem que
/// carregar os campos (senão 1206), e o lote continua válido no XSD v02.
#[test]
fn lote_v2_com_intermediario_valida_no_xsd_sp() {
    use fiscal_nfse_mun::saopaulo::{assinatura_string_v2, build_lote_rps_v2};
    let mut inp = sample();
    inp.rps.intermediario = Some(Intermediario {
        doc: "09999999000106".into(),
        im: Some("99999999".into()),
        iss_retido: true,
    });
    assert!(assinatura_string_v2(&inp).ends_with("209999999000106S"));
    let cert =
        fiscal_crypto::certificate::load_certificate(&test_pfx(), "minhasenha").expect("cert");
    let assinatura = fiscal_crypto::certificate::rsa_sha1_base64(
        assinatura_string_v2(&inp).as_bytes(),
        &cert.private_key,
    )
    .expect("ass rps");
    let lote = build_lote_rps_v2(&inp, &assinatura);
    assert!(lote.contains(
        "<CPFCNPJIntermediario><CNPJ>09999999000106</CNPJ></CPFCNPJIntermediario>\
         <InscricaoMunicipalIntermediario>99999999</InscricaoMunicipalIntermediario>\
         <ISSRetidoIntermediario>true</ISSRetidoIntermediario>"
    ));
    assert!(lote.contains("<ValorFinalCobrado>"));
    assert!(!lote.contains("<ValorInicialCobrado>"));
    let signed = fiscal_crypto::certificate::sign_sp_lote_xml(
        &lote,
        SP_LOTE_ROOT,
        &cert.private_key,
        &cert.certificate,
    )
    .expect("sign lote");
    if let Err(errs) = fiscal_xsd::schemas::sp_lote_rps_v2().validate(&signed) {
        panic!(
            "PedidoEnvioLoteRPS v2 c/ intermediario falhou no XSD SP v02:\n{}",
            errs.join("\n")
        );
    }
}
