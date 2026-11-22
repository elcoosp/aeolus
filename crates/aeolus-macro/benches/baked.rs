//! The baked-theme bench (divan, stable — the workspace bench idiom):
//! the expansion-time walk the `theme:` argument runs —
//! `ThemeSource::from_css` (the host sheet's parse) + `TokenTable::baked`
//! (the T0/T1 bake) + `compile_theme` (the themed walk) — measured
//! beside the embedded shadcn bake the no-arg macros share. This is the
//! per-expansion cost the M1 milestone added to the macro surface; the
//! numbers land in `reports/08-benchmarks/BENCHMARKS.md` beside the
//! other gates when the bench environment runs them.
//!
//! ```sh
//! cargo bench -p aeolus-macro --bench baked
//! ```

use aeolus_compile::{Config, compile_theme};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The suite's own fixture (one file: `@theme` tokens plus a `.dark`
/// block — the web-v4 shape the `theme:` argument bakes).
const FIXTURE: &str = include_str!("../tests/fixtures/app.css");

/// The themed walk's class string.
const CLASSES: &str = "flex items-center gap-2 p-4 bg-primary text-primary-foreground rounded-lg";

/// The no-arg macros' embedded bake: shadcn light + baked table + the
/// themed walk — the expansion cost every `tw!` call site already paid.
#[divan::bench]
fn shadcn_bake_walk(bencher: Bencher) {
    bencher.bench(|| {
        let source = ThemeSource::shadcn().expect("embedded palette parses");
        let table = TokenTable::baked(&source).expect("baked table builds");
        let compiled =
            compile_theme(CLASSES, &table, &Config::default()).expect("class string lowers");
        compiled.diagnostics.is_empty()
    });
}

/// The `theme:` argument's bake: the host sheet's `from_css` (light +
/// its own `.dark` block) + the baked table + the themed walk.
#[divan::bench(name = "theme_arg_bake_walk")]
fn themed_bake_walk(bencher: Bencher) {
    bencher.bench(|| {
        let source = ThemeSource::from_app_css(FIXTURE).expect("the host sheet parses");
        let table = TokenTable::baked(&source).expect("baked table builds");
        let compiled =
            compile_theme(CLASSES, &table, &Config::default()).expect("class string lowers");
        compiled.diagnostics.is_empty()
    });
}
