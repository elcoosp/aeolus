//! The cross-document references answer (the wire's
//! `textDocument/references` — the engine's first cross-document
//! question): the occurrences of the cursor's candidate across EVERY
//! open document, the highlights' textual equality riding the whole
//! document map. Written FIRST (the RED spec): the missing engine
//! method's compile failure WAS the capture.
//!
//! The suite pins the answer's house rules beside the answers:
//!
//! * the TEXTUAL identity (the highlights' own contract, no compile
//!   verdict riding): a refused class's occurrences answer exactly
//!   like a lowered one's, and a variant-prefixed sibling
//!   (`hover:flex-col` beside `flex-col`) is a DIFFERENT spelling;
//! * the whole document map is the answer's scope: occurrences from
//!   every open document, the CLOSED one dropping out, an unknown
//!   cursor URI and a cursor outside any candidate answering empty;
//! * DETERMINISM over the hash map: the documents ride URI order
//!   (sorted, whatever the open order was), the candidates document
//!   order within each — the answer is pure over the map;
//! * byte-true UTF-16 ranges (the multi-byte neighbour counts its
//!   pairs, not its bytes);
//! * the wire half: the request answers `Location` quads (the URI the
//!   protocol names, the range it jumps to), the pre-handshake gate
//!   quiet-drops, malformed params answer the protocol's own Null,
//!   and the `includeDeclaration` context rides unanswered (a class
//!   spelling has no declaration site among open documents — every
//!   occurrence is a reference, the honest boundary the rustdoc
//!   carries).

