//! Fuzz the metamorphic invariant (architecture plan §5.3's differential
//! line): arbitrary bytes → class string → the three pipeline walks
//! (compile ⇒ interpret ⇒ equals emit) must never panic, the two
//! diagnostics channels must report IDENTICALLY on every input, the
//! sheet must always be the per-candidate parts, and every clean
//! lowering (no diagnostics on either channel) must AGREE end to end —
//! the corpus's documented divergences ride the report's typed notes,
//! never a silent mismatch.
//!
//! The deterministic corpus battery lives in
//! `crates/aeolus-compile/tests/metamorphic.rs`; this target covers the
//! adversarial half the battery cannot enumerate.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // The whole-string failure stays the empty input (plan §3.1's
        // totality rule); whitespace-only inputs are that same refusal.
        if s.trim().is_empty() {
            return;
        }
        let config = aeolus_compile::Config::default();
        // Panic freedom IS the first invariant: every walk below is
        // total on non-empty input, and a panic here is the bug.
        let report = match aeolus_compile::metamorphic_check(s, &config) {
            Ok(report) => report,
            Err(err) => {
                assert!(
                    matches!(err, aeolus_compile::CompileError::Empty),
                    "only the empty input errors: {err}"
                );
                return;
            }
        };
        // The diagnostics channels are byte-equal on EVERY input —
        // lowered or refused, the two walks share the typed surfaces.
        assert!(report.diagnostics_equal, "diagnostics drifted on {s:?}");
        // The sheet is the per-candidate parts, on every input.
        assert!(report.compositional, "the sheet decomposed on {s:?}");
        // A clean lowering answers on BOTH sides — the answer must be
        // the same one (the strong contract the battery pins over the
        // corpus; here the adversarial grammar walks it).
        let clean = aeolus_compile::compile(s, &config)
            .map(|compiled| compiled.diagnostics.is_empty())
            .unwrap_or(false);
        if clean {
            assert!(report.agree(), "a clean lowering diverged on {s:?}:\n{report}");
        }
    }
});
