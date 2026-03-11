use fiscal::qrcode::{build_nfce_consult_url, build_nfce_qr_code_url};
use fiscal::types::{EmissionType, NfceQrCodeParams, QrCodeVersion, SefazEnvironment};

// ── buildNfceQrCodeUrl — v200 ───────────────────────────────────────────────

mod build_nfce_qr_code_url_v200 {
    use super::*;

    #[test]
    fn generates_online_qr_code_url_tp_emis_1() {
        let url = build_nfce_qr_code_url(&NfceQrCodeParams {
            version: QrCodeVersion::V200,
            access_key: "35260112345678000199650010000000011123456780".into(),
            environment: SefazEnvironment::Homologation,
            emission_type: EmissionType::Normal,
            csc_id: Some("000001".into()),
            csc_token: Some("ABCDEF123456".into()),
            qr_code_base_url:
                "https://www.homologacao.nfce.fazenda.sp.gov.br/qrcode".into(),
            issued_at: None,
            total_value: None,
            total_icms: None,
            digest_value: None,
            dest_document: None,
            dest_id_type: None,
        })
        .expect("should build v200 online QR code URL");

        assert!(url.contains(
            "https://www.homologacao.nfce.fazenda.sp.gov.br/qrcode?p="
        ));
        assert!(url.contains("35260112345678000199650010000000011123456780"));
        assert!(url.contains("|2|")); // version
        // Should end with hex SHA-1 hash (40 uppercase hex chars)
        let after_last_pipe = url.rsplit('|').next().unwrap();
        assert_eq!(after_last_pipe.len(), 40);
        assert!(after_last_pipe.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_lowercase()));
    }

    #[test]
    fn generates_offline_qr_code_url_tp_emis_9() {
        let url = build_nfce_qr_code_url(&NfceQrCodeParams {
            version: QrCodeVersion::V200,
            access_key: "35260112345678000199650010000000011123456780".into(),
            environment: SefazEnvironment::Homologation,
            emission_type: EmissionType::Offline,
            csc_id: Some("000001".into()),
            csc_token: Some("ABCDEF123456".into()),
            qr_code_base_url: "https://example.com/qrcode".into(),
            issued_at: Some("2026-01-15T10:30:00-03:00".into()),
            total_value: Some("100.00".into()),
            total_icms: None,
            digest_value: Some("abc123digest==".into()),
            dest_document: None,
            dest_id_type: None,
        })
        .expect("should build v200 offline QR code URL");

        assert!(url.contains("?p="));
        assert!(url.contains("|2|")); // version
    }

    #[test]
    fn throws_without_csc_for_v200() {
        let result = build_nfce_qr_code_url(&NfceQrCodeParams {
            version: QrCodeVersion::V200,
            access_key: "35260112345678000199650010000000011123456780".into(),
            environment: SefazEnvironment::Homologation,
            emission_type: EmissionType::Normal,
            csc_id: Some("".into()),
            csc_token: Some("".into()),
            qr_code_base_url: "https://example.com/qrcode".into(),
            issued_at: None,
            total_value: None,
            total_icms: None,
            digest_value: None,
            dest_document: None,
            dest_id_type: None,
        });

        assert!(result.is_err(), "v200 without CSC should return Err");
    }
}

// ── buildNfceQrCodeUrl — v300 ───────────────────────────────────────────────

mod build_nfce_qr_code_url_v300 {
    use super::*;

    #[test]
    fn generates_online_qr_code_url_no_csc_needed() {
        let url = build_nfce_qr_code_url(&NfceQrCodeParams {
            version: QrCodeVersion::V300,
            access_key: "35260112345678000199650010000000011123456780".into(),
            environment: SefazEnvironment::Homologation,
            emission_type: EmissionType::Normal,
            qr_code_base_url: "https://example.com/qrcode".into(),
            csc_token: None,
            csc_id: None,
            issued_at: None,
            total_value: None,
            total_icms: None,
            digest_value: None,
            dest_document: None,
            dest_id_type: None,
        })
        .expect("should build v300 online QR code URL");

        assert!(url.contains("?p="));
        assert!(url.contains("|3|")); // version 3
        assert!(url.contains("|2"));  // tpAmb
    }
}

// ── buildNfceConsultUrl ─────────────────────────────────────────────────────

mod build_nfce_consult_url_tests {
    use super::*;

    #[test]
    fn builds_consultation_url() {
        let url = build_nfce_consult_url(
            "https://www.homologacao.nfce.fazenda.sp.gov.br/consulta",
            "35260112345678000199650010000000011123456780",
            SefazEnvironment::Homologation,
        );

        assert!(url.contains(
            "https://www.homologacao.nfce.fazenda.sp.gov.br/consulta"
        ));
        assert!(url.contains("35260112345678000199650010000000011123456780"));
    }
}
