use std::hint::black_box;
use std::time::Instant;

use fiscal::certificate::sign_xml;
use fiscal::format_utils::{format_cents_2, format_rate_4};
use fiscal::newtypes::{Cents, IbgeCode, Rate, Rate4};
use fiscal::state_codes::get_state_code;
use fiscal::tax_element::{TaxElement, TaxField, serialize_tax_element};
use fiscal::tax_icms::{IcmsTotals, create_icms_totals, merge_icms_totals};
use fiscal::types::{
    InvoiceItemData, InvoiceModel, IssuerData, PaymentData, RecipientData, SefazEnvironment,
    TaxRegime,
};
use fiscal::xml_builder::InvoiceBuilder;
use fiscal::xml_utils::{TagContent, escape_xml, tag};

// ── Helpers (copied from benches/fiscal_bench.rs) ───────────────────────────

fn make_sample_issuer() -> IssuerData {
    IssuerData::new(
        "04123456000190",
        "9012345678",
        "Auto Eletrica Barbosa LTDA",
        TaxRegime::Normal,
        "PR",
        IbgeCode("4106902".to_string()),
        "Curitiba",
        "Rua XV de Novembro",
        "1000",
        "Centro",
        "80020310",
    )
    .trade_name("Auto Eletrica Barbosa")
}

fn make_sample_recipient() -> RecipientData {
    RecipientData::new("12345678901234", "Cliente Teste LTDA")
        .state_code("PR")
        .state_tax_id("1234567890")
        .street("Rua das Flores")
        .street_number("500")
        .district("Batel")
        .city_code(IbgeCode("4106902".to_string()))
        .city_name("Curitiba")
        .zip_code("80420120")
}

fn make_sample_item(n: u32) -> InvoiceItemData {
    InvoiceItemData::new(
        n,
        format!("{n:03}"),
        "Servico de eletrica automotiva",
        "00000000",
        "5102",
        "UN",
        1.0,
        Cents(15000),
        Cents(15000),
        "00",
        Rate(1800),
        Cents(2700),
        "01",
        "01",
    )
    .orig("0")
    .icms_mod_bc(0)
    .pis_v_bc(Cents(15000))
    .pis_p_pis(Rate4(16500))
    .pis_v_pis(Cents(248))
    .cofins_v_bc(Cents(15000))
    .cofins_p_cofins(Rate4(76000))
    .cofins_v_cofins(Cents(1140))
}

fn make_sample_unsigned_nfe_xml() -> String {
    concat!(
        r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
        r#"<infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
        "<ide><cUF>41</cUF><cNF>12345678</cNF><natOp>VENDA</natOp>",
        "<mod>55</mod><serie>1</serie><nNF>123</nNF>",
        "<dhEmi>2026-03-11T10:30:00-03:00</dhEmi>",
        "<tpNF>1</tpNF><idDest>1</idDest><cMunFG>4106902</cMunFG>",
        "<tpImp>1</tpImp><tpEmis>1</tpEmis><cDV>0</cDV>",
        "<tpAmb>2</tpAmb><finNFe>1</finNFe><indFinal>1</indFinal>",
        "<indPres>1</indPres><procEmi>0</procEmi><verProc>1.0</verProc></ide>",
        "<emit><CNPJ>04123456000190</CNPJ><xNome>Auto Eletrica Barbosa LTDA</xNome>",
        "<enderEmit><xLgr>Rua XV</xLgr><nro>1000</nro><xBairro>Centro</xBairro>",
        "<cMun>4106902</cMun><xMun>Curitiba</xMun><UF>PR</UF><CEP>80020310</CEP>",
        "<cPais>1058</cPais><xPais>Brasil</xPais></enderEmit>",
        "<IE>9012345678</IE><CRT>3</CRT></emit>",
        "<det nItem=\"1\"><prod><cProd>001</cProd><cEAN>SEM GTIN</cEAN>",
        "<xProd>Servico de eletrica</xProd>",
        "<NCM>00000000</NCM><CFOP>5102</CFOP><uCom>UN</uCom>",
        "<qCom>1.0000</qCom><vUnCom>150.0000000000</vUnCom><vProd>150.00</vProd>",
        "<cEANTrib>SEM GTIN</cEANTrib><uTrib>UN</uTrib>",
        "<qTrib>1.0000</qTrib><vUnTrib>150.0000000000</vUnTrib>",
        "<indTot>1</indTot></prod>",
        "<imposto><ICMS><ICMS00><orig>0</orig><CST>00</CST><modBC>0</modBC>",
        "<vBC>150.00</vBC><pICMS>18.0000</pICMS><vICMS>27.00</vICMS>",
        "</ICMS00></ICMS></imposto></det>",
        "<total><ICMSTot><vBC>150.00</vBC><vICMS>27.00</vICMS>",
        "<vNF>150.00</vNF></ICMSTot></total>",
        "<transp><modFrete>9</modFrete></transp>",
        "<pag><detPag><tPag>01</tPag><vPag>150.00</vPag></detPag></pag>",
        "</infNFe></NFe>",
    )
    .to_string()
}

