//! The theme-resolution observability face's contract tests (TDD: written
//! FIRST — the RED capture is the "no `ThemeObserver` trait / no
//! `TokenTable::with_observer` entry" compile failure, recorded in
//! `reports/10-tdd-evidence/42-theme-observer.md`) — the same battery
//! shape the compile crate's `walk_observer.rs` / `dyn_observer.rs`
//! suites pin for their faces.
//!
//! The pins, in the order the module docs promise them:
//!
//! * the defaults stay observation-free (every constructor starts with
//!   no observer; the built-in [`aeolus_theme::NoThemeObserver`] wired
//!   explicitly stays silent too);
//! * the built-in [`aeolus_theme::ThemeStats`] collector totals the
//!   resolution arms (the tier carry, the literals, the mode pairs,
//!   the typed misses — and the rgba channel's own split);
//! * the per-resolve events stream in caller order with the typed
//!   outcome (`ResolveOutcome` / `RgbaOutcome` — a host never parses
//!   prose to count);
//! * the misses stream TYPED and verbatim (the same [`ResolveError`]
//!   the call returns — the event mirrors the answer, never a
//!   re-wrapped stringly verdict);
//! * the seam is answer-neutral: the observed table's answers ARE the
//!   plain table's (the differential pin — the face adds a channel,
//!   never changes a decision);
//! * the wiring rides `Clone` (a cloned table streams into the same
//!   handle) and one `Arc<ThemeStats>` spans several tables (the
//!   `Arc<T>` blanket forwards the channels).

use std::sync::{Arc, Mutex};

use aeolus_theme::{
    NoThemeObserver, ResolveError, ResolveOutcome, RgbaOutcome, ThemeObserver, ThemeSource,
    ThemeStats, Tier, TokenTable,
};

fn shadcn() -> ThemeSource {
    ThemeSource::shadcn_dark().expect("embedded palettes parse")
}

fn baked_table() -> TokenTable {
    TokenTable::baked(&shadcn()).expect("default-seeded source builds")
}

fn runtime_table() -> TokenTable {
    TokenTable::runtime(&shadcn()).expect("default-seeded source builds")
}

/// The first token that carries a dark value (the T1 pair shape) —
/// discovered, never hardcoded against a palette edit.
fn first_pair(table: &TokenTable) -> u16 {
    table
        .names()
        .enumerate()
        .find(|(idx, _)| table.has_dark(u16::try_from(*idx).expect("table ids fit u16")))
        .map(|(idx, _)| u16::try_from(idx).expect("table ids fit u16"))
        .expect("the shadcn dark source carries paired tokens")
}

/// The first token that does NOT carry a dark value (the T0 literal
/// shape).
fn first_literal(table: &TokenTable) -> u16 {
    table
        .names()
        .enumerate()
        .find(|(idx, _)| !table.has_dark(u16::try_from(*idx).expect("table ids fit u16")))
        .map(|(idx, _)| u16::try_from(idx).expect("table ids fit u16"))
        .expect("the shadcn source carries light-only tokens")
}

/// A hand-rolled recorder: every event's Debug line, in stream order.
#[derive(Default)]
struct Recorder {
    events: Mutex<Vec<String>>,
}

impl Recorder {
    fn lines(&self) -> Vec<String> {
        self.events
            .lock()
            .expect("the recorder's own lock, never poisoned")
            .clone()
    }
}

impl ThemeObserver for Recorder {
    fn on_resolve(&self, id: u16, alpha: Option<f32>, outcome: ResolveOutcome) {
        self.events
            .lock()
            .expect("the recorder's own lock, never poisoned")
            .push(format!("resolve {id} {alpha:?} {outcome:?}"));
    }

    fn on_resolve_rgba(&self, id: u16, alpha: f32, dark: bool, outcome: RgbaOutcome) {
        self.events
            .lock()
            .expect("the recorder's own lock, never poisoned")
            .push(format!("resolve_rgba {id} {alpha} {dark} {outcome:?}"));
    }
}

/// The every-constructor default holds: a fresh table observes nothing,
/// and the explicit `NoThemeObserver` wired after construction stays a
/// silent no-op — the seam adds a channel, never turns one on by
/// surprise.
#[test]
fn the_defaults_stay_observation_free() {
    let table = baked_table();
    let answer = table.resolve(first_literal(&table), None);
    assert!(
        answer.is_ok(),
        "the plain table answers without any observer wired"
    );

    let table = runtime_table().with_observer(NoThemeObserver);
    let id = table
        .id("primary")
        .expect("the shadcn source names primary");
    let answer = table.resolve(id, None);
    assert!(
        answer.is_ok(),
        "the wired-nothing table still answers — the seam adds a channel, \
         never gates one"
    );

    let stats = Arc::new(ThemeStats::new());
    assert_eq!(
        stats.snapshot(),
        aeolus_theme::ThemeSnapshot::default(),
        "an untouched handle stays the zero session"
    );
}

