//! The view-state cell's observability face (the slow lane's answers
//! beside the per-cell counters): [`ClassCell::with_observer`] wires a
//! host-implemented [`DynObserver`] — the SAME seam the bounded pool
//! (`DynCache::with_observer`) carries — and the cell's decisions ride
//! it in order: miss, then store, then hit for a repeated string,
//! under the composed key the [`DynStore`] vocabulary already probes.
//!
//! The pins, in the order the module docs promise them:
//!
//! * the decisions stream IN ORDER — the key carries the input
//!   verbatim after the identity prefix (the composed session key,
//!   never a hash), and the identity is part of it (a themed session
//!   of one string is a DIFFERENT slot from the theme-less one);
//! * the built-in [`CacheStats`] collector answers through the cell:
//!   hit / miss / store totals and a `hit_rate` that refuses to invent
//!   one for an empty session;
//! * ONE stats object spans the bounded pool AND the view cells (the
//!   app-wide budget's shape) while each slot's own counters stay
//!   per-slot;
//! * the cell and the cache report ONE key vocabulary — the same
//!   session and input compose the same key on both surfaces (the
//!   public [`session_key`] composer's own product);
//! * errors (the empty input) miss and never store, and the last good
//!   program still stands — the cell's degradation contract, observed;
//! * the store event carries the entry's diagnostic count (the honest
//!   per-class reports ride the cell beside its own channel);
//! * the default is observation-free: every constructor path installs
//!   [`NoObserver`], every entry point observes when one is wired, and
//!   the observer never changes a decision (the counters match the
//!   observation-free run exactly);
//! * the shared stats stay exact across threads: cells are per-thread
//!   (the `!Sync` contract), the handle spans them all.

use std::sync::Arc;
use std::sync::Mutex;

use aeolus_compile::{CacheSnapshot, CacheStats, Config, DynCache, DynObserver, session_key};
use aeolus_rt::{ClassCell, ThemeSource, TokenTable};

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

/// The decision stream for one cell, with the composed key stripped to
/// its INPUT tail (the identity prefix separates on `\u{1}`) — the pin
/// checks the ORDER and the identity, not the fingerprint bytes.
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
    let cell = ClassCell::new().with_observer(Arc::clone(&recording));
    let config = Config::default();

    cell.get("p-4 flex-col", &config).expect("compiles");
    cell.get("p-4 flex-col", &config).expect("hits");

    let expected: Vec<String> = vec![
        "miss p-4 flex-col".into(),
        "store p-4 flex-col 0".into(),
        "hit p-4 flex-col".into(),
    ];
    assert_eq!(
        decisions(&recording),
        expected,
        "a miss lowers and stores; the repeated frame hits the slot"
    );
}

#[test]
fn the_identity_is_part_of_the_observed_key() {
    let recording = Arc::new(Recording::default());
    let cell = ClassCell::new().with_observer(Arc::clone(&recording));
    let config = Config::default();
    let source = ThemeSource::shadcn().expect("embedded palette parses");
    let tokens = TokenTable::runtime(&source).expect("default-seeded source builds");

    cell.get("bg-primary p-4", &config)
        .expect("lowers what it can");
    cell.get_theme("bg-primary p-4", &tokens, &config)
        .expect("themed compile");

    let observed = recording.events();
    let misses: Vec<&String> = observed.iter().filter(|l| l.starts_with("miss")).collect();
    assert_eq!(misses.len(), 2, "one miss per identity slot");
    assert_ne!(
        misses[0], misses[1],
        "the theme-less and themed sessions of one string are DISTINCT \
         slots — the keys differ"
    );
}

