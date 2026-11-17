//! The `DynCache` observer seam's contracts (the observability face the
//! module docs promise beside the `tracing` events and the hit/miss
//! counters): a host-implemented [`DynObserver`] wires in through
//! [`DynCache::with_observer`] and sees every decision the cache makes,
//! in order, with the same key vocabulary the [`DynStore`] seam probes.
//!
//! The pins, in the order the module docs promise them:
//!
//! * the decisions stream IN ORDER — miss, then store, then hit for a
//!   repeated string — and the key carries the input verbatim (the
//!   composed identity key, never a hash);
//! * the built-in [`CacheStats`] collector answers the session: hit /
//!   miss / store totals, `lookups`, and a `hit_rate` that refuses to
//!   invent one for an empty session;
//! * one stats object spans MULTIPLE caches (the shared handle is the
//!   app-wide budget's shape) while each cache's own counters stay
//!   per-cache;
//! * errors (the empty input) miss and never store — the counters'
//!   existing contract, now observed;
//! * the store event carries the entry's diagnostic count (the honest
//!   per-class reports ride the cache);
//! * the default is observation-free: every constructor path installs
//!   [`NoObserver`], and the counters keep answering unchanged;
//! * the observer is `Send + Sync` like the cache: a shared stats
//!   object stays exact under concurrent compiles.

use std::sync::Arc;
use std::sync::Mutex;

use aeolus_compile::{
    CacheSnapshot, CacheStats, Config, DynCache, DynObserver, TwoGenStore, compile,
};
use aeolus_theme::{ThemeSource, TokenTable};

/// A host-implemented observer: records every decision as a readable
/// line (`hit <key>` / `miss <key>` / `store <key> <diagnostics>`) —
/// the seam contract is the subject, not the aggregation.
#[derive(Default)]
struct Recording {
    events: Mutex<Vec<String>>,
}

