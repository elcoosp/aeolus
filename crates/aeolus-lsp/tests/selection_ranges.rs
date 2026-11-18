//! The selection-range answer (the wire's
//! `textDocument/selectionRange` — the editor's expand-selection walk):
//! one parent chain per requested position — the cursor's candidate
//! span, its parent the class string's own content span — answered
//! from the SAME locator the hover, the highlights, the references and
//! the rename walk. Written FIRST (the RED spec): the missing engine
//! method's compile failure WAS the capture.
//!
//! The suite pins the answer's house rules beside the answers:
//!
//! * the 1:1 contract (the protocol's own): one answer per requested
//!   position, request order kept — a multi-cursor request rides;
//! * the chain: a cursor inside a candidate answers the candidate's
//!   byte-true span with the string's content span as its parent; a
//!   cursor inside the string but between classes answers the content
//!   span alone (no candidate level, no invented inner range); a
//!   cursor outside any string answers the protocol's own minimal
//!   range — the position's zero-width caret, no parent;
//! * byte-true UTF-16 ranges (the multi-byte neighbour counts its
//!   pairs, not its bytes);
//! * an unknown URI answers the same 1:1 zero-width carêts (the
//!   contract holds wherever the cursor was);
//! * the wire half: the request answers the nested quads (the parent
//!   chain riding the protocol's own `parent` key), the capability
//!   advertises, the pre-handshake gate quiet-drops, malformed params
//!   answer the protocol's own Null, and the answer's length equals
//!   the request's at the wire.

