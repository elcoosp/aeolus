//! The token-table microbenchmarks (divan, stable — the workspace bench
//! idiom). These are the plan §3.4 paths the theme pass owns: the table
//! build (the per-theme cold path, once per theme), the runtime-token
//! resolve (the T2 hot path: one load + the alpha ride) and the baked
//! resolve (the T0/T1 hot path: literal or pair index).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (see `reports/10-tdd-evidence/14-theme-pass.md`
//! for the method note).

use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The shadcn light+dark source (the embedded palettes) — the build
/// benches' input.
fn shadcn_source() -> ThemeSource {
    ThemeSource::shadcn_dark().expect("embedded palettes parse")
}

/// The table build, T1 shape (light + dark pairs) — once per theme.
#[divan::bench]
fn table_baked_shadcn_cold(bencher: Bencher) {
    let source = shadcn_source();
    bencher.bench(|| TokenTable::baked(&source).expect("default-seeded source builds"));
}

/// The table build, T2 shape — same build, different tier tag.
#[divan::bench]
fn table_runtime_shadcn_cold(bencher: Bencher) {
    let source = shadcn_source();
    bencher.bench(|| TokenTable::runtime(&source).expect("default-seeded source builds"));
}

/// The T2 hot path: id lookup + the runtime resolve, no modifier (one
/// binary search + one slot read + the `Tok` construction).
#[divan::bench]
fn resolve_token_hot(bencher: Bencher) {
    let source = shadcn_source();
    let table = TokenTable::runtime(&source).expect("builds");
    let id = table.id("primary").expect("primary is a shadcn token");
    bencher.bench(|| table.resolve(id, None).expect("resolves"));
}

/// The T2 hot path with the `/90` modifier riding the slot.
#[divan::bench]
fn resolve_token_alpha_hot(bencher: Bencher) {
    let source = shadcn_source();
    let table = TokenTable::runtime(&source).expect("builds");
    let id = table.id("primary").expect("primary is a shadcn token");
    bencher.bench(|| table.resolve(id, Some(0.9)).expect("resolves"));
}

/// The T1 hot path: the baked pair index (the mode-select form).
#[divan::bench]
fn resolve_token_baked_pair_hot(bencher: Bencher) {
    let source = shadcn_source();
    let table = TokenTable::baked(&source).expect("builds");
    let id = table.id("primary").expect("primary is a shadcn token");
    bencher.bench(|| table.resolve(id, None).expect("resolves"));
}

/// The `@source` scan (the §5.4 registration seam): the statement walk
/// over a theme CSS — comment strip, the three forms, source order.
/// The shadcn source (no directives) is the clean-file floor.
#[divan::bench]
fn source_directives_scan(bencher: Bencher) {
    let css = "@theme {\n  --color-primary: #0ea5e9;\n}\n\
               /* registration */\n\
               @source inline(\"p-4 bg-primary hover:bg-primary/90\");\n\
               @source \"src/views.rs\";\n\
               @source not \"src/generated.rs\";\n";
    bencher.bench(|| aeolus_theme::source_directives(css).expect("well-formed directives parse"));
}

/// The scan's clean floor: the built-in palette carries no directives —
/// the walk is one comment strip + one failed `find`.
#[divan::bench]
fn source_directives_clean_floor(bencher: Bencher) {
    let css = shadcn_css_light();
    bencher.bench(|| aeolus_theme::source_directives(css).expect("no directives"));
}

fn shadcn_css_light() -> &'static str {
    // The embedded palette's own light CSS is crate-private; the bench
    // rides the public parse over an equivalent-size file instead.
    const LIGHT: &str = include_str!("../../aeolus-core/data/shadcn-light.css");
    LIGHT
}
