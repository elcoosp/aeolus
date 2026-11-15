//! Consumer demonstration: the LSP engine's inlay-hint answer (the
//! editor's `textDocument/inlayHint`, LSP 3.17's inline answers — the
//! eighth request beside diagnostics-push, hover, completion, the
//! colour swatches, the diagnostic pull, the semantic tokens and the
//! colour presentations) driven headless. Every lowered class rides one
//! hint at its END: the CSS arm's own declarations compacted to the
//! single-line label, the hover's markdown riding as the tooltip; a
//! refused candidate never hints (its story is the diagnostics' and the
//! legend's). The stdio server itself is `cargo run -p aeolus-lsp --bin
//! aeolus-lsp` (the transport's thin shell over the same engine).
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_inlay_hints
//! ```

use aeolus_lsp::{Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// A synthetic .rs document the way `tw!` call sites look: one scale
/// class, one token class (the default shadcn bake resolves its baked
/// tier), one unknown root — the refusal the hints never carry.
const DOC: &str = "fn card() {\n    let a = \"p-4\";\n    let b = \"bg-primary\";\n    let bad = \"not-a-root\";\n}\n";

fn main() {
    // The state machine is the server's whole behaviour, testable and
    // demonstrable without a stdio pipe: initialize, didOpen, then the
    // hint request — the message sequence a client drives when the
    // editor turns inlay hints on for the file.
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

    // The hints: one per lowered class, riding at the class's end — the
    // compact CSS label, the markdown tooltip carrying the WHY.
    let handled = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(15),
            method: "textDocument/inlayHint".to_string(),
            params: json!({
                "textDocument": {"uri": "file:///card.rs"},
                "range": {
                    "start": {"line": 0, "character": 0},
                    "end": {"line": 9, "character": 0},
                },
            }),
        }),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("inlayHint answers a response");
    };
    let result = response.result.expect("the hints carry a result");
    println!("== the hints (file:///card.rs) ==");
    for hint in result.as_array().expect("the hint array rides") {
        println!(
            "  at {}:{}  label = {}",
            hint["position"]["line"],
            hint["position"]["character"],
            hint["label"].as_str().expect("the label rides"),
        );
        let tooltip = &hint["tooltip"]["value"];
        println!(
            "            tooltip = {}",
            tooltip
                .as_str()
                .expect("the tooltip rides")
                .lines()
                .next()
                .unwrap_or_default(),
        );
    }

    // The refused candidate never hints — the diagnostics published at
    // didOpen carry its story, the legend highlights it.
    println!("\n== the refused candidate (not-a-root) ==");
    let handled = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(16),
            method: "textDocument/documentDiagnostic".to_string(),
            params: json!({"textDocument": {"uri": "file:///card.rs"}}),
        }),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("documentDiagnostic answers a response");
    };
    let result = response.result.expect("the report carries a result");
    for item in result["items"].as_array().expect("the items ride") {
        println!(
            "  {} [{}] (no hint — the refusal's story is the diagnostics')",
            item["message"]
                .as_str()
                .expect("the message rides")
                .lines()
                .next()
                .unwrap_or_default(),
            item["code"].as_str().expect("the code rides"),
        );
    }
}
