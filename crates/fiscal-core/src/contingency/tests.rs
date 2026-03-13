use super::adjust::*;
use super::manager::*;
use crate::FiscalError;
use crate::types::{ContingencyType, EmissionType, InvoiceModel};

#[test]
fn new_contingency_is_inactive() {
    let c = Contingency::new();
    assert!(c.contingency_type.is_none());
    assert!(!c.is_active());
    assert_eq!(c.emission_type(), 1);
}

#[test]
fn default_is_inactive() {
    let c = Contingency::default();
    assert!(c.contingency_type.is_none());
    assert!(!c.is_active());
}

#[test]
fn activate_sets_fields() {
    let mut c = Contingency::new();
    c.activate(
        ContingencyType::SvcAn,
        "A valid reason for contingency mode activation",
    )
    .unwrap();
    assert_eq!(c.contingency_type, Some(ContingencyType::SvcAn));
    assert_eq!(c.emission_type(), 6);
    assert!(c.is_active());
    assert!(c.reason.is_some());
    assert!(c.activated_at.is_some());
}

#[test]
fn activate_svc_rs() {
    let mut c = Contingency::new();
    c.activate(
        ContingencyType::SvcRs,
        "A valid reason for contingency mode activation",
    )
    .unwrap();
    assert_eq!(c.emission_type(), 7);
    assert_eq!(c.emission_type_enum(), EmissionType::SvcRs);
}

#[test]
fn activate_offline() {
    let mut c = Contingency::new();
    c.activate(
        ContingencyType::Offline,
        "A valid reason for contingency mode activation",
    )
    .unwrap();
    assert_eq!(c.emission_type(), 9);
    assert_eq!(c.emission_type_enum(), EmissionType::Offline);
}

#[test]
fn activate_epec() {
    let mut c = Contingency::new();
    c.activate(
        ContingencyType::Epec,
        "A valid reason for contingency mode activation",
    )
    .unwrap();
    assert_eq!(c.emission_type(), 4);
    assert_eq!(c.emission_type_enum(), EmissionType::Epec);
}

#[test]
fn activate_fs_da() {
    let mut c = Contingency::new();
    c.activate(
        ContingencyType::FsDa,
        "A valid reason for contingency mode activation",
    )
    .unwrap();
    assert_eq!(c.emission_type(), 5);
    assert_eq!(c.emission_type_enum(), EmissionType::FsDa);
}

#[test]
fn activate_fs_ia() {
    let mut c = Contingency::new();
    c.activate(
        ContingencyType::FsIa,
        "A valid reason for contingency mode activation",
    )
    .unwrap();
    assert_eq!(c.emission_type(), 2);
    assert_eq!(c.emission_type_enum(), EmissionType::FsIa);
}

#[test]
fn activate_rejects_short_reason() {
    let mut c = Contingency::new();
    let result = c.activate(ContingencyType::SvcAn, "Short");
    assert!(result.is_err());
}

#[test]
fn activate_rejects_long_reason() {
    let mut c = Contingency::new();
    let motive = "A".repeat(256);
    let result = c.activate(ContingencyType::SvcAn, &motive);
    assert!(result.is_err());
}

#[test]
fn activate_accepts_255_char_reason() {
    let mut c = Contingency::new();
    let motive = "A".repeat(255);
    let result = c.activate(ContingencyType::SvcAn, &motive);
    assert!(result.is_ok(), "255-char reason must be accepted");
}

#[test]
fn activate_rejects_256_char_reason() {
    let mut c = Contingency::new();
    let motive = "A".repeat(256);
    let result = c.activate(ContingencyType::SvcAn, &motive);
    assert!(result.is_err(), "256-char reason must be rejected");
}

#[test]
fn deactivate_clears_state() {
    let mut c = Contingency::new();
    c.activate(
        ContingencyType::SvcAn,
        "A valid reason for contingency mode activation",
    )
    .unwrap();
    c.deactivate();
    assert!(c.contingency_type.is_none());
    assert!(!c.is_active());
    assert_eq!(c.emission_type(), 1);
    assert_eq!(c.emission_type_enum(), EmissionType::Normal);
}

#[test]
fn load_from_json() {
    let json = r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"SVCAN","tpEmis":6}"#;
    let c = Contingency::load(json).unwrap();
    assert_eq!(c.contingency_type, Some(ContingencyType::SvcAn));
    assert_eq!(c.emission_type(), 6);
    assert_eq!(c.reason.as_deref(), Some("Testes Unitarios"));
    assert!(c.is_active());
}

