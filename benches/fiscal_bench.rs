use chrono::TimeZone;
use fiscal::format_utils::{
    format_cents, format_cents_2, format_cents_10, format_decimal,
    format_rate, format_rate_4, format_rate4,
    format_cents_or_none, format_cents_or_zero, format_rate4_or_zero,
};
use fiscal::xml_utils::{escape_xml, extract_xml_tag_value, tag, TagContent};
use fiscal::tax_element::{TaxElement, TaxField, filter_fields, serialize_tax_element};
use fiscal::tax_icms::{IcmsTotals, create_icms_totals, merge_icms_totals};
use fiscal::tax_icms::{IcmsData, build_icms_xml, build_icms_part_xml, build_icms_st_xml, build_icms_uf_dest_xml};
use fiscal::tax_pis_cofins_ipi::{
    PisData, PisStData, CofinsData, CofinsStData, IpiData, IiData,
    build_pis_xml, build_pis_st_xml, build_cofins_xml, build_cofins_st_xml, build_ipi_xml, build_ii_xml,
};
use fiscal::tax_issqn::{IssqnData, build_issqn_xml, build_issqn_xml_with_totals, create_issqn_totals, build_imposto_devol};
use fiscal::tax_is::{IsData, build_is_xml};
use fiscal::gtin::{is_valid_gtin, calculate_check_digit};
use fiscal::state_codes::{get_state_code, get_state_by_code};
use fiscal::xml_builder::access_key::{build_access_key, calculate_mod11, generate_numeric_code, format_year_month};
use fiscal::xml_builder::tax_id::TaxId;
use fiscal::xml_builder::build_invoice_xml;
use fiscal::xml_builder::ide::{build_ide, format_datetime_nfe};
use fiscal::xml_builder::emit::{build_emit, build_address_fields};
use fiscal::xml_builder::dest::build_dest;
use fiscal::xml_builder::det::build_det;
use fiscal::xml_builder::total::{build_total, OtherTotals};
use fiscal::xml_builder::transp::build_transp;
use fiscal::xml_builder::pag::build_pag;
use fiscal::xml_builder::optional::{
    build_cobr, build_inf_adic, build_intermediary, build_tech_responsible,
    build_purchase, build_export, build_withdrawal, build_delivery, build_aut_xml,
};
use fiscal::types::{
    AccessKeyParams, InvoiceBuildData, InvoiceModel, EmissionType,
    SefazEnvironment, TaxRegime, IssuerData, RecipientData, InvoiceItemData,
    PaymentData, TransportData, CarrierData, VehicleData,
    VolumeData, BillingData, BillingInvoice, Installment, LocationData,
    IntermediaryData, TechResponsibleData, PurchaseData, ExportData,
    AuthorizedXml, NfceQrCodeParams, PutQRTagParams, QrCodeVersion,
    ContingencyType,
};
use fiscal::complement::{attach_protocol, attach_inutilizacao, attach_event_protocol, attach_b2b};
use fiscal::qrcode::{build_nfce_qr_code_url, build_nfce_consult_url, put_qr_tag};
use fiscal::contingency::{Contingency, contingency_for_state, adjust_nfe_contingency};
use fiscal::sefaz::request_builders::{
    build_autorizacao_request, build_status_request, build_consulta_request,
    build_cancela_request, build_inutilizacao_request, build_cce_request,
    build_dist_dfe_request, build_cadastro_request,
};
use fiscal::sefaz::urls::{get_sefaz_url, get_nfce_consult_url as get_nfce_consult_url_sefaz};
use fiscal::sefaz::response_parsers::{
    parse_autorizacao_response, parse_status_response, parse_cancellation_response,
};
use fiscal::standardize::{identify_xml_type, xml_to_json};
use fiscal::certificate::sign_xml;

fn main() {
    divan::main();
}

// ── Format Utils ─────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_format_cents_2(bencher: divan::Bencher) {
    bencher.bench(|| format_cents_2(divan::black_box(123456)));
}

#[divan::bench]
fn bench_format_cents_10dp(bencher: divan::Bencher) {
    bencher.bench(|| format_cents(divan::black_box(123456), 10));
}

#[divan::bench]
fn bench_format_rate_4(bencher: divan::Bencher) {
    bencher.bench(|| format_rate_4(divan::black_box(1800)));
}

#[divan::bench]
fn bench_format_rate4_pis(bencher: divan::Bencher) {
    bencher.bench(|| format_rate4(divan::black_box(16500)));
}

#[divan::bench]
fn bench_format_cents_or_zero_some(bencher: divan::Bencher) {
    bencher.bench(|| format_cents_or_zero(divan::black_box(Some(9999)), 2));
}

#[divan::bench]
fn bench_format_cents_or_zero_none(bencher: divan::Bencher) {
    bencher.bench(|| format_cents_or_zero(divan::black_box(None), 2));
}

// ── XML Utils ────────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_escape_xml_clean(bencher: divan::Bencher) {
    bencher.bench(|| escape_xml(divan::black_box("Auto Eletrica Barbosa LTDA")));
}

#[divan::bench]
fn bench_escape_xml_dirty(bencher: divan::Bencher) {
    bencher.bench(|| escape_xml(divan::black_box("M&M's <special> \"quoted\" & 'apos'")));
}

#[divan::bench]
fn bench_tag_simple_text(bencher: divan::Bencher) {
    bencher.bench(|| {
        tag("xNome", &[], divan::black_box("Test Company").into())
    });
}

#[divan::bench]
fn bench_tag_with_attrs(bencher: divan::Bencher) {
    bencher.bench(|| {
        tag(
            "det",
            divan::black_box(&[("nItem", "1")]),
            TagContent::Children(vec![
                tag("cProd", &[], "001".into()),
                tag("xProd", &[], "Widget".into()),
            ]),
        )
    });
}

#[divan::bench]
fn bench_tag_nested_invoice_item(bencher: divan::Bencher) {
    bencher.bench(|| {
        tag("det", &[("nItem", "1")], TagContent::Children(vec![
            tag("prod", &[], TagContent::Children(vec![
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
            ])),
            tag("imposto", &[], TagContent::Children(vec![
                tag("ICMS", &[], TagContent::Children(vec![
                    tag("ICMS00", &[], TagContent::Children(vec![
                        tag("orig", &[], "0".into()),
                        tag("CST", &[], "00".into()),
                        tag("modBC", &[], "0".into()),
                        tag("vBC", &[], "150.00".into()),
                        tag("pICMS", &[], "18.0000".into()),
                        tag("vICMS", &[], "27.00".into()),
                    ])),
                ])),
            ])),
        ]))
    });
}

#[divan::bench]
fn bench_tag_empty(bencher: divan::Bencher) {
    bencher.bench(|| tag("Signature", &[], TagContent::None));
}

// ── Tax Element ──────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_serialize_icms00(bencher: divan::Bencher) {
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
    bencher.bench(|| serialize_tax_element(divan::black_box(&element)));
}

#[divan::bench]
fn bench_serialize_ipi_with_outer_fields(bencher: divan::Bencher) {
    let element = TaxElement {
        outer_tag: Some("IPI".to_string()),
        outer_fields: vec![TaxField::new("cEnq", "999")],
        variant_tag: "IPITrib".to_string(),
        fields: vec![
            TaxField::new("CST", "50"),
            TaxField::new("vBC", "100.00"),
            TaxField::new("pIPI", "5.0000"),
            TaxField::new("vIPI", "5.00"),
        ],
    };
    bencher.bench(|| serialize_tax_element(divan::black_box(&element)));
}

#[divan::bench]
fn bench_serialize_no_outer(bencher: divan::Bencher) {
    let element = TaxElement {
        outer_tag: None,
        outer_fields: vec![],
        variant_tag: "II".to_string(),
        fields: vec![
            TaxField::new("vBC", "1000.00"),
            TaxField::new("vDespAdu", "50.00"),
            TaxField::new("vII", "100.00"),
            TaxField::new("vIOF", "10.00"),
        ],
    };
    bencher.bench(|| serialize_tax_element(divan::black_box(&element)));
}

// ── ICMS Totals ──────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_create_icms_totals(bencher: divan::Bencher) {
    bencher.bench(|| create_icms_totals());
}

