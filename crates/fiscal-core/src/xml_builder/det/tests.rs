use super::*;
use crate::newtypes::{Cents, IbgeCode, Rate, Rate4};
use crate::tax_issqn::IssqnData as TaxIssqnData;
use crate::types::{
    ArmaData, CideData, CombData, EncerranteData, GCredData, InvoiceItemData, InvoiceModel,
    IssuerData, MedData, OrigCombData, RastroData, SefazEnvironment, TaxRegime, VeicProdData,
};

fn sample_build_data() -> InvoiceBuildData {
    let issuer = IssuerData::new(
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
    );

    InvoiceBuildData {
        schema_version: crate::types::SchemaVersion::PL009,
        model: InvoiceModel::Nfe,
        series: 1,
        number: 1,
        emission_type: crate::types::EmissionType::Normal,
        environment: SefazEnvironment::Homologation,
        issued_at: chrono::Utc::now()
            .with_timezone(&chrono::FixedOffset::west_opt(3 * 3600).expect("valid offset")),
        operation_nature: "VENDA".to_string(),
        issuer,
        recipient: None,
        items: Vec::new(),
        payments: Vec::new(),
        change_amount: None,
        payment_card_details: None,
        contingency: None,
        exit_at: None,
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
        issqn_tot: None,
        cana: None,
        agropecuario: None,
        compra_gov: None,
        pag_antecipado: None,
        is_tot: None,
        ibs_cbs_tot: None,
        v_nf_tot_override: None,
        destination_indicator: None,
        ver_proc: None,
        only_ascii: false,
        calculation_method: crate::types::CalculationMethod::V2,
    }
}

fn sample_item() -> InvoiceItemData {
    InvoiceItemData::new(
        1,
        "001",
        "Gasolina Comum",
        "27101259",
        "5102",
        "LT",
        50.0,
        Cents(599),
        Cents(29950),
        "102",
        Rate(0),
        Cents(0),
        "99",
        "99",
    )
}

// ── Combustíveis ────────────────────────────────────────────────────────

#[test]
fn comb_minimal_produces_correct_xml() {
    let comb = CombData::new("210203001", "GLP", "SP");
    let xml = build_comb_xml(&comb);

    assert_eq!(
        xml,
        "<comb>\
            <cProdANP>210203001</cProdANP>\
            <descANP>GLP</descANP>\
            <UFCons>SP</UFCons>\
        </comb>"
    );
}

#[test]
fn comb_with_glp_percentages() {
    let comb = CombData::new("210203001", "GLP", "SP")
        .p_glp("60.0000")
        .p_gn_n("25.0000")
        .p_gn_i("15.0000")
        .v_part("3.50");

    let xml = build_comb_xml(&comb);

    assert_eq!(
        xml,
        "<comb>\
            <cProdANP>210203001</cProdANP>\
            <descANP>GLP</descANP>\
            <pGLP>60.0000</pGLP>\
            <pGNn>25.0000</pGNn>\
            <pGNi>15.0000</pGNi>\
            <vPart>3.50</vPart>\
            <UFCons>SP</UFCons>\
        </comb>"
    );
}

#[test]
fn comb_with_codif_and_qtemp() {
    let comb = CombData::new("320102001", "GASOLINA COMUM", "PR")
        .codif("123456789")
        .q_temp("1000.0000");

    let xml = build_comb_xml(&comb);

    assert_eq!(
        xml,
        "<comb>\
            <cProdANP>320102001</cProdANP>\
            <descANP>GASOLINA COMUM</descANP>\
            <CODIF>123456789</CODIF>\
            <qTemp>1000.0000</qTemp>\
            <UFCons>PR</UFCons>\
        </comb>"
    );
}

#[test]
fn comb_with_cide() {
    let cide = CideData::new("1000.0000", "0.0700", "70.00");
    let comb = CombData::new("320102001", "GASOLINA COMUM", "SP").cide(cide);

    let xml = build_comb_xml(&comb);

    assert_eq!(
        xml,
        "<comb>\
            <cProdANP>320102001</cProdANP>\
            <descANP>GASOLINA COMUM</descANP>\
            <UFCons>SP</UFCons>\
            <CIDE>\
                <qBCProd>1000.0000</qBCProd>\
                <vAliqProd>0.0700</vAliqProd>\
                <vCIDE>70.00</vCIDE>\
            </CIDE>\
        </comb>"
    );
}

#[test]
fn comb_with_encerrante() {
    let enc = EncerranteData::new("1", "1", "1234.567", "1284.567").n_bomba("2");
    let comb = CombData::new("320102001", "GASOLINA COMUM", "SP").encerrante(enc);

    let xml = build_comb_xml(&comb);

    assert_eq!(
        xml,
        "<comb>\
            <cProdANP>320102001</cProdANP>\
            <descANP>GASOLINA COMUM</descANP>\
            <UFCons>SP</UFCons>\
            <encerrante>\
                <nBico>1</nBico>\
                <nBomba>2</nBomba>\
                <nTanque>1</nTanque>\
                <vEncIni>1234.567</vEncIni>\
                <vEncFin>1284.567</vEncFin>\
            </encerrante>\
        </comb>"
    );
}

#[test]
fn comb_encerrante_without_bomba() {
    let enc = EncerranteData::new("3", "2", "5000.000", "5050.000");
    let comb = CombData::new("320102001", "GASOLINA COMUM", "RJ").encerrante(enc);

    let xml = build_comb_xml(&comb);

    assert_eq!(
        xml,
        "<comb>\
            <cProdANP>320102001</cProdANP>\
            <descANP>GASOLINA COMUM</descANP>\
            <UFCons>RJ</UFCons>\
            <encerrante>\
                <nBico>3</nBico>\
                <nTanque>2</nTanque>\
                <vEncIni>5000.000</vEncIni>\
                <vEncFin>5050.000</vEncFin>\
            </encerrante>\
        </comb>"
    );
}

#[test]
fn comb_with_pbio() {
    let comb = CombData::new("810102001", "OLEO DIESEL B S10", "SP").p_bio("15.0000");

    let xml = build_comb_xml(&comb);

    assert_eq!(
        xml,
        "<comb>\
            <cProdANP>810102001</cProdANP>\
            <descANP>OLEO DIESEL B S10</descANP>\
            <UFCons>SP</UFCons>\
            <pBio>15.0000</pBio>\
        </comb>"
    );
}

#[test]
fn comb_with_orig_comb_single() {
    let orig = OrigCombData::new("0", "35", "100.0000");
    let comb = CombData::new("320102001", "GASOLINA COMUM", "SP").orig_comb(vec![orig]);

    let xml = build_comb_xml(&comb);

    assert_eq!(
        xml,
        "<comb>\
            <cProdANP>320102001</cProdANP>\
            <descANP>GASOLINA COMUM</descANP>\
            <UFCons>SP</UFCons>\
            <origComb>\
                <indImport>0</indImport>\
                <cUFOrig>35</cUFOrig>\
                <pOrig>100.0000</pOrig>\
            </origComb>\
        </comb>"
    );
}

#[test]
fn comb_with_orig_comb_multiple() {
    let orig1 = OrigCombData::new("0", "35", "70.0000");
    let orig2 = OrigCombData::new("1", "99", "30.0000");
    let comb = CombData::new("320102001", "GASOLINA COMUM", "SP").orig_comb(vec![orig1, orig2]);

    let xml = build_comb_xml(&comb);

    assert_eq!(
        xml,
        "<comb>\
            <cProdANP>320102001</cProdANP>\
            <descANP>GASOLINA COMUM</descANP>\
            <UFCons>SP</UFCons>\
            <origComb>\
                <indImport>0</indImport>\
                <cUFOrig>35</cUFOrig>\
                <pOrig>70.0000</pOrig>\
            </origComb>\
            <origComb>\
                <indImport>1</indImport>\
                <cUFOrig>99</cUFOrig>\
                <pOrig>30.0000</pOrig>\
            </origComb>\
        </comb>"
    );
}

#[test]
fn comb_full_with_all_fields() {
    let cide = CideData::new("500.0000", "0.0700", "35.00");
    let enc = EncerranteData::new("1", "1", "10000.000", "10050.000").n_bomba("1");
    let orig = OrigCombData::new("0", "35", "100.0000");

    let comb = CombData::new("210203001", "GLP", "SP")
        .p_glp("60.0000")
        .p_gn_n("25.0000")
        .p_gn_i("15.0000")
        .v_part("3.50")
        .codif("999888777")
        .q_temp("500.0000")
        .cide(cide)
        .encerrante(enc)
        .p_bio("12.0000")
        .orig_comb(vec![orig]);

    let xml = build_comb_xml(&comb);

    assert_eq!(
        xml,
        "<comb>\
            <cProdANP>210203001</cProdANP>\
            <descANP>GLP</descANP>\
            <pGLP>60.0000</pGLP>\
            <pGNn>25.0000</pGNn>\
            <pGNi>15.0000</pGNi>\
            <vPart>3.50</vPart>\
            <CODIF>999888777</CODIF>\
            <qTemp>500.0000</qTemp>\
            <UFCons>SP</UFCons>\
            <CIDE>\
                <qBCProd>500.0000</qBCProd>\
                <vAliqProd>0.0700</vAliqProd>\
                <vCIDE>35.00</vCIDE>\
            </CIDE>\
            <encerrante>\
                <nBico>1</nBico>\
                <nBomba>1</nBomba>\
                <nTanque>1</nTanque>\
                <vEncIni>10000.000</vEncIni>\
                <vEncFin>10050.000</vEncFin>\
            </encerrante>\
            <pBio>12.0000</pBio>\
            <origComb>\
                <indImport>0</indImport>\
                <cUFOrig>35</cUFOrig>\
                <pOrig>100.0000</pOrig>\
            </origComb>\
        </comb>"
    );
}

