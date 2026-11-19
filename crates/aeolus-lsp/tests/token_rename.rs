// ---------------------------------------------------------------------------
// The token-rename contracts (the command surface's second command): the
// `aeolus.renameToken` command renames a THEME TOKEN across the whole
// workspace — not a class spelling, the identity the token itself. The
// cursor's candidate decomposes onto a colour-group root whose remainder
// the riding table carries (the definition face's own walk, factored once),
// and every candidate over EVERY open document whose decomposition names
// the SAME token rides one edit over the token's own span — the variant
// prefixes and `/opacity` modifier tails preserved, the root untouched.
// The table-gated identity is the boundary the textual `rename` face does
// not have: `bg-primary` and `dark:text-primary` are different spellings
// and the same token, so one rename takes them together. Every miss is
// honest: a non-token class, a cursor outside any candidate, a theme-less
// config, an empty new name — all answer the empty fold, never a guessed
// edit; the malformed arguments answer the protocol's own null (the state
// machine's quiet vocabulary).
//
// Written FIRST (the RED spec).
// ---------------------------------------------------------------------------

use aeolus_lsp::{AeolusEngine, EngineConfig, Session, handle_message};
use aeolus_theme::{ThemeSource, TokenTable};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// The theme fixture the battery's table bakes: one prefixed key (the
/// `--color-primary` form) — the token the fold renames.
const THEME_CSS: &str = concat!(
    "@theme {\n",
    "  --color-primary: oklch(0.205 0 0);\n",
    "}\n",
    ":root {\n",
    "  --muted: oklch(0.97 0 0);\n",
    "}\n",
);

/// The cursor document: a variant-prefixed token class with an opacity
/// modifier (the prefix and the tail the fold must preserve), a
/// second-root usage of the SAME token, a non-token class the fold must
/// skip.
const DOC_A: &str = concat!(
    "fn card() {\n",
    "    let a = \"hover:bg-primary/90 p-4\";\n",
    "    let b = \"border-primary\";\n",
    "}\n",
);

/// The sibling document: two more usages of the token — one bare, one
/// riding the `dark:` variant — and a class-free string the fold skips.
const DOC_B: &str = concat!(
    "fn panel() {\n",
    "    let c = \"text-primary dark:text-primary\";\n",
    "    let d = \"flex-col items-center\";\n",
    "}\n",
);

const URI_A: &str = "file:///card.rs";
const URI_B: &str = "file:///panel.rs";

const NEW_NAME: &str = "brand";

fn fixture_table() -> TokenTable {
    let source = ThemeSource::from_css(THEME_CSS, None).expect("the fixture parses");
    TokenTable::baked(&source).expect("the bake succeeds")
}