#[divan::bench]
fn bench_merge_icms_totals(bencher: divan::Bencher) {
    let source = IcmsTotals {
        v_bc: 10000,
        v_icms: 1800,
        v_icms_deson: 0,
        v_bc_st: 5000,
        v_st: 900,
        v_fcp: 200,
        v_fcp_st: 100,
        v_fcp_st_ret: 0,
        v_fcp_uf_dest: 0,
        v_icms_uf_dest: 0,
        v_icms_uf_remet: 0,
        q_bc_mono: 0,
        v_icms_mono: 0,
        q_bc_mono_reten: 0,
        v_icms_mono_reten: 0,
        q_bc_mono_ret: 0,
        v_icms_mono_ret: 0,
    };
    bencher.bench(|| {
        let mut target = create_icms_totals();
        merge_icms_totals(&mut target, divan::black_box(&source));
        target
    });
}

#[divan::bench]
fn bench_merge_icms_totals_10_items(bencher: divan::Bencher) {
    let source = IcmsTotals {
        v_bc: 10000, v_icms: 1800, v_icms_deson: 0, v_bc_st: 0,
        v_st: 0, v_fcp: 200, v_fcp_st: 0, v_fcp_st_ret: 0,
        v_fcp_uf_dest: 0, v_icms_uf_dest: 0, v_icms_uf_remet: 0,
        q_bc_mono: 0, v_icms_mono: 0, q_bc_mono_reten: 0,
        v_icms_mono_reten: 0, q_bc_mono_ret: 0, v_icms_mono_ret: 0,
    };
    bencher.bench(|| {
        let mut target = create_icms_totals();
        for _ in 0..10 {
            merge_icms_totals(&mut target, divan::black_box(&source));
        }
        target
    });
}

// ── ICMS Builders ────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_build_icms_xml_cst00(bencher: divan::Bencher) {
    let data = IcmsData {
        tax_regime: 3,
        orig: "0".into(),
        cst: Some("00".into()),
        mod_bc: Some("0".into()),
        v_bc: Some(15000),
        p_icms: Some(1800),
        v_icms: Some(2700),
        ..Default::default()
    };
    bencher.bench(|| build_icms_xml(divan::black_box(&data)));
}

#[divan::bench]
fn bench_build_icms_xml_cst10_full(bencher: divan::Bencher) {
    let data = IcmsData {
        tax_regime: 3,
        orig: "0".into(),
        cst: Some("10".into()),
        mod_bc: Some("0".into()),
        v_bc: Some(15000),
        p_icms: Some(1800),
        v_icms: Some(2700),
        mod_bc_st: Some("4".into()),
        p_mva_st: Some(4000),
        v_bc_st: Some(21000),
        p_icms_st: Some(1800),
        v_icms_st: Some(1080),
        v_bc_fcp: Some(15000),
        p_fcp: Some(200),
        v_fcp: Some(300),
        v_bc_fcp_st: Some(21000),
        p_fcp_st: Some(200),
        v_fcp_st: Some(420),
        ..Default::default()
    };
    bencher.bench(|| build_icms_xml(divan::black_box(&data)));
}

#[divan::bench]
fn bench_build_icms_xml_csosn102(bencher: divan::Bencher) {
    let data = IcmsData {
        tax_regime: 1,
        orig: "0".into(),
        csosn: Some("102".into()),
        ..Default::default()
    };
    bencher.bench(|| build_icms_xml(divan::black_box(&data)));
}

#[divan::bench]
fn bench_build_icms_part_xml(bencher: divan::Bencher) {
    let data = IcmsData {
        tax_regime: 3,
        orig: "0".into(),
        cst: Some("10".into()),
        mod_bc: Some("0".into()),
        v_bc: Some(10000),
        p_icms: Some(1200),
        v_icms: Some(1200),
        mod_bc_st: Some("4".into()),
        v_bc_st: Some(14000),
        p_icms_st: Some(1800),
        v_icms_st: Some(720),
        p_bc_op: Some(10000),
        uf_st: Some("SP".into()),
        ..Default::default()
    };
    bencher.bench(|| build_icms_part_xml(divan::black_box(&data)));
}

#[divan::bench]
fn bench_build_icms_uf_dest_xml(bencher: divan::Bencher) {
    let data = IcmsData {
        tax_regime: 3,
        v_bc_uf_dest: Some(10000),
        p_icms_uf_dest: Some(1800),
        p_icms_inter: Some(1200),
        v_icms_uf_dest: Some(600),
        v_icms_uf_remet: Some(0),
        v_fcp_uf_dest: Some(200),
        v_bc_fcp_uf_dest: Some(10000),
        p_fcp_uf_dest: Some(200),
        ..Default::default()
    };
    bencher.bench(|| build_icms_uf_dest_xml(divan::black_box(&data)));
}

// ── PIS/COFINS/IPI/II ────────────────────────────────────────────────────────

#[divan::bench]
fn bench_build_pis_xml_aliq(bencher: divan::Bencher) {
    let data = PisData {
        cst: "01".to_string(),
        v_bc: Some(10000),
        p_pis: Some(16500),
        v_pis: Some(165),
        ..Default::default()
    };
    bencher.bench(|| build_pis_xml(divan::black_box(&data)));
}

#[divan::bench]
fn bench_build_cofins_xml_aliq(bencher: divan::Bencher) {
    let data = CofinsData {
        cst: "01".to_string(),
        v_bc: Some(10000),
        p_cofins: Some(76000),
        v_cofins: Some(760),
        ..Default::default()
    };
    bencher.bench(|| build_cofins_xml(divan::black_box(&data)));
}

#[divan::bench]
fn bench_build_ipi_xml_trib(bencher: divan::Bencher) {
    let data = IpiData {
        cst: "50".to_string(),
        c_enq: "999".to_string(),
        v_bc: Some(10000),
        p_ipi: Some(50000),
        v_ipi: Some(500),
        ..Default::default()
    };
    bencher.bench(|| build_ipi_xml(divan::black_box(&data)));
}

#[divan::bench]
fn bench_build_ii_xml(bencher: divan::Bencher) {
    let data = IiData { v_bc: 100000, v_desp_adu: 5000, v_ii: 10000, v_iof: 1000 };
    bencher.bench(|| build_ii_xml(divan::black_box(&data)));
}

#[divan::bench]
fn bench_build_pis_st_xml(bencher: divan::Bencher) {
    let data = PisStData {
        v_bc: Some(10000),
        p_pis: Some(16500),
        v_pis: 165,
        ..Default::default()
    };
    bencher.bench(|| build_pis_st_xml(divan::black_box(&data)));
}

#[divan::bench]
fn bench_build_cofins_st_xml(bencher: divan::Bencher) {
    let data = CofinsStData {
        v_bc: Some(10000),
        p_cofins: Some(76000),
        v_cofins: 760,
        ..Default::default()
    };
    bencher.bench(|| build_cofins_st_xml(divan::black_box(&data)));
}

// ── ISSQN / IS ───────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_build_issqn_xml(bencher: divan::Bencher) {
    let data = IssqnData {
        v_bc: 50000,
        v_aliq: 500,
        v_issqn: 2500,
        c_mun_fg: "4106902".to_string(),
        c_list_serv: "14.01".to_string(),
        ..Default::default()
    };
    bencher.bench(|| build_issqn_xml(divan::black_box(&data)));
}

#[divan::bench]
fn bench_build_is_xml(bencher: divan::Bencher) {
    let data = IsData {
        cst_is: "01".to_string(),
        c_class_trib_is: "1234567".to_string(),
        v_bc_is: Some("500.00".to_string()),
        p_is: Some("5.0000".to_string()),
        v_is: "25.00".to_string(),
        ..Default::default()
    };
    bencher.bench(|| build_is_xml(divan::black_box(&data)));
}

// ── GTIN ─────────────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_gtin_valid(bencher: divan::Bencher) {
    bencher.bench(|| is_valid_gtin(divan::black_box("7891234567895")));
}

#[divan::bench]
fn bench_gtin_sem_gtin(bencher: divan::Bencher) {
    bencher.bench(|| is_valid_gtin(divan::black_box("SEM GTIN")));
}

// ── State Codes ──────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_get_state_code(bencher: divan::Bencher) {
    bencher.bench(|| get_state_code(divan::black_box("PR")));
}

#[divan::bench]
fn bench_get_state_by_code(bencher: divan::Bencher) {
    bencher.bench(|| get_state_by_code(divan::black_box("41")));
}

#[divan::bench]
fn bench_state_code_all_27(bencher: divan::Bencher) {
    let states = [
        "AC", "AL", "AP", "AM", "BA", "CE", "DF", "ES", "GO",
        "MA", "MT", "MS", "MG", "PA", "PB", "PR", "PE", "PI",
        "RJ", "RN", "RS", "RO", "RR", "SC", "SP", "SE", "TO",
    ];
    bencher.bench(|| {
        for uf in &states {
            let _ = get_state_code(divan::black_box(uf));
        }
    });
}

