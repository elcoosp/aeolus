//! Fuzz target (v0.2, plan §5.7): class-string → plan → `Probe` evaluation
//! must be deterministic and panic-free for ANY probe answer combination.

#![no_main]

use libfuzzer_sys::fuzz_target;

use aeolus_core::eval::Probe;
use aeolus_core::ast::Media;
use aeolus_core::Test;

/// A probe driven by raw fuzz bytes (every answer flips deterministically).
struct FuzzProbe {
    bits: u32,
}

impl Probe for FuzzProbe {
    fn test(&self, t: &Test) -> bool {
        match t {
            Test::State(_) => self.bits & 1 != 0,
            Test::Data(_, _) => self.bits & 2 != 0,
            Test::Aria(_, _) => self.bits & 4 != 0,
            Test::Structural(_) => self.bits & 8 != 0,
        }
    }

    fn media(&self, m: Media) -> bool {
        matches!(m, Media::Dark) && self.bits & 16 != 0
    }

    fn viewport_width(&self) -> Option<f32> {
        (self.bits & 32 != 0).then(|| {
            let raw = (self.bits >> 8) as f32;
            raw.abs().clamp(0.0, 100_000.0)
        })
    }

    fn container_width(&self, _name: Option<&str>) -> Option<f32> {
        (self.bits & 64 != 0).then(|| {
            let raw = (self.bits >> 12) as f32;
            raw.abs().clamp(0.0, 100_000.0)
        })
    }
}

fuzz_target!(|data: &str| {
    let theme = match aeolus_core::TailwindTheme::from_theme_css(
        aeolus_core::TAILWIND_DEFAULT_PALETTE,
    ) {
        Ok(t) => t,
        Err(_) => return,
    };
    let resolver = aeolus_core::Resolver::new(&theme);
    if let Ok(r) = resolver.resolve(data) {
        // Determinism: two evaluations with the same probe agree.
        let probe = FuzzProbe { bits: data.len() as u32 ^ 0x5a5a_5a5a };
        let a = r.plan.resolve(&probe);
        let b = r.plan.resolve(&probe);
        assert_eq!(a, b, "Probe evaluation must be deterministic");
        // Never panics on the rules-only path either.
        let _ = r.plan.resolve_rules_only(&probe);
    }
});
