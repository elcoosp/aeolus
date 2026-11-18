//! Consumer demonstration: the engine's walk face — the compile-walk
//! observability seam riding the pooled path. The cache face
//! (`lsp_cache`) answers "what did the pool decide"; this one answers
//! "what did the walk do when the pool missed": one [`WalkStats`]
//! handle totals the session's lowerings (the lex totals, the
//! per-candidate verdicts, the assembly counts), a keystroke that
//! leaves the strings alone streams NOTHING (the pool answered; the
//! walk never ran), and a change lowers only its new spelling. The
//! engine holds ONE walk slot (the builders' replace semantics, the
//! cache face's own), so a host that wants two channels composes them
//! host-side — the demo's `Telemetry` pair.
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_walk_observer
//! ```

use std::sync::{Arc, Mutex};

use aeolus_lsp::{AeolusEngine, CandidateOutcome, EngineConfig, WalkObserver, WalkStats};

/// A host-implemented channel: the per-candidate verdicts, walk order —
/// the shape an editor's telemetry panel writes.
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

/// Host-side composition: the engine holds ONE walk slot, so two
/// channels ride one forwarding observer (the trait's no-op defaults
/// make the composite two forwards — the `Arc` handles stay alive
/// beside the engine).
struct Telemetry {
    stats: Arc<WalkStats>,
    log: Arc<Recorder>,
}

impl WalkObserver for Telemetry {
    fn on_lexed(&self, candidates: usize, degraded: usize) {
        self.stats.on_lexed(candidates, degraded);
    }

    fn on_candidate(&self, raw: &str, outcome: CandidateOutcome) {
        self.stats.on_candidate(raw, outcome);
        self.log.on_candidate(raw, outcome);
    }

    fn on_assembled(&self, layers: usize) {
        self.stats.on_assembled(layers);
    }
}

/// A synthetic .rs document the way `tw!` call sites look.
const DOC: &str = "fn card() {\n    let a = \"p-4 flex-col gap-2 rounded-md\";\n    let b = \"hover:bg-primary/90 dark:shadow-lg\";\n    let bad = \"not-a-root\";\n}\n";

/// The didChange's new text: string `a` rides unchanged, `b` swaps its
/// modifier — the change's one fresh lower.
const DOC_CHANGED: &str = "fn card() {\n    let a = \"p-4 flex-col gap-2 rounded-md\";\n    let b = \"hover:bg-primary/50 dark:shadow-lg\";\n    let bad = \"not-a-root\";\n}\n";

fn main() {
    let stats = Arc::new(WalkStats::new());
    let log = Arc::new(Recorder {
        events: Mutex::new(Vec::new()),
    });
    let mut lsp = AeolusEngine::new(EngineConfig::default()).with_walk_observer(Telemetry {
        stats: Arc::clone(&stats),
        log: Arc::clone(&log),
    });

    // 1. The didOpen: every ScanHit lowers once — the walks stream.
    lsp.open("file:///card.rs", DOC, 1);
    lsp.analyze("file:///card.rs");
    println!("── didOpen: three strings, three observed walks ──");
    println!("    snapshot: {:#?}", stats.snapshot());

    // 2. The keystroke that changed nothing: the pool answers, the
    //    walk never runs, the face stays silent.
    lsp.analyze("file:///card.rs");
    println!("── re-analyze of the unchanged document (all hits, no walks) ──");
    println!("    snapshot: {:#?}", stats.snapshot());

    // 3. The didChange: the kept strings hit, the swapped spelling
    //    lowers — one new walk's verdicts in walk order.
    lsp.change("file:///card.rs", DOC_CHANGED, 2);
    lsp.analyze("file:///card.rs");
    println!("── didChange: the kept strings hit, the new spelling walks ──");
    println!("    snapshot: {:#?}", stats.snapshot());
    println!("\n── per-candidate verdicts (Recorder, every walk so far) ──");
    for line in log.events.lock().expect("the demo never poisons").iter() {
        println!("    {line}");
    }
}
