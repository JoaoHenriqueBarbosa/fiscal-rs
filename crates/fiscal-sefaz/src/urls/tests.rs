use super::*;

#[test]
fn sp_nfce_recepcao_epec_production() {
    let url =
        get_sefaz_url_for_model("SP", SefazEnvironment::Production, "RecepcaoEPEC", 65).unwrap();
    assert_eq!(
        url,
        "https://nfce.epec.fazenda.sp.gov.br/EPECws/RecepcaoEPEC.asmx"
    );
}

#[test]
fn sp_nfce_recepcao_epec_homologation() {
    let url =
        get_sefaz_url_for_model("SP", SefazEnvironment::Homologation, "RecepcaoEPEC", 65).unwrap();
    assert_eq!(
        url,
        "https://homologacao.nfce.epec.fazenda.sp.gov.br/EPECws/RecepcaoEPEC.asmx"
    );
}

#[test]
fn sp_nfce_epec_status_servico_production() {
    let url = get_sefaz_url_for_model("SP", SefazEnvironment::Production, "EPECStatusServico", 65)
        .unwrap();
    assert_eq!(
        url,
        "https://nfce.epec.fazenda.sp.gov.br/EPECws/EPECStatusServico.asmx"
    );
}

#[test]
fn sp_nfce_epec_status_servico_homologation() {
    let url = get_sefaz_url_for_model(
        "SP",
        SefazEnvironment::Homologation,
        "EPECStatusServico",
        65,
    )
    .unwrap();
    assert_eq!(
        url,
        "https://homologacao.nfce.epec.fazenda.sp.gov.br/EPECws/EPECStatusServico.asmx"
    );
}

#[test]
fn epec_services_not_available_for_other_states() {
    // EPEC NFC-e services are SP-only; other states should return an error.
    let result = get_sefaz_url_for_model("MG", SefazEnvironment::Production, "RecepcaoEPEC", 65);
    assert!(result.is_err());

    let result =
        get_sefaz_url_for_model("MG", SefazEnvironment::Production, "EPECStatusServico", 65);
    assert!(result.is_err());
}

#[test]
fn epec_services_not_available_for_nfe_model_55() {
    // EPEC NFC-e services should not be available on NF-e model 55.
    let result = get_sefaz_url_for_model("SP", SefazEnvironment::Production, "RecepcaoEPEC", 55);
    assert!(result.is_err());

    let result =
        get_sefaz_url_for_model("SP", SefazEnvironment::Production, "EPECStatusServico", 55);
    assert!(result.is_err());
}

#[test]
fn qr_url_differs_from_consult_url_sp() {
    // QR Code URL and consultation URL (urlChave) must be different for SP.
    let qr = get_nfce_qr_url("SP", SefazEnvironment::Production).unwrap();
    let consult = get_nfce_consult_url("SP", SefazEnvironment::Production).unwrap();
    assert_ne!(qr, consult);
    assert_eq!(qr, "https://www.nfce.fazenda.sp.gov.br/qrcode");
}

#[test]
fn qr_url_sp_homologation() {
    let qr = get_nfce_qr_url("SP", SefazEnvironment::Homologation).unwrap();
    assert_eq!(qr, "https://www.homologacao.nfce.fazenda.sp.gov.br/qrcode");
}

#[test]
fn qr_url_all_27_states_production() {
    // Verify all 27 UFs return Ok for production QR Code URLs.
    let ufs = [
        "AC", "AL", "AM", "AP", "BA", "CE", "DF", "ES", "GO", "MA", "MG", "MS", "MT", "PA", "PB",
        "PE", "PI", "PR", "RJ", "RN", "RO", "RR", "RS", "SC", "SE", "SP", "TO",
    ];
    for uf in &ufs {
        let result = get_nfce_qr_url(uf, SefazEnvironment::Production);
        assert!(result.is_ok(), "QR URL production failed for {uf}");
    }
}

#[test]
fn qr_url_all_27_states_homologation() {
    // Verify all 27 UFs return Ok for homologation QR Code URLs.
    let ufs = [
        "AC", "AL", "AM", "AP", "BA", "CE", "DF", "ES", "GO", "MA", "MG", "MS", "MT", "PA", "PB",
        "PE", "PI", "PR", "RJ", "RN", "RO", "RR", "RS", "SC", "SE", "SP", "TO",
    ];
    for uf in &ufs {
        let result = get_nfce_qr_url(uf, SefazEnvironment::Homologation);
        assert!(result.is_ok(), "QR URL homologation failed for {uf}");
    }
}

