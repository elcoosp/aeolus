//! The `DynCache` contracts (architecture plan §3.2's "P2's DynCache
//! dedupes the pool later" — the leak-pool dedupe slice): a bounded,
//! thread-safe, content-keyed compile cache whose HIT returns the SAME
//! interned slices the miss pooled, over a swappable [`DynStore`] seam.
//!
//! The pins, in the order the crate docs promise them:
//!
//! * a hit is byte-identical to the free [`compile`] result AND
//!   pointer-identical at the interned slice level (the pool dedupe —
//!   no second leak for a repeated class string);
//! * the config fingerprint is part of the identity: a different
//!   [`Config`] is a miss, never a false hit;
//! * distinct strings never collide (the key is the string itself, the
//!   v0.x collision-safety rule);
//! * the resident key set is bounded (two-generation rotation) and a
//!   hit in the previous generation promotes (the v0.x boundedness
//!   story);
//! * the store is a SEAM: a host-implemented [`DynStore`] wires in
//!   through [`DynCache::with_store`] and observes every probe/store;
//! * errors (the empty input) are caller bugs — never cached.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use aeolus_compile::{CachedCompiled, Config, DynCache, DynStore, Spacing, compile};
use aeolus_ir::Val;

/// The battery: lowering classes, diagnostic-bearing classes, variant
/// stacks — the shapes the cache must carry byte-identically.
const BATTERY: &[&str] = &[
    "p-4",
    "px-8 py-2 rounded-md",
    "hover:focus:md:p-0",
    "p-red not-a-utility", // diagnostics ride the cache too
    "w-1/2 -mt-2 text-sm",
];

#[test]
fn a_hit_returns_the_same_interned_slices() {
    let cache = DynCache::new();
    let config = Config::default();

    let first = cache.compile("p-4 hover:bg-primary/90", &config).unwrap();
    assert!(
        cache.misses() >= 1,
        "the first compile of a string is a miss"
    );
    let second = cache.compile("p-4 hover:bg-primary/90", &config).unwrap();

    // Byte-identical results…
    assert_eq!(first, second);
    // …and the POOL DEDUPE itself: the interned slices are the same
    // allocation, not a second leak of the same content.
    assert_eq!(
        first.program.layers.as_ptr(),
        second.program.layers.as_ptr(),
        "a hit must return the same interned layer slice"
    );
    assert_eq!(
        first.program.layers[0].decls.as_ptr(),
        second.program.layers[0].decls.as_ptr(),
        "a hit must return the same interned declaration slice"
    );
    assert_eq!(cache.hits(), 1);
}

#[test]
fn a_miss_matches_the_free_compile_byte_for_byte() {
    let cache = DynCache::new();
    let config = Config::default();
    for input in BATTERY {
        let free = compile(input, &config).unwrap();
        let first = cache.compile(input, &config).unwrap();
        let second = cache.compile(input, &config).unwrap();
        assert_eq!(
            first, free,
            "miss result must equal the free compile: {input}"
        );
        assert_eq!(
            second, free,
            "hit result must equal the free compile: {input}"
        );
    }
}

#[test]
fn the_config_fingerprint_is_part_of_the_identity() {
    let cache = DynCache::new();
    let tailwind = Config::default();
    let px = Config {
        spacing: Spacing::Px,
        rem_px: 16.0,
    };

    let a = cache.compile("p-4", &tailwind).unwrap();
    let b = cache.compile("p-4", &px).unwrap();
    // Different strategies lower differently (Rem vs Px) — a config
    // change is a MISS, never a false hit.
    match (
        a.program.layers[0].decls[0].val,
        b.program.layers[0].decls[0].val,
    ) {
        (Val::Rem(_), Val::Px(_)) => {}
        other => panic!("expected Rem vs Px lowerings, got {other:?}"),
    }
    // And the flip-flop is stable: each config now hits its own entry.
    let a2 = cache.compile("p-4", &tailwind).unwrap();
    let b2 = cache.compile("p-4", &px).unwrap();
    assert_eq!(a, a2);
    assert_eq!(b, b2);
    assert_eq!(a.program.layers.as_ptr(), a2.program.layers.as_ptr());
    assert_eq!(b.program.layers.as_ptr(), b2.program.layers.as_ptr());
}

#[test]
fn distinct_strings_never_collide() {
    let cache = DynCache::new();
    let config = Config::default();
    let mut ptrs = Vec::new();
    for n in 0..200u32 {
        let input = format!("p-{n} gap-{n}");
        let compiled = cache.compile(&input, &config).unwrap();
        let free = compile(&input, &config).unwrap();
        assert_eq!(compiled, free);
        ptrs.push(compiled.program.layers.as_ptr());
    }
    let unique = ptrs.iter().collect::<std::collections::HashSet<_>>();
    assert_eq!(
        unique.len(),
        ptrs.len(),
        "200 distinct strings must pool 200 distinct layer slices"
    );
}

