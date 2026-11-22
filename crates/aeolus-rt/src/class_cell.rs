//! The dynamic-strings slow lane (architecture plan §3.6's surface (3),
//! the third user-facing surface): the per-view, UI-thread, single-slot
//! cell behind "store in view state; compiled when the string changes,
//! a hash compare per frame otherwise".
//!
//! ```text
//! ClassCell.get("p-4 ...")   ─ content+identity match ─▶ the SAME
//!                            │                          interned program, no work
//!                            └ a change ─▶ the full V1 compile (the
//!                              SAME compiler as `tw!` — no smaller
//!                              matcher) ─▶ the program + the honest
//!                              diagnostics, interned for the next frame
//! ```
//!
//! The design is the plan's sketch with the landed IR's storage model:
//! the sketch's `hash: u64` fast path keys on EXACT CONTENT plus the
//! compile identity (config, table/bag address — the v0.x
//! collision-safety rule the `DynCache` docs carry, never hash-only,
//! and the same `(ptr, len)`-precedent identity the cache keys), and
//! the cached value is the interned [`StaticProgram`] the compile
//! stages already leak into the pool — the macro's frozen form,
//! runtime-built. The keyword texts ride
//! [`aeolus_compile::keyword_alignment`], the seam the macro freeze and
//! the codegen target walk too.
//!
//! The cell is deliberately NOT the bounded pool — that is the host's
//! `DynCache` (thread-safe, two-generation rotation). The cell is the
//! one slot a view owns: a string flip-flop (`A → B → A`) recompiles,
//! and the recompiles dedupe nothing beyond the single slot. The UI
//! thread renders sequentially per window, so the [`RefCell`] interior
//! is the only synchronization — no [`RwLock`], no atomics, the plan's
//! own note — and the `PhantomData<Rc<()>>` marker makes the type
//! `!Sync` so the contract fails to compile instead of waiting for a
//! race.
//!
//! The observability face is the POOL's own seam joined: a
//! host-supplied [`DynObserver`] wired through
//! [`ClassCell::with_observer`] sees every decision the cell makes in
//! order — hit/miss/store, under the composed session key the
//! [`DynStore`](aeolus_compile::DynStore) vocabulary already probes
//! ([`session_key`]: the identity prefix, the input verbatim — never a
//! hash, the collision-safety rule) — so one observer stream reads the
//! bounded pool AND every view cell the app wires it into. The
//! every-constructor default is the observer-free
//! [`NoObserver`](aeolus_compile::NoObserver): the observer adds a
//! channel, it never changes a decision.

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

use aeolus_compile::{
    ClassKeywords, CompileError, Compiled, Config, Customs, Diagnostic, DynObserver, NoObserver,
    compile, compile_theme, compile_theme_with, compile_with, keyword_alignment,
    keyword_texts_with, session_key,
};
use aeolus_ir::Layer;
use aeolus_theme::TokenTable;

use crate::StaticProgram;

static EMPTY_LAYERS: [Layer; 0] = [];
static EMPTY_KEYWORDS: [&str; 0] = [];

/// The empty program a fresh cell renders — no layers, no row texts:
/// a view that never compiled still lays out.
fn empty_program() -> StaticProgram {
    StaticProgram::new(&EMPTY_LAYERS, &EMPTY_KEYWORDS)
}

/// The slot key: the exact config and the table/bag identities — the
/// same shape the compile cache's private `DynIdentity` keys (the v0.x
/// `(ptr, len)` precedent: one table and one bag per session, so the
/// ADDRESS is the identity; an EMPTY bag keys as zero like its
/// absence, the custom seam's byte-identity contract).
#[derive(Debug, PartialEq)]
struct SlotIdentity {
    config: Config,
    tokens: usize,
    customs: usize,
}

impl SlotIdentity {
    fn themeless(config: Config) -> Self {
        Self {
            config,
            tokens: 0,
            customs: 0,
        }
    }

    fn theme(config: Config, tokens: &TokenTable) -> Self {
        Self {
            config,
            tokens: std::ptr::from_ref(tokens).addr(),
            customs: 0,
        }
    }

    fn custom(config: Config, customs: &Customs) -> Self {
        Self {
            config,
            tokens: 0,
            customs: bag_identity(customs),
        }
    }

    fn theme_custom(config: Config, tokens: &TokenTable, customs: &Customs) -> Self {
        Self {
            config,
            tokens: std::ptr::from_ref(tokens).addr(),
            customs: bag_identity(customs),
        }
    }
}

/// The bag's identity address — zero for the EMPTY bag (its
/// byte-identity to the plain entry points is the custom seam's
/// differential contract).
fn bag_identity(customs: &Customs) -> usize {
    if customs.is_empty() {
        0
    } else {
        std::ptr::from_ref(customs).addr()
    }
}

