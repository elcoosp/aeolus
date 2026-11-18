// ---------------------------------------------------------------------------
// The executeCommand contracts (the server's command surface): the
// `aeolus.sortDocument` command rides the class-sort seam's editor face —
// the same canonical-order edits the `textDocument/formatting` request
// answers, invoked as a command (an editor keybinding, a refactoring
// menu, a code-action's follow-up — the protocol's own extension point
// where a client drives the server). The command table is CLOSED and
// advertised (one command, the sort face): an unknown command answers
// the protocol's own null — the state machine's quiet vocabulary — and
// a command without its argument rides the same way. The edits are the
// engine's own `format` answer, byte for byte.
//
// Written FIRST (the RED spec).
// ---------------------------------------------------------------------------

use aeolus_lsp::{AeolusEngine, EngineConfig, Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// The document the command answers over: two out-of-order class
/// strings the sort seam re-joins (the variant-prefixed sibling must
/// ride AFTER its utility — `hover:bg-primary flex` is the reversed
/// form), one already-canonical string the walk skips.
const DOC: &str = concat!(
    "fn card() {\n",
    "    let a = \"rounded-md p-4 flex-col gap-2\";\n",
    "    let b = \"hover:bg-primary flex\";\n",
    "    let c = \"flex-col items-center\";\n",
    "}\n",
);

const URI: &str = "file:///card.rs";

fn notification(method: &str, params: serde_json::Value) -> Message {
    Message::Notification(Notification {
        method: method.to_string(),
        params,
    })
}

fn request(id: i32, method: &str, params: serde_json::Value) -> Message {
    Message::Request(Request {
        id: RequestId::from(id),
        method: method.to_string(),
        params,
    })
}

fn initialize(session: &mut Session) {
    let handled = handle_message(
        session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    assert!(!handled.outgoing.is_empty(), "initialize answers");
}

fn wire_session() -> Session {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let _ = handle_message(
        &mut session,
        notification(
            "textDocument/didOpen",
            json!({
                "textDocument": {"uri": URI, "languageId": "rust", "version": 1, "text": DOC}
            }),
        ),
    );
    session
}

#[test]
fn initialize_advertises_the_execute_command_provider() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("initialize answers a response");
    };
    let result = response.result.expect("the capabilities carry a result");
    let commands = result["capabilities"]["executeCommandProvider"]["commands"]
        .as_array()
        .expect("the command table rides the capability");
    assert_eq!(
        commands.len(),
        1,
        "the table is closed: the sort face, the one command"
    );
    assert_eq!(commands[0], "aeolus.sortDocument");
}

#[test]
fn the_command_answers_the_formatting_edits() {
    let mut session = wire_session();
    let handled = handle_message(
        &mut session,
        request(
            1,
            "workspace/executeCommand",
            json!({
                "command": "aeolus.sortDocument",
                "arguments": [URI]
            }),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the command answers");
    };
    let result = response
        .result
        .expect("an out-of-order document answers edits");
    let edits = result.as_array().expect("an array of edits");
    // The two out-of-order strings ride one edit each; the
    // already-canonical third answers none.
    assert_eq!(edits.len(), 2, "one edit per out-of-order class string");
    // The edits ARE the engine's own format answer, byte for byte.
    let mut engine = AeolusEngine::new(EngineConfig::default());
    engine.open(URI, DOC, 1);
    let expected = engine.format(URI);
    assert_eq!(edits.len(), expected.len());
    for (edit, source) in edits.iter().zip(&expected) {
        assert_eq!(
            edit["newText"], source.new_text,
            "the command rides the sort seam's canonical join"
        );
        assert_eq!(
            edit["range"]["start"]["line"],
            json!(source.range.start.line)
        );
    }
}

#[test]
fn an_already_canonical_document_answers_an_empty_list() {
    let mut session = wire_session();
    let handled = handle_message(
        &mut session,
        notification(
            "textDocument/didChange",
            json!({
                "textDocument": {"uri": URI, "version": 2},
                "contentChanges": [{"text": "fn card() {\n    let a = \"flex-col items-center\";\n}\n"}]
            }),
        ),
    );
    let _ = handled;
    let handled = handle_message(
        &mut session,
        request(
            1,
            "workspace/executeCommand",
            json!({
                "command": "aeolus.sortDocument",
                "arguments": [URI]
            }),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the command answers");
    };
    let result = response
        .result
        .expect("the canonical document answers a result");
    assert_eq!(
        result.as_array().map(Vec::len),
        Some(0),
        "the no-op contract: an already-canonical string answers no edit"
    );
}

#[test]
fn an_unknown_uri_answers_an_empty_list() {
    let mut session = wire_session();
    let handled = handle_message(
        &mut session,
        request(
            1,
            "workspace/executeCommand",
            json!({
                "command": "aeolus.sortDocument",
                "arguments": ["file:///not-open.rs"]
            }),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the command answers");
    };
    let result = response.result.expect("the command answers a result");
    assert_eq!(
        result.as_array().map(Vec::len),
        Some(0),
        "a closed document answers the honest empty"
    );
}

#[test]
fn an_unknown_command_answers_null() {
    let mut session = wire_session();
    let handled = handle_message(
        &mut session,
        request(
            1,
            "workspace/executeCommand",
            json!({
                "command": "aeolus.notACommand",
                "arguments": [URI]
            }),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the command answers");
    };
    assert!(
        response.result.is_none() || response.result == Some(json!(null)),
        "the closed command table answers the protocol's own null"
    );
}

#[test]
fn the_command_without_its_argument_answers_null() {
    let mut session = wire_session();
    let handled = handle_message(
        &mut session,
        request(
            1,
            "workspace/executeCommand",
            json!({
                "command": "aeolus.sortDocument"
            }),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the command answers");
    };
    assert!(
        response.result.is_none() || response.result == Some(json!(null)),
        "no URI named, no walk run — the honest null"
    );
}

#[test]
fn the_pre_handshake_gate_holds() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(
            1,
            "workspace/executeCommand",
            json!({
                "command": "aeolus.sortDocument",
                "arguments": [URI]
            }),
        ),
    );
    assert!(
        handled.outgoing.is_empty(),
        "no request answers before the initialize handshake"
    );
}

#[test]
fn malformed_params_answer_null() {
    let mut session = wire_session();
    let handled = handle_message(
        &mut session,
        request(1, "workspace/executeCommand", json!(null)),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the malformed request still answers");
    };
    assert!(
        response.result.is_none() || response.result == Some(json!(null)),
        "malformed params answer the protocol's own empty"
    );
}
