//! The cross-document rename answer (the wire's `textDocument/rename`
//! — the rename face's workspace upgrade): the cursor's candidate's
//! edits riding EVERY open document, the references' fold over the
//! rename edits. Written FIRST (the RED spec): the missing answer
//! type's compile failure WAS the capture.
//!
//! The suite pins the answer's house rules beside the answers:
//!
//! * the TEXTUAL identity (the highlights' own contract, no compile
//!   verdict riding): a refused class's occurrences edit exactly like
//!   a lowered one's, and a variant-prefixed sibling (`hover:flex-col`
//!   beside `flex-col`) is a DIFFERENT spelling;
//! * the whole document map is the answer's scope: edits from every
//!   open document, the CLOSED one dropping out, an unknown cursor URI
//!   and a cursor outside any candidate answering empty;
//! * DETERMINISM over the hash map: the edits ride URI order (sorted,
//!   whatever the open order was), document order within each — the
//!   answer is pure over the map;
//! * byte-true UTF-16 ranges (the multi-byte neighbour counts its
//!   pairs, not its bytes);
//! * the empty new name answers NO edits anywhere (never a deletion
//!   walk — the protocol's validity clause, the quiet shape);
//! * the wire half: the request answers the FULL `changes` map (one
//!   key per carrying document, the protocol's own workspace-edit
//!   shape), the pre-handshake gate quiet-drops, malformed params
//!   answer the protocol's own Null, and a URI the `Uri` parser
//!   rejects skips its key (the publish half's own rule).

use aeolus_lsp::{AeolusEngine, EngineConfig, EngineRenameEdit, Session, handle_message};
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