/// The built-in collector totals the resolution arms: the Runtime
/// tier's carries, the Baked tier's literals and mode pairs, and the
/// typed misses — one number per arm, the snapshot the host asserts on.
#[test]
fn the_collector_totals_the_resolution_arms() {
    let stats = Arc::new(ThemeStats::new());
    let table = runtime_table().with_observer(Arc::clone(&stats));
    let a = table
        .id("primary")
        .expect("the shadcn source names primary");
    let b = first_literal(&table);
    let _ = table.resolve(a, None);
    let _ = table.resolve(b, Some(0.5));
    let snap = stats.snapshot();
    assert_eq!(
        (snap.resolves, snap.carried),
        (2, 2),
        "the runtime tier carries every resolve as Tok"
    );
    assert_eq!(
        (snap.literals, snap.pairs, snap.misses),
        (0, 0, 0),
        "no baked arm fired on a runtime table"
    );

    let stats = Arc::new(ThemeStats::new());
    let table = baked_table().with_observer(Arc::clone(&stats));
    let literal = first_literal(&table);
    let pair = first_pair(&table);
    let _ = table.resolve(literal, None);
    let _ = table.resolve(literal, Some(0.5));
    let _ = table.resolve(pair, None);
    let snap = stats.snapshot();
    assert_eq!(
        (snap.resolves, snap.literals, snap.pairs),
        (3, 2, 1),
        "the baked tier splits literals (with and without alpha) from pairs"
    );
    assert_eq!(
        (snap.carried, snap.misses),
        (0, 0),
        "no runtime arm fired on a baked table"
    );
}

/// The misses stream TYPED and verbatim: the event's payload is the
/// same `ResolveError` the call returns — the event mirrors the answer,
/// never a re-wrapped prose verdict.
#[test]
fn the_misses_stream_typed() {
    let stats = Arc::new(ThemeStats::new());
    let table = baked_table().with_observer(Arc::clone(&stats));
    let pair = first_pair(&table);

    let unknown = table.resolve(9_999, None);
    let alpha_on_pair = table.resolve(pair, Some(0.5));
    let not_finite = table.resolve(first_literal(&table), Some(f32::NAN));

    assert!(
        matches!(unknown, Err(ResolveError::UnknownId(9_999))),
        "the out-of-range id refuses typed"
    );
    assert!(
        matches!(alpha_on_pair, Err(ResolveError::AlphaOnPair)),
        "an alpha on the pair slot refuses typed"
    );
    assert!(
        matches!(not_finite, Err(ResolveError::AlphaNotFinite(_))),
        "a non-finite alpha refuses typed"
    );

    let snap = stats.snapshot();
    assert_eq!(
        (snap.resolves, snap.misses),
        (3, 3),
        "every refusal IS a resolve event with a Missed outcome"
    );
    assert_eq!(
        (snap.literals, snap.pairs, snap.carried),
        (0, 0, 0),
        "a refused resolve never counts into a value arm"
    );
}

/// The per-resolve events stream in CALLER order with the exact typed
/// outcome — a host's telemetry reads the lowering's own sequence.
#[test]
fn the_per_resolve_events_stream_in_caller_order() {
    let recorder = Arc::new(Recorder::default());
    let table = baked_table().with_observer(Arc::clone(&recorder));
    let literal = first_literal(&table);
    let pair = first_pair(&table);

    let _first = table.resolve(literal, None);
    let _second = table.resolve(pair, None);
    let third = table.resolve(9_999, None);

    let lines = recorder.lines();
    assert_eq!(
        lines.len(),
        3,
        "one event per resolve, in caller order: {lines:?}"
    );
    let first_line = &lines[0];
    let second_line = &lines[1];
    let third_line = &lines[2];
    assert!(
        first_line.starts_with(&format!("resolve {literal} None Literal")),
        "the literal arm names its outcome: {first_line}"
    );
    assert!(
        second_line.starts_with(&format!("resolve {pair} None ModePair")),
        "the pair arm names its outcome: {second_line}"
    );
    assert!(
        third_line.starts_with("resolve 9999 None Missed(UnknownId(9999))"),
        "the refusal names its typed payload: {third_line}"
    );
    assert_eq!(third, Err(ResolveError::UnknownId(9_999)));
}

