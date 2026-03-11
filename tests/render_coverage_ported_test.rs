// Ported from TypeScript render-coverage-ported.test.ts (31 tests)
//
// Each TypeScript describe()/it() block becomes a Rust mod/test.
// All tests compile but will fail at runtime (implementations use todo!()).

mod common;

use fiscal::newtypes::{Cents, Rate, Rate4, IbgeCode};
use chrono::FixedOffset;
use fiscal::types::*;
use fiscal::xml_builder::build_invoice_xml;
use fiscal::xml_utils::{tag, TagContent};

use common::expect_xml_tag_values as expect_xml_contains;

/// Build a DateTime<FixedOffset> for 2025-01-15T10:30:00-03:00
fn make_issued_at() -> chrono::DateTime<FixedOffset> {
    let offset = FixedOffset::west_opt(3 * 3600).unwrap();
    chrono::NaiveDate::from_ymd_opt(2025, 1, 15)
        .unwrap()
        .and_hms_opt(10, 30, 0)
        .unwrap()
        .and_local_timezone(offset)
        .unwrap()
}

/// Shared issuer for model 55 tests (tax_regime = Normal)
fn make_issuer_normal() -> IssuerData {
    IssuerData {
        tax_id: "58716523000119".into(),
        state_tax_id: "111222333444".into(),
        company_name: "Empresa Teste".into(),
        trade_name: None,
        tax_regime: TaxRegime::Normal,
        state_code: "SP".into(),
        city_code: IbgeCode("3550308".into()),
        city_name: "Sao Paulo".into(),
        street: "Rua Teste".into(),
        street_number: "100".into(),
        district: "Centro".into(),
        zip_code: "01001000".into(),
        address_complement: None,
    }
}

/// Shared issuer for NFC-e with trade_name and Simples Nacional
fn make_issuer_simples() -> IssuerData {
    IssuerData {
        tax_id: "58716523000119".into(),
        state_tax_id: "111222333444".into(),
        company_name: "Empresa Teste".into(),
        trade_name: None,
        tax_regime: TaxRegime::SimplesNacional,
        state_code: "SP".into(),
        city_code: IbgeCode("3550308".into()),
        city_name: "Sao Paulo".into(),
        street: "Rua Teste".into(),
        street_number: "100".into(),
        district: "Centro".into(),
        zip_code: "01001000".into(),
        address_complement: None,
    }
}

