//! The theme-resolution observability seam: the token table's own
//! resolution answers as a default-no-op observer trait, over the SAME
//! `resolve`/`resolve_rgba` primitives every consumer drives — the
//! compile-walk face ([`WalkObserver`](https://docs.rs/aeolus-compile)
//! over in the compile crate) pointed at the theme stage instead of
//! the walk.
//!
//! ```text
//! TokenTable::with_observer(stats)
//!   └─ resolve(id, alpha)            — the tier arm's answer
//!   │    └─ on_resolve               — ResolveOutcome (the arm, or the typed miss)
//!   └─ resolve_rgba(id, alpha, dark) — the runtime path's mode answer
//!        └─ on_resolve_rgba          — RgbaOutcome (the mode half, or the typed miss)
//! ```
//!
//! The house rules the compile crate's two faces set, kept here: the
//! trait's methods are all no-ops by default (the seam adds a channel,
//! never replaces one — the `tracing` spans and the [`ResolveError`]
//! the calls return keep answering); the built-in
//! [`NoThemeObserver`] is the every-constructor default (every table
//! constructor starts observation-free, and
//! [`TokenTable::with_observer`](crate::TokenTable::with_observer) wires one after construction); an
//! `Arc<T>` of any observer IS an observer (one host handle totals
//! several tables); and the seam is answer-NEUTRAL — the observed
//! table's answers are the plain table's, which the differential
//! pins. The events report decisions the table already made (the
//! tier arm that answered, the typed refusal verbatim), never
//! opinions: a host's telemetry reads the theme stage's own truth.
//!
//! The channel rides the table itself, so the events reach every
//! caller without a single call-site change upstream: the compile
//! walk's lowering consults `resolve(id, alpha)` per colour
//! declaration, so a table wired before
//! [`compile_theme`](https://docs.rs/aeolus-compile) streams the
//! theme stage's per-declaration verdicts during the walk — the stage
//! boundary the walk face's lex/candidate/assemble events sit above.
//! Events stream in caller order, after the answer is computed and
//! before the call returns; a miss IS an event (the typed refusal
//! rides `ResolveOutcome::Missed`), and a refused resolve never
//! counts into a value arm.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::ResolveError;

/// One [`TokenTable::resolve`](crate::TokenTable::resolve) verdict — the tier arm that answered,
/// typed so a host never parses prose to count. The payload mirrors
/// the call's own `Result`: the `Ok` arms name the value form, the
/// `Err` arm carries the typed refusal verbatim.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResolveOutcome {
    /// The `Runtime` tier's carry: [`Val::Tok`](aeolus_ir::Val::Tok)
    /// `{ id, alpha }` — no colour math ran (the host multiplies at
    /// theme-resolution time).
    TokCarried,
    /// The `Baked` tier's light literal: [`Val::Rgba`](aeolus_ir::Val::Rgba), the
    /// premultiplied bake when an alpha rode
    /// (`Color::mix_with_transparent`, the audited `/N` identity).
    Literal,
    /// The `Baked` tier's `(light, dark)` const pair:
    /// [`Val::ModePair`](aeolus_ir::Val::ModePair), the slot into `TokenTable::pair_at`.
    ModePair,
    /// The typed refusal, verbatim — the same [`ResolveError`] the
    /// call returns (`UnknownId`, `AlphaOnPair` or `AlphaNotFinite`).
    Missed(ResolveError),
}

/// One [`TokenTable::resolve_rgba`](crate::TokenTable::resolve_rgba) verdict — the mode answer, typed
/// the same way. The `dark` flag is the caller's request; the outcome
/// names the arm that actually answered it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RgbaOutcome {
    /// The `Runtime` tier's active half: the `dark` flag selected the
    /// loaded colour, the alpha premultiplied (one load, one multiply
    /// — plan §3.4's T2 cost).
    ModeHalf,
    /// The `Baked` tier's light literal: the light-only token's
    /// premultiplied bake (the `dark` flag had nothing to select — a
    /// light-only source serves both modes).
    Literal,
    /// The typed refusal, verbatim (`AlphaNotFinite`, `UnknownId` or
    /// `AlphaOnPair` — the pair form's no-alpha-arm boundary).
    Missed(ResolveError),
}

/// The theme-resolution observer: the resolution questions the table
/// answers as hosts and the compile walk consult it. Every method is
/// a no-op default — implement the channels a host reads.
pub trait ThemeObserver: Send + Sync + 'static {
    /// One [`TokenTable::resolve`](crate::TokenTable::resolve) verdict, in caller order (the
    /// lowering's own per-declaration order when the compile walk
    /// rides a wired table): `_id` the token id, `_alpha` the opacity
    /// modifier as the caller passed it, `_outcome` the tier arm that
    /// answered or the typed refusal. A miss IS an event — the
    /// unresolved consult is a theme-stage decision the host reads.
    fn on_resolve(&self, _id: u16, _alpha: Option<f32>, _outcome: ResolveOutcome) {}

    /// One [`TokenTable::resolve_rgba`](crate::TokenTable::resolve_rgba) verdict, in caller order:
    /// `_id` the token id, `_alpha` the modifier, `_dark` the mode
    /// request, `_outcome` the arm that answered (`ModeHalf` — the
    /// flag selected; `Literal` — a light-only token serves both
    /// modes) or the typed refusal.
    fn on_resolve_rgba(&self, _id: u16, _alpha: f32, _dark: bool, _outcome: RgbaOutcome) {}
}