// ── Format Utils (missing) ──────────────────────────────────────────────────

#[divan::bench]
fn bench_format_cents_10(bencher: divan::Bencher) {
    bencher.bench(|| format_cents_10(divan::black_box(150_00)));
}

#[divan::bench]
fn bench_format_decimal(bencher: divan::Bencher) {
    bencher.bench(|| format_decimal(divan::black_box(1.5678), 4));
}

#[divan::bench]
fn bench_format_rate(bencher: divan::Bencher) {
    bencher.bench(|| format_rate(divan::black_box(1800), 4));
}

#[divan::bench]
fn bench_format_cents_or_none_some(bencher: divan::Bencher) {
    bencher.bench(|| format_cents_or_none(divan::black_box(Some(9999)), 2));
}

#[divan::bench]
fn bench_format_cents_or_none_none(bencher: divan::Bencher) {
    bencher.bench(|| format_cents_or_none(divan::black_box(None), 2));
}

#[divan::bench]
fn bench_format_rate4_or_zero_some(bencher: divan::Bencher) {
    bencher.bench(|| format_rate4_or_zero(divan::black_box(Some(16500))));
}

#[divan::bench]
fn bench_format_rate4_or_zero_none(bencher: divan::Bencher) {
    bencher.bench(|| format_rate4_or_zero(divan::black_box(None)));
}

// ── XML Utils (missing) ─────────────────────────────────────────────────────

#[divan::bench]
fn bench_extract_xml_tag_value(bencher: divan::Bencher) {
    let xml = "<nfeProc><protNFe><infProt><cStat>100</cStat><xMotivo>Autorizado</xMotivo></infProt></protNFe></nfeProc>";
    bencher.bench(|| extract_xml_tag_value(divan::black_box(xml), divan::black_box("cStat")));
}

#[divan::bench]
fn bench_extract_xml_tag_value_not_found(bencher: divan::Bencher) {
    let xml = "<nfe><infNFe><ide><cUF>41</cUF></ide></infNFe></nfe>";
    bencher.bench(|| extract_xml_tag_value(divan::black_box(xml), divan::black_box("missing")));
}

// ── Tax Element (missing) ───────────────────────────────────────────────────

#[divan::bench]
fn bench_filter_fields_mixed(bencher: divan::Bencher) {
    bencher.bench(|| {
        let fields: Vec<Option<TaxField>> = vec![
            Some(TaxField::new("orig", "0")),
            None,
            Some(TaxField::new("CST", "00")),
            None,
            Some(TaxField::new("modBC", "0")),
            None,
            Some(TaxField::new("vBC", "150.00")),
        ];
        filter_fields(divan::black_box(fields))
    });
}

// ── ICMS ST Builder (missing) ───────────────────────────────────────────────

#[divan::bench]
fn bench_build_icms_st_xml(bencher: divan::Bencher) {
    let data = IcmsData {
        tax_regime: 3,
        orig: "0".into(),
        cst: Some("60".into()),
        v_bc_st_ret: Some(15000),
        v_icms_st_ret: Some(2700),
        v_bc_st_dest: Some(12000),
        v_icms_st_dest: Some(2160),
        v_bc_fcp_st_ret: Some(15000),
        p_fcp_st_ret: Some(200),
        v_fcp_st_ret: Some(300),
        ..Default::default()
    };
    bencher.bench(|| build_icms_st_xml(divan::black_box(&data)));
}

// ── ISSQN (missing) ─────────────────────────────────────────────────────────

#[divan::bench]
fn bench_build_issqn_xml_with_totals(bencher: divan::Bencher) {
    let data = IssqnData {
        v_bc: 50000,
        v_aliq: 500,
        v_issqn: 2500,
        c_mun_fg: "4106902".to_string(),
        c_list_serv: "14.01".to_string(),
        v_deducao: Some(1000),
        v_iss_ret: Some(500),
        ..Default::default()
    };
    bencher.bench(|| {
        let mut totals = create_issqn_totals();
        build_issqn_xml_with_totals(divan::black_box(&data), &mut totals)
    });
}

#[divan::bench]
fn bench_create_issqn_totals(bencher: divan::Bencher) {
    bencher.bench(|| create_issqn_totals());
}

#[divan::bench]
fn bench_build_imposto_devol(bencher: divan::Bencher) {
    bencher.bench(|| build_imposto_devol(divan::black_box(10000), divan::black_box(500)));
}

// ── GTIN (missing) ──────────────────────────────────────────────────────────

#[divan::bench]
fn bench_gtin_calculate_check_digit(bencher: divan::Bencher) {
    bencher.bench(|| calculate_check_digit(divan::black_box("7891234567895")));
}

// ── Access Key ──────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_build_access_key(bencher: divan::Bencher) {
    let params = AccessKeyParams {
        state_code: "41".to_string(),
        year_month: "2603".to_string(),
        tax_id: "04123456000190".to_string(),
        model: InvoiceModel::Nfe,
        series: 1,
        number: 123,
        emission_type: EmissionType::Normal,
        numeric_code: "12345678".to_string(),
    };
    bencher.bench(|| build_access_key(divan::black_box(&params)));
}

#[divan::bench]
fn bench_calculate_mod11(bencher: divan::Bencher) {
    bencher.bench(|| {
        calculate_mod11(divan::black_box(
            "4125030412345678901255001000000001100000001",
        ))
    });
}

#[divan::bench]
fn bench_generate_numeric_code(bencher: divan::Bencher) {
    bencher.bench(|| generate_numeric_code());
}

#[divan::bench]
fn bench_format_year_month(bencher: divan::Bencher) {
    let dt = chrono::FixedOffset::west_opt(3 * 3600)
        .unwrap()
        .with_ymd_and_hms(2026, 3, 11, 10, 30, 0)
        .unwrap();
    bencher.bench(|| format_year_month(divan::black_box(&dt)));
}

// ── TaxId ───────────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_tax_id_cpf(bencher: divan::Bencher) {
    bencher.bench(|| {
        let tid = TaxId::new(divan::black_box("12345678901"));
        let _ = tid.is_cpf();
        let _ = tid.tag_name();
        let _ = tid.padded();
    });
}

#[divan::bench]
fn bench_tax_id_cnpj(bencher: divan::Bencher) {
    bencher.bench(|| {
        let tid = TaxId::new(divan::black_box("04123456000190"));
        let _ = tid.is_cpf();
        let _ = tid.tag_name();
        let _ = tid.padded();
    });
}

#[divan::bench]
fn bench_tax_id_to_xml_tag(bencher: divan::Bencher) {
    bencher.bench(|| {
        let tid = TaxId::new(divan::black_box("04123456000190"));
        tid.to_xml_tag()
    });
}

// ── IDE Builder ─────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_format_datetime_nfe(bencher: divan::Bencher) {
    let dt = chrono::FixedOffset::west_opt(3 * 3600)
        .unwrap()
        .with_ymd_and_hms(2026, 3, 11, 10, 30, 0)
        .unwrap();
    bencher.bench(|| format_datetime_nfe(divan::black_box(&dt), divan::black_box("PR")));
}

#[divan::bench]
fn bench_build_ide(bencher: divan::Bencher) {
    let data = make_sample_invoice_data();
    let access_key = "41260304123456000190550010000001231123456780";
    bencher.bench(|| {
        build_ide(
            divan::black_box(&data),
            divan::black_box("41"),
            divan::black_box("12345678"),
            divan::black_box(access_key),
        )
    });
}

// ── Emit Builder ────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_build_emit(bencher: divan::Bencher) {
    let data = make_sample_invoice_data();
    bencher.bench(|| build_emit(divan::black_box(&data)));
}

#[divan::bench]
fn bench_build_address_fields(bencher: divan::Bencher) {
    bencher.bench(|| {
        build_address_fields(
            divan::black_box("Rua das Flores"),
            divan::black_box("123"),
            divan::black_box(Some("Sala 1")),
            divan::black_box("Centro"),
            divan::black_box("4106902"),
            divan::black_box("Curitiba"),
            divan::black_box("PR"),
            divan::black_box(Some("80000000")),
            divan::black_box(true),
        )
    });
}

// ── Dest Builder ────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_build_dest_nfe(bencher: divan::Bencher) {
    let data = make_sample_invoice_data();
    bencher.bench(|| build_dest(divan::black_box(&data)));
}

#[divan::bench]
fn bench_build_dest_nfce(bencher: divan::Bencher) {
    let mut data = make_sample_invoice_data();
    data.model = InvoiceModel::Nfce;
    bencher.bench(|| build_dest(divan::black_box(&data)));
}

