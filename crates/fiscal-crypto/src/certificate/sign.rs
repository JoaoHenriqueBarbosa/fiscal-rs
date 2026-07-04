//! XML-DSig signing for NF-e, events, and inutilização documents.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::sign::Signer;
use sha1::{Digest as _, Sha1};
use sha2::Sha256;

use fiscal_core::FiscalError;

use super::c14n::{
    canonicalize_xml, ensure_inherited_namespace, extract_element, extract_element_id,
    remove_signature_element,
};
use super::pfx::SignatureAlgorithm;

/// Sign an NF-e XML with RSA-SHA1 enveloped XMLDSig signature.
///
/// Produces a `<Signature>` element inserted inside `<NFe>` after `</infNFe>`,
/// using C14N canonicalization, SHA-1 digest, and RSA-SHA1 signing.
///
/// The signed element is identified by the `Id` attribute on `<infNFe>`.
///
/// For SHA-256 support, use [`sign_xml_with_algorithm`].
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infNFe>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The signing operation fails
pub fn sign_xml(xml: &str, private_key: &str, certificate: &str) -> Result<String, FiscalError> {
    sign_xml_with_algorithm(xml, private_key, certificate, SignatureAlgorithm::Sha1)
}

/// Sign an NF-e XML with the specified hash algorithm.
///
/// Same as [`sign_xml`] but allows choosing between SHA-1 and SHA-256.
/// Use [`SignatureAlgorithm::Sha256`] for ICP-Brasil v5 certificates.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infNFe>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The signing operation fails
pub fn sign_xml_with_algorithm(
    xml: &str,
    private_key: &str,
    certificate: &str,
    algorithm: SignatureAlgorithm,
) -> Result<String, FiscalError> {
    sign_xml_generic(xml, private_key, certificate, "infNFe", "NFe", algorithm)
}

/// Sign a SEFAZ event XML with RSA-SHA1 enveloped XMLDSig signature.
///
/// Same algorithm as [`sign_xml`] but targets `<infEvento>` inside `<evento>`.
///
/// For SHA-256 support, use [`sign_event_xml_with_algorithm`].
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infEvento>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The signing operation fails
pub fn sign_event_xml(
    xml: &str,
    private_key: &str,
    certificate: &str,
) -> Result<String, FiscalError> {
    sign_event_xml_with_algorithm(xml, private_key, certificate, SignatureAlgorithm::Sha1)
}

/// Sign a SEFAZ event XML with the specified hash algorithm.
///
/// Same as [`sign_event_xml`] but allows choosing between SHA-1 and SHA-256.
/// Use [`SignatureAlgorithm::Sha256`] for ICP-Brasil v5 certificates.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infEvento>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The signing operation fails
pub fn sign_event_xml_with_algorithm(
    xml: &str,
    private_key: &str,
    certificate: &str,
    algorithm: SignatureAlgorithm,
) -> Result<String, FiscalError> {
    sign_xml_generic(
        xml,
        private_key,
        certificate,
        "infEvento",
        "evento",
        algorithm,
    )
}

/// Sign a SEFAZ inutilização XML with RSA-SHA1 enveloped XMLDSig signature.
///
/// Same algorithm as [`sign_xml`] but targets `<infInut>` inside `<inutNFe>`.
///
/// For SHA-256 support, use [`sign_inutilizacao_xml_with_algorithm`].
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infInut>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The signing operation fails
pub fn sign_inutilizacao_xml(
    xml: &str,
    private_key: &str,
    certificate: &str,
) -> Result<String, FiscalError> {
    sign_inutilizacao_xml_with_algorithm(xml, private_key, certificate, SignatureAlgorithm::Sha1)
}

/// Sign a SEFAZ inutilização XML with the specified hash algorithm.
///
/// Same as [`sign_inutilizacao_xml`] but allows choosing between SHA-1 and SHA-256.
/// Use [`SignatureAlgorithm::Sha256`] for ICP-Brasil v5 certificates.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infInut>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The signing operation fails
pub fn sign_inutilizacao_xml_with_algorithm(
    xml: &str,
    private_key: &str,
    certificate: &str,
    algorithm: SignatureAlgorithm,
) -> Result<String, FiscalError> {
    sign_xml_generic(
        xml,
        private_key,
        certificate,
        "infInut",
        "inutNFe",
        algorithm,
    )
}

