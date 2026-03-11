use fiscal::format_utils::{
    format_cents, format_cents_2, format_cents_10, format_cents_or_none, format_cents_or_zero,
    format_decimal, format_rate, format_rate_4, format_rate4, format_rate4_or_zero,
};
use fiscal::newtypes::Cents;
use fiscal::state_codes::{get_state_by_code, get_state_code};
use fiscal::tax_element::{TaxElement, TaxField, serialize_tax_element};
use fiscal::tax_icms::{IcmsTotals, create_icms_totals, merge_icms_totals};
use fiscal::xml_utils::{escape_xml, extract_xml_tag_value, tag};
use insta::{assert_debug_snapshot, assert_snapshot};

// ── XML Utils ───────────────────────────────────────────────────────────────

#[test]
fn snapshot_simple_tag() {
    assert_snapshot!(tag("xNome", &[], "Test Company".into()));
}

#[test]
fn snapshot_tag_empty_children() {
    assert_snapshot!(tag("emit", &[], vec![].into()));
}

#[test]
fn snapshot_tag_with_text_escaping() {
    assert_snapshot!(tag("xNome", &[], "A & B <Corp> \"LLC\"".into()));
}

#[test]
fn snapshot_tag_with_attrs() {
    assert_snapshot!(tag(
        "det",
        &[("nItem", "1")],
        vec![tag(
            "prod",
            &[],
            vec![
                tag("cProd", &[], "001".into()),
                tag("xProd", &[], "Widget".into()),
            ]
            .into(),
        ),]
        .into(),
    ));
}

#[test]
fn snapshot_tag_with_multiple_attrs() {
    assert_snapshot!(tag(
        "infNFe",
        &[("versao", "4.00"), ("Id", "NFe35...")],
        vec![tag("ide", &[], vec![].into())].into(),
    ));
}

#[test]
fn snapshot_tag_self_closing_like() {
    // Even "no content" produces <tag></tag> (not self-closing)
    assert_snapshot!(tag("xMotivo", &[], vec![].into()));
}

#[test]
fn snapshot_escape_xml_ampersand() {
    assert_snapshot!(escape_xml("Tom & Jerry"));
}

#[test]
fn snapshot_escape_xml_all_chars() {
    assert_snapshot!(escape_xml("<tag attr=\"val\" & 'single'>"));
}

#[test]
fn snapshot_escape_xml_clean_string() {
    assert_snapshot!(escape_xml("Normal text 123"));
}

#[test]
fn snapshot_extract_xml_tag_value() {
    let xml = "<nfeProc><protNFe><infProt><cStat>100</cStat></infProt></protNFe></nfeProc>";
    assert_snapshot!(extract_xml_tag_value(xml, "cStat").unwrap());
}

#[test]
fn snapshot_extract_xml_tag_value_not_found() {
    let xml = "<root><a>1</a></root>";
    assert_debug_snapshot!(extract_xml_tag_value(xml, "missing"));
}

// ── TaxElement Serialization ────────────────────────────────────────────────

#[test]
fn snapshot_tax_element_icms00() {
    let element = TaxElement {
        outer_tag: Some("ICMS".into()),
        outer_fields: vec![],
        variant_tag: "ICMS00".into(),
        fields: vec![
            TaxField::new("orig", "0"),
            TaxField::new("CST", "00"),
            TaxField::new("modBC", "3"),
            TaxField::new("vBC", "100.00"),
            TaxField::new("pICMS", "18.0000"),
            TaxField::new("vICMS", "18.00"),
        ],
    };
    assert_snapshot!(serialize_tax_element(&element));
}

#[test]
fn snapshot_tax_element_pis_aliq() {
    let element = TaxElement {
        outer_tag: Some("PIS".into()),
        outer_fields: vec![],
        variant_tag: "PISAliq".into(),
        fields: vec![
            TaxField::new("CST", "01"),
            TaxField::new("vBC", "100.00"),
            TaxField::new("pPIS", "1.6500"),
            TaxField::new("vPIS", "1.65"),
        ],
    };
    assert_snapshot!(serialize_tax_element(&element));
}

#[test]
fn snapshot_tax_element_ipi_with_outer_fields() {
    let element = TaxElement {
        outer_tag: Some("IPI".into()),
        outer_fields: vec![TaxField::new("cEnq", "999")],
        variant_tag: "IPITrib".into(),
        fields: vec![
            TaxField::new("CST", "50"),
            TaxField::new("vBC", "100.00"),
            TaxField::new("pIPI", "5.0000"),
            TaxField::new("vIPI", "5.00"),
        ],
    };
    assert_snapshot!(serialize_tax_element(&element));
}