use aeolus_lsp::{AeolusEngine, EngineConfig, EngineReference, Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

const DOC_A: &str = "fn card() {\n    let a = \"flex-col\";\n    let b = \"hover:flex-col\";\n}\n";
const DOC_B: &str =
    "fn page() {\n    let a = \"p-4 flex-col not-a-root\";\n    let b = \"hé flex-col\";\n}\n";
const DOC_C: &str = "fn list() {\n    let a = \"flex-col\";\n}\n";

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

fn references_params(uri: &str, line: u32, character: u32) -> serde_json::Value {
    json!({
        "textDocument": {"uri": uri},
        "position": {"line": line, "character": character},
        "context": {"includeDeclaration": true},
    })
}

fn initialize(session: &mut Session) {
    let handled = handle_message(
        session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    assert_eq!(handled.outgoing.len(), 1, "initialize answers its result");
}

/// The `flex-col` spelling's cursor position in a fixture document —
/// the first occurrence's first character.
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
    engine.open("file:///a.rs", DOC_A, 1);
    engine.open("file:///b.rs", DOC_B, 1);
    engine.open("file:///c.rs", DOC_C, 1);
    engine
}

// ---------------------------------------------------------------------------
// The engine-level pins
// ---------------------------------------------------------------------------

#[test]
fn cross_document_occurrences_ride() {
    let engine = wired_engine();
    let (line, character) = cursor_of(DOC_B, "flex-col");
    let references = engine.references("file:///b.rs", line, character);
    // Three documents: a.rs one bare occurrence, b.rs two (the queried
    // line-1 one and the `hé `-neighboured line-2 one), c.rs one —
    // a.rs's `hover:flex-col` sibling is a DIFFERENT spelling, no ride.
    assert_eq!(references.len(), 4, "every open document rides");
    assert_eq!(references[0].uri, "file:///a.rs");
    assert_eq!(references[0].range.start.line, 1);
    assert_eq!(references[1].uri, "file:///b.rs");
    assert_eq!(references[2].uri, "file:///b.rs");
    assert_eq!(references[3].uri, "file:///c.rs");
}

#[test]
fn determinism_is_uri_order_not_map_order() {
    // The map is a hash map: open the documents in REVERSE URI order —
    // the answer still rides URI order (the purity pin).
    let mut engine = AeolusEngine::new(EngineConfig::default());
    engine.open("file:///c.rs", DOC_C, 1);
    engine.open("file:///a.rs", DOC_A, 1);
    engine.open("file:///b.rs", DOC_B, 1);
    let (line, character) = cursor_of(DOC_A, "flex-col");
    let references = engine.references("file:///a.rs", line, character);
    let uris: Vec<&str> = references.iter().map(|r| r.uri.as_str()).collect();
    assert_eq!(
        uris,
        [
            "file:///a.rs",
            "file:///b.rs",
            "file:///b.rs",
            "file:///c.rs"
        ]
    );
}

#[test]
fn the_cursor_occurrence_rides_the_answer() {
    let engine = wired_engine();
    let (line, character) = cursor_of(DOC_C, "flex-col");
    let references = engine.references("file:///c.rs", line, character);
    let here = references
        .iter()
        .find(|r| r.uri == "file:///c.rs")
        .expect("the queried document rides");
    assert_eq!(here.range.start.line, 1, "the cursor's own occurrence");
}

#[test]
fn variant_prefixed_sibling_is_a_different_spelling() {
    let engine = wired_engine();
    // DOC_A's second string carries `hover:flex-col` — a different raw
    // text, no reference (the highlights' own equality rule).
    let (line, character) = cursor_of(DOC_A, "flex-col");
    let references = engine.references("file:///a.rs", line, character);
    let in_a: Vec<&EngineReference> = references
        .iter()
        .filter(|r| r.uri == "file:///a.rs")
        .collect();
    assert_eq!(
        in_a.len(),
        1,
        "the hover:-prefixed sibling is a different spelling"
    );
    assert_eq!(in_a[0].range.start.line, 1);
    assert_eq!(in_a[0].range.start.character, 13);
}

#[test]
fn refused_class_references_ride_too() {
    let engine = wired_engine();
    let (line, character) = cursor_of(DOC_B, "not-a-root");
    let references = engine.references("file:///b.rs", line, character);
    assert_eq!(references.len(), 1, "the refusal's own occurrence");
    assert_eq!(references[0].uri, "file:///b.rs");
}

#[test]
fn unknown_uri_and_outside_candidate_answer_empty() {
    let engine = wired_engine();
    assert!(engine.references("file:///missing.rs", 0, 0).is_empty());
    assert!(engine.references("file:///a.rs", 0, 0).is_empty());
}

#[test]
fn closed_document_drops_out() {
    let mut engine = wired_engine();
    engine.close("file:///b.rs");
    let (line, character) = cursor_of(DOC_A, "flex-col");
    let references = engine.references("file:///a.rs", line, character);
    assert!(
        references.iter().all(|r| r.uri != "file:///b.rs"),
        "the closed document's occurrences drop"
    );
}

#[test]
fn ranges_are_byte_true_utf16_pairs() {
    let engine = wired_engine();
    // DOC_B line 2: `    let b = "hé flex-col";` — the accented
    // neighbour counts its UTF-16 pairs, not its bytes: the expected
    // column derives from the fixture's own pair count.
    let (line, character) = cursor_of(DOC_B, "flex-col");
    let references = engine.references("file:///b.rs", line, character);
    let hinted = references
        .iter()
        .find(|r| r.uri == "file:///b.rs" && r.range.start.line == 2)
        .expect("the multi-byte line's occurrence rides");
    let line_text = DOC_B.split('\n').nth(2).expect("the fixture holds");
    let expected_start: usize = line_text
        .find("flex-col")
        .map(|offset| {
            line_text[..offset]
                .chars()
                .map(char::len_utf16)
                .sum::<usize>()
        })
        .expect("the spelling rides the line");
    assert_eq!(hinted.range.start.character as usize, expected_start);
    assert_eq!(
        hinted.range.end.character as usize,
        expected_start + "flex-col".chars().map(char::len_utf16).sum::<usize>(),
    );
}

// ---------------------------------------------------------------------------
// The wire-level pins
// ---------------------------------------------------------------------------

#[test]
fn wire_references_answer_locations() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let _ = handle_message(&mut session, did_open("file:///a.rs", DOC_A));
    let _ = handle_message(&mut session, did_open("file:///c.rs", DOC_C));
    let (line, character) = cursor_of(DOC_A, "flex-col");
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/references",
            references_params("file:///a.rs", line, character),
        ),
    );
    assert_eq!(handled.outgoing.len(), 1, "the response rides");
    let result = match &handled.outgoing[0] {
        Message::Response(response) => response.result.clone(),
        other => panic!("a response rode, got {other:?}"),
    };
    let locations = result.expect("the locations answer");
    let locations = locations.as_array().expect("an array of locations");
    assert_eq!(
        locations.len(),
        2,
        "a.rs and c.rs carry one occurrence each"
    );
    assert_eq!(locations[0]["uri"], "file:///a.rs");
    assert_eq!(locations[0]["range"]["start"]["line"], 1);
    assert_eq!(locations[1]["uri"], "file:///c.rs");
}

#[test]
fn wire_pre_handshake_references_quiet_drops() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/references",
            references_params("file:///a.rs", 1, 8),
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
        request(1, "textDocument/references", json!({"position": "nope"})),
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
fn wire_refused_neighbour_rides_the_same_shape() {
    // The textual contract at the wire: a refused class's occurrence
    // answers exactly like a lowered one's (no compile verdict riding).
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let _ = handle_message(&mut session, did_open("file:///b.rs", DOC_B));
    let (line, character) = cursor_of(DOC_B, "not-a-root");
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/references",
            references_params("file:///b.rs", line, character),
        ),
    );
    let result = match &handled.outgoing[0] {
        Message::Response(response) => response.result.clone(),
        other => panic!("a response rode, got {other:?}"),
    };
    let locations = result.expect("the locations answer");
    assert_eq!(
        locations.as_array().map(Vec::len),
        Some(1),
        "the refusal's own occurrence"
    );
}