#[test]
fn comb_in_det_xml() {
    let comb = CombData::new("320102001", "GASOLINA COMUM", "SP");
    let item = sample_item().comb(comb);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    // <comb> appears inside <prod>
    let prod_start = result.xml.find("<prod>").expect("<prod> must exist");
    let prod_end = result.xml.find("</prod>").expect("</prod> must exist");
    let prod_section = &result.xml[prod_start..prod_end];

    assert!(prod_section.contains("<comb>"));
    assert!(prod_section.contains("<cProdANP>320102001</cProdANP>"));
    assert!(prod_section.contains("<descANP>GASOLINA COMUM</descANP>"));
    assert!(prod_section.contains("<UFCons>SP</UFCons>"));
    assert!(prod_section.contains("</comb>"));
}

// ── ISSQN ───────────────────────────────────────────────────────────────

#[test]
fn issqn_item_produces_issqn_tag_not_icms() {
    let issqn_data = TaxIssqnData::new(10000, 500, 500, "3550308", "14.01")
        .ind_iss("1")
        .ind_incentivo("2");
    let item = sample_item().issqn(issqn_data);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    // ISSQN tag present inside <imposto>
    assert!(result.xml.contains("<ISSQN>"));
    assert!(result.xml.contains("<vBC>100.00</vBC>"));
    assert!(result.xml.contains("<vAliq>5.0000</vAliq>"));
    assert!(result.xml.contains("<vISSQN>5.00</vISSQN>"));
    assert!(result.xml.contains("<cMunFG>3550308</cMunFG>"));
    assert!(result.xml.contains("<cListServ>14.01</cListServ>"));
    assert!(result.xml.contains("<indISS>1</indISS>"));
    assert!(result.xml.contains("<indIncentivo>2</indIncentivo>"));
    assert!(result.xml.contains("</ISSQN>"));

    // ICMS should NOT be present for ISSQN items
    assert!(!result.xml.contains("<ICMS>"));
    assert!(!result.xml.contains("</ICMS>"));
    assert!(result.has_issqn);
}

#[test]
fn issqn_item_with_all_optional_fields() {
    let issqn_data = TaxIssqnData::new(20000, 300, 600, "3304557", "07.02")
        .v_deducao(1000)
        .v_outro(500)
        .v_desc_incond(200)
        .v_desc_cond(100)
        .v_iss_ret(300)
        .ind_iss("1")
        .c_servico("1234")
        .c_mun("3304557")
        .c_pais("1058")
        .n_processo("ABC123")
        .ind_incentivo("1");

    let item = sample_item().issqn(issqn_data);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<vBC>200.00</vBC>"));
    assert!(result.xml.contains("<vAliq>3.0000</vAliq>"));
    assert!(result.xml.contains("<vISSQN>6.00</vISSQN>"));
    assert!(result.xml.contains("<vDeducao>10.00</vDeducao>"));
    assert!(result.xml.contains("<vOutro>5.00</vOutro>"));
    assert!(result.xml.contains("<vDescIncond>2.00</vDescIncond>"));
    assert!(result.xml.contains("<vDescCond>1.00</vDescCond>"));
    assert!(result.xml.contains("<vISSRet>3.00</vISSRet>"));
    assert!(result.xml.contains("<cServico>1234</cServico>"));
    assert!(result.xml.contains("<cMun>3304557</cMun>"));
    assert!(result.xml.contains("<cPais>1058</cPais>"));
    assert!(result.xml.contains("<nProcesso>ABC123</nProcesso>"));
    assert!(result.xml.contains("<indIncentivo>1</indIncentivo>"));
    assert!(result.has_issqn);
}

#[test]
fn non_issqn_item_has_icms_and_no_issqn() {
    let item = sample_item();
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<ICMS"));
    assert!(!result.xml.contains("<ISSQN>"));
    assert!(!result.has_issqn);
}

// ── Declaração de Importação (DI) ──────────────────────────────────────

#[test]
fn di_minimal_with_one_adi() {
    use crate::types::{AdiData, DiData};

    let adi = AdiData::new("1", "FABRICANTE_X").n_adicao("001");
    let di = DiData::new(
        "1234567890",
        "2025-01-15",
        "Santos",
        "SP",
        "2025-01-20",
        "1",
        "1",
        "EXP001",
        vec![adi],
    );
    let xml = build_di_xml(&di);

    assert_eq!(
        xml,
        "<DI>\
            <nDI>1234567890</nDI>\
            <dDI>2025-01-15</dDI>\
            <xLocDesemb>Santos</xLocDesemb>\
            <UFDesemb>SP</UFDesemb>\
            <dDesemb>2025-01-20</dDesemb>\
            <tpViaTransp>1</tpViaTransp>\
            <tpIntermedio>1</tpIntermedio>\
            <cExportador>EXP001</cExportador>\
            <adi>\
                <nAdicao>001</nAdicao>\
                <nSeqAdic>1</nSeqAdic>\
                <cFabricante>FABRICANTE_X</cFabricante>\
            </adi>\
        </DI>"
    );
}

#[test]
fn di_with_all_optional_fields() {
    use crate::types::{AdiData, DiData};

    let adi = AdiData::new("1", "FAB_Y")
        .n_adicao("002")
        .v_desc_di(Cents(15000))
        .n_draw("20259999999");
    let di = DiData::new(
        "DI-2025-001",
        "2025-03-01",
        "Paranagua",
        "PR",
        "2025-03-05",
        "1",
        "2",
        "EXP002",
        vec![adi],
    )
    .v_afrmm(Cents(5000))
    .cnpj("12345678000199")
    .uf_terceiro("RJ");

    let xml = build_di_xml(&di);

    assert_eq!(
        xml,
        "<DI>\
            <nDI>DI-2025-001</nDI>\
            <dDI>2025-03-01</dDI>\
            <xLocDesemb>Paranagua</xLocDesemb>\
            <UFDesemb>PR</UFDesemb>\
            <dDesemb>2025-03-05</dDesemb>\
            <tpViaTransp>1</tpViaTransp>\
            <vAFRMM>50.00</vAFRMM>\
            <tpIntermedio>2</tpIntermedio>\
            <CNPJ>12345678000199</CNPJ>\
            <UFTerceiro>RJ</UFTerceiro>\
            <cExportador>EXP002</cExportador>\
            <adi>\
                <nAdicao>002</nAdicao>\
                <nSeqAdic>1</nSeqAdic>\
                <cFabricante>FAB_Y</cFabricante>\
                <vDescDI>150.00</vDescDI>\
                <nDraw>20259999999</nDraw>\
            </adi>\
        </DI>"
    );
}

#[test]
fn di_with_cpf_instead_of_cnpj() {
    use crate::types::{AdiData, DiData};

    let adi = AdiData::new("1", "FAB_Z");
    let di = DiData::new(
        "DI-CPF",
        "2025-06-01",
        "Recife",
        "PE",
        "2025-06-03",
        "7",
        "3",
        "EXP003",
        vec![adi],
    )
    .cpf("12345678901");

    let xml = build_di_xml(&di);
    assert!(xml.contains("<CPF>12345678901</CPF>"));
    assert!(!xml.contains("<CNPJ>"));
}

#[test]
fn di_with_multiple_adi() {
    use crate::types::{AdiData, DiData};

    let adi1 = AdiData::new("1", "FAB_A").n_adicao("001");
    let adi2 = AdiData::new("2", "FAB_B").n_adicao("001");
    let di = DiData::new(
        "DI-MULTI",
        "2025-01-01",
        "Santos",
        "SP",
        "2025-01-05",
        "1",
        "1",
        "EXP-M",
        vec![adi1, adi2],
    );
    let xml = build_di_xml(&di);

    // Both adi elements present
    let count = xml.matches("<adi>").count();
    assert_eq!(count, 2, "expected 2 <adi> elements, got {count}");
    assert!(xml.contains("<nSeqAdic>1</nSeqAdic>"));
    assert!(xml.contains("<nSeqAdic>2</nSeqAdic>"));
    assert!(xml.contains("<cFabricante>FAB_A</cFabricante>"));
    assert!(xml.contains("<cFabricante>FAB_B</cFabricante>"));
}