fn make_test_keypair() -> (String, String) {
    use openssl::bn::BigNum;
    use openssl::hash::MessageDigest;
    use openssl::pkey::PKey;
    use openssl::rsa::Rsa;
    use openssl::x509::{X509, X509NameBuilder};

    let rsa = Rsa::generate(2048).unwrap();
    let pkey = PKey::from_rsa(rsa).unwrap();

    let mut name_builder = X509NameBuilder::new().unwrap();
    name_builder
        .append_entry_by_text("CN", "Bench Test")
        .unwrap();
    let name = name_builder.build();

    let mut builder = X509::builder().unwrap();
    builder.set_version(2).unwrap();
    let serial = BigNum::from_u32(1).unwrap();
    builder
        .set_serial_number(&serial.to_asn1_integer().unwrap())
        .unwrap();
    builder.set_subject_name(&name).unwrap();
    builder.set_issuer_name(&name).unwrap();
    builder
        .set_not_before(&openssl::asn1::Asn1Time::days_from_now(0).unwrap())
        .unwrap();
    builder
        .set_not_after(&openssl::asn1::Asn1Time::days_from_now(365).unwrap())
        .unwrap();
    builder.set_pubkey(&pkey).unwrap();
    builder.sign(&pkey, MessageDigest::sha256()).unwrap();
    let cert = builder.build();

    let private_key_pem = String::from_utf8(pkey.private_key_to_pem_pkcs8().unwrap()).unwrap();
    let cert_pem = String::from_utf8(cert.to_pem().unwrap()).unwrap();

    (private_key_pem, cert_pem)
}

// ── Benchmark harness ───────────────────────────────────────────────────────

struct BenchResult {
    name: &'static str,
    ns_per_op: f64,
    ops_per_sec: u64,
}

fn bench<F: FnMut()>(name: &'static str, iterations: u64, mut f: F) -> BenchResult {
    // Warmup: 10% of iterations (at least 1)
    let warmup = (iterations / 10).max(1);
    for _ in 0..warmup {
        black_box(f());
    }

    let start = Instant::now();
    for _ in 0..iterations {
        black_box(f());
    }
    let elapsed = start.elapsed();

    let total_ns = elapsed.as_nanos() as f64;
    let ns_per_op = total_ns / iterations as f64;
    let ops_per_sec = if ns_per_op > 0.0 {
        (1_000_000_000.0 / ns_per_op) as u64
    } else {
        0
    };

    BenchResult {
        name,
        ns_per_op,
        ops_per_sec,
    }
}

