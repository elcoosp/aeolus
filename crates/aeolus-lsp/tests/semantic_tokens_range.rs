// ---------------------------------------------------------------------------
// The semantic-tokens range contracts (LSP 3.16's
// `textDocument/semanticTokens/range` — the highlighter feed's
// SELECTION-SCOPED form): the editor requests the legend for a RANGE —
// the visible viewport, a selection, a line — and the answer is the
// full-token walk restricted to the candidates the requested range
// touches. A candidate whose span INTERSECTS the half-open byte range
// rides its token (a partial selection still names the token); a
// candidate wholly outside stays out AND never compiles — the walk's
// win the bench arm pins beside. The whole-document range is the
// `semanticTokens/full` answer byte for byte (the differential pin the
// range-formatting face keeps), the wire answer re-baselines the delta
// encoding per response (each array encodes from its own start), and a
// reversed or empty range answers the honest empty (never an invented
// legend). The malformed params and the pre-handshake request ride the
// state machine's own quiet vocabulary.
//
// Written FIRST (the RED spec).
// ---------------------------------------------------------------------------

use aeolus_lsp::{AeolusEngine, EngineConfig, EngineSemanticKind, Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// The document the range answers over: one lowered string, one themed
/// string with variant prefixes, one refused string — the full walk's
/// own three verdicts, at three separate lines the range can slice.
const DOC: &str = concat!(
    "fn card() {\n",
    "    let a = \"p-4 flex-col gap-2 rounded-md\";\n",
    "    let b = \"hover:bg-primary/90 dark:shadow-lg\";\n",
    "    let bad = \"not-a-root\";\n",
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
        }
    })
}

fn range_tokens(session: &mut Session, id: i32, params: serde_json::Value) -> serde_json::Value {
    let handled = handle_message(
        session,
        request(id, "textDocument/semanticTokens/range", params),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the range request answers");
    };
    response.result.expect("the range answer carries a result")
}

/// The wire's delta encoding, decoded back onto absolute positions (the
/// LSP's own algorithm, run in reverse over the protocol's FLAT word
/// array — five u32s per token, the deltas the full arm's wire test
/// pins): each token's line is the previous token's line plus the delta
/// (the first relative to row 0), the column relative to the previous
/// START on the same line.
fn decode(result: &serde_json::Value) -> Vec<(u32, u32, u32, usize)> {
    let data = result["data"]
        .as_array()
        .expect("the flat delta array rides");
    assert_eq!(
        data.len() % 5,
        0,
        "the protocol's flat encoding: five words per token"
    );
    let words: Vec<u32> = data
        .iter()
        .map(|value| value.as_u64().expect("a u32 word") as u32)
        .collect();
    let mut out = Vec::new();
    let mut prev_line = 0u32;
    let mut prev_character = 0u32;
    for group in words.chunks_exact(5) {
        let (delta_line, delta_start) = (group[0], group[1]);
        let line = prev_line + delta_line;
        let character = if delta_line == 0 {
            prev_character + delta_start
        } else {
            delta_start
        };
        out.push((line, character, group[2], group[3] as usize));
        prev_line = line;
        prev_character = character;
    }
    out
}

#[test]
fn initialize_advertises_the_semantic_tokens_range_capability() {
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
        result["capabilities"]["semanticTokensProvider"]["range"],
        json!(true),
        "the selection-scoped highlighter face rides the capability"
    );
}

#[test]
fn the_whole_document_range_matches_the_full_walk() {
    let mut session = wire_session();
    let result = range_tokens(
        &mut session,
        1,
        range_params(0, 0, DOC.lines().count() as u32, 0),
    );
    // The parity pin: the whole-document range IS the full walk's own
    // answer, byte for byte (the same array, the same deltas).
    let mut engine = AeolusEngine::new(EngineConfig::default());
    engine.open(URI, DOC, 1);
    let expected = engine.semantic_tokens(URI);
    let decoded = decode(&result);
    assert_eq!(
        decoded.len(),
        expected.len(),
        "the same candidates ride the restricted walk"
    );
    for ((line, character, length, kind), token) in decoded.iter().zip(&expected) {
        assert_eq!(*line, token.range.start.line, "the same row rides");
        assert_eq!(
            *character, token.range.start.character,
            "the same column rides"
        );
        assert_eq!(
            *length,
            token.range.end.character - token.range.start.character,
            "the same length rides"
        );
        let legend = match token.kind {
            EngineSemanticKind::Lowered => 0,
            EngineSemanticKind::Refused => 1,
        };
        assert_eq!(*kind, legend, "the same legend index rides");
    }
}