#[test]
fn di_in_det_xml_between_ind_tot_and_xped() {
    use crate::types::{AdiData, DiData};

    let adi = AdiData::new("1", "FAB").n_adicao("001");
    let di = DiData::new(
        "DI-001",
        "2025-01-15",
        "Santos",
        "SP",
        "2025-01-20",
        "1",
        "1",
        "EXP",
        vec![adi],
    );
    let item = sample_item().di(vec![di]).x_ped("PO-123");
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    let xml = &result.xml;
    let ind_tot_pos = xml.find("</indTot>").expect("</indTot> must exist");
    let di_pos = xml.find("<DI>").expect("<DI> must exist");
    let xped_pos = xml.find("<xPed>").expect("<xPed> must exist");

    assert!(di_pos > ind_tot_pos, "DI must come after indTot");
    assert!(xped_pos > di_pos, "xPed must come after DI");
}

// ── Detalhe de Exportação (detExport) ──────────────────────────────────

#[test]
fn det_export_with_n_draw_only() {
    use crate::types::DetExportData;

    let dex = DetExportData::new().n_draw("20250000001");
    let xml = build_det_export_xml(&dex);

    assert_eq!(
        xml,
        "<detExport>\
            <nDraw>20250000001</nDraw>\
        </detExport>"
    );
}

#[test]
fn det_export_with_export_ind() {
    use crate::types::DetExportData;

    let dex = DetExportData::new()
        .n_draw("20250000002")
        .n_re("123456789012")
        .ch_nfe("12345678901234567890123456789012345678901234")
        .q_export(100.5);
    let xml = build_det_export_xml(&dex);

    assert_eq!(
        xml,
        "<detExport>\
            <nDraw>20250000002</nDraw>\
            <exportInd>\
                <nRE>123456789012</nRE>\
                <chNFe>12345678901234567890123456789012345678901234</chNFe>\
                <qExport>100.5000</qExport>\
            </exportInd>\
        </detExport>"
    );
}

#[test]
fn det_export_empty() {
    use crate::types::DetExportData;

    let dex = DetExportData::new();
    let xml = build_det_export_xml(&dex);

    assert_eq!(xml, "<detExport></detExport>");
}

#[test]
fn det_export_in_det_xml_between_ind_tot_and_xped() {
    use crate::types::DetExportData;

    let dex = DetExportData::new().n_draw("20250000001");
    let item = sample_item().det_export(vec![dex]).x_ped("PO-456");
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    let xml = &result.xml;
    let ind_tot_pos = xml.find("</indTot>").expect("</indTot> must exist");
    let det_exp_pos = xml.find("<detExport>").expect("<detExport> must exist");
    let xped_pos = xml.find("<xPed>").expect("<xPed> must exist");

    assert!(
        det_exp_pos > ind_tot_pos,
        "detExport must come after indTot"
    );
    assert!(xped_pos > det_exp_pos, "xPed must come after detExport");
}

// ── Imposto Devolvido (impostoDevol) ───────────────────────────────────

#[test]
fn imposto_devol_produces_correct_xml() {
    use crate::types::ImpostoDevolData;

    let devol = ImpostoDevolData::new(Rate(10000), Cents(5000));
    let item = sample_item().imposto_devol(devol);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains(
        "<impostoDevol>\
            <pDevol>100.00</pDevol>\
            <IPI>\
                <vIPIDevol>50.00</vIPIDevol>\
            </IPI>\
        </impostoDevol>"
    ));
    assert_eq!(result.v_ipi_devol, 5000);
}

#[test]
fn imposto_devol_50_percent() {
    use crate::types::ImpostoDevolData;

    let devol = ImpostoDevolData::new(Rate(5000), Cents(2500));
    let item = sample_item().imposto_devol(devol);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<pDevol>50.00</pDevol>"));
    assert!(result.xml.contains("<vIPIDevol>25.00</vIPIDevol>"));
    assert_eq!(result.v_ipi_devol, 2500);
}

#[test]
fn imposto_devol_after_imposto_before_inf_ad_prod() {
    use crate::types::ImpostoDevolData;

    let devol = ImpostoDevolData::new(Rate(10000), Cents(1000));
    let item = sample_item().imposto_devol(devol).inf_ad_prod("test info");
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    let imposto_end = result
        .xml
        .find("</imposto>")
        .expect("</imposto> must exist");
    let devol_pos = result
        .xml
        .find("<impostoDevol>")
        .expect("<impostoDevol> must exist");
    let inf_ad_pos = result
        .xml
        .find("<infAdProd>")
        .expect("<infAdProd> must exist");

    assert!(
        devol_pos > imposto_end,
        "impostoDevol must come after </imposto>"
    );
    assert!(
        inf_ad_pos > devol_pos,
        "infAdProd must come after impostoDevol"
    );
}

#[test]
fn no_imposto_devol_when_none() {
    let item = sample_item();
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(!result.xml.contains("<impostoDevol>"));
    assert_eq!(result.v_ipi_devol, 0);
}

// ── NVE (Nomenclatura de Valor Aduaneiro e Estatística) ──────────────

#[test]
fn nve_single_code_produces_correct_xml() {
    let item = sample_item().nve("AA0001");
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<NVE>AA0001</NVE>"));
    // NVE must appear after NCM
    let ncm_pos = result.xml.find("<NCM>").expect("<NCM> must exist");
    let nve_pos = result
        .xml
        .find("<NVE>AA0001</NVE>")
        .expect("<NVE> must exist");
    assert!(nve_pos > ncm_pos, "NVE must come after NCM");
}

#[test]
fn nve_multiple_codes_produces_correct_xml() {
    let item = sample_item().nve("AA0001").nve("BB0002").nve("CC0003");
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<NVE>AA0001</NVE>"));
    assert!(result.xml.contains("<NVE>BB0002</NVE>"));
    assert!(result.xml.contains("<NVE>CC0003</NVE>"));
    // Verify order: AA0001 before BB0002 before CC0003
    let pos_a = result.xml.find("<NVE>AA0001</NVE>").expect("AA0001");
    let pos_b = result.xml.find("<NVE>BB0002</NVE>").expect("BB0002");
    let pos_c = result.xml.find("<NVE>CC0003</NVE>").expect("CC0003");
    assert!(pos_a < pos_b, "NVE codes must preserve insertion order");
    assert!(pos_b < pos_c, "NVE codes must preserve insertion order");
}

#[test]
fn nve_eight_codes_is_valid() {
    let item = sample_item()
        .nve("AA0001")
        .nve("AA0002")
        .nve("AA0003")
        .nve("AA0004")
        .nve("AA0005")
        .nve("AA0006")
        .nve("AA0007")
        .nve("AA0008");
    let data = sample_build_data();
    let result = build_det(&item, &data);
    assert!(result.is_ok(), "8 NVE codes should be valid");
    let xml = result.expect("valid").xml;
    assert_eq!(xml.matches("<NVE>").count(), 8);
}

#[test]
fn nve_nine_codes_returns_error() {
    let item = sample_item()
        .nve("AA0001")
        .nve("AA0002")
        .nve("AA0003")
        .nve("AA0004")
        .nve("AA0005")
        .nve("AA0006")
        .nve("AA0007")
        .nve("AA0008")
        .nve("AA0009");
    let data = sample_build_data();
    let result = build_det(&item, &data);
    assert!(result.is_err(), "9 NVE codes should be rejected");
    let err = result.unwrap_err();
    assert_eq!(
        err,
        FiscalError::InvalidTaxData("Item 1: NVE limited to 8 entries, got 9".to_string())
    );
}

#[test]
fn nve_empty_vec_produces_no_nve_tags() {
    let item = sample_item();
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(!result.xml.contains("<NVE>"));
}

#[test]
fn nve_appears_before_cest() {
    let item = sample_item().nve("AA0001").cest("1234567");
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    let nve_pos = result
        .xml
        .find("<NVE>AA0001</NVE>")
        .expect("<NVE> must exist");
    let cest_pos = result.xml.find("<CEST>").expect("<CEST> must exist");
    assert!(nve_pos < cest_pos, "NVE must come before CEST");
}

// ── gCred (crédito presumido ICMS) ──────────────────────────────────────

#[test]
fn gcred_single_with_value_produces_correct_xml() {
    let gc = GCredData::new("SP000001", Rate4(50000)).v_cred_presumido(Cents(1500));
    let item = sample_item().g_cred(vec![gc]);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains(
        "<gCred><cCredPresumido>SP000001</cCredPresumido>\
         <pCredPresumido>5.0000</pCredPresumido>\
         <vCredPresumido>15.00</vCredPresumido></gCred>"
    ));
}

#[test]
fn gcred_without_value_omits_v_cred_presumido() {
    let gc = GCredData::new("RJ000002", Rate4(120000));
    let item = sample_item().g_cred(vec![gc]);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains(
        "<gCred>\
            <cCredPresumido>RJ000002</cCredPresumido>\
            <pCredPresumido>12.0000</pCredPresumido>\
        </gCred>"
    ));
    assert!(!result.xml.contains("<vCredPresumido>"));
}

