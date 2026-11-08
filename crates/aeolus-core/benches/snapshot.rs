//! Artifact-path benchmarks (v0.7 test-pyramid hardening): the plan
//! snapshot writer/reader and the plan-level overlay, on a RICH plan
//! (the shadcn Button table with its state layers, the v0.7 pseudo-element
//! and has-* shapes).
//!
//! Run with `cargo bench -p aeolus-core --bench snapshot` (divan). Numbers
//! recorded in report 08 were measured on the CI sandbox — treat as
//! relative, not absolute.

use aeolus_core::*;
use divan::Bencher;

fn theme() -> TailwindTheme {
    TailwindTheme::from_theme_css(include_str!("../data/shadcn-light.css")).unwrap()
}

/// The Button class table plus the v0.7 surface: state layers, a
/// pseudo-element variant, a has-data rule — every plan region the
/// snapshot schema covers.
const RICH: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 selection:bg-primary has-checked:bg-green-500 dark:bg-primary md:p-8 @container/card-header";

fn rich_plan() -> StylePlan {
    Resolver::new(&theme()).resolve(RICH).unwrap().plan
}

fn main() {
    divan::main();
}

#[divan::bench]
fn snapshot_write_rich(b: Bencher) {
    let plan = rich_plan();
    b.bench(|| plan.to_snapshot_json());
}

#[divan::bench]
fn snapshot_read_rich(b: Bencher) {
    let plan = rich_plan();
    let json = plan.to_snapshot_json();
    b.bench(|| StylePlan::from_snapshot_json(divan::black_box(&json)).unwrap());
}

#[divan::bench]
fn snapshot_roundtrip_rich(b: Bencher) {
    let plan = rich_plan();
    b.bench(|| {
        let json = plan.to_snapshot_json();
        StylePlan::from_snapshot_json(&json).unwrap()
    });
}

#[divan::bench]
fn overlay_rich(b: Bencher) {
    let base = rich_plan();
    let over = Resolver::new(&theme())
        .resolve("bg-red-500 p-4 hover:bg-blue-500 sm:text-lg")
        .unwrap()
        .plan;
    b.bench(|| base.overlay(&over));
}

#[divan::bench]
fn diff_text_only_change(b: Bencher) {
    // The regression path: two styles differing ONLY in a non-first text
    // property — the case the old single-bit text family mis-reported.
    let mut a = ResolvedStyle::default();
    a.text
        .push((plan::TextProperty::FontSize, plan::TextValue::Px(14.0)));
    a.text
        .push((plan::TextProperty::FontWeight, plan::TextValue::Weight(500)));
    let mut c = a.clone();
    c.text[1] = (plan::TextProperty::FontWeight, plan::TextValue::Weight(700));
    b.bench(|| plan::diff(&a, &c));
}
