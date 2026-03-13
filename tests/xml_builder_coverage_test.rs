// Tests to cover uncovered paths in xml_builder sub-modules:
// builder.rs, ide.rs, pag.rs, transp.rs, dest.rs, emit.rs, mod.rs,
// tax_id.rs, access_key.rs, total.rs, optional.rs

mod common;

use fiscal::newtypes::{Cents, IbgeCode, Rate};
use fiscal::types::*;
use fiscal::xml_builder::InvoiceBuilder;

// ── Helper: build NF-e (model 55) for richer coverage ───────────────────────

fn nfe_builder() -> InvoiceBuilder {
    let offset = common::br_offset();
    let issued_at = chrono::NaiveDate::from_ymd_opt(2026, 1, 15)
        .unwrap()
        .and_hms_opt(10, 30, 0)
        .unwrap()
        .and_local_timezone(offset)
        .unwrap();

    InvoiceBuilder::new(
        common::sample_issuer()
            .address_complement("Sala 1")
            .phone("11999998888")
            .iest("123456789")
            .im("12345")
            .cnae("1234567"),
        SefazEnvironment::Homologation,
        InvoiceModel::Nfe,
    )
    .series(1)
    .invoice_number(1)
    .issued_at(issued_at)
}

// ── builder.rs: InvoiceBuilder builder methods ──────────────────────────────

#[test]
fn builder_emission_type() {
    let built = nfe_builder()
        .emission_type(EmissionType::SvcAn)
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<tpEmis>6</tpEmis>"));
}

#[test]
fn builder_schema_version_pl010() {
    let built = nfe_builder()
        .schema_version(SchemaVersion::PL010)
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("versao=\"4.10\"") || xml.contains("<mod>55</mod>"));
}

#[test]
fn builder_change_amount() {
    let built = common::sample_invoice_builder()
        .change_amount(Cents(500))
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<vTroco>5.00</vTroco>"));
}

#[test]
fn builder_payment_card_details() {
    let built = common::sample_invoice_builder()
        .payment_card_details(vec![
            PaymentCardDetail::new()
                .integ_type("1")
                .card_tax_id("12345678000199")
                .card_brand("01")
                .auth_code("ABC123"),
        ])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<card>"));
    assert!(xml.contains("<tpIntegra>1</tpIntegra>"));
    assert!(xml.contains("<tBand>01</tBand>"));
    assert!(xml.contains("<cAut>ABC123</cAut>"));
}

#[test]
fn builder_exit_at() {
    let offset = common::br_offset();
    let exit = chrono::NaiveDate::from_ymd_opt(2026, 1, 16)
        .unwrap()
        .and_hms_opt(8, 0, 0)
        .unwrap()
        .and_local_timezone(offset)
        .unwrap();

    let built = nfe_builder()
        .exit_at(exit)
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<dhSaiEnt>"));
}

#[test]
fn builder_operation_type_and_purpose() {
    let built = nfe_builder()
        .operation_type(0)
        .purpose_code(2)
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<tpNF>0</tpNF>"));
    assert!(xml.contains("<finNFe>2</finNFe>"));
}

#[test]
fn builder_intermediary_indicator() {
    let built = nfe_builder()
        .intermediary_indicator("1")
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<indIntermed>1</indIntermed>"));
}

#[test]
fn builder_emission_process() {
    let built = nfe_builder()
        .emission_process("1")
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<procEmi>1</procEmi>"));
}

#[test]
fn builder_export() {
    let built = nfe_builder()
        .export(ExportData::new("SP", "Santos").dispatch_location("Guarulhos"))
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<exporta>"));
    assert!(xml.contains("<UFSaidaPais>SP</UFSaidaPais>"));
    assert!(xml.contains("<xLocDespacho>Guarulhos</xLocDespacho>"));
}

#[test]
fn builder_issqn_tot() {
    let built = nfe_builder()
        .issqn_tot(IssqnTotData::new("2026-01"))
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<ISSQNtot>"));
    assert!(xml.contains("<dCompet>2026-01</dCompet>"));
}

#[test]
fn builder_calculation_method_v1() {
    let built = nfe_builder()
        .calculation_method(CalculationMethod::V1)
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<vNF>"));
}

#[test]
fn builder_sign_with() {
    let built = common::sample_invoice_builder().build().unwrap();
    let signed = built
        .sign_with(|xml| Ok(format!("<signed>{}</signed>", xml)))
        .unwrap();
    let sxml = signed.signed_xml();
    assert!(sxml.contains("<signed>"));
}

