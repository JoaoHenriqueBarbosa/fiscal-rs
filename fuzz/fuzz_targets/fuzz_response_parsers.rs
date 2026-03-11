#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = fiscal_sefaz::response_parsers::parse_autorizacao_response(s);
        let _ = fiscal_sefaz::response_parsers::parse_status_response(s);
        let _ = fiscal_sefaz::response_parsers::parse_cancellation_response(s);
    }
});