/// Base item for ICMS CST 00 with PIS/COFINS CST 01
fn make_base_item() -> InvoiceItemData {
    InvoiceItemData {
        item_number: 1,
        product_code: "001".into(),
        description: "Produto Teste".into(),
        ncm: "61091000".into(),
        cfop: "5102".into(),
        unit_of_measure: "UN".into(),
        quantity: 1.0,
        unit_price: Cents(10000),
        total_price: Cents(10000),
        c_ean: None,
        c_ean_trib: None,
        cest: None,
        v_frete: None,
        v_seg: None,
        v_desc: None,
        v_outro: None,
        orig: None,
        icms_cst: "00".into(),
        icms_rate: Rate(1800),
        icms_amount: Cents(1800),
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
        pis_cst: "01".into(),
        pis_v_bc: None,
        pis_p_pis: None,
        pis_v_pis: None,
        pis_q_bc_prod: None,
        pis_v_aliq_prod: None,
        cofins_cst: "01".into(),
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

/// Base invoice data factory for model 55
fn make_base_invoice_data() -> InvoiceBuildData {
    InvoiceBuildData {
        model: InvoiceModel::Nfe,
        series: 1,
        number: 1,
        emission_type: EmissionType::Normal,
        environment: SefazEnvironment::Homologation,
        issued_at: make_issued_at(),
        operation_nature: "VENDA".into(),
        issuer: make_issuer_normal(),
        recipient: None,
        items: vec![make_base_item()],
        payments: vec![PaymentData {
            method: "01".into(),
            amount: Cents(10000),
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

// ═══════════════════════════════════════════════════════════════════════════
//  render() with complete NF-e (model 55)
// ═══════════════════════════════════════════════════════════════════════════

mod render_complete_nfe_55 {
    use super::*;

    #[test]
    fn test_render_complete_nfe55_has_all_sections() {
        let mut data = make_base_invoice_data();

        // Add issuer with trade name
        data.issuer.trade_name = Some("Teste".into());

        // Add recipient
        data.recipient = Some(RecipientData {
            tax_id: "12345678901".into(),
            name: "Cliente Teste".into(),
            state_code: Some("SP".into()),
            ..Default::default()
        });

        // Item with full PIS/COFINS
        data.items = vec![{
            let mut item = make_base_item();
            item.pis_v_bc = Some(Cents(10000));
            item.pis_p_pis = Some(Rate4(16500));
            item.pis_v_pis = Some(Cents(165));
            item.cofins_v_bc = Some(Cents(10000));
            item.cofins_p_cofins = Some(Rate4(76000));
            item.cofins_v_cofins = Some(Cents(760));
            item
        }];

        data.payments = vec![PaymentData {
            method: "01".into(),
            amount: Cents(10000),
        }];

        data.withdrawal = Some(LocationData {
            tax_id: "99887766000100".into(),
            name: Some("Empresa Origem".into()),
            street: "Rua Retirada".into(),
            number: "50".into(),
            complement: None,
            district: "Industrial".into(),
            city_code: IbgeCode("4106902".into()),
            city_name: "Curitiba".into(),
            state_code: "PR".into(),
            zip_code: None,
        });

        data.delivery = Some(LocationData {
            tax_id: "11222333000181".into(),
            name: Some("Empresa Destino".into()),
            street: "Rua Entrega".into(),
            number: "200".into(),
            complement: None,
            district: "Centro".into(),
            city_code: IbgeCode("3550308".into()),
            city_name: "Sao Paulo".into(),
            state_code: "SP".into(),
            zip_code: None,
        });

        data.authorized_xml = Some(vec![AuthorizedXml {
            tax_id: "12345678000195".into(),
        }]);

        data.billing = Some(BillingData {
            invoice: Some(BillingInvoice {
                number: "001".into(),
                original_value: Cents(10000),
                discount_value: None,
                net_value: Cents(10000),
            }),
            installments: Some(vec![Installment {
                number: "001".into(),
                due_date: "2025-02-15".into(),
                value: Cents(10000),
            }]),
        });

        data.transport = Some(TransportData {
            freight_mode: "0".into(),
            carrier: Some(CarrierData {
                tax_id: Some("12345678000195".into()),
                name: Some("Transportadora".into()),
                ..Default::default()
            }),
            ..Default::default()
        });

        data.intermediary = Some(IntermediaryData {
            tax_id: "55667788000199".into(),
            id_cad_int_tran: None,
        });

        data.additional_info = Some(AdditionalInfo {
            taxpayer_note: Some("Nota teste".into()),
            ..Default::default()
        });

        data.export = Some(ExportData {
            exit_state: "SP".into(),
            export_location: "Porto de Santos".into(),
            dispatch_location: None,
        });

        data.purchase = Some(PurchaseData {
            order_number: Some("PED-001".into()),
            ..Default::default()
        });

        data.tech_responsible = Some(TechResponsibleData {
            tax_id: "11223344000155".into(),
            contact: "Suporte".into(),
            email: "suporte@teste.com".into(),
            phone: None,
        });

        data.references = Some(vec![ReferenceDoc::Nfe {
            access_key: "35170358716523000119550010000000291000000291".into(),
        }]);

        let result = build_invoice_xml(&data).unwrap();
        let xml = &result.xml;

        // Check all sections present in correct order
        assert!(xml.contains("<ide>"));
        assert!(xml.contains("<emit>"));
        assert!(xml.contains("<dest>"));
        assert!(xml.contains("<retirada>"));
        assert!(xml.contains("<entrega>"));
        assert!(xml.contains("<autXML>"));
        assert!(xml.contains("<det "));
        assert!(xml.contains("<total>"));
        assert!(xml.contains("<transp>"));
        assert!(xml.contains("<cobr>"));
        assert!(xml.contains("<pag>"));
        assert!(xml.contains("<infIntermed>"));
        assert!(xml.contains("<infAdic>"));
        assert!(xml.contains("<exporta>"));
        assert!(xml.contains("<compra>"));
        assert!(xml.contains("<infRespTec>"));

        // Verify order: retirada before entrega before autXML before det
        let retirada_pos = xml.find("<retirada>").unwrap();
        let entrega_pos = xml.find("<entrega>").unwrap();
        let aut_xml_pos = xml.find("<autXML>").unwrap();
        let det_pos = xml.find("<det ").unwrap();
        let total_pos = xml.find("<total>").unwrap();
        let transp_pos = xml.find("<transp>").unwrap();
        let cobr_pos = xml.find("<cobr>").unwrap();
        let pag_pos = xml.find("<pag>").unwrap();
        assert!(retirada_pos < entrega_pos);
        assert!(entrega_pos < aut_xml_pos);
        assert!(aut_xml_pos < det_pos);
        assert!(det_pos < total_pos);
        assert!(total_pos < transp_pos);
        assert!(transp_pos < cobr_pos);
        assert!(cobr_pos < pag_pos);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  render() with NFC-e (model 65)
// ═══════════════════════════════════════════════════════════════════════════

mod render_nfce_model_65 {
    use super::*;

    #[test]
    fn test_render_nfce_model_65_produces_xml_with_mod_65_ind_final_1_tp_imp_4() {
        let mut data = make_base_invoice_data();
        data.model = InvoiceModel::Nfce;
        data.issuer = make_issuer_simples();
        data.consumer_type = Some("1".into());
        data.buyer_presence = Some("1".into());
        data.print_format = Some("4".into());

        // NFC-e item with Simples Nacional
        let mut item = make_base_item();
        item.description = "Produto NFC-e".into();
        item.unit_price = Cents(5000);
        item.total_price = Cents(5000);
        item.icms_cst = "102".into();
        item.icms_rate = Rate(0);
        item.icms_amount = Cents(0);
        item.icms_mod_bc = None;
        item.pis_cst = "07".into();
        item.cofins_cst = "07".into();
        data.items = vec![item];
        data.payments = vec![PaymentData {
            method: "01".into(),
            amount: Cents(5000),
        }];

        let result = build_invoice_xml(&data).unwrap();
        let xml = &result.xml;

        assert!(xml.contains("<mod>65</mod>"));
        assert!(xml.contains("<indFinal>1</indFinal>"));
        assert!(xml.contains("<tpImp>4</tpImp>"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  TraitTagTotal: tagICMSTot
// ═══════════════════════════════════════════════════════════════════════════

mod trait_tag_total_icms_tot {
    use super::*;

    #[test]
    fn test_tag_icms_tot_with_accumulated_values_builds_icms_tot_with_all_monetary_fields() {
        let xml = tag(
            "ICMSTot",
            &[],
            TagContent::Children(vec![
                tag("vBC", &[], "1000.00".into()),
                tag("vICMS", &[], "180.00".into()),
                tag("vICMSDeson", &[], "10.00".into()),
                tag("vBCST", &[], "200.00".into()),
                tag("vST", &[], "36.00".into()),
                tag("vProd", &[], "1000.00".into()),
                tag("vFrete", &[], "50.00".into()),
                tag("vSeg", &[], "25.00".into()),
                tag("vDesc", &[], "15.00".into()),
                tag("vII", &[], "30.00".into()),
                tag("vIPI", &[], "45.00".into()),
                tag("vPIS", &[], "16.50".into()),
                tag("vCOFINS", &[], "76.00".into()),
                tag("vOutro", &[], "5.00".into()),
                tag("vNF", &[], "1196.50".into()),
                tag("vTotTrib", &[], "383.50".into()),
                tag("vFCP", &[], "20.00".into()),
                tag("vFCPST", &[], "4.00".into()),
                tag("vFCPSTRet", &[], "2.00".into()),
            ]),
        );

        expect_xml_contains(&xml, &[
            ("vBC", "1000.00"),
            ("vICMS", "180.00"),
            ("vICMSDeson", "10.00"),
            ("vBCST", "200.00"),
            ("vST", "36.00"),
            ("vFrete", "50.00"),
            ("vSeg", "25.00"),
            ("vDesc", "15.00"),
            ("vII", "30.00"),
            ("vIPI", "45.00"),
            ("vPIS", "16.50"),
            ("vCOFINS", "76.00"),
            ("vOutro", "5.00"),
            ("vTotTrib", "383.50"),
            ("vFCP", "20.00"),
            ("vFCPST", "4.00"),
            ("vFCPSTRet", "2.00"),
        ]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  TraitTagTotal: tagISSQNTot
// ═══════════════════════════════════════════════════════════════════════════

mod trait_tag_total_issqn_tot {
    use super::*;

    #[test]
    fn test_tag_issqn_tot_builds_issqn_tot_with_all_fields() {
        let xml = tag(
            "ISSQNtot",
            &[],
            TagContent::Children(vec![
                tag("vServ", &[], "500.00".into()),
                tag("vBC", &[], "500.00".into()),
                tag("vISS", &[], "25.00".into()),
                tag("vPIS", &[], "8.25".into()),
                tag("vCOFINS", &[], "38.00".into()),
                tag("dCompet", &[], "2017-03-03".into()),
                tag("vDeducao", &[], "10.00".into()),
                tag("vOutro", &[], "5.00".into()),
                tag("vDescIncond", &[], "3.00".into()),
                tag("vDescCond", &[], "2.00".into()),
                tag("vISSRet", &[], "12.50".into()),
                tag("cRegTrib", &[], "5".into()),
            ]),
        );

        expect_xml_contains(&xml, &[
            ("vServ", "500.00"),
            ("vISS", "25.00"),
            ("dCompet", "2017-03-03"),
            ("vDeducao", "10.00"),
            ("vISSRet", "12.50"),
            ("cRegTrib", "5"),
        ]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  TraitTagTotal: tagretTrib
// ═══════════════════════════════════════════════════════════════════════════

mod trait_tag_total_ret_trib {
    use super::*;

    #[test]
    fn test_tag_ret_trib_builds_ret_trib_with_all_retention_fields() {
        let xml = tag(
            "retTrib",
            &[],
            TagContent::Children(vec![
                tag("vRetPIS", &[], "10.00".into()),
                tag("vRetCOFINS", &[], "46.00".into()),
                tag("vRetCSLL", &[], "5.00".into()),
                tag("vBCIRRF", &[], "100.00".into()),
                tag("vIRRF", &[], "15.00".into()),
                tag("vBCRetPrev", &[], "200.00".into()),
                tag("vRetPrev", &[], "22.00".into()),
            ]),
        );

        expect_xml_contains(&xml, &[
            ("vRetPIS", "10.00"),
            ("vRetCOFINS", "46.00"),
            ("vRetCSLL", "5.00"),
            ("vBCIRRF", "100.00"),
            ("vIRRF", "15.00"),
            ("vBCRetPrev", "200.00"),
            ("vRetPrev", "22.00"),
        ]);
    }

    #[test]
    fn test_tag_ret_trib_should_be_inside_total_section_in_rendered_xml() {
        let mut data = make_base_invoice_data();

        data.ret_trib = Some(RetTribData {
            v_ret_pis: Some(Cents(1000)),
            v_ret_cofins: Some(Cents(4600)),
            v_ret_csll: Some(Cents(500)),
            v_bc_irrf: Some(Cents(10000)),
            v_irrf: Some(Cents(1500)),
            v_bc_ret_prev: Some(Cents(20000)),
            v_ret_prev: Some(Cents(2200)),
        });

        let result = build_invoice_xml(&data).unwrap();
        let xml = &result.xml;

        // retTrib must be inside <total>
        assert!(xml.contains("<retTrib>"));
        let total_pos = xml.find("<total>").unwrap();
        let ret_trib_pos = xml.find("<retTrib>").unwrap();
        let total_end_pos = xml.find("</total>").unwrap();
        assert!(ret_trib_pos > total_pos);
        assert!(ret_trib_pos < total_end_pos);

        expect_xml_contains(xml, &[
            ("vRetPIS", "10.00"),
            ("vRetCOFINS", "46.00"),
            ("vRetCSLL", "5.00"),
            ("vBCIRRF", "100.00"),
            ("vIRRF", "15.00"),
            ("vBCRetPrev", "200.00"),
            ("vRetPrev", "22.00"),
        ]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  TraitTagTransp: full transport group
// ═══════════════════════════════════════════════════════════════════════════

mod trait_tag_transp {
    use super::*;

    #[test]
    fn test_tag_transp_full_builds_full_transport_section_with_all_child_tags() {
        let xml = tag(
            "transp",
            &[],
            TagContent::Children(vec![
                tag("modFrete", &[], "0".into()),
                tag(
                    "transporta",
                    &[],
                    TagContent::Children(vec![
                        tag("CNPJ", &[], "12345678000195".into()),
                        tag("xNome", &[], "Transportadora ABC".into()),
                        tag("IE", &[], "111222333444".into()),
                        tag("xEnder", &[], "Rua do Transporte, 500".into()),
                        tag("xMun", &[], "Campinas".into()),
                        tag("UF", &[], "SP".into()),
                    ]),
                ),
                tag(
                    "veicTransp",
                    &[],
                    TagContent::Children(vec![
                        tag("placa", &[], "ABC1D23".into()),
                        tag("UF", &[], "SP".into()),
                        tag("RNTC", &[], "12345678".into()),
                    ]),
                ),
                tag(
                    "reboque",
                    &[],
                    TagContent::Children(vec![
                        tag("placa", &[], "XYZ9F87".into()),
                        tag("UF", &[], "SP".into()),
                        tag("RNTC", &[], "87654321".into()),
                    ]),
                ),
                tag(
                    "vol",
                    &[],
                    TagContent::Children(vec![
                        tag("qVol", &[], "10".into()),
                        tag("esp", &[], "CAIXA".into()),
                        tag("marca", &[], "MARCA X".into()),
                        tag("nVol", &[], "001".into()),
                        tag("pesoL", &[], "100.500".into()),
                        tag("pesoB", &[], "120.300".into()),
                        tag(
                            "lacres",
                            &[],
                            TagContent::Children(vec![tag("nLacre", &[], "LACRE001".into())]),
                        ),
                        tag(
                            "lacres",
                            &[],
                            TagContent::Children(vec![tag("nLacre", &[], "LACRE002".into())]),
                        ),
                    ]),
                ),
            ]),
        );

        expect_xml_contains(&xml, &[
            ("modFrete", "0"),
            ("placa", "ABC1D23"),
            ("RNTC", "12345678"),
            ("qVol", "10"),
            ("esp", "CAIXA"),
            ("marca", "MARCA X"),
            ("nVol", "001"),
            ("pesoL", "100.500"),
            ("pesoB", "120.300"),
        ]);
        assert!(xml.contains("<transporta>"));
        assert!(xml.contains("<veicTransp>"));
        assert!(xml.contains("<reboque>"));
        assert!(xml.contains("<vol>"));
        assert!(xml.contains("<lacres>"));
        assert!(xml.contains("<nLacre>LACRE001</nLacre>"));
        assert!(xml.contains("<nLacre>LACRE002</nLacre>"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  TraitTagDet: prod with optional fields, taginfAdProd, tagObsItem
// ═══════════════════════════════════════════════════════════════════════════

mod trait_tag_det_prod {
    use super::*;

    #[test]
    fn test_tag_prod_with_all_optional_fields() {
        let xml = tag(
            "prod",
            &[],
            TagContent::Children(vec![
                tag("cProd", &[], "001".into()),
                tag("cEAN", &[], "7891234567890".into()),
                tag("xProd", &[], "Produto Completo".into()),
                tag("NCM", &[], "61091000".into()),
                tag("CEST", &[], "2806300".into()),
                tag("indEscala", &[], "S".into()),
                tag("CNPJFab", &[], "12345678000195".into()),
                tag("CFOP", &[], "5102".into()),
                tag("uCom", &[], "UN".into()),
                tag("qCom", &[], "5.0000".into()),
                tag("vUnCom", &[], "20.0000000000".into()),
                tag("vProd", &[], "100.00".into()),
                tag("cEANTrib", &[], "7891234567890".into()),
                tag("uTrib", &[], "UN".into()),
                tag("qTrib", &[], "5.0000".into()),
                tag("vUnTrib", &[], "20.0000000000".into()),
                tag("vFrete", &[], "10.00".into()),
                tag("vSeg", &[], "5.00".into()),
                tag("vDesc", &[], "3.00".into()),
                tag("vOutro", &[], "2.00".into()),
                tag("indTot", &[], "1".into()),
                tag("xPed", &[], "PED-12345".into()),
                tag("nItemPed", &[], "001".into()),
            ]),
        );

        expect_xml_contains(&xml, &[
            ("cEAN", "7891234567890"),
            ("CEST", "2806300"),
            ("indEscala", "S"),
            ("CNPJFab", "12345678000195"),
            ("vFrete", "10.00"),
            ("vSeg", "5.00"),
            ("vDesc", "3.00"),
            ("vOutro", "2.00"),
            ("xPed", "PED-12345"),
            ("nItemPed", "001"),
        ]);
    }

    #[test]
    fn test_tag_prod_full_render_with_inf_ad_prod_and_obs_item() {
        let mut data = make_base_invoice_data();

        let mut item = make_base_item();
        item.description = "Produto Completo".into();
        item.inf_ad_prod = Some("Informacao adicional do produto item 1".into());
        item.obs_item = Some(ObsItemData {
            obs_cont: Some(ObsField {
                x_campo: "CampoTeste".into(),
                x_texto: "ValorTeste".into(),
            }),
            obs_fisco: None,
        });
        data.items = vec![item];

        let result = build_invoice_xml(&data).unwrap();
        let xml = &result.xml;

        // infAdProd should be inside det, after imposto
        assert!(xml.contains("<infAdProd>Informacao adicional do produto item 1</infAdProd>"));
        let imposto_end_pos = xml.find("</imposto>").unwrap();
        let inf_ad_prod_pos = xml.find("<infAdProd>").unwrap();
        let det_end_pos = xml.find("</det>").unwrap();
        assert!(inf_ad_prod_pos > imposto_end_pos);
        assert!(inf_ad_prod_pos < det_end_pos);

        // obsItem should be inside det
        assert!(xml.contains("<obsItem>"));
        assert!(xml.contains("xCampo=\"CampoTeste\""));
        assert!(xml.contains("<xTexto>ValorTeste</xTexto>"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  TraitTagDetOptions: batch tracking, vehicles, medicine, weapons
// ═══════════════════════════════════════════════════════════════════════════

mod trait_tag_det_options {
    use super::*;

    #[test]
    fn test_tag_rastro_batch_tracking() {
        let mut data = make_base_invoice_data();
        let mut item = make_base_item();
        item.rastro = Some(vec![
            RastroData {
                n_lote: "LOTE2025A".into(),
                q_lote: 100.0,
                d_fab: "2025-01-15".into(),
                d_val: "2026-01-15".into(),
                c_agreg: Some("AGR001".into()),
            },
            RastroData {
                n_lote: "LOTE2025B".into(),
                q_lote: 50.0,
                d_fab: "2025-02-10".into(),
                d_val: "2026-02-10".into(),
                c_agreg: None,
            },
        ]);
        data.items = vec![item];

        let result = build_invoice_xml(&data).unwrap();
        let xml = &result.xml;

        // rastro should be inside <prod>
        assert!(xml.contains("<rastro>"));
        let prod_pos = xml.find("<prod>").unwrap();
        let rastro_pos = xml.find("<rastro>").unwrap();
        let prod_end_pos = xml.find("</prod>").unwrap();
        assert!(rastro_pos > prod_pos);
        assert!(rastro_pos < prod_end_pos);

        expect_xml_contains(xml, &[
            ("nLote", "LOTE2025A"),
            ("qLote", "100.000"),
            ("dFab", "2025-01-15"),
            ("dVal", "2026-01-15"),
            ("cAgreg", "AGR001"),
        ]);
        // Second rastro
        assert!(xml.contains("<nLote>LOTE2025B</nLote>"));
        // Count rastro occurrences
        let count = xml.matches("<rastro>").count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_tag_veic_prod_vehicle() {
        let mut data = make_base_invoice_data();
        let mut item = make_base_item();
        item.veic_prod = Some(VeicProdData {
            tp_op: "1".into(),
            chassi: "9BWSU19F08B302158".into(),
            c_cor: "1".into(),
            x_cor: "BRANCA".into(),
            pot: "150".into(),
            cilin: "1600".into(),
            peso_l: "1200".into(),
            peso_b: "1350".into(),
            n_serie: "AAA111222".into(),
            tp_comb: "16".into(),
            n_motor: "MOT12345".into(),
            cmt: "1800.0000".into(),
            dist: "2600".into(),
            ano_mod: "2025".into(),
            ano_fab: "2024".into(),
            tp_pint: "M".into(),
            tp_veic: "06".into(),
            esp_veic: "1".into(),
            vin: "R".into(),
            cond_veic: "1".into(),
            c_mod: "123456".into(),
            c_cor_denatran: "01".into(),
            lota: "5".into(),
            tp_rest: "0".into(),
        });
        data.items = vec![item];

        let result = build_invoice_xml(&data).unwrap();
        let xml = &result.xml;

        // veicProd should be inside <prod>
        assert!(xml.contains("<veicProd>"));
        let prod_pos = xml.find("<prod>").unwrap();
        let veic_pos = xml.find("<veicProd>").unwrap();
        let prod_end_pos = xml.find("</prod>").unwrap();
        assert!(veic_pos > prod_pos);
        assert!(veic_pos < prod_end_pos);

        expect_xml_contains(xml, &[
            ("chassi", "9BWSU19F08B302158"),
            ("xCor", "BRANCA"),
            ("pot", "150"),
            ("nMotor", "MOT12345"),
            ("anoMod", "2025"),
            ("anoFab", "2024"),
            ("tpRest", "0"),
        ]);
    }

    #[test]
    fn test_tag_med_medicine() {
        let mut data = make_base_invoice_data();
        let mut item = make_base_item();
        item.med = Some(MedData {
            c_prod_anvisa: Some("1234567890123".into()),
            x_motivo_isencao: None,
            v_pmc: Cents(4990),
        });
        data.items = vec![item];

        let result = build_invoice_xml(&data).unwrap();
        let xml = &result.xml;

        // med should be inside <prod>
        assert!(xml.contains("<med>"));
        let prod_pos = xml.find("<prod>").unwrap();
        let med_pos = xml.find("<med>").unwrap();
        let prod_end_pos = xml.find("</prod>").unwrap();
        assert!(med_pos > prod_pos);
        assert!(med_pos < prod_end_pos);

        expect_xml_contains(xml, &[
            ("cProdANVISA", "1234567890123"),
            ("vPMC", "49.90"),
        ]);
    }

    #[test]
    fn test_tag_arma_weapon() {
        let mut data = make_base_invoice_data();
        let mut item = make_base_item();
        item.arma = Some(vec![ArmaData {
            tp_arma: "0".into(),
            n_serie: "SR12345".into(),
            n_cano: "CN67890".into(),
            descr: "REVOLVER CALIBRE 38".into(),
        }]);
        data.items = vec![item];

        let result = build_invoice_xml(&data).unwrap();
        let xml = &result.xml;

        // arma should be inside <prod>
        assert!(xml.contains("<arma>"));
        let prod_pos = xml.find("<prod>").unwrap();
        let arma_pos = xml.find("<arma>").unwrap();
        let prod_end_pos = xml.find("</prod>").unwrap();
        assert!(arma_pos > prod_pos);
        assert!(arma_pos < prod_end_pos);

        expect_xml_contains(xml, &[
            ("tpArma", "0"),
            ("nSerie", "SR12345"),
            ("nCano", "CN67890"),
            ("descr", "REVOLVER CALIBRE 38"),
        ]);
    }

    #[test]
    fn test_tag_recopi_render_includes_nrecopi() {
        let mut data = make_base_invoice_data();
        let mut item = make_base_item();
        item.n_recopi = Some("20250101120000123456".into());
        data.items = vec![item];

        let result = build_invoice_xml(&data).unwrap();
        let xml = &result.xml;

        // nRECOPI should be inside <prod>
        assert!(xml.contains("<nRECOPI>20250101120000123456</nRECOPI>"));
        let prod_pos = xml.find("<prod>").unwrap();
        let recopi_pos = xml.find("<nRECOPI>").unwrap();
        let prod_end_pos = xml.find("</prod>").unwrap();
        assert!(recopi_pos > prod_pos);
        assert!(recopi_pos < prod_end_pos);
    }

    #[test]
    fn test_tag_recopi_empty_should_not_render_tag() {
        let mut data = make_base_invoice_data();
        let mut item = make_base_item();
        item.n_recopi = Some(String::new());
        data.items = vec![item];

        let result = build_invoice_xml(&data).unwrap();
        let xml = &result.xml;

        // Empty nRECOPI should not produce the tag (falsy check)
        assert!(!xml.contains("<nRECOPI>"));
    }

    #[test]
    fn test_tag_dfe_referenciado() {
        let mut data = make_base_invoice_data();
        let mut item = make_base_item();
        item.dfe_referenciado = Some(DFeReferenciadoData {
            chave_acesso: "35170358716523000119550010000000291000000291".into(),
            n_item: Some("1".into()),
        });
        data.items = vec![item];

        let result = build_invoice_xml(&data).unwrap();
        let xml = &result.xml;

        // DFeReferenciado should be inside <det>, after imposto
        assert!(xml.contains("<DFeReferenciado>"));
        let imposto_end_pos = xml.find("</imposto>").unwrap();
        let dfe_ref_pos = xml.find("<DFeReferenciado>").unwrap();
        let det_end_pos = xml.find("</det>").unwrap();
        assert!(dfe_ref_pos > imposto_end_pos);
        assert!(dfe_ref_pos < det_end_pos);

        expect_xml_contains(xml, &[
            ("chaveAcesso", "35170358716523000119550010000000291000000291"),
            ("nItem", "1"),
        ]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  TraitTagRefs: NF-e reference tags in ide
// ═══════════════════════════════════════════════════════════════════════════

mod trait_tag_refs {
    use super::*;

    #[test]
    fn test_tag_ref_nfe_in_ide() {
        let xml = tag(
            "ide",
            &[],
            TagContent::Children(vec![
                tag("cUF", &[], "35".into()),
                tag(
                    "NFref",
                    &[],
                    TagContent::Children(vec![tag(
                        "refNFe",
                        &[],
                        "35170358716523000119550010000000291000000291".into(),
                    )]),
                ),
            ]),
        );

        assert!(xml.contains("<NFref>"));
        assert!(xml.contains(
            "<refNFe>35170358716523000119550010000000291000000291</refNFe>"
        ));
        // NFref should be inside ide
        let ide_pos = xml.find("<ide>").unwrap();
        let nfref_pos = xml.find("<NFref>").unwrap();
        let ide_end_pos = xml.find("</ide>").unwrap();
        assert!(nfref_pos > ide_pos);
        assert!(nfref_pos < ide_end_pos);
    }

    #[test]
    fn test_tag_ref_nf_in_ide() {
        let xml = tag(
            "NFref",
            &[],
            TagContent::Children(vec![tag(
                "refNF",
                &[],
                TagContent::Children(vec![
                    tag("cUF", &[], "35".into()),
                    tag("AAMM", &[], "1703".into()),
                    tag("CNPJ", &[], "58716523000119".into()),
                    tag("mod", &[], "01".into()),
                    tag("serie", &[], "1".into()),
                    tag("nNF", &[], "100".into()),
                ]),
            )]),
        );

        assert!(xml.contains("<NFref>"));
        assert!(xml.contains("<refNF>"));
        expect_xml_contains(&xml, &[("AAMM", "1703"), ("mod", "01")]);
    }

    #[test]
    fn test_tag_ref_nfp_with_cnpj_in_ide() {
        let xml = tag(
            "NFref",
            &[],
            TagContent::Children(vec![tag(
                "refNFP",
                &[],
                TagContent::Children(vec![
                    tag("cUF", &[], "35".into()),
                    tag("AAMM", &[], "1703".into()),
                    tag("CNPJ", &[], "58716523000119".into()),
                    tag("IE", &[], "123456789012".into()),
                    tag("mod", &[], "04".into()),
                    tag("serie", &[], "1".into()),
                    tag("nNF", &[], "50".into()),
                ]),
            )]),
        );

        assert!(xml.contains("<refNFP>"));
        expect_xml_contains(&xml, &[
            ("CNPJ", "58716523000119"),
            ("IE", "123456789012"),
            ("mod", "04"),
        ]);
    }

    #[test]
    fn test_tag_ref_nfp_with_cpf_in_ide() {
        let xml = tag(
            "NFref",
            &[],
            TagContent::Children(vec![tag(
                "refNFP",
                &[],
                TagContent::Children(vec![
                    tag("cUF", &[], "35".into()),
                    tag("AAMM", &[], "1703".into()),
                    tag("CPF", &[], "12345678901".into()),
                    tag("IE", &[], "ISENTO".into()),
                    tag("mod", &[], "04".into()),
                    tag("serie", &[], "0".into()),
                    tag("nNF", &[], "10".into()),
                ]),
            )]),
        );

        assert!(xml.contains("<refNFP>"));
        expect_xml_contains(&xml, &[("CPF", "12345678901"), ("IE", "ISENTO")]);
    }

    #[test]
    fn test_tag_ref_cte_in_ide() {
        let xml = tag(
            "NFref",
            &[],
            TagContent::Children(vec![tag(
                "refCTe",
                &[],
                "35170358716523000119570010000000011000000014".into(),
            )]),
        );

        assert!(xml.contains("<NFref>"));
        assert!(xml.contains(
            "<refCTe>35170358716523000119570010000000011000000014</refCTe>"
        ));
    }

    #[test]
    fn test_tag_ref_ecf_in_ide() {
        let xml = tag(
            "NFref",
            &[],
            TagContent::Children(vec![tag(
                "refECF",
                &[],
                TagContent::Children(vec![
                    tag("mod", &[], "2D".into()),
                    tag("nECF", &[], "123".into()),
                    tag("nCOO", &[], "456789".into()),
                ]),
            )]),
        );

        assert!(xml.contains("<refECF>"));
        expect_xml_contains(&xml, &[
            ("mod", "2D"),
            ("nECF", "123"),
            ("nCOO", "456789"),
        ]);
    }

    #[test]
    fn test_multiple_refs_in_ide() {
        let xml = tag(
            "ide",
            &[],
            TagContent::Children(vec![
                tag(
                    "NFref",
                    &[],
                    TagContent::Children(vec![tag(
                        "refNFe",
                        &[],
                        "35170358716523000119550010000000291000000291".into(),
                    )]),
                ),
                tag(
                    "NFref",
                    &[],
                    TagContent::Children(vec![tag(
                        "refCTe",
                        &[],
                        "35170358716523000119570010000000011000000014".into(),
                    )]),
                ),
            ]),
        );

        assert!(xml.contains("<refNFe>"));
        assert!(xml.contains("<refCTe>"));
        // Count NFref occurrences
        let count = xml.matches("<NFref>").count();
        assert_eq!(count, 2);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  TraitTagPag: payment with change and card details
// ═══════════════════════════════════════════════════════════════════════════

mod trait_tag_pag {
    use super::*;

    #[test]
    fn test_tag_pag_with_vtroco_and_card_details() {
        let xml = tag(
            "pag",
            &[],
            TagContent::Children(vec![
                tag(
                    "detPag",
                    &[],
                    TagContent::Children(vec![
                        tag("tPag", &[], "03".into()),
                        tag("vPag", &[], "150.00".into()),
                        tag(
                            "card",
                            &[],
                            TagContent::Children(vec![
                                tag("tpIntegra", &[], "1".into()),
                                tag("CNPJ", &[], "12345678000195".into()),
                                tag("tBand", &[], "01".into()),
                                tag("cAut", &[], "AUTH123456".into()),
                            ]),
                        ),
                    ]),
                ),
                tag("vTroco", &[], "50.00".into()),
            ]),
        );

        assert!(xml.contains("<pag>"));
        assert!(xml.contains("<vTroco>50.00</vTroco>"));
        assert!(xml.contains("<detPag>"));
        expect_xml_contains(&xml, &[
            ("tPag", "03"),
            ("vPag", "150.00"),
            ("tpIntegra", "1"),
            ("tBand", "01"),
            ("cAut", "AUTH123456"),
        ]);
        assert!(xml.contains("<card>"));

        // vTroco must come AFTER detPag
        let det_pag_pos = xml.find("<detPag>").unwrap();
        let v_troco_pos = xml.find("<vTroco>").unwrap();
        assert!(v_troco_pos > det_pag_pos);
    }

    #[test]
    fn test_tag_pag_multiple_payments() {
        let xml = tag(
            "pag",
            &[],
            TagContent::Children(vec![
                tag(
                    "detPag",
                    &[],
                    TagContent::Children(vec![
                        tag("tPag", &[], "01".into()),
                        tag("vPag", &[], "100.00".into()),
                    ]),
                ),
                tag(
                    "detPag",
                    &[],
                    TagContent::Children(vec![
                        tag("tPag", &[], "03".into()),
                        tag("vPag", &[], "50.00".into()),
                    ]),
                ),
            ]),
        );

        let count = xml.matches("<detPag>").count();
        assert_eq!(count, 2);
        assert!(xml.contains("<tPag>01</tPag>"));
        assert!(xml.contains("<tPag>03</tPag>"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  TraitCalculations: totals from multiple items
// ═══════════════════════════════════════════════════════════════════════════

mod trait_calculations {
    use super::*;

    #[test]
    fn test_calculations_totals_from_multiple_items() {
        let mut data = make_base_invoice_data();

        let mut item1 = make_base_item();
        item1.item_number = 1;
        item1.product_code = "001".into();
        item1.description = "Produto A".into();
        item1.quantity = 2.0;
        item1.unit_price = Cents(10000);
        item1.total_price = Cents(20000);
        item1.icms_amount = Cents(3600);
        item1.pis_v_bc = Some(Cents(20000));
        item1.pis_p_pis = Some(Rate4(16500));
        item1.pis_v_pis = Some(Cents(330));
        item1.cofins_v_bc = Some(Cents(20000));
        item1.cofins_p_cofins = Some(Rate4(76000));
        item1.cofins_v_cofins = Some(Cents(1520));

        let mut item2 = make_base_item();
        item2.item_number = 2;
        item2.product_code = "002".into();
        item2.description = "Produto B".into();
        item2.quantity = 3.0;
        item2.unit_price = Cents(10000);
        item2.total_price = Cents(30000);
        item2.icms_amount = Cents(5400);
        item2.pis_v_bc = Some(Cents(30000));
        item2.pis_p_pis = Some(Rate4(16500));
        item2.pis_v_pis = Some(Cents(495));
        item2.cofins_v_bc = Some(Cents(30000));
        item2.cofins_p_cofins = Some(Rate4(76000));
        item2.cofins_v_cofins = Some(Cents(2280));

        data.items = vec![item1, item2];
        data.payments = vec![PaymentData {
            method: "01".into(),
            amount: Cents(50000),
        }];

        let result = build_invoice_xml(&data).unwrap();
        let xml = &result.xml;

        // vProd should be sum of item totals: 20000 + 30000 = 50000 cents = 500.00
        assert!(xml.contains("<vProd>500.00</vProd>"));
        // vNF should equal vProd for simple case
        assert!(xml.contains("<vNF>500.00</vNF>"));
        // ICMS totals: 3600 + 5400 = 9000 cents = 90.00
        assert!(xml.contains("<vICMS>90.00</vICMS>"));
    }

    #[test]
    fn test_calculations_with_different_tax_values() {
        let mut data = make_base_invoice_data();
        data.number = 2;

        let mut item = make_base_item();
        item.description = "Produto Unico".into();
        item.quantity = 1.0;
        item.unit_price = Cents(26400);
        item.total_price = Cents(26400);
        item.icms_amount = Cents(4752);
        item.pis_v_bc = Some(Cents(26400));
        item.pis_p_pis = Some(Rate4(16500));
        item.pis_v_pis = Some(Cents(436));
        item.cofins_v_bc = Some(Cents(26400));
        item.cofins_p_cofins = Some(Rate4(76000));
        item.cofins_v_cofins = Some(Cents(2006));

        data.items = vec![item];
        data.payments = vec![PaymentData {
            method: "01".into(),
            amount: Cents(26400),
        }];

        let result = build_invoice_xml(&data).unwrap();
        let xml = &result.xml;

        // vNF = vProd = 26400 cents = 264.00
        assert!(xml.contains("<vNF>264.00</vNF>"));
        assert!(xml.contains("<vProd>264.00</vProd>"));
        // PIS = 4.36, COFINS = 20.06
        assert!(xml.contains("<vPIS>4.36</vPIS>"));
        assert!(xml.contains("<vCOFINS>20.06</vCOFINS>"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  TraitTagCobr: billing section
// ═══════════════════════════════════════════════════════════════════════════

mod trait_tag_cobr {
    use super::*;

    #[test]
    fn test_tag_cobr_builds_cobr_with_fat_and_dup() {
        let xml = tag(
            "cobr",
            &[],
            TagContent::Children(vec![
                tag(
                    "fat",
                    &[],
                    TagContent::Children(vec![
                        tag("nFat", &[], "001".into()),
                        tag("vOrig", &[], "100.00".into()),
                        tag("vDesc", &[], "0.00".into()),
                        tag("vLiq", &[], "100.00".into()),
                    ]),
                ),
                tag(
                    "dup",
                    &[],
                    TagContent::Children(vec![
                        tag("nDup", &[], "001".into()),
                        tag("dVenc", &[], "2017-04-03".into()),
                        tag("vDup", &[], "100.00".into()),
                    ]),
                ),
            ]),
        );

        assert!(xml.contains("<cobr>"));
        assert!(xml.contains("<fat>"));
        expect_xml_contains(&xml, &[
            ("nFat", "001"),
            ("nDup", "001"),
            ("dVenc", "2017-04-03"),
            ("vDup", "100.00"),
        ]);
    }
}