impl Recording {
    fn events(&self) -> Vec<String> {
        self.events.lock().expect("the test never poisons").clone()
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
}

/// The decision stream for one string, with the composed key stripped
/// to its INPUT tail (the identity prefix separates on `\u{1}`) — the
/// pin checks the ORDER and the identity, not the fingerprint bytes.
fn decisions(recording: &Arc<Recording>) -> Vec<String> {
    recording
        .events()
        .into_iter()
        .map(|line| {
            let (kind, rest) = line.split_once(' ').unwrap_or(("?", ""));
            // The store line carries the diagnostic count as its tail;
            // the key itself is everything before it (it may hold the
            // battery's own spaces).
            let (key, count) = if kind == "store" {
                rest.rsplit_once(' ').unwrap_or(("", "?"))
            } else {
                (rest, "?")
            };
            let input = key.rsplit('\u{1}').next().unwrap_or("");
            if kind == "store" {
                format!("{kind} {input} {count}")
            } else {
                format!("{kind} {input}")
            }
        })
        .collect()
}

#[test]
fn the_decisions_stream_in_order() {
    let recording = Arc::new(Recording::default());
    let cache = DynCache::new().with_observer(Arc::clone(&recording));
    let config = Config::default();

    cache.compile("p-4", &config).unwrap();
    cache.compile("p-4", &config).unwrap();

    let expected: Vec<String> = vec!["miss p-4".into(), "store p-4 0".into(), "hit p-4".into()];
    assert_eq!(
        decisions(&recording),
        expected,
        "a miss lowers and stores; the repeat hits the stored entry"
    );
}

#[test]
fn the_identity_is_part_of_the_observed_key() {
    let recording = Arc::new(Recording::default());
    let cache = DynCache::new().with_observer(Arc::clone(&recording));
    let config = Config::default();
    let source = ThemeSource::shadcn_dark().expect("embedded palette parses");
    let tokens = TokenTable::runtime(&source).expect("default-seeded source builds");

    cache.compile("p-4", &config).unwrap();
    cache.compile_theme("p-4", &tokens, &config).unwrap();

    let observed = recording.events();
    let misses: Vec<&String> = observed.iter().filter(|l| l.starts_with("miss")).collect();
    assert_eq!(misses.len(), 2, "one miss per identity slot");
    assert_ne!(
        misses[0], misses[1],
        "the theme-less and themed compiles of one string are DISTINCT slots — the keys differ"
    );
}

#[test]
fn the_built_in_stats_answer_the_session() {
    let stats = Arc::new(CacheStats::new());
    let cache = DynCache::new().with_observer(Arc::clone(&stats));
    let config = Config::default();

    for input in ["p-4", "px-8", "gap-2"] {
        cache.compile(input, &config).unwrap();
        cache.compile(input, &config).unwrap();
    }

    let snapshot: CacheSnapshot = stats.snapshot();
    assert_eq!(
        snapshot,
        CacheSnapshot {
            hits: 3,
            misses: 3,
            stores: 3,
        },
        "three distinct strings lowered once and hit once each"
    );
    assert_eq!(snapshot.lookups(), 6);
    assert_eq!(snapshot.hit_rate(), Some(0.5));

    let fresh = CacheStats::new();
    assert_eq!(
        fresh.snapshot().hit_rate(),
        None,
        "an empty session has no hit rate to invent"
    );
}

#[test]
fn one_stats_object_spans_two_caches() {
    let stats = Arc::new(CacheStats::new());
    let first = DynCache::new().with_observer(Arc::clone(&stats));
    let second = DynCache::new().with_observer(Arc::clone(&stats));
    let config = Config::default();

    first.compile("p-4", &config).unwrap();
    first.compile("px-8", &config).unwrap();
    second.compile("gap-2", &config).unwrap();

    let snapshot = stats.snapshot();
    assert_eq!(
        snapshot,
        CacheSnapshot {
            hits: 0,
            misses: 3,
            stores: 3,
        },
        "the shared handle totals the WHOLE app's decisions"
    );
    // The caches' own counters stay per-cache (the existing totals).
    assert_eq!(first.misses(), 2);
    assert_eq!(second.misses(), 1);
}

#[test]
fn errors_miss_but_never_store() {
    let recording = Arc::new(Recording::default());
    let cache = DynCache::new().with_observer(Arc::clone(&recording));
    let config = Config::default();

    assert!(cache.compile("", &config).is_err());
    assert!(cache.compile("   ", &config).is_err());

    let expected: Vec<String> = vec!["miss ".into(), "miss    ".into()];
    assert_eq!(
        decisions(&recording),
        expected,
        "the caller bug misses (the walk never runs) and never stores — \
         the key tail is the input verbatim, whitespace included"
    );
}

#[test]
fn the_store_event_carries_the_diagnostic_count() {
    let recording = Arc::new(Recording::default());
    let cache = DynCache::new().with_observer(Arc::clone(&recording));
    let config = Config::default();

    let free = compile("p-red not-a-utility", &config).unwrap();
    assert!(
        !free.diagnostics.is_empty(),
        "the battery string must carry diagnostics for the pin to mean anything"
    );
    cache.compile("p-red not-a-utility", &config).unwrap();

    let expected: Vec<String> = vec![
        "miss p-red not-a-utility".into(),
        format!("store p-red not-a-utility {}", free.diagnostics.len()),
    ];
    assert_eq!(
        decisions(&recording),
        expected,
        "the store event names the honest per-class report count"
    );
}

#[test]
fn the_defaults_stay_observation_free() {
    let config = Config::default();
    // Every constructor path: fresh, observation-free, counters intact.
    let plain = DynCache::new();
    plain.compile("p-4", &config).unwrap();
    assert_eq!(plain.hits(), 0);
    assert_eq!(plain.misses(), 1);

    let capped = DynCache::with_cap(4);
    capped.compile("p-4", &config).unwrap();
    assert_eq!(capped.misses(), 1);

    let stored = DynCache::with_store(TwoGenStore::with_cap(2))
        .with_observer(Arc::new(Recording::default()));
    stored.compile("p-4", &config).unwrap();
    stored.compile("p-4", &config).unwrap();
    assert_eq!(stored.hits(), 1, "the observer rides ANY store");
}

#[test]
fn the_shared_stats_stay_exact_under_concurrency() {
    let stats = Arc::new(CacheStats::new());
    let cache = Arc::new(DynCache::new().with_observer(Arc::clone(&stats)));
    let config = Config::default();
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let cache = Arc::clone(&cache);
            std::thread::spawn(move || {
                for round in 0..50u32 {
                    for n in 0..8u32 {
                        let input = format!("gap-{n} mt-{round}");
                        cache.compile(&input, &config).unwrap();
                    }
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("worker thread panics");
    }

    let snapshot = stats.snapshot();
    assert_eq!(
        snapshot.lookups(),
        cache.hits() + cache.misses(),
        "the observer and the counters agree on every decision, exactly"
    );
    assert_eq!(snapshot.hits, cache.hits());
    assert_eq!(snapshot.misses, cache.misses());
    assert_eq!(
        snapshot.hit_rate(),
        Some(cache.hits() as f64 / snapshot.lookups() as f64),
        "the shared stats' hit rate is the cache's own, computed exactly"
    );
}
