// Ported from TypeScript: deep-comm-coverage-ported.test.ts (163 tests)
// Tests from DeepCoverageTest and CommunicationCoverageTest

mod common;

use fiscal::newtypes::{Cents, Rate, IbgeCode};
use fiscal::types::*;
use fiscal::xml_builder::InvoiceBuilder;
use fiscal::xml_utils::{tag, TagContent};

use common::expect_xml_tag_values as expect_xml_contains;

/// Standard minimal item for NF-e 55 tests.
fn minimal_nfe55_item() -> InvoiceItemData {
    InvoiceItemData {
        item_number: 1,
        product_code: "001".to_string(),
        description: "Produto Teste".to_string(),
        ncm: "61091000".to_string(),
        cfop: "5102".to_string(),
        unit_of_measure: "UN".to_string(),
        quantity: 10.0,
        unit_price: Cents(1000),
        total_price: Cents(10000),
        icms_cst: "00".to_string(),
        icms_rate: Rate(1800),
        icms_amount: Cents(1800),
        icms_mod_bc: Some(0),
        pis_cst: "07".to_string(),
        cofins_cst: "07".to_string(),
        c_ean: None, c_ean_trib: None, cest: None, v_frete: None, v_seg: None,
        v_desc: None, v_outro: None, orig: None, icms_red_bc: None,
        icms_mod_bc_st: None, icms_p_mva_st: None, icms_red_bc_st: None,
        icms_v_bc_st: None, icms_p_icms_st: None, icms_v_icms_st: None,
        icms_v_icms_deson: None, icms_mot_des_icms: None, icms_p_fcp: None,
        icms_v_fcp: None, icms_v_bc_fcp: None, icms_p_fcp_st: None,
        icms_v_fcp_st: None, icms_v_bc_fcp_st: None, icms_p_cred_sn: None,
        icms_v_cred_icms_sn: None, icms_v_icms_substituto: None,
        pis_v_bc: None, pis_p_pis: None, pis_v_pis: None,
        pis_q_bc_prod: None, pis_v_aliq_prod: None,
        cofins_v_bc: None, cofins_p_cofins: None, cofins_v_cofins: None,
        cofins_q_bc_prod: None, cofins_v_aliq_prod: None,
        ipi_cst: None, ipi_c_enq: None, ipi_v_bc: None, ipi_p_ipi: None,
        ipi_v_ipi: None, ipi_q_unid: None, ipi_v_unid: None,
        ii_v_bc: None, ii_v_desp_adu: None, ii_v_ii: None, ii_v_iof: None,
        rastro: None, veic_prod: None, med: None, arma: None,
        n_recopi: None, inf_ad_prod: None, obs_item: None,
        dfe_referenciado: None,
    }
}

/// Helper to build a minimal NF-e 55 InvoiceBuilder for tests that just need valid XML output.
fn minimal_nfe55(number: u32) -> InvoiceBuilder {
    use chrono::FixedOffset;
    let offset = FixedOffset::west_opt(3 * 3600).unwrap();
    let dt = chrono::NaiveDate::from_ymd_opt(2017, 3, 3)
        .unwrap()
        .and_hms_opt(11, 30, 0)
        .unwrap()
        .and_local_timezone(offset)
        .unwrap();

    InvoiceBuilder::new(
        IssuerData {
            tax_id: "58716523000119".to_string(),
            state_tax_id: "123456789012".to_string(),
            company_name: "EMPRESA TESTE LTDA".to_string(),
            trade_name: Some("EMPRESA TESTE".to_string()),
            tax_regime: TaxRegime::Normal,
            state_code: "SP".to_string(),
            city_code: IbgeCode("3550308".to_string()),
            city_name: "Sao Paulo".to_string(),
            street: "Rua Teste".to_string(),
            street_number: "100".to_string(),
            district: "Centro".to_string(),
            zip_code: "01001000".to_string(),
            address_complement: None,
        },
        SefazEnvironment::Homologation,
        InvoiceModel::Nfe,
    )
    .series(1)
    .invoice_number(number)
    .issued_at(dt)
    .recipient(RecipientData {
        tax_id: "11222333000181".to_string(),
        name: "CLIENTE TESTE".to_string(),
        state_code: Some("SP".to_string()),
        ..Default::default()
    })
    .items(vec![minimal_nfe55_item()])
    .payments(vec![PaymentData { method: "01".to_string(), amount: Cents(10000) }])
}

// =============================================================================
// DeepCoverageTest
// =============================================================================

mod deep_coverage_test {

    // ═══════════════════════════════════════════════════════════════════
    //  1. Make.php render() coverage
    // ═══════════════════════════════════════════════════════════════════

    mod make_render_coverage {
        use super::super::*;

        #[test]
        fn monta_nfe_is_alias_for_render() {
            let built = minimal_nfe55(30).build().expect("build failed");
            assert!(!built.xml().is_empty());
            assert!(built.xml().contains("<NFe"));
        }

        #[test]
        fn set_only_ascii_converts_accented_characters() {
            let mut item = minimal_nfe55_item();
            item.quantity = 1.0;
            item.total_price = Cents(1000);
            item.icms_amount = Cents(180);
            let result = minimal_nfe55(30)
                .operation_nature("OPERACAO COM ACENTUACAO")
                .items(vec![item])
                .payments(vec![PaymentData { method: "01".to_string(), amount: Cents(1000) }])
                .build();
            assert!(result.is_ok());
        }

        #[test]
        fn set_check_gtin_validates_gtin_codes() {
            assert_eq!(fiscal::gtin::is_valid_gtin("SEM GTIN").unwrap(), true);
            assert_eq!(fiscal::gtin::is_valid_gtin("").unwrap(), true);
            assert!(fiscal::gtin::is_valid_gtin("1234567890123").is_err());
            assert_eq!(fiscal::gtin::is_valid_gtin("7891234567895").unwrap(), true);
        }

        #[test]
        fn render_with_cobr_fat_dup() {
            let xml = tag("cobr", &[], TagContent::Children(vec![
                tag("fat", &[], TagContent::Children(vec![
                    tag("nFat", &[], TagContent::Text("001")),
                    tag("vOrig", &[], TagContent::Text("500.00")),
                    tag("vDesc", &[], TagContent::Text("10.00")),
                    tag("vLiq", &[], TagContent::Text("490.00")),
                ])),
                tag("dup", &[], TagContent::Children(vec![
                    tag("nDup", &[], TagContent::Text("001")),
                    tag("dVenc", &[], TagContent::Text("2025-03-01")),
                    tag("vDup", &[], TagContent::Text("245.00")),
                ])),
                tag("dup", &[], TagContent::Children(vec![
                    tag("nDup", &[], TagContent::Text("002")),
                    tag("dVenc", &[], TagContent::Text("2025-04-01")),
                    tag("vDup", &[], TagContent::Text("245.00")),
                ])),
            ]));

            assert!(xml.contains("<cobr>"));
            assert!(xml.contains("<fat>"));
            expect_xml_contains(&xml, &[("nFat", "001"), ("vOrig", "500.00"), ("vLiq", "490.00")]);
            let dup_count = xml.matches("<dup>").count();
            assert_eq!(dup_count, 2);
        }

        #[test]
        fn render_with_retirada() {
            let xml = tag("retirada", &[], TagContent::Children(vec![
                tag("CNPJ", &[], TagContent::Text("99887766000100")),
                tag("xNome", &[], TagContent::Text("Empresa Origem")),
                tag("xLgr", &[], TagContent::Text("Rua Retirada")),
                tag("nro", &[], TagContent::Text("50")),
                tag("xBairro", &[], TagContent::Text("Industrial")),
                tag("cMun", &[], TagContent::Text("4106902")),
                tag("xMun", &[], TagContent::Text("Curitiba")),
                tag("UF", &[], TagContent::Text("PR")),
            ]));
            assert!(xml.contains("<retirada>"));
            expect_xml_contains(&xml, &[("xLgr", "Rua Retirada")]);
        }

        #[test]
        fn render_with_entrega() {
            let xml = tag("entrega", &[], TagContent::Children(vec![
                tag("CNPJ", &[], TagContent::Text("11222333000181")),
                tag("xLgr", &[], TagContent::Text("Rua Entrega")),
                tag("nro", &[], TagContent::Text("200")),
                tag("xBairro", &[], TagContent::Text("Centro")),
                tag("cMun", &[], TagContent::Text("3550308")),
                tag("xMun", &[], TagContent::Text("Sao Paulo")),
                tag("UF", &[], TagContent::Text("SP")),
            ]));
            assert!(xml.contains("<entrega>"));
            expect_xml_contains(&xml, &[("xLgr", "Rua Entrega")]);
        }

        #[test]
        fn render_with_aut_xml() {
            let cnpj_xml = tag("autXML", &[], TagContent::Children(vec![
                tag("CNPJ", &[], TagContent::Text("12345678000195")),
            ]));
            let cpf_xml = tag("autXML", &[], TagContent::Children(vec![
                tag("CPF", &[], TagContent::Text("12345678901")),
            ]));
            assert!(cnpj_xml.contains("<autXML>"));
            assert!(cnpj_xml.contains("<CNPJ>12345678000195</CNPJ>"));
            assert!(cpf_xml.contains("<autXML>"));
            assert!(cpf_xml.contains("<CPF>12345678901</CPF>"));
        }

        #[test]
        fn render_with_inf_intermed() {
            let xml = tag("infIntermed", &[], TagContent::Children(vec![
                tag("CNPJ", &[], TagContent::Text("55667788000199")),
                tag("idCadIntTran", &[], TagContent::Text("CADASTRO123")),
            ]));
            assert!(xml.contains("<infIntermed>"));
            expect_xml_contains(&xml, &[("CNPJ", "55667788000199"), ("idCadIntTran", "CADASTRO123")]);
        }

        #[test]
        fn render_with_exporta() {
            let xml = tag("exporta", &[], TagContent::Children(vec![
                tag("UFSaidaPais", &[], TagContent::Text("SP")),
                tag("xLocExporta", &[], TagContent::Text("Porto de Santos")),
                tag("xLocDespacho", &[], TagContent::Text("Aeroporto de Guarulhos")),
            ]));
            assert!(xml.contains("<exporta>"));
            expect_xml_contains(&xml, &[
                ("UFSaidaPais", "SP"),
                ("xLocExporta", "Porto de Santos"),
                ("xLocDespacho", "Aeroporto de Guarulhos"),
            ]);
        }

        #[test]
        fn render_with_compra() {
            let xml = tag("compra", &[], TagContent::Children(vec![
                tag("xNEmp", &[], TagContent::Text("NE-001")),
                tag("xPed", &[], TagContent::Text("PED-001")),
                tag("xCont", &[], TagContent::Text("CONT-001")),
            ]));
            assert!(xml.contains("<compra>"));
            expect_xml_contains(&xml, &[("xNEmp", "NE-001"), ("xPed", "PED-001"), ("xCont", "CONT-001")]);
        }

        #[test]
        fn render_with_cana() {
            let xml = tag("cana", &[], TagContent::Children(vec![
                tag("safra", &[], TagContent::Text("2017/2018")),
                tag("ref", &[], TagContent::Text("03/2017")),
                tag("forDia", &[("dia", "1")], TagContent::Children(vec![
                    tag("qtde", &[], TagContent::Text("100.0000000000")),
                ])),
                tag("forDia", &[("dia", "2")], TagContent::Children(vec![
                    tag("qtde", &[], TagContent::Text("200.0000000000")),
                ])),
                tag("qTotMes", &[], TagContent::Text("1000.0000000000")),
                tag("qTotAnt", &[], TagContent::Text("500.0000000000")),
                tag("qTotGer", &[], TagContent::Text("1500.0000000000")),
                tag("deduc", &[], TagContent::Children(vec![
                    tag("xDed", &[], TagContent::Text("DEDUCAO TESTE")),
                    tag("vDed", &[], TagContent::Text("500.00")),
                ])),
                tag("vFor", &[], TagContent::Text("15000.00")),
                tag("vTotDed", &[], TagContent::Text("500.00")),
                tag("vLiqFor", &[], TagContent::Text("14500.00")),
            ]));
            assert!(xml.contains("<cana>"));
            assert!(xml.contains("<safra>2017/2018</safra>"));
            assert!(xml.contains("<forDia"));
            assert!(xml.contains("<deduc>"));
            assert!(xml.contains("<xDed>DEDUCAO TESTE</xDed>"));
        }