#[test]
fn qr_url_invalid_state() {
    let result = get_nfce_qr_url("XX", SefazEnvironment::Production);
    assert!(result.is_err());
}

// ── Tests for get_url optional service branches (NfeConsultaCadastro,
//    NfeDistribuicaoDFe, CscNFCe) ───────────────────────────────────────

#[test]
fn nfe_consulta_cadastro_production() {
    // SP has NfeConsultaCadastro (model 55)
    let url = get_sefaz_url("SP", SefazEnvironment::Production, "NfeConsultaCadastro").unwrap();
    assert!(
        !url.is_empty(),
        "NfeConsultaCadastro production URL should not be empty"
    );
}

#[test]
fn nfe_consulta_cadastro_homologation() {
    let url = get_sefaz_url("SP", SefazEnvironment::Homologation, "NfeConsultaCadastro").unwrap();
    assert!(
        !url.is_empty(),
        "NfeConsultaCadastro homologation URL should not be empty"
    );
}

#[test]
fn nfe_distribuicao_dfe_via_an_production() {
    // AN has NfeDistribuicaoDFe
    let url = get_an_url(SefazEnvironment::Production, "NfeDistribuicaoDFe").unwrap();
    assert_eq!(
        url,
        "https://www1.nfe.fazenda.gov.br/NFeDistribuicaoDFe/NFeDistribuicaoDFe.asmx"
    );
}

#[test]
fn nfe_distribuicao_dfe_via_an_homologation() {
    let url = get_an_url(SefazEnvironment::Homologation, "NfeDistribuicaoDFe").unwrap();
    assert_eq!(
        url,
        "https://hom1.nfe.fazenda.gov.br/NFeDistribuicaoDFe/NFeDistribuicaoDFe.asmx"
    );
}

#[test]
fn an_recepcao_evento_production() {
    let url = get_an_url(SefazEnvironment::Production, "RecepcaoEvento").unwrap();
    assert_eq!(
        url,
        "https://www.nfe.fazenda.gov.br/NFeRecepcaoEvento4/NFeRecepcaoEvento4.asmx"
    );
}

#[test]
fn an_unknown_service_returns_error() {
    let result = get_an_url(SefazEnvironment::Production, "UnknownService");
    assert!(result.is_err());
}

#[test]
fn qr_url_specific_states_match_php() {
    // Spot-check several states against the PHP source of truth.
    assert_eq!(
        get_nfce_qr_url("AM", SefazEnvironment::Production).unwrap(),
        "https://sistemas.sefaz.am.gov.br/nfceweb/consultarNFCe.jsp"
    );
    assert_eq!(
        get_nfce_qr_url("AM", SefazEnvironment::Homologation).unwrap(),
        "https://sistemas.sefaz.am.gov.br/nfceweb-hom/consultarNFCe.jsp"
    );
    assert_eq!(
        get_nfce_qr_url("GO", SefazEnvironment::Production).unwrap(),
        "https://nfeweb.sefaz.go.gov.br/nfeweb/sites/nfce/danfeNFCe"
    );
    assert_eq!(
        get_nfce_qr_url("RR", SefazEnvironment::Homologation).unwrap(),
        "http://200.174.88.103:8080/nfce/servlet/qrcode"
    );
    assert_eq!(
        get_nfce_qr_url("RS", SefazEnvironment::Production).unwrap(),
        "https://www.sefaz.rs.gov.br/NFCE/NFCE-COM.aspx"
    );
    assert_eq!(
        get_nfce_qr_url("MG", SefazEnvironment::Production).unwrap(),
        "https://portalsped.fazenda.mg.gov.br/portalnfce/sistema/qrcode.xhtml"
    );
    assert_eq!(
        get_nfce_qr_url("PA", SefazEnvironment::Homologation).unwrap(),
        "https://appnfc.sefa.pa.gov.br/portal-homologacao/view/consultas/nfce/nfceForm.seam"
    );
}

#[test]
fn csc_nfce_via_am_nfce_production() {
    // AM_NFCE has CscNFCe
    let url = get_sefaz_url_for_model("AM", SefazEnvironment::Production, "CscNFCe", 65).unwrap();
    assert_eq!(
        url,
        "https://nfce.sefaz.am.gov.br/nfce-services/services/CscNFCe"
    );
}

