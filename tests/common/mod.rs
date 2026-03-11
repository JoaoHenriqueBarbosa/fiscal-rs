// Shared test helpers and fixtures used by multiple test files.
//
// Usage: add `mod common;` at the top of any integration test file.
// Note: unused-function warnings are expected since each test binary uses a subset.

#![allow(dead_code)]

use fiscal::types::*;
use fiscal::newtypes::{Cents, Rate, IbgeCode};
use fiscal::xml_builder::InvoiceBuilder;

// ── XML assertion helpers ───────────────────────────────────────────────────

/// Assert an XML string contains all expected `<tag>value</tag>` pairs.
pub fn expect_xml_tag_values(xml: &str, expectations: &[(&str, &str)]) {
    for (tag_name, value) in expectations {
        let expected = format!("<{tag_name}>{value}</{tag_name}>");
        assert!(
            xml.contains(&expected),
            "Expected XML to contain {expected}\n\nActual XML:\n{xml}"
        );
    }
}

/// Assert an XML string contains every given substring.
pub fn expect_xml_contains(xml: &str, substrings: &[&str]) {
    for s in substrings {
        assert!(
            xml.contains(s),
            "Expected XML to contain {s:?}\n\nActual XML:\n{xml}"
        );
    }
}

/// Assert an XML string does NOT contain any of the given substrings.
pub fn expect_xml_not_contains(xml: &str, substrings: &[&str]) {
    for s in substrings {
        assert!(
            !xml.contains(s),
            "Expected XML to NOT contain {s:?}\n\nActual XML:\n{xml}"
        );
    }
}

/// Assert an XML string contains the open and close of a wrapper tag.
pub fn expect_wrapped_in(xml: &str, wrapper: &str) {
    let open = format!("<{wrapper}>");
    let close = format!("</{wrapper}>");
    assert!(xml.contains(&open), "Expected XML to contain {open}\n\nXML:\n{xml}");
    assert!(xml.contains(&close), "Expected XML to contain {close}\n\nXML:\n{xml}");
}

// ── Sample data factories ───────────────────────────────────────────────────

/// Standard Brazilian timezone offset (UTC-3).
pub fn br_offset() -> chrono::FixedOffset {
    chrono::FixedOffset::west_opt(3 * 3600).unwrap()
}

/// Sample issuer with Simples Nacional tax regime.
pub fn sample_issuer() -> IssuerData {
    IssuerData {
        tax_id: "12345678000199".to_string(),
        state_tax_id: "123456789".to_string(),
        company_name: "Test Company".to_string(),
        trade_name: Some("Test".to_string()),
        tax_regime: TaxRegime::SimplesNacional,
        state_code: "SP".to_string(),
        city_code: IbgeCode("3550308".to_string()),
        city_name: "Sao Paulo".to_string(),
        street: "Av Paulista".to_string(),
        street_number: "1000".to_string(),
        district: "Bela Vista".to_string(),
        zip_code: "01310100".to_string(),
        address_complement: None,
    }
}

/// Sample issuer with Regime Normal tax regime.
pub fn sample_issuer_normal() -> IssuerData {
    IssuerData {
        tax_id: "58716523000119".to_string(),
        state_tax_id: "111222333444".to_string(),
        company_name: "Empresa Teste".to_string(),
        trade_name: None,
        tax_regime: TaxRegime::Normal,
        state_code: "SP".to_string(),
        city_code: IbgeCode("3550308".to_string()),
        city_name: "Sao Paulo".to_string(),
        street: "Rua Teste".to_string(),
        street_number: "100".to_string(),
        district: "Centro".to_string(),
        zip_code: "01001000".to_string(),
        address_complement: None,
    }
}

/// Sample recipient (CPF, individual).
pub fn sample_recipient() -> RecipientData {
    RecipientData {
        tax_id: "12345678901".to_string(),
        name: "John Doe".to_string(),
        state_code: None,
        state_tax_id: None,
        street: None,
        street_number: None,
        district: None,
        city_code: None,
        city_name: None,
        zip_code: None,
        complement: None,
    }
}

/// Sample invoice item with minimal ICMS/PIS/COFINS configuration.
pub fn sample_item() -> InvoiceItemData {
    InvoiceItemData {
        item_number: 1,
        product_code: "1".to_string(),
        description: "Product A".to_string(),
        ncm: "84715010".to_string(),
        cfop: "5102".to_string(),
        unit_of_measure: "UN".to_string(),
        quantity: 2.0,
        unit_price: Cents(1000),
        total_price: Cents(2000),
        c_ean: None,
        c_ean_trib: None,
        cest: None,
        v_frete: None,
        v_seg: None,
        v_desc: None,
        v_outro: None,
        orig: None,
        icms_cst: "102".to_string(),
        icms_rate: Rate(0),
        icms_amount: Cents(0),
        icms_mod_bc: None,
        icms_red_bc: None,
        icms_mod_bc_st: None,
        icms_p_mva_st: None,
        icms_red_bc_st: None,
        icms_v_bc_st: None,
        icms_p_icms_st: None,
        icms_v_icms_st: None,
        icms_v_icms_deson: None,
        icms_mot_des_icms: None,
        icms_p_fcp: None,
        icms_v_fcp: None,
        icms_v_bc_fcp: None,
        icms_p_fcp_st: None,
        icms_v_fcp_st: None,
        icms_v_bc_fcp_st: None,
        icms_p_cred_sn: None,
        icms_v_cred_icms_sn: None,
        icms_v_icms_substituto: None,
        pis_cst: "99".to_string(),
        pis_v_bc: None,
        pis_p_pis: None,
        pis_v_pis: None,
        pis_q_bc_prod: None,
        pis_v_aliq_prod: None,
        cofins_cst: "99".to_string(),
        cofins_v_bc: None,
        cofins_p_cofins: None,
        cofins_v_cofins: None,
        cofins_q_bc_prod: None,
        cofins_v_aliq_prod: None,
        ipi_cst: None,
        ipi_c_enq: None,
        ipi_v_bc: None,
        ipi_p_ipi: None,
        ipi_v_ipi: None,
        ipi_q_unid: None,
        ipi_v_unid: None,
        ii_v_bc: None,
        ii_v_desp_adu: None,
        ii_v_ii: None,
        ii_v_iof: None,
        rastro: None,
        veic_prod: None,
        med: None,
        arma: None,
        n_recopi: None,
        inf_ad_prod: None,
        obs_item: None,
        dfe_referenciado: None,
    }
}

/// Sample payment (cash, R$20.00).
pub fn sample_payment() -> PaymentData {
    PaymentData {
        method: "01".to_string(),
        amount: Cents(2000),
    }
}

/// Build a sample NFC-e InvoiceBuilder in homologation (Draft state).
///
/// Call `.build().unwrap()` to get the Built state with XML.
pub fn sample_invoice_builder() -> InvoiceBuilder {
    let offset = br_offset();
    let issued_at = chrono::NaiveDate::from_ymd_opt(2026, 1, 15)
        .unwrap()
        .and_hms_opt(10, 30, 0)
        .unwrap()
        .and_local_timezone(offset)
        .unwrap();

    InvoiceBuilder::new(sample_issuer(), SefazEnvironment::Homologation, InvoiceModel::Nfce)
        .series(1)
        .invoice_number(1)
        .issued_at(issued_at)
        .add_item(sample_item())
        .payments(vec![sample_payment()])
}

/// Fixtures directory path for sped-nfe PHP reference files.
pub const FIXTURES_PATH: &str =
    "/home/john/projects/FinOpenPOS/.reference/sped-nfe/tests/fixtures/";