#[test]
fn a_range_over_one_string_answers_only_that_strings_tokens() {
    let mut session = wire_session();
    // Line 1 only (the first lowered string): the other lines' candidates
    // stay out of the answer.
    let result = range_tokens(&mut session, 1, range_params(1, 0, 2, 0));
    let decoded = decode(&result);
    assert_eq!(decoded.len(), 4, "four candidates on line 1, four tokens");
    assert!(
        decoded.iter().all(|(line, _, _, _)| *line == 1),
        "every token rides line 1"
    );
}

#[test]
fn a_partial_overlap_still_names_the_token() {
    let mut session = wire_session();
    // The selection covers `flex-col`'s tail through `gap-2`'s head
    // (columns 18..27: `p-4` at 13..16 and `rounded-md` at 32..41 stay
    // out) — every candidate whose span touches the range rides its
    // token, whole (the highlighter never emits a fragment).
    let result = range_tokens(&mut session, 1, range_params(1, 18, 1, 27));
    let decoded = decode(&result);
    assert_eq!(decoded.len(), 2, "the two intersecting candidates ride");
    assert_eq!(decoded[0].1, 17, "flex-col rides whole");
    assert_eq!(decoded[1].1, 26, "gap-2 rides whole");
}

#[test]
fn a_range_outside_any_string_answers_empty() {
    let mut session = wire_session();
    // The blank span between `fn card() {` and the first string.
    let result = range_tokens(&mut session, 1, range_params(0, 0, 1, 5));
    let decoded = decode(&result);
    assert_eq!(
        decoded.len(),
        0,
        "no candidate the range touches — no token invented"
    );
}

#[test]
fn a_reversed_range_answers_empty() {
    let mut session = wire_session();
    let result = range_tokens(&mut session, 1, range_params(2, 10, 1, 5));
    let decoded = decode(&result);
    assert_eq!(
        decoded.len(),
        0,
        "an end before its start is no selection — the honest empty"
    );
}

#[test]
fn a_range_over_the_refused_string_answers_the_refused_kind() {
    let mut session = wire_session();
    // Line 3 carries the refused candidate: the legend's classification
    // rides the restricted walk exactly like the full one (the compile
    // verdict, no parallel matcher).
    let result = range_tokens(&mut session, 1, range_params(3, 0, 4, 0));
    let decoded = decode(&result);
    assert_eq!(decoded.len(), 1, "one refused candidate, one token");
    assert_eq!(
        decoded[0].3, 1,
        "the refused candidate rides legend index 1"
    );
}

#[test]
fn an_unknown_uri_answers_empty() {
    let mut session = wire_session();
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/semanticTokens/range",
            json!({
                "textDocument": {"uri": "file:///not-open.rs"},
                "range": {"start": {"line": 0, "character": 0}, "end": {"line": 9, "character": 0}}
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
        result["data"].as_array().map(Vec::len),
        Some(0),
        "a closed document answers the honest empty"
    );
}

#[test]
fn the_wire_answer_rebaselines_the_delta_encoding() {
    let mut session = wire_session();
    // A mid-document range: the answer's FIRST token carries its own
    // absolute line as the delta (the array encodes from its own start,
    // row 0 — the full walk's contract carried over, not the document's).
    let result = range_tokens(&mut session, 1, range_params(2, 0, 3, 0));
    let data = result["data"]
        .as_array()
        .expect("the flat delta array rides");
    assert_eq!(
        data.len(),
        10,
        "line 2's two candidates ride, five words each"
    );
    assert_eq!(
        data[0].as_u64(),
        Some(2),
        "the restricted answer's baseline is the array's own start"
    );
}

#[test]
fn the_pre_handshake_gate_holds() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/semanticTokens/range",
            range_params(0, 0, 9, 0),
        ),
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
        request(1, "textDocument/semanticTokens/range", json!(null)),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the malformed request still answers");
    };
    assert!(
        response.result.is_none() || response.result == Some(json!(null)),
        "malformed params answer the protocol's own empty"
    );
}