fn cursor_of(doc: &str, needle: &str, into: usize) -> (u32, u32) {
    let offset = doc.find(needle).expect("the fixture holds") + into;
    let line = doc[..offset].matches('\n').count() as u32;
    let line_start = doc[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = doc[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    (line, character)
}

/// The range one edit rides: `(line, character-UTF16)` pairs over the
/// document's own bytes.
fn range_of(doc: &str, needle: &str) -> (u32, u32, u32, u32) {
    let offset = doc.find(needle).expect("the fixture holds");
    let cursor = cursor_of(doc, needle, 0);
    let end = offset + needle.len();
    let end_line = doc[..end].matches('\n').count() as u32;
    let end_line_start = doc[..end].rfind('\n').map_or(0, |i| i + 1);
    let end_character = doc[end_line_start..end]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    (cursor.0, cursor.1, end_line, end_character)
}

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
    let mut session = Session::new(EngineConfig::new(Some(fixture_table())));
    initialize(&mut session);
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
    session
}

/// Apply the engine's own edits to the document text: the renamed bytes
/// the fold's edits splice into the source. The edits ride in document
/// order, so the walk applies them back-to-front.
fn apply_edits(doc: &str, edits: &[aeolus_lsp::EngineRenameEdit]) -> String {
    #[derive(Clone, Copy)]
    struct Pos {
        line: u32,
        character: u32,
    }
    fn offset_of(doc: &str, pos: Pos) -> usize {
        let mut line = 0u32;
        let mut start = 0usize;
        while line < pos.line {
            let relative = doc[start..].find('\n').expect("the fixture holds");
            start += relative + 1;
            line += 1;
        }
        let mut units = 0u32;
        for (index, ch) in doc[start..].char_indices() {
            if doc[start + index..].starts_with('\n') || units >= pos.character {
                return start + index;
            }
            units += ch.len_utf16() as u32;
        }
        doc.len()
    }
    let mut out = doc.to_string();
    for edit in edits.iter().rev() {
        let start = offset_of(
            &out,
            Pos {
                line: edit.range.start.line,
                character: edit.range.start.character,
            },
        );
        let end = offset_of(
            &out,
            Pos {
                line: edit.range.end.line,
                character: edit.range.end.character,
            },
        );
        out.replace_range(start..end, &edit.new_text);
    }
    out
}

#[test]
fn initialize_advertises_the_rename_token_command() {
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
    let expected = vec![json!("aeolus.sortDocument"), json!("aeolus.renameToken")];
    assert_eq!(
        commands, &expected,
        "the table stays closed: the sort face and the token-rename face"
    );
}

#[test]
fn the_command_answers_the_workspace_edit_map() {
    let mut session = wire_session();
    let (line, character) = cursor_of(DOC_A, "primary", 0);
    let handled = handle_message(
        &mut session,
        request(
            1,
            "workspace/executeCommand",
            json!({
                "command": "aeolus.renameToken",
                "arguments": [URI_A, line, character, NEW_NAME]
            }),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the command answers");
    };
    let result = response
        .result
        .expect("a token cursor answers the edit map");
    let changes = result["changes"]
        .as_object()
        .expect("the rename face's own changes map");
    // One key per carrying document, URI order across the map.
    let keys: Vec<&String> = changes.keys().collect();
    assert_eq!(keys.len(), 2, "both token-carrying documents answer");
    assert_eq!(keys[0], URI_A, "URI order across the map");
    assert_eq!(keys[1], URI_B);
    assert_eq!(
        changes[URI_A].as_array().map(Vec::len),
        Some(2),
        "the cursor document carries its two token usages"
    );
    assert_eq!(
        changes[URI_B].as_array().map(Vec::len),
        Some(2),
        "the sibling document carries its two token usages"
    );
    for list in changes.values() {
        for edit in list.as_array().expect("an array of edits") {
            assert_eq!(
                edit["newText"], NEW_NAME,
                "every edit renames the token's own span, nothing else"
            );
        }
    }
}

#[test]
fn the_edits_rename_the_token_span_only() {
    // The fold's identity test: the prefix, the root and the modifier
    // tail all survive — only the token's own bytes ride the edits.
    let mut lsp = AeolusEngine::new(EngineConfig::new(Some(fixture_table())));
    lsp.open(URI_A, DOC_A, 1);
    lsp.open(URI_B, DOC_B, 1);
    let (line, character) = cursor_of(DOC_A, "primary", 0);
    let edits = lsp.rename_token(URI_A, line, character, NEW_NAME);
    assert_eq!(edits.len(), 4, "one edit per token usage across the map");
    let renamed_a = apply_edits(DOC_A, &edits[..2]);
    assert!(
        renamed_a.contains("\"hover:bg-brand/90 p-4\""),
        "the variant prefix and the /opacity tail survive: {renamed_a}"
    );
    assert!(
        renamed_a.contains("\"border-brand\""),
        "the second root rides the same token: {renamed_a}"
    );
    let renamed_b = apply_edits(DOC_B, &edits[2..]);
    assert!(
        renamed_b.contains("\"text-brand dark:text-brand\""),
        "the sibling document's bare and variant usages ride together: {renamed_b}"
    );
}

#[test]
fn the_wire_rides_the_engine_s_own_answer() {
    // The command's map IS the engine's fold, byte for byte — the
    // sort command's own parity contract (no parallel edit shape at
    // the wire).
    let mut session = wire_session();
    let (line, character) = cursor_of(DOC_A, "primary", 0);
    let handled = handle_message(
        &mut session,
        request(
            1,
            "workspace/executeCommand",
            json!({
                "command": "aeolus.renameToken",
                "arguments": [URI_A, line, character, NEW_NAME]
            }),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the command answers");
    };
    let result = response.result.expect("the map answers");
    let mut engine = AeolusEngine::new(EngineConfig::new(Some(fixture_table())));
    engine.open(URI_A, DOC_A, 1);
    engine.open(URI_B, DOC_B, 1);
    let edits = engine.rename_token(URI_A, line, character, NEW_NAME);
    let changes = result["changes"].as_object().expect("the changes map");
    let mut expected_per_uri: std::collections::HashMap<&str, usize> =
        std::collections::HashMap::new();
    for edit in &edits {
        *expected_per_uri.entry(edit.uri.as_str()).or_default() += 1;
    }
    assert_eq!(
        changes.len(),
        expected_per_uri.len(),
        "the map carries exactly the engine's documents"
    );
    for (uri, count) in &expected_per_uri {
        assert_eq!(
            changes[*uri].as_array().map(Vec::len),
            Some(*count),
            "per-document edit counts ride the engine's own fold"
        );
    }
}

#[test]
fn a_non_token_cursor_answers_no_edits() {
    let mut lsp = AeolusEngine::new(EngineConfig::new(Some(fixture_table())));
    lsp.open(URI_A, DOC_A, 1);
    let (line, character) = cursor_of(DOC_A, "p-4", 0);
    let edits = lsp.rename_token(URI_A, line, character, NEW_NAME);
    assert!(
        edits.is_empty(),
        "a spacing class names no token — the honest empty fold"
    );
}

#[test]
fn a_cursor_outside_any_candidate_answers_no_edits() {
    let mut lsp = AeolusEngine::new(EngineConfig::new(Some(fixture_table())));
    lsp.open(URI_A, DOC_A, 1);
    let (line, character) = cursor_of(DOC_A, "let a", 0);
    let edits = lsp.rename_token(URI_A, line, character, NEW_NAME);
    assert!(
        edits.is_empty(),
        "the cursor rests on code, not a class — no decomposition, no fold"
    );
}

#[test]
fn a_theme_less_config_answers_no_edits() {
    // The identity IS the table: no table, no token, no fold — the
    // honest boundary the definition face keeps.
    let mut lsp = AeolusEngine::new(EngineConfig::new(None));
    lsp.open(URI_A, DOC_A, 1);
    let (line, character) = cursor_of(DOC_A, "primary", 0);
    let edits = lsp.rename_token(URI_A, line, character, NEW_NAME);
    assert!(
        edits.is_empty(),
        "a theme-less config carries no token identity"
    );
}

#[test]
fn an_empty_new_name_answers_no_edits() {
    // The textual rename face's own validity clause: no deletion walk,
    // the quiet shape.
    let mut lsp = AeolusEngine::new(EngineConfig::new(Some(fixture_table())));
    lsp.open(URI_A, DOC_A, 1);
    let (line, character) = cursor_of(DOC_A, "primary", 0);
    let edits = lsp.rename_token(URI_A, line, character, "");
    assert!(edits.is_empty(), "an empty name renames nothing");
}

#[test]
fn a_name_the_table_does_not_carry_still_edits() {
    // The NEW spelling is the editor's business (a token the theme file
    // is about to declare): the fold renames the usages, the
    // diagnostics channel tells the truth about what the new spellings
    // lower — the textual rename face's own boundary.
    let mut lsp = AeolusEngine::new(EngineConfig::new(Some(fixture_table())));
    lsp.open(URI_A, DOC_A, 1);
    let (line, character) = cursor_of(DOC_A, "primary", 0);
    let edits = lsp.rename_token(URI_A, line, character, "not-declared-yet");
    assert_eq!(
        edits.len(),
        2,
        "the fold keys on the OLD token's identity, never the new name"
    );
    for edit in &edits {
        assert_eq!(edit.new_text, "not-declared-yet");
    }
}

#[test]
fn an_unknown_uri_answers_no_edits() {
    let lsp = AeolusEngine::new(EngineConfig::new(Some(fixture_table())));
    let (line, character) = cursor_of(DOC_A, "primary", 0);
    let edits = lsp.rename_token("file:///not-open.rs", line, character, NEW_NAME);
    assert!(
        edits.is_empty(),
        "a closed document answers the honest empty"
    );
}

#[test]
fn an_unknown_command_uri_still_answers_the_closed_table_s_null() {
    let mut session = wire_session();
    let handled = handle_message(
        &mut session,
        request(
            1,
            "workspace/executeCommand",
            json!({
                "command": "aeolus.notACommand",
                "arguments": [URI_A, 1, 22, NEW_NAME]
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
fn malformed_arguments_answer_null() {
    let mut session = wire_session();
    // Missing arguments entirely.
    let handled = handle_message(
        &mut session,
        request(
            1,
            "workspace/executeCommand",
            json!({"command": "aeolus.renameToken"}),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the command answers");
    };
    assert!(
        response.result.is_none() || response.result == Some(json!(null)),
        "no position named, no fold run — the honest null"
    );
    // The position half typed wrong: a string where the line rides.
    let handled = handle_message(
        &mut session,
        request(
            2,
            "workspace/executeCommand",
            json!({
                "command": "aeolus.renameToken",
                "arguments": [URI_A, "1", 22, NEW_NAME]
            }),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the command answers");
    };
    assert!(
        response.result.is_none() || response.result == Some(json!(null)),
        "a mistyped argument answers the protocol's own null"
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
                "command": "aeolus.renameToken",
                "arguments": [URI_A, 1, 22, NEW_NAME]
            }),
        ),
    );
    assert!(
        handled.outgoing.is_empty(),
        "no request answers before the initialize handshake"
    );
}

#[test]
fn the_token_spans_ride_byte_true() {
    // The wire's ranges must land exactly on the token's own bytes —
    // the definition face's byte-true contract, carried over to the
    // edits. One exact pin per document: the cursor document's first
    // edit covers `primary` inside `hover:bg-primary/90`; the sibling
    // document's first edit covers `primary` inside `text-primary`.
    let mut lsp = AeolusEngine::new(EngineConfig::new(Some(fixture_table())));
    lsp.open(URI_A, DOC_A, 1);
    lsp.open(URI_B, DOC_B, 1);
    let (line, character) = cursor_of(DOC_A, "primary", 0);
    let edits = lsp.rename_token(URI_A, line, character, NEW_NAME);
    let expected_a = range_of(DOC_A, "primary");
    assert_eq!(
        (edits[0].range.start.line, edits[0].range.start.character),
        (expected_a.0, expected_a.1),
        "the first edit rides the token's own start"
    );
    assert_eq!(
        (edits[0].range.end.line, edits[0].range.end.character),
        (expected_a.2, expected_a.3),
        "the first edit rides the token's own end"
    );
    // The sibling document's edits ride URI order after the cursor
    // document's own two.
    let expected_b_first = range_of(DOC_B, "primary");
    assert_eq!(
        (edits[2].range.start.line, edits[2].range.start.character),
        (expected_b_first.0, expected_b_first.1),
        "the sibling's first usage rides its own span"
    );
}
