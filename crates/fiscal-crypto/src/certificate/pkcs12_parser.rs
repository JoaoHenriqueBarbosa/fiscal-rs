//! Pure-Rust PKCS#12 (PFX) parser using the `pkcs12` crate (RustCrypto).
//!
//! The `pkcs12` crate handles the outer PFX structure (including BER
//! indefinite-length encoding), provides the correct RFC 7292 Appendix B
//! key derivation function, and exposes MacData for integrity verification.
//!
//! This module keeps minimal DER helpers only for the inner structures that
//! `pkcs12` v0.1 does not yet handle: EncryptedPrivateKeyInfo, PKCS#12 PBE
//! params, and certBag value extraction.
//!
//! Supports legacy Brazilian A1 certificates (PBES1/RC2-40-CBC,
//! PBES1/3DES-CBC) and modern certificates (PBES2/AES-128/256-CBC).

use cbc::cipher::block_padding::Pkcs7;
use cbc::cipher::{BlockModeDecrypt, KeyIvInit};
use der::Decode;
use fiscal_core::FiscalError;
use pkcs12::kdf::{Pkcs12KeyType, derive_key_utf8};

// ── OID constants ────────────────────────────────────────────────────────────

const OID_ID_DATA: &str = "1.2.840.113549.1.7.1";
const OID_ID_ENCRYPTED_DATA: &str = "1.2.840.113549.1.7.6";
const OID_PBES1_RC2_40: &str = "1.2.840.113549.1.12.1.6";
const OID_PBES1_3DES: &str = "1.2.840.113549.1.12.1.3";
const OID_PBES2: &str = "1.2.840.113549.1.5.13";

const OID_KEY_BAG: &str = "1.2.840.113549.1.12.10.1.1";
const OID_PKCS8_SHROUDED_KEY_BAG: &str = "1.2.840.113549.1.12.10.1.2";
const OID_CERT_BAG_PKCS12: &str = "1.2.840.113549.1.12.10.1.3";
const OID_CERT_BAG_PKCS9: &str = "1.2.840.113549.1.9.22.1";

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

// ── Minimal DER helpers for inner structures ─────────────────────────────────

/// Read a DER TLV (Tag-Length-Value) from bytes.
///
/// Kept for parsing inner structures (EncryptedPrivateKeyInfo, PBE params,
/// certBag values) that the `pkcs12` crate does not yet handle.
/// Returns `(tag, value_bytes, rest_of_data)`.
fn read_tlv(data: &[u8]) -> Result<(u8, &[u8], &[u8]), String> {
    if data.len() < 2 {
        return Err("TLV: too short for header".into());
    }
    let tag = data[0];
    let len_byte = data[1] as usize;

    let (len, header_len) = if len_byte < 0x80 {
        (len_byte, 2)
    } else if len_byte == 0x80 {
        return Err(
            "BER indefinite-length not supported for inner structures — use DER encoding".into(),
        );
    } else {
        let num_len_bytes = len_byte & 0x7F;
        if data.len() < 2 + num_len_bytes {
            return Err("TLV: too short for long length".into());
        }
        let mut len = 0usize;
        for i in 0..num_len_bytes {
            len = (len << 8) | data[2 + i] as usize;
        }
        (len, 2 + num_len_bytes)
    };

    if data.len() < header_len + len {
        return Err(format!(
            "TLV: value length {len} exceeds data ({} available)",
            data.len() - header_len
        ));
    }

    Ok((
        tag,
        &data[header_len..header_len + len],
        &data[header_len + len..],
    ))
}

fn read_sequence(data: &[u8]) -> Result<(&[u8], &[u8]), String> {
    let (tag, value, rest) = read_tlv(data)?;
    if tag != 0x30 {
        return Err(format!("Expected SEQUENCE (0x30), got tag 0x{tag:02x}"));
    }
    Ok((value, rest))
}

fn read_octet_string(data: &[u8]) -> Result<(&[u8], &[u8]), String> {
    let (tag, value, rest) = read_tlv(data)?;
    if tag != 0x04 {
        return Err(format!("Expected OCTET STRING (0x04), got tag 0x{tag:02x}"));
    }
    Ok((value, rest))
}

