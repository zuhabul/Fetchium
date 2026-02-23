#![no_main]
// Fuzz JSON deserialization of core types. Must NEVER panic.
use hsx_core::types::{ResultItem, Segment};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(json_str) = std::str::from_utf8(data) {
        let _ = serde_json::from_str::<ResultItem>(json_str);
        let _ = serde_json::from_str::<Vec<ResultItem>>(json_str);
        let _ = serde_json::from_str::<Segment>(json_str);
    }
});
