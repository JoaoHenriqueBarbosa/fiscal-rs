use crate::FiscalError;
use crate::newtypes::IbgeCode;
use crate::types::{AccessKeyParams, InvoiceModel};
use crate::xml_builder::access_key::build_access_key;
use crate::xml_utils::extract_xml_tag_value;

use super::manager::Contingency;

/// Adjust an NF-e XML string for contingency mode.
///
/// Modifies the XML to:
/// 1. Replace the `<tpEmis>` value with the contingency emission type
/// 2. Insert `<dhCont>` (contingency datetime) and `<xJust>` (reason) inside `<ide>`
/// 3. Recalculate the access key and check digit
///
/// If the contingency is not active (no type set), returns the XML unchanged.
/// If the XML already has a non-normal `<tpEmis>` (not `1`), returns unchanged.
///
/// # Errors
///
/// Returns [`FiscalError::Contingency`] if the XML belongs to an NFC-e (model 65),
/// since SVC-AN/SVC-RS contingency does not apply to NFC-e documents.
///
/// Returns [`FiscalError::XmlParsing`] if required XML tags cannot be found.
pub fn adjust_nfe_contingency(xml: &str, contingency: &Contingency) -> Result<String, FiscalError> {
    // If no contingency is active, return XML unchanged
    if contingency.contingency_type.is_none() {
        return Ok(xml.to_string());
    }

    // Remove XML signature if present
    let mut xml = remove_signature(xml);

    // Check model - must be NF-e (55), not NFC-e (65)
    let model = extract_xml_tag_value(&xml, "mod").unwrap_or_default();
    if model == "65" {
        return Err(FiscalError::Contingency(
            "The XML belongs to a model 65 document (NFC-e), incorrect for SVCAN or SVCRS contingency.".to_string(),
        ));
    }

    // Check if already in contingency mode
    let current_tp_emis = extract_xml_tag_value(&xml, "tpEmis").unwrap_or_default();
    if current_tp_emis != "1" {
        // Already configured for contingency, return as-is
        return Ok(xml);
    }

    // Extract fields for access key recalculation
    let c_uf = extract_xml_tag_value(&xml, "cUF").unwrap_or_default();
    let c_nf = extract_xml_tag_value(&xml, "cNF").unwrap_or_default();
    let n_nf = extract_xml_tag_value(&xml, "nNF").unwrap_or_default();
    let serie = extract_xml_tag_value(&xml, "serie").unwrap_or_default();
    let dh_emi = extract_xml_tag_value(&xml, "dhEmi").unwrap_or_default();

    // Extract emitter CNPJ or CPF from <emit> block
    let emit_doc = extract_emitter_doc(&xml);

    // Parse emission date for year/month
    let (year, month) = parse_year_month(&dh_emi);

    // Format contingency datetime with timezone from dhEmi
    let tz_offset = extract_tz_offset(&dh_emi);
    let dth_cont = format_timestamp_with_offset(contingency.timestamp, &tz_offset);

    let reason = contingency.reason.as_deref().unwrap_or("").trim();
    let tp_emis = contingency.emission_type();

    // Replace tpEmis value
    xml = xml.replacen(
        &format!("<tpEmis>{current_tp_emis}</tpEmis>"),
        &format!("<tpEmis>{tp_emis}</tpEmis>"),
        1,
    );

    // Insert dhCont
    if xml.contains("<dhCont>") {
        // Replace existing dhCont
        let re_start = xml.find("<dhCont>").unwrap();
        let re_end = xml.find("</dhCont>").unwrap() + "</dhCont>".len();
        xml = format!(
            "{}<dhCont>{dth_cont}</dhCont>{}",
            &xml[..re_start],
            &xml[re_end..]
        );
    } else if xml.contains("<NFref>") {
        xml = xml.replacen("<NFref>", &format!("<dhCont>{dth_cont}</dhCont><NFref>"), 1);
    } else {
        xml = xml.replacen("</ide>", &format!("<dhCont>{dth_cont}</dhCont></ide>"), 1);
    }

    // Insert xJust
    if xml.contains("<xJust>") {
        // Replace existing xJust
        let re_start = xml.find("<xJust>").unwrap();
        let re_end = xml.find("</xJust>").unwrap() + "</xJust>".len();
        xml = format!(
            "{}<xJust>{reason}</xJust>{}",
            &xml[..re_start],
            &xml[re_end..]
        );
    } else if xml.contains("<NFref>") {
        xml = xml.replacen("<NFref>", &format!("<xJust>{reason}</xJust><NFref>"), 1);
    } else {
        xml = xml.replacen("</ide>", &format!("<xJust>{reason}</xJust></ide>"), 1);
    }

    // Recalculate access key
    let model_enum = match model.as_str() {
        "65" => InvoiceModel::Nfce,
        _ => InvoiceModel::Nfe,
    };
    let emission_type_enum = contingency.emission_type_enum();

    let new_key = build_access_key(&AccessKeyParams {
        state_code: IbgeCode(c_uf),
        year_month: format!("{year}{month}"),
        tax_id: emit_doc,
        model: model_enum,
        series: serie.parse().unwrap_or(0),
        number: n_nf.parse().unwrap_or(0),
        emission_type: emission_type_enum,
        numeric_code: c_nf,
    })?;

    // Update cDV (check digit is last char of access key)
    let new_cdv = &new_key[new_key.len() - 1..];
    // Replace <cDV> tag
    if let Some(start) = xml.find("<cDV>") {
        if let Some(end) = xml[start..].find("</cDV>") {
            let full_end = start + end + "</cDV>".len();
            xml = format!("{}<cDV>{new_cdv}</cDV>{}", &xml[..start], &xml[full_end..]);
        }
    }

    // Update infNFe Id attribute
    // Match pattern: Id="NFeXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
    if let Some(id_start) = xml.find("Id=\"NFe") {
        let after_nfe = id_start + 7; // past Id="NFe
        // Find the closing quote — the key is 44 digits
        if xml.len() >= after_nfe + 44 {
            let id_end = after_nfe + 44;
            xml = format!("{}NFe{new_key}{}", &xml[..after_nfe], &xml[id_end..]);
        }
    }

    Ok(xml)
}

