//! Pure-Rust PKCS#12 (PFX) parser.
//!
//! Replaces OpenSSL's `Pkcs12::from_der()` and `parse2()`.
//! Supports both modern (PBES2/AES-256-CBC) and legacy Brazilian A1 certificates
//! (PBES1/RC2-40-CBC and PBES1/3DES-CBC).
//!
//! Uses manual DER parsing to avoid complex trait requirements from the `der` crate.

use cbc::cipher::block_padding::Pkcs7;
use cbc::cipher::{BlockModeDecrypt, KeyIvInit};
use fiscal_core::FiscalError;

// ── Parsed PKCS#12 result ────────────────────────────────────────────────────

/// Parsed contents of a PKCS#12/PFX file.
#[derive(Debug, Clone)]
pub struct ParsedPkcs12 {
    /// X.509 certificate (DER bytes)
    pub cert: Vec<u8>,
    /// Private key (PKCS#8 plaintext DER bytes)
    pub pkey: Vec<u8>,
    /// CA certificate chain (DER bytes each)
    pub ca: Vec<Vec<u8>>,
}

// ── Minimal DER parsing helpers ──────────────────────────────────────────────

/// Read a DER TLV (Tag-Length-Value) from bytes. Returns (tag, value_bytes, rest_of_data).
fn read_tlv(data: &[u8]) -> Result<(u8, &[u8], &[u8]), String> {
    if data.len() < 2 {
        return Err("DER: too short for TLV header".into());
    }
    let tag = data[0];
    let len_byte = data[1] as usize;

    let (len, header_len) = if len_byte < 0x80 {
        (len_byte, 2)
    } else {
        let num_len_bytes = len_byte & 0x7F;
        if data.len() < 2 + num_len_bytes {
            return Err("DER: too short for long length".into());
        }
        let mut len = 0usize;
        for i in 0..num_len_bytes {
            len = (len << 8) | data[2 + i] as usize;
        }
        (len, 2 + num_len_bytes)
    };

    if data.len() < header_len + len {
        return Err(format!(
            "DER: value length {len} exceeds data ({} bytes available)",
            data.len() - header_len
        ));
    }

    let value = &data[header_len..header_len + len];
    let rest = &data[header_len + len..];
    Ok((tag, value, rest))
}

/// Read a SEQUENCE: tag=0x30, returns the inner bytes.
fn read_sequence(data: &[u8]) -> Result<(&[u8], &[u8]), String> {
    let (tag, value, rest) = read_tlv(data)?;
    if tag != 0x30 {
        return Err(format!("Expected SEQUENCE (0x30), got tag 0x{tag:02x}"));
    }
    Ok((value, rest))
}

/// Read an OCTET STRING: tag=0x04, returns the inner bytes.
fn read_octet_string(data: &[u8]) -> Result<(&[u8], &[u8]), String> {
    let (tag, value, rest) = read_tlv(data)?;
    if tag != 0x04 {
        return Err(format!("Expected OCTET STRING (0x04), got tag 0x{tag:02x}"));
    }
    Ok((value, rest))
}

/// Read a context-specific [0] EXPLICIT tagged value: tag=0xA0.
fn read_context0(data: &[u8]) -> Result<(&[u8], &[u8]), String> {
    let (tag, value, rest) = read_tlv(data)?;
    if tag != 0xA0 {
        return Err(format!("Expected [0] EXPLICIT (0xA0), got tag 0x{tag:02x}"));
    }
    Ok((value, rest))
}

/// Read a context-specific [0] IMPLICIT tagged value: tag=0x80.
fn read_context0_implicit(data: &[u8]) -> Result<(&[u8], &[u8]), String> {
    let (tag, value, rest) = read_tlv(data)?;
    if tag != 0x80 {
        return Err(format!("Expected [0] IMPLICIT (0x80), got tag 0x{tag:02x}"));
    }
    Ok((value, rest))
}