#[test]
fn load_svc_rs_from_json() {
    let json = r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"SVCRS","tpEmis":7}"#;
    let c = Contingency::load(json).unwrap();
    assert_eq!(c.contingency_type, Some(ContingencyType::SvcRs));
    assert_eq!(c.emission_type(), 7);
}

#[test]
fn load_epec_from_json() {
    let json = r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"EPEC","tpEmis":4}"#;
    let c = Contingency::load(json).unwrap();
    assert_eq!(c.contingency_type, Some(ContingencyType::Epec));
    assert_eq!(c.emission_type(), 4);
}

#[test]
fn load_fs_da_from_json() {
    let json = r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"FSDA","tpEmis":5}"#;
    let c = Contingency::load(json).unwrap();
    assert_eq!(c.contingency_type, Some(ContingencyType::FsDa));
    assert_eq!(c.emission_type(), 5);
}

#[test]
fn load_fs_ia_from_json() {
    let json = r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"FSIA","tpEmis":2}"#;
    let c = Contingency::load(json).unwrap();
    assert_eq!(c.contingency_type, Some(ContingencyType::FsIa));
    assert_eq!(c.emission_type(), 2);
}

#[test]
fn load_offline_from_json() {
    let json =
        r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"OFFLINE","tpEmis":9}"#;
    let c = Contingency::load(json).unwrap();
    assert_eq!(c.contingency_type, Some(ContingencyType::Offline));
    assert_eq!(c.emission_type(), 9);
}

#[test]
fn load_deactivated_from_json() {
    let json = r#"{"motive":"","timestamp":0,"type":"","tpEmis":1}"#;
    let c = Contingency::load(json).unwrap();
    assert!(c.contingency_type.is_none());
    assert!(!c.is_active());
    assert_eq!(c.emission_type(), 1);
}

#[test]
fn load_rejects_unknown_type() {
    let json =
        r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"UNKNOWN","tpEmis":1}"#;
    let result = Contingency::load(json);
    assert!(result.is_err());
}

#[test]
fn to_json_activated() {
    let json = r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"SVCAN","tpEmis":6}"#;
    let c = Contingency::load(json).unwrap();
    assert_eq!(c.to_json(), json);
}

#[test]
fn to_json_deactivated() {
    let c = Contingency::new();
    assert_eq!(
        c.to_json(),
        r#"{"motive":"","timestamp":0,"type":"","tpEmis":1}"#
    );
}

#[test]
fn to_json_roundtrip() {
    let json = r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"SVCRS","tpEmis":7}"#;
    let c = Contingency::load(json).unwrap();
    let output = c.to_json();
    assert_eq!(output, json);
    // Load again and verify
    let c2 = Contingency::load(&output).unwrap();
    assert_eq!(c2.contingency_type, c.contingency_type);
    assert_eq!(c2.reason, c.reason);
    assert_eq!(c2.timestamp, c.timestamp);
}

#[test]
fn deactivate_produces_correct_json() {
    let json = r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"SVCAN","tpEmis":6}"#;
    let mut c = Contingency::load(json).unwrap();
    c.deactivate();
    assert_eq!(
        c.to_json(),
        r#"{"motive":"","timestamp":0,"type":"","tpEmis":1}"#
    );
}

#[test]
fn display_matches_to_json() {
    let json = r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"SVCAN","tpEmis":6}"#;
    let c = Contingency::load(json).unwrap();
    assert_eq!(format!("{c}"), c.to_json());
}

#[test]
fn extract_json_string_works() {
    let json = r#"{"motive":"hello world","type":"SVCAN"}"#;
    assert_eq!(
        extract_json_string(json, "motive"),
        Some("hello world".to_string())
    );
    assert_eq!(extract_json_string(json, "type"), Some("SVCAN".to_string()));
}

#[test]
fn extract_json_number_works() {
    let json = r#"{"timestamp":1480700623,"tpEmis":6}"#;
    assert_eq!(extract_json_number(json, "timestamp"), Some(1480700623));
    assert_eq!(extract_json_number(json, "tpEmis"), Some(6));
}

#[test]
fn format_timestamp_with_offset_formats_correctly() {
    // 1480700623 = 2016-12-02T17:43:43Z = 2016-12-02T14:43:43-03:00
    let result = format_timestamp_with_offset(1480700623, "-03:00");
    assert_eq!(result, "2016-12-02T14:43:43-03:00");
}

#[test]
fn contingency_for_state_sp() {
    assert_eq!(contingency_for_state("SP").as_str(), "svc-an");
}

