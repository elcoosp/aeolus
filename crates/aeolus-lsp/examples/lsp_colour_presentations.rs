//! Consumer demonstration: the LSP engine's colour-presentation answer
//! (the editor's `textDocument/colorPresentation`, the colorProvider
//! capability's paired half — the seventh request beside diagnostics-
//! push, hover, completion, the colour swatches, the diagnostic pull
//! and the semantic tokens) driven headless. The picker picks a colour;
//! the config's own table walks backwards and answers every spelling
//! that re-resolves to it — the same round trip the swatches ride. The
//! stdio server itself is `cargo run -p aeolus-lsp --bin aeolus-lsp`
//! (the transport's thin shell over the same engine).
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_colour_presentations
//! ```

use aeolus_lsp::{Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// A synthetic .rs document the way `tw!` call sites look: one token
/// class — the swatch its documentColor answers is the colour the
/// presentations spell back.
const DOC: &str = "fn card() {\n    let a = \"bg-primary\";\n}\n";

fn main() {
    // The state machine is the server's whole behaviour, testable and
    // demonstrable without a stdio pipe: initialize, didOpen, the
    // swatch pull, then the presentation — the message sequence a
    // client drives when the colour picker opens.
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

    // The swatch: the colour the walk resolves (the picker's own
    // request, the fourth one) — byte rgba onto the protocol's
    // 0..1 fractions, channel for channel.
    let handled = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(9),
            method: "textDocument/documentColor".to_string(),
            params: json!({"textDocument": {"uri": "file:///card.rs"}}),
        }),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("documentColor answers a response");
    };
    let result = response.result.expect("the swatches carry a result");
    let swatch = &result.as_array().expect("the colour array rides")[0];
    println!("== the swatch (file:///card.rs) ==");
    println!("{}", serde_json::to_string_pretty(swatch).unwrap());
    let color = swatch["color"].clone();
    let range = swatch["range"].clone();

    // The presentation: every spelling the table resolves to the SAME
    // bytes — the colour-group roots × the token × the audited opacity
    // steps, each riding as the edit the picker's swap writes.
    let handled = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(14),
            method: "textDocument/colorPresentation".to_string(),
            params: json!({
                "textDocument": {"uri": "file:///card.rs"},
                "color": color,
                "range": range,
            }),
        }),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("colorPresentation answers a response");
    };
    let result = response.result.expect("the presentations carry a result");
    println!("\n== the presentations (the spellings that re-resolve) ==");
    for item in result.as_array().expect("the presentation array rides") {
        println!(
            "  {:<28} edit.newText = {}",
            item["label"].as_str().expect("the label rides"),
            item["textEdit"]["newText"]
                .as_str()
                .expect("the edit rides"),
        );
    }

    // An unmatched colour answers the honest empty — never a guessed
    // spelling.
    let handled = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(15),
            method: "textDocument/colorPresentation".to_string(),
            params: json!({
                "textDocument": {"uri": "file:///card.rs"},
                "color": {"red": 0.004, "green": 0.008, "blue": 0.012, "alpha": 0.016},
                "range": range,
            }),
        }),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("colorPresentation answers a response");
    };
    let result = response.result.expect("the presentations carry a result");
    println!("\n== an unmatched colour ==");
    println!(
        "{} presentation(s) — the honest empty answer",
        result.as_array().expect("the array rides").len()
    );
}