/// The one slot: what the cell's last successful compile left behind.
struct Slot {
    identity: SlotIdentity,
    /// The content key — the class string VERBATIM (the exact-equality
    /// match the per-frame path applies).
    input: Box<str>,
    program: StaticProgram,
    diagnostics: Vec<Diagnostic>,
    /// The composed session key the store decision reported, cached at
    /// swap-in (a pure function of identity + input): the warm path's
    /// hit decision re-reports it without RE-FORMATTING the string —
    /// the §4 warm target's own saving (the key format was the hit
    /// path's dominant cost).
    key: Box<str>,
}

/// The dynamic-strings slow lane's view-state cell (plan §3.6's
/// surface (3)): ONE slot keyed on the class string's exact content
/// plus the compile identity — "compiled when the string changes, a
/// hash compare per frame otherwise".
///
/// A hit hands back the SAME interned program the last compile left
/// (pointer-stable, the compile cache's own contract, zero compile work — the
/// counters prove it); a miss runs the FULL V1 compile and interns the
/// aligned program; an error (only the empty input) leaves the last
/// good program standing and is never cached. Per-class misses ride
/// the typed [`Diagnostic`] channel (`diagnostics`), never a silent
/// drop — the same coded diagnostics the compile walk reports.
///
/// The cell is `!Sync` (`!Send` too) BY DESIGN: the UI thread renders
/// sequentially per window (plan §3.5), so a `&self` render path with
/// interior mutability is the whole contract — no locks, no atomics.
///
/// ```no_run
/// use aeolus_compile::Config;
/// use aeolus_rt::ClassCell;
///
/// struct Card {
///     user_classes: String,
///     classes: ClassCell,
/// }
///
/// impl Card {
///     fn render(&self) -> String {
///         // Per frame: a content match costs a compare, a change
///         // compiles once. The last good program stands on an error.
///         match self.classes.get(&self.user_classes, &Config::default()) {
///             Ok(program) => format!("{} layer(s)", program.layers.len()),
///             Err(_) => "no classes this frame".to_string(),
///         }
///     }
/// }
/// ```
pub struct ClassCell {
    slot: RefCell<Slot>,
    /// The wired observer — the pool's own seam (`DynObserver`),
    /// `NoObserver` the every-constructor default. `Send + Sync` is
    /// the trait's own bound (the built-in [`CacheStats`] shape): the
    /// CELL stays `!Sync` while the handle may be shared across the
    /// app's cells.
    observer: Arc<dyn DynObserver>,
    hits: std::cell::Cell<u64>,
    misses: std::cell::Cell<u64>,
    // The UI-thread contract (plan §3.6's "UI-thread, !Sync, no
    // RwLock/Arc atomics"): the marker is the type-system form of the
    // note — sharing across threads fails to compile.
    _thread: PhantomData<Rc<()>>,
}

impl Default for ClassCell {
    fn default() -> Self {
        Self::new()
    }
}

impl ClassCell {
    /// The fresh cell: the empty program, no diagnostics, empty
    /// counters, observation-free (the [`NoObserver`] default — every
    /// constructor path starts quiet, the cache's own contract).
    pub fn new() -> Self {
        Self {
            slot: RefCell::new(Slot {
                identity: SlotIdentity::themeless(Config::default()),
                input: String::new().into_boxed_str(),
                program: empty_program(),
                diagnostics: Vec::new(),
                key: String::new().into_boxed_str(),
            }),
            observer: Arc::new(NoObserver),
            hits: std::cell::Cell::new(0),
            misses: std::cell::Cell::new(0),
            _thread: PhantomData,
        }
    }

    /// The cell with a host-supplied [`DynObserver`] — the bounded
    /// pool's observability seam joined, chained after `new` (the
    /// `DynCache::with_observer` shape: the built-in
    /// [`CacheStats`](aeolus_compile::CacheStats) rides an `Arc` the
    /// host keeps a handle to, ONE handle spanning the pool and every
    /// view cell the app wires it into). The observer adds a channel;
    /// it never changes a decision — the differential pins every
    /// contract byte-identical with and without one.
    pub fn with_observer(mut self, observer: impl DynObserver + 'static) -> Self {
        self.observer = Arc::new(observer);
        self
    }

    /// [`compile`](aeolus_compile::compile) over the cell: the slow lane's
    /// themeless entrance.
    ///
    /// # Errors
    /// The same [`CompileError`] as [`compile`](aeolus_compile::compile) (only
    /// the empty input). The last good program stands and the counters
    /// are unchanged beyond the miss the probe cost — errors are never
    /// cached.
    pub fn get(&self, classes: &str, config: &Config) -> Result<StaticProgram, CompileError> {
        let identity = SlotIdentity::themeless(*config);
        let key = session_key(*config, None, &Customs::new(), classes);
        self.swap(identity, key, classes, &Customs::new(), compile)
    }