/// Read an OID: tag=0x06, returns the dotted string representation.
fn read_oid(data: &[u8]) -> Result<(String, &[u8]), String> {
    let (tag, value, rest) = read_tlv(data)?;
    if tag != 0x06 {
        return Err(format!("Expected OID (0x06), got tag 0x{tag:02x}"));
    }
    if value.is_empty() {
        return Err("OID: empty value".into());
    }
    // Decode OID from bytes
    let mut s = String::new();
    // First two components: first_byte / 40 . first_byte % 40
    let first = value[0] as u32;
    s.push_str(&format!("{}.{}", first / 40, first % 40));

    let mut i = 1;
    while i < value.len() {
        let mut component: u32 = 0;
        while i < value.len() {
            let byte = value[i];
            i += 1;
            component = (component << 7) | (byte & 0x7F) as u32;
            if byte & 0x80 == 0 {
                break;
            }
        }
        s.push_str(&format!(".{component}"));
    }
    Ok((s, rest))
}

/// Read an INTEGER: tag=0x02, returns as u32.
fn read_integer_u32(data: &[u8]) -> Result<(u32, &[u8]), String> {
    let (tag, value, rest) = read_tlv(data)?;
    if tag != 0x02 {
        return Err(format!("Expected INTEGER (0x02), got tag 0x{tag:02x}"));
    }
    let mut n: u32 = 0;
    for &b in value {
        n = (n << 8) | b as u32;
    }
    Ok((n, rest))
}

// ── PBES1 key derivation ─────────────────────────────────────────────────────

/// Derive key and IV for PBES1 (RFC 8018, Section 6.1).
fn pbes1_derive_key_iv(password: &[u8], salt: &[u8], key_len: usize) -> (Vec<u8>, Vec<u8>) {
    use sha1::Digest as _;
    let mut hasher = sha1::Sha1::new();
    sha1::Digest::update(&mut hasher, password);
    sha1::Digest::update(&mut hasher, salt);
    let hash = sha1::Digest::finalize(hasher);
    let key = hash[..key_len].to_vec();
    let iv = hash[key_len..key_len + 8].to_vec();
    (key, iv)
}

// ── Decryption functions ──────────────────────────────────────────────────────

/// Decrypt RC2-40-CBC encrypted data.
fn decrypt_rc2_40_cbc(
    encrypted: &[u8],
    password: &str,
    salt: &[u8],
) -> Result<Vec<u8>, FiscalError> {
    let (key, iv) = pbes1_derive_key_iv(password.as_bytes(), salt, 5);
    type Rc2Cbc = cbc::Decryptor<rc2::Rc2>;
    let decryptor = Rc2Cbc::new_from_slices(&key, &iv)
        .map_err(|e| FiscalError::Certificate(format!("RC2 CBC init: {e:?}")))?;
    let mut buf = encrypted.to_vec();
    let unpadded = decryptor
        .decrypt_padded::<Pkcs7>(&mut buf)
        .map_err(|e| FiscalError::Certificate(format!("RC2 decrypt: {e:?}")))?;
    Ok(unpadded.to_vec())
}

fn decrypt_3des_cbc(encrypted: &[u8], password: &str, salt: &[u8]) -> Result<Vec<u8>, FiscalError> {
    let (key, iv) = pbes1_derive_key_iv(password.as_bytes(), salt, 24);
    type DesCbc = cbc::Decryptor<des::TdesEde3>;
    let decryptor = DesCbc::new_from_slices(&key, &iv)
        .map_err(|e| FiscalError::Certificate(format!("3DES CBC init: {e:?}")))?;
    let mut buf = encrypted.to_vec();
    let unpadded = decryptor
        .decrypt_padded::<Pkcs7>(&mut buf)
        .map_err(|e| FiscalError::Certificate(format!("3DES decrypt: {e:?}")))?;
    Ok(unpadded.to_vec())
}

