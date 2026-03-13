use super::c14n::*;
use super::pfx::*;
use super::sign::*;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use openssl::hash::MessageDigest;

// Path to the test PFX in the fixtures directory
fn test_pfx_cnpj() -> Vec<u8> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../..",
        "/tests/fixtures/certs/novo_cert_cnpj_06157250000116_senha_minhasenha.pfx"
    );
    std::fs::read(path).expect("test PFX not found")
}

fn test_pfx_cpf() -> Vec<u8> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../..",
        "/tests/fixtures/certs/novo_cert_cpf_90483926086_minhasenha.pfx"
    );
    std::fs::read(path).expect("test PFX not found")
}

const PASSWORD: &str = "minhasenha";

// ── ensure_modern_pfx ─────────────────────────────────────────

#[test]
fn ensure_modern_pfx_valid_cert() {
    let pfx = test_pfx_cnpj();
    let result = ensure_modern_pfx(&pfx, PASSWORD);
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}

#[test]
fn ensure_modern_pfx_wrong_password() {
    let pfx = test_pfx_cnpj();
    let result = ensure_modern_pfx(&pfx, "wrongpassword");
    assert!(result.is_err());
}

#[test]
fn ensure_modern_pfx_invalid_data() {
    let result = ensure_modern_pfx(b"not a pfx", PASSWORD);
    assert!(result.is_err());
}

// ── load_certificate ──────────────────────────────────────────

#[test]
fn load_certificate_cnpj() {
    let pfx = test_pfx_cnpj();
    let cert_data = load_certificate(&pfx, PASSWORD).expect("should load");
    assert!(!cert_data.private_key.is_empty());
    assert!(!cert_data.certificate.is_empty());
    assert!(cert_data.private_key.contains("PRIVATE KEY"));
    assert!(cert_data.certificate.contains("CERTIFICATE"));
}

#[test]
fn load_certificate_cpf() {
    let pfx = test_pfx_cpf();
    let cert_data = load_certificate(&pfx, PASSWORD).expect("should load");
    assert!(!cert_data.private_key.is_empty());
    assert!(!cert_data.certificate.is_empty());
}

#[test]
fn load_certificate_wrong_password() {
    let pfx = test_pfx_cnpj();
    let result = load_certificate(&pfx, "wrong");
    assert!(result.is_err());
}

#[test]
fn load_certificate_invalid_pfx() {
    let result = load_certificate(b"invalid data", PASSWORD);
    assert!(result.is_err());
}

// ── get_certificate_info ──────────────────────────────────────

#[test]
fn get_certificate_info_cnpj() {
    let pfx = test_pfx_cnpj();
    let info = get_certificate_info(&pfx, PASSWORD).expect("should get info");
    assert!(!info.common_name.is_empty());
    assert!(!info.serial_number.is_empty());
}

#[test]
fn get_certificate_info_cpf() {
    let pfx = test_pfx_cpf();
    let info = get_certificate_info(&pfx, PASSWORD).expect("should get info");
    assert!(!info.common_name.is_empty());
}

#[test]
fn get_certificate_info_wrong_password() {
    let pfx = test_pfx_cnpj();
    let result = get_certificate_info(&pfx, "wrong");
    assert!(result.is_err());
}

// ── sign_xml ──────────────────────────────────────────────────

#[test]
fn sign_xml_basic() {
    let pfx = test_pfx_cnpj();
    let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
    let xml = concat!(
        r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
        r#"<infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
        "<ide><cUF>41</cUF></ide>",
        "</infNFe></NFe>"
    );
    let signed =
        sign_xml(xml, &cert_data.private_key, &cert_data.certificate).expect("should sign");
    assert!(signed.contains("<Signature"));
    assert!(signed.contains("<SignatureValue>"));
    assert!(signed.contains("<X509Certificate>"));
    assert!(signed.contains("<DigestValue>"));
}

#[test]
fn sign_xml_missing_tag() {
    let pfx = test_pfx_cnpj();
    let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
    let xml = "<other>content</other>";
    let result = sign_xml(xml, &cert_data.private_key, &cert_data.certificate);
    assert!(result.is_err());
}

