//! Fuzz `parse_class_string`: the parser must never panic — any input is
//! either parsed or reported in `errors` (honest-degradation contract).

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let (tokens, errors) = aeolus_core::parser::parse_class_string(s);
        // Contract: every token either parses or produces an error entry,
        // and the total count matches the whitespace-split word count.
        let words = s.split_ascii_whitespace().count();
        assert_eq!(
            tokens.len() + errors.len(),
            words,
            "parser must account for every word"
        );
    }
});
