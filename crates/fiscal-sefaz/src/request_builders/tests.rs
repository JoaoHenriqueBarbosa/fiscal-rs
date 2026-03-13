use super::*;

use fiscal_core::constants::{NFE_NAMESPACE, NFE_VERSION};
use fiscal_core::types::SefazEnvironment;

use super::event_core::{build_event_id, event_description};
use super::helpers::{extract_section, strip_xml_declaration, tax_id_xml_tag};

// Synthetic 44-digit access key for tests (all zeros is fine for XML structure tests).
const TEST_KEY: &str = "35240112345678000195550010000000011000000019";
const TEST_CNPJ: &str = "12345678000195";
const TEST_CPF: &str = "12345678901";

// ── Fix #1: cOrgao = 91 for manifestacao ────────────────────────

#[test]
fn manifesta_request_uses_c_orgao_91() {
    let xml = build_manifesta_request(
        TEST_KEY,
        "210210",
        None,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
    );
    assert!(
        xml.contains("<cOrgao>91</cOrgao>"),
        "Manifestacao must use cOrgao=91 (Ambiente Nacional), got: {xml}"
    );
}

#[test]
fn manifesta_request_confirmation_has_correct_desc() {
    let xml = build_manifesta_request(
        TEST_KEY,
        "210200",
        None,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
    );
    assert!(xml.contains("<descEvento>Confirmacao da Operacao</descEvento>"));
    assert!(xml.contains("<tpEvento>210200</tpEvento>"));
}

#[test]
fn manifesta_request_operation_not_performed_includes_justification() {
    let xml = build_manifesta_request(
        TEST_KEY,
        "210240",
        Some("Motivo teste da operacao nao realizada"),
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
    );
    assert!(xml.contains("<xJust>Motivo teste da operacao nao realizada</xJust>"));
    assert!(xml.contains("<tpEvento>210240</tpEvento>"));
}

#[test]
fn cancela_request_uses_c_orgao_from_access_key() {
    let xml = build_cancela_request(
        TEST_KEY,
        "135220000009921",
        "Erro na emissao da NF-e",
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
    );
    // First 2 digits of TEST_KEY = "35" (SP)
    assert!(
        xml.contains("<cOrgao>35</cOrgao>"),
        "Cancellation must use cOrgao from access key (35), got: {xml}"
    );
}

// ── Fix #3: CPF in inutilizacao ─────────────────────────────────

#[test]
fn inutilizacao_with_cnpj_uses_cnpj_tag() {
    let xml = build_inutilizacao_request(
        24,
        TEST_CNPJ,
        "55",
        1,
        1,
        10,
        "Pulo de numeracao",
        SefazEnvironment::Homologation,
        "SP",
    );
    assert!(
        xml.contains(&format!("<CNPJ>{TEST_CNPJ}</CNPJ>")),
        "Should use <CNPJ> tag for 14-digit tax ID"
    );
    assert!(!xml.contains("<CPF>"), "Should not contain <CPF> tag");
}

#[test]
fn inutilizacao_with_cpf_uses_cpf_tag() {
    let xml = build_inutilizacao_request(
        24,
        TEST_CPF,
        "55",
        1,
        1,
        10,
        "Pulo de numeracao",
        SefazEnvironment::Homologation,
        "MT",
    );
    assert!(
        xml.contains(&format!("<CPF>{TEST_CPF}</CPF>")),
        "Should use <CPF> tag for 11-digit tax ID"
    );
    assert!(!xml.contains("<CNPJ>"), "Should not contain <CNPJ> tag");
}

// ── Fix #4: CNPJ/CPF padding in inutilizacao ID ────────────────

#[test]
fn inutilizacao_id_pads_cnpj_to_14_digits() {
    let xml = build_inutilizacao_request(
        24,
        TEST_CNPJ,
        "55",
        1,
        1,
        10,
        "Pulo de numeracao",
        SefazEnvironment::Homologation,
        "SP",
    );
    // Expected ID: ID + cUF(35) + year(24) + padded_cnpj(14 digits) + model(55) + serie(001) + ini(000000001) + fin(000000010)
    // CNPJ "12345678000195" is already 14 digits, no padding needed
    let expected_id = format!("ID3524{TEST_CNPJ}55001000000001000000010");
    assert!(
        xml.contains(&format!("Id=\"{expected_id}\"")),
        "ID should contain padded CNPJ (14 digits), expected {expected_id}, got:\n{xml}"
    );
}

#[test]
fn inutilizacao_id_pads_cpf_to_14_digits() {
    let xml = build_inutilizacao_request(
        24,
        TEST_CPF,
        "55",
        1,
        1,
        10,
        "Pulo de numeracao",
        SefazEnvironment::Homologation,
        "MT",
    );
    // CPF "12345678901" padded to 14 = "00012345678901"
    let padded = format!("{:0>14}", TEST_CPF);
    let expected_id = format!("ID5124{padded}55001000000001000000010");
    assert!(
        xml.contains(&format!("Id=\"{expected_id}\"")),
        "ID should pad CPF to 14 digits, expected {expected_id}, got:\n{xml}"
    );
}

// ── DistDFe request ─────────────────────────────────────────────

#[test]
fn dist_dfe_request_with_ult_nsu() {
    let xml = build_dist_dfe_request("SP", TEST_CNPJ, None, None, SefazEnvironment::Homologation);
    assert!(xml.contains("<distNSU><ultNSU>000000000000000</ultNSU></distNSU>"));
    assert!(xml.contains(&format!("<CNPJ>{TEST_CNPJ}</CNPJ>")));
    assert!(xml.contains("<cUFAutor>35</cUFAutor>"));
}

#[test]
fn dist_dfe_request_with_access_key() {
    let xml = build_dist_dfe_request(
        "SP",
        TEST_CNPJ,
        None,
        Some(TEST_KEY),
        SefazEnvironment::Homologation,
    );
    assert!(xml.contains(&format!("<consChNFe><chNFe>{TEST_KEY}</chNFe></consChNFe>")));
}

#[test]
fn dist_dfe_request_with_cpf() {
    let xml = build_dist_dfe_request("SP", TEST_CPF, None, None, SefazEnvironment::Homologation);
    assert!(xml.contains(&format!("<CPF>{TEST_CPF}</CPF>")));
}

// ── Cadastro request ────────────────────────────────────────────

#[test]
fn cadastro_request_with_cnpj() {
    let xml = build_cadastro_request("SP", "CNPJ", TEST_CNPJ);
    assert!(xml.contains(&format!("<CNPJ>{TEST_CNPJ}</CNPJ>")));
    assert!(xml.contains("<UF>SP</UF>"));
    assert!(xml.contains("<xServ>CONS-CAD</xServ>"));
}

#[test]
fn cadastro_request_with_cpf() {
    let xml = build_cadastro_request("MT", "CPF", TEST_CPF);
    assert!(xml.contains(&format!("<CPF>{TEST_CPF}</CPF>")));
    assert!(xml.contains("<UF>MT</UF>"));
}

#[test]
fn cadastro_request_with_ie() {
    let xml = build_cadastro_request("SP", "IE", "123456789");
    assert!(xml.contains("<IE>123456789</IE>"));
}

// ── tax_id_xml_tag helper ───────────────────────────────────────

#[test]
fn tax_id_xml_tag_detects_cpf_and_cnpj() {
    assert_eq!(tax_id_xml_tag("12345678901"), "<CPF>12345678901</CPF>");
    assert_eq!(
        tax_id_xml_tag("12345678000195"),
        "<CNPJ>12345678000195</CNPJ>"
    );
}

// ── Event ID format ─────────────────────────────────────────────

#[test]
fn event_id_format() {
    let id = build_event_id(210210, TEST_KEY, 1);
    assert_eq!(id, format!("ID210210{TEST_KEY}01"));
}

#[test]
fn event_id_seq_padding() {
    let id = build_event_id(110111, TEST_KEY, 3);
    assert_eq!(id, format!("ID110111{TEST_KEY}03"));
}

// ── EPEC request ─────────────────────────────────────────────────

/// Sample NF-e XML for EPEC extraction tests.
fn sample_nfe_xml() -> String {
    format!(
        concat!(
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<infNFe versao="4.00" Id="NFe{key}">"#,
            r#"<ide><tpNF>1</tpNF><dhEmi>2026-03-12T10:00:00-03:00</dhEmi></ide>"#,
            r#"<emit><CNPJ>12345678000199</CNPJ><IE>123456789</IE></emit>"#,
            r#"<dest><CNPJ>98765432000188</CNPJ><UF>RJ</UF><IE>987654321</IE></dest>"#,
            r#"<total><ICMSTot><vNF>1500.00</vNF><vICMS>270.00</vICMS><vST>0.00</vST></ICMSTot></total>"#,
            r#"<infAdic/>"#,
            r#"<infRespTec><verProc>fiscal-rs 0.1.0</verProc></infRespTec>"#,
            r#"</infNFe></NFe>"#,
        ),
        key = TEST_KEY
    )
}

#[test]
fn extract_epec_data_from_nfe_xml() {
    let xml = sample_nfe_xml();
    let data = extract_epec_data(&xml, None).unwrap();

    assert_eq!(data.access_key, TEST_KEY);
    assert_eq!(data.c_orgao_autor, "35");
    assert_eq!(data.ver_aplic, "fiscal-rs 0.1.0");
    assert_eq!(data.dh_emi, "2026-03-12T10:00:00-03:00");
    assert_eq!(data.tp_nf, "1");
    assert_eq!(data.emit_ie, "123456789");
    assert_eq!(data.dest_uf, "RJ");
    assert_eq!(data.dest_id_tag, "<CNPJ>98765432000188</CNPJ>");
    assert_eq!(data.dest_ie.as_deref(), Some("987654321"));
    assert_eq!(data.v_nf, "1500.00");
    assert_eq!(data.v_icms, "270.00");
    assert_eq!(data.v_st, "0.00");
    assert_eq!(data.tax_id, "12345678000199");
}

#[test]
fn extract_epec_data_with_ver_aplic_override() {
    let xml = sample_nfe_xml();
    let data = extract_epec_data(&xml, Some("MyApp 2.0")).unwrap();
    assert_eq!(data.ver_aplic, "MyApp 2.0");
}

#[test]
fn extract_epec_data_with_cpf_dest() {
    let xml = format!(
        concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe{key}">"#,
            r#"<ide><tpNF>1</tpNF><dhEmi>2026-03-12T10:00:00-03:00</dhEmi></ide>"#,
            r#"<emit><CNPJ>12345678000199</CNPJ><IE>123456789</IE></emit>"#,
            r#"<dest><CPF>12345678909</CPF><UF>SP</UF></dest>"#,
            r#"<total><ICMSTot><vNF>100.00</vNF><vICMS>18.00</vICMS><vST>0.00</vST></ICMSTot></total>"#,
            r#"<infRespTec><verProc>test</verProc></infRespTec>"#,
            r#"</infNFe></NFe>"#,
        ),
        key = TEST_KEY
    );
    let data = extract_epec_data(&xml, None).unwrap();
    assert_eq!(data.dest_id_tag, "<CPF>12345678909</CPF>");
    assert_eq!(data.dest_ie, None);
}