#[test]
fn snapshot_tax_element_icms_uf_dest_no_outer() {
    let element = TaxElement {
        outer_tag: None,
        outer_fields: vec![],
        variant_tag: "ICMSUFDest".into(),
        fields: vec![
            TaxField::new("vBCUFDest", "1000.00"),
            TaxField::new("vBCFCPUFDest", "1000.00"),
            TaxField::new("pFCPUFDest", "2.0000"),
            TaxField::new("pICMSUFDest", "18.0000"),
            TaxField::new("pICMSInter", "12.0000"),
            TaxField::new("pICMSInterPart", "100.0000"),
            TaxField::new("vFCPUFDest", "20.00"),
            TaxField::new("vICMSUFDest", "60.00"),
            TaxField::new("vICMSUFRemet", "0.00"),
        ],
    };
    assert_snapshot!(serialize_tax_element(&element));
}

#[test]
fn snapshot_tax_element_ii_no_outer() {
    let element = TaxElement {
        outer_tag: None,
        outer_fields: vec![],
        variant_tag: "II".into(),
        fields: vec![
            TaxField::new("vBC", "5000.00"),
            TaxField::new("vDespAdu", "200.00"),
            TaxField::new("vII", "800.00"),
            TaxField::new("vIOF", "50.00"),
        ],
    };
    assert_snapshot!(serialize_tax_element(&element));
}

#[test]
fn snapshot_tax_element_cofins() {
    let element = TaxElement {
        outer_tag: Some("COFINS".into()),
        outer_fields: vec![],
        variant_tag: "COFINSAliq".into(),
        fields: vec![
            TaxField::new("CST", "01"),
            TaxField::new("vBC", "100.00"),
            TaxField::new("pCOFINS", "7.6000"),
            TaxField::new("vCOFINS", "7.60"),
        ],
    };
    assert_snapshot!(serialize_tax_element(&element));
}

#[test]
fn snapshot_tax_element_xml_escaping_in_fields() {
    let element = TaxElement {
        outer_tag: None,
        outer_fields: vec![],
        variant_tag: "obs".into(),
        fields: vec![TaxField::new("xTexto", "Value > 100 & special <chars>")],
    };
    assert_snapshot!(serialize_tax_element(&element));
}

// ── Format Utils ────────────────────────────────────────────────────────────

#[test]
fn snapshot_format_cents_zero() {
    assert_snapshot!(format_cents(0, 2), @"0.00");
}

#[test]
fn snapshot_format_cents_1050() {
    assert_snapshot!(format_cents_2(1050), @"10.50");
}

#[test]
fn snapshot_format_cents_123456() {
    assert_snapshot!(format_cents_2(123456), @"1234.56");
}

#[test]
fn snapshot_format_cents_negative() {
    assert_snapshot!(format_cents_2(-500), @"-5.00");
}

#[test]
fn snapshot_format_cents_10_places() {
    assert_snapshot!(format_cents_10(1050), @"10.5000000000");
}

#[test]
fn snapshot_format_cents_1_cent() {
    assert_snapshot!(format_cents_2(1), @"0.01");
}

#[test]
fn snapshot_format_decimal() {
    assert_snapshot!(format_decimal(3.14159, 4), @"3.1416");
}

#[test]
fn snapshot_format_rate_18_percent() {
    assert_snapshot!(format_rate(1800, 4), @"18.0000");
}

#[test]
fn snapshot_format_rate_12_percent() {
    assert_snapshot!(format_rate_4(1200), @"12.0000");
}

#[test]
fn snapshot_format_rate_7_percent() {
    assert_snapshot!(format_rate(700, 4), @"7.0000");
}

#[test]
fn snapshot_format_rate_4_percent() {
    assert_snapshot!(format_rate(400, 4), @"4.0000");
}

#[test]
fn snapshot_format_rate_zero() {
    assert_snapshot!(format_rate(0, 4), @"0.0000");
}

#[test]
fn snapshot_format_rate4_pis() {
    // PIS 1.65% stored as 16500
    assert_snapshot!(format_rate4(16500), @"1.6500");
}

#[test]
fn snapshot_format_rate4_cofins() {
    // COFINS 7.60% stored as 76000
    assert_snapshot!(format_rate4(76000), @"7.6000");
}

#[test]
fn snapshot_format_rate4_zero() {
    assert_snapshot!(format_rate4(0), @"0.0000");
}