/// Decrypt PBES2 (AES-CBC) encrypted data.
fn decrypt_pbes2(
    encrypted: &[u8],
    password: &str,
    params_data: &[u8],
) -> Result<Vec<u8>, FiscalError> {
    // Parse PBES2-params: SEQUENCE { KDF-SEQUENCE, EncScheme-SEQUENCE }
    let (pbes2_inner, _) = read_sequence(params_data).map_err(FiscalError::Certificate)?;

    // KDF: SEQUENCE { OID(PBKDF2), SEQUENCE { salt, iterations, [keyLength], [prf] } }
    let (kdf_seq, enc_scheme_data) =
        read_sequence(pbes2_inner).map_err(FiscalError::Certificate)?;
    let (_pbkdf2_oid, kdf_params_data) = read_oid(kdf_seq).map_err(FiscalError::Certificate)?;
    let (kdf_params_inner, _) = read_sequence(kdf_params_data).map_err(FiscalError::Certificate)?;

    // KDF params: OCTET STRING (salt), INTEGER (iterations), [INTEGER keyLength], [SEQUENCE prf]
    let (salt_val, remaining) =
        read_octet_string(kdf_params_inner).map_err(FiscalError::Certificate)?;
    let (iterations, remaining) = read_integer_u32(remaining).map_err(FiscalError::Certificate)?;

    // Parse optional keyLength and prf (AlgorithmIdentifier)
    let mut prf_is_sha256 = false;
    let mut remaining_salt = remaining;
    if !remaining_salt.is_empty() && remaining_salt[0] == 0x02 {
        // Optional keyLength INTEGER
        if let Ok((_kl, rest)) = read_integer_u32(remaining_salt) {
            remaining_salt = rest;
        }
    }
    if !remaining_salt.is_empty() && remaining_salt[0] == 0x30 {
        // Optional PRF SEQUENCE { OID, NULL }
        if let Ok((prf_inner, _)) = read_sequence(remaining_salt) {
            if let Ok((prf_oid, _)) = read_oid(prf_inner) {
                prf_is_sha256 = prf_oid.contains(".2.9");
            }
        }
    }

    // Encryption scheme: SEQUENCE { OID, OCTET STRING (IV) }
    let (enc_seq_inner, _) = read_sequence(enc_scheme_data).map_err(FiscalError::Certificate)?;
    let (enc_oid, iv_data) = read_oid(enc_seq_inner).map_err(FiscalError::Certificate)?;
    let (iv, _) = read_octet_string(iv_data).map_err(FiscalError::Certificate)?;

    let key_len = if enc_oid.contains(".42") { 32 } else { 16 };

    // Derive key using PBKDF2 with appropriate PRF
    let mut key = vec![0u8; key_len];
    if prf_is_sha256 {
        let _ = pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(
            password.as_bytes(),
            salt_val,
            iterations,
            &mut key,
        );
    } else {
        let _ = pbkdf2::pbkdf2::<hmac::Hmac<sha1::Sha1>>(
            password.as_bytes(),
            salt_val,
            iterations,
            &mut key,
        );
    }

    if key_len == 32 {
        type Aes256Cbc = cbc::Decryptor<aes::Aes256>;
        let mut buf = encrypted.to_vec();
        let unpadded = Aes256Cbc::new_from_slices(&key, iv)
            .map_err(|e| FiscalError::Certificate(format!("AES256 CBC: {e:?}")))?
            .decrypt_padded::<Pkcs7>(&mut buf)
            .map_err(|e| FiscalError::Certificate(format!("AES256 decrypt: {e:?}")))?;
        Ok(unpadded.to_vec())
    } else {
        type Aes128Cbc = cbc::Decryptor<aes::Aes128>;
        let mut buf = encrypted.to_vec();
        let unpadded = Aes128Cbc::new_from_slices(&key, iv)
            .map_err(|e| FiscalError::Certificate(format!("AES128 CBC: {e:?}")))?
            .decrypt_padded::<Pkcs7>(&mut buf)
            .map_err(|e| FiscalError::Certificate(format!("AES128 decrypt: {e:?}")))?;
        Ok(unpadded.to_vec())
    }
}

// ── PKCS#12 ContentInfo parsing ──────────────────────────────────────────────

/// Parse a PKCS#7 ContentInfo INNER (without outer SEQUENCE header).
/// ContentInfo ::= SEQUENCE { OID, [0] EXPLICIT content }
/// `data` must be the inner bytes of the ContentInfo SEQUENCE.
fn parse_content_info_inner(data: &[u8]) -> Result<(String, Vec<u8>), String> {
    let (oid, after_oid) = read_oid(data)?;
    let (content, _) = read_context0(after_oid)?;
    Ok((oid, content.to_vec()))
}

/// Parse a PKCS#7 ContentInfo with outer SEQUENCE header.
fn parse_content_info(data: &[u8]) -> Result<(String, Vec<u8>), String> {
    let (ci_inner, _) = read_sequence(data)?;
    parse_content_info_inner(ci_inner)
}

