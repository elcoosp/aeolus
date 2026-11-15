//! Consumer demonstration: the LSP engine's completion answer (the P5
//! row's editor surface, the third question beside diagnostics and
//! hover) driven headless — the same popup an editor session renders,
//! answered from the compiler's own audited tables. The stdio server
//! itself is `cargo run -p aeolus-lsp --bin aeolus-lsp` (the transport's
//! thin shell over the same engine).
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_completion
//! ```

use aeolus_lsp::{AeolusEngine, EngineConfig};

/// A synthetic .rs document carrying the three completion shapes: a
/// variant fragment, a keyword suffix, a theme-token fragment — plus
/// the honest empty answers (an unknown root, a prose string).
const DOC: &str = "fn card() {\n    let a = \"hov dark:bg-pri\";\n    let b = \"flex-col\";\n    let c = \"not-a-root\";\n}\n";

fn main() {
    // The default engine rides the built-in shadcn bake (the CLI's own
    // default theme spec); a host CSS rides `EngineConfig::new(Some(
    // TokenTable::baked(&ThemeSource::from_css(...)?)))` — the same
    // construction the CLI's `--theme <path>` accepts.
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", DOC, 1);

    // The variant half: `hov` typed, no `:` yet — the closed set the
    // compile crate audits (every suggested spelling folds).
    println!("== complete `hov` (variant stack or root, both in play) ==");
    print_completions(&lsp, DOC, "hov");

    // The utility half after a variant stack: `dark:bg-pri` — the
    // colour rows' theme tokens through the compile walk's own gate.
    println!("\n== complete `dark:bg-pri` (the theme-token source) ==");
    print_completions(&lsp, DOC, "dark:bg-pri");

    // The keyword tables: `flex-c` — the registry's own audited
    // spellings, the row's longhands the detail.
    println!("\n== complete `flex-c` (the keyword source) ==");
    print_completions(&lsp, DOC, "flex-c");

    // The honest empty answer: an unknown root completes nothing.
    println!("\n== complete `not-a-root` (the empty answer) ==");
    print_completions(&lsp, DOC, "not-a-root");
}

/// Print the completion at the fragment's typing edge (right after the
/// needle's last byte inside `doc`).
fn print_completions(lsp: &AeolusEngine, doc: &str, needle: &str) {
    let offset = doc.find(needle).expect("the fixture holds") + needle.len();
    let line = doc[..offset].matches('\n').count() as u32;
    let line_start = doc[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = doc[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    let items = lsp.complete("file:///card.rs", line, character);
    if items.is_empty() {
        println!("  (empty — the honest boundary)");
        return;
    }
    for item in items {
        println!(
            "  {:>2}. {:<24} [{}] {}",
            item.rank,
            item.label,
            kind_label(item.kind),
            item.detail
        );
    }
}

/// The item kind's label (the server maps these onto the protocol's
/// CompletionItemKind values).
fn kind_label(kind: aeolus_lsp::EngineCompletionKind) -> &'static str {
    match kind {
        aeolus_lsp::EngineCompletionKind::Utility => "utility",
        aeolus_lsp::EngineCompletionKind::Token => "token",
        aeolus_lsp::EngineCompletionKind::Variant => "variant",
    }
}
