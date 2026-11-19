// ---------------------------------------------------------------------------
// The range-formatting contracts (LSP 3.12's `textDocument/rangeFormatting`
// — the sort seam's SELECTION-SCOPED form): the editor formats a RANGE —
// a selection, a line, the block the cursor sits in — and the answer is
// the document formatting face's own walk, restricted to the class
// strings the range touches. A string whose span INTERSECTS the
// half-open byte range rides its edit (the whole string reorders — the
// sort seam's own canonical join; a selection over part of a string
// still names the string); a string wholly outside the range stays out.
// An already-canonical string answers nothing wherever the range sits
// (the no-op contract a selection-scoped format-on-save rides), the
// whole-document range is the formatting request's own answer byte for
// byte, and a reversed or empty range answers the honest empty (never an
// invented walk). The malformed params and the pre-handshake request
// ride the state machine's own quiet vocabulary.
//
// Written FIRST (the RED spec).
// ---------------------------------------------------------------------------

use aeolus_lsp::{AeolusEngine, EngineConfig, Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// The document the range answers over: two out-of-order class strings
/// and one already-canonical string — the same battery the
/// `textDocument/formatting` contracts pin (the whole-document range
/// must answer byte for byte what that face answers).
const DOC: &str = concat!(
    "fn card() {\n",
    "    let a = \"rounded-md p-4 flex-col gap-2\";\n",
    "    let b = \"hover:bg-primary flex\";\n",
    "    let c = \"flex-col items-center\";\n",
    "}\n",
);

const URI: &str = "file:///card.rs";

fn notification(method: &str, params: serde_json::Value) -> Message {
    Message::Notification(Notification {
        method: method.to_string(),
        params,
    })
}

fn request(id: i32, method: &str, params: serde_json::Value) -> Message {
    Message::Request(Request {
        id: RequestId::from(id),
        method: method.to_string(),
        params,
    })
}

fn initialize(session: &mut Session) {
    let handled = handle_message(
        session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    assert!(!handled.outgoing.is_empty(), "initialize answers");
}

fn wire_session() -> Session {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let _ = handle_message(
        &mut session,
        notification(
            "textDocument/didOpen",
            json!({
                "textDocument": {"uri": URI, "languageId": "rust", "version": 1, "text": DOC}
            }),
        ),
    );
    session
}

fn range_params(
    start_line: u32,
    start_char: u32,
    end_line: u32,
    end_char: u32,
) -> serde_json::Value {
    json!({
        "textDocument": {"uri": URI},
        "range": {
            "start": {"line": start_line, "character": start_char},
            "end": {"line": end_line, "character": end_char}
        },
        "options": {"tabSize": 4, "insertSpaces": true}
    })
}

fn range_edits(session: &mut Session, id: i32, params: serde_json::Value) -> serde_json::Value {
    let handled = handle_message(session, request(id, "textDocument/rangeFormatting", params));
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the range request answers");
    };
    response.result.expect("the range answer carries a result")
}

#[test]
fn initialize_advertises_the_range_formatting_provider() {
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
        result["capabilities"]["documentRangeFormattingProvider"],
        json!(true),
        "the selection-scoped sort face rides the capability"
    );
}

#[test]
fn the_whole_document_range_matches_the_formatting_answer() {
    let mut session = wire_session();
    let result = range_edits(
        &mut session,
        1,
        range_params(0, 0, DOC.lines().count() as u32, 0),
    );
    let edits = result.as_array().expect("an array of edits");
    // The parity pin: the whole-document range IS the formatting
    // request's own answer, byte for byte.
    let mut engine = AeolusEngine::new(EngineConfig::default());
    engine.open(URI, DOC, 1);
    let expected = engine.format(URI);
    assert_eq!(edits.len(), expected.len(), "the same strings ride");
    for (edit, source) in edits.iter().zip(&expected) {
        assert_eq!(
            edit["newText"], source.new_text,
            "the canonical join is the sort seam's own"
        );
        assert_eq!(
            edit["range"]["start"]["line"],
            json!(source.range.start.line),
            "the edit rides the same string"
        );
    }
}