// ── Private helpers ─────────────────────────────────────────────────────────

/// Remove XML digital signature block if present.
fn remove_signature(xml: &str) -> String {
    // Remove <Signature xmlns...>...</Signature>
    if let Some(start) = xml.find("<Signature") {
        if let Some(end) = xml.find("</Signature>") {
            let full_end = end + "</Signature>".len();
            return format!("{}{}", xml[..start].trim_end(), &xml[full_end..])
                .trim()
                .to_string();
        }
    }
    xml.to_string()
}

/// Extract the emitter's CNPJ or CPF from the <emit> block.
pub(super) fn extract_emitter_doc(xml: &str) -> String {
    if let Some(emit_start) = xml.find("<emit>") {
        if let Some(emit_end) = xml.find("</emit>") {
            let emit_block = &xml[emit_start..emit_end];
            // Try CNPJ first
            if let Some(cnpj) = extract_inner(emit_block, "CNPJ") {
                return cnpj;
            }
            // Then CPF
            if let Some(cpf) = extract_inner(emit_block, "CPF") {
                return cpf;
            }
        }
    }
    String::new()
}

/// Extract inner text from a simple XML tag.
fn extract_inner(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml[start..end].to_string())
}

/// Parse YY and MM from an ISO datetime string like "2018-09-25T00:00:00-03:00".
pub(super) fn parse_year_month(dh_emi: &str) -> (String, String) {
    if dh_emi.len() >= 7 {
        let year = &dh_emi[2..4]; // "18"
        let month = &dh_emi[5..7]; // "09"
        (year.to_string(), month.to_string())
    } else {
        ("00".to_string(), "00".to_string())
    }
}

/// Extract timezone offset from an ISO datetime string.
/// Returns something like "-03:00". Defaults to "-03:00" if not found.
pub(super) fn extract_tz_offset(dh_emi: &str) -> String {
    // Look for +HH:MM or -HH:MM at the end
    if dh_emi.len() >= 6 {
        let tail = &dh_emi[dh_emi.len() - 6..];
        if (tail.starts_with('+') || tail.starts_with('-')) && tail.as_bytes()[3] == b':' {
            return tail.to_string();
        }
    }
    "-03:00".to_string()
}

/// Format a unix timestamp as ISO datetime with a given timezone offset.
pub(super) fn format_timestamp_with_offset(timestamp: u64, offset: &str) -> String {
    // Parse offset to get total seconds
    let offset_seconds = parse_offset_seconds(offset);

    // Create a chrono FixedOffset and format
    if let Some(fo) = chrono::FixedOffset::east_opt(offset_seconds) {
        if let Some(dt) = chrono::DateTime::from_timestamp(timestamp as i64, 0) {
            let local = dt.with_timezone(&fo);
            return local.format("%Y-%m-%dT%H:%M:%S").to_string() + offset;
        }
    }

    // Fallback: just format as UTC
    format!("1970-01-01T00:00:00{offset}")
}

/// Parse a timezone offset string like "-03:00" into total seconds.
fn parse_offset_seconds(offset: &str) -> i32 {
    if offset.len() < 6 {
        return 0;
    }
    let sign: i32 = if offset.starts_with('-') { -1 } else { 1 };
    let hours: i32 = offset[1..3].parse().unwrap_or(0);
    let minutes: i32 = offset[4..6].parse().unwrap_or(0);
    sign * (hours * 3600 + minutes * 60)
}
