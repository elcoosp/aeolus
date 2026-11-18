//! Consumer demonstration: the theme lifecycle — the editor's `app.css`
//! save button re-themes a live session. A host wires ONE loader
//! ([`ThemeLoader`], the seam; [`FileThemeLoader`] the disk form) and
//! the wire half drives it: the `workspace/didChangeWatchedFiles`
//! notification naming the loader's URI re-bakes the table, the engine's
//! retheme retires the pooled rows lowered under the outgoing one, and
//! every open document's diagnostics republish. The no-op save (the
//! re-baked table equals the riding one) answers nothing — the
//! byte-neutral shape the differential pins.
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_theme_reload
//! ```

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;

use aeolus_lsp::{EngineConfig, FileThemeLoader, Session, ThemeError, ThemeLoader, handle_message};
use aeolus_theme::{ThemeSource, TokenTable};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// The session's first theme: the brand colour the swatches resolve.
const V1_CSS: &str = ":root { --color-primary: #336699; --color-surface: #eeeeee; }";
/// The designer's rebrand: same token names, new brand colour — the
/// reload's controlled input.
const V2_CSS: &str = ":root { --color-primary: #aa4455; --color-surface: #eeeeee; }";

/// A synthetic .rs document the way `tw!` call sites look.
const DOC: &str = "fn card() {\n    let a = \"bg-primary text-surface\";\n}\n";

/// A host's OWN loader over a baked css string (the seam's inline form
/// — a VFS read, a watch service, a test double all take this shape;
/// [`FileThemeLoader`] is the disk form). One uri owned, one table
/// served — here the css swaps on demand so the demo can play both
/// saves.
struct EditableCssLoader {
    uri: String,
    css: Mutex<&'static str>,
}

impl EditableCssLoader {
    fn save(&self, css: &'static str) {
        *self.css.lock().unwrap_or_else(PoisonError::into_inner) = css;
    }
}

impl ThemeLoader for EditableCssLoader {
    fn uri(&self) -> &str {
        &self.uri
    }

    fn load(&self) -> Result<TokenTable, ThemeError> {
        let source = ThemeSource::from_css(
            *self.css.lock().unwrap_or_else(PoisonError::into_inner),
            None,
        )?;
        TokenTable::baked(&source)
    }
}

fn message(method: &str, params: serde_json::Value, id: Option<i32>) -> Message {
    match id {
        Some(id) => Message::Request(Request {
            id: RequestId::from(id),
            method: method.to_string(),
            params,
        }),
        None => Message::Notification(Notification {
            method: method.to_string(),
            params,
        }),
    }
}

/// The documentColour request's first swatch, printed as the resolved
/// bytes (the wire's 0..1 fractions scaled back).
fn first_swatch(session: &mut Session, id: i32) -> Option<[u8; 4]> {
    let handled = handle_message(
        session,
        message(
            "textDocument/documentColor",
            json!({"textDocument": {"uri": "file:///card.rs"}}),
            Some(id),
        ),
    );
    for outgoing in handled.outgoing {
        let Message::Response(response) = outgoing else {
            continue;
        };
        let result = response.result?;
        return Some([
            (result[0]["color"]["red"].as_f64()? * 255.0).round() as u8,
            (result[0]["color"]["green"].as_f64()? * 255.0).round() as u8,
            (result[0]["color"]["blue"].as_f64()? * 255.0).round() as u8,
            (result[0]["color"]["alpha"].as_f64()? * 255.0).round() as u8,
        ]);
    }
    None
}

fn main() {
    // The host's loader: the watched app.css beside its on-disk path —
    // the disk form `FileThemeLoader::new(uri, path)` is what a real
    // editor integration wires; the editable form lets the demo play
    // the saves.
    let loader = Arc::new(EditableCssLoader {
        uri: "file:///project/app.css".to_string(),
        css: Mutex::new(V1_CSS),
    });
    let source = ThemeSource::from_css(V1_CSS, None).expect("the demo css parses");
    let mut session = Session::new(EngineConfig::new(Some(
        TokenTable::baked(&source).expect("the demo css bakes"),
    )))
    .with_theme_loader(Arc::clone(&loader) as Arc<dyn ThemeLoader>);

    let _ = handle_message(
        &mut session,
        message("initialize", json!({"capabilities": {}}), Some(0)),
    );
    let _ = handle_message(
        &mut session,
        message(
            "textDocument/didOpen",
            json!({
                "textDocument": {"uri": "file:///card.rs", "languageId": "rust", "version": 1, "text": DOC}
            }),
            None,
        ),
    );
    println!("── didOpen over the v1 theme ──");
    println!(
        "    swatch: {:?}  (the v1 brand colour)",
        first_swatch(&mut session, 1).expect("the colour class answers")
    );

    // The designer saves the rebrand: the css body changes on the
    // loader's side, the editor's watch service notifies, the wire half
    // re-bakes and re-themes — and republishes every open document.
    loader.save(V2_CSS);
    let handled = handle_message(
        &mut session,
        message(
            "workspace/didChangeWatchedFiles",
            json!({"changes": [{"uri": "file:///project/app.css", "type": 2}]}),
            None,
        ),
    );
    let republished = handled
        .outgoing
        .iter()
        .filter(|m| {
            matches!(
                m,
                Message::Notification(n) if n.method == "textDocument/publishDiagnostics"
            )
        })
        .count();
    println!("── the rebrand save (a watched-files event for the theme uri) ──");
    println!("    republished documents: {republished}");
    println!(
        "    swatch: {:?}  (the v2 brand colour — the old pool retired)",
        first_swatch(&mut session, 2).expect("the colour class answers")
    );

    // The save that changed nothing: the re-baked table equals the
    // riding one, the byte-neutral no-op — nothing republishes.
    let handled = handle_message(
        &mut session,
        message(
            "workspace/didChangeWatchedFiles",
            json!({"changes": [{"uri": "file:///project/app.css", "type": 2}]}),
            None,
        ),
    );
    println!("── the no-op save (the same css re-baked) ──");
    println!(
        "    outgoing messages: {}  (the pool kept serving)",
        handled.outgoing.len()
    );

    // The failure lane: the file the loader reads vanishes — the typed
    // error keeps the riding table, nothing republishes, the session
    // answers the last good theme.
    println!("── a failed load keeps the last good table (the typed ThemeError lane) ──");
    let _ = FileThemeLoader::new("file:///missing.css", "/definitely/missing.css");
    println!("    (the disk form's read error rides ThemeError::Source — see its rustdoc)");
}
