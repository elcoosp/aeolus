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

#[test]
fn requests_after_shutdown_ride_silently() {
    // The shutdown flag is the lifecycle's own gate (not dead state): a
    // request after the client shut down has no capabilities contract
    // left, so it rides the same quiet-drop — and a repeated `shutdown`
    // still answers null (a client waiting on it must not hang).
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let _ = handle_message(
        &mut session,
        request(9, "shutdown", serde_json::Value::Null),
    );
    let _ = handle_message(&mut session, did_open(&uri(), 1, DOC));
    let handled = handle_message(
        &mut session,
        request(
            12,
            "textDocument/hover",
            json!({
                "textDocument": {"uri": uri()},
                "position": {"line": 1, "character": 16}
            }),
        ),
    );
    assert!(
        handled.outgoing.is_empty(),
        "post-shutdown hover rides silently: {handled:?}"
    );
    let handled = handle_message(
        &mut session,
        request(13, "shutdown", serde_json::Value::Null),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("a repeated shutdown still answers (never a hang)");
    };
    assert!(response.result.is_none());
}
// ---------------------------------------------------------------------------
// The completion wire (the third request the server answers): the
// capability rides initialize, the request maps the engine's answer
// onto the protocol's CompletionList, the gate and the malformed-params
// contract match the hover path's own.
// ---------------------------------------------------------------------------

const COMPLETION_DOC: &str = "fn card() {\n    let s = \"bg-pri\";\n}\n";

/// The completion position: right after `bg-pri` (the token's end).
fn completion_request(id: i32, uri: &str) -> Message {
    let offset = COMPLETION_DOC.find("bg-pri").expect("the fixture holds") + "bg-pri".len();
    let line = COMPLETION_DOC[..offset].matches('\n').count() as u32;
    let line_start = COMPLETION_DOC[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = COMPLETION_DOC[line_start..offset]
        .chars()
        .map(|c| c.len_utf16() as u32)
        .sum::<u32>();
    request(
        id,
        "textDocument/completion",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character }
        }),
    )
}

#[test]
fn initialize_advertises_completion() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("initialize answers a response");
    };
    let result = response.result.expect("the capabilities carry a result");
    let provider = &result["capabilities"]["completionProvider"];
    assert!(
        !provider.is_null(),
        "the completion capability rides: {result}"
    );
    assert_eq!(
        provider["resolveProvider"], false,
        "the engine's answer is final — no resolve round-trip"
    );
    let triggers = provider["triggerCharacters"]
        .as_array()
        .expect("the trigger characters ride");
    let triggers: Vec<&str> = triggers
        .iter()
        .map(|value| value.as_str().expect("a string trigger"))
        .collect();
    assert!(
        triggers.contains(&"-") && triggers.contains(&":"),
        "the class grammar's own separators trigger: {triggers:?}"
    );
}

#[test]
fn completion_answers_a_completion_list() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    handle_message(&mut session, did_open(&uri(), 1, COMPLETION_DOC));
    let handled = handle_message(&mut session, completion_request(7, &uri()));
    assert!(!handled.exit, "completion does not end the session");
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("completion answers a response");
    };
    assert_eq!(response.id, RequestId::from(7));
    let result = response.result.expect("the completion carries a result");
    assert_eq!(result["isIncomplete"], false, "the answer is final");
    let items = result["items"].as_array().expect("the items ride");
    assert!(
        items.iter().any(|item| item["label"] == "bg-primary"),
        "the theme token completes over the wire: {items:?}"
    );
    let item = items
        .iter()
        .find(|item| item["label"] == "bg-primary")
        .expect("the item");
    // CompletionItemKind::VALUE = 12 (the theme-token mapping), the
    // detail names the token, the rank renders as the sort key.
    assert_eq!(item["kind"], 12);
    assert!(
        item["detail"]
            .as_str()
            .expect("the detail rides")
            .contains("primary"),
        "the detail names the token: {}",
        item["detail"]
    );
    assert!(
        item["sortText"].is_string(),
        "the rank renders as the sort key: {}",
        item["sortText"]
    );
}

#[test]
fn completion_before_initialize_rides_silently() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(&mut session, completion_request(7, &uri()));
    assert!(
        handled.outgoing.is_empty(),
        "requests before initialize ride the quiet-drop contract"
    );
}

