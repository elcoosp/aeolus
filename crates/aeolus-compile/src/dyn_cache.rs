//! The dynamic-string cache (architecture plan §3.2's `DynCache` — the
//! "P2's DynCache dedupes the pool later" promise, kept): a bounded,
//! thread-safe, content-keyed cache over the compile seam's leak pool.
//!
//! ```text
//! DynCache::new()
//!   .compile("p-4 hover:bg-primary/90", &config)   -> miss  (compile + pool)
//!   .compile("p-4 hover:bg-primary/90", &config)   -> hit   (same slices)
//! ```
//!
//! The problem it solves is the one the crate docs' storage note names:
//! every [`compile`](crate::compile) call interns its program into the
//! leak pool the IR documents as the dynamic form's storage ("one
//! interned `Box<[Decl]>` shared by reference") — so one compile per
//! DISTINCT class string was the intended call pattern, and a UI loop
//! recompiling the same dynamic string every frame pooled a fresh copy
//! each frame. The cache makes the repeated compile the CHEAP path:
//!
//! * a HIT returns the same interned [`Program`] the miss pooled — the
//!   slices are pointer-identical (the tests pin the addresses), so a
//!   repeated string pools exactly once while the cache holds it;
//! * the KEY is the class string itself — never a hash of it (the v0.x
//!   `PlanCache`'s collision-safety rule, plan §3.4.3: "a hash collision
//!   can never return the wrong plan");
//! * the stored IDENTITY is the compile-time configuration fingerprint:
//!   a different [`Config`] is a miss, never a false hit;
//! * the resident key set is BOUNDED by a two-generation rotation (the
//!   v0.x boundedness story): when the current generation fills to the
//!   cap it becomes the previous generation and a fresh current starts;
//!   a hit in the previous generation PROMOTES into the current one, so
//!   steady-state churn keeps its working set resident at ≤ 2×cap keys;
//! * the storage is a SEAM: the pipeline's cache is generic over
//!   [`DynStore`] (the same shape as the custom seam's
//!   [`ClassifyVariant`](crate::ClassifyVariant) — the fold is generic
//!   over the classifier, the cache is generic over the store), with
//!   [`TwoGenStore`] as the built-in and a host's own store (a sharded
//!   map, an arena, a test double) wireable through
//!   [`DynCache::with_store`].
//!
//! Observability follows the crate's opt-in instrumentation seam: the
//! `tracing` feature spans the hit/miss/store decisions (no subscriber
//! is installed here — the host app owns one), and the [`DynCache`]
//! surface exposes the hit/miss counters and the resident key count
//! directly for hosts that budget memory or assert cache behaviour in
//! their own tests.
//!
//! What this slice deliberately is NOT (each documented where it
//! belongs): caching the emit half (`emit_css` and siblings — the CSS
//! sheet is owned text, and the same identity-plus-input keying applies
//! once that slice lands) and evicting leaked slices (an evicted
//! string's slices stay in the leak pool — they are `&'static` by the
//! IR's documented storage; the cache bounds the RESIDENT KEY SET and
//! the dedupe is per-resident-entry, which is what bounds the pool's
//! growth RATE).
//!
//! Errors are never cached: the only whole-string failure is the empty
//! input ([`CompileError::Empty`](crate::CompileError::Empty)) and it is
//! a caller bug — it reports on every call, never becomes an entry.

use std::collections::HashMap;
use std::hash::RandomState;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use aeolus_ir::Program;
use aeolus_theme::TokenTable;

use crate::{CompileError, Compiled, Config, Customs, Diagnostic};

/// Default bound on the current generation of [`TwoGenStore`] — the v0.x
/// `DEFAULT_CACHE_CAP` heritage number: the shadcn v1 surface resolves to
/// a few hundred distinct programs in a typical app, so 4096 current +
/// 4096 previous bound the resident map at a few thousand keys while
/// comfortably exceeding real app class churn.
pub const DEFAULT_DYN_CACHE_CAP: usize = 4096;