// ── PFX outer structure ──────────────────────────────────────────────────────

/// Parse the outer PFX: SEQUENCE { INTEGER(3), ContentInfo, [MacData] }
/// Returns the authSafe content bytes.
fn parse_pfx(data: &[u8]) -> Result<Vec<u8>, String> {
    let (pfx_inner, _) = read_sequence(data)?;

    // version INTEGER (must be 3)
    let (version, after_version) = read_integer_u32(pfx_inner)?;
    if version != 3 {
        return Err(format!("PFX version must be 3, got {version}"));
    }

    // authSafe ContentInfo
    let (_ci_oid, ci_content) = parse_content_info(after_version)?;
    if !_ci_oid.starts_with("1.2.840.113549.1.7") {
        return Err(format!("Expected PKCS#7 type OID, got {_ci_oid}"));
    }

    // The content is an OCTET STRING wrapping the AuthenticatedSafe BER data
    let (auth_safe, _) = read_octet_string(&ci_content)?;
    Ok(auth_safe.to_vec())
}

// ── EncryptedData decryption ─────────────────────────────────────────────────

/// Decrypt PKCS#7 EncryptedData.
/// Structure: SEQUENCE { INTEGER(version), SEQUENCE { OID(dataType), SEQUENCE { OID(algo), params }, [0] IMPLICIT OCTET STRING } }
fn decrypt_encrypted_data(data: &[u8], password: &str) -> Result<Vec<u8>, FiscalError> {
    let (ed_inner, _) = read_sequence(data).map_err(FiscalError::Certificate)?;

    // version
    let (_version, after_version) = read_integer_u32(ed_inner).map_err(FiscalError::Certificate)?;

    // encryptedContentInfo SEQUENCE
    let (eci_inner, _) = read_sequence(after_version).map_err(FiscalError::Certificate)?;

    // dataType OID
    let (_data_oid, after_dt) = read_oid(eci_inner).map_err(FiscalError::Certificate)?;

    // contentEncryptionAlgorithm SEQUENCE { OID, params }
    let (algo_seq, after_algo) = read_sequence(after_dt).map_err(FiscalError::Certificate)?;
    let (algo_oid, algo_params) = read_oid(algo_seq).map_err(FiscalError::Certificate)?;

    // encryptedContent [0] IMPLICIT OCTET STRING
    let (encrypted_val, _) =
        read_context0_implicit(after_algo).map_err(FiscalError::Certificate)?;

    match algo_oid.as_str() {
        "1.2.840.113549.1.12.1.6" => {
            // pbeWithSHA1And40BitRC2-CBC
            let (pbes1_inner, _) = read_sequence(algo_params).map_err(FiscalError::Certificate)?;
            let (salt_val, _) = read_octet_string(pbes1_inner).map_err(FiscalError::Certificate)?;
            decrypt_rc2_40_cbc(encrypted_val, password, salt_val)
        }
        "1.2.840.113549.1.12.1.3" => {
            // pbeWithSHA1And3-KeyTripleDES-CBC
            let (pbes1_inner, _) = read_sequence(algo_params).map_err(FiscalError::Certificate)?;
            let (salt_val, _) = read_octet_string(pbes1_inner).map_err(FiscalError::Certificate)?;
            decrypt_3des_cbc(encrypted_val, password, salt_val)
        }
        "1.2.840.113549.1.5.13" => {
            // PBES2
            decrypt_pbes2(encrypted_val, password, algo_params)
        }
        _ => Err(FiscalError::Certificate(format!(
            "Unsupported PKCS#12 encryption: {algo_oid}"
        ))),
    }
}

// ── PKCS#8 EncryptedPrivateKeyInfo ───────────────────────────────────────────

