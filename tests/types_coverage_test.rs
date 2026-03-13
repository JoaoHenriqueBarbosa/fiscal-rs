// Tests to cover all uncovered builder methods and enum variants in types.rs.

mod common;

use fiscal::newtypes::{Cents, IbgeCode, Rate, Rate4};
use fiscal::types::*;

// ── InvoiceModel Display ────────────────────────────────────────────────────

#[test]
fn invoice_model_display() {
    assert_eq!(format!("{}", InvoiceModel::Nfe), "55");
    assert_eq!(format!("{}", InvoiceModel::Nfce), "65");
}

// ── EmissionType::as_str — uncovered variants ───────────────────────────────

#[test]
fn emission_type_as_str_all_variants() {
    assert_eq!(EmissionType::Normal.as_str(), "1");
    assert_eq!(EmissionType::FsIa.as_str(), "2");
    assert_eq!(EmissionType::Epec.as_str(), "4");
    assert_eq!(EmissionType::FsDa.as_str(), "5");
    assert_eq!(EmissionType::SvcAn.as_str(), "6");
    assert_eq!(EmissionType::SvcRs.as_str(), "7");
    assert_eq!(EmissionType::Offline.as_str(), "9");
}

// ── ContingencyType::as_str — uncovered variants ────────────────────────────

#[test]
fn contingency_type_as_str_uncovered_variants() {
    assert_eq!(ContingencyType::Epec.as_str(), "epec");
    assert_eq!(ContingencyType::FsDa.as_str(), "fs-da");
    assert_eq!(ContingencyType::FsIa.as_str(), "fs-ia");
    assert_eq!(ContingencyType::Offline.as_str(), "offline");
}

// ── IssuerData builder methods ──────────────────────────────────────────────

#[test]
fn issuer_data_optional_builders() {
    let issuer = common::sample_issuer()
        .address_complement("Sala 1")
        .phone("11999998888")
        .iest("123456789")
        .im("12345")
        .cnae("1234567");

    assert_eq!(issuer.address_complement.as_deref(), Some("Sala 1"));
    assert_eq!(issuer.phone.as_deref(), Some("11999998888"));
    assert_eq!(issuer.iest.as_deref(), Some("123456789"));
    assert_eq!(issuer.im.as_deref(), Some("12345"));
    assert_eq!(issuer.cnae.as_deref(), Some("1234567"));
}

// ── RecipientData builder methods ───────────────────────────────────────────

#[test]
fn recipient_data_optional_builders() {
    let recipient = common::sample_recipient()
        .complement("Apto 42")
        .phone("1133334444")
        .email("test@example.com")
        .isuf("12345")
        .im("67890")
        .ind_ie_dest("9")
        .country_code("1058")
        .country_name("Brasil")
        .zip_code("01001000");

    assert_eq!(recipient.complement.as_deref(), Some("Apto 42"));
    assert_eq!(recipient.phone.as_deref(), Some("1133334444"));
    assert_eq!(recipient.email.as_deref(), Some("test@example.com"));
    assert_eq!(recipient.isuf.as_deref(), Some("12345"));
    assert_eq!(recipient.im.as_deref(), Some("67890"));
    assert_eq!(recipient.ind_ie_dest.as_deref(), Some("9"));
    assert_eq!(recipient.country_code.as_deref(), Some("1058"));
    assert_eq!(recipient.country_name.as_deref(), Some("Brasil"));
    assert_eq!(recipient.zip_code.as_deref(), Some("01001000"));
}

// ── PaymentData builder methods ─────────────────────────────────────────────

#[test]
fn payment_data_optional_builders() {
    let payment = common::sample_payment()
        .ind_pag("0")
        .x_pag("Dinheiro")
        .d_pag("2026-01-15")
        .cnpj_pag("12345678000199")
        .uf_pag("SP");

    assert_eq!(payment.ind_pag.as_deref(), Some("0"));
    assert_eq!(payment.x_pag.as_deref(), Some("Dinheiro"));
    assert_eq!(payment.d_pag.as_deref(), Some("2026-01-15"));
    assert_eq!(payment.cnpj_pag.as_deref(), Some("12345678000199"));
    assert_eq!(payment.uf_pag.as_deref(), Some("SP"));
}

// ── PaymentCardDetail builder methods ───────────────────────────────────────