#[test]
fn extract_epec_data_with_id_estrangeiro() {
    let xml = format!(
        concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe{key}">"#,
            r#"<ide><tpNF>1</tpNF><dhEmi>2026-03-12T10:00:00-03:00</dhEmi></ide>"#,
            r#"<emit><CNPJ>12345678000199</CNPJ><IE>123456789</IE></emit>"#,
            r#"<dest><idEstrangeiro>ABC123</idEstrangeiro><UF>EX</UF></dest>"#,
            r#"<total><ICMSTot><vNF>500.00</vNF><vICMS>0.00</vICMS><vST>0.00</vST></ICMSTot></total>"#,
            r#"<infRespTec><verProc>test</verProc></infRespTec>"#,
            r#"</infNFe></NFe>"#,
        ),
        key = TEST_KEY
    );
    let data = extract_epec_data(&xml, None).unwrap();
    assert_eq!(data.dest_id_tag, "<idEstrangeiro>ABC123</idEstrangeiro>");
}

#[test]
fn extract_epec_data_rejects_missing_inf_nfe() {
    let xml = "<NFe><ide/></NFe>";
    let err = extract_epec_data(xml, None).unwrap_err();
    assert!(matches!(err, fiscal_core::FiscalError::XmlParsing(_)));
}

#[test]
fn build_epec_request_structure() {
    let xml = sample_nfe_xml();
    let data = extract_epec_data(&xml, None).unwrap();
    let request = build_epec_request(&data, SefazEnvironment::Homologation);

    // Event type
    assert!(
        request.contains("<tpEvento>110140</tpEvento>"),
        "EPEC must have tpEvento=110140"
    );
    // Description
    assert!(
        request.contains("<descEvento>EPEC</descEvento>"),
        "EPEC must have descEvento=EPEC"
    );
    // cOrgao in envelope should be 91 (AN)
    assert!(
        request.contains("<cOrgao>91</cOrgao>"),
        "EPEC envelope must use cOrgao=91 (AN)"
    );
    // cOrgaoAutor in detEvento should be the issuer's UF code
    assert!(
        request.contains("<cOrgaoAutor>35</cOrgaoAutor>"),
        "EPEC detEvento must contain cOrgaoAutor from issuer UF"
    );
    // tpAutor=1
    assert!(request.contains("<tpAutor>1</tpAutor>"));
    // verAplic
    assert!(request.contains("<verAplic>fiscal-rs 0.1.0</verAplic>"));
    // dhEmi
    assert!(request.contains("<dhEmi>2026-03-12T10:00:00-03:00</dhEmi>"));
    // tpNF
    assert!(request.contains("<tpNF>1</tpNF>"));
    // Issuer IE
    assert!(request.contains("<IE>123456789</IE>"));
    // Dest section
    assert!(request.contains("<dest>"));
    assert!(request.contains("<UF>RJ</UF>"));
    assert!(request.contains("<CNPJ>98765432000188</CNPJ>"));
    assert!(
        request.contains("<IE>987654321</IE>"),
        "Dest IE should be present"
    );
    assert!(request.contains("<vNF>1500.00</vNF>"));
    assert!(request.contains("<vICMS>270.00</vICMS>"));
    assert!(request.contains("<vST>0.00</vST>"));
    assert!(request.contains("</dest>"));
    // nSeqEvento=1 always
    assert!(request.contains("<nSeqEvento>1</nSeqEvento>"));
    // envEvento wrapper
    assert!(request.contains("<envEvento"));
    assert!(request.contains("</envEvento>"));
    // Access key
    assert!(request.contains(&format!("<chNFe>{TEST_KEY}</chNFe>")));
    // Event ID
    assert!(request.contains(&format!("Id=\"ID110140{TEST_KEY}01\"")));
    // tpAmb=2 (homologation)
    assert!(request.contains("<tpAmb>2</tpAmb>"));
    // Issuer tax ID in envelope
    assert!(request.contains("<CNPJ>12345678000199</CNPJ>"));
}

#[test]
fn build_epec_request_without_dest_ie() {
    let xml = format!(
        concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe{key}">"#,
            r#"<ide><tpNF>0</tpNF><dhEmi>2026-01-01T08:00:00-03:00</dhEmi></ide>"#,
            r#"<emit><CNPJ>12345678000199</CNPJ><IE>111222333</IE></emit>"#,
            r#"<dest><CPF>12345678909</CPF><UF>MG</UF></dest>"#,
            r#"<total><ICMSTot><vNF>200.00</vNF><vICMS>36.00</vICMS><vST>5.00</vST></ICMSTot></total>"#,
            r#"<infRespTec><verProc>test</verProc></infRespTec>"#,
            r#"</infNFe></NFe>"#,
        ),
        key = TEST_KEY
    );
    let data = extract_epec_data(&xml, None).unwrap();
    let request = build_epec_request(&data, SefazEnvironment::Production);

    // Should NOT contain dest IE (CPF dest, no IE)
    // The dest section should have CPF but no IE
    assert!(request.contains("<CPF>12345678909</CPF>"));
    assert!(request.contains("<tpAmb>1</tpAmb>")); // Production
    assert!(request.contains("<tpNF>0</tpNF>")); // Entrada
    assert!(request.contains("<vST>5.00</vST>"));
}

// ── Cancelamento por substituição (110112) ────────────────────

#[test]
fn cancel_substituicao_request_structure() {
    let ref_key = "35240112345678000195650010000000021000000028";
    let xml = build_cancel_substituicao_request(
        TEST_KEY,
        ref_key,
        "135220000009921",
        "Erro na emissao da NFCe",
        "FISCAL-RS-1.0",
        1,
        SefazEnvironment::Homologation,
        "12345678000199",
    );
    assert!(xml.contains("<tpEvento>110112</tpEvento>"));
    assert!(xml.contains("<descEvento>Cancelamento por substituicao</descEvento>"));
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<tpAutor>1</tpAutor>"));
    assert!(xml.contains("<verAplic>FISCAL-RS-1.0</verAplic>"));
    assert!(xml.contains("<nProt>135220000009921</nProt>"));
    assert!(xml.contains("<xJust>Erro na emissao da NFCe</xJust>"));
    assert!(xml.contains(&format!("<chNFeRef>{ref_key}</chNFeRef>")));
    assert!(xml.contains("<cOrgao>35</cOrgao>"));
}

#[test]
fn cancel_substituicao_event_id_format() {
    let xml = build_cancel_substituicao_request(
        TEST_KEY,
        "35240112345678000195650010000000021000000028",
        "135220000009921",
        "Justificativa teste",
        "APP-1.0",
        1,
        SefazEnvironment::Homologation,
        "12345678000199",
    );
    let expected_id = format!("ID110112{TEST_KEY}01");
    assert!(xml.contains(&format!("Id=\"{expected_id}\"")));
}

// ── Ator interessado (110150) ─────────────────────────────────

#[test]
fn ator_interessado_request_with_cnpj() {
    let xml = build_ator_interessado_request(
        TEST_KEY,
        1,
        "FISCAL-RS-1.0",
        Some("12345678000199"),
        None,
        0,
        "SP",
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
    );
    assert!(xml.contains("<tpEvento>110150</tpEvento>"));
    assert!(xml.contains("<descEvento>Ator interessado na NF-e</descEvento>"));
    assert!(xml.contains("<cOrgao>91</cOrgao>"));
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<tpAutor>1</tpAutor>"));
    assert!(xml.contains("<autXML><CNPJ>12345678000199</CNPJ></autXML>"));
    assert!(xml.contains("<tpAutorizacao>0</tpAutorizacao>"));
    assert!(!xml.contains("<xCondUso>"));
}

#[test]
fn ator_interessado_request_with_cpf_and_cond_uso() {
    let xml = build_ator_interessado_request(
        TEST_KEY,
        2,
        "APP-2.0",
        None,
        Some("12345678901"),
        1,
        "SP",
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
    );
    assert!(xml.contains("<autXML><CPF>12345678901</CPF></autXML>"));
    assert!(xml.contains("<tpAutorizacao>1</tpAutorizacao>"));
    assert!(xml.contains("<xCondUso>"));
    assert!(xml.contains("transportador declarado no campo CNPJ/CPF"));
}

// ── Comprovante de entrega (110130) ───────────────────────────

#[test]
fn comprovante_entrega_request_structure() {
    let xml = build_comprovante_entrega_request(
        TEST_KEY,
        "FISCAL-RS-1.0",
        "2024-01-15T10:30:00-03:00",
        "12345678901",
        "Joao da Silva",
        Some("-23.5505"),
        Some("-46.6333"),
        "abc123hashbase64==",
        "2024-01-15T10:31:00-03:00",
        "SP",
        1,
        SefazEnvironment::Homologation,
        "12345678000199",
    );
    assert!(xml.contains("<tpEvento>110130</tpEvento>"));
    assert!(xml.contains("<descEvento>Comprovante de Entrega da NF-e</descEvento>"));
    assert!(xml.contains("<cOrgao>91</cOrgao>"));
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<tpAutor>1</tpAutor>"));
    assert!(xml.contains("<dhEntrega>2024-01-15T10:30:00-03:00</dhEntrega>"));
    assert!(xml.contains("<nDoc>12345678901</nDoc>"));
    assert!(xml.contains("<xNome>Joao da Silva</xNome>"));
    assert!(xml.contains("<latGPS>-23.5505</latGPS>"));
    assert!(xml.contains("<longGPS>-46.6333</longGPS>"));
    assert!(xml.contains("<hashComprovante>abc123hashbase64==</hashComprovante>"));
    assert!(xml.contains("<dhHashComprovante>2024-01-15T10:31:00-03:00</dhHashComprovante>"));
}

#[test]
fn comprovante_entrega_without_gps() {
    let xml = build_comprovante_entrega_request(
        TEST_KEY,
        "APP-1.0",
        "2024-01-15T10:30:00-03:00",
        "12345678901",
        "Maria Santos",
        None,
        None,
        "hashvalue==",
        "2024-01-15T10:31:00-03:00",
        "SP",
        1,
        SefazEnvironment::Homologation,
        "12345678000199",
    );
    assert!(!xml.contains("<latGPS>"));
    assert!(!xml.contains("<longGPS>"));
    assert!(xml.contains("<hashComprovante>hashvalue==</hashComprovante>"));
}

// ── Cancelamento comprovante de entrega (110131) ──────────────

#[test]
fn cancel_comprovante_entrega_request_structure() {
    let xml = build_cancel_comprovante_entrega_request(
        TEST_KEY,
        "FISCAL-RS-1.0",
        "135220000009999",
        "SP",
        1,
        SefazEnvironment::Homologation,
        "12345678000199",
    );
    assert!(xml.contains("<tpEvento>110131</tpEvento>"));
    assert!(xml.contains("<descEvento>Cancelamento Comprovante de Entrega da NF-e</descEvento>"));
    assert!(xml.contains("<cOrgao>91</cOrgao>"));
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<tpAutor>1</tpAutor>"));
    assert!(xml.contains("<verAplic>FISCAL-RS-1.0</verAplic>"));
    assert!(xml.contains("<nProtEvento>135220000009999</nProtEvento>"));
}

