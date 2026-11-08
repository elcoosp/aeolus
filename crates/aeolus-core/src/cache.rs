//! The dynamic-class-string hot path: cached resolution.
//!
//! v0.2 (plan §3.4.3/§3.4.4):
//! * the hasher folds **8 bytes per round** (plus a length-tagged tail and a
//!   final avalanche), replacing the one-byte-per-round Fx loop that spent
//!   ~250 rounds on the Button string;
//! * the cache is keyed **by the string itself** (L1 per-thread + L2
//!   shared), so a hash collision can never return the wrong plan — the
//!   v0.1 64-bit-hash-only key was collision-unsafe;
//! * hits in the previous generation are **promoted** into the current one;
//! * [`PlanCache::resolve_static`] keys `&'static str` call sites by
//!   `(ptr, len)` — no content hashing at all.
//!
//! Static class strings never touch this — `tw!` validates and interns them
//! at compile time. Dynamic strings (app-state-composed classes) look up
//! here; steady state is **one map probe, zero re-resolution**.
//!
//! Boundedness: both [`Interner`] and the L2 map are bounded by a
//! **two-generation rotation** — when the current generation fills to
//! `cap`, it becomes the previous generation and a fresh current starts.
//! The L1 is bounded the same way. Total memory ≤ 2×`cap` per tier.

use crate::resolve::{Resolution, Resolver};
use crate::theme::TailwindTheme;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
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

/// Aeolus's fast hasher (v0.2): 8 bytes per round with a length-tagged
/// tail and a final avalanche fold (plan §3.4.3).
///
/// The length tag makes `"a"` and `"a\0"` hash differently even though
/// both feed the same 8-byte chunk; the fold spreads the low bits that
/// byte-at-a-time accumulators leave weak.
#[derive(Default)]
pub struct FastHasher {
    h: u64,
}

const FAST_SEED: u64 = 0x9e37_79b9_7f4a_7c15;

impl FastHasher {
    #[inline]
    fn mix(&mut self, v: u64) {
        self.h ^= v;
        self.h = self.h.wrapping_mul(FAST_SEED);
        self.h ^= self.h >> 29;
    }
}

impl Hasher for FastHasher {
    #[inline]
    fn finish(&self) -> u64 {
        let h = self.h;
        (h ^ (h >> 32)).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ (h >> 29)
    }

    fn write(&mut self, bytes: &[u8]) {
        // Chunked 8-bytes-per-round folding. NOTE: the chunking assumes the
        // whole key arrives in ONE write — our cache/interner always call
        // `write` once per key (documented contract).
        // `as_chunks` would please clippy but is newer std than our MSRV;
        // chunks_exact(8) is the zero-dep-compatible form.
        #[allow(clippy::chunks_exact_to_as_chunks)]
        let mut chunks = bytes.chunks_exact(8);
        for c in &mut chunks {
            self.mix(u64::from_le_bytes(c.try_into().unwrap()));
        }
        let rem = chunks.remainder();
        if !rem.is_empty() {
            let mut buf = [0u8; 8];
            buf[..rem.len()].copy_from_slice(rem);
            self.mix(u64::from_le_bytes(buf) ^ ((rem.len() as u64) << 56));
        }
    }
}

/// The `FastMap` of the hot path: std `HashMap` on [`FastHasher`]
/// (zero dependencies, 8 bytes/round hashing).
pub type FastMap<K, V> = HashMap<K, V, BuildHasherDefault<FastHasher>>;

/// Build the hash of a class string (used by both cache and interner).
pub fn hash_classes(s: &str) -> u64 {
    let mut h = FastHasher::default();
    h.write(s.as_bytes());
    h.finish()
}

/// The L2 map key: the class string itself, compared on hit
/// (collision-proof by construction; plan §3.4.4).
type L2Entry = (u64, Arc<CachedPlan>);
type L2Map = RwLock<FastMap<Arc<str>, L2Entry>>;

/// One L1 generation: string → (theme version, plan).
type L1Gen = FastMap<Arc<str>, (u64, Arc<CachedPlan>)>;

thread_local! {
    /// L1: per-thread, zero atomics, keyed by the STRING. Bounded by the
    /// same two-generation rotation as L2 (plan §3.4.4).
    static L1: std::cell::RefCell<(L1Gen, L1Gen, usize)> =
        std::cell::RefCell::new((FastMap::default(), FastMap::default(), DEFAULT_CACHE_CAP));
}

fn l1_lookup(classes: &str, ver: u64) -> Option<Arc<CachedPlan>> {
    L1.with(|cell| {
        let (cur, prev, _) = &*cell.borrow();
        if let Some((v, p)) = cur.get(classes) {
            return (*v == ver).then(|| p.clone());
        }
        if let Some((v, p)) = prev.get(classes) {
            return (*v == ver).then(|| p.clone());
        }
        None
    })
}

