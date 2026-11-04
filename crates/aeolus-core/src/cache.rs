//! The dynamic-class-string hot path: hash-cached resolution.
//!
//! Static class strings never touch this — `tw!` validates and interns them
//! at compile time. Dynamic strings (app-state-composed classes) hash-lookup
//! here; steady state is **one hash lookup, zero re-resolution**.
//!
//! Cache key: `(class_hash, theme_version)` — variant activation is not part
//! of the key because the plan is variant-*shaped*, not variant-*resolved*
//! (the framework activates layers).
//!
//! Boundedness (Phase 2 hardening): both [`Interner`] and [`PlanCache`] are
//! bounded by a **two-generation rotation** — when the current generation
//! fills to `cap`, it becomes the previous generation and a fresh current
//! starts. A lookup consults both generations, so entries survive one extra
//! generation before eviction (total memory ≤ 2×`cap`). Eviction is
//! generation-based, not LRU: the tradeoff is that a hot-but-old entry can
//! be evicted, in exchange for O(1) eviction with zero dependencies
//! (aeolus-core stays dep-free; an LRU-feature is deliberately not added).
//! Evicted interner IDs are never reused — re-interning an evicted string
//! yields a NEW id (callers treat ids as per-generation cache keys).

use crate::resolve::{Resolution, Resolver};
use crate::theme::TailwindTheme;
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::{Arc, RwLock};

/// Default bound on the current generation of [`Interner`] / [`PlanCache`].
///
/// The shadcn v1 surface resolves to a few hundred distinct plans in a
/// typical app; 4096 current + 4096 previous generations bound memory at a
/// few hundred KiB while comfortably exceeding real app class churn.
pub const DEFAULT_CACHE_CAP: usize = 4096;

/// A resolved plan plus its diagnostics, shared behind an `Arc`.
#[derive(Clone, Debug)]
pub struct CachedPlan {
    /// The resolution (plan + warnings) produced against one theme version.
    pub resolution: Resolution,
}

/// FxHash — fast, order-independent, good enough for string keys (and
/// dependency-free).
#[derive(Default)]
pub struct FxHasher {
    hash: u64,
}

const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

impl FxHasher {
    fn add(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.hash = (self.hash.rotate_left(5) ^ (*b as u64)).wrapping_mul(SEED);
        }
    }
}

impl Hasher for FxHasher {
    fn finish(&self) -> u64 {
        self.hash
    }
    fn write(&mut self, bytes: &[u8]) {
        self.add(bytes);
    }
}

/// Build the hash of a class string (used by both cache and interner).
pub fn hash_classes(s: &str) -> u64 {
    let mut h = FxHasher::default();
    h.add(s.as_bytes());
    h.finish()
}

/// Poison-tolerant read: a panicked writer leaves the guarded data
/// logically consistent here (plain maps of already-resolved values, no
/// cross-lock invariants), so recovering the inner value is safe and the
/// caller never panics (Phase 2: `expect("lock")` removed on purpose).
fn read_map<'a, V>(
    lock: &'a RwLock<HashMap<u64, V>>,
) -> std::sync::RwLockReadGuard<'a, HashMap<u64, V>> {
    lock.read().unwrap_or_else(|e| e.into_inner())
}

fn write_map<'a, V>(
    lock: &'a RwLock<HashMap<u64, V>>,
) -> std::sync::RwLockWriteGuard<'a, HashMap<u64, V>> {
    lock.write().unwrap_or_else(|e| e.into_inner())
}

/// A thread-safe resolver cache keyed by `(class_hash, theme_version)`,
/// bounded by two-generation rotation (see module docs).
pub struct PlanCache {
    cap: usize,
    current: RwLock<HashMap<u64, Arc<CachedPlan>>>,
    previous: RwLock<HashMap<u64, Arc<CachedPlan>>>,
}

impl Default for PlanCache {
    fn default() -> Self {
        Self::with_cap(DEFAULT_CACHE_CAP)
    }
}

impl PlanCache {
    /// A cache with the default capacity ([`DEFAULT_CACHE_CAP`]).
    pub fn new() -> Self {
        Self::default()
    }

    /// A cache that holds at most `cap` plans per generation (≤ 2×cap total).
    pub fn with_cap(cap: usize) -> Self {
        Self {
            cap: cap.max(1),
            current: RwLock::new(HashMap::new()),
            previous: RwLock::new(HashMap::new()),
        }
    }

    /// Resolve `classes` against `theme`, caching by content hash.
    pub fn resolve(
        &self,
        theme: &TailwindTheme,
        classes: &str,
    ) -> Result<Arc<CachedPlan>, Vec<crate::color::ParseError>> {
        let key = hash_classes(classes) ^ theme.version().rotate_left(32);
        if let Some(hit) = read_map(&self.current).get(&key) {
            return Ok(hit.clone());
        }
        if let Some(hit) = read_map(&self.previous).get(&key) {
            return Ok(hit.clone());
        }
        let resolver = Resolver::new(theme);
        let resolution = resolver.resolve(classes)?;
        let arc = Arc::new(CachedPlan { resolution });
        let mut current = write_map(&self.current);
        if self.cap > 0 && current.len() >= self.cap {
            // Rotate: the current generation becomes the previous one.
            *write_map(&self.previous) = std::mem::take(&mut *current);
            debug_assert!(current.is_empty());
        }
        current.insert(key, arc.clone());
        Ok(arc)
    }