// ── Det Builder ─────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_build_det_simple(bencher: divan::Bencher) {
    let data = make_sample_invoice_data();
    let item = &data.items[0];
    bencher.bench(|| build_det(divan::black_box(item), divan::black_box(&data)));
}

// ── Total Builder ───────────────────────────────────────────────────────────

#[divan::bench]
fn bench_build_total(bencher: divan::Bencher) {
    let icms = IcmsTotals {
        v_bc: 15000, v_icms: 2700, v_icms_deson: 0,
        v_bc_st: 0, v_st: 0, v_fcp: 0, v_fcp_st: 0,
        v_fcp_st_ret: 0, v_fcp_uf_dest: 0,
        v_icms_uf_dest: 0, v_icms_uf_remet: 0,
        q_bc_mono: 0, v_icms_mono: 0,
        q_bc_mono_reten: 0, v_icms_mono_reten: 0,
        q_bc_mono_ret: 0, v_icms_mono_ret: 0,
    };
    let other = OtherTotals { v_ipi: 500, v_pis: 165, v_cofins: 760, v_ii: 0 };
    bencher.bench(|| {
        build_total(
            divan::black_box(15000),
            divan::black_box(&icms),
            divan::black_box(&other),
            divan::black_box(None),
        )
    });
}

// ── Transp Builder ──────────────────────────────────────────────────────────

#[divan::bench]
fn bench_build_transp_no_transport(bencher: divan::Bencher) {
    let mut data = make_sample_invoice_data();
    data.transport = None;
    bencher.bench(|| build_transp(divan::black_box(&data)));
}

#[divan::bench]
fn bench_build_transp_with_carrier(bencher: divan::Bencher) {
    let data = make_sample_invoice_data_with_transport();
    bencher.bench(|| build_transp(divan::black_box(&data)));
}

// ── Pag Builder ─────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_build_pag_cash(bencher: divan::Bencher) {
    let payments = vec![PaymentData { method: "01".to_string(), amount: 15000 }];
    bencher.bench(|| {
        build_pag(divan::black_box(&payments), divan::black_box(None), divan::black_box(None))
    });
}

#[divan::bench]
fn bench_build_pag_with_change(bencher: divan::Bencher) {
    let payments = vec![PaymentData { method: "01".to_string(), amount: 20000 }];
    bencher.bench(|| {
        build_pag(
            divan::black_box(&payments),
            divan::black_box(Some(5000)),
            divan::black_box(None),
        )
    });
}

#[divan::bench]
fn bench_build_pag_empty(bencher: divan::Bencher) {
    let payments: Vec<PaymentData> = vec![];
    bencher.bench(|| {
        build_pag(divan::black_box(&payments), divan::black_box(None), divan::black_box(None))
    });
}

// ── Optional Builders ───────────────────────────────────────────────────────

#[divan::bench]
fn bench_build_cobr(bencher: divan::Bencher) {
    let billing = BillingData {
        invoice: Some(BillingInvoice {
            number: "001".to_string(),
            original_value: 15000,
            discount_value: Some(500),
            net_value: 14500,
        }),
        installments: Some(vec![
            Installment { number: "001".to_string(), due_date: "2026-04-11".to_string(), value: 7250 },
            Installment { number: "002".to_string(), due_date: "2026-05-11".to_string(), value: 7250 },
        ]),
    };
    bencher.bench(|| build_cobr(divan::black_box(&billing)));
}

#[divan::bench]
fn bench_build_inf_adic_homologation(bencher: divan::Bencher) {
    let data = make_sample_invoice_data();
    bencher.bench(|| build_inf_adic(divan::black_box(&data)));
}

#[divan::bench]
fn bench_build_intermediary(bencher: divan::Bencher) {
    let intermed = IntermediaryData {
        tax_id: "04123456000190".to_string(),
        id_cad_int_tran: Some("ABC12345".to_string()),
    };
    bencher.bench(|| build_intermediary(divan::black_box(&intermed)));
}

#[divan::bench]
fn bench_build_tech_responsible(bencher: divan::Bencher) {
    let tech = TechResponsibleData {
        tax_id: "14363848000190".to_string(),
        contact: "Solusys".to_string(),
        email: "contato@solusys.com.br".to_string(),
        phone: Some("4332341234".to_string()),
    };
    bencher.bench(|| build_tech_responsible(divan::black_box(&tech)));
}

#[divan::bench]
fn bench_build_purchase(bencher: divan::Bencher) {
    let purchase = PurchaseData {
        order_number: Some("PO-2026-001".to_string()),
        contract_number: Some("CT-999".to_string()),
        purchase_note: Some("NE-123".to_string()),
    };
    bencher.bench(|| build_purchase(divan::black_box(&purchase)));
}

#[divan::bench]
fn bench_build_export(bencher: divan::Bencher) {
    let exp = ExportData {
        exit_state: "PR".to_string(),
        export_location: "Porto de Paranagua".to_string(),
        dispatch_location: Some("Terminal de Cargas".to_string()),
    };
    bencher.bench(|| build_export(divan::black_box(&exp)));
}

#[divan::bench]
fn bench_build_withdrawal(bencher: divan::Bencher) {
    let loc = make_sample_location();
    bencher.bench(|| build_withdrawal(divan::black_box(&loc)));
}

#[divan::bench]
fn bench_build_delivery(bencher: divan::Bencher) {
    let loc = make_sample_location();
    bencher.bench(|| build_delivery(divan::black_box(&loc)));
}

#[divan::bench]
fn bench_build_aut_xml(bencher: divan::Bencher) {
    let auth = AuthorizedXml { tax_id: "04123456000190".to_string() };
    bencher.bench(|| build_aut_xml(divan::black_box(&auth)));
}

// ── Full Invoice XML Builder ────────────────────────────────────────────────

#[divan::bench]
fn bench_build_invoice_xml_simple(bencher: divan::Bencher) {
    let data = make_sample_invoice_data();
    bencher.bench(|| build_invoice_xml(divan::black_box(&data)));
}

#[divan::bench]
fn bench_build_invoice_xml_10_items(bencher: divan::Bencher) {
    let data = make_sample_invoice_data_10_items();
    bencher.bench(|| build_invoice_xml(divan::black_box(&data)));
}

// ── Complement ──────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_attach_protocol(bencher: divan::Bencher) {
    let request_xml = make_sample_signed_nfe_xml();
    let response_xml = make_sample_autorizacao_response_xml();
    bencher.bench(|| {
        attach_protocol(
            divan::black_box(&request_xml),
            divan::black_box(&response_xml),
        )
    });
}

#[divan::bench]
fn bench_attach_inutilizacao(bencher: divan::Bencher) {
    let request_xml = concat!(
        r#"<inutNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00">"#,
        r#"<infInut Id="ID41260304123456000190550010000001001000000100">"#,
        "<tpAmb>2</tpAmb><xServ>INUTILIZAR</xServ>",
        "<cUF>41</cUF><ano>26</ano><CNPJ>04123456000190</CNPJ>",
        "<mod>55</mod><serie>1</serie><nNFIni>100</nNFIni><nNFFin>100</nNFFin>",
        "<xJust>Teste de inutilizacao benchmark</xJust>",
        "</infInut></inutNFe>",
    );
    let response_xml = concat!(
        r#"<retInutNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00">"#,
        "<infInut><tpAmb>2</tpAmb><cStat>102</cStat>",
        "<xMotivo>Inutilizacao de numero homologado</xMotivo>",
        "</infInut></retInutNFe>",
    );
    bencher.bench(|| {
        attach_inutilizacao(
            divan::black_box(request_xml),
            divan::black_box(response_xml),
        )
    });
}

