//! The custom-authoring CSS-file form's microbenchmarks (divan,
//! stable — the workspace bench idiom): the registration walk (the
//! statement parse plus the `@apply` compile), the compile hit through
//! an applied utility against its expanded body, and the alignment the
//! dynamic lane's `get_with` runs. The applied registration interns —
//! one compile per registration is the intended call pattern, so the
//! interesting numbers are the PARSE cost and the per-compile hit.
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (see `reports/10-tdd-evidence/21-xtask-
//! generator-and-probe-seam.md` for the red-to-green record).

use aeolus_compile::{Config, Customs, compile_with, customs_from_css, keyword_texts_with};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The file form the examples pin: two applied utilities, one keyword
/// carrying, one variant alias.
const SHEET: &str = "@custom-variant night (&:where(.dark, .dark *));\n\
                     @utility brand-badge { @apply rounded-md px-2 py-0.5; }\n\
                     @utility brand-gone { @apply hidden; }\n";

/// The registration walk: the statement parse plus the two `@apply`
/// compiles (the registration-time cost, paid once per utility).
#[divan::bench]
fn customs_from_css_walk(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| customs_from_css(SHEET, &config).expect("parses"));
}

/// The compile hit through an applied utility — the spliced body, the
/// same shape the expanded built-in string lowers to.
#[divan::bench]
fn compile_applied(bencher: Bencher) {
    let config = Config::default();
    let customs = customs_from_css(SHEET, &config).expect("parses");
    bencher.bench(|| compile_with("hover:brand-badge", &customs, &config).expect("compiles"));
}

/// The same string through the expanded built-ins — the applied seam's
/// differential cost floor.
#[divan::bench]
fn compile_expanded(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| {
        aeolus_compile::compile("hover:rounded-md hover:px-2 hover:py-0.5", &config)
            .expect("compiles")
    });
}

/// The customs-aware annotation walk over empty registrations — the
/// additive contract's cost floor (the theme-less walk's own shape).
#[divan::bench]
fn keyword_texts_empty_customs(bencher: Bencher) {
    let config = Config::default();
    let customs = Customs::new();
    bencher.bench(|| {
        keyword_texts_with("hidden flex-col bg-primary/90", &customs, &config).expect("annotates")
    });
}

/// The functional star form's compile hit — the suffix through the
/// typed spacing grammar the registry rows ride (the file form's
/// per-compile cost, the registration having interned the form).
#[divan::bench]
fn compile_functional_star(bencher: Bencher) {
    let config = Config::default();
    let customs = customs_from_css("@utility brand-pad-* { padding: --spacing(n); }\n", &config)
        .expect("parses");
    bencher.bench(|| compile_with("brand-pad-4", &customs, &config).expect("compiles"));
}
