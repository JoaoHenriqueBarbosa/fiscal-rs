//! Configuration validation for Brazilian fiscal documents.
//!
//! This module mirrors the PHP `Config::validate()` method from sped-nfe,
//! which parses a JSON string against a schema and returns a validated
//! configuration object.
//!
//! # Required fields
//!
//! | Field         | Type    | Constraint                          |
//! |---------------|---------|-------------------------------------|
//! | `tpAmb`       | integer | 1 (produção) or 2 (homologação)     |
//! | `razaosocial` | string  | non-empty                           |
//! | `siglaUF`     | string  | exactly 2 characters, valid UF      |
//! | `cnpj`        | string  | 11 digits (CPF) or 14 digits (CNPJ) |
//! | `schemes`     | string  | non-empty                           |
//! | `versao`      | string  | non-empty                           |
//!
//! # Optional fields
//!
//! `atualizacao`, `tokenIBPT`, `CSC`, `CSCid`, and `aProxyConf` are all
//! optional and may be omitted or set to `null`.
//!
//! # Example
//!
//! ```
//! use fiscal_core::config::validate_config;
//!
//! let json = r#"{
//!     "tpAmb": 2,
//!     "razaosocial": "EMPRESA LTDA",
//!     "siglaUF": "SP",
//!     "cnpj": "93623057000128",
//!     "schemes": "PL_009_V4",
//!     "versao": "4.00"
//! }"#;
//!
//! let config = validate_config(json).unwrap();
//! assert_eq!(config.tp_amb, 2);
//! assert_eq!(config.sigla_uf, "SP");
//! ```

use serde::{Deserialize, Serialize};

use crate::FiscalError;

/// Proxy configuration for SEFAZ communication.
///
/// All fields are optional; when present they configure HTTP proxy settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyConfig {
    /// Proxy server IP address.
    #[serde(default, deserialize_with = "deserialize_null_string")]
    pub proxy_ip: Option<String>,
    /// Proxy server port.
    #[serde(default, deserialize_with = "deserialize_null_string")]
    pub proxy_port: Option<String>,
    /// Proxy authentication user.
    #[serde(default, deserialize_with = "deserialize_null_string")]
    pub proxy_user: Option<String>,
    /// Proxy authentication password.
    #[serde(default, deserialize_with = "deserialize_null_string")]
    pub proxy_pass: Option<String>,
}

/// Intermediate struct for serde deserialization (matches PHP JSON field names).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawConfig {
    atualizacao: Option<String>,
    #[serde(alias = "tpAmb")]
    tp_amb: Option<u8>,
    razaosocial: Option<String>,
    #[serde(alias = "siglaUF")]
    sigla_uf: Option<String>,
    cnpj: Option<String>,
    schemes: Option<String>,
    versao: Option<String>,
    #[serde(
        default,
        alias = "tokenIBPT",
        deserialize_with = "deserialize_null_string"
    )]
    token_ibpt: Option<String>,
    #[serde(default, rename = "CSC", deserialize_with = "deserialize_null_string")]
    csc: Option<String>,
    #[serde(
        default,
        rename = "CSCid",
        deserialize_with = "deserialize_null_string"
    )]
    csc_id: Option<String>,
    #[serde(default, rename = "aProxyConf")]
    a_proxy_conf: Option<ProxyConfig>,
}

/// Validated fiscal configuration.
///
/// Created by [`validate_config`] after parsing and validating a JSON string
/// against the same rules as the PHP `Config::validate()` method.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiscalConfig {
    /// Date/time of the last configuration update (optional).
    pub atualizacao: Option<String>,
    /// Environment type: 1 = production, 2 = homologation.
    pub tp_amb: u8,
    /// Company legal name.
    pub razaosocial: String,
    /// Two-letter Brazilian state abbreviation (UF).
    pub sigla_uf: String,
    /// CNPJ (14 digits) or CPF (11 digits).
    pub cnpj: String,
    /// Schema version path (e.g. `"PL_009_V4"`).
    pub schemes: String,
    /// NF-e layout version (e.g. `"4.00"`).
    pub versao: String,
    /// IBPT transparency token (optional).
    pub token_ibpt: Option<String>,
    /// NFC-e security code (optional).
    pub csc: Option<String>,
    /// NFC-e security code ID (optional).
    pub csc_id: Option<String>,
    /// Proxy configuration (optional).
    pub a_proxy_conf: Option<ProxyConfig>,
}