#[test]
fn gcred_multiple_entries_up_to_four() {
    let entries = vec![
        GCredData::new("SP000001", Rate4(10000)).v_cred_presumido(Cents(100)),
        GCredData::new("SP000002", Rate4(20000)).v_cred_presumido(Cents(200)),
        GCredData::new("SP000003", Rate4(30000)).v_cred_presumido(Cents(300)),
        GCredData::new("SP000004", Rate4(40000)).v_cred_presumido(Cents(400)),
    ];
    let item = sample_item().g_cred(entries);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(
        result
            .xml
            .contains("<cCredPresumido>SP000001</cCredPresumido>")
    );
    assert!(
        result
            .xml
            .contains("<cCredPresumido>SP000002</cCredPresumido>")
    );
    assert!(
        result
            .xml
            .contains("<cCredPresumido>SP000003</cCredPresumido>")
    );
    assert!(
        result
            .xml
            .contains("<cCredPresumido>SP000004</cCredPresumido>")
    );
}

#[test]
fn gcred_truncates_at_four_entries() {
    let entries = vec![
        GCredData::new("SP000001", Rate4(10000)),
        GCredData::new("SP000002", Rate4(20000)),
        GCredData::new("SP000003", Rate4(30000)),
        GCredData::new("SP000004", Rate4(40000)),
        GCredData::new("SP000005", Rate4(50000)),
    ];
    let item = sample_item().g_cred(entries);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(
        result
            .xml
            .contains("<cCredPresumido>SP000004</cCredPresumido>")
    );
    assert!(
        !result
            .xml
            .contains("<cCredPresumido>SP000005</cCredPresumido>")
    );
}

#[test]
fn gcred_positioned_after_cbenef_before_cfop() {
    let gc = GCredData::new("MG000001", Rate4(50000)).v_cred_presumido(Cents(1000));
    let item = sample_item().c_benef("SEM CBENEF").g_cred(vec![gc]);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    let cbenef_pos = result.xml.find("<cBenef>").expect("cBenef should exist");
    let gcred_pos = result.xml.find("<gCred>").expect("gCred should exist");
    let cfop_pos = result.xml.find("<CFOP>").expect("CFOP should exist");

    assert!(gcred_pos > cbenef_pos, "gCred must come after cBenef");
    assert!(gcred_pos < cfop_pos, "gCred must come before CFOP");
}

#[test]
fn gcred_empty_vec_produces_no_gcred_tags() {
    let item = sample_item();
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(!result.xml.contains("<gCred>"));
}

// ── Helper: Normal tax regime build data ─────────────────────────────

fn normal_build_data() -> InvoiceBuildData {
    let mut data = sample_build_data();
    data.issuer.tax_regime = TaxRegime::Normal;
    data
}

fn pl010_build_data() -> InvoiceBuildData {
    let mut data = sample_build_data();
    data.schema_version = crate::types::SchemaVersion::PL010;
    data
}

// ── CSOSN variants (Simples Nacional) ────────────────────────────────

#[test]
fn csosn_101_produces_correct_xml() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "101",
        Rate(1800),
        Cents(180),
        "99",
        "99",
    )
    .icms_p_cred_sn(Rate(500))
    .icms_v_cred_icms_sn(Cents(50));
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<ICMSSN101>"));
    assert!(result.xml.contains("<CSOSN>101</CSOSN>"));
    assert!(result.xml.contains("<pCredSN>"));
    assert!(result.xml.contains("<vCredICMSSN>"));
}

#[test]
fn csosn_101_missing_p_cred_sn_returns_error() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "101",
        Rate(0),
        Cents(0),
        "99",
        "99",
    )
    .icms_v_cred_icms_sn(Cents(50));
    let data = sample_build_data();
    let result = build_det(&item, &data);
    assert!(result.is_err());
}

#[test]
fn csosn_101_missing_v_cred_icms_sn_returns_error() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "101",
        Rate(0),
        Cents(0),
        "99",
        "99",
    )
    .icms_p_cred_sn(Rate(500));
    let data = sample_build_data();
    let result = build_det(&item, &data);
    assert!(result.is_err());
}

#[test]
fn csosn_empty_defaults_to_102() {
    // When icms_cst is empty for Simples, it should default to "102"
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "",
        Rate(0),
        Cents(0),
        "99",
        "99",
    );
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<ICMSSN102>"));
    assert!(result.xml.contains("<CSOSN>102</CSOSN>"));
}

#[test]
fn csosn_103_produces_correct_xml() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "103",
        Rate(0),
        Cents(0),
        "99",
        "99",
    );
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");
    // 102, 103, 300, 400 all share the ICMSSN102 tag name
    assert!(result.xml.contains("<ICMSSN102>"));
    assert!(result.xml.contains("<CSOSN>103</CSOSN>"));
}

#[test]
fn csosn_300_produces_correct_xml() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "300",
        Rate(0),
        Cents(0),
        "99",
        "99",
    );
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");
    assert!(result.xml.contains("<ICMSSN102>"));
    assert!(result.xml.contains("<CSOSN>300</CSOSN>"));
}

#[test]
fn csosn_400_produces_correct_xml() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "400",
        Rate(0),
        Cents(0),
        "99",
        "99",
    );
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");
    assert!(result.xml.contains("<ICMSSN102>"));
    assert!(result.xml.contains("<CSOSN>400</CSOSN>"));
}

#[test]
fn csosn_201_produces_correct_xml() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "201",
        Rate(1800),
        Cents(180),
        "99",
        "99",
    )
    .icms_mod_bc_st(4)
    .icms_v_bc_st(Cents(1200))
    .icms_p_icms_st(Rate(1200))
    .icms_v_icms_st(Cents(144))
    .icms_p_mva_st(Rate(5000))
    .icms_red_bc_st(Rate(1000))
    .icms_v_bc_fcp_st(Cents(1200))
    .icms_p_fcp_st(Rate(200))
    .icms_v_fcp_st(Cents(24))
    .icms_p_cred_sn(Rate(500))
    .icms_v_cred_icms_sn(Cents(50));
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<ICMSSN201>"));
    assert!(result.xml.contains("<CSOSN>201</CSOSN>"));
    assert!(result.xml.contains("<modBCST>4</modBCST>"));
    assert!(result.xml.contains("<vBCST>12.00</vBCST>"));
    assert!(result.xml.contains("<pICMSST>"));
    assert!(result.xml.contains("<vICMSST>"));
    assert!(result.xml.contains("<pMVAST>"));
    assert!(result.xml.contains("<pRedBCST>"));
}

#[test]
fn csosn_202_produces_correct_xml() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "202",
        Rate(0),
        Cents(0),
        "99",
        "99",
    )
    .icms_mod_bc_st(4)
    .icms_v_bc_st(Cents(1200))
    .icms_p_icms_st(Rate(1200))
    .icms_v_icms_st(Cents(144))
    .icms_p_mva_st(Rate(5000))
    .icms_red_bc_st(Rate(1000))
    .icms_v_bc_fcp_st(Cents(1200))
    .icms_p_fcp_st(Rate(200))
    .icms_v_fcp_st(Cents(24));
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<ICMSSN202>"));
    assert!(result.xml.contains("<CSOSN>202</CSOSN>"));
    assert!(result.xml.contains("<modBCST>4</modBCST>"));
}

#[test]
fn csosn_203_produces_correct_xml() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "203",
        Rate(0),
        Cents(0),
        "99",
        "99",
    )
    .icms_mod_bc_st(4)
    .icms_v_bc_st(Cents(1200))
    .icms_p_icms_st(Rate(1200))
    .icms_v_icms_st(Cents(144))
    .icms_p_mva_st(Rate(5000))
    .icms_red_bc_st(Rate(1000))
    .icms_v_bc_fcp_st(Cents(1200))
    .icms_p_fcp_st(Rate(200))
    .icms_v_fcp_st(Cents(24));
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    // 202 and 203 share the ICMSSN202 tag name
    assert!(result.xml.contains("<ICMSSN202>"));
    assert!(result.xml.contains("<CSOSN>203</CSOSN>"));
    assert!(result.xml.contains("<modBCST>4</modBCST>"));
}

#[test]
fn csosn_500_produces_correct_xml() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "500",
        Rate(0),
        Cents(0),
        "99",
        "99",
    )
    .icms_v_icms_substituto(Cents(200));
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<ICMSSN500>"));
    assert!(result.xml.contains("<CSOSN>500</CSOSN>"));
    assert!(result.xml.contains("<vICMSSubstituto>"));
}

