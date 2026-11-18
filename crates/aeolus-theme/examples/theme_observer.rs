//! Consumer demonstration: the theme-resolution observability seam —
//! the token table's own resolution answers as a default-no-op observer
//! channel, the compile-walk face's twin pointed at the theme stage.
//! The built-in `ThemeStats` collector totals the tier arms across
//! several tables (one handle, one session), a custom observer records
//! the per-resolve verdicts in caller order, and the seam's
//! answer-neutrality shows in the values: the observed table's
//! resolutions are the plain table's own. A table wired before the
//! compile walk's `compile_theme` streams the lowering's
//! per-declaration consults — the theme stage the walk face's events
//! sit above.

use std::sync::{Arc, Mutex};

use aeolus_theme::{
    ResolveOutcome, RgbaOutcome, ThemeObserver, ThemeSource, ThemeStats, Tier, TokenTable,
};

/// A host-implemented channel: the per-resolve verdicts, caller order —
/// the shape a telemetry host writes.
struct Recorder {
    events: Mutex<Vec<String>>,
}

impl ThemeObserver for Recorder {
    fn on_resolve(&self, id: u16, alpha: Option<f32>, outcome: ResolveOutcome) {
        let line = match outcome {
            ResolveOutcome::TokCarried => format!("  resolve {id} {alpha:?} → Tok carried"),
            ResolveOutcome::Literal => format!("  resolve {id} {alpha:?} → baked literal"),
            ResolveOutcome::ModePair => format!("  resolve {id} {alpha:?} → mode pair slot"),
            ResolveOutcome::Missed(error) => format!("  resolve {id} {alpha:?} → missed: {error}"),
        };
        self.events
            .lock()
            .expect("the demo never poisons")
            .push(line);
    }

    fn on_resolve_rgba(&self, id: u16, alpha: f32, dark: bool, outcome: RgbaOutcome) {
        let line = match outcome {
            RgbaOutcome::ModeHalf => {
                format!("  resolve_rgba {id} {alpha} dark={dark} → active half")
            }
            RgbaOutcome::Literal => {
                format!("  resolve_rgba {id} {alpha} dark={dark} → light literal")
            }
            RgbaOutcome::Missed(error) => {
                format!("  resolve_rgba {id} {alpha} dark={dark} → missed: {error}")
            }
        };
        self.events
            .lock()
            .expect("the demo never poisons")
            .push(line);
    }
}

fn main() {
    // The built-in collector: one handle totals several tables (the
    // runtime session AND a baked probe, the `Arc` blanket's shape).
    let stats = Arc::new(ThemeStats::new());
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");

    let runtime = TokenTable::runtime(&source)
        .expect("default-seeded source builds")
        .with_observer(Arc::clone(&stats));
    let baked = TokenTable::baked(&source)
        .expect("default-seeded source builds")
        .with_observer(Arc::clone(&stats));

    let primary = runtime.id("primary").expect("primary is a shadcn token");
    let _ = runtime.resolve(primary, None);
    let _ = runtime.resolve(primary, Some(0.9));
    let _ = baked.resolve(primary, None);
    let _ = runtime.resolve_rgba(primary, 0.5, true);
    let _ = runtime.resolve(9_999, None); // the miss IS an event

    let snap = stats.snapshot();
    println!("── session totals (one handle, two tables) ──");
    println!(
        "tier channel   → {} resolves ({} carried, {} pairs, {} missed)",
        snap.resolves, snap.carried, snap.pairs, snap.misses
    );
    println!(
        "rgba channel   → {} resolves ({} mode halves, {} literals, {} missed)",
        snap.rgba_resolves, snap.rgba_halves, snap.rgba_literals, snap.rgba_misses
    );

    // A custom observer: the per-resolve verdicts in caller order —
    // the lowering's own sequence when the compile walk rides a wired
    // table.
    let recorder = Arc::new(Recorder {
        events: Mutex::new(Vec::new()),
    });
    let observed = TokenTable::runtime(&source)
        .expect("default-seeded source builds")
        .with_observer(Arc::clone(&recorder));
    let _ = observed.resolve(primary, Some(0.9));
    let _ = observed.resolve_rgba(primary, 0.5, false);
    let _ = observed.resolve(9_999, None);
    println!("\n── per-resolve verdicts (Recorder) ──");
    for line in recorder
        .events
        .lock()
        .expect("the demo never poisons")
        .iter()
    {
        println!("{line}");
    }

    // Answer-neutrality: the observed table's answers ARE the plain
    // table's — the face adds a channel, never changes a decision.
    let plain = TokenTable::runtime(&source).expect("default-seeded source builds");
    let same = plain.resolve(primary, Some(0.9)) == observed.resolve(primary, Some(0.9));
    println!("\n── answer-neutrality ──");
    println!(
        "tier = {:?}; observed answers identical to plain: {}",
        runtime.tier(),
        same
    );
    let _ = Tier::Runtime; // the tier vocabulary the snapshot rides
}
