//! Consumer demonstration: the engine's dyn-cache ride — the editor
//! session's answers dedupe through the bounded pool, so a keystroke
//! that leaves the strings alone rides the cache instead of
//! re-lowering. The observability face rides the engine's own
//! constructor: one [`CacheStats`] handle totals the whole session
//! (hits, misses, stores, evictions), and the answers stay
//! byte-identical beside it.
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_cache
//! ```

use std::sync::Arc;

use aeolus_lsp::{AeolusEngine, CacheStats, EngineConfig};

/// A synthetic .rs document the way `tw!` call sites look.
const DOC: &str = "fn card() {\n    let a = \"p-4 flex-col gap-2 rounded-md\";\n    let b = \"hover:bg-primary/90 dark:shadow-lg\";\n}\n";

/// The didChange's new text: string `a` rides unchanged, `b` swaps its
/// modifier — the change's one fresh lower.
const DOC_CHANGED: &str = "fn card() {\n    let a = \"p-4 flex-col gap-2 rounded-md\";\n    let b = \"hover:bg-primary/50 dark:shadow-lg\";\n}\n";

fn main() {
    let stats = Arc::new(CacheStats::new());
    let mut lsp = AeolusEngine::new(EngineConfig::default()).with_observer(Arc::clone(&stats));

    // 1. The didOpen: every ScanHit lowers once.
    lsp.open("file:///card.rs", DOC, 1);
    let diagnostics = lsp.analyze("file:///card.rs");
    println!(
        "── didOpen: {} finding(s) from the first walk ──",
        diagnostics.len()
    );
    println!("    snapshot: {:#?}", stats.snapshot());

    // 2. The keystroke that changed nothing: the whole document rides
    //    the pool.
    lsp.analyze("file:///card.rs");
    println!("── re-analyze of the unchanged document (every string a hit) ──");
    println!("    snapshot: {:#?}", stats.snapshot());

    // 3. The didChange: the kept strings hit, the new spelling lowers.
    lsp.change("file:///card.rs", DOC_CHANGED, 2);
    lsp.analyze("file:///card.rs");
    println!("── didChange: the kept strings hit, the new one lowers ──");
    println!("    snapshot: {:#?}", stats.snapshot());

    // 4. The cursor answers ride the same pool: the hover's
    //    single-candidate compile hits the entry analyze stored.
    let offset = DOC_CHANGED.find("bg-primary").expect("the fixture holds") + 4;
    let line = DOC_CHANGED[..offset].matches('\n').count() as u32;
    let line_start = DOC_CHANGED[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = DOC_CHANGED[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    if let Some(hover) = lsp.hover("file:///card.rs", line, character) {
        println!("── hover at the colour class ──");
        println!("    {}", hover.contents.lines().next().unwrap_or_default());
        println!("    snapshot: {:#?}", stats.snapshot());
    }
}
