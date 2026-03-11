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
    IssuerData::new(
        "12345678000199", "123456789", "Test Company",
        TaxRegime::SimplesNacional, "SP",
        IbgeCode("3550308".to_string()), "Sao Paulo",
        "Av Paulista", "1000", "Bela Vista", "01310100",
    ).trade_name("Test")
}

/// Sample issuer with Regime Normal tax regime.
pub fn sample_issuer_normal() -> IssuerData {
    IssuerData::new(
        "58716523000119", "111222333444", "Empresa Teste",
        TaxRegime::Normal, "SP",
        IbgeCode("3550308".to_string()), "Sao Paulo",
        "Rua Teste", "100", "Centro", "01001000",
    )
}

/// Sample recipient (CPF, individual).
pub fn sample_recipient() -> RecipientData {
    RecipientData::new("12345678901", "John Doe")
}

/// Sample invoice item with minimal ICMS/PIS/COFINS configuration.
pub fn sample_item() -> InvoiceItemData {
    InvoiceItemData::new(
        1, "1", "Product A", "84715010", "5102", "UN",
        2.0, Cents(1000), Cents(2000),
        "102", Rate(0), Cents(0), "99", "99",
    )
}

/// Sample payment (cash, R$20.00).
pub fn sample_payment() -> PaymentData {
    PaymentData::new("01", Cents(2000))
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