/// The identity a cached entry was produced under: the compile-time
/// configuration fingerprint plus the SESSION inputs' addresses (the
/// theme table and the registrations — both immutable through their
/// API, both bound once per host session, the intended call pattern
/// the free functions' docs already prescribe). A hit requires the
/// probe's identity to EQUAL the entry's on every field — a different
/// [`Config`] lowers differently (the spacing knob bakes `Rem` vs
/// `Px`), a different table resolves tokens differently, a different
/// registrations bag authors differently — so the identity is exact
/// equality, never a hash.
///
/// The addresses are ADDRESSES, not content hashes: the v0.x
/// `PlanCache::resolve_static` precedent ("keys `&'static str` call
/// sites by `(ptr, len)` — no content hashing at all", plan §3.4.3),
/// scoped by the same discipline — one table and one registrations bag
/// per session. The empty registrations are the ZERO address (their
/// byte-identity to the plain entry points is the custom seam's
/// differential contract), so the theme-less and custom-less paths
/// never key on a temporary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DynIdentity {
    /// The [`Config`] fingerprint: the [`crate::Spacing`] discriminant
    /// and the `rem_px` bit pattern, mixed into one word.
    pub config: u64,
    /// The theme table's address (0 = the theme-less compile).
    pub tokens: usize,
    /// The registrations' address (0 = none, or empty registrations).
    pub customs: usize,
}

impl DynIdentity {
    /// The identity for the theme-less, custom-less compile: the
    /// configuration fingerprint alone.
    pub(crate) fn of(config: Config) -> Self {
        Self {
            config: Self::config_fingerprint(config),
            tokens: 0,
            customs: 0,
        }
    }

    /// The identity for a themed compile (the table's address joins).
    pub(crate) fn themed(config: Config, tokens: &TokenTable) -> Self {
        Self {
            config: Self::config_fingerprint(config),
            tokens: std::ptr::from_ref(tokens).addr(),
            customs: 0,
        }
    }

    /// The identity for a customized compile (the bag's address joins —
    /// unless the registrations are EMPTY, whose byte-identity to the
    /// plain entry points is the custom seam's differential contract,
    /// so they key as zero like their absence).
    pub(crate) fn custom(config: Config, customs: &Customs) -> Self {
        Self {
            config: Self::config_fingerprint(config),
            tokens: 0,
            customs: if customs.is_empty() {
                0
            } else {
                std::ptr::from_ref(customs).addr()
            },
        }
    }

    /// The identity for the full session: table and bag both join.
    pub(crate) fn themed_custom(config: Config, tokens: &TokenTable, customs: &Customs) -> Self {
        Self {
            config: Self::config_fingerprint(config),
            tokens: std::ptr::from_ref(tokens).addr(),
            customs: if customs.is_empty() {
                0
            } else {
                std::ptr::from_ref(customs).addr()
            },
        }
    }

    /// The [`Config`] fingerprint: the [`crate::Spacing`] discriminant
    /// and the `rem_px` bit pattern, mixed into one word.
    fn config_fingerprint(config: Config) -> u64 {
        let spacing = match config.spacing {
            crate::Spacing::Tailwind => 0u64,
            crate::Spacing::Px => 1u64,
        };
        spacing | ((config.rem_px.to_bits() as u64) << 8)
    }

    /// The key prefix the cache composes with the input string: the
    /// identity's fields, hex-separated, then the input. Distinct
    /// identities own distinct keys — a config change, a different
    /// table or a different bag is a different SLOT, never a false hit
    /// and never a thrash.
    fn key_prefix(self) -> String {
        format!(
            "{:016x}\u{1}{:x}\u{1}{:x}\u{1}",
            self.config, self.tokens, self.customs
        )
    }
}

/// One cached compile result: the identity it was produced under, the
/// interned program (the pool's own slices — a hit hands back the same
/// allocation) and the per-class diagnostics, shared behind an `Arc` so
/// a hit copies the program (Copy) and bumps the report counter, and
/// never re-renders the reports.
#[derive(Clone, Debug)]
pub struct CachedCompiled {
    /// The [`Config`] fingerprint the entry lowered under.
    pub identity: DynIdentity,
    /// The interned program (the leak pool's own slices).
    pub program: Program,
    /// The honest per-class reports, shared.
    pub diagnostics: Arc<[Diagnostic]>,
}