#[test]
fn completion_with_malformed_params_answers_null() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let handled = handle_message(
        &mut session,
        request(8, "textDocument/completion", json!({"uri": 42})),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the request still answers (never a hang)");
    };
    assert_eq!(response.result, Some(serde_json::Value::Null));
}
// ---------------------------------------------------------------------------
// The documentColor wire (the fourth request): the capability rides
// initialize, the request maps the engine's swatches onto the
// protocol's ColorInformation (the HSL the protocol paints with).
// ---------------------------------------------------------------------------

const COLOUR_DOC: &str = "fn card() {\n    let s = \"bg-primary\";\n}\n";

#[test]
fn initialize_advertises_colour_swatches() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("initialize answers a response");
    };
    let result = response.result.expect("the capabilities carry a result");
    assert_eq!(
        result["capabilities"]["colorProvider"], true,
        "the colour-provider capability rides: {result}"
    );
}

#[test]
fn document_color_answers_swatches() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    handle_message(&mut session, did_open(&uri(), 1, COLOUR_DOC));
    let handled = handle_message(
        &mut session,
        request(
            9,
            "textDocument/documentColor",
            json!({ "textDocument": { "uri": uri() } }),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("documentColor answers a response");
    };
    let result = response.result.expect("the swatches carry a result");
    let items = result.as_array().expect("the colour array rides");
    assert_eq!(items.len(), 1, "{result}");
    let item = &items[0];
    let range = &item["range"];
    assert_eq!(range["start"]["line"], 1);
    let color = &item["color"];
    assert_eq!(color["alpha"], 1.0, "the light tier's swatch is opaque");
    for key in ["red", "green", "blue"] {
        let value = color[key]
            .as_f64()
            .expect("the protocol's rgba channels ride");
        assert!(
            (0.0..=1.0).contains(&value),
            "{key} is the protocol's 0..1 fraction: {value}"
        );
    }
}

#[test]
fn document_color_before_initialize_rides_silently() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(
            9,
            "textDocument/documentColor",
            json!({ "textDocument": { "uri": uri() } }),
        ),
    );
    assert!(handled.outgoing.is_empty(), "the quiet-drop contract");
}

#[test]
fn document_color_with_malformed_params_answers_null() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let handled = handle_message(
        &mut session,
        request(10, "textDocument/documentColor", json!({"uri": 42})),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the request still answers (never a hang)");
    };
    assert_eq!(response.result, Some(serde_json::Value::Null));
}
// ---------------------------------------------------------------------------
// The documentDiagnostic pull (the fifth request, LSP 3.17's pull
// model): the capability rides initialize, the request maps the
// engine's analyze walk onto the protocol's Full report — the SAME
// findings the push channel publishes, pulled on demand. The battery
// was written FIRST; the RED capture is the quiet-drop on the
// unadvertised method.
// ---------------------------------------------------------------------------

const PULL_DOC: &str = "fn card() {\n    let s = \"p-4 not-a-root\";\n}\n";

fn pull_request(id: i32, uri: &str) -> Message {
    request(
        id,
        "textDocument/documentDiagnostic",
        json!({ "textDocument": { "uri": uri } }),
    )
}

#[test]
fn initialize_advertises_the_pull_diagnostics() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("initialize answers a response");
    };
    let result = response.result.expect("the capabilities carry a result");
    let provider = &result["capabilities"]["diagnosticProvider"];
    assert!(
        !provider.is_null(),
        "the pull-diagnostics capability rides: {result}"
    );
    assert_eq!(
        provider["interFileDependencies"], false,
        "the engine answers per document — no cross-file model"
    );
    assert_eq!(
        provider["workspaceDiagnostics"], false,
        "no workspace-wide pull: the document map is the answer's scope"
    );
}

