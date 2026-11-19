//! Consumer demonstration: the range-scoped highlighter — the wire's
//! `textDocument/semanticTokens/range` (LSP 3.16) answered headless.
//! The highlighter feed already answered the whole-document
//! `semanticTokens/full`; the range form restricts the SAME walk to the
//! candidates the requested byte range touches — a candidate whose span
//! intersects the range rides its token whole (a partial selection
//! still names the token), a candidate wholly outside stays out AND
//! never compiles (the walk's win over the full legend), the
//! whole-document range IS the full answer byte for byte, and a
//! reversed range answers the honest empty. Each answer re-baselines
//! the delta encoding from its own start — the wire's own contract.
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_semantic_tokens_range
//! ```

use aeolus_lsp::{Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// A synthetic .rs document: one lowered string, one themed string with
/// variant prefixes, one refused string — the full walk's three
/// verdicts, at three lines the range can slice.
const DOC: &str = concat!(
    "fn card() {\n",
    "    let a = \"p-4 flex-col gap-2 rounded-md\";\n",
    "    let b = \"hover:bg-primary/90 dark:shadow-lg\";\n",
    "    let bad = \"not-a-root\";\n",
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
        }
    })
}

/// One semanticTokens/range round trip: the response's flat delta
/// array, decoded onto one `(line, column, length, legend)` row per
/// token (the protocol's own algorithm, run in reverse).
fn run(session: &mut Session, id: i32, params: serde_json::Value) -> Vec<(u32, u32, u32, u32)> {
    let handled = handle_message(
        session,
        request(id, "textDocument/semanticTokens/range", params),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the range request answers");
    };
    let words: Vec<u32> = response.result.unwrap_or(json!(null))["data"]
        .as_array()
        .expect("the flat delta array rides")
        .iter()
        .map(|value| value.as_u64().expect("a u32 word") as u32)
        .collect();
    let mut out = Vec::new();
    let mut prev_line = 0u32;
    let mut prev_character = 0u32;
    for group in words.chunks_exact(5) {
        let (delta_line, delta_start) = (group[0], group[1]);
        let line = prev_line + delta_line;
        let character = if delta_line == 0 {
            prev_character + delta_start
        } else {
            delta_start
        };
        out.push((line, character, group[2], group[3]));
        prev_line = line;
        prev_character = character;
    }
    out
}

/// One whole-document semanticTokens/full round trip (the parity
/// comparison's other half).
fn run_full(session: &mut Session, id: i32) -> Vec<(u32, u32, u32, u32)> {
    let handled = handle_message(
        session,
        request(
            id,
            "textDocument/semanticTokens/full",
            json!({"textDocument": {"uri": URI}}),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the full request answers");
    };
    let words: Vec<u32> = response.result.unwrap_or(json!(null))["data"]
        .as_array()
        .expect("the flat delta array rides")
        .iter()
        .map(|value| value.as_u64().expect("a u32 word") as u32)
        .collect();
    let mut out = Vec::new();
    let mut prev_line = 0u32;
    let mut prev_character = 0u32;
    for group in words.chunks_exact(5) {
        let (delta_line, delta_start) = (group[0], group[1]);
        let line = prev_line + delta_line;
        let character = if delta_line == 0 {
            prev_character + delta_start
        } else {
            delta_start
        };
        out.push((line, character, group[2], group[3]));
        prev_line = line;
        prev_character = character;
    }
    out
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

    println!("── a range over one string (line 1) ──");
    for (line, character, length, legend) in run(&mut session, 1, range(1, 0, 2, 0)) {
        println!(
            "    line {line} col {character} len {length} legend {legend} ({})",
            if legend == 0 { "class" } else { "refused" }
        );
    }

    println!("── a range over the themed string (line 2) ──");
    for (line, character, length, legend) in run(&mut session, 2, range(2, 0, 3, 0)) {
        println!(
            "    line {line} col {character} len {length} legend {legend} ({})",
            if legend == 0 { "class" } else { "refused" }
        );
    }

    println!("── the honest empties ──");
    let blank = run(&mut session, 3, range(0, 0, 1, 5));
    println!("    a blank span → {} token(s)", blank.len());
    let reversed = run(&mut session, 4, range(2, 10, 1, 5));
    println!("    reversed range → {} token(s)", reversed.len());

    println!("── the whole document: the full walk byte for byte ──");
    let whole = run(&mut session, 5, range(0, 0, DOC.lines().count() as u64, 0));
    let full = run_full(&mut session, 6);
    println!(
        "    range tokens: {}, full tokens: {} — equal: {}",
        whole.len(),
        full.len(),
        whole == full
    );
}
