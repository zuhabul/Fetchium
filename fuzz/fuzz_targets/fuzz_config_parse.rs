#![no_main]
// Fuzz TOML config parsing. Must NEVER panic.
use hsx_core::config::HsxConfig;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(toml_str) = std::str::from_utf8(data) {
        let _ = toml::from_str::<HsxConfig>(toml_str);
    }
});