#[test]
fn pull_diagnostics_answer_the_full_report() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    handle_message(&mut session, did_open(&uri(), 1, PULL_DOC));
    let handled = handle_message(&mut session, pull_request(11, &uri()));
    assert!(!handled.exit, "the pull does not end the session");
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the pull answers a response");
    };
    assert_eq!(response.id, RequestId::from(11));
    let result = response.result.expect("the report carries a result");
    assert_eq!(
        result["kind"], "full",
        "the engine always recomputes: {result}"
    );
    let items = result["items"].as_array().expect("the items ride");
    assert_eq!(items.len(), 1, "the one miss rides: {result}");
    let item = &items[0];
    assert_eq!(item["source"], "aeolus");
    assert_eq!(
        item["severity"], 2,
        "the miss rides the push channel's own WARNING tier (findings are non-fatal)"
    );
    assert!(
        item["code"]
            .as_str()
            .expect("the stable machine code rides")
            .starts_with("aeolus::compile::"),
        "the code is the pipeline's own: {}",
        item["code"]
    );
    assert!(
        item["message"]
            .as_str()
            .expect("the message rides")
            .contains("not-a-root"),
        "the message names the token: {}",
        item["message"]
    );
    assert_eq!(item["range"]["start"]["line"], 1, "the range is byte-true");
}

#[test]
fn pull_diagnostics_answer_the_empty_report_on_a_clean_document() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    handle_message(&mut session, did_open(&uri(), 1, "fn clean() {}\n"));
    let handled = handle_message(&mut session, pull_request(11, &uri()));
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the pull answers a response");
    };
    let result = response.result.expect("the report carries a result");
    assert_eq!(result["kind"], "full");
    let items = result["items"].as_array().expect("the items ride");
    assert!(
        items.is_empty(),
        "the clean document pulls an empty report: {result}"
    );
}

#[test]
fn pull_diagnostics_mirror_the_push_channel() {
    // The pull model's honesty contract: the pulled report carries the
    // SAME findings the push channel published for the same document —
    // one walk, two transports.
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let opened = handle_message(&mut session, did_open(&uri(), 1, PULL_DOC));
    let Some(Message::Notification(note)) = opened.outgoing.into_iter().find(
        |m| matches!(m, Message::Notification(n) if n.method == "textDocument/publishDiagnostics"),
    ) else {
        panic!("didOpen publishes diagnostics");
    };
    let pushed = note.params["diagnostics"].as_array().expect("an array");
    let handled = handle_message(&mut session, pull_request(12, &uri()));
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the pull answers a response");
    };
    let result = response.result.expect("the report carries a result");
    let pulled = result["items"].as_array().expect("the items ride");
    assert_eq!(pushed.len(), pulled.len(), "same findings, both channels");
    let pushed_codes: Vec<&str> = pushed
        .iter()
        .map(|d| d["code"].as_str().expect("the code rides"))
        .collect();
    let pulled_codes: Vec<&str> = pulled
        .iter()
        .map(|d| d["code"].as_str().expect("the code rides"))
        .collect();
    assert_eq!(
        pushed_codes, pulled_codes,
        "the codes agree channel to channel"
    );
}

#[test]
fn pull_diagnostics_before_initialize_rides_silently() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(&mut session, pull_request(11, &uri()));
    assert!(
        handled.outgoing.is_empty(),
        "requests before initialize ride the quiet-drop contract"
    );
}

#[test]
fn pull_diagnostics_with_malformed_params_answers_null() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let handled = handle_message(
        &mut session,
        request(11, "textDocument/documentDiagnostic", json!({"uri": 42})),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the request still answers (never a hang)");
    };
    assert_eq!(response.result, Some(serde_json::Value::Null));
}
// ---------------------------------------------------------------------------
// The semanticTokens wire (the sixth request, LSP 3.16's highlighter
// feed): the capability rides initialize with the legend named, the
// request maps the engine's absolute tokens onto the protocol's delta
// encoding. The legend's ORDER is the wire contract the encoder indexes
// — the tests pin it beside the deltas.
// ---------------------------------------------------------------------------

fn semantic_tokens_request(id: i32, uri: &str) -> Message {
    request(
        id,
        "textDocument/semanticTokens/full",
        json!({ "textDocument": { "uri": uri } }),
    )
}