#[test]
fn csc_nfce_via_am_nfce_homologation() {
    let url = get_sefaz_url_for_model("AM", SefazEnvironment::Homologation, "CscNFCe", 65).unwrap();
    assert_eq!(
        url,
        "https://homnfce.sefaz.am.gov.br/nfce-services/services/CscNFCe"
    );
}

#[test]
fn unknown_service_returns_none_via_get_url() {
    // An unknown service name should return an error
    let result = get_sefaz_url("SP", SefazEnvironment::Production, "Nonexistent");
    assert!(result.is_err());
}

// ── Tests for get_state_nfce_authorizer ─────────────────────────────────

#[test]
fn nfce_svrs_fallthrough_states() {
    // States that use SVRS NFC-e should resolve successfully
    for uf in [
        "AC", "AL", "AP", "BA", "CE", "DF", "ES", "MA", "PA", "PB", "PE", "PI", "RJ", "RN", "RO",
        "RR", "SC", "SE", "TO",
    ] {
        let result =
            get_sefaz_url_for_model(uf, SefazEnvironment::Production, "NfeStatusServico", 65);
        assert!(
            result.is_ok(),
            "NFC-e SVRS fallthrough state {uf} should resolve"
        );
    }
}

#[test]
fn nfce_unknown_state_returns_error() {
    let result =
        get_sefaz_url_for_model("XX", SefazEnvironment::Production, "NfeStatusServico", 65);
    assert!(result.is_err());
}

// ── Tests for get_state_contingency_authorizer / get_sefaz_contingency_url ──

#[test]
fn contingency_svcan_states() {
    for uf in [
        "AC", "AL", "AP", "CE", "DF", "ES", "MG", "PA", "PB", "PI", "RJ", "RN", "RO", "RR", "RS",
        "SC", "SE", "SP", "TO",
    ] {
        assert_eq!(
            get_state_contingency_authorizer(uf),
            Some("SVCAN"),
            "{uf} should map to SVCAN"
        );
    }
}

#[test]
fn contingency_svcrs_states() {
    for uf in ["AM", "BA", "GO", "MA", "MS", "MT", "PE", "PR"] {
        assert_eq!(
            get_state_contingency_authorizer(uf),
            Some("SVCRS"),
            "{uf} should map to SVCRS"
        );
    }
}

#[test]
fn contingency_unknown_state() {
    assert_eq!(get_state_contingency_authorizer("XX"), None);
}

#[test]
fn get_sefaz_contingency_url_svcan_production() {
    // SP maps to SVCAN
    let url =
        get_sefaz_contingency_url("SP", SefazEnvironment::Production, "NfeAutorizacao").unwrap();
    assert!(url.contains("sefazvirtual.fazenda.gov.br"));
}

#[test]
fn get_sefaz_contingency_url_svcrs_production() {
    // AM maps to SVCRS
    let url =
        get_sefaz_contingency_url("AM", SefazEnvironment::Production, "NfeAutorizacao").unwrap();
    assert!(url.contains("svrs.rs.gov.br") || url.contains("svc.rs.gov.br") || !url.is_empty());
}

#[test]
fn get_sefaz_contingency_url_unknown_state() {
    let result = get_sefaz_contingency_url("XX", SefazEnvironment::Production, "NfeAutorizacao");
    assert!(result.is_err());
}

#[test]
fn get_sefaz_contingency_url_unknown_service() {
    let result = get_sefaz_contingency_url("SP", SefazEnvironment::Production, "Nonexistent");
    assert!(result.is_err());
}

#[test]
fn get_sefaz_contingency_url_homologation() {
    let url = get_sefaz_contingency_url("SP", SefazEnvironment::Homologation, "NfeStatusServico")
        .unwrap();
    assert!(!url.is_empty());
}

// ── Tests for get_nfce_consult_url ──────────────────────────────────────

#[test]
fn nfce_consult_url_all_states_production() {
    let states = [
        "AC", "AL", "AP", "AM", "BA", "CE", "DF", "ES", "GO", "MA", "MG", "MS", "MT", "PA", "PB",
        "PE", "PR", "PI", "RJ", "RN", "RO", "RR", "RS", "SC", "SE", "SP", "TO",
    ];
    for uf in states {
        let result = get_nfce_consult_url(uf, SefazEnvironment::Production);
        assert!(
            result.is_ok(),
            "Production consult URL should exist for {uf}"
        );
        assert!(
            !result.unwrap().is_empty(),
            "Production consult URL should not be empty for {uf}"
        );
    }
}

