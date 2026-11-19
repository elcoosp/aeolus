//! Consumer demonstration: the incremental sync — the wire's
//! `textDocument/didChange` patched through RANGED content changes
//! (`TextDocumentSyncKind::INCREMENTAL`). The engine's document map no
//! longer requires the whole text: each change splices sequentially
//! against the text the previous change left (the protocol's own
//! application order), a range-less change staying the full-text
//! replace the full-sync clients keep sending, the version tag landing
//! after the patch, and every keystroke's publish carrying the patched
//! document's own findings.
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_incremental_sync
//! ```

use aeolus_lsp::{Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

const URI: &str = "file:///card.rs";

/// The open text: one lowered string and one unknown root the
/// diagnostics publish.
const OPEN: &str = "fn card() {\n    let a = \"rounded-md not-a-root\";\n}\n";

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

fn change(version: i32, changes: serde_json::Value) -> Message {
    notification(
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": URI, "version": version},
            "contentChanges": changes,
        }),
    )
}

/// One diagnostics pull: the codes the patched document answers.
fn pull(session: &mut Session, id: i32) -> Vec<String> {
    let handled = handle_message(
        session,
        request(
            id,
            "textDocument/documentDiagnostic",
            json!({"textDocument": {"uri": URI}}),
        ),
    );
    let mut codes = Vec::new();
    for message in handled.outgoing {
        let Message::Response(response) = message else {
            continue;
        };
        for item in response.result.unwrap_or(json!(null))["items"]
            .as_array()
            .expect("the report carries items")
        {
            codes.push(item["code"].as_str().expect("the code rides").to_string());
        }
    }
    // The outline rides beside: the patched document's candidate
    // spellings, the splice's own truth.
    let outlined = handle_message(
        session,
        request(
            id + 100,
            "textDocument/documentSymbol",
            json!({"textDocument": {"uri": URI}}),
        ),
    );
    for message in outlined.outgoing {
        let Message::Response(response) = message else {
            continue;
        };
        let names: Vec<String> = response
            .result
            .unwrap_or(json!(null))
            .as_array()
            .expect("the outline rides")
            .iter()
            .map(|symbol| symbol["name"].as_str().expect("the name rides").to_string())
            .collect();
        println!("    outline: {names:?}");
    }
    codes
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
                "textDocument": {"uri": URI, "languageId": "rust", "version": 1, "text": OPEN}
            }),
        ),
    );

    println!("── the open document's findings ──");
    for code in pull(&mut session, 1) {
        println!("    {code}");
    }

    println!("── one ranged splice: `not-a-root` → `p-8` ──");
    let _ = handle_message(
        &mut session,
        change(
            2,
            json!([{
                "range": {
                    "start": {"line": 1, "character": 24},
                    "end": {"line": 1, "character": 34}
                },
                "text": "p-8"
            }]),
        ),
    );
    let codes = pull(&mut session, 3);
    println!(
        "    after the patch: {}",
        if codes.is_empty() {
            "clean".to_string()
        } else {
            format!("{codes:?}")
        }
    );

    println!("── sequential splices in ONE notification ──");
    // The FIRST edit replaces `rounded-md` (cols 13..23) with `p-12`;
    // the SECOND's coordinates ride the text the FIRST left — `p-8`
    // moved from 24..27 to 18..21, and the range keys on the CURRENT
    // text (the protocol's sequential application order).
    let _ = handle_message(
        &mut session,
        change(
            3,
            json!([
                {
                    "range": {
                        "start": {"line": 1, "character": 13},
                        "end": {"line": 1, "character": 23}
                    },
                    "text": "p-12"
                },
                {
                    "range": {
                        "start": {"line": 1, "character": 18},
                        "end": {"line": 1, "character": 21}
                    },
                    "text": "gap-2"
                }
            ]),
        ),
    );
    let codes = pull(&mut session, 4);
    println!(
        "    after both patches: {}",
        if codes.is_empty() {
            "clean".to_string()
        } else {
            format!("{codes:?}")
        }
    );

    println!("── the keystroke walk: `gap-2` retyped one splice at a time ──");
    // The editor's delete-to-retype: the tail of `gap-2` (cols 19..23)
    // goes first (an EMPTY replacement — the deletion splice), the
    // half-typed `g` the only candidate left.
    let _ = handle_message(
        &mut session,
        change(
            4,
            json!([{
                "range": {
                    "start": {"line": 1, "character": 19},
                    "end": {"line": 1, "character": 23}
                },
                "text": ""
            }]),
        ),
    );
    let codes = pull(&mut session, 5);
    println!("    mid-typing: {codes:?}");
    // The completion splice: zero-width, the remaining four characters
    // inserted at the caret.
    let _ = handle_message(
        &mut session,
        change(
            5,
            json!([{
                "range": {
                    "start": {"line": 1, "character": 19},
                    "end": {"line": 1, "character": 19}
                },
                "text": "ap-2"
            }]),
        ),
    );
    let codes = pull(&mut session, 6);
    println!(
        "    keystroke complete: {}",
        if codes.is_empty() {
            "clean".to_string()
        } else {
            format!("{codes:?}")
        }
    );

    println!("── the full-text replace rides beside the ranged shape ──");
    let _ = handle_message(
        &mut session,
        change(
            6,
            json!([{ "text": "fn card() {\n    let a = \"p-4 not-a-root\";\n}\n" }]),
        ),
    );
    let codes = pull(&mut session, 7);
    println!("    after the range-less change: {codes:?}");
}