#[test]
fn a_range_over_one_string_answers_one_edit() {
    let mut session = wire_session();
    // Line 1 only (the first out-of-order string): the other strings
    // stay out of the answer.
    let result = range_edits(&mut session, 1, range_params(1, 0, 2, 0));
    let edits = result.as_array().expect("an array of edits");
    assert_eq!(edits.len(), 1, "one string in range, one edit");
    assert_eq!(edits[0]["range"]["start"]["line"], json!(1));
}

#[test]
fn a_partial_overlap_still_sorts_the_whole_string() {
    let mut session = wire_session();
    // The selection covers the string's middle (`p-4 flex-col`) — the
    // string's span intersects the range, so the WHOLE string rides its
    // edit (the sort seam reorders the string, never a fragment).
    let result = range_edits(&mut session, 1, range_params(1, 24, 1, 35));
    let edits = result.as_array().expect("an array of edits");
    assert_eq!(edits.len(), 1, "a partial selection still names the string");
    // And the edit's range is the string's OWN content span — the
    // formatting face's byte-true contract, carried over.
    assert_eq!(edits[0]["range"]["start"]["character"], json!(13));
}

#[test]
fn a_range_over_two_strings_answers_two_edits() {
    let mut session = wire_session();
    let result = range_edits(&mut session, 1, range_params(1, 0, 3, 0));
    let edits = result.as_array().expect("an array of edits");
    assert_eq!(edits.len(), 2, "both out-of-order strings ride");
}

#[test]
fn a_range_over_canonical_strings_answers_empty() {
    let mut session = wire_session();
    // Line 3 carries the already-canonical string: the no-op contract a
    // selection-scoped format rides — an already-canonical string
    // answers nothing wherever the range sits.
    let only_canonical = range_edits(&mut session, 1, range_params(3, 0, 4, 0));
    let edits = only_canonical.as_array().expect("an array of edits");
    assert_eq!(
        edits.len(),
        0,
        "an already-canonical string answers nothing wherever the range sits"
    );
}

#[test]
fn a_range_outside_any_string_answers_empty() {
    let mut session = wire_session();
    // The blank span between `fn card() {` and the first string.
    let result = range_edits(&mut session, 1, range_params(0, 0, 1, 5));
    let edits = result.as_array().expect("an array of edits");
    assert_eq!(
        edits.len(),
        0,
        "no string the range touches — no edit invented"
    );
}

#[test]
fn a_reversed_range_answers_empty() {
    let mut session = wire_session();
    let result = range_edits(&mut session, 1, range_params(2, 10, 1, 5));
    let edits = result.as_array().expect("an array of edits");
    assert_eq!(
        edits.len(),
        0,
        "an end before its start is no selection — the honest empty"
    );
}

#[test]
fn an_unknown_uri_answers_empty() {
    let mut session = wire_session();
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/rangeFormatting",
            json!({
                "textDocument": {"uri": "file:///not-open.rs"},
                "range": {"start": {"line": 0, "character": 0}, "end": {"line": 9, "character": 0}},
                "options": {"tabSize": 4, "insertSpaces": true}
            }),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the request answers");
    };
    let result = response
        .result
        .expect("the closed document answers a result");
    assert_eq!(
        result.as_array().map(Vec::len),
        Some(0),
        "a closed document answers the honest empty"
    );
}

#[test]
fn the_pre_handshake_gate_holds() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(1, "textDocument/rangeFormatting", range_params(0, 0, 9, 0)),
    );
    assert!(
        handled.outgoing.is_empty(),
        "no request answers before the initialize handshake"
    );
}

#[test]
fn malformed_params_answer_null() {
    let mut session = wire_session();
    let handled = handle_message(
        &mut session,
        request(1, "textDocument/rangeFormatting", json!(null)),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the malformed request still answers");
    };
    assert!(
        response.result.is_none() || response.result == Some(json!(null)),
        "malformed params answer the protocol's own empty"
    );
}