/// The rgba channel reports the mode answer: the Runtime tier's active
/// half (the `dark` flag selects), the Baked tier's light literal, and
/// the typed refusals — the same `ResolveError` the call returns.
#[test]
fn the_rgba_channel_reports_the_mode_answer() {
    let stats = Arc::new(ThemeStats::new());
    let table = runtime_table().with_observer(Arc::clone(&stats));
    let id = table
        .id("primary")
        .expect("the shadcn source names primary");
    let _ = table.resolve_rgba(id, 0.5, true);
    let _ = table.resolve_rgba(id, 0.5, false);
    assert_eq!(
        stats.snapshot(),
        aeolus_theme::ThemeSnapshot {
            resolves: 0,
            carried: 0,
            literals: 0,
            pairs: 0,
            misses: 0,
            rgba_resolves: 2,
            rgba_literals: 0,
            rgba_halves: 2,
            rgba_misses: 0,
        },
        "the runtime tier's active halves total into the rgba channel"
    );

    let stats = Arc::new(ThemeStats::new());
    let table = baked_table().with_observer(Arc::clone(&stats));
    let literal = first_literal(&table);
    let pair = first_pair(&table);
    let _ = table.resolve_rgba(literal, 1.0, true);
    let _ = table.resolve_rgba(pair, 0.5, true);
    let _ = table.resolve_rgba(9_999, 1.0, false);
    let snap = stats.snapshot();
    assert_eq!(
        (
            snap.rgba_resolves,
            snap.rgba_literals,
            snap.rgba_halves,
            snap.rgba_misses
        ),
        (3, 1, 0, 2),
        "the baked literal answers, the pair's alpha-on-pair and the \
         unknown id both refuse"
    );
    assert_eq!(
        (snap.resolves, snap.carried),
        (0, 0),
        "the tier channel stays silent on an rgba-only session"
    );
}

/// The seam is answer-neutral: the observed table's answers ARE the
/// plain table's — the face adds a channel, never changes a decision.
#[test]
fn the_observed_table_answers_identically() {
    let plain = runtime_table();
    let observed = runtime_table().with_observer(ThemeStats::new());
    for name in plain.names() {
        let id = plain.id(name).expect("the table's own name resolves");
        for alpha in [None, Some(0.0), Some(0.5), Some(1.0)] {
            assert_eq!(
                plain.resolve(id, alpha),
                observed.resolve(id, alpha),
                "resolve({id}, {alpha:?}) is identical with and without an observer"
            );
            for dark in [false, true] {
                assert_eq!(
                    plain.resolve_rgba(id, alpha.unwrap_or(1.0), dark),
                    observed.resolve_rgba(id, alpha.unwrap_or(1.0), dark),
                    "resolve_rgba({id}, …, {dark}) is identical with and without an observer"
                );
            }
        }
    }
    let plain = baked_table();
    let observed = baked_table().with_observer(ThemeStats::new());
    for name in plain.names() {
        let id = plain.id(name).expect("the table's own name resolves");
        for alpha in [None, Some(0.5)] {
            assert_eq!(
                plain.resolve(id, alpha),
                observed.resolve(id, alpha),
                "baked resolve({id}, {alpha:?}) is identical with and without an observer"
            );
        }
    }
}

/// The wiring rides `Clone` and one `Arc<ThemeStats>` spans several
/// tables: a cloned table streams into the same handle, and two tables
/// wired with the same handle total together (the `Arc<T>` blanket's
/// shared-handle shape, the cache face's own).
#[test]
fn the_wiring_shares_across_tables_and_clones() {
    let stats = Arc::new(ThemeStats::new());
    let table = runtime_table().with_observer(Arc::clone(&stats));
    let cloned = table.clone();
    let second = runtime_table().with_observer(Arc::clone(&stats));

    let id = table
        .id("primary")
        .expect("the shadcn source names primary");
    let _ = table.resolve(id, None);
    let _ = cloned.resolve(id, None);
    let _ = second.resolve(id, None);

    let snap = stats.snapshot();
    assert_eq!(
        (snap.resolves, snap.carried),
        (3, 3),
        "the original, the clone and the second table total into ONE handle"
    );
}

/// The `Arc<T>` blanket: an `Arc<T>` of any observer IS an observer —
/// the recorded events prove the forwarding (the cache face's own
/// blanket shape, mirrored here).
#[test]
fn the_arc_blanket_forwards_the_channels() {
    let recorder: Arc<Recorder> = Arc::new(Recorder::default());
    let table = runtime_table().with_observer(Arc::clone(&recorder));
    let id = table
        .id("primary")
        .expect("the shadcn source names primary");
    let _ = table.resolve(id, None);
    let _ = table.resolve_rgba(id, 0.5, true);
    assert_eq!(
        recorder.lines().len(),
        2,
        "both channels forwarded through the Arc blanket"
    );
}

/// The tier vocabulary the snapshots ride stays honest: a Runtime
/// table's snapshot never claims a baked arm, and the tier getter is
/// unaffected by the wiring (the face adds a channel, not a state).
#[test]
fn the_tier_and_snapshot_split_stay_honest() {
    assert_eq!(runtime_table().tier(), Tier::Runtime);
    assert_eq!(baked_table().tier(), Tier::Baked);
    let stats = Arc::new(ThemeStats::new());
    let table = baked_table().with_observer(Arc::clone(&stats));
    let pair = first_pair(&table);
    let literal = first_literal(&table);
    let _ = table.resolve(pair, None);
    let _ = table.resolve(literal, None);
    let snap = stats.snapshot();
    assert_eq!(
        (snap.pairs, snap.literals, snap.carried),
        (1, 1, 0),
        "the baked tier's split answers the snapshot, arm by arm"
    );
}
