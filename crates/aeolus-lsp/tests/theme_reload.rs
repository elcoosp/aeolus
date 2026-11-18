// ---------------------------------------------------------------------------
// The theme lifecycle's contracts (the reload seam): the engine's
// retheme answers — the swap retires every pooled row lowered under the
// outgoing table, the equal-content swap is the byte-neutral no-op the
// pool survives, the theme-less boundary stays honest — and the wire's
// watched-files half drives the SAME seam through the state machine.
//
// Written FIRST (the RED spec).
// ---------------------------------------------------------------------------

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;

use aeolus_lsp::{
    AeolusEngine, CacheStats, EngineConfig, FileThemeLoader, RethemeOutcome, Session, ThemeError,
    ThemeLoader, handle_message,
};
use aeolus_theme::{ThemeSource, TokenTable};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// Two synthetic sources over the same token names — the swap's
/// controlled input: the colours differ, the surface token does not
/// (the equality fast-path's shared-row probe).
const RED_CSS: &str = ":root { --color-primary: #336699; --color-surface: #eeeeee; }";
const BLUE_CSS: &str = ":root { --color-primary: #aa4455; --color-surface: #eeeeee; }";

/// The document the engine answers over: one colour class (the swatch
/// walks), one plain token class.
const DOC: &str = "fn card() {\n    let a = \"bg-primary text-surface\";\n}\n";

const URI: &str = "file:///card.rs";
const THEME_URI: &str = "file:///app.css";

fn table(css: &str) -> TokenTable {
    let source = ThemeSource::from_css(css, None).expect("the fixture css parses");
    TokenTable::baked(&source).expect("the fixture css bakes")
}

fn swatch(engine: &AeolusEngine) -> [u8; 4] {
    let colours = engine.document_colours(URI);
    assert_eq!(colours.len(), 1, "the fixture carries one colour class");
    colours[0].rgba
}

// ---------------------------------------------------------------------------
// The engine face
// ---------------------------------------------------------------------------

#[test]
fn retheme_swaps_the_token_answers() {
    let mut engine = AeolusEngine::new(EngineConfig::new(Some(table(RED_CSS))));
    engine.open(URI, DOC, 1);
    assert_eq!(
        swatch(&engine),
        [0x33, 0x66, 0x99, 0xff],
        "the red table answers"
    );

    assert_eq!(
        engine.retheme(Some(table(BLUE_CSS))),
        RethemeOutcome::Swapped,
        "a different table is the swap"
    );
    assert_eq!(
        swatch(&engine),
        [0xaa, 0x44, 0x55, 0xff],
        "the new table answers — a pooled row from the old one never does"
    );
}

#[test]
fn retheme_the_same_content_answers_unchanged_and_keeps_the_pool() {
    let stats = Arc::new(CacheStats::new());
    let mut engine = AeolusEngine::new(EngineConfig::new(Some(table(RED_CSS))))
        .with_observer(Arc::clone(&stats));
    engine.open(URI, DOC, 1);
    let _ = engine.analyze(URI);
    let _ = engine.analyze(URI); // the warm pass
    let warm = stats.snapshot();

    // A re-bake of the SAME css: a different table object, the same
    // content — the no-op save's shape.
    assert_eq!(
        engine.retheme(Some(table(RED_CSS))),
        RethemeOutcome::Unchanged,
        "equal content is the no-op"
    );
    assert_eq!(
        stats.snapshot().misses,
        warm.misses,
        "the no-op compiles nothing"
    );
    let _ = engine.analyze(URI);
    assert_eq!(
        stats.snapshot().misses,
        warm.misses,
        "the pool survives the no-op: the warm strings still hit"
    );
}

#[test]
fn the_swap_clears_the_pool_and_the_new_table_answers() {
    let stats = Arc::new(CacheStats::new());
    let mut engine = AeolusEngine::new(EngineConfig::new(Some(table(RED_CSS))))
        .with_observer(Arc::clone(&stats));
    engine.open(URI, DOC, 1);
    let _ = engine.analyze(URI);
    let _ = engine.analyze(URI); // the warm pass
    let warm = stats.snapshot().misses;

    assert_eq!(
        engine.retheme(Some(table(BLUE_CSS))),
        RethemeOutcome::Swapped
    );
    let _ = engine.analyze(URI);
    assert!(
        stats.snapshot().misses > warm,
        "the swap emptied the pool: the strings re-lower under the new table"
    );
    assert_eq!(swatch(&engine), [0xaa, 0x44, 0x55, 0xff]);
}

