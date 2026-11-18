//! Consumer demonstration: the LSP engine's selection ranges (the
//! editor's `textDocument/selectionRange` — the expand-selection walk)
//! driven headless. One parent chain per requested position, request
//! order kept (the protocol's own 1:1 contract): a cursor inside a
//! class answers the candidate's byte-true span with the class
//! string's own content span riding as its parent — the SAME locator
//! the hover, the highlights, the references and the rename walk; a
//! cursor between classes answers the content span alone (no
//! candidate level, no invented inner range); a cursor outside any
//! string answers the protocol's own minimal range — the position's
//! zero-width caret, no parent. The stdio server itself is
//! `cargo run -p aeolus-lsp --bin aeolus-lsp` (the transport's thin
//! shell over the same engine).
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_selection_ranges
//! ```

use aeolus_lsp::{Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// A synthetic .rs document: one class string the three cursor shapes
/// walk — a candidate, the gap between classes, the code outside any
/// string.
const DOC: &str = "fn card() {\n    let a = \"p-4 flex-col hé\";\n}\n";

/// The needle's 0-based (line, character) in a fixture — the cursor
/// position a client sends on expand-selection.
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

/// One wire request against the session (the harness's own shape).
fn drive(session: &mut Session, id: i32, method: &str, params: serde_json::Value) {
    let handled = handle_message(
        session,
        Message::Request(Request {
            id: RequestId::from(id),
            method: method.to_string(),
            params,
        }),
    );
    println!("== {method} ==");
    for message in handled.outgoing {
        if let Message::Response(response) = message {
            println!(
                "{}",
                serde_json::to_string_pretty(&response.result).expect("the answer serializes")
            );
        }
    }
}

fn main() {
    // The state machine is the server's whole behaviour, testable and
    // demonstrable without a stdio pipe: initialize, didOpen, then the
    // multi-cursor selectionRange request — the message sequence a
    // client drives on expand-selection.
    let mut session = Session::new(Default::default());
    let _ = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(0),
            method: "initialize".to_string(),
            params: json!({"capabilities": {}}),
        }),
    );
    let _ = handle_message(
        &mut session,
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

    // Three cursors in ONE request — the multi-cursor shape the
    // protocol's 1:1 contract keeps in request order: inside the
    // candidate (the two-level chain), on the gap between the classes
    // (the content span alone), outside any string (the zero-width
    // caret).
    let (line, character) = cursor_of(DOC, "flex-col");
    let line_text = DOC.split('\n').nth(1).expect("the fixture holds");
    let gap = line_text
        .find("p-4 ")
        .map(|offset| {
            line_text[..offset + 3]
                .chars()
                .map(char::len_utf16)
                .sum::<usize>()
        })
        .expect("the gap rides the line") as u32;
    drive(
        &mut session,
        1,
        "textDocument/selectionRange",
        json!({
            "textDocument": {"uri": "file:///card.rs"},
            "positions": [
                {"line": line, "character": character},
                {"line": 1, "character": gap},
                {"line": 0, "character": 2},
            ],
        }),
    );

    // The accented neighbour: the content span's END column counts the
    // accent's UTF-16 pairs, not its bytes (the position model the
    // engine owns).
    let (line, character) = cursor_of(DOC, " hé");
    drive(
        &mut session,
        2,
        "textDocument/selectionRange",
        json!({
            "textDocument": {"uri": "file:///card.rs"},
            "positions": [{"line": line, "character": character}],
        }),
    );
}
