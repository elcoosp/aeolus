// ---------------------------------------------------------------------------
// The token goto-definition contracts (the editor's `textDocument/definition`
// face): the cursor on a theme-token class answers WHERE the token lives —
// the theme file's own declaration, byte-true in the wire's UTF-16 pairs.
// The identity rides the compiler's own vocabulary: the candidate under the
// cursor (the shared locator) must lower cleanly, its utility half must
// decompose onto a colour-group root whose remainder the riding table
// carries, and the origin is the theme pass's own provenance scan under the
// config's named origin URI. Every miss is honest: a refused class, a
// non-token class, a token the file does not declare, a theme-less config,
// an unnamed origin — all answer None, never a guessed span. A host store
// (`TokenOrigins`) rides FIRST when wired — the engine's provenance path is
// the built-in, the store the override.
//
// Written FIRST (the RED spec).
// ---------------------------------------------------------------------------

use aeolus_lsp::{
    AeolusEngine, EngineConfig, EngineDefinition, Session, TokenOrigins, handle_message,
};
use aeolus_theme::{ThemeSource, TokenTable};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// The theme fixture the battery's table bakes: one prefixed key, one
/// bare shadcn key — both on known lines.
const THEME_CSS: &str = concat!(
    "@theme {\n",
    "  --color-primary: oklch(0.205 0 0);\n",
    "}\n",
    ":root {\n",
    "  --muted: oklch(0.97 0 0);\n",
    "}\n",
);

const THEME_URI: &str = "file:///project/app.css";

/// The document the cursor answers over: a bare token class, a
/// variant-prefixed one, a non-token class, a refused one.
const DOC: &str = concat!(
    "fn card() {\n",
    "    let a = \"bg-primary p-4\";\n",
    "    let b = \"hover:bg-primary/90 not-a-root\";\n",
    "}\n",
);

const URI: &str = "file:///card.rs";

fn fixture_table() -> TokenTable {
    let source = ThemeSource::from_css(THEME_CSS, None).expect("the fixture parses");
    TokenTable::baked(&source).expect("the bake succeeds")
}

/// The engine the config path answers over: the fixture table + the
/// named origin URI.
fn engine(doc: &str) -> AeolusEngine {
    let mut lsp =
        AeolusEngine::new(EngineConfig::new(Some(fixture_table())).with_theme_origin(THEME_URI));
    lsp.open(URI, doc, 1);
    lsp
}

/// The byte offset of `needle`'s `into`-th byte, as (line, character).
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

// ---------------------------------------------------------------------------
// The engine face
// ---------------------------------------------------------------------------

#[test]
fn definition_answers_the_declaration_site() {
    let lsp = engine(DOC);
    let (line, character) = cursor_of(DOC, "bg-primary", 4);
    let answer = lsp
        .definition(URI, line, character)
        .expect("a declared token class answers");

    assert_eq!(answer.uri, THEME_URI, "the config's origin URI rides");
    assert_eq!(answer.range.start.line, 1, "the fixture's declaration line");
    assert_eq!(
        answer.range.start.character, 2,
        "the fixture's declaration column"
    );
    assert_eq!(answer.range.end.line, 1, "the key never spans lines");
    assert_eq!(
        answer.range.end.character, 17,
        "the range covers `--color-primary` (2 + 15)"
    );
}

#[test]
fn the_variant_prefixed_token_class_answers() {
    let lsp = engine(DOC);
    let (line, character) = cursor_of(DOC, "hover:bg-primary/90", 9);
    let answer = lsp
        .definition(URI, line, character)
        .expect("the variant stack and the modifier tail are presentation, not identity");

    assert_eq!(
        answer.range.start.line, 1,
        "the same declaration the bare class answers"
    );
}

#[test]
fn a_refused_class_answers_none() {
    let lsp = engine(DOC);
    let (line, character) = cursor_of(DOC, "not-a-root", 4);

    assert!(
        lsp.definition(URI, line, character).is_none(),
        "a class the walk refused carries no token"
    );
}

#[test]
fn a_non_token_class_answers_none() {
    let lsp = engine(DOC);
    let (line, character) = cursor_of(DOC, "p-4", 1);

    assert!(
        lsp.definition(URI, line, character).is_none(),
        "a non-colour root decomposes onto no token"
    );
}

