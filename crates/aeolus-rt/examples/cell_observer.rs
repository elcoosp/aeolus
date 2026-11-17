//! Consumer demonstration: the view-state cell's observability face —
//! the bounded pool's own seam joined by the slow lane. A
//! host-supplied [`DynObserver`] wired through
//! [`ClassCell::with_observer`] sees every decision the cell makes
//! (miss, store, hit — the composed session key, the store event
//! carrying the entry's diagnostic count), in order; the built-in
//! [`CacheStats`] collector totals the session and ONE stats object
//! spans the `DynCache` pool AND every view cell the app wires it
//! into — the app-wide budget's shape.
//!
//! ```sh
//! cargo run -p aeolus-rt --example cell_observer
//! ```
//!
//! The cell stays the UI-thread surface (`!Sync` by design — each
//! view owns its slot); the handle is the shareable half. The
//! observation-free default (no observer) is the zero-cost shape the
//! differential pins: the observer adds a channel, it never changes a
//! decision.

use std::sync::Arc;

use aeolus_compile::{CacheStats, Config, DynCache, DynObserver};
use aeolus_rt::ClassCell;

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

    // 1. The built-in stats: one shareable handle, one view cell, a
    //    UI loop's steady state (one miss, then hits), beside the
    //    bounded pool the same handle watches — the totals span both.
    let stats = Arc::new(CacheStats::new());
    let pool = DynCache::new().with_observer(Arc::clone(&stats));
    let cell = ClassCell::new().with_observer(Arc::clone(&stats));
    println!("── 60 frames of one dynamic string, CacheStats watching ──");
    for _ in 0..60 {
        pool.compile("p-4 hover:bg-primary/90 rounded-md", &config)
            .expect("non-empty input compiles");
        cell.get("p-4 hover:bg-primary/90 rounded-md", &config)
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
    println!(
        "  (the pool's own counters: {} hits / {} misses — the slots stay \
         per-slot, the handle spans the app)",
        pool.hits(),
        pool.misses(),
    );

    // 2. One stats object across TWO view cells — each view owns its
    //    slot (the `!Sync` contract), the shared handle is the budget
    //    that spans them.
    let shared = Arc::new(CacheStats::new());
    let sidebar = ClassCell::new().with_observer(Arc::clone(&shared));
    let main_pane = ClassCell::new().with_observer(Arc::clone(&shared));
    sidebar
        .get("w-64 border-r", &config)
        .expect("non-empty input compiles");
    main_pane
        .get("flex-1 overflow-auto", &config)
        .expect("non-empty input compiles");
    println!("── one stats object across two view cells ──");
    println!("  shared snapshot: {shared:?}");

    // 3. The streaming observer on a mixed battery: the decisions
    //    print AS they happen, diagnostics count included — the honest
    //    per-class reports ride the store events, and the cell's own
    //    diagnostics channel answers beside them.
    let watched = ClassCell::new().with_observer(PrintObserver);
    let battery = [
        "p-4",
        "p-red not-a-utility", // carries typed diagnostics
        "hover:focus:md:p-0",
    ];
    println!("── two rounds of a mixed battery, decisions streamed ──");
    for _ in 0..2 {
        for input in battery {
            watched
                .get(input, &config)
                .expect("non-empty input compiles");
        }
    }
    println!(
        "  (the composed key prefixes each input with its identity — the \
         same vocabulary the DynStore seam probes; `aeolus_rt::session_key` \
         composes it)"
    );
}
