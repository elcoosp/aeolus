//! Consumer demonstration: the outline folding face — the wire's
//! `textDocument/foldingRange` answered headless. One fold per class
//! string the extraction seam names — the string's own content span,
//! document order kept. The fold is a SHAPE-level answer (the
//! highlights' textual family): no compile verdict rides, a refused
//! candidate's string folds exactly like a lowered one's, and the
//! extraction seam's own pre-filter closes the answer set (a prose
//! literal with no `-`/`:` token is not a class string, so it never
//! folds). The multi-line raw-string call sites are the folds the
//! editor actually collapses.
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_folding_ranges
//! ```

use aeolus_lsp::{AeolusEngine, EngineConfig};

/// A synthetic .rs document the way `tw!` call sites look: two
/// single-line quoted class strings, a refused candidate riding one of
/// them, a prose literal the extractor pre-filters out, and the
/// multi-line raw string the fold actually folds (the quoted walk stops
/// at a newline — the raw-string form is the multi-line shape).
const DOC: &str = concat!(
    "fn card() {\n",
    "    let a = \"p-4 flex-col gap-2 rounded-md\";\n",
    "    let b = \"hover:bg-primary not-a-root\";\n",
    "    let prose = \"a plain sentence, no tokens\";\n",
    "    let c = r\"flex-col\n",
    "        items-center\";\n",
    "}\n",
);

fn main() {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", DOC, 1);

    println!("── the document's folds (one per class string) ──");
    for fold in lsp.folding_ranges("file:///card.rs") {
        println!(
            "    fold: line {}, column {} → line {}, column {}",
            fold.start.line, fold.start.character, fold.end.line, fold.end.character
        );
    }

    // The multi-line raw string is the fold the editor collapses: the
    // content spans lines 4 and 5, so the client renders a real fold
    // marker there.
    println!("── the collapsed shape of the raw-string call site ──");
    let folds = lsp.folding_ranges("file:///card.rs");
    let raw = &folds[folds.len() - 1];
    println!(
        "    `flex-col items-center` folds from line {} to line {} — the client collapses the literal",
        raw.start.line, raw.end.line
    );

    // The closed-document boundary: the honest empty.
    println!("── an unknown URI ──");
    println!(
        "    folds: {} (the closed document answers empty)",
        lsp.folding_ranges("file:///not-open.rs").len()
    );
}