impl CachedCompiled {
    /// The entry as a fresh [`Compiled`] (the hit's return value): the
    /// program copies, the reports clone out of the shared slice — an
    /// allocation only when the string actually carried diagnostics
    /// (the common valid-string hit is allocation-free).
    fn to_compiled(&self) -> Compiled {
        Compiled {
            program: self.program,
            diagnostics: self.diagnostics.to_vec(),
        }
    }

    /// The entry from a freshly lowered [`Compiled`] (the miss's stored
    /// value).
    fn from_compiled(identity: DynIdentity, compiled: &Compiled) -> Self {
        Self {
            identity,
            program: compiled.program,
            diagnostics: Arc::from(compiled.diagnostics.clone()),
        }
    }
}

/// The storage seam: keyed slots for interned compile results. The
/// cache is generic over this trait (the custom seam's shape: the fold
/// is generic over [`ClassifyVariant`](crate::ClassifyVariant), the
/// cache over the store), so a host supplies its own boundedness
/// policy — a sharded map, an arena, a test double — without touching
/// the pipeline.
///
/// The key is the class string CONTENT (the caller owns the spelling;
/// the built-in store copies it into its map). The value carries the
/// identity it was produced under; the CACHE does the identity check
/// (the store never interprets it).
pub trait DynStore: Send + Sync {
    /// The entry stored under `key`, if the store holds one.
    fn probe(&self, key: &str) -> Option<CachedCompiled>;

    /// Store (or overwrite) the entry under `key`. Overwrites are the
    /// config flip-flop's mechanism: the same string re-lowered under a
    /// different fingerprint replaces the stale entry.
    fn store(&self, key: String, value: CachedCompiled);

    /// The resident key count (the observability view hosts budget
    /// memory with). A store without a bound reports what it holds.
    fn resident(&self) -> usize;
}

/// The built-in [`DynStore`]: bounded two-generation rotation with
/// promotion — the v0.x `PlanCache` L2's design (plan §3.4.3),
/// transplanted to the V1 seam's interned-`Program` model.
///
/// * `current` serves hits; when it fills to the cap it becomes
///   `previous` and a fresh current starts (the rotation);
/// * a hit in `previous` promotes: the entry moves into `current`, so a
///   working set that recurs within one generation gap stays hot;
/// * a store over an existing current key replaces in place (the config
///   flip-flop); a store of a key that also sits in `previous` clears
///   the stale twin, so a stale fingerprint can never resurface;
/// * the whole map sits behind one `RwLock`: hits in `current` take the
///   READ lock only (the v0.x F2 lesson — "resolve_static still takes
///   the exclusive lock on a hit" — is the regression this seam refuses
///   to repeat); promotions and rotations take the write lock.
///
/// Hashing is std's `RandomState` (SipHash): the V1 graph stays
/// dependency-light (the v0.x `FastHasher` stays `aeolus-core`'s), and
/// the probe cost is noise against the pipeline pass the hit skips.
#[derive(Debug)]
pub struct TwoGenStore {
    cap: usize,
    inner: RwLock<TwoGenInner>,
}

#[derive(Debug)]
struct TwoGenInner {
    current: HashMap<Box<str>, CachedCompiled>,
    previous: HashMap<Box<str>, CachedCompiled>,
    hasher: RandomState,
}

impl TwoGenStore {
    /// The store at the default cap ([`DEFAULT_DYN_CACHE_CAP`]).
    pub fn new() -> Self {
        Self::with_cap(DEFAULT_DYN_CACHE_CAP)
    }

    /// The store at a custom cap (the current generation's bound; the
    /// resident key set tops out at 2×cap with the previous generation).
    ///
    /// # Panics
    /// `cap == 0` (an unbounded-then-rotate map with no slots would
    /// churn every generation; the caller means `cap >= 1`).
    pub fn with_cap(cap: usize) -> Self {
        assert!(cap >= 1, "TwoGenStore::with_cap needs at least one slot");
        Self {
            cap,
            inner: RwLock::new(TwoGenInner {
                current: HashMap::with_capacity_and_hasher(cap, RandomState::new()),
                previous: HashMap::default(),
                hasher: RandomState::new(),
            }),
        }
    }
}