#[divan::bench]
fn bench_attach_event_protocol(bencher: divan::Bencher) {
    let request_xml = concat!(
        r#"<envEvento xmlns="http://www.portalfiscal.inf.br/nfe" versao="1.00">"#,
        "<idLote>1</idLote>",
        r#"<evento xmlns="http://www.portalfiscal.inf.br/nfe" versao="1.00">"#,
        r#"<infEvento Id="ID11011141260304123456000190550010000001231123456780101">"#,
        "<cOrgao>91</cOrgao><tpAmb>2</tpAmb>",
        "<CNPJ>04123456000190</CNPJ>",
        "<chNFe>41260304123456000190550010000001231123456780</chNFe>",
        "<dhEvento>2026-03-11T10:30:00-03:00</dhEvento>",
        "<tpEvento>110111</tpEvento><nSeqEvento>1</nSeqEvento>",
        "<verEvento>1.00</verEvento>",
        r#"<detEvento versao="1.00">"#,
        "<descEvento>Cancelamento</descEvento>",
        "<nProt>141260000012345</nProt>",
        "<xJust>Cancelamento por erro de digitacao</xJust>",
        "</detEvento></infEvento></evento></envEvento>",
    );
    let response_xml = concat!(
        r#"<retEvento xmlns="http://www.portalfiscal.inf.br/nfe" versao="1.00">"#,
        "<infEvento>",
        "<cStat>135</cStat>",
        "<xMotivo>Evento registrado e vinculado a NF-e</xMotivo>",
        "<nProt>141260000099999</nProt>",
        "</infEvento></retEvento>",
    );
    bencher.bench(|| {
        attach_event_protocol(
            divan::black_box(request_xml),
            divan::black_box(response_xml),
        )
    });
}

#[divan::bench]
fn bench_attach_b2b(bencher: divan::Bencher) {
    let nfe_proc_xml = concat!(
        r#"<?xml version="1.0" encoding="UTF-8"?>"#,
        r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">"#,
        r#"<NFe><infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
        "<ide><cUF>41</cUF></ide></infNFe></NFe>",
        r#"<protNFe versao="4.00"><infProt>"#,
        "<cStat>100</cStat><nProt>141260000012345</nProt>",
        "</infProt></protNFe></nfeProc>",
    );
    let b2b_xml = "<NFeB2BFin><ideNFe>test data</ideNFe></NFeB2BFin>";
    bencher.bench(|| {
        attach_b2b(
            divan::black_box(nfe_proc_xml),
            divan::black_box(b2b_xml),
            divan::black_box(None),
        )
    });
}

// ── QR Code ─────────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_build_nfce_qr_code_url_v300_online(bencher: divan::Bencher) {
    let params = NfceQrCodeParams {
        access_key: "41260304123456000190650010000001231123456780".to_string(),
        version: QrCodeVersion::V300,
        environment: SefazEnvironment::Homologation,
        emission_type: EmissionType::Normal,
        qr_code_base_url: "http://www.fazenda.pr.gov.br/nfce/qrcode".to_string(),
        csc_token: None,
        csc_id: None,
        issued_at: None,
        total_value: None,
        total_icms: None,
        digest_value: None,
        dest_document: None,
        dest_id_type: None,
    };
    bencher.bench(|| build_nfce_qr_code_url(divan::black_box(&params)));
}

#[divan::bench]
fn bench_build_nfce_qr_code_url_v200_online(bencher: divan::Bencher) {
    let params = NfceQrCodeParams {
        access_key: "41260304123456000190650010000001231123456780".to_string(),
        version: QrCodeVersion::V200,
        environment: SefazEnvironment::Homologation,
        emission_type: EmissionType::Normal,
        qr_code_base_url: "http://www.fazenda.pr.gov.br/nfce/qrcode".to_string(),
        csc_token: Some("000001".to_string()),
        csc_id: Some("ABCDEF0123456789ABCDEF0123456789".to_string()),
        issued_at: None,
        total_value: None,
        total_icms: None,
        digest_value: None,
        dest_document: None,
        dest_id_type: None,
    };
    bencher.bench(|| build_nfce_qr_code_url(divan::black_box(&params)));
}

#[divan::bench]
fn bench_build_nfce_consult_url(bencher: divan::Bencher) {
    bencher.bench(|| {
        build_nfce_consult_url(
            divan::black_box("http://www.fazenda.pr.gov.br/nfce/consulta"),
            divan::black_box("41260304123456000190650010000001231123456780"),
            divan::black_box(SefazEnvironment::Homologation),
        )
    });
}

#[divan::bench]
fn bench_put_qr_tag(bencher: divan::Bencher) {
    let xml = make_sample_nfce_signed_xml();
    let params = PutQRTagParams {
        xml: xml.clone(),
        csc_token: String::new(),
        csc_id: String::new(),
        version: "300".to_string(),
        qr_code_base_url: "http://www.fazenda.pr.gov.br/nfce/qrcode".to_string(),
        url_chave: "http://www.fazenda.pr.gov.br/nfce/consulta".to_string(),
    };
    bencher.bench(|| put_qr_tag(divan::black_box(&params)));
}

// ── Contingency ─────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_contingency_for_state_pr(bencher: divan::Bencher) {
    bencher.bench(|| contingency_for_state(divan::black_box("PR")));
}

#[divan::bench]
fn bench_contingency_for_state_sp(bencher: divan::Bencher) {
    bencher.bench(|| contingency_for_state(divan::black_box("SP")));
}

#[divan::bench]
fn bench_contingency_load_json(bencher: divan::Bencher) {
    let json = r#"{"motive":"Testes de contingencia para benchmark","timestamp":1480700623,"type":"SVCAN","tpEmis":6}"#;
    bencher.bench(|| Contingency::load(divan::black_box(json)));
}

#[divan::bench]
fn bench_contingency_activate(bencher: divan::Bencher) {
    bencher.bench(|| {
        let mut c = Contingency::new();
        c.activate(
            divan::black_box(ContingencyType::SvcAn),
            divan::black_box("Indisponibilidade do servico de autorizacao SEFAZ PR"),
        )
    });
}

#[divan::bench]
fn bench_adjust_nfe_contingency(bencher: divan::Bencher) {
    let xml = make_sample_nfe_xml_for_contingency();
    let mut contingency = Contingency::new();
    contingency.activate(
        ContingencyType::SvcAn,
        "Indisponibilidade do servico de autorizacao SEFAZ PR",
    ).unwrap();
    bencher.bench(|| {
        adjust_nfe_contingency(
            divan::black_box(&xml),
            divan::black_box(&contingency),
        )
    });
}

// ── SEFAZ Request Builders ──────────────────────────────────────────────────

#[divan::bench]
fn bench_build_autorizacao_request(bencher: divan::Bencher) {
    let signed_xml = make_sample_signed_nfe_xml();
    bencher.bench(|| {
        build_autorizacao_request(
            divan::black_box(&signed_xml),
            divan::black_box("202603111030"),
            divan::black_box(true),
            divan::black_box(false),
        )
    });
}

#[divan::bench]
fn bench_build_status_request(bencher: divan::Bencher) {
    bencher.bench(|| {
        build_status_request(
            divan::black_box("PR"),
            divan::black_box(SefazEnvironment::Homologation),
        )
    });
}

#[divan::bench]
fn bench_build_consulta_request(bencher: divan::Bencher) {
    bencher.bench(|| {
        build_consulta_request(
            divan::black_box("41260304123456000190550010000001231123456780"),
            divan::black_box(SefazEnvironment::Homologation),
        )
    });
}

#[divan::bench]
fn bench_build_cancela_request(bencher: divan::Bencher) {
    bencher.bench(|| {
        build_cancela_request(
            divan::black_box("41260304123456000190550010000001231123456780"),
            divan::black_box("141260000012345"),
            divan::black_box("Erro de digitacao no valor do produto"),
            divan::black_box(1),
            divan::black_box(SefazEnvironment::Homologation),
            divan::black_box("04123456000190"),
        )
    });
}

#[divan::bench]
fn bench_build_inutilizacao_request(bencher: divan::Bencher) {
    bencher.bench(|| {
        build_inutilizacao_request(
            divan::black_box(26),
            divan::black_box("04123456000190"),
            divan::black_box("55"),
            divan::black_box(1),
            divan::black_box(100),
            divan::black_box(110),
            divan::black_box("Numeracao pulada por erro de sistema"),
            divan::black_box(SefazEnvironment::Homologation),
            divan::black_box("PR"),
        )
    });
}

#[divan::bench]
fn bench_build_cce_request(bencher: divan::Bencher) {
    bencher.bench(|| {
        build_cce_request(
            divan::black_box("41260304123456000190550010000001231123456780"),
            divan::black_box("Correcao do endereco do destinatario: Rua XV de Novembro 1000"),
            divan::black_box(1),
            divan::black_box(SefazEnvironment::Homologation),
            divan::black_box("04123456000190"),
        )
    });
}

#[divan::bench]
fn bench_build_dist_dfe_request(bencher: divan::Bencher) {
    bencher.bench(|| {
        build_dist_dfe_request(
            divan::black_box("PR"),
            divan::black_box("04123456000190"),
            divan::black_box(None),
            divan::black_box(None),
            divan::black_box(SefazEnvironment::Homologation),
        )
    });
}

