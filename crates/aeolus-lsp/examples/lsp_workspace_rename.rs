//! Consumer demonstration: the LSP engine's cross-document rename (the
//! editor's `textDocument/rename` — the rename face's workspace
//! upgrade, the references fold's third consumer) driven headless. The
//! cursor's candidate walks the WHOLE document map — one edit per
//! textual occurrence over every open document, each riding its
//! document's URI and the new spelling as a new `EngineRenameEdit`:
//! URI order across the map (sorted, whatever the open order was),
//! document order within each. The identity is TEXTUAL: a refused
//! class's occurrences edit exactly like a lowered one's (no compile
//! verdict riding), and a variant-prefixed sibling (`hover:flex-col`
//! beside `flex-col`) is a DIFFERENT spelling, untouched. The wire
//! half answers the FULL `changes` map — one key per carrying
//! document — the protocol's own workspace-edit shape; the stdio
//! server itself is `cargo run -p aeolus-lsp --bin aeolus-lsp` (the
//! transport's thin shell over the same engine).
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_workspace_rename
//! ```

use aeolus_lsp::{Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// Three synthetic .rs documents: the renamed spelling once per
/// document (card.rs also carries a `hover:`-prefixed sibling that is a
/// different candidate, page.rs a refused neighbour).
const CARD: &str = "fn card() {\n    let a = \"flex-col\";\n    let b = \"hover:flex-col\";\n}\n";
const PAGE: &str = "fn page() {\n    let a = \"p-4 flex-col not-a-root\";\n}\n";
const LIST: &str = "fn list() {\n    let a = \"flex-col\";\n}\n";

/// The `flex-col` spelling's 0-based (line, character) in a fixture —
/// the cursor position a client sends on rename.
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

fn main() {
    // The state machine is the server's whole behaviour, testable and
    // demonstrable without a stdio pipe: initialize, three didOpens
    // (the cross-document answer's own scope), then the rename request
    // — the message sequence a client drives on a rename gesture.
    let mut session = Session::new(Default::default());
    let _ = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(0),
            method: "initialize".to_string(),
            params: json!({"capabilities": {}}),
        }),
    );
    // The didOpen versions count in the protocol's own i32 units —
    // the literals ride directly, no cast.
    for (version, (uri, text)) in [
        (1i32, ("file:///card.rs", CARD)),
        (2i32, ("file:///page.rs", PAGE)),
        (3i32, ("file:///list.rs", LIST)),
    ] {
        let _ = handle_message(
            &mut session,
            Message::Notification(Notification {
                method: "textDocument/didOpen".to_string(),
                params: json!({
                    "textDocument": {
                        "uri": uri,
                        "languageId": "rust",
                        "version": version,
                        "text": text,
                    }
                }),
            }),
        );
    }

    // The rename request from card.rs's bare occurrence: the FULL
    // changes map — one key per carrying document, the new spelling on
    // every edit. The variant-prefixed sibling in card.rs stays
    // untouched (a different raw text, the highlights' own equality).
    let (line, character) = cursor_of(CARD, "flex-col");
    let renamed = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(1),
            method: "textDocument/rename".to_string(),
            params: json!({
                "textDocument": {"uri": "file:///card.rs"},
                "position": {"line": line, "character": character},
                "newName": "flex-row",
            }),
        }),
    );
    println!("== the workspace's `flex-col` -> `flex-row` edits ==");
    for message in renamed.outgoing {
        if let Message::Response(response) = message {
            println!(
                "{}",
                serde_json::to_string_pretty(&response.result).expect("the edit serializes")
            );
        }
    }

    // The refused neighbour: the same textual equality over a class the
    // compiler refuses — the occurrences edit exactly like a lowered
    // one's (no compile verdict riding).
    let (line, character) = cursor_of(PAGE, "not-a-root");
    let refused = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(2),
            method: "textDocument/rename".to_string(),
            params: json!({
                "textDocument": {"uri": "file:///page.rs"},
                "position": {"line": line, "character": character},
                "newName": "not-a-token",
            }),
        }),
    );
    println!("\n== the refused class's edit ==");
    for message in refused.outgoing {
        if let Message::Response(response) = message {
            println!(
                "{}",
                serde_json::to_string_pretty(&response.result).expect("the edit serializes")
            );
        }
    }

    // The quiet empty: a cursor outside any candidate answers the
    // empty changes map (the honest no-op, never a guessed edit).
    let outside = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(3),
            method: "textDocument/rename".to_string(),
            params: json!({
                "textDocument": {"uri": "file:///card.rs"},
                "position": {"line": 0, "character": 2},
                "newName": "flex-row",
            }),
        }),
    );
    for message in outside.outgoing {
        if let Message::Response(response) = message {
            println!(
                "\n== outside any candidate ==\n{}",
                serde_json::to_string_pretty(&response.result).expect("the empty serializes")
            );
        }
    }
}