#[test]
fn the_built_in_stats_answer_the_session() {
    let stats = Arc::new(CacheStats::new());
    let cell = ClassCell::new().with_observer(Arc::clone(&stats));
    let config = Config::default();

    for input in ["p-4", "px-8", "gap-2"] {
        cell.get(input, &config).expect("compiles");
        cell.get(input, &config).expect("hits");
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
fn one_stats_object_spans_the_pool_and_the_cells() {
    let stats = Arc::new(CacheStats::new());
    let pool = DynCache::new().with_observer(Arc::clone(&stats));
    let sidebar = ClassCell::new().with_observer(Arc::clone(&stats));
    let main_pane = ClassCell::new().with_observer(Arc::clone(&stats));
    let config = Config::default();

    pool.compile("w-64 border-r", &config).expect("compiles");
    pool.compile("w-64 border-r", &config).expect("hits");
    sidebar
        .get("flex-1 overflow-auto", &config)
        .expect("compiles");
    sidebar.get("flex-1 overflow-auto", &config).expect("hits");
    main_pane.get("gap-2", &config).expect("compiles");

    let snapshot = stats.snapshot();
    assert_eq!(
        snapshot,
        CacheSnapshot {
            hits: 2,
            misses: 3,
            stores: 3,
        },
        "the shared handle totals the WHOLE app's decisions — the \
         bounded pool beside the view cells"
    );
    // Each slot's own counters stay its own (the existing totals).
    assert_eq!(pool.hits(), 1);
    assert_eq!(pool.misses(), 1);
    assert_eq!(sidebar.hits(), 1);
    assert_eq!(sidebar.misses(), 1);
    assert_eq!(main_pane.misses(), 1);
}

#[test]
fn the_cell_and_the_cache_report_one_key_vocabulary() {
    let recording = Arc::new(Recording::default());
    let cell = ClassCell::new().with_observer(Arc::clone(&recording));
    let pool = DynCache::new().with_observer(Arc::clone(&recording));
    let config = Config::default();
    let classes = "p-4 flex-col";

    cell.get(classes, &config).expect("compiles");
    pool.compile(classes, &config).expect("compiles");

    let observed = recording.events();
    let keys: Vec<&str> = observed
        .iter()
        .filter_map(|l| l.strip_prefix("miss "))
        .collect();
    assert_eq!(keys.len(), 2, "one miss observed per surface");
    assert_eq!(
        keys[0], keys[1],
        "the same session and input compose the SAME key on both \
         surfaces — the cell joins the pool's vocabulary"
    );
    // And the key is the public composer's own product: the
    // theme-less, custom-less session, input verbatim after the
    // identity prefix.
    let composed = session_key(config, None, &aeolus_compile::Customs::new(), classes);
    assert_eq!(keys[0], &composed, "the observed key is the composed one");
}

#[test]
fn errors_miss_but_never_store() {
    let recording = Arc::new(Recording::default());
    let cell = ClassCell::new().with_observer(Arc::clone(&recording));
    let config = Config::default();

    let good = cell.get("p-4 flex-col", &config).expect("compiles");
    assert!(
        cell.get("", &config).is_err(),
        "the empty input is a caller bug"
    );
    assert!(cell.get("   ", &config).is_err());

    let expected: Vec<String> = vec![
        "miss p-4 flex-col".into(),
        "store p-4 flex-col 0".into(),
        "miss ".into(),
        "miss    ".into(),
    ];
    assert_eq!(
        decisions(&recording),
        expected,
        "the caller bug misses (the walk never runs) and never stores — \
         the key tail is the input verbatim, whitespace included"
    );
    // And the degradation contract holds under observation: the last
    // good program stands, the slot untouched by the failed calls.
    let standing = cell.program();
    assert_eq!(
        standing.layers.len(),
        good.layers.len(),
        "the last good program is still the render answer"
    );
}

#[test]
fn the_store_event_carries_the_diagnostic_count() {
    let recording = Arc::new(Recording::default());
    let cell = ClassCell::new().with_observer(Arc::clone(&recording));
    let config = Config::default();

    cell.get("p-4 not-a-real-utility-xyz", &config)
        .expect("the string still lowers what it can");
    assert_eq!(
        cell.diagnostics().len(),
        1,
        "the battery string carries one typed miss for the pin to mean \
         anything"
    );

    let expected: Vec<String> = vec![
        "miss p-4 not-a-real-utility-xyz".into(),
        "store p-4 not-a-real-utility-xyz 1".into(),
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
    let plain = ClassCell::new();
    plain.get("p-4", &config).expect("compiles");
    assert_eq!(plain.hits(), 0);
    assert_eq!(plain.misses(), 1);

    let defaulted = ClassCell::default();
    defaulted.get("p-4", &config).expect("compiles");
    assert_eq!(defaulted.misses(), 1);

    // The observer rides EVERY entry point: the four compile surfaces
    // all stream their decisions when one is wired.
    let recording = Arc::new(Recording::default());
    let source = ThemeSource::shadcn().expect("embedded palette parses");
    let tokens = TokenTable::runtime(&source).expect("default-seeded source builds");
    let customs = aeolus_compile::Customs::new();
    let watched = ClassCell::new().with_observer(Arc::clone(&recording));
    watched.get("p-4", &config).expect("compiles");
    watched
        .get_theme("bg-primary", &tokens, &config)
        .expect("themed compile");
    watched
        .get_with("p-4", &customs, &config)
        .expect("customized compile");
    watched
        .get_theme_with("bg-primary", &tokens, &customs, &config)
        .expect("full-session compile");
    let stores = recording
        .events()
        .iter()
        .filter(|l| l.starts_with("store"))
        .count();
    assert_eq!(stores, 4, "every entry point's store decision streams");

    // And the observer never changes a decision: the same script on an
    // observation-free cell leaves the SAME counters.
    let bare = ClassCell::new();
    bare.get("p-4", &config).expect("compiles");
    bare.get_theme("bg-primary", &tokens, &config)
        .expect("themed compile");
    bare.get_with("p-4", &customs, &config)
        .expect("customized compile");
    bare.get_theme_with("bg-primary", &tokens, &customs, &config)
        .expect("full-session compile");
    assert_eq!(watched.hits(), bare.hits(), "the hits match exactly");
    assert_eq!(watched.misses(), bare.misses(), "the misses match exactly");
}

#[test]
fn the_shared_stats_stay_exact_across_threads() {
    let stats = Arc::new(CacheStats::new());
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let stats = Arc::clone(&stats);
            std::thread::spawn(move || {
                // Each thread owns its cell (the `!Sync` AND `!Send`
                // contract — the UI-thread surface never crosses a
                // thread boundary); the SHARED handle spans them all.
                let cell = ClassCell::new().with_observer(stats);
                for round in 0..50u32 {
                    for n in 0..8u32 {
                        let input = format!("gap-{n} mt-{round}");
                        cell.get(&input, &Config::default()).expect("compiles");
                    }
                }
                (cell.hits(), cell.misses())
            })
        })
        .collect();
    let mut per_cell_hits = 0u64;
    let mut per_cell_misses = 0u64;
    for handle in handles {
        let (hits, misses) = handle.join().expect("worker thread panics");
        per_cell_hits += hits;
        per_cell_misses += misses;
    }

    let snapshot = stats.snapshot();
    assert_eq!(
        snapshot.lookups(),
        per_cell_hits + per_cell_misses,
        "the observer and the per-cell counters agree on every decision, \
         exactly"
    );
    assert_eq!(snapshot.hits, per_cell_hits);
    assert_eq!(snapshot.misses, per_cell_misses);
    assert_eq!(
        snapshot.hit_rate(),
        Some(per_cell_hits as f64 / snapshot.lookups() as f64),
        "the shared stats' hit rate is the cells' own, computed exactly"
    );
}