#[test]
fn payment_card_detail_builders() {
    let card = PaymentCardDetail::new()
        .integ_type("1")
        .card_tax_id("12345678000199")
        .card_brand("01")
        .auth_code("ABC123");

    assert_eq!(card.integ_type.as_deref(), Some("1"));
    assert_eq!(card.card_tax_id.as_deref(), Some("12345678000199"));
    assert_eq!(card.card_brand.as_deref(), Some("01"));
    assert_eq!(card.auth_code.as_deref(), Some("ABC123"));
}

// ── CarrierData builder methods ─────────────────────────────────────────────

#[test]
fn carrier_data_optional_builders() {
    let carrier = CarrierData::new()
        .tax_id("12345678000199")
        .state_tax_id("123456789")
        .state_code("SP")
        .address("Rua Teste 100");

    assert_eq!(carrier.tax_id.as_deref(), Some("12345678000199"));
    assert_eq!(carrier.state_tax_id.as_deref(), Some("123456789"));
    assert_eq!(carrier.state_code.as_deref(), Some("SP"));
    assert_eq!(carrier.address.as_deref(), Some("Rua Teste 100"));
}

// ── VehicleData builder methods ─────────────────────────────────────────────

#[test]
fn vehicle_data_rntc_builder() {
    let vehicle = VehicleData::new("ABC1234", "SP").rntc("RNTC123");
    assert_eq!(vehicle.rntc.as_deref(), Some("RNTC123"));
}

// ── VolumeData builder methods ──────────────────────────────────────────────

#[test]
fn volume_data_all_builders() {
    let volume = VolumeData::new()
        .quantity(10)
        .species("CAIXA")
        .brand("TestBrand")
        .number("VOL001")
        .net_weight(15.5)
        .gross_weight(16.0)
        .seals(vec!["SEAL001".to_string(), "SEAL002".to_string()]);

    assert_eq!(volume.quantity, Some(10));
    assert_eq!(volume.species.as_deref(), Some("CAIXA"));
    assert_eq!(volume.brand.as_deref(), Some("TestBrand"));
    assert_eq!(volume.number.as_deref(), Some("VOL001"));
    assert_eq!(volume.net_weight, Some(15.5));
    assert_eq!(volume.gross_weight, Some(16.0));
    assert_eq!(volume.seals.as_ref().unwrap().len(), 2);
}

// ── BillingInvoice builder ──────────────────────────────────────────────────

#[test]
fn billing_invoice_discount_value() {
    let inv = BillingInvoice::new("001", Cents(10000), Cents(9000))
        .discount_value(Cents(1000));
    assert_eq!(inv.discount_value, Some(Cents(1000)));
}

// ── LocationData builder methods ────────────────────────────────────────────

#[test]
fn location_data_optional_builders() {
    let location = LocationData::new(
        "12345678000199",
        "Rua Teste",
        "100",
        "Centro",
        IbgeCode("3550308".to_string()),
        "Sao Paulo",
        "SP",
    )
    .name("Local Teste")
    .complement("Sala 2")
    .zip_code("01001000");

    assert_eq!(location.name.as_deref(), Some("Local Teste"));
    assert_eq!(location.complement.as_deref(), Some("Sala 2"));
    assert_eq!(location.zip_code.as_deref(), Some("01001000"));
}

// ── AdditionalInfo builder methods ──────────────────────────────────────────

#[test]
fn additional_info_all_builders() {
    let info = AdditionalInfo::new()
        .tax_authority_note("Tax note")
        .contributor_obs(vec![FieldText::new("campo1", "texto1")])
        .fiscal_obs(vec![FieldText::new("campo2", "texto2")])
        .process_refs(vec![ProcessRef::new("PROC001", "0")]);

    assert_eq!(info.tax_authority_note.as_deref(), Some("Tax note"));
    assert!(info.contributor_obs.is_some());
    assert!(info.fiscal_obs.is_some());
    assert!(info.process_refs.is_some());
}

// ── FieldText::new covers lines 1405-1408 ──────────────────────────────────

#[test]
fn field_text_new() {
    let ft = FieldText::new("xCampo", "xTexto");
    assert_eq!(ft.field, "xCampo");
    assert_eq!(ft.text, "xTexto");
}

// ── ProcessRef::new covers lines 1425-1428 ─────────────────────────────────

