//! The dynamic-class-string hot path: hash-cached resolution.
//!
//! Static class strings never touch this — `tw!` validates and interns them
//! at compile time. Dynamic strings (app-state-composed classes) hash-lookup
//! here; steady state is **one hash lookup, zero re-resolution**.
//!
//! Cache key: `(class_hash, theme_version)` — variant activation is not part
//! of the key because the plan is variant-*shaped*, not variant-*resolved*
//! (the framework activates layers).

use crate::resolve::{Resolution, Resolver};
use crate::theme::TailwindTheme;
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::Arc;

/// A resolved plan plus its diagnostics, shared behind an `Arc`.
#[derive(Clone, Debug)]
pub struct CachedPlan {
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

/// A thread-safe resolver cache keyed by `(class_hash, theme_version)`.
#[derive(Default)]
pub struct PlanCache {
    map: std::sync::RwLock<HashMap<u64, Arc<CachedPlan>>>,
}

impl PlanCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolve `classes` against `theme`, caching by content hash.
    pub fn resolve(&self, theme: &TailwindTheme, classes: &str) -> Result<Arc<CachedPlan>, Vec<crate::color::ParseError>> {
        let key = hash_classes(classes) ^ theme.version().rotate_left(32);
        if let Some(hit) = self.map.read().expect("cache lock").get(&key) {
            return Ok(hit.clone());
        }
        let resolver = Resolver::new(theme);
        let resolution = resolver.resolve(classes)?;
        let arc = Arc::new(CachedPlan { resolution });
        self.map.write().expect("cache lock").insert(key, arc.clone());
        Ok(arc)
    }

    /// Number of cached plans.
    pub fn len(&self) -> usize {
        self.map.read().expect("cache lock").len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Interner mapping class strings to dense IDs (compile-time trie path
/// uses the same IDs produced by `tw!`).
#[derive(Default)]
pub struct Interner {
    ids: std::sync::RwLock<HashMap<u64, u32>>,
    next: std::sync::atomic::AtomicU32,
}

impl Interner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Intern a class string; the returned ID is stable per string content.
    pub fn intern(&self, s: &str) -> u32 {
        let h = hash_classes(s);
        if let Some(id) = self.ids.read().expect("interner lock").get(&h) {
            return *id;
        }
        let id = self
            .next
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.ids.write().expect("interner lock").insert(h, id);
        id
    }

    pub fn len(&self) -> usize {
        self.ids.read().expect("interner lock").len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