#[test]
fn contingency_for_state_am() {
    assert_eq!(contingency_for_state("AM").as_str(), "svc-rs");
}

#[test]
fn try_contingency_for_state_valid() {
    assert_eq!(
        try_contingency_for_state("SP").unwrap(),
        ContingencyType::SvcAn
    );
    assert_eq!(
        try_contingency_for_state("AM").unwrap(),
        ContingencyType::SvcRs
    );
}

#[test]
fn try_contingency_for_state_invalid() {
    assert!(try_contingency_for_state("XX").is_err());
}

#[test]
fn check_web_service_nfe_svc_an_ok() {
    let mut c = Contingency::new();
    c.activate(
        ContingencyType::SvcAn,
        "A valid reason for contingency mode activation",
    )
    .unwrap();
    assert!(c.check_web_service_availability(InvoiceModel::Nfe).is_ok());
}

#[test]
fn check_web_service_nfe_svc_rs_ok() {
    let mut c = Contingency::new();
    c.activate(
        ContingencyType::SvcRs,
        "A valid reason for contingency mode activation",
    )
    .unwrap();
    assert!(c.check_web_service_availability(InvoiceModel::Nfe).is_ok());
}

#[test]
fn check_web_service_nfce_svc_fails() {
    let mut c = Contingency::new();
    c.activate(
        ContingencyType::SvcAn,
        "A valid reason for contingency mode activation",
    )
    .unwrap();
    assert!(
        c.check_web_service_availability(InvoiceModel::Nfce)
            .is_err()
    );
}

#[test]
fn check_web_service_epec_no_webservice() {
    let mut c = Contingency::new();
    c.activate(
        ContingencyType::Epec,
        "A valid reason for contingency mode activation",
    )
    .unwrap();
    let err = c
        .check_web_service_availability(InvoiceModel::Nfe)
        .unwrap_err();
    assert!(err.to_string().contains("EPEC"));
}

#[test]
fn check_web_service_normal_mode_ok() {
    let c = Contingency::new();
    assert!(c.check_web_service_availability(InvoiceModel::Nfe).is_ok());
    assert!(c.check_web_service_availability(InvoiceModel::Nfce).is_ok());
}

#[test]
fn contingency_type_display() {
    assert_eq!(format!("{}", ContingencyType::SvcAn), "SVCAN");
    assert_eq!(format!("{}", ContingencyType::SvcRs), "SVCRS");
    assert_eq!(format!("{}", ContingencyType::Epec), "EPEC");
    assert_eq!(format!("{}", ContingencyType::FsDa), "FSDA");
    assert_eq!(format!("{}", ContingencyType::FsIa), "FSIA");
    assert_eq!(format!("{}", ContingencyType::Offline), "OFFLINE");
}

#[test]
fn contingency_type_from_str() {
    assert_eq!(
        "SVCAN".parse::<ContingencyType>().unwrap(),
        ContingencyType::SvcAn
    );
    assert_eq!(
        "SVC-AN".parse::<ContingencyType>().unwrap(),
        ContingencyType::SvcAn
    );
    assert_eq!(
        "SVCRS".parse::<ContingencyType>().unwrap(),
        ContingencyType::SvcRs
    );
    assert_eq!(
        "EPEC".parse::<ContingencyType>().unwrap(),
        ContingencyType::Epec
    );
    assert_eq!(
        "FSDA".parse::<ContingencyType>().unwrap(),
        ContingencyType::FsDa
    );
    assert_eq!(
        "FSIA".parse::<ContingencyType>().unwrap(),
        ContingencyType::FsIa
    );
    assert_eq!(
        "OFFLINE".parse::<ContingencyType>().unwrap(),
        ContingencyType::Offline
    );
    assert!("UNKNOWN".parse::<ContingencyType>().is_err());
}

#[test]
fn contingency_type_from_tp_emis() {
    assert_eq!(
        ContingencyType::from_tp_emis(2),
        Some(ContingencyType::FsIa)
    );
    assert_eq!(
        ContingencyType::from_tp_emis(4),
        Some(ContingencyType::Epec)
    );
    assert_eq!(
        ContingencyType::from_tp_emis(5),
        Some(ContingencyType::FsDa)
    );
    assert_eq!(
        ContingencyType::from_tp_emis(6),
        Some(ContingencyType::SvcAn)
    );
    assert_eq!(
        ContingencyType::from_tp_emis(7),
        Some(ContingencyType::SvcRs)
    );
    assert_eq!(
        ContingencyType::from_tp_emis(9),
        Some(ContingencyType::Offline)
    );
    assert_eq!(ContingencyType::from_tp_emis(1), None);
    assert_eq!(ContingencyType::from_tp_emis(0), None);
    assert_eq!(ContingencyType::from_tp_emis(3), None);
}

