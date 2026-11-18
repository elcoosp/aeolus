//! Consumer demonstration: the LSP engine's document-symbol face (the
//! editor's `textDocument/documentSymbol` — the outline capability
//! beside the hover, the highlights, the quick fixes, the rename and
//! the formatting) driven headless. The engine answers ONE symbol per
//! candidate class the extraction seam's strings carry — the spelling
//! the name, its byte-true range, the semantic legend's own verdict
//! (a lowered class is the protocol's `Class` kind, a refused one
//! stays `String`, the raw spelling) and the pipeline's own detail
//! line (the compact CSS the class lowers to, the refusal's stable
//! code beside). The stdio server itself is `cargo run -p aeolus-lsp
//! --bin aeolus-lsp` (the transport's thin shell over the same
//! engine).
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_symbols
//! ```

use aeolus_lsp::{Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// A synthetic .rs document the way `tw!` call sites look: two lowered
/// strings and one unknown root — the outline names every candidate,
/// the verdict riding beside each name.
const DOC: &str = "fn card() {\n    let a = \"p-4 flex-col\";\n    let b = \"hover:bg-primary/90\";\n    let bad = \"not-a-root\";\n}\n";

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
    // outline request — the message sequence a client drives when the
    // symbol panel opens.
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

    // The outline: one symbol per candidate, document order — the
    // editor's symbol tree renders the names, jumps by the ranges and
    // reads the verdict off the kind.
    drive(
        &mut session,
        1,
        "textDocument/documentSymbol",
        json!({"textDocument": {"uri": "file:///card.rs"}}),
    );

    // A document with no class strings: the honest empty answer (the
    // outline stays out of the editor's way).
    let _ = handle_message(
        &mut session,
        Message::Notification(Notification {
            method: "textDocument/didChange".to_string(),
            params: json!({
                "textDocument": {"uri": "file:///card.rs", "version": 2},
                "contentChanges": [{"text": "fn clean() {}\n"}],
            }),
        }),
    );
    drive(
        &mut session,
        2,
        "textDocument/documentSymbol",
        json!({"textDocument": {"uri": "file:///card.rs"}}),
    );
}