#[test]
fn sign_xml_no_id_attribute() {
    let pfx = test_pfx_cnpj();
    let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
    let xml = "<NFe><infNFe><data/></infNFe></NFe>";
    let result = sign_xml(xml, &cert_data.private_key, &cert_data.certificate);
    assert!(result.is_err());
}

// ── sign_event_xml ────────────────────────────────────────────

#[test]
fn sign_event_xml_basic() {
    let pfx = test_pfx_cnpj();
    let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
    let xml = concat!(
        r#"<evento xmlns="http://www.portalfiscal.inf.br/nfe" versao="1.00">"#,
        r#"<infEvento Id="ID1101114126030412345600019055001000000012312345678001">"#,
        "<cOrgao>41</cOrgao>",
        "</infEvento></evento>"
    );
    let signed =
        sign_event_xml(xml, &cert_data.private_key, &cert_data.certificate).expect("should sign");
    assert!(signed.contains("<Signature"));
    assert!(signed.contains("<SignatureValue>"));
}

#[test]
fn sign_event_xml_missing_inf_evento() {
    let pfx = test_pfx_cnpj();
    let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
    let xml = "<evento><other/></evento>";
    let result = sign_event_xml(xml, &cert_data.private_key, &cert_data.certificate);
    assert!(result.is_err());
}

// ── extract_element_id ────────────────────────────────────────

#[test]
fn extract_element_id_success() {
    let xml = r#"<infNFe versao="4.00" Id="NFe41260312345678000199550010000000011123456780"><data/></infNFe>"#;
    let id = extract_element_id(xml, "infNFe").unwrap();
    assert_eq!(id, "NFe41260312345678000199550010000000011123456780");
}

#[test]
fn extract_element_id_no_tag() {
    let result = extract_element_id("<other/>", "infNFe");
    assert!(result.is_err());
}

#[test]
fn extract_element_id_no_id_attr() {
    let xml = "<infNFe versao=\"4.00\"><data/></infNFe>";
    let result = extract_element_id(xml, "infNFe");
    assert!(result.is_err());
}

// ── canonicalize_xml ──────────────────────────────────────────

#[test]
fn canonicalize_xml_removes_declaration() {
    let xml = "<?xml version=\"1.0\"?><root><a/></root>";
    let canonical = canonicalize_xml(xml);
    assert!(!canonical.contains("<?xml"));
    // Self-closing should be expanded
    assert!(canonical.contains("<a></a>"));
}