// ── Insucesso na entrega (110192) ─────────────────────────────

#[test]
fn insucesso_entrega_request_structure() {
    let xml = build_insucesso_entrega_request(
        TEST_KEY,
        "FISCAL-RS-1.0",
        "2024-01-15T14:00:00-03:00",
        Some(3),
        1,
        None,
        Some("-23.5505"),
        Some("-46.6333"),
        "hashinsucesso==",
        "2024-01-15T14:01:00-03:00",
        "SP",
        1,
        SefazEnvironment::Homologation,
        "12345678000199",
    );
    assert!(xml.contains("<tpEvento>110192</tpEvento>"));
    assert!(xml.contains("<descEvento>Insucesso na Entrega da NF-e</descEvento>"));
    assert!(xml.contains("<cOrgao>91</cOrgao>"));
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<dhTentativaEntrega>2024-01-15T14:00:00-03:00</dhTentativaEntrega>"));
    assert!(xml.contains("<nTentativa>3</nTentativa>"));
    assert!(xml.contains("<tpMotivo>1</tpMotivo>"));
    assert!(!xml.contains("<xJustMotivo>"));
    assert!(xml.contains("<latGPS>-23.5505</latGPS>"));
    assert!(xml.contains("<longGPS>-46.6333</longGPS>"));
    assert!(xml.contains("<hashTentativaEntrega>hashinsucesso==</hashTentativaEntrega>"));
    assert!(
        xml.contains("<dhHashTentativaEntrega>2024-01-15T14:01:00-03:00</dhHashTentativaEntrega>")
    );
}

#[test]
fn insucesso_entrega_with_reason_type_4_includes_justification() {
    let xml = build_insucesso_entrega_request(
        TEST_KEY,
        "APP-1.0",
        "2024-01-15T14:00:00-03:00",
        None,
        4,
        Some("Destinatario mudou de endereco"),
        None,
        None,
        "hashval==",
        "2024-01-15T14:01:00-03:00",
        "SP",
        1,
        SefazEnvironment::Homologation,
        "12345678000199",
    );
    assert!(xml.contains("<tpMotivo>4</tpMotivo>"));
    assert!(xml.contains("<xJustMotivo>Destinatario mudou de endereco</xJustMotivo>"));
    assert!(!xml.contains("<nTentativa>"));
    assert!(!xml.contains("<latGPS>"));
}

// ── Cancelamento insucesso entrega (110193) ───────────────────

#[test]
fn cancel_insucesso_entrega_request_structure() {
    let xml = build_cancel_insucesso_entrega_request(
        TEST_KEY,
        "FISCAL-RS-1.0",
        "135220000009888",
        "SP",
        1,
        SefazEnvironment::Homologation,
        "12345678000199",
    );
    assert!(xml.contains("<tpEvento>110193</tpEvento>"));
    assert!(xml.contains("<descEvento>Cancelamento Insucesso na Entrega da NF-e</descEvento>"));
    assert!(xml.contains("<cOrgao>91</cOrgao>"));
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<verAplic>FISCAL-RS-1.0</verAplic>"));
    assert!(xml.contains("<nProtEvento>135220000009888</nProtEvento>"));
}

// ── Prorrogação ICMS (111500/111501) ────────────────────────────

#[test]
fn prorrogacao_first_term_request_structure() {
    let items = vec![
        ProrrogacaoItem {
            num_item: 1,
            qtde: 10.0,
        },
        ProrrogacaoItem {
            num_item: 2,
            qtde: 5.5,
        },
    ];
    let xml = build_prorrogacao_request(
        TEST_KEY,
        "135220000009921",
        &items,
        false,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
    );
    assert!(
        xml.contains("<tpEvento>111500</tpEvento>"),
        "First-term prorrogacao must use tpEvento=111500"
    );
    assert!(xml.contains("<descEvento>Pedido de Prorrogacao</descEvento>"));
    assert!(xml.contains("<nProt>135220000009921</nProt>"));
    assert!(xml.contains("<itemPedido numItem=\"1\"><qtdeItem>10</qtdeItem></itemPedido>"));
    assert!(xml.contains("<itemPedido numItem=\"2\"><qtdeItem>5.5</qtdeItem></itemPedido>"));
    assert!(xml.contains("<cOrgao>35</cOrgao>"));
    assert!(xml.contains(&format!("<chNFe>{TEST_KEY}</chNFe>")));
    let expected_id = format!("ID111500{TEST_KEY}01");
    assert!(xml.contains(&format!("Id=\"{expected_id}\"")));
}

#[test]
fn prorrogacao_second_term_request_structure() {
    let items = vec![ProrrogacaoItem {
        num_item: 1,
        qtde: 3.0,
    }];
    let xml = build_prorrogacao_request(
        TEST_KEY,
        "135220000009921",
        &items,
        true,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
    );
    assert!(
        xml.contains("<tpEvento>111501</tpEvento>"),
        "Second-term prorrogacao must use tpEvento=111501"
    );
    assert!(xml.contains("<descEvento>Pedido de Prorrogacao</descEvento>"));
}

// ── Cancelamento de prorrogação ICMS (111502/111503) ────────────

#[test]
fn cancel_prorrogacao_first_term_request_structure() {
    let xml = build_cancel_prorrogacao_request(
        TEST_KEY,
        "135220000009921",
        false,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
    );
    assert!(
        xml.contains("<tpEvento>111502</tpEvento>"),
        "First-term cancel prorrogacao must use tpEvento=111502"
    );
    assert!(xml.contains("<descEvento>Cancelamento de Pedido de Prorrogacao</descEvento>"));
    let expected_id_cancelado = format!("ID111500{TEST_KEY}01");
    assert!(
        xml.contains(&format!(
            "<idPedidoCancelado>{expected_id_cancelado}</idPedidoCancelado>"
        )),
        "Must contain idPedidoCancelado referencing the original prorrogacao event"
    );
    assert!(xml.contains("<nProt>135220000009921</nProt>"));
    assert!(xml.contains("<cOrgao>35</cOrgao>"));
}

#[test]
fn cancel_prorrogacao_second_term_request_structure() {
    let xml = build_cancel_prorrogacao_request(
        TEST_KEY,
        "135220000009921",
        true,
        2,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
    );
    assert!(
        xml.contains("<tpEvento>111503</tpEvento>"),
        "Second-term cancel prorrogacao must use tpEvento=111503"
    );
    assert!(xml.contains("<descEvento>Cancelamento de Pedido de Prorrogacao</descEvento>"));
    // second term: orig_event = 111501
    let expected_id_cancelado = format!("ID111501{TEST_KEY}02");
    assert!(
        xml.contains(&format!(
            "<idPedidoCancelado>{expected_id_cancelado}</idPedidoCancelado>"
        )),
        "Must contain idPedidoCancelado referencing the 2nd-term prorrogacao event"
    );
}

// ── CSC request (admCscNFCe) ───────────────────────────────────

#[test]
fn csc_request_query_structure() {
    let xml = build_csc_request(
        SefazEnvironment::Homologation,
        1,
        "12345678000195",
        None,
        None,
    );
    assert!(xml.contains("<admCscNFCe versao=\"1.00\""));
    assert!(xml.contains(&format!("xmlns=\"{NFE_NAMESPACE}\"")));
    assert!(xml.contains("<tpAmb>2</tpAmb>"));
    assert!(xml.contains("<indOp>1</indOp>"));
    assert!(xml.contains("<raizCNPJ>12345678</raizCNPJ>"));
    assert!(!xml.contains("<dadosCsc>"));
}

#[test]
fn csc_request_new_csc_structure() {
    let xml = build_csc_request(
        SefazEnvironment::Production,
        2,
        "12345678000195",
        None,
        None,
    );
    assert!(xml.contains("<tpAmb>1</tpAmb>"));
    assert!(xml.contains("<indOp>2</indOp>"));
    assert!(xml.contains("<raizCNPJ>12345678</raizCNPJ>"));
    assert!(!xml.contains("<dadosCsc>"));
}

#[test]
fn csc_request_revoke_includes_dados_csc() {
    let xml = build_csc_request(
        SefazEnvironment::Homologation,
        3,
        "12345678000195",
        Some("000001"),
        Some("ABC123DEF456"),
    );
    assert!(xml.contains("<indOp>3</indOp>"));
    assert!(xml.contains("<dadosCsc>"));
    assert!(xml.contains("<idCsc>000001</idCsc>"));
    assert!(xml.contains("<codigoCsc>ABC123DEF456</codigoCsc>"));
    assert!(xml.contains("</dadosCsc>"));
}

#[test]
fn csc_request_raiz_cnpj_is_first_8_digits() {
    let xml = build_csc_request(
        SefazEnvironment::Homologation,
        1,
        "98765432000188",
        None,
        None,
    );
    assert!(xml.contains("<raizCNPJ>98765432</raizCNPJ>"));
}

// ── Event batch request ─────────────────────────────────────────

#[test]
fn event_batch_request_structure() {
    let events = vec![
        EventItem {
            access_key: TEST_KEY.to_string(),
            event_type: event_types::CONFIRMATION,
            seq: 1,
            tax_id: TEST_CNPJ.to_string(),
            additional_tags: String::new(),
        },
        EventItem {
            access_key: TEST_KEY.to_string(),
            event_type: event_types::AWARENESS,
            seq: 1,
            tax_id: TEST_CNPJ.to_string(),
            additional_tags: String::new(),
        },
    ];
    let xml = build_event_batch_request(
        "AN",
        &events,
        Some("202401151030001"),
        SefazEnvironment::Homologation,
    );

    assert!(xml.contains("<envEvento"));
    assert!(xml.contains("<idLote>202401151030001</idLote>"));
    // Two <evento> elements
    let evento_count = xml.matches("<evento ").count();
    assert_eq!(
        evento_count, 2,
        "Should have 2 <evento> elements, got {evento_count}"
    );
    // Both event types present
    assert!(xml.contains("<tpEvento>210200</tpEvento>"));
    assert!(xml.contains("<tpEvento>210210</tpEvento>"));
    // cOrgao=91 for AN
    assert!(xml.contains("<cOrgao>91</cOrgao>"));
    // Descriptions
    assert!(xml.contains("<descEvento>Confirmacao da Operacao</descEvento>"));
    assert!(xml.contains("<descEvento>Ciencia da Operacao</descEvento>"));
}