/// Sign a CT-e XML with RSA-SHA1 enveloped XMLDSig signature.
///
/// Same algorithm as [`sign_xml`] but targets `<infCte>` inside `<CTe>`.
/// SEFAZ requires SHA-1 for the CT-e — SHA-256 yields rejection.
///
/// For SHA-256 support (rare), use [`sign_cte_xml_with_algorithm`].
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infCte>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The signing operation fails
pub fn sign_cte_xml(
    xml: &str,
    private_key: &str,
    certificate: &str,
) -> Result<String, FiscalError> {
    sign_cte_xml_with_algorithm(xml, private_key, certificate, SignatureAlgorithm::Sha1)
}

/// Sign a CT-e XML with the specified hash algorithm.
///
/// Same as [`sign_cte_xml`] but allows choosing between SHA-1 and SHA-256.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infCte>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The signing operation fails
pub fn sign_cte_xml_with_algorithm(
    xml: &str,
    private_key: &str,
    certificate: &str,
    algorithm: SignatureAlgorithm,
) -> Result<String, FiscalError> {
    sign_xml_generic(xml, private_key, certificate, "infCte", "CTe", algorithm)
}

/// Sign a CT-e OS (model 67) XML with RSA-SHA1 enveloped XMLDSig signature.
///
/// Same as [`sign_cte_xml`] but the `<Signature>` is inserted as a child of the
/// `<CTeOS>` root (CT-e OS uses a distinct root element). SEFAZ requires SHA-1.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infCte>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The signing operation fails
pub fn sign_cteos_xml(
    xml: &str,
    private_key: &str,
    certificate: &str,
) -> Result<String, FiscalError> {
    sign_cteos_xml_with_algorithm(xml, private_key, certificate, SignatureAlgorithm::Sha1)
}

/// Sign a CT-e OS XML with the specified hash algorithm.
///
/// Same as [`sign_cteos_xml`] but allows choosing between SHA-1 and SHA-256.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infCte>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The signing operation fails
pub fn sign_cteos_xml_with_algorithm(
    xml: &str,
    private_key: &str,
    certificate: &str,
    algorithm: SignatureAlgorithm,
) -> Result<String, FiscalError> {
    sign_xml_generic(xml, private_key, certificate, "infCte", "CTeOS", algorithm)
}

/// Sign a GTV-e (model 64) XML with RSA-SHA1 enveloped XMLDSig — `<Signature>`
/// inserted as a child of the `<GTVe>` root.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if the XML lacks an `<infCte>` with an
/// `Id`, or the key/cert cannot be parsed, or signing fails.
pub fn sign_gtve_xml(
    xml: &str,
    private_key: &str,
    certificate: &str,
) -> Result<String, FiscalError> {
    sign_gtve_xml_with_algorithm(xml, private_key, certificate, SignatureAlgorithm::Sha1)
}

/// Sign a GTV-e XML with the specified hash algorithm.
///
/// # Errors
///
/// See [`sign_gtve_xml`].
pub fn sign_gtve_xml_with_algorithm(
    xml: &str,
    private_key: &str,
    certificate: &str,
    algorithm: SignatureAlgorithm,
) -> Result<String, FiscalError> {
    sign_xml_generic(xml, private_key, certificate, "infCte", "GTVe", algorithm)
}

/// Sign an MDF-e (model 58) XML with RSA-SHA1 — `<Signature>` as a child of
/// the `<MDFe>` root, referencing `<infMDFe>`.
///
/// The MDF-e uses the same enveloped XML-DSig as the NF-e (RSA-SHA1, SHA-1
/// digest). SEFAZ rejects SHA-256 for MDF-e.
///
/// # Errors
///
/// See [`sign_cte_xml`] (here the signed element is `<infMDFe>`).
pub fn sign_mdfe_xml(
    xml: &str,
    private_key: &str,
    certificate: &str,
) -> Result<String, FiscalError> {
    sign_mdfe_xml_with_algorithm(xml, private_key, certificate, SignatureAlgorithm::Sha1)
}

/// Sign an MDF-e XML with the specified hash algorithm.
///
/// # Errors
///
/// See [`sign_mdfe_xml`].
pub fn sign_mdfe_xml_with_algorithm(
    xml: &str,
    private_key: &str,
    certificate: &str,
    algorithm: SignatureAlgorithm,
) -> Result<String, FiscalError> {
    sign_xml_generic(xml, private_key, certificate, "infMDFe", "MDFe", algorithm)
}