/// Decrypt a PKCS#8 EncryptedPrivateKeyInfo (pkcs8ShroudedKeyBag value).
fn decrypt_pkcs8_key(data: &[u8], password: &str) -> Result<Vec<u8>, FiscalError> {
    let (epki_inner, _) = read_sequence(data).map_err(FiscalError::Certificate)?;

    // encryptionAlgorithm: SEQUENCE { OID, params }
    let (algo_seq, after_algo) = read_sequence(epki_inner).map_err(FiscalError::Certificate)?;
    let (algo_oid, algo_params) = read_oid(algo_seq).map_err(FiscalError::Certificate)?;

    // encryptedData: OCTET STRING
    let (encrypted, _) = read_octet_string(after_algo).map_err(FiscalError::Certificate)?;

    match algo_oid.as_str() {
        "1.2.840.113549.1.12.1.6" => {
            let (pbes1_inner, _) = read_sequence(algo_params).map_err(FiscalError::Certificate)?;
            let (salt_val, _) = read_octet_string(pbes1_inner).map_err(FiscalError::Certificate)?;
            decrypt_rc2_40_cbc(encrypted, password, salt_val)
        }
        "1.2.840.113549.1.12.1.3" => {
            let (pbes1_inner, _) = read_sequence(algo_params).map_err(FiscalError::Certificate)?;
            let (salt_val, _) = read_octet_string(pbes1_inner).map_err(FiscalError::Certificate)?;
            decrypt_3des_cbc(encrypted, password, salt_val)
        }
        "1.2.840.113549.1.5.13" => decrypt_pbes2(encrypted, password, algo_params),
        _ => Err(FiscalError::Certificate(format!(
            "Unsupported key encryption: {algo_oid}"
        ))),
    }
}

// ── SafeContents parsing ─────────────────────────────────────────────────────

/// Parse a certBag value: SEQUENCE { OID(certId), [0] EXPLICIT certValue }
///
/// For x509Certificate (certId 1.2.840.113549.1.9.22.1),
/// the certValue is an OCTET STRING wrapping the DER-encoded X.509 certificate.
fn parse_cert_bag(data: &[u8]) -> Result<Vec<u8>, String> {
    let (bag_inner, _) = read_sequence(data)?;
    let (_cert_id, after_cert_id) = read_oid(bag_inner)?;
    let (cert_val, _) = read_context0(after_cert_id)?;
    // certValue is an OCTET STRING wrapping the actual DER certificate
    let (cert_der, _) = read_octet_string(cert_val)?;
    Ok(cert_der.to_vec())
}