#[test]
fn initialize_advertises_semantic_tokens() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("initialize answers a response");
    };
    let result = response.result.expect("the capabilities carry a result");
    let provider = &result["capabilities"]["semanticTokensProvider"];
    assert!(
        !provider.is_null(),
        "the semantic-tokens capability rides: {result}"
    );
    let legend = &provider["legend"];
    let token_types = legend["tokenTypes"].as_array().expect("the legend rides");
    let types: Vec<&str> = token_types
        .iter()
        .map(|value| value.as_str().expect("a legend entry is a string"))
        .collect();
    // The legend the engine's kinds index into: a lowered candidate is
    // the protocol's own `class`, a refused one the server's custom
    // `refused` type (an editor that does not know it renders neutral).
    assert_eq!(types.first(), Some(&"class"), "index 0: the lowered class");
    assert_eq!(types.get(1), Some(&"refused"), "index 1: the typed refusal");
    assert!(
        legend["tokenModifiers"]
            .as_array()
            .is_some_and(std::vec::Vec::is_empty),
        "no modifiers: the kinds carry the whole story"
    );
    assert_eq!(
        provider["full"], true,
        "the full-document walk is the advertised shape"
    );
}

#[test]
fn semantic_tokens_answer_delta_encoded() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    handle_message(&mut session, did_open(&uri(), 1, PULL_DOC));
    let handled = handle_message(&mut session, semantic_tokens_request(13, &uri()));
    assert!(!handled.exit, "the walk does not end the session");
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("semanticTokens answers a response");
    };
    assert_eq!(response.id, RequestId::from(13));
    let result = response.result.expect("the tokens carry a result");
    let data = result["data"]
        .as_array()
        .expect("the flat delta array rides");
    assert_eq!(data.len(), 10, "two candidates, five u32s each: {result}");
    let words: Vec<u32> = data
        .iter()
        .map(|value| value.as_u64().expect("a u32 word") as u32)
        .collect();
    // The fixture: `p-4 not-a-root` on line 1 — the first token's
    // deltas are ABSOLUTE (line, character); the second rides the
    // deltas the protocol re-derives.
    let p4 = PULL_DOC
        .find("\"p-4 not-a-root\"")
        .expect("the fixture holds")
        + 1;
    let line_start = PULL_DOC[..p4].rfind('\n').map_or(0, |i| i + 1);
    let p4_character = PULL_DOC[line_start..p4]
        .chars()
        .map(char::len_utf16)
        .sum::<usize>() as u32;
    assert_eq!(words[0], 1, "deltaLine: the token's line (absolute first)");
    assert_eq!(words[1], p4_character, "deltaStart: the absolute column");
    assert_eq!(words[2], 3, "length: p-4");
    assert_eq!(words[3], 0, "tokenType: class (the legend's index 0)");
    assert_eq!(words[4], 0, "no modifiers");
    // `not-a-root` follows on the SAME line: deltaLine 0, deltaStart
    // relative to the PREVIOUS TOKEN's start (the protocol's own rule)
    // — the miss's absolute column (p-4's + the spelling + the space)
    // minus the previous token's start.
    assert_eq!(words[5], 0, "deltaLine: same line");
    assert_eq!(
        words[6],
        p4_character + 3 + 1 - p4_character,
        "deltaStart: relative to the previous token's start"
    );
    assert_eq!(words[7], 10, "length: not-a-root");
    assert_eq!(words[8], 1, "tokenType: refused (the legend's index 1)");
    assert_eq!(words[9], 0, "no modifiers");
}

#[test]
fn semantic_tokens_answer_empty_on_a_clean_document() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    handle_message(&mut session, did_open(&uri(), 1, "fn clean() {}\n"));
    let handled = handle_message(&mut session, semantic_tokens_request(13, &uri()));
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("semanticTokens answers a response");
    };
    let result = response.result.expect("the tokens carry a result");
    let data = result["data"]
        .as_array()
        .expect("the flat delta array rides");
    assert!(data.is_empty(), "no candidates, no tokens: {result}");
}

#[test]
fn semantic_tokens_before_initialize_rides_silently() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(&mut session, semantic_tokens_request(13, &uri()));
    assert!(
        handled.outgoing.is_empty(),
        "requests before initialize ride the quiet-drop contract"
    );
}

#[test]
fn semantic_tokens_with_malformed_params_answers_null() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let handled = handle_message(
        &mut session,
        request(13, "textDocument/semanticTokens/full", json!({"uri": 42})),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the request still answers (never a hang)");
    };
    assert_eq!(response.result, Some(serde_json::Value::Null));
}
// ---------------------------------------------------------------------------
// The colorPresentation wire (the seventh request): the paired half of
// the colorProvider capability — the swatches advertise, the picker's
// presentations spell. The request's colour (the protocol's 0..1
// fractions) maps back onto the engine's byte rgba, the answer's
// spellings ride as the picker's edits over the requested range.
// ---------------------------------------------------------------------------

