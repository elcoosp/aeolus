//! Consumer demonstration: the selection-scoped sort — the wire's
//! `textDocument/rangeFormatting` (LSP 3.12) answered headless. The
//! class-sort seam's editor face already answered the whole-document
//! `textDocument/formatting`; the range form restricts the SAME walk to
//! the class strings the requested byte range touches — a selection
//! over part of a string still names the string (the whole string
//! reorders, never a fragment), a string outside the range stays out,
//! an already-canonical string answers nothing wherever the range sits,
//! and a reversed range answers the honest empty. The whole-document
//! range IS the formatting request's own answer, byte for byte.
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_range_format
//! ```

use aeolus_lsp::{Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// A synthetic .rs document: two out-of-order class strings, one
/// already-canonical string the walk skips.
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

fn range(start_line: u64, start_char: u64, end_line: u64, end_char: u64) -> serde_json::Value {
    json!({
        "textDocument": {"uri": URI},
        "range": {
            "start": {"line": start_line, "character": start_char},
            "end": {"line": end_line, "character": end_char}
        },
        "options": {"tabSize": 4, "insertSpaces": true}
    })
}

/// One range-formatting round trip: the response's JSON result.
fn run(session: &mut Session, id: i32, params: serde_json::Value) -> serde_json::Value {
    let handled = handle_message(session, request(id, "textDocument/rangeFormatting", params));
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the range request answers");
    };
    response.result.unwrap_or(json!(null))
}

/// One whole-document formatting round trip (the parity comparison's
/// other half).
fn run_formatting(session: &mut Session, id: i32, params: serde_json::Value) -> serde_json::Value {
    let handled = handle_message(session, request(id, "textDocument/formatting", params));
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the formatting request answers");
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

    println!("── a range over one string ──");
    let one = run(&mut session, 1, range(1, 0, 2, 0));
    for edit in one.as_array().expect("an array of edits") {
        println!(
            "    line {} → {}",
            edit["range"]["start"]["line"], edit["newText"]
        );
    }

    println!("── a partial selection still names the string ──");
    let partial = run(&mut session, 2, range(1, 24, 1, 35));
    println!(
        "    edits: {} (the string's own content span rides)",
        partial.as_array().map_or(0, Vec::len)
    );

    println!("── a range over two strings ──");
    let two = run(&mut session, 3, range(1, 0, 3, 0));
    println!("    edits: {}", two.as_array().map_or(0, Vec::len));

    println!("── the honest empties ──");
    let canonical = run(&mut session, 4, range(3, 0, 4, 0));
    println!(
        "    already-canonical line → {} edit(s)",
        canonical.as_array().map_or(0, Vec::len)
    );
    let reversed = run(&mut session, 5, range(2, 10, 1, 5));
    println!(
        "    reversed range → {} edit(s)",
        reversed.as_array().map_or(0, Vec::len)
    );

    println!("── the whole document: the formatting answer byte for byte ──");
    let whole = run(&mut session, 6, range(0, 0, DOC.lines().count() as u64, 0));
    let formatted = run_formatting(
        &mut session,
        7,
        json!({
            "textDocument": {"uri": URI},
            "options": {"tabSize": 4, "insertSpaces": true}
        }),
    );
    println!(
        "    rangeFormatting edits: {}, formatting edits: {} — equal: {}",
        whole.as_array().map_or(0, Vec::len),
        formatted.as_array().map_or(0, Vec::len),
        whole == formatted
    );
}
