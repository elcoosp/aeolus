// ---------------------------------------------------------------------------
// The incremental-sync contracts (the wire's `textDocument/didChange`
// upgraded to LSP 3.17's ranged changes — `TextDocumentSyncKind::
// INCREMENTAL`): the editor sends content CHANGES with ranges — the
// keystroke's splice, the multi-cursor paste, the completion's insert —
// and the engine patches its document map instead of requiring the
// whole text. Each change applies SEQUENTIALLY against the text the
// previous change left (the protocol's own application order); a ranged
// change's positions ride the CURRENT text's UTF-16 pairs; a
// range-less change is the full-text replace (the backward-compat
// shape a full-sync client sends, still legal under the incremental
// capability). The version tag rides the notification once, landing
// after the patch. An unknown URI's current text is the EMPTY string —
// the implicit-open contract generalized (the ranged edits compose
// against it, a full-text change replaces it). `rangeLength` rides
// unused: the informational field the protocol marks deprecated beside
// the range.
//
// Written FIRST (the RED spec).
// ---------------------------------------------------------------------------

use aeolus_lsp::{Handled, Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

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

fn wire_session(text: &str) -> Session {
    let mut session = Session::new(Default::default());
    initialize(&mut session);
    let _ = handle_message(
        &mut session,
        notification(
            "textDocument/didOpen",
            json!({
                "textDocument": {"uri": URI, "languageId": "rust", "version": 1, "text": text}
            }),
        ),
    );
    session
}

fn publish_findings(handled: Handled) -> Vec<String> {
    let mut out = Vec::new();
    for message in handled.outgoing {
        let Message::Notification(notification) = message else {
            continue;
        };
        if notification.method != "textDocument/publishDiagnostics" {
            continue;
        }
        for diagnostic in notification.params["diagnostics"]
            .as_array()
            .expect("the publish carries an array")
        {
            out.push(
                diagnostic["code"]
                    .as_str()
                    .expect("the code rides")
                    .to_string(),
            );
        }
    }
    out
}

fn change(session: &mut Session, version: i32, changes: serde_json::Value) -> Handled {
    handle_message(
        session,
        notification(
            "textDocument/didChange",
            json!({
                "textDocument": {"uri": URI, "version": version},
                "contentChanges": changes,
            }),
        ),
    )
}

/// The document-symbol count the wire reports (the patched text's own
/// outline — the probe that distinguishes a real splice from the
/// whole-text replace: a spliced document keeps its quotes and its
/// classes, a wholesale replace of a fragment carries none).
fn symbol_count(session: &mut Session, id: i32) -> usize {
    let handled = handle_message(
        session,
        request(
            id,
            "textDocument/documentSymbol",
            json!({"textDocument": {"uri": URI}}),
        ),
    );
    for message in handled.outgoing {
        let Message::Response(response) = message else {
            continue;
        };
        return response
            .result
            .expect("the outline answers")
            .as_array()
            .map(Vec::len)
            .unwrap_or(0);
    }
    panic!("the outline request answers");
}

#[test]
fn initialize_advertises_incremental_sync() {
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
        result["capabilities"]["textDocumentSync"]["change"],
        json!(2),
        "the sync kind upgrades to INCREMENTAL (2)"
    );
}

#[test]
fn a_ranged_change_patches_the_document() {
    // The unknown class `not-a-root` replaced by the lowered `p-8`: the
    // publish after the patch carries the patched document's findings —
    // the refusal gone, the splice real.
    let open_text = "fn card() {\n    let a = \"not-a-root\";\n}\n";
    let mut session = wire_session(open_text);
    let handled = change(
        &mut session,
        2,
        json!([{
            "range": {
                "start": {"line": 1, "character": 13},
                "end": {"line": 1, "character": 23}
            },
            "text": "p-8"
        }]),
    );
    let findings = publish_findings(handled);
    assert!(
        findings.is_empty(),
        "the patched document lowers clean: {findings:?}"
    );
    // The splice is real: the patched text keeps its quotes and its
    // class (one outline symbol), never the whole-text replace the
    // fragment-only payload would spell.
    assert_eq!(
        symbol_count(&mut session, 1),
        1,
        "the patched document carries the spliced class"
    );
}