#[test]
fn process_ref_new() {
    let pr = ProcessRef::new("PROC123", "0");
    assert_eq!(pr.number, "PROC123");
    assert_eq!(pr.origin, "0");
}

// ── PurchaseData builder methods ────────────────────────────────────────────

#[test]
fn purchase_data_builders() {
    let po = PurchaseData::new()
        .order_number("PED001")
        .purchase_note("Nota compra 123")
        .contract_number("CONT-001");

    assert_eq!(po.order_number.as_deref(), Some("PED001"));
    assert_eq!(po.purchase_note.as_deref(), Some("Nota compra 123"));
    assert_eq!(po.contract_number.as_deref(), Some("CONT-001"));
}

// ── ExportData builder method ───────────────────────────────────────────────

#[test]
fn export_data_dispatch_location_builder() {
    let export = ExportData::new("SP", "Santos")
        .dispatch_location("Guarulhos");
    assert_eq!(export.dispatch_location.as_deref(), Some("Guarulhos"));
}

// ── RetTribData builder methods ─────────────────────────────────────────────

#[test]
fn ret_trib_data_builders() {
    let ret = RetTribData::new()
        .v_ret_pis(Cents(100))
        .v_ret_cofins(Cents(200))
        .v_ret_csll(Cents(300))
        .v_bc_irrf(Cents(10000))
        .v_irrf(Cents(400))
        .v_bc_ret_prev(Cents(20000))
        .v_ret_prev(Cents(500));

    assert_eq!(ret.v_ret_pis, Some(Cents(100)));
    assert_eq!(ret.v_ret_cofins, Some(Cents(200)));
    assert_eq!(ret.v_ret_csll, Some(Cents(300)));
    assert_eq!(ret.v_bc_irrf, Some(Cents(10000)));
    assert_eq!(ret.v_irrf, Some(Cents(400)));
    assert_eq!(ret.v_bc_ret_prev, Some(Cents(20000)));
    assert_eq!(ret.v_ret_prev, Some(Cents(500)));
}

// ── MedData builder ─────────────────────────────────────────────────────────

#[test]
fn med_data_builders() {
    let med = MedData::new(Cents(1000))
        .c_prod_anvisa("1234567890123")
        .x_motivo_isencao("Isento");
    assert_eq!(med.c_prod_anvisa.as_deref(), Some("1234567890123"));
    assert_eq!(med.x_motivo_isencao.as_deref(), Some("Isento"));
}

// ── NfceQrCodeParams Debug impl ─────────────────────────────────────────────

#[test]
fn nfce_qr_code_params_debug() {
    let params = NfceQrCodeParams::new(
        "12345678901234567890123456789012345678901234",
        QrCodeVersion::V200,
        SefazEnvironment::Homologation,
        EmissionType::Normal,
        "http://example.com/qrcode",
    );
    let debug_str = format!("{:?}", params);
    assert!(debug_str.contains("NfceQrCodeParams"));
    assert!(debug_str.contains("access_key"));
}

// ── NfceQrCodeParams builder methods ────────────────────────────────────────

#[test]
fn nfce_qr_code_params_builders() {
    let params = NfceQrCodeParams::new(
        "12345678901234567890123456789012345678901234",
        QrCodeVersion::V200,
        SefazEnvironment::Homologation,
        EmissionType::Normal,
        "http://example.com/qrcode",
    )
    .csc_token("000001")
    .csc_id("CSC01")
    .issued_at("2026-01-15T10:30:00-03:00")
    .total_value("20.00")
    .total_icms("0.00")
    .digest_value("abc123")
    .dest_document("12345678901")
    .dest_id_type("2");

    assert_eq!(params.csc_token.as_deref(), Some("000001"));
    assert_eq!(params.csc_id.as_deref(), Some("CSC01"));
    assert_eq!(params.issued_at.as_deref(), Some("2026-01-15T10:30:00-03:00"));
    assert_eq!(params.total_value.as_deref(), Some("20.00"));
    assert_eq!(params.total_icms.as_deref(), Some("0.00"));
    assert_eq!(params.digest_value.as_deref(), Some("abc123"));
    assert_eq!(params.dest_document.as_deref(), Some("12345678901"));
    assert_eq!(params.dest_id_type.as_deref(), Some("2"));
}

// ── InvoiceItemData — all uncovered chainable setters ───────────────────────

