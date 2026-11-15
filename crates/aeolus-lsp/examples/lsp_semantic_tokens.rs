//! Consumer demonstration: the LSP engine's semantic-token answer
//! (the editor's `textDocument/semanticTokens/full`, LSP 3.16's
//! highlighter feed — the sixth request beside diagnostics-push, hover,
//! completion, the colour swatches and the diagnostic pull) driven
//! headless. Every candidate the extraction seam finds rides one
//! compile outcome onto the wire's legend: a lowered class highlights
//! as `class`, a refused one as the custom `refused` type. The stdio
//! server itself is `cargo run -p aeolus-lsp --bin aeolus-lsp` (the
//! transport's thin shell over the same engine).
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_semantic_tokens
//! ```

use aeolus_lsp::{Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// A synthetic .rs document the way `tw!` call sites look: two lowered
/// strings, one unknown root — the refusal the legend's second entry
/// highlights.
const DOC: &str = "fn card() {\n    let a = \"p-4 flex-col gap-2 rounded-md\";\n    let b = \"hover:bg-primary/90 dark:shadow-lg\";\n    let bad = \"not-a-root\";\n}\n";

fn main() {
    // The state machine is the server's whole behaviour, testable and
    // demonstrable without a stdio pipe: initialize, didOpen, then the
    // semantic-tokens pull — the message sequence a client drives when
    // the highlighter (re)paints a file.
    let mut session = Session::new(Default::default());

    // The capability surface names the legend (the wire contract the
    // delta encoding indexes).
    let handled = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(0),
            method: "initialize".to_string(),
            params: json!({"capabilities": {}}),
        }),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("initialize answers a response");
    };
    println!("== the advertised legend ==");
    println!(
        "{}",
        serde_json::to_string_pretty(
            &response.result.unwrap()["capabilities"]["semanticTokensProvider"]["legend"]
        )
        .unwrap()
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

    // The pull: the flat delta array (five u32s per token) decoded back
    // into the document's highlights — the walk the editor renders per
    // (re)paint.
    let handled = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(14),
            method: "textDocument/semanticTokens/full".to_string(),
            params: json!({"textDocument": {"uri": "file:///card.rs"}}),
        }),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("semanticTokens answers a response");
    };
    let result = response.result.expect("the tokens carry a result");
    let data = result["data"].as_array().expect("the flat array rides");
    let words: Vec<u32> = data
        .iter()
        .map(|value| value.as_u64().expect("a u32 word") as u32)
        .collect();

    println!("\n== the decoded highlights (file:///card.rs) ==");
    println!("legend: 0 = class (lowered), 1 = refused");
    let (mut line, mut character) = (0u32, 0u32);
    for chunk in words.chunks_exact(5) {
        let (delta_line, delta_start, length, token_type) =
            (chunk[0], chunk[1], chunk[2], chunk[3]);
        line += delta_line;
        character = if delta_line == 0 {
            character + delta_start
        } else {
            delta_start
        };
        let kind = match token_type {
            0 => "class",
            1 => "refused",
            _ => "unknown",
        };
        println!("  line {line} col {character} ..+{length}  → {kind}");
    }
}
