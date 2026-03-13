//! Attach SEFAZ authorization protocol to a signed NFe XML (`<nfeProc>` wrapper).

use crate::FiscalError;
use crate::status_codes::VALID_PROTOCOL_STATUSES;
use crate::xml_utils::extract_xml_tag_value;

use super::helpers::{
    DEFAULT_VERSION, extract_all_tags, extract_attribute, extract_inf_nfe_id, extract_tag, join_xml,
};

/// Attach the SEFAZ authorization protocol to a signed NFe XML,
/// producing the `<nfeProc>` wrapper required for storage and DANFE.
///
/// The function extracts the `<NFe>` from `request_xml` and the matching
/// `<protNFe>` from `response_xml`, validates the protocol status, and
/// joins them into a single `<nfeProc>` document.
///
/// If the response contains multiple `<protNFe>` nodes (batch response),
/// the function attempts to match by digest value and access key. When no
/// exact match is found it falls back to the first available `<protNFe>`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlParsing`] if:
/// - Either input is empty
/// - The `<NFe>` tag is missing from `request_xml`
/// - No `<protNFe>` can be found in `response_xml`
///
/// Returns [`FiscalError::SefazRejection`] if the protocol status code
/// is not in [`VALID_PROTOCOL_STATUSES`].
pub fn attach_protocol(request_xml: &str, response_xml: &str) -> Result<String, FiscalError> {
    if request_xml.is_empty() {
        return Err(FiscalError::XmlParsing("Request XML (NFe) is empty".into()));
    }
    if response_xml.is_empty() {
        return Err(FiscalError::XmlParsing(
            "Response XML (protocol) is empty".into(),
        ));
    }

    let nfe_content = extract_tag(request_xml, "NFe")
        .ok_or_else(|| FiscalError::XmlParsing("Could not find <NFe> tag in request XML".into()))?;

    // Extract digest and access key from the NFe for matching
    let digest_nfe = extract_xml_tag_value(request_xml, "DigestValue");
    let access_key = extract_inf_nfe_id(request_xml);

    // Try to find a matching protNFe by digest + access key
    let mut matched_prot: Option<String> = None;

    let prot_nodes = extract_all_tags(response_xml, "protNFe");

    for prot in &prot_nodes {
        let dig_val = extract_xml_tag_value(prot, "digVal");
        let ch_nfe = extract_xml_tag_value(prot, "chNFe");

        if let (Some(dn), Some(dv)) = (&digest_nfe, &dig_val) {
            if let (Some(ak), Some(cn)) = (&access_key, &ch_nfe) {
                if dn == dv && ak == cn {
                    // Exact match — validate status
                    let c_stat = extract_xml_tag_value(prot, "cStat").unwrap_or_default();
                    if !VALID_PROTOCOL_STATUSES.contains(&c_stat.as_str()) {
                        let x_motivo = extract_xml_tag_value(prot, "xMotivo").unwrap_or_default();
                        return Err(FiscalError::SefazRejection {
                            code: c_stat,
                            message: x_motivo,
                        });
                    }
                    matched_prot = Some(prot.clone());
                    break;
                }
            }
        }
    }

    if matched_prot.is_none() {
        // Check if any protNFe had a digVal (but didn't match)
        let mut found_dig_val = false;
        for prot in &prot_nodes {
            if extract_xml_tag_value(prot, "digVal").is_some() {
                found_dig_val = true;
                break;
            }
        }

        if !prot_nodes.is_empty() && !found_dig_val {
            // digVal is null in the response — error 18 per PHP
            let first_prot = &prot_nodes[0];
            let c_stat = extract_xml_tag_value(first_prot, "cStat").unwrap_or_default();
            let x_motivo = extract_xml_tag_value(first_prot, "xMotivo").unwrap_or_default();
            let msg = format!("digVal ausente na resposta SEFAZ: [{c_stat}] {x_motivo}");
            return Err(FiscalError::SefazRejection {
                code: c_stat,
                message: msg,
            });
        }

        if found_dig_val {
            // digVal exists but didn't match our DigestValue — error 5 per PHP
            let key_info = access_key.as_deref().unwrap_or("unknown");
            return Err(FiscalError::XmlParsing(format!(
                "Os digest são diferentes [{key_info}]"
            )));
        }

        // No protNFe at all
        let single_prot = extract_tag(response_xml, "protNFe").ok_or_else(|| {
            FiscalError::XmlParsing("Could not find <protNFe> in response XML".into())
        })?;

        // Validate status on the fallback protNFe
        let c_stat = extract_xml_tag_value(&single_prot, "cStat").unwrap_or_default();
        if !VALID_PROTOCOL_STATUSES.contains(&c_stat.as_str()) {
            let x_motivo = extract_xml_tag_value(&single_prot, "xMotivo").unwrap_or_default();
            return Err(FiscalError::SefazRejection {
                code: c_stat,
                message: x_motivo,
            });
        }
        matched_prot = Some(single_prot);
    }

    let version = extract_attribute(&nfe_content, "infNFe", "versao")
        .unwrap_or_else(|| DEFAULT_VERSION.to_string());

    Ok(join_xml(
        &nfe_content,
        &matched_prot.unwrap(),
        "nfeProc",
        &version,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attach_protocol_empty_request_xml() {
        let err = attach_protocol("", "<protNFe/>").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_protocol_empty_response_xml() {
        let err = attach_protocol("<NFe/>", "").unwrap_err();
        assert!(matches!(err, FiscalError::XmlParsing(_)));
    }

    #[test]
    fn attach_protocol_matching_digest_and_key() {
        let request = concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe35260112345678000199650010000000011123456780">"#,
            r#"<ide/></infNFe>"#,
            r#"<Signature><SignedInfo/><SignatureValue/>"#,
            r#"<KeyInfo><DigestValue>abc123</DigestValue></KeyInfo></Signature>"#,
            r#"</NFe>"#
        );
        let response = concat!(
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<digVal>abc123</digVal>"#,
            r#"<chNFe>35260112345678000199650010000000011123456780</chNFe>"#,
            r#"<cStat>100</cStat>"#,
            r#"<xMotivo>Autorizado</xMotivo>"#,
            r#"</infProt></protNFe>"#
        );
        let result = attach_protocol(request, response).unwrap();
        assert!(result.contains("<nfeProc"));
        assert!(result.contains("</nfeProc>"));
        assert!(result.contains("<NFe>"));
        assert!(result.contains("<protNFe"));
    }

    #[test]
    fn attach_protocol_rejected_status_in_exact_match() {
        let request = concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe35260112345678000199650010000000011123456780">"#,
            r#"<ide/></infNFe>"#,
            r#"<Signature><SignedInfo/><SignatureValue/>"#,
            r#"<KeyInfo><DigestValue>abc123</DigestValue></KeyInfo></Signature>"#,
            r#"</NFe>"#
        );
        let response = concat!(
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<digVal>abc123</digVal>"#,
            r#"<chNFe>35260112345678000199650010000000011123456780</chNFe>"#,
            r#"<cStat>999</cStat>"#,
            r#"<xMotivo>Rejeitada</xMotivo>"#,
            r#"</infProt></protNFe>"#
        );
        let err = attach_protocol(request, response).unwrap_err();
        assert!(matches!(err, FiscalError::SefazRejection { .. }));
    }

    #[test]
    fn attach_protocol_fallback_rejected_status() {
        // No digest match, falls back to first protNFe which is rejected
        let request = concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe35260112345678000199650010000000011123456780">"#,
            r#"<ide/></infNFe></NFe>"#
        );
        let response = concat!(
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<cStat>999</cStat>"#,
            r#"<xMotivo>Rejeitada</xMotivo>"#,
            r#"</infProt></protNFe>"#
        );
        let err = attach_protocol(request, response).unwrap_err();
        assert!(matches!(err, FiscalError::SefazRejection { .. }));
    }

    // ── attach_protocol: fallback protNFe with invalid cStat (lines 112-116) ──

    #[test]
    fn attach_protocol_fallback_prot_invalid_status() {
        // Request with NFe, digest, access key
        let request = concat!(
            r#"<NFe><infNFe versao="4.00" Id="NFe35260112345678000199550010000000011123456780">"#,
            r#"<DigestValue>abc123</DigestValue>"#,
            r#"</infNFe></NFe>"#
        );
        // Response with single protNFe that has NO digVal (trigger fallback),
        // but status is invalid
        let response = concat!(
            r#"<protNFe versao="4.00"><infProt>"#,
            r#"<cStat>999</cStat>"#,
            r#"<xMotivo>Rejeitado</xMotivo>"#,
            r#"</infProt></protNFe>"#
        );
        let err = attach_protocol(request, response).unwrap_err();
        match err {
            FiscalError::SefazRejection { code, .. } => assert_eq!(code, "999"),
            other => panic!("Expected SefazRejection, got {:?}", other),
        }
    }
}