    /// Number of cached plans across both generations.
    pub fn len(&self) -> usize {
        read_map(&self.current).len() + read_map(&self.previous).len()
    }

    /// True when nothing is cached.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Interner mapping class strings to dense IDs (compile-time trie path
/// uses the same IDs produced by `tw!`), bounded by two-generation
/// rotation (see module docs).
pub struct Interner {
    cap: usize,
    current: RwLock<HashMap<u64, u32>>,
    previous: RwLock<HashMap<u64, u32>>,
    next: std::sync::atomic::AtomicU32,
}

impl Default for Interner {
    fn default() -> Self {
        Self::with_cap(DEFAULT_CACHE_CAP)
    }
}

impl Interner {
    /// An interner with the default capacity ([`DEFAULT_CACHE_CAP`]).
    pub fn new() -> Self {
        Self::default()
    }

    /// An interner that holds at most `cap` strings per generation
    /// (≤ 2×cap total).
    pub fn with_cap(cap: usize) -> Self {
        Self {
            cap: cap.max(1),
            current: RwLock::new(HashMap::new()),
            previous: RwLock::new(HashMap::new()),
            next: std::sync::atomic::AtomicU32::new(0),
        }
    }

    /// Intern a class string; the returned ID is stable per string content
    /// *within the two-generation window* — re-interning an evicted string
    /// yields a fresh ID (documented tradeoff of boundedness).
    pub fn intern(&self, s: &str) -> u32 {
        let h = hash_classes(s);
        if let Some(id) = read_map(&self.current).get(&h) {
            return *id;
        }
        if let Some(id) = read_map(&self.previous).get(&h) {
            return *id;
        }
        let id = self.next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut current = write_map(&self.current);
        if self.cap > 0 && current.len() >= self.cap {
            *write_map(&self.previous) = std::mem::take(&mut *current);
        }
        current.insert(h, id);
        id
    }

    /// Number of interned strings across both generations.
    pub fn len(&self) -> usize {
        read_map(&self.current).len() + read_map(&self.previous).len()
    }

    /// True when nothing was interned yet.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::TailwindTheme;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    /// Phase-2 item: a poisoned RwLock must never panic a caller. The
    /// lock guard is logically consistent after a writer panicked, so
    /// recovering the inner data is always safe here (the map contents are
    /// plain resolved plans, no cross-lock invariants).
    #[test]
    fn poisoned_lock_does_not_panic_the_caller() {
        let cache = PlanCache::new();
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _w = cache.current.write().unwrap();
            panic!("poison the cache map lock");
        }));
        let theme = TailwindTheme::new();
        let r = catch_unwind(AssertUnwindSafe(|| cache.resolve(&theme, "p-4").is_ok()));
        assert!(
            r.is_ok() && r.unwrap(),
            "resolve must survive a poisoned lock"
        );

        let interner = Interner::new();
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _w = interner.current.write().unwrap();
            panic!("poison the interner lock");
        }));
        let id = catch_unwind(AssertUnwindSafe(|| interner.intern("p-4")));
        assert!(id.is_ok(), "intern must survive a poisoned lock");
    }

    /// Phase-2 item: the interner is bounded; exceeding the cap rotates
    /// generations (previous survives one more generation, then is evicted).
    #[test]
    fn interner_evicts_beyond_cap() {
        let interner = Interner::with_cap(2);
        let a = interner.intern("a");
        let b = interner.intern("b");
        assert_eq!(interner.len(), 2);
        let c = interner.intern("c"); // fills the current generation
        assert!(
            interner.len() <= 4,
            "total bounded at 2x cap, got {}",
            interner.len()
        );
        // Previous-generation entries keep their ids for one more generation.
        assert_eq!(
            interner.intern("b"),
            b,
            "b survives in the previous generation"
        );
        let _ = interner.intern("d"); // current {c,d}
        let _ = interner.intern("e"); // rotates: prev {c,d}, cur {e}
        assert_eq!(interner.intern("c"), c, "c survives one more generation");
        assert_ne!(
            interner.intern("a"),
            a,
            "a was evicted after two generations"
        );
    }

    /// Phase-2 item: the plan cache is bounded the same way, and warm
    /// entries from the previous generation still hit (Arc-identical).
    #[test]
    fn plan_cache_evicts_beyond_cap() {
        let cache = PlanCache::with_cap(2);
        let theme = TailwindTheme::new();
        let a = cache.resolve(&theme, "p-4").unwrap();
        cache.resolve(&theme, "p-2").unwrap();
        assert_eq!(cache.len(), 2);
        let c = cache.resolve(&theme, "p-1").unwrap(); // rotates
        assert!(
            cache.len() <= 4,
            "total bounded at 2x cap, got {}",
            cache.len()
        );
        let a2 = cache.resolve(&theme, "p-4").unwrap();
        assert!(Arc::ptr_eq(&a, &a2), "previous-generation entry still hits");
        let c2 = cache.resolve(&theme, "p-1").unwrap();
        assert!(Arc::ptr_eq(&c, &c2), "current-generation entry still hits");
    }
}
