//! Consumer demonstration: the token goto-definition face — the wire's
//! `textDocument/definition` answered headless. The cursor on a
//! theme-token class answers WHERE the token lives: the theme file's
//! own `--color-{name}` (or bare shadcn `--{name}`) declaration, byte-true
//! in the UTF-16 pairs, from the theme pass's provenance scan. The
//! identity rides the compiler's own vocabulary — the candidate must
//! lower cleanly and its utility half must decompose onto a colour
//! root whose remainder the table carries — and every miss is honest:
//! a non-token class, a seeded token the file never declares, a cursor
//! outside any candidate all answer nothing.
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_definition
//! ```

use aeolus_lsp::{AeolusEngine, EngineConfig};
use aeolus_theme::{ThemeSource, TokenTable};

/// The theme file the session's table comes from — the file the
/// definitions jump into (the `--theme` file's URI, the same file the
/// `FileThemeLoader` watches). One prefixed key and one bare shadcn
/// key: both spellings the engine's decomposition answers.
const THEME_CSS: &str = concat!(
    "@theme {\n",
    "  --color-primary: oklch(0.205 0 0);\n",
    "  --color-secondary: oklch(0.97 0 0);\n",
    "}\n",
    ":root {\n",
    "  --muted: oklch(0.97 0 0);\n",
    "}\n",
);

const THEME_URI: &str = "file:///project/app.css";

/// A synthetic .rs document the way `tw!` call sites look: a bare
/// token class, a variant-prefixed one over the `/opacity` modifier,
/// a non-token class and a seeded palette class the theme file never
/// declares.
const DOC: &str = concat!(
    "fn card() {\n",
    "    let a = \"bg-primary p-4\";\n",
    "    let b = \"hover:bg-secondary/90 bg-red-500\";\n",
    "}\n",
);

const URI: &str = "file:///card.rs";

/// The cursor position over `needle`'s `into`-th byte, in the LSP's
/// own (line, UTF-16 character) pairs.
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

fn main() {
    // The session's table bakes from the theme file's own text — the
    // provenance scan rides it, the origin URI names the file the
    // definitions answer into. A live server composes the same pair
    // from `Session::with_theme_loader(FileThemeLoader::new(...))`.
    let source = ThemeSource::from_css(THEME_CSS, None).expect("the theme parses");
    let table = TokenTable::baked(&source).expect("the bake succeeds");
    let mut lsp = AeolusEngine::new(EngineConfig::new(Some(table)).with_theme_origin(THEME_URI));
    lsp.open(URI, DOC, 1);

    println!("── the cursor's token classes answer their declarations ──");
    for (needle, into) in [("bg-primary", 4), ("hover:bg-secondary/90", 9)] {
        let (line, character) = cursor_of(DOC, needle, into);
        let answer = lsp.definition(URI, line, character);
        match answer {
            Some(site) => println!(
                "    `{needle}` → {}:{}:{} (the key's own {} columns)",
                site.uri,
                site.range.start.line,
                site.range.start.character,
                site.range.end.character - site.range.start.character
            ),
            None => println!("    `{needle}` → (nothing — the honest dark)"),
        }
    }

    // The honest misses: a non-colour root decomposes onto no token;
    // `red-500` rides the palette's own seed, the watched file never
    // declares it — no origin, never a guessed span.
    println!("── the honest misses ──");
    for needle in ["p-4", "bg-red-500"] {
        let (line, character) = cursor_of(DOC, needle, 2);
        let answer = lsp.definition(URI, line, character);
        println!(
            "    `{needle}` → {}",
            answer.map_or(String::from("(nothing — the honest dark)"), |_| {
                String::from("(a site)")
            })
        );
    }

    // The closed-document boundary: the honest empty.
    println!("── an unknown URI ──");
    println!(
        "    definition: {} (the closed document answers nothing)",
        lsp.definition("file:///not-open.rs", 0, 0).is_none()
    );
}
