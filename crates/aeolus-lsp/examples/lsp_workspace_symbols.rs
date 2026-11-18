//! Consumer demonstration: the LSP engine's workspace symbols (the
//! editor's `workspace/symbol` — the references fold's SECOND consumer,
//! the document outline generalized over the WHOLE document map). Every
//! answer is one outline entry per candidate class whose spelling
//! matches the query — the query the FILTER, a case-insensitive
//! substring (the variant prefixes included: `bg-` matches
//! `hover:bg-primary/90`), the empty query riding every open candidate
//! — beside the URI of the open document that carries it, URI order
//! across the map (sorted, whatever the open order was), document order
//! within each. The legend's own verdict rides each entry: a lowered
//! class renders the compact CSS (the inlay hint's label), a refused
//! one the stable code the diagnostics key on. The stdio server itself
//! is `cargo run -p aeolus-lsp --bin aeolus-lsp` (the transport's thin
//! shell over the same engine).
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_workspace_symbols
//! ```

use aeolus_lsp::{Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// Two synthetic .rs documents: the outline's own shapes — lowered
/// classes, a refused one, a variant-prefixed spelling.
const CARD: &str = "fn card() {\n    let a = \"flex-col\";\n    let b = \"hover:bg-primary/90\";\n    let bad = \"not-a-root\";\n}\n";
const PAGE: &str = "fn page() {\n    let a = \"p-4 gap-2\";\n}\n";

fn main() {
    // The state machine is the server's whole behaviour, testable and
    // demonstrable without a stdio pipe: initialize, two didOpens, then
    // the workspace-symbol requests — the message sequence a client
    // drives on the symbol search.
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

    // The empty query: the filter's identity — every open candidate
    // rides, URI order across the map.
    let everything = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(1),
            method: "workspace/symbol".to_string(),
            params: json!({"query": ""}),
        }),
    );
    println!("== every open candidate ==");
    for message in everything.outgoing {
        if let Message::Response(response) = message {
            println!(
                "{}",
                serde_json::to_string_pretty(&response.result).expect("the symbols serialize")
            );
        }
    }

    // A real filter: the typed prefix, case-insensitive — `BG-` matches
    // the variant-prefixed spelling the same way `bg-` would.
    let filtered = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(2),
            method: "workspace/symbol".to_string(),
            params: json!({"query": "BG-"}),
        }),
    );
    println!("\n== the `BG-` filter's matches ==");
    for message in filtered.outgoing {
        if let Message::Response(response) = message {
            println!(
                "{}",
                serde_json::to_string_pretty(&response.result).expect("the symbols serialize")
            );
        }
    }

    // The refused candidate rides the same outline: its detail line is
    // the stable code the diagnostics key on (the legend's own
    // boundary, no prose).
    let refused = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(3),
            method: "workspace/symbol".to_string(),
            params: json!({"query": "not-a-root"}),
        }),
    );
    println!("\n== the refused candidate's entry ==");
    for message in refused.outgoing {
        if let Message::Response(response) = message {
            println!(
                "{}",
                serde_json::to_string_pretty(&response.result).expect("the symbols serialize")
            );
        }
    }

    // The quiet empty: a query nothing matches answers no symbol (the
    // honest empty, never a guessed entry).
    let miss = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(4),
            method: "workspace/symbol".to_string(),
            params: json!({"query": "grid-cols-7"}),
        }),
    );
    for message in miss.outgoing {
        if let Message::Response(response) = message {
            println!(
                "\n== no match ==\n{}",
                serde_json::to_string_pretty(&response.result).expect("the empty serializes")
            );
        }
    }
}
