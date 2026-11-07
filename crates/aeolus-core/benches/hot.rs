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

// v0.7 (AEOLUS_V0.7.md §8): the variant-engine surfaces. The Input table
// carries the pseudo-element variants (selection:/placeholder:/file:) that
// v0.7 taught the parser; the Card header carries the named container +
// arbitrary row template + auto-rows + conditional grid-cols.
const INPUT: &str = "h-9 w-full min-w-0 rounded-md border border-input bg-transparent px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none selection:bg-primary selection:text-primary-foreground file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground placeholder:text-muted-foreground disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm dark:bg-input/30";

const CARD_HEADER: &str = "@container/card-header grid auto-rows-min grid-rows-[auto_auto] items-start gap-2 px-6 has-data-[slot=card-action]:grid-cols-[1fr_auto] [.border-b]:pb-6";

#[divan::bench]
fn resolve_input_cold(b: Bencher) {
    let t = theme();
    b.counter(ItemsCount::new(INPUT.split_ascii_whitespace().count()))
        .bench(|| Resolver::new(&t).resolve(divan::black_box(INPUT)).unwrap());
}

#[divan::bench]
fn resolve_card_header_cold(b: Bencher) {
    let t = theme();
    b.counter(ItemsCount::new(
        CARD_HEADER.split_ascii_whitespace().count(),
    ))
    .bench(|| {
        Resolver::new(&t)
            .resolve(divan::black_box(CARD_HEADER))
            .unwrap()
    });
}