/// Sign a BP-e (model 63) XML with RSA-SHA1 — `<Signature>` as a child of the
/// `<BPe>` root, referencing `<infBPe>`.
///
/// # Errors
///
/// See [`sign_cte_xml`] (here the signed element is `<infBPe>`).
pub fn sign_bpe_xml(
    xml: &str,
    private_key: &str,
    certificate: &str,
) -> Result<String, FiscalError> {
    sign_bpe_xml_with_algorithm(xml, private_key, certificate, SignatureAlgorithm::Sha1)
}

/// Sign a BP-e XML with the specified hash algorithm.
///
/// # Errors
///
/// See [`sign_bpe_xml`].
pub fn sign_bpe_xml_with_algorithm(
    xml: &str,
    private_key: &str,
    certificate: &str,
    algorithm: SignatureAlgorithm,
) -> Result<String, FiscalError> {
    sign_xml_generic(xml, private_key, certificate, "infBPe", "BPe", algorithm)
}

/// Sign an NFS-e Nacional DPS XML with RSA-SHA1 — `<Signature>` as a child of
/// the `<DPS>` root, referencing `<infDPS>`.
///
/// # Errors
///
/// See [`sign_cte_xml`] (here the signed element is `<infDPS>`).
pub fn sign_dps_xml(
    xml: &str,
    private_key: &str,
    certificate: &str,
) -> Result<String, FiscalError> {
    sign_dps_xml_with_algorithm(xml, private_key, certificate, SignatureAlgorithm::Sha1)
}

/// Sign a DPS XML with the specified hash algorithm.
///
/// # Errors
///
/// See [`sign_dps_xml`].
pub fn sign_dps_xml_with_algorithm(
    xml: &str,
    private_key: &str,
    certificate: &str,
    algorithm: SignatureAlgorithm,
) -> Result<String, FiscalError> {
    sign_xml_generic(xml, private_key, certificate, "infDPS", "DPS", algorithm)
}

/// Sign an NFS-e event request (`<pedRegEvento>`) — assina `<infPedReg>`.
///
/// # Errors
///
/// Ver [`sign_dps_xml`] (aqui o elemento assinado é `<infPedReg>`).
pub fn sign_nfse_evento_xml(
    xml: &str,
    private_key: &str,
    certificate: &str,
) -> Result<String, FiscalError> {
    sign_xml_generic(
        xml,
        private_key,
        certificate,
        "infPedReg",
        "pedRegEvento",
        SignatureAlgorithm::Sha1,
    )
}

/// Sign a CT-e event XML with RSA-SHA1 enveloped XMLDSig signature.
///
/// Targets `<infEvento>` inside `<eventoCTe>` (bare element, like the MDF-e —
/// not wrapped in an `<envEvento>` batch). SEFAZ requires SHA-1 for the CT-e.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infEvento>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The signing operation fails
pub fn sign_cte_event_xml(
    xml: &str,
    private_key: &str,
    certificate: &str,
) -> Result<String, FiscalError> {
    sign_cte_event_xml_with_algorithm(xml, private_key, certificate, SignatureAlgorithm::Sha1)
}

/// Sign a CT-e event XML with the specified hash algorithm.
///
/// Same as [`sign_cte_event_xml`] but allows choosing between SHA-1 and SHA-256.
/// Use SHA-1 for SEFAZ submission.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The XML does not contain an `<infEvento>` element with an `Id` attribute
/// - The private key or certificate PEM cannot be parsed
/// - The signing operation fails
pub fn sign_cte_event_xml_with_algorithm(
    xml: &str,
    private_key: &str,
    certificate: &str,
    algorithm: SignatureAlgorithm,
) -> Result<String, FiscalError> {
    sign_xml_generic(
        xml,
        private_key,
        certificate,
        "infEvento",
        "eventoCTe",
        algorithm,
    )
}

/// Sign an RSA-SHA1 payload and return the raw Base64 result.
///
/// Used by providers that require a raw RSA-SHA1 signature over an arbitrary
/// byte string (e.g. São Paulo NFS-e per-RPS `<Assinatura>` field or
/// `AssinaturaCancelamento`).
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if the private key cannot be parsed or
/// the signing operation fails.
pub fn rsa_sha1_base64(data: &[u8], private_key_pem: &str) -> Result<String, FiscalError> {
    let pkey = PKey::private_key_from_pem(private_key_pem.as_bytes())
        .map_err(|e| FiscalError::Certificate(format!("Failed to parse private key: {e}")))?;
    let mut signer = Signer::new(MessageDigest::sha1(), &pkey)
        .map_err(|e| FiscalError::Certificate(format!("Failed to create signer: {e}")))?;
    signer
        .update(data)
        .map_err(|e| FiscalError::Certificate(format!("Failed to update signer: {e}")))?;
    let sig = signer
        .sign_to_vec()
        .map_err(|e| FiscalError::Certificate(format!("RSA-SHA1 signing failed: {e}")))?;
    Ok(BASE64.encode(&sig))
}