/// Parse SafeContents (SEQUENCE OF SafeBag) and extract cert + key.
fn parse_safe_contents(data: &[u8], password: &str) -> Result<ParsedPkcs12, FiscalError> {
    let (sc_inner, _) =
        read_sequence(data).map_err(|e| FiscalError::Certificate(format!("SafeContents: {e}")))?;

    let mut remaining = sc_inner;
    let mut cert_der: Option<Vec<u8>> = None;
    let mut pkey_der: Option<Vec<u8>> = None;
    let mut ca_certs: Vec<Vec<u8>> = Vec::new();

    while !remaining.is_empty() {
        // Each SafeBag is a SEQUENCE
        let (bag_inner, rest) = read_sequence(remaining)
            .map_err(|e| FiscalError::Certificate(format!("SafeBag: {e}")))?;
        remaining = rest;

        // bagId OID
        let (bag_id, after_bag_id) =
            read_oid(bag_inner).map_err(|e| FiscalError::Certificate(format!("bagId: {e}")))?;

        // bagValue [0] EXPLICIT
        let (bag_value, _after_bag) = read_context0(after_bag_id)
            .map_err(|e| FiscalError::Certificate(format!("bagValue: {e}")))?;

        match bag_id.as_str() {
            "1.2.840.113549.1.12.10.1.3" | "1.2.840.113549.1.9.22.1" => {
                // certBag (PKCS#12 or PKCS#9 OID)
                if let Ok(cert_bytes) = parse_cert_bag(bag_value) {
                    if cert_der.is_none() {
                        cert_der = Some(cert_bytes);
                    } else {
                        ca_certs.push(cert_bytes);
                    }
                }
            }
            "1.2.840.113549.1.12.10.1.2" => {
                // pkcs8ShroudedKeyBag
                if let Ok(decrypted) = decrypt_pkcs8_key(bag_value, password) {
                    pkey_der = Some(decrypted);
                }
            }
            "1.2.840.113549.1.12.10.1.1" => {
                // keyBag (unencrypted PKCS#8)
                pkey_der = Some(bag_value.to_vec());
            }
            _ => {} // Unknown bag type — skip
        }
    }

    // Return what we found — caller merges across sections
    Ok(ParsedPkcs12 {
        cert: cert_der.unwrap_or_default(),
        pkey: pkey_der.unwrap_or_default(),
        ca: ca_certs,
    })
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Parse a PKCS#12/PFX DER buffer and extract certificate, private key, and CA chain.
///
/// Handles legacy (PBES1/RC2-40-CBC, PBES1/3DES-CBC) and modern
/// (PBES2/AES-256-CBC) encryption schemes used in Brazilian A1 certificates.
pub fn pkcs12_parse(data: &[u8], password: &str) -> Result<ParsedPkcs12, FiscalError> {
    // 1. Parse PFX outer structure → get authSafe OCTET STRING bytes
    let auth_safe = parse_pfx(data)
        .map_err(|e| FiscalError::Certificate(format!("Failed to parse PFX: {e}")))?;

    // 2. The authSafe is BER-encoded AuthenticatedSafe: SEQUENCE OF ContentInfo
    let (auth_safe_inner, _) = read_sequence(&auth_safe)
        .map_err(|e| FiscalError::Certificate(format!("AuthSafe: {e}")))?;

    // 3. Process each ContentInfo independently — each one wraps a separate SafeContents
    let mut remaining = auth_safe_inner;
    let mut final_cert: Option<Vec<u8>> = None;
    let mut final_pkey: Option<Vec<u8>> = None;
    let mut final_ca: Vec<Vec<u8>> = Vec::new();

    while !remaining.is_empty() {
        let (ci_inner, rest) = read_sequence(remaining)
            .map_err(|e| FiscalError::Certificate(format!("AuthSafe CI: {e}")))?;
        remaining = rest;

        let (content_type, ci_content) = parse_content_info_inner(ci_inner)
            .map_err(|e| FiscalError::Certificate(format!("ContentInfo: {e}")))?;

        let safe_contents = if content_type == "1.2.840.113549.1.7.1" {
            let (sc, _) = read_octet_string(&ci_content)
                .map_err(|e| FiscalError::Certificate(format!("id-data: {e}")))?;
            sc.to_vec()
        } else if content_type == "1.2.840.113549.1.7.6" {
            decrypt_encrypted_data(&ci_content, password)?
        } else {
            continue;
        };

        // Parse this section's SafeContents and merge results
        let parsed = parse_safe_contents(&safe_contents, password)?;
        if !parsed.cert.is_empty() {
            if final_cert.is_none() {
                final_cert = Some(parsed.cert);
            } else {
                final_ca.push(parsed.cert);
            }
        }
        if !parsed.pkey.is_empty() && final_pkey.is_none() {
            final_pkey = Some(parsed.pkey);
        }
        final_ca.extend(parsed.ca);
    }

    let cert = final_cert
        .ok_or_else(|| FiscalError::Certificate("PFX does not contain a certificate".into()))?;
    let pkey = final_pkey
        .ok_or_else(|| FiscalError::Certificate("PFX does not contain a private key".into()))?;

    Ok(ParsedPkcs12 {
        cert,
        pkey,
        ca: final_ca,
    })
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn parse_real_pfx_cnpj() {
        let pfx = test_pfx_cnpj();
        let result = pkcs12_parse(&pfx, "minhasenha");
        assert!(result.is_ok(), "Parse failed: {:?}", result.as_ref().err());
        let parsed = result.unwrap();
        assert!(!parsed.cert.is_empty(), "cert must not be empty");
        assert!(!parsed.pkey.is_empty(), "pkey must not be empty");
    }

    #[test]
    fn parse_real_pfx_cpf() {
        let pfx = test_pfx_cpf();
        let result = pkcs12_parse(&pfx, "minhasenha");
        assert!(
            result.is_ok(),
            "Parse CPF failed: {:?}",
            result.as_ref().err()
        );
        let parsed = result.unwrap();
        assert!(!parsed.cert.is_empty());
        assert!(!parsed.pkey.is_empty());
    }

    #[test]
    fn parse_pfx_wrong_password() {
        let pfx = test_pfx_cnpj();
        let result = pkcs12_parse(&pfx, "wrongpassword");
        assert!(result.is_err(), "Should fail with wrong password");
    }

    #[test]
    fn parse_pfx_invalid_data() {
        let result = pkcs12_parse(b"not a valid pfx file", "password");
        assert!(result.is_err(), "Should fail with invalid data");
    }
}