#[test]
fn invoice_item_data_ean_and_nve_builders() {
    let item = common::sample_item()
        .c_ean("7891234567890")
        .c_ean_trib("7891234567890")
        .nve("AB0001")
        .nve("CD0002");

    assert_eq!(item.c_ean.as_deref(), Some("7891234567890"));
    assert_eq!(item.c_ean_trib.as_deref(), Some("7891234567890"));
    assert_eq!(item.nve.len(), 2);
}

#[test]
fn invoice_item_data_cest_builders() {
    let item = common::sample_item()
        .cest("1234567")
        .cest_ind_escala("S")
        .cest_cnpj_fab("12345678000199")
        .c_benef("SEM CBENEF");

    assert_eq!(item.cest.as_deref(), Some("1234567"));
    assert_eq!(item.cest_ind_escala.as_deref(), Some("S"));
    assert_eq!(item.cest_cnpj_fab.as_deref(), Some("12345678000199"));
    assert_eq!(item.c_benef.as_deref(), Some("SEM CBENEF"));
}

#[test]
fn invoice_item_data_tipi_and_order_builders() {
    let item = common::sample_item()
        .extipi("01")
        .x_ped("PED001")
        .n_item_ped("1")
        .n_fci("12345678-1234-1234-1234-123456789012");

    assert_eq!(item.extipi.as_deref(), Some("01"));
    assert_eq!(item.x_ped.as_deref(), Some("PED001"));
    assert_eq!(item.n_item_ped.as_deref(), Some("1"));
    assert_eq!(item.n_fci.as_deref(), Some("12345678-1234-1234-1234-123456789012"));
}

#[test]
fn invoice_item_data_freight_ins_desc_outro_builders() {
    let item = common::sample_item()
        .v_frete(Cents(500))
        .v_seg(Cents(100))
        .v_desc(Cents(200))
        .v_outro(Cents(50));

    assert_eq!(item.v_frete, Some(Cents(500)));
    assert_eq!(item.v_seg, Some(Cents(100)));
    assert_eq!(item.v_desc, Some(Cents(200)));
    assert_eq!(item.v_outro, Some(Cents(50)));
}

#[test]
fn invoice_item_data_icms_builders() {
    let item = common::sample_item()
        .orig("0")
        .icms_mod_bc(3)
        .icms_red_bc(Rate(5000))
        .icms_mod_bc_st(4)
        .icms_p_mva_st(Rate(4000))
        .icms_red_bc_st(Rate(3000))
        .icms_v_bc_st(Cents(10000))
        .icms_p_icms_st(Rate(1800))
        .icms_v_icms_st(Cents(1800))
        .icms_v_icms_deson(Cents(500))
        .icms_mot_des_icms(9)
        .icms_p_fcp(Rate(200))
        .icms_v_fcp(Cents(40))
        .icms_v_bc_fcp(Cents(2000))
        .icms_p_fcp_st(Rate(200))
        .icms_v_fcp_st(Cents(40))
        .icms_v_bc_fcp_st(Cents(2000))
        .icms_p_cred_sn(Rate(150))
        .icms_v_cred_icms_sn(Cents(30))
        .icms_v_icms_substituto(Cents(100))
        .icms_ind_deduz_deson("1");

    assert_eq!(item.orig.as_deref(), Some("0"));
    assert_eq!(item.icms_mod_bc, Some(3));
    assert_eq!(item.icms_red_bc, Some(Rate(5000)));
    assert_eq!(item.icms_mod_bc_st, Some(4));
    assert_eq!(item.icms_p_mva_st, Some(Rate(4000)));
    assert_eq!(item.icms_red_bc_st, Some(Rate(3000)));
    assert_eq!(item.icms_v_bc_st, Some(Cents(10000)));
    assert_eq!(item.icms_p_icms_st, Some(Rate(1800)));
    assert_eq!(item.icms_v_icms_st, Some(Cents(1800)));
    assert_eq!(item.icms_v_icms_deson, Some(Cents(500)));
    assert_eq!(item.icms_mot_des_icms, Some(9));
    assert_eq!(item.icms_p_fcp, Some(Rate(200)));
    assert_eq!(item.icms_v_fcp, Some(Cents(40)));
    assert_eq!(item.icms_v_bc_fcp, Some(Cents(2000)));
    assert_eq!(item.icms_p_fcp_st, Some(Rate(200)));
    assert_eq!(item.icms_v_fcp_st, Some(Cents(40)));
    assert_eq!(item.icms_v_bc_fcp_st, Some(Cents(2000)));
    assert_eq!(item.icms_p_cred_sn, Some(Rate(150)));
    assert_eq!(item.icms_v_cred_icms_sn, Some(Cents(30)));
    assert_eq!(item.icms_v_icms_substituto, Some(Cents(100)));
    assert_eq!(item.icms_ind_deduz_deson.as_deref(), Some("1"));
}

