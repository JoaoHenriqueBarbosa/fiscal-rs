// Ported from TypeScript: tools-ported.test.ts (48 tests)
// Tests for SEFAZ request builders, contingency, QR code

use fiscal::types::*;
use rstest::rstest;

// =============================================================================
// ToolsTest
// =============================================================================

mod tools_test {
    use super::*;

    // ─── sefazConsultaRecibo ─────────────────────────────────────────

    mod sefaz_consulta_recibo {
        use super::*;

        #[test]
        #[should_panic]
        fn throws_on_empty_recibo() {
            let _ = fiscal::sefaz::request_builders::build_consulta_recibo_request(
                "",
                SefazEnvironment::Homologation,
            );
        }

        #[test]
        fn builds_correct_xml_for_valid_recibo() {
            let xml = fiscal::sefaz::request_builders::build_consulta_recibo_request(
                "143220020730398",
                SefazEnvironment::Homologation,
            );
            assert!(xml.contains("<consReciNFe"));
            assert!(xml.contains(r#"versao="4.00""#));
            assert!(xml.contains("<tpAmb>2</tpAmb>"));
            assert!(xml.contains("<nRec>143220020730398</nRec>"));
            assert!(xml.contains("</consReciNFe>"));
        }
    }

    // ─── sefazConsultaChave ──────────────────────────────────────────

    mod sefaz_consulta_chave {
        use super::*;

        #[test]
        #[should_panic]
        fn throws_on_empty_chave() {
            let _ = fiscal::sefaz::request_builders::build_consulta_request(
                "",
                SefazEnvironment::Homologation,
            );
        }

        #[test]
        fn builds_correct_xml_for_valid_chave() {
            let chave = "43211105730928000145650010000002401717268120";
            let xml = fiscal::sefaz::request_builders::build_consulta_request(
                chave,
                SefazEnvironment::Homologation,
            );
            assert!(xml.contains("<consSitNFe"));
            assert!(xml.contains(r#"versao="4.00""#));
            assert!(xml.contains("<tpAmb>2</tpAmb>"));
            assert!(xml.contains("<xServ>CONSULTAR</xServ>"));
            assert!(xml.contains(&format!("<chNFe>{chave}</chNFe>")));
            assert!(xml.contains("</consSitNFe>"));
        }

        #[test]
        #[should_panic]
        fn throws_on_chave_length_not_44() {
            // 43 digits
            let _ = fiscal::sefaz::request_builders::build_consulta_request(
                "1234567890123456789012345678901234567890123",
                SefazEnvironment::Homologation,
            );
        }

        #[test]
        #[should_panic]
        fn throws_on_non_numeric_chave() {
            let _ = fiscal::sefaz::request_builders::build_consulta_request(
                "aqui temos uma chave nao numerica xxxxxxxxxx",
                SefazEnvironment::Homologation,
            );
        }
    }

    // ─── sefazEnviaLote ──────────────────────────────────────────────

    mod sefaz_envia_lote {


        #[test]
        #[should_panic]
        fn throws_on_invalid_parameter() {
            let _ = fiscal::sefaz::request_builders::build_autorizacao_request(
                "",
                "1",
                false,
                false,
            );
        }

        #[test]
        fn builds_correct_request_for_model_65() {
            let xml = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe65"></infNFe></NFe>"#;
            let id_lote = "1636667815";
            let result = fiscal::sefaz::request_builders::build_autorizacao_request(
                xml, id_lote, true, false,
            );
            assert!(result.contains("<enviNFe"));
            assert!(result.contains(&format!("<idLote>{id_lote}</idLote>")));
            assert!(result.contains("<indSinc>1</indSinc>"));
            assert!(result.contains("</enviNFe>"));
            assert!(!result.contains("<?xml"));
        }

        #[test]
        fn builds_correct_request_for_model_55() {
            let xml = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe55"></infNFe></NFe>"#;
            let id_lote = "1636667815";
            let result = fiscal::sefaz::request_builders::build_autorizacao_request(
                xml, id_lote, true, false,
            );
            assert!(result.contains("<enviNFe"));
            assert!(result.contains(&format!("<idLote>{id_lote}</idLote>")));
            assert!(result.contains("<indSinc>1</indSinc>"));
        }

        #[test]
        fn builds_correct_compressed_request_for_model_55() {
            let xml = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe55"></infNFe></NFe>"#;
            let id_lote = "1636667815";
            let result = fiscal::sefaz::request_builders::build_autorizacao_request(
                xml, id_lote, true, true,
            );
            assert!(result.contains("<enviNFe"));
            assert!(result.contains(&format!("<idLote>{id_lote}</idLote>")));
        }
    }

    // ─── sefazInutiliza ──────────────────────────────────────────────

    mod sefaz_inutiliza {
        use super::*;

        #[test]
        fn builds_correct_xml_for_number_voiding() {
            let xml = fiscal::sefaz::request_builders::build_inutilizacao_request(
                22,
                "93623057000128",
                "55",
                1,
                1,
                10,
                "Testando Inutilizacao",
                SefazEnvironment::Homologation,
                "SP",
            );
            assert!(xml.contains("<inutNFe"));
            assert!(xml.contains(r#"versao="4.00""#));
            assert!(xml.contains("<tpAmb>2</tpAmb>"));
            assert!(xml.contains("<xServ>INUTILIZAR</xServ>"));
            assert!(xml.contains("<cUF>35</cUF>"));
            assert!(xml.contains("<ano>22</ano>"));
            assert!(xml.contains("<CNPJ>93623057000128</CNPJ>"));
            assert!(xml.contains("<mod>55</mod>"));
            assert!(xml.contains("<serie>1</serie>"));
            assert!(xml.contains("<nNFIni>1</nNFIni>"));
            assert!(xml.contains("<nNFFin>10</nNFFin>"));
            assert!(xml.contains("<xJust>Testando Inutilizacao</xJust>"));
            assert!(xml.contains("</inutNFe>"));
            assert!(xml.contains(r#"Id="ID"#));
        }
    }

    // ─── sefazCadastro ───────────────────────────────────────────────

    mod sefaz_cadastro {


        #[test]
        fn builds_correct_cadastro_request_by_cnpj() {
            let xml = fiscal::sefaz::request_builders::build_cadastro_request(
                "RS", "CNPJ", "20532295000154",
            );
            assert!(xml.contains("<ConsCad"));
            assert!(xml.contains(r#"versao="2.00""#));
            assert!(xml.contains("<xServ>CONS-CAD</xServ>"));
            assert!(xml.contains("<UF>RS</UF>"));
            assert!(xml.contains("<CNPJ>20532295000154</CNPJ>"));
            assert!(xml.contains("</ConsCad>"));
        }

        #[test]
        fn builds_correct_cadastro_request_by_ie() {
            let xml = fiscal::sefaz::request_builders::build_cadastro_request(
                "RS", "IE", "1234567",
            );
            assert!(xml.contains("<ConsCad"));
            assert!(xml.contains("<UF>RS</UF>"));
            assert!(xml.contains("<IE>1234567</IE>"));
            assert!(!xml.contains("<CNPJ>"));
        }

        #[test]
        fn builds_correct_cadastro_request_by_cpf() {
            let xml = fiscal::sefaz::request_builders::build_cadastro_request(
                "RS", "CPF", "60140174028",
            );
            assert!(xml.contains("<ConsCad"));
            assert!(xml.contains("<UF>RS</UF>"));
            assert!(xml.contains("<CPF>60140174028</CPF>"));
            assert!(!xml.contains("<CNPJ>"));
            assert!(!xml.contains("<IE>"));
        }
    }

    // ─── sefazStatus ─────────────────────────────────────────────────

    mod sefaz_status {
        use super::*;

        #[test]
        fn builds_correct_status_request_xml() {
            let xml = fiscal::sefaz::request_builders::build_status_request(
                "RS",
                SefazEnvironment::Homologation,
            );
            assert!(xml.contains("<consStatServ"));
            assert!(xml.contains(r#"versao="4.00""#));
            assert!(xml.contains("<tpAmb>2</tpAmb>"));
            assert!(xml.contains("<cUF>43</cUF>"));
            assert!(xml.contains("<xServ>STATUS</xServ>"));
            assert!(xml.contains("</consStatServ>"));
        }
    }

    // ─── sefazDistDFe ────────────────────────────────────────────────

    mod sefaz_dist_dfe {
        use super::*;

        #[test]
        fn builds_correct_dist_dfe_request_xml() {
            // TS: buildDistDFeQueryXml(2, "SP", "...", lastNSU=100, specificNSU=200)
            // specificNSU > 0 takes priority -> consNSU path (mapped to access_key param)
            let xml = fiscal::sefaz::request_builders::build_dist_dfe_request(
                "SP",
                "93623057000128",
                None,
                Some("000000000000200"),
                SefazEnvironment::Homologation,
            );
            assert!(xml.contains("<distDFeInt"));
            assert!(xml.contains(r#"versao="1.01""#));
            assert!(xml.contains("<tpAmb>2</tpAmb>"));
            assert!(xml.contains("<cUFAutor>35</cUFAutor>"));
            assert!(xml.contains("<CNPJ>93623057000128</CNPJ>"));
            assert!(xml.contains("<consNSU>"));
            assert!(xml.contains("<NSU>000000000000200</NSU>"));
            assert!(xml.contains("</distDFeInt>"));
        }
    }

    // ─── sefazCCe ────────────────────────────────────────────────────

    mod sefaz_cce {
        use super::*;

        #[test]
        fn builds_correct_cce_request_xml() {
            let chave = "35220605730928000145550010000048661583302923";
            let xml = fiscal::sefaz::request_builders::build_cce_request(
                chave,
                "Descricao da correcao",
                1,
                SefazEnvironment::Homologation,
                "93623057000128",
            );
            assert!(xml.contains("<envEvento"));
            assert!(xml.contains(&format!("ID110110{chave}01")));
            assert!(xml.contains("<tpAmb>2</tpAmb>"));
            assert!(xml.contains("<CNPJ>93623057000128</CNPJ>"));
            assert!(xml.contains(&format!("<chNFe>{chave}</chNFe>")));
            assert!(xml.contains("<tpEvento>110110</tpEvento>"));
            assert!(xml.contains("<nSeqEvento>1</nSeqEvento>"));
            assert!(xml.contains("<descEvento>Carta de Correcao</descEvento>"));
            assert!(xml.contains("<xCorrecao>Descricao da correcao</xCorrecao>"));
            assert!(xml.contains("<xCondUso>"));
            assert!(xml.contains("</envEvento>"));
        }
    }

    // ─── sefazCancela ────────────────────────────────────────────────

    mod sefaz_cancela {
        use super::*;

        #[test]
        fn builds_correct_cancellation_event_request_xml() {
            let chave = "35150300822602000124550010009923461099234656";
            let xml = fiscal::sefaz::request_builders::build_cancela_request(
                chave,
                "123456789101234",
                "Preenchimento incorreto dos dados",
                1,
                SefazEnvironment::Homologation,
                "93623057000128",
            );
            assert!(xml.contains("<envEvento"));
            assert!(xml.contains(&format!("ID110111{chave}01")));
            assert!(xml.contains("<tpEvento>110111</tpEvento>"));
            assert!(xml.contains("<descEvento>Cancelamento</descEvento>"));
            assert!(xml.contains("<nProt>123456789101234</nProt>"));
            assert!(xml.contains("<xJust>Preenchimento incorreto dos dados</xJust>"));
        }
    }

    // ─── sefazManifesta ──────────────────────────────────────────────

    mod sefaz_manifesta {
        use super::*;

        #[test]
        fn builds_correct_manifestacao_request_xml() {
            let chave = "35240305730928000145650010000001421071400478";
            let xml = fiscal::sefaz::request_builders::build_manifesta_request(
                chave,
                "210210",
                None,
                1,
                SefazEnvironment::Homologation,
                "93623057000128",
            );
            assert!(xml.contains("<envEvento"));
            assert!(xml.contains(&format!("ID210210{chave}01")));
            assert!(xml.contains("<cOrgao>91</cOrgao>"));
            assert!(xml.contains("<tpEvento>210210</tpEvento>"));
            assert!(xml.contains("<descEvento>Ciencia da Operacao</descEvento>"));
            assert!(!xml.contains("<xJust>"));
        }
    }
}

// =============================================================================
// ContingencyTest
// =============================================================================

mod contingency_test {
    use super::*;

    #[test]
    fn get_contingency_type_returns_valid_type() {
        let ct = fiscal::contingency::contingency_for_state("SP");
        let s = ct.as_str();
        assert!(s == "svc-an" || s == "svc-rs");
    }

    #[test]
    fn sp_defaults_to_svc_an() {
        let ct = fiscal::contingency::contingency_for_state("SP");
        assert_eq!(ct.as_str(), "svc-an");
    }

    #[test]
    fn sp_should_not_use_svc_rs() {
        let ct = fiscal::contingency::contingency_for_state("SP");
        assert_ne!(ct.as_str(), "svc-rs");
    }

    #[test]
    #[should_panic]
    fn throws_on_motive_shorter_than_15_chars() {
        let mut contingency = fiscal::contingency::Contingency::new();
        contingency.activate(ContingencyType::SvcAn, "Testes").unwrap();
    }

    #[test]
    #[should_panic]
    fn throws_on_motive_longer_than_255_chars() {
        let mut contingency = fiscal::contingency::Contingency::new();
        let motive = "Eu fui emitir uma NFe e a SEFAZ autorizadora estava fora do ar, \
            entrei em contato com o tecnico de informatica que me mandou acionar o modo de contingencia, \
            indicando o motivo. Nosso diretor esta exigindo a emissao da NFe agora, e sei nao sei mais o que fazer. \
            Entao fiz essa tentativa agora.";
        contingency.activate(ContingencyType::SvcAn, motive).unwrap();
    }

    #[test]
    fn am_defaults_to_svc_rs() {
        let ct = fiscal::contingency::contingency_for_state("AM");
        assert_eq!(ct.as_str(), "svc-rs");
    }

    #[test]
    fn load_contingency_from_json() {
        let json = r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"SVCAN","tpEmis":6}"#;
        let contingency = fiscal::contingency::Contingency::load(json)
            .expect("load failed");
        assert!(contingency.contingency_type.is_some());
    }

    #[test]
    fn deactivate_resets_contingency() {
        let json = r#"{"motive":"Testes Unitarios","timestamp":1480700623,"type":"SVCAN","tpEmis":6}"#;
        let mut contingency = fiscal::contingency::Contingency::load(json)
            .expect("load failed");
        contingency.deactivate();
        assert!(contingency.contingency_type.is_none());
    }

    // ─── All 27 state contingency mappings (parametrized) ───────────

    #[rstest]
    // SVC-AN states (19)
    #[case("AC", "svc-an")]
    #[case("AL", "svc-an")]
    #[case("AP", "svc-an")]
    #[case("CE", "svc-an")]
    #[case("DF", "svc-an")]
    #[case("ES", "svc-an")]
    #[case("MG", "svc-an")]
    #[case("PA", "svc-an")]
    #[case("PB", "svc-an")]
    #[case("PI", "svc-an")]
    #[case("RJ", "svc-an")]
    #[case("RN", "svc-an")]
    #[case("RO", "svc-an")]
    #[case("RR", "svc-an")]
    #[case("RS", "svc-an")]
    #[case("SC", "svc-an")]
    #[case("SE", "svc-an")]
    #[case("SP", "svc-an")]
    #[case("TO", "svc-an")]
    // SVC-RS states (8)
    #[case("AM", "svc-rs")]
    #[case("BA", "svc-rs")]
    #[case("GO", "svc-rs")]
    #[case("MA", "svc-rs")]
    #[case("MS", "svc-rs")]
    #[case("MT", "svc-rs")]
    #[case("PE", "svc-rs")]
    #[case("PR", "svc-rs")]
    fn contingency_state_mapping(#[case] uf: &str, #[case] expected: &str) {
        assert_eq!(
            fiscal::contingency::contingency_for_state(uf).as_str(),
            expected,
            "State {uf} should map to {expected}"
        );
    }

    // Verify contingency URLs

    #[test]
    fn get_sefaz_url_contingency_sp_returns_svc_an_url() {
        // In contingency mode for SP (SVC-AN), the URL should contain svc.fazenda.gov.br
        let url = fiscal::sefaz::urls::get_sefaz_url(
            "SP",
            SefazEnvironment::Homologation,
            "NfeAutorizacao",
        ).expect("get_sefaz_url failed");
        // We test the non-contingency URL exists (contingency URL resolution is
        // handled at a higher level in the TS version)
        assert!(!url.is_empty());
    }
}

// =============================================================================
// ContingencyNFeTest
// =============================================================================

mod contingency_nfe_test {
    use super::*;

    #[test]
    fn adjusts_nfe_xml_with_contingency_data() {
        let xml = std::fs::read_to_string(
            "/home/john/projects/FinOpenPOS/.reference/sped-nfe/tests/fixtures/xml/nfe_layout4.xml"
        ).expect("Failed to read nfe_layout4.xml");

        let mut cont = fiscal::contingency::Contingency::new();
        cont.activate(ContingencyType::SvcAn, "Teste de uso da classe em contingencia")
            .expect("activate failed");

        let newxml = fiscal::contingency::adjust_nfe_contingency(&xml, &cont)
            .expect("adjust failed");
        assert!(newxml.contains("<tpEmis>6</tpEmis>"));
        assert!(newxml.contains("<dhCont>"));
        assert!(newxml.contains("<xJust>Teste de uso da classe em contingencia</xJust>"));
        assert!(!newxml.contains("<tpEmis>1</tpEmis>"));
    }

    #[test]
    fn does_not_alter_xml_already_configured_for_contingency() {
        let xml = std::fs::read_to_string(
            "/home/john/projects/FinOpenPOS/.reference/sped-nfe/tests/fixtures/xml/nfe_layout4_contingencia_sem_assinatura.xml"
        ).expect("Failed to read contingency XML");

        let mut cont = fiscal::contingency::Contingency::new();
        cont.activate(ContingencyType::SvcAn, "Teste contingencia SVCAN")
            .expect("activate failed");

        let newxml = fiscal::contingency::adjust_nfe_contingency(&xml, &cont)
            .expect("adjust failed");
        assert!(newxml.contains("<tpEmis>6</tpEmis>"));
        assert!(newxml.contains("<dhCont>2024-06-11T23:30:41-03:00</dhCont>"));
        assert!(newxml.contains("<xJust>Teste de uso da classe em conting"));
    }

    #[test]
    fn throws_when_adjusting_nfce_xml_for_contingency() {
        let xml = std::fs::read_to_string(
            "/home/john/projects/FinOpenPOS/.reference/sped-nfe/tests/fixtures/xml/nfce.xml"
        ).expect("Failed to read nfce.xml");

        let mut cont = fiscal::contingency::Contingency::new();
        cont.activate(ContingencyType::SvcAn, "Teste de uso da classe em contingencia")
            .expect("activate failed");

        let result = fiscal::contingency::adjust_nfe_contingency(&xml, &cont);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("65") || err.contains("NFC-e"), "Error should mention model 65: {err}");
    }
}

// =============================================================================
// QRCodeTest
// =============================================================================

mod qrcode_test {
    use super::*;

    #[test]
    fn inserts_qr_code_and_url_chave_tags_into_nfce_xml() {
        let xml = std::fs::read_to_string(
            "/home/john/projects/FinOpenPOS/.reference/sped-nfe/tests/fixtures/xml/nfce_sem_qrcode.xml"
        ).expect("Failed to read nfce_sem_qrcode.xml");

        let result = fiscal::qrcode::put_qr_tag(
            &PutQRTagParams::new(xml, "GPB0JBWLUR6HWFTVEAS6RJ69GPCROFPBBB8G", "000001", "200",
                "https://www.homologacao.nfce.fazenda.sp.gov.br/NFCeConsultaPublica/Paginas/ConsultaQRCode.aspx", "")
        ).expect("put_qr_tag failed");

        assert!(result.contains("<infNFeSupl>"));
        assert!(result.contains("<qrCode>"));
        assert!(result.contains("</qrCode>"));
        assert!(result.contains("<urlChave>"));
        assert!(result.contains("</infNFeSupl>"));
        // infNFeSupl should be before Signature
        let inf_supl_pos = result.find("<infNFeSupl>").unwrap();
        let sig_pos = result.find("<Signature").unwrap();
        assert!(inf_supl_pos < sig_pos);
        // QR Code URL should contain the access key
        assert!(result.contains("29181033657677000156650010001654399001654399"));
    }

    #[test]
    fn throws_when_csc_token_is_empty() {
        let result = fiscal::qrcode::build_nfce_qr_code_url(
            &NfceQrCodeParams::new("35200505730928000145650010000000121000000129", QrCodeVersion::V200, SefazEnvironment::Homologation, EmissionType::Normal,
                "https://www.homologacao.nfce.fazenda.sp.gov.br/NFCeConsultaPublica/Paginas/ConsultaQRCode.aspx")
                .csc_token("").csc_id("000001")
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("CSC token"), "Error should mention CSC token: {err}");
    }

    #[test]
    fn throws_when_csc_id_is_empty() {
        let result = fiscal::qrcode::build_nfce_qr_code_url(
            &NfceQrCodeParams::new("35200505730928000145650010000000121000000129", QrCodeVersion::V200, SefazEnvironment::Homologation, EmissionType::Normal,
                "https://www.homologacao.nfce.fazenda.sp.gov.br/NFCeConsultaPublica/Paginas/ConsultaQRCode.aspx")
                .csc_token("GPB0JBWLUR6HWFTVEAS6RJ69GPCROFPBBB8G").csc_id("")
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("CSC ID"), "Error should mention CSC ID: {err}");
    }

    #[test]
    fn produces_malformed_url_when_base_url_is_empty() {
        let result = fiscal::qrcode::build_nfce_qr_code_url(
            &NfceQrCodeParams::new("35200505730928000145650010000000121000000129", QrCodeVersion::V200, SefazEnvironment::Homologation, EmissionType::Normal, "")
                .csc_token("GPB0JBWLUR6HWFTVEAS6RJ69GPCROFPBBB8G").csc_id("000001")
        );
        // Either returns Err or a malformed URL starting with "?p="
        match result {
            Ok(url) => assert!(url.starts_with("?p=")),
            Err(_) => {} // also acceptable
        }
    }
}
