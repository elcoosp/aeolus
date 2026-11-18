//! Consumer demonstration: the simultaneous-edit face — the wire's
//! `textDocument/linkedEditingRange` answered headless. Every textual
//! occurrence of the cursor's candidate IN THE REQUESTING document
//! rides one range (the cursor's own included, document order kept):
//! the client edits them together. The identity is the highlights'
//! textual equality — a variant-prefixed sibling is a DIFFERENT
//! spelling, a refused candidate's occurrences answer exactly like a
//! lowered one's — and the scope stays same-file (the cross-document
//! fold is the rename face's answer).
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_linked_ranges
//! ```

use aeolus_lsp::{AeolusEngine, EngineConfig};

/// A synthetic .rs document the way `tw!` call sites look: the same
/// class twice (the linked pair), a variant-prefixed sibling (a
/// different spelling), a refused candidate (the textual identity's
/// honest ride).
const DOC: &str = "fn card() {\n    let a = \"p-4 bg-primary\";\n    let b = \"hover:bg-primary\";\n    let c = \"bg-primary not-a-root\";\n}\n";

fn main() {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", DOC, 1);

    // 1. The cursor on the plain `bg-primary` in string a: the two
    //    plain spellings answer (strings a and c) — the variant-prefixed
    //    sibling in b stays OUT (a different spelling, renamed whole by
    //    its own face).
    let offset = DOC.find("bg-primary").expect("the fixture holds") + 2;
    let line = DOC[..offset].matches('\n').count() as u32;
    let line_start = DOC[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = DOC[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    println!("── cursor on the plain `bg-primary` (string a) ──");
    for range in lsp.linked_ranges("file:///card.rs", line, character) {
        println!(
            "    linked range: line {}, columns {}..{}",
            range.start.line, range.start.character, range.end.character
        );
    }

    // 2. The cursor on the variant-prefixed sibling: ONE range — the
    //    whole spelling links alone.
    let offset = DOC.find("hover:bg-primary").expect("the fixture holds") + 9;
    let line = DOC[..offset].matches('\n').count() as u32;
    let line_start = DOC[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = DOC[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    println!("── cursor on the variant-prefixed sibling (string b) ──");
    for range in lsp.linked_ranges("file:///card.rs", line, character) {
        println!(
            "    linked range: line {}, columns {}..{}  (the whole spelling)",
            range.start.line, range.start.character, range.end.character
        );
    }

    // 3. The refused candidate: the textual identity's honest ride —
    //    the occurrence answers like any candidate's.
    let offset = DOC.find("not-a-root").expect("the fixture holds") + 3;
    let line = DOC[..offset].matches('\n').count() as u32;
    let line_start = DOC[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = DOC[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    println!("── cursor on the refused candidate (string c) ──");
    let ranges = lsp.linked_ranges("file:///card.rs", line, character);
    println!(
        "    linked ranges: {} (the spelling's own occurrences)",
        ranges.len()
    );

    // 4. The cursor outside any string: the honest empty.
    println!("── cursor between the strings ──");
    println!(
        "    linked ranges: {} (nothing to link)",
        lsp.linked_ranges("file:///card.rs", 4, 0).len()
    );
}