#[test]
fn retheme_the_themeless_boundary_stays_honest() {
    let mut engine = AeolusEngine::new(EngineConfig::new(None));
    engine.open(URI, DOC, 1);
    assert_eq!(
        engine.retheme(None),
        RethemeOutcome::Unchanged,
        "themeless onto themeless is the no-op"
    );
    assert!(
        engine.document_colours(URI).is_empty(),
        "the theme-less boundary resolves no colour"
    );

    assert_eq!(
        engine.retheme(Some(table(RED_CSS))),
        RethemeOutcome::Swapped,
        "a first theme is the swap"
    );
    assert_eq!(
        swatch(&engine),
        [0x33, 0x66, 0x99, 0xff],
        "the themed walk answers once a table rides"
    );
}

// ---------------------------------------------------------------------------
// The loader seam
// ---------------------------------------------------------------------------

#[test]
fn the_file_loader_reads_the_disk() {
    let path = std::env::temp_dir().join(format!("aeolus-theme-reload-{}.css", std::process::id()));
    std::fs::write(&path, RED_CSS).expect("the fixture file writes");
    let loader = FileThemeLoader::new(THEME_URI, &path);
    assert_eq!(loader.uri(), THEME_URI);

    let tokens = loader.load().expect("the fixture file bakes");
    let primary = tokens
        .id("primary")
        .and_then(|id| tokens.light_rgba(id))
        .expect("the fixture names a primary token");
    assert_eq!(primary, [0x33, 0x66, 0x99, 0xff]);

    let _ = std::fs::remove_file(&path);
}

// ---------------------------------------------------------------------------
// The wire face
// ---------------------------------------------------------------------------

/// The scripted loader: a host's own [`ThemeLoader`] over a queue of
/// css bodies — the first load answers the head, every later load the
/// SAME (a one-step script). The trait is the seam; the disk form is
/// [`FileThemeLoader`].
struct ScriptedLoader {
    uri: String,
    css: Mutex<Vec<&'static str>>,
}

impl ScriptedLoader {
    fn new(uri: &str, css: &'static str) -> Self {
        Self {
            uri: uri.to_string(),
            css: Mutex::new(vec![css]),
        }
    }

    fn stepping(uri: &str, css: Vec<&'static str>) -> Self {
        Self {
            uri: uri.to_string(),
            css: Mutex::new(css),
        }
    }
}

impl ThemeLoader for ScriptedLoader {
    fn uri(&self) -> &str {
        &self.uri
    }

    fn load(&self) -> Result<TokenTable, ThemeError> {
        let mut slot = self.css.lock().unwrap_or_else(PoisonError::into_inner);
        if slot.len() > 1 {
            let _ = slot.remove(0);
        }
        let source = ThemeSource::from_css(slot[0], None)?;
        TokenTable::baked(&source)
    }
}

/// The loader that always fails (the disk error's stand-in: the typed
/// `ThemeError` is what a failed read carries).
struct FailingLoader {
    uri: String,
}

impl ThemeLoader for FailingLoader {
    fn uri(&self) -> &str {
        &self.uri
    }

