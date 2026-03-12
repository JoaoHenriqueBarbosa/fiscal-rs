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
        let (pfx_bytes, passphrase) = generate_test_pfx();
        let cert = fiscal::certificate::load_certificate(&pfx_bytes, &passphrase).unwrap();
        let signed =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();

        // Verify complete signature structure
        assert!(signed.contains("<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\">"));
        assert!(signed.contains("</Signature>"));
        assert!(signed.contains("<X509Certificate>"));
        assert!(signed.contains("<DigestValue>"));
        assert!(signed.contains("<SignatureValue>"));

        // SignedInfo must NOT have its own xmlns (inherits from Signature)
        assert!(
            !signed.contains("<SignedInfo xmlns="),
            "SignedInfo must not have redundant xmlns declaration"
        );
        assert!(signed.contains("<SignedInfo>"));
    }

    #[test]
    fn c14n_sorts_attributes_alphabetically() {
        // The sample XML has: versao="4.00" Id="NFe..."
        // C14N requires: Id="NFe..." versao="4.00" (alphabetical order)
        let (pfx_bytes, passphrase) = generate_test_pfx();
        let cert = fiscal::certificate::load_certificate(&pfx_bytes, &passphrase).unwrap();
        let signed =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();

        // The DigestValue must be computed from the C14N canonical form where
        // attributes are sorted. We verify this by checking the same XML signed
        // twice with different attribute order gives the SAME DigestValue.
        let xml_alt_order = sample_xml().replace(
            r#"versao="4.00" Id="NFe35260112345678000199650010000000011123456780""#,
            r#"Id="NFe35260112345678000199650010000000011123456780" versao="4.00""#,
        );
        let signed_alt =
            fiscal::certificate::sign_xml(&xml_alt_order, &cert.private_key, &cert.certificate)
                .unwrap();

        let extract_digest = |xml: &str| -> String {
            let start = xml.find("<DigestValue>").unwrap() + "<DigestValue>".len();
            let end = xml[start..].find("</DigestValue>").unwrap() + start;
            xml[start..end].to_string()
        };

        assert_eq!(
            extract_digest(&signed),
            extract_digest(&signed_alt),
            "C14N must produce identical DigestValue regardless of attribute order in input"
        );
    }

    #[test]
    fn c14n_preserves_namespace_before_attributes() {
        // In C14N, xmlns declarations come before regular attributes.
        // Verify the signed XML has correct attribute order on infNFe.
        let (pfx_bytes, passphrase) = generate_test_pfx();
        let cert = fiscal::certificate::load_certificate(&pfx_bytes, &passphrase).unwrap();
        let signed =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();

        // The infNFe in the output should still have xmlns before Id/versao
        // (it's the output XML, not the canonical form, but the original input preserves this)
        let inf_start = signed.find("<infNFe").unwrap();
        let inf_tag_end = signed[inf_start..].find('>').unwrap() + inf_start;
        let tag = &signed[inf_start..=inf_tag_end];

        let xmlns_pos = tag.find("xmlns=").expect("xmlns not found on infNFe");
        let id_pos = tag.find("Id=").expect("Id not found on infNFe");
        assert!(
            xmlns_pos < id_pos,
            "xmlns must appear before Id in the output tag"
        );
    }

    #[test]
    fn deterministic_digest_for_known_xml() {
        // Sign the same XML with the same key and verify the DigestValue
        // is deterministic (digest depends only on canonical content).
        let (pfx_bytes, passphrase) = generate_test_pfx();
        let cert = fiscal::certificate::load_certificate(&pfx_bytes, &passphrase).unwrap();

        let signed1 =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();
        let signed2 =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();

        let extract_digest = |xml: &str| -> String {
            let start = xml.find("<DigestValue>").unwrap() + "<DigestValue>".len();
            let end = xml[start..].find("</DigestValue>").unwrap() + start;
            xml[start..end].to_string()
        };

        assert_eq!(
            extract_digest(&signed1),
            extract_digest(&signed2),
            "DigestValue must be deterministic for same input"
        );

        // SignatureValue must also be deterministic (RSA with PKCS#1 v1.5 is deterministic)
        let extract_sigval = |xml: &str| -> String {
            let start = xml.find("<SignatureValue>").unwrap() + "<SignatureValue>".len();
            let end = xml[start..].find("</SignatureValue>").unwrap() + start;
            xml[start..end].to_string()
        };

        assert_eq!(
            extract_sigval(&signed1),
            extract_sigval(&signed2),
            "SignatureValue must be deterministic for same input and key"
        );
    }

    #[test]
    fn self_verifies_signature_with_openssl() {
        // Independently verify the RSA-SHA1 SignatureValue using OpenSSL.
        // This proves the signing round-trip is correct: the SignatureValue
        // can be verified against the canonical SignedInfo using the same key.
        use openssl::base64;
        use openssl::hash::MessageDigest;
        use openssl::pkey::PKey;
        use openssl::sign::Verifier;

        let (pfx_bytes, passphrase) = generate_test_pfx();
        let cert_data = fiscal::certificate::load_certificate(&pfx_bytes, &passphrase).unwrap();
        let signed = fiscal::certificate::sign_xml(
            &sample_xml(),
            &cert_data.private_key,
            &cert_data.certificate,
        )
        .unwrap();

        // Extract SignedInfo (without xmlns) and reconstruct canonical form
        // (with xmlns, as SEFAZ does during verification — C14N includes
        // in-scope namespace from parent <Signature xmlns="...">)
        let si_start = signed.find("<SignedInfo>").unwrap();
        let si_end = signed.find("</SignedInfo>").unwrap() + "</SignedInfo>".len();
        let signed_info = &signed[si_start..si_end];
        let canonical_si = signed_info.replacen(
            "<SignedInfo>",
            "<SignedInfo xmlns=\"http://www.w3.org/2000/09/xmldsig#\">",
            1,
        );

        // Extract SignatureValue
        let sigval_start = signed.find("<SignatureValue>").unwrap() + "<SignatureValue>".len();
        let sigval_end =
            signed[sigval_start..].find("</SignatureValue>").unwrap() + sigval_start;
        let signature_bytes = base64::decode_block(&signed[sigval_start..sigval_end]).unwrap();

        // Verify RSA-SHA1 signature
        let pkey = PKey::private_key_from_pem(cert_data.private_key.as_bytes()).unwrap();
        let mut verifier = Verifier::new(MessageDigest::sha1(), &pkey).unwrap();
        verifier.update(canonical_si.as_bytes()).unwrap();
        assert!(
            verifier.verify(&signature_bytes).unwrap(),
            "RSA-SHA1 signature must verify against canonical SignedInfo"
        );
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
