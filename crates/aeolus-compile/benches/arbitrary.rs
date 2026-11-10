//! The arbitrary-value unit grammar's microbenchmarks (divan, stable —
//! the workspace bench idiom): the value pass's last P2 slice cost —
//! compiling arbitrary-valued utilities through the grammar (classify +
//! fan + intern) against the spacing scale's own floor on the same
//! roots, and the shadcn Button string the other benches key on (its
//! `focus-visible:ring-[3px]` rides the grammar in-tree now).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (see `reports/10-tdd-evidence/18-arbitrary-values.md`
//! for the red-to-green record).

use aeolus_compile::{Config, DynCache, compile_theme};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// Arbitrary-valued utilities across the grammar's classes: units,
/// percentages, colours, hints, deg rows, and a var reference.
const ARBITRARY_CLASSES: &str = "w-[32px] p-[1.5rem] w-[50%] opacity-[0.37] rounded-[3px] bg-[#0ea5e9] \
     text-[13px] w-[length:32px] rotate-[45deg] border-[2px]";

/// The same roots on the spacing/keyword scales — the floor the grammar
/// is measured against (no arbitrary groups, same row walk).
const SCALED_CLASSES: &str = "w-8 p-6 w-1/2 opacity-37 rounded-md bg-red-500 text-sm border-2";

/// The shadcn v4 Button table verbatim — two of its classes
/// (`focus-visible:ring-[3px]`, `has-[>svg]`'s sibling grammar) ride
/// the arbitrary path in-tree now.
const BUTTON_CLASSES: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2 has-[>svg]:px-3";

fn table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

/// The grammar's cost: ten arbitrary-valued utilities through lex →
/// lookup → classify → fan → intern.
#[divan::bench]
fn compile_arbitrary_battery(bencher: Bencher) {
    let tokens = table();
    bencher.bench(|| {
        compile_theme(ARBITRARY_CLASSES, &tokens, &Config::default())
            .expect("non-empty input compiles")
    });
}

/// The spacing-scale floor on the same roots — the delta is the
/// grammar's own classify cost (the parse is the lexer's either way).
#[divan::bench]
fn compile_scaled_floor(bencher: Bencher) {
    let tokens = table();
    bencher.bench(|| {
        compile_theme(SCALED_CLASSES, &tokens, &Config::default())
            .expect("non-empty input compiles")
    });
}

/// The cached loop's shape: the same arbitrary battery through the
/// DynCache (one miss pools it; the hits are pointer-identical).
#[divan::bench]
fn dyn_cache_arbitrary_hit(bencher: Bencher) {
    let tokens = table();
    let cache = DynCache::new();
    cache
        .compile_theme(ARBITRARY_CLASSES, &tokens, &Config::default())
        .expect("non-empty input compiles");
    bencher.bench(|| {
        cache
            .compile_theme(ARBITRARY_CLASSES, &tokens, &Config::default())
            .expect("non-empty input compiles")
    });
}

/// The Button string through the grammar-bearing pipeline — the same
/// number the interp/static benches' compile floors report, kept here
/// for the emit-vs-compile ratios in one place.
#[divan::bench]
fn compile_button(bencher: Bencher) {
    let tokens = table();
    bencher.bench(|| {
        compile_theme(BUTTON_CLASSES, &tokens, &Config::default())
            .expect("non-empty input compiles");
    });
}