fn read_oid(data: &[u8]) -> Result<(String, &[u8]), String> {
    let (tag, value, rest) = read_tlv(data)?;
    if tag != 0x06 {
        return Err(format!("Expected OID (0x06), got tag 0x{tag:02x}"));
    }
    if value.is_empty() {
        return Err("OID: empty value".into());
    }
    let mut s = String::new();
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

// ── PBE params parsing ───────────────────────────────────────────────────────

/// Parse `pkcs-12PbeParams`:  SEQUENCE { salt OCTET STRING, iterations INTEGER }
/// Returns `(salt_bytes, iterations)`.
fn parse_pbes1_params(params_data: &[u8]) -> Result<(&[u8], i32), String> {
    let (inner, _) = read_sequence(params_data)?;
    let (salt, remaining) = read_octet_string(inner)?;
    let mut iterations: i32 = 1; // default per RFC 7292 (deprecated but present)
    if !remaining.is_empty() {
        if let Ok((iters, _)) = read_integer_u32(remaining) {
            iterations = iters as i32;
        }
    }
    Ok((salt, iterations))
}

// ── PBES1 key derivation (RFC 7292 Appendix B) ───────────────────────────────

/// Derive key and IV using the RFC 7292 Appendix B KDF (via `pkcs12::kdf`).
///
/// Uses `derive_key_utf8` which encodes the password as BMPString
/// (UTF-16BE + null terminator, no BOM), applies the diversifier
/// (ID=1 for key, ID=2 for IV), and iterates the hash `iterations` times.
fn pbes1_derive_key_iv(
    password: &str,
    salt: &[u8],
    iterations: i32,
    key_len: usize,
) -> Result<(Vec<u8>, Vec<u8>), FiscalError> {
    let key = derive_key_utf8::<sha1::Sha1>(
        password,
        salt,
        Pkcs12KeyType::EncryptionKey,
        iterations,
        key_len,
    )
    .map_err(|e| FiscalError::Certificate(format!("PBES1 key derivation: {e}")))?;

    let iv = derive_key_utf8::<sha1::Sha1>(password, salt, Pkcs12KeyType::Iv, iterations, 8)
        .map_err(|e| FiscalError::Certificate(format!("PBES1 IV derivation: {e}")))?;

    Ok((key, iv))
}

// ── Decryption functions ──────────────────────────────────────────────────────

/// Decrypt RC2-40-CBC encrypted data.
fn decrypt_rc2_40_cbc(
    encrypted: &[u8],
    password: &str,
    salt: &[u8],
    iterations: i32,
) -> Result<Vec<u8>, FiscalError> {
    let (key, iv) = pbes1_derive_key_iv(password, salt, iterations, 5)?;
    type Rc2Cbc = cbc::Decryptor<rc2::Rc2>;
    let decryptor = Rc2Cbc::new_from_slices(&key, &iv)
        .map_err(|e| FiscalError::Certificate(format!("RC2 CBC init: {e:?}")))?;
    let mut buf = encrypted.to_vec();
    decryptor
        .decrypt_padded::<Pkcs7>(&mut buf)
        .map_err(|e| FiscalError::Certificate(format!("RC2 decrypt: {e:?}")))
        .map(|v| v.to_vec())
}

/// Decrypt 3DES-CBC encrypted data.
fn decrypt_3des_cbc(
    encrypted: &[u8],
    password: &str,
    salt: &[u8],
    iterations: i32,
) -> Result<Vec<u8>, FiscalError> {
    let (key, iv) = pbes1_derive_key_iv(password, salt, iterations, 24)?;
    type DesCbc = cbc::Decryptor<des::TdesEde3>;
    let decryptor = DesCbc::new_from_slices(&key, &iv)
        .map_err(|e| FiscalError::Certificate(format!("3DES CBC init: {e:?}")))?;
    let mut buf = encrypted.to_vec();
    decryptor
        .decrypt_padded::<Pkcs7>(&mut buf)
        .map_err(|e| FiscalError::Certificate(format!("3DES decrypt: {e:?}")))
        .map(|v| v.to_vec())
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

    // KDF params: OCTET STRING (salt), INTEGER (iterations)
    let (salt_val, remaining) =
        read_octet_string(kdf_params_inner).map_err(FiscalError::Certificate)?;
    let (iterations, remaining) = read_integer_u32(remaining).map_err(FiscalError::Certificate)?;

    // Parse optional keyLength and prf
    let mut prf_is_sha256 = false;
    let mut remaining = remaining;
    if !remaining.is_empty() && remaining[0] == 0x02 {
        if let Ok((_kl, rest)) = read_integer_u32(remaining) {
            remaining = rest;
        }
    }
    if !remaining.is_empty() && remaining[0] == 0x30 {
        if let Ok((prf_inner, _)) = read_sequence(remaining) {
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

// ── EncryptedPrivateKeyInfo decryption ────────────────────────────────────────

/// Decrypt a PKCS#8 EncryptedPrivateKeyInfo (pkcs8ShroudedKeyBag value).
fn decrypt_pkcs8_key(data: &[u8], password: &str) -> Result<Vec<u8>, FiscalError> {
    let (epki_inner, _) = read_sequence(data)
        .map_err(|e| FiscalError::Certificate(format!("EncryptedPrivateKeyInfo outer: {e}")))?;

    // encryptionAlgorithm: SEQUENCE { OID, params }
    let (algo_seq, after_algo) = read_sequence(epki_inner)
        .map_err(|e| FiscalError::Certificate(format!("EncryptedPrivateKeyInfo algo: {e}")))?;
    let (algo_oid, algo_params) = read_oid(algo_seq).map_err(FiscalError::Certificate)?;

    // encryptedData: OCTET STRING
    let (encrypted, _) = read_octet_string(after_algo).map_err(FiscalError::Certificate)?;

    match algo_oid.as_str() {
        OID_PBES1_RC2_40 => {
            let (salt, iterations) =
                parse_pbes1_params(algo_params).map_err(FiscalError::Certificate)?;
            decrypt_rc2_40_cbc(encrypted, password, salt, iterations)
        }
        OID_PBES1_3DES => {
            let (salt, iterations) =
                parse_pbes1_params(algo_params).map_err(FiscalError::Certificate)?;
            decrypt_3des_cbc(encrypted, password, salt, iterations)
        }
        OID_PBES2 => decrypt_pbes2(encrypted, password, algo_params),
        _ => Err(FiscalError::Certificate(format!(
            "Unsupported key encryption: {algo_oid}"
        ))),
    }
}

// ── EncryptedData decryption (CMS type) ──────────────────────────────────────

/// Decrypt a CMS EncryptedData content.
///
/// The `content_bytes` are the inner bytes of the `[0] EXPLICIT` field of the
/// ContentInfo. They contain the EncryptedData SEQUENCE.
fn decrypt_cms_encrypted_data(ed_inner: &[u8], password: &str) -> Result<Vec<u8>, FiscalError> {
    // `ed_inner` is the inner value of the EncryptedData SEQUENCE
    // (already stripped of the outer 0x30 tag).
    // EncryptedData ::= SEQUENCE {
    //     version CMSVersion,
    //     encryptedContentInfo EncryptedContentInfo }

    // version
    let (_version, after_version) = read_integer_u32(ed_inner).map_err(FiscalError::Certificate)?;

    // encryptedContentInfo SEQUENCE
    let (eci_inner, _) = read_sequence(after_version).map_err(FiscalError::Certificate)?;

    // contentType OID
    let (_data_oid, after_dt) = read_oid(eci_inner).map_err(FiscalError::Certificate)?;

    // contentEncryptionAlgorithm SEQUENCE { OID, params }
    let (algo_seq, after_algo) = read_sequence(after_dt).map_err(FiscalError::Certificate)?;
    let (algo_oid, algo_params) = read_oid(algo_seq).map_err(FiscalError::Certificate)?;

    // encryptedContent [0] IMPLICIT OCTET STRING
    let (tag, encrypted_val, _rest) = read_tlv(after_algo).map_err(FiscalError::Certificate)?;
    if tag != 0x80 {
        return Err(FiscalError::Certificate(format!(
            "Expected [0] IMPLICIT (0x80), got tag 0x{tag:02x}"
        )));
    }

    match algo_oid.as_str() {
        OID_PBES1_RC2_40 => {
            let (salt, iterations) =
                parse_pbes1_params(algo_params).map_err(FiscalError::Certificate)?;
            decrypt_rc2_40_cbc(encrypted_val, password, salt, iterations)
        }
        OID_PBES1_3DES => {
            let (salt, iterations) =
                parse_pbes1_params(algo_params).map_err(FiscalError::Certificate)?;
            decrypt_3des_cbc(encrypted_val, password, salt, iterations)
        }
        OID_PBES2 => decrypt_pbes2(encrypted_val, password, algo_params),
        _ => Err(FiscalError::Certificate(format!(
            "Unsupported encryption algorithm: {algo_oid}"
        ))),
    }
}

// ── SafeBag value extraction ─────────────────────────────────────────────────

/// Extract X.509 certificate DER from a certBag value.
///
/// `bag_value` is the full TLV of the SafeBag's `[0] EXPLICIT` field.
/// The structure inside is: SEQUENCE { OID(certId), [0] EXPLICIT { OCTET STRING { cert DER } } }
fn extract_cert_from_bag_value(bag_value: &[u8]) -> Result<Vec<u8>, String> {
    // Strip the outer [0] EXPLICIT wrapper (bag_value includes the 0xA0 tag+length)
    let (tag, inner, _) = read_tlv(bag_value)?;
    if tag != 0xA0 {
        return Err(format!(
            "Expected [0] EXPLICIT (0xA0) wrapping certBag, got 0x{tag:02x}"
        ));
    }
    // inner is: SEQUENCE { OID(certId), [0] EXPLICIT certValue }
    let (bag_inner, _) = read_sequence(inner)?;
    let (_cert_id, after_cert_id) = read_oid(bag_inner)?;
    // [0] EXPLICIT certValue
    let (tag, cert_val, _rest) = read_tlv(after_cert_id)?;
    if tag != 0xA0 {
        return Err(format!(
            "Expected [0] EXPLICIT (0xA0) for certValue, got 0x{tag:02x}"
        ));
    }
    // certValue is OCTET STRING wrapping DER certificate
    let (cert_der, _) = read_octet_string(cert_val)?;
    Ok(cert_der.to_vec())
}

/// Decrypt an EncryptedPrivateKeyInfo from a pkcs8ShroudedKeyBag bag_value.
///
/// `bag_value` is the full TLV of the SafeBag's `[0] EXPLICIT` field.
/// The inner content is the EncryptedPrivateKeyInfo SEQUENCE.
fn decrypt_key_from_bag_value(bag_value: &[u8], password: &str) -> Result<Vec<u8>, FiscalError> {
    // Strip the outer [0] EXPLICIT wrapper
    let (tag, inner, _) = read_tlv(bag_value).map_err(FiscalError::Certificate)?;
    if tag != 0xA0 {
        return Err(FiscalError::Certificate(format!(
            "Expected [0] EXPLICIT (0xA0) wrapping key bag, got 0x{tag:02x}"
        )));
    }
    decrypt_pkcs8_key(inner, password)
}

// ── SafeContents parsing ─────────────────────────────────────────────────────

/// Parse SafeContents (SEQUENCE OF SafeBag) from DER bytes using the `pkcs12` crate.
fn parse_safe_contents_der(data: &[u8]) -> Result<Vec<pkcs12::safe_bag::SafeBag>, FiscalError> {
    let safe_bags: pkcs12::safe_bag::SafeContents = der::Decode::from_der(data)
        .map_err(|e| FiscalError::Certificate(format!("SafeContents decode: {e}")))?;
    Ok(safe_bags)
}

/// Process parsed SafeBags and extract certificate, private key, and CA chain.
fn collect_from_safe_bags(
    safe_bags: &[pkcs12::safe_bag::SafeBag],
    password: &str,
) -> Result<ParsedPkcs12, FiscalError> {
    let mut cert_der: Option<Vec<u8>> = None;
    let mut pkey_der: Option<Vec<u8>> = None;
    let mut ca_certs: Vec<Vec<u8>> = Vec::new();

    for bag in safe_bags {
        let bag_id = bag.bag_id.to_string();
        match bag_id.as_str() {
            OID_KEY_BAG => {
                // keyBag: bag_value is [0] EXPLICIT wrapping PrivateKeyInfo
                // Strip the wrapper to get the raw PKCS#8 DER
                if pkey_der.is_none() {
                    if let Ok((tag, inner, _)) = read_tlv(&bag.bag_value) {
                        if tag == 0xA0 {
                            pkey_der = Some(inner.to_vec());
                        }
                    }
                }
            }
            OID_PKCS8_SHROUDED_KEY_BAG => {
                // pkcs8ShroudedKeyBag: bag_value is EncryptedPrivateKeyInfo DER
                if pkey_der.is_none() {
                    match decrypt_key_from_bag_value(&bag.bag_value, password) {
                        Ok(decrypted) => {
                            pkey_der = Some(decrypted);
                        }
                        Err(e) => {
                            return Err(FiscalError::Certificate(format!(
                                "Failed to decrypt private key: {e}"
                            )));
                        }
                    }
                }
            }
            OID_CERT_BAG_PKCS12 | OID_CERT_BAG_PKCS9 => {
                // certBag: bag_value is SEQUENCE { OID, [0] EXPLICIT certValue }
                match extract_cert_from_bag_value(&bag.bag_value) {
                    Ok(cert_bytes) => {
                        if cert_der.is_none() {
                            cert_der = Some(cert_bytes);
                        } else {
                            ca_certs.push(cert_bytes);
                        }
                    }
                    Err(e) => {
                        return Err(FiscalError::Certificate(format!(
                            "Failed to extract certificate: {e}"
                        )));
                    }
                }
            }
            _ => {} // Unknown bag type — skip
        }
    }

    Ok(ParsedPkcs12 {
        cert: cert_der.unwrap_or_default(),
        pkey: pkey_der.unwrap_or_default(),
        ca: ca_certs,
    })
}

// ── MAC verification ─────────────────────────────────────────────────────────

/// OID for SHA-256 digest algorithm.
const OID_SHA256: &str = "2.16.840.1.101.3.4.2.1";

/// Verify the PKCS#12 MAC (HMAC over authSafe bytes).
///
/// The KDF hash function matches the MAC digest algorithm (RFC 7292
/// Appendix B specifies SHA-1, but newer OpenSSL versions use SHA-256
/// when the MAC algorithm is SHA-256 for consistency).
///
/// Returns an error if verification fails (e.g., wrong password).
fn verify_pfx_mac(
    mac_data: &pkcs12::mac_data::MacData,
    auth_safe_bytes: &[u8],
    password: &str,
) -> Result<(), FiscalError> {
    let mac_salt = mac_data.mac_salt.as_bytes();
    let iterations = mac_data.iterations;
    let algo_oid = mac_data.mac.algorithm.oid.to_string();
    let digest_len = mac_data.mac.digest.as_bytes().len();

    let expected = mac_data.mac.digest.as_bytes();

    let ok = if algo_oid == OID_SHA256 {
        // KDF with SHA-256 (matches newer OpenSSL behavior)
        let mac_key = derive_key_utf8::<sha2::Sha256>(
            password,
            mac_salt,
            Pkcs12KeyType::Mac,
            iterations,
            digest_len,
        )
        .map_err(|e| FiscalError::Certificate(format!("MAC key derivation: {e}")))?;

        use hmac::Mac as _;
        let mut mac = <hmac::Hmac<sha2::Sha256> as hmac::Mac>::new_from_slice(&mac_key)
            .map_err(|e| FiscalError::Certificate(format!("HMAC-SHA256 init: {e}")))?;
        mac.update(auth_safe_bytes);
        let computed = mac.finalize().into_bytes();
        computed.as_slice() == expected
    } else {
        // KDF with SHA-1 (RFC 7292 standard)
        let mac_key = derive_key_utf8::<sha1::Sha1>(
            password,
            mac_salt,
            Pkcs12KeyType::Mac,
            iterations,
            digest_len,
        )
        .map_err(|e| FiscalError::Certificate(format!("MAC key derivation: {e}")))?;

        use hmac::Mac as _;
        let mut mac = <hmac::Hmac<sha1::Sha1> as hmac::Mac>::new_from_slice(&mac_key)
            .map_err(|e| FiscalError::Certificate(format!("HMAC-SHA1 init: {e}")))?;
        mac.update(auth_safe_bytes);
        let computed = mac.finalize().into_bytes();
        computed.as_slice() == expected
    };

    if !ok {
        return Err(FiscalError::Certificate(
            "PKCS#12 MAC verification failed — password may be incorrect".into(),
        ));
    }

    Ok(())
}

// ── ContentInfo parsing (minimal, avoids depending on `cms` crate directly) ──

/// Parsed ContentInfo inner structure.
struct ContentInfoInner {
    content_type: String,
    /// Raw bytes of the [0] EXPLICIT content value.
    content: Vec<u8>,
}

/// Parse a single ContentInfo (without outer SEQUENCE header).
fn parse_content_info_inner(data: &[u8]) -> Result<ContentInfoInner, String> {
    let (oid, after_oid) = read_oid(data)?;
    let (tag, content_val, _rest) = read_tlv(after_oid)?;
    if tag != 0xA0 {
        return Err(format!("Expected [0] EXPLICIT (0xA0), got tag 0x{tag:02x}"));
    }
    Ok(ContentInfoInner {
        content_type: oid,
        content: content_val.to_vec(),
    })
}

/// Parse a ContentInfo SEQUENCE: { OID, [0] EXPLICIT { ... } }
fn parse_content_info(data: &[u8]) -> Result<(ContentInfoInner, &[u8]), String> {
    let (ci_inner, rest) = read_sequence(data)?;
    let ci = parse_content_info_inner(ci_inner)?;
    Ok((ci, rest))
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Parse a PKCS#12/PFX DER buffer and extract certificate, private key, and CA chain.
///
/// Handles legacy (PBES1/RC2-40-CBC, PBES1/3DES-CBC) and modern
/// (PBES2/AES-128/256-CBC) encryption schemes used in Brazilian A1 certificates.
///
/// Uses the `pkcs12` crate for correct ASN.1 parsing (including BER
/// indefinite-length), RFC 7292 Appendix B key derivation, and MAC-based
/// integrity verification.
pub fn pkcs12_parse(data: &[u8], password: &str) -> Result<ParsedPkcs12, FiscalError> {
    // 1. Parse the outer PFX structure using the pkcs12 crate.
    //    This handles BER indefinite-length encoding correctly.
    let pfx = pkcs12::pfx::Pfx::from_der(data)
        .map_err(|e| FiscalError::Certificate(format!("Failed to parse PFX: {e}")))?;

    // 2. Extract authSafe bytes from the PFX ContentInfo.
    //    The authSafe content is: [0] EXPLICIT OCTET STRING { AuthenticatedSafe }
    //    The der crate's Any::value() returns the OCTET STRING's value bytes
    //    (i.e. the AuthenticatedSafe BER data), with the OCTET STRING tag
    //    already stripped by the der decoder.
    let auth_safe_bytes = pfx.auth_safe.content.value();

    // 3. Verify MAC if present (integrity check + password validation).
    if let Some(ref mac_data) = pfx.mac_data {
        verify_pfx_mac(mac_data, auth_safe_bytes, password)?;
    }

    // 4. Parse the AuthenticatedSafe: SEQUENCE OF ContentInfo (BER).
    //    We parse this manually to avoid adding `cms` as a direct dependency.
    let (auth_safe_inner, _) = read_sequence(auth_safe_bytes)
        .map_err(|e| FiscalError::Certificate(format!("AuthenticatedSafe: {e}")))?;

    // 5. Process each ContentInfo — decrypt if needed, then parse SafeContents
    let mut final_cert: Option<Vec<u8>> = None;
    let mut final_pkey: Option<Vec<u8>> = None;
    let mut final_ca: Vec<Vec<u8>> = Vec::new();

    let mut remaining = auth_safe_inner;
    while !remaining.is_empty() {
        let (ci, rest) = parse_content_info(remaining).map_err(|e| {
            FiscalError::Certificate(format!(
                "AuthSafe ContentInfo at offset {}: {e}",
                auth_safe_inner.len() - remaining.len()
            ))
        })?;
        remaining = rest;

        let safe_contents_der = match ci.content_type.as_str() {
            OID_ID_DATA => {
                // Content is: OCTET STRING { SafeContents }
                let (tag, inner, _) = read_tlv(&ci.content).map_err(FiscalError::Certificate)?;
                if tag != 0x04 {
                    return Err(FiscalError::Certificate(format!(
                        "Expected OCTET STRING for id-data content, got tag 0x{tag:02x}"
                    )));
                }
                inner.to_vec()
            }
            OID_ID_ENCRYPTED_DATA => {
                // Content is: SEQUENCE { EncryptedData }
                let (tag, ed_bytes, _) = read_tlv(&ci.content).map_err(FiscalError::Certificate)?;
                if tag != 0x30 {
                    return Err(FiscalError::Certificate(format!(
                        "Expected SEQUENCE for EncryptedData, got tag 0x{tag:02x}"
                    )));
                }
                decrypt_cms_encrypted_data(ed_bytes, password)?
            }
            _ => {
                // Unknown content type — skip this section
                continue;
            }
        };

        // Parse SafeContents from this section using pkcs12 crate
        let safe_bags = parse_safe_contents_der(&safe_contents_der)?;
        let parsed = collect_from_safe_bags(&safe_bags, password)?;

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

    // ── Legacy fixture helpers ────────────────────────────────────────────

    fn test_pfx_legacy_rc2_40() -> Vec<u8> {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../..",
            "/tests/fixtures/certs/legacy_rc2_40_senha_minhasenha.pfx"
        );
        std::fs::read(path).expect("legacy RC2-40 fixture not found")
    }

    fn test_pfx_legacy_3des() -> Vec<u8> {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../..",
            "/tests/fixtures/certs/legacy_3des_senha_minhasenha.pfx"
        );
        std::fs::read(path).expect("legacy 3DES fixture not found")
    }

    fn test_pfx_legacy_windows_style() -> Vec<u8> {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../..",
            "/tests/fixtures/certs/legacy_windows_style_senha_minhasenha.pfx"
        );
        std::fs::read(path).expect("legacy Windows-style fixture not found")
    }

    // ── Legacy encryption tests ───────────────────────────────────────────

    #[test]
    fn parse_legacy_rc2_40() {
        let pfx = test_pfx_legacy_rc2_40();
        let result = pkcs12_parse(&pfx, "minhasenha");
        assert!(
            result.is_ok(),
            "RC2-40 parse failed: {:?}",
            result.as_ref().err()
        );
        let parsed = result.unwrap();
        assert!(!parsed.cert.is_empty(), "cert must not be empty");
        assert!(!parsed.pkey.is_empty(), "pkey must not be empty");
    }

    #[test]
    fn parse_legacy_3des() {
        let pfx = test_pfx_legacy_3des();
        let result = pkcs12_parse(&pfx, "minhasenha");
        assert!(
            result.is_ok(),
            "3DES parse failed: {:?}",
            result.as_ref().err()
        );
        let parsed = result.unwrap();
        assert!(!parsed.cert.is_empty(), "cert must not be empty");
        assert!(!parsed.pkey.is_empty(), "pkey must not be empty");
    }

    #[test]
    fn parse_legacy_windows_style() {
        let pfx = test_pfx_legacy_windows_style();
        let result = pkcs12_parse(&pfx, "minhasenha");
        assert!(
            result.is_ok(),
            "Windows-style parse failed: {:?}",
            result.as_ref().err()
        );
        let parsed = result.unwrap();
        assert!(!parsed.cert.is_empty(), "cert must not be empty");
        assert!(!parsed.pkey.is_empty(), "pkey must not be empty");
    }

    #[test]
    fn legacy_rc2_40_wrong_password() {
        let pfx = test_pfx_legacy_rc2_40();
        let result = pkcs12_parse(&pfx, "wrongpassword");
        assert!(result.is_err(), "RC2-40 wrong password should fail");
    }

    #[test]
    fn legacy_3des_wrong_password() {
        let pfx = test_pfx_legacy_3des();
        let result = pkcs12_parse(&pfx, "wrongpassword");
        assert!(result.is_err(), "3DES wrong password should fail");
    }

    #[test]
    fn read_tlv_rejects_indefinite_length() {
        // 0x30 = SEQUENCE tag, 0x80 = indefinite-length marker
        let indefinite = [0x30, 0x80, 0x00, 0x00];
        let result = super::read_tlv(&indefinite);
        assert!(result.is_err(), "indefinite-length should be rejected");
        assert!(
            result
                .unwrap_err()
                .contains("indefinite-length not supported"),
            "error should mention indefinite-length"
        );
    }
}
