//! The LSP engine's microbenchmarks (divan, stable — the workspace
//! bench idiom): the analyze walk over a synthetic .rs document (the
//! extraction seam through the compile pipeline, per-position range
//! translation), the hover resolution at a lowered class, the
//! completion at a typing edge and the colour-swatch walk (the
//! per-candidate compile resolving the IR's colour slots) — the paths
//! the editor session runs per keystroke and per didOpen.
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

/// The typing-edge path: the completion at a fragment inside the second
/// string's colour class (`bg-pri` typed, the token source answering).
#[divan::bench]
fn completion_fragment(bencher: Bencher) {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", DOC, 1);
    let offset = DOC.find("bg-primary").expect("the fixture holds") + 4;
    let line = DOC[..offset].matches('\n').count() as u32;
    let line_start = DOC[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = DOC[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    bencher.bench(|| lsp.complete("file:///card.rs", line, character));
}

/// The didOpen path: the colour swatches (the per-candidate compile
/// walk resolving the table's light-tier bytes) over the SAME document.
#[divan::bench]
fn document_colours_walk(bencher: Bencher) {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", DOC, 1);
    bencher.bench(|| lsp.document_colours("file:///card.rs"));
}

/// The highlighter path (LSP 3.16's semanticTokens/full): the
/// per-candidate compile walk classifying every candidate's outcome —
/// the same shape the colour walk rides, the classification the
/// diagnostics list's emptiness.
#[divan::bench]
fn semantic_tokens_walk(bencher: Bencher) {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", DOC, 1);
    bencher.bench(|| lsp.semantic_tokens("file:///card.rs"));
}

/// The picker path (the colorProvider capability's paired
/// colorPresentation request): the config's table walked backwards —
/// the colour-group rows × the token names × the audited opacity steps,
/// the exact resolve matching the requested bytes.
#[divan::bench]
fn colour_presentations_walk(bencher: Bencher) {
    let lsp = AeolusEngine::new(EngineConfig::default());
    bencher.bench(|| lsp.colour_presentations([93, 90, 208, 255]));
}

/// The inline-answers path (LSP 3.17's inlayHint): the per-candidate
/// compile walk rendering the CSS arm's own declarations compactly at
/// each lowered class's end — the hints the editor session answers per
/// didOpen (and per edit's re-hinting), the refused candidates staying
/// quiet.
#[divan::bench]
fn inlay_hints_walk(bencher: Bencher) {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", DOC, 1);
    bencher.bench(|| lsp.inlay_hints("file:///card.rs"));
}

/// The pull path (LSP 3.17's documentDiagnostic): the client's
/// on-demand recompute through the state machine — initialize and
/// didOpen once, then the request per iteration (the params parse, the
/// analyze walk, the wire mapping and the report serialization: what a
/// refresh or a newly focused file costs the session).
#[divan::bench]
fn document_diagnostics_pull(bencher: Bencher) {
    use aeolus_lsp::{Session, handle_message};
    use lsp_server::{Message, Notification, Request, RequestId};
    use serde_json::json;
    use std::sync::Mutex;

    let session = Mutex::new(Session::new(Default::default()));
    let _ = handle_message(
        &mut session.lock().expect("the bench session locks"),
        Message::Request(Request {
            id: RequestId::from(0),
            method: "initialize".to_string(),
            params: json!({"capabilities": {}}),
        }),
    );
    let _ = handle_message(
        &mut session.lock().expect("the bench session locks"),
        Message::Notification(Notification {
            method: "textDocument/didOpen".to_string(),
            params: json!({
                "textDocument": {
                    "uri": "file:///card.rs",
                    "languageId": "rust",
                    "version": 1,
                    "text": DOC,
                }
            }),
        }),
    );
    bencher.bench(|| {
        handle_message(
            &mut session.lock().expect("the bench session locks"),
            Message::Request(Request {
                id: RequestId::from(11),
                method: "textDocument/documentDiagnostic".to_string(),
                params: json!({"textDocument": {"uri": "file:///card.rs"}}),
            }),
        )
    });
}

/// The quick-fix document: one keyword-table miss, one theme-token
/// miss under a variant prefix — the two folds the seam answers.
const ACTIONS_DOC: &str = "fn card() {\n    let bad = \"justify-betwen hover:bg-priamry\";\n}\n";

/// The quick-fix path (LSP 3.16's codeAction): the cursor's candidate
/// through the suggestion seam — the compile walk's diagnostics folded
/// over the audited tables, the corrections composed onto the full
/// token — the cost a squiggle's lightbulb pays per invocation.
#[divan::bench]
fn code_actions_walk(bencher: Bencher) {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", ACTIONS_DOC, 1);
    let offset = ACTIONS_DOC
        .find("justify-betwen")
        .expect("the fixture holds")
        + 2;
    let line = ACTIONS_DOC[..offset].matches('\n').count() as u32;
    let line_start = ACTIONS_DOC[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = ACTIONS_DOC[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    bencher.bench(|| lsp.code_actions("file:///card.rs", line, character));
}

/// The click-to-find path (LSP 3.16's documentHighlight): the
/// document's occurrences of the cursor's candidate — the shared
/// locator's walk over the extraction seam's strings.
#[divan::bench]
fn document_highlights_walk(bencher: Bencher) {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", DOC, 1);
    let offset = DOC.find("flex-col").expect("the fixture holds") + 2;
    let line = DOC[..offset].matches('\n').count() as u32;
    let line_start = DOC[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = DOC[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    bencher.bench(|| lsp.document_highlights("file:///card.rs", line, character));
}

/// The format-on-save path (the class-sort seam's editor face): the
/// document's class strings through the themeless sort walk — the
/// registry's read side and the variant fold per string, the canonical
/// join compared against the hit text — the cost a format request pays
/// per didOpen (and a dirty buffer per save).
#[divan::bench]
fn format_walk(bencher: Bencher) {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", DOC, 1);
    bencher.bench(|| lsp.format("file:///card.rs"));
}