#[test]
fn event_batch_skips_epec() {
    let events = vec![
        EventItem {
            access_key: TEST_KEY.to_string(),
            event_type: event_types::EPEC,
            seq: 1,
            tax_id: TEST_CNPJ.to_string(),
            additional_tags: String::new(),
        },
        EventItem {
            access_key: TEST_KEY.to_string(),
            event_type: event_types::AWARENESS,
            seq: 1,
            tax_id: TEST_CNPJ.to_string(),
            additional_tags: String::new(),
        },
    ];
    let xml = build_event_batch_request("AN", &events, Some("123"), SefazEnvironment::Homologation);

    // EPEC should be skipped
    assert!(!xml.contains("<tpEvento>110140</tpEvento>"));
    // Ciencia should be present
    assert!(xml.contains("<tpEvento>210210</tpEvento>"));
    // Only one <evento> element
    let evento_count = xml.matches("<evento ").count();
    assert_eq!(
        evento_count, 1,
        "EPEC should be skipped, got {evento_count} events"
    );
}

#[test]
fn event_batch_with_additional_tags() {
    let events = vec![EventItem {
        access_key: TEST_KEY.to_string(),
        event_type: event_types::OPERATION_NOT_PERFORMED,
        seq: 1,
        tax_id: TEST_CNPJ.to_string(),
        additional_tags: "<xJust>Motivo teste</xJust>".to_string(),
    }];
    let xml = build_event_batch_request("AN", &events, Some("456"), SefazEnvironment::Homologation);
    assert!(xml.contains("<xJust>Motivo teste</xJust>"));
    assert!(xml.contains("<tpEvento>210240</tpEvento>"));
    assert!(xml.contains("<descEvento>Operacao nao Realizada</descEvento>"));
}

#[test]
#[should_panic(expected = "limited to 20")]
fn event_batch_rejects_more_than_20() {
    let events: Vec<EventItem> = (0..21)
        .map(|i| EventItem {
            access_key: TEST_KEY.to_string(),
            event_type: event_types::AWARENESS,
            seq: i + 1,
            tax_id: TEST_CNPJ.to_string(),
            additional_tags: String::new(),
        })
        .collect();
    let _ = build_event_batch_request("AN", &events, None, SefazEnvironment::Homologation);
}

#[test]
fn event_batch_with_cpf_tax_id() {
    let events = vec![EventItem {
        access_key: TEST_KEY.to_string(),
        event_type: event_types::AWARENESS,
        seq: 1,
        tax_id: TEST_CPF.to_string(),
        additional_tags: String::new(),
    }];
    let xml = build_event_batch_request("AN", &events, Some("789"), SefazEnvironment::Homologation);
    assert!(xml.contains(&format!("<CPF>{TEST_CPF}</CPF>")));
}

#[test]
fn event_batch_event_id_format() {
    let events = vec![EventItem {
        access_key: TEST_KEY.to_string(),
        event_type: event_types::CONFIRMATION,
        seq: 3,
        tax_id: TEST_CNPJ.to_string(),
        additional_tags: String::new(),
    }];
    let xml = build_event_batch_request("SP", &events, Some("111"), SefazEnvironment::Homologation);
    let expected_id = format!("ID210200{TEST_KEY}03");
    assert!(
        xml.contains(&format!("Id=\"{expected_id}\"")),
        "Event ID must follow ID{{tpEvento}}{{chNFe}}{{nSeqEvento:02}} format"
    );
    // cOrgao=35 for SP
    assert!(xml.contains("<cOrgao>35</cOrgao>"));
}

// ── Conciliação financeira (110750/110751) ────────────────────────

#[test]
fn conciliacao_request_structure() {
    let det_pag = vec![ConciliacaoDetPag {
        ind_pag: Some("0".to_string()),
        t_pag: "01".to_string(),
        x_pag: Some("Dinheiro".to_string()),
        v_pag: "150.00".to_string(),
        d_pag: "2024-06-15".to_string(),
        cnpj_pag: None,
        uf_pag: None,
        cnpj_if: None,
        t_band: None,
        c_aut: None,
        cnpj_receb: None,
        uf_receb: None,
    }];
    let xml = build_conciliacao_request(
        TEST_KEY,
        "FISCAL-RS-1.0",
        &det_pag,
        false,
        None,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
        None,
    );

    assert!(xml.contains("<tpEvento>110750</tpEvento>"));
    assert!(xml.contains("<descEvento>ECONF</descEvento>"));
    assert!(xml.contains("<verAplic>FISCAL-RS-1.0</verAplic>"));
    assert!(xml.contains("<detPag>"));
    assert!(xml.contains("<indPag>0</indPag>"));
    assert!(xml.contains("<tPag>01</tPag>"));
    assert!(xml.contains("<xPag>Dinheiro</xPag>"));
    assert!(xml.contains("<vPag>150.00</vPag>"));
    assert!(xml.contains("<dPag>2024-06-15</dPag>"));
    assert!(xml.contains("</detPag>"));
    assert!(xml.contains("<cOrgao>35</cOrgao>"));
}

#[test]
fn conciliacao_request_with_card_payment() {
    let det_pag = vec![ConciliacaoDetPag {
        ind_pag: Some("0".to_string()),
        t_pag: "03".to_string(),
        x_pag: None,
        v_pag: "250.50".to_string(),
        d_pag: "2024-06-15".to_string(),
        cnpj_pag: Some("11222333000144".to_string()),
        uf_pag: Some("SP".to_string()),
        cnpj_if: Some("99888777000166".to_string()),
        t_band: Some("01".to_string()),
        c_aut: Some("AUTH123".to_string()),
        cnpj_receb: Some("55444333000122".to_string()),
        uf_receb: Some("RJ".to_string()),
    }];
    let xml = build_conciliacao_request(
        TEST_KEY,
        "APP-2.0",
        &det_pag,
        false,
        None,
        1,
        SefazEnvironment::Production,
        TEST_CNPJ,
        None,
    );

    assert!(xml.contains("<tPag>03</tPag>"));
    assert!(xml.contains("<CNPJPag>11222333000144</CNPJPag>"));
    assert!(xml.contains("<UFPag>SP</UFPag>"));
    assert!(xml.contains("<CNPJIF>99888777000166</CNPJIF>"));
    assert!(xml.contains("<tBand>01</tBand>"));
    assert!(xml.contains("<cAut>AUTH123</cAut>"));
    assert!(xml.contains("<CNPJReceb>55444333000122</CNPJReceb>"));
    assert!(xml.contains("<UFReceb>RJ</UFReceb>"));
    assert!(xml.contains("<tpAmb>1</tpAmb>"));
}

#[test]
fn conciliacao_cancel_request_structure() {
    let xml = build_conciliacao_request(
        TEST_KEY,
        "FISCAL-RS-1.0",
        &[],
        true,
        Some("135220000009999"),
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
        None,
    );

    assert!(xml.contains("<tpEvento>110751</tpEvento>"));
    assert!(xml.contains("<descEvento>Cancelamento Conciliação Financeira</descEvento>"));
    assert!(xml.contains("<verAplic>FISCAL-RS-1.0</verAplic>"));
    assert!(xml.contains("<nProtEvento>135220000009999</nProtEvento>"));
    assert!(!xml.contains("<detPag>"));
}

#[test]
fn conciliacao_multiple_payments() {
    let det_pag = vec![
        ConciliacaoDetPag {
            ind_pag: Some("0".to_string()),
            t_pag: "01".to_string(),
            x_pag: None,
            v_pag: "100.00".to_string(),
            d_pag: "2024-06-15".to_string(),
            cnpj_pag: None,
            uf_pag: None,
            cnpj_if: None,
            t_band: None,
            c_aut: None,
            cnpj_receb: None,
            uf_receb: None,
        },
        ConciliacaoDetPag {
            ind_pag: Some("1".to_string()),
            t_pag: "03".to_string(),
            x_pag: None,
            v_pag: "200.00".to_string(),
            d_pag: "2024-07-15".to_string(),
            cnpj_pag: None,
            uf_pag: None,
            cnpj_if: None,
            t_band: None,
            c_aut: None,
            cnpj_receb: None,
            uf_receb: None,
        },
    ];
    let xml = build_conciliacao_request(
        TEST_KEY,
        "APP-1.0",
        &det_pag,
        false,
        None,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
        None,
    );

    let det_pag_count = xml.matches("<detPag>").count();
    assert_eq!(det_pag_count, 2, "Should have 2 <detPag> elements");
    assert!(xml.contains("<vPag>100.00</vPag>"));
    assert!(xml.contains("<vPag>200.00</vPag>"));
    assert!(xml.contains("<dPag>2024-06-15</dPag>"));
    assert!(xml.contains("<dPag>2024-07-15</dPag>"));
}

// ── Event type constants ────────────────────────────────────────

#[test]
fn conciliacao_event_type_values() {
    assert_eq!(event_types::CONCILIACAO, 110750);
    assert_eq!(event_types::CANCEL_CONCILIACAO, 110751);
}

// ── Download (dist_dfe by access key) ───────────────────────────

#[test]
fn download_uses_cons_ch_nfe() {
    // sefazDownload builds a distDFeInt with consChNFe
    let xml = build_dist_dfe_request(
        "SP",
        TEST_CNPJ,
        None,
        Some(TEST_KEY),
        SefazEnvironment::Homologation,
    );
    assert!(xml.contains(&format!("<consChNFe><chNFe>{TEST_KEY}</chNFe></consChNFe>")));
    assert!(xml.contains("<tpAmb>2</tpAmb>"));
    assert!(xml.contains("<cUFAutor>35</cUFAutor>"));
    assert!(xml.contains(&format!("<CNPJ>{TEST_CNPJ}</CNPJ>")));
}

// ── EPEC NFC-e status (SP only) ─────────────────────────────────

#[test]
fn epec_nfce_status_request_structure() {
    let xml = build_epec_nfce_status_request("SP", SefazEnvironment::Homologation);
    assert!(xml.contains("<consStatServ"));
    assert!(xml.contains(&format!("xmlns=\"{NFE_NAMESPACE}\"")));
    assert!(xml.contains(&format!("versao=\"{NFE_VERSION}\"")));
    assert!(xml.contains("<tpAmb>2</tpAmb>"));
    assert!(xml.contains("<cUF>35</cUF>"));
    assert!(xml.contains("<xServ>STATUS</xServ>"));
    assert!(xml.contains("</consStatServ>"));
}

#[test]
fn epec_nfce_status_request_production() {
    let xml = build_epec_nfce_status_request("SP", SefazEnvironment::Production);
    assert!(xml.contains("<tpAmb>1</tpAmb>"));
}

// ── EPEC NFC-e data extraction ──────────────────────────────────

/// Sample NFC-e XML for EPEC NFC-e extraction tests (with dest).
fn sample_nfce_xml_with_dest() -> String {
    format!(
        concat!(
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<infNFe versao="4.00" Id="NFe{key}">"#,
            r#"<ide><tpNF>1</tpNF><dhEmi>2026-03-12T10:00:00-03:00</dhEmi></ide>"#,
            r#"<emit><CNPJ>12345678000199</CNPJ><IE>123456789</IE></emit>"#,
            r#"<dest><CPF>12345678909</CPF><UF>SP</UF></dest>"#,
            r#"<total><ICMSTot><vNF>150.00</vNF><vICMS>27.00</vICMS><vST>0.00</vST></ICMSTot></total>"#,
            r#"<infRespTec><verProc>fiscal-rs 0.1.0</verProc></infRespTec>"#,
            r#"</infNFe></NFe>"#,
        ),
        key = TEST_KEY
    )
}

