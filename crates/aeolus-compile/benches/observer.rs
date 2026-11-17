//! The observer seam's microbenchmarks (divan, stable — the workspace
//! bench idiom): the seam's cost ON the hot paths it rides. The hit
//! path is the cache's own hot loop (one composed-key probe), so the
//! benches pin the observer's overhead against the observation-free
//! cache — the no-op default ([`NoObserver`]), the built-in counter
//! ([`CacheStats`]) and the WORST case (a recording observer that
//! formats and pushes a line per decision) — plus the snapshot read
//! the host's budget loop would run beside the hits.
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (see `reports/10-tdd-evidence/16-dyn-cache.md`
//! for the dyn-cache slice's own red-to-green record).

use std::sync::Arc;
use std::sync::Mutex;

use aeolus_compile::{CacheStats, Config, DynCache, DynObserver};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The shadcn v4 Button table verbatim (base + default variant + default
/// size) — the same string the dyn-cache benches key on.
const BUTTON_CLASSES: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2 has-[>svg]:px-3";

/// The observation-free baseline: the constructor paths' default on the
/// HOT path — one warm cache, the composed-key probe per call.
#[divan::bench]
fn hit_no_observer(bencher: Bencher) {
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

/// The built-in counter on the hot path: one virtual call and one
/// relaxed atomic add per hit.
#[divan::bench]
fn hit_stats_observer(bencher: Bencher) {
    let cache = DynCache::new().with_observer(Arc::new(CacheStats::new()));
    cache
        .compile(BUTTON_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    bencher.bench(|| {
        cache
            .compile(BUTTON_CLASSES, &Config::default())
            .expect("non-empty input compiles")
    });
}

/// The worst case a host can wire: a recording observer that formats a
/// line and appends to a mutex-guarded vector per decision.
struct Recording(Mutex<Vec<String>>);

impl DynObserver for Recording {
    fn on_hit(&self, key: &str) {
        self.0
            .lock()
            .expect("bench never poisons")
            .push(format!("hit {key}"));
    }

    fn on_miss(&self, key: &str) {
        self.0
            .lock()
            .expect("bench never poisons")
            .push(format!("miss {key}"));
    }

    fn on_store(&self, key: &str, diagnostics: usize) {
        self.0
            .lock()
            .expect("bench never poisons")
            .push(format!("store {key} {diagnostics}"));
    }
}

#[divan::bench]
fn hit_recording_observer(bencher: Bencher) {
    let cache = DynCache::new().with_observer(Arc::new(Recording(Mutex::new(Vec::new()))));
    cache
        .compile(BUTTON_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    bencher.bench(|| {
        cache
            .compile(BUTTON_CLASSES, &Config::default())
            .expect("non-empty input compiles")
    });
}

/// The snapshot read: three relaxed loads — the read the host's budget
/// loop or test assert runs beside the hits.
#[divan::bench]
fn cache_stats_snapshot(bencher: Bencher) {
    let stats = Arc::new(CacheStats::new());
    bencher.bench(|| stats.snapshot());
}