#[test]
fn csosn_900_produces_correct_xml() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "900",
        Rate(1800),
        Cents(180),
        "99",
        "99",
    )
    .icms_mod_bc(3)
    .icms_red_bc(Rate(1000))
    .icms_mod_bc_st(4)
    .icms_p_mva_st(Rate(5000))
    .icms_red_bc_st(Rate(1000))
    .icms_v_bc_st(Cents(1200))
    .icms_p_icms_st(Rate(1200))
    .icms_v_icms_st(Cents(144))
    .icms_v_bc_fcp_st(Cents(1200))
    .icms_p_fcp_st(Rate(200))
    .icms_v_fcp_st(Cents(24))
    .icms_p_cred_sn(Rate(500))
    .icms_v_cred_icms_sn(Cents(50));
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<ICMSSN900>"));
    assert!(result.xml.contains("<CSOSN>900</CSOSN>"));
    assert!(result.xml.contains("<modBC>3</modBC>"));
}

#[test]
fn csosn_unsupported_returns_error() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "999",
        Rate(0),
        Cents(0),
        "99",
        "99",
    );
    let data = sample_build_data();
    let result = build_det(&item, &data);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, FiscalError::UnsupportedIcmsCsosn(ref c) if c == "999"),
        "expected UnsupportedIcmsCsosn, got {:?}",
        err
    );
}

// ── ICMS CST variants (Normal tax regime) ────────────────────────────

#[test]
fn cst_10_produces_correct_xml() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "10",
        Rate(1800),
        Cents(180),
        "99",
        "99",
    )
    .icms_mod_bc_st(4)
    .icms_v_bc_st(Cents(1200))
    .icms_p_icms_st(Rate(1200))
    .icms_v_icms_st(Cents(144))
    .icms_v_bc_fcp(Cents(1000))
    .icms_p_fcp(Rate(200))
    .icms_v_fcp(Cents(20))
    .icms_p_mva_st(Rate(5000))
    .icms_red_bc_st(Rate(1000))
    .icms_v_bc_fcp_st(Cents(1200))
    .icms_p_fcp_st(Rate(200))
    .icms_v_fcp_st(Cents(24));
    let data = normal_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<ICMS10>"));
    assert!(result.xml.contains("<CST>10</CST>"));
    assert!(result.xml.contains("<modBCST>4</modBCST>"));
    assert!(result.xml.contains("<vBCST>12.00</vBCST>"));
    assert!(result.xml.contains("<pICMSST>"));
    assert!(result.xml.contains("<vICMSST>"));
}

#[test]
fn cst_20_produces_correct_xml() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "20",
        Rate(1800),
        Cents(180),
        "99",
        "99",
    )
    .icms_mod_bc(3)
    .icms_red_bc(Rate(2000))
    .icms_v_bc_fcp(Cents(1000))
    .icms_p_fcp(Rate(200))
    .icms_v_fcp(Cents(20))
    .icms_v_icms_deson(Cents(50))
    .icms_mot_des_icms(9)
    .icms_ind_deduz_deson("1");
    let data = normal_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<ICMS20>"));
    assert!(result.xml.contains("<CST>20</CST>"));
    assert!(result.xml.contains("<pRedBC>"));
    assert!(result.xml.contains("<vICMSDeson>"));
    assert!(result.xml.contains("<motDesICMS>9</motDesICMS>"));
    assert!(result.xml.contains("<indDeduzDeson>1</indDeduzDeson>"));
    assert!(result.ind_deduz_deson);
}

#[test]
fn cst_30_produces_correct_xml() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "30",
        Rate(0),
        Cents(0),
        "99",
        "99",
    )
    .icms_mod_bc_st(4)
    .icms_v_bc_st(Cents(1200))
    .icms_p_icms_st(Rate(1200))
    .icms_v_icms_st(Cents(144))
    .icms_p_mva_st(Rate(5000))
    .icms_red_bc_st(Rate(1000))
    .icms_v_bc_fcp_st(Cents(1200))
    .icms_p_fcp_st(Rate(200))
    .icms_v_fcp_st(Cents(24))
    .icms_v_icms_deson(Cents(50))
    .icms_mot_des_icms(9)
    .icms_ind_deduz_deson("1");
    let data = normal_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<ICMS30>"));
    assert!(result.xml.contains("<CST>30</CST>"));
    assert!(result.xml.contains("<modBCST>4</modBCST>"));
    assert!(result.xml.contains("<vICMSDeson>"));
    assert!(result.xml.contains("<motDesICMS>9</motDesICMS>"));
}

#[test]
fn cst_40_produces_correct_xml() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "40",
        Rate(0),
        Cents(0),
        "99",
        "99",
    )
    .icms_v_icms_deson(Cents(100))
    .icms_mot_des_icms(1)
    .icms_ind_deduz_deson("1");
    let data = normal_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<ICMS40>"));
    assert!(result.xml.contains("<CST>40</CST>"));
    assert!(result.xml.contains("<vICMSDeson>"));
    assert!(result.xml.contains("<motDesICMS>1</motDesICMS>"));
}

#[test]
fn cst_41_produces_correct_xml() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "41",
        Rate(0),
        Cents(0),
        "99",
        "99",
    )
    .icms_v_icms_deson(Cents(100))
    .icms_mot_des_icms(1)
    .icms_ind_deduz_deson("1");
    let data = normal_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    // 40, 41, 50 all share the ICMS40 tag name
    assert!(result.xml.contains("<ICMS40>"));
    assert!(result.xml.contains("<CST>41</CST>"));
    assert!(result.xml.contains("<vICMSDeson>"));
}

#[test]
fn cst_50_produces_correct_xml() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "50",
        Rate(0),
        Cents(0),
        "99",
        "99",
    )
    .icms_v_icms_deson(Cents(100))
    .icms_mot_des_icms(1)
    .icms_ind_deduz_deson("1");
    let data = normal_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    // 40, 41, 50 all share the ICMS40 tag name
    assert!(result.xml.contains("<ICMS40>"));
    assert!(result.xml.contains("<CST>50</CST>"));
    assert!(result.xml.contains("<vICMSDeson>"));
}

#[test]
fn cst_51_produces_correct_xml() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "51",
        Rate(1800),
        Cents(180),
        "99",
        "99",
    )
    .icms_mod_bc(3)
    .icms_red_bc(Rate(1000))
    .icms_v_bc_fcp(Cents(1000))
    .icms_p_fcp(Rate(200))
    .icms_v_fcp(Cents(20));
    let data = normal_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<ICMS51>"));
    assert!(result.xml.contains("<CST>51</CST>"));
    assert!(result.xml.contains("<modBC>3</modBC>"));
    assert!(result.xml.contains("<pRedBC>"));
    assert!(result.xml.contains("<vICMS>"));
}

#[test]
fn cst_60_produces_correct_xml() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "60",
        Rate(0),
        Cents(0),
        "99",
        "99",
    )
    .icms_v_icms_substituto(Cents(200));
    let data = normal_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<ICMS60>"));
    assert!(result.xml.contains("<CST>60</CST>"));
    assert!(result.xml.contains("<vICMSSubstituto>"));
}

#[test]
fn cst_70_produces_correct_xml() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "70",
        Rate(1800),
        Cents(180),
        "99",
        "99",
    )
    .icms_mod_bc(3)
    .icms_red_bc(Rate(2000))
    .icms_mod_bc_st(4)
    .icms_v_bc_st(Cents(1200))
    .icms_p_icms_st(Rate(1200))
    .icms_v_icms_st(Cents(144))
    .icms_v_bc_fcp(Cents(1000))
    .icms_p_fcp(Rate(200))
    .icms_v_fcp(Cents(20))
    .icms_p_mva_st(Rate(5000))
    .icms_red_bc_st(Rate(1000))
    .icms_v_bc_fcp_st(Cents(1200))
    .icms_p_fcp_st(Rate(200))
    .icms_v_fcp_st(Cents(24))
    .icms_v_icms_deson(Cents(50))
    .icms_mot_des_icms(9)
    .icms_ind_deduz_deson("1");
    let data = normal_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<ICMS70>"));
    assert!(result.xml.contains("<CST>70</CST>"));
    assert!(result.xml.contains("<pRedBC>"));
    assert!(result.xml.contains("<modBCST>4</modBCST>"));
    assert!(result.xml.contains("<vICMSDeson>"));
}

#[test]
fn cst_90_produces_correct_xml() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "90",
        Rate(1800),
        Cents(180),
        "99",
        "99",
    )
    .icms_mod_bc(3)
    .icms_red_bc(Rate(1000))
    .icms_v_bc_fcp(Cents(1000))
    .icms_p_fcp(Rate(200))
    .icms_v_fcp(Cents(20))
    .icms_mod_bc_st(4)
    .icms_p_mva_st(Rate(5000))
    .icms_red_bc_st(Rate(1000))
    .icms_v_bc_st(Cents(1200))
    .icms_p_icms_st(Rate(1200))
    .icms_v_icms_st(Cents(144))
    .icms_v_bc_fcp_st(Cents(1200))
    .icms_p_fcp_st(Rate(200))
    .icms_v_fcp_st(Cents(24))
    .icms_v_icms_deson(Cents(50))
    .icms_mot_des_icms(9)
    .icms_ind_deduz_deson("1");
    let data = normal_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<ICMS90>"));
    assert!(result.xml.contains("<CST>90</CST>"));
    assert!(result.xml.contains("<modBC>3</modBC>"));
    assert!(result.xml.contains("<modBCST>4</modBCST>"));
    assert!(result.xml.contains("<vICMSDeson>"));
}