/// Sample NFC-e XML without dest (consumer sale).
fn sample_nfce_xml_without_dest() -> String {
    format!(
        concat!(
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<infNFe versao="4.00" Id="NFe{key}">"#,
            r#"<ide><tpNF>1</tpNF><dhEmi>2026-03-12T14:30:00-03:00</dhEmi></ide>"#,
            r#"<emit><CNPJ>12345678000199</CNPJ><IE>123456789</IE></emit>"#,
            r#"<total><ICMSTot><vNF>50.00</vNF><vICMS>9.00</vICMS><vST>0.00</vST></ICMSTot></total>"#,
            r#"<infRespTec><verProc>fiscal-rs 0.1.0</verProc></infRespTec>"#,
            r#"</infNFe></NFe>"#,
        ),
        key = TEST_KEY
    )
}

#[test]
fn extract_epec_nfce_data_with_dest() {
    let xml = sample_nfce_xml_with_dest();
    let data = extract_epec_nfce_data(&xml, None).unwrap();

    assert_eq!(data.access_key, TEST_KEY);
    assert_eq!(data.c_orgao_autor, "35");
    assert_eq!(data.ver_aplic, "fiscal-rs 0.1.0");
    assert_eq!(data.dh_emi, "2026-03-12T10:00:00-03:00");
    assert_eq!(data.tp_nf, "1");
    assert_eq!(data.emit_ie, "123456789");
    assert_eq!(data.dest_uf.as_deref(), Some("SP"));
    assert_eq!(data.dest_id_tag.as_deref(), Some("<CPF>12345678909</CPF>"));
    assert_eq!(data.dest_ie, None);
    assert_eq!(data.v_nf, "150.00");
    assert_eq!(data.v_icms, "27.00");
    assert_eq!(data.tax_id, "12345678000199");
}

#[test]
fn extract_epec_nfce_data_without_dest() {
    let xml = sample_nfce_xml_without_dest();
    let data = extract_epec_nfce_data(&xml, None).unwrap();

    assert_eq!(data.access_key, TEST_KEY);
    assert_eq!(data.dest_uf, None);
    assert_eq!(data.dest_id_tag, None);
    assert_eq!(data.dest_ie, None);
    assert_eq!(data.v_nf, "50.00");
    assert_eq!(data.v_icms, "9.00");
}

#[test]
fn extract_epec_nfce_data_with_ver_aplic_override() {
    let xml = sample_nfce_xml_with_dest();
    let data = extract_epec_nfce_data(&xml, Some("MyPOS 3.0")).unwrap();
    assert_eq!(data.ver_aplic, "MyPOS 3.0");
}

#[test]
fn extract_epec_nfce_data_rejects_missing_inf_nfe() {
    let xml = "<NFe><ide/></NFe>";
    let err = extract_epec_nfce_data(xml, None).unwrap_err();
    assert!(matches!(err, fiscal_core::FiscalError::XmlParsing(_)));
}

// ── EPEC NFC-e request building ─────────────────────────────────

#[test]
fn build_epec_nfce_request_with_dest() {
    let xml = sample_nfce_xml_with_dest();
    let data = extract_epec_nfce_data(&xml, None).unwrap();
    let request = build_epec_nfce_request(&data, SefazEnvironment::Homologation);

    // Event type
    assert!(
        request.contains("<tpEvento>110140</tpEvento>"),
        "EPEC NFC-e must have tpEvento=110140"
    );
    // Description
    assert!(
        request.contains("<descEvento>EPEC</descEvento>"),
        "EPEC NFC-e must have descEvento=EPEC"
    );
    // cOrgao in envelope should be the state code (35 for SP), NOT 91
    assert!(
        request.contains("<cOrgao>35</cOrgao>"),
        "EPEC NFC-e envelope must use cOrgao=35 (state), not 91 (AN)"
    );
    // cOrgaoAutor in detEvento
    assert!(request.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    // tpAutor=1
    assert!(request.contains("<tpAutor>1</tpAutor>"));
    // verAplic
    assert!(request.contains("<verAplic>fiscal-rs 0.1.0</verAplic>"));
    // dhEmi
    assert!(request.contains("<dhEmi>2026-03-12T10:00:00-03:00</dhEmi>"));
    // tpNF
    assert!(request.contains("<tpNF>1</tpNF>"));
    // Issuer IE
    assert!(request.contains("<IE>123456789</IE>"));
    // Dest section with CPF
    assert!(request.contains("<dest>"));
    assert!(request.contains("<UF>SP</UF>"));
    assert!(request.contains("<CPF>12345678909</CPF>"));
    assert!(request.contains("</dest>"));
    // vNF and vICMS (no vST for NFC-e EPEC)
    assert!(request.contains("<vNF>150.00</vNF>"));
    assert!(request.contains("<vICMS>27.00</vICMS>"));
    assert!(
        !request.contains("<vST>"),
        "EPEC NFC-e must NOT contain <vST>"
    );
    // nSeqEvento=1
    assert!(request.contains("<nSeqEvento>1</nSeqEvento>"));
    // envEvento wrapper
    assert!(request.contains("<envEvento"));
    assert!(request.contains("</envEvento>"));
    // Access key
    assert!(request.contains(&format!("<chNFe>{TEST_KEY}</chNFe>")));
    // Event ID
    assert!(request.contains(&format!("Id=\"ID110140{TEST_KEY}01\"")));
    // tpAmb=2 (homologation)
    assert!(request.contains("<tpAmb>2</tpAmb>"));
    // Issuer tax ID in envelope
    assert!(request.contains("<CNPJ>12345678000199</CNPJ>"));
}

#[test]
fn build_epec_nfce_request_without_dest() {
    let xml = sample_nfce_xml_without_dest();
    let data = extract_epec_nfce_data(&xml, None).unwrap();
    let request = build_epec_nfce_request(&data, SefazEnvironment::Production);

    // Event type
    assert!(request.contains("<tpEvento>110140</tpEvento>"));
    // Production
    assert!(request.contains("<tpAmb>1</tpAmb>"));
    // Should NOT contain <dest> section
    assert!(
        !request.contains("<dest>"),
        "EPEC NFC-e without recipient must NOT contain <dest>"
    );
    // Should still have vNF and vICMS
    assert!(request.contains("<vNF>50.00</vNF>"));
    assert!(request.contains("<vICMS>9.00</vICMS>"));
    // cOrgao = state code, not 91
    assert!(request.contains("<cOrgao>35</cOrgao>"));
}

#[test]
fn build_epec_nfce_request_uses_state_c_orgao_not_an() {
    let xml = sample_nfce_xml_with_dest();
    let data = extract_epec_nfce_data(&xml, None).unwrap();
    let request = build_epec_nfce_request(&data, SefazEnvironment::Homologation);

    // The key difference from regular EPEC: cOrgao = state code (35), not 91
    assert!(
        !request.contains("<cOrgao>91</cOrgao>"),
        "EPEC NFC-e must NOT use cOrgao=91 (that is for standard EPEC to AN)"
    );
    assert!(
        request.contains("<cOrgao>35</cOrgao>"),
        "EPEC NFC-e must use cOrgao from the issuer's state"
    );
}

// ── RTC event_description coverage (lines 102-117) ──────────────

#[test]
fn event_description_rtc_events() {
    assert_eq!(
        event_description(event_types::RTC_CANCELA_EVENTO),
        "Cancelamento de Evento"
    );
    assert_eq!(
        event_description(event_types::RTC_INFO_PAGTO_INTEGRAL),
        "Informação de efetivo pagamento integral para liberar crédito presumido do adquirente"
    );
    assert_eq!(
        event_description(event_types::RTC_IMPORTACAO_ZFM),
        "Importação em ALC/ZFM não convertida em isenção"
    );
    assert_eq!(
        event_description(event_types::RTC_ROUBO_PERDA_FORNECEDOR),
        "Perecimento, perda, roubo ou furto durante o transporte contratado pelo fornecedor"
    );
    assert_eq!(
        event_description(event_types::RTC_FORNECIMENTO_NAO_REALIZADO),
        "Fornecimento não realizado com pagamento antecipado"
    );
    assert_eq!(
        event_description(event_types::RTC_ATUALIZACAO_DATA_ENTREGA),
        "Atualização da Data de Previsão de Entrega"
    );
    assert_eq!(
        event_description(event_types::RTC_SOL_APROP_CRED_PRESUMIDO),
        "Solicitação de Apropriação de crédito presumido"
    );
    assert_eq!(
        event_description(event_types::RTC_DESTINO_CONSUMO_PESSOAL),
        "Destinação de item para consumo pessoal"
    );
    assert_eq!(
        event_description(event_types::RTC_ROUBO_PERDA_ADQUIRENTE),
        "Perecimento, perda, roubo ou furto durante o transporte contratado pelo adquirente"
    );
    assert_eq!(
        event_description(event_types::RTC_ACEITE_DEBITO),
        "Aceite de débito na apuração por emissão de nota de crédito"
    );
    assert_eq!(
        event_description(event_types::RTC_IMOBILIZACAO_ITEM),
        "Imobilização de Item"
    );
    assert_eq!(
        event_description(event_types::RTC_APROPRIACAO_CREDITO_COMB),
        "Solicitação de Apropriação de Crédito de Combustível"
    );
    assert_eq!(
        event_description(event_types::RTC_APROPRIACAO_CREDITO_BENS),
        "Solicitação de Apropriação de Crédito para bens e serviços que dependem de atividade do adquirente"
    );
    assert_eq!(
        event_description(event_types::RTC_MANIF_TRANSF_CRED_IBS),
        "Manifestação sobre Pedido de Transferência de Crédito de IBS em Operação de Sucessão"
    );
    assert_eq!(
        event_description(event_types::RTC_MANIF_TRANSF_CRED_CBS),
        "Manifestação sobre Pedido de Transferência de Crédito de CBS em Operação de Sucessão"
    );
}

#[test]
fn event_description_unknown_returns_empty() {
    assert_eq!(event_description(999999), "");
}

// ── Manifestacao with OPERATION_NOT_PERFORMED but no justification (line 432) ──

#[test]
fn manifesta_operation_not_performed_empty_justification() {
    let xml = build_manifesta_request(
        TEST_KEY,
        "210240",
        None,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
    );
    // With None justification, should NOT contain <xJust>
    assert!(!xml.contains("<xJust>"));
    assert!(xml.contains("<tpEvento>210240</tpEvento>"));
}

#[test]
fn manifesta_operation_not_performed_empty_string_justification() {
    let xml = build_manifesta_request(
        TEST_KEY,
        "210240",
        Some(""),
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
    );
    // Empty string justification should NOT produce <xJust>
    assert!(!xml.contains("<xJust>"));
}

// ── DistDFe with specific NSU (line 498) ────────────────────────

#[test]
fn dist_dfe_request_with_specific_nsu() {
    let xml = build_dist_dfe_request(
        "SP",
        TEST_CNPJ,
        Some("123456789012345"),
        None,
        SefazEnvironment::Homologation,
    );
    // NSU that does NOT start with '0' should use consNSU
    assert!(xml.contains("<consNSU><NSU>123456789012345</NSU></consNSU>"));
}

#[test]
fn dist_dfe_request_access_key_non_numeric_uses_cons_nsu() {
    // Non-44-digit or non-numeric "access key" treated as NSU
    let xml = build_dist_dfe_request(
        "SP",
        TEST_CNPJ,
        None,
        Some("ABC123"),
        SefazEnvironment::Homologation,
    );
    assert!(xml.contains("<consNSU><NSU>ABC123</NSU></consNSU>"));
}

// ── Cadastro with unknown search type (line 528) ────────────────

#[test]
fn cadastro_request_with_unknown_search_type() {
    let xml = build_cadastro_request("SP", "UNKNOWN", "12345");
    // Unknown type should produce empty filter
    assert!(xml.contains("<xServ>CONS-CAD</xServ>"));
    assert!(xml.contains("<UF>SP</UF>"));
    // Should not contain any filter tag
    assert!(!xml.contains("<CNPJ>"));
    assert!(!xml.contains("<CPF>"));
    assert!(!xml.contains("<IE>"));
}

// ── cancel_substituicao asserts (lines 563, 571) ────────────────

#[test]
#[should_panic(expected = "Cancellation justification (xJust) is required")]
fn cancel_substituicao_rejects_empty_justification() {
    build_cancel_substituicao_request(
        TEST_KEY,
        "35240112345678000195650010000000021000000028",
        "135220000009921",
        "",
        "APP-1.0",
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
    );
}

#[test]
#[should_panic(expected = "Reference access key (chNFeRef) is required")]
fn cancel_substituicao_rejects_empty_ref_key() {
    build_cancel_substituicao_request(
        TEST_KEY,
        "",
        "135220000009921",
        "Motivo",
        "APP-1.0",
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
    );
}

#[test]
#[should_panic(expected = "Application version (verAplic) is required")]
fn cancel_substituicao_rejects_empty_ver_aplic() {
    build_cancel_substituicao_request(
        TEST_KEY,
        "35240112345678000195650010000000021000000028",
        "135220000009921",
        "Motivo",
        "",
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
    );
}

// ── Ator interessado panic (line 634) ───────────────────────────

#[test]
#[should_panic(expected = "Either authorized_cnpj or authorized_cpf must be provided")]
fn ator_interessado_panics_without_cnpj_or_cpf() {
    build_ator_interessado_request(
        TEST_KEY,
        1,
        "APP-1.0",
        None,
        None,
        0,
        "SP",
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
    );
}

// ── extract_epec_data invalid access key length (lines 987, 989) ─

#[test]
fn extract_epec_data_rejects_short_access_key() {
    let xml = concat!(
        r#"<NFe><infNFe versao="4.00" Id="NFe12345">"#,
        r#"<ide><tpNF>1</tpNF><dhEmi>2026-03-12T10:00:00-03:00</dhEmi></ide>"#,
        r#"<emit><CNPJ>12345678000199</CNPJ><IE>123456789</IE></emit>"#,
        r#"<dest><CNPJ>98765432000188</CNPJ><UF>RJ</UF></dest>"#,
        r#"<total><ICMSTot><vNF>100.00</vNF><vICMS>18.00</vICMS><vST>0.00</vST></ICMSTot></total>"#,
        r#"<infRespTec><verProc>test</verProc></infRespTec>"#,
        r#"</infNFe></NFe>"#,
    );
    let err = extract_epec_data(xml, None).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("Invalid access key length"), "got: {msg}");
}

