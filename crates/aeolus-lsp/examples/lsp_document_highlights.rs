//! Consumer demonstration: the LSP engine's click-to-find highlights
//! (the editor's `textDocument/documentHighlight` — the occurrences
//! answer beside the hover, the completion, the colour swatches, the
//! diagnostic pull, the semantic tokens, the colour presentations, the
//! inlay hints, the quick fixes and the formatting) driven headless.
//! Every highlight is one candidate occurrence whose text EQUALS the
//! cursor's — document order, the cursor's own occurrence included —
//! and the highlight is TEXTUAL, not semantic: a refused class's
//! occurrences highlight exactly like a lowered one's (no compile
//! verdict riding). The stdio server itself is `cargo run -p
//! aeolus-lsp --bin aeolus-lsp` (the transport's thin shell over the
//! same engine).
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_document_highlights
//! ```

use aeolus_lsp::{Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// A synthetic .rs document: the cursor's class twice as its own token
/// (the second beside a refused neighbour) and once under a variant
/// prefix — a DIFFERENT raw text, no highlight.
const DOC: &str = "fn card() {\n    let a = \"flex-col\";\n    let b = \"hover:flex-col\";\n    let c = \"not-a-root flex-col\";\n}\n";

/// The `flex-col` occurrence's 0-based (line, character) — the cursor
/// position a client sends on click-to-find.
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
    // demonstrable without a stdio pipe: initialize, didOpen, then the
    // highlight request — the message sequence a client drives on
    // click-to-find.
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

    // The highlight request: the cursor's own occurrence beside its
    // exact-text siblings.
    let (line, character) = cursor_of(DOC, "flex-col");
    let highlights = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(1),
            method: "textDocument/documentHighlight".to_string(),
            params: json!({
                "textDocument": {"uri": "file:///card.rs"},
                "position": {"line": line, "character": character},
            }),
        }),
    );
    println!("== the highlights under the cursor ==");
    for message in highlights.outgoing {
        if let Message::Response(response) = message {
            println!(
                "{}",
                serde_json::to_string_pretty(&response.result).expect("the highlights serialize")
            );
        }
    }

    // The refused neighbour: the same textual equality over a class the
    // compiler refuses — the occurrences highlight exactly like a
    // lowered one's (the editor's own click-to-find, no verdict riding).
    let (line, character) = cursor_of(DOC, "not-a-root");
    let refused = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(2),
            method: "textDocument/documentHighlight".to_string(),
            params: json!({
                "textDocument": {"uri": "file:///card.rs"},
                "position": {"line": line, "character": character},
            }),
        }),
    );
    println!("\n== the refused class's occurrence ==");
    for message in refused.outgoing {
        if let Message::Response(response) = message {
            println!(
                "{}",
                serde_json::to_string_pretty(&response.result).expect("the highlights serialize")
            );
        }
    }

    // The quiet empty: a cursor outside any candidate answers no
    // highlight (the honest empty, never a guessed range).
    let outside = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(3),
            method: "textDocument/documentHighlight".to_string(),
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
