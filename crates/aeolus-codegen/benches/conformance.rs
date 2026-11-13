//! The conformance score's microbenchmarks (divan, stable — the
//! workspace bench idiom): the §5.1 seam's two costs, the report walk
//! (one verdict per longhand over `Prop::ALL`) and the rendered table
//! (the artifact contract's byte-stable Display). Both read a local
//! fixture capability — the seam is backend-neutral, so the bench
//! never links a UI framework.

use aeolus_codegen::conformance::{Capability, Conformance, Tier};
use aeolus_ir::Prop;
use divan::Bencher;

fn main() {
    divan::main();
}

/// The fixture: native except one polyfill and a few refusals — the
/// realistic three-tier shape (the GPUI table's own profile).
struct Fixture;

impl Capability for Fixture {
    fn name(&self) -> &'static str {
        "fixture"
    }

    fn tier(&self, prop: Prop) -> Tier {
        match prop {
            Prop::RingWidth | Prop::RingColor => Tier::Polyfill,
            Prop::Rotate | Prop::TransitionProperty | Prop::Animation => Tier::Refused,
            _ => Tier::Native,
        }
    }
}

#[divan::bench]
fn build_the_report(bencher: Bencher) {
    bencher.bench(|| Conformance::of(&Fixture));
}

#[divan::bench]
fn render_the_report(bencher: Bencher) {
    let report = Conformance::of(&Fixture);
    bencher.bench(|| report.to_string());
}