#[test]
fn escape_json_string_basic() {
    assert_eq!(escape_json_string("hello"), "hello");
    assert_eq!(escape_json_string(r#"say "hi""#), r#"say \"hi\""#);
    assert_eq!(escape_json_string("a\\b"), "a\\\\b");
}

#[test]
#[should_panic(expected = "Unknown state abbreviation")]
fn contingency_for_state_unknown_panics() {
    contingency_for_state("XX");
}

#[test]
fn display_for_contingency_matches_to_json() {
    let c = Contingency::new();
    assert_eq!(c.to_string(), c.to_json());
}

// ── adjust_nfe_contingency tests ────────────────────────────────

#[test]
fn adjust_nfe_contingency_inactive_returns_unchanged() {
    let c = Contingency::new();
    let xml = "<NFe><infNFe/></NFe>";
    let result = adjust_nfe_contingency(xml, &c).unwrap();
    assert_eq!(result, xml);
}

#[test]
fn adjust_nfe_contingency_model65_returns_error() {
    let mut c = Contingency::new();
    c.activate(
        ContingencyType::SvcAn,
        "Motivo de contingencia teste valido",
    )
    .unwrap();
    let xml = "<NFe><infNFe><ide><mod>65</mod><tpEmis>1</tpEmis></ide></infNFe></NFe>";
    let err = adjust_nfe_contingency(xml, &c).unwrap_err();
    assert!(matches!(err, FiscalError::Contingency(_)));
}

#[test]
fn adjust_nfe_contingency_already_non_normal_returns_unchanged() {
    let mut c = Contingency::new();
    c.activate(
        ContingencyType::SvcAn,
        "Motivo de contingencia teste valido",
    )
    .unwrap();
    let xml = "<NFe><infNFe><ide><mod>55</mod><tpEmis>6</tpEmis></ide></infNFe></NFe>";
    let result = adjust_nfe_contingency(xml, &c).unwrap();
    assert!(result.contains("<tpEmis>6</tpEmis>"));
}

#[test]
fn adjust_nfe_contingency_replaces_tp_emis_and_inserts_dh_cont() {
    let mut c = Contingency::new();
    c.activate(
        ContingencyType::SvcAn,
        "Motivo de contingencia teste valido",
    )
    .unwrap();
    let xml = concat!(
        r#"<NFe><infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
        "<ide><cUF>41</cUF><cNF>12345678</cNF><natOp>VENDA</natOp>",
        "<mod>55</mod><serie>1</serie><nNF>123</nNF>",
        "<dhEmi>2026-03-11T10:30:00-03:00</dhEmi>",
        "<tpNF>1</tpNF><idDest>1</idDest><cMunFG>4106902</cMunFG>",
        "<tpImp>1</tpImp><tpEmis>1</tpEmis><cDV>0</cDV>",
        "<tpAmb>2</tpAmb></ide>",
        "<emit><CNPJ>04123456000190</CNPJ></emit>",
        "</infNFe></NFe>"
    );
    let result = adjust_nfe_contingency(xml, &c).unwrap();
    assert!(result.contains("<tpEmis>6</tpEmis>"));
    assert!(result.contains("<dhCont>"));
    assert!(result.contains("<xJust>"));
}

#[test]
fn adjust_nfe_contingency_replaces_existing_dh_cont() {
    let mut c = Contingency::new();
    c.activate(
        ContingencyType::SvcAn,
        "Motivo de contingencia teste valido",
    )
    .unwrap();
    let xml = concat!(
        r#"<NFe><infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
        "<ide><cUF>41</cUF><cNF>12345678</cNF><natOp>VENDA</natOp>",
        "<mod>55</mod><serie>1</serie><nNF>123</nNF>",
        "<dhEmi>2026-03-11T10:30:00-03:00</dhEmi>",
        "<tpNF>1</tpNF><idDest>1</idDest><cMunFG>4106902</cMunFG>",
        "<tpImp>1</tpImp><tpEmis>1</tpEmis><cDV>0</cDV>",
        "<tpAmb>2</tpAmb>",
        "<dhCont>2020-01-01T00:00:00-03:00</dhCont>",
        "<xJust>old reason</xJust>",
        "</ide>",
        "<emit><CNPJ>04123456000190</CNPJ></emit>",
        "</infNFe></NFe>"
    );
    let result = adjust_nfe_contingency(xml, &c).unwrap();
    assert!(result.contains("<tpEmis>6</tpEmis>"));
    assert!(!result.contains("old reason"));
    assert!(result.contains("Motivo de contingencia teste valido"));
}

#[test]
fn adjust_nfe_contingency_inserts_before_nfref() {
    let mut c = Contingency::new();
    c.activate(
        ContingencyType::SvcRs,
        "Motivo de contingencia teste valido para NFRef",
    )
    .unwrap();
    let xml = concat!(
        r#"<NFe><infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
        "<ide><cUF>41</cUF><cNF>12345678</cNF><natOp>VENDA</natOp>",
        "<mod>55</mod><serie>1</serie><nNF>123</nNF>",
        "<dhEmi>2026-03-11T10:30:00-03:00</dhEmi>",
        "<tpNF>1</tpNF><idDest>1</idDest><cMunFG>4106902</cMunFG>",
        "<tpImp>1</tpImp><tpEmis>1</tpEmis><cDV>0</cDV>",
        "<tpAmb>2</tpAmb><NFref><refNFe>123</refNFe></NFref></ide>",
        "<emit><CNPJ>04123456000190</CNPJ></emit>",
        "</infNFe></NFe>"
    );
    let result = adjust_nfe_contingency(xml, &c).unwrap();
    assert!(result.contains("<tpEmis>7</tpEmis>"));
    // dhCont and xJust should appear before <NFref>
    let dh_pos = result.find("<dhCont>").unwrap();
    let nfref_pos = result.find("<NFref>").unwrap();
    assert!(dh_pos < nfref_pos);
}

#[test]
fn adjust_nfe_contingency_removes_signature() {
    let mut c = Contingency::new();
    c.activate(
        ContingencyType::SvcAn,
        "Motivo de contingencia teste valido",
    )
    .unwrap();
    let xml = concat!(
        r#"<NFe><infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
        "<ide><cUF>41</cUF><cNF>12345678</cNF><natOp>VENDA</natOp>",
        "<mod>55</mod><serie>1</serie><nNF>123</nNF>",
        "<dhEmi>2026-03-11T10:30:00-03:00</dhEmi>",
        "<tpNF>1</tpNF><idDest>1</idDest><cMunFG>4106902</cMunFG>",
        "<tpImp>1</tpImp><tpEmis>1</tpEmis><cDV>0</cDV>",
        "<tpAmb>2</tpAmb></ide>",
        "<emit><CNPJ>04123456000190</CNPJ></emit>",
        "</infNFe>",
        r#"<Signature xmlns="http://www.w3.org/2000/09/xmldsig#"><SignedInfo/></Signature>"#,
        "</NFe>"
    );
    let result = adjust_nfe_contingency(xml, &c).unwrap();
    assert!(!result.contains("<Signature"));
}

#[test]
fn extract_emitter_doc_cpf() {
    let xml = "<root><emit><CPF>12345678901</CPF></emit></root>";
    assert_eq!(extract_emitter_doc(xml), "12345678901");
}

#[test]
fn extract_emitter_doc_no_emit() {
    let xml = "<root><other/></root>";
    assert_eq!(extract_emitter_doc(xml), "");
}

#[test]
fn parse_year_month_short_input() {
    let (y, m) = parse_year_month("2026");
    assert_eq!(y, "00");
    assert_eq!(m, "00");
}

#[test]
fn extract_tz_offset_no_offset() {
    assert_eq!(extract_tz_offset("2026"), "-03:00");
}

#[test]
fn format_timestamp_with_offset_bad_offset() {
    // Very short offset, should fall through to fallback
    let result = format_timestamp_with_offset(0, "X");
    assert!(result.contains("1970"));
}

#[test]
fn escape_json_string_control_chars() {
    let s = escape_json_string("a\nb\tc\rd");
    assert_eq!(s, "a\\nb\\tc\\rd");
}

#[test]
fn all_27_states_have_mapping() {
    let states = [
        "AC", "AL", "AM", "AP", "BA", "CE", "DF", "ES", "GO", "MA", "MG", "MS", "MT", "PA", "PB",
        "PE", "PI", "PR", "RJ", "RN", "RO", "RR", "RS", "SC", "SE", "SP", "TO",
    ];
    for uf in states {
        let ct = contingency_for_state(uf);
        assert!(
            ct == ContingencyType::SvcAn || ct == ContingencyType::SvcRs,
            "State {uf} should map to SVC-AN or SVC-RS"
        );
    }
}