#[test]
fn invoice_item_data_pis_cofins_builders() {
    let item = common::sample_item()
        .pis_v_bc(Cents(10000))
        .pis_p_pis(Rate4(165))
        .pis_v_pis(Cents(165))
        .pis_q_bc_prod(1000)
        .pis_v_aliq_prod(100)
        .cofins_v_bc(Cents(10000))
        .cofins_p_cofins(Rate4(760))
        .cofins_v_cofins(Cents(760))
        .cofins_q_bc_prod(1000)
        .cofins_v_aliq_prod(100);

    assert_eq!(item.pis_v_bc, Some(Cents(10000)));
    assert_eq!(item.pis_p_pis, Some(Rate4(165)));
    assert_eq!(item.pis_v_pis, Some(Cents(165)));
    assert_eq!(item.pis_q_bc_prod, Some(1000));
    assert_eq!(item.pis_v_aliq_prod, Some(100));
    assert_eq!(item.cofins_v_bc, Some(Cents(10000)));
    assert_eq!(item.cofins_p_cofins, Some(Rate4(760)));
    assert_eq!(item.cofins_v_cofins, Some(Cents(760)));
    assert_eq!(item.cofins_q_bc_prod, Some(1000));
    assert_eq!(item.cofins_v_aliq_prod, Some(100));
}

#[test]
fn invoice_item_data_ipi_builders() {
    let item = common::sample_item()
        .ipi_cst("50")
        .ipi_c_enq("999")
        .ipi_v_bc(Cents(10000))
        .ipi_p_ipi(Rate(500))
        .ipi_v_ipi(Cents(500))
        .ipi_q_unid(100)
        .ipi_v_unid(50);

    assert_eq!(item.ipi_cst.as_deref(), Some("50"));
    assert_eq!(item.ipi_c_enq.as_deref(), Some("999"));
    assert_eq!(item.ipi_v_bc, Some(Cents(10000)));
    assert_eq!(item.ipi_p_ipi, Some(Rate(500)));
    assert_eq!(item.ipi_v_ipi, Some(Cents(500)));
    assert_eq!(item.ipi_q_unid, Some(100));
    assert_eq!(item.ipi_v_unid, Some(50));
}

#[test]
fn invoice_item_data_ii_builders() {
    let item = common::sample_item()
        .ii_v_bc(Cents(50000))
        .ii_v_desp_adu(Cents(1000))
        .ii_v_ii(Cents(5000))
        .ii_v_iof(Cents(500));

    assert_eq!(item.ii_v_bc, Some(Cents(50000)));
    assert_eq!(item.ii_v_desp_adu, Some(Cents(1000)));
    assert_eq!(item.ii_v_ii, Some(Cents(5000)));
    assert_eq!(item.ii_v_iof, Some(Cents(500)));
}

#[test]
fn invoice_item_data_product_detail_builders() {
    let item = common::sample_item()
        .rastro(vec![RastroData::new("LOT001", 100.0, "2025-01-01", "2026-01-01")])
        .veic_prod(VeicProdData::new(
            "1", "9BWHE21JX24060960", "1", "BRANCA", "100", "1000",
            "1500", "1800", "AAA111", "1", "MTR001", "1500", "2600",
            "2026", "2026", "M", "6", "1", "R", "1", "999999",
            "01", "5", "0",
        ))
        .med(MedData::new(Cents(1000)))
        .arma(vec![ArmaData::new("0", "AAA111", "BBB222", "Pistola")])
        .comb(CombData::new("320101001", "GASOLINA C COMUM", "SP"))
        .n_recopi("20260101");

    assert!(item.rastro.is_some());
    assert!(item.veic_prod.is_some());
    assert!(item.med.is_some());
    assert!(item.arma.is_some());
    assert!(item.comb.is_some());
    assert_eq!(item.n_recopi.as_deref(), Some("20260101"));
}

