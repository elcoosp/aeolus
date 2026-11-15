//! Consumer demonstration: the LSP engine's pull-diagnostics answer
//! (the editor's `textDocument/documentDiagnostic`, LSP 3.17's pull
//! model — the fifth request beside diagnostics-push, hover, completion
//! and the colour swatches) driven headless. The pulled report is the
//! SAME findings the push channel publishes: one analyze walk, two
//! transports. The stdio server itself is
//! `cargo run -p aeolus-lsp --bin aeolus-lsp` (the transport's thin
//! shell over the same engine).
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_diagnostics
//! ```

use aeolus_lsp::{Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// A synthetic .rs document the way `tw!` call sites look: two lowered
/// strings, one unknown root — the miss the pull report carries.
const DOC: &str = "fn card() {\n    let a = \"p-4 flex-col gap-2 rounded-md\";\n    let bad = \"not-a-root\";\n}\n";

fn main() {
    // The state machine is the server's whole behaviour, testable and
    // demonstrable without a stdio pipe: initialize, didOpen, then the
    // pull — the message sequence a client drives on refresh or on a
    // newly focused file.
    let mut session = Session::new(Default::default());
    let _ = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(0),
            method: "initialize".to_string(),
            params: json!({"capabilities": {}}),
        }),
    );

    // The capability surface names the pull model beside the push
    // channel (the diagnosticProvider the client reads at initialize).
    let caps = json!({"capabilities": {
        "hoverProvider": true,
        "colorProvider": true,
        "diagnosticProvider": {
            "interFileDependencies": false,
            "workspaceDiagnostics": false
        }
    }});
    println!("== the advertised pull capability ==");
    println!("{caps}");

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

    // The pull: the full report recomputed on demand — the same findings
    // the didOpen push carried, over the protocol's Full report shape.
    let handled = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(11),
            method: "textDocument/documentDiagnostic".to_string(),
            params: json!({"textDocument": {"uri": "file:///card.rs"}}),
        }),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the pull answers a response");
    };
    println!("\n== the pulled report (file:///card.rs) ==");
    println!(
        "{}",
        serde_json::to_string_pretty(&response.result.unwrap()).unwrap()
    );

    // The clean document pulls an empty report — the honest answer, no
    // cached result IDs (the engine always recomputes).
    let _ = handle_message(
        &mut session,
        Message::Notification(Notification {
            method: "textDocument/didOpen".to_string(),
            params: json!({
                "textDocument": {
                    "uri": "file:///clean.rs",
                    "languageId": "rust",
                    "version": 1,
                    "text": "fn clean() {}\n",
                }
            }),
        }),
    );
    let handled = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(12),
            method: "textDocument/documentDiagnostic".to_string(),
            params: json!({"textDocument": {"uri": "file:///clean.rs"}}),
        }),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the pull answers a response");
    };
    println!("\n== the clean document's report ==");
    println!(
        "{}",
        serde_json::to_string_pretty(&response.result.unwrap()).unwrap()
    );
}
