#![no_main]
// Fuzz robots.txt parsing. Must NEVER panic.
use hsx_core::http::robots::parse_robots_txt;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(content) = std::str::from_utf8(data) {
        let rules = parse_robots_txt(content);
        // Verify allows() doesn't panic on arbitrary paths
        let _ = rules.allows("/");
        let _ = rules.allows("/admin/panel");
    }
});
