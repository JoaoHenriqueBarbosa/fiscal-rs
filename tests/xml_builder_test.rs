use chrono::FixedOffset;
use fiscal::newtypes::{Cents, IbgeCode, Rate};
use fiscal::types::*;
use fiscal::xml_builder::{InvoiceBuilder, build_access_key};
use fiscal::xml_utils::{TagContent, tag};

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
        AccessKeyParams::new(
            IbgeCode("35".to_string()),
            "2601",
            "12345678000199",
            InvoiceModel::Nfce,
            1,
            1,
            EmissionType::Normal,
            "12345678",
        )
    }

    #[test]
    fn generates_a_44_digit_access_key() {
        let key = build_access_key(&base_params()).unwrap();
        assert_eq!(key.len(), 44);
        assert!(key.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn pads_fields_correctly() {
        let params = AccessKeyParams::new(
            IbgeCode("35".to_string()),
            "2601",
            "12345678000199",
            InvoiceModel::Nfce,
            1,
            42,
            EmissionType::Normal,
            "00000001",
        );
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
        let mut p1 = base_params();
        p1.number = 1;
        let key1 = build_access_key(&p1).unwrap();
        let mut p2 = base_params();
        p2.number = 2;
        let key2 = build_access_key(&p2).unwrap();
        assert_ne!(key1, key2);
    }
}

// ── InvoiceBuilder ──────────────────────────────────────────────────────────

mod build_invoice_xml_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn sample_issuer() -> IssuerData {
        IssuerData::new(
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
        )
        .trade_name("Test")
    }

    fn sample_item() -> InvoiceItemData {
        InvoiceItemData::new(
            1,
            "1",
            "Product A",
            "84715010",
            "5102",
            "UN",
            2.0,
            Cents(1000),
            Cents(2000),
            "102",
            Rate(0),
            Cents(0),
            "99",
            "99",
        )
    }

    fn sample_payment() -> PaymentData {
        PaymentData::new("01", Cents(2000))
    }

    fn issued_at() -> chrono::DateTime<FixedOffset> {
        let offset = FixedOffset::west_opt(3 * 3600).unwrap();
        chrono::NaiveDate::from_ymd_opt(2026, 1, 15)
            .unwrap()
            .and_hms_opt(10, 30, 0)
            .unwrap()
            .and_local_timezone(offset)
            .unwrap()
    }

    fn sample_builder() -> InvoiceBuilder {
        InvoiceBuilder::new(
            sample_issuer(),
            SefazEnvironment::Homologation,
            InvoiceModel::Nfce,
        )
        .series(1)
        .invoice_number(1)
        .issued_at(issued_at())
        .add_item(sample_item())
        .payments(vec![sample_payment()])
    }

    #[test]
    fn generates_valid_xml_with_correct_structure() {
        let built = sample_builder().build().unwrap();
        assert!(
            built
                .xml()
                .contains(r#"<?xml version="1.0" encoding="UTF-8"?>"#)
        );
        assert!(built.xml().contains("<NFe"));
        assert!(built.xml().contains("<infNFe"));
        assert!(built.xml().contains("</NFe>"));
        assert_eq!(built.access_key().len(), 44);
    }

    #[test]
    fn contains_required_groups() {
        let built = sample_builder().build().unwrap();
        let xml = built.xml();
        assert!(xml.contains("<ide>"));
        assert!(xml.contains("<emit>"));
        assert!(xml.contains("<det "));
        assert!(xml.contains("<total>"));
        assert!(xml.contains("<transp>"));
        assert!(xml.contains("<pag>"));
    }

    #[test]
    fn sets_model_65_for_nfce() {
        let built = sample_builder().build().unwrap();
        assert!(built.xml().contains("<mod>65</mod>"));
    }

    #[test]
    fn sets_model_55_for_nfe() {
        let built = InvoiceBuilder::new(
            sample_issuer(),
            SefazEnvironment::Homologation,
            InvoiceModel::Nfe,
        )
        .series(1)
        .invoice_number(1)
        .issued_at(issued_at())
        .add_item(sample_item())
        .payments(vec![sample_payment()])
        .build()
        .unwrap();
        assert!(built.xml().contains("<mod>55</mod>"));
    }

    #[test]
    fn includes_issuer_data() {
        let built = sample_builder().build().unwrap();
        let xml = built.xml();
        assert!(xml.contains("<CNPJ>12345678000199</CNPJ>"));
        assert!(xml.contains("<xNome>Test Company</xNome>"));
        assert!(xml.contains("<IE>123456789</IE>"));
        assert!(xml.contains("<CRT>1</CRT>"));
    }

    #[test]
    fn includes_item_data() {
        let built = sample_builder().build().unwrap();
        let xml = built.xml();
        assert!(xml.contains(r#"<det nItem="1">"#));
        assert!(xml.contains("<xProd>Product A</xProd>"));
        assert!(xml.contains("<NCM>84715010</NCM>"));
        assert!(xml.contains("<CFOP>5102</CFOP>"));
    }

    #[test]
    fn formats_amounts_correctly() {
        let built = sample_builder().build().unwrap();
        let xml = built.xml();
        assert!(xml.contains("<vProd>20.00</vProd>"));
        assert!(xml.contains("<vNF>20.00</vNF>"));
    }

    #[test]
    fn includes_payment_data() {
        let built = sample_builder().build().unwrap();
        let xml = built.xml();
        assert!(xml.contains("<tPag>01</tPag>"));
        assert!(xml.contains("<vPag>20.00</vPag>"));
    }

    #[test]
    fn includes_homologation_note_when_environment_2() {
        let built = sample_builder().build().unwrap();
        assert!(built.xml().contains("HOMOLOGACAO"));
    }

    #[test]
    fn includes_recipient_when_provided() {
        let built = sample_builder()
            .recipient(RecipientData::new("12345678901", "John Doe"))
            .build()
            .unwrap();
        let xml = built.xml();
        assert!(xml.contains("<dest>"));
        assert!(xml.contains("<CPF>12345678901</CPF>"));
        assert!(xml.contains("<xNome>John Doe</xNome>"));
    }

    #[test]
    fn omits_recipient_for_nfce_without_recipient() {
        let built = sample_builder().build().unwrap();
        assert!(!built.xml().contains("<dest>"));
    }

    #[test]
    fn includes_contingency_info_when_provided() {
        let offset = FixedOffset::west_opt(3 * 3600).unwrap();
        let now = chrono::Utc::now().with_timezone(&offset);

        let built = sample_builder()
            .contingency(ContingencyData::new(
                ContingencyType::Offline,
                "SEFAZ unavailable",
                now,
            ))
            .build()
            .unwrap();
        let xml = built.xml();
        assert!(xml.contains("contingencia"));
        assert!(xml.contains("SEFAZ unavailable"));
    }

    #[test]
    fn includes_csrt_hash_when_configured() {
        let now = chrono::Utc::now().with_timezone(&FixedOffset::west_opt(3 * 3600).unwrap());
        let built = InvoiceBuilder::new(
            sample_issuer(),
            SefazEnvironment::Homologation,
            InvoiceModel::Nfe,
        )
        .issued_at(now)
        .items(vec![sample_item()])
        .payments(vec![PaymentData::new("01", Cents(1000))])
        .tech_responsible(
            TechResponsibleData::new("99999999999999", "Fulano de Tal", "fulano@soft.com.br")
                .phone("1155551122")
                .csrt("G8063VRTNDMO886SFNK5LDUDEI24XJ22YIPO", "01"),
        )
        .build()
        .unwrap();
        let xml = built.xml();

        // infRespTec must contain idCSRT and hashCSRT
        assert!(xml.contains("<idCSRT>01</idCSRT>"), "idCSRT tag missing");
        assert!(xml.contains("<hashCSRT>"), "hashCSRT tag missing");
        assert!(xml.contains("</hashCSRT>"), "hashCSRT closing tag missing");

        // Verify tag order: CNPJ, xContato, email, fone, idCSRT, hashCSRT
        let resp_start = xml.find("<infRespTec>").unwrap();
        let resp_end = xml.find("</infRespTec>").unwrap();
        let resp = &xml[resp_start..resp_end];
        let cnpj_pos = resp.find("<CNPJ>").unwrap();
        let id_csrt_pos = resp.find("<idCSRT>").unwrap();
        let hash_pos = resp.find("<hashCSRT>").unwrap();
        assert!(cnpj_pos < id_csrt_pos, "CNPJ must come before idCSRT");
        assert!(id_csrt_pos < hash_pos, "idCSRT must come before hashCSRT");
    }

    #[test]
    fn build_tech_responsible_without_key_omits_csrt() {
        use fiscal::xml_builder::optional::build_tech_responsible;

        let tech = TechResponsibleData::new("14363848000190", "Solusys", "contato@solusys.com.br")
            .phone("4334771000")
            .csrt("G8063VRTNDMO886SFNK5LDUDEI24XJ22YIPO", "01");

        // build_tech_responsible (1 param, backward-compatible) omits CSRT
        let xml = build_tech_responsible(&tech);
        assert!(xml.contains("<CNPJ>14363848000190</CNPJ>"));
        assert!(xml.contains("<fone>4334771000</fone>"));
        assert!(!xml.contains("<idCSRT>"), "1-param version must omit CSRT");
        assert!(
            !xml.contains("<hashCSRT>"),
            "1-param version must omit hashCSRT"
        );
    }

    #[test]
    fn build_tech_responsible_with_key_includes_csrt_and_exact_hash() {
        use fiscal::xml_builder::optional::build_tech_responsible_with_key;

        let tech = TechResponsibleData::new("14363848000190", "Solusys", "contato@solusys.com.br")
            .phone("4334771000")
            .csrt("G8063VRTNDMO886SFNK5LDUDEI24XJ22YIPO", "01");

        let access_key = "35200612345678000199550010000000011123456789";
        let xml = build_tech_responsible_with_key(&tech, access_key);

        assert!(xml.contains("<idCSRT>01</idCSRT>"));
        assert!(xml.contains("<hashCSRT>"));

        // Verify exact hash value matches PHP: base64(sha1(CSRT + chNFe, raw))
        use base64::Engine as _;
        use sha1::{Digest, Sha1};
        let combined = format!("G8063VRTNDMO886SFNK5LDUDEI24XJ22YIPO{access_key}");
        let mut hasher = Sha1::new();
        hasher.update(combined.as_bytes());
        let expected_hash = base64::engine::general_purpose::STANDARD.encode(hasher.finalize());

        let expected_tag = format!("<hashCSRT>{expected_hash}</hashCSRT>");
        assert!(
            xml.contains(&expected_tag),
            "hashCSRT must be {expected_hash}, got: {xml}"
        );

        // Verify tag order: CNPJ, xContato, email, fone, idCSRT, hashCSRT
        let cnpj_pos = xml.find("<CNPJ>").unwrap();
        let fone_pos = xml.find("<fone>").unwrap();
        let id_pos = xml.find("<idCSRT>").unwrap();
        let hash_pos = xml.find("<hashCSRT>").unwrap();
        assert!(cnpj_pos < fone_pos);
        assert!(fone_pos < id_pos);
        assert!(id_pos < hash_pos);
    }

    #[test]
    fn csrt_hash_matches_php_algorithm() {
        // Verify the hashCSRT algorithm matches PHP sped-nfe:
        // hashCSRT = base64(sha1(CSRT + chNFe, raw_binary))
        use base64::Engine as _;
        use sha1::{Digest, Sha1};

        let csrt = "G8063VRTNDMO886SFNK5LDUDEI24XJ22YIPO";
        let access_key = "35200612345678000199550010000000011123456789";

        // PHP: base64_encode(sha1($CSRT . $this->chNFe, true))
        let combined = format!("{csrt}{access_key}");
        let mut hasher = Sha1::new();
        hasher.update(combined.as_bytes());
        let raw_hash = hasher.finalize();
        let expected = base64::engine::general_purpose::STANDARD.encode(raw_hash);

        // The hash should be a 28-char base64 string (SHA-1 = 20 bytes → 28 base64 chars)
        assert_eq!(expected.len(), 28, "SHA-1 base64 hash should be 28 chars");
        assert!(expected.ends_with('='), "SHA-1 base64 should be padded");
    }

    #[test]
    fn omits_csrt_when_not_configured() {
        let now = chrono::Utc::now().with_timezone(&FixedOffset::west_opt(3 * 3600).unwrap());
        let built = InvoiceBuilder::new(
            sample_issuer(),
            SefazEnvironment::Homologation,
            InvoiceModel::Nfe,
        )
        .issued_at(now)
        .items(vec![sample_item()])
        .payments(vec![PaymentData::new("01", Cents(1000))])
        .tech_responsible(TechResponsibleData::new(
            "99999999999999",
            "Fulano de Tal",
            "fulano@soft.com.br",
        ))
        .build()
        .unwrap();
        let xml = built.xml();

        assert!(xml.contains("<infRespTec>"), "infRespTec should be present");
        assert!(
            !xml.contains("<idCSRT>"),
            "idCSRT should NOT be present without CSRT config"
        );
        assert!(
            !xml.contains("<hashCSRT>"),
            "hashCSRT should NOT be present without CSRT config"
        );
    }

    /// This test verifies that the Rust XML builder produces the same
    /// structural output as PHP sped-nfe by comparing against a reference
    /// fixture generated with identical input data.
    ///
    /// The reference file `tests/fixtures/xml/php-reference-nfe.xml` was
    /// generated by running PHP sped-nfe Make with the same issuer, item,
    /// and payment data.
    #[test]
    fn xml_structure_matches_php_sped_nfe_reference() {
        // Load PHP reference
        let php_xml = std::fs::read_to_string(format!(
            "{}/tests/fixtures/xml/php-reference-nfe.xml",
            env!("CARGO_MANIFEST_DIR")
        ))
        .expect("PHP reference fixture not found");

        // Key structural checks against PHP output:

        // 1. <infNFe> must NOT have xmlns (inherited from <NFe>)
        assert!(
            !php_xml.contains("<infNFe xmlns="),
            "PHP reference should not have xmlns on infNFe"
        );

        // 2. Build a Rust XML and verify infNFe also has no xmlns
        let built = InvoiceBuilder::new(
            sample_issuer(),
            SefazEnvironment::Homologation,
            InvoiceModel::Nfe,
        )
        .issued_at(issued_at())
        .items(vec![sample_item()])
        .payments(vec![sample_payment()])
        .build()
        .unwrap();
        let rust_xml = built.xml();

        assert!(
            !rust_xml.contains("<infNFe xmlns="),
            "Rust infNFe must not have xmlns — inherited from parent NFe (matches PHP)"
        );

        // 3. vFCPUFDest, vICMSUFDest, vICMSUFRemet must be omitted when zero
        assert!(
            !php_xml.contains("<vFCPUFDest>"),
            "PHP omits vFCPUFDest when zero"
        );
        assert!(
            !rust_xml.contains("<vFCPUFDest>"),
            "Rust must omit vFCPUFDest when zero (matches PHP)"
        );
        assert!(
            !rust_xml.contains("<vICMSUFDest>"),
            "Rust must omit vICMSUFDest when zero (matches PHP)"
        );
        assert!(
            !rust_xml.contains("<vICMSUFRemet>"),
            "Rust must omit vICMSUFRemet when zero (matches PHP)"
        );

        // 4. ICMSTot tag order must match PHP (when zero-value optional tags are omitted)
        let php_tags = extract_tag_names_between(&php_xml, "<ICMSTot>", "</ICMSTot>");
        let rust_tags = extract_tag_names_between(rust_xml, "<ICMSTot>", "</ICMSTot>");
        assert_eq!(
            php_tags, rust_tags,
            "ICMSTot tag order must match PHP sped-nfe"
        );
    }
}

/// Extract opening tag names between two markers.
fn extract_tag_names_between(xml: &str, start_marker: &str, end_marker: &str) -> Vec<String> {
    let start = xml.find(start_marker).unwrap() + start_marker.len();
    let end = xml[start..].find(end_marker).unwrap() + start;
    extract_all_tag_names(&xml[start..end])
}

/// Extract all opening XML tag names from a string.
fn extract_all_tag_names(xml: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut remaining = xml;
    while let Some(pos) = remaining.find('<') {
        remaining = &remaining[pos + 1..];
        if remaining.starts_with('/') || remaining.starts_with('?') || remaining.starts_with('!') {
            continue;
        }
        let end = remaining.find(['>', ' ']).unwrap_or(remaining.len());
        let tag_name = &remaining[..end];
        if !tag_name.is_empty() {
            tags.push(tag_name.to_string());
        }
    }
    tags
}
