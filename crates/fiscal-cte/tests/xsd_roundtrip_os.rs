//! Gate: build a CT-e OS (model 67), sign it, and validate the signed document
//! against the **official cteOS_v4.00 XSD** (via `fiscal-xsd`). A misplaced
//! element fails here, before transmission.

mod common;

use fiscal_crypto::certificate::load_certificate;
use fiscal_cte::{build_cteos_xml, sign_cteos_xml};

fn test_pfx() -> Vec<u8> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../..",
        "/tests/fixtures/certs/novo_cert_cnpj_06157250000116_senha_minhasenha.pfx"
    );
    std::fs::read(path).expect("test PFX not found")
}

#[test]
fn signed_cteos_validates_against_official_xsd() {
    let xml = build_cteos_xml(&common::sample_cteos()).unwrap();

    let cert = load_certificate(&test_pfx(), "minhasenha").expect("load cert");
    let signed = sign_cteos_xml(&xml, &cert.private_key, &cert.certificate).expect("sign");

    if let Err(errs) = fiscal_xsd::schemas::cteos().validate(&signed) {
        panic!(
            "CT-e OS assinado falhou no XSD oficial 4.00:\n{}",
            errs.join("\n")
        );
    }
}
