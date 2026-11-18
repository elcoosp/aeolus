//! The `DynCache` eviction face's contracts (the observability seam's
//! third decision beside hit/miss/store): the built-in [`TwoGenStore`]
//! drops resident entries when its generations turn — the whole
//! `previous` map at a rotation, a stale fingerprint twin a fresh store
//! clears — and a wired [`DynObserver`] now SEES what left, with the
//! same composed-key vocabulary the probe/store events ride and a typed
//! [`EvictReason`] naming why.
//!
//! The pins, in the order the module docs promise them:
//!
//! * the rotation reports WHAT it dropped — every key the discarded
//!   generation carried rides one `evict` event under
//!   [`EvictReason::Rotation`], through the cache's own
//!   [`DynCache::with_observer`] wire-up (the store joins the
//!   observation through the seam);
//! * the stale twin reports its clear — a fresh store of a key whose
//!   previous-generation twin still sits behind the rotation evicts the
//!   twin under [`EvictReason::StaleTwin`];
//! * a promotion is NOT an eviction — the entry moved between
//!   generations stays resident, and no event rides;
//! * the built-in [`CacheStats`] totals the evictions beside the
//!   hit/miss/store counters it already keeps;
//! * the replace-in-place arm is a replacement, not an eviction (the
//!   key stays resident; only the stale twin leaves);
//! * the observation-free default stays byte-identical: no observer,
//!   no events, no changed decisions, the two-generation resident
//!   bounds intact;
//! * the shared stats stay exact under concurrent rotation churn.

use std::sync::Arc;
use std::sync::Mutex;

use aeolus_compile::{
    CacheSnapshot, CacheStats, CachedCompiled, Config, DynCache, DynIdentity, DynObserver,
    DynStore, EvictReason, TwoGenStore, compile,
};

/// A host-implemented observer: records every decision as a readable
/// line (`hit <key>` / `evict <reason> <key>`) — the seam contract is
/// the subject, not the aggregation. The hit/miss/store lines the
/// neighbouring suite pins are recorded too, so a pin can assert the
/// whole stream's order.
#[derive(Default)]
struct Recording {
    events: Mutex<Vec<String>>,
}

impl Recording {
    fn events(&self) -> Vec<String> {
        self.events.lock().expect("the test never poisons").clone()
    }

    fn evictions(&self) -> Vec<(EvictReason, String)> {
        self.events()
            .into_iter()
            .filter_map(|line| {
                let rest = line.strip_prefix("evict ")?;
                let (reason, key) = rest.split_once(' ')?;
                let reason = match reason {
                    "rotation" => EvictReason::Rotation,
                    "stale-twin" => EvictReason::StaleTwin,
                    other => panic!("unknown reason tag {other:?}"),
                };
                Some((reason, key.to_string()))
            })
            .collect()
    }
}

impl DynObserver for Recording {
    fn on_hit(&self, key: &str) {
        self.events
            .lock()
            .expect("the test never poisons")
            .push(format!("hit {key}"));
    }

    fn on_miss(&self, key: &str) {
        self.events
            .lock()
            .expect("the test never poisons")
            .push(format!("miss {key}"));
    }

    fn on_store(&self, key: &str, diagnostics: usize) {
        self.events
            .lock()
            .expect("the test never poisons")
            .push(format!("store {key} {diagnostics}"));
    }

    fn on_evict(&self, key: &str, reason: EvictReason) {
        let tag = match reason {
            EvictReason::Rotation => "rotation",
            EvictReason::StaleTwin => "stale-twin",
        };
        self.events
            .lock()
            .expect("the test never poisons")
            .push(format!("evict {tag} {key}"));
    }
}

/// The observed key stripped to its INPUT tail (the identity prefix
/// separates on `\u{1}`) — the pins check what left, not the
/// fingerprint bytes.
fn input_tail(key: &str) -> &str {
    key.rsplit('\u{1}').next().unwrap_or(key)
}