fn rename_params(uri: &str, line: u32, character: u32, new_name: &str) -> serde_json::Value {
    json!({
        "textDocument": {"uri": uri},
        "position": {"line": line, "character": character},
        "newName": new_name,
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
fn edits_ride_every_open_document() {
    let engine = wired_engine();
    let (line, character) = cursor_of(DOC_B, "flex-col");
    let edits = engine.rename("file:///b.rs", line, character, "flex-row");
    // Three documents: a.rs one bare occurrence, b.rs two (the queried
    // line-1 one and the `hé `-neighboured line-2 one), c.rs one —
    // a.rs's `hover:flex-col` sibling is a DIFFERENT spelling, no edit.
    assert_eq!(edits.len(), 4, "every open document's occurrences edit");
    assert_eq!(edits[0].uri, "file:///a.rs");
    assert_eq!(edits[0].range.start.line, 1);
    assert_eq!(edits[0].new_text, "flex-row");
    assert_eq!(edits[1].uri, "file:///b.rs");
    assert_eq!(edits[2].uri, "file:///b.rs");
    assert_eq!(edits[3].uri, "file:///c.rs");
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
    let edits = engine.rename("file:///a.rs", line, character, "flex-row");
    let uris: Vec<&str> = edits.iter().map(|edit| edit.uri.as_str()).collect();
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
fn the_cursor_occurrence_edits_too() {
    let engine = wired_engine();
    let (line, character) = cursor_of(DOC_C, "flex-col");
    let edits = engine.rename("file:///c.rs", line, character, "flex-row");
    let here = edits
        .iter()
        .find(|edit| edit.uri == "file:///c.rs")
        .expect("the queried document's occurrence edits");
    assert_eq!(here.range.start.line, 1, "the cursor's own occurrence");
}

#[test]
fn variant_prefixed_sibling_is_a_different_spelling() {
    let engine = wired_engine();
    // DOC_A's second string carries `hover:flex-col` — a different raw
    // text, no edit (the highlights' own equality rule).
    let (line, character) = cursor_of(DOC_A, "flex-col");
    let edits = engine.rename("file:///a.rs", line, character, "flex-row");
    let in_a: Vec<&EngineRenameEdit> = edits
        .iter()
        .filter(|edit| edit.uri == "file:///a.rs")
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
fn refused_class_occurrences_edit_too() {
    let engine = wired_engine();
    let (line, character) = cursor_of(DOC_B, "not-a-root");
    let edits = engine.rename("file:///b.rs", line, character, "px-8");
    assert_eq!(edits.len(), 1, "the refusal's own occurrence edits");
    assert_eq!(edits[0].uri, "file:///b.rs");
    assert_eq!(edits[0].new_text, "px-8");
}

#[test]
fn unknown_uri_and_outside_candidate_answer_empty() {
    let engine = wired_engine();
    assert!(engine.rename("file:///missing.rs", 0, 0, "px-8").is_empty());
    assert!(engine.rename("file:///a.rs", 0, 0, "px-8").is_empty());
}

#[test]
fn the_empty_name_answers_no_edits_anywhere() {
    let engine = wired_engine();
    // The guard rides BEFORE the walk: no document's occurrences
    // answer, the deletion walk never runs (the protocol's validity
    // clause).
    let (line, character) = cursor_of(DOC_B, "flex-col");
    assert!(
        engine
            .rename("file:///b.rs", line, character, "")
            .is_empty(),
        "the empty name answers no edits (never a deletion walk)"
    );
}

#[test]
fn closed_document_drops_out() {
    let mut engine = wired_engine();
    engine.close("file:///b.rs");
    let (line, character) = cursor_of(DOC_A, "flex-col");
    let edits = engine.rename("file:///a.rs", line, character, "flex-row");
    assert!(
        edits.iter().all(|edit| edit.uri != "file:///b.rs"),
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
    let edits = engine.rename("file:///b.rs", line, character, "flex-row");
    let hinted = edits
        .iter()
        .find(|edit| edit.uri == "file:///b.rs" && edit.range.start.line == 2)
        .expect("the multi-byte line's occurrence edits");
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
fn wire_rename_answers_the_full_changes_map() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let _ = handle_message(&mut session, did_open("file:///a.rs", DOC_A));
    let _ = handle_message(&mut session, did_open("file:///c.rs", DOC_C));
    let (line, character) = cursor_of(DOC_A, "flex-col");
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/rename",
            rename_params("file:///a.rs", line, character, "flex-row"),
        ),
    );
    assert_eq!(handled.outgoing.len(), 1, "the response rides");
    let result = match &handled.outgoing[0] {
        Message::Response(response) => response.result.clone(),
        other => panic!("a response rode, got {other:?}"),
    };
    let edit = result.expect("the workspace edit answer");
    let changes = edit["changes"]
        .as_object()
        .expect("the changes map rides the answer");
    assert_eq!(
        changes.len(),
        2,
        "one key per carrying document: the FULL map, not the requesting URI's only"
    );
    let a_edits = changes["file:///a.rs"]
        .as_array()
        .expect("a.rs's edit list rides");
    assert_eq!(a_edits.len(), 1);
    assert_eq!(a_edits[0]["newText"], "flex-row");
    assert_eq!(a_edits[0]["range"]["start"]["line"], 1);
    let c_edits = changes["file:///c.rs"]
        .as_array()
        .expect("c.rs's edit list rides");
    assert_eq!(c_edits.len(), 1);
    assert_eq!(c_edits[0]["newText"], "flex-row");
}

#[test]
fn wire_pre_handshake_rename_quiet_drops() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/rename",
            rename_params("file:///a.rs", 1, 8, "flex-row"),
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
        request(1, "textDocument/rename", json!({"newName": 7})),
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
fn wire_outside_candidate_answers_the_empty_map() {
    // The cursor outside any candidate: no edits anywhere — the quiet
    // shape (the engine's own guard, the wire's empty changes map).
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let _ = handle_message(&mut session, did_open("file:///a.rs", DOC_A));
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/rename",
            rename_params("file:///a.rs", 0, 2, "flex-row"),
        ),
    );
    let result = match &handled.outgoing[0] {
        Message::Response(response) => response.result.clone(),
        other => panic!("a response rode, got {other:?}"),
    };
    let edit = result.expect("the workspace edit answer");
    let changes = edit["changes"]
        .as_object()
        .expect("the changes map rides the answer");
    assert!(
        changes.is_empty(),
        "no candidate, no edits, no keys: the quiet shape"
    );
}

#[test]
fn wire_refused_neighbour_edits_the_same_shape() {
    // The textual contract at the wire: a refused class's occurrence
    // edits exactly like a lowered one's (no compile verdict riding).
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let _ = handle_message(&mut session, did_open("file:///b.rs", DOC_B));
    let (line, character) = cursor_of(DOC_B, "not-a-root");
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/rename",
            rename_params("file:///b.rs", line, character, "px-8"),
        ),
    );
    let result = match &handled.outgoing[0] {
        Message::Response(response) => response.result.clone(),
        other => panic!("a response rode, got {other:?}"),
    };
    let edit = result.expect("the workspace edit answer");
    let changes = edit["changes"]
        .as_object()
        .expect("the changes map rides the answer");
    assert_eq!(changes.len(), 1, "the refusal's own occurrence, one key");
    assert_eq!(changes["file:///b.rs"][0]["newText"], "px-8");
}
