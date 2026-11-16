//! Consumer demonstration: the LSP engine's canonical-order formatting
//! (the editor's `textDocument/formatting` — the class-sort seam's
//! editor face, the tenth answer beside diagnostics-push, hover,
//! completion, the colour swatches, the diagnostic pull, the semantic
//! tokens, the colour presentations, the inlay hints and the quick
//! fixes) driven headless. Every edit replaces one class string's
//! content with the registry's canonical order — the sort seam's own
//! join — and an already-canonical string answers nothing: the no-op
//! contract a format-on-save rides. The stdio server itself is `cargo
//! run -p aeolus-lsp --bin aeolus-lsp` (the transport's thin shell over
//! the same engine).
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_format
//! ```

use aeolus_lsp::{Session, handle_message};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// A synthetic .rs document the way `tw!` call sites look: one
/// out-of-order string (the variant bucket rides after the base in the
/// canonical order), one already-sorted string, one whitespace run the
/// canonical join normalizes.
const DOC: &str = "fn card() {\n    let a = \"hover:flex-col p-4 flex-col\";\n    let b = \"p-4 flex-col\";\n    let c = \"gap-2  p-4\";\n}\n";

fn main() {
    // The state machine is the server's whole behaviour, testable and
    // demonstrable without a stdio pipe: initialize, didOpen, then the
    // formatting request — the message sequence a client drives on
    // format-on-save.
    let mut session = Session::new(Default::default());
    let _ = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(0),
            method: "initialize".to_string(),
            params: json!({"capabilities": {}}),
        }),
    );
    let _ = handle_message(
        &mut session,
        Message::Notification(Notification {
            method: "textDocument/didOpen".to_string(),
            params: json!({
                "textDocument": {
                    "uri": "file:///card.rs",
                    "languageId": "rust",
                    "version": 1,
                    "text": DOC,
                }
            }),
        }),
    );

    // The formatting request: the canonical-order edits over the
    // document's class strings.
    let formatting = handle_message(
        &mut session,
        Message::Request(Request {
            id: RequestId::from(1),
            method: "textDocument/formatting".to_string(),
            params: json!({
                "textDocument": {"uri": "file:///card.rs"},
                "options": {"tabSize": 4, "insertSpaces": true},
            }),
        }),
    );

    // Collect the edits first (the print walk and the apply walk both
    // read them), then print.
    let edits: Vec<(u64, u64, u64, u64, String)> = formatting
        .outgoing
        .iter()
        .filter_map(|message| match message {
            Message::Response(response) => response.result.as_ref().and_then(|value| {
                value.as_array().map(|items| {
                    items
                        .iter()
                        .map(|edit| {
                            (
                                edit["range"]["start"]["line"].as_u64().unwrap_or(0),
                                edit["range"]["start"]["character"].as_u64().unwrap_or(0),
                                edit["range"]["end"]["line"].as_u64().unwrap_or(0),
                                edit["range"]["end"]["character"].as_u64().unwrap_or(0),
                                edit["newText"].as_str().unwrap_or_default().to_string(),
                            )
                        })
                        .collect::<Vec<_>>()
                })
            }),
            _ => None,
        })
        .flatten()
        .collect();

    println!("== the canonical-order edits ==");
    for message in formatting.outgoing {
        if let Message::Response(response) = message {
            println!(
                "{}",
                serde_json::to_string_pretty(&response.result).expect("the edits serialize")
            );
        }
    }

    // The edits applied: the second string answered no edit (the no-op
    // contract — the edit list IS the diff), the whitespace run
    // normalized beside the reordered variant bucket.
    let mut formatted = DOC.to_string();
    // Apply back to front within the document (the edits ride in
    // document order; the demo applies each over the line/character
    // model the engine answers).
    for (start_line, start_char, end_line, end_char, new_text) in edits.iter().rev() {
        let start = line_character_offset(&formatted, *start_line, *start_char);
        let end = line_character_offset(&formatted, *end_line, *end_char);
        if let (Some(start), Some(end)) = (start, end) {
            formatted.replace_range(start..end, new_text);
        }
    }
    println!("== the document after the edits ==");
    print!("{formatted}");
}

/// The `(line, character-UTF16)` position's byte offset (the demo's
/// own inversion of the engine's model — the engine owns the
/// conversion, an editor host rides its own).
fn line_character_offset(text: &str, line: u64, character: u64) -> Option<usize> {
    let mut current_line = 0u64;
    let mut line_start = 0usize;
    while current_line < line {
        let relative = text[line_start..].find('\n')?;
        line_start += relative + 1;
        current_line += 1;
    }
    let mut units = 0u64;
    for (index, ch) in text[line_start..].char_indices() {
        if units >= character || ch == '\n' {
            return Some(line_start + index);
        }
        units += u64::from(ch.len_utf16() as u16);
    }
    Some(text.len())
}