#[test]
fn sequential_ranged_changes_compose() {
    // Two changes in ONE notification: the first renames the class, the
    // second re-spans over the result of the first (the protocol's own
    // application order — each change rides the text the previous left).
    // `flex-col gap-2` → `flex-row gap-2` → `flex-row gap-4`: the second
    // edit's range keys on the FIRST patch's column arithmetic (the
    // `gap-2` position is unchanged, but a range past the first edit
    // would still be defined against the CURRENT text).
    let open_text = "fn card() {\n    let a = \"flex-col gap-2\";\n}\n";
    let mut session = wire_session(open_text);
    let handled = change(
        &mut session,
        2,
        json!([
            {
                "range": {
                    "start": {"line": 1, "character": 13},
                    "end": {"line": 1, "character": 21}
                },
                "text": "flex-row"
            },
            {
                "range": {
                    "start": {"line": 1, "character": 22},
                    "end": {"line": 1, "character": 27}
                },
                "text": "gap-4"
            }
        ]),
    );
    let findings = publish_findings(handled);
    assert!(
        findings.is_empty(),
        "both patches compose onto the lowered string: {findings:?}"
    );
    // Both patches are real: the composed text carries both classes
    // (two outline symbols), never the last fragment alone.
    assert_eq!(
        symbol_count(&mut session, 1),
        2,
        "the composed document carries both spliced classes"
    );
}

#[test]
fn a_full_text_change_still_replaces() {
    // The backward-compat shape: a range-less change is the full-text
    // replace — a full-sync client's payload stays legal under the
    // incremental capability.
    let open_text = "fn card() {\n    let a = \"p-4\";\n}\n";
    let mut session = wire_session(open_text);
    let full = "fn card() {\n    let a = \"rounded-md not-a-root\";\n}\n";
    let handled = change(&mut session, 2, json!([{ "text": full }]));
    let findings = publish_findings(handled);
    assert_eq!(
        findings,
        vec!["aeolus::compile::unknown_root".to_string()],
        "the full text replaced the map wholesale — the refusal rides"
    );
}

#[test]
fn ranged_and_full_text_changes_compose_in_order() {
    // A ranged splice then a full-text replace: the full change wins the
    // final text (it replaces whatever the splice left) — the protocol's
    // sequential application over the mixed shapes.
    let open_text = "fn card() {\n    let a = \"p-4\";\n}\n";
    let mut session = wire_session(open_text);
    let handled = change(
        &mut session,
        2,
        json!([
            {
                "range": {
                    "start": {"line": 1, "character": 13},
                    "end": {"line": 1, "character": 16}
                },
                "text": "p-8"
            },
            { "text": "fn card() {\n    let a = \"not-a-root\";\n}\n" }
        ]),
    );
    let findings = publish_findings(handled);
    assert_eq!(
        findings,
        vec!["aeolus::compile::unknown_root".to_string()],
        "the trailing full-text change names the final text"
    );
}

#[test]
fn utf16_ranges_patch_multibyte_text_byte_true() {
    // The columns are UTF-16 code units: the class sits after a wide
    // character (€ = one UTF-16 unit, three UTF-8 bytes) — the splice's
    // byte offsets derive through the engine's own conversion, the
    // patched text byte-true.
    let open_text = "fn card() {\n    let label = \"€ not-a-root\";\n}\n";
    let mut session = wire_session(open_text);
    let handled = change(
        &mut session,
        2,
        json!([{
            "range": {
                "start": {"line": 1, "character": 17},
                "end": {"line": 1, "character": 29}
            },
            "text": "p-8"
        }]),
    );
    let findings = publish_findings(handled);
    assert!(
        findings.is_empty(),
        "the wide character's UTF-16 column patched byte-true: {findings:?}"
    );
}

#[test]
fn the_version_tag_lands_after_the_patch() {
    let open_text = "fn card() {\n    let a = \"p-4\";\n}\n";
    let mut session = wire_session(open_text);
    let handled = change(
        &mut session,
        7,
        json!([{
            "range": {
                "start": {"line": 1, "character": 13},
                "end": {"line": 1, "character": 16}
            },
            "text": "p-8"
        }]),
    );
    for message in &handled.outgoing {
        let Message::Notification(notification) = message else {
            continue;
        };
        if notification.method == "textDocument/publishDiagnostics" {
            assert_eq!(
                notification.params["version"],
                json!(7),
                "the notification's version rides the didChange's tag"
            );
        }
    }
}

