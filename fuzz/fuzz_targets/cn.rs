//! Fuzz `cn()`: the merge must never panic and must be idempotent even on
//! adversarial input.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let classes: Vec<&str> = s.split('\u{0}').collect();
        let merged = aeolus_core::cn(&classes);
        let merged_words: Vec<&str> = merged.split_ascii_whitespace().collect();
        // Idempotence on every input.
        let twice = aeolus_core::cn(&merged_words);
        assert_eq!(twice, merged, "cn must be idempotent on {s:?}");
    }
});
