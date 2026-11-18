//! The runtime surface's observability face: the dyn-cache seam's
//! types are NAMEABLE from `aeolus_rt` — the host-facing crate the
//! backends re-export through — beside the view-state cell that joins
//! the same seam. A host consuming the runtime crate names the
//! observer trait, the every-constructor default, the built-in
//! collector and the composed-key vocabulary WITHOUT adding
//! `aeolus-compile` to its manifest.
//!
//! The pins:
//! * [`DynObserver`] is implementable and [`NoObserver`] constructible
//!   through the runtime surface alone (the wire-up a host writes);
//! * [`CacheStats`] / [`CacheSnapshot`] answer through the runtime
//!   surface, one shared handle spanning the bounded pool AND the view
//!   cells (the app-wide budget's shape, consumed the way a host
//!   consumes it — every import from `aeolus_rt`);
//! * the composed-key vocabulary ([`session_key`]) is nameable and is
//!   the key both surfaces report their decisions under.

use std::sync::Arc;
use std::sync::Mutex;

use aeolus_compile::Config;
use aeolus_rt::{
    CacheSnapshot, CacheStats, ClassCell, Customs, DynCache, DynObserver, NoObserver, ThemeSource,
    TokenTable, session_key,
};

/// A host-implemented observer through the runtime surface's own
/// trait: records the store decisions it sees.
struct StoreRecorder {
    events: Mutex<Vec<String>>,
}

impl DynObserver for StoreRecorder {
    fn on_store(&self, key: &str, diagnostics: usize) {
        self.events
            .lock()
            .expect("the test never poisons")
            .push(format!("store {key} {diagnostics}"));
    }
}

#[test]
fn the_observability_face_is_nameable_from_the_runtime_surface() {
    // The trait is implementable and the default constructible through
    // `aeolus_rt` alone — the wire-up a host writes. The default's
    // no-op bodies answer every event (the point: the default adds no
    // channel and costs nothing).
    let default = NoObserver;
    default.on_hit("p-4");
    default.on_miss("p-4");
    default.on_store("p-4", 0);
    let cell = ClassCell::new().with_observer(StoreRecorder {
        events: Mutex::new(Vec::new()),
    });
    let config = Config::default();
    cell.get("p-4 flex-col", &config).expect("compiles");
    assert_eq!(
        cell.hits(),
        0,
        "the first call misses (the store decision rode the wired \
         observer)"
    );
    assert_eq!(cell.misses(), 1);
}

#[test]
fn the_shared_handle_spans_pool_and_cells_through_the_runtime_surface() {
    let stats = Arc::new(CacheStats::new());
    let pool = DynCache::new().with_observer(Arc::clone(&stats));
    let cell = ClassCell::new().with_observer(Arc::clone(&stats));
    let config = Config::default();

    pool.compile("w-64", &config).expect("compiles");
    cell.get("w-64", &config).expect("compiles");
    cell.get("w-64", &config).expect("hits");

    let snapshot: CacheSnapshot = stats.snapshot();
    assert_eq!(
        snapshot,
        CacheSnapshot {
            hits: 1,
            misses: 2,
            stores: 2,
            evictions: 0,
        },
        "one handle, two surfaces, one budget — the runtime crate's own \
         names"
    );
    assert_eq!(snapshot.lookups(), 3);
    assert_eq!(snapshot.hit_rate(), Some(1.0 / 3.0));
}

#[test]
fn the_session_key_is_nameable_and_is_the_reported_vocabulary() {
    let config = Config::default();
    let classes = "p-4 flex-col";
    let composed = session_key(config, None, &Customs::new(), classes);

    // The composed key is the identity prefix (identity fields,
    // `\u{1}`-separated) followed by the input verbatim — never a
    // hash of either.
    let fields: Vec<&str> = composed.split('\u{1}').collect();
    assert_eq!(fields.len(), 4, "three identity fields + the input");
    assert_eq!(fields[3], classes, "the input rides verbatim");
    assert!(
        fields[0].chars().all(|c| c.is_ascii_hexdigit()),
        "the config fingerprint is one hex word"
    );

    // And it is the key both surfaces report: a themed session of the
    // same string composes a DIFFERENT key (the table's identity joins),
    // an empty bag keys as its absence (the custom seam's
    // byte-identity contract).
    let source = ThemeSource::shadcn().expect("embedded palette parses");
    let tokens = TokenTable::runtime(&source).expect("default-seeded source builds");
    let themed = session_key(config, Some(&tokens), &Customs::new(), classes);
    assert_ne!(
        composed, themed,
        "the theme-less and themed sessions of one string are DISTINCT \
         slots"
    );
    let plain_with_bag = session_key(config, None, &Customs::new(), classes);
    assert_eq!(
        composed, plain_with_bag,
        "the empty bag keys as zero — its byte-identity to the plain \
         entry points"
    );
}