/// The observer's every-constructor default: all channels no-op — the
/// explicit "observe nothing" handle (the answer-neutrality suite's
/// silent twin).
#[derive(Debug, Default)]
pub struct NoThemeObserver;

impl ThemeObserver for NoThemeObserver {}

/// Shared-observer forwarding: an `Arc<T>` of any observer IS an
/// observer (the compile crate's faces' own blanket — one host handle
/// totals several tables and stays alive beside them).
impl<O: ThemeObserver + ?Sized> ThemeObserver for Arc<O> {
    fn on_resolve(&self, id: u16, alpha: Option<f32>, outcome: ResolveOutcome) {
        (**self).on_resolve(id, alpha, outcome);
    }

    fn on_resolve_rgba(&self, id: u16, alpha: f32, dark: bool, outcome: RgbaOutcome) {
        (**self).on_resolve_rgba(id, alpha, dark, outcome);
    }
}

/// The built-in [`ThemeObserver`]: the theme stage's resolution totals
/// in one shareable handle — the counts a host's telemetry reads
/// without holding the table. Sharing rides the [`Arc`] blanket (one
/// handle totals every table wired with it — the compile walk's table
/// and a host's own runtime resolves alike), the snapshot the `Copy`
/// value the host asserts on.
#[derive(Debug, Default)]
pub struct ThemeStats {
    resolves: AtomicU64,
    carried: AtomicU64,
    literals: AtomicU64,
    pairs: AtomicU64,
    misses: AtomicU64,
    rgba_resolves: AtomicU64,
    rgba_literals: AtomicU64,
    rgba_halves: AtomicU64,
    rgba_misses: AtomicU64,
}

impl ThemeStats {
    /// The empty session.
    pub fn new() -> Self {
        Self::default()
    }

    /// The session's totals so far — the `Copy` value the host's own
    /// telemetry or test asserts on.
    pub fn snapshot(&self) -> ThemeSnapshot {
        let load = |cell: &AtomicU64| cell.load(Ordering::Relaxed);
        ThemeSnapshot {
            resolves: load(&self.resolves),
            carried: load(&self.carried),
            literals: load(&self.literals),
            pairs: load(&self.pairs),
            misses: load(&self.misses),
            rgba_resolves: load(&self.rgba_resolves),
            rgba_literals: load(&self.rgba_literals),
            rgba_halves: load(&self.rgba_halves),
            rgba_misses: load(&self.rgba_misses),
        }
    }
}

impl ThemeObserver for ThemeStats {
    fn on_resolve(&self, _id: u16, _alpha: Option<f32>, outcome: ResolveOutcome) {
        self.resolves.fetch_add(1, Ordering::Relaxed);
        match outcome {
            ResolveOutcome::TokCarried => self.carried.fetch_add(1, Ordering::Relaxed),
            ResolveOutcome::Literal => self.literals.fetch_add(1, Ordering::Relaxed),
            ResolveOutcome::ModePair => self.pairs.fetch_add(1, Ordering::Relaxed),
            ResolveOutcome::Missed(_) => self.misses.fetch_add(1, Ordering::Relaxed),
        };
    }

    fn on_resolve_rgba(&self, _id: u16, _alpha: f32, _dark: bool, outcome: RgbaOutcome) {
        self.rgba_resolves.fetch_add(1, Ordering::Relaxed);
        match outcome {
            RgbaOutcome::ModeHalf => self.rgba_halves.fetch_add(1, Ordering::Relaxed),
            RgbaOutcome::Literal => self.rgba_literals.fetch_add(1, Ordering::Relaxed),
            RgbaOutcome::Missed(_) => self.rgba_misses.fetch_add(1, Ordering::Relaxed),
        };
    }
}

/// One observer session's totals: a `Copy` value the host holds
/// without holding the observer. The two channels total separately —
/// the tier arm's split (the compile walk's lowering answers) and the
/// rgba channel's own (the runtime path's mode answers).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ThemeSnapshot {
    /// The [`TokenTable::resolve`](crate::TokenTable::resolve) consults, across every wired table.
    pub resolves: u64,
    /// The consults the `Runtime` tier answered by carrying
    /// (`Val::Tok` — no math ran).
    pub carried: u64,
    /// The consults the `Baked` tier answered with a light literal
    /// (with or without a premultiplied alpha).
    pub literals: u64,
    /// The consults the `Baked` tier answered with a `(light, dark)`
    /// const pair.
    pub pairs: u64,
    /// The tier consults that refused typed (the misses; the payload
    /// rides the event, the count answers "how often").
    pub misses: u64,
    /// The [`TokenTable::resolve_rgba`](crate::TokenTable::resolve_rgba) consults, across every wired
    /// table.
    pub rgba_resolves: u64,
    /// The rgba consults a light-only baked literal answered (the
    /// `dark` flag had nothing to select).
    pub rgba_literals: u64,
    /// The rgba consults the `Runtime` tier's active half answered
    /// (the mode flag selected).
    pub rgba_halves: u64,
    /// The rgba consults that refused typed.
    pub rgba_misses: u64,
}
