//! Consumer demonstration: the LSP engine (the P5 row's
//! hover/diagnostics surface, plan §5.5's tooling-parity line) driven
//! headless — the same two questions an editor session asks, answered
//! from the same pipeline the CLI runs. The stdio server itself is
//! `cargo run -p aeolus-lsp --bin aeolus-lsp` (the transport's thin
//! shell over the same engine).
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_hover
//! ```

use aeolus_lsp::{AeolusEngine, EngineConfig};

/// A synthetic .rs document the way `tw!` call sites look: one lowered
/// string, one unknown root, one unknown variant — the honest refusal
/// shapes the diagnostics publish carries.
const DOC: &str = "fn card() {\n    let s = \"p-4 flex-col gap-2\";\n    let t = \"not-a-root hover:unknown:thing\";\n}\n";

fn main() {
    // The default engine rides the built-in shadcn bake (the CLI's own
    // default theme spec); a host CSS rides `EngineConfig::new(Some(
    // TokenTable::baked(&ThemeSource::from_css(...)?)))` — the same
    // construction the CLI's `--theme <path>` accepts.
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", DOC, 1);

    // The diagnostics publish: every typed refusal with its byte-true
    // UTF-16 range and stable code — exactly what the server sends on
    // didOpen/didChange.
    println!("== diagnostics (file:///card.rs) ==");
    for finding in lsp.analyze("file:///card.rs") {
        println!(
            "{}:{}..{}:{} [{}] {}",
            finding.range.start.line,
            finding.range.start.character,
            finding.range.end.line,
            finding.range.end.character,
            finding.code,
            finding.message.replace('\n', " ")
        );
    }

    // The hover resolutions: the cursor inside a lowered class renders
    // the folded condition and the CSS arm's own declaration texts.
    println!("\n== hover (cursor inside `flex-col`) ==");
    let string_start = DOC.find("flex-col").expect("the fixture holds");
    let (line, character) = position_of(DOC, string_start + 3);
    if let Some(hover) = lsp.hover("file:///card.rs", line, character) {
        println!("{}", hover.contents);
    }

    // The refusal hover: the typed refusal (code + help) is the answer
    // the editor sees — never a silent drop.
    println!("\n== hover (cursor inside `not-a-root`) ==");
    let bad_start = DOC.find("not-a-root").expect("the fixture holds");
    let (line, character) = position_of(DOC, bad_start + 2);
    if let Some(hover) = lsp.hover("file:///card.rs", line, character) {
        println!("{}", hover.contents);
    }

    // The honest empty answer: a cursor outside any candidate string.
    println!("\n== hover (cursor at line 0, outside any string) ==");
    match lsp.hover("file:///card.rs", 0, 0) {
        None => println!("None — no candidate under the cursor"),
        Some(hover) => println!("{}", hover.contents),
    }
}

/// Byte offset → (line, character-UTF16) — the conversion the engine
/// owns; the example only finds its fixture's offsets.
fn position_of(text: &str, offset: usize) -> (u32, u32) {
    let head = &text[..offset];
    let line = head.matches('\n').count() as u32;
    let line_start = head.rfind('\n').map_or(0, |i| i + 1);
    let character = head[line_start..]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    (line, character)
}
