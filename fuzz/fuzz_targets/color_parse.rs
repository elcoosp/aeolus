//! Fuzz `Color::parse`: no panic on any input; parsed colors convert to
//! finite sRGB.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(c) = aeolus_core::Color::parse(s) {
            let [r, g, b, a] = c.to_srgba();
            assert!(
                r.is_finite() && g.is_finite() && b.is_finite() && a.is_finite(),
                "parsed color {s:?} must convert to finite sRGB"
            );
            let _ = c.to_rgba8();
            let _ = c.to_oklch_string();
        }
    }
});
