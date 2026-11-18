//! The cross-document workspace-symbol answer (the wire's
//! `workspace/symbol` — the references fold's second consumer): every
//! candidate class across EVERY open document whose spelling matches
//! the query, the document symbols' outline riding the whole document
//! map. Written FIRST (the RED spec): the missing engine method's
//! compile failure WAS the capture.
//!
//! The suite pins the answer's house rules beside the answers:
//!
//! * the OUTLINE contract generalized (the document symbols' own walk):
//!   the spelling the name, the byte-true range, the semantic legend's
//!   own verdict (lowered / refused) and the pipeline's own detail
//!   line — one symbol per candidate, no parallel matcher;
//! * the FILTER is the query: case-insensitive substring on the
//!   spelling; the EMPTY query filters nothing (the whole open set
//!   rides — the protocol's own shape, the query is a filter);
//! * DETERMINISM over the hash map: the documents ride URI order
//!   (sorted, whatever the open order was), the candidates document
//!   order within each — the answer is pure over the map;
//! * the whole document map is the answer's scope: the CLOSED document
//!   dropping out, a miss answering empty;
//! * the wire half: the request answers the protocol's own
//!   `SymbolInformation` quads (the lowered class `Class`, the refused
//!   one `String` — the documentSymbol arm's own mapping), the
//!   capability rides `initialize`, the pre-handshake gate
//!   quiet-drops, malformed params answer the wire's own null result.

use aeolus_lsp::{
    AeolusEngine, EngineConfig, EngineSemanticKind, EngineWorkspaceSymbol, Session, handle_message,
};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

const DOC_A: &str =
    "fn card() {\n    let a = \"flex-col\";\n    let b = \"hover:bg-primary/90\";\n}\n";
const DOC_B: &str = "fn page() {\n    let a = \"not-a-root flex-col\";\n}\n";
const DOC_C: &str = "fn list() {\n    let a = \"p-4 gap-2\";\n}\n";

fn request(id: i32, method: &str, params: serde_json::Value) -> Message {
    Message::Request(Request {
        id: RequestId::from(id),
        method: method.to_string(),
        params,
    })
}

fn notification(method: &str, params: serde_json::Value) -> Message {
    Message::Notification(Notification {
        method: method.to_string(),
        params,
    })
}

fn did_open(uri: &str, text: &str) -> Message {
    notification(
        "textDocument/didOpen",
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "rust",
                "version": 1,
                "text": text,
            }
        }),
    )
}