#[test]
fn cst_unsupported_returns_error() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "99",
        Rate(0),
        Cents(0),
        "99",
        "99",
    );
    let data = normal_build_data();
    let result = build_det(&item, &data);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, FiscalError::UnsupportedIcmsCst(ref c) if c == "99"),
        "expected UnsupportedIcmsCst, got {:?}",
        err
    );
}

// ── IPI (optional) ───────────────────────────────────────────────────

#[test]
fn ipi_produces_correct_xml() {
    let item = sample_item()
        .ipi_cst("50")
        .ipi_c_enq("999")
        .ipi_v_bc(Cents(10000))
        .ipi_p_ipi(Rate(500))
        .ipi_v_ipi(Cents(500));
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<IPI>"));
    assert!(result.xml.contains("<CST>50</CST>"));
    assert!(result.xml.contains("<cEnq>999</cEnq>"));
    assert!(result.xml.contains("<vIPI>5.00</vIPI>"));
    assert_eq!(result.v_ipi, 500);
}

#[test]
fn ipi_default_c_enq_when_missing() {
    let item = sample_item()
        .ipi_cst("50")
        .ipi_v_bc(Cents(10000))
        .ipi_p_ipi(Rate(500))
        .ipi_v_ipi(Cents(500));
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<cEnq>999</cEnq>"));
}

#[test]
fn ipi_with_quantity_based() {
    let item = sample_item()
        .ipi_cst("50")
        .ipi_q_unid(100)
        .ipi_v_unid(50)
        .ipi_v_ipi(Cents(5000));
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<IPI>"));
    assert_eq!(result.v_ipi, 5000);
}

#[test]
fn no_ipi_when_cst_absent() {
    let item = sample_item();
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(!result.xml.contains("<IPI>"));
    assert_eq!(result.v_ipi, 0);
}

// ── II (Import Tax) ──────────────────────────────────────────────────

#[test]
fn ii_produces_correct_xml() {
    let item = sample_item()
        .ii_v_bc(Cents(50000))
        .ii_v_desp_adu(Cents(5000))
        .ii_v_ii(Cents(10000))
        .ii_v_iof(Cents(2000));
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<II>"));
    assert!(result.xml.contains("<vBC>500.00</vBC>"));
    assert!(result.xml.contains("<vDespAdu>50.00</vDespAdu>"));
    assert!(result.xml.contains("<vII>100.00</vII>"));
    assert!(result.xml.contains("<vIOF>20.00</vIOF>"));
    assert_eq!(result.v_ii, 10000);
}

#[test]
fn no_ii_when_absent() {
    let item = sample_item();
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(!result.xml.contains("<II>"));
    assert_eq!(result.v_ii, 0);
}

// ── PIS-ST / COFINS-ST ──────────────────────────────────────────────

#[test]
fn pis_st_replaces_pis_and_accumulates_when_ind_soma_1() {
    use crate::tax_pis_cofins_ipi::PisStData;
    let pis_st = PisStData::new(Cents(500))
        .v_bc(Cents(10000))
        .p_pis(Rate4(16500))
        .ind_soma_pis_st(1);
    let item = sample_item().pis_st(pis_st);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<PISST>"));
    assert!(!result.xml.contains("<PISAliq>"));
    assert_eq!(result.v_pis_st, 500);
}

#[test]
fn pis_st_does_not_accumulate_when_ind_soma_0() {
    use crate::tax_pis_cofins_ipi::PisStData;
    let pis_st = PisStData::new(Cents(500))
        .v_bc(Cents(10000))
        .p_pis(Rate4(16500))
        .ind_soma_pis_st(0);
    let item = sample_item().pis_st(pis_st);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<PISST>"));
    assert_eq!(result.v_pis_st, 0);
}

#[test]
fn cofins_st_replaces_cofins_and_accumulates_when_ind_soma_1() {
    use crate::tax_pis_cofins_ipi::CofinsStData;
    let cofins_st = CofinsStData::new(Cents(750))
        .v_bc(Cents(10000))
        .p_cofins(Rate4(76000))
        .ind_soma_cofins_st(1);
    let item = sample_item().cofins_st(cofins_st);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<COFINSST>"));
    assert!(!result.xml.contains("<COFINSAliq>"));
    assert_eq!(result.v_cofins_st, 750);
}

#[test]
fn cofins_st_does_not_accumulate_when_ind_soma_0() {
    use crate::tax_pis_cofins_ipi::CofinsStData;
    let cofins_st = CofinsStData::new(Cents(750))
        .v_bc(Cents(10000))
        .p_cofins(Rate4(76000))
        .ind_soma_cofins_st(0);
    let item = sample_item().cofins_st(cofins_st);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<COFINSST>"));
    assert_eq!(result.v_cofins_st, 0);
}

// ── IS / IBS-CBS (PL010 schema) ─────────────────────────────────────

#[test]
fn is_data_emitted_with_pl010_schema() {
    use crate::tax_is::IsData;
    let is = IsData::new("00", "001", "5.00");
    let item = sample_item().is_data(is);
    let data = pl010_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<IS>"));
}

#[test]
fn is_data_not_emitted_with_pl009_schema() {
    use crate::tax_is::IsData;
    let is = IsData::new("00", "001", "5.00");
    let item = sample_item().is_data(is);
    let data = sample_build_data(); // PL009
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(!result.xml.contains("<IS>"));
}

#[test]
fn ibs_cbs_data_emitted_with_pl010_schema() {
    use crate::tax_ibs_cbs::IbsCbsData;
    let ibs_cbs = IbsCbsData::new("00", "001");
    let item = sample_item().ibs_cbs(ibs_cbs);
    let data = pl010_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<IBSCBS>"));
}

// ── CEST with indEscala and CNPJFab ─────────────────────────────────

#[test]
fn cest_with_ind_escala_and_cnpj_fab() {
    let item = sample_item()
        .cest("1234567")
        .cest_ind_escala("S")
        .cest_cnpj_fab("12345678000199");
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<CEST>1234567</CEST>"));
    assert!(result.xml.contains("<indEscala>S</indEscala>"));
    assert!(result.xml.contains("<CNPJFab>12345678000199</CNPJFab>"));
}

// ── EXTIPI ──────────────────────────────────────────────────────────

#[test]
fn extipi_produces_correct_xml() {
    let item = sample_item().extipi("01");
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<EXTIPI>01</EXTIPI>"));
    // EXTIPI must appear after NCM and before CFOP
    let ncm_pos = result.xml.find("<NCM>").unwrap();
    let extipi_pos = result.xml.find("<EXTIPI>").unwrap();
    let cfop_pos = result.xml.find("<CFOP>").unwrap();
    assert!(extipi_pos > ncm_pos);
    assert!(extipi_pos < cfop_pos);
}

// ── nItemPed, nFCI ──────────────────────────────────────────────────

#[test]
fn n_item_ped_produces_correct_xml() {
    let item = sample_item().n_item_ped("5");
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<nItemPed>5</nItemPed>"));
}

#[test]
fn n_fci_produces_correct_xml() {
    let item = sample_item().n_fci("B01F70AF-10BF-4B1F-848C-65FF57F616FE");
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(
        result
            .xml
            .contains("<nFCI>B01F70AF-10BF-4B1F-848C-65FF57F616FE</nFCI>")
    );
}

// ── Veículo (veicProd) ──────────────────────────────────────────────

#[test]
fn veic_prod_produces_correct_xml() {
    let veic = VeicProdData::new(
        "1",
        "9BWZZZ377VT004251",
        "1",
        "PRATA",
        "100",
        "1600",
        "1050",
        "1250",
        "ABC123",
        "1",
        "MOT123",
        "1500",
        "2600",
        "2025",
        "2025",
        "M",
        "06",
        "1",
        "R",
        "1",
        "MOD001",
        "02",
        "5",
        "0",
    );
    let item = sample_item().veic_prod(veic);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<veicProd>"));
    assert!(result.xml.contains("<tpOp>1</tpOp>"));
    assert!(result.xml.contains("<chassi>9BWZZZ377VT004251</chassi>"));
    assert!(result.xml.contains("<cCor>1</cCor>"));
    assert!(result.xml.contains("<xCor>PRATA</xCor>"));
    assert!(result.xml.contains("<pot>100</pot>"));
    assert!(result.xml.contains("<cilin>1600</cilin>"));
    assert!(result.xml.contains("<pesoL>1050</pesoL>"));
    assert!(result.xml.contains("<pesoB>1250</pesoB>"));
    assert!(result.xml.contains("<nSerie>ABC123</nSerie>"));
    assert!(result.xml.contains("<tpComb>1</tpComb>"));
    assert!(result.xml.contains("<nMotor>MOT123</nMotor>"));
    assert!(result.xml.contains("<CMT>1500</CMT>"));
    assert!(result.xml.contains("<dist>2600</dist>"));
    assert!(result.xml.contains("<anoMod>2025</anoMod>"));
    assert!(result.xml.contains("<anoFab>2025</anoFab>"));
    assert!(result.xml.contains("<tpPint>M</tpPint>"));
    assert!(result.xml.contains("<tpVeic>06</tpVeic>"));
    assert!(result.xml.contains("<espVeic>1</espVeic>"));
    assert!(result.xml.contains("<VIN>R</VIN>"));
    assert!(result.xml.contains("<condVeic>1</condVeic>"));
    assert!(result.xml.contains("<cMod>MOD001</cMod>"));
    assert!(result.xml.contains("<cCorDENATRAN>02</cCorDENATRAN>"));
    assert!(result.xml.contains("<lota>5</lota>"));
    assert!(result.xml.contains("<tpRest>0</tpRest>"));
    assert!(result.xml.contains("</veicProd>"));
}

