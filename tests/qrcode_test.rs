use fiscal::qrcode::{build_nfce_consult_url, build_nfce_qr_code_url};
use fiscal::types::{EmissionType, NfceQrCodeParams, QrCodeVersion, SefazEnvironment};

// ── buildNfceQrCodeUrl — v200 ───────────────────────────────────────────────

mod build_nfce_qr_code_url_v200 {
    use super::*;

    #[test]
    fn generates_online_qr_code_url_tp_emis_1() {
        let url = build_nfce_qr_code_url(
            &NfceQrCodeParams::new(
                "35260112345678000199650010000000011123456780",
                QrCodeVersion::V200,
                SefazEnvironment::Homologation,
                EmissionType::Normal,
                "https://www.homologacao.nfce.fazenda.sp.gov.br/qrcode",
            )
            .csc_id("000001")
            .csc_token("ABCDEF123456"),
        )
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
        let url = build_nfce_qr_code_url(
            &NfceQrCodeParams::new(
                "35260112345678000199650010000000011123456780",
                QrCodeVersion::V200,
                SefazEnvironment::Homologation,
                EmissionType::Offline,
                "https://example.com/qrcode",
            )
            .csc_id("000001")
            .csc_token("ABCDEF123456")
            .issued_at("2026-01-15T10:30:00-03:00")
            .total_value("100.00")
            .digest_value("abc123digest=="),
        )
        .expect("should build v200 offline QR code URL");

        assert!(url.contains("?p="));
        assert!(url.contains("|2|")); // version
    }

    #[test]
    fn throws_without_csc_for_v200() {
        let result = build_nfce_qr_code_url(
            &NfceQrCodeParams::new(
                "35260112345678000199650010000000011123456780",
                QrCodeVersion::V200,
                SefazEnvironment::Homologation,
                EmissionType::Normal,
                "https://example.com/qrcode",
            )
            .csc_id("")
            .csc_token(""),
        );

        assert!(result.is_err(), "v200 without CSC should return Err");
    }
}

// ── buildNfceQrCodeUrl — v300 ───────────────────────────────────────────────

mod build_nfce_qr_code_url_v300 {
    use super::*;

    #[test]
    fn generates_online_qr_code_url_no_csc_needed() {
        let url = build_nfce_qr_code_url(
            &NfceQrCodeParams::new(
                "35260112345678000199650010000000011123456780",
                QrCodeVersion::V300,
                SefazEnvironment::Homologation,
                EmissionType::Normal,
                "https://example.com/qrcode",
            ),
        )
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
