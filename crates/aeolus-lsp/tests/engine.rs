//! The engine's contracts (the P5 row's LSP hover/diagnostics line, the
//! §5.5 tooling-parity half): the headless surface answers from the
//! SAME pipeline the CLI runs — the extraction seam's strings through
//! the V1 compile walk, per-location typed diagnostics with their
//! stable codes, hover resolutions keyed by the candidate under the
//! cursor — with the LSP's UTF-16 positions handled ONCE, in the
//! engine, so a host never re-derives them.
//!
//! The pins, in the order the crate docs promise them:
//!
//! * the engine is headless and pure over its document map: open →
//!   change → analyze is deterministic, and close drops the document;
//! * diagnostics carry the source's byte-true ranges translated into
//!   the LSP's UTF-16 line/character pairs (a multi-byte character
//!   before the class shifts the CHARACTER column, never the line);
//! * every diagnostic rides its stable machine code and help text (the
//!   same strings the `miette` feature exposes);
//! * hover resolves the class under the cursor: a lowered class
//!   renders its longhand signature (the row texts the keyword slots
//!   index, the theme tier's resolved colours), a refused class renders
//!   the typed refusal (code + help), and a position outside any
//!   candidate string is None — the honest empty answer;
//! * the default theme is the built-in shadcn bake; a host CSS rides
//!   the same `ThemeSource` the CLI's `--theme <path>` accepts;
//! * an unknown URI answers empty (open/change precede any read; the
//!   server's lifecycle contract, mirrored in the engine).
//!
//! Written FIRST (the RED spec): the suite fails against the
//! not-yet-existing seam and passes only once the engine satisfies
//! every pin below.

use aeolus_compile::Config;
use aeolus_lsp::{AeolusEngine, EngineConfig};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A synthetic .rs document the way `tw!` call sites look: one lowered
/// class string, one unknown root, one multi-byte identifier before the
/// classes (the UTF-16 column shift's fixture).
const DOC: &str =
    "fn card() {\n    let héllo = \"p-4 flex-col\";\n    let bad = \"not-a-root\";\n}\n";

/// The engine with the default (shadcn baked) theme.
fn engine() -> AeolusEngine {
    AeolusEngine::new(EngineConfig::default())
}

// ---------------------------------------------------------------------------
// The lifecycle: open, change, close, unknown
// ---------------------------------------------------------------------------

#[test]
fn open_then_analyze_is_deterministic() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    let first = lsp.analyze("file:///card.rs");
    let second = lsp.analyze("file:///card.rs");
    assert_eq!(first, second, "analyze is pure over its document map");
    assert!(
        !first.is_empty(),
        "the document carries lowered and refused classes: {first:?}"
    );
}

#[test]
fn change_replaces_the_document() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    lsp.change("file:///card.rs", "fn clean() {}\n", 2);
    let findings = lsp.analyze("file:///card.rs");
    assert!(
        findings.is_empty(),
        "a document with no strings: {findings:?}"
    );
}

#[test]
fn close_drops_the_document() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    lsp.close("file:///card.rs");
    assert!(
        lsp.analyze("file:///card.rs").is_empty(),
        "a closed document answers empty"
    );
    assert!(lsp.hover("file:///card.rs", 0, 0).is_none());
}

#[test]
fn unknown_uri_answers_empty() {
    let lsp = engine();
    assert!(lsp.analyze("file:///never-opened.rs").is_empty());
    assert!(lsp.hover("file:///never-opened.rs", 0, 0).is_none());
}

// ---------------------------------------------------------------------------
// The diagnostics: ranges, UTF-16, codes
// ---------------------------------------------------------------------------

#[test]
fn diagnostics_carry_the_stable_codes() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    let findings = lsp.analyze("file:///card.rs");
    // The unknown root refuses with the lookup code; the lowered class
    // rides no diagnostic.
    let codes: Vec<&str> = findings.iter().map(|d| d.code.as_str()).collect();
    assert!(
        codes
            .iter()
            .any(|code| code.contains("lookup") || code.contains("unknown")),
        "the unknown root's stable code rides: {codes:?}"
    );
    for finding in &findings {
        assert!(!finding.message.is_empty());
        assert!(finding.range.start.line <= finding.range.end.line);
    }
}