#[test]
fn a_token_the_file_does_not_declare_answers_none() {
    let lsp = engine(DOC);
    // `red-500` rides the v4 palette's own seed — the table carries the
    // name, the watched file never declares it: the honest dark.
    let doc = "fn x() {\n    let a = \"bg-red-500\";\n}\n";
    let mut seeded =
        AeolusEngine::new(EngineConfig::new(Some(fixture_table())).with_theme_origin(THEME_URI));
    seeded.open(URI, doc, 1);
    let (line, character) = cursor_of(doc, "bg-red-500", 4);

    assert!(
        seeded.definition(URI, line, character).is_none(),
        "a seeded token the file does not declare answers no origin"
    );
    let _ = lsp;
}

#[test]
fn the_themeless_boundary_answers_none() {
    let mut lsp = AeolusEngine::new(EngineConfig::new(None).with_theme_origin(THEME_URI));
    lsp.open(URI, DOC, 1);
    let (line, character) = cursor_of(DOC, "bg-primary", 4);

    assert!(
        lsp.definition(URI, line, character).is_none(),
        "the theme-less boundary has no table to decompose against"
    );
}

#[test]
fn an_unnamed_origin_answers_none() {
    let mut lsp = AeolusEngine::new(EngineConfig::new(Some(fixture_table())));
    lsp.open(URI, DOC, 1);
    let (line, character) = cursor_of(DOC, "bg-primary", 4);

    assert!(
        lsp.definition(URI, line, character).is_none(),
        "without the origin URI there is no site to name — the honest dark"
    );
}

#[test]
fn the_dark_pair_class_answers() {
    let light = "@theme {\n  --color-primary: oklch(0.205 0 0);\n}\n";
    let dark = ".dark {\n  --color-primary: oklch(0.985 0 0);\n}\n";
    let source = ThemeSource::from_css(light, Some(dark)).expect("the fixture parses");
    let table = TokenTable::baked(&source).expect("the bake succeeds");
    let mut lsp = AeolusEngine::new(EngineConfig::new(Some(table)).with_theme_origin(THEME_URI));
    lsp.open(URI, DOC, 1);
    let (line, character) = cursor_of(DOC, "bg-primary", 4);

    let answer = lsp
        .definition(URI, line, character)
        .expect("a paired token answers its light-sheet site");
    assert_eq!(
        answer.range.start.line, 1,
        "the light sheet is the canonical site"
    );
}

#[test]
fn the_unknown_uri_answers_none() {
    let lsp = engine(DOC);

    assert!(
        lsp.definition("file:///not-open.rs", 0, 0).is_none(),
        "a closed document answers no definition"
    );
}

#[test]
fn the_answer_is_pure_over_the_document_map() {
    let lsp = engine(DOC);
    let (line, character) = cursor_of(DOC, "bg-primary", 4);
    let first = lsp.definition(URI, line, character);
    let second = lsp.definition(URI, line, character);

    assert_eq!(first, second, "the same question answers the same site");
}

// ---------------------------------------------------------------------------
// The host store (the TokenOrigins seam)
// ---------------------------------------------------------------------------

/// A host's own store: the URI the host names and the tokens it knows,
/// each answering a fixed site. The test's inline impl — the seam the
/// `ThemeLoader`/`DynStore` precedents keep.
struct HostStore {
    uri: &'static str,
}

impl TokenOrigins for HostStore {
    fn definition(&self, token: &str) -> Option<EngineDefinition> {
        // The host's own map: `primary` rides a virtual-fs URI, and a
        // token the theme pass never scanned (`brand`) still answers —
        // the store is the host's OWN statement, not the engine's scan.
        match token {
            "primary" => Some(EngineDefinition {
                uri: self.uri.to_string(),
                range: aeolus_lsp::EngineRange {
                    start: aeolus_lsp::EnginePosition {
                        line: 9,
                        character: 4,
                    },
                    end: aeolus_lsp::EnginePosition {
                        line: 9,
                        character: 12,
                    },
                },
            }),
            "brand" => Some(EngineDefinition {
                uri: self.uri.to_string(),
                range: aeolus_lsp::EngineRange {
                    start: aeolus_lsp::EnginePosition {
                        line: 12,
                        character: 0,
                    },
                    end: aeolus_lsp::EnginePosition {
                        line: 12,
                        character: 9,
                    },
                },
            }),
            _ => None,
        }
    }
}