/// Sign an ABRASF 2.x NFS-e XML with RSA-SHA1 enveloped XMLDSig signature.
///
/// Signs `<InfDeclaracaoPrestacaoServico Id="...">` and inserts `<Signature>`
/// as the last child of the enclosing `<Rps>` element.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if the XML is missing the signed
/// element, the private key cannot be parsed, or signing fails.
pub fn sign_abrasf_xml(
    xml: &str,
    private_key: &str,
    certificate: &str,
) -> Result<String, FiscalError> {
    sign_xml_generic(
        xml,
        private_key,
        certificate,
        "InfDeclaracaoPrestacaoServico",
        "Rps",
        SignatureAlgorithm::Sha1,
    )
}

/// Sign a São Paulo NFS-e lote XML with RSA-SHA1 enveloped XMLDSig.
///
/// Uses `Reference URI=""` (whole-document reference) and inserts `<Signature>`
/// as the last child of `root_tag`. Used for `PedidoEnvioLoteRPS`,
/// `PedidoCancelamentoNFe`, `PedidoConsultaNFe`, etc.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if the closing `root_tag` is missing,
/// the private key cannot be parsed, or signing fails.
pub fn sign_sp_lote_xml(
    xml: &str,
    root_tag: &str,
    private_key_pem: &str,
    certificate_pem: &str,
) -> Result<String, FiscalError> {
    // 1. Remove any existing <Signature> (enveloped transform)
    let without_sig = remove_signature_element(xml);

    // 2. Canonicalize the entire document
    let canonical = canonicalize_xml(&without_sig);

    // 3. Compute SHA-1 digest
    let digest = compute_digest(canonical.as_bytes(), SignatureAlgorithm::Sha1);

    // 4. Build SignedInfo with URI="" (whole-document reference)
    let (sig_method_uri, dig_method_uri) = (
        "http://www.w3.org/2000/09/xmldsig#rsa-sha1",
        "http://www.w3.org/2000/09/xmldsig#sha1",
    );
    let mut signed_info = String::with_capacity(1024);
    signed_info.push_str("<SignedInfo>");
    signed_info.push_str("<CanonicalizationMethod Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"></CanonicalizationMethod>");
    signed_info.push_str("<SignatureMethod Algorithm=\"");
    signed_info.push_str(sig_method_uri);
    signed_info.push_str("\"></SignatureMethod>");
    signed_info.push_str("<Reference URI=\"\">");
    signed_info.push_str("<Transforms><Transform Algorithm=\"http://www.w3.org/2000/09/xmldsig#enveloped-signature\"></Transform>");
    signed_info.push_str("<Transform Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"></Transform></Transforms>");
    signed_info.push_str("<DigestMethod Algorithm=\"");
    signed_info.push_str(dig_method_uri);
    signed_info.push_str("\"></DigestMethod>");
    signed_info.push_str("<DigestValue>");
    signed_info.push_str(&digest);
    signed_info.push_str("</DigestValue>");
    signed_info.push_str("</Reference>");
    signed_info.push_str("</SignedInfo>");

    // 5. Canonicalize SignedInfo (adds inherited xmlns from <Signature>)
    let canonical_signed_info = signed_info.replacen(
        "<SignedInfo>",
        "<SignedInfo xmlns=\"http://www.w3.org/2000/09/xmldsig#\">",
        1,
    );

    // 6. RSA-SHA1 sign
    let pkey = PKey::private_key_from_pem(private_key_pem.as_bytes())
        .map_err(|e| FiscalError::Certificate(format!("Failed to parse private key: {e}")))?;
    let mut signer = Signer::new(MessageDigest::sha1(), &pkey)
        .map_err(|e| FiscalError::Certificate(format!("Failed to create signer: {e}")))?;
    signer
        .update(canonical_signed_info.as_bytes())
        .map_err(|e| FiscalError::Certificate(format!("Failed to update signer: {e}")))?;
    let sig_bytes = signer
        .sign_to_vec()
        .map_err(|e| FiscalError::Certificate(format!("RSA-SHA1 signing failed: {e}")))?;
    let signature_value = BASE64.encode(&sig_bytes);

    // 7. Build full <Signature> element
    let cert_base64 = extract_cert_base64(certificate_pem);
    let signature_xml = build_signature_element(&signed_info, &signature_value, &cert_base64);

    // 8. Insert before closing root tag
    let closing_tag = format!("</{root_tag}>");
    if let Some(pos) = xml.rfind(&closing_tag) {
        Ok(format!("{}{signature_xml}{}", &xml[..pos], &xml[pos..]))
    } else {
        Err(FiscalError::Certificate(format!(
            "<{root_tag}> closing tag not found in XML"
        )))
    }
}

// ── Private helpers ─────────────────────────────────────────────────────────

/// Generic XML-DSig signing for both NFe and event documents.
///
/// `signed_tag` is the element whose Id attribute is referenced (e.g. "infNFe").
/// `parent_tag` is the element that receives the `<Signature>` child (e.g. "NFe").
/// `algorithm` selects between SHA-1 and SHA-256 for digest and RSA signing.
fn sign_xml_generic(
    xml: &str,
    private_key_pem: &str,
    certificate_pem: &str,
    signed_tag: &str,
    parent_tag: &str,
    algorithm: SignatureAlgorithm,
) -> Result<String, FiscalError> {
    // 1. Extract the Id from the signed element
    let id = extract_element_id(xml, signed_tag)?;

    // 2. Extract the signed element content (including the element itself)
    let signed_element = extract_element(xml, signed_tag).ok_or_else(|| {
        FiscalError::Certificate(format!("<{signed_tag}> element not found in XML"))
    })?;

    // 3. Apply enveloped-signature transform: remove any existing <Signature> from the content
    let without_sig = remove_signature_element(&signed_element);

    // 4. In C14N inclusive, inherited namespaces from ancestor elements must
    //    appear on the root element of the canonicalized subset. If the signed
    //    element doesn't explicitly declare xmlns but the parent does, we must
    //    add it — this matches what PHP's DOMDocument C14N does automatically.
    let with_inherited_ns = ensure_inherited_namespace(&without_sig, xml, signed_tag);

    // 5. Canonicalize the signed element (C14N 1.0 — sorts attributes)
    let canonical = canonicalize_xml(&with_inherited_ns);

    // 6. Compute digest of the canonical form using the selected algorithm
    let digest = compute_digest(canonical.as_bytes(), algorithm);

    // 7. Build the SignedInfo XML (without xmlns — it inherits from parent Signature)
    let signed_info = build_signed_info(&id, &digest, algorithm);

    // 8. Canonicalize the SignedInfo for signing.
    //    In C14N inclusive, when SignedInfo is canonicalized as a subset of
    //    <Signature xmlns="...">, the in-scope namespace is included on
    //    SignedInfo even though it's inherited. We must sign this form so
    //    SEFAZ can verify against the same canonical representation.
    let canonical_signed_info = signed_info.replacen(
        "<SignedInfo>",
        "<SignedInfo xmlns=\"http://www.w3.org/2000/09/xmldsig#\">",
        1,
    );

    // 9. RSA sign the canonical SignedInfo with the selected digest algorithm
    let pkey = PKey::private_key_from_pem(private_key_pem.as_bytes())
        .map_err(|e| FiscalError::Certificate(format!("Failed to parse private key: {e}")))?;

    let openssl_digest = match algorithm {
        SignatureAlgorithm::Sha1 => MessageDigest::sha1(),
        SignatureAlgorithm::Sha256 => MessageDigest::sha256(),
    };

    let mut signer = Signer::new(openssl_digest, &pkey)
        .map_err(|e| FiscalError::Certificate(format!("Failed to create signer: {e}")))?;

    signer
        .update(canonical_signed_info.as_bytes())
        .map_err(|e| FiscalError::Certificate(format!("Failed to update signer: {e}")))?;

    let algo_name = match algorithm {
        SignatureAlgorithm::Sha1 => "RSA-SHA1",
        SignatureAlgorithm::Sha256 => "RSA-SHA256",
    };

    let signature_bytes = signer
        .sign_to_vec()
        .map_err(|e| FiscalError::Certificate(format!("{algo_name} signing failed: {e}")))?;

    let signature_value = BASE64.encode(&signature_bytes);

    // 10. Extract certificate Base64 (strip PEM headers)
    let cert_base64 = extract_cert_base64(certificate_pem);

    // 11. Build the full <Signature> element
    let signature_xml = build_signature_element(&signed_info, &signature_value, &cert_base64);

    // 12. Insert the <Signature> inside the parent element, after the signed element
    let closing_tag = format!("</{parent_tag}>");
    let result = if let Some(pos) = xml.rfind(&closing_tag) {
        format!("{}{signature_xml}{}", &xml[..pos], &xml[pos..])
    } else {
        return Err(FiscalError::Certificate(format!(
            "<{parent_tag}> closing tag not found in XML"
        )));
    };

    Ok(result)
}

/// Compute the Base64-encoded digest of `data` using the selected algorithm.
pub(super) fn compute_digest(data: &[u8], algorithm: SignatureAlgorithm) -> String {
    match algorithm {
        SignatureAlgorithm::Sha1 => {
            let mut hasher = Sha1::new();
            sha1::Digest::update(&mut hasher, data);
            BASE64.encode(sha1::Digest::finalize(hasher))
        }
        SignatureAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            sha2::Digest::update(&mut hasher, data);
            BASE64.encode(sha2::Digest::finalize(hasher))
        }
    }
}

/// Build the `<SignedInfo>` element for XML-DSig.
///
/// The `xmlns` is intentionally omitted here because `<SignedInfo>` inherits
/// its namespace from the parent `<Signature xmlns="...">`. In C14N 1.0,
/// redundant namespace declarations are suppressed, so both the signing and
/// verification sides must use the same canonical form (without `xmlns` on
/// `<SignedInfo>`).
///
/// The `algorithm` parameter selects the appropriate XML-DSig URIs for
/// `<SignatureMethod>` and `<DigestMethod>`.
fn build_signed_info(
    reference_id: &str,
    digest_value: &str,
    algorithm: SignatureAlgorithm,
) -> String {
    let (signature_method_uri, digest_method_uri) = match algorithm {
        SignatureAlgorithm::Sha1 => (
            "http://www.w3.org/2000/09/xmldsig#rsa-sha1",
            "http://www.w3.org/2000/09/xmldsig#sha1",
        ),
        SignatureAlgorithm::Sha256 => (
            "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256",
            "http://www.w3.org/2001/04/xmlenc#sha256",
        ),
    };

    let mut s = String::with_capacity(1024);
    s.push_str("<SignedInfo>");
    s.push_str("<CanonicalizationMethod Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"></CanonicalizationMethod>");
    s.push_str("<SignatureMethod Algorithm=\"");
    s.push_str(signature_method_uri);
    s.push_str("\"></SignatureMethod>");
    s.push_str("<Reference URI=\"#");
    s.push_str(reference_id);
    s.push_str("\">");
    s.push_str("<Transforms>");
    s.push_str("<Transform Algorithm=\"http://www.w3.org/2000/09/xmldsig#enveloped-signature\"></Transform>");
    s.push_str(
        "<Transform Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"></Transform>",
    );
    s.push_str("</Transforms>");
    s.push_str("<DigestMethod Algorithm=\"");
    s.push_str(digest_method_uri);
    s.push_str("\"></DigestMethod>");
    s.push_str("<DigestValue>");
    s.push_str(digest_value);
    s.push_str("</DigestValue>");
    s.push_str("</Reference>");
    s.push_str("</SignedInfo>");
    s
}

/// Build the full `<Signature>` element including SignedInfo, SignatureValue, and KeyInfo.
fn build_signature_element(signed_info: &str, signature_value: &str, cert_base64: &str) -> String {
    let mut s = String::with_capacity(2048);
    s.push_str("<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\">");
    s.push_str(signed_info);
    s.push_str("<SignatureValue>");
    s.push_str(signature_value);
    s.push_str("</SignatureValue>");
    s.push_str("<KeyInfo><X509Data><X509Certificate>");
    s.push_str(cert_base64);
    s.push_str("</X509Certificate></X509Data></KeyInfo>");
    s.push_str("</Signature>");
    s
}

/// Strip PEM headers/footers from a certificate and return the raw Base64 content.
pub(super) fn extract_cert_base64(cert_pem: &str) -> String {
    cert_pem
        .replace("-----BEGIN CERTIFICATE-----", "")
        .replace("-----END CERTIFICATE-----", "")
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect()
}