/// The documentColor → colorPresentation round trip the harness reuses:
/// the swatch the fourth request answers is the colour the seventh
/// spells back.
fn swatch_of(session: &mut Session) -> (serde_json::Value, serde_json::Value) {
    let handled = handle_message(
        session,
        request(
            9,
            "textDocument/documentColor",
            json!({ "textDocument": { "uri": uri() } }),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("documentColor answers a response");
    };
    let result = response.result.expect("the swatches carry a result");
    let item = &result.as_array().expect("the colour array rides")[0];
    (item["color"].clone(), item["range"].clone())
}

#[test]
fn colour_presentation_answers_the_spellings() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    handle_message(&mut session, did_open(&uri(), 1, COLOUR_DOC));
    let (color, range) = swatch_of(&mut session);
    let handled = handle_message(
        &mut session,
        request(
            14,
            "textDocument/colorPresentation",
            json!({
                "textDocument": { "uri": uri() },
                "color": color,
                "range": range,
            }),
        ),
    );
    assert!(!handled.exit, "the presentation does not end the session");
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("colorPresentation answers a response");
    };
    assert_eq!(response.id, RequestId::from(14));
    let result = response.result.expect("the presentations carry a result");
    let items = result.as_array().expect("the presentation array rides");
    assert!(
        !items.is_empty(),
        "the swatch's own colour spells back: {result}"
    );
    let primary = items
        .iter()
        .find(|item| item["label"] == "bg-primary")
        .expect("the opaque token spelling rides");
    // The edit writes the spelling over the range the picker names
    // (the colour's own range — the swap writes the class).
    let edit = &primary["textEdit"];
    assert_eq!(edit["newText"], "bg-primary");
    assert_eq!(edit["range"], range, "the requested range rides");
}

#[test]
fn colour_presentation_of_an_unmatched_colour_answers_empty() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    handle_message(&mut session, did_open(&uri(), 1, COLOUR_DOC));
    let handled = handle_message(
        &mut session,
        request(
            14,
            "textDocument/colorPresentation",
            json!({
                "textDocument": { "uri": uri() },
                "color": {"red": 0.004, "green": 0.008, "blue": 0.012, "alpha": 0.016},
                "range": {"start": {"line": 1, "character": 13}, "end": {"line": 1, "character": 23}},
            }),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("colorPresentation answers a response");
    };
    let result = response.result.expect("the presentations carry a result");
    let items = result.as_array().expect("the presentation array rides");
    assert!(
        items.is_empty(),
        "an unmatched colour answers the honest empty: {result}"
    );
}

#[test]
fn colour_presentation_before_initialize_rides_silently() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(
            14,
            "textDocument/colorPresentation",
            json!({
                "textDocument": { "uri": uri() },
                "color": {"red": 0.0, "green": 0.0, "blue": 0.0, "alpha": 1.0},
                "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}},
            }),
        ),
    );
    assert!(
        handled.outgoing.is_empty(),
        "requests before initialize ride the quiet-drop contract"
    );
}

#[test]
fn colour_presentation_with_malformed_params_answers_null() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let handled = handle_message(
        &mut session,
        request(14, "textDocument/colorPresentation", json!({"uri": 42})),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the request still answers (never a hang)");
    };
    assert_eq!(response.result, Some(serde_json::Value::Null));
}
// ---------------------------------------------------------------------------
// The inlayHint wire (the eighth request, LSP 3.17's inline answers): the
// capability rides initialize (no resolve round-trip — the answer is
// final, the completion's own contract), the request maps the engine's
// hints onto the protocol's InlayHint (the class's end position, the CSS
// label, the markdown tooltip, the type kind). The params' visible range
// is accepted and answered document-wide — the engine has no incremental
// model, the pull diagnostics' own contract.
// ---------------------------------------------------------------------------

fn inlay_hints_request(id: i32, uri: &str) -> Message {
    request(
        id,
        "textDocument/inlayHint",
        json!({
            "textDocument": { "uri": uri },
            "range": {
                "start": {"line": 0, "character": 0},
                "end": {"line": 9, "character": 0},
            },
        }),
    )
}