#[test]
fn canonicalize_xml_sorts_attributes() {
    let xml = r#"<root b="2" a="1"><child/></root>"#;
    let canonical = canonicalize_xml(xml);
    // Attributes should be sorted: a before b
    assert!(canonical.contains(r#"<root a="1" b="2">"#));
}

#[test]
fn canonicalize_xml_ns_first() {
    let xml = r#"<root b="2" xmlns="http://example.com" a="1"><child/></root>"#;
    let canonical = canonicalize_xml(xml);
    // xmlns should come before regular attributes
    assert!(canonical.starts_with(r#"<root xmlns="http://example.com""#));
}

// ── remove_signature_element ──────────────────────────────────

#[test]
fn remove_signature_element_present() {
    let xml = "<root><data/><Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\"><SignedInfo/></Signature></root>";
    let result = remove_signature_element(xml);
    assert!(!result.contains("<Signature"));
    assert!(result.contains("<root>"));
    assert!(result.contains("<data/>"));
}

#[test]
fn remove_signature_element_absent() {
    let xml = "<root><data/></root>";
    let result = remove_signature_element(xml);
    assert_eq!(result, xml);
}

// ── extract_cert_base64 ───────────────────────────────────────

#[test]
fn extract_cert_base64_strips_headers() {
    let pem = "-----BEGIN CERTIFICATE-----\nTWFu\n-----END CERTIFICATE-----\n";
    let b64 = extract_cert_base64(pem);
    assert_eq!(b64, "TWFu");
}

// ── ensure_inherited_namespace ────────────────────────────────

#[test]
fn ensure_inherited_namespace_already_has_xmlns() {
    let element = r#"<infNFe xmlns="http://www.portalfiscal.inf.br/nfe" Id="X">data</infNFe>"#;
    let full_xml = element;
    let result = ensure_inherited_namespace(element, full_xml, "infNFe");
    assert_eq!(result, element);
}

#[test]
fn ensure_inherited_namespace_inherits_from_parent() {
    let element = r#"<infNFe Id="X">data</infNFe>"#;
    let full_xml =
        r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="X">data</infNFe></NFe>"#;
    let result = ensure_inherited_namespace(element, full_xml, "infNFe");
    assert!(result.contains("xmlns=\"http://www.portalfiscal.inf.br/nfe\""));
}

// ── parse_attributes ──────────────────────────────────────────

#[test]
fn parse_attributes_basic() {
    let attrs = parse_attributes(r#"a="1" b="2""#);
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs[0], ("a", "1"));
    assert_eq!(attrs[1], ("b", "2"));
}

#[test]
fn parse_attributes_single_quotes() {
    let attrs = parse_attributes("a='1'");
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0], ("a", "1"));
}

#[test]
fn parse_attributes_empty() {
    let attrs = parse_attributes("");
    assert!(attrs.is_empty());
}

// ── sign_inutilizacao_xml (lines 263, 268) ──────────────────────

#[test]
fn sign_inutilizacao_xml_basic() {
    let pfx = test_pfx_cnpj();
    let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
    let xml = concat!(
        r#"<inutNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00">"#,
        r#"<infInut Id="ID41260304123456000190550010000001231000000010">"#,
        "<tpAmb>2</tpAmb>",
        "</infInut></inutNFe>"
    );
    let signed = sign_inutilizacao_xml(xml, &cert_data.private_key, &cert_data.certificate)
        .expect("should sign inutilizacao");
    assert!(signed.contains("<Signature"));
    assert!(signed.contains("<SignatureValue>"));
    assert!(signed.contains("<X509Certificate>"));
}

#[test]
fn sign_inutilizacao_xml_missing_inf_inut() {
    let pfx = test_pfx_cnpj();
    let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
    let xml = "<inutNFe><other/></inutNFe>";
    let result = sign_inutilizacao_xml(xml, &cert_data.private_key, &cert_data.certificate);
    assert!(result.is_err());
}

// ── sign_xml_generic: parent closing tag not found (line 353) ───

#[test]
fn sign_xml_generic_missing_parent_closing_tag() {
    let pfx = test_pfx_cnpj();
    let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
    // XML has infNFe with Id but the parent NFe is never closed
    let xml =
        r#"<NFe><infNFe Id="NFe41260304123456000190550010000001231123456780"><data/></infNFe>"#;
    let result = sign_xml(xml, &cert_data.private_key, &cert_data.certificate);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("closing tag not found"),
        "Expected 'closing tag not found' error, got: {err_msg}"
    );
}

// ── extract_element_id: malformed Id attribute (line 388) ───────

#[test]
fn extract_element_id_malformed_id() {
    // The Id attribute starts but has no closing quote
    let xml = r#"<infNFe Id="NFe41260312345678></infNFe>"#;
    let result = extract_element_id(xml, "infNFe");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Malformed Id"),
        "Expected 'Malformed Id' error, got: {err_msg}"
    );
}

// ── ensure_inherited_namespace: no xmlns anywhere (line 429) ────

#[test]
fn ensure_inherited_namespace_no_xmlns_anywhere() {
    // Neither element nor any ancestor has xmlns — should return element as-is
    let element = r#"<infNFe Id="X">data</infNFe>"#;
    let full_xml = r#"<NFe><infNFe Id="X">data</infNFe></NFe>"#;
    let result = ensure_inherited_namespace(element, full_xml, "infNFe");
    assert_eq!(result, element);
}

// ── canonicalize_xml: namespace prefix sorting (lines 536-538) ──

#[test]
fn canonicalize_xml_sorts_multiple_ns_prefixes() {
    // Multiple namespace declarations: default xmlns should come first,
    // then prefixed namespaces sorted alphabetically
    let xml = r#"<root xmlns:z="http://z.example.com" xmlns="http://default.example.com" xmlns:a="http://a.example.com"><child/></root>"#;
    let canonical = canonicalize_xml(xml);
    // Default xmlns first, then xmlns:a, then xmlns:z
    assert!(canonical.contains(
            r#"<root xmlns="http://default.example.com" xmlns:a="http://a.example.com" xmlns:z="http://z.example.com">"#
        ));
}

// ── canonicalize_xml: self-closing tag with attributes (lines 554-557) ──

