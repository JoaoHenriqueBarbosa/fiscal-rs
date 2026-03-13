//! Tests for XML builder critical fixes:
//! 1. vNF calculation with proper formula
//! 2. vFrete/vSeg/vDesc/vOutro accumulated from items
//! 3. transp order: transporta before retTransp
//! 4. indIEDest forced to "9" for NFC-e
//! 5. Homologation xNome/xProd substitution
//! 6. indTot variable from item data
//! 7. vTotTrib in ICMSTot when > 0
//! 8. indIntermed conditional

use chrono::FixedOffset;
use fiscal::newtypes::{Cents, IbgeCode, Rate};
use fiscal::types::*;
use fiscal::xml_builder::InvoiceBuilder;

fn br_offset() -> FixedOffset {
    FixedOffset::west_opt(3 * 3600).expect("valid offset")
}

fn fixed_issued_at() -> chrono::DateTime<FixedOffset> {
    chrono::NaiveDate::from_ymd_opt(2026, 1, 15)
        .unwrap()
        .and_hms_opt(10, 30, 0)
        .unwrap()
        .and_local_timezone(br_offset())
        .unwrap()
}

fn sample_issuer() -> IssuerData {
    IssuerData::new(
        "12345678000199",
        "123456789",
        "Test Company",
        TaxRegime::SimplesNacional,
        "SP",
        IbgeCode("3550308".to_string()),
        "Sao Paulo",
        "Av Paulista",
        "1000",
        "Bela Vista",
        "01310100",
    )
    .trade_name("Test")
}

fn sample_item() -> InvoiceItemData {
    InvoiceItemData::new(
        1,
        "1",
        "Product A",
        "84715010",
        "5102",
        "UN",
        2.0,
        Cents(1000),
        Cents(2000),
        "102",
        Rate(0),
        Cents(0),
        "99",
        "99",
    )
}

fn sample_payment() -> PaymentData {
    PaymentData::new("01", Cents(2000))
}

fn nfce_builder() -> InvoiceBuilder {
    InvoiceBuilder::new(
        sample_issuer(),
        SefazEnvironment::Homologation,
        InvoiceModel::Nfce,
    )
    .series(1)
    .invoice_number(1)
    .issued_at(fixed_issued_at())
}

fn nfe_builder() -> InvoiceBuilder {
    InvoiceBuilder::new(
        sample_issuer(),
        SefazEnvironment::Homologation,
        InvoiceModel::Nfe,
    )
    .series(1)
    .invoice_number(1)
    .issued_at(fixed_issued_at())
}

fn production_nfce_builder() -> InvoiceBuilder {
    InvoiceBuilder::new(
        sample_issuer(),
        SefazEnvironment::Production,
        InvoiceModel::Nfce,
    )
    .series(1)
    .invoice_number(1)
    .issued_at(fixed_issued_at())
}