// ── Medicamento (med) ───────────────────────────────────────────────

#[test]
fn med_with_anvisa_code() {
    let med = MedData::new(Cents(5000)).c_prod_anvisa("1234567890123");
    let item = sample_item().med(med);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<med>"));
    assert!(
        result
            .xml
            .contains("<cProdANVISA>1234567890123</cProdANVISA>")
    );
    assert!(result.xml.contains("<vPMC>50.00</vPMC>"));
    assert!(result.xml.contains("</med>"));
}

#[test]
fn med_with_exemption_reason() {
    let med = MedData::new(Cents(3000)).x_motivo_isencao("Medicamento isento de registro");
    let item = sample_item().med(med);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<med>"));
    assert!(
        result
            .xml
            .contains("<xMotivoIsencao>Medicamento isento de registro</xMotivoIsencao>")
    );
    assert!(result.xml.contains("<vPMC>30.00</vPMC>"));
    assert!(!result.xml.contains("<cProdANVISA>"));
}

// ── Arma (weapon) ───────────────────────────────────────────────────

#[test]
fn arma_single_produces_correct_xml() {
    let arma = ArmaData::new("0", "SN12345", "CN6789", "Pistola Taurus");
    let item = sample_item().arma(vec![arma]);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<arma>"));
    assert!(result.xml.contains("<tpArma>0</tpArma>"));
    assert!(result.xml.contains("<nSerie>SN12345</nSerie>"));
    assert!(result.xml.contains("<nCano>CN6789</nCano>"));
    assert!(result.xml.contains("<descr>Pistola Taurus</descr>"));
    assert!(result.xml.contains("</arma>"));
}

#[test]
fn arma_multiple_produces_multiple_elements() {
    let a1 = ArmaData::new("0", "SN001", "CN001", "Arma 1");
    let a2 = ArmaData::new("1", "SN002", "CN002", "Arma 2");
    let item = sample_item().arma(vec![a1, a2]);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert_eq!(result.xml.matches("<arma>").count(), 2);
    assert!(result.xml.contains("<nSerie>SN001</nSerie>"));
    assert!(result.xml.contains("<nSerie>SN002</nSerie>"));
}

// ── nRECOPI ─────────────────────────────────────────────────────────

#[test]
fn n_recopi_produces_correct_xml() {
    let item = sample_item().n_recopi("20250000001234567890");
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(
        result
            .xml
            .contains("<nRECOPI>20250000001234567890</nRECOPI>")
    );
}

#[test]
fn n_recopi_empty_not_emitted() {
    let item = sample_item().n_recopi("");
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(!result.xml.contains("<nRECOPI>"));
}

// ── Rastro (batch tracking) ─────────────────────────────────────────

#[test]
fn rastro_single_produces_correct_xml() {
    let r = RastroData::new("LOTE001", 10.5, "2025-01-01", "2026-01-01");
    let item = sample_item().rastro(vec![r]);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<rastro>"));
    assert!(result.xml.contains("<nLote>LOTE001</nLote>"));
    assert!(result.xml.contains("<qLote>10.500</qLote>"));
    assert!(result.xml.contains("<dFab>2025-01-01</dFab>"));
    assert!(result.xml.contains("<dVal>2026-01-01</dVal>"));
    assert!(result.xml.contains("</rastro>"));
}

#[test]
fn rastro_with_c_agreg() {
    let r = RastroData::new("LOTE002", 5.0, "2025-06-01", "2026-06-01").c_agreg("AGREG001");
    let item = sample_item().rastro(vec![r]);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<cAgreg>AGREG001</cAgreg>"));
}

// ── obsItem with obsFisco ───────────────────────────────────────────

#[test]
fn obs_item_with_obs_cont_only() {
    use crate::types::{ObsField, ObsItemData};
    let obs = ObsItemData::new().obs_cont(ObsField::new("campo1", "texto1"));
    let item = sample_item().obs_item(obs);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<obsItem>"));
    assert!(result.xml.contains("<obsCont xCampo=\"campo1\">"));
    assert!(result.xml.contains("<xTexto>texto1</xTexto>"));
    assert!(!result.xml.contains("<obsFisco"));
}

#[test]
fn obs_item_with_obs_fisco() {
    use crate::types::{ObsField, ObsItemData};
    let obs = ObsItemData::new().obs_fisco(ObsField::new("campo_fisco", "texto_fisco"));
    let item = sample_item().obs_item(obs);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<obsItem>"));
    assert!(result.xml.contains("<obsFisco xCampo=\"campo_fisco\">"));
    assert!(result.xml.contains("<xTexto>texto_fisco</xTexto>"));
}

#[test]
fn obs_item_with_both_obs_cont_and_obs_fisco() {
    use crate::types::{ObsField, ObsItemData};
    let obs = ObsItemData::new()
        .obs_cont(ObsField::new("campo_cont", "texto_cont"))
        .obs_fisco(ObsField::new("campo_fisco", "texto_fisco"));
    let item = sample_item().obs_item(obs);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<obsCont xCampo=\"campo_cont\">"));
    assert!(result.xml.contains("<obsFisco xCampo=\"campo_fisco\">"));
}

// ── DFeReferenciado ─────────────────────────────────────────────────

#[test]
fn dfe_referenciado_without_n_item() {
    use crate::types::DFeReferenciadoData;
    let dfe = DFeReferenciadoData::new("12345678901234567890123456789012345678901234");
    let item = sample_item().dfe_referenciado(dfe);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<DFeReferenciado>"));
    assert!(
        result
            .xml
            .contains("<chaveAcesso>12345678901234567890123456789012345678901234</chaveAcesso>")
    );
    assert!(!result.xml.contains("<nItem>"));
}

#[test]
fn dfe_referenciado_with_n_item() {
    use crate::types::DFeReferenciadoData;
    let dfe = DFeReferenciadoData::new("12345678901234567890123456789012345678901234").n_item("3");
    let item = sample_item().dfe_referenciado(dfe);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<DFeReferenciado>"));
    assert!(result.xml.contains("<nItem>3</nItem>"));
}

// ── Homologation xProd substitution (NFC-e) ─────────────────────────

#[test]
fn nfce_homologation_substitutes_xprod_for_item_1() {
    let item = sample_item();
    let mut data = sample_build_data();
    data.model = InvoiceModel::Nfce;
    data.environment = SefazEnvironment::Homologation;
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains(HOMOLOGATION_XPROD));
}

#[test]
fn nfce_homologation_does_not_substitute_for_item_2() {
    let mut item = sample_item();
    item.item_number = 2;
    let mut data = sample_build_data();
    data.model = InvoiceModel::Nfce;
    data.environment = SefazEnvironment::Homologation;
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(!result.xml.contains(HOMOLOGATION_XPROD));
    assert!(result.xml.contains("<xProd>Gasolina Comum</xProd>"));
}

// ── v_frete, v_seg, v_desc, v_outro ─────────────────────────────────

#[test]
fn optional_value_fields_in_det_result() {
    let item = sample_item()
        .v_frete(Cents(1000))
        .v_seg(Cents(500))
        .v_desc(Cents(200))
        .v_outro(Cents(300));
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<vFrete>10.00</vFrete>"));
    assert!(result.xml.contains("<vSeg>5.00</vSeg>"));
    assert!(result.xml.contains("<vDesc>2.00</vDesc>"));
    assert!(result.xml.contains("<vOutro>3.00</vOutro>"));
    assert_eq!(result.v_frete, 1000);
    assert_eq!(result.v_seg, 500);
    assert_eq!(result.v_desc, 200);
    assert_eq!(result.v_outro, 300);
}

// ── ind_tot override ────────────────────────────────────────────────

#[test]
fn ind_tot_zero_excludes_from_total() {
    let item = sample_item().ind_tot(0);
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<indTot>0</indTot>"));
    assert_eq!(result.ind_tot, 0);
}

// ── v_tot_trib ──────────────────────────────────────────────────────

#[test]
fn v_tot_trib_propagated_to_result() {
    let item = sample_item().v_tot_trib(Cents(1234));
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert_eq!(result.v_tot_trib, 1234);
}

// ── xPed ────────────────────────────────────────────────────────────