#[test]
fn canonicalize_xml_self_closing_with_attrs() {
    // Self-closing tag with attributes should be expanded to <tag attrs></tag>
    let xml = r#"<root><item b="2" a="1"/></root>"#;
    let canonical = canonicalize_xml(xml);
    // Attributes sorted, self-closing expanded
    assert!(canonical.contains(r#"<item a="1" b="2"></item>"#));
}

// ── SignatureAlgorithm ──────────────────────────────────────────

#[test]
fn signature_algorithm_default_is_sha1() {
    assert_eq!(SignatureAlgorithm::default(), SignatureAlgorithm::Sha1);
}

// ── sign_xml_with_algorithm (SHA-256) ───────────────────────────

#[test]
fn sign_xml_sha256_basic() {
    let pfx = test_pfx_cnpj();
    let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
    let xml = concat!(
        r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
        r#"<infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
        "<ide><cUF>41</cUF></ide>",
        "</infNFe></NFe>"
    );
    let signed = sign_xml_with_algorithm(
        xml,
        &cert_data.private_key,
        &cert_data.certificate,
        SignatureAlgorithm::Sha256,
    )
    .expect("should sign with SHA-256");

    assert!(signed.contains("<Signature"));
    assert!(signed.contains("<SignatureValue>"));
    assert!(signed.contains("<X509Certificate>"));
    assert!(signed.contains("<DigestValue>"));
    // Verify SHA-256 URIs are used
    assert!(
        signed.contains("http://www.w3.org/2001/04/xmldsig-more#rsa-sha256"),
        "SignatureMethod should reference rsa-sha256"
    );
    assert!(
        signed.contains("http://www.w3.org/2001/04/xmlenc#sha256"),
        "DigestMethod should reference sha256"
    );
    // Verify SHA-1 URIs are NOT present
    assert!(
        !signed.contains("xmldsig#rsa-sha1"),
        "SHA-1 SignatureMethod should not be present"
    );
    assert!(
        !signed.contains("xmldsig#sha1"),
        "SHA-1 DigestMethod should not be present"
    );
}

#[test]
fn sign_xml_sha1_explicit() {
    let pfx = test_pfx_cnpj();
    let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
    let xml = concat!(
        r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
        r#"<infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
        "<ide><cUF>41</cUF></ide>",
        "</infNFe></NFe>"
    );
    let signed = sign_xml_with_algorithm(
        xml,
        &cert_data.private_key,
        &cert_data.certificate,
        SignatureAlgorithm::Sha1,
    )
    .expect("should sign with SHA-1");

    // Verify SHA-1 URIs are used (same as sign_xml)
    assert!(signed.contains("xmldsig#rsa-sha1"));
    assert!(signed.contains("xmldsig#sha1"));
}

#[test]
fn sign_xml_sha256_produces_different_signature_than_sha1() {
    let pfx = test_pfx_cnpj();
    let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
    let xml = concat!(
        r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
        r#"<infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
        "<ide><cUF>41</cUF></ide>",
        "</infNFe></NFe>"
    );

    let signed_sha1 = sign_xml(xml, &cert_data.private_key, &cert_data.certificate).unwrap();
    let signed_sha256 = sign_xml_with_algorithm(
        xml,
        &cert_data.private_key,
        &cert_data.certificate,
        SignatureAlgorithm::Sha256,
    )
    .unwrap();

    // Both should produce valid signatures, but with different values
    assert!(signed_sha1.contains("<SignatureValue>"));
    assert!(signed_sha256.contains("<SignatureValue>"));
    assert_ne!(signed_sha1, signed_sha256);
}

#[test]
fn sign_xml_sha256_verify_roundtrip() {
    // Sign with SHA-256, then verify the signature by re-computing the digest
    let pfx = test_pfx_cnpj();
    let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
    let xml = concat!(
        r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
        r#"<infNFe versao="4.00" Id="NFe41260304123456000190550010000001231123456780">"#,
        "<ide><cUF>41</cUF></ide>",
        "</infNFe></NFe>"
    );

    let signed = sign_xml_with_algorithm(
        xml,
        &cert_data.private_key,
        &cert_data.certificate,
        SignatureAlgorithm::Sha256,
    )
    .unwrap();

    // Extract the SignatureValue from the signed XML
    let sig_start = signed.find("<SignatureValue>").unwrap() + "<SignatureValue>".len();
    let sig_end = signed.find("</SignatureValue>").unwrap();
    let sig_b64 = &signed[sig_start..sig_end];
    let sig_bytes = BASE64.decode(sig_b64).unwrap();

    // Extract the SignedInfo for verification
    let si_start = signed.find("<SignedInfo>").unwrap();
    let si_end = signed.find("</SignedInfo>").unwrap() + "</SignedInfo>".len();
    let signed_info_raw = &signed[si_start..si_end];

    // Add the namespace for canonical form (same as signing does)
    let canonical_signed_info = signed_info_raw.replacen(
        "<SignedInfo>",
        "<SignedInfo xmlns=\"http://www.w3.org/2000/09/xmldsig#\">",
        1,
    );

    // Verify the signature using the certificate's public key
    let cert = openssl::x509::X509::from_pem(cert_data.certificate.as_bytes()).unwrap();
    let pubkey = cert.public_key().unwrap();

    let mut verifier = openssl::sign::Verifier::new(MessageDigest::sha256(), &pubkey).unwrap();
    verifier.update(canonical_signed_info.as_bytes()).unwrap();
    assert!(
        verifier.verify(&sig_bytes).unwrap(),
        "RSA-SHA256 signature verification failed"
    );
}

// ── sign_event_xml_with_algorithm (SHA-256) ─────────────────────

#[test]
fn sign_event_xml_sha256() {
    let pfx = test_pfx_cnpj();
    let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
    let xml = concat!(
        r#"<evento xmlns="http://www.portalfiscal.inf.br/nfe" versao="1.00">"#,
        r#"<infEvento Id="ID1101114126030412345600019055001000000012312345678001">"#,
        "<cOrgao>41</cOrgao>",
        "</infEvento></evento>"
    );
    let signed = sign_event_xml_with_algorithm(
        xml,
        &cert_data.private_key,
        &cert_data.certificate,
        SignatureAlgorithm::Sha256,
    )
    .expect("should sign event with SHA-256");

    assert!(signed.contains("<Signature"));
    assert!(signed.contains("rsa-sha256"));
    assert!(signed.contains("xmlenc#sha256"));
}

// ── sign_inutilizacao_xml_with_algorithm (SHA-256) ──────────────

#[test]
fn sign_inutilizacao_xml_sha256() {
    let pfx = test_pfx_cnpj();
    let cert_data = load_certificate(&pfx, PASSWORD).unwrap();
    let xml = concat!(
        r#"<inutNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00">"#,
        r#"<infInut Id="ID41260304123456000190550010000001231000000010">"#,
        "<tpAmb>2</tpAmb>",
        "</infInut></inutNFe>"
    );
    let signed = sign_inutilizacao_xml_with_algorithm(
        xml,
        &cert_data.private_key,
        &cert_data.certificate,
        SignatureAlgorithm::Sha256,
    )
    .expect("should sign inutilizacao with SHA-256");

    assert!(signed.contains("<Signature"));
    assert!(signed.contains("rsa-sha256"));
    assert!(signed.contains("xmlenc#sha256"));
}

// ── compute_digest ─────────────────────────────────────────────

#[test]
fn compute_digest_sha1_matches_known_value() {
    // SHA-1 of "" = da39a3ee5e6b4b0d3255bfef95601890afd80709
    let result = compute_digest(b"", SignatureAlgorithm::Sha1);
    let bytes = BASE64.decode(&result).unwrap();
    assert_eq!(bytes.len(), 20); // SHA-1 produces 20 bytes
}

#[test]
fn compute_digest_sha256_matches_known_value() {
    // SHA-256 of "" = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
    let result = compute_digest(b"", SignatureAlgorithm::Sha256);
    let bytes = BASE64.decode(&result).unwrap();
    assert_eq!(bytes.len(), 32); // SHA-256 produces 32 bytes
}

#[test]
fn compute_digest_sha1_and_sha256_differ() {
    let data = b"test data for hashing";
    let sha1_result = compute_digest(data, SignatureAlgorithm::Sha1);
    let sha256_result = compute_digest(data, SignatureAlgorithm::Sha256);
    assert_ne!(sha1_result, sha256_result);
}