#[divan::bench]
fn bench_build_cadastro_request(bencher: divan::Bencher) {
    bencher.bench(|| {
        build_cadastro_request(
            divan::black_box("PR"),
            divan::black_box("CNPJ"),
            divan::black_box("04123456000190"),
        )
    });
}

// ── SEFAZ URLs ──────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_get_sefaz_url_pr_autorizacao(bencher: divan::Bencher) {
    bencher.bench(|| {
        get_sefaz_url(
            divan::black_box("PR"),
            divan::black_box(SefazEnvironment::Homologation),
            divan::black_box("NfeAutorizacao"),
        )
    });
}

#[divan::bench]
fn bench_get_sefaz_url_sp_status(bencher: divan::Bencher) {
    bencher.bench(|| {
        get_sefaz_url(
            divan::black_box("SP"),
            divan::black_box(SefazEnvironment::Production),
            divan::black_box("NfeStatusServico"),
        )
    });
}

#[divan::bench]
fn bench_get_sefaz_url_svrs_state(bencher: divan::Bencher) {
    bencher.bench(|| {
        get_sefaz_url(
            divan::black_box("SC"),
            divan::black_box(SefazEnvironment::Homologation),
            divan::black_box("NfeAutorizacao"),
        )
    });
}

#[divan::bench]
fn bench_get_nfce_consult_url_pr(bencher: divan::Bencher) {
    bencher.bench(|| {
        get_nfce_consult_url_sefaz(
            divan::black_box("PR"),
            divan::black_box(SefazEnvironment::Homologation),
        )
    });
}

// ── Standardize ─────────────────────────────────────────────────────────────

#[divan::bench]
fn bench_identify_xml_type_nfe(bencher: divan::Bencher) {
    let xml = r#"<?xml version="1.0"?><NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780"><ide><cUF>41</cUF></ide></infNFe></NFe>"#;
    bencher.bench(|| identify_xml_type(divan::black_box(xml)));
}

#[divan::bench]
fn bench_identify_xml_type_nfe_proc(bencher: divan::Bencher) {
    let xml = r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe"><NFe><infNFe><ide><cUF>41</cUF></ide></infNFe></NFe><protNFe><infProt><cStat>100</cStat></infProt></protNFe></nfeProc>"#;
    bencher.bench(|| identify_xml_type(divan::black_box(xml)));
}

#[divan::bench]
fn bench_identify_xml_type_ret_cons(bencher: divan::Bencher) {
    let xml = r#"<retConsStatServ xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><tpAmb>2</tpAmb><cStat>107</cStat><xMotivo>Servico em Operacao</xMotivo><cUF>41</cUF></retConsStatServ>"#;
    bencher.bench(|| identify_xml_type(divan::black_box(xml)));
}

#[divan::bench]
fn bench_xml_to_json_small(bencher: divan::Bencher) {
    let xml = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe123"><ide><cUF>41</cUF><natOp>VENDA</natOp><mod>55</mod><serie>1</serie><nNF>123</nNF></ide></infNFe></NFe>"#;
    bencher.bench(|| xml_to_json(divan::black_box(xml)));
}

#[divan::bench]
fn bench_xml_to_json_medium(bencher: divan::Bencher) {
    let xml = make_sample_nfe_xml_for_json();
    bencher.bench(|| xml_to_json(divan::black_box(&xml)));
}

// ── Response Parsers ────────────────────────────────────────────────────────

#[divan::bench]
fn bench_parse_autorizacao_response_with_protocol(bencher: divan::Bencher) {
    let xml = concat!(
        "<retEnviNFe><cStat>104</cStat>",
        r#"<protNFe versao="4.00"><infProt>"#,
        "<cStat>100</cStat>",
        "<xMotivo>Autorizado o uso da NF-e</xMotivo>",
        "<nProt>141260000012345</nProt>",
        "<dhRecbto>2026-03-11T10:30:00-03:00</dhRecbto>",
        "</infProt></protNFe></retEnviNFe>",
    );
    bencher.bench(|| parse_autorizacao_response(divan::black_box(xml)));
}

#[divan::bench]
fn bench_parse_autorizacao_response_batch_only(bencher: divan::Bencher) {
    let xml = "<retEnviNFe><cStat>105</cStat><xMotivo>Lote em processamento</xMotivo></retEnviNFe>";
    bencher.bench(|| parse_autorizacao_response(divan::black_box(xml)));
}

#[divan::bench]
fn bench_parse_autorizacao_response_soap_wrapped(bencher: divan::Bencher) {
    let xml = concat!(
        r#"<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">"#,
        "<soap:Body>",
        r#"<nfeResultMsg:nfeAutorizacaoLoteResult xmlns:nfeResultMsg="http://www.portalfiscal.inf.br/nfe/wsdl/NFeAutorizacao4">"#,
        r#"<nfe:retEnviNFe xmlns:nfe="http://www.portalfiscal.inf.br/nfe">"#,
        "<nfe:cStat>104</nfe:cStat>",
        r#"<nfe:protNFe versao="4.00">"#,
        "<nfe:infProt>",
        "<nfe:cStat>100</nfe:cStat>",
        "<nfe:xMotivo>Autorizado o uso da NF-e</nfe:xMotivo>",
        "<nfe:nProt>141260000012345</nfe:nProt>",
        "<nfe:dhRecbto>2026-03-11T10:30:00-03:00</nfe:dhRecbto>",
        "</nfe:infProt>",
        "</nfe:protNFe>",
        "</nfe:retEnviNFe>",
        "</nfeResultMsg:nfeAutorizacaoLoteResult>",
        "</soap:Body></soap:Envelope>",
    );
    bencher.bench(|| parse_autorizacao_response(divan::black_box(xml)));
}

#[divan::bench]
fn bench_parse_status_response(bencher: divan::Bencher) {
    let xml = concat!(
        "<retConsStatServ><cStat>107</cStat>",
        "<xMotivo>Servico em Operacao</xMotivo>",
        "<tMed>1</tMed></retConsStatServ>",
    );
    bencher.bench(|| parse_status_response(divan::black_box(xml)));
}

#[divan::bench]
fn bench_parse_status_response_soap(bencher: divan::Bencher) {
    let xml = concat!(
        r#"<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">"#,
        "<soap:Body>",
        r#"<nfe:retConsStatServ xmlns:nfe="http://www.portalfiscal.inf.br/nfe">"#,
        "<nfe:cStat>107</nfe:cStat>",
        "<nfe:xMotivo>Servico em Operacao</nfe:xMotivo>",
        "<nfe:tMed>2</nfe:tMed>",
        "</nfe:retConsStatServ>",
        "</soap:Body></soap:Envelope>",
    );
    bencher.bench(|| parse_status_response(divan::black_box(xml)));
}

#[divan::bench]
fn bench_parse_cancellation_response(bencher: divan::Bencher) {
    let xml = concat!(
        "<retEvento><infEvento>",
        "<cStat>135</cStat>",
        "<xMotivo>Evento registrado e vinculado a NF-e</xMotivo>",
        "<nProt>141260000099999</nProt>",
        "</infEvento></retEvento>",
    );
    bencher.bench(|| parse_cancellation_response(divan::black_box(xml)));
}

// ── Certificate (sign_xml) ──────────────────────────────────────────────────