impl Default for TwoGenStore {
    fn default() -> Self {
        Self::new()
    }
}

impl DynStore for TwoGenStore {
    fn probe(&self, key: &str) -> Option<CachedCompiled> {
        // Fast path: the read lock serves every current-generation hit.
        if let Some(entry) = self
            .inner
            .read()
            .ok()
            .and_then(|inner| inner.current.get(key).cloned())
        {
            return Some(entry);
        }
        // Promotion path: a previous-generation hit needs the write
        // lock (and a re-check — another thread may have promoted or
        // rotated between the drop and the re-acquire).
        let mut inner = self.inner.write().ok()?;
        if let Some(entry) = inner.current.get(key).cloned() {
            return Some(entry);
        }
        let entry = inner.previous.remove(key)?;
        inner.current.insert(key.into(), entry.clone());
        Some(entry)
    }

    fn store(&self, key: String, value: CachedCompiled) {
        let Ok(mut inner) = self.inner.write() else {
            return;
        };
        if inner.current.contains_key(key.as_str()) {
            // Replace in place — no rotation on a config flip-flop.
            inner.previous.remove(key.as_str());
            inner.current.insert(key.into(), value);
            return;
        }
        if inner.current.len() >= self.cap {
            // The rotation: current becomes previous, a fresh current
            // starts. The old previous (one generation further back) is
            // dropped from the MAP — its leaked slices stay pooled.
            let current = std::mem::take(&mut inner.current);
            inner.previous = current;
            inner.current =
                HashMap::with_capacity_and_hasher(self.cap, std::mem::take(&mut inner.hasher));
        }
        inner.previous.remove(key.as_str());
        inner.current.insert(key.into(), value);
    }

    fn resident(&self) -> usize {
        self.inner
            .read()
            .map_or(0, |inner| inner.current.len() + inner.previous.len())
    }
}

