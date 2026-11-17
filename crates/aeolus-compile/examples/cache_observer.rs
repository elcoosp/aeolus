//! Consumer demonstration: the `DynCache` observer seam — the
//! observability face beside the `tracing` events and the hit/miss
//! counters. A host-supplied [`DynObserver`] sees every decision the
//! cache makes (miss, store, hit — the key the store itself probes,
//! the store event carrying the entry's diagnostic count), in order;
//! the built-in [`CacheStats`] collector totals the session and one
//! stats object can span every cache the app wires it into.
//!
//! ```sh
//! cargo run -p aeolus-compile --example cache_observer
//! ```
//!
//! The three channels compose, they do not replace each other: the
//! counters answer HOW OFTEN, the observer answers WHAT (per-decision
//! keys, streamable), `tracing` answers WHEN (spans, under its own
//! feature) — and the observation-free default (no observer) is the
//! zero-cost shape the differential pins.

use std::sync::Arc;

use aeolus_compile::{CacheStats, Config, DynCache, DynObserver};

/// A host-implemented observer: prints each decision as it happens —
/// the streaming shape a debug overlay or a test driver would use.
struct PrintObserver;

impl DynObserver for PrintObserver {
    fn on_hit(&self, key: &str) {
        println!("    hit   {key}");
    }

    fn on_miss(&self, key: &str) {
        println!("    miss  {key}");
    }

    fn on_store(&self, key: &str, diagnostics: usize) {
        println!("    store {key} ({diagnostics} diagnostics)");
    }
}

fn main() {
    let config = Config::default();

    // 1. The built-in stats: one shareable handle, one cache, a UI
    //    loop's steady state (one miss, then hits).
    let stats = Arc::new(CacheStats::new());
    let cache = DynCache::new().with_observer(Arc::clone(&stats));
    println!("── 60 frames of one dynamic string, CacheStats watching ──");
    for _ in 0..60 {
        cache
            .compile("p-4 hover:bg-primary/90 rounded-md", &config)
            .expect("non-empty input compiles");
    }
    let snapshot = stats.snapshot();
    println!(
        "  snapshot: {} hits / {} misses / {} stores, hit rate {}",
        snapshot.hits,
        snapshot.misses,
        snapshot.stores,
        snapshot
            .hit_rate()
            .map(|rate| format!("{rate:.3}"))
            .unwrap_or_else(|| "n/a".into()),
    );

    // 2. One stats object across TWO caches — the app-wide budget's
    //    shape: the totals span both, each cache's counters stay its
    //    own.
    let shared = Arc::new(CacheStats::new());
    let sidebar = DynCache::new().with_observer(Arc::clone(&shared));
    let main_pane = DynCache::new().with_observer(Arc::clone(&shared));
    sidebar
        .compile("w-64 border-r", &config)
        .expect("non-empty input compiles");
    main_pane
        .compile("flex-1 overflow-auto", &config)
        .expect("non-empty input compiles");
    println!("── one stats object across two caches ──");
    println!("  shared snapshot: {shared:?}");

    // 3. The streaming observer on a mixed battery: the decisions
    //    print AS they happen, diagnostics count included — the honest
    //    per-class reports ride the store events.
    let watched = DynCache::new().with_observer(PrintObserver);
    let battery = [
        "p-4",
        "p-red not-a-utility", // carries typed diagnostics
        "hover:focus:md:p-0",
    ];
    println!("── two rounds of a mixed battery, decisions streamed ──");
    for _ in 0..2 {
        for input in battery {
            watched
                .compile(input, &config)
                .expect("non-empty input compiles");
        }
    }
    println!("  (the composed key prefixes each input with its identity —");
    println!("   the same vocabulary the DynStore seam probes)");
}
