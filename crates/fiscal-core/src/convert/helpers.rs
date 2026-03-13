//! Helper functions for TXT validation and XML string building.

use std::collections::HashMap;
use crate::xml_utils::escape_xml;
use super::get_structure;

pub(super) type Fields = HashMap<String, String>;

pub(super) fn fields_to_std(fields: &[&str], struct_def: &str) -> Fields {
    let struct_fields: Vec<&str> = struct_def.split('|').collect();
    let mut std = Fields::new();
    let len = struct_fields.len().saturating_sub(1);
    for (i, name) in struct_fields.iter().enumerate().take(len).skip(1) {
        let data = fields.get(i).copied().unwrap_or("");
        if !name.is_empty() && !data.is_empty() {
            std.insert(name.to_string(), data.to_string());
        }
    }
    std
}

pub(super) fn xml_tag(name: &str, content: &str) -> String {
    format!("<{name}>{content}</{name}>")
}

pub(super) fn add_child(arr: &mut Vec<String>, name: &str, value: Option<&str>) {
    if let Some(v) = value {
        arr.push(format!("<{name}>{}</{name}>", escape_xml(v)));
    }
}

pub(super) fn add_child_str(arr: &mut Vec<String>, name: &str, value: &str) {
    arr.push(format!("<{name}>{}</{name}>", escape_xml(value)));
}

/// Pad a decimal string value to the given number of decimal places.
///
/// If the value already has a decimal point, it pads (or truncates) to
/// exactly `places` decimal digits. If the value has no decimal point, a
/// `.` followed by `places` zeros is appended. Empty strings are returned
/// unchanged.
pub(super) fn pad_decimal(value: &str, places: usize) -> String {
    if value.is_empty() {
        return String::new();
    }
    if let Some(dot_pos) = value.find('.') {
        let integer = &value[..dot_pos];
        let frac = &value[dot_pos + 1..];
        if frac.len() >= places {
            format!("{integer}.{}", &frac[..places])
        } else {
            let mut padded = String::from(frac);
            while padded.len() < places {
                padded.push('0');
            }
            format!("{integer}.{padded}")
        }
    } else {
        let zeros = "0".repeat(places);
        format!("{value}.{zeros}")
    }
}

/// Add a child element with a decimal value padded to `places` decimal digits.
pub(super) fn add_child_dec(arr: &mut Vec<String>, name: &str, value: Option<&str>, places: usize) {
    if let Some(v) = value {
        let formatted = pad_decimal(v, places);
        arr.push(format!("<{name}>{}</{name}>", escape_xml(&formatted)));
    }
}

/// Add a child element (from a non-empty string) with decimal padding.
pub(super) fn add_child_str_dec(arr: &mut Vec<String>, name: &str, value: &str, places: usize) {
    let formatted = pad_decimal(value, places);
    arr.push(format!("<{name}>{}</{name}>", escape_xml(&formatted)));
}

pub(super) fn validate_txt_lines(lines: &[&str], layout: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let mut num = 0;
    let mut entities: Option<HashMap<&str, &str>> = None;

    for row in lines {
        if row.is_empty() {
            continue;
        }
        let fields: Vec<&str> = row.split('|').collect();
        let ref_upper = fields[0].to_uppercase();
        if ref_upper.is_empty() {
            continue;
        }
        if ref_upper == "NOTAFISCAL" {
            continue;
        }

        if ref_upper == "A" {
            num = 0;
            let ver = fields.get(1).unwrap_or(&"4.00");
            entities = get_structure(ver, layout).ok();
        }
        if ref_upper == "I" {
            num += 1;
        }

        // Check trailing pipe
        let last_char = row.chars().last().unwrap_or(' ');
        if last_char != '|' {
            let char_desc = match last_char {
                ' ' => "[ESP]".to_string(),
                '\r' => "[CR]".to_string(),
                '\t' => "[TAB]".to_string(),
                _ => String::new(),
            };
            errors.push(format!(
                "ERRO: ({num}) Todas as linhas devem terminar com 'pipe' e n\u{e3}o {char_desc}. [{row}]"
            ));
            continue;
        }

        let ent = match &entities {
            Some(e) => e,
            None => {
                errors.push("ERRO: O TXT n\u{e3}o cont\u{e9}m um marcador A".into());
                return errors;
            }
        };

        if !ent.contains_key(ref_upper.as_str()) {
            errors.push(format!(
                "ERRO: ({num}) Essa refer\u{ea}ncia n\u{e3}o est\u{e1} definida. [{row}]"
            ));
            continue;
        }

        let count = fields.len() - 1;
        let def = ent[ref_upper.as_str()];
        let default_count = def.split('|').count() - 1;
        if default_count != count {
            errors.push(format!(
                "ERRO: ({num}) O n\u{fa}mero de par\u{e2}metros na linha est\u{e1} errado (esperado #{default_count}) -> (encontrado #{count}). [ {row} ] Esperado [ {def} ]"
            ));
            continue;
        }

        // Check fields for forbidden characters
        for field in &fields {
            if field.is_empty() {
                continue;
            }
            if !field.trim().is_empty() && field.chars().all(|c| c == ' ') {
                errors.push(format!(
                    "ERRO: ({num}) Existem apenas espa\u{e7}os no campo dos dados. [{row}]"
                ));
                continue;
            }
            if field.contains('>')
                || field.contains('<')
                || field.contains('"')
                || field.contains('\'')
                || field.contains('\t')
                || field.contains('\r')
            {
                errors.push(format!(
                    "ERRO: ({num}) Existem caracteres especiais n\u{e3}o permitidos, como por ex. caracteres de controle, sinais de maior ou menor, aspas ou apostrofes, na entidade [{row}]"
                ));
                continue;
            }
        }
    }

    errors
}