    /// [`compile_theme`](aeolus_compile::compile_theme) over the cell: the
    /// runtime token table joins the identity (one slot per session's
    /// table).
    ///
    /// # Errors
    /// The same contract as [`ClassCell::get`].
    pub fn get_theme(
        &self,
        classes: &str,
        tokens: &TokenTable,
        config: &Config,
    ) -> Result<StaticProgram, CompileError> {
        let identity = SlotIdentity::theme(*config, tokens);
        let key = session_key(*config, Some(tokens), &Customs::new(), classes);
        self.swap(identity, key, classes, &Customs::new(), |input, config| {
            compile_theme(input, tokens, config)
        })
    }

    /// [`compile_with`](aeolus_compile::compile_with) over the cell: the
    /// registered customs join the identity (an EMPTY bag keys as zero
    /// like its absence).
    ///
    /// # Errors
    /// The same contract as [`ClassCell::get`].
    pub fn get_with(
        &self,
        classes: &str,
        customs: &Customs,
        config: &Config,
    ) -> Result<StaticProgram, CompileError> {
        let identity = SlotIdentity::custom(*config, customs);
        let key = session_key(*config, None, customs, classes);
        self.swap(identity, key, classes, customs, |input, config| {
            compile_with(input, customs, config)
        })
    }

    /// [`compile_theme_with`](aeolus_compile::compile_theme_with) over the cell:
    /// the table and the bag both join — the full session's slot, the
    /// whole compile surface cell-shaped.
    ///
    /// # Errors
    /// The same contract as [`ClassCell::get`].
    pub fn get_theme_with(
        &self,
        classes: &str,
        tokens: &TokenTable,
        customs: &Customs,
        config: &Config,
    ) -> Result<StaticProgram, CompileError> {
        let identity = SlotIdentity::theme_custom(*config, tokens, customs);
        let key = session_key(*config, Some(tokens), customs, classes);
        self.swap(identity, key, classes, customs, |input, config| {
            compile_theme_with(input, tokens, customs, config)
        })
    }

    /// The last good program — the render path's read side (a fresh
    /// cell hands back the empty program; an errored compile keeps the
    /// previous one standing).
    pub fn program(&self) -> StaticProgram {
        self.slot.borrow().program
    }

    /// The CURRENT string's per-class misses — the same coded
    /// diagnostics the compile walk reports. A clean recompile clears
    /// the channel; an errored compile leaves it untouched (the
    /// diagnostics belong to the program that is standing).
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        self.slot.borrow().diagnostics.clone()
    }

    /// Content-match hits since construction (the per-frame fast path).
    pub fn hits(&self) -> u64 {
        self.hits.get()
    }

    /// Compiles since construction (every identity/content miss,
    /// errored or not — the compile cache's counting contract).
    pub fn misses(&self) -> u64 {
        self.misses.get()
    }

    /// The one path every entry point rides: probe (content + identity
    /// — never hash-only, the collision-safety rule), lower on a miss,
    /// align, intern, store on a success. The composed session key
    /// ([`session_key`]) formats ONCE per call and rides the observer's
    /// events — the SAME vocabulary the pool's [`DynStore`](aeolus_compile::DynStore)
    /// probes, so one stream reads the whole app. The alignment reads
    /// the SAME customs the compile ran (`keyword_texts_with`) — an
    /// applied body's registered keyword texts reach the frozen slice,
    /// and the theme-less entries' empty bag stays byte-identical to
    /// the old theme-less call.
    ///
    /// The decisions ride the wired observer IN ORDER (the cache's own
    /// contract): hit on a probe match; miss before the lowering runs;
    /// store on a success, carrying the entry's honest per-class
    /// diagnostic count. An error reaches `on_miss` and never
    /// `on_store` — errors are never stored.
    fn swap(
        &self,
        identity: SlotIdentity,
        key: String,
        classes: &str,
        customs: &Customs,
        lower: impl FnOnce(&str, &Config) -> Result<Compiled, CompileError>,
    ) -> Result<StaticProgram, CompileError> {
        let mut slot = self.slot.borrow_mut();
        // The warm path: identity + content match, the cached key
        // re-reported (the store decision's own string) — the hit path
        // formats nothing (the §4 warm target's own saving).
        if slot.identity == identity && slot.input.as_ref() == classes {
            self.observer.on_hit(&slot.key);
            self.hits.set(self.hits.get() + 1);
            return Ok(slot.program);
        }
        self.misses.set(self.misses.get() + 1);
        self.observer.on_miss(&key);
        let compiled = lower(classes, &identity.config)?;
        let annotations: Vec<ClassKeywords> =
            keyword_texts_with(classes, customs, &identity.config)?;
        let texts = keyword_alignment(&compiled, &annotations);
        let program = StaticProgram::new(
            compiled.program.layers,
            // The interned form: the compile stages leak the layers; the
            // aligned row texts leak beside them (the same documented
            // storage — one distinct string, one pool entry).
            Box::leak(texts.into_boxed_slice()),
        );
        self.observer.on_store(&key, compiled.diagnostics.len());
        *slot = Slot {
            identity,
            input: classes.into(),
            program,
            diagnostics: compiled.diagnostics,
            key: key.into(),
        };
        Ok(program)
    }
}