#[test]
fn initialize_advertises_inlay_hints() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("initialize answers a response");
    };
    let result = response.result.expect("the capabilities carry a result");
    let provider = &result["capabilities"]["inlayHintProvider"];
    assert!(
        !provider.is_null(),
        "the inlay-hint capability rides: {result}"
    );
    assert_eq!(
        provider["resolveProvider"], false,
        "the engine's answer is final — no resolve round-trip"
    );
}

#[test]
fn inlay_hints_answer_the_wire_shape() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    handle_message(&mut session, did_open(&uri(), 1, PULL_DOC));
    let handled = handle_message(&mut session, inlay_hints_request(15, &uri()));
    assert!(!handled.exit, "the hint request does not end the session");
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("inlayHint answers a response");
    };
    assert_eq!(response.id, RequestId::from(15));
    let result = response.result.expect("the hints carry a result");
    let hints = result.as_array().expect("the hint array rides");
    // The fixture `p-4 not-a-root`: one hint — the refused candidate
    // never rides (its story is the diagnostics' and the legend's).
    assert_eq!(hints.len(), 1, "{result}");
    let hint = &hints[0];
    // The hint rides at the class's END: `p-4` starts at column 13 on
    // line 1 (ASCII — bytes are UTF-16 columns), so the end is 16.
    let position = &hint["position"];
    assert_eq!(position["line"], 1);
    assert_eq!(position["character"], 16);
    // The label is the CSS arm's own declaration; the kind is the
    // protocol's type annotation (1); the tooltip is the markdown the
    // hover renders.
    assert!(
        hint["label"]
            .as_str()
            .expect("the label rides as a string")
            .contains("padding"),
        "the CSS label renders: {}",
        hint["label"]
    );
    assert_eq!(hint["kind"], 1, "the type-annotation kind");
    let tooltip = &hint["tooltip"];
    assert_eq!(
        tooltip["kind"], "markdown",
        "the tooltip is the hover's markdown"
    );
    assert!(
        tooltip["value"]
            .as_str()
            .expect("the tooltip carries a value")
            .contains("```css"),
        "the tooltip carries the CSS block: {}",
        tooltip["value"]
    );
}

#[test]
fn inlay_hints_before_initialize_rides_silently() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(&mut session, inlay_hints_request(15, &uri()));
    assert!(
        handled.outgoing.is_empty(),
        "requests before initialize ride the quiet-drop contract"
    );
}

#[test]
fn inlay_hints_with_malformed_params_answers_null() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let handled = handle_message(
        &mut session,
        request(15, "textDocument/inlayHint", json!({"uri": 42})),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the request still answers (never a hang)");
    };
    assert_eq!(response.result, Some(serde_json::Value::Null));
}

// ---------------------------------------------------------------------------
// The codeAction wire (the quick fixes: the finding, the edit, the
// capability) — the P5 row's suggestion seam at the editor.
// ---------------------------------------------------------------------------

const ACTIONS_DOC: &str = "fn card() {\n    let a = \"flex-clo\";\n}\n";

/// The initialize handshake's capability read (the answer JSON's
/// serverCapabilities object).
fn capabilities_of(session: &mut Session) -> serde_json::Value {
    let handled = handle_message(
        session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("initialize answers");
    };
    response.result.expect("the initialize result")
}

#[test]
fn initialize_advertises_the_code_action_provider() {
    let mut session = Session::new(Default::default());
    let capabilities = capabilities_of(&mut session);
    assert_eq!(
        capabilities["capabilities"]["codeActionProvider"],
        json!(true),
        "the quick-fix capability is advertised: {capabilities}"
    );
}

#[test]
fn initialize_advertises_the_document_highlight_provider() {
    let mut session = Session::new(Default::default());
    let capabilities = capabilities_of(&mut session);
    assert_eq!(
        capabilities["capabilities"]["documentHighlightProvider"],
        json!(true),
        "the highlight capability is advertised: {capabilities}"
    );
}