#[test]
fn the_host_store_rides_first() {
    let mut lsp =
        AeolusEngine::new(EngineConfig::new(Some(fixture_table())).with_theme_origin(THEME_URI))
            .with_token_origins(HostStore {
                uri: "memfs:///virtual.css",
            });
    lsp.open(URI, DOC, 1);
    let (line, character) = cursor_of(DOC, "bg-primary", 4);
    let answer = lsp
        .definition(URI, line, character)
        .expect("the store answers");

    assert_eq!(
        answer.uri, "memfs:///virtual.css",
        "the wired store wins over the config's provenance path"
    );
}

#[test]
fn the_host_store_answers_where_provenance_is_dark() {
    // `brand` is no token name the table carries — the store's own
    // vocabulary answers it; a cursor ON a class that names it is the
    // only way the question reaches the store (the decomposition gate
    // keeps). The store's dark answer rides for a token the config's
    // own scan never found.
    let doc = "fn x() {\n    let a = \"bg-brand\";\n}\n";
    let mut lsp =
        AeolusEngine::new(EngineConfig::new(Some(fixture_table()))).with_token_origins(HostStore {
            uri: "memfs:///virtual.css",
        });
    lsp.open(URI, doc, 1);
    let (line, character) = cursor_of(doc, "bg-brand", 4);

    // `brand` is not the table's name: the decomposition gate refuses
    // before the store is consulted — the identity stays the compiler's
    // own vocabulary.
    assert!(
        lsp.definition(URI, line, character).is_none(),
        "a class the table cannot name never reaches the store"
    );
}

#[test]
fn the_host_store_keeps_the_compile_gate() {
    // A refused class answers None THROUGH the store too: the walk's
    // verdict rides ahead of every origin source.
    let doc = "fn x() {\n    let a = \"hover:bg-primary not-a-root\";\n}\n";
    let mut lsp =
        AeolusEngine::new(EngineConfig::new(Some(fixture_table()))).with_token_origins(HostStore {
            uri: "memfs:///virtual.css",
        });
    lsp.open(URI, doc, 1);
    let (line, character) = cursor_of(doc, "not-a-root", 4);

    assert!(
        lsp.definition(URI, line, character).is_none(),
        "a refused class answers none even with a store wired"
    );
}

// ---------------------------------------------------------------------------
// The wire half
// ---------------------------------------------------------------------------

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
    let mut session =
        Session::new(EngineConfig::new(Some(fixture_table())).with_theme_origin(THEME_URI));
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
fn initialize_advertises_definition() {
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
        result["capabilities"]["definitionProvider"], true,
        "the goto-definition face is advertised"
    );
}

#[test]
fn the_definition_arm_answers_a_location() {
    let mut session = wire_session();
    let (line, character) = cursor_of(DOC, "bg-primary", 4);
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/definition",
            json!({
                "textDocument": {"uri": URI},
                "position": {"line": line, "character": character}
            }),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the definition request answers");
    };
    // The engine answers ONE site (the light sheet's canonical
    // declaration) — the wire maps the Option onto the protocol's own
    // single `Location` shape, `null` the miss.
    let result = response.result.expect("a defined token answers a result");
    assert_eq!(result["uri"], THEME_URI);
    assert_eq!(result["range"]["start"]["line"], 1);
    assert_eq!(result["range"]["start"]["character"], 2);
    assert_eq!(result["range"]["end"]["character"], 17);
}

#[test]
fn the_arm_outside_a_candidate_answers_null() {
    let mut session = wire_session();
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/definition",
            json!({
                "textDocument": {"uri": URI},
                "position": {"line": 0, "character": 0}
            }),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the definition request answers");
    };
    assert!(
        response.result.is_none() || response.result == Some(json!(null)),
        "a cursor outside any candidate answers the protocol's own null"
    );
}

#[test]
fn the_pre_handshake_gate_holds() {
    let mut session =
        Session::new(EngineConfig::new(Some(fixture_table())).with_theme_origin(THEME_URI));
    let handled = handle_message(
        &mut session,
        request(
            1,
            "textDocument/definition",
            json!({
                "textDocument": {"uri": URI},
                "position": {"line": 1, "character": 8}
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
        request(1, "textDocument/definition", json!(null)),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        panic!("the malformed request still answers");
    };
    assert!(
        response.result.is_none() || response.result == Some(json!(null)),
        "malformed params answer the protocol's own empty"
    );
}