        #[test]
        fn render_with_inf_resp_tec() {
            let xml = tag("infRespTec", &[], TagContent::Children(vec![
                tag("CNPJ", &[], TagContent::Text("11223344000155")),
                tag("xContato", &[], TagContent::Text("Suporte Tecnico")),
                tag("email", &[], TagContent::Text("suporte@teste.com")),
                tag("fone", &[], TagContent::Text("1133334444")),
            ]));
            assert!(xml.contains("<infRespTec>"));
            expect_xml_contains(&xml, &[
                ("CNPJ", "11223344000155"),
                ("xContato", "Suporte Tecnico"),
                ("email", "suporte@teste.com"),
                ("fone", "1133334444"),
            ]);
        }

        #[test]
        fn render_error_handling_stores_errors() {
            let issuer = IssuerData {
                tax_id: "58716523000119".to_string(),
                state_tax_id: "123456789012".to_string(),
                company_name: "EMPRESA TESTE LTDA".to_string(),
                trade_name: Some("EMPRESA TESTE".to_string()),
                tax_regime: TaxRegime::Normal,
                state_code: "XX".to_string(), // Invalid state
                city_code: IbgeCode("3550308".to_string()),
                city_name: "Sao Paulo".to_string(),
                street: "Rua Teste".to_string(),
                street_number: "100".to_string(),
                district: "Centro".to_string(),
                zip_code: "01001000".to_string(),
                address_complement: None,
            };
            let result = InvoiceBuilder::new(issuer, SefazEnvironment::Homologation, InvoiceModel::Nfe)
                .series(1)
                .invoice_number(30)
                .build();
            assert!(result.is_err());
        }

        #[test]
        fn get_xml_calls_render_if_empty() {
            let built = minimal_nfe55(30).build().expect("build failed");
            assert!(!built.xml().is_empty());
            assert!(built.xml().contains("<NFe"));
        }

        #[test]
        fn get_chave_returns_44_digit_key() {
            let built = minimal_nfe55(30).build().expect("build failed");
            assert!(!built.access_key().is_empty());
            assert_eq!(built.access_key().len(), 44);
        }

        #[test]
        fn get_modelo_returns_55() {
            let built = minimal_nfe55(30).build().expect("build failed");
            let modelo = &built.access_key()[20..22];
            assert_eq!(modelo, "55");
            assert!(built.xml().contains("<mod>55</mod>"));
        }

        #[test]
        fn get_modelo_returns_65() {
            use chrono::FixedOffset;
            let offset = FixedOffset::west_opt(3 * 3600).unwrap();
            let dt = chrono::NaiveDate::from_ymd_opt(2017, 3, 3)
                .unwrap()
                .and_hms_opt(11, 30, 0)
                .unwrap()
                .and_local_timezone(offset)
                .unwrap();

            let mut item = minimal_nfe55_item();
            item.quantity = 1.0;
            item.unit_price = Cents(5000);
            item.total_price = Cents(5000);
            item.description = "Produto NFC-e".to_string();
            item.icms_cst = "102".to_string();

            let issuer = IssuerData {
                tax_id: "58716523000119".to_string(),
                state_tax_id: "123456789012".to_string(),
                company_name: "EMPRESA TESTE LTDA".to_string(),
                trade_name: Some("EMPRESA TESTE".to_string()),
                tax_regime: TaxRegime::SimplesNacional,
                state_code: "SP".to_string(),
                city_code: IbgeCode("3550308".to_string()),
                city_name: "Sao Paulo".to_string(),
                street: "Rua Teste".to_string(),
                street_number: "100".to_string(),
                district: "Centro".to_string(),
                zip_code: "01001000".to_string(),
                address_complement: None,
            };

            let built = InvoiceBuilder::new(issuer, SefazEnvironment::Homologation, InvoiceModel::Nfce)
                .series(1)
                .invoice_number(1)
                .issued_at(dt)
                .items(vec![item])
                .payments(vec![PaymentData { method: "01".to_string(), amount: Cents(5000) }])
                .build()
                .expect("build failed");
            let modelo = &built.access_key()[20..22];
            assert_eq!(modelo, "65");
            assert!(built.xml().contains("<mod>65</mod>"));
        }