fn initialize(session: &mut Session) {
    let handled = handle_message(
        session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    assert_eq!(handled.outgoing.len(), 1, "initialize answers its result");
}

fn wired_engine() -> AeolusEngine {
    let mut engine = AeolusEngine::new(EngineConfig::default());
    engine.open("file:///a.rs", DOC_A, 1);
    engine.open("file:///b.rs", DOC_B, 1);
    engine.open("file:///c.rs", DOC_C, 1);
    engine
}

// ---------------------------------------------------------------------------
// The engine-level pins
// ---------------------------------------------------------------------------

#[test]
fn the_empty_query_rides_every_open_document() {
    let engine = wired_engine();
    let symbols = engine.workspace_symbols("");
    // a.rs: flex-col + hover:bg-primary/90; b.rs: not-a-root + flex-col
    // (two strings? one string, two candidates); c.rs: p-4 + gap-2 —
    // six candidates across three documents.
    assert_eq!(symbols.len(), 6, "every candidate rides");
    let uris: Vec<&str> = symbols.iter().map(|s| s.uri.as_str()).collect();
    assert_eq!(
        uris,
        [
            "file:///a.rs",
            "file:///a.rs",
            "file:///b.rs",
            "file:///b.rs",
            "file:///c.rs",
            "file:///c.rs"
        ]
    );
    assert_eq!(symbols[0].symbol.name, "flex-col");
    assert_eq!(symbols[0].symbol.kind, EngineSemanticKind::Lowered);
    assert_eq!(symbols[1].symbol.name, "hover:bg-primary/90");
}

#[test]
fn the_filter_is_a_case_insensitive_substring() {
    let engine = wired_engine();
    let flex = engine.workspace_symbols("FLEX");
    assert_eq!(flex.len(), 2, "a.rs and b.rs carry a flex spelling");
    assert_eq!(flex[0].uri, "file:///a.rs");
    assert_eq!(flex[0].symbol.name, "flex-col");
    assert_eq!(flex[1].uri, "file:///b.rs");
    let bg = engine.workspace_symbols("bg-");
    assert_eq!(bg.len(), 1, "the variant-prefixed spelling matches too");
    assert_eq!(bg[0].symbol.name, "hover:bg-primary/90");
}

#[test]
fn a_miss_answers_empty() {
    let engine = wired_engine();
    assert!(engine.workspace_symbols("grid-cols-7").is_empty());
}

#[test]
fn refused_candidates_symbol_with_their_code() {
    let engine = wired_engine();
    let refused = engine.workspace_symbols("not-a-root");
    assert_eq!(refused.len(), 1);
    assert_eq!(refused[0].symbol.kind, EngineSemanticKind::Refused);
    assert!(
        !refused[0].symbol.detail.is_empty(),
        "the stable code rides the detail line"
    );
}

#[test]
fn determinism_is_uri_order_not_map_order() {
    let mut engine = AeolusEngine::new(EngineConfig::default());
    engine.open("file:///c.rs", DOC_C, 1);
    engine.open("file:///a.rs", DOC_A, 1);
    engine.open("file:///b.rs", DOC_B, 1);
    let symbols = engine.workspace_symbols("");
    let uris: Vec<&str> = symbols.iter().map(|s| s.uri.as_str()).collect();
    assert_eq!(
        uris.first(),
        Some(&"file:///a.rs"),
        "the answer rides URI order, whatever the open order was"
    );
    assert_eq!(symbols.len(), 6);
}

#[test]
fn closed_document_drops_out() {
    let mut engine = wired_engine();
    engine.close("file:///b.rs");
    assert_eq!(
        engine.workspace_symbols("").len(),
        4,
        "the closed document's candidates drop"
    );
}

// ---------------------------------------------------------------------------
// The wire-level pins
// ---------------------------------------------------------------------------

#[test]
fn wire_workspace_symbol_answers_symbol_information() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let _ = handle_message(&mut session, did_open("file:///a.rs", DOC_A));
    let handled = handle_message(
        &mut session,
        request(1, "workspace/symbol", json!({"query": "flex"})),
    );
    assert_eq!(handled.outgoing.len(), 1, "the response rides");
    let result = match &handled.outgoing[0] {
        Message::Response(response) => response.result.clone(),
        other => panic!("a response rode, got {other:?}"),
    };
    let symbols = result.expect("the symbols answer");
    let symbols = symbols.as_array().expect("an array of symbols");
    assert_eq!(symbols.len(), 1, "a.rs's flex-col");
    assert_eq!(symbols[0]["name"], "flex-col");
    assert_eq!(symbols[0]["location"]["uri"], "file:///a.rs");
    assert_eq!(symbols[0]["location"]["range"]["start"]["line"], 1);
}

#[test]
fn wire_capability_rides_initialize() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    let result = match &handled.outgoing[0] {
        Message::Response(response) => response.result.clone(),
        other => panic!("a response rode, got {other:?}"),
    };
    let capabilities = result.expect("the initialize result");
    assert_eq!(
        capabilities["capabilities"]["workspaceSymbolProvider"],
        json!(true),
        "the workspace-symbol capability is advertised"
    );
}

#[test]
fn wire_pre_handshake_quiet_drops() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(1, "workspace/symbol", json!({"query": "flex"})),
    );
    assert!(handled.outgoing.is_empty(), "the pre-handshake gate holds");
}

#[test]
fn wire_malformed_params_answer_null() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let handled = handle_message(
        &mut session,
        request(1, "workspace/symbol", json!({"query": 42})),
    );
    let result = match &handled.outgoing[0] {
        Message::Response(response) => response.result.clone(),
        other => panic!("a response rode, got {other:?}"),
    };
    assert!(
        matches!(&result, Some(value) if value.is_null()),
        "malformed params answer the wire's own null result"
    );
}

#[test]
fn wire_workspace_symbols_plural_stays_unknown() {
    // The pre-existing quiet-drop pin's own method spelling
    // (`workspace/symbols`, the client's typo) rides silently still —
    // the capability names the protocol's own singular method.
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let handled = handle_message(
        &mut session,
        request(1, "workspace/symbols", json!({"query": "flex"})),
    );
    assert!(handled.outgoing.is_empty(), "the typo stays unknown");
}

#[test]
fn engine_workspace_symbol_type_rides_the_map() {
    // The answer shape itself: the URI beside the outline symbol (the
    // references answer's own composition).
    let engine = wired_engine();
    let symbols: Vec<EngineWorkspaceSymbol> = engine.workspace_symbols("p-4");
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].uri, "file:///c.rs");
    assert_eq!(symbols[0].symbol.name, "p-4");
}