#[test]
fn an_unknown_uri_patches_the_empty_base() {
    // A didChange for a URI never opened: the implicit-open contract
    // generalized — the current text is EMPTY, the ranged splice inserts
    // into it, a full-text change replaces it. The publish answers the
    // patched text's own findings, never a panic.
    let mut session = wire_session("fn card() {\n    let a = \"p-4\";\n}\n");
    let handled = handle_message(
        &mut session,
        notification(
            "textDocument/didChange",
            json!({
                "textDocument": {"uri": "file:///never-opened.rs", "version": 1},
                "contentChanges": [{
                    "range": {
                        "start": {"line": 0, "character": 0},
                        "end": {"line": 0, "character": 0}
                    },
                    "text": "let s = \"not-a-root\";"
                }],
            }),
        ),
    );
    let findings = publish_findings(handled);
    assert_eq!(
        findings,
        vec!["aeolus::compile::unknown_root".to_string()],
        "the empty base took the splice — the patched text's findings ride"
    );
}

#[test]
fn the_patched_document_answers_the_full_open_parity() {
    // The parity pin: a document assembled by ranged splices answers the
    // diagnostics of the SAME text opened directly, byte for byte —
    // over NON-empty findings (the refusal the spliced text carries), a
    // parity an empty-vs-empty answer could fake.
    let final_text = "fn card() {\n    let a = \"rounded-md not-a-root p-8\";\n}\n";
    let mut session = wire_session("fn card() {\n    let a = \"rounded-md\";\n}\n");
    let _ = change(
        &mut session,
        2,
        json!([{
            "range": {
                "start": {"line": 1, "character": 23},
                "end": {"line": 1, "character": 23}
            },
            "text": " not-a-root p-8"
        }]),
    );
    let mut codes = Vec::new();
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/documentDiagnostic",
            json!({
                "textDocument": {"uri": URI}
            }),
        ),
    );
    for message in handled.outgoing {
        let Message::Response(response) = message else {
            continue;
        };
        for item in response.result.expect("the pull answers")["items"]
            .as_array()
            .expect("the report carries items")
        {
            codes.push(item["code"].as_str().expect("the code rides").to_string());
        }
    }
    let mut engine = aeolus_lsp::AeolusEngine::new(aeolus_lsp::EngineConfig::default());
    engine.open(URI, final_text, 9);
    let engine_codes: Vec<String> = engine
        .analyze(URI)
        .iter()
        .map(|finding| finding.code.clone())
        .collect();
    assert!(
        !codes.is_empty(),
        "the spliced document carries findings — the parity is real"
    );
    assert_eq!(
        codes, engine_codes,
        "the spliced document answers the direct open's findings"
    );
}

#[test]
fn the_insertion_at_the_typing_edge_patches() {
    // The keystroke walk: `p-` typed one character at a time at the
    // string's end — three ranged insertions composing to `p-4 fl`, the
    // patched text's diagnostics riding each publish (the engine's
    // honesty contract at the editor, now keystroke-shaped).
    let open_text = "fn card() {\n    let a = \"p-4\";\n}\n";
    let mut session = wire_session(open_text);
    let steps = [("f", 16u32), ("l", 17u32), (" ", 18u32)];
    for ((text, column), version) in steps.into_iter().zip(2..) {
        let handled = change(
            &mut session,
            version,
            json!([{
                "range": {
                    "start": {"line": 1, "character": column},
                    "end": {"line": 1, "character": column}
                },
                "text": text
            }]),
        );
        assert!(
            !handled.outgoing.is_empty(),
            "every keystroke's patch publishes the patched findings"
        );
    }
    // The final text holds `p-4 fl ` — the pull answers its findings
    // (an unknown-root refusal for the half-typed `fl`, the honest
    // boundary the compile keeps).
    let handled = handle_message(
        &mut session,
        request(
            9,
            "textDocument/documentDiagnostic",
            json!({
                "textDocument": {"uri": URI}
            }),
        ),
    );
    let mut saw_refusal = false;
    for message in handled.outgoing {
        let Message::Response(response) = message else {
            continue;
        };
        for item in response.result.expect("the pull answers")["items"]
            .as_array()
            .expect("the report carries items")
        {
            if item["code"] == json!("aeolus::compile::shape_rejected") {
                saw_refusal = true;
            }
        }
    }
    assert!(
        saw_refusal,
        "the keystrokes composed onto the patched text — the half-typed root refuses honestly"
    );
}
