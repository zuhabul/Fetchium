#![no_main]
// Fuzz TOML config parsing. Must NEVER panic.
use fetchium_core::config::FetchiumConfig;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(toml_str) = std::str::from_utf8(data) {
        let _ = toml::from_str::<FetchiumConfig>(toml_str);
    }
});
