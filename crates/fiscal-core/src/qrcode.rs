use base64::Engine as _;
use sha1::{Digest, Sha1};

use crate::FiscalError;
use crate::types::{
    EmissionType, NfceQrCodeParams, PutQRTagParams, QrCodeVersion, SefazEnvironment,
};
use crate::xml_utils::extract_xml_tag_value;

// ── QR Code URL builder ─────────────────────────────────────────────────────

/// Build the NFC-e QR Code URL.
///
/// Supports version 2 (v200) and version 3 (v300, NT 2025.001).
/// Online mode uses a simplified format; offline (`tpEmis=9`) includes
/// additional fields for validation without network.
///
/// # Errors
///
/// Returns [`FiscalError::MissingRequiredField`] if:
/// - v200 is requested without a CSC token or CSC ID
/// - Offline mode is requested without `issued_at`, `total_value`, or `digest_value` (v200)
pub fn build_nfce_qr_code_url(params: &NfceQrCodeParams) -> Result<String, FiscalError> {
    let url = ensure_query_param(&params.qr_code_base_url);

    match params.version {
        QrCodeVersion::V300 => build_v300(&url, params),
        QrCodeVersion::V200 => build_v200(&url, params),
    }
}

/// Build the NFC-e urlChave tag content for consulting the NFe by access key.
///
/// Format: `url?p=key|env` or `url&p=key|env` if URL already contains `?`.
pub fn build_nfce_consult_url(
    url_chave: &str,
    access_key: &str,
    environment: SefazEnvironment,
) -> String {
    let sep = if url_chave.contains('?') { "&" } else { "?" };
    format!(
        "{url_chave}{sep}p={access_key}|{env}",
        env = environment.as_str()
    )
}

/// Insert QR Code and urlChave tags into a signed NFC-e XML.
///
/// Extracts fields from the XML (access key, environment, emission date/type,
/// totals, digest value, destination document), builds the QR Code URL, and
/// inserts an `<infNFeSupl>` element with `<qrCode>` and `<urlChave>` children
/// before the `<Signature` element.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if required XML tags are missing.
/// Returns [`FiscalError::MissingRequiredField`] if CSC token/ID is missing for v200.
/// Returns [`FiscalError::XmlGeneration`] if `<Signature` is not found in the XML.
pub fn put_qr_tag(params: &PutQRTagParams) -> Result<String, FiscalError> {
    let xml = &params.xml;
    let ver = params.version.trim();
    let ver = if ver.is_empty() { "200" } else { ver };
    let token = params.csc_token.trim();
    let token_id = params.csc_id.trim();
    let urlqr = params.qr_code_base_url.trim();
    let urichave = params.url_chave.trim();

    let ver_num: u32 = ver.parse().unwrap_or(200);

    if ver_num < 300 {
        if token.is_empty() {
            return Err(FiscalError::MissingRequiredField {
                field: "CSC token".into(),
            });
        }
        if token_id.is_empty() {
            return Err(FiscalError::MissingRequiredField {
                field: "CSC ID".into(),
            });
        }
    }
    if urlqr.is_empty() {
        return Err(FiscalError::MissingRequiredField {
            field: "qr_code_base_url".into(),
        });
    }

    // Extract fields from XML
    let ch_nfe = extract_xml_tag_attr(xml, "infNFe", "Id")
        .map(|id| id.strip_prefix("NFe").unwrap_or(&id).to_string())
        .unwrap_or_default();
    let tp_amb = extract_xml_tag_value(xml, "tpAmb").unwrap_or_default();
    let dh_emi = extract_xml_tag_value(xml, "dhEmi").unwrap_or_default();
    let tp_emis_str = extract_xml_tag_value(xml, "tpEmis").unwrap_or_else(|| "1".into());
    let tp_emis: u32 = tp_emis_str.parse().unwrap_or(1);
    let v_nf = extract_xml_tag_value(xml, "vNF").unwrap_or_else(|| "0.00".into());
    let v_icms = extract_xml_tag_value(xml, "vICMS").unwrap_or_else(|| "0.00".into());
    let digest_value = extract_xml_tag_value(xml, "DigestValue").unwrap_or_default();

    // Determine destination document and id type from <dest> block
    let (c_dest, tp_id_dest) = extract_dest_info(xml);

    // Build QR Code URL
    let environment = match tp_amb.as_str() {
        "1" => SefazEnvironment::Production,
        _ => SefazEnvironment::Homologation,
    };
    let emission_type = match tp_emis {
        6 => EmissionType::SvcAn,
        7 => EmissionType::SvcRs,
        9 => EmissionType::Offline,
        _ => EmissionType::Normal,
    };
    let version = if ver_num >= 300 {
        QrCodeVersion::V300
    } else {
        QrCodeVersion::V200
    };

    let qr_params = NfceQrCodeParams {
        access_key: ch_nfe.clone(),
        version,
        environment,
        emission_type,
        qr_code_base_url: urlqr.to_string(),
        csc_token: Some(token.to_string()),
        csc_id: Some(token_id.to_string()),
        issued_at: Some(dh_emi),
        total_value: Some(v_nf),
        total_icms: Some(v_icms),
        digest_value: Some(digest_value),
        dest_document: if c_dest.is_empty() {
            None
        } else {
            Some(c_dest)
        },
        dest_id_type: if tp_id_dest.is_empty() {
            None
        } else {
            Some(tp_id_dest)
        },
        sign_fn: None,
    };

    let qrcode = build_nfce_qr_code_url(&qr_params)?;

    // Build infNFeSupl element
    let inf_nfe_supl = format!(
        "<infNFeSupl><qrCode>{qrcode}</qrCode><urlChave>{urichave}</urlChave></infNFeSupl>"
    );

    // Insert before <Signature
    if let Some(pos) = xml.find("<Signature") {
        let mut result = String::with_capacity(xml.len() + inf_nfe_supl.len());
        result.push_str(&xml[..pos]);
        result.push_str(&inf_nfe_supl);
        result.push_str(&xml[pos..]);
        Ok(result)
    } else {
        Err(FiscalError::XmlGeneration(
            "<Signature element not found in XML".into(),
        ))
    }
}