// ═══════════════════════════════════════════════════════════════════════════
// Fix 1: vNF calculation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn vnf_equals_vprod_when_no_extras() {
    let built = nfce_builder()
        .add_item(sample_item())
        .payments(vec![sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();

    // vProd = 20.00, no desc/frete/seg/outro/ST/FCPST/II/IPI
    assert!(
        xml.contains("<vNF>20.00</vNF>"),
        "vNF should equal vProd when no extras: {xml}"
    );
}

#[test]
fn vnf_subtracts_discount() {
    let item = sample_item().v_desc(Cents(300)); // 3.00 discount
    let built = nfce_builder()
        .add_item(item)
        .payments(vec![PaymentData::new("01", Cents(1700))])
        .build()
        .unwrap();
    let xml = built.xml();

    // vNF = vProd(20.00) - vDesc(3.00) = 17.00
    assert!(
        xml.contains("<vNF>17.00</vNF>"),
        "vNF should be vProd - vDesc = 17.00: {xml}"
    );
}

#[test]
fn vnf_adds_freight_insurance_other() {
    let item = sample_item()
        .v_frete(Cents(500)) // 5.00 freight
        .v_seg(Cents(200)) // 2.00 insurance
        .v_outro(Cents(100)); // 1.00 other
    let payment = PaymentData::new("01", Cents(2800));
    let built = nfce_builder()
        .add_item(item)
        .payments(vec![payment])
        .build()
        .unwrap();
    let xml = built.xml();

    // vNF = vProd(20.00) + vFrete(5.00) + vSeg(2.00) + vOutro(1.00) = 28.00
    assert!(
        xml.contains("<vNF>28.00</vNF>"),
        "vNF should be 28.00 with freight+seg+outro: {xml}"
    );
}

#[test]
fn vnf_full_formula_with_discount_and_extras() {
    let item = sample_item()
        .v_desc(Cents(300)) // 3.00 discount
        .v_frete(Cents(500)) // 5.00 freight
        .v_seg(Cents(200)) // 2.00 insurance
        .v_outro(Cents(100)); // 1.00 other
    let payment = PaymentData::new("01", Cents(2500));
    let built = nfce_builder()
        .add_item(item)
        .payments(vec![payment])
        .build()
        .unwrap();
    let xml = built.xml();

    // vNF = vProd(20.00) - vDesc(3.00) + vFrete(5.00) + vSeg(2.00) + vOutro(1.00) = 25.00
    assert!(
        xml.contains("<vNF>25.00</vNF>"),
        "vNF should be 25.00: {xml}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Fix 2: vFrete/vSeg/vDesc/vOutro accumulated from items
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn vfrete_accumulated_from_items() {
    let item1 = InvoiceItemData::new(
        1,
        "1",
        "Product A",
        "84715010",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "102",
        Rate(0),
        Cents(0),
        "99",
        "99",
    )
    .v_frete(Cents(300));

    let item2 = InvoiceItemData::new(
        2,
        "2",
        "Product B",
        "84715010",
        "5102",
        "UN",
        1.0,
        Cents(2000),
        Cents(2000),
        "102",
        Rate(0),
        Cents(0),
        "99",
        "99",
    )
    .v_frete(Cents(200));

    let built = nfce_builder()
        .add_item(item1)
        .add_item(item2)
        .payments(vec![PaymentData::new("01", Cents(3500))])
        .build()
        .unwrap();
    let xml = built.xml();

    // Total freight = 3.00 + 2.00 = 5.00
    assert!(
        xml.contains("<vFrete>5.00</vFrete>"),
        "vFrete should be accumulated from items = 5.00"
    );
}

#[test]
fn vdesc_accumulated_from_items() {
    let item1 = sample_item().v_desc(Cents(100));
    let item2 = InvoiceItemData::new(
        2,
        "2",
        "Product B",
        "84715010",
        "5102",
        "UN",
        1.0,
        Cents(3000),
        Cents(3000),
        "102",
        Rate(0),
        Cents(0),
        "99",
        "99",
    )
    .v_desc(Cents(250));

    let built = nfce_builder()
        .add_item(item1)
        .add_item(item2)
        .payments(vec![PaymentData::new("01", Cents(4650))])
        .build()
        .unwrap();
    let xml = built.xml();

    // Total discount = 1.00 + 2.50 = 3.50
    assert!(
        xml.contains("<vDesc>3.50</vDesc>"),
        "vDesc should be accumulated from items = 3.50"
    );
}

#[test]
fn vseg_and_voutro_accumulated_from_items() {
    let item1 = sample_item().v_seg(Cents(150)).v_outro(Cents(50));
    let item2 = InvoiceItemData::new(
        2,
        "2",
        "Product B",
        "84715010",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "102",
        Rate(0),
        Cents(0),
        "99",
        "99",
    )
    .v_seg(Cents(100))
    .v_outro(Cents(75));

    let built = nfce_builder()
        .add_item(item1)
        .add_item(item2)
        .payments(vec![PaymentData::new("01", Cents(3375))])
        .build()
        .unwrap();
    let xml = built.xml();

    // vSeg = 1.50 + 1.00 = 2.50
    assert!(
        xml.contains("<vSeg>2.50</vSeg>"),
        "vSeg should be accumulated = 2.50"
    );
    // vOutro = 0.50 + 0.75 = 1.25
    assert!(
        xml.contains("<vOutro>1.25</vOutro>"),
        "vOutro should be accumulated = 1.25"
    );
}

#[test]
fn zero_values_when_items_have_no_extras() {
    let built = nfce_builder()
        .add_item(sample_item())
        .payments(vec![sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();

    assert!(
        xml.contains("<vFrete>0.00</vFrete>"),
        "vFrete should be 0.00"
    );
    assert!(xml.contains("<vSeg>0.00</vSeg>"), "vSeg should be 0.00");
    assert!(xml.contains("<vDesc>0.00</vDesc>"), "vDesc should be 0.00");
    assert!(
        xml.contains("<vOutro>0.00</vOutro>"),
        "vOutro should be 0.00"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Fix 3: transp order — transporta BEFORE retTransp
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn transp_order_transporta_before_ret_transp() {
    let carrier = CarrierData::new()
        .tax_id("12345678000199")
        .name("Carrier Co");
    let retained = RetainedIcmsTransp::new(
        Cents(10000),
        Rate(1200),
        Cents(1200),
        "5353",
        IbgeCode("3550308".to_string()),
    );
    let transport = TransportData::new("0")
        .carrier(carrier)
        .retained_icms(retained);

    let built = nfe_builder()
        .add_item(sample_item())
        .payments(vec![sample_payment()])
        .transport(transport)
        .build()
        .unwrap();
    let xml = built.xml();

    let transporta_pos = xml.find("<transporta>").expect("should have transporta");
    let ret_transp_pos = xml.find("<retTransp>").expect("should have retTransp");
    assert!(
        transporta_pos < ret_transp_pos,
        "transporta (pos {transporta_pos}) must come before retTransp (pos {ret_transp_pos}) per XSD schema"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Fix 4: indIEDest forced to "9" for NFC-e
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn nfce_forces_ind_ie_dest_9() {
    let recipient = RecipientData::new("12345678000199", "Buyer Corp").state_tax_id("123456789"); // has IE, but NFC-e should force "9"
    let built = nfce_builder()
        .add_item(sample_item())
        .payments(vec![sample_payment()])
        .recipient(recipient)
        .build()
        .unwrap();
    let xml = built.xml();

    assert!(
        xml.contains("<indIEDest>9</indIEDest>"),
        "NFC-e must force indIEDest=9 regardless of state_tax_id: {xml}"
    );
    // NFC-e should not emit IE even when state_tax_id is present
    let dest_section = &xml[xml.find("<dest>").unwrap()..xml.find("</dest>").unwrap()];
    assert!(
        !dest_section.contains("<IE>"),
        "NFC-e should not emit IE tag: {dest_section}"
    );
}

#[test]
fn nfe_uses_ind_ie_dest_1_when_ie_present() {
    let recipient = RecipientData::new("12345678000199", "Buyer Corp")
        .state_tax_id("123456789")
        .street("Rua A")
        .street_number("100")
        .district("Centro")
        .city_code(IbgeCode("3550308".to_string()))
        .city_name("Sao Paulo")
        .state_code("SP")
        .zip_code("01310100");

    let built = nfe_builder()
        .add_item(sample_item())
        .payments(vec![sample_payment()])
        .recipient(recipient)
        .build()
        .unwrap();
    let xml = built.xml();

    assert!(
        xml.contains("<indIEDest>1</indIEDest>"),
        "NF-e with IE should use indIEDest=1: {xml}"
    );
    assert!(
        xml.contains("<IE>123456789</IE>"),
        "NF-e should emit IE tag"
    );
}

#[test]
fn nfe_uses_ind_ie_dest_9_when_no_ie() {
    let recipient = RecipientData::new("12345678901", "Person")
        .street("Rua B")
        .street_number("200")
        .district("Bairro")
        .city_code(IbgeCode("3550308".to_string()))
        .city_name("Sao Paulo")
        .state_code("SP")
        .zip_code("01310100");

    let built = nfe_builder()
        .add_item(sample_item())
        .payments(vec![sample_payment()])
        .recipient(recipient)
        .build()
        .unwrap();
    let xml = built.xml();

    assert!(
        xml.contains("<indIEDest>9</indIEDest>"),
        "NF-e without IE should use indIEDest=9"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Fix 5: Homologation xNome/xProd substitution
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn homologation_replaces_xnome_in_dest() {
    let built = nfce_builder()
        .add_item(sample_item())
        .payments(vec![sample_payment()])
        .recipient(RecipientData::new("12345678901", "Real Name"))
        .build()
        .unwrap();
    let xml = built.xml();

    assert!(
        xml.contains("<xNome>NF-E EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL</xNome>"),
        "Homologation should replace dest xNome"
    );
    assert!(
        !xml.contains("<xNome>Real Name</xNome>"),
        "Original name should not appear in homologation"
    );
}

#[test]
fn homologation_nfce_replaces_xprod_item_1() {
    let built = nfce_builder()
        .add_item(sample_item())
        .payments(vec![sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();

    assert!(
        xml.contains(
            "<xProd>NOTA FISCAL EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL</xProd>"
        ),
        "Homologation NFC-e item 1 xProd should be replaced"
    );
}

#[test]
fn homologation_nfce_does_not_replace_xprod_item_2() {
    let item2 = InvoiceItemData::new(
        2,
        "2",
        "Real Product B",
        "84715010",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "102",
        Rate(0),
        Cents(0),
        "99",
        "99",
    );
    let built = nfce_builder()
        .add_item(sample_item())
        .add_item(item2)
        .payments(vec![PaymentData::new("01", Cents(3000))])
        .build()
        .unwrap();
    let xml = built.xml();

    // Item 2 should keep original description
    assert!(
        xml.contains("<xProd>Real Product B</xProd>"),
        "Item 2 xProd should not be replaced in homologation"
    );
}

#[test]
fn homologation_nfe_does_not_replace_xprod() {
    let built = nfe_builder()
        .add_item(sample_item())
        .payments(vec![sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();

    // NF-e model 55 does not replace xProd even in homologation
    assert!(
        xml.contains("<xProd>Product A</xProd>"),
        "NF-e (model 55) should NOT replace xProd even in homologation"
    );
}

#[test]
fn production_does_not_replace_xnome() {
    let built = production_nfce_builder()
        .add_item(sample_item())
        .payments(vec![sample_payment()])
        .recipient(RecipientData::new("12345678901", "Real Name"))
        .build()
        .unwrap();
    let xml = built.xml();

    assert!(
        xml.contains("<xNome>Real Name</xNome>"),
        "Production should keep original xNome"
    );
}

#[test]
fn production_does_not_replace_xprod() {
    let built = production_nfce_builder()
        .add_item(sample_item())
        .payments(vec![sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();

    assert!(
        xml.contains("<xProd>Product A</xProd>"),
        "Production should keep original xProd"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Fix 6: indTot variable
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ind_tot_defaults_to_1() {
    let built = nfce_builder()
        .add_item(sample_item())
        .payments(vec![sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();

    assert!(
        xml.contains("<indTot>1</indTot>"),
        "Default indTot should be 1"
    );
}

#[test]
fn ind_tot_set_to_0_excludes_from_total() {
    let item1 = sample_item(); // indTot=1 (default), vProd=20.00
    let item2 = InvoiceItemData::new(
        2,
        "2",
        "Free Sample",
        "84715010",
        "5102",
        "UN",
        1.0,
        Cents(500),
        Cents(500),
        "102",
        Rate(0),
        Cents(0),
        "99",
        "99",
    )
    .ind_tot(0); // Not included in total

    let built = nfce_builder()
        .add_item(item1)
        .add_item(item2)
        .payments(vec![PaymentData::new("01", Cents(2000))])
        .build()
        .unwrap();
    let xml = built.xml();

    // indTot=0 should appear in XML
    assert!(
        xml.contains("<indTot>0</indTot>"),
        "indTot=0 should be in XML"
    );

    // vProd in ICMSTot should only include item 1 (20.00), not item 2 (5.00)
    // Extract the ICMSTot vProd
    let icms_tot_start = xml.find("<ICMSTot>").unwrap();
    let icms_tot_end = xml.find("</ICMSTot>").unwrap();
    let icms_tot = &xml[icms_tot_start..icms_tot_end];
    assert!(
        icms_tot.contains("<vProd>20.00</vProd>"),
        "ICMSTot vProd should only include indTot=1 items = 20.00, got: {icms_tot}"
    );

    // vNF should also be 20.00 (only indTot=1 items)
    assert!(
        icms_tot.contains("<vNF>20.00</vNF>"),
        "vNF should only include indTot=1 items = 20.00"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Fix 7: vTotTrib in ICMSTot
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn vtottrib_omitted_when_zero() {
    let built = nfce_builder()
        .add_item(sample_item())
        .payments(vec![sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();

    assert!(
        !xml.contains("<vTotTrib>"),
        "vTotTrib should be omitted when zero"
    );
}

#[test]
fn vtottrib_included_when_positive() {
    let item = sample_item().v_tot_trib(Cents(350)); // 3.50
    let built = nfce_builder()
        .add_item(item)
        .payments(vec![sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();

    assert!(
        xml.contains("<vTotTrib>3.50</vTotTrib>"),
        "vTotTrib should be included when > 0: {xml}"
    );
}

#[test]
fn vtottrib_accumulated_from_multiple_items() {
    let item1 = sample_item().v_tot_trib(Cents(200)); // 2.00
    let item2 = InvoiceItemData::new(
        2,
        "2",
        "Product B",
        "84715010",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "102",
        Rate(0),
        Cents(0),
        "99",
        "99",
    )
    .v_tot_trib(Cents(150)); // 1.50

    let built = nfce_builder()
        .add_item(item1)
        .add_item(item2)
        .payments(vec![PaymentData::new("01", Cents(3000))])
        .build()
        .unwrap();
    let xml = built.xml();

    // Total vTotTrib = 2.00 + 1.50 = 3.50
    assert!(
        xml.contains("<vTotTrib>3.50</vTotTrib>"),
        "vTotTrib should be accumulated = 3.50"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Fix 8: indIntermed conditional
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ind_intermed_omitted_when_not_set() {
    let built = nfce_builder()
        .add_item(sample_item())
        .payments(vec![sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();

    assert!(
        !xml.contains("<indIntermed>"),
        "indIntermed should be omitted when not set"
    );
}

#[test]
fn ind_intermed_included_when_explicitly_set() {
    let built = nfce_builder()
        .add_item(sample_item())
        .payments(vec![sample_payment()])
        .intermediary_indicator("0")
        .build()
        .unwrap();
    let xml = built.xml();

    assert!(
        xml.contains("<indIntermed>0</indIntermed>"),
        "indIntermed should be present when explicitly set: {xml}"
    );
}

#[test]
fn ind_intermed_value_1_when_set() {
    let built = nfce_builder()
        .add_item(sample_item())
        .payments(vec![sample_payment()])
        .intermediary_indicator("1")
        .build()
        .unwrap();
    let xml = built.xml();

    assert!(
        xml.contains("<indIntermed>1</indIntermed>"),
        "indIntermed=1 should appear when set"
    );
}