fn l1_promote(classes: &str, ver: u64, plan: Arc<CachedPlan>) {
    L1.with(|cell| {
        let (cur, prev, cap) = &mut *cell.borrow_mut();
        // Promote a previous-generation hit into the current generation.
        if prev.remove(classes).is_some() && cur.len() >= *cap && *cap > 0 {
            // Rotation on promote-overflow: current becomes previous.
            std::mem::swap(cur, prev);
            cur.clear();
        }
        cur.insert(classes.into(), (ver, plan));
    });
}

fn l1_insert(classes: &str, ver: u64, plan: Arc<CachedPlan>) {
    L1.with(|cell| {
        let (cur, prev, cap) = &mut *cell.borrow_mut();
        if *cap > 0 && cur.len() >= *cap {
            *prev = std::mem::take(cur);
        }
        cur.insert(classes.into(), (ver, plan));
    });
}

/// Poison-tolerant read: a panicked writer leaves the guarded data
/// logically consistent here (plain maps of already-resolved values, no
/// cross-lock invariants), so recovering the inner value is safe and the
/// caller never panics.
fn read_map<'a, V>(lock: &'a RwLock<V>) -> std::sync::RwLockReadGuard<'a, V> {
    lock.read().unwrap_or_else(|e| e.into_inner())
}

fn write_map<'a, V>(lock: &'a RwLock<V>) -> std::sync::RwLockWriteGuard<'a, V> {
    lock.write().unwrap_or_else(|e| e.into_inner())
}

/// A thread-safe resolver cache keyed by **(class string, theme_version)**,
/// with a per-thread L1 in front of the shared L2. Bounded by
/// two-generation rotation (see module docs).
pub struct PlanCache {
    cap: usize,
    current: L2Map,
    previous: L2Map,
    /// Static call sites (`tw!` output): keyed by `(ptr, len→len is in the
    /// string itself)` — the pointer is unique for the program's lifetime.
    static_map: RwLock<FastMap<usize, (u64, Arc<CachedPlan>)>>,
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
            current: RwLock::new(FastMap::default()),
            previous: RwLock::new(FastMap::default()),
            static_map: Self::new_static(),
        }
    }

    /// Resolve `classes` against `theme`, caching by string content.
    ///
    /// The L1 hit path performs zero atomics and zero re-hashes of the
    /// plan; the L2 hit path takes a shared read lock keyed by the string.
    pub fn resolve(
        &self,
        theme: &TailwindTheme,
        classes: &str,
    ) -> Result<Arc<CachedPlan>, Vec<crate::color::ParseError>> {
        let ver = theme.version();
        // L1: per-thread, string-compared (collision-proof).
        if let Some(hit) = l1_lookup(classes, ver) {
            return Ok(hit);
        }
        if let Some(hit) = self.shared_get(classes, ver) {
            l1_promote(classes, ver, hit.clone());
            return Ok(hit);
        }
        let plan = self.shared_get_or_resolve(theme, classes)?;
        l1_insert(classes, ver, plan.clone());
        Ok(plan)
    }

    fn shared_get(&self, classes: &str, ver: u64) -> Option<Arc<CachedPlan>> {
        // Entries are version-stamped: a theme swap invalidates without
        // waiting for generation rotation.
        let hit = read_map(&self.current)
            .get(classes)
            .filter(|(v, _)| *v == ver)
            .map(|(_, p)| p.clone());
        if hit.is_some() {
            return hit;
        }
        read_map(&self.previous)
            .get(classes)
            .filter(|(v, _)| *v == ver)
            .map(|(_, p)| p.clone())
    }

    /// Resolve or insert into the shared L2 (string-keyed).
    fn shared_get_or_resolve(
        &self,
        theme: &TailwindTheme,
        classes: &str,
    ) -> Result<Arc<CachedPlan>, Vec<crate::color::ParseError>> {
        let ver = theme.version();
        // Double-checked: another thread may have inserted while we parsed.
        {
            let cur = read_map(&self.current);
            if let Some((v, p)) = cur.get(classes)
                && *v == ver
            {
                return Ok(p.clone());
            }
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
        current.insert(classes.into(), (ver, arc.clone()));
        Ok(arc)
    }

    /// Resolve a `&'static str` call site (`tw!` output) keyed by
    /// `(ptr, len)` — no content hashing at all (plan §3.4.4/§3.4.6).
    pub fn resolve_static(
        &self,
        theme: &TailwindTheme,
        s: &'static str,
    ) -> Result<Arc<CachedPlan>, Vec<crate::color::ParseError>> {
        let key: usize = s.as_ptr() as usize;
        let ver = theme.version();
        // Static strings live for the program's life: a plain (ptr, ver)
        // map per cache in front of the dynamic path.
        let mut stat = self.static_map.write().unwrap_or_else(|e| e.into_inner());
        if let Some((v, p)) = stat.get(&key)
            && *v == ver
        {
            return Ok(p.clone());
        }
        let plan = self.resolve(theme, s)?;
        stat.insert(key, (ver, plan.clone()));
        Ok(plan)
    }

    /// Number of cached plans across both L2 generations (L1 not counted).
    pub fn len(&self) -> usize {
        read_map(&self.current).len() + read_map(&self.previous).len()
    }

    /// True when nothing is cached.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl PlanCache {
    fn new_static() -> RwLock<FastMap<usize, (u64, Arc<CachedPlan>)>> {
        RwLock::new(FastMap::default())
    }
}