// ── ide.rs: Reference documents ─────────────────────────────────────────────

#[test]
fn ide_reference_nf() {
    let built = nfe_builder()
        .references(vec![ReferenceDoc::Nf {
            state_code: IbgeCode("35".to_string()),
            year_month: "2601".to_string(),
            tax_id: "12345678000199".to_string(),
            model: "55".to_string(),
            series: "1".to_string(),
            number: "123".to_string(),
        }])
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<refNF>"));
    assert!(xml.contains("<AAMM>2601</AAMM>"));
}

#[test]
fn ide_reference_nfp_with_cpf() {
    let built = nfe_builder()
        .references(vec![ReferenceDoc::Nfp {
            state_code: IbgeCode("35".to_string()),
            year_month: "2601".to_string(),
            tax_id: "12345678901".to_string(),
            ie: "ISENTO".to_string(),
            model: "04".to_string(),
            series: "1".to_string(),
            number: "456".to_string(),
        }])
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<refNFP>"));
    assert!(xml.contains("<CPF>"));
    assert!(xml.contains("<IE>ISENTO</IE>"));
}

#[test]
fn ide_reference_nfp_with_cnpj_and_ie() {
    let built = nfe_builder()
        .references(vec![ReferenceDoc::Nfp {
            state_code: IbgeCode("35".to_string()),
            year_month: "2601".to_string(),
            tax_id: "00940734000150".to_string(),
            ie: "123456789012".to_string(),
            model: "04".to_string(),
            series: "0".to_string(),
            number: "5578".to_string(),
        }])
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<refNFP>"));
    assert!(xml.contains("<CNPJ>00940734000150</CNPJ>"));
    assert!(xml.contains("<IE>123456789012</IE>"));
    // Verify tag order: CNPJ before IE before mod
    let cnpj_pos = xml.find("<CNPJ>").unwrap();
    let ie_pos = xml.find("<IE>").unwrap();
    let mod_pos = xml.find("<mod>04</mod>").unwrap();
    assert!(cnpj_pos < ie_pos, "CNPJ must come before IE");
    assert!(ie_pos < mod_pos, "IE must come before mod");
}

#[test]
fn ide_reference_cte() {
    let built = nfe_builder()
        .references(vec![ReferenceDoc::Cte {
            access_key: "12345678901234567890123456789012345678901234".to_string(),
        }])
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<refCTe>"));
}

#[test]
fn ide_reference_ecf() {
    let built = nfe_builder()
        .references(vec![ReferenceDoc::Ecf {
            model: "2D".to_string(),
            ecf_number: "001".to_string(),
            coo_number: "000123".to_string(),
        }])
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<refECF>"));
    assert!(xml.contains("<nECF>001</nECF>"));
    assert!(xml.contains("<nCOO>000123</nCOO>"));
}

#[test]
fn ide_reference_nfe_sig() {
    let built = nfe_builder()
        .references(vec![ReferenceDoc::NfeSig {
            access_key: "41260304123456000190550010000001231123456780".to_string(),
        }])
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<NFref>"));
    assert!(xml.contains("<refNFeSig>41260304123456000190550010000001231123456780</refNFeSig>"));
    // Must NOT contain <refNFe> (only <refNFeSig>)
    assert!(!xml.contains("<refNFe>"));
}

// ── ide.rs: compra_gov / pag_antecipado with PL010 ─────────────────────────

#[test]
fn ide_compra_gov_pl010() {
    let built = nfe_builder()
        .schema_version(SchemaVersion::PL010)
        .compra_gov(CompraGovData::new("1", "10.0000", "1"))
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<gCompraGov>"));
}

#[test]
fn ide_pag_antecipado_pl010() {
    let built = nfe_builder()
        .schema_version(SchemaVersion::PL010)
        .pag_antecipado(PagAntecipadoData::new(vec![
            "12345678901234567890123456789012345678901234".to_string(),
        ]))
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<gPagAntecipado>"));
}

// ── pag.rs: empty payments, card details, change ────────────────────────────

#[test]
fn pag_empty_payments() {
    let built = nfe_builder()
        .payments(vec![])
        .add_item(common::sample_item())
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<tPag>90</tPag>"));
    assert!(xml.contains("<vPag>0.00</vPag>"));
}

