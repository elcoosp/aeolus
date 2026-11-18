//! Consumer demonstration: the `DynCache` eviction face — the
//! observability seam's answer to "what did my cap cost me". The
//! built-in [`TwoGenStore`] drops resident entries when its generations
//! turn (the whole discarded generation at a rotation) and when a fresh
//! store clears a stale fingerprint twin; a wired [`DynObserver`] sees
//! every leave-taking under a typed [`EvictReason`] and the same
//! composed-key vocabulary the hit/miss/store stream rides.
//!
//! ```sh
//! cargo run -p aeolus-compile --example cache_eviction
//! ```
//!
//! The wire-up is the cache's own constructor: `DynCache::with_observer`
//! reaches the store through the [`DynStore::attach_observer`] seam, so
//! ONE stats handle spans the cache's decisions AND the store's — and
//! the callbacks fire after the store's write lock releases, so a host
//! observer can consult the cache it watches without deadlocking it.
//! The observation-free default stays byte-identical (the differential
//! pins hold): no observer, no events, no changed decisions.

use std::sync::Arc;

use aeolus_compile::{
    CacheStats, Config, DynCache, DynObserver, DynStore, EvictReason, TwoGenStore,
};

/// A host-implemented observer: prints each eviction as it happens —
/// the streaming shape a budget overlay or a leak hunter would use.
/// The other channels (hit/miss/store) stay quiet here; an implementor
/// opts into the channels it wants.
struct EvictionWatch;

impl DynObserver for EvictionWatch {
    fn on_evict(&self, key: &str, reason: EvictReason) {
        let input = key.rsplit('\u{1}').next().unwrap_or(key);
        match reason {
            EvictReason::Rotation => println!("    evict (rotation)      {input}"),
            EvictReason::StaleTwin => println!("    evict (stale twin)    {input}"),
            EvictReason::Invalidated => println!("    evict (invalidated)   {input}"),
        }
    }
}

fn main() {
    let config = Config::default();

    // 1. The cap's pressure: a small-cap cache churning through distinct
    //    strings. Each rotation drops the generation one step back —
    //    the events name what left, in decision order.
    let stats = Arc::new(CacheStats::new());
    let cache = DynCache::with_store(TwoGenStore::with_cap(4)).with_observer(Arc::clone(&stats));
    println!("── 10 distinct strings through a cap-4 cache, EvictionWatch watching ──");
    for n in 0..10u32 {
        let input = format!("p-{n} gap-{n} rounded-md");
        cache
            .compile(&input, &config)
            .expect("non-empty input compiles");
    }
    let snapshot = stats.snapshot();
    println!(
        "  snapshot: {} hits / {} misses / {} stores / {} evictions, resident {} (2 x cap = 8)",
        snapshot.hits,
        snapshot.misses,
        snapshot.stores,
        snapshot.evictions,
        cache.resident(),
    );

    // 2. A promotion is a MOVE, not an eviction: a working set that
    //    recurs within one generation gap stays resident — and the
    //    event stream stays quiet about it. One FIXED hot string probed
    //    every round keeps promoting (each repeat lands in current
    //    before the next rotation); only the churn's rotations print.
    let promoted = DynCache::with_store(TwoGenStore::with_cap(2)).with_observer(EvictionWatch);
    println!("── one recurring string beside churn: promotions stay quiet ──");
    for round in 0..3u32 {
        promoted
            .compile("w-hot", &config)
            .expect("non-empty input compiles");
        promoted
            .compile(&format!("churn-{round}-a"), &config)
            .expect("non-empty input compiles");
        promoted
            .compile("w-hot", &config)
            .expect("the hot string recurs");
        promoted
            .compile(&format!("churn-{round}-b"), &config)
            .expect("non-empty input compiles");
    }
    println!(
        "  (the recurring `w-hot` never printed — its repeats are promotions; \
         only the churn's rotations leave)"
    );

    // 3. The raw store seam: a host using TwoGenStore directly wires the
    //    same face through `attach_observer` — the seam's own
    //    composition, the trait naming the contract, the built-in store
    //    a consumer of it.
    let store = TwoGenStore::with_cap(2);
    store.attach_observer(Arc::new(EvictionWatch));
    println!("── the raw store seam: stale-twin clears, the flip-flop's cleanup ──");
    for key in ["a", "b", "c"] {
        DynStore::store(&store, key.to_string(), store_entry(key, config));
    }
    DynStore::store(&store, "a".to_string(), store_entry("a", config));
    println!("  (the fresh store of `a` cleared the stale twin the rotation left behind)");
}

/// One real entry for the raw-store demo: the pipeline's own compiled
/// shape under a zeroed fingerprint (the store never interprets it).
fn store_entry(key: &str, config: Config) -> aeolus_compile::CachedCompiled {
    let compiled = aeolus_compile::compile(key, &config).expect("non-empty input compiles");
    aeolus_compile::CachedCompiled {
        identity: aeolus_compile::DynIdentity {
            config: 0,
            tokens: 0,
            customs: 0,
        },
        program: compiled.program,
        diagnostics: compiled.diagnostics.into(),
    }
}
