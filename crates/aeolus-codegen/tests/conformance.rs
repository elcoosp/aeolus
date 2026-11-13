//! The per-backend conformance score's contracts (architecture plan
//! §5.1: "Publish a per-backend conformance score (WPT / caniuse-style
//! badge)" — the third artifact story's seam): the [`Capability`] trait
//! (one tiered verdict per IR longhand) and the [`Conformance`] report
//! (the §5.1 score, the partition over `Prop::ALL`, the rendered
//! table).
//!
//! The pins:
//!
//! * the partition is COMPLETE — every longhand of [`Prop::ALL`] lands
//!   in exactly one tier, so a capability table that forgets a prop is
//!   a test failure, not a silent score inflation;
//! * the score math is the report's own partition (expressible over
//!   total; the native-only strict score), never an independent claim;
//! * the render is deterministic — same capability, same bytes (the
//!   artifact contract the committed `docs/CONFORMANCE.md` page rides);
//! * a demo target pins the whole shape end to end without touching
//!   any real backend's tables.
//!
//! Written FIRST (the RED spec): the suite fails until the seam
//! satisfies every pin below.

use std::fmt::Write as _;

use aeolus_codegen::conformance::{Capability, Conformance, Tier};
use aeolus_ir::Prop;

// ---------------------------------------------------------------------------
// The demo target: every longhand native except two known refusals and
// one polyfill — the fixed point the pins read against.
// ---------------------------------------------------------------------------

/// The demo capability: `BoxShadow` polyfilled (the ring precedent),
/// `Rotate` and `TransitionProperty` refused (the motion boundary),
/// everything else native. Deliberately NOT a real backend's table —
/// the pins must hold for ANY capability, so the fixture is local.
struct Demo;

impl Capability for Demo {
    fn name(&self) -> &'static str {
        "demo"
    }

    fn tier(&self, prop: Prop) -> Tier {
        match prop {
            Prop::BoxShadow => Tier::Polyfill,
            Prop::Rotate | Prop::TransitionProperty => Tier::Refused,
            _ => Tier::Native,
        }
    }
}

// ---------------------------------------------------------------------------
// 1. The partition: complete, disjoint, Prop::ALL-shaped
// ---------------------------------------------------------------------------

#[test]
fn the_partition_covers_every_longhand_exactly_once() {
    let report = Conformance::of(&Demo);
    let total = report.native().len() + report.polyfill().len() + report.refused().len();
    assert_eq!(
        total,
        Prop::ALL.len(),
        "the tiers must partition Prop::ALL exactly: {total} vs {}",
        Prop::ALL.len()
    );
    // Disjoint: no longhand lands twice.
    let mut seen: Vec<Prop> = Vec::new();
    for prop in report
        .native()
        .iter()
        .chain(report.polyfill())
        .chain(report.refused())
    {
        assert!(!seen.contains(prop), "{prop:?} lands in two tiers");
        seen.push(*prop);
    }
    // And every longhand lands SOMEWHERE.
    for prop in Prop::ALL {
        assert!(seen.contains(prop), "{prop:?} landed in no tier");
    }
}

#[test]
fn the_tiers_carry_the_fixture_verdicts() {
    let report = Conformance::of(&Demo);
    assert!(report.native().contains(&Prop::PaddingTop));
    assert!(report.polyfill().contains(&Prop::BoxShadow));
    assert!(report.refused().contains(&Prop::Rotate));
    assert!(report.refused().contains(&Prop::TransitionProperty));
}

// ---------------------------------------------------------------------------
// 2. The score math: the partition's own arithmetic
// ---------------------------------------------------------------------------

#[test]
fn the_score_reads_the_partition() {
    let report = Conformance::of(&Demo);
    let total = Prop::ALL.len() as f32;
    let expressible = (report.native().len() + report.polyfill().len()) as f32;
    let expected = expressible / total * 100.0;
    assert!(
        (report.score() - expected).abs() < 1e-6,
        "score {} vs partition math {expected}",
        report.score()
    );
    let native_expected = report.native().len() as f32 / total * 100.0;
    assert!(
        (report.native_score() - native_expected).abs() < 1e-6,
        "native score {} vs partition math {native_expected}",
        report.native_score()
    );
    // The polyfill tier lifts the expressible score above the native one.
    assert!(report.score() > report.native_score());
}

// ---------------------------------------------------------------------------
// 3. The render: deterministic, badge-shaped, table-shaped
// ---------------------------------------------------------------------------

#[test]
fn the_render_is_deterministic_and_names_the_backend() {
    let first = Conformance::of(&Demo).to_string();
    let second = Conformance::of(&Demo).to_string();
    assert_eq!(first, second, "same capability, same bytes");
    assert!(first.contains("demo"), "{first}");
    // Every longhand's name appears in the table (the audit surface).
    for prop in Prop::ALL {
        assert!(first.contains(prop.name()), "{first} missing {prop:?}");
    }
}

#[test]
fn the_report_writes_through_fmt_write() {
    let mut buf = String::new();
    write!(buf, "{}", Conformance::of(&Demo)).expect("display writes");
    assert!(buf.contains("demo"));
}