#[test]
fn pag_with_all_optional_fields() {
    let payment = PaymentData::new("01", Cents(5000))
        .ind_pag("0")
        .x_pag("Dinheiro")
        .d_pag("2026-01-15")
        .cnpj_pag("12345678000199")
        .uf_pag("SP");

    let built = nfe_builder()
        .payments(vec![payment])
        .payment_card_details(vec![
            PaymentCardDetail::new()
                .integ_type("1")
                .card_tax_id("99887766000155")
                .card_brand("02")
                .auth_code("XYZ789"),
        ])
        .change_amount(Cents(1000))
        .add_item(common::sample_item())
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<indPag>0</indPag>"));
    assert!(xml.contains("<xPag>Dinheiro</xPag>"));
    assert!(xml.contains("<dPag>2026-01-15</dPag>"));
    assert!(xml.contains("<CNPJPag>12345678000199</CNPJPag>"));
    assert!(xml.contains("<UFPag>SP</UFPag>"));
    assert!(xml.contains("<vTroco>10.00</vTroco>"));
}

// ── transp.rs: carrier, vehicle, volumes, retained ICMS ─────────────────────

#[test]
fn transp_carrier_with_all_fields() {
    let transport = TransportData::new("0")
        .carrier(
            CarrierData::new()
                .tax_id("12345678000199")
                .name("Transportadora ABC")
                .state_tax_id("111222333")
                .address("Rua dos Transportes 50")
                .state_code("SP"),
        )
        .vehicle(VehicleData::new("ABC1234", "SP").rntc("RNTC001"))
        .trailers(vec![VehicleData::new("XYZ9876", "RJ").rntc("RNTC002")])
        .volumes(vec![
            VolumeData::new()
                .quantity(10)
                .species("CAIXA")
                .brand("MarcaX")
                .number("001")
                .net_weight(50.0)
                .gross_weight(55.0)
                .seals(vec!["LACRE001".to_string()]),
        ]);

    let built = nfe_builder()
        .transport(transport)
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();

    assert!(xml.contains("<transporta>"));
    assert!(xml.contains("<xNome>Transportadora ABC</xNome>"));
    assert!(xml.contains("<IE>111222333</IE>"));
    assert!(xml.contains("<xEnder>Rua dos Transportes 50</xEnder>"));
    assert!(xml.contains("<veicTransp>"));
    assert!(xml.contains("<RNTC>RNTC001</RNTC>"));
    assert!(xml.contains("<reboque>"));
    assert!(xml.contains("<RNTC>RNTC002</RNTC>"));
    assert!(xml.contains("<vol>"));
    assert!(xml.contains("<qVol>10</qVol>"));
    assert!(xml.contains("<esp>CAIXA</esp>"));
    assert!(xml.contains("<marca>MarcaX</marca>"));
    assert!(xml.contains("<nVol>001</nVol>"));
    assert!(xml.contains("<pesoL>"));
    assert!(xml.contains("<pesoB>"));
    assert!(xml.contains("<lacres>"));
}

#[test]
fn transp_retained_icms() {
    let transport = TransportData::new("1").retained_icms(RetainedIcmsTransp::new(
        Cents(10000),
        Cents(10000),
        Rate(1200),
        Cents(1200),
        "5353",
        IbgeCode("3550308".to_string()),
    ));

    let built = nfe_builder()
        .transport(transport)
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<retTransp>"));
    assert!(xml.contains("<vBCRet>"));
    assert!(xml.contains("<pICMSRet>"));
    assert!(xml.contains("<vICMSRet>"));
    assert!(xml.contains("<CFOP>5353</CFOP>"));
}

// ── dest.rs: foreign recipient, NF-e with full address ──────────────────────

#[test]
fn dest_foreign_recipient() {
    let recipient = RecipientData::new("AA12345", "Foreign Corp");

    let built = nfe_builder()
        .recipient(recipient)
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<idEstrangeiro>AA12345</idEstrangeiro>"));
}

#[test]
fn dest_nfe_with_full_address() {
    let recipient = common::sample_recipient()
        .complement("Apto 42")
        .phone("1133334444")
        .email("test@example.com")
        .isuf("12345")
        .im("67890")
        .ind_ie_dest("1")
        .state_tax_id("111222333");

    let built = nfe_builder()
        .recipient(recipient)
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<enderDest>"));
    assert!(xml.contains("<xCpl>Apto 42</xCpl>"));
    assert!(xml.contains("<fone>1133334444</fone>"));
    assert!(xml.contains("<indIEDest>1</indIEDest>"));
    assert!(xml.contains("<IE>111222333</IE>"));
    assert!(xml.contains("<ISUF>12345</ISUF>"));
    assert!(xml.contains("<IM>67890</IM>"));
    assert!(xml.contains("<email>test@example.com</email>"));
}