#[test]
fn invoice_item_data_service_and_misc_builders() {
    let item = common::sample_item()
        .issqn(fiscal::tax_issqn::IssqnData::new(
            10000, 500, 500, "9999", "1401",
        ))
        .inf_ad_prod("Info adicional do produto")
        .g_cred(vec![GCredData::new("SP000001", Rate4(100))])
        .ind_tot(0)
        .v_tot_trib(Cents(350));

    assert!(item.issqn.is_some());
    assert_eq!(item.inf_ad_prod.as_deref(), Some("Info adicional do produto"));
    assert_eq!(item.g_cred.len(), 1);
    assert_eq!(item.ind_tot, Some(0));
    assert_eq!(item.v_tot_trib, Some(Cents(350)));
}

#[test]
fn invoice_item_data_is_ibs_cbs_builders() {
    let item = common::sample_item()
        .is_data(fiscal::tax_is::IsData::new("00", "01", "5.00"))
        .ibs_cbs(fiscal::tax_ibs_cbs::IbsCbsData::new("00", "01"));

    assert!(item.is_data.is_some());
    assert!(item.ibs_cbs.is_some());
}

#[test]
fn invoice_item_data_pis_st_cofins_st_builders() {
    let item = common::sample_item()
        .pis_st(fiscal::tax_pis_cofins_ipi::PisStData::new(Cents(165)))
        .cofins_st(fiscal::tax_pis_cofins_ipi::CofinsStData::new(Cents(760)));

    assert!(item.pis_st.is_some());
    assert!(item.cofins_st.is_some());
}

#[test]
fn invoice_item_data_obs_item_and_dfe_ref_builders() {
    let item = common::sample_item()
        .obs_item(
            ObsItemData::new()
                .obs_cont(ObsField::new("campo", "valor"))
        )
        .dfe_referenciado(
            DFeReferenciadoData::new("12345678901234567890123456789012345678901234")
                .n_item("1")
        )
        .di(vec![])
        .det_export(vec![])
        .imposto_devol(ImpostoDevolData::new(Rate(1000), Cents(500)));

    assert!(item.obs_item.is_some());
    assert!(item.dfe_referenciado.is_some());
    assert!(item.di.is_some());
    assert!(item.det_export.is_some());
    assert!(item.imposto_devol.is_some());
}

// ── CarrierData::municipality builder (lines 1064-1066) ─────────────────

#[test]
fn carrier_data_municipality_builder() {
    let carrier = CarrierData::new()
        .municipality("Sao Paulo");
    assert_eq!(carrier.municipality.as_deref(), Some("Sao Paulo"));
}

// ── ProcessRef::with_tp_ato (lines 1454-1456) ──────────────────────────

#[test]
fn process_ref_with_tp_ato() {
    let pr = ProcessRef::with_tp_ato("PROC123", "0", "08");
    assert_eq!(pr.number, "PROC123");
    assert_eq!(pr.origin, "0");
    assert_eq!(pr.tp_ato.as_deref(), Some("08"));
}

// ── InvoiceItemData: c_barra, c_barra_trib, taxable_unit, taxable_quantity,
//    taxable_unit_price (lines 2858-2885) ────────────────────────────────

#[test]
fn invoice_item_data_c_barra_builders() {
    let item = common::sample_item()
        .c_barra("BARCODE123")
        .c_barra_trib("BARCODETRIB456");
    assert_eq!(item.c_barra.as_deref(), Some("BARCODE123"));
    assert_eq!(item.c_barra_trib.as_deref(), Some("BARCODETRIB456"));
}

#[test]
fn invoice_item_data_taxable_fields_builders() {
    let item = common::sample_item()
        .taxable_unit("KG")
        .taxable_quantity(5.0)
        .taxable_unit_price(Cents(500));
    assert_eq!(item.taxable_unit.as_deref(), Some("KG"));
    assert_eq!(item.taxable_quantity, Some(5.0));
    assert_eq!(item.taxable_unit_price, Some(Cents(500)));
}

// ── InvoiceItemData::ind_bem_movel_usado (lines 3241-3243) ──────────────

#[test]
fn invoice_item_data_ind_bem_movel_usado_builder() {
    let item = common::sample_item()
        .ind_bem_movel_usado(true);
    assert_eq!(item.ind_bem_movel_usado, Some(true));
}
