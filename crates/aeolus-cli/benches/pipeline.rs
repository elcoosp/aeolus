//! Pipeline benchmarks (v0.7 §4): the headless consumption path —
//! extraction + linting + snapshot emission on realistic inputs.
//! Run with `cargo bench -p aeolus-cli`.

use aeolus_cli::{check_string, extract_candidate_strings, load_theme};
use aeolus_core::{Resolver, StylePlan, TailwindTheme, shadcn_css};
use divan::Bencher;
use std::fmt::Write as _;

fn theme() -> TailwindTheme {
    TailwindTheme::from_theme_css(shadcn_css::LIGHT).unwrap()
}

const BUTTON: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90";

/// A synthetic 1k-line source buffer mixing real class strings with prose
/// (the extraction filter must skip prose).
fn source_buffer() -> String {
    let mut s = String::with_capacity(64 * 1024);
    for i in 0..250 {
        let _ = writeln!(
            s,
            "fn comp{i}() {{ let cls = \"{BUTTON}\"; let note = \"renders the thing number {i}\"; }}"
        );
    }
    s
}

fn main() {
    divan::main();
}

#[divan::bench]
fn extract_1k_lines(b: Bencher) {
    let src = source_buffer();
    b.bench(|| extract_candidate_strings(divan::black_box(&src)));
}

#[divan::bench]
fn lint_button_string(b: Bencher) {
    let t = theme();
    b.bench(|| check_string(divan::black_box(BUTTON), &t));
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

// The `--theme <spec>` seam (v0.9.1 consumer-handoff): the built-in
// `shadcn` spec re-parses the embedded palette each call — the cost a
// CLI process pays once at startup, now measured alongside the pipeline.
#[divan::bench]
fn theme_spec_load_builtin(b: Bencher) {
    b.bench(|| load_theme(divan::black_box("shadcn")));
}

#[divan::bench]
fn theme_spec_error_render(b: Bencher) {
    // The typed-error path: constructing and rendering the failure a
    // bare-word spec produces (work the error message does, not the
    // happy path).
    b.bench(|| {
        let e = load_theme("nope").expect_err("nope is not a built-in spec");
        e.to_string()
    });
}