/// The compile seam's dynamic-string cache: the leak-pool dedupe the
/// crate docs defer to "P2's DynCache". See the module docs for the
/// design; the entry points mirror the free
/// functions and return the SAME [`Compiled`] values (the differential
/// pins miss and hit byte-identical to [`compile`](crate::compile)).
///
/// ```no_run
/// use aeolus_compile::{Config, DynCache};
///
/// let cache = DynCache::new();
/// let config = Config::default();
/// for _ in 0..60 {
///     // The UI-loop shape: the same dynamic string every frame —
///     // one miss, then hits that hand back the same interned slices.
///     let compiled = cache.compile("p-4 hover:bg-primary/90", &config)?;
///     let _ = compiled.program.layers.len();
/// }
/// assert!(cache.hits() >= 59);
/// # Ok::<(), aeolus_compile::CompileError>(())
/// ```
pub struct DynCache<S: DynStore = TwoGenStore> {
    store: S,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl DynCache<TwoGenStore> {
    /// The cache at the built-in store's default cap.
    pub fn new() -> Self {
        Self::with_store(TwoGenStore::new())
    }

    /// The cache with a custom generation cap (see
    /// [`TwoGenStore::with_cap`]).
    pub fn with_cap(cap: usize) -> Self {
        Self::with_store(TwoGenStore::with_cap(cap))
    }
}

impl<S: DynStore> DynCache<S> {
    /// The cache over a host-supplied store — the seam's constructor
    /// (the tests wire a counting double through it).
    pub fn with_store(store: S) -> Self {
        Self {
            store,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    /// [`compile`](crate::compile) over the cache: a hit hands back the
    /// same interned slices the miss pooled; a miss lowers through the
    /// plain pipeline and stores. Errors are never cached (the empty
    /// input is a caller bug — see the module docs).
    ///
    /// The cache key is the CONFIGURATION FINGERPRINT composed with the
    /// class string — never a hash of either (the v0.x collision-safety
    /// rule): a string lowered under two configs owns two resident
    /// entries, each handing back its own interned slices.
    ///
    /// # Errors
    /// The same [`CompileError`] as [`compile`](crate::compile) (only
    /// the empty input), reported on every call.
    pub fn compile(&self, input: &str, config: &Config) -> Result<Compiled, CompileError> {
        self.cached(input, DynIdentity::of(*config), || {
            crate::run(input, *config, None, &Customs::new())
        })
    }

    /// [`compile_theme`](crate::compile_theme) over the cache: the theme
    /// table joins the identity (one slot per session's table), the
    /// lowering is the same themed pipeline. A hit hands back the same
    /// interned slices the miss pooled.
    ///
    /// # Errors
    /// The same [`CompileError`] as [`compile_theme`](crate::compile_theme).
    pub fn compile_theme(
        &self,
        input: &str,
        tokens: &TokenTable,
        config: &Config,
    ) -> Result<Compiled, CompileError> {
        let identity = DynIdentity::themed(*config, tokens);
        self.cached(input, identity, || {
            crate::run(input, *config, Some(tokens), &Customs::new())
        })
    }

    /// [`compile_with`](crate::compile_with) over the cache: the
    /// registrations join the identity (one slot per session's bag; an
    /// EMPTY bag keys as zero — its byte-identity to the plain entry
    /// points is the custom seam's differential contract).
    ///
    /// # Errors
    /// The same [`CompileError`] as [`compile_with`](crate::compile_with).
    pub fn compile_with(
        &self,
        input: &str,
        customs: &Customs,
        config: &Config,
    ) -> Result<Compiled, CompileError> {
        let identity = DynIdentity::custom(*config, customs);
        self.cached(input, identity, || {
            crate::run(input, *config, None, customs)
        })
    }

    /// [`compile_theme_with`](crate::compile_theme_with) over the cache:
    /// the theme table and the registrations both join the identity —
    /// the full session's slot.
    ///
    /// # Errors
    /// The same [`CompileError`] as
    /// [`compile_theme_with`](crate::compile_theme_with).
    pub fn compile_theme_with(
        &self,
        input: &str,
        tokens: &TokenTable,
        customs: &Customs,
        config: &Config,
    ) -> Result<Compiled, CompileError> {
        let identity = DynIdentity::themed_custom(*config, tokens, customs);
        self.cached(input, identity, || {
            crate::run(input, *config, Some(tokens), customs)
        })
    }

    /// The one cached path every entry point rides: probe (identity in
    /// the key), lower on a miss, store on a success. Errors propagate
    /// uncounted-as-entries and never stored.
    fn cached(
        &self,
        input: &str,
        identity: DynIdentity,
        lower: impl FnOnce() -> Result<Compiled, CompileError>,
    ) -> Result<Compiled, CompileError> {
        if let Some(cached) = self
            .store
            .probe(&format!("{}{input}", identity.key_prefix()))
            && cached.identity == identity
        {
            #[cfg(feature = "tracing")]
            tracing::debug!(input_len = input.len(), "dyn cache: hit");
            self.hits.fetch_add(1, Ordering::Relaxed);
            return Ok(cached.to_compiled());
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        #[cfg(feature = "tracing")]
        tracing::debug!(input_len = input.len(), "dyn cache: miss");
        let compiled = lower()?;
        #[cfg(feature = "tracing")]
        tracing::debug!(
            layers = compiled.program.layers.len(),
            diagnostics = compiled.diagnostics.len(),
            "dyn cache: store"
        );
        self.store.store(
            format!("{}{input}", identity.key_prefix()),
            CachedCompiled::from_compiled(identity, &compiled),
        );
        Ok(compiled)
    }

    /// The hit count since construction (the observability view the
    /// example and the tests assert on).
    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    /// The miss count since construction.
    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    /// The resident key count (the store's view — at most 2×cap on the
    /// built-in store).
    pub fn resident(&self) -> usize {
        self.store.resident()
    }
}

impl<S: DynStore + Default> Default for DynCache<S> {
    fn default() -> Self {
        Self::with_store(S::default())
    }
}

impl<S: DynStore> std::fmt::Debug for DynCache<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynCache")
            .field("hits", &self.hits())
            .field("misses", &self.misses())
            .field("resident", &self.resident())
            .finish()
    }
}