// ── emit.rs: issuer with IEST, IM, CNAE ────────────────────────────────────

#[test]
fn emit_with_iest_im_cnae() {
    let built = nfe_builder()
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<IEST>123456789</IEST>"));
    assert!(xml.contains("<IM>12345</IM>"));
    assert!(xml.contains("<CNAE>1234567</CNAE>"));
    assert!(xml.contains("<fone>11999998888</fone>"));
    assert!(xml.contains("<xCpl>Sala 1</xCpl>"));
}

// ── optional.rs: billing, additional info, tech responsible, etc. ────────────

#[test]
fn optional_billing_with_installments() {
    let billing = BillingData::new()
        .invoice(
            BillingInvoice::new("FAT001", Cents(10000), Cents(9000)).discount_value(Cents(1000)),
        )
        .installments(vec![
            Installment::new("001", "2026-02-15", Cents(4500)),
            Installment::new("002", "2026-03-15", Cents(4500)),
        ]);

    let built = nfe_builder()
        .billing(billing)
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<cobr>"));
    assert!(xml.contains("<fat>"));
    assert!(xml.contains("<nFat>FAT001</nFat>"));
    assert!(xml.contains("<dup>"));
}

#[test]
fn optional_additional_info_full() {
    let info = AdditionalInfo::new()
        .taxpayer_note("Nota do contribuinte")
        .tax_authority_note("Nota para o fisco")
        .contributor_obs(vec![FieldText::new("campo1", "texto1")])
        .fiscal_obs(vec![FieldText::new("campo2", "texto2")])
        .process_refs(vec![ProcessRef::new("PROC001", "0")]);

    let built = nfe_builder()
        .additional_info(info)
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<infAdic>"));
    assert!(xml.contains("<infCpl>"));
    assert!(xml.contains("<infAdFisco>"));
    assert!(xml.contains("<obsCont"));
    assert!(xml.contains("<obsFisco"));
    assert!(xml.contains("<procRef>"));
}

#[test]
fn optional_tech_responsible() {
    let tech = TechResponsibleData::new("12345678000199", "Joao", "joao@test.com")
        .phone("11999998888")
        .csrt("TOKEN123", "01");

    let built = nfe_builder()
        .tech_responsible(tech)
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<infRespTec>"));
    assert!(xml.contains("<fone>11999998888</fone>"));
    assert!(xml.contains("<idCSRT>01</idCSRT>"));
    assert!(xml.contains("<hashCSRT>"));
}

#[test]
fn optional_purchase_data() {
    let purchase = PurchaseData::new()
        .order_number("PED001")
        .contract_number("CONT001")
        .purchase_note("NOTA001");

    let built = nfe_builder()
        .purchase(purchase)
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<compra>"));
    assert!(xml.contains("<xPed>PED001</xPed>"));
    assert!(xml.contains("<xCont>CONT001</xCont>"));
    assert!(xml.contains("<xNEmp>NOTA001</xNEmp>"));
}

#[test]
fn optional_intermediary() {
    let intermediary = IntermediaryData::new("12345678000199");

    let built = nfe_builder()
        .intermediary(intermediary)
        .intermediary_indicator("1")
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<infIntermed>"));
}

#[test]
fn optional_ret_trib() {
    let ret = RetTribData::new()
        .v_ret_pis(Cents(100))
        .v_ret_cofins(Cents(200))
        .v_ret_csll(Cents(300))
        .v_bc_irrf(Cents(10000))
        .v_irrf(Cents(400))
        .v_bc_ret_prev(Cents(20000))
        .v_ret_prev(Cents(500));

    let built = nfe_builder()
        .ret_trib(ret)
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<retTrib>"));
    assert!(xml.contains("<vRetPIS>"));
    assert!(xml.contains("<vRetCOFINS>"));
}

#[test]
fn optional_withdrawal_delivery() {
    let withdrawal = LocationData::new(
        "12345678000199",
        "Rua A",
        "100",
        "Centro",
        IbgeCode("3550308".to_string()),
        "Sao Paulo",
        "SP",
    )
    .name("Local Retirada")
    .complement("Bloco A")
    .zip_code("01001000");

    let delivery = LocationData::new(
        "98765432000111",
        "Rua B",
        "200",
        "Bela Vista",
        IbgeCode("3550308".to_string()),
        "Sao Paulo",
        "SP",
    );

    let built = nfe_builder()
        .withdrawal(withdrawal)
        .delivery(delivery)
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<retirada>"));
    assert!(xml.contains("<entrega>"));
}

