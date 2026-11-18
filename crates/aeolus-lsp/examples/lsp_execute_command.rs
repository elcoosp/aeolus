//! Consumer demonstration: the server's command surface — the wire's
//! `workspace/executeCommand` answered headless. The `aeolus.sortDocument`
//! command rides the class-sort seam's editor face: the same
//! canonical-order edits the `textDocument/formatting` request answers,
//! invoked as a command (an editor keybinding, a refactoring menu —
//! the protocol's own extension point where a client drives the
//! server). The command table is CLOSED and advertised; an unknown
//! command and a missing argument answer the protocol's own null, and
//! the edits ride the engine's own `format` answer byte for byte.
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_execute_command
//! ```

use aeolus_lsp::{Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// A synthetic .rs document: two out-of-order class strings (the
/// variant-prefixed sibling rides after its utility in the canonical
/// form), one already-canonical string the walk skips.
const DOC: &str = concat!(
    "fn card() {\n",
    "    let a = \"rounded-md p-4 flex-col gap-2\";\n",
    "    let b = \"hover:bg-primary flex\";\n",
    "    let c = \"flex-col items-center\";\n",
    "}\n",
);

const URI: &str = "file:///card.rs";

fn request(id: i32, method: &str, params: serde_json::Value) -> Message {
    Message::Request(Request {
        id: RequestId::from(id),
        method: method.to_string(),
        params,
    })
}

fn notification(method: &str, params: serde_json::Value) -> Message {
    Message::Notification(Notification {
        method: method.to_string(),
        params,
    })
}

/// One command round trip: the response's JSON result.
fn run(session: &mut Session, id: i32, params: serde_json::Value) -> serde_json::Value {
    let handled = handle_message(session, request(id, "workspace/executeCommand", params));
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the command answers");
    };
    response.result.unwrap_or(json!(null))
}

fn main() {
    let mut session = Session::new(Default::default());
    let _ = handle_message(
        &mut session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    let _ = handle_message(
        &mut session,
        notification(
            "textDocument/didOpen",
            json!({
                "textDocument": {"uri": URI, "languageId": "rust", "version": 1, "text": DOC}
            }),
        ),
    );

    println!("── the sort command, invoked as a command ──");
    let result = run(
        &mut session,
        1,
        json!({"command": "aeolus.sortDocument", "arguments": [URI]}),
    );
    for edit in result.as_array().expect("an array of edits") {
        println!(
            "    line {} → {}",
            edit["range"]["start"]["line"], edit["newText"]
        );
    }

    println!("── the closed command table ──");
    let unknown = run(
        &mut session,
        2,
        json!({"command": "aeolus.notACommand", "arguments": [URI]}),
    );
    println!("    unknown command → {unknown}");
    let missing = run(&mut session, 3, json!({"command": "aeolus.sortDocument"}));
    println!("    missing argument → {missing}");

    println!("── the already-canonical document ──");
    let _ = handle_message(
        &mut session,
        notification(
            "textDocument/didChange",
            json!({
                "textDocument": {"uri": URI, "version": 2},
                "contentChanges": [{"text": "fn card() {\n    let a = \"flex-col items-center\";\n}\n"}]
            }),
        ),
    );
    let quiet = run(
        &mut session,
        4,
        json!({"command": "aeolus.sortDocument", "arguments": [URI]}),
    );
    println!("    edits: {}", quiet.as_array().map_or(0, Vec::len));
}
