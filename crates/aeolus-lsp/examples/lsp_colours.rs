//! Consumer demonstration: the LSP engine's colour-swatch answer (the
//! editor's `textDocument/documentColor`, the fourth question beside
//! diagnostics, hover and completion) driven headless — the swatches an
//! editor paints beside the class strings, resolved from the SAME
//! pipeline the CLI runs. The stdio server itself is
//! `cargo run -p aeolus-lsp --bin aeolus-lsp` (the transport's thin
//! shell over the same engine).
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_colours
//! ```

use aeolus_lsp::{AeolusEngine, EngineConfig};

/// A synthetic .rs document carrying the swatch shapes: a theme-token
/// class, a modifier-baked token, a keyword slot (the honest boundary —
/// a CSS keyword is not a swatch) and a non-colour class.
const DOC: &str = "fn card() {\n    let a = \"bg-primary text-primary/50 border-secondary\";\n    let b = \"bg-transparent p-4\";\n}\n";

fn main() {
    // The default engine rides the built-in shadcn bake (the CLI's own
    // default theme spec); a host CSS rides `EngineConfig::new(Some(
    // TokenTable::baked(&ThemeSource::from_css(...)?)))` — the same
    // construction the CLI's `--theme <path>` accepts.
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", DOC, 1);

    // The swatches: every colour-valued candidate with its byte-true
    // range and the table's resolved light-tier rgba — exactly what the
    // server maps onto the protocol's ColorInformation.
    println!("== colour swatches (file:///card.rs) ==");
    for swatch in lsp.document_colours("file:///card.rs") {
        let [r, g, b, a] = swatch.rgba;
        println!(
            "{}:{}..{}:{}  rgb({r}, {g}, {b}) alpha {a}",
            swatch.range.start.line,
            swatch.range.start.character,
            swatch.range.end.line,
            swatch.range.end.character,
        );
    }

    // The honest boundaries: the theme-less config answers empty for
    // every token class (the compile walk's own boundary — the colour
    // tokens lower through the theme pass, the theme-less default
    // refuses), the keyword slots stay unnamed (`bg-transparent` — a
    // CSS keyword, not a swatch), and the non-colour classes carry
    // nothing to paint.
    println!("\n== the theme-less boundary (file:///plain.rs) ==");
    let mut plain = AeolusEngine::new(EngineConfig::new(None));
    plain.open(
        "file:///plain.rs",
        "fn s() {\n    let t = \"bg-primary\";\n}\n",
        1,
    );
    let swatches = plain.document_colours("file:///plain.rs");
    assert!(swatches.is_empty(), "the theme-less boundary answers empty");
    println!("  (empty — the token classes refuse without the table)");
}
