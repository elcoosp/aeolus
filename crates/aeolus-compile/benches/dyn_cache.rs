//! The dyn-cache microbenchmarks (divan, stable — the workspace bench
//! idiom). The plan §3.2 `DynCache` slice's cold and hot paths, against
//! the uncached free functions they must beat: the miss pays the probe
//! AND the pipeline, the hit pays one map probe and hands back the same
//! interned slices — the v0.x `resolve_cached_hit` story (405 ns
//! against the 4.6 µs cold resolve, `reports/08-benchmarks`) transplanted
//! to the V1 seam's interned-`Program` model.
//!
//! The bench bodies `expect` on the compile: a non-empty class string
//! always produces a `Compiled` (only the empty input hard-fails).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (see `reports/10-tdd-evidence/16-dyn-cache.md`
//! for the red-to-green record).

use aeolus_compile::{Config, DynCache, compile};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The shadcn v4 Button table verbatim (base + default variant + default
/// size) — the same 33-class string the v0.x hot-path benchmarks and the
/// compile-seam benches key on.
const BUTTON_CLASSES: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2 has-[>svg]:px-3";

/// The uncached floor: the plain free function (what the cache must
/// beat on the second and every later call).
#[divan::bench]
fn compile_uncached_button(bencher: Bencher) {
    bencher
        .bench(|| compile(BUTTON_CLASSES, &Config::default()).expect("non-empty input compiles"));
}

/// The cache's cold path: a fresh cache per call — the miss pays the
/// probe, the pipeline AND the store (the honest first-call cost).
#[divan::bench]
fn dyn_cache_miss_cold_button(bencher: Bencher) {
    bencher.bench(|| {
        let cache = DynCache::new();
        cache
            .compile(BUTTON_CLASSES, &Config::default())
            .expect("non-empty input compiles")
    });
}

/// The cache's hot path: one warm cache, the SAME string every call —
/// one composed-key probe, the same interned slices handed back, no
/// pipeline pass.
#[divan::bench]
fn dyn_cache_hit_warm_button(bencher: Bencher) {
    let cache = DynCache::new();
    cache
        .compile(BUTTON_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    bencher.bench(|| {
        cache
            .compile(BUTTON_CLASSES, &Config::default())
            .expect("non-empty input compiles")
    });
}

/// The single-utility hot path — the floor the hit must stay close to
/// (one small probe against one small lowering).
#[divan::bench]
fn dyn_cache_hit_warm_single_utility(bencher: Bencher) {
    let cache = DynCache::new();
    cache
        .compile("p-4", &Config::default())
        .expect("non-empty input compiles");
    bencher.bench(|| {
        cache
            .compile("p-4", &Config::default())
            .expect("non-empty input compiles")
    });
}

/// The single-utility uncached floor, for the same-shape ratio.
#[divan::bench]
fn compile_uncached_single_utility(bencher: Bencher) {
    bencher.bench(|| compile("p-4", &Config::default()).expect("non-empty input compiles"));
}

/// The warm themed session: the table binds once, the token-lowering
/// class string repeats — the hit skips the theme resolve too.
#[divan::bench]
fn dyn_cache_hit_warm_themed(bencher: Bencher) {
    let source = aeolus_theme::ThemeSource::shadcn_dark().expect("embedded palette parses");
    let tokens = aeolus_theme::TokenTable::runtime(&source).expect("table builds");
    let cache = DynCache::new();
    cache
        .compile_theme("bg-primary/90", &tokens, &Config::default())
        .expect("non-empty input compiles");
    bencher.bench(|| {
        cache
            .compile_theme("bg-primary/90", &tokens, &Config::default())
            .expect("non-empty input compiles")
    });
}