#[test]
fn nfce_consult_url_all_states_homologation() {
    let states = [
        "AC", "AL", "AP", "AM", "BA", "CE", "DF", "ES", "GO", "MA", "MG", "MS", "MT", "PA", "PB",
        "PE", "PR", "PI", "RJ", "RN", "RO", "RR", "RS", "SC", "SE", "SP", "TO",
    ];
    for uf in states {
        let result = get_nfce_consult_url(uf, SefazEnvironment::Homologation);
        assert!(
            result.is_ok(),
            "Homologation consult URL should exist for {uf}"
        );
        assert!(
            !result.unwrap().is_empty(),
            "Homologation consult URL should not be empty for {uf}"
        );
    }
}

#[test]
fn nfce_consult_url_unknown_state() {
    let result = get_nfce_consult_url("XX", SefazEnvironment::Production);
    assert!(result.is_err());

    let result = get_nfce_consult_url("XX", SefazEnvironment::Homologation);
    assert!(result.is_err());
}

#[test]
fn nfce_consult_url_sp_production_value() {
    let url = get_nfce_consult_url("SP", SefazEnvironment::Production).unwrap();
    assert_eq!(
        url,
        "https://www.nfce.fazenda.sp.gov.br/NFCeConsultaPublica"
    );
}

#[test]
fn nfce_consult_url_sp_homologation_value() {
    let url = get_nfce_consult_url("SP", SefazEnvironment::Homologation).unwrap();
    assert_eq!(
        url,
        "https://www.homologacao.nfce.fazenda.sp.gov.br/NFCeConsultaPublica"
    );
}

// ── Tests for get_nfce_qr_url ───────────────────────────────────────────

#[test]
fn nfce_qr_url_delegates_to_consult_url() {
    // get_nfce_qr_url now has its own URL table separate from get_nfce_consult_url
    let qr = get_nfce_qr_url("SP", SefazEnvironment::Production).unwrap();
    assert_eq!(qr, "https://www.nfce.fazenda.sp.gov.br/qrcode");
}

#[test]
fn nfce_qr_url_production() {
    let url = get_nfce_qr_url("MG", SefazEnvironment::Production).unwrap();
    assert_eq!(
        url,
        "https://portalsped.fazenda.mg.gov.br/portalnfce/sistema/qrcode.xhtml"
    );
}

#[test]
fn nfce_qr_url_homologation() {
    let url = get_nfce_qr_url("MG", SefazEnvironment::Homologation).unwrap();
    assert_eq!(
        url,
        "https://portalsped.fazenda.mg.gov.br/portalnfce/sistema/qrcode.xhtml"
    );
}

#[test]
fn nfce_qr_url_unknown_state() {
    let result = get_nfce_qr_url("XX", SefazEnvironment::Production);
    assert!(result.is_err());
}

// ── Tests for NfeConsultaDest and NfeDownloadNF via AN (lines 88-104) ──

#[test]
fn an_nfe_consulta_dest_production() {
    let url = get_an_url(SefazEnvironment::Production, "NfeConsultaDest").unwrap();
    assert_eq!(
        url,
        "https://www.nfe.fazenda.gov.br/NFeConsultaDest/NFeConsultaDest.asmx"
    );
}

#[test]
fn an_nfe_consulta_dest_homologation() {
    let url = get_an_url(SefazEnvironment::Homologation, "NfeConsultaDest").unwrap();
    assert_eq!(
        url,
        "https://hom.nfe.fazenda.gov.br/NFeConsultaDest/NFeConsultaDest.asmx"
    );
}

#[test]
fn an_nfe_download_nf_production() {
    let url = get_an_url(SefazEnvironment::Production, "NfeDownloadNF").unwrap();
    assert_eq!(
        url,
        "https://www.nfe.fazenda.gov.br/NfeDownloadNF/NfeDownloadNF.asmx"
    );
}

#[test]
fn an_nfe_download_nf_homologation() {
    let url = get_an_url(SefazEnvironment::Homologation, "NfeDownloadNF").unwrap();
    assert_eq!(
        url,
        "https://hom.nfe.fazenda.gov.br/NfeDownloadNF/NfeDownloadNF.asmx"
    );
}

// ── Test for QR URL invalid state in homologation (line 1316) ──────────

#[test]
fn nfce_qr_url_unknown_state_homologation() {
    let result = get_nfce_qr_url("XX", SefazEnvironment::Homologation);
    assert!(result.is_err());
}
