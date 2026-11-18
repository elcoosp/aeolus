//! Consumer demonstration: the LSP engine's cross-document references
//! (the editor's `textDocument/references` — the engine's FIRST
//! cross-document answer, the highlights' textual equality riding the
//! WHOLE document map). Every reference is one candidate occurrence
//! whose text equals the cursor's, beside the URI of the open document
//! that carries it — URI order across the map (sorted, whatever the
//! open order was), document order within each. The identity is
//! TEXTUAL, not semantic: a refused class's occurrences answer exactly
//! like a lowered one's (no compile verdict riding), and a
//! variant-prefixed sibling (`hover:flex-col` beside `flex-col`) is a
//! DIFFERENT spelling. The stdio server itself is `cargo run -p
//! aeolus-lsp --bin aeolus-lsp` (the transport's thin shell over the
//! same engine).
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_references
//! ```

use aeolus_lsp::{Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// Three synthetic .rs documents: the queried spelling once per
/// document (card.rs also carries a `hover:`-prefixed sibling that is a
/// different candidate, page.rs a refused neighbour).
const CARD: &str = "fn card() {\n    let a = \"flex-col\";\n    let b = \"hover:flex-col\";\n}\n";
const PAGE: &str = "fn page() {\n    let a = \"p-4 flex-col not-a-root\";\n}\n";
const LIST: &str = "fn list() {\n    let a = \"flex-col\";\n}\n";

/// The `flex-col` spelling's 0-based (line, character) in a fixture —
/// the cursor position a client sends on find-references.
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
    // (the cross-document answer's own scope), then the references
    // request — the message sequence a client drives on
    // find-references.
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

    // The references request from card.rs's bare occurrence: one
    // `Location` per carrying document, URI order across the map.
    let (line, character) = cursor_of(CARD, "flex-col");
    let references = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(1),
            method: "textDocument/references".to_string(),
            params: json!({
                "textDocument": {"uri": "file:///card.rs"},
                "position": {"line": line, "character": character},
                "context": {"includeDeclaration": true},
            }),
        }),
    );
    println!("== the workspace's `flex-col` occurrences ==");
    for message in references.outgoing {
        if let Message::Response(response) = message {
            println!(
                "{}",
                serde_json::to_string_pretty(&response.result).expect("the references serialize")
            );
        }
    }

    // The refused neighbour: the same textual equality over a class the
    // compiler refuses — the occurrences answer exactly like a lowered
    // one's (no compile verdict riding).
    let (line, character) = cursor_of(PAGE, "not-a-root");
    let refused = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(2),
            method: "textDocument/references".to_string(),
            params: json!({
                "textDocument": {"uri": "file:///page.rs"},
                "position": {"line": line, "character": character},
                "context": {"includeDeclaration": true},
            }),
        }),
    );
    println!("\n== the refused class's occurrence ==");
    for message in refused.outgoing {
        if let Message::Response(response) = message {
            println!(
                "{}",
                serde_json::to_string_pretty(&response.result).expect("the references serialize")
            );
        }
    }

    // The quiet empty: a cursor outside any candidate answers no
    // reference (the honest empty, never a guessed range).
    let outside = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(3),
            method: "textDocument/references".to_string(),
            params: json!({
                "textDocument": {"uri": "file:///card.rs"},
                "position": {"line": 0, "character": 0},
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
