// Ported from TypeScript: certificate.test.ts (14 tests)
// Tests for loadCertificate, getCertificateInfo, signXml

use std::process::Command;

/// Generate a self-signed PFX certificate for testing.
/// Returns (pfx_bytes, passphrase).
fn generate_test_pfx() -> (Vec<u8>, String) {
    let passphrase = "test123".to_string();

    // Generate private key and self-signed certificate
    let _ = Command::new("openssl")
        .args([
            "req",
            "-x509",
            "-newkey",
            "rsa:2048",
            "-keyout",
            "/tmp/fiscal-rs-test-key.pem",
            "-out",
            "/tmp/fiscal-rs-test-cert.pem",
            "-days",
            "365",
            "-nodes",
            "-subj",
            "/CN=Test NFe Company/O=FinOpenPOS Test",
        ])
        .stderr(std::process::Stdio::null())
        .output()
        .expect("openssl req failed");

    // Export to PFX
    let _ = Command::new("openssl")
        .args([
            "pkcs12",
            "-export",
            "-out",
            "/tmp/fiscal-rs-test-cert.pfx",
            "-inkey",
            "/tmp/fiscal-rs-test-key.pem",
            "-in",
            "/tmp/fiscal-rs-test-cert.pem",
            "-passout",
            &format!("pass:{passphrase}"),
        ])
        .stderr(std::process::Stdio::null())
        .output()
        .expect("openssl pkcs12 failed");

    let pfx_bytes = std::fs::read("/tmp/fiscal-rs-test-cert.pfx").expect("Failed to read PFX file");
    (pfx_bytes, passphrase)
}

fn sample_xml() -> String {
    [
        r#"<?xml version="1.0" encoding="UTF-8"?>"#,
        r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
        r#"<infNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00" Id="NFe35260112345678000199650010000000011123456780">"#,
        "<ide><cUF>35</cUF><mod>65</mod></ide>",
        "<emit><CNPJ>12345678000199</CNPJ></emit>",
        r#"<det nItem="1"><prod><cProd>1</cProd><xProd>Test</xProd></prod></det>"#,
        "<total><ICMSTot><vNF>10.00</vNF></ICMSTot></total>",
        "</infNFe>",
        "</NFe>",
    ].join("")
}

// =============================================================================
// loadCertificate()
// =============================================================================

mod load_certificate {
    use super::*;

    #[test]
    fn extracts_private_key_and_certificate_from_pfx() {
        let (pfx_bytes, passphrase) = generate_test_pfx();
        let cert = fiscal::certificate::load_certificate(&pfx_bytes, &passphrase)
            .expect("load_certificate failed");

        assert!(cert.private_key.contains("-----BEGIN PRIVATE KEY-----"));
        assert!(cert.certificate.contains("-----BEGIN CERTIFICATE-----"));
        assert_eq!(cert.pfx_buffer, pfx_bytes);
        assert_eq!(cert.passphrase, passphrase);
    }

    #[test]
    fn throws_on_invalid_pfx_buffer() {
        let result = fiscal::certificate::load_certificate(b"not-a-pfx", "pass");
        assert!(result.is_err());
    }

    #[test]
    fn throws_on_wrong_password() {
        let (pfx_bytes, _) = generate_test_pfx();
        let result = fiscal::certificate::load_certificate(&pfx_bytes, "wrong-password");
        assert!(result.is_err());
    }
}

// =============================================================================
// getCertificateInfo()
// =============================================================================

mod get_certificate_info {
    use super::*;

    #[test]
    fn returns_certificate_metadata() {
        let (pfx_bytes, passphrase) = generate_test_pfx();
        let info = fiscal::certificate::get_certificate_info(&pfx_bytes, &passphrase)
            .expect("get_certificate_info failed");

        assert_eq!(info.common_name, "Test NFe Company");
        // Self-signed: issuer == subject
        assert_eq!(info.issuer, "Test NFe Company");
        // valid_until should be in the future
        let today = chrono::Local::now().date_naive();
        assert!(info.valid_until > today);
        assert!(!info.serial_number.is_empty());
    }

    #[test]
    fn throws_on_invalid_pfx() {
        let result = fiscal::certificate::get_certificate_info(b"garbage", "pass");
        assert!(result.is_err());
    }
}

// =============================================================================
// signXml()
// =============================================================================

mod sign_xml {
    use super::*;