#[divan::bench]
fn bench_sign_xml(bencher: divan::Bencher) {
    // Generate a self-signed RSA key pair for benchmarking
    let (private_key_pem, cert_pem) = make_test_keypair();
    let xml = make_sample_unsigned_nfe_xml();
    bencher.bench(|| {
        sign_xml(
            divan::black_box(&xml),
            divan::black_box(&private_key_pem),
            divan::black_box(&cert_pem),
        )
    });
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn make_sample_issuer() -> IssuerData {
    IssuerData {
        tax_id: "04123456000190".to_string(),
        state_tax_id: "9012345678".to_string(),
        company_name: "Auto Eletrica Barbosa LTDA".to_string(),
        trade_name: Some("Auto Eletrica Barbosa".to_string()),
        tax_regime: TaxRegime::Normal,
        state_code: "PR".to_string(),
        city_code: "4106902".to_string(),
        city_name: "Curitiba".to_string(),
        street: "Rua XV de Novembro".to_string(),
        street_number: "1000".to_string(),
        district: "Centro".to_string(),
        zip_code: "80020310".to_string(),
        address_complement: None,
    }
}

fn make_sample_recipient() -> RecipientData {
    RecipientData {
        tax_id: "12345678901234".to_string(),
        name: "Cliente Teste LTDA".to_string(),
        state_code: Some("PR".to_string()),
        state_tax_id: Some("1234567890".to_string()),
        street: Some("Rua das Flores".to_string()),
        street_number: Some("500".to_string()),
        district: Some("Batel".to_string()),
        city_code: Some("4106902".to_string()),
        city_name: Some("Curitiba".to_string()),
        zip_code: Some("80420120".to_string()),
        complement: None,
    }
}

fn make_sample_item(n: u32) -> InvoiceItemData {
    InvoiceItemData {
        item_number: n,
        product_code: format!("{n:03}"),
        description: "Servico de eletrica automotiva".to_string(),
        ncm: "00000000".to_string(),
        cfop: "5102".to_string(),
        unit_of_measure: "UN".to_string(),
        quantity: 1.0,
        unit_price: 15000,
        total_price: 15000,
        c_ean: None,
        c_ean_trib: None,
        cest: None,
        v_frete: None,
        v_seg: None,
        v_desc: None,
        v_outro: None,
        orig: Some("0".to_string()),
        icms_cst: "00".to_string(),
        icms_rate: 1800,
        icms_amount: 2700,
        icms_mod_bc: Some(0),
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
        pis_cst: "01".to_string(),
        pis_v_bc: Some(15000),
        pis_p_pis: Some(16500),
        pis_v_pis: Some(248),
        pis_q_bc_prod: None,
        pis_v_aliq_prod: None,
        cofins_cst: "01".to_string(),
        cofins_v_bc: Some(15000),
        cofins_p_cofins: Some(76000),
        cofins_v_cofins: Some(1140),
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

fn make_sample_invoice_data() -> InvoiceBuildData {
    let dt = chrono::FixedOffset::west_opt(3 * 3600)
        .unwrap()
        .with_ymd_and_hms(2026, 3, 11, 10, 30, 0)
        .unwrap();

    InvoiceBuildData {
        model: InvoiceModel::Nfe,
        series: 1,
        number: 123,
        emission_type: EmissionType::Normal,
        environment: SefazEnvironment::Homologation,
        issued_at: dt,
        operation_nature: "VENDA".to_string(),
        issuer: make_sample_issuer(),
        recipient: Some(make_sample_recipient()),
        items: vec![make_sample_item(1)],
        payments: vec![PaymentData { method: "01".to_string(), amount: 15000 }],
        change_amount: None,
        payment_card_details: None,
        contingency: None,
        operation_type: Some(1),
        purpose_code: Some(1),
        intermediary_indicator: None,
        emission_process: None,
        consumer_type: None,
        buyer_presence: None,
        print_format: None,
        references: None,
        transport: None,
        billing: None,
        withdrawal: None,
        delivery: None,
        authorized_xml: None,
        additional_info: None,
        intermediary: None,
        ret_trib: None,
        tech_responsible: None,
        purchase: None,
        export: None,
    }
}

fn make_sample_invoice_data_10_items() -> InvoiceBuildData {
    let mut data = make_sample_invoice_data();
    data.items = (1..=10).map(|n| make_sample_item(n)).collect();
    data.payments = vec![PaymentData { method: "01".to_string(), amount: 150000 }];
    data
}

fn make_sample_invoice_data_with_transport() -> InvoiceBuildData {
    let mut data = make_sample_invoice_data();
    data.transport = Some(TransportData {
        freight_mode: "0".to_string(),
        carrier: Some(CarrierData {
            tax_id: Some("04123456000190".to_string()),
            name: Some("Transportadora ABC".to_string()),
            state_tax_id: Some("1234567890".to_string()),
            state_code: Some("PR".to_string()),
            address: Some("Rua dos Transportes 100".to_string()),
        }),
        vehicle: Some(VehicleData {
            plate: "ABC1D23".to_string(),
            state_code: "PR".to_string(),
            rntc: Some("12345".to_string()),
        }),
        trailers: None,
        volumes: Some(vec![VolumeData {
            quantity: Some(1),
            species: Some("CAIXA".to_string()),
            brand: Some("MARCA".to_string()),
            number: None,
            net_weight: Some(10.5),
            gross_weight: Some(12.0),
            seals: None,
        }]),
        retained_icms: None,
    });
    data
}

fn make_sample_location() -> LocationData {
    LocationData {
        tax_id: "04123456000190".to_string(),
        name: Some("Deposito Central".to_string()),
        street: "Rua do Deposito".to_string(),
        number: "200".to_string(),
        complement: None,
        district: "Industrial".to_string(),
        city_code: "4106902".to_string(),
        city_name: "Curitiba".to_string(),
        state_code: "PR".to_string(),
        zip_code: Some("81000000".to_string()),
    }
}

fn make_sample_signed_nfe_xml() -> String {
    concat!(
        r#"<?xml version="1.0" encoding="UTF-8"?>"#,
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
        "<xProd>NOTA FISCAL EMITIDA EM AMBIENTE DE HOMOLOGACAO</xProd>",
        "<NCM>00000000</NCM><CFOP>5102</CFOP><uCom>UN</uCom>",
        "<qCom>1.0000</qCom><vUnCom>150.0000000000</vUnCom><vProd>150.00</vProd>",
        "<cEANTrib>SEM GTIN</cEANTrib><uTrib>UN</uTrib>",
        "<qTrib>1.0000</qTrib><vUnTrib>150.0000000000</vUnTrib>",
        "<indTot>1</indTot></prod>",
        "<imposto><ICMS><ICMS00><orig>0</orig><CST>00</CST><modBC>0</modBC>",
        "<vBC>150.00</vBC><pICMS>18.0000</pICMS><vICMS>27.00</vICMS>",
        "</ICMS00></ICMS></imposto></det>",
        "<total><ICMSTot><vBC>150.00</vBC><vICMS>27.00</vICMS>",
        "<vICMSDeson>0.00</vICMSDeson><vFCPUFDest>0.00</vFCPUFDest>",
        "<vICMSUFDest>0.00</vICMSUFDest><vICMSUFRemet>0.00</vICMSUFRemet>",
        "<vFCP>0.00</vFCP><vBCST>0.00</vBCST><vST>0.00</vST>",
        "<vFCPST>0.00</vFCPST><vFCPSTRet>0.00</vFCPSTRet>",
        "<vProd>150.00</vProd><vFrete>0.00</vFrete><vSeg>0.00</vSeg>",
        "<vDesc>0.00</vDesc><vII>0.00</vII><vIPI>0.00</vIPI>",
        "<vIPIDevol>0.00</vIPIDevol><vPIS>0.00</vPIS><vCOFINS>0.00</vCOFINS>",
        "<vOutro>0.00</vOutro><vNF>150.00</vNF></ICMSTot></total>",
        "<transp><modFrete>9</modFrete></transp>",
        "<pag><detPag><tPag>01</tPag><vPag>150.00</vPag></detPag></pag>",
        "</infNFe>",
        r#"<Signature xmlns="http://www.w3.org/2000/09/xmldsig#">"#,
        "<SignedInfo><CanonicalizationMethod Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"></CanonicalizationMethod>",
        "<SignatureMethod Algorithm=\"http://www.w3.org/2000/09/xmldsig#rsa-sha1\"></SignatureMethod>",
        "<Reference URI=\"#NFe41260304123456000190550010000001231123456780\">",
        "<Transforms><Transform Algorithm=\"http://www.w3.org/2000/09/xmldsig#enveloped-signature\"></Transform>",
        "<Transform Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"></Transform></Transforms>",
        "<DigestMethod Algorithm=\"http://www.w3.org/2000/09/xmldsig#sha1\"></DigestMethod>",
        "<DigestValue>AAAAAAAAAAAAAAAAAAAAAAAAAAAA</DigestValue>",
        "</Reference></SignedInfo>",
        "<SignatureValue>BBBBBBBBBBBBBBBBBBBBBBBB</SignatureValue>",
        "<KeyInfo><X509Data><X509Certificate>CCCCCCCC</X509Certificate></X509Data></KeyInfo>",
        "</Signature></NFe>",
    ).to_string()
}

fn make_sample_autorizacao_response_xml() -> String {
    concat!(
        r#"<retEnviNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00">"#,
        "<tpAmb>2</tpAmb><cStat>104</cStat>",
        "<xMotivo>Lote processado</xMotivo>",
        r#"<protNFe versao="4.00"><infProt>"#,
        "<tpAmb>2</tpAmb>",
        "<chNFe>41260304123456000190550010000001231123456780</chNFe>",
        "<digVal>AAAAAAAAAAAAAAAAAAAAAAAAAAAA</digVal>",
        "<cStat>100</cStat>",
        "<xMotivo>Autorizado o uso da NF-e</xMotivo>",
        "<nProt>141260000012345</nProt>",
        "<dhRecbto>2026-03-11T10:30:00-03:00</dhRecbto>",
        "</infProt></protNFe></retEnviNFe>",
    ).to_string()
}

fn make_sample_nfce_signed_xml() -> String {
    concat!(
        r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
        r#"<infNFe versao="4.00" Id="NFe41260304123456000190650010000001231123456780">"#,
        "<ide><cUF>41</cUF><cNF>12345678</cNF><natOp>VENDA</natOp>",
        "<mod>65</mod><serie>1</serie><nNF>123</nNF>",
        "<dhEmi>2026-03-11T10:30:00-03:00</dhEmi>",
        "<tpNF>1</tpNF><idDest>1</idDest><cMunFG>4106902</cMunFG>",
        "<tpImp>4</tpImp><tpEmis>1</tpEmis><cDV>0</cDV>",
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
        "<vICMSDeson>0.00</vICMSDeson><vFCPUFDest>0.00</vFCPUFDest>",
        "<vICMSUFDest>0.00</vICMSUFDest><vICMSUFRemet>0.00</vICMSUFRemet>",
        "<vFCP>0.00</vFCP><vBCST>0.00</vBCST><vST>0.00</vST>",
        "<vFCPST>0.00</vFCPST><vFCPSTRet>0.00</vFCPSTRet>",
        "<vProd>150.00</vProd><vFrete>0.00</vFrete><vSeg>0.00</vSeg>",
        "<vDesc>0.00</vDesc><vII>0.00</vII><vIPI>0.00</vIPI>",
        "<vIPIDevol>0.00</vIPIDevol><vPIS>0.00</vPIS><vCOFINS>0.00</vCOFINS>",
        "<vOutro>0.00</vOutro><vNF>150.00</vNF></ICMSTot></total>",
        "<transp><modFrete>9</modFrete></transp>",
        "<pag><detPag><tPag>01</tPag><vPag>150.00</vPag></detPag></pag>",
        "</infNFe>",
        r#"<Signature xmlns="http://www.w3.org/2000/09/xmldsig#">"#,
        "<SignedInfo><CanonicalizationMethod Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"></CanonicalizationMethod>",
        "<SignatureMethod Algorithm=\"http://www.w3.org/2000/09/xmldsig#rsa-sha1\"></SignatureMethod>",
        "<Reference URI=\"#NFe41260304123456000190650010000001231123456780\">",
        "<Transforms><Transform Algorithm=\"http://www.w3.org/2000/09/xmldsig#enveloped-signature\"></Transform>",
        "<Transform Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"></Transform></Transforms>",
        "<DigestMethod Algorithm=\"http://www.w3.org/2000/09/xmldsig#sha1\"></DigestMethod>",
        "<DigestValue>AAAAAAAAAAAAAAAAAAAAAAAAAAAA</DigestValue>",
        "</Reference></SignedInfo>",
        "<SignatureValue>BBBBBBBBBBBBBBBBBBBBBBBB</SignatureValue>",
        "<KeyInfo><X509Data><X509Certificate>CCCCCCCC</X509Certificate></X509Data></KeyInfo>",
        "</Signature></NFe>",
    ).to_string()
}

fn make_sample_nfe_xml_for_contingency() -> String {
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
    ).to_string()
}

fn make_sample_nfe_xml_for_json() -> String {
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
        "<xFant>Auto Eletrica Barbosa</xFant>",
        "<enderEmit><xLgr>Rua XV de Novembro</xLgr><nro>1000</nro><xBairro>Centro</xBairro>",
        "<cMun>4106902</cMun><xMun>Curitiba</xMun><UF>PR</UF><CEP>80020310</CEP>",
        "<cPais>1058</cPais><xPais>Brasil</xPais></enderEmit>",
        "<IE>9012345678</IE><CRT>3</CRT></emit>",
        "<dest><CNPJ>12345678901234</CNPJ><xNome>Cliente Teste LTDA</xNome>",
        "<enderDest><xLgr>Rua das Flores</xLgr><nro>500</nro><xBairro>Batel</xBairro>",
        "<cMun>4106902</cMun><xMun>Curitiba</xMun><UF>PR</UF><CEP>80420120</CEP>",
        "<cPais>1058</cPais><xPais>Brasil</xPais></enderDest>",
        "<indIEDest>1</indIEDest><IE>1234567890</IE></dest>",
        "<det nItem=\"1\"><prod><cProd>001</cProd><cEAN>SEM GTIN</cEAN>",
        "<xProd>Servico de eletrica automotiva</xProd>",
        "<NCM>00000000</NCM><CFOP>5102</CFOP><uCom>UN</uCom>",
        "<qCom>1.0000</qCom><vUnCom>150.0000000000</vUnCom><vProd>150.00</vProd>",
        "<cEANTrib>SEM GTIN</cEANTrib><uTrib>UN</uTrib>",
        "<qTrib>1.0000</qTrib><vUnTrib>150.0000000000</vUnTrib>",
        "<indTot>1</indTot></prod>",
        "<imposto><ICMS><ICMS00><orig>0</orig><CST>00</CST><modBC>0</modBC>",
        "<vBC>150.00</vBC><pICMS>18.0000</pICMS><vICMS>27.00</vICMS>",
        "</ICMS00></ICMS>",
        "<PIS><PISAliq><CST>01</CST><vBC>150.00</vBC><pPIS>1.6500</pPIS><vPIS>2.48</vPIS></PISAliq></PIS>",
        "<COFINS><COFINSAliq><CST>01</CST><vBC>150.00</vBC><pCOFINS>7.6000</pCOFINS><vCOFINS>11.40</vCOFINS></COFINSAliq></COFINS>",
        "</imposto></det>",
        "<total><ICMSTot><vBC>150.00</vBC><vICMS>27.00</vICMS>",
        "<vICMSDeson>0.00</vICMSDeson><vFCPUFDest>0.00</vFCPUFDest>",
        "<vICMSUFDest>0.00</vICMSUFDest><vICMSUFRemet>0.00</vICMSUFRemet>",
        "<vFCP>0.00</vFCP><vBCST>0.00</vBCST><vST>0.00</vST>",
        "<vFCPST>0.00</vFCPST><vFCPSTRet>0.00</vFCPSTRet>",
        "<vProd>150.00</vProd><vFrete>0.00</vFrete><vSeg>0.00</vSeg>",
        "<vDesc>0.00</vDesc><vII>0.00</vII><vIPI>0.00</vIPI>",
        "<vIPIDevol>0.00</vIPIDevol><vPIS>2.48</vPIS><vCOFINS>11.40</vCOFINS>",
        "<vOutro>0.00</vOutro><vNF>150.00</vNF></ICMSTot></total>",
        "<transp><modFrete>9</modFrete></transp>",
        "<pag><detPag><tPag>01</tPag><vPag>150.00</vPag></detPag></pag>",
        "</infNFe></NFe>",
    ).to_string()
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
    ).to_string()
}