/// The codeAction request the harness reuses (the range's start is the
/// cursor the editor invokes the action on).
fn code_action_request(id: i32, uri: &str, line: u32, character: u32) -> Message {
    request(
        id,
        "textDocument/codeAction",
        json!({
            "textDocument": {"uri": uri},
            "range": {
                "start": {"line": line, "character": character},
                "end": {"line": line, "character": character},
            },
            "context": {"diagnostics": []},
        }),
    )
}

#[test]
fn code_action_answers_the_quick_fix_with_the_edit() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let _ = handle_message(&mut session, did_open(&uri(), 1, ACTIONS_DOC));
    // The cursor inside the miss: the offset the engine's own
    // conversions resolve (line 1, the candidate's second character).
    let handled = handle_message(&mut session, code_action_request(20, &uri(), 1, 15));
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the request answers");
    };
    let result = response.result.expect("the action list");
    let actions = result.as_array().expect("an array of actions");
    assert!(
        !actions.is_empty(),
        "the miss carries corrections: {result}"
    );
    let first = &actions[0];
    assert_eq!(first["kind"], "quickfix", "the editors' filter kind");
    assert!(
        first["title"]
            .as_str()
            .expect("a title")
            .contains("did you mean"),
        "the CLI-shaped title: {first}"
    );
    let diagnostics = first["diagnostics"]
        .as_array()
        .expect("the repaired finding");
    assert_eq!(diagnostics.len(), 1, "one finding per action");
    let edit = &first["edit"]["changes"][&uri()][0];
    assert_eq!(edit["newText"], "flex-col", "the corrected token: {edit}");
    assert_eq!(edit["range"]["start"]["line"], 1, "the finding's own range");
}

#[test]
fn code_action_outside_a_candidate_answers_empty() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let _ = handle_message(&mut session, did_open(&uri(), 1, ACTIONS_DOC));
    let handled = handle_message(&mut session, code_action_request(21, &uri(), 0, 2));
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the request answers");
    };
    assert_eq!(
        response.result,
        Some(serde_json::json!([])),
        "outside any candidate: the honest empty menu"
    );
}

#[test]
fn code_action_before_initialize_rides_silently() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(&mut session, code_action_request(22, &uri(), 1, 15));
    assert!(
        handled.outgoing.is_empty(),
        "requests before initialize ride the quiet-drop contract"
    );
}

#[test]
fn code_action_with_malformed_params_answers_null() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let handled = handle_message(
        &mut session,
        request(23, "textDocument/codeAction", json!({"uri": 42})),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the request still answers (never a hang)");
    };
    assert_eq!(response.result, Some(serde_json::Value::Null));
}

// ---------------------------------------------------------------------------
// The documentHighlight wire (the click-to-find occurrences).
// ---------------------------------------------------------------------------

/// The documentHighlight request the harness reuses.
fn highlight_request(id: i32, uri: &str, line: u32, character: u32) -> Message {
    request(
        id,
        "textDocument/documentHighlight",
        json!({
            "textDocument": {"uri": uri},
            "position": {"line": line, "character": character},
        }),
    )
}

#[test]
fn document_highlight_answers_the_occurrences() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let _ = handle_message(&mut session, did_open(&uri(), 1, ACTIONS_DOC));
    let handled = handle_message(&mut session, highlight_request(30, &uri(), 1, 15));
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the request answers");
    };
    let result = response.result.expect("the highlight list");
    let highlights = result.as_array().expect("an array of highlights");
    assert_eq!(
        highlights.len(),
        1,
        "the fixture's single occurrence: {result}"
    );
    assert_eq!(highlights[0]["range"]["start"]["line"], 1);
    // The occurrence's own start (the class's first character), not the
    // cursor's column: the edit/highlight range is the finding's.
    assert_eq!(highlights[0]["range"]["start"]["character"], 13);
}

#[test]
fn document_highlight_outside_a_candidate_answers_empty() {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let _ = handle_message(&mut session, did_open(&uri(), 1, ACTIONS_DOC));
    let handled = handle_message(&mut session, highlight_request(31, &uri(), 0, 2));
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the request answers");
    };
    assert_eq!(response.result, Some(serde_json::json!([])));
}

#[test]
fn document_highlight_before_initialize_rides_silently() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(&mut session, highlight_request(32, &uri(), 1, 15));
    assert!(
        handled.outgoing.is_empty(),
        "requests before initialize ride the quiet-drop contract"
    );
}