    fn load(&self) -> Result<TokenTable, ThemeError> {
        Err(ThemeError::Source {
            message: "the fixture's read always fails".to_string(),
        })
    }
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

fn did_open(uri: &str, text: &str) -> Message {
    notification(
        "textDocument/didOpen",
        json!({
            "textDocument": {"uri": uri, "languageId": "rust", "version": 1, "text": text}
        }),
    )
}

fn watched_files(uris: &[&str]) -> Message {
    notification(
        "workspace/didChangeWatchedFiles",
        json!({"changes": uris.iter().map(|u| json!({"uri": u, "type": 2})).collect::<Vec<_>>()}),
    )
}

fn handshake(session: &mut Session) {
    let handled = handle_message(
        session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    assert!(!handled.outgoing.is_empty(), "initialize answers");
}

/// The documentColour request's first swatch, as the wire's 0..1
/// fractions scaled back to bytes (the swatch's own round-trip).
fn wire_swatch(session: &mut Session, id: i32) -> Option<[u8; 4]> {
    let handled = handle_message(
        session,
        request(
            id,
            "textDocument/documentColor",
            json!({"textDocument": {"uri": URI}}),
        ),
    );
    let Some(Message::Response(response)) = handled.outgoing.into_iter().next() else {
        return None;
    };
    let result = response.result?;
    let red = result[0]["color"]["red"].as_f64()?;
    let green = result[0]["color"]["green"].as_f64()?;
    let blue = result[0]["color"]["blue"].as_f64()?;
    let alpha = result[0]["color"]["alpha"].as_f64()?;
    Some([
        (red * 255.0).round() as u8,
        (green * 255.0).round() as u8,
        (blue * 255.0).round() as u8,
        (alpha * 255.0).round() as u8,
    ])
}

#[test]
fn watched_theme_file_rethemes_the_session() {
    let mut session = Session::new(EngineConfig::new(Some(table(RED_CSS))))
        .with_theme_loader(ScriptedLoader::stepping(THEME_URI, vec![RED_CSS, BLUE_CSS]));
    handshake(&mut session);
    let _ = handle_message(&mut session, did_open(URI, DOC));

    assert_eq!(
        wire_swatch(&mut session, 1),
        Some([0x33, 0x66, 0x99, 0xff]),
        "the pre-retheme swatch rides the red table"
    );

    let handled = handle_message(&mut session, watched_files(&[THEME_URI]));
    assert!(
        !handled.exit,
        "the watched-files notification never ends the session"
    );
    assert!(
        handled
            .outgoing
            .iter()
            .any(|m| matches!(m, Message::Notification(n) if n.method == "textDocument/publishDiagnostics")),
        "the swap republishes the open document's findings"
    );

    assert_eq!(
        wire_swatch(&mut session, 2),
        Some([0xaa, 0x44, 0x55, 0xff]),
        "the post-retheme swatch rides the new table"
    );
}

#[test]
fn the_noop_save_answers_nothing() {
    let mut session = Session::new(EngineConfig::new(Some(table(RED_CSS))))
        .with_theme_loader(ScriptedLoader::new(THEME_URI, RED_CSS));
    handshake(&mut session);
    let _ = handle_message(&mut session, did_open(URI, DOC));

    let handled = handle_message(&mut session, watched_files(&[THEME_URI]));
    assert!(
        handled.outgoing.is_empty(),
        "an equal-content save is the byte-neutral no-op: nothing republishes"
    );
}

#[test]
fn watched_unrelated_file_rides_silently() {
    let mut session = Session::new(EngineConfig::new(Some(table(RED_CSS))))
        .with_theme_loader(ScriptedLoader::stepping(THEME_URI, vec![RED_CSS, BLUE_CSS]));
    handshake(&mut session);
    let _ = handle_message(&mut session, did_open(URI, DOC));

    let handled = handle_message(&mut session, watched_files(&["file:///other.css"]));
    assert!(
        handled.outgoing.is_empty(),
        "an event for a file the loader does not own changes nothing"
    );
    assert_eq!(
        wire_swatch(&mut session, 1),
        Some([0x33, 0x66, 0x99, 0xff]),
        "the answers stay the old table's"
    );
}

#[test]
fn wire_without_a_loader_rides_silently() {
    let mut session = Session::new(EngineConfig::new(Some(table(RED_CSS))));
    handshake(&mut session);
    let _ = handle_message(&mut session, did_open(URI, DOC));

    let handled = handle_message(&mut session, watched_files(&[THEME_URI]));
    assert!(
        handled.outgoing.is_empty(),
        "a session with no theme lifecycle answers nothing on the channel"
    );
}

#[test]
fn the_failed_load_keeps_the_old_table() {
    let mut session =
        Session::new(EngineConfig::new(Some(table(RED_CSS)))).with_theme_loader(FailingLoader {
            uri: THEME_URI.to_string(),
        });
    handshake(&mut session);
    let _ = handle_message(&mut session, did_open(URI, DOC));

    let handled = handle_message(&mut session, watched_files(&[THEME_URI]));
    assert!(
        handled.outgoing.is_empty(),
        "a failed load changes nothing: the old table still answers"
    );
    assert_eq!(
        wire_swatch(&mut session, 1),
        Some([0x33, 0x66, 0x99, 0xff]),
        "the session stays on the last good table"
    );
}

#[test]
fn the_retheme_republishes_every_open_document() {
    let mut session = Session::new(EngineConfig::new(Some(table(RED_CSS))))
        .with_theme_loader(ScriptedLoader::stepping(THEME_URI, vec![RED_CSS, BLUE_CSS]));
    handshake(&mut session);
    let _ = handle_message(&mut session, did_open("file:///b.rs", DOC));
    let _ = handle_message(&mut session, did_open("file:///a.rs", DOC));

    let handled = handle_message(&mut session, watched_files(&[THEME_URI]));
    let published: Vec<String> = handled
        .outgoing
        .iter()
        .filter_map(|m| match m {
            Message::Notification(n) if n.method == "textDocument/publishDiagnostics" => Some(
                n.params["uri"]
                    .as_str()
                    .expect("the publish names a uri")
                    .to_string(),
            ),
            _ => None,
        })
        .collect();
    assert_eq!(
        published,
        vec!["file:///a.rs".to_string(), "file:///b.rs".to_string()],
        "the republish covers EVERY open document, uri order"
    );
}
