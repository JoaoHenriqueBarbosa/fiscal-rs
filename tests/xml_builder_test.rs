use fiscal::xml_utils::{tag, TagContent};
use fiscal::xml_builder::{build_access_key, build_invoice_xml};
use fiscal::types::*;
use chrono::FixedOffset;
use fiscal::newtypes::{Cents, Rate, IbgeCode};

// ── tag() ────────────────────────────────────────────────────────────────────

mod tag_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn builds_self_closing_tag_with_no_children() {
        assert_eq!(
            tag("xNome", &[], TagContent::Text("Test")),
            "<xNome>Test</xNome>"
        );
    }

    #[test]
    fn builds_tag_with_attributes_and_child_array() {
        let result = tag(
            "det",
            &[("nItem", "1")],
            TagContent::Children(vec![tag("prod", &[], TagContent::None)]),
        );
        assert_eq!(result, r#"<det nItem="1"><prod></prod></det>"#);
    }

    #[test]
    fn builds_empty_tag_no_children() {
        assert_eq!(tag("empty", &[], TagContent::None), "<empty></empty>");
    }

    #[test]
    fn escapes_special_xml_characters_in_text() {
        assert_eq!(
            tag("name", &[], TagContent::Text("A & B")),
            "<name>A &amp; B</name>"
        );
        assert_eq!(
            tag("name", &[], TagContent::Text(r#"<"test">"#)),
            "<name>&lt;&quot;test&quot;&gt;</name>"
        );
    }

    #[test]
    fn does_not_escape_children_array_raw_xml() {
        let result = tag(
            "parent",
            &[],
            TagContent::Children(vec![tag("child", &[], TagContent::Text("value"))]),
        );
        assert_eq!(result, "<parent><child>value</child></parent>");
    }
}

// ── buildAccessKey() ─────────────────────────────────────────────────────────

mod build_access_key_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn base_params() -> AccessKeyParams {
        AccessKeyParams {
            state_code: IbgeCode("35".to_string()),
            year_month: "2601".to_string(),
            tax_id: "12345678000199".to_string(),
            model: InvoiceModel::Nfce,
            series: 1,
            number: 1,
            emission_type: EmissionType::Normal,
            numeric_code: "12345678".to_string(),
        }
    }

    #[test]
    fn generates_a_44_digit_access_key() {
        let key = build_access_key(&base_params()).unwrap();
        assert_eq!(key.len(), 44);
        assert!(key.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn pads_fields_correctly() {
        let params = AccessKeyParams {
            number: 42,
            numeric_code: "00000001".to_string(),
            ..base_params()
        };
        let key = build_access_key(&params).unwrap();

        // cUF=35
        assert_eq!(&key[0..2], "35");
        // AAMM=2601
        assert_eq!(&key[2..6], "2601");
        // CNPJ=12345678000199
        assert_eq!(&key[6..20], "12345678000199");
        // mod=65
        assert_eq!(&key[20..22], "65");
        // serie=001
        assert_eq!(&key[22..25], "001");
        // nNF=000000042
        assert_eq!(&key[25..34], "000000042");
        // tpEmis=1
        assert_eq!(&key[34..35], "1");
        // cNF=00000001
        assert_eq!(&key[35..43], "00000001");
        // Last digit is check digit (mod 11)
        assert!(key.chars().nth(43).unwrap().is_ascii_digit());
    }

    #[test]
    fn produces_deterministic_check_digit() {
        let params = base_params();
        let key1 = build_access_key(&params).unwrap();
        let key2 = build_access_key(&params).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn different_inputs_produce_different_keys() {
        let key1 = build_access_key(&AccessKeyParams {
            number: 1,
            ..base_params()
        }).unwrap();
        let key2 = build_access_key(&AccessKeyParams {
            number: 2,
            ..base_params()
        }).unwrap();
        assert_ne!(key1, key2);
    }
}

// ── buildInvoiceXml() ────────────────────────────────────────────────────────

mod build_invoice_xml_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn sample_data() -> InvoiceBuildData {
        let offset = FixedOffset::west_opt(3 * 3600).unwrap();
        let issued_at = chrono::NaiveDate::from_ymd_opt(2026, 1, 15)
            .unwrap()
            .and_hms_opt(10, 30, 0)
            .unwrap()
            .and_local_timezone(offset)
            .unwrap();

        InvoiceBuildData {
            model: InvoiceModel::Nfce,
            series: 1,
            number: 1,
            emission_type: EmissionType::Normal,
            environment: SefazEnvironment::Homologation,
            issued_at,
            operation_nature: "VENDA".to_string(),
            issuer: IssuerData {
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
            },
            recipient: None,
            items: vec![InvoiceItemData {
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
            }],
            payments: vec![PaymentData {
                method: "01".to_string(),
                amount: Cents(2000),
            }],
            change_amount: None,
            payment_card_details: None,
            contingency: None,
            operation_type: None,
            purpose_code: None,
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

    #[test]
    fn generates_valid_xml_with_correct_structure() {
        let result = build_invoice_xml(&sample_data()).unwrap();
        assert!(result.xml.contains(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(result.xml.contains("<NFe"));
        assert!(result.xml.contains("<infNFe"));
        assert!(result.xml.contains("</NFe>"));
        assert_eq!(result.access_key.len(), 44);
    }

    #[test]
    fn contains_required_groups() {
        let result = build_invoice_xml(&sample_data()).unwrap();
        let xml = &result.xml;
        assert!(xml.contains("<ide>"));
        assert!(xml.contains("<emit>"));
        assert!(xml.contains("<det "));
        assert!(xml.contains("<total>"));
        assert!(xml.contains("<transp>"));
        assert!(xml.contains("<pag>"));
    }

    #[test]
    fn sets_model_65_for_nfce() {
        let result = build_invoice_xml(&sample_data()).unwrap();
        assert!(result.xml.contains("<mod>65</mod>"));
    }

    #[test]
    fn sets_model_55_for_nfe() {
        let mut data = sample_data();
        data.model = InvoiceModel::Nfe;
        let result = build_invoice_xml(&data).unwrap();
        assert!(result.xml.contains("<mod>55</mod>"));
    }

    #[test]
    fn includes_issuer_data() {
        let result = build_invoice_xml(&sample_data()).unwrap();
        let xml = &result.xml;
        assert!(xml.contains("<CNPJ>12345678000199</CNPJ>"));
        assert!(xml.contains("<xNome>Test Company</xNome>"));
        assert!(xml.contains("<IE>123456789</IE>"));
        assert!(xml.contains("<CRT>1</CRT>"));
    }

    #[test]
    fn includes_item_data() {
        let result = build_invoice_xml(&sample_data()).unwrap();
        let xml = &result.xml;
        assert!(xml.contains(r#"<det nItem="1">"#));
        assert!(xml.contains("<xProd>Product A</xProd>"));
        assert!(xml.contains("<NCM>84715010</NCM>"));
        assert!(xml.contains("<CFOP>5102</CFOP>"));
    }

    #[test]
    fn formats_amounts_correctly() {
        let result = build_invoice_xml(&sample_data()).unwrap();
        let xml = &result.xml;
        assert!(xml.contains("<vProd>20.00</vProd>"));
        assert!(xml.contains("<vNF>20.00</vNF>"));
    }

    #[test]
    fn includes_payment_data() {
        let result = build_invoice_xml(&sample_data()).unwrap();
        let xml = &result.xml;
        assert!(xml.contains("<tPag>01</tPag>"));
        assert!(xml.contains("<vPag>20.00</vPag>"));
    }

    #[test]
    fn includes_homologation_note_when_environment_2() {
        let result = build_invoice_xml(&sample_data()).unwrap();
        assert!(result.xml.contains("HOMOLOGACAO"));
    }

    #[test]
    fn includes_recipient_when_provided() {
        let mut data = sample_data();
        data.recipient = Some(RecipientData {
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
        });
        let result = build_invoice_xml(&data).unwrap();
        let xml = &result.xml;
        assert!(xml.contains("<dest>"));
        assert!(xml.contains("<CPF>12345678901</CPF>"));
        assert!(xml.contains("<xNome>John Doe</xNome>"));
    }

    #[test]
    fn omits_recipient_for_nfce_without_recipient() {
        let result = build_invoice_xml(&sample_data()).unwrap();
        assert!(!result.xml.contains("<dest>"));
    }

    #[test]
    fn includes_contingency_info_when_provided() {
        let offset = FixedOffset::west_opt(3 * 3600).unwrap();
        let now = chrono::Utc::now().with_timezone(&offset);

        let mut data = sample_data();
        data.contingency = Some(ContingencyData {
            contingency_type: ContingencyType::Offline,
            reason: "SEFAZ unavailable".to_string(),
            at: now,
        });
        let result = build_invoice_xml(&data).unwrap();
        let xml = &result.xml;
        assert!(xml.contains("contingencia"));
        assert!(xml.contains("SEFAZ unavailable"));
    }
}
