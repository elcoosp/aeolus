//! The sort seam's microbenchmarks (divan, stable — the workspace bench
//! idiom; plan §5.5's "class sorting (official order from the
//! registry)"). The walk is what a formatter pays per class string on
//! save: the registry read per class, the variant fold per stack, the
//! stable key sort. Three inputs:
//!
//! * the pipeline's floor (one utility, no variants),
//! * the full shadcn Button surface (the 33-class string the other
//!   benches key on — with the arbitrary-variant classes riding the
//!   honest tail, which is PART of the measured cost),
//! * the honest-tail path (unknown classes, the typed diagnostics).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (see `reports/10-tdd-evidence/13-compile-seam.md`
//! for the method note).

use aeolus_compile::sort;
use divan::Bencher;

fn main() {
    divan::main();
}

/// The shadcn v4 Button table verbatim (base + default variant + default
/// size, `crates/aeolus-shadcn/src/classes.rs` @ 4aaea19) — the same
/// 33-class string the compile/interp/rust-token benches key on.
const BUTTON_CLASSES: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2 has-[>svg]:px-3";

/// One utility, no variants — the sort's floor (lex, one lookup, one
/// key, no variants to bucket).
#[divan::bench]
fn sort_single_utility(bencher: Bencher) {
    bencher.bench(|| sort("p-4").expect("non-empty input sorts"));
}

/// The full shadcn Button surface: the registry walk over every base
/// class AND the honest tail (arbitrary variants, unsupported
/// compositions ride the typed diagnostics) in one measured pass.
#[divan::bench]
fn sort_button_classes(bencher: Bencher) {
    bencher.bench(|| sort(BUTTON_CLASSES).expect("non-empty input sorts"));
}

/// The honest tail: every class misses, so the walk degrades per class
/// into the typed diagnostics — the linter-adjacent cold path.
#[divan::bench]
fn sort_unknown_classes(bencher: Bencher) {
    bencher.bench(|| {
        sort("not-a-utility-1 not-a-utility-2 not-a-utility-3")
            .expect("non-empty input sorts (into the tail)")
    });
}
