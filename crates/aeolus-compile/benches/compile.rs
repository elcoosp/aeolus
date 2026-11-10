//! The compile-seam microbenchmarks (divan, stable — the workspace bench
//! idiom). These are the plan §3.1 cold paths the pipeline owns per
//! class string: the single-utility compile, the variant-stack fold,
//! the full shadcn Button surface (the v0.x hot-path metric's input,
//! now through the V1 seam — arbitrary-variant classes ride the
//! degradation path, which is PART of the measured cost), and the
//! typed-miss cold path the linter hits.
//!
//! The bench bodies `expect` on the compile: a non-empty class string
//! always produces a `Compiled` (only the empty input hard-fails), and
//! `Compiled` deliberately has no `Default` — an "empty compile" no
//! string produced is not a value this seam should fabricate.
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (see `reports/10-tdd-evidence/13-compile-seam.md`
//! for the method note).

use aeolus_compile::{
    Config, CustomForm, CustomUtility, Customs, cn, compile, compile_theme_with, compile_with,
};
use aeolus_ir::Prop;
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The shadcn v4 Button table verbatim (base + default variant + default
/// size, `crates/aeolus-shadcn/src/classes.rs` @ 4aaea19) — the same
/// 33-class string the v0.x hot-path benchmarks key on.
const BUTTON_CLASSES: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2 has-[>svg]:px-3";

/// A user-override string in the `cn(base, user)` shape the merge
/// exists for: shape-overlapping utilities (`rounded-full`, `px-6`)
/// plus a theme-token colour with a dark variant — the knobs the
/// last-wins dedupe turns.
const USER_OVERRIDE_CLASSES: &str =
    "rounded-full px-6 py-3 bg-red-500 dark:bg-primary/40 text-base";

/// One utility, no variants — the pipeline's floor (lex, one lookup,
/// one 4-decl fan-out, one layer).
#[divan::bench]
fn compile_single_utility_cold(bencher: Bencher) {
    bencher.bench(|| compile("p-4", &Config::default()).expect("non-empty input compiles"));
}

/// A stacked variant fold (`hover:focus:md:`) — the classification walk
/// per segment plus the three-condition assembly.
#[divan::bench]
fn compile_variant_stack_cold(bencher: Bencher) {
    bencher.bench(|| {
        compile("hover:focus:md:p-0", &Config::default()).expect("non-empty input compiles")
    });
}

/// The full shadcn Button surface: scale/keyword lowering AND the
/// honest degradation path (colour tokens, arbitrary variants) in one
/// measured pass — the number backends will key on.
#[divan::bench]
fn compile_button_surface_cold(bencher: Bencher) {
    bencher
        .bench(|| compile(BUTTON_CLASSES, &Config::default()).expect("non-empty input compiles"));
}

/// The typed-miss cold path (`UnknownRoot`): the linter's "did you
/// mean" seed with its diagnostic allocation.
#[divan::bench]
fn compile_unknown_root_cold(bencher: Bencher) {
    bencher
        .bench(|| compile("not-a-utility", &Config::default()).expect("non-empty input compiles"));
}

/// The IR-level `cn()` merge (plan §3.2 "hosts the `cn()` engine at IR
/// level"): the shadcn Button surface merged with a user override,
/// rightmost-wins — concatenate, cascade-sort, last-wins dedupe,
/// intern. The cold path the backends' `cn(base, user)` composition
/// rides.
#[divan::bench]
fn cn_button_surface_with_override_cold(bencher: Bencher) {
    let base = compile(BUTTON_CLASSES, &Config::default()).expect("non-empty input compiles");
    let user =
        compile(USER_OVERRIDE_CLASSES, &Config::default()).expect("non-empty input compiles");
    bencher.bench(|| cn(&[&base, &user]).expect("non-empty inputs merge"));
}

/// The custom-authoring seam's registrations (plan §5.4's
/// `@utility`/`@custom-variant` surface): one functional spacing
/// utility over the padding fan-out — the same lowering shape as the
/// built-in `p-*` row, read through the custom fallback.
#[divan::bench]
fn compile_custom_utility_cold(bencher: Bencher) {
    let customs = Customs::new().utility(CustomUtility::functional(
        "brand-p",
        CustomForm::Spacing,
        &[
            Prop::PaddingTop,
            Prop::PaddingRight,
            Prop::PaddingBottom,
            Prop::PaddingLeft,
        ],
    ));
    bencher.bench(|| {
        compile_with("brand-p-4", &customs, &Config::default()).expect("non-empty input compiles")
    });
}

/// The custom seam's theme-token functional form (`--value(color)`):
/// the suffix resolves through the runtime-tier `TokenTable` — the
/// built-in colour path's own resolve, read through the custom
/// fallback.
#[divan::bench]
fn compile_custom_token_utility_cold(bencher: Bencher) {
    let source = ThemeSource::shadcn_dark().expect("the embedded shadcn source parses");
    let tokens = TokenTable::runtime(&source).expect("the pair surface fits the u16 slot");
    let customs = Customs::new().utility(CustomUtility::functional(
        "brand-ink",
        CustomForm::Token,
        &[Prop::Color],
    ));
    bencher.bench(|| {
        compile_theme_with("brand-ink-primary", &tokens, &customs, &Config::default())
            .expect("non-empty input compiles")
    });
}
