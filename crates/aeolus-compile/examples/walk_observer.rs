//! Consumer demonstration: the compile-walk observability seam (the
//! dyn-cache face's twin, pointed at the walk) — the same pipeline the
//! plain entry points drive, its stage boundaries reported as it
//! assembles. The built-in `WalkStats` collector totals the stages
//! across several strings (one handle, one session), a custom observer
//! records the per-candidate verdicts in walk order, and the seam's
//! byte-neutrality shows in the artifacts: the observed compile's
//! program and diagnostics are the plain compile's own.

use std::sync::{Arc, Mutex};

use aeolus_compile::{
    CandidateOutcome, Config, Customs, WalkObserver, WalkStats, compile, compile_observed,
};

/// A host-implemented channel: the per-candidate verdicts, walk order —
/// the shape a telemetry host writes.
struct Recorder {
    events: Mutex<Vec<String>>,
}

impl WalkObserver for Recorder {
    fn on_candidate(&self, raw: &str, outcome: CandidateOutcome) {
        let line = match outcome {
            CandidateOutcome::Lowered { decls, diagnostics } => {
                format!("{raw}: lowered {decls} decls, {diagnostics} reports")
            }
            CandidateOutcome::Missed { diagnostics } => {
                format!("{raw}: missed, {diagnostics} reports")
            }
        };
        self.events
            .lock()
            .expect("the demo never poisons")
            .push(line);
    }
}

fn main() {
    // The built-in collector: one handle totals several walks (the
    // session's own shape).
    let stats = Arc::new(WalkStats::new());
    for input in [
        "p-4 flex items-center",
        "hover:bg-primary/90",
        "not-a-root-xyz p-2",
    ] {
        let observed = compile_observed(
            input,
            None,
            &Customs::new(),
            &Config::default(),
            stats.clone() as Arc<dyn WalkObserver>,
        )
        .expect("non-empty input compiles");
        println!(
            "\"{input}\" → {} layers, {} reports",
            observed.program.layers.len(),
            observed.diagnostics.len()
        );
    }
    let snap = stats.snapshot();
    println!(
        "\nsession: {} walks, {} candidates, {} degraded, {} lowered, {} missed, {} decls, {} reports, {} layers",
        snap.walks,
        snap.candidates,
        snap.degraded,
        snap.lowered,
        snap.missed,
        snap.decls,
        snap.diagnostics,
        snap.layers
    );

    // The custom channel: the per-candidate verdicts, walk order.
    let recorder = Arc::new(Recorder {
        events: Mutex::new(Vec::new()),
    });
    compile_observed(
        "p-4 hover:p-8 not-a-root-xyz",
        None,
        &Customs::new(),
        &Config::default(),
        recorder.clone() as Arc<dyn WalkObserver>,
    )
    .expect("non-empty input compiles");
    println!("\n── per-candidate verdicts (Recorder) ──");
    for event in recorder
        .events
        .lock()
        .expect("the demo never poisons")
        .iter()
    {
        println!("  {event}");
    }

    // The seam's byte-neutrality: the observed compile's artifact IS
    // the plain compile's artifact (the differential's own pin).
    let plain = compile("p-4 flex items-center", &Config::default()).expect("compiles");
    let observed = compile_observed(
        "p-4 flex items-center",
        None,
        &Customs::new(),
        &Config::default(),
        Arc::new(WalkStats::new()) as Arc<dyn WalkObserver>,
    )
    .expect("compiles");
    println!(
        "\nbyte-neutrality: plain layers == observed layers → {}",
        plain.program.layers.len() == observed.program.layers.len()
    );
}
