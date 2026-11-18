//! Consumer demonstration: the LSP engine's rename face (the editor's
//! `textDocument/prepareRename` + `textDocument/rename` — the rename
//! capability beside the hover, the highlights, the quick fixes and the
//! formatting) driven headless. `prepareRename` answers the candidate's
//! own byte-true span (the full spelling, variant prefixes included —
//! the highlights' raw-text contract), `rename` rides ONE edit per
//! textual occurrence of the cursor's candidate, the new spelling on
//! every edit — the references' own fold over the rename edits (the
//! workspace upgrade: one edit per occurrence over EVERY open document,
//! each riding its document's URI), purely textual, no compile verdict
//! riding (the diagnostics channel tells the truth about the NEW
//! spelling the moment the didChange lands). The cross-document ride
//! over an open SET is `lsp_workspace_rename`'s own demo; the stdio
//! server itself is `cargo run -p aeolus-lsp --bin aeolus-lsp` (the
//! transport's thin shell over the same engine).
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_rename
//! ```

use aeolus_lsp::{Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// A synthetic .rs document: the class twice as its own token and once
/// under a variant prefix — a DIFFERENT raw text, untouched by a
/// rename keyed on the plain spelling.
const DOC: &str = "fn card() {\n    let a = \"flex-col\";\n    let b = \"hover:flex-col\";\n}\n";

/// The occurrence's 0-based (line, character) — the cursor position a
/// client sends on rename.
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
    // rename face's two requests — the message sequence a client
    // drives on a rename gesture.
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

    // prepareRename: the renameable span under the cursor (the editor
    // shows the inline rename box over exactly this range).
    let (line, character) = cursor_of(DOC, "flex-col");
    drive(
        &mut session,
        1,
        "textDocument/prepareRename",
        json!({
            "textDocument": {"uri": "file:///card.rs"},
            "position": {"line": line, "character": character},
        }),
    );

    // rename: one edit per textual occurrence of the plain spelling —
    // the variant-prefixed sibling is a different raw text, untouched.
    let (line, character) = cursor_of(DOC, "flex-col");
    drive(
        &mut session,
        2,
        "textDocument/rename",
        json!({
            "textDocument": {"uri": "file:///card.rs"},
            "position": {"line": line, "character": character},
            "newName": "flex-row",
        }),
    );

    // Outside any candidate: prepareRename answers null (the
    // protocol's cannot-rename), rename rides no changes (the honest
    // empty, never a guessed edit).
    drive(
        &mut session,
        3,
        "textDocument/prepareRename",
        json!({
            "textDocument": {"uri": "file:///card.rs"},
            "position": {"line": 0, "character": 2},
        }),
    );
}
