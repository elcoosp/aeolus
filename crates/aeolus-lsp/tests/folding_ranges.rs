// ---------------------------------------------------------------------------
// The folding-range contracts (the editor's outline folding face): every
// class string the extraction seam finds answers ONE fold — the string's
// own content span, document order kept. The fold is a SHAPE-level answer
// (the highlights' textual family): no compile verdict rides, a refused
// candidate's string folds exactly like a lowered one's, and the closed
// class strings are the closed answer set (the extraction seam's own
// pre-filter — a prose literal with no `-`/`:` token is not a class
// string, so it never folds).
//
// Written FIRST (the RED spec).
// ---------------------------------------------------------------------------

use aeolus_lsp::{AeolusEngine, EngineConfig, Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// The document the battery answers over: two single-line quoted class
/// strings, a variant-prefixed sibling string, a refused candidate, a
/// prose literal the extractor pre-filters out, and the multi-line raw
/// string the fold actually folds (the quoted walk stops at a newline —
/// the raw-string form is the multi-line shape).
const DOC: &str = concat!(
    "fn card() {\n",
    "    let a = \"p-4 bg-primary\";\n",
    "    let b = \"hover:bg-primary not-a-root\";\n",
    "    let prose = \"a plain sentence, no tokens\";\n",
    "    let c = r\"flex-col\n",
    "        items-center\";\n",
    "}\n",
);

/// The accented-neighbour document: the multi-byte identifier shifts
/// every column after it — the byte-truth probe.
const ACCENTED_DOC: &str = "fn héllo() {\n    let a = \"bg-primary\";\n}\n";

const URI: &str = "file:///card.rs";

fn engine(doc: &str) -> AeolusEngine {
    let mut lsp = AeolusEngine::new(EngineConfig::new(None));
    lsp.open(URI, doc, 1);
    lsp
}

// ---------------------------------------------------------------------------
// The engine face
// ---------------------------------------------------------------------------

#[test]
fn folding_ranges_answer_one_fold_per_class_string() {
    let lsp = engine(DOC);
    let folds = lsp.folding_ranges(URI);

    // String a, string b, and the raw string c. The prose literal never
    // passes the extraction seam's own pre-filter (no `-`/`:` token in
    // any token), so it is not a class string and never folds.
    assert_eq!(
        folds.len(),
        3,
        "one fold per extracted class string; the prose literal stays out"
    );
    assert_eq!(folds[0].start.line, 1, "string a rides first (scan order)");
    assert_eq!(folds[1].start.line, 2, "string b rides second");
    assert_eq!(folds[2].start.line, 4, "the raw string rides last");
}

#[test]
fn the_fold_spans_the_string_content_document_order_kept() {
    let lsp = engine(DOC);
    let folds = lsp.folding_ranges(URI);

    // The fold covers the string's own content span (the extractor's
    // trimmed hit — the quotes stay the editor's territory): string a's
    // content starts at column 13 (`let a = "` is 13 columns) and runs
    // the content's length.
    assert_eq!(folds[0].start.character, 13, "the content's own start");
    assert_eq!(
        folds[0].end.character - folds[0].start.character,
        "p-4 bg-primary".len() as u32,
        "the content's own length"
    );
    assert_eq!(
        folds[0].start.line, folds[0].end.line,
        "single-line string a"
    );
}

#[test]
fn the_multi_line_raw_string_folds_across_its_lines() {
    let lsp = engine(DOC);
    let folds = lsp.folding_ranges(URI);

    // The raw string's content spans lines 4 and 5 — the fold the
    // client actually renders.
    assert_eq!(folds[2].start.line, 4);
    assert_eq!(folds[2].end.line, 5, "the fold spans the literal's lines");
    assert_eq!(
        folds[2].end.character, 20,
        "8 spaces + 12 columns of `items-center` — the half-open end past the content"
    );
}

#[test]
fn refused_candidates_fold_like_any_string() {
    let lsp = engine(DOC);
    let folds = lsp.folding_ranges(URI);

    // String b carries the refused `not-a-root` beside the lowered
    // sibling — the fold is textual, no compile verdict rides.
    assert_eq!(folds[1].start.line, 2);
    assert_eq!(
        folds[1].end.character - folds[1].start.character,
        "hover:bg-primary not-a-root".len() as u32,
        "the whole string content folds, refused candidates included"
    );
}

#[test]
fn the_unknown_uri_answers_empty() {
    let lsp = engine(DOC);
    assert!(
        lsp.folding_ranges("file:///not-open.rs").is_empty(),
        "the closed document answers empty"
    );
}

#[test]
fn the_answer_is_byte_true_utf16_pairs() {
    let lsp = engine(ACCENTED_DOC);
    let folds = lsp.folding_ranges(URI);

    assert_eq!(folds.len(), 1);
    // `bg-primary` starts after `héllo` — the é counts its TWO utf16
    // pairs, not its two bytes, so the column is 13, not 12.
    assert_eq!(folds[0].start.character, 13, "the accent's pairs counted");
    assert_eq!(folds[0].start.line, 1);
}

#[test]
fn the_answer_is_pure_over_the_document_map() {
    let lsp = engine(DOC);
    let first = lsp.folding_ranges(URI);
    let second = lsp.folding_ranges(URI);
    assert_eq!(first, second, "the same input answers the same output");
}

// ---------------------------------------------------------------------------
// The wire face
// ---------------------------------------------------------------------------

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
            "textDocument": {"uri": uri, "languageId": "rust", "version": 1, "text": text}
        }),
    )
}

