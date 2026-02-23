#![no_main]
// Fuzz HTML extraction (CEP Layer 1). Must NEVER panic regardless of input.
use hsx_core::extract::layer1;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(html) = std::str::from_utf8(data) {
        let _ = layer1::extract(html, "https://fuzz.example.com/page");
    }
});