fn main() {
    let mut results: Vec<BenchResult> = Vec::new();

    // ── Fast ops: 1_000_000 iterations ──────────────────────────────────────

    results.push(bench("format_cents_2", 1_000_000, || {
        format_cents_2(black_box(123456));
    }));

    results.push(bench("format_rate_4", 1_000_000, || {
        format_rate_4(black_box(1800));
    }));

    results.push(bench("escape_xml_clean", 1_000_000, || {
        escape_xml(black_box("Auto Eletrica Barbosa LTDA"));
    }));

    results.push(bench("escape_xml_dirty", 1_000_000, || {
        escape_xml(black_box("M&M's <special> \"quoted\" & 'apos'"));
    }));

    results.push(bench("tag_simple_text", 1_000_000, || {
        tag("xNome", &[], black_box("Test Company").into());
    }));

    results.push(bench("get_state_code", 1_000_000, || {
        let _ = get_state_code(black_box("PR"));
    }));

    // ── Medium ops: 100_000 iterations ──────────────────────────────────────

    results.push(bench("tag_nested_invoice_item", 100_000, || {
        tag(
            "det",
            &[("nItem", "1")],
            TagContent::Children(vec![
                tag(
                    "prod",
                    &[],
                    TagContent::Children(vec![
                        tag("cProd", &[], "001".into()),
                        tag("cEAN", &[], "SEM GTIN".into()),
                        tag("xProd", &[], "Servico de eletrica".into()),
                        tag("NCM", &[], "00000000".into()),
                        tag("CFOP", &[], "5102".into()),
                        tag("uCom", &[], "UN".into()),
                        tag("qCom", &[], "1.0000".into()),
                        tag("vUnCom", &[], "150.0000000000".into()),
                        tag("vProd", &[], "150.00".into()),
                        tag("cEANTrib", &[], "SEM GTIN".into()),
                        tag("uTrib", &[], "UN".into()),
                        tag("qTrib", &[], "1.0000".into()),
                        tag("vUnTrib", &[], "150.0000000000".into()),
                        tag("indTot", &[], "1".into()),
                    ]),
                ),
                tag(
                    "imposto",
                    &[],
                    TagContent::Children(vec![tag(
                        "ICMS",
                        &[],
                        TagContent::Children(vec![tag(
                            "ICMS00",
                            &[],
                            TagContent::Children(vec![
                                tag("orig", &[], "0".into()),
                                tag("CST", &[], "00".into()),
                                tag("modBC", &[], "0".into()),
                                tag("vBC", &[], "150.00".into()),
                                tag("pICMS", &[], "18.0000".into()),
                                tag("vICMS", &[], "27.00".into()),
                            ]),
                        )]),
                    )]),
                ),
            ]),
        );
    }));

    {
        let element = TaxElement {
            outer_tag: Some("ICMS".to_string()),
            outer_fields: vec![],
            variant_tag: "ICMS00".to_string(),
            fields: vec![
                TaxField::new("orig", "0"),
                TaxField::new("CST", "00"),
                TaxField::new("modBC", "0"),
                TaxField::new("vBC", "150.00"),
                TaxField::new("pICMS", "18.0000"),
                TaxField::new("vICMS", "27.00"),
            ],
        };
        results.push(bench("serialize_icms00", 100_000, || {
            serialize_tax_element(black_box(&element));
        }));
    }

    results.push(bench("create_icms_totals", 100_000, || {
        create_icms_totals();
    }));

    {
        let source = IcmsTotals::new()
            .v_bc(Cents(10000))
            .v_icms(Cents(1800))
            .v_fcp(Cents(200));
        results.push(bench("merge_icms_totals_10", 100_000, || {
            let mut target = create_icms_totals();
            for _ in 0..10 {
                merge_icms_totals(&mut target, black_box(&source));
            }
            black_box(target);
        }));
    }

    {
        let issuer = make_sample_issuer();
        let recipient = make_sample_recipient();
        let item = make_sample_item(1);
        let payment = PaymentData::new("01", Cents(15000));
        results.push(bench("invoice_builder_simple", 100_000, || {
            InvoiceBuilder::new(
                black_box(issuer.clone()),
                SefazEnvironment::Homologation,
                InvoiceModel::Nfe,
            )
            .series(1)
            .invoice_number(123)
            .recipient(black_box(recipient.clone()))
            .add_item(black_box(item.clone()))
            .payments(vec![black_box(payment.clone())])
            .build()
            .unwrap();
        }));
    }

    // ── Slow ops: 10_000 iterations ─────────────────────────────────────────

    {
        let (private_key_pem, cert_pem) = make_test_keypair();
        let xml = make_sample_unsigned_nfe_xml();
        results.push(bench("sign_xml", 10_000, || {
            sign_xml(
                black_box(&xml),
                black_box(&private_key_pem),
                black_box(&cert_pem),
            )
            .unwrap();
        }));
    }

    // ── Output JSON ─────────────────────────────────────────────────────────

    let mut json = String::from("[");
    for (i, r) in results.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        json.push_str(&format!(
            r#"{{"name":"{}","ns_per_op":{:.1},"ops_per_sec":{}}}"#,
            r.name, r.ns_per_op, r.ops_per_sec,
        ));
    }
    json.push(']');
    println!("{json}");
}