// ── extract_epec_data missing dest ID (lines 1020-1021) ─────────

#[test]
fn extract_epec_data_rejects_dest_without_id() {
    let xml = format!(
        concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe{key}">"#,
            r#"<ide><tpNF>1</tpNF><dhEmi>2026-03-12T10:00:00-03:00</dhEmi></ide>"#,
            r#"<emit><CNPJ>12345678000199</CNPJ><IE>123456789</IE></emit>"#,
            r#"<dest><UF>SP</UF></dest>"#,
            r#"<total><ICMSTot><vNF>100.00</vNF><vICMS>18.00</vICMS><vST>0.00</vST></ICMSTot></total>"#,
            r#"<infRespTec><verProc>test</verProc></infRespTec>"#,
            r#"</infNFe></NFe>"#,
        ),
        key = TEST_KEY,
    );
    let err = extract_epec_data(&xml, None).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("Missing CNPJ/CPF/idEstrangeiro"), "got: {msg}");
}

// ── extract_epec_nfce_data invalid access key (lines 1243, 1245) ─

#[test]
fn extract_epec_nfce_data_rejects_short_access_key() {
    let xml = concat!(
        r#"<NFe><infNFe versao="4.00" Id="NFe999">"#,
        r#"<ide><tpNF>1</tpNF><dhEmi>2026-03-12T10:00:00-03:00</dhEmi></ide>"#,
        r#"<emit><CNPJ>12345678000199</CNPJ><IE>123456789</IE></emit>"#,
        r#"<total><ICMSTot><vNF>50.00</vNF><vICMS>9.00</vICMS></ICMSTot></total>"#,
        r#"<infRespTec><verProc>test</verProc></infRespTec>"#,
        r#"</infNFe></NFe>"#,
    );
    let err = extract_epec_nfce_data(xml, None).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("Invalid access key length"), "got: {msg}");
}

// ── extract_epec_nfce_data dest with CNPJ (line 1267) ──────────

#[test]
fn extract_epec_nfce_data_dest_with_cnpj() {
    let xml = format!(
        concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe{key}">"#,
            r#"<ide><tpNF>1</tpNF><dhEmi>2026-03-12T10:00:00-03:00</dhEmi></ide>"#,
            r#"<emit><CNPJ>12345678000199</CNPJ><IE>123456789</IE></emit>"#,
            r#"<dest><CNPJ>98765432000188</CNPJ><UF>RJ</UF><IE>987654321</IE></dest>"#,
            r#"<total><ICMSTot><vNF>200.00</vNF><vICMS>36.00</vICMS></ICMSTot></total>"#,
            r#"<infRespTec><verProc>test</verProc></infRespTec>"#,
            r#"</infNFe></NFe>"#,
        ),
        key = TEST_KEY,
    );
    let data = extract_epec_nfce_data(&xml, None).unwrap();
    assert_eq!(
        data.dest_id_tag.as_deref(),
        Some("<CNPJ>98765432000188</CNPJ>")
    );
    assert_eq!(data.dest_ie.as_deref(), Some("987654321"));
}

// ── extract_epec_nfce_data dest with idEstrangeiro (lines 1271-1272) ─

#[test]
fn extract_epec_nfce_data_dest_with_id_estrangeiro() {
    let xml = format!(
        concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe{key}">"#,
            r#"<ide><tpNF>1</tpNF><dhEmi>2026-03-12T10:00:00-03:00</dhEmi></ide>"#,
            r#"<emit><CNPJ>12345678000199</CNPJ><IE>123456789</IE></emit>"#,
            r#"<dest><idEstrangeiro>PASS123</idEstrangeiro><UF>EX</UF></dest>"#,
            r#"<total><ICMSTot><vNF>300.00</vNF><vICMS>0.00</vICMS></ICMSTot></total>"#,
            r#"<infRespTec><verProc>test</verProc></infRespTec>"#,
            r#"</infNFe></NFe>"#,
        ),
        key = TEST_KEY,
    );
    let data = extract_epec_nfce_data(&xml, None).unwrap();
    assert_eq!(
        data.dest_id_tag.as_deref(),
        Some("<idEstrangeiro>PASS123</idEstrangeiro>")
    );
    assert_eq!(data.dest_uf.as_deref(), Some("EX"));
}

// ── build_epec_nfce_request with dest_ie present (line 1343) ────

#[test]
fn build_epec_nfce_request_with_dest_ie() {
    let data = EpecNfceData {
        access_key: TEST_KEY.to_string(),
        c_orgao_autor: "35".to_string(),
        ver_aplic: "test".to_string(),
        dh_emi: "2026-03-12T10:00:00-03:00".to_string(),
        tp_nf: "1".to_string(),
        emit_ie: "123456789".to_string(),
        dest_uf: Some("RJ".to_string()),
        dest_id_tag: Some("<CNPJ>98765432000188</CNPJ>".to_string()),
        dest_ie: Some("987654321".to_string()),
        v_nf: "1500.00".to_string(),
        v_icms: "270.00".to_string(),
        tax_id: "12345678000199".to_string(),
    };
    let request = build_epec_nfce_request(&data, SefazEnvironment::Homologation);
    assert!(request.contains("<IE>987654321</IE>"));
    assert!(
        request.contains("<dest><UF>RJ</UF><CNPJ>98765432000188</CNPJ><IE>987654321</IE></dest>")
    );
}

// ── CSC request with short CNPJ (line 1502) ─────────────────────

#[test]
fn csc_request_with_short_cnpj() {
    let xml = build_csc_request(SefazEnvironment::Homologation, 1, "12345", None, None);
    // When CNPJ < 8 digits, raizCNPJ = entire string
    assert!(xml.contains("<raizCNPJ>12345</raizCNPJ>"));
}

// ── Event batch with lot_id=None (lines 1628-1631) ──────────────

#[test]
fn event_batch_request_auto_lot_id() {
    let events = vec![EventItem {
        access_key: TEST_KEY.to_string(),
        event_type: event_types::AWARENESS,
        seq: 1,
        tax_id: TEST_CNPJ.to_string(),
        additional_tags: String::new(),
    }];
    let xml = build_event_batch_request("AN", &events, None, SefazEnvironment::Homologation);
    // Should have a numeric auto-generated lot ID (timestamp in millis)
    assert!(xml.contains("<idLote>"));
    assert!(xml.contains("</idLote>"));
    // The lot ID should be numeric (system time millis)
    let lot_start = xml.find("<idLote>").unwrap() + 8;
    let lot_end = xml[lot_start..].find("</idLote>").unwrap() + lot_start;
    let lot_value = &xml[lot_start..lot_end];
    assert!(
        lot_value.chars().all(|c| c.is_ascii_digit()),
        "Auto-generated lot ID should be numeric, got: {lot_value}"
    );
}

// ── extract_section partial tag match (line 1773) ───────────────

#[test]
fn extract_section_rejects_partial_tag_match() {
    let xml = "<emitter>data</emitter>";
    // Looking for <emit> should NOT match <emitter>
    let result = extract_section(xml, "emit");
    assert!(
        result.is_none(),
        "extract_section should not match <emitter> when looking for <emit>"
    );
}

#[test]
fn extract_section_matches_exact_tag() {
    let xml = "<emit>data</emit>";
    let result = extract_section(xml, "emit");
    assert_eq!(result.as_deref(), Some("<emit>data</emit>"));
}

