//! Consumer demonstration: the LSP engine's quick fixes (the editor's
//! `textDocument/codeAction`, LSP 3.16's code actions — the ninth
//! request beside diagnostics-push, hover, completion, the colour
//! swatches, the diagnostic pull, the semantic tokens, the colour
//! presentations and the inlay hints) driven headless, with the
//! click-to-find highlights (`textDocument/documentHighlight`) beside
//! them. Every action repairs one finding with a replacement the
//! pipeline LOWERS — the "did you mean" seam folds the compiler's own
//! audited tables (the registry's rows, the closed variant set), the
//! variant prefixes and `/modifier` tails composed back onto the
//! corrected token. The stdio server itself is `cargo run -p aeolus-lsp
//! --bin aeolus-lsp` (the transport's thin shell over the same engine).
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_code_actions
//! ```

use aeolus_lsp::{Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// A synthetic .rs document the way `tw!` call sites look: one keyword
/// miss beside a variant miss (the two folds the seam answers), then a
/// clean class the highlights walk finds twice.
const DOC: &str = "fn card() {\n    let bad = \"justify-betwen havor:p-4\";\n    let ok = \"flex-col\";\n    let same = \"flex-col\";\n}\n";

fn main() {
    // The state machine is the server's whole behaviour, testable and
    // demonstrable without a stdio pipe: initialize, didOpen, then the
    // two requests — the message sequence a client drives when the
    // editor invokes a quick fix on a squiggle and when the user
    // click-to-finds a class.
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

    // The quick fix on the keyword miss: the cursor rests inside
    // `justify-betwen` (line 1, inside the class).
    let actions = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(1),
            method: "textDocument/codeAction".to_string(),
            params: json!({
                "textDocument": {"uri": "file:///card.rs"},
                "range": {"start": {"line": 1, "character": 26},
                          "end": {"line": 1, "character": 26}},
                "context": {"diagnostics": []},
            }),
        }),
    );
    println!("== the quick fixes on the keyword miss ==");
    for message in actions.outgoing {
        if let Message::Response(response) = message {
            println!(
                "{}",
                serde_json::to_string_pretty(&response.result).expect("the actions serialize")
            );
        }
    }

    // The quick fix on the variant miss: the correction composes the
    // utility half back on (`hover:p-4`), the closed variant set's own
    // spelling.
    let actions = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(2),
            method: "textDocument/codeAction".to_string(),
            params: json!({
                "textDocument": {"uri": "file:///card.rs"},
                "range": {"start": {"line": 1, "character": 32},
                          "end": {"line": 1, "character": 32}},
                "context": {"diagnostics": []},
            }),
        }),
    );
    println!("== the quick fixes on the variant miss ==");
    for message in actions.outgoing {
        if let Message::Response(response) = message {
            println!(
                "{}",
                serde_json::to_string_pretty(&response.result).expect("the actions serialize")
            );
        }
    }

    // The click-to-find highlights on the clean class: every occurrence
    // whose text equals the cursor's candidate, document order — the
    // second string's copy included, the refused classes never asked.
    let highlights = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(3),
            method: "textDocument/documentHighlight".to_string(),
            params: json!({
                "textDocument": {"uri": "file:///card.rs"},
                "position": {"line": 2, "character": 18},
            }),
        }),
    );
    println!("== the document highlights on the clean class ==");
    for message in highlights.outgoing {
        if let Message::Response(response) = message {
            println!(
                "{}",
                serde_json::to_string_pretty(&response.result).expect("the highlights serialize")
            );
        }
    }
}
