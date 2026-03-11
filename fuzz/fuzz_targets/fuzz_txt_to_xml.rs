#![no_main]
use libfuzzer_sys::fuzz_target;

/// Known layout strings to exercise all code paths in `txt_to_xml`.
const LAYOUTS: &[&str] = &["local", "local_v12", "local_v13", "sebrae"];

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Use the first byte (mod number of layouts) to pick a layout, rest is TXT input.
        let (layout_idx, txt) = if let Some((first, rest)) = data.split_first() {
            ((*first as usize) % LAYOUTS.len(), rest)
        } else {
            return;
        };
        if let Ok(txt_str) = std::str::from_utf8(txt) {
            let layout = LAYOUTS[layout_idx];
            let _ = fiscal_core::convert::txt_to_xml(txt_str, layout);
        }
    }
});