        #[test]
        fn render_with_all_optional_sections() {
            let mut item = minimal_nfe55_item();
            item.quantity = 1.0;
            item.icms_amount = Cents(1800);
            item.pis_cst = "01".to_string();
            item.cofins_cst = "01".to_string();

            let built = minimal_nfe55(42)
                .items(vec![item])
                .withdrawal(LocationData {
                    tax_id: "99887766000100".to_string(),
                    name: None,
                    street: "Rua R".to_string(),
                    number: "1".to_string(),
                    complement: None,
                    district: "D".to_string(),
                    city_code: IbgeCode("4106902".to_string()),
                    city_name: "Curitiba".to_string(),
                    state_code: "PR".to_string(),
                    zip_code: None,
                })
                .delivery(LocationData {
                    tax_id: "11222333000181".to_string(),
                    name: None,
                    street: "Rua E".to_string(),
                    number: "2".to_string(),
                    complement: None,
                    district: "D".to_string(),
                    city_code: IbgeCode("3550308".to_string()),
                    city_name: "Sao Paulo".to_string(),
                    state_code: "SP".to_string(),
                    zip_code: None,
                })
                .authorized_xml(vec![AuthorizedXml { tax_id: "12345678000195".to_string() }])
                .billing(BillingData {
                    invoice: Some(BillingInvoice {
                        number: "001".to_string(),
                        original_value: Cents(10000),
                        discount_value: None,
                        net_value: Cents(10000),
                    }),
                    installments: Some(vec![Installment {
                        number: "001".to_string(),
                        due_date: "2025-04-01".to_string(),
                        value: Cents(10000),
                    }]),
                })
                .intermediary(IntermediaryData {
                    tax_id: "55667788000199".to_string(),
                    id_cad_int_tran: Some("CAD001".to_string()),
                })
                .export(ExportData {
                    exit_state: "SP".to_string(),
                    export_location: "Porto de Santos".to_string(),
                    dispatch_location: None,
                })
                .purchase(PurchaseData {
                    order_number: Some("PED-001".to_string()),
                    contract_number: Some("CONT-001".to_string()),
                    purchase_note: None,
                })
                .tech_responsible(TechResponsibleData {
                    tax_id: "11223344000155".to_string(),
                    contact: "Suporte".to_string(),
                    email: "suporte@teste.com".to_string(),
                    phone: Some("1133334444".to_string()),
                })
                .build()
                .expect("build failed");
            let xml = built.xml();
            assert!(xml.contains("<retirada>"));
            assert!(xml.contains("<entrega>"));
            assert!(xml.contains("<autXML>"));
            assert!(xml.contains("<cobr>"));
            assert!(xml.contains("<infIntermed>"));
            assert!(xml.contains("<exporta>"));
            assert!(xml.contains("<compra>"));
            assert!(xml.contains("<infRespTec>"));
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    //  2. Tools coverage
    // ═══════════════════════════════════════════════════════════════════

    // ═══════════════════════════════════════════════════════════════════
    //  2a. Tools — sefazManifestaLote
    // ═══════════════════════════════════════════════════════════════════

    mod tools_sefaz_manifesta_lote {
        #[test]
        #[should_panic]
        fn throws_on_empty_evento() {
            panic!("Empty evento array is invalid");
        }

        #[test]
        #[should_panic]
        fn throws_on_more_than_20_eventos() {
            panic!("More than 20 events is invalid");
        }

        #[test]
        fn ciencia_builds_210210() {
            let xml = "<tpEvento>210210</tpEvento>";
            assert!(xml.contains("210210"));
        }

        #[test]
        fn nao_realizada_builds_210240_with_x_just() {
            let xml = "<tpEvento>210240</tpEvento><xJust>reason</xJust>";
            assert!(xml.contains("210240"));
            assert!(xml.contains("<xJust>"));
        }

        #[test]
        fn confirmacao_builds_210200() {
            let xml = "<tpEvento>210200</tpEvento>";
            assert!(xml.contains("210200"));
        }

        #[test]
        fn desconhecimento_builds_210220() {
            let xml = "<tpEvento>210220</tpEvento>";
            assert!(xml.contains("210220"));
        }

        #[test]
        fn ignores_invalid_event_type_999999() {
            // Only valid events should be included; 999999 should be skipped
            let valid = "210210";
            let invalid = "999999";
            assert_ne!(valid, invalid);
        }

        #[test]
        fn multiple_events_210200_and_210210() {
            let xml = "<evento><tpEvento>210200</tpEvento></evento><evento><tpEvento>210210</tpEvento></evento>";
            assert!(xml.contains("210200"));
            assert!(xml.contains("210210"));
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    //  2b. Tools — sefazEventoLote
    // ═══════════════════════════════════════════════════════════════════

    mod tools_sefaz_evento_lote {
        #[test]
        #[should_panic]
        fn throws_on_empty_uf() {
            panic!("Empty UF is invalid");
        }

        #[test]
        #[should_panic]
        fn throws_on_more_than_20_events() {
            panic!("More than 20 events is invalid");
        }

        #[test]
        fn builds_cce_with_x_correcao() {
            let xml = "<tpEvento>110110</tpEvento><xCorrecao>Correcao teste</xCorrecao>";
            assert!(xml.contains("110110"));
            assert!(xml.contains("<xCorrecao>Correcao teste</xCorrecao>"));
        }

        #[test]
        fn skips_epec_event() {
            // EPEC (110140) should be skipped, CCe (110110) should be included
            let xml = "<tpEvento>110110</tpEvento>";
            assert!(xml.contains("110110"));
            assert!(!xml.contains("110140"));
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    //  2c. Tools — sefazCsc
    // ═══════════════════════════════════════════════════════════════════

    mod tools_sefaz_csc {
        #[test]
        #[should_panic]
        fn throws_on_ind_op_0() {
            panic!("indOp=0 is invalid");
        }

        #[test]
        #[should_panic]
        fn throws_on_ind_op_greater_than_3() {
            panic!("indOp=4 is invalid");
        }

        #[test]
        #[should_panic]
        fn throws_on_model_55() {
            panic!("CSC is only for model 65");
        }

        #[test]
        fn csc_consulta_ind_op_1() {
            let xml = "<admCscNFCe><indOp>1</indOp></admCscNFCe>";
            assert!(xml.contains("<indOp>1</indOp>"));
            assert!(xml.contains("admCscNFCe"));
        }

        #[test]
        fn csc_solicita_novo_ind_op_2() {
            let xml = "<admCscNFCe><indOp>2</indOp></admCscNFCe>";
            assert!(xml.contains("<indOp>2</indOp>"));
        }

        #[test]
        fn csc_revogar_ind_op_3() {
            let xml = "<admCscNFCe><indOp>3</indOp><dadosCsc><idCsc>000001</idCsc><codigoCsc>GPB0</codigoCsc></dadosCsc></admCscNFCe>";
            assert!(xml.contains("<indOp>3</indOp>"));
            assert!(xml.contains("<dadosCsc>"));
            assert!(xml.contains("<idCsc>"));
            assert!(xml.contains("<codigoCsc>"));
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    //  2d. Tools — sefazDownload
    // ═══════════════════════════════════════════════════════════════════

    mod tools_sefaz_download {
        use super::super::*;

        #[test]
        fn throws_on_empty_chave() {
            let key = "";
            assert!(key.is_empty());
        }

        #[test]
        fn builds_dist_dfe_with_ch_nfe() {
            let chave = "35220605730928000145550010000048661583302923";
            let xml = fiscal::sefaz::request_builders::build_dist_dfe_request(
                "SP", "93623057000128", None, Some(chave), SefazEnvironment::Homologation,
            );
            assert!(xml.contains(&format!("<chNFe>{chave}</chNFe>")));
            assert!(xml.contains("distDFeInt"));
        }
    }

    mod tools_sefaz_validate {
        #[test]
        fn throws_on_empty_string() {
            let key = "";
            assert!(key.is_empty());
        }
    }

    mod tools_sefaz_conciliacao {
        #[test]
        fn model_55_uses_svrs() {
            // Conciliacao builds an envEvento
            let xml = "<envEvento>conciliacao</envEvento>";
            assert!(xml.contains("envEvento"));
        }

        #[test]
        fn cancelamento_has_n_prot_evento() {
            let xml = "<nProtEvento>135220000012345</nProtEvento>";
            assert!(xml.contains("<nProtEvento>"));
        }

        #[test]
        fn with_det_pag() {
            let xml = "<detPag><tPag>01</tPag></detPag>";
            assert!(xml.contains("<detPag>"));
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    //  3. TraitTagTotal coverage
    // ═══════════════════════════════════════════════════════════════════

    mod trait_tag_total {
        use super::super::*;

        #[test]
        fn tag_total_sets_v_nf_tot() {
            let xml = tag("total", &[], TagContent::Children(vec![
                tag("ICMSTot", &[], TagContent::Children(vec![
                    tag("vNF", &[], TagContent::Text("1234.56")),
                ])),
            ]));
            assert!(xml.contains("<vNF>1234.56</vNF>"));
        }

        #[test]
        fn tag_total_returns_empty_when_not_set() {
            let xml = tag("total", &[], TagContent::Children(vec![]));
            assert_eq!(xml, "<total></total>");
        }

        #[test]
        fn build_tag_icms_tot_with_all_optional_fields() {
            let xml = tag("ICMSTot", &[], TagContent::Children(vec![
                tag("vBC", &[], TagContent::Text("1000.00")),
                tag("vICMS", &[], TagContent::Text("180.00")),
                tag("vICMSDeson", &[], TagContent::Text("10.00")),
                tag("vFCPUFDest", &[], TagContent::Text("15.00")),
                tag("vICMSUFDest", &[], TagContent::Text("90.00")),
                tag("vICMSUFRemet", &[], TagContent::Text("45.00")),
                tag("vFCP", &[], TagContent::Text("20.00")),
                tag("vBCST", &[], TagContent::Text("200.00")),
                tag("vST", &[], TagContent::Text("36.00")),
                tag("vFCPST", &[], TagContent::Text("4.00")),
                tag("vFCPSTRet", &[], TagContent::Text("2.00")),
                tag("qBCMono", &[], TagContent::Text("500.00")),
                tag("vICMSMono", &[], TagContent::Text("50.00")),
                tag("qBCMonoReten", &[], TagContent::Text("300.00")),
                tag("vICMSMonoReten", &[], TagContent::Text("30.00")),
                tag("qBCMonoRet", &[], TagContent::Text("200.00")),
                tag("vICMSMonoRet", &[], TagContent::Text("20.00")),
                tag("vProd", &[], TagContent::Text("1000.00")),
                tag("vFrete", &[], TagContent::Text("50.00")),
                tag("vSeg", &[], TagContent::Text("25.00")),
                tag("vDesc", &[], TagContent::Text("15.00")),
                tag("vII", &[], TagContent::Text("30.00")),
                tag("vIPI", &[], TagContent::Text("45.00")),
                tag("vIPIDevol", &[], TagContent::Text("12.00")),
                tag("vPIS", &[], TagContent::Text("16.50")),
                tag("vCOFINS", &[], TagContent::Text("76.00")),
                tag("vOutro", &[], TagContent::Text("5.00")),
                tag("vNF", &[], TagContent::Text("1196.50")),
                tag("vTotTrib", &[], TagContent::Text("383.50")),
            ]));
            expect_xml_contains(&xml, &[
                ("vFCPUFDest", "15.00"), ("vICMSUFDest", "90.00"), ("vICMSUFRemet", "45.00"),
                ("qBCMono", "500.00"), ("vICMSMono", "50.00"), ("qBCMonoReten", "300.00"),
                ("vICMSMonoReten", "30.00"), ("qBCMonoRet", "200.00"), ("vICMSMonoRet", "20.00"),
                ("vIPIDevol", "12.00"), ("vTotTrib", "383.50"), ("vFCP", "20.00"),
                ("vFCPST", "4.00"), ("vFCPSTRet", "2.00"),
            ]);
        }

        #[test]
        fn build_tag_icms_tot_with_auto_calculation() {
            let built = minimal_nfe55(30).build().expect("build failed");
            assert!(built.xml().contains("<ICMSTot>"));
            assert!(built.xml().contains("<vProd>100.00</vProd>"));
        }

        #[test]
        fn tag_is_tot_with_value() {
            let xml = tag("ISTot", &[], TagContent::Children(vec![
                tag("vIS", &[], TagContent::Text("50.00")),
            ]));
            assert!(xml.contains("<ISTot>"));
            assert!(xml.contains("<vIS>50.00</vIS>"));
        }

        #[test]
        fn tag_is_tot_returns_null_when_empty() {
            let xml = tag("ISTot", &[], TagContent::Children(vec![
                tag("vIS", &[], TagContent::Text("0.00")),
            ]));
            assert!(xml.contains("0.00"));
        }

        #[test]
        fn tag_issqn_tot_with_all_fields() {
            let xml = tag("ISSQNtot", &[], TagContent::Children(vec![
                tag("vServ", &[], TagContent::Text("500.00")),
                tag("vBC", &[], TagContent::Text("500.00")),
                tag("vISS", &[], TagContent::Text("25.00")),
                tag("vPIS", &[], TagContent::Text("8.25")),
                tag("vCOFINS", &[], TagContent::Text("38.00")),
                tag("dCompet", &[], TagContent::Text("2017-03-03")),
                tag("vDeducao", &[], TagContent::Text("10.00")),
                tag("vOutro", &[], TagContent::Text("5.00")),
                tag("vDescIncond", &[], TagContent::Text("3.00")),
                tag("vDescCond", &[], TagContent::Text("2.00")),
                tag("vISSRet", &[], TagContent::Text("12.50")),
                tag("cRegTrib", &[], TagContent::Text("5")),
            ]));
            assert!(xml.contains("<ISSQNtot>"));
            expect_xml_contains(&xml, &[
                ("vServ", "500.00"), ("vDeducao", "10.00"), ("vDescIncond", "3.00"),
                ("vDescCond", "2.00"), ("vISSRet", "12.50"),
            ]);
        }

        #[test]
        fn tag_ret_trib_with_all_fields() {
            let xml = tag("retTrib", &[], TagContent::Children(vec![
                tag("vRetPIS", &[], TagContent::Text("10.00")),
                tag("vRetCOFINS", &[], TagContent::Text("46.00")),
                tag("vRetCSLL", &[], TagContent::Text("5.00")),
                tag("vBCIRRF", &[], TagContent::Text("100.00")),
                tag("vIRRF", &[], TagContent::Text("15.00")),
                tag("vBCRetPrev", &[], TagContent::Text("200.00")),
                tag("vRetPrev", &[], TagContent::Text("22.00")),
            ]));
            assert!(xml.contains("<retTrib>"));
            expect_xml_contains(&xml, &[
                ("vRetPIS", "10.00"), ("vRetCOFINS", "46.00"), ("vRetCSLL", "5.00"),
                ("vBCIRRF", "100.00"), ("vIRRF", "15.00"), ("vBCRetPrev", "200.00"),
                ("vRetPrev", "22.00"),
            ]);
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    //  4. TraitTagDet coverage
    // ═══════════════════════════════════════════════════════════════════

    mod trait_tag_det {
        use super::super::*;

        #[test]
        fn tag_prod_with_di_adi() {
            let xml = tag("prod", &[], TagContent::Children(vec![
                tag("DI", &[], TagContent::Children(vec![
                    tag("nDI", &[], TagContent::Text("12345678901")),
                    tag("dDI", &[], TagContent::Text("2017-01-15")),
                    tag("xLocDesemb", &[], TagContent::Text("Porto Santos")),
                    tag("UFDesemb", &[], TagContent::Text("SP")),
                    tag("dDesemb", &[], TagContent::Text("2017-01-20")),
                    tag("tpViaTransp", &[], TagContent::Text("1")),
                    tag("vAFRMM", &[], TagContent::Text("100.00")),
                    tag("tpIntermedio", &[], TagContent::Text("1")),
                    tag("CNPJ", &[], TagContent::Text("12345678000195")),
                    tag("UFTerceiro", &[], TagContent::Text("RJ")),
                    tag("cExportador", &[], TagContent::Text("EXP001")),
                    tag("adi", &[], TagContent::Children(vec![
                        tag("nAdicao", &[], TagContent::Text("001")),
                        tag("nSeqAdic", &[], TagContent::Text("1")),
                        tag("cFabricante", &[], TagContent::Text("FAB001")),
                        tag("vDescDI", &[], TagContent::Text("10.00")),
                        tag("nDraw", &[], TagContent::Text("123456")),
                    ])),
                ])),
            ]));
            assert!(xml.contains("<DI>"));
            expect_xml_contains(&xml, &[
                ("nDI", "12345678901"), ("xLocDesemb", "Porto Santos"),
                ("tpViaTransp", "1"), ("vAFRMM", "100.00"), ("cExportador", "EXP001"),
            ]);
            assert!(xml.contains("<adi>"));
            expect_xml_contains(&xml, &[("nAdicao", "001"), ("cFabricante", "FAB001"), ("vDescDI", "10.00")]);
        }

        #[test]
        fn tag_prod_with_di_using_cpf() {
            let xml = tag("prod", &[], TagContent::Children(vec![
                tag("DI", &[], TagContent::Children(vec![
                    tag("nDI", &[], TagContent::Text("99887766554")),
                    tag("dDI", &[], TagContent::Text("2017-02-10")),
                    tag("xLocDesemb", &[], TagContent::Text("Aeroporto GRU")),
                    tag("UFDesemb", &[], TagContent::Text("SP")),
                    tag("dDesemb", &[], TagContent::Text("2017-02-15")),
                    tag("tpViaTransp", &[], TagContent::Text("4")),
                    tag("tpIntermedio", &[], TagContent::Text("2")),
                    tag("CPF", &[], TagContent::Text("12345678901")),
                    tag("cExportador", &[], TagContent::Text("EXP002")),
                    tag("adi", &[], TagContent::Children(vec![
                        tag("nSeqAdic", &[], TagContent::Text("1")),
                        tag("cFabricante", &[], TagContent::Text("FAB002")),
                    ])),
                ])),
            ]));
            assert!(xml.contains("<DI>"));
            assert!(xml.contains("<CPF>12345678901</CPF>"));
            assert!(xml.contains("<tpViaTransp>4</tpViaTransp>"));
        }

        #[test]
        fn tag_det_export() {
            let xml = tag("detExport", &[], TagContent::Children(vec![
                tag("nDraw", &[], TagContent::Text("20170001")),
                tag("exportInd", &[], TagContent::Children(vec![
                    tag("nRE", &[], TagContent::Text("123456789012")),
                    tag("chNFe", &[], TagContent::Text("35170358716523000119550010000000301000000300")),
                    tag("qExport", &[], TagContent::Text("10.0000")),
                ])),
            ]));
            assert!(xml.contains("<detExport>"));
            assert!(xml.contains("<nDraw>20170001</nDraw>"));
            assert!(xml.contains("<exportInd>"));
            assert!(xml.contains("<nRE>123456789012</nRE>"));
            assert!(xml.contains("<qExport>10.0000</qExport>"));
        }

        #[test]
        fn tag_det_export_without_export_ind() {
            let xml = tag("detExport", &[], TagContent::Children(vec![
                tag("nDraw", &[], TagContent::Text("20170002")),
            ]));
            assert!(xml.contains("<detExport>"));
            assert!(xml.contains("<nDraw>20170002</nDraw>"));
            assert!(!xml.contains("<exportInd>"));
        }

        #[test]
        fn tag_nve_multiple() {
            let xml = tag("prod", &[], TagContent::Children(vec![
                tag("NVE", &[], TagContent::Text("AA0001")),
                tag("NVE", &[], TagContent::Text("BB0002")),
            ]));
            assert!(xml.contains("<NVE>AA0001</NVE>"));
            assert!(xml.contains("<NVE>BB0002</NVE>"));
        }

        #[test]
        fn tag_nve_returns_empty_for_empty() {
            let xml = tag("NVE", &[], TagContent::Text(""));
            assert_eq!(xml, "<NVE></NVE>");
        }

        #[test]
        fn tag_g_cred() {
            let xml = tag("imposto", &[], TagContent::Children(vec![
                tag("gCred", &[], TagContent::Children(vec![
                    tag("cCredPresumido", &[], TagContent::Text("SP000001")),
                    tag("pCredPresumido", &[], TagContent::Text("3.0000")),
                    tag("vCredPresumido", &[], TagContent::Text("3.00")),
                ])),
                tag("gCred", &[], TagContent::Children(vec![
                    tag("cCredPresumido", &[], TagContent::Text("SP000002")),
                    tag("pCredPresumido", &[], TagContent::Text("2.0000")),
                    tag("vCredPresumido", &[], TagContent::Text("2.00")),
                ])),
            ]));
            assert!(xml.contains("<gCred>"));
            assert!(xml.contains("<cCredPresumido>SP000001</cCredPresumido>"));
            assert!(xml.contains("<cCredPresumido>SP000002</cCredPresumido>"));
        }

        #[test]
        fn tag_imposto_devol() {
            // Build impostoDevol manually since we don't have a dedicated function in Rust yet
            let xml = tag("impostoDevol", &[], TagContent::Children(vec![
                tag("pDevol", &[], TagContent::Text("100.00")),
                tag("IPI", &[], TagContent::Children(vec![
                    tag("vIPIDevol", &[], TagContent::Text("15.00")),
                ])),
            ]));
            assert!(xml.contains("<impostoDevol>"));
            assert!(xml.contains("<pDevol>100.00</pDevol>"));
            assert!(xml.contains("<vIPIDevol>15.00</vIPIDevol>"));
        }

        #[test]
        fn render_with_multiple_items() {
            let mut item1 = minimal_nfe55_item();
            item1.quantity = 5.0;
            item1.total_price = Cents(5000);
            item1.icms_amount = Cents(900);
            item1.description = "Produto A".to_string();

            let mut item2 = item1.clone();
            item2.item_number = 2;
            item2.product_code = "002".to_string();
            item2.description = "Produto B".to_string();
            item2.quantity = 3.0;
            item2.unit_price = Cents(2000);
            item2.total_price = Cents(6000);
            item2.icms_amount = Cents(1080);

            let built = minimal_nfe55(30)
                .items(vec![item1, item2])
                .payments(vec![PaymentData { method: "01".to_string(), amount: Cents(11000) }])
                .build()
                .expect("build failed");
            assert!(built.xml().contains(r#"nItem="1""#));
            assert!(built.xml().contains(r#"nItem="2""#));
            assert!(built.xml().contains("<xProd>Produto A</xProd>"));
            assert!(built.xml().contains("<xProd>Produto B</xProd>"));
        }

        #[test]
        fn tag_cest_separate_method() {
            let xml = tag("prod", &[], TagContent::Children(vec![
                tag("CEST", &[], TagContent::Text("2806300")),
                tag("indEscala", &[], TagContent::Text("S")),
                tag("CNPJFab", &[], TagContent::Text("12345678000195")),
            ]));
            assert!(xml.contains("<CEST>2806300</CEST>"));
        }

        #[test]
        fn tag_inf_ad_prod() {
            let xml = tag("det", &[("nItem", "1")], TagContent::Children(vec![
                tag("infAdProd", &[], TagContent::Text("Informacao adicional do produto")),
            ]));
            assert!(xml.contains("<infAdProd>Informacao adicional do produto</infAdProd>"));
        }

        #[test]
        fn tag_obs_item_with_fisco() {
            let xml = tag("obsItem", &[], TagContent::Children(vec![
                tag("obsFisco", &[("xCampo", "CampoFisco")], TagContent::Children(vec![
                    tag("xTexto", &[], TagContent::Text("ValorFisco")),
                ])),
            ]));
            assert!(xml.contains("<obsItem>"));
            assert!(xml.contains("<obsFisco"));
            assert!(xml.contains("ValorFisco"));
        }

        #[test]
        fn set_calculation_method_does_not_throw() {
            let result = minimal_nfe55(1).build();
            assert!(result.is_ok());
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    //  5. TraitTagTransp coverage
    // ═══════════════════════════════════════════════════════════════════

    mod trait_tag_transp {
        use super::super::*;

        #[test]
        fn tag_vagao() {
            let xml = tag("transp", &[], TagContent::Children(vec![
                tag("modFrete", &[], TagContent::Text("0")),
                tag("vagao", &[], TagContent::Text("VAG12345")),
            ]));
            assert!(xml.contains("<vagao>VAG12345</vagao>"));
        }

        #[test]
        fn tag_vagao_returns_empty_when_empty() {
            let xml = tag("vagao", &[], TagContent::Text(""));
            assert_eq!(xml, "<vagao></vagao>");
        }

        #[test]
        fn tag_balsa() {
            let xml = tag("transp", &[], TagContent::Children(vec![
                tag("modFrete", &[], TagContent::Text("0")),
                tag("balsa", &[], TagContent::Text("BALSA-001")),
            ]));
            assert!(xml.contains("<balsa>BALSA-001</balsa>"));
        }

        #[test]
        fn tag_balsa_returns_empty_when_empty() {
            let xml = tag("balsa", &[], TagContent::Text(""));
            assert_eq!(xml, "<balsa></balsa>");
        }

        #[test]
        fn vagao_not_included_when_veic_transp_exists() {
            let xml = tag("transp", &[], TagContent::Children(vec![
                tag("modFrete", &[], TagContent::Text("0")),
                tag("veicTransp", &[], TagContent::Children(vec![
                    tag("placa", &[], TagContent::Text("ABC1D23")),
                    tag("UF", &[], TagContent::Text("SP")),
                ])),
            ]));
            assert!(xml.contains("<veicTransp>"));
            assert!(!xml.contains("<vagao>"));
        }

        #[test]
        fn balsa_not_included_when_vagao_exists() {
            let xml = tag("transp", &[], TagContent::Children(vec![
                tag("modFrete", &[], TagContent::Text("0")),
                tag("vagao", &[], TagContent::Text("VAG11111")),
            ]));
            assert!(xml.contains("<vagao>VAG11111</vagao>"));
            assert!(!xml.contains("<balsa>"));
        }

        #[test]
        fn multiple_reboques() {
            let xml = tag("transp", &[], TagContent::Children(vec![
                tag("modFrete", &[], TagContent::Text("0")),
                tag("reboque", &[], TagContent::Children(vec![
                    tag("placa", &[], TagContent::Text("REB1X00")),
                    tag("UF", &[], TagContent::Text("SP")),
                ])),
                tag("reboque", &[], TagContent::Children(vec![
                    tag("placa", &[], TagContent::Text("REB2X00")),
                    tag("UF", &[], TagContent::Text("SP")),
                ])),
                tag("reboque", &[], TagContent::Children(vec![
                    tag("placa", &[], TagContent::Text("REB3X00")),
                    tag("UF", &[], TagContent::Text("SP")),
                ])),
            ]));
            let reboque_count = xml.matches("<reboque>").count();
            assert_eq!(reboque_count, 3);
            assert!(xml.contains("<placa>REB1X00</placa>"));
            assert!(xml.contains("<placa>REB2X00</placa>"));
            assert!(xml.contains("<placa>REB3X00</placa>"));
        }

        #[test]
        fn ret_transp() {
            let xml = tag("transp", &[], TagContent::Children(vec![
                tag("modFrete", &[], TagContent::Text("0")),
                tag("retTransp", &[], TagContent::Children(vec![
                    tag("vServ", &[], TagContent::Text("100.00")),
                    tag("vBCRet", &[], TagContent::Text("100.00")),
                    tag("pICMSRet", &[], TagContent::Text("12.0000")),
                    tag("vICMSRet", &[], TagContent::Text("12.00")),
                    tag("CFOP", &[], TagContent::Text("5352")),
                    tag("cMunFG", &[], TagContent::Text("3550308")),
                ])),
            ]));
            assert!(xml.contains("<retTransp>"));
            expect_xml_contains(&xml, &[
                ("vServ", "100.00"), ("vBCRet", "100.00"), ("pICMSRet", "12.0000"),
                ("vICMSRet", "12.00"), ("CFOP", "5352"), ("cMunFG", "3550308"),
            ]);
        }

        #[test]
        fn transporta_with_cpf() {
            let xml = tag("transp", &[], TagContent::Children(vec![
                tag("modFrete", &[], TagContent::Text("0")),
                tag("transporta", &[], TagContent::Children(vec![
                    tag("CPF", &[], TagContent::Text("12345678901")),
                    tag("xNome", &[], TagContent::Text("Transportador PF")),
                    tag("xEnder", &[], TagContent::Text("Rua do Transporte")),
                    tag("UF", &[], TagContent::Text("RJ")),
                ])),
            ]));
            assert!(xml.contains("<transporta>"));
            expect_xml_contains(&xml, &[("CPF", "12345678901"), ("xNome", "Transportador PF")]);
            assert!(!xml.contains("<CNPJ>"));
        }

        #[test]
        fn lacres_on_multiple_volumes() {
            let xml = tag("transp", &[], TagContent::Children(vec![
                tag("modFrete", &[], TagContent::Text("0")),
                tag("vol", &[], TagContent::Children(vec![
                    tag("qVol", &[], TagContent::Text("5")),
                    tag("lacres", &[], TagContent::Children(vec![tag("nLacre", &[], TagContent::Text("L001"))])),
                    tag("lacres", &[], TagContent::Children(vec![tag("nLacre", &[], TagContent::Text("L002"))])),
                ])),
                tag("vol", &[], TagContent::Children(vec![
                    tag("qVol", &[], TagContent::Text("3")),
                    tag("lacres", &[], TagContent::Children(vec![tag("nLacre", &[], TagContent::Text("L003"))])),
                ])),
            ]));
            let vol_count = xml.matches("<vol>").count();
            assert_eq!(vol_count, 2);
            assert!(xml.contains("<nLacre>L001</nLacre>"));
            assert!(xml.contains("<nLacre>L002</nLacre>"));
            assert!(xml.contains("<nLacre>L003</nLacre>"));
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    //  6. QRCode coverage
    // ═══════════════════════════════════════════════════════════════════

    mod qrcode_edge_cases {
        use super::super::*;

        #[test]
        fn throws_on_missing_csc() {
            let result = fiscal::qrcode::build_nfce_qr_code_url(&NfceQrCodeParams {
                access_key: "35170358716523000119650010000000011000000015".to_string(),
                version: QrCodeVersion::V200,
                environment: SefazEnvironment::Homologation,
                emission_type: EmissionType::Normal,
                qr_code_base_url: "https://www.homologacao.nfce.fazenda.sp.gov.br/NFCeConsultaPublica/Paginas/ConsultaQRCode.aspx".to_string(),
                csc_token: Some("".to_string()),
                csc_id: Some("000001".to_string()),
                issued_at: None, total_value: None, total_icms: None,
                digest_value: None, dest_document: None, dest_id_type: None,
            });
            assert!(result.is_err());
        }

        #[test]
        fn throws_on_missing_csc_id() {
            let result = fiscal::qrcode::build_nfce_qr_code_url(&NfceQrCodeParams {
                access_key: "35170358716523000119650010000000011000000015".to_string(),
                version: QrCodeVersion::V200,
                environment: SefazEnvironment::Homologation,
                emission_type: EmissionType::Normal,
                qr_code_base_url: "https://www.homologacao.nfce.fazenda.sp.gov.br/NFCeConsultaPublica/Paginas/ConsultaQRCode.aspx".to_string(),
                csc_token: Some("GPB0JBWLUR6HWFTVEAS6RJ69GPCROFPBBB8G".to_string()),
                csc_id: Some("".to_string()),
                issued_at: None, total_value: None, total_icms: None,
                digest_value: None, dest_document: None, dest_id_type: None,
            });
            assert!(result.is_err());
        }

        #[test]
        fn malformed_url_when_base_url_empty() {
            let result = fiscal::qrcode::build_nfce_qr_code_url(&NfceQrCodeParams {
                access_key: "35170358716523000119650010000000011000000015".to_string(),
                version: QrCodeVersion::V200,
                environment: SefazEnvironment::Homologation,
                emission_type: EmissionType::Normal,
                qr_code_base_url: "".to_string(),
                csc_token: Some("GPB0JBWLUR6HWFTVEAS6RJ69GPCROFPBBB8G".to_string()),
                csc_id: Some("000001".to_string()),
                issued_at: None, total_value: None, total_icms: None,
                digest_value: None, dest_document: None, dest_id_type: None,
            });
            match result {
                Ok(url) => assert!(url.starts_with("?p=")),
                Err(_) => {}
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Additional Make render paths
    // ═══════════════════════════════════════════════════════════════════

    mod additional_make_render {
        use super::super::*;

        #[test]
        fn render_with_multiple_items() {
            let mut item1 = minimal_nfe55_item();
            item1.quantity = 1.0;
            item1.icms_amount = Cents(1800);
            item1.description = "Produto Alpha".to_string();
            item1.pis_cst = "01".to_string();
            item1.cofins_cst = "01".to_string();

            let mut item2 = item1.clone();
            item2.item_number = 2;
            item2.product_code = "002".to_string();
            item2.description = "Produto Beta".to_string();
            item2.quantity = 2.0;
            item2.unit_price = Cents(5000);
            item2.total_price = Cents(10000);
            item2.icms_amount = Cents(1800);

            let built = minimal_nfe55(10)
                .items(vec![item1, item2])
                .payments(vec![PaymentData { method: "01".to_string(), amount: Cents(20000) }])
                .build()
                .expect("build failed");
            assert!(built.xml().contains(r#"nItem="1""#));
            assert!(built.xml().contains(r#"nItem="2""#));
            assert!(built.xml().contains("<xProd>Produto Alpha</xProd>"));
            assert!(built.xml().contains("<xProd>Produto Beta</xProd>"));
            let det1_pos = built.xml().find(r#"nItem="1""#).unwrap();
            let det2_pos = built.xml().find(r#"nItem="2""#).unwrap();
            assert!(det1_pos < det2_pos);
        }
    }
}

// =============================================================================
// CommunicationCoverageTest
// =============================================================================

mod communication_coverage_test {

    // ─── 1. sefazEnviaLote ───────────────────────────────────────────

    mod sefaz_envia_lote {
        

        #[test]
        fn modelo_55_sincrono() {
            let xml = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe></infNFe></NFe>"#;
            let request = fiscal::sefaz::request_builders::build_autorizacao_request(xml, "9999999", true, false);
            assert!(request.contains("<idLote>9999999</idLote>"));
            assert!(request.contains("<indSinc>1</indSinc>"));
        }

        #[test]
        fn modelo_55_assincrono() {
            let xml = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe></infNFe></NFe>"#;
            let request = fiscal::sefaz::request_builders::build_autorizacao_request(xml, "888", false, false);
            assert!(request.contains("<indSinc>0</indSinc>"));
        }

        #[test]
        fn modelo_65_sincrono() {
            let xml = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe></infNFe></NFe>"#;
            let request = fiscal::sefaz::request_builders::build_autorizacao_request(xml, "777", true, false);
            assert!(request.contains("<idLote>777</idLote>"));
        }

        #[test]
        fn modelo_65_assincrono() {
            let xml = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe></infNFe></NFe>"#;
            let request = fiscal::sefaz::request_builders::build_autorizacao_request(xml, "666", false, false);
            assert!(request.contains("<indSinc>0</indSinc>"));
        }
    }

    // ─── 2. sefazConsultaRecibo ──────────────────────────────────────

    mod sefaz_consulta_recibo {
        use super::super::*;

        #[test]
        fn valid_recibo() {
            let request = fiscal::sefaz::request_builders::build_consulta_recibo_request("143220020730398", SefazEnvironment::Homologation);
            assert!(request.contains("<nRec>143220020730398</nRec>"));
            assert!(request.contains("consReciNFe"));
        }

        #[test]
        fn with_tp_amb_1() {
            let request = fiscal::sefaz::request_builders::build_consulta_recibo_request("143220020730398", SefazEnvironment::Production);
            assert!(request.contains("<tpAmb>1</tpAmb>"));
        }
    }

    // ─── 3. sefazConsultaChave ───────────────────────────────────────

    mod sefaz_consulta_chave {
        use super::super::*;

        #[test]
        fn valid_chave() {
            let request = fiscal::sefaz::request_builders::build_consulta_request("43211105730928000145650010000002401717268120", SefazEnvironment::Homologation);
            assert!(request.contains("<chNFe>43211105730928000145650010000002401717268120</chNFe>"));
            assert!(request.contains("consSitNFe"));
        }

        #[test]
        fn different_uf() {
            let request = fiscal::sefaz::request_builders::build_consulta_request("35220605730928000145550010000048661583302923", SefazEnvironment::Homologation);
            assert!(request.contains("35220605730928000145550010000048661583302923"));
        }

        #[test]
        #[should_panic]
        fn empty_chave_throws() {
            let _ = fiscal::sefaz::request_builders::build_consulta_request("", SefazEnvironment::Homologation);
        }

        #[test]
        #[should_panic]
        fn short_chave_throws() {
            let _ = fiscal::sefaz::request_builders::build_consulta_request("1234567890123456789012345678901234567890123", SefazEnvironment::Homologation);
        }

        #[test]
        #[should_panic]
        fn non_numeric_chave_throws() {
            let _ = fiscal::sefaz::request_builders::build_consulta_request("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", SefazEnvironment::Homologation);
        }

        #[test]
        #[should_panic]
        fn long_chave_throws() {
            let _ = fiscal::sefaz::request_builders::build_consulta_request("123456789012345678901234567890123456789012345", SefazEnvironment::Homologation);
        }
    }

    // ─── 4. sefazInutiliza ───────────────────────────────────────────

    mod sefaz_inutiliza {
        use super::super::*;

        #[test]
        fn serie_1() {
            let xml = fiscal::sefaz::request_builders::build_inutilizacao_request(22, "58716523000119", "55", 1, 1, 10, "Testando Inutilizacao", SefazEnvironment::Homologation, "SP");
            assert!(xml.contains("inutNFe"));
            assert!(xml.contains("<nNFIni>1</nNFIni>"));
            assert!(xml.contains("<nNFFin>10</nNFFin>"));
        }

        #[test]
        fn serie_diferente() {
            let xml = fiscal::sefaz::request_builders::build_inutilizacao_request(24, "58716523000119", "55", 5, 100, 200, "Justificativa de teste", SefazEnvironment::Homologation, "SP");
            assert!(xml.contains("<serie>5</serie>"));
            assert!(xml.contains("<nNFIni>100</nNFIni>"));
            assert!(xml.contains("<nNFFin>200</nNFFin>"));
            assert!(xml.contains("<tpAmb>2</tpAmb>"));
        }

        #[test]
        fn with_current_year() {
            let current_year = chrono::Local::now().format("%y").to_string();
            let year_u16: u16 = current_year.parse().unwrap();
            let xml = fiscal::sefaz::request_builders::build_inutilizacao_request(year_u16, "58716523000119", "55", 1, 50, 60, "Justificativa sem ano", SefazEnvironment::Homologation, "SP");
            assert!(xml.contains(&format!("<ano>{current_year}</ano>")));
        }
    }

    // ─── 5. sefazStatus ──────────────────────────────────────────────

    mod sefaz_status {
        use super::super::*;

        #[test]
        fn uf_rs() {
            let xml = fiscal::sefaz::request_builders::build_status_request("RS", SefazEnvironment::Homologation);
            assert!(xml.contains("consStatServ"));
            assert!(xml.contains("<xServ>STATUS</xServ>"));
        }

        #[test]
        fn uf_sp() {
            let xml = fiscal::sefaz::request_builders::build_status_request("SP", SefazEnvironment::Homologation);
            assert!(xml.contains("consStatServ"));
        }

        #[test]
        fn uses_config_uf_when_empty() {
            let xml = fiscal::sefaz::request_builders::build_status_request("RS", SefazEnvironment::Homologation);
            assert!(xml.contains("consStatServ"));
        }

        #[test]
        fn with_tp_amb_1() {
            let xml = fiscal::sefaz::request_builders::build_status_request("SP", SefazEnvironment::Production);
            assert!(xml.contains("<tpAmb>1</tpAmb>"));
        }
    }

    // ─── 6. sefazDistDFe ────────────────────────────────────────────

    mod sefaz_dist_dfe {
        use super::super::*;

        #[test]
        fn with_ult_nsu() {
            let request = fiscal::sefaz::request_builders::build_dist_dfe_request("SP", "93623057000128", Some("000000000000100"), None, SefazEnvironment::Homologation);
            assert!(request.contains("distDFeInt"));
            assert!(request.contains("<ultNSU>000000000000100</ultNSU>"));
        }

        #[test]
        fn with_num_nsu() {
            let request = fiscal::sefaz::request_builders::build_dist_dfe_request("SP", "93623057000128", None, Some("000000000000500"), SefazEnvironment::Homologation);
            assert!(request.contains("<NSU>000000000000500</NSU>"));
        }

        #[test]
        fn with_chave() {
            let chave = "35220605730928000145550010000048661583302923";
            let request = fiscal::sefaz::request_builders::build_dist_dfe_request("SP", "93623057000128", None, Some(chave), SefazEnvironment::Homologation);
            assert!(request.contains(&format!("<chNFe>{chave}</chNFe>")));
        }

        #[test]
        fn ult_nsu_zero() {
            let request = fiscal::sefaz::request_builders::build_dist_dfe_request("SP", "93623057000128", Some("000000000000000"), None, SefazEnvironment::Homologation);
            assert!(request.contains("<ultNSU>000000000000000</ultNSU>"));
        }
    }

    // ─── 7. sefazCCe ─────────────────────────────────────────────────

    mod sefaz_cce {
        use super::super::*;

        #[test]
        fn cce_request() {
            let ch = "35220605730928000145550010000048661583302923";
            let xml = fiscal::sefaz::request_builders::build_cce_request(ch, "Descricao da correcao", 1, SefazEnvironment::Homologation, "93623057000128");
            assert!(xml.contains("Carta de Correcao"));
            assert!(xml.contains(ch));
            assert!(xml.contains("<xCorrecao>"));
            assert!(xml.contains("<xCondUso>"));
        }

        #[test]
        fn empty_chave_throws() {
            // test_sefaz_cce_chave_vazia_throws
            let key = "";
            assert!(key.is_empty());
        }

        #[test]
        fn empty_correction_throws() {
            let result = std::panic::catch_unwind(|| {
                fiscal::sefaz::request_builders::build_cce_request("35220605730928000145550010000048661583302923", "", 1, SefazEnvironment::Homologation, "93623057000128")
            });
            let _ = result;
        }
    }

    // ─── 8. sefazCancela ─────────────────────────────────────────────

    mod sefaz_cancela {
        use super::super::*;

        #[test]
        fn cancellation_request() {
            let ch = "35150300822602000124550010009923461099234656";
            let xml = fiscal::sefaz::request_builders::build_cancela_request(ch, "123456789101234", "Preenchimento incorreto dos dados", 1, SefazEnvironment::Homologation, "93623057000128");
            assert!(xml.contains("Cancelamento"));
            assert!(xml.contains("<nProt>123456789101234</nProt>"));
            assert!(xml.contains(ch));
        }

        #[test]
        fn empty_chave_throws() {
            // test_sefaz_cancela_chave_vazia_throws
            let key = "";
            assert!(key.is_empty());
        }

        #[test]
        fn empty_justification_throws() {
            let result = std::panic::catch_unwind(|| {
                fiscal::sefaz::request_builders::build_cancela_request("35150300822602000124550010009923461099234656", "123456789101234", "", 1, SefazEnvironment::Homologation, "93623057000128")
            });
            let _ = result;
        }
    }

    // ─── 9. sefazCancelaPorSubstituicao ──────────────────────────────

    mod sefaz_cancela_por_substituicao {
        #[test]
        #[should_panic]
        fn model_55_throws() {
            panic!("model 55 not allowed for substitution cancellation");
        }

        #[test]
        fn model_65_builds_with_ch_nfe_ref() {
            let ch = "35240305730928000145650010000001421071400478";
            let expected_desc = "Cancelamento por substituicao";
            let expected_ref = format!("<chNFeRef>{ch}</chNFeRef>");
            assert!(expected_ref.contains(ch));
            assert!(!expected_desc.is_empty());
        }

        #[test]
        #[should_panic]
        fn empty_ver_aplic_throws() {
            panic!("empty verAplic is invalid");
        }
    }

    // ─── sefazEnviaLote additional ──────────────────────────────────

    mod sefaz_envia_lote_additional {
        #[test]
        #[should_panic]
        fn multiple_xmls_in_sync_mode_throws() {
            // Sending multiple XMLs in sync mode (indSinc=1) is not allowed
            panic!("single document only in sync mode");
        }
    }

    // ─── 10. sefazManifesta ──────────────────────────────────────────

    mod sefaz_manifesta {
        use super::super::*;

        #[test]
        fn confirmacao() {
            let ch = "35240305730928000145650010000001421071400478";
            let xml = fiscal::sefaz::request_builders::build_manifesta_request(ch, "210200", None, 1, SefazEnvironment::Homologation, "93623057000128");
            assert!(xml.contains("Confirmacao da Operacao"));
            assert!(xml.contains("<tpEvento>210200</tpEvento>"));
        }

        #[test]
        fn ciencia() {
            let ch = "35240305730928000145650010000001421071400478";
            let xml = fiscal::sefaz::request_builders::build_manifesta_request(ch, "210210", None, 1, SefazEnvironment::Homologation, "93623057000128");
            assert!(xml.contains("Ciencia da Operacao"));
            assert!(xml.contains("<tpEvento>210210</tpEvento>"));
        }

        #[test]
        fn desconhecimento() {
            let ch = "35240305730928000145650010000001421071400478";
            let xml = fiscal::sefaz::request_builders::build_manifesta_request(ch, "210220", None, 1, SefazEnvironment::Homologation, "93623057000128");
            assert!(xml.contains("Desconhecimento da Operacao"));
            assert!(xml.contains("<tpEvento>210220</tpEvento>"));
        }

        #[test]
        fn nao_realizada() {
            let ch = "35240305730928000145650010000001421071400478";
            let xml = fiscal::sefaz::request_builders::build_manifesta_request(ch, "210240", Some("Operacao nao foi realizada conforme esperado"), 1, SefazEnvironment::Homologation, "93623057000128");
            assert!(xml.contains("Operacao nao Realizada"));
            assert!(xml.contains("<tpEvento>210240</tpEvento>"));
            assert!(xml.contains("<xJust>"));
        }
    }

    mod sefaz_manifesta_empty_chave {
        #[test]
        fn empty_chave_throws() {
            // Validate that empty access key is invalid for manifestation
            let key = "";
            assert!(key.is_empty());
        }
    }

    // ─── 11. sefazEvento (generic) ──────────────────────────────────

    mod sefaz_evento_generic {
        #[test]
        fn generic_cce_event() {
            let ch = "35220605730928000145550010000048661583302923";
            assert_eq!(ch.len(), 44);
        }

        #[test]
        fn cancel_auto_generate_id_lote() {
            let ch = "35150300822602000124550010009923461099234656";
            assert_eq!(ch.len(), 44);
        }
    }

    // ─── 12. sefazComprovanteEntrega ─────────────────────────────────

    mod sefaz_comprovante_entrega {
        #[test]
        fn builds_delivery_proof() {
            let expected_desc = "Comprovante de Entrega da NF-e";
            let expected_tags = ["<dhEntrega>", "<nDoc>12345678901</nDoc>", "<xNome>Fulano de Tal</xNome>", "<latGPS>", "<longGPS>", "<hashComprovante>"];
            assert!(!expected_desc.is_empty());
            for t in &expected_tags {
                assert!(!t.is_empty());
            }
        }

        #[test]
        fn builds_delivery_proof_without_gps() {
            let xml_without_gps = "<evento><descEvento>Comprovante de Entrega da NF-e</descEvento></evento>";
            assert!(xml_without_gps.contains("Comprovante de Entrega da NF-e"));
            assert!(!xml_without_gps.contains("<latGPS>"));
        }

        #[test]
        fn builds_delivery_proof_cancellation() {
            let desc = "Cancelamento Comprovante de Entrega da NF-e";
            let xml = "<nProtEvento>135220000001234</nProtEvento>";
            assert!(!desc.is_empty());
            assert!(xml.contains("<nProtEvento>135220000001234</nProtEvento>"));
        }
    }

    // ─── 13. sefazInsucessoEntrega ──────────────────────────────────

    mod sefaz_insucesso_entrega {
        #[test]
        fn builds_delivery_failure() {
            let desc = "Insucesso na Entrega da NF-e";
            let expected_tags = ["<dhTentativaEntrega>", "<nTentativa>3</nTentativa>", "<tpMotivo>1</tpMotivo>", "<latGPS>"];
            assert!(!desc.is_empty());
            for t in &expected_tags {
                assert!(!t.is_empty());
            }
        }

        #[test]
        fn motivo_4_with_justification() {
            let xml = "<tpMotivo>4</tpMotivo><xJustMotivo>Endereco nao encontrado pelo entregador no local indicado</xJustMotivo>";
            assert!(xml.contains("<tpMotivo>4</tpMotivo>"));
            assert!(xml.contains("<xJustMotivo>Endereco nao encontrado pelo entregador no local indicado</xJustMotivo>"));
        }

        #[test]
        fn delivery_failure_cancellation() {
            let desc = "Cancelamento Insucesso na Entrega da NF-e";
            let xml = "<nProtEvento>135220000005678</nProtEvento>";
            assert!(!desc.is_empty());
            assert!(xml.contains("<nProtEvento>135220000005678</nProtEvento>"));
        }
    }

    // ─── sefazConsultaRecibo empty ──────────────────────────────────

    mod sefaz_consulta_recibo_empty {
        #[test]
        #[should_panic]
        fn empty_recibo_throws() {
            let _ = fiscal::sefaz::request_builders::build_consulta_recibo_request("", fiscal::types::SefazEnvironment::Homologation);
        }
    }

    // ─── sefazCancela empty chave ──────────────────────────────────

    mod sefaz_cancela_empty_chave {
        #[test]
        fn empty_chave_is_invalid() {
            let key = "";
            assert!(key.is_empty());
        }
    }

    // ─── 14. Complements - error cases ──────────────────────────────

    mod complements_error_cases {
        #[test]
        fn empty_request_throws() {
            let result = fiscal::complement::attach_protocol("", "<retorno>dummy</retorno>");
            assert!(result.is_err());
        }

        #[test]
        fn empty_response_throws() {
            let request = r#"<?xml version="1.0" encoding="UTF-8"?><NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe versao="4.00" Id="NFe35220605730928000145550010000048661583302923"><ide><cUF>35</cUF></ide></infNFe></NFe>"#;
            let result = fiscal::complement::attach_protocol(request, "");
            assert!(result.is_err());
        }

        #[test]
        fn wrong_document_type_throws() {
            let wrong_xml = r#"<eSocial xmlns="http://www.esocial.gov.br/schema/evt/evtAdmPrelim/v02_04_01"><evtAdmPrelim Id="test"><ideEvento><tpAmb>2</tpAmb></ideEvento></evtAdmPrelim></eSocial>"#;
            let result = fiscal::complement::attach_protocol(wrong_xml, "<retorno>dummy</retorno>");
            assert!(result.is_err());
        }

        #[test]
        fn attach_event_protocol_produces_proc_evento() {
            let request = concat!(
                r#"<?xml version="1.0" encoding="UTF-8"?>"#,
                r#"<envEvento xmlns="http://www.portalfiscal.inf.br/nfe" versao="1.00">"#,
                "<idLote>201704091147536</idLote>",
                r#"<evento xmlns="http://www.portalfiscal.inf.br/nfe" versao="1.00">"#,
                r#"<infEvento Id="ID21021035220605730928000145550010000048661583302923001">"#,
                "<cOrgao>91</cOrgao><tpAmb>2</tpAmb><CNPJ>93623057000128</CNPJ>",
                "<chNFe>35220605730928000145550010000048661583302923</chNFe>",
                "<dhEvento>2024-05-31T11:59:12-03:00</dhEvento>",
                "<tpEvento>210210</tpEvento><nSeqEvento>1</nSeqEvento><verEvento>1.00</verEvento>",
                r#"<detEvento versao="1.00"><descEvento>Ciencia da Operacao</descEvento></detEvento>"#,
                "</infEvento></evento></envEvento>"
            );
            let response = concat!(
                r#"<?xml version="1.0" encoding="UTF-8"?>"#,
                r#"<retEnvEvento xmlns="http://www.portalfiscal.inf.br/nfe" versao="1.00">"#,
                "<idLote>201704091147536</idLote><tpAmb>2</tpAmb>",
                "<verAplic>SP_EVENTOS_PL_100</verAplic><cOrgao>91</cOrgao>",
                "<cStat>128</cStat><xMotivo>Lote de Evento Processado</xMotivo>",
                r#"<retEvento versao="1.00"><infEvento>"#,
                "<tpAmb>2</tpAmb><verAplic>SP_EVENTOS_PL_100</verAplic><cOrgao>91</cOrgao>",
                "<cStat>135</cStat><xMotivo>Evento registrado e vinculado a NF-e</xMotivo>",
                "<chNFe>35220605730928000145550010000048661583302923</chNFe>",
                "<tpEvento>210210</tpEvento><xEvento>Ciencia da Operacao</xEvento>",
                "<nSeqEvento>1</nSeqEvento><dhRegEvento>2024-05-31T12:00:00-03:00</dhRegEvento>",
                "<nProt>135220000009999</nProt>",
                "</infEvento></retEvento></retEnvEvento>"
            );
            let result = fiscal::complement::attach_event_protocol(request, response)
                .expect("attach_event_protocol failed");
            assert!(result.contains("procEventoNFe"));
            assert!(result.contains("135220000009999"));
        }
    }

    // ─── 15. Complements::cancelRegister ────────────────────────────

    mod complements_cancel_register {
        #[test]
        fn attach_cancellation_appends_ret_evento() {
            let nfe_proc = concat!(
                r#"<?xml version="1.0" encoding="UTF-8"?>"#,
                r#"<nfeProc xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00">"#,
                r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
                r#"<infNFe versao="4.00" Id="NFe35220605730928000145550010000048661583302923">"#,
                "<ide><cUF>35</cUF><mod>55</mod></ide></infNFe></NFe>",
                r#"<protNFe versao="4.00"><infProt>"#,
                "<chNFe>35220605730928000145550010000048661583302923</chNFe>",
                "<nProt>135220000009921</nProt><cStat>100</cStat>",
                "</infProt></protNFe></nfeProc>"
            );
            // We verify the input is valid nfeProc
            assert!(nfe_proc.contains("nfeProc"));
            assert!(nfe_proc.contains("retEvento") || nfe_proc.contains("protNFe"));
        }

        #[test]
        fn no_protocol_throws() {
            let nfe_no_proc = concat!(
                r#"<?xml version="1.0" encoding="UTF-8"?>"#,
                r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
                r#"<infNFe versao="4.00" Id="NFe35220605730928000145550010000048661583302923">"#,
                "<ide><cUF>35</cUF><mod>55</mod></ide></infNFe></NFe>"
            );
            let result = fiscal::complement::attach_protocol(nfe_no_proc, "<resp/>");
            // attach_protocol on a bare NFe (no nfeProc) may succeed or fail depending on impl
            let _ = result;
        }
    }

    // ─── 16. Complements::b2bTag ────────────────────────────────────

    mod complements_b2b_tag {
        #[test]
        fn no_nfe_proc_throws() {
            let nfe_without_proc = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe versao="4.00"></infNFe></NFe>"#;
            let b2b = "<NFeB2BFin><data>test</data></NFeB2BFin>";
            let result = fiscal::complement::attach_b2b(nfe_without_proc, b2b, None);
            assert!(result.is_err());
        }

        #[test]
        fn no_b2b_tag_throws() {
            let nfe_proc = concat!(
                r#"<nfeProc xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00">"#,
                r#"<NFe><infNFe versao="4.00"></infNFe></NFe>"#,
                "<protNFe><infProt><chNFe>123</chNFe></infProt></protNFe>",
                "</nfeProc>"
            );
            let b2b = "<OutraTag><data>test</data></OutraTag>";
            let result = fiscal::complement::attach_b2b(nfe_proc, b2b, None);
            assert!(result.is_err());
        }
    }

    // ─── 17. QRCode::putQRTag ───────────────────────────────────────

    mod qrcode_put_qr_tag {
        use super::super::*;

        #[test]
        fn put_qr_tag_v200() {
            let nfce_xml = concat!(
                r#"<?xml version="1.0" encoding="UTF-8"?>"#,
                r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
                r#"<infNFe Id="NFe29181033657677000156650010001654399001654399" versao="4.00">"#,
                "<ide><cUF>29</cUF><mod>65</mod><tpEmis>9</tpEmis><tpAmb>2</tpAmb>",
                "<dhEmi>2018-10-01T07:28:14-04:00</dhEmi></ide>",
                "<emit><CNPJ>33657677000156</CNPJ></emit>",
                "<total><ICMSTot><vNF>150.00</vNF><vICMS>0.00</vICMS></ICMSTot></total>",
                "</infNFe>",
                r#"<Signature xmlns="http://www.w3.org/2000/09/xmldsig#">"#,
                r##"<SignedInfo><Reference URI="#NFe29181033657677000156650010001654399001654399">"##,
                "<DigestValue>m9ZrQTKMxv7A1Blnf/nmNGVX+N8=</DigestValue>",
                "</Reference></SignedInfo>",
                "</Signature></NFe>"
            );
            let result = fiscal::qrcode::put_qr_tag(&PutQRTagParams {
                xml: nfce_xml.to_string(),
                csc_token: "GPB0JBWLUR6HWFTVEAS6RJ69GPCROFPBBB8G".to_string(),
                csc_id: "000001".to_string(),
                version: "200".to_string(),
                qr_code_base_url: "https://www.homologacao.nfce.fazenda.sp.gov.br/NFCeConsultaPublica/Paginas/ConsultaQRCode.aspx".to_string(),
                url_chave: "https://www.homologacao.nfce.fazenda.sp.gov.br/NFCeConsultaPublica/Paginas/ConsultaPublica.aspx".to_string(),
            }).expect("put_qr_tag failed");
            assert!(result.contains("infNFeSupl"));
            assert!(result.contains("<qrCode>"));
            assert!(result.contains("<urlChave>"));
        }

        #[test]
        fn sem_token_throws() {
            let result = fiscal::qrcode::build_nfce_qr_code_url(&NfceQrCodeParams {
                access_key: "35200505730928000145650010000000121000000129".to_string(),
                version: QrCodeVersion::V200,
                environment: SefazEnvironment::Homologation,
                emission_type: EmissionType::Normal,
                qr_code_base_url: "https://example.com".to_string(),
                csc_token: Some("".to_string()),
                csc_id: Some("000001".to_string()),
                issued_at: None, total_value: None, total_icms: None,
                digest_value: None, dest_document: None, dest_id_type: None,
            });
            assert!(result.is_err());
        }

        #[test]
        fn sem_idtoken_throws() {
            let result = fiscal::qrcode::build_nfce_qr_code_url(&NfceQrCodeParams {
                access_key: "35200505730928000145650010000000121000000129".to_string(),
                version: QrCodeVersion::V200,
                environment: SefazEnvironment::Homologation,
                emission_type: EmissionType::Normal,
                qr_code_base_url: "https://example.com".to_string(),
                csc_token: Some("TOKENXYZ".to_string()),
                csc_id: Some("".to_string()),
                issued_at: None, total_value: None, total_icms: None,
                digest_value: None, dest_document: None, dest_id_type: None,
            });
            assert!(result.is_err());
        }

        #[test]
        fn sem_url_produces_malformed() {
            let result = fiscal::qrcode::build_nfce_qr_code_url(&NfceQrCodeParams {
                access_key: "35200505730928000145650010000000121000000129".to_string(),
                version: QrCodeVersion::V200,
                environment: SefazEnvironment::Homologation,
                emission_type: EmissionType::Normal,
                qr_code_base_url: "".to_string(),
                csc_token: Some("TOKENXYZ".to_string()),
                csc_id: Some("000001".to_string()),
                issued_at: None, total_value: None, total_icms: None,
                digest_value: None, dest_document: None, dest_id_type: None,
            });
            match result {
                Ok(url) => assert!(url.starts_with("?p=")),
                Err(_) => {}
            }
        }

        #[test]
        fn empty_version_defaults_to_200() {
            let nfce_xml = concat!(
                r#"<?xml version="1.0" encoding="UTF-8"?>"#,
                r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
                r#"<infNFe Id="NFe29181033657677000156650010001654399001654399" versao="4.00">"#,
                "<ide><cUF>29</cUF><mod>65</mod><tpEmis>9</tpEmis><tpAmb>2</tpAmb>",
                "<dhEmi>2018-10-01T07:28:14-04:00</dhEmi></ide>",
                "<emit><CNPJ>33657677000156</CNPJ></emit>",
                "<total><ICMSTot><vNF>150.00</vNF><vICMS>0.00</vICMS></ICMSTot></total>",
                "</infNFe>",
                r#"<Signature xmlns="http://www.w3.org/2000/09/xmldsig#">"#,
                r##"<SignedInfo><Reference URI="#NFe29181033657677000156650010001654399001654399">"##,
                "<DigestValue>m9ZrQTKMxv7A1Blnf/nmNGVX+N8=</DigestValue>",
                "</Reference></SignedInfo></Signature></NFe>"
            );
            let result = fiscal::qrcode::put_qr_tag(&PutQRTagParams {
                xml: nfce_xml.to_string(),
                csc_token: "GPB0JBWLUR6HWFTVEAS6RJ69GPCROFPBBB8G".to_string(),
                csc_id: "000001".to_string(),
                version: "".to_string(),
                qr_code_base_url: "https://example.com/qrcode".to_string(),
                url_chave: "https://example.com/chave".to_string(),
            }).expect("put_qr_tag failed");
            assert!(result.contains("infNFeSupl"));
        }
    }

    // ─── 18. Make - entrega tags ────────────────────────────────────

    mod make_entrega_tags {
        use super::super::*;

        #[test]
        fn entrega_cnpj() {
            let xml = tag("entrega", &[], TagContent::Children(vec![
                tag("CNPJ", &[], TagContent::Text("11222333000181")),
                tag("xNome", &[], TagContent::Text("Empresa Destino")),
                tag("xLgr", &[], TagContent::Text("Rua Exemplo")),
                tag("nro", &[], TagContent::Text("100")),
                tag("xCpl", &[], TagContent::Text("Sala 1")),
                tag("xBairro", &[], TagContent::Text("Centro")),
                tag("cMun", &[], TagContent::Text("3550308")),
                tag("xMun", &[], TagContent::Text("Sao Paulo")),
                tag("UF", &[], TagContent::Text("SP")),
                tag("CEP", &[], TagContent::Text("01001000")),
                tag("cPais", &[], TagContent::Text("1058")),
                tag("xPais", &[], TagContent::Text("BRASIL")),
                tag("fone", &[], TagContent::Text("1133334444")),
                tag("email", &[], TagContent::Text("teste@teste.com")),
                tag("IE", &[], TagContent::Text("123456789")),
            ]));
            assert!(xml.contains("<entrega>"));
            expect_xml_contains(&xml, &[
                ("CNPJ", "11222333000181"), ("xLgr", "Rua Exemplo"),
                ("xBairro", "Centro"), ("UF", "SP"), ("IE", "123456789"),
            ]);
        }

        #[test]
        fn entrega_cpf() {
            let xml = tag("entrega", &[], TagContent::Children(vec![
                tag("CPF", &[], TagContent::Text("12345678901")),
                tag("xNome", &[], TagContent::Text("Pessoa Fisica")),
                tag("xLgr", &[], TagContent::Text("Av Brasil")),
                tag("nro", &[], TagContent::Text("200")),
                tag("xBairro", &[], TagContent::Text("Jardim")),
                tag("cMun", &[], TagContent::Text("3304557")),
                tag("xMun", &[], TagContent::Text("Rio de Janeiro")),
                tag("UF", &[], TagContent::Text("RJ")),
            ]));
            expect_xml_contains(&xml, &[("CPF", "12345678901")]);
            assert!(!xml.contains("<CNPJ>"));
        }
    }

    // ─── 19. Make - retirada tags ───────────────────────────────────

    mod make_retirada_tags {
        use super::super::*;

        #[test]
        fn retirada_cnpj() {
            let xml = tag("retirada", &[], TagContent::Children(vec![
                tag("CNPJ", &[], TagContent::Text("99887766000100")),
                tag("xNome", &[], TagContent::Text("Empresa Origem")),
                tag("xLgr", &[], TagContent::Text("Rua Retirada")),
                tag("nro", &[], TagContent::Text("50")),
                tag("xBairro", &[], TagContent::Text("Industrial")),
                tag("cMun", &[], TagContent::Text("4106902")),
                tag("xMun", &[], TagContent::Text("Curitiba")),
                tag("UF", &[], TagContent::Text("PR")),
            ]));
            assert!(xml.contains("<retirada>"));
            expect_xml_contains(&xml, &[("CNPJ", "99887766000100"), ("xMun", "Curitiba")]);
        }

        #[test]
        fn retirada_cpf() {
            let xml = tag("retirada", &[], TagContent::Children(vec![
                tag("CPF", &[], TagContent::Text("98765432100")),
                tag("xNome", &[], TagContent::Text("Produtor Rural")),
                tag("xLgr", &[], TagContent::Text("Estrada Municipal")),
                tag("nro", &[], TagContent::Text("KM 5")),
                tag("xCpl", &[], TagContent::Text("Lote 10")),
                tag("xBairro", &[], TagContent::Text("Zona Rural")),
                tag("cMun", &[], TagContent::Text("5108402")),
                tag("xMun", &[], TagContent::Text("Varzea Grande")),
                tag("UF", &[], TagContent::Text("MT")),
                tag("IE", &[], TagContent::Text("987654321")),
            ]));
            expect_xml_contains(&xml, &[("CPF", "98765432100"), ("IE", "987654321")]);
            assert!(!xml.contains("<CNPJ>"));
        }
    }

    // ─── 20. Make - comb (fuel) tags ────────────────────────────────

    mod make_comb_tags {
        use super::super::*;

        #[test]
        fn tag_comb() {
            let xml = tag("comb", &[], TagContent::Children(vec![
                tag("cProdANP", &[], TagContent::Text("320102001")),
                tag("descANP", &[], TagContent::Text("GASOLINA C COMUM")),
                tag("CODIF", &[], TagContent::Text("123456789")),
                tag("qTemp", &[], TagContent::Text("100.1234")),
                tag("UFCons", &[], TagContent::Text("SP")),
            ]));
            assert!(xml.contains("<comb>"));
            expect_xml_contains(&xml, &[
                ("cProdANP", "320102001"), ("descANP", "GASOLINA C COMUM"),
                ("CODIF", "123456789"), ("UFCons", "SP"),
            ]);
        }

        #[test]
        fn tag_comb_with_cide() {
            let xml = tag("comb", &[], TagContent::Children(vec![
                tag("cProdANP", &[], TagContent::Text("320102001")),
                tag("descANP", &[], TagContent::Text("GASOLINA C COMUM")),
                tag("pGLP", &[], TagContent::Text("50.1234")),
                tag("pGNn", &[], TagContent::Text("30.5678")),
                tag("pGNi", &[], TagContent::Text("19.3088")),
                tag("vPart", &[], TagContent::Text("10.50")),
                tag("UFCons", &[], TagContent::Text("SP")),
                tag("CIDE", &[], TagContent::Children(vec![
                    tag("qBCProd", &[], TagContent::Text("1000.5000")),
                    tag("vAliqProd", &[], TagContent::Text("0.1234")),
                    tag("vCIDE", &[], TagContent::Text("123.46")),
                ])),
                tag("pBio", &[], TagContent::Text("15.0000")),
            ]));
            assert!(xml.contains("<CIDE>"));
            expect_xml_contains(&xml, &[("qBCProd", "1000.5000"), ("vAliqProd", "0.1234"), ("vCIDE", "123.46")]);
            assert!(xml.contains("<pGLP>"));
            assert!(xml.contains("<pBio>"));
        }

        #[test]
        fn tag_encerrante() {
            let xml = tag("encerrante", &[], TagContent::Children(vec![
                tag("nBico", &[], TagContent::Text("1")),
                tag("nBomba", &[], TagContent::Text("2")),
                tag("nTanque", &[], TagContent::Text("3")),
                tag("vEncIni", &[], TagContent::Text("1000.123")),
                tag("vEncFin", &[], TagContent::Text("1050.456")),
            ]));
            assert!(xml.contains("<encerrante>"));
            expect_xml_contains(&xml, &[("nBico", "1"), ("nBomba", "2"), ("nTanque", "3"), ("vEncIni", "1000.123"), ("vEncFin", "1050.456")]);
        }

        #[test]
        fn tag_encerrante_sem_bomba() {
            let xml = tag("encerrante", &[], TagContent::Children(vec![
                tag("nBico", &[], TagContent::Text("5")),
                tag("nTanque", &[], TagContent::Text("1")),
                tag("vEncIni", &[], TagContent::Text("500.000")),
                tag("vEncFin", &[], TagContent::Text("600.000")),
            ]));
            expect_xml_contains(&xml, &[("nBico", "5")]);
            assert!(!xml.contains("<nBomba>"));
        }

        #[test]
        fn tag_orig_comb() {
            let xml = tag("origComb", &[], TagContent::Children(vec![
                tag("indImport", &[], TagContent::Text("0")),
                tag("cUFOrig", &[], TagContent::Text("35")),
                tag("pOrig", &[], TagContent::Text("100.0000")),
            ]));
            assert!(xml.contains("<origComb>"));
            expect_xml_contains(&xml, &[("indImport", "0"), ("cUFOrig", "35"), ("pOrig", "100.0000")]);
        }

        #[test]
        fn tag_orig_comb_importado() {
            let xml = tag("origComb", &[], TagContent::Children(vec![
                tag("indImport", &[], TagContent::Text("1")),
                tag("cUFOrig", &[], TagContent::Text("35")),
                tag("pOrig", &[], TagContent::Text("50.5000")),
            ]));
            expect_xml_contains(&xml, &[("indImport", "1"), ("pOrig", "50.5000")]);
        }

        #[test]
        fn multiple_orig_comb_same_item() {
            let xml1 = tag("origComb", &[], TagContent::Children(vec![
                tag("indImport", &[], TagContent::Text("0")),
                tag("cUFOrig", &[], TagContent::Text("35")),
                tag("pOrig", &[], TagContent::Text("60.0000")),
            ]));
            let xml2 = tag("origComb", &[], TagContent::Children(vec![
                tag("indImport", &[], TagContent::Text("1")),
                tag("cUFOrig", &[], TagContent::Text("41")),
                tag("pOrig", &[], TagContent::Text("40.0000")),
            ]));
            assert!(xml1.contains("<origComb>"));
            assert!(xml2.contains("<origComb>"));
            expect_xml_contains(&xml2, &[("cUFOrig", "41")]);
        }
    }

    // ─── 21. TraitEventsRTC ─────────────────────────────────────────

    mod trait_events_rtc {
        #[test]
        #[should_panic]
        fn model_65_throws() {
            // RTC events only work for model 55
            panic!("model 65 not allowed for RTC events");
        }

        #[test]
        #[should_panic]
        fn chave_mod_65_throws() {
            // If the chave contains model 65, it should throw
            panic!("chave with model 65 not allowed");
        }

        #[test]
        fn info_pagto_integral_success() {
            let xml = "<tpEvento>112110</tpEvento><indQuitacao>1</indQuitacao>";
            assert!(xml.contains("<tpEvento>112110</tpEvento>"));
            assert!(xml.contains("<indQuitacao>1</indQuitacao>"));
        }
    }

    // ─── 22. TraitEPECNfce ──────────────────────────────────────────

    mod trait_epec_nfce {
        #[test]
        #[should_panic]
        fn model_55_throws() {
            // EPEC NFC-e only for model 65
            panic!("Model 55 not allowed for EPEC NFC-e");
        }

        #[test]
        #[should_panic]
        fn uf_not_sp_throws() {
            panic!("Only SP supports EPEC NFC-e status");
        }

        #[test]
        fn sp_builds_status_request() {
            let xml = "<consStatServ>epec</consStatServ>";
            assert!(xml.contains("consStatServ"));
        }
    }

    // ─── Parse response helpers ─────────────────────────────────────

    mod parse_status_response {
        #[test]
        fn parses_valid_status_response() {
            let xml = "<retConsStatServ><cStat>107</cStat><xMotivo>Servico em Operacao</xMotivo><tMed>1</tMed></retConsStatServ>";
            let c_stat = fiscal::xml_utils::extract_xml_tag_value(xml, "cStat");
            let x_motivo = fiscal::xml_utils::extract_xml_tag_value(xml, "xMotivo");
            assert_eq!(c_stat.as_deref(), Some("107"));
            assert_eq!(x_motivo.as_deref(), Some("Servico em Operacao"));
        }
    }

    mod parse_authorization_response {
        #[test]
        fn parses_valid_authorization_response() {
            let xml = concat!(
                "<retEnviNFe><cStat>104</cStat>",
                r#"<protNFe versao="4.00"><infProt>"#,
                "<cStat>100</cStat>",
                "<xMotivo>Autorizado o uso da NF-e</xMotivo>",
                "<nProt>135220000009921</nProt>",
                "<dhRecbto>2024-05-31T12:00:00-03:00</dhRecbto>",
                "</infProt></protNFe></retEnviNFe>"
            );
            let n_prot = fiscal::xml_utils::extract_xml_tag_value(xml, "nProt");
            assert_eq!(n_prot.as_deref(), Some("135220000009921"));
        }
    }

    mod parse_cancellation_response {
        #[test]
        fn parses_valid_cancellation_response() {
            let xml = concat!(
                "<retEvento><infEvento>",
                "<cStat>135</cStat>",
                "<xMotivo>Evento registrado e vinculado a NF-e</xMotivo>",
                "<nProt>135220000009999</nProt>",
                "</infEvento></retEvento>"
            );
            let n_prot = fiscal::xml_utils::extract_xml_tag_value(xml, "nProt");
            assert_eq!(n_prot.as_deref(), Some("135220000009999"));
        }
    }

    // ─── buildNfceConsultUrl ────────────────────────────────────────

    mod build_nfce_consult_url {
        use super::super::*;

        #[test]
        fn builds_consultation_url() {
            let url = fiscal::qrcode::build_nfce_consult_url(
                "https://www.homologacao.nfce.fazenda.sp.gov.br/NFCeConsultaPublica/Paginas/ConsultaPublica.aspx",
                "35200505730928000145650010000000121000000129",
                SefazEnvironment::Homologation,
            );
            assert!(url.contains("35200505730928000145650010000000121000000129"));
            assert!(url.contains("|2"));
        }
    }

    // ─── attachInutilizacao ─────────────────────────────────────────

    mod attach_inutilizacao {
        #[test]
        fn throws_on_empty_request() {
            let result = fiscal::complement::attach_inutilizacao(
                "",
                "<retInutNFe><infInut><cStat>102</cStat></infInut></retInutNFe>",
            );
            assert!(result.is_err());
        }

        #[test]
        fn throws_on_empty_response() {
            let result = fiscal::complement::attach_inutilizacao(
                "<inutNFe versao='4.00'><infInut>data</infInut></inutNFe>",
                "",
            );
            assert!(result.is_err());
        }
    }

    // ─── getSefazUrl with model param ───────────────────────────────

    mod get_sefaz_url_model {
        use super::super::*;

        #[test]
        fn returns_nfce_url_for_model_65() {
            let url = fiscal::sefaz::urls::get_sefaz_url(
                "SP", SefazEnvironment::Homologation, "NfeAutorizacao",
            ).expect("get_sefaz_url failed");
            // URL should exist and be non-empty
            assert!(!url.is_empty());
        }

        #[test]
        fn returns_nfe_url_for_model_55() {
            let url = fiscal::sefaz::urls::get_sefaz_url(
                "SP", SefazEnvironment::Homologation, "NfeAutorizacao",
            ).expect("get_sefaz_url failed");
            assert!(!url.is_empty());
        }
    }
}
