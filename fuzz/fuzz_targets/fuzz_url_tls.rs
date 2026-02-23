#![no_main]
// Fuzz TLS enforcement / URL parsing. Must NEVER panic regardless of input.
use hsx_core::http::tls::enforce_tls;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(url_str) = std::str::from_utf8(data) {
        let _ = enforce_tls(url_str);
    }
});