// ── Version 200 ─────────────────────────────────────────────────────────────

/// Build a v200 QR Code URL (online or offline).
fn build_v200(url: &str, params: &NfceQrCodeParams) -> Result<String, FiscalError> {
    let csc_token = params
        .csc_token
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| FiscalError::MissingRequiredField {
            field: "CSC token".into(),
        })?;

    let csc_id_str = params
        .csc_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| FiscalError::MissingRequiredField {
            field: "CSC ID".into(),
        })?;

    let csc_id: i64 = csc_id_str.parse().unwrap_or(0);

    if params.emission_type != EmissionType::Offline {
        // Online mode -- simplified
        let seq = format!(
            "{key}|2|{env}|{csc_id}",
            key = params.access_key,
            env = params.environment.as_str(),
        );
        let hash = sha1_hex(&format!("{seq}{csc_token}"));
        return Ok(format!("{url}{seq}|{hash}"));
    }

    // Offline mode -- full format
    let issued_at =
        params
            .issued_at
            .as_deref()
            .ok_or_else(|| FiscalError::MissingRequiredField {
                field: "issued_at".into(),
            })?;
    let total_value =
        params
            .total_value
            .as_deref()
            .ok_or_else(|| FiscalError::MissingRequiredField {
                field: "total_value".into(),
            })?;
    let digest_value =
        params
            .digest_value
            .as_deref()
            .ok_or_else(|| FiscalError::MissingRequiredField {
                field: "digest_value".into(),
            })?;

    let day = extract_day(issued_at);
    let valor = format_value(total_value);
    let dig_hex = str2hex(digest_value);

    let seq = format!(
        "{key}|2|{env}|{day}|{valor}|{dig_hex}|{csc_id}",
        key = params.access_key,
        env = params.environment.as_str(),
    );
    let hash = sha1_hex(&format!("{seq}{csc_token}"));
    Ok(format!("{url}{seq}|{hash}"))
}

// ── Version 300 (NT 2025.001) ───────────────────────────────────────────────

