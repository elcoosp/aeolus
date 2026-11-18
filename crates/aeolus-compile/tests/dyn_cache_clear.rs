// ---------------------------------------------------------------------------
// The theme lifecycle's invalidation seam (the reload handoff): the
// store's wholesale-leave channel — every entry leaves under ONE typed
// reason, the decisions keep flowing afterwards, and the cache-level
// clear keeps the answers byte-identical (a cleared entry is a miss,
// never a wrong answer).
//
// Written FIRST (the RED spec).
// ---------------------------------------------------------------------------

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;

use aeolus_compile::{
    CachedCompiled, Config, DynCache, DynIdentity, DynObserver, DynStore, EvictReason, TwoGenStore,
    compile,
};

/// The battery: the shapes the invalidation must carry (a lowered
/// class, a variant stack, a diagnostic-bearing miss).
const BATTERY: &[&str] = &["p-4", "px-8 py-2 rounded-md", "hover:focus:md:p-0"];

/// One real entry per key: the compiles come from the cache's own
/// pipeline (the store carries interned programs, never synthetic
/// ones — the rotation pins' own seeding shape).
fn entries(count: usize) -> Vec<(String, CachedCompiled)> {
    let config = Config::default();
    BATTERY
        .iter()
        .cycle()
        .take(count)
        .enumerate()
        .map(|(i, class)| {
            let compiled = compile(&format!("{class} x{i}"), &config).unwrap();
            // The store never validates identities — the invalidation
            // pins carry real programs under a zeroed fingerprint.
            let entry = CachedCompiled {
                identity: DynIdentity {
                    config: 0,
                    tokens: 0,
                    customs: 0,
                },
                program: compiled.program,
                diagnostics: compiled.diagnostics.into(),
            };
            (format!("k{i}"), entry)
        })
        .collect()
}

/// The eviction recorder: the host observer the typed-reason pins ride.
struct Recorder {
    evictions: Arc<Mutex<Vec<(String, &'static str)>>>,
}

impl DynObserver for Recorder {
    fn on_evict(&self, key: &str, reason: EvictReason) {
        let word = match reason {
            EvictReason::Rotation => "rotation",
            EvictReason::StaleTwin => "stale-twin",
            EvictReason::Invalidated => "invalidated",
        };
        self.evictions
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((key.to_string(), word));
    }
}

#[test]
fn invalidate_all_empties_the_store_and_the_decisions_keep_flowing() {
    let store = TwoGenStore::with_cap(2);
    for (key, entry) in entries(3) {
        store.store(key, entry);
    }
    assert_eq!(store.resident(), 3, "the fixture starts seeded");

    store.invalidate_all();

    assert_eq!(
        store.resident(),
        0,
        "the wholesale leave empties both generations"
    );
    assert!(
        store.probe("k0").is_none() && store.probe("k2").is_none(),
        "no key answers after the invalidation"
    );

    // The store keeps working: the next store probes back like any
    // fresh store would.
    store.store(
        "fresh".to_string(),
        entries(1).into_iter().next().unwrap().1,
    );
    assert!(
        store.probe("fresh").is_some(),
        "the store answers after the clear"
    );
}

#[test]
fn invalidate_all_reports_the_typed_invalidations() {
    let store = TwoGenStore::with_cap(4);
    let recorder = Arc::new(Mutex::new(Vec::new()));
    store.attach_observer(Arc::new(Recorder {
        evictions: Arc::clone(&recorder),
    }) as Arc<dyn DynObserver>);
    for (key, entry) in entries(2) {
        store.store(key, entry);
    }

    store.invalidate_all();

    let mut names = recorder
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    names.sort();
    assert_eq!(
        names,
        vec![
            ("k0".to_string(), "invalidated"),
            ("k1".to_string(), "invalidated"),
        ],
        "every leave rides the typed Invalidated reason, each key once"
    );
}

#[test]
fn the_cache_clear_empties_the_pool_and_the_answers_stay() {
    let cache = DynCache::new();
    let config = Config::default();
    let first = cache.compile("p-4 flex-col", &config).unwrap();
    let warm = cache.compile("p-4 flex-col", &config).unwrap();
    assert_eq!(cache.hits(), 1, "the fixture starts warm");

    cache.clear();

    assert_eq!(cache.resident(), 0, "the pool emptied");
    let after = cache.compile("p-4 flex-col", &config).unwrap();
    // The cleared entry re-lowers (a miss), and the answer is the SAME
    // bytes the pooled one carried — clear changes the cost, never a
    // decision.
    assert_eq!(first, after);
    assert_eq!(first, warm);
    assert_eq!(cache.misses(), 2, "the re-lowering is the new miss");
    assert_eq!(
        cache.hits(),
        1,
        "the warm hit before the clear still totals"
    );
    let repeat = cache.compile("p-4 flex-col", &config).unwrap();
    assert_eq!(repeat, first);
    assert_eq!(cache.hits(), 2, "the re-lowered entry pools again");
}