#[test]
fn the_rotation_reports_what_it_dropped() {
    let recording = Arc::new(Recording::default());
    let cache =
        DynCache::with_store(TwoGenStore::with_cap(2)).with_observer(Arc::clone(&recording));
    let config = Config::default();

    for input in ["p-1", "p-2", "p-3", "p-4", "p-5", "p-6"] {
        cache.compile(input, &config).unwrap();
    }

    let events = recording.evictions();
    let mut evicted: Vec<&str> = events
        .iter()
        .map(|(reason, key)| {
            assert_eq!(
                *reason,
                EvictReason::Rotation,
                "every drop a fresh store causes here is the generation turning"
            );
            input_tail(key)
        })
        .collect();
    evicted.sort_unstable();
    assert_eq!(
        evicted,
        vec!["p-1", "p-2"],
        "cap-2 churn: the first rotation turns the first pair into previous, \
         the second drops it — the discarded generation rides the events"
    );
    assert_eq!(
        cache.resident(),
        4,
        "the resident set stays the two-generation bound: current + previous"
    );
}

#[test]
fn the_stale_twin_reports_its_clear() {
    let recording = Arc::new(Recording::default());
    let store = TwoGenStore::with_cap(2);
    store.attach_observer(recording.clone());

    // The twin's shape, driven through the raw store seam (the cache's
    // probe would PROMOTE a previous-generation hit before any store
    // saw it): a, b fill the first generation; c turns it; a's fresh
    // store clears the stale twin behind the rotation.
    DynStore::store(&store, "a".into(), probe_battery_entry());
    DynStore::store(&store, "b".into(), probe_battery_entry());
    DynStore::store(&store, "c".into(), probe_battery_entry());
    assert!(
        recording.evictions().is_empty(),
        "the first rotation drops an empty generation"
    );
    DynStore::store(&store, "a".into(), probe_battery_entry());

    let evictions = recording.evictions();
    assert_eq!(
        evictions,
        vec![(EvictReason::StaleTwin, "a".to_string())],
        "the fresh store of a cleared the stale twin the rotation left behind"
    );
    assert_eq!(
        store.resident(),
        3,
        "c and the fresh a are resident; b stays behind in previous"
    );
}

#[test]
fn a_promotion_is_not_an_eviction() {
    let recording = Arc::new(Recording::default());
    let cache =
        DynCache::with_store(TwoGenStore::with_cap(2)).with_observer(Arc::clone(&recording));
    let config = Config::default();

    cache.compile("p-1", &config).unwrap();
    cache.compile("p-2", &config).unwrap();
    cache.compile("p-3", &config).unwrap();
    // p-1 sits in previous now; its repeat PROMOTES it into current —
    // the entry moved, nothing left the store.
    cache.compile("p-1", &config).unwrap();

    let evictions = recording.evictions();
    assert!(
        evictions.is_empty(),
        "a promotion moves the entry between generations; no drop, no event"
    );
    assert_eq!(
        cache.resident(),
        3,
        "p-1 moved into current; p-2 stays previous"
    );
}

#[test]
fn the_built_in_stats_answer_the_evictions() {
    let stats = Arc::new(CacheStats::new());
    let cache = DynCache::with_store(TwoGenStore::with_cap(2)).with_observer(Arc::clone(&stats));
    let config = Config::default();

    for input in ["p-1", "p-2", "p-3", "p-4", "p-5", "p-6"] {
        cache.compile(input, &config).unwrap();
    }

    let snapshot: CacheSnapshot = stats.snapshot();
    assert_eq!(
        snapshot,
        CacheSnapshot {
            hits: 0,
            misses: 6,
            stores: 6,
            evictions: 2,
        },
        "six strings lowered once; one rotation dropped the first generation's two entries"
    );
    assert_eq!(snapshot.lookups(), 6);
}

#[test]
fn a_replace_in_place_is_not_an_eviction() {
    let recording = Arc::new(Recording::default());
    let store = TwoGenStore::with_cap(2);
    store.attach_observer(recording.clone());

    DynStore::store(&store, "a".into(), probe_battery_entry());
    DynStore::store(&store, "a".into(), probe_battery_entry());

    assert!(
        recording.evictions().is_empty(),
        "the config flip-flop replaces the current entry in place — the key never left"
    );
    assert_eq!(store.resident(), 1);
}