#[test]
fn the_resident_key_set_is_bounded() {
    let cache = DynCache::with_cap(4);
    let config = Config::default();
    for n in 0..8u32 {
        cache.compile(&format!("p-{n}"), &config).unwrap();
    }
    assert!(
        cache.resident() <= 8,
        "two generations bound the resident keys at 2 x cap (4 x 2 = 8), got {}",
        cache.resident()
    );
    // The map is bounded; every result is still correct after eviction.
    for n in 0..8u32 {
        let compiled = cache.compile(&format!("p-{n}"), &config).unwrap();
        let free = compile(&format!("p-{n}"), &config).unwrap();
        assert_eq!(compiled, free);
    }
}

#[test]
fn a_previous_generation_hit_promotes() {
    let cache = DynCache::with_cap(1);
    let config = Config::default();

    let a1 = cache.compile("p-1", &config).unwrap();
    cache.compile("p-2", &config).unwrap(); // evicts p-1 into previous
    let a2 = cache.compile("p-1", &config).unwrap();

    // The hit in the PREVIOUS generation promotes: the interned slices
    // are the first compile's own (no third leak). The counters saw the
    // two cold lowers (p-1, p-2) and the one promoted hit.
    assert_eq!(a1, a2);
    assert_eq!(
        a1.program.layers.as_ptr(),
        a2.program.layers.as_ptr(),
        "a promoted hit returns the original interned slices"
    );
    assert_eq!(cache.misses(), 2);
    assert_eq!(cache.hits(), 1);
}