#[test]
fn extract_section_matches_tag_with_attributes() {
    let xml = r#"<emit versao="4.00">data</emit>"#;
    let result = extract_section(xml, "emit");
    assert_eq!(
        result.as_deref(),
        Some(r#"<emit versao="4.00">data</emit>"#)
    );
}

// ── strip_xml_declaration (lines 2508-2510) ─────────────────────

#[test]
fn strip_xml_declaration_removes_declaration() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?><root>data</root>"#;
    let result = strip_xml_declaration(xml);
    assert_eq!(result, "<root>data</root>");
}

#[test]
fn strip_xml_declaration_no_declaration() {
    let xml = "<root>data</root>";
    let result = strip_xml_declaration(xml);
    assert_eq!(result, "<root>data</root>");
}

#[test]
fn strip_xml_declaration_with_leading_whitespace() {
    let xml = r#"<?xml version="1.0"?>  <root>data</root>"#;
    let result = strip_xml_declaration(xml);
    assert_eq!(result, "<root>data</root>");
}

// ── RtcItem::new and builder methods (lines 1814, 1830-1847) ────

#[test]
fn rtc_item_new_defaults() {
    let item = RtcItem::new(1, 10.50, 5.25);
    assert_eq!(item.item, 1);
    assert!((item.v_ibs - 10.50).abs() < f64::EPSILON);
    assert!((item.v_cbs - 5.25).abs() < f64::EPSILON);
    assert!(item.quantidade.is_none());
    assert!(item.unidade.is_none());
    assert!(item.chave.is_none());
    assert!(item.n_item.is_none());
    assert!(item.g_controle_estoque_v_ibs.is_none());
    assert!(item.g_controle_estoque_v_cbs.is_none());
    assert!(item.v_cred_ibs.is_none());
    assert!(item.v_cred_cbs.is_none());
}

#[test]
fn rtc_item_builder_methods() {
    let item = RtcItem::new(1, 10.0, 5.0)
        .quantidade(3.5)
        .unidade("UN")
        .chave("35240112345678000195550010000000011000000019")
        .n_item(2);
    assert_eq!(item.quantidade, Some(3.5));
    assert_eq!(item.unidade.as_deref(), Some("UN"));
    assert_eq!(
        item.chave.as_deref(),
        Some("35240112345678000195550010000000011000000019")
    );
    assert_eq!(item.n_item, Some(2));
}

// ── RTC: build_rtc_info_pagto_integral (lines 1876-1899) ────────

#[test]
fn rtc_info_pagto_integral_structure() {
    let xml = build_rtc_info_pagto_integral(
        TEST_KEY,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
        "SP",
        "FISCAL-RS-1.0",
        None,
    );
    assert!(xml.contains("<tpEvento>112110</tpEvento>"));
    assert!(xml.contains("<descEvento>Informação de efetivo pagamento integral para liberar crédito presumido do adquirente</descEvento>"));
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<tpAutor>1</tpAutor>"));
    assert!(xml.contains("<verAplic>FISCAL-RS-1.0</verAplic>"));
    assert!(xml.contains("<indQuitacao>1</indQuitacao>"));
}

// ── RTC: build_rtc_sol_aprop_cred_presumido (lines 1904-1951) ───

#[test]
fn rtc_sol_aprop_cred_presumido_structure() {
    let itens = vec![RtcCredPresItem {
        item: 1,
        v_bc: 1000.00,
        g_ibs: Some(RtcCredPresSub {
            c_cred_pres: "001".to_string(),
            p_cred_pres: 0.0525,
            v_cred_pres: 52.50,
        }),
        g_cbs: Some(RtcCredPresSub {
            c_cred_pres: "002".to_string(),
            p_cred_pres: 0.0312,
            v_cred_pres: 31.20,
        }),
    }];
    let xml = build_rtc_sol_aprop_cred_presumido(
        TEST_KEY,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
        "SP",
        "APP-1.0",
        &itens,
        None,
    );
    assert!(xml.contains("<tpEvento>211110</tpEvento>"));
    assert!(
        xml.contains("<descEvento>Solicitação de Apropriação de crédito presumido</descEvento>")
    );
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<tpAutor>2</tpAutor>"));
    assert!(xml.contains("<verAplic>APP-1.0</verAplic>"));
    assert!(xml.contains("<gCredPres nItem=\"1\">"));
    assert!(xml.contains("<vBC>1000.00</vBC>"));
    assert!(xml.contains("<gIBS><cCredPres>001</cCredPres><pCredPres>0.0525</pCredPres><vCredPres>52.50</vCredPres></gIBS>"));
    assert!(xml.contains("<gCBS><cCredPres>002</cCredPres><pCredPres>0.0312</pCredPres><vCredPres>31.20</vCredPres></gCBS>"));
    assert!(xml.contains("</gCredPres>"));
}

// ── RTC: build_rtc_destino_consumo_pessoal (lines 1957-1996) ────

#[test]
fn rtc_destino_consumo_pessoal_structure() {
    let itens = vec![
        RtcItem::new(1, 15.00, 7.50)
            .quantidade(2.0)
            .unidade("UN")
            .chave(TEST_KEY)
            .n_item(1),
    ];
    let xml = build_rtc_destino_consumo_pessoal(
        TEST_KEY,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
        "SP",
        2,
        "APP-1.0",
        &itens,
        None,
    );
    assert!(xml.contains("<tpEvento>211120</tpEvento>"));
    assert!(xml.contains("<descEvento>Destinação de item para consumo pessoal</descEvento>"));
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<tpAutor>2</tpAutor>"));
    assert!(xml.contains("<gConsumo nItem=\"1\">"));
    assert!(xml.contains("<vIBS>15.00</vIBS>"));
    assert!(xml.contains("<vCBS>7.50</vCBS>"));
    assert!(xml.contains("<qConsumo>2.0000</qConsumo>"));
    assert!(xml.contains("<uConsumo>UN</uConsumo>"));
    assert!(xml.contains(&format!("<chaveAcesso>{TEST_KEY}</chaveAcesso>")));
    assert!(xml.contains("<nItem>1</nItem>"));
    assert!(xml.contains("</gConsumo>"));
}

// ── RTC: build_rtc_aceite_debito (lines 2001-2025) ──────────────

#[test]
fn rtc_aceite_debito_structure() {
    let xml = build_rtc_aceite_debito(
        TEST_KEY,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
        "SP",
        "APP-1.0",
        1,
        None,
    );
    assert!(xml.contains("<tpEvento>211128</tpEvento>"));
    assert!(xml.contains(
        "<descEvento>Aceite de débito na apuração por emissão de nota de crédito</descEvento>"
    ));
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<tpAutor>2</tpAutor>"));
    assert!(xml.contains("<verAplic>APP-1.0</verAplic>"));
    assert!(xml.contains("<indAceitacao>1</indAceitacao>"));
}

// ── RTC: build_rtc_imobilizacao_item (lines 2030-2065) ──────────

#[test]
fn rtc_imobilizacao_item_structure() {
    let itens = vec![RtcItem::new(1, 20.00, 10.00).quantidade(5.0).unidade("PC")];
    let xml = build_rtc_imobilizacao_item(
        TEST_KEY,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
        "SP",
        "APP-1.0",
        &itens,
        None,
    );
    assert!(xml.contains("<tpEvento>211130</tpEvento>"));
    assert!(xml.contains("<descEvento>Imobilização de Item</descEvento>"));
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<tpAutor>2</tpAutor>"));
    assert!(xml.contains("<gImobilizacao nItem=\"1\">"));
    assert!(xml.contains("<vIBS>20.00</vIBS>"));
    assert!(xml.contains("<vCBS>10.00</vCBS>"));
    assert!(xml.contains("<qImobilizado>5.0000</qImobilizado>"));
    assert!(xml.contains("<uImobilizado>PC</uImobilizado>"));
    assert!(xml.contains("</gImobilizacao>"));
}

// ── RTC: build_rtc_apropriacao_credito_comb (lines 2070-2105) ───

#[test]
fn rtc_apropriacao_credito_comb_structure() {
    let itens = vec![
        RtcItem::new(1, 30.00, 15.00)
            .quantidade(100.0)
            .unidade("LT"),
    ];
    let xml = build_rtc_apropriacao_credito_comb(
        TEST_KEY,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
        "SP",
        "APP-1.0",
        &itens,
        None,
    );
    assert!(xml.contains("<tpEvento>211140</tpEvento>"));
    assert!(
        xml.contains(
            "<descEvento>Solicitação de Apropriação de Crédito de Combustível</descEvento>"
        )
    );
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<tpAutor>2</tpAutor>"));
    assert!(xml.contains("<gConsumoComb nItem=\"1\">"));
    assert!(xml.contains("<vIBS>30.00</vIBS>"));
    assert!(xml.contains("<vCBS>15.00</vCBS>"));
    assert!(xml.contains("<qComb>100.0000</qComb>"));
    assert!(xml.contains("<uComb>LT</uComb>"));
    assert!(xml.contains("</gConsumoComb>"));
}

// ── RTC: build_rtc_apropriacao_credito_bens (lines 2110-2141) ───

#[test]
fn rtc_apropriacao_credito_bens_structure() {
    let mut item = RtcItem::new(1, 50.00, 25.00);
    item.v_cred_ibs = Some(45.00);
    item.v_cred_cbs = Some(22.50);
    let itens = vec![item];
    let xml = build_rtc_apropriacao_credito_bens(
        TEST_KEY,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
        "SP",
        "APP-1.0",
        &itens,
        None,
    );
    assert!(xml.contains("<tpEvento>211150</tpEvento>"));
    assert!(xml.contains("<descEvento>Solicitação de Apropriação de Crédito para bens e serviços que dependem de atividade do adquirente</descEvento>"));
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<tpAutor>2</tpAutor>"));
    assert!(xml.contains("<gCredito nItem=\"1\">"));
    assert!(xml.contains("<vCredIBS>45.00</vCredIBS>"));
    assert!(xml.contains("<vCredCBS>22.50</vCredCBS>"));
    assert!(xml.contains("</gCredito>"));
}

#[test]
fn rtc_apropriacao_credito_bens_uses_ibs_cbs_fallback() {
    // Without explicit v_cred_ibs/v_cred_cbs, should use v_ibs/v_cbs
    let itens = vec![RtcItem::new(1, 50.00, 25.00)];
    let xml = build_rtc_apropriacao_credito_bens(
        TEST_KEY,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
        "SP",
        "APP-1.0",
        &itens,
        None,
    );
    assert!(xml.contains("<vCredIBS>50.00</vCredIBS>"));
    assert!(xml.contains("<vCredCBS>25.00</vCredCBS>"));
}

// ── RTC: build_rtc_manif_transf_cred_ibs (lines 2146-2170) ─────

#[test]
fn rtc_manif_transf_cred_ibs_structure() {
    let xml = build_rtc_manif_transf_cred_ibs(
        TEST_KEY,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
        "SP",
        "APP-1.0",
        1,
        None,
    );
    assert!(xml.contains("<tpEvento>212110</tpEvento>"));
    assert!(xml.contains("<descEvento>Manifestação sobre Pedido de Transferência de Crédito de IBS em Operação de Sucessão</descEvento>"));
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<tpAutor>8</tpAutor>"));
    assert!(xml.contains("<verAplic>APP-1.0</verAplic>"));
    assert!(xml.contains("<indAceitacao>1</indAceitacao>"));
}