/// Parse and validate a JSON configuration string.
///
/// This is the Rust equivalent of the PHP `Config::validate($content)` method.
/// It performs the following checks:
///
/// 1. The input must be valid JSON.
/// 2. The JSON root must be an object (not an array or scalar).
/// 3. All required fields must be present and non-null.
/// 4. `tpAmb` must be 1 or 2.
/// 5. `siglaUF` must be exactly 2 characters.
/// 6. `cnpj` must be 11 or 14 digits (CPF or CNPJ).
///
/// # Errors
///
/// Returns [`FiscalError::ConfigValidation`] if any validation rule fails.
pub fn validate_config(json: &str) -> Result<FiscalConfig, FiscalError> {
    if json.is_empty() {
        return Err(FiscalError::ConfigValidation(
            "Não foi passado um json válido.".into(),
        ));
    }

    // First check: must be a JSON object
    let value: serde_json::Value = serde_json::from_str(json).map_err(|e| {
        FiscalError::ConfigValidation(format!("Não foi passado um json válido: {e}"))
    })?;

    if !value.is_object() {
        return Err(FiscalError::ConfigValidation(
            "Não foi passado um json válido.".into(),
        ));
    }

    // Deserialize into raw struct
    let raw: RawConfig = serde_json::from_value(value).map_err(|e| {
        FiscalError::ConfigValidation(format!("Erro ao deserializar configuração: {e}"))
    })?;

    // Validate required fields
    let mut errors = Vec::new();

    let tp_amb = match raw.tp_amb {
        Some(v) => v,
        None => {
            errors.push("[tpAmb] Campo obrigatório".to_string());
            0
        }
    };

    let razaosocial = match raw.razaosocial {
        Some(ref v) if !v.is_empty() => v.clone(),
        Some(_) => {
            errors.push("[razaosocial] Campo obrigatório".to_string());
            String::new()
        }
        None => {
            errors.push("[razaosocial] Campo obrigatório".to_string());
            String::new()
        }
    };

    let sigla_uf = match raw.sigla_uf {
        Some(ref v) if !v.is_empty() => v.clone(),
        Some(_) => {
            errors.push("[siglaUF] Campo obrigatório".to_string());
            String::new()
        }
        None => {
            errors.push("[siglaUF] Campo obrigatório".to_string());
            String::new()
        }
    };

    let cnpj = match raw.cnpj {
        Some(ref v) if !v.is_empty() => v.clone(),
        Some(_) => {
            errors.push("[cnpj] Campo obrigatório".to_string());
            String::new()
        }
        None => {
            errors.push("[cnpj] Campo obrigatório".to_string());
            String::new()
        }
    };

    let schemes = match raw.schemes {
        Some(ref v) if !v.is_empty() => v.clone(),
        Some(_) => {
            errors.push("[schemes] Campo obrigatório".to_string());
            String::new()
        }
        None => {
            errors.push("[schemes] Campo obrigatório".to_string());
            String::new()
        }
    };

    let versao = match raw.versao {
        Some(ref v) if !v.is_empty() => v.clone(),
        Some(_) => {
            errors.push("[versao] Campo obrigatório".to_string());
            String::new()
        }
        None => {
            errors.push("[versao] Campo obrigatório".to_string());
            String::new()
        }
    };

    // Semantic validations (only if the field was provided)
    if tp_amb != 0 && tp_amb != 1 && tp_amb != 2 {
        errors.push(format!(
            "[tpAmb] Valor inválido: {tp_amb}. Esperado 1 (produção) ou 2 (homologação)"
        ));
    }

    if !sigla_uf.is_empty() && sigla_uf.len() != 2 {
        errors.push(format!(
            "[siglaUF] Deve ter exatamente 2 caracteres, recebido: \"{}\"",
            sigla_uf
        ));
    }

    if !cnpj.is_empty() {
        let all_digits = cnpj.chars().all(|c| c.is_ascii_digit());
        let valid_len = cnpj.len() == 11 || cnpj.len() == 14;
        if !all_digits || !valid_len {
            errors.push(format!(
                "[cnpj] Deve conter 11 (CPF) ou 14 (CNPJ) dígitos, recebido: \"{}\"",
                cnpj
            ));
        }
    }

    if !errors.is_empty() {
        return Err(FiscalError::ConfigValidation(errors.join("\n")));
    }

    Ok(FiscalConfig {
        atualizacao: raw.atualizacao,
        tp_amb,
        razaosocial,
        sigla_uf,
        cnpj,
        schemes,
        versao,
        token_ibpt: raw.token_ibpt,
        csc: raw.csc,
        csc_id: raw.csc_id,
        a_proxy_conf: raw.a_proxy_conf,
    })
}

