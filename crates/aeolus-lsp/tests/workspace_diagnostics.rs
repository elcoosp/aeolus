// ---------------------------------------------------------------------------
// The workspace-pull diagnostic contracts (LSP 3.17's `workspace/diagnostic`
// — the pull channel generalized over the whole document map): every OPEN
// document answers beside its result identity — the fingerprint the client
// caches against — and a matching previous id answers the unchanged
// verdict, the client's cache the report rides from (the retheme
// fast-path's equality shape, at the pull channel).
//
// Written FIRST (the RED spec).
// ---------------------------------------------------------------------------

use aeolus_lsp::{AeolusEngine, EngineConfig, Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// The battery's document: one lowered string, one refused candidate —
/// the findings the full reports carry.
const DOC_A: &str = "fn card() {\n    let a = \"p-4 flex-col not-a-root\";\n}\n";

/// The second document: the SAME text (the content-keyed identity's
/// controlled input) opened under a different URI.
const DOC_B: &str = DOC_A;

const URI_A: &str = "file:///card-a.rs";
const URI_B: &str = "file:///card-b.rs";

fn engine() -> AeolusEngine {
    let mut lsp = AeolusEngine::new(EngineConfig::new(None));
    lsp.open(URI_A, DOC_A, 1);
    lsp.open(URI_B, DOC_B, 2);
    lsp
}

// ---------------------------------------------------------------------------
// The engine face
// ---------------------------------------------------------------------------

#[test]
fn the_pull_answers_every_open_document_in_uri_order() {
    let lsp = engine();
    let reports = lsp.workspace_diagnostics(&[]);

    assert_eq!(
        reports.len(),
        2,
        "every open document answers, whatever the open order was"
    );
    assert_eq!(reports[0].uri, URI_A, "URI order across the map");
    assert_eq!(reports[1].uri, URI_B);
}

#[test]
fn the_full_report_carries_the_findings_and_the_identity() {
    let lsp = engine();
    let reports = lsp.workspace_diagnostics(&[]);

    let outcome = &reports[0].outcome;
    let aeolus_lsp::EngineWorkspaceOutcome::Full { diagnostics } = outcome else {
        panic!("no previous ids: every document answers the full report");
    };
    assert_eq!(
        diagnostics.len(),
        1,
        "the refused candidate's finding rides the report"
    );
    assert!(
        !reports[0].result_id.is_empty(),
        "the result identity rides every report"
    );
    assert_eq!(
        reports[0].version, 1,
        "the document's own version tag rides"
    );
}

#[test]
fn the_result_identity_is_content_keyed() {
    // Same text, same version → the same identity, whatever the URI is
    // (the fingerprint is the DOCUMENT's, not the session's).
    let lsp = engine();
    let reports = lsp.workspace_diagnostics(&[]);
    assert_eq!(
        reports[0].result_id, reports[1].result_id,
        "identical text at the same version answers the same identity"
    );
}

#[test]
fn a_change_answers_a_new_identity() {
    let mut lsp = engine();
    let before = lsp.workspace_diagnostics(&[])[0].result_id.clone();
    lsp.change(
        URI_A,
        "fn card() {\n    let a = \"p-4 not-a-root\";\n}\n",
        5,
    );
    let after = lsp.workspace_diagnostics(&[])[0].result_id.clone();
    assert_ne!(
        before, after,
        "a didChange with new text answers a new identity"
    );
}

#[test]
fn an_equal_content_change_keeps_the_identity() {
    let mut lsp = engine();
    let before = lsp.workspace_diagnostics(&[])[0].result_id.clone();
    // The equal-content re-save: the version bumps, the text does not —
    // the findings are pure over the text, so the identity holds (the
    // byte-neutral no-op the retheme fast-path keeps, at the pull).
    lsp.change(URI_A, DOC_A, 5);
    let after = lsp.workspace_diagnostics(&[])[0].result_id.clone();
    assert_eq!(before, after, "the content is the identity, not the tag");
}

#[test]
fn a_matching_previous_id_answers_unchanged() {
    let lsp = engine();
    let id = lsp.workspace_diagnostics(&[])[0].result_id.clone();
    let reports = lsp.workspace_diagnostics(&[(URI_A, id.as_str())]);

    assert_eq!(reports.len(), 2, "the scope stays the whole map");
    assert_eq!(
        reports[0].outcome,
        aeolus_lsp::EngineWorkspaceOutcome::Unchanged,
        "the client's cache is the report — nothing recomputes"
    );
    assert_eq!(
        reports[0].result_id, id,
        "the identity echoes: the client keys its cache on it"
    );
    assert!(
        matches!(
            reports[1].outcome,
            aeolus_lsp::EngineWorkspaceOutcome::Full { .. }
        ),
        "a uri the previous list never named answers full"
    );
}

#[test]
fn a_stale_previous_id_answers_full() {
    let mut lsp = engine();
    let stale = lsp.workspace_diagnostics(&[])[0].result_id.clone();
    // The text moves: the identity rides the content, so the cached id
    // goes stale and the report recomputes.
    lsp.change(
        URI_A,
        "fn card() {\n    let a = \"p-4 not-a-root\";\n}\n",
        3,
    );
    let reports = lsp.workspace_diagnostics(&[(URI_A, stale.as_str())]);

    assert!(
        matches!(
            reports[0].outcome,
            aeolus_lsp::EngineWorkspaceOutcome::Full { .. }
        ),
        "the changed document's identity moved — the report recomputes"
    );
}

#[test]
fn unopened_uris_in_the_previous_list_ride_unanswered() {
    let lsp = engine();
    let reports = lsp.workspace_diagnostics(&[("file:///gone.rs", "v1:deadbeef")]);

    assert_eq!(
        reports.len(),
        2,
        "the map is the answer's scope: only open documents answer"
    );
}

#[test]
fn the_answer_is_pure_over_the_document_map() {
    let lsp = engine();
    let id = lsp.workspace_diagnostics(&[])[1].result_id.clone();
    let first = lsp.workspace_diagnostics(&[(URI_B, id.as_str())]);
    let second = lsp.workspace_diagnostics(&[(URI_B, id.as_str())]);
    assert_eq!(first, second, "the same input answers the same output");
}

// ---------------------------------------------------------------------------
// The wire face
// ---------------------------------------------------------------------------

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

fn did_open(uri: &str, text: &str) -> Message {
    notification(
        "textDocument/didOpen",
        json!({
            "textDocument": {"uri": uri, "languageId": "rust", "version": 1, "text": text}
        }),
    )
}

fn handshake(session: &mut Session) {
    let handled = handle_message(
        session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    assert!(!handled.outgoing.is_empty(), "initialize answers");
}

fn pull(session: &mut Session, id: i32, previous: serde_json::Value) -> serde_json::Value {
    let handled = handle_message(session, request(id, "workspace/diagnostic", previous));
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the workspace pull answers");
    };
    response.result.expect("the pull answer carries a result")
}

#[test]
fn wire_workspace_pull_answers_the_items() {
    let mut session = Session::new(Default::default());
    handshake(&mut session);
    let _ = handle_message(&mut session, did_open(URI_A, DOC_A));
    let _ = handle_message(&mut session, did_open(URI_B, DOC_B));

    let result = pull(&mut session, 1, json!({"previousResultIds": []}));
    let items = result["items"].as_array().expect("the items ride");
    assert_eq!(items.len(), 2, "every open document rides one item");
    assert_eq!(items[0]["uri"], URI_A);
    assert_eq!(items[0]["kind"], "full", "no previous ids: full reports");
    assert!(
        items[0].get("resultId").is_some(),
        "the identity rides the item"
    );
    let findings = items[0]["items"].as_array().expect("the findings ride");
    assert_eq!(findings.len(), 1, "the refused candidate's finding rides");
}

#[test]
fn wire_the_unchanged_item_answers_without_findings() {
    let mut session = Session::new(Default::default());
    handshake(&mut session);
    let _ = handle_message(&mut session, did_open(URI_A, DOC_A));

    // First pull: the full report names the identity.
    let first = pull(&mut session, 1, json!({"previousResultIds": []}));
    let id = first["items"][0]["resultId"]
        .as_str()
        .expect("the identity rides")
        .to_string();

    // Second pull with that identity: the item answers unchanged — no
    // findings ride (the client's cache is the report).
    let second = pull(
        &mut session,
        2,
        json!({"previousResultIds": [{"uri": URI_A, "value": id}]}),
    );
    let items = second["items"].as_array().expect("the items ride");
    assert_eq!(items[0]["kind"], "unchanged");
    assert_eq!(items[0]["resultId"], id, "the client's value echoed");
    assert!(
        items[0].get("items").is_none(),
        "an unchanged item carries no findings"
    );
}

#[test]
fn wire_initialize_advertises_the_workspace_pull() {
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
        result["capabilities"]["diagnosticProvider"]["workspaceDiagnostics"], true,
        "the pull channel's workspace half is advertised beside the document pull"
    );
}

#[test]
fn wire_pre_handshake_pull_quiet_drops() {
    let mut session = Session::new(Default::default());
    let handled = handle_message(
        &mut session,
        request(1, "workspace/diagnostic", json!({"previousResultIds": []})),
    );
    assert!(
        handled.outgoing.is_empty(),
        "the pre-handshake gate rides before any answer"
    );
}

#[test]
fn wire_malformed_params_answer_null() {
    let mut session = Session::new(Default::default());
    handshake(&mut session);
    let handled = handle_message(&mut session, request(1, "workspace/diagnostic", json!({})));
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the malformed request still answers");
    };
    assert_eq!(
        response.result,
        Some(serde_json::Value::Null),
        "missing previousResultIds is malformed: the protocol's own empty answer"
    );
}
