//! The server loop's wire contracts (the P5 row's LSP half): a
//! `handle`-driven JSON-RPC session — initialize capabilities,
//! didOpen/didChange publish diagnostics, hover requests answer from
//! the engine, shutdown closes cleanly — every message deterministic
//! and testable without a stdio pipe (the loop is the stdio transport's
//! thin shell; the STATE MACHINE is the crate's own surface).
//!
//! Written FIRST (the RED spec).

use aeolus_lsp::{Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

const DOC: &str = "fn card() {\n    let s = \"p-4 not-a-root\";\n}\n";

fn uri() -> String {
    "file:///card.rs".to_string()
}

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

/// The didOpen notification the harness reuses.
fn did_open(uri: &str, version: i32, text: &str) -> Message {
    notification(
        "textDocument/didOpen",
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "rust",
                "version": version,
                "text": text,
            }
        }),
    )
}

/// The initialize handshake the harness reuses (the protocol gate: no
/// request answers before it).
fn initialize(session: &mut Session) {
    let handled = handle_message(
        session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    assert!(!handled.outgoing.is_empty(), "initialize answers");
}

#[test]
fn initialize_advertises_hover_and_full_sync() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    assert!(!handled.exit, "initialize does not end the session");
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("initialize answers a response");
    };
    assert_eq!(response.id, RequestId::from(0));
    let result = response.result.expect("the capabilities carry a result");
    assert_eq!(result["capabilities"]["hoverProvider"], true);
    assert_eq!(
        result["capabilities"]["textDocumentSync"]["openClose"],
        true
    );
    assert_eq!(result["capabilities"]["textDocumentSync"]["change"], 1);
    // The server info names the pipeline it rides.
    assert_eq!(result["serverInfo"]["name"], "aeolus");
}

#[test]
fn did_open_publishes_diagnostics() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(&mut session, did_open(&uri(), 1, DOC));
    let Some(Message::Notification(note)) = handled.outgoing.into_iter().find(
        |m| matches!(m, Message::Notification(n) if n.method == "textDocument/publishDiagnostics"),
    ) else {
        panic!("didOpen publishes diagnostics");
    };
    assert_eq!(note.params["uri"], uri());
    let diagnostics = note.params["diagnostics"].as_array().expect("an array");
    assert!(
        diagnostics.iter().any(|d| d["message"]
            .as_str()
            .unwrap_or_default()
            .contains("not-a-root")),
        "the unknown root rides the publish: {diagnostics:?}"
    );
}

#[test]
fn did_change_republishes() {
    let mut session = Session::new(Default::default());
    let _ = handle_message(&mut session, did_open(&uri(), 1, DOC));
    let handled = handle_message(
        &mut session,
        notification(
            "textDocument/didChange",
            json!({
                "textDocument": {"uri": uri(), "version": 2},
                "contentChanges": [{"text": "fn clean() {}\n"}]
            }),
        ),
    );
    let Some(Message::Notification(note)) = handled.outgoing.into_iter().find(
        |m| matches!(m, Message::Notification(n) if n.method == "textDocument/publishDiagnostics"),
    ) else {
        panic!("didChange republishes diagnostics");
    };
    let diagnostics = note.params["diagnostics"].as_array().expect("an array");
    assert!(
        diagnostics.is_empty(),
        "the clean document: {diagnostics:?}"
    );
}

#[test]
fn did_close_publishes_the_empty_answer() {
    let mut session = Session::new(Default::default());
    let _ = handle_message(&mut session, did_open(&uri(), 1, DOC));
    let handled = handle_message(
        &mut session,
        notification(
            "textDocument/didClose",
            json!({"textDocument": {"uri": uri()}}),
        ),
    );
    // The LSP contract: a closed document publishes an EMPTY diagnostic
    // set so the editor clears its squiggles.
    let Some(Message::Notification(note)) = handled.outgoing.into_iter().find(
        |m| matches!(m, Message::Notification(n) if n.method == "textDocument/publishDiagnostics"),
    ) else {
        panic!("didClose clears the publish");
    };
    let diagnostics = note.params["diagnostics"].as_array().expect("an array");
    assert!(diagnostics.is_empty());
}

#[test]
fn hover_request_answers_from_the_engine() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let _ = handle_message(&mut session, did_open(&uri(), 1, DOC));
    // The cursor inside `p-4`.
    let start = DOC.find("\"p-4 not-a-root\"").expect("the fixture holds") + 3;
    let line = DOC[..start].matches('\n').count();
    let line_start = DOC[..start].rfind('\n').map_or(0, |i| i + 1);
    let character = DOC[line_start..start]
        .chars()
        .map(char::len_utf16)
        .sum::<usize>();
    let handled = handle_message(
        &mut session,
        request(
            7,
            "textDocument/hover",
            json!({
                "textDocument": {"uri": uri()},
                "position": {"line": line, "character": character}
            }),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("hover answers a response");
    };
    assert_eq!(response.id, RequestId::from(7));
    let result = response.result.expect("the cursor is on a class");
    let contents = &result["contents"]["value"];
    assert!(
        contents.as_str().unwrap_or_default().contains("p-4"),
        "the class renders: {result}"
    );
}

#[test]
fn hover_off_a_string_answers_null() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let _ = handle_message(&mut session, did_open(&uri(), 1, DOC));
    let handled = handle_message(
        &mut session,
        request(
            8,
            "textDocument/hover",
            json!({
                "textDocument": {"uri": uri()},
                "position": {"line": 0, "character": 0}
            }),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("hover answers a response");
    };
    assert!(response.result.is_none(), "the honest empty answer");
}

#[test]
fn shutdown_answers_null_and_exit_ends_the_loop() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(9, "shutdown", serde_json::Value::Null),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("shutdown answers a response");
    };
    assert!(response.result.is_none());
    // The exit notification ends the session (the transport's own
    // contract): the handler reports the end via the `exit` flag.
    let handled = handle_message(&mut session, notification("exit", serde_json::Value::Null));
    assert!(handled.exit, "exit ends the session");
    assert!(handled.outgoing.is_empty());
}

#[test]
fn unknown_methods_ride_silently() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(10, "workspace/symbols", json!({"query": "p-4"})),
    );
    assert!(
        handled.outgoing.is_empty(),
        "an unhandled request answers nothing"
    );
    assert!(!handled.exit);
    let handled = handle_message(
        &mut session,
        notification("$/randomPing", json!({"value": 42})),
    );
    assert!(handled.outgoing.is_empty());
    assert!(!handled.exit);
}

#[test]
fn requests_before_initialize_ride_silently() {
    // The protocol gate: a hover before the handshake answers nothing
    // (the client has no capabilities contract yet).
    let mut session = Session::new(Default::default());
    let _ = handle_message(&mut session, did_open(&uri(), 1, DOC));
    let handled = handle_message(
        &mut session,
        request(
            11,
            "textDocument/hover",
            json!({
                "textDocument": {"uri": uri()},
                "position": {"line": 1, "character": 16}
            }),
        ),
    );
    assert!(
        handled.outgoing.is_empty(),
        "pre-initialize hover: {handled:?}"
    );
    assert!(!handled.exit);
}
