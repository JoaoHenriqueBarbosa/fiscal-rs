#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Split input: first 20 bytes as tag name, rest as XML
        let (tag_part, xml_part) = if s.len() > 20 {
            (&s[..20], &s[20..])
        } else {
            (s, s)
        };
        let tag_name = tag_part.trim();
        if !tag_name.is_empty() {
            let _ = fiscal_core::xml_utils::extract_xml_tag_value(xml_part, tag_name);
        }
    }
});
