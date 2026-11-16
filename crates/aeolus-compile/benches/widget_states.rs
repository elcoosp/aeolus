//! The widget-state family's benchmarks (the tier beside the
//! element-state family): the interp walk over a widget-state-mixed
//! program — the fact-bearing env (the fire path: `open:`, `inert:` and
//! `default:` answer from their one affirmative fact each) against the
//! default env (the resting path: all three stay the quiet miss a
//! selector that does not match is).

use aeolus_compile::{Config, DynCache, Env, interp};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The Button string's widget-state half — the layers the widget states
/// answer in the pipeline's real consumer (`open:` beside its
/// `inert:`/`default:` siblings; the flat fold keeps one condition per
/// layer, the mix pins the whole tier's walk).
const WIDGET_STATE_CLASSES: &str = "open:p-8 inert:hidden default:ring-2";

#[divan::bench]
fn interp_widget_state_button(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(WIDGET_STATE_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let env = Env {
        opened: true,
        inert: true,
        form_default: true,
        ..Env::default()
    };
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}

/// The same program under the default env: the `open:`, `inert:` and
/// `default:` layers are all the quiet miss a selector that does not
/// match is (the resting env asserts none of the three facts).
#[divan::bench]
fn interp_widget_state_default_env(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(WIDGET_STATE_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let env = Env::default();
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}
