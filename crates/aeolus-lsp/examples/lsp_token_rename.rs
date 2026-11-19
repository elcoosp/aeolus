//! Consumer demonstration: the token-rename fold — the command surface's
//! second command. The wire's `workspace/executeCommand` answered
//! `aeolus.sortDocument` first; `aeolus.renameToken` beside it renames a
//! THEME TOKEN across the whole workspace: the cursor's candidate
//! decomposes onto a colour-group root whose remainder the riding table
//! carries (the definition face's own walk), and every candidate over
//! EVERY open document whose decomposition names the SAME token rides
//! one edit over the token's own span — `bg-primary` and
//! `dark:text-primary` are different spellings and the same token, so
//! one rename takes them together, the variant prefixes and `/opacity`
//! modifier tails preserved. An unknown command and a mistyped argument
//! answer the protocol's own null; a non-token cursor answers the empty
//! fold.
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_token_rename
//! ```

use aeolus_lsp::{Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// Two open documents, four usages of one token across three roots and
/// two variant prefixes — the fold takes them together.
const DOC_A: &str = concat!(
    "fn card() {\n",
    "    let a = \"hover:bg-primary/90 p-4\";\n",
    "    let b = \"border-primary\";\n",
    "}\n",
);
const DOC_B: &str = concat!(
    "fn panel() {\n",
    "    let c = \"text-primary dark:text-primary\";\n",
    "    let d = \"flex-col items-center\";\n",
    "}\n",
);

const URI_A: &str = "file:///card.rs";
const URI_B: &str = "file:///panel.rs";

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
    for (uri, doc) in [(URI_A, DOC_A), (URI_B, DOC_B)] {
        let _ = handle_message(
            &mut session,
            notification(
                "textDocument/didOpen",
                json!({
                    "textDocument": {"uri": uri, "languageId": "rust", "version": 1, "text": doc}
                }),
            ),
        );
    }

    println!("── the token rename, cursor inside `bg-primary` ──");
    let result = run(
        &mut session,
        1,
        json!({
            "command": "aeolus.renameToken",
            "arguments": [URI_A, 1, 22, "brand"]
        }),
    );
    let changes = result["changes"].as_object().expect("the changes map");
    for (uri, edits) in changes {
        for edit in edits.as_array().expect("an array of edits") {
            println!(
                "    {uri} @ line {} → {}",
                edit["range"]["start"]["line"], edit["newText"]
            );
        }
    }

    println!("── what the edits splice into ──");
    let renamed_a = splice(DOC_A, URI_A, &result);
    let renamed_b = splice(DOC_B, URI_B, &result);
    println!("    {renamed_a}");
    println!("    {renamed_b}");

    println!("── the honest misses ──");
    let non_token = run(
        &mut session,
        2,
        json!({
            "command": "aeolus.renameToken",
            "arguments": [URI_A, 1, 33, "brand"] // the cursor rests on `p-4`
        }),
    );
    println!(
        "    non-token cursor → {} edit(s)",
        non_token["changes"]
            .as_object()
            .map(|m| m
                .values()
                .map(|v| v.as_array().map_or(0, Vec::len))
                .sum::<usize>())
            .unwrap_or(0)
    );
    let unknown = run(
        &mut session,
        3,
        json!({
            "command": "aeolus.notACommand",
            "arguments": [URI_A, 1, 22, "brand"]
        }),
    );
    println!("    unknown command → {unknown}");
    let mistyped = run(
        &mut session,
        4,
        json!({
            "command": "aeolus.renameToken",
            "arguments": [URI_A, "1", 22, "brand"] // the line rides a string
        }),
    );
    println!("    mistyped argument → {mistyped}");
}

/// One wire position onto the document's bytes (the example's own
/// read-out arithmetic — the engine's `offset_of` shape over u64 JSON
/// numbers).
fn offset_of(doc: &str, line: u64, character: u64) -> usize {
    let mut current = 0u64;
    let mut start = 0usize;
    while current < line {
        match doc[start..].find('\n') {
            Some(relative) => start += relative + 1,
            None => return doc.len(),
        }
        current += 1;
    }
    let mut units = 0u64;
    for (index, ch) in doc[start..].char_indices() {
        if doc[start + index..].starts_with('\n') || units >= character {
            return start + index;
        }
        units += u64::try_from(ch.len_utf16()).unwrap_or(0);
    }
    doc.len()
}

/// Apply the answer's edits to one document: the renamed bytes the fold
/// splices into the source (back-to-front, the ranges stay byte-true).
fn splice(doc: &str, uri: &str, result: &serde_json::Value) -> String {
    let Some(edits) = result["changes"][uri].as_array() else {
        return doc.to_string();
    };
    let mut out = doc.to_string();
    for edit in edits.iter().rev() {
        let start = offset_of(
            &out,
            edit["range"]["start"]["line"].as_u64().unwrap_or(0),
            edit["range"]["start"]["character"].as_u64().unwrap_or(0),
        );
        let end = offset_of(
            &out,
            edit["range"]["end"]["line"].as_u64().unwrap_or(0),
            edit["range"]["end"]["character"].as_u64().unwrap_or(0),
        );
        out.replace_range(start..end, edit["newText"].as_str().unwrap_or_default());
    }
    out
}