fn make_test_keypair() -> (String, String) {
    use openssl::rsa::Rsa;
    use openssl::pkey::PKey;
    use openssl::x509::{X509NameBuilder, X509};
    use openssl::bn::BigNum;
    use openssl::hash::MessageDigest;

    let rsa = Rsa::generate(2048).unwrap();
    let pkey = PKey::from_rsa(rsa).unwrap();

    let mut name_builder = X509NameBuilder::new().unwrap();
    name_builder.append_entry_by_text("CN", "Bench Test").unwrap();
    let name = name_builder.build();

    let mut builder = X509::builder().unwrap();
    builder.set_version(2).unwrap();
    let serial = BigNum::from_u32(1).unwrap();
    builder.set_serial_number(&serial.to_asn1_integer().unwrap()).unwrap();
    builder.set_subject_name(&name).unwrap();
    builder.set_issuer_name(&name).unwrap();
    builder.set_not_before(&openssl::asn1::Asn1Time::days_from_now(0).unwrap()).unwrap();
    builder.set_not_after(&openssl::asn1::Asn1Time::days_from_now(365).unwrap()).unwrap();
    builder.set_pubkey(&pkey).unwrap();
    builder.sign(&pkey, MessageDigest::sha256()).unwrap();
    let cert = builder.build();

    let private_key_pem = String::from_utf8(pkey.private_key_to_pem_pkcs8().unwrap()).unwrap();
    let cert_pem = String::from_utf8(cert.to_pem().unwrap()).unwrap();

    (private_key_pem, cert_pem)
}