fn handshake(session: &mut Session) {
    let handled = handle_message(
        session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    assert!(!handled.outgoing.is_empty(), "initialize answers");
}

fn folding_request(session: &mut Session, id: i32, uri: &str) -> serde_json::Value {
    let handled = handle_message(
        session,
        request(
            id,
            "textDocument/foldingRange",
            json!({"textDocument": {"uri": uri}}),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the folding request answers");
    };
    response
        .result
        .expect("the folding answer carries a result")
}

#[test]
fn wire_folding_ranges_answer_the_folds() {
    let mut session = Session::new(Default::default());
    handshake(&mut session);
    let _ = handle_message(&mut session, did_open(URI, DOC));

    let result = folding_request(&mut session, 1, URI);
    let folds = result.as_array().expect("the folds ride the array form");
    assert_eq!(folds.len(), 3, "the engine's answer, wire-shaped");
    assert_eq!(folds[0]["startLine"], 1);
    assert_eq!(folds[2]["endLine"], 5, "the raw string's fold spans lines");
    for fold in folds {
        assert!(
            fold.get("startCharacter").is_some(),
            "the columns ride: the client folds the exact span"
        );
        assert!(
            fold.get("kind").is_none(),
            "no kind rides: a plain region fold, the client's own default"
        );
    }
}

#[test]
fn wire_folding_ranges_unknown_uri_answers_the_empty_array() {
    let mut session = Session::new(Default::default());
    handshake(&mut session);
    let _ = handle_message(&mut session, did_open(URI, DOC));

    let result = folding_request(&mut session, 1, "file:///not-open.rs");
    assert_eq!(
        result.as_array().map(Vec::len),
        Some(0),
        "the closed document answers the protocol's empty array"
    );
}

#[test]
fn initialize_advertises_folding() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("initialize answers a response");
    };
    let result = response.result.expect("the capabilities carry a result");
    assert_eq!(
        result["capabilities"]["foldingRangeProvider"], true,
        "the plain true form: the answer is the engine's own walk"
    );
}

#[test]
fn wire_pre_handshake_folding_ranges_quiet_drops() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/foldingRange",
            json!({"textDocument": {"uri": URI}}),
        ),
    );
    assert!(
        handled.outgoing.is_empty(),
        "the pre-handshake gate rides before any answer"
    );
}

#[test]
fn wire_malformed_params_answer_null() {
    let mut session = Session::new(Default::default());
    handshake(&mut session);
    let handled = handle_message(
        &mut session,
        request(1, "textDocument/foldingRange", json!({})),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the malformed request still answers");
    };
    assert_eq!(
        response.result,
        Some(serde_json::Value::Null),
        "the protocol's own empty answer"
    );
}