#[test]
fn x_ped_produces_correct_xml() {
    let item = sample_item().x_ped("PEDIDO-001");
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<xPed>PEDIDO-001</xPed>"));
}

// ── infAdProd ───────────────────────────────────────────────────────

#[test]
fn inf_ad_prod_produces_correct_xml() {
    let item = sample_item().inf_ad_prod("informacao adicional do produto");
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(
        result
            .xml
            .contains("<infAdProd>informacao adicional do produto</infAdProd>")
    );
}

// ── ind_deduz_deson ─────────────────────────────────────────────────

#[test]
fn ind_deduz_deson_true_when_set_to_1() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "40",
        Rate(0),
        Cents(0),
        "99",
        "99",
    )
    .icms_v_icms_deson(Cents(100))
    .icms_mot_des_icms(1)
    .icms_ind_deduz_deson("1");
    let data = normal_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");
    assert!(result.ind_deduz_deson);
}

#[test]
fn ind_deduz_deson_false_when_not_set() {
    let item = sample_item();
    let data = sample_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");
    assert!(!result.ind_deduz_deson);
}

// ── orig override ───────────────────────────────────────────────────

#[test]
fn custom_orig_used_in_icms() {
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000),
        "00",
        Rate(1800),
        Cents(180),
        "99",
        "99",
    )
    .orig("1");
    let data = normal_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(result.xml.contains("<orig>1</orig>"));
}

// ── tpCredPresIBSZFM (PL010 only, inside <prod>) ────────────────────

#[test]
fn tp_cred_pres_ibs_zfm_emitted_with_pl010_schema() {
    let item = sample_item()
        .tp_cred_pres_ibs_zfm("1")
        .c_benef("SEM CBENEF");
    let data = pl010_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(
        result
            .xml
            .contains("<tpCredPresIBSZFM>1</tpCredPresIBSZFM>")
    );
    // Must appear after cBenef
    let cbenef_pos = result.xml.find("<cBenef>").expect("cBenef should exist");
    let tp_pos = result
        .xml
        .find("<tpCredPresIBSZFM>")
        .expect("tpCredPresIBSZFM should exist");
    assert!(
        tp_pos > cbenef_pos,
        "tpCredPresIBSZFM must come after cBenef"
    );
}

#[test]
fn tp_cred_pres_ibs_zfm_not_emitted_with_pl009_schema() {
    let item = sample_item().tp_cred_pres_ibs_zfm("1");
    let data = sample_build_data(); // PL009
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(
        !result.xml.contains("<tpCredPresIBSZFM>"),
        "tpCredPresIBSZFM must not be emitted with PL009"
    );
}

#[test]
fn tp_cred_pres_ibs_zfm_not_emitted_when_not_set() {
    let item = sample_item();
    let data = pl010_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(
        !result.xml.contains("<tpCredPresIBSZFM>"),
        "tpCredPresIBSZFM must not be emitted when not set"
    );
}

#[test]
fn tp_cred_pres_ibs_zfm_position_after_cbenef_before_gcred() {
    let gc = GCredData::new("ABC1234567", Rate4(2500)).v_cred_presumido(Cents(500));
    let item = sample_item()
        .c_benef("SEM CBENEF")
        .tp_cred_pres_ibs_zfm("2")
        .g_cred(vec![gc]);
    let data = pl010_build_data();
    let result = build_det(&item, &data).expect("build_det should succeed");

    let cbenef_pos = result.xml.find("<cBenef>").expect("cBenef should exist");
    let tp_pos = result
        .xml
        .find("<tpCredPresIBSZFM>")
        .expect("tpCredPresIBSZFM should exist");
    let gcred_pos = result.xml.find("<gCred>").expect("gCred should exist");
    assert!(
        tp_pos > cbenef_pos && tp_pos < gcred_pos,
        "tpCredPresIBSZFM must be between cBenef and gCred"
    );
}

// ── vItem (PL010 only, inside <det>) ─────────────────────────────────

#[test]
fn v_item_emitted_with_pl010_and_ibs_cbs() {
    use crate::tax_ibs_cbs::IbsCbsData;
    let ibs_cbs = IbsCbsData::new("00", "001");
    let item = sample_item().ibs_cbs(ibs_cbs);
    let mut data = pl010_build_data();
    // At least one item in the invoice must have IBS/CBS
    data.items = vec![item.clone()];
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(
        result.xml.contains("<vItem>"),
        "vItem must be emitted with PL010 and IBS/CBS data"
    );
}

#[test]
fn v_item_not_emitted_with_pl009() {
    use crate::tax_ibs_cbs::IbsCbsData;
    let ibs_cbs = IbsCbsData::new("00", "001");
    let item = sample_item().ibs_cbs(ibs_cbs);
    let mut data = sample_build_data(); // PL009
    data.items = vec![item.clone()];
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(
        !result.xml.contains("<vItem>"),
        "vItem must not be emitted with PL009"
    );
}

#[test]
fn v_item_not_emitted_without_ibs_cbs() {
    let item = sample_item();
    let mut data = pl010_build_data();
    data.items = vec![item.clone()];
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(
        !result.xml.contains("<vItem>"),
        "vItem must not be emitted without any IBS/CBS data"
    );
}

#[test]
fn v_item_auto_calculated_from_values() {
    use crate::tax_ibs_cbs::IbsCbsData;
    // vProd=1000, vDesc=100, vFrete=50, vSeg=30, vOutro=20
    // Expected: 1000 - 100 + 50 + 30 + 20 = 1000 cents = 10.00
    let ibs_cbs = IbsCbsData::new("00", "001");
    let item = InvoiceItemData::new(
        1,
        "001",
        "Produto",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(1000),
        Cents(1000), // vProd
        "102",       // CSOSN for Simples Nacional
        Rate(1800),
        Cents(180),
        "99",
        "99",
    )
    .v_desc(Cents(100))
    .v_frete(Cents(50))
    .v_seg(Cents(30))
    .v_outro(Cents(20))
    .ibs_cbs(ibs_cbs);
    let mut data = pl010_build_data();
    data.items = vec![item.clone()];
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(
        result.xml.contains("<vItem>10.00</vItem>"),
        "vItem should be auto-calculated as 10.00, got xml: {}",
        result.xml
    );
}

#[test]
fn v_item_user_supplied_takes_precedence() {
    use crate::tax_ibs_cbs::IbsCbsData;
    let ibs_cbs = IbsCbsData::new("00", "001");
    let item = sample_item().ibs_cbs(ibs_cbs).v_item(Cents(9999)); // 99.99
    let mut data = pl010_build_data();
    data.items = vec![item.clone()];
    let result = build_det(&item, &data).expect("build_det should succeed");

    assert!(
        result.xml.contains("<vItem>99.99</vItem>"),
        "vItem should use user-supplied value 99.99"
    );
}

#[test]
fn v_item_position_after_obs_item_before_dfe_referenciado() {
    use crate::tax_ibs_cbs::IbsCbsData;
    use crate::types::{DFeReferenciadoData, ObsField, ObsItemData};
    let ibs_cbs = IbsCbsData::new("00", "001");
    let obs = ObsItemData::new().obs_cont(ObsField::new("campo1", "texto1"));
    let dfe = DFeReferenciadoData::new("12345678901234567890123456789012345678901234");
    let item = sample_item()
        .ibs_cbs(ibs_cbs)
        .obs_item(obs)
        .dfe_referenciado(dfe)
        .v_item(Cents(5000));
    let mut data = pl010_build_data();
    data.items = vec![item.clone()];
    let result = build_det(&item, &data).expect("build_det should succeed");

    let obs_pos = result.xml.find("<obsItem>").expect("obsItem should exist");
    let v_item_pos = result.xml.find("<vItem>").expect("vItem should exist");
    let dfe_pos = result
        .xml
        .find("<DFeReferenciado>")
        .expect("DFeReferenciado should exist");
    assert!(
        v_item_pos > obs_pos && v_item_pos < dfe_pos,
        "vItem must be between obsItem and DFeReferenciado"
    );
}

#[test]
fn v_item_emitted_for_item_without_ibs_cbs_when_another_item_has_it() {
    use crate::tax_ibs_cbs::IbsCbsData;
    // item1 has no IBS/CBS, but item2 does
    let item1 = sample_item();
    let ibs_cbs = IbsCbsData::new("00", "001");
    let item2 = InvoiceItemData::new(
        2,
        "002",
        "Produto 2",
        "27101259",
        "5102",
        "UN",
        1.0,
        Cents(500),
        Cents(500),
        "00",
        Rate(1800),
        Cents(90),
        "99",
        "99",
    )
    .ibs_cbs(ibs_cbs);
    let mut data = pl010_build_data();
    data.items = vec![item1.clone(), item2.clone()];
    // Build item1 (no ibs_cbs) — should still emit vItem because another item has it
    let result = build_det(&item1, &data).expect("build_det should succeed");

    assert!(
        result.xml.contains("<vItem>"),
        "vItem must be emitted even for items without IBS/CBS when another item has it"
    );
}