use aeolus_lsp::{AeolusEngine, EngineConfig, EnginePosition, Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// The fixture's (line, character) pairs onto the engine's typed
/// position (the answer's own units, UTF-16 pairs).
fn pos(line: u32, character: u32) -> EnginePosition {
    EnginePosition { line, character }
}

const DOC: &str = "fn card() {\n    let a = \"p-4 flex-col hé\";\n}\n";

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

fn selection_params(uri: &str, positions: &[(u32, u32)]) -> serde_json::Value {
    let positions: Vec<serde_json::Value> = positions
        .iter()
        .map(|(line, character)| json!({"line": line, "character": character}))
        .collect();
    json!({
        "textDocument": {"uri": uri},
        "positions": positions,
    })
}

fn initialize(session: &mut Session) {
    let handled = handle_message(
        session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    assert_eq!(handled.outgoing.len(), 1, "initialize answers its result");
}

/// The needle's 0-based (line, character) in a fixture document — the
/// cursor position a client sends on expand-selection.
fn cursor_of(doc: &str, needle: &str) -> (u32, u32) {
    let offset = doc.find(needle).expect("the fixture holds") + 2;
    let line = doc[..offset].bytes().filter(|b| *b == b'\n').count() as u32;
    let character = doc[..offset]
        .rsplit('\n')
        .next()
        .expect("a prefix has a last line")
        .chars()
        .count() as u32;
    (line, character)
}

fn wired_engine() -> AeolusEngine {
    let mut engine = AeolusEngine::new(EngineConfig::default());
    engine.open("file:///card.rs", DOC, 1);
    engine
}

// ---------------------------------------------------------------------------
// The engine-level pins
// ---------------------------------------------------------------------------

#[test]
fn the_candidate_chain_answers_two_levels() {
    let engine = wired_engine();
    // The cursor inside `flex-col`: the candidate's own span, its
    // parent the string's whole content span (the quotes stay out —
    // the extractor's own trimmed hit, the editor's own territory).
    let (line, character) = cursor_of(DOC, "flex-col");
    let answers = engine.selection_ranges("file:///card.rs", &[pos(line, character)]);
    assert_eq!(answers.len(), 1, "one answer per position");
    let selection = &answers[0];
    assert_eq!(selection.range.start.line, 1);
    assert_eq!(selection.range.start.character, 17, "flex-col's column");
    assert_eq!(selection.range.end.character, 25, "flex-col's end");
    let parent = selection
        .parent
        .as_ref()
        .expect("the content span rides as the parent");
    assert_eq!(parent.range.start.character, 13, "the content's open");
    assert_eq!(parent.range.end.character, 28, "the content's close");
    assert!(
        parent.parent.is_none(),
        "the content span is the chain's head"
    );
}

#[test]
fn between_classes_answers_the_content_span_alone() {
    let engine = wired_engine();
    // The cursor on the space between the two classes: no candidate
    // level (no invented inner range), the content span answering as
    // the chain's head.
    let line_text = DOC.split('\n').nth(1).expect("the fixture holds");
    let space_column: usize = line_text
        .find("p-4 ")
        .map(|offset| {
            line_text[..offset + 3]
                .chars()
                .map(char::len_utf16)
                .sum::<usize>()
        })
        .expect("the gap rides the line");
    let answers = engine.selection_ranges("file:///card.rs", &[pos(1, space_column as u32)]);
    let selection = &answers[0];
    assert_eq!(selection.range.start.character, 13, "the content's open");
    assert_eq!(selection.range.end.character, 28, "the content's close");
    assert!(
        selection.parent.is_none(),
        "no candidate level above the content span"
    );
}

#[test]
fn outside_any_string_answers_the_zero_width_caret() {
    let engine = wired_engine();
    // The cursor on `fn`: outside any class string — the protocol's
    // own minimal answer, the position's zero-width caret.
    let answers = engine.selection_ranges("file:///card.rs", &[pos(0, 2)]);
    let selection = &answers[0];
    assert_eq!(selection.range.start.line, 0);
    assert_eq!(selection.range.start.character, 2);
    assert_eq!(selection.range.end.character, 2, "zero-width");
    assert!(selection.parent.is_none(), "no chain to ride");
}

#[test]
fn multi_cursor_positions_ride_1to1_in_request_order() {
    let engine = wired_engine();
    let (line, character) = cursor_of(DOC, "flex-col");
    let answers = engine.selection_ranges(
        "file:///card.rs",
        &[pos(line, character), pos(0, 2), pos(line, character + 2)],
    );
    assert_eq!(
        answers.len(),
        3,
        "one answer per requested position, request order kept"
    );
    assert_eq!(answers[0].range.start.character, 17, "the candidate chain");
    assert_eq!(answers[1].range.start.character, 2, "the caret");
    // The third cursor rides TWO columns into the same spelling — the
    // candidate span's own start answers (the chain keys on the span,
    // not the cursor's offset inside it).
    assert_eq!(answers[2].range.start.character, 17, "the candidate chain");
    assert!(answers[0].parent.is_some());
    assert!(answers[2].parent.is_some());
}

#[test]
fn unknown_uri_answers_the_1to1_zero_width_carats() {
    let engine = wired_engine();
    let answers = engine.selection_ranges("file:///missing.rs", &[pos(0, 0), pos(3, 1)]);
    assert_eq!(answers.len(), 2, "the 1:1 contract holds");
    assert!(answers.iter().all(|answer| answer.parent.is_none()));
    assert_eq!(answers[0].range.start.character, 0);
    assert_eq!(answers[0].range.end.character, 0, "zero-width");
    assert_eq!(answers[1].range.start.character, 1);
}

#[test]
fn ranges_are_byte_true_utf16_pairs() {
    let engine = wired_engine();
    // The cursor ON the accented neighbour `é`: the content span's
    // END column counts the accent's UTF-16 pairs, not its bytes —
    // the expected column derives from the fixture's own pair count.
    let (line, character) = cursor_of(DOC, " hé");
    assert_eq!(character as usize, 27, "the accent's own column");
    let answers = engine.selection_ranges("file:///card.rs", &[pos(line, character)]);
    let selection = &answers[0];
    let line_text = DOC.split('\n').nth(1).expect("the fixture holds");
    let string_text = line_text
        .split('"')
        .nth(1)
        .expect("the class string rides the line");
    let expected_end: usize = line_text
        .find(string_text)
        .map(|offset| {
            line_text[..offset + string_text.len()]
                .chars()
                .map(char::len_utf16)
                .sum::<usize>()
        })
        .expect("the content rides the line");
    let parent = selection
        .parent
        .as_ref()
        .expect("the candidate's parent is the content span");
    assert_eq!(parent.range.end.character as usize, expected_end);
}

// ---------------------------------------------------------------------------
// The wire-level pins
// ---------------------------------------------------------------------------

#[test]
fn wire_selection_ranges_answer_the_nested_quads() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let _ = handle_message(&mut session, did_open("file:///card.rs", DOC));
    let (line, character) = cursor_of(DOC, "flex-col");
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/selectionRange",
            selection_params("file:///card.rs", &[(line, character), (0, 2)]),
        ),
    );
    assert_eq!(handled.outgoing.len(), 1, "the response rides");
    let result = match &handled.outgoing[0] {
        Message::Response(response) => response.result.clone(),
        other => panic!("a response rode, got {other:?}"),
    };
    let ranges = result.expect("the selection ranges answer");
    let ranges = ranges.as_array().expect("an array of ranges");
    assert_eq!(ranges.len(), 2, "the 1:1 contract at the wire");
    let candidate = &ranges[0];
    assert_eq!(candidate["range"]["start"]["character"], 17);
    let parent = candidate
        .get("parent")
        .expect("the parent chain rides the wire");
    assert_eq!(parent["range"]["start"]["character"], 13);
    assert!(parent.get("parent").is_none(), "the chain's head");
    let caret = &ranges[1];
    assert_eq!(caret["range"]["start"]["character"], 2);
    assert_eq!(caret["range"]["end"]["character"], 2, "zero-width");
    assert!(caret.get("parent").is_none(), "no chain to ride");
}

#[test]
fn wire_initialize_advertises_selection_ranges() {
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
        result["capabilities"]["selectionRangeProvider"], true,
        "the selection-range capability advertises (the plain true form): {result}"
    );
}

#[test]
fn wire_pre_handshake_selection_ranges_quiet_drop() {
    let mut session = Session::new(Default::default());
    let (line, character) = cursor_of(DOC, "flex-col");
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/selectionRange",
            selection_params("file:///card.rs", &[(line, character)]),
        ),
    );
    assert!(handled.outgoing.is_empty(), "the pre-handshake gate holds");
}

#[test]
fn wire_malformed_params_answer_null() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/selectionRange",
            json!({"positions": "nope"}),
        ),
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
fn wire_positions_length_matches_the_answer() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let _ = handle_message(&mut session, did_open("file:///card.rs", DOC));
    let (line, character) = cursor_of(DOC, "flex-col");
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/selectionRange",
            selection_params(
                "file:///card.rs",
                &[(0, 2), (line, character), (line + 1, 4)],
            ),
        ),
    );
    let result = match &handled.outgoing[0] {
        Message::Response(response) => response.result.clone(),
        other => panic!("a response rode, got {other:?}"),
    };
    let ranges = result.expect("the selection ranges answer");
    let ranges = ranges.as_array().expect("an array of ranges");
    assert_eq!(ranges.len(), 3, "one answer per requested position");
}
