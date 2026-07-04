//! Gate: build a GTV-e (model 64), sign it, validate against the official
//! GTVe_v4.00 XSD via fiscal-xsd.
mod common;
use fiscal_crypto::certificate::load_certificate;
use fiscal_cte::{build_gtve_xml, sign_gtve_xml};

fn test_pfx() -> Vec<u8> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../..",
        "/tests/fixtures/certs/novo_cert_cnpj_06157250000116_senha_minhasenha.pfx"
    ))
    .expect("pfx")
}

#[test]
fn signed_gtve_validates_against_official_xsd() {
    let xml = build_gtve_xml(&common::sample_gtve()).unwrap();
    let cert = load_certificate(&test_pfx(), "minhasenha").expect("cert");
    let signed = sign_gtve_xml(&xml, &cert.private_key, &cert.certificate).expect("sign");
    if let Err(errs) = fiscal_xsd::schemas::gtve().validate(&signed) {
        panic!("GTV-e falhou no XSD oficial:\n{}", errs.join("\n"));
    }
}