/// Custom deserializer that treats JSON `null` as `None` for `Option<String>`.
fn deserialize_null_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.filter(|s| !s.is_empty()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_config_json() -> &'static str {
        r#"{
            "atualizacao": "2017-02-20 09:11:21",
            "tpAmb": 2,
            "razaosocial": "SUA RAZAO SOCIAL LTDA",
            "siglaUF": "SP",
            "cnpj": "93623057000128",
            "schemes": "PL_010_V1.30",
            "versao": "4.00",
            "tokenIBPT": "AAAAAAA",
            "CSC": "GPB0JBWLUR6HWFTVEAS6RJ69GPCROFPBBB8G",
            "CSCid": "000001",
            "aProxyConf": {
                "proxyIp": "",
                "proxyPort": "",
                "proxyUser": "",
                "proxyPass": ""
            }
        }"#
    }

    fn minimal_config_json() -> &'static str {
        r#"{
            "tpAmb": 2,
            "razaosocial": "SUA RAZAO SOCIAL LTDA",
            "siglaUF": "SP",
            "cnpj": "99999999999999",
            "schemes": "PL_009_V4",
            "versao": "4.00"
        }"#
    }

    #[test]
    fn validate_full_config() {
        let config = validate_config(full_config_json()).unwrap();
        assert_eq!(config.tp_amb, 2);
        assert_eq!(config.razaosocial, "SUA RAZAO SOCIAL LTDA");
        assert_eq!(config.sigla_uf, "SP");
        assert_eq!(config.cnpj, "93623057000128");
        assert_eq!(config.schemes, "PL_010_V1.30");
        assert_eq!(config.versao, "4.00");
        assert_eq!(config.atualizacao.as_deref(), Some("2017-02-20 09:11:21"));
        assert_eq!(config.token_ibpt.as_deref(), Some("AAAAAAA"));
        assert_eq!(
            config.csc.as_deref(),
            Some("GPB0JBWLUR6HWFTVEAS6RJ69GPCROFPBBB8G")
        );
        assert_eq!(config.csc_id.as_deref(), Some("000001"));
        assert!(config.a_proxy_conf.is_some());
    }

    #[test]
    fn validate_minimal_config_without_optionals() {
        let config = validate_config(minimal_config_json()).unwrap();
        assert_eq!(config.tp_amb, 2);
        assert_eq!(config.cnpj, "99999999999999");
        assert!(config.atualizacao.is_none());
        assert!(config.token_ibpt.is_none());
        assert!(config.csc.is_none());
        assert!(config.csc_id.is_none());
        assert!(config.a_proxy_conf.is_none());
    }

    #[test]
    fn empty_string_fails() {
        let err = validate_config("").unwrap_err();
        assert!(matches!(err, FiscalError::ConfigValidation(_)));
    }

    #[test]
    fn invalid_json_fails() {
        let err = validate_config("not json at all").unwrap_err();
        assert!(matches!(err, FiscalError::ConfigValidation(_)));
    }

    #[test]
    fn json_array_fails() {
        let err = validate_config("[1,2,3]").unwrap_err();
        assert!(matches!(err, FiscalError::ConfigValidation(_)));
    }

    #[test]
    fn missing_tp_amb_fails() {
        let json = r#"{
            "razaosocial": "SUA RAZAO SOCIAL LTDA",
            "siglaUF": "SP",
            "cnpj": "99999999999999",
            "schemes": "PL_009_V4",
            "versao": "4.00"
        }"#;
        let err = validate_config(json).unwrap_err();
        match &err {
            FiscalError::ConfigValidation(msg) => assert!(msg.contains("[tpAmb]")),
            _ => panic!("expected ConfigValidation, got: {err:?}"),
        }
    }

    #[test]
    fn missing_razaosocial_fails() {
        let json = r#"{
            "tpAmb": 2,
            "siglaUF": "SP",
            "cnpj": "99999999999999",
            "schemes": "PL_009_V4",
            "versao": "4.00"
        }"#;
        let err = validate_config(json).unwrap_err();
        match &err {
            FiscalError::ConfigValidation(msg) => assert!(msg.contains("[razaosocial]")),
            _ => panic!("expected ConfigValidation, got: {err:?}"),
        }
    }

    #[test]
    fn missing_sigla_uf_fails() {
        let json = r#"{
            "tpAmb": 2,
            "razaosocial": "SUA RAZAO SOCIAL LTDA",
            "cnpj": "99999999999999",
            "schemes": "PL_009_V4",
            "versao": "4.00"
        }"#;
        let err = validate_config(json).unwrap_err();
        match &err {
            FiscalError::ConfigValidation(msg) => assert!(msg.contains("[siglaUF]")),
            _ => panic!("expected ConfigValidation, got: {err:?}"),
        }
    }

    #[test]
    fn missing_cnpj_fails() {
        let json = r#"{
            "tpAmb": 2,
            "razaosocial": "SUA RAZAO SOCIAL LTDA",
            "siglaUF": "SP",
            "schemes": "PL_008_V4",
            "versao": "4.00"
        }"#;
        let err = validate_config(json).unwrap_err();
        match &err {
            FiscalError::ConfigValidation(msg) => assert!(msg.contains("[cnpj]")),
            _ => panic!("expected ConfigValidation, got: {err:?}"),
        }
    }

    #[test]
    fn missing_schemes_fails() {
        let json = r#"{
            "tpAmb": 2,
            "razaosocial": "SUA RAZAO SOCIAL LTDA",
            "siglaUF": "SP",
            "cnpj": "99999999999999",
            "versao": "4.00"
        }"#;
        let err = validate_config(json).unwrap_err();
        match &err {
            FiscalError::ConfigValidation(msg) => assert!(msg.contains("[schemes]")),
            _ => panic!("expected ConfigValidation, got: {err:?}"),
        }
    }

    #[test]
    fn missing_versao_fails() {
        let json = r#"{
            "tpAmb": 2,
            "razaosocial": "SUA RAZAO SOCIAL LTDA",
            "siglaUF": "SP",
            "cnpj": "99999999999999",
            "schemes": "PL_009_V4"
        }"#;
        let err = validate_config(json).unwrap_err();
        match &err {
            FiscalError::ConfigValidation(msg) => assert!(msg.contains("[versao]")),
            _ => panic!("expected ConfigValidation, got: {err:?}"),
        }
    }

    #[test]
    fn config_with_cpf_is_valid() {
        let json = r#"{
            "tpAmb": 2,
            "razaosocial": "SUA RAZAO SOCIAL LTDA",
            "siglaUF": "SP",
            "cnpj": "99999999999",
            "schemes": "PL_009_V4",
            "versao": "4.00"
        }"#;
        let config = validate_config(json).unwrap();
        assert_eq!(config.cnpj, "99999999999");
    }

    #[test]
    fn invalid_tp_amb_fails() {
        let json = r#"{
            "tpAmb": 3,
            "razaosocial": "SUA RAZAO SOCIAL LTDA",
            "siglaUF": "SP",
            "cnpj": "99999999999999",
            "schemes": "PL_009_V4",
            "versao": "4.00"
        }"#;
        let err = validate_config(json).unwrap_err();
        match &err {
            FiscalError::ConfigValidation(msg) => assert!(msg.contains("[tpAmb]")),
            _ => panic!("expected ConfigValidation, got: {err:?}"),
        }
    }

    #[test]
    fn invalid_sigla_uf_length_fails() {
        let json = r#"{
            "tpAmb": 2,
            "razaosocial": "SUA RAZAO SOCIAL LTDA",
            "siglaUF": "SPP",
            "cnpj": "99999999999999",
            "schemes": "PL_009_V4",
            "versao": "4.00"
        }"#;
        let err = validate_config(json).unwrap_err();
        match &err {
            FiscalError::ConfigValidation(msg) => assert!(msg.contains("[siglaUF]")),
            _ => panic!("expected ConfigValidation, got: {err:?}"),
        }
    }

    #[test]
    fn invalid_cnpj_format_fails() {
        let json = r#"{
            "tpAmb": 2,
            "razaosocial": "SUA RAZAO SOCIAL LTDA",
            "siglaUF": "SP",
            "cnpj": "123",
            "schemes": "PL_009_V4",
            "versao": "4.00"
        }"#;
        let err = validate_config(json).unwrap_err();
        match &err {
            FiscalError::ConfigValidation(msg) => assert!(msg.contains("[cnpj]")),
            _ => panic!("expected ConfigValidation, got: {err:?}"),
        }
    }

    #[test]
    fn cnpj_with_non_digits_fails() {
        let json = r#"{
            "tpAmb": 2,
            "razaosocial": "SUA RAZAO SOCIAL LTDA",
            "siglaUF": "SP",
            "cnpj": "93.623.057/0001-28",
            "schemes": "PL_009_V4",
            "versao": "4.00"
        }"#;
        let err = validate_config(json).unwrap_err();
        match &err {
            FiscalError::ConfigValidation(msg) => assert!(msg.contains("[cnpj]")),
            _ => panic!("expected ConfigValidation, got: {err:?}"),
        }
    }

    #[test]
    fn null_optional_fields_are_accepted() {
        let json = r#"{
            "tpAmb": 2,
            "razaosocial": "SUA RAZAO SOCIAL LTDA",
            "siglaUF": "SP",
            "cnpj": "99999999999999",
            "schemes": "PL_009_V4",
            "versao": "4.00",
            "tokenIBPT": null,
            "CSC": null,
            "CSCid": null,
            "aProxyConf": null
        }"#;
        let config = validate_config(json).unwrap();
        assert!(config.token_ibpt.is_none());
        assert!(config.csc.is_none());
        assert!(config.csc_id.is_none());
        assert!(config.a_proxy_conf.is_none());
    }

    #[test]
    fn multiple_missing_fields_reports_all() {
        let json = r#"{}"#;
        let err = validate_config(json).unwrap_err();
        match &err {
            FiscalError::ConfigValidation(msg) => {
                assert!(msg.contains("[tpAmb]"));
                assert!(msg.contains("[razaosocial]"));
                assert!(msg.contains("[siglaUF]"));
                assert!(msg.contains("[cnpj]"));
                assert!(msg.contains("[schemes]"));
                assert!(msg.contains("[versao]"));
            }
            _ => panic!("expected ConfigValidation, got: {err:?}"),
        }
    }

    #[test]
    fn tp_amb_production_is_valid() {
        let json = r#"{
            "tpAmb": 1,
            "razaosocial": "EMPRESA PROD",
            "siglaUF": "MG",
            "cnpj": "12345678901234",
            "schemes": "PL_009_V4",
            "versao": "4.00"
        }"#;
        let config = validate_config(json).unwrap();
        assert_eq!(config.tp_amb, 1);
    }

    #[test]
    fn proxy_config_fields_parsed() {
        let json = r#"{
            "tpAmb": 2,
            "razaosocial": "EMPRESA LTDA",
            "siglaUF": "RJ",
            "cnpj": "12345678901234",
            "schemes": "PL_009_V4",
            "versao": "4.00",
            "aProxyConf": {
                "proxyIp": "192.168.1.1",
                "proxyPort": "8080",
                "proxyUser": "user",
                "proxyPass": "pass"
            }
        }"#;
        let config = validate_config(json).unwrap();
        let proxy = config.a_proxy_conf.unwrap();
        assert_eq!(proxy.proxy_ip.as_deref(), Some("192.168.1.1"));
        assert_eq!(proxy.proxy_port.as_deref(), Some("8080"));
        assert_eq!(proxy.proxy_user.as_deref(), Some("user"));
        assert_eq!(proxy.proxy_pass.as_deref(), Some("pass"));
    }
}