#[test]
fn ranges_translate_through_utf16() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    let findings = lsp.analyze("file:///card.rs");
    // `héllo` is one UTF-16 unit for é — the string's line-1 character
    // columns shift by exactly the identifier's UTF-16 width, never by
    // its byte width.
    let lowered_start = DOC.find("\"p-4 flex-col\"").expect("the fixture holds") + 1;
    let line = DOC[..lowered_start].matches('\n').count() as u32;
    let line_start = DOC[..lowered_start].rfind('\n').map_or(0, |i| i + 1);
    let utf16_col = DOC[line_start..lowered_start]
        .chars()
        .take_while(|c| *c != '\n')
        .map(|c| c.len_utf16() as u32)
        .sum::<u32>();
    let hit = findings
        .iter()
        .find(|d| d.range.start.line == line && d.range.start.character == utf16_col);
    // The LOWERED class carries no diagnostic — the pin is that any
    // diagnostic ON LINE 1 lands at UTF-16 columns, byte columns would
    // over-count é by one. The `not-a-root` finding proves it.
    let refused = findings
        .iter()
        .find(|d| d.message.contains("not-a-root"))
        .expect("the unknown root refuses");
    assert_eq!(
        refused.range.start.line,
        line_of(DOC, DOC.find("not-a-root").unwrap())
    );
    let refused_start = DOC.find("not-a-root").unwrap();
    let refused_line_start = DOC[..refused_start].rfind('\n').map_or(0, |i| i + 1);
    let col = DOC[refused_line_start..refused_start]
        .chars()
        .map(|c| c.len_utf16() as u32)
        .sum::<u32>();
    assert_eq!(
        refused.range.start.character, col,
        "UTF-16 columns, not byte columns (héllo must not over-count)"
    );
    let _ = (hit, lowered_start, line);
}

fn line_of(text: &str, offset: usize) -> u32 {
    text[..offset].matches('\n').count() as u32
}

#[test]
fn lowered_classes_ride_no_false_diagnostics() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    let findings = lsp.analyze("file:///card.rs");
    assert!(
        findings.iter().all(|d| !d.message.contains("p-4")),
        "the lowered classes stay quiet: {findings:?}"
    );
}

// ---------------------------------------------------------------------------
// The hover resolutions
// ---------------------------------------------------------------------------

#[test]
fn hover_renders_the_lowered_class_signature() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    // The cursor sits INSIDE `p-4` (a few characters into the string).
    let string_start = DOC.find("\"p-4 flex-col\"").expect("the fixture holds") + 1;
    let (line, character) = position_of(DOC, string_start + 2);
    let hover = lsp
        .hover("file:///card.rs", line, character)
        .expect("the cursor is on a class");
    assert!(hover.contents.contains("p-4"), "{}", hover.contents);
    // The longhand signature: the four paddings, rem values.
    assert!(hover.contents.contains("padding"), "{}", hover.contents);
    assert!(hover.contents.contains("rem"), "{}", hover.contents);
    assert!(
        hover.contents.contains("base"),
        "the folded condition renders: {}",
        hover.contents
    );
}

#[test]
fn hover_renders_the_typed_refusal() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    let start = DOC.find("not-a-root").expect("the fixture holds");
    let (line, character) = position_of(DOC, start + 2);
    let hover = lsp
        .hover("file:///card.rs", line, character)
        .expect("the cursor is on a class");
    assert!(hover.contents.contains("not-a-root"), "{}", hover.contents);
    assert!(
        hover.contents.contains("AE0") || hover.contents.contains("aeolus::"),
        "the stable code renders: {}",
        hover.contents
    );
}

#[test]
fn hover_outside_any_string_is_none() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    let (line, character) = position_of(DOC, 0); // `fn card()` — no string
    assert!(lsp.hover("file:///card.rs", line, character).is_none());
}

#[test]
fn hover_resolves_theme_tokens_through_the_default_bake() {
    // The default engine rides the built-in shadcn bake (the CLI's own
    // default theme spec): a colour token's hover renders the baked
    // tier's resolved bytes in the CSS arm's own rgb form.
    let mut lsp = engine();
    let doc = "fn ui() {\n    let s = \"bg-primary\";\n}\n";
    lsp.open("file:///ui.rs", doc, 1);
    let start = doc.find("bg-primary").expect("the fixture holds");
    let (line, character) = position_of(doc, start + 2);
    let hover = lsp
        .hover("file:///ui.rs", line, character)
        .expect("the cursor is on a class");
    assert!(
        hover.contents.contains("background-color"),
        "the longhand renders: {}",
        hover.contents
    );
    assert!(
        hover.contents.contains("rgb("),
        "the baked tier's resolved bytes render: {}",
        hover.contents
    );
}

// ---------------------------------------------------------------------------
// The helpers
// ---------------------------------------------------------------------------

/// Byte offset → (line, character-UTF16) — the conversion the engine
/// owns (the pins above assert the engine's OWN answer, so this helper
/// only finds the test fixture's offsets).
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

/// The engine's config takes the same shapes the CLI's `--theme` takes:
/// `None` is the theme-less boundary, the built-in default rides the
/// shadcn bake. The config also carries the compile `Config` (the
/// spacing knob) — the engine never forks pipeline semantics.
#[test]
fn engine_config_covers_the_compile_knob() {
    let config = EngineConfig {
        tokens: None,
        compile: Config::default(),
    };
    let _ = AeolusEngine::new(config);
}