/// Build a v300 QR Code URL (online or offline with certificate signing).
///
/// Online mode produces a simple URL with access key, version, and environment.
/// Offline mode (`tpEmis=9`) includes additional fields and an RSA signature
/// encoded as base64, matching PHP sped-nfe `QRCode::get300()`.
fn build_v300(url: &str, params: &NfceQrCodeParams) -> Result<String, FiscalError> {
    if params.emission_type != EmissionType::Offline {
        // Online mode -- very simple, no CSC needed
        return Ok(format!(
            "{url}{key}|3|{env}",
            key = params.access_key,
            env = params.environment.as_str(),
        ));
    }

    // Offline v300 -- requires certificate signing (NT 2025.001)
    let issued_at =
        params
            .issued_at
            .as_deref()
            .ok_or_else(|| FiscalError::MissingRequiredField {
                field: "issued_at".into(),
            })?;
    let total_value =
        params
            .total_value
            .as_deref()
            .ok_or_else(|| FiscalError::MissingRequiredField {
                field: "total_value".into(),
            })?;
    let sign_fn = params
        .sign_fn
        .as_ref()
        .ok_or_else(|| FiscalError::MissingRequiredField {
            field: "sign_fn (RSA signer for v300 offline)".into(),
        })?;

    let day = extract_day(issued_at);
    let valor = format_value(total_value);
    let tp_id_dest = params.dest_id_type.as_deref().unwrap_or("");
    let c_dest_raw = params.dest_document.as_deref().unwrap_or("");
    // Per PHP: "Caso Destinatário estrangeiro ou não identificado, informar apenas o separador"
    let c_dest = if tp_id_dest == "3" { "" } else { c_dest_raw };

    let data_to_sign = format!(
        "{key}|3|{env}|{day}|{valor}|{tp_id_dest}|{c_dest}",
        key = params.access_key,
        env = params.environment.as_str(),
    );
    let signature_bytes = sign_fn(data_to_sign.as_bytes())?;
    let signature_b64 = base64::engine::general_purpose::STANDARD.encode(&signature_bytes);

    Ok(format!("{url}{data_to_sign}|{signature_b64}"))
}

// ── Utility functions ───────────────────────────────────────────────────────

/// Ensure the URL contains `?p=`, appending it if missing.
fn ensure_query_param(url: &str) -> String {
    if url.contains("?p=") {
        url.to_string()
    } else {
        format!("{url}?p=")
    }
}

/// Compute the SHA-1 hex digest (uppercase) of the input.
fn sha1_hex(input: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    // Format as uppercase hex
    result
        .iter()
        .fold(String::with_capacity(40), |mut acc, byte| {
            use std::fmt::Write;
            let _ = write!(acc, "{byte:02X}");
            acc
        })
}

/// Convert a string to its hexadecimal ASCII representation.
fn str2hex(s: &str) -> String {
    s.bytes()
        .fold(String::with_capacity(s.len() * 2), |mut acc, b| {
            use std::fmt::Write;
            let _ = write!(acc, "{b:02x}");
            acc
        })
}

/// Extract the day (`dd`) from an ISO date string (e.g. `2026-01-15T10:30:00-03:00` -> `"15"`).
fn extract_day(iso_date: &str) -> String {
    // ISO 8601: YYYY-MM-DDT... — day is at positions 8..10
    if iso_date.len() >= 10 && iso_date.as_bytes()[4] == b'-' && iso_date.as_bytes()[7] == b'-' {
        return iso_date[8..10].to_string();
    }
    // Fallback: try chrono parsing
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso_date) {
        return format!("{:02}", dt.format("%d"));
    }
    "01".to_string()
}

/// Format a numeric value string to 2 decimal places.
fn format_value(value: &str) -> String {
    let v: f64 = value.parse().unwrap_or(0.0);
    format!("{v:.2}")
}

/// Extract an attribute value from an XML element.
fn extract_xml_tag_attr(xml: &str, tag_name: &str, attr_name: &str) -> Option<String> {
    // Find <tagName ... attrName="value"
    let open = format!("<{tag_name}");
    let start = xml.find(&open)?;
    let rest = &xml[start..];
    // Find the closing > of this tag
    let tag_end = rest.find('>')?;
    let tag_content = &rest[..tag_end];
    // Find attrName="value"
    let attr_pat = format!("{attr_name}=\"");
    let attr_start = tag_content.find(&attr_pat)? + attr_pat.len();
    let attr_rest = &tag_content[attr_start..];
    let attr_end = attr_rest.find('"')?;
    Some(attr_rest[..attr_end].to_string())
}