#[test]
fn the_observation_free_default_stays_byte_identical() {
    let config = Config::default();
    let battery = [
        "p-4 hover:bg-primary/90",
        "px-8 py-2 rounded-md",
        "hover:focus:md:p-0",
        "p-red not-a-utility",
        "w-1/2 -mt-2 text-sm",
        "p-4 hover:bg-primary/90",
    ];

    let quiet = DynCache::with_store(TwoGenStore::with_cap(2));
    let observed = DynCache::with_store(TwoGenStore::with_cap(2))
        .with_observer(Arc::new(Recording::default()));

    for input in battery {
        let quiet_answer = quiet.compile(input, &config).unwrap();
        let observed_answer = observed.compile(input, &config).unwrap();
        assert_eq!(
            quiet_answer, observed_answer,
            "the observer adds a channel; it never changes a decision"
        );
    }

    assert_eq!(
        quiet.hits(),
        observed.hits(),
        "the counters answer identically"
    );
    assert_eq!(quiet.misses(), observed.misses());
    assert_eq!(
        quiet.resident(),
        observed.resident(),
        "the bounds hold beside the stream"
    );
    assert_eq!(quiet.resident(), 4, "the two-generation bound: 2 x cap");
}

#[test]
fn the_shared_stats_stay_exact_under_rotation_churn() {
    /// The one-handle-two-shapes composite: the built-in totals AND the
    /// worst-case per-event log — the exactness pin's own subject.
    struct Tally {
        stats: Arc<CacheStats>,
        log: Mutex<Vec<String>>,
    }

    impl DynObserver for Tally {
        fn on_hit(&self, key: &str) {
            self.stats.on_hit(key);
        }

        fn on_miss(&self, key: &str) {
            self.stats.on_miss(key);
        }

        fn on_store(&self, key: &str, diagnostics: usize) {
            self.stats.on_store(key, diagnostics);
        }

        fn on_evict(&self, key: &str, reason: EvictReason) {
            self.stats.on_evict(key, reason);
            self.log
                .lock()
                .expect("the test never poisons")
                .push(format!("{reason:?} {key}"));
        }
    }

    let tally = Arc::new(Tally {
        stats: Arc::new(CacheStats::new()),
        log: Mutex::new(Vec::new()),
    });
    let cache =
        Arc::new(DynCache::with_store(TwoGenStore::with_cap(4)).with_observer(Arc::clone(&tally)));
    let config = Config::default();
    let handles: Vec<_> = (0..4)
        .map(|thread| {
            let cache = Arc::clone(&cache);
            std::thread::spawn(move || {
                for round in 0..12u32 {
                    let input = format!("gap-{thread}-{round}");
                    cache.compile(&input, &config).unwrap();
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("worker thread panics");
    }

    let snapshot = tally.stats.snapshot();
    assert_eq!(
        snapshot.evictions,
        tally.log.lock().expect("the test never poisons").len() as u64,
        "the built-in total and the raw event stream agree exactly under churn"
    );
    assert!(
        snapshot.evictions > 0,
        "48 distinct strings through a cap-4 store must rotate at least once"
    );
    assert_eq!(
        snapshot.lookups(),
        cache.hits() + cache.misses(),
        "the observer and the counters agree on every decision, exactly"
    );
}

/// A `CachedCompiled` the raw-store pins store: the battery's own
/// compiled shape under a zeroed fingerprint (the store never
/// interprets the identity — the rotation pins' own convention).
fn probe_battery_entry() -> CachedCompiled {
    let compiled = compile("p-4", &Config::default()).expect("non-empty input compiles");
    CachedCompiled {
        identity: DynIdentity {
            config: 0,
            tokens: 0,
            customs: 0,
        },
        program: compiled.program,
        diagnostics: compiled.diagnostics.into(),
    }
}
