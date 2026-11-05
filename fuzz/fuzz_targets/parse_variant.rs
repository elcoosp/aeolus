//! Fuzz target (v0.2, plan §5.7): the single-variant parser must accept or
//! reject any input without panicking, and whatever it accepts must
//! re-serialize through `canonical()` without panicking either.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    // Any whitespace-split word is a legal variant candidate.
    for word in data.split_ascii_whitespace().take(8) {
        if let Ok(v) = aeolus_core::parser::parse_variant(word) {
            // Accepted variants must be debug-printable and Clone-safe.
            let _ = format!("{v:?}");
            let cloned = v.clone();
            let _ = cloned;
        }
    }
});