#[test]
fn snapshot_format_cents_or_none_some() {
    assert_debug_snapshot!(format_cents_or_none(Some(1050), 2), @r#"
    Some(
        "10.50",
    )
    "#);
}

#[test]
fn snapshot_format_cents_or_none_none() {
    assert_debug_snapshot!(format_cents_or_none(None, 2), @"None");
}

#[test]
fn snapshot_format_cents_or_zero_some() {
    assert_snapshot!(format_cents_or_zero(Some(999), 2), @"9.99");
}

#[test]
fn snapshot_format_cents_or_zero_none() {
    assert_snapshot!(format_cents_or_zero(None, 2), @"0.00");
}

#[test]
fn snapshot_format_rate4_or_zero_some() {
    assert_snapshot!(format_rate4_or_zero(Some(16500)), @"1.6500");
}

#[test]
fn snapshot_format_rate4_or_zero_none() {
    assert_snapshot!(format_rate4_or_zero(None), @"0.0000");
}

// ── State Codes ─────────────────────────────────────────────────────────────

#[test]
fn snapshot_all_state_codes() {
    let mut mappings: Vec<String> = vec![
        "AC", "AL", "AM", "AP", "BA", "CE", "DF", "ES", "GO", "MA", "MG", "MS", "MT", "PA", "PB",
        "PE", "PI", "PR", "RJ", "RN", "RO", "RR", "RS", "SC", "SE", "SP", "TO",
    ]
    .into_iter()
    .map(|uf| format!("{uf} -> {}", get_state_code(uf).unwrap()))
    .collect();
    mappings.sort();
    assert_snapshot!(mappings.join("\n"));
}

#[test]
fn snapshot_all_reverse_lookups() {
    let codes = vec![
        "11", "12", "13", "14", "15", "16", "17", "21", "22", "23", "24", "25", "26", "27", "28",
        "29", "31", "32", "33", "35", "41", "42", "43", "50", "51", "52", "53",
    ];
    let mut mappings: Vec<String> = codes
        .into_iter()
        .map(|code| format!("{code} -> {}", get_state_by_code(code).unwrap()))
        .collect();
    mappings.sort();
    assert_snapshot!(mappings.join("\n"));
}

#[test]
fn snapshot_state_code_unknown() {
    assert_debug_snapshot!(get_state_code("XX"), @r###"
    Err(
        InvalidStateCode(
            "XX",
        ),
    )
    "###);
}

#[test]
fn snapshot_state_by_code_unknown() {
    assert_debug_snapshot!(get_state_by_code("99"), @r###"
    Err(
        InvalidStateCode(
            "99",
        ),
    )
    "###);
}

#[test]
fn snapshot_specific_state_pr() {
    assert_snapshot!(get_state_code("PR").unwrap(), @"41");
}

#[test]
fn snapshot_specific_state_sp() {
    assert_snapshot!(get_state_code("SP").unwrap(), @"35");
}

#[test]
fn snapshot_reverse_lookup_41() {
    assert_snapshot!(get_state_by_code("41").unwrap(), @"PR");
}

#[test]
fn snapshot_reverse_lookup_35() {
    assert_snapshot!(get_state_by_code("35").unwrap(), @"SP");
}

// ── ICMS Totals ─────────────────────────────────────────────────────────────

#[test]
fn snapshot_create_icms_totals() {
    let totals = create_icms_totals();
    assert_debug_snapshot!(totals);
}

#[test]
fn snapshot_merge_icms_totals() {
    let mut target = create_icms_totals();
    let mut source = IcmsTotals::new();
    source.v_bc = Cents(10000);
    source.v_icms = Cents(1800);
    source.v_bc_st = Cents(5000);
    source.v_st = Cents(900);
    source.v_fcp = Cents(200);
    source.v_fcp_st = Cents(100);
    source.v_fcp_st_ret = Cents(50);
    source.v_fcp_uf_dest = Cents(300);
    source.v_icms_uf_dest = Cents(600);
    merge_icms_totals(&mut target, &source);
    assert_debug_snapshot!(target);
}

#[test]
fn snapshot_merge_icms_totals_accumulates() {
    let mut target = IcmsTotals::new();
    target.v_bc = Cents(10000);
    target.v_icms = Cents(1800);
    target.v_icms_deson = Cents(100);
    target.v_fcp = Cents(200);
    let mut source = IcmsTotals::new();
    source.v_bc = Cents(20000);
    source.v_icms = Cents(3600);
    source.v_icms_deson = Cents(200);
    source.v_bc_st = Cents(5000);
    source.v_st = Cents(900);
    source.v_fcp = Cents(400);
    source.v_fcp_st = Cents(100);
    source.v_fcp_st_ret = Cents(50);
    source.q_bc_mono = 1000;
    source.v_icms_mono = Cents(500);
    merge_icms_totals(&mut target, &source);
    assert_debug_snapshot!(target);
}