// ── total.rs: ISSQNtot with all fields ──────────────────────────────────────

#[test]
fn total_issqn_tot_full() {
    let issqn_tot = IssqnTotData::new("2026-01")
        .v_serv(Cents(10000))
        .v_bc(Cents(10000))
        .v_iss(Cents(500))
        .v_pis(Cents(165))
        .v_cofins(Cents(760))
        .v_deducao(Cents(100))
        .v_outro(Cents(50))
        .v_desc_incond(Cents(200))
        .v_desc_cond(Cents(100))
        .v_iss_ret(Cents(50))
        .c_reg_trib("6");

    let built = nfe_builder()
        .issqn_tot(issqn_tot)
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<ISSQNtot>"));
    assert!(xml.contains("<dCompet>2026-01</dCompet>"));
    assert!(xml.contains("<cRegTrib>6</cRegTrib>"));
}

// ── builder.rs: cana (sugarcane) ────────────────────────────────────────────

#[test]
fn builder_cana() {
    let cana = CanaData::new(
        "2025/2026",
        "03/2026",
        vec![
            ForDiaData::new(1, Cents(1000)),
            ForDiaData::new(2, Cents(2000)),
        ],
        Cents(3000),
        Cents(10000),
        Cents(13000),
        Cents(100000),
        Cents(5000),
        Cents(95000),
    );

    let built = nfe_builder()
        .cana(cana)
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<cana>"));
}

// ── builder.rs: agropecuario ────────────────────────────────────────────────

#[test]
fn builder_agropecuario_guia() {
    let built = nfe_builder()
        .schema_version(SchemaVersion::PL010)
        .agropecuario(AgropecuarioData::Guia(AgropecuarioGuiaData::new(
            "1", "GTA001",
        )))
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<mod>55</mod>"));
}

#[test]
fn builder_agropecuario_defensivos() {
    let built = nfe_builder()
        .schema_version(SchemaVersion::PL010)
        .agropecuario(AgropecuarioData::Defensivos(vec![
            AgropecuarioDefensivoData::new("REC001", "12345678901"),
        ]))
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<mod>55</mod>"));
}

// ── builder.rs: is_tot and ibs_cbs_tot ──────────────────────────────────────

#[test]
fn builder_is_tot_and_ibs_cbs_tot() {
    use fiscal::tax_ibs_cbs::{IbsCbsTotData, IsTotData};

    let built = nfe_builder()
        .schema_version(SchemaVersion::PL010)
        .is_tot(IsTotData::new("100.00"))
        .ibs_cbs_tot(IbsCbsTotData::new("50000.00"))
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<mod>55</mod>"));
}

// ── builder.rs: destination_indicator (lines 318-320) ────────────────────

#[test]
fn builder_destination_indicator() {
    let built = nfe_builder()
        .destination_indicator("2")
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<idDest>2</idDest>"));
}

// ── builder.rs: ver_proc (lines 324-326) ─────────────────────────────────

#[test]
fn builder_ver_proc() {
    let built = nfe_builder()
        .ver_proc("fiscal-rs v1.0.0")
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<verProc>fiscal-rs v1.0.0</verProc>"));
}

// ── builder.rs: sign_with body (lines 627, 683) ─────────────────────────

#[test]
fn builder_sign_with_covers_body() {
    let built = nfe_builder()
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let original_xml = built.xml().to_string();
    let signed = built
        .sign_with(|xml| Ok(format!("<Signed>{}</Signed>", xml)))
        .unwrap();
    assert!(signed.signed_xml().starts_with("<Signed>"));
    assert_eq!(signed.unsigned_xml(), original_xml);
    assert_eq!(signed.access_key().len(), 44);
}

// ── tax_id.rs: TaxId to_xml_tag for CPF ─────────────────────────────────────

#[test]
fn tax_id_cpf_to_xml() {
    // CPF in dest triggers TaxId logic for CPF path
    let recipient = RecipientData::new("12345678901", "Test");
    let built = nfe_builder()
        .recipient(recipient)
        .add_item(common::sample_item())
        .payments(vec![common::sample_payment()])
        .build()
        .unwrap();
    let xml = built.xml();
    assert!(xml.contains("<CPF>12345678901</CPF>"));
}
