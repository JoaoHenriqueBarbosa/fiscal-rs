#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = fiscal_core::standardize::identify_xml_type(s);
        let _ = fiscal_core::standardize::xml_to_json(s);
    }
});
