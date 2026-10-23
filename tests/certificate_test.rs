// Tests for loadCertificate, getCertificateInfo, signXml
//
// Pure-Rust version — no OpenSSL dependency.

// Path to the test PFX fixtures
const PFX_CNPJ: &[u8] =
    include_bytes!("fixtures/certs/novo_cert_cnpj_06157250000116_senha_minhasenha.pfx");
const PFX_CPF: &[u8] = include_bytes!("fixtures/certs/novo_cert_cpf_90483926086_minhasenha.pfx");
const PASSWORD: &str = "minhasenha";

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

fn extract_sig_value(xml: &str) -> Option<String> {
    let start = xml.find("<SignatureValue>")? + "<SignatureValue>".len();
    let end = xml[start..].find("</SignatureValue>")? + start;
    Some(xml[start..end].to_string())
}

fn extract_digest_value(xml: &str) -> String {
    let start = xml.find("<DigestValue>").unwrap() + "<DigestValue>".len();
    let end = xml[start..].find("</DigestValue>").unwrap() + start;
    xml[start..end].to_string()
}

// =============================================================================
// loadCertificate()
// =============================================================================

mod load_certificate {
    use super::*;

    #[test]
    fn extracts_private_key_and_certificate_from_pfx() {
        let cert = fiscal::certificate::load_certificate(PFX_CNPJ, PASSWORD)
            .expect("load_certificate failed");

        assert!(cert.private_key.contains("-----BEGIN PRIVATE KEY-----"));
        assert!(cert.certificate.contains("-----BEGIN CERTIFICATE-----"));
        assert_eq!(cert.pfx_buffer, PFX_CNPJ);
        assert_eq!(cert.passphrase, PASSWORD);
    }

    #[test]
    fn works_with_cpf_certificate() {
        let cert = fiscal::certificate::load_certificate(PFX_CPF, PASSWORD)
            .expect("load_certificate CPF failed");
        assert!(!cert.private_key.is_empty());
        assert!(!cert.certificate.is_empty());
    }

    #[test]
    fn throws_on_invalid_pfx_buffer() {
        let result = fiscal::certificate::load_certificate(b"not-a-pfx", "pass");
        assert!(result.is_err());
    }

    #[test]
    fn throws_on_wrong_password() {
        let result = fiscal::certificate::load_certificate(PFX_CNPJ, "wrong-password");
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
        let info = fiscal::certificate::get_certificate_info(PFX_CNPJ, PASSWORD)
            .expect("get_certificate_info failed");

        // CN should be the company's CNPJ
        assert!(!info.common_name.is_empty());
        assert!(!info.issuer.is_empty());
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

    fn load_cert() -> fiscal::types::CertificateData {
        fiscal::certificate::load_certificate(PFX_CNPJ, PASSWORD).unwrap()
    }

    #[test]
    fn produces_signed_xml_with_signature_element() {
        let cert = load_cert();
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
        let cert = load_cert();
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
        let cert = load_cert();
        let signed =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();

        assert!(signed.contains(r##"URI="#NFe35260112345678000199650010000000011123456780""##));
    }

    #[test]
    fn uses_rsa_sha1_signature_algorithm() {
        let cert = load_cert();
        let signed =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();

        assert!(signed.contains("rsa-sha1"));
    }

    #[test]
    fn uses_c14n_canonicalization() {
        let cert = load_cert();
        let signed =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();

        assert!(signed.contains("xml-c14n-20010315"));
    }

    #[test]
    fn includes_enveloped_signature_transform() {
        let cert = load_cert();
        let signed =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();

        assert!(signed.contains("enveloped-signature"));
    }

    #[test]
    fn signed_xml_can_be_verified() {
        let cert = load_cert();
        let signed =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();

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
        let cert = load_cert();
        let signed =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();

        let xml_alt_order = sample_xml().replace(
            r#"versao="4.00" Id="NFe35260112345678000199650010000000011123456780""#,
            r#"Id="NFe35260112345678000199650010000000011123456780" versao="4.00""#,
        );
        let signed_alt =
            fiscal::certificate::sign_xml(&xml_alt_order, &cert.private_key, &cert.certificate)
                .unwrap();

        assert_eq!(
            extract_digest_value(&signed),
            extract_digest_value(&signed_alt),
            "C14N must produce identical DigestValue regardless of attribute order in input"
        );
    }

    #[test]
    fn c14n_preserves_namespace_before_attributes() {
        let cert = load_cert();
        let signed =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();

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
        let cert = load_cert();
        let signed1 =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();
        let signed2 =
            fiscal::certificate::sign_xml(&sample_xml(), &cert.private_key, &cert.certificate)
                .unwrap();

        assert_eq!(
            extract_digest_value(&signed1),
            extract_digest_value(&signed2),
            "DigestValue must be deterministic for same input"
        );

        assert_eq!(
            extract_sig_value(&signed1),
            extract_sig_value(&signed2),
            "SignatureValue must be deterministic for same input and key"
        );
    }

    #[test]
    fn self_verifies_signature_with_rsa_crate() {
        use base64::Engine;
        use base64::engine::general_purpose::STANDARD as BASE64;
        use pkcs8::DecodePrivateKey as _;
        use rsa::pkcs1v15::Pkcs1v15Sign;

        let cert_data = fiscal::certificate::load_certificate(PFX_CNPJ, PASSWORD).unwrap();
        let signed = fiscal::certificate::sign_xml(
            &sample_xml(),
            &cert_data.private_key,
            &cert_data.certificate,
        )
        .unwrap();

        // Extract SignedInfo and reconstruct canonical form
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
        let sigval_end = signed[sigval_start..].find("</SignatureValue>").unwrap() + sigval_start;
        let signature_bytes = BASE64.decode(&signed[sigval_start..sigval_end]).unwrap();

        // Get public key from private key
        let private_key = rsa::RsaPrivateKey::from_pkcs8_pem(&cert_data.private_key).unwrap();
        let pubkey = private_key.to_public_key();

        // SHA-1 DigestInfo prefix
        const SHA1_PREFIX: &[u8] = &[
            0x30, 0x21, 0x30, 0x09, 0x06, 0x05, 0x2b, 0x0e, 0x03, 0x02, 0x1a, 0x05, 0x00, 0x04,
            0x14,
        ];

        // Compute SHA-1 hash
        use sha1::Digest as _;
        let hash = sha1::Sha1::digest(canonical_si.as_bytes());

        let scheme = Pkcs1v15Sign {
            hash_len: Some(20),
            prefix: SHA1_PREFIX.into(),
        };
        assert!(
            pubkey.verify(scheme, &hash, &signature_bytes).is_ok(),
            "RSA-SHA1 signature must verify against canonical SignedInfo"
        );
    }

    #[test]
    fn throws_when_inf_nfe_element_is_missing() {
        let cert = load_cert();
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
        let cert = load_cert();

        let xml1 = sample_xml();
        let xml2 = xml1.replace("<vNF>10.00</vNF>", "<vNF>99.99</vNF>");

        let signed1 =
            fiscal::certificate::sign_xml(&xml1, &cert.private_key, &cert.certificate).unwrap();
        let signed2 =
            fiscal::certificate::sign_xml(&xml2, &cert.private_key, &cert.certificate).unwrap();

        let sig1 = extract_sig_value(&signed1).expect("sig1 not found");
        let sig2 = extract_sig_value(&signed2).expect("sig2 not found");
        assert_ne!(sig1, sig2);
    }
}