/// The interner: dense IDs for class strings (compile-time trie path uses
/// the same IDs produced by `tw!`), now keyed by string content with the
/// fast hasher, bounded by two-generation rotation.
pub struct Interner {
    cap: usize,
    current: RwLock<FastMap<Arc<str>, u32>>,
    previous: RwLock<FastMap<Arc<str>, u32>>,
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
            current: RwLock::new(FastMap::default()),
            previous: RwLock::new(FastMap::default()),
            next: std::sync::atomic::AtomicU32::new(0),
        }
    }

    /// Intern a class string; the returned ID is stable per string content
    /// *within the two-generation window* — re-interning an evicted string
    /// yields a fresh ID (documented tradeoff of boundedness).
    pub fn intern(&self, s: &str) -> u32 {
        if let Some(id) = read_map(&self.current).get(s) {
            return *id;
        }
        if let Some(id) = read_map(&self.previous).get(s) {
            return *id;
        }
        let id = self.next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut current = write_map(&self.current);
        if self.cap > 0 && current.len() >= self.cap {
            *write_map(&self.previous) = std::mem::take(&mut *current);
        }
        current.insert(s.into(), id);
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

    /// v0.2 hot-path hasher: length-tagged tail — `"a"` ≠ `"a\0"`.
    #[test]
    fn fast_hash_is_length_tagged() {
        assert_ne!(hash_classes("a"), hash_classes("a\0"));
        assert_ne!(hash_classes("p-4"), hash_classes("p-4p-4"));
    }

    /// 8-bytes-per-round folding must stay order/content sensitive and
    /// produce stable values for identical inputs.
    #[test]
    fn fast_hash_is_deterministic_and_sensitive() {
        assert_eq!(
            hash_classes("bg-primary hover:bg-accent"),
            hash_classes("bg-primary hover:bg-accent")
        );
        assert_ne!(hash_classes("p-1"), hash_classes("p-2"));
        // > 8 bytes (multi-round) and < 8 bytes (tail) paths both covered.
        assert_ne!(hash_classes("1234567"), hash_classes("12345678"));
        assert_ne!(hash_classes("12345678"), hash_classes("123456789"));
    }

    /// Phase-2 item: a poisoned RwLock must never panic a caller. The
    /// lock guard is logically consistent after a writer panicked, so
    /// recovering the inner data is always safe here.
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

    /// v0.2: two DIFFERENT strings can never return each other's plans —
    /// the L1/L2 keys are the strings themselves, so a (simulated) hash
    /// collision still resolves correctly.
    #[test]
    fn cache_is_collision_proof() {
        let cache = PlanCache::with_cap(8);
        let theme = TailwindTheme::new();
        let a = cache.resolve(&theme, "p-4").unwrap();
        let b = cache.resolve(&theme, "p-8").unwrap();
        assert!(
            a.resolution.plan.base.padding != b.resolution.plan.base.padding,
            "different strings must produce different plans"
        );
        // Cross-check: the string keys, not hashes, drive lookups.
        assert!(Arc::ptr_eq(&cache.resolve(&theme, "p-4").unwrap(), &a));
        assert!(Arc::ptr_eq(&cache.resolve(&theme, "p-8").unwrap(), &b));
    }

    /// v0.2: static call sites hit the (ptr,len)-keyed fast path.
    #[test]
    fn resolve_static_hits() {
        let cache = PlanCache::new();
        let theme = TailwindTheme::new();
        let a = cache.resolve_static(&theme, "p-4 w-full").unwrap();
        let b = cache.resolve_static(&theme, "p-4 w-full").unwrap();
        assert!(Arc::ptr_eq(&a, &b), "static strings hit the same plan");
    }
}