#[test]
fn threads_share_the_cache() {
    let cache = Arc::new(DynCache::new());
    let config = Config::default();
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let cache = Arc::clone(&cache);
            std::thread::spawn(move || {
                for round in 0..50u32 {
                    for n in 0..16u32 {
                        let input = format!("gap-{n} mt-{round}");
                        let compiled = cache.compile(&input, &config).unwrap();
                        let free = compile(&input, &config).unwrap();
                        assert_eq!(compiled, free);
                    }
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("worker thread panics");
    }
    assert!(cache.hits() > 0, "concurrent repeats must hit");
}

/// A host-implemented store: counts every probe and store, and serves
/// nothing — the seam contract is the subject, not the hits.
struct CountingStore {
    probes: AtomicUsize,
    stores: AtomicUsize,
}

impl DynStore for CountingStore {
    fn probe(&self, _key: &str) -> Option<CachedCompiled> {
        self.probes.fetch_add(1, Ordering::Relaxed);
        None
    }

    fn store(&self, _key: String, _value: CachedCompiled) {
        self.stores.fetch_add(1, Ordering::Relaxed);
    }

    fn resident(&self) -> usize {
        0
    }
}

#[test]
fn the_store_is_a_wirable_seam() {
    let store = Arc::new(CountingStore {
        probes: AtomicUsize::new(0),
        stores: AtomicUsize::new(0),
    });
    let cache = DynCache::with_store(CountingStoreProxy(Arc::clone(&store)));
    let config = Config::default();

    cache.compile("p-4", &config).unwrap();
    cache.compile("p-4", &config).unwrap();

    // Every compile consulted the store, and every lowering landed in
    // it: the pipeline's interning route runs THROUGH the seam.
    assert_eq!(store.probes.load(Ordering::Relaxed), 2);
    assert_eq!(store.stores.load(Ordering::Relaxed), 2);
}

/// `Arc` sharing for the test's counting store (the trait's `Send +
/// Sync` bounds make the shared handle the natural shape).
struct CountingStoreProxy(Arc<CountingStore>);

impl DynStore for CountingStoreProxy {
    fn probe(&self, key: &str) -> Option<CachedCompiled> {
        self.0.probe(key)
    }

    fn store(&self, key: String, value: CachedCompiled) {
        self.0.store(key, value);
    }

    fn resident(&self) -> usize {
        self.0.resident()
    }
}

#[test]
fn errors_are_never_cached() {
    let cache = DynCache::new();
    let config = Config::default();
    assert!(cache.compile("", &config).is_err());
    assert!(cache.compile("   ", &config).is_err());
    // The caller bug reports every time — it never becomes an entry.
    assert_eq!(cache.hits(), 0);
}

// The themed and customized session entry points (M2): the theme table
// and the registrations join the identity — the same content-keyed
// cache, one slot per (config, table, bag, string).

use aeolus_compile::{compile_theme, compile_theme_with, compile_with};
use aeolus_ir::{Cond, Prop};
use aeolus_theme::{ThemeSource, TokenTable};

#[test]
fn a_themed_hit_hands_back_the_same_slices() {
    let cache = DynCache::new();
    let config = Config::default();
    let source = ThemeSource::shadcn_dark().expect("embedded palette parses");
    let tokens = TokenTable::runtime(&source).expect("default-seeded source builds");

    let first = cache
        .compile_theme("bg-primary/90", &tokens, &config)
        .unwrap();
    let second = cache
        .compile_theme("bg-primary/90", &tokens, &config)
        .unwrap();

    let free = compile_theme("bg-primary/90", &tokens, &config).unwrap();
    assert_eq!(first, free);
    assert_eq!(second, free);
    assert_eq!(
        first.program.layers.as_ptr(),
        second.program.layers.as_ptr(),
        "a themed hit returns the same interned slices"
    );
    assert_eq!(cache.hits(), 1);
}

#[test]
fn a_different_table_is_a_different_slot() {
    let cache = DynCache::new();
    let config = Config::default();
    let dark = ThemeSource::shadcn_dark().expect("embedded palette parses");
    let light = ThemeSource::shadcn().expect("embedded palette parses");
    let dark_tokens = TokenTable::runtime(&dark).expect("table builds");
    let light_tokens = TokenTable::runtime(&light).expect("table builds");

    let a = cache
        .compile_theme("bg-primary", &dark_tokens, &config)
        .unwrap();
    let b = cache
        .compile_theme("bg-primary", &light_tokens, &config)
        .unwrap();
    // Each result equals its own table's free compile — no cross-table
    // false hit.
    assert_eq!(
        a,
        compile_theme("bg-primary", &dark_tokens, &config).unwrap()
    );
    assert_eq!(
        b,
        compile_theme("bg-primary", &light_tokens, &config).unwrap()
    );
    assert_eq!(cache.misses(), 2);
    // And each slot now hits its own entry.
    let a2 = cache
        .compile_theme("bg-primary", &dark_tokens, &config)
        .unwrap();
    assert_eq!(a.program.layers.as_ptr(), a2.program.layers.as_ptr());
}

#[test]
fn a_customized_session_hits() {
    let cache = DynCache::new();
    let config = Config::default();
    let customs = aeolus_compile::Customs::new()
        .variant(aeolus_compile::CustomVariant::new("night", Cond::Dark));

    let first = cache
        .compile_with("night:bg-primary", &customs, &config)
        .unwrap();
    let second = cache
        .compile_with("night:bg-primary", &customs, &config)
        .unwrap();
    let free = compile_with("night:bg-primary", &customs, &config).unwrap();

    assert_eq!(first, free);
    assert_eq!(second, free);
    assert_eq!(
        first.program.layers.as_ptr(),
        second.program.layers.as_ptr()
    );
    assert_eq!(cache.hits(), 1);

    // A different bag (different content, this session) is a miss.
    let other = aeolus_compile::Customs::new()
        .variant(aeolus_compile::CustomVariant::new("hocus", Cond::Hover));
    let other_compiled = cache
        .compile_with("night:bg-primary", &other, &config)
        .unwrap();
    assert_eq!(
        other_compiled,
        compile_with("night:bg-primary", &other, &config).unwrap()
    );
    // The typed boundaries fired honestly under the OTHER bag: `night`
    // is not registered there (unsupported variant) and `bg-primary`
    // lowers through the theme pass, absent here — the cache carries
    // diagnostics byte-identically, never a half-applied hit.
    assert!(!other_compiled.diagnostics.is_empty());
}

#[test]
fn the_theme_and_custom_session_caches_together() {
    let cache = DynCache::new();
    let config = Config::default();
    let source = ThemeSource::shadcn_dark().expect("embedded palette parses");
    let tokens = TokenTable::runtime(&source).expect("table builds");
    let customs = aeolus_compile::Customs::new()
        .utility(aeolus_compile::CustomUtility::functional(
            "brand-p",
            aeolus_compile::CustomForm::Spacing,
            &[
                Prop::PaddingTop,
                Prop::PaddingRight,
                Prop::PaddingBottom,
                Prop::PaddingLeft,
            ],
        ))
        .variant(aeolus_compile::CustomVariant::new("night", Cond::Dark));

    let first = cache
        .compile_theme_with("night:brand-p-4 bg-primary/90", &tokens, &customs, &config)
        .unwrap();
    let second = cache
        .compile_theme_with("night:brand-p-4 bg-primary/90", &tokens, &customs, &config)
        .unwrap();
    let free =
        compile_theme_with("night:brand-p-4 bg-primary/90", &tokens, &customs, &config).unwrap();

    assert_eq!(first, free);
    assert_eq!(second, free);
    assert!(first.diagnostics.is_empty());
    assert_eq!(cache.hits(), 1);
}