// ── RTC: build_rtc_manif_transf_cred_cbs (lines 2175-2199) ─────

#[test]
fn rtc_manif_transf_cred_cbs_structure() {
    let xml = build_rtc_manif_transf_cred_cbs(
        TEST_KEY,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
        "SP",
        "APP-1.0",
        0,
        None,
    );
    assert!(xml.contains("<tpEvento>212120</tpEvento>"));
    assert!(xml.contains("<descEvento>Manifestação sobre Pedido de Transferência de Crédito de CBS em Operação de Sucessão</descEvento>"));
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<tpAutor>8</tpAutor>"));
    assert!(xml.contains("<verAplic>APP-1.0</verAplic>"));
    assert!(xml.contains("<indAceitacao>0</indAceitacao>"));
}

// ── RTC: build_rtc_cancela_evento (lines 2205-2230) ─────────────

#[test]
fn rtc_cancela_evento_structure() {
    let xml = build_rtc_cancela_evento(
        TEST_KEY,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
        "SP",
        "APP-1.0",
        "211120",
        "135220000009999",
        None,
    );
    assert!(xml.contains("<tpEvento>110001</tpEvento>"));
    assert!(xml.contains("<descEvento>Cancelamento de Evento</descEvento>"));
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<verAplic>APP-1.0</verAplic>"));
    assert!(xml.contains("<tpEventoAut>211120</tpEventoAut>"));
    assert!(xml.contains("<nProtEvento>135220000009999</nProtEvento>"));
}

// ── RTC: build_rtc_importacao_zfm (lines 2235-2270) ─────────────

#[test]
fn rtc_importacao_zfm_structure() {
    let itens = vec![RtcItem::new(1, 12.00, 6.00).quantidade(10.0).unidade("KG")];
    let xml = build_rtc_importacao_zfm(
        TEST_KEY,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
        "SP",
        "APP-1.0",
        &itens,
        None,
    );
    assert!(xml.contains("<tpEvento>112120</tpEvento>"));
    assert!(
        xml.contains("<descEvento>Importação em ALC/ZFM não convertida em isenção</descEvento>")
    );
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<tpAutor>1</tpAutor>"));
    assert!(xml.contains("<gConsumo nItem=\"1\">"));
    assert!(xml.contains("<vIBS>12.00</vIBS>"));
    assert!(xml.contains("<vCBS>6.00</vCBS>"));
    assert!(xml.contains("<qtde>10.0000</qtde>"));
    assert!(xml.contains("<unidade>KG</unidade>"));
    assert!(xml.contains("</gConsumo>"));
}

// ── RTC: build_rtc_roubo_perda_adquirente (lines 2275-2310) ─────

#[test]
fn rtc_roubo_perda_adquirente_structure() {
    let itens = vec![RtcItem::new(1, 8.00, 4.00).quantidade(3.0).unidade("CX")];
    let xml = build_rtc_roubo_perda_adquirente(
        TEST_KEY,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
        "SP",
        "APP-1.0",
        &itens,
        None,
    );
    assert!(xml.contains("<tpEvento>211124</tpEvento>"));
    assert!(xml.contains("<descEvento>Perecimento, perda, roubo ou furto durante o transporte contratado pelo adquirente</descEvento>"));
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<tpAutor>2</tpAutor>"));
    assert!(xml.contains("<gPerecimento nItem=\"1\">"));
    assert!(xml.contains("<vIBS>8.00</vIBS>"));
    assert!(xml.contains("<vCBS>4.00</vCBS>"));
    assert!(xml.contains("<qPerecimento>3.0000</qPerecimento>"));
    assert!(xml.contains("<uPerecimento>CX</uPerecimento>"));
    assert!(xml.contains("</gPerecimento>"));
}

// ── RTC: build_rtc_roubo_perda_fornecedor (lines 2315-2353) ─────

#[test]
fn rtc_roubo_perda_fornecedor_structure() {
    let mut item = RtcItem::new(1, 16.00, 8.00).quantidade(7.0).unidade("UN");
    item.g_controle_estoque_v_ibs = Some(14.00);
    item.g_controle_estoque_v_cbs = Some(7.00);
    let itens = vec![item];
    let xml = build_rtc_roubo_perda_fornecedor(
        TEST_KEY,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
        "SP",
        "APP-1.0",
        &itens,
        None,
    );
    assert!(xml.contains("<tpEvento>112130</tpEvento>"));
    assert!(xml.contains("<descEvento>Perecimento, perda, roubo ou furto durante o transporte contratado pelo fornecedor</descEvento>"));
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<tpAutor>1</tpAutor>"));
    assert!(xml.contains("<gPerecimento nItem=\"1\">"));
    assert!(xml.contains("<vIBS>16.00</vIBS>"));
    assert!(xml.contains("<vCBS>8.00</vCBS>"));
    assert!(xml.contains("<qPerecimento>7.0000</qPerecimento>"));
    assert!(xml.contains("<uPerecimento>UN</uPerecimento>"));
    // gControleEstoque also contains vIBS/vCBS
    assert!(xml.contains("<vIBS>14.00</vIBS>"));
    assert!(xml.contains("<vCBS>7.00</vCBS>"));
    assert!(xml.contains("</gPerecimento>"));
}

// ── RTC: build_rtc_fornecimento_nao_realizado (lines 2358-2393) ──

#[test]
fn rtc_fornecimento_nao_realizado_structure() {
    let itens = vec![RtcItem::new(1, 22.00, 11.00).quantidade(4.0).unidade("MT")];
    let xml = build_rtc_fornecimento_nao_realizado(
        TEST_KEY,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
        "SP",
        "APP-1.0",
        &itens,
        None,
    );
    assert!(xml.contains("<tpEvento>112140</tpEvento>"));
    assert!(
        xml.contains(
            "<descEvento>Fornecimento não realizado com pagamento antecipado</descEvento>"
        )
    );
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<tpAutor>1</tpAutor>"));
    assert!(xml.contains("<gItemNaoFornecido nItem=\"1\">"));
    assert!(xml.contains("<vIBS>22.00</vIBS>"));
    assert!(xml.contains("<vCBS>11.00</vCBS>"));
    assert!(xml.contains("<qNaoFornecida>4.0000</qNaoFornecida>"));
    assert!(xml.contains("<uNaoFornecida>MT</uNaoFornecida>"));
    assert!(xml.contains("</gItemNaoFornecido>"));
}

// ── RTC: build_rtc_atualizacao_data_entrega (lines 2398-2422) ───

#[test]
fn rtc_atualizacao_data_entrega_structure() {
    let xml = build_rtc_atualizacao_data_entrega(
        TEST_KEY,
        1,
        SefazEnvironment::Homologation,
        TEST_CNPJ,
        "SP",
        "APP-1.0",
        "2026-06-15",
        None,
    );
    assert!(xml.contains("<tpEvento>112150</tpEvento>"));
    assert!(xml.contains("<descEvento>Atualização da Data de Previsão de Entrega</descEvento>"));
    assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
    assert!(xml.contains("<tpAutor>1</tpAutor>"));
    assert!(xml.contains("<verAplic>APP-1.0</verAplic>"));
    assert!(xml.contains("<dPrevEntrega>2026-06-15</dPrevEntrega>"));
}

// ── build_autorizacao_batch_request ──────────────────────────────

#[test]
fn batch_request_single_xml() {
    let nfe = "<NFe><infNFe>data1</infNFe></NFe>";
    let result = build_autorizacao_batch_request(&[nfe], "123", 0).unwrap();
    assert!(result.contains("<idLote>123</idLote>"));
    assert!(result.contains("<indSinc>0</indSinc>"));
    assert!(result.contains("<NFe><infNFe>data1</infNFe></NFe>"));
    assert!(
        result
            .starts_with("<enviNFe xmlns=\"http://www.portalfiscal.inf.br/nfe\" versao=\"4.00\">")
    );
    assert!(result.ends_with("</enviNFe>"));
}

#[test]
fn batch_request_multiple_xmls() {
    let nfe1 = "<NFe><infNFe>data1</infNFe></NFe>";
    let nfe2 = "<NFe><infNFe>data2</infNFe></NFe>";
    let nfe3 = "<NFe><infNFe>data3</infNFe></NFe>";
    let result = build_autorizacao_batch_request(&[nfe1, nfe2, nfe3], "456", 0).unwrap();
    assert!(result.contains("<indSinc>0</indSinc>"));
    assert!(result.contains("<NFe><infNFe>data1</infNFe></NFe><NFe><infNFe>data2</infNFe></NFe><NFe><infNFe>data3</infNFe></NFe>"));
}

#[test]
fn batch_request_strips_xml_declarations() {
    let nfe1 = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><NFe><infNFe>d1</infNFe></NFe>";
    let nfe2 = "<?xml version=\"1.0\"?><NFe><infNFe>d2</infNFe></NFe>";
    let result = build_autorizacao_batch_request(&[nfe1, nfe2], "789", 0).unwrap();
    assert!(!result.contains("<?xml"));
    assert!(result.contains("<NFe><infNFe>d1</infNFe></NFe>"));
    assert!(result.contains("<NFe><infNFe>d2</infNFe></NFe>"));
}

#[test]
fn batch_request_sync_single_ok() {
    let nfe = "<NFe><infNFe>data</infNFe></NFe>";
    let result = build_autorizacao_batch_request(&[nfe], "1", 1).unwrap();
    assert!(result.contains("<indSinc>1</indSinc>"));
}

#[test]
fn batch_request_sync_multiple_rejected() {
    let nfe1 = "<NFe>1</NFe>";
    let nfe2 = "<NFe>2</NFe>";
    let err = build_autorizacao_batch_request(&[nfe1, nfe2], "1", 1).unwrap_err();
    assert!(
        matches!(err, fiscal_core::FiscalError::InvalidTaxData(_)),
        "expected InvalidTaxData, got: {err}"
    );
}

#[test]
fn batch_request_rejects_empty() {
    let err = build_autorizacao_batch_request(&[], "1", 0).unwrap_err();
    assert!(matches!(err, fiscal_core::FiscalError::InvalidTaxData(_)));
}

#[test]
fn batch_request_rejects_more_than_50() {
    let xmls: Vec<&str> = (0..51).map(|_| "<NFe>x</NFe>").collect();
    let err = build_autorizacao_batch_request(&xmls, "1", 0).unwrap_err();
    assert!(matches!(err, fiscal_core::FiscalError::InvalidTaxData(_)));
    let msg = err.to_string();
    assert!(
        msg.contains("50"),
        "error should mention the 50-doc limit: {msg}"
    );
}

#[test]
fn batch_request_accepts_exactly_50() {
    let xmls: Vec<&str> = (0..50).map(|_| "<NFe>x</NFe>").collect();
    let result = build_autorizacao_batch_request(&xmls, "1", 0);
    assert!(result.is_ok());
}