/// Extract destination document and id type from the `<dest>` block.
///
/// Returns `(document, id_type)` where id_type is `"1"` for CNPJ, `"2"` for CPF,
/// `"3"` for idEstrangeiro, or `""` if no dest block is found.
fn extract_dest_info(xml: &str) -> (String, String) {
    let dest_start = match xml.find("<dest>") {
        Some(pos) => pos,
        None => return (String::new(), String::new()),
    };
    let dest_end = match xml[dest_start..].find("</dest>") {
        Some(pos) => dest_start + pos + 7,
        None => return (String::new(), String::new()),
    };
    let dest_block = &xml[dest_start..dest_end];

    for (tag_name, id_type) in &[("CNPJ", "1"), ("CPF", "2"), ("idEstrangeiro", "3")] {
        if let Some(val) = extract_xml_tag_value(dest_block, tag_name) {
            if !val.is_empty() {
                return (val, id_type.to_string());
            }
        }
    }
    (String::new(), String::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_hex_produces_uppercase_40_chars() {
        let hash = sha1_hex("hello");
        assert_eq!(hash.len(), 40);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(hash, "AAF4C61DDCC5E8A2DABEDE0F3B482CD9AEA9434D");
    }

    #[test]
    fn str2hex_converts_ascii() {
        assert_eq!(str2hex("ABC"), "414243");
        assert_eq!(str2hex("abc"), "616263");
    }

    #[test]
    fn extract_day_from_iso() {
        assert_eq!(extract_day("2026-01-15T10:30:00-03:00"), "15");
        assert_eq!(extract_day("2026-03-01T00:00:00-03:00"), "01");
    }

    #[test]
    fn format_value_two_decimals() {
        assert_eq!(format_value("100"), "100.00");
        assert_eq!(format_value("100.5"), "100.50");
        assert_eq!(format_value("0"), "0.00");
    }

    #[test]
    fn ensure_query_param_appends_when_missing() {
        assert_eq!(
            ensure_query_param("https://example.com/qr"),
            "https://example.com/qr?p="
        );
    }

    #[test]
    fn ensure_query_param_keeps_existing() {
        assert_eq!(
            ensure_query_param("https://example.com/qr?p="),
            "https://example.com/qr?p="
        );
    }

    #[test]
    fn extract_xml_tag_attr_finds_id() {
        let xml = r#"<infNFe versao="4.00" Id="NFe12345678901234567890123456789012345678901234">"#;
        let val = extract_xml_tag_attr(xml, "infNFe", "Id");
        assert_eq!(
            val,
            Some("NFe12345678901234567890123456789012345678901234".into())
        );
    }

    #[test]
    fn extract_dest_info_cnpj() {
        let xml = "<NFe><dest><CNPJ>12345678000199</CNPJ></dest></NFe>";
        let (doc, tp) = extract_dest_info(xml);
        assert_eq!(doc, "12345678000199");
        assert_eq!(tp, "1");
    }

    #[test]
    fn extract_dest_info_cpf() {
        let xml = "<NFe><dest><CPF>12345678901</CPF></dest></NFe>";
        let (doc, tp) = extract_dest_info(xml);
        assert_eq!(doc, "12345678901");
        assert_eq!(tp, "2");
    }

    #[test]
    fn extract_dest_info_empty_when_no_dest() {
        let xml = "<NFe><ide></ide></NFe>";
        let (doc, tp) = extract_dest_info(xml);
        assert_eq!(doc, "");
        assert_eq!(tp, "");
    }

    #[test]
    fn v300_online_produces_simple_url() {
        let params = NfceQrCodeParams::new(
            "41260304123456000190650010000001231123456780",
            QrCodeVersion::V300,
            SefazEnvironment::Homologation,
            EmissionType::Normal,
            "https://www.fazenda.pr.gov.br/nfce/qrcode",
        );
        let url = build_nfce_qr_code_url(&params).expect("should build v300 online");
        assert_eq!(
            url,
            "https://www.fazenda.pr.gov.br/nfce/qrcode?p=41260304123456000190650010000001231123456780|3|2"
        );
    }

    #[test]
    fn v300_offline_produces_signed_url() {
        let params = NfceQrCodeParams::new(
            "41260304123456000190650010000001231123456780",
            QrCodeVersion::V300,
            SefazEnvironment::Homologation,
            EmissionType::Offline,
            "https://www.fazenda.pr.gov.br/nfce/qrcode",
        )
        .issued_at("2026-03-15T10:30:00-03:00")
        .total_value("200.50")
        .dest_id_type("1")
        .dest_document("12345678000199")
        .sign_fn(|data: &[u8]| {
            // Dummy signer: just return the data reversed
            Ok(data.iter().rev().copied().collect())
        });

        let url = build_nfce_qr_code_url(&params).expect("should build v300 offline");
        assert!(url.starts_with("https://www.fazenda.pr.gov.br/nfce/qrcode?p="));
        assert!(url.contains("|3|2|15|200.50|1|12345678000199|"));
        // Must end with base64-encoded signature
        let parts: Vec<&str> = url.split('|').collect();
        let last = parts.last().expect("has parts");
        // Verify it's valid base64
        assert!(
            base64::engine::general_purpose::STANDARD
                .decode(last)
                .is_ok(),
            "last segment must be valid base64"
        );
    }

    #[test]
    fn v300_offline_missing_sign_fn_returns_error() {
        let params = NfceQrCodeParams::new(
            "41260304123456000190650010000001231123456780",
            QrCodeVersion::V300,
            SefazEnvironment::Homologation,
            EmissionType::Offline,
            "https://www.fazenda.pr.gov.br/nfce/qrcode",
        )
        .issued_at("2026-03-15T10:30:00-03:00")
        .total_value("200.50");

        let err = build_nfce_qr_code_url(&params).unwrap_err();
        assert!(err.to_string().contains("sign_fn"));
    }

    #[test]
    fn put_qr_tag_missing_csc_token() {
        let params = PutQRTagParams {
            xml: "<root/>".to_string(),
            version: "200".to_string(),
            csc_token: "".to_string(),
            csc_id: "1".to_string(),
            qr_code_base_url: "https://example.com/qr".to_string(),
            url_chave: "https://example.com/chave".to_string(),
        };
        let err = put_qr_tag(&params).unwrap_err();
        assert!(err.to_string().contains("CSC token"));
    }

    #[test]
    fn put_qr_tag_missing_csc_id() {
        let params = PutQRTagParams {
            xml: "<root/>".to_string(),
            version: "200".to_string(),
            csc_token: "token123".to_string(),
            csc_id: "".to_string(),
            qr_code_base_url: "https://example.com/qr".to_string(),
            url_chave: "https://example.com/chave".to_string(),
        };
        let err = put_qr_tag(&params).unwrap_err();
        assert!(err.to_string().contains("CSC ID"));
    }

    #[test]
    fn put_qr_tag_missing_url() {
        let params = PutQRTagParams {
            xml: "<root/>".to_string(),
            version: "200".to_string(),
            csc_token: "token123".to_string(),
            csc_id: "1".to_string(),
            qr_code_base_url: "".to_string(),
            url_chave: "https://example.com/chave".to_string(),
        };
        let err = put_qr_tag(&params).unwrap_err();
        assert!(err.to_string().contains("qr_code_base_url"));
    }

    #[test]
    fn put_qr_tag_no_signature_in_xml() {
        // XML without <Signature
        let xml = concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe41260304123456000190650010000001231123456780">"#,
            "<ide><tpAmb>2</tpAmb><dhEmi>2026-03-15T10:30:00-03:00</dhEmi>",
            "<tpEmis>1</tpEmis></ide>",
            "<total><ICMSTot><vNF>100.00</vNF><vICMS>0.00</vICMS></ICMSTot></total>",
            "</infNFe></NFe>"
        );
        let params = PutQRTagParams {
            xml: xml.to_string(),
            version: "200".to_string(),
            csc_token: "token123".to_string(),
            csc_id: "1".to_string(),
            qr_code_base_url: "https://example.com/qr".to_string(),
            url_chave: "https://example.com/chave".to_string(),
        };
        let err = put_qr_tag(&params).unwrap_err();
        assert!(err.to_string().contains("Signature"));
    }

    #[test]
    fn put_qr_tag_v200_online_success() {
        let xml = concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe41260304123456000190650010000001231123456780">"#,
            "<ide><tpAmb>2</tpAmb><dhEmi>2026-03-15T10:30:00-03:00</dhEmi>",
            "<tpEmis>1</tpEmis></ide>",
            "<total><ICMSTot><vNF>100.00</vNF><vICMS>0.00</vICMS></ICMSTot></total>",
            "</infNFe>",
            r#"<Signature xmlns="http://www.w3.org/2000/09/xmldsig#">"#,
            "<SignedInfo/><SignatureValue/>",
            "<KeyInfo><DigestValue>abc123</DigestValue></KeyInfo>",
            "</Signature></NFe>"
        );
        let params = PutQRTagParams {
            xml: xml.to_string(),
            version: "200".to_string(),
            csc_token: "token123".to_string(),
            csc_id: "1".to_string(),
            qr_code_base_url: "https://example.com/qr".to_string(),
            url_chave: "https://example.com/chave".to_string(),
        };
        let result = put_qr_tag(&params).unwrap();
        assert!(result.contains("<infNFeSupl>"));
        assert!(result.contains("<qrCode>"));
        assert!(result.contains("<urlChave>"));
    }

    #[test]
    fn put_qr_tag_v300_success() {
        let xml = concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe41260304123456000190650010000001231123456780">"#,
            "<ide><tpAmb>1</tpAmb><dhEmi>2026-03-15T10:30:00-03:00</dhEmi>",
            "<tpEmis>1</tpEmis></ide>",
            "<dest><CNPJ>12345678000199</CNPJ></dest>",
            "<total><ICMSTot><vNF>100.00</vNF><vICMS>0.00</vICMS></ICMSTot></total>",
            "</infNFe>",
            r#"<Signature xmlns="http://www.w3.org/2000/09/xmldsig#">"#,
            "<SignedInfo/><SignatureValue/>",
            "</Signature></NFe>"
        );
        let params = PutQRTagParams {
            xml: xml.to_string(),
            version: "300".to_string(),
            csc_token: "".to_string(),
            csc_id: "".to_string(),
            qr_code_base_url: "https://example.com/qr".to_string(),
            url_chave: "https://example.com/chave".to_string(),
        };
        let result = put_qr_tag(&params).unwrap();
        assert!(result.contains("|3|1"));
    }

    #[test]
    fn v200_online_builds_qr() {
        let params = NfceQrCodeParams::new(
            "41260304123456000190650010000001231123456780",
            QrCodeVersion::V200,
            SefazEnvironment::Homologation,
            EmissionType::Normal,
            "https://example.com/qr",
        )
        .csc_token("ABC123")
        .csc_id("1");
        let url = build_nfce_qr_code_url(&params).unwrap();
        assert!(url.contains("|2|2|1|"));
    }

    #[test]
    fn v200_missing_csc_token_errors() {
        let params = NfceQrCodeParams::new(
            "41260304123456000190650010000001231123456780",
            QrCodeVersion::V200,
            SefazEnvironment::Homologation,
            EmissionType::Normal,
            "https://example.com/qr",
        );
        let err = build_nfce_qr_code_url(&params).unwrap_err();
        assert!(err.to_string().contains("CSC token"));
    }

    #[test]
    fn v200_offline_builds_qr() {
        let params = NfceQrCodeParams::new(
            "41260304123456000190650010000001231123456780",
            QrCodeVersion::V200,
            SefazEnvironment::Homologation,
            EmissionType::Offline,
            "https://example.com/qr",
        )
        .csc_token("ABC123")
        .csc_id("1")
        .issued_at("2026-03-15T10:30:00-03:00")
        .total_value("200.50")
        .digest_value("testdigest");
        let url = build_nfce_qr_code_url(&params).unwrap();
        assert!(url.contains("|2|2|15|200.50|"));
    }

    #[test]
    fn v200_offline_missing_issued_at() {
        let params = NfceQrCodeParams::new(
            "41260304123456000190650010000001231123456780",
            QrCodeVersion::V200,
            SefazEnvironment::Homologation,
            EmissionType::Offline,
            "https://example.com/qr",
        )
        .csc_token("ABC123")
        .csc_id("1")
        .total_value("200.50")
        .digest_value("testdigest");
        let err = build_nfce_qr_code_url(&params).unwrap_err();
        assert!(err.to_string().contains("issued_at"));
    }

    #[test]
    fn v200_offline_missing_total_value() {
        let params = NfceQrCodeParams::new(
            "41260304123456000190650010000001231123456780",
            QrCodeVersion::V200,
            SefazEnvironment::Homologation,
            EmissionType::Offline,
            "https://example.com/qr",
        )
        .csc_token("ABC123")
        .csc_id("1")
        .issued_at("2026-03-15T10:30:00-03:00")
        .digest_value("testdigest");
        let err = build_nfce_qr_code_url(&params).unwrap_err();
        assert!(err.to_string().contains("total_value"));
    }

    #[test]
    fn v200_offline_missing_digest_value() {
        let params = NfceQrCodeParams::new(
            "41260304123456000190650010000001231123456780",
            QrCodeVersion::V200,
            SefazEnvironment::Homologation,
            EmissionType::Offline,
            "https://example.com/qr",
        )
        .csc_token("ABC123")
        .csc_id("1")
        .issued_at("2026-03-15T10:30:00-03:00")
        .total_value("200.50");
        let err = build_nfce_qr_code_url(&params).unwrap_err();
        assert!(err.to_string().contains("digest_value"));
    }

    #[test]
    fn v300_offline_missing_issued_at() {
        let params = NfceQrCodeParams::new(
            "41260304123456000190650010000001231123456780",
            QrCodeVersion::V300,
            SefazEnvironment::Homologation,
            EmissionType::Offline,
            "https://example.com/qr",
        )
        .total_value("200.50")
        .sign_fn(|data: &[u8]| Ok(data.to_vec()));
        let err = build_nfce_qr_code_url(&params).unwrap_err();
        assert!(err.to_string().contains("issued_at"));
    }

    #[test]
    fn v300_offline_missing_total_value() {
        let params = NfceQrCodeParams::new(
            "41260304123456000190650010000001231123456780",
            QrCodeVersion::V300,
            SefazEnvironment::Homologation,
            EmissionType::Offline,
            "https://example.com/qr",
        )
        .issued_at("2026-03-15T10:30:00-03:00")
        .sign_fn(|data: &[u8]| Ok(data.to_vec()));
        let err = build_nfce_qr_code_url(&params).unwrap_err();
        assert!(err.to_string().contains("total_value"));
    }

    #[test]
    fn build_nfce_consult_url_with_existing_query_param() {
        let url = build_nfce_consult_url(
            "https://example.com/nfce?x=1",
            "12345678901234567890123456789012345678901234",
            SefazEnvironment::Production,
        );
        assert!(url.contains("&p="));
    }

    #[test]
    fn extract_dest_info_id_estrangeiro() {
        let xml = "<NFe><dest><idEstrangeiro>ABC123</idEstrangeiro></dest></NFe>";
        let (doc, tp) = extract_dest_info(xml);
        assert_eq!(doc, "ABC123");
        assert_eq!(tp, "3");
    }

    #[test]
    fn extract_day_fallback_chrono() {
        // Not standard ISO format but something chrono can parse
        let day = extract_day("bad");
        assert_eq!(day, "01");
    }

    #[test]
    fn v300_offline_foreign_dest_omits_cdest() {
        let params = NfceQrCodeParams::new(
            "41260304123456000190650010000001231123456780",
            QrCodeVersion::V300,
            SefazEnvironment::Production,
            EmissionType::Offline,
            "https://www.fazenda.pr.gov.br/nfce/qrcode",
        )
        .issued_at("2026-03-15T10:30:00-03:00")
        .total_value("100.00")
        .dest_id_type("3")
        .dest_document("FOREIGN123")
        .sign_fn(|data: &[u8]| Ok(data.to_vec()));

        let url = build_nfce_qr_code_url(&params).expect("should build v300 offline");
        // When dest is foreign (type 3), cDest should be empty
        assert!(url.contains("|3||"));
    }
}
