// ---------------------------------------------------------------------------
// The linked-editing-range contracts (the editor's simultaneous-edit
// face): every textual occurrence of the cursor's candidate IN THE
// REQUESTING document answers as a range the client edits together —
// the highlights' textual equality rendered as the wire's
// `LinkedEditingRanges`. The cross-document scope stays the RENAME
// face's (one face, one scope; linked editing is same-file), the
// identity stays TEXTUAL, the guards keep their shapes.
//
// Written FIRST (the RED spec).
// ---------------------------------------------------------------------------

use aeolus_lsp::{AeolusEngine, EngineConfig, Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// The document the battery answers over: the same class in two
/// strings, a variant-prefixed sibling, a refused candidate — the
/// identity model's controlled input.
const DOC: &str = "fn card() {\n    let a = \"p-4 bg-primary\";\n    let b = \"hover:bg-primary not-a-root\";\n    let c = \"bg-primary flex-col\";\n}\n";

/// The accented-neighbour document: the multi-byte identifier shifts
/// every column after it — the byte-truth probe.
const ACCENTED_DOC: &str = "fn héllo() {\n    let a = \"bg-primary\";\n}\n";

const URI: &str = "file:///card.rs";

fn engine(doc: &str) -> AeolusEngine {
    let mut lsp = AeolusEngine::new(EngineConfig::new(None));
    lsp.open(URI, doc, 1);
    lsp
}

/// The byte offset of `needle`'s first character plus `into` bytes, as
/// the engine's (line, character) probe.
fn at(doc: &str, needle: &str, into: usize) -> (u32, u32) {
    let offset = doc.find(needle).expect("the fixture holds") + into;
    let line = doc[..offset].matches('\n').count() as u32;
    let line_start = doc[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = doc[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    (line, character)
}

// ---------------------------------------------------------------------------
// The engine face
// ---------------------------------------------------------------------------

#[test]
fn linked_ranges_answer_every_occurrence_in_the_document() {
    let lsp = engine(DOC);
    let (line, character) = at(DOC, "bg-primary", 2);
    let ranges = lsp.linked_ranges(URI, line, character);

    assert_eq!(
        ranges.len(),
        2,
        "the two PLAIN `bg-primary` spellings (strings a and c); the variant-prefixed sibling in b is a different spelling — the battery's own next pin"
    );
    // The cursor's own occurrence rides the answer, document order
    // (the extraction seam's scan order) preserved.
    assert_eq!(
        ranges[0].start.line, 1,
        "the first occurrence sits on string a's line"
    );
    assert_eq!(ranges[1].start.line, 3, "the second on string c's line");
}

#[test]
fn a_variant_prefixed_sibling_is_a_different_spelling() {
    let lsp = engine(DOC);
    let (line, character) = at(DOC, "hover:bg-primary", 7);
    let ranges = lsp.linked_ranges(URI, line, character);

    assert_eq!(
        ranges.len(),
        1,
        "the variant-prefixed spelling links alone — the rename face's own token rule"
    );
    assert_eq!(ranges[0].start.line, 2);
}

#[test]
fn refused_candidate_occurrences_ride_too() {
    let lsp = engine(DOC);
    let (line, character) = at(DOC, "not-a-root", 3);
    let ranges = lsp.linked_ranges(URI, line, character);

    assert_eq!(
        ranges.len(),
        1,
        "the identity is TEXTUAL: a refused candidate's occurrences answer like any candidate's"
    );
    assert_eq!(ranges[0].start.line, 2);
}

#[test]
fn outside_any_candidate_answers_empty() {
    let lsp = engine(DOC);
    // The gap between `p-4` and `bg-primary` inside string a: the
    // quote's offset plus the four bytes of `"p-4` — the space itself.
    let offset = DOC.find("\"p-4 bg-primary\"").expect("the fixture holds") + "\"p-4".len();
    let line = DOC[..offset].matches('\n').count() as u32;
    let line_start = DOC[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = doc_column(DOC, line_start, offset);

    assert!(
        lsp.linked_ranges(URI, line, character).is_empty(),
        "no candidate under the cursor, no linked ranges"
    );
}

/// The column walk the `at` helper keeps (the byte offset inside its
/// line, as UTF-16 pairs).
fn doc_column(doc: &str, line_start: usize, offset: usize) -> u32 {
    doc[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum()
}

#[test]
fn the_unknown_uri_answers_empty() {
    let lsp = engine(DOC);
    let (line, character) = at(DOC, "bg-primary", 2);
    assert!(
        lsp.linked_ranges("file:///not-open.rs", line, character)
            .is_empty(),
        "the closed document answers empty"
    );
}

#[test]
fn the_answer_is_byte_true_utf16_pairs() {
    let lsp = engine(ACCENTED_DOC);
    let (line, character) = at(ACCENTED_DOC, "bg-primary", 2);
    let ranges = lsp.linked_ranges(URI, line, character);

    assert_eq!(ranges.len(), 1);
    let range = ranges[0];
    // `bg-primary` starts after `héllo` — the é counts its TWO utf16
    // pairs, not its two bytes, so the column is 10, not 9.
    assert_eq!(range.start.character, 13, "the accent's pairs counted");
    assert_eq!(
        range.end.character - range.start.character,
        10,
        "the spelling's own length"
    );
    assert_eq!(range.start.line, 1);
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

fn linked_request(session: &mut Session, id: i32, line: u32, character: u32) -> serde_json::Value {
    let handled = handle_message(
        session,
        request(
            id,
            "textDocument/linkedEditingRange",
            json!({
                "textDocument": {"uri": URI},
                "position": {"line": line, "character": character}
            }),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the linked request answers");
    };
    response.result.expect("the linked answer carries a result")
}

#[test]
fn wire_linked_ranges_answer_the_ranges() {
    let mut session = Session::new(Default::default());
    handshake(&mut session);
    let _ = handle_message(&mut session, did_open(URI, DOC));

    let (line, character) = at(DOC, "bg-primary", 2);
    let result = linked_request(&mut session, 1, line, character);
    let ranges = result["ranges"].as_array().expect("the ranges ride");
    assert_eq!(ranges.len(), 2, "the engine's answer, wire-shaped");
    assert_eq!(ranges[0]["start"]["line"], 1);
    assert_eq!(ranges[1]["start"]["line"], 3);
    let start = ranges[0]["start"]["character"]
        .as_u64()
        .expect("a column rides");
    let end = ranges[0]["end"]["character"]
        .as_u64()
        .expect("a column rides");
    assert!(start < end, "the range spans the spelling, not a caret");
    assert!(
        result.get("wordPattern").is_none(),
        "no word pattern rides: the answer's ranges are the whole contract"
    );
}

#[test]
fn initialize_advertises_linked_editing() {
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
        result["capabilities"]["linkedEditingRangeProvider"], true,
        "the plain true form: the answer is the engine's own walk"
    );
}

#[test]
fn wire_pre_handshake_linked_ranges_quiet_drops() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/linkedEditingRange",
            json!({
                "textDocument": {"uri": URI},
                "position": {"line": 1, "character": 12}
            }),
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
        request(1, "textDocument/linkedEditingRange", json!({})),
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
