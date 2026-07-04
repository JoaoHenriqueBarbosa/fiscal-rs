//! End-to-end gate: build a CT-e Normal, sign it, and validate the signed
//! document against the **official CT-e 4.00 XSD** (via `fiscal-xsd`).
//!
//! This is the real proof that the `builder.rs` block ordering matches the
//! schema — a misplaced element fails XSD validation here, before any
//! transmission. The certificate CNPJ need not match `emit/CNPJ`: XSD checks
//! structure only, not business rules.

mod common;

use fiscal_crypto::certificate::load_certificate;
use fiscal_cte::{build_cte_xml, sign_cte_xml};

fn test_pfx() -> Vec<u8> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../..",
        "/tests/fixtures/certs/novo_cert_cnpj_06157250000116_senha_minhasenha.pfx"
    );
    std::fs::read(path).expect("test PFX not found")
}

fn gate(data: &fiscal_cte::CteBuildData, ctx: &str) {
    let xml = build_cte_xml(data).unwrap();
    let cert = load_certificate(&test_pfx(), "minhasenha").expect("load cert");
    let signed = sign_cte_xml(&xml, &cert.private_key, &cert.certificate).expect("sign");
    if let Err(errs) = fiscal_xsd::schemas::cte().validate(&signed) {
        panic!("{ctx} falhou no XSD oficial 4.00:\n{}", errs.join("\n"));
    }
}

#[test]
fn signed_cte_validates_against_official_xsd() {
    gate(&common::sample_cte(), "CT-e Normal");
}

#[test]
fn cte_complementar_validates() {
    // tpCTe 1: emite infCteComp{chCTe} no lugar de infCTeNorm.
    let mut d = common::sample_cte();
    d.ide.tp_cte = "1".into();
    d.inf_cte_comp = vec!["3".repeat(44)];
    gate(&d, "CT-e Complementar");
}

#[test]
fn cte_substituto_validates() {
    // tpCTe 3: infCTeNorm com infCteSub{chCte, indAlteraToma}.
    let mut d = common::sample_cte();
    d.ide.tp_cte = "3".into();
    d.inf_cte_norm.inf_cte_sub = Some(fiscal_cte::types::InfCteSub {
        ch_cte: "3".repeat(44),
        ind_altera_toma: Some("1".into()),
    });
    gate(&d, "CT-e Substituto");
}
