//! The compile-walk observability seam: the walk's own stage
//! boundaries as a default-no-op observer trait, over the SAME
//! pipeline every entry point drives — the dyn-cache face
//! ([`crate::DynObserver`]) pointed at the walk instead of the pool.
//!
//! ```text
//! compile_observed(input, …, observer)
//!   └─ run(…)                  — the ONE walk (lex → fold → read → sort)
//!        ├─ on_lexed           — the lexer's stage totals
//!        ├─ on_candidate ×k    — the per-candidate fold/read verdict
//!        └─ on_assembled       — the specificity sort's layer count
//! ```
//!
//! The house rules the cache face set, kept here: the trait's methods
//! are all no-ops by default (the seam adds a channel, never replaces
//! one — the `tracing` spans and the artifact's own diagnostics keep
//! answering); the built-in [`NoWalkObserver`] is the every-constructor
//! default; an `Arc<T>` of any observer IS an observer (one host
//! handle totals several walks); and the seam is byte-NEUTRAL — the
//! observed walk's [`Compiled`](crate::Compiled) is the plain
//! compile's artifact, which the differential pins. The events report
//! decisions the walk already made (the stage totals, the typed
//! per-candidate outcome), never opinions: a host's telemetry reads
//! the pipeline's own truth.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// One candidate's walk verdict — the fold/read stages' own outcome,
/// typed so a host never parses prose to count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidateOutcome {
    /// The candidate lowered at least one declaration (its condition
    /// group joined the program): `_decls` the declarations it
    /// produced, `_diagnostics` the typed reports it also pushed (the
    /// sibling-row boundaries — a candidate can lower AND report).
    Lowered { decls: usize, diagnostics: usize },
    /// The candidate produced no declaration — every miss reported
    /// typed (the fold miss, the modifier gate, the unknown root, the
    /// value pass's boundary); `_diagnostics` the reports it pushed.
    Missed { diagnostics: usize },
}

/// The compile-walk observer: the stage-boundary questions the walk
/// answers as it assembles one string's program. Every method is a
/// no-op default — implement the stages a host reads.
pub trait WalkObserver: Send + Sync + 'static {
    /// The lexer's stage totals: `_candidates` the tokens that lexed
    /// (the per-candidate events follow, in walk order), `_degraded`
    /// the typed per-token failures (they ride the artifact's own
    /// diagnostics and never reach [`WalkObserver::on_candidate`]).
    fn on_lexed(&self, _candidates: usize, _degraded: usize) {}

    /// One candidate's fold/read verdict, in walk order (the lex
    /// stage's own order). The candidate raw is the LEXED spelling
    /// (the variant prefixes ride it — `hover:p-8` is one token).
    fn on_candidate(&self, _raw: &str, _outcome: CandidateOutcome) {}

    /// The assembly stage's answer: `_layers` the specificity-sorted
    /// condition groups the program interned. The walk returns right
    /// after (a caller bug's whole-string failure reports before the
    /// lexer runs and never reaches this channel).
    fn on_assembled(&self, _layers: usize) {}
}

/// The observer's every-constructor default: all channels no-op — the
/// explicit "observe nothing" handle (the byte-neutrality suite's
/// silent twin).
#[derive(Debug, Default)]
pub struct NoWalkObserver;

impl WalkObserver for NoWalkObserver {}

/// Shared-observer forwarding: an `Arc<T>` of any observer IS an
/// observer (the cache face's own blanket — one host handle totals
/// several walks and stays alive beside them).
impl<O: WalkObserver + ?Sized> WalkObserver for Arc<O> {
    fn on_lexed(&self, candidates: usize, degraded: usize) {
        (**self).on_lexed(candidates, degraded);
    }

    fn on_candidate(&self, raw: &str, outcome: CandidateOutcome) {
        (**self).on_candidate(raw, outcome);
    }

    fn on_assembled(&self, layers: usize) {
        (**self).on_assembled(layers);
    }
}

/// The built-in [`WalkObserver`]: the walk's stage totals in one
/// shareable handle — the counts a host's telemetry reads without
/// holding the walk. Sharing rides the [`Arc`] blanket (the cache
/// stats' own shape: one handle totals several walks, the handle
/// alive beside them), the snapshot the `Copy` value the host
/// asserts on.
#[derive(Debug, Default)]
pub struct WalkStats {
    walks: AtomicU64,
    candidates: AtomicU64,
    degraded: AtomicU64,
    lowered: AtomicU64,
    missed: AtomicU64,
    decls: AtomicU64,
    diagnostics: AtomicU64,
    layers: AtomicU64,
}

impl WalkStats {
    /// The empty session.
    pub fn new() -> Self {
        Self::default()
    }

    /// The session's totals so far — the `Copy` value the host's own
    /// telemetry or test asserts on.
    pub fn snapshot(&self) -> WalkSnapshot {
        let load = |cell: &AtomicU64| cell.load(Ordering::Relaxed);
        WalkSnapshot {
            walks: load(&self.walks),
            candidates: load(&self.candidates),
            degraded: load(&self.degraded),
            lowered: load(&self.lowered),
            missed: load(&self.missed),
            decls: load(&self.decls),
            diagnostics: load(&self.diagnostics),
            layers: load(&self.layers),
        }
    }
}

impl WalkObserver for WalkStats {
    fn on_lexed(&self, candidates: usize, degraded: usize) {
        self.walks.fetch_add(1, Ordering::Relaxed);
        self.candidates
            .fetch_add(candidates as u64, Ordering::Relaxed);
        self.degraded.fetch_add(degraded as u64, Ordering::Relaxed);
        // The degraded failures ARE the artifact's diagnostics (the Lex
        // kind) — the counter answers the artifact's total, degraded
        // included, so a host reads one number for one truth.
        self.diagnostics
            .fetch_add(degraded as u64, Ordering::Relaxed);
    }

    fn on_candidate(&self, _raw: &str, outcome: CandidateOutcome) {
        match outcome {
            CandidateOutcome::Lowered { decls, diagnostics } => {
                self.lowered.fetch_add(1, Ordering::Relaxed);
                self.decls.fetch_add(decls as u64, Ordering::Relaxed);
                self.diagnostics
                    .fetch_add(diagnostics as u64, Ordering::Relaxed);
            }
            CandidateOutcome::Missed { diagnostics } => {
                self.missed.fetch_add(1, Ordering::Relaxed);
                self.diagnostics
                    .fetch_add(diagnostics as u64, Ordering::Relaxed);
            }
        }
    }

    fn on_assembled(&self, layers: usize) {
        self.layers.fetch_add(layers as u64, Ordering::Relaxed);
    }
}

/// One observer session's totals: a `Copy` value the host holds
/// without holding the observer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct WalkSnapshot {
    /// The walks the session observed (one per string).
    pub walks: u64,
    /// The tokens that lexed, across every walk.
    pub candidates: u64,
    /// The typed per-token lex failures, across every walk.
    pub degraded: u64,
    /// The candidates that lowered at least one declaration.
    pub lowered: u64,
    /// The candidates that produced no declaration.
    pub missed: u64,
    /// The declarations the walk produced, across every walk.
    pub decls: u64,
    /// The typed reports the walk pushed, across every walk (the
    /// degraded lex failures included).
    pub diagnostics: u64,
    /// The condition layers the walks interned, across every walk.
    pub layers: u64,
}
