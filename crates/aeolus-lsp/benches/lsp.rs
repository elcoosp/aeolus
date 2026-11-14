//! The LSP engine's microbenchmarks (divan, stable — the workspace
//! bench idiom): the analyze walk over a synthetic .rs document (the
//! extraction seam through the compile pipeline, per-position range
//! translation) and the hover resolution at a lowered class — the two
//! paths the editor session runs per keystroke.
//!
//! The bench bodies `expect` on the engine: the fixture document is
//! static, the answers deterministic. Numbers land in
//! `reports/08-benchmarks/BENCHMARKS.md` when the gate environment
//! runs them.

use aeolus_lsp::{AeolusEngine, EngineConfig};
use divan::Bencher;

fn main() {
    divan::main();
}

/// A synthetic .rs document the way `tw!` call sites look: two lowered
/// strings, one unknown root — the analyze bench's input and the hover
/// bench's document.
const DOC: &str = "fn card() {\n    let a = \"p-4 flex-col gap-2 rounded-md\";\n    let b = \"hover:bg-primary/90 dark:shadow-lg\";\n    let bad = \"not-a-root\";\n}\n";

/// The per-keystroke path: open → analyze (the diagnostics publish).
#[divan::bench]
fn analyze_document(bencher: Bencher) {
    bencher.bench(|| {
        let mut lsp = AeolusEngine::new(EngineConfig::default());
        lsp.open("file:///card.rs", DOC, 1);
        lsp.analyze("file:///card.rs")
    });
}

/// The per-cursor-move path: the hover resolution over the SAME
/// document, cursor inside the second string's class.
#[divan::bench]
fn hover_resolved_class(bencher: Bencher) {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", DOC, 1);
    let offset = DOC.find("bg-primary").expect("the fixture holds") + 4;
    let line = DOC[..offset].matches('\n').count() as u32;
    let line_start = DOC[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = DOC[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    bencher.bench(|| lsp.hover("file:///card.rs", line, character));
}
