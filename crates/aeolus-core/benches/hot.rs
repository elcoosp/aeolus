//! Hot-path benchmarks: resolve, diff-apply, intern, cn().
//!
//! Run with `cargo bench -p aeolus-core` (divan). Numbers recorded in
//! report 08 were measured on the CI sandbox (2 cores, shared) — treat as
//! relative, not absolute.

use aeolus_core::*;
use divan::Bencher;
use divan::counter::ItemsCount;

fn theme() -> TailwindTheme {
    TailwindTheme::from_theme_css(include_str!("../data/shadcn-light.css")).unwrap()
}

const BUTTON: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90";

fn main() {
    divan::main();
}

#[divan::bench]
fn parse_button(b: Bencher) {
    let t = theme();
    b.bench(|| {
        let resolver = Resolver::new(&t);
        let _ = divan::black_box(&resolver);
        aeolus_core::parser::parse_class_string(divan::black_box(BUTTON)).0
    });
}

#[divan::bench]
fn resolve_button_cold(b: Bencher) {
    let t = theme();
    b.counter(ItemsCount::new(BUTTON.split_ascii_whitespace().count()))
        .bench(|| Resolver::new(&t).resolve(divan::black_box(BUTTON)).unwrap());
}

#[divan::bench]
fn resolve_cached_hit(b: Bencher) {
    let t = theme();
    let cache = PlanCache::new();
    let _ = cache.resolve(&t, BUTTON).unwrap();
    b.bench(|| cache.resolve(&t, divan::black_box(BUTTON)).unwrap());
}

#[divan::bench]
fn diff_unchanged(b: Bencher) {
    let t = theme();
    let a = Resolver::new(&t).resolve(BUTTON).unwrap().plan.base;
    let c = a.clone();
    b.bench(|| plan::diff(&a, divan::black_box(&c)));
}

#[divan::bench]
fn intern_hit(b: Bencher) {
    let interner = Interner::new();
    interner.intern(BUTTON);
    b.bench(|| interner.intern(divan::black_box(BUTTON)));
}

#[divan::bench]
fn cn_button_merge(b: Bencher) {
    b.bench(|| {
        cn(&[
            "inline-flex items-center gap-2 rounded-md text-sm bg-primary",
            "px-8 py-8 rounded-full",
        ])
    });
}

#[divan::bench]
fn cn_no_conflicts(b: Bencher) {
    b.bench(|| {
        cn(&[
            "flex",
            "items-center",
            "gap-2",
            "text-sm",
            "font-medium",
            "bg-primary",
        ])
    });
}

#[divan::bench]
fn color_mix_opacity(b: Bencher) {
    let c = Color::parse("oklch(0.205 0 0)").unwrap();
    b.bench(|| c.mix_with_transparent(90.0));
}

#[divan::bench]
fn theme_parse_shadcn(b: Bencher) {
    b.bench(|| TailwindTheme::from_theme_css(include_str!("../data/shadcn-light.css")).unwrap());
}

// v0.4 (AEOLUS_V0.4.md §4): the verification emitter's cost. The report-08
// allocation-counter follow-up was re-tested: divan 0.1.21 still ships no
// `AllocCount` counter (the tracker never landed in 0.1.x) — documented in
// report 08.
#[divan::bench]
fn css_write_button(b: Bencher) {
    let t = theme();
    let r = Resolver::new(&t);
    b.bench(|| r.to_css(divan::black_box(BUTTON)).unwrap());
}

// v0.7 (AEOLUS_V0.7.md): plan algebra + snapshot artifacts. The compose
// cost rides on the same diff/overlay slot walk the rebuild path uses; the
// snapshot costs are the writer/reader pair's own (JSON parse + walk).
#[divan::bench]
fn plan_overlay_button(b: Bencher) {
    let t = theme();
    let r = Resolver::new(&t);
    let base = r.resolve(BUTTON).unwrap().plan;
    let over = r
        .resolve("px-8 py-8 rounded-full uppercase shadow-2xl ring-4")
        .unwrap()
        .plan;
    b.bench(|| divan::black_box(&base).overlay(divan::black_box(&over)));
}

/// v0.8 §1: the edge-wise overlay path on a PARTIAL-edge override — the
/// mask-driven merge the layer/application semantics ride on.
#[divan::bench]
fn plan_overlay_partial_edges(b: Bencher) {
    let t = theme();
    let r = Resolver::new(&t);
    let base = r.resolve(BUTTON).unwrap().plan;
    let over = r.resolve("px-8 rounded-tr-full hover:py-2").unwrap().plan;
    b.bench(|| divan::black_box(&base).overlay(divan::black_box(&over)));
}

#[divan::bench]
fn snapshot_write_button(b: Bencher) {
    let t = theme();
    let plan = Resolver::new(&t).resolve(BUTTON).unwrap().plan;
    b.bench(|| divan::black_box(&plan).to_snapshot_json());
}

#[divan::bench]
fn snapshot_read_button(b: Bencher) {
    let t = theme();
    let json = Resolver::new(&t)
        .resolve(BUTTON)
        .unwrap()
        .plan
        .to_snapshot_json();
    b.bench(|| StylePlan::from_snapshot_json(divan::black_box(&json)));
}

#[divan::bench]
fn snapshot_roundtrip_button(b: Bencher) {
    let t = theme();
    let plan = Resolver::new(&t).resolve(BUTTON).unwrap().plan;
    b.bench(|| {
        let json = plan.to_snapshot_json();
        StylePlan::from_snapshot_json(&json).unwrap()
    });
}
