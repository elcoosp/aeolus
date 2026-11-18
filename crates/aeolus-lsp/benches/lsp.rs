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
use lsp_server::Message;

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

/// The re-answer path over the session's pool: the SAME engine, the
/// same document re-analyzed (the keystroke that changed nothing) —
/// every ScanHit rides the cache instead of re-lowering (the dyn-cache
/// slice's latency half, the engine's own ride).
#[divan::bench]
fn analyze_warm_pool(bencher: Bencher) {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", DOC, 1);
    bencher.bench(|| lsp.analyze("file:///card.rs"));
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

/// The cross-document references answer (the engine's first
/// cross-document question): the cursor's candidate walked across
/// every open document — ONE here, the highlights' own scope.
#[divan::bench]
fn references_one_document(bencher: Bencher) {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", DOC, 1);
    let offset = DOC.find("flex-col").expect("the fixture holds") + 2;
    let line = DOC[..offset].matches('\n').count() as u32;
    let line_start = DOC[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = DOC[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    bencher.bench(|| lsp.references("file:///card.rs", line, character));
}

/// The SAME query over EIGHT open documents (the map's own scaling
/// story: one scan per document, the URI sort beside — the editor
/// session's realistic open set).
#[divan::bench]
fn references_eight_documents(bencher: Bencher) {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    for index in 0..8 {
        lsp.open(&format!("file:///card-{index}.rs"), DOC, 1);
    }
    let offset = DOC.find("flex-col").expect("the fixture holds") + 2;
    let line = DOC[..offset].matches('\n').count() as u32;
    let line_start = DOC[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = DOC[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    bencher.bench(|| lsp.references("file:///card-0.rs", line, character));
}

/// The workspace-symbol answer (the references fold's second
/// consumer): the outline walk over EVERY open document with the EMPTY
/// query riding every open candidate — the filter's identity shape.
#[divan::bench]
fn workspace_symbols_all_query(bencher: Bencher) {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    for index in 0..8 {
        lsp.open(&format!("file:///card-{index}.rs"), DOC, 1);
    }
    bencher.bench(|| lsp.workspace_symbols(""));
}

/// The SAME walk under a real filter (the editor's typed prefix) — the
/// substring test's own cost beside the walk.
#[divan::bench]
fn workspace_symbols_filtered_query(bencher: Bencher) {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    for index in 0..8 {
        lsp.open(&format!("file:///card-{index}.rs"), DOC, 1);
    }
    bencher.bench(|| lsp.workspace_symbols("bg-"));
}

/// The rename path (the rename face's second half): the references'
/// fold over the rename edits — the cursor's candidate walked across
/// every open document with the new spelling riding each occurrence —
/// the cost a rename gesture pays per invocation (the prepare half is
/// the locator's span answer, the walk's first read).
#[divan::bench]
fn rename_walk(bencher: Bencher) {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", DOC, 1);
    let offset = DOC.find("flex-col").expect("the fixture holds") + 2;
    let line = DOC[..offset].matches('\n').count() as u32;
    let line_start = DOC[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = DOC[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    bencher.bench(|| lsp.rename("file:///card.rs", line, character, "flex-row"));
}

/// The SAME rename over EIGHT open documents (the references arm's own
/// scaling story, the edits riding the map: one scan per document
/// beside the URI sort — the editor session's realistic open set).
#[divan::bench]
fn rename_eight_documents(bencher: Bencher) {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    for index in 0..8 {
        lsp.open(&format!("file:///card-{index}.rs"), DOC, 1);
    }
    let offset = DOC.find("flex-col").expect("the fixture holds") + 2;
    let line = DOC[..offset].matches('\n').count() as u32;
    let line_start = DOC[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = DOC[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    bencher.bench(|| lsp.rename("file:///card-0.rs", line, character, "flex-row"));
}

/// The expand-selection path (LSP 3.16's selectionRange): the shared
/// locator's walk building the parent chains — the candidate span the
/// cursor keys, the string's content span its parent — the cost a
/// multi-cursor expand pays per invocation.
#[divan::bench]
fn selection_ranges_walk(bencher: Bencher) {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", DOC, 1);
    let offset = DOC.find("flex-col").expect("the fixture holds") + 2;
    let line = DOC[..offset].matches('\n').count() as u32;
    let line_start = DOC[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = DOC[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    bencher.bench(|| {
        lsp.selection_ranges(
            "file:///card.rs",
            &[
                aeolus_lsp::EnginePosition { line, character },
                aeolus_lsp::EnginePosition {
                    line: 0,
                    character: 2,
                },
            ],
        )
    });
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

/// The outline path (LSP 3.16's documentSymbol): the per-candidate
/// compile walk naming every class — the legend's verdict beside the
/// compact-CSS detail — the cost an outline panel pays per didOpen
/// (and per edit's re-outline).
#[divan::bench]
fn document_symbols_walk(bencher: Bencher) {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", DOC, 1);
    bencher.bench(|| lsp.document_symbols("file:///card.rs"));
}

/// The wire face's traffic script: the didOpen publish, an answered
/// hover, a re-publishing didChange, one quiet-dropped request and one
/// quiet-dropped notification — the per-message mix an editor session
/// rides at the wire (the lifecycle push, the cursor answers, the
/// unadvertised methods).
fn wire_script() -> Vec<Message> {
    use lsp_server::{Notification, Request, RequestId};
    use serde_json::json;

    let hover_params = json!({
        "textDocument": {"uri": "file:///card.rs"},
        "position": {"line": 1, "character": 14}
    });
    vec![
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
        Message::Request(Request {
            id: RequestId::from(1),
            method: "textDocument/hover".to_string(),
            params: hover_params,
        }),
        Message::Notification(Notification {
            method: "textDocument/didChange".to_string(),
            params: json!({
                "textDocument": {"uri": "file:///card.rs", "version": 2},
                "contentChanges": [{"text": DOC}]
            }),
        }),
        Message::Request(Request {
            id: RequestId::from(2),
            method: "textDocument/definitely-not-mine".to_string(),
            params: json!({}),
        }),
        Message::Notification(Notification {
            method: "$/setTrace".to_string(),
            params: json!({"value": "off"}),
        }),
    ]
}

/// The wire face's cost, unobserved: the state machine's own verdict
/// loop over the traffic script (the publish, the response, the quiet
/// drops) — the baseline the observer's channel costs stand against.
#[divan::bench]
fn wire_session_unobserved(bencher: Bencher) {
    use aeolus_lsp::{Session, handle_message};
    use std::sync::Mutex;

    let script = wire_script();
    let session = Mutex::new(Session::new(Default::default()));
    bencher.bench(|| {
        let mut session = session.lock().expect("the bench session locks");
        for message in &script {
            let _ = handle_message(&mut session, message.clone());
        }
    });
}

/// The wire face's cost, counted: the SAME traffic over a session
/// wired with the built-in [`TransportStats`](aeolus_lsp::TransportStats)
/// — the channel's overhead story (one atomic add per message verdict,
/// the answers byte-identical to the unwired loop's).
#[divan::bench]
fn wire_session_counted(bencher: Bencher) {
    use aeolus_lsp::{Session, TransportStats, handle_message};
    use std::sync::{Arc, Mutex};

    let script = wire_script();
    let stats = Arc::new(TransportStats::new());
    let session = Mutex::new(Session::new(Default::default()).with_observer(Arc::clone(&stats)));
    bencher.bench(|| {
        let mut session = session.lock().expect("the bench session locks");
        for message in &script {
            let _ = handle_message(&mut session, message.clone());
        }
    });
}

// ---------------------------------------------------------------------------
// The theme lifecycle's arms (the reload seam)
// ---------------------------------------------------------------------------

/// The reload seam's fixtures: the riding css and the rebrand beside it
/// (same token names, new brand colour), pre-parsed ONCE so the arms
/// price the bake + swap + recompile, not the css parse.
const RIDE_CSS: &str = ":root { --color-primary: #336699; --color-surface: #eeeeee; }";
const REBRAND_CSS: &str = ":root { --color-primary: #aa4455; --color-surface: #eeeeee; }";

/// The lifecycle's first cost, the loader's own: one
/// [`ThemeSource::from_css`] + [`TokenTable::baked`] — the read the
/// `load()` seam serves and the watched-files arm pays on every
/// re-theme.
#[divan::bench]
fn theme_bake(bencher: Bencher) {
    use aeolus_theme::{ThemeSource, TokenTable};

    let source = ThemeSource::from_css(RIDE_CSS, None).expect("the fixture parses");
    bencher.bench(|| TokenTable::baked(&source).expect("the fixture bakes"));
}

/// The swap path: warm pool → re-theme to the REBRAND (a real swap) →
/// re-analyze the document (the pool was retired, the strings re-lower
/// under the new table). The lifecycle's honest whole: the bake, the
/// invalidation and the recompile storm a real save pays.
#[divan::bench]
fn retheme_swap(bencher: Bencher) {
    use aeolus_theme::{ThemeSource, TokenTable};
    use std::sync::Mutex;

    let riding = ThemeSource::from_css(RIDE_CSS, None).expect("the fixture parses");
    let rebrand = ThemeSource::from_css(REBRAND_CSS, None).expect("the fixture parses");
    let mut lsp = AeolusEngine::new(EngineConfig::new(Some(
        TokenTable::baked(&riding).expect("the fixture bakes"),
    )));
    lsp.open("file:///card.rs", DOC, 1);
    let _ = lsp.analyze("file:///card.rs");
    // The retheme mutates the engine (the `Fn` bench body's own shape —
    // the wire arms' Mutex pattern), the lock is uncontended.
    let lsp = Mutex::new(lsp);
    bencher.bench(|| {
        let mut lsp = lsp.lock().expect("the bench engine locks");
        let _ = lsp.retheme(Some(
            TokenTable::baked(&rebrand).expect("the fixture bakes"),
        ));
        lsp.analyze("file:///card.rs")
    });
}

/// The no-op save path: the SAME css re-baked every iteration — the
/// equality fast-path's own cost (the tier/names/colours walk beside
/// the untouched pool), the shape a format-on-save rides for free.
#[divan::bench]
fn retheme_unchanged(bencher: Bencher) {
    use aeolus_theme::{ThemeSource, TokenTable};
    use std::sync::Mutex;

    let riding = ThemeSource::from_css(RIDE_CSS, None).expect("the fixture parses");
    let mut lsp = AeolusEngine::new(EngineConfig::new(Some(
        TokenTable::baked(&riding).expect("the fixture bakes"),
    )));
    lsp.open("file:///card.rs", DOC, 1);
    let _ = lsp.analyze("file:///card.rs");
    // The mutation wrapper again (the swap arm's own shape).
    let lsp = Mutex::new(lsp);
    bencher.bench(|| {
        let mut lsp = lsp.lock().expect("the bench engine locks");
        let outcome = lsp.retheme(Some(TokenTable::baked(&riding).expect("the fixture bakes")));
        assert_eq!(outcome, aeolus_lsp::RethemeOutcome::Unchanged);
    });
}

/// The simultaneous-edit path (LSP 3.16's linkedEditingRange): the
/// highlights' textual equality rendered as the client's
/// edit-together ranges — the cursor's candidate against every string
/// in the document, the linked-editing gesture's per-cursor cost.
#[divan::bench]
fn linked_ranges_walk(bencher: Bencher) {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", DOC, 1);
    let offset = DOC.find("flex-col").expect("the fixture holds") + 2;
    let line = DOC[..offset].matches('\n').count() as u32;
    let line_start = DOC[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = DOC[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    bencher.bench(|| lsp.linked_ranges("file:///card.rs", line, character));
}