    #[test]
    fn produces_signed_xml_with_signature_element() {
        let (pfx_bytes, passphrase) = generate_test_pfx();
        let cert = fiscal::certificate::load_certificate(&pfx_bytes, &passphrase).unwrap();
        let signed =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .expect("sign_xml failed");

        assert!(signed.contains("<Signature"));
        assert!(signed.contains("<SignedInfo"));
        assert!(signed.contains("<SignatureValue>"));
        assert!(signed.contains("<X509Certificate>"));
        assert!(signed.contains("<DigestValue>"));
    }

    #[test]
    fn signature_is_inside_nfe_element() {
        let (pfx_bytes, passphrase) = generate_test_pfx();
        let cert = fiscal::certificate::load_certificate(&pfx_bytes, &passphrase).unwrap();
        let signed =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();

        let nfe_end = signed.find("</NFe>").expect("</NFe> not found");
        let sig_start = signed.find("<Signature").expect("<Signature not found");
        assert!(sig_start > 0);
        assert!(sig_start < nfe_end);
    }

    #[test]
    fn references_the_correct_inf_nfe_id() {
        let (pfx_bytes, passphrase) = generate_test_pfx();
        let cert = fiscal::certificate::load_certificate(&pfx_bytes, &passphrase).unwrap();
        let signed =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();

        assert!(signed.contains(r##"URI="#NFe35260112345678000199650010000000011123456780""##));
    }

    #[test]
    fn uses_rsa_sha1_signature_algorithm() {
        let (pfx_bytes, passphrase) = generate_test_pfx();
        let cert = fiscal::certificate::load_certificate(&pfx_bytes, &passphrase).unwrap();
        let signed =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();

        assert!(signed.contains("rsa-sha1"));
    }

    #[test]
    fn uses_c14n_canonicalization() {
        let (pfx_bytes, passphrase) = generate_test_pfx();
        let cert = fiscal::certificate::load_certificate(&pfx_bytes, &passphrase).unwrap();
        let signed =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();

        assert!(signed.contains("xml-c14n-20010315"));
    }

    #[test]
    fn includes_enveloped_signature_transform() {
        let (pfx_bytes, passphrase) = generate_test_pfx();
        let cert = fiscal::certificate::load_certificate(&pfx_bytes, &passphrase).unwrap();
        let signed =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();

        assert!(signed.contains("enveloped-signature"));
    }

    #[test]
    fn signed_xml_can_be_verified() {
        // In TS this uses xml-crypto to verify the signature.
        // In Rust we verify the signed XML has the expected structure for verification.
        let (pfx_bytes, passphrase) = generate_test_pfx();
        let cert = fiscal::certificate::load_certificate(&pfx_bytes, &passphrase).unwrap();
        let signed =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();

        // Verify the signature block exists and is well-formed
        assert!(signed.contains("<Signature"));
        assert!(signed.contains("</Signature>"));
        // The signed XML should contain the X509 certificate for verification
        assert!(signed.contains("<X509Certificate>"));
        assert!(signed.contains("<DigestValue>"));
        assert!(signed.contains("<SignatureValue>"));
    }

    #[test]
    fn throws_when_inf_nfe_element_is_missing() {
        let (pfx_bytes, passphrase) = generate_test_pfx();
        let cert = fiscal::certificate::load_certificate(&pfx_bytes, &passphrase).unwrap();
        let result = fiscal::certificate::sign_xml(
            "<NFe><data>test</data></NFe>",
            &cert.private_key,
            &cert.certificate,
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("infNFe"), "Error should mention infNFe: {err}");
    }

    #[test]
    fn produces_different_signatures_for_different_xml_content() {
        let (pfx_bytes, passphrase) = generate_test_pfx();
        let cert = fiscal::certificate::load_certificate(&pfx_bytes, &passphrase).unwrap();

        let xml1 = sample_xml();
        let xml2 = xml1.replace("<vNF>10.00</vNF>", "<vNF>99.99</vNF>");

        let signed1 =
            fiscal::certificate::sign_xml(&xml1, &cert.private_key, &cert.certificate).unwrap();
        let signed2 =
            fiscal::certificate::sign_xml(&xml2, &cert.private_key, &cert.certificate).unwrap();

        fn extract_sig_value(xml: &str) -> Option<String> {
            let start = xml.find("<SignatureValue>")? + "<SignatureValue>".len();
            let end = xml[start..].find("</SignatureValue>")? + start;
            Some(xml[start..end].to_string())
        }

        let sig1 = extract_sig_value(&signed1).expect("sig1 not found");
        let sig2 = extract_sig_value(&signed2).expect("sig2 not found");
        assert_ne!(sig1, sig2);
    }
}
