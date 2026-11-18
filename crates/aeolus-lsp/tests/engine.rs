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
use aeolus_lsp::{
    AeolusEngine, EngineCompletion, EngineCompletionKind, EngineConfig, EngineSemanticKind,
    ThemeError,
};
use aeolus_theme::{ThemeSource, TokenTable};

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
// The config: the default's typed boundary
// ---------------------------------------------------------------------------

#[test]
fn try_default_answers_the_baked_shape() {
    // The fallible constructor rides the same bake `Default` panics on
    // (the two-lane contract the CLI's own theme loading keeps): the
    // typed path answers the themed engine's shape, byte-identically.
    let typed = EngineConfig::try_default().expect("the built-in shadcn bake parses and bakes");
    assert!(typed.tokens.is_some(), "the bake's table rides");
    let mut typed_engine = AeolusEngine::new(typed);
    let mut default_engine = engine();
    typed_engine.open("file:///card.rs", DOC, 1);
    default_engine.open("file:///card.rs", DOC, 1);
    let offset = DOC.find("\"p-4").expect("the fixture holds") + 2;
    let (line, character) = line_character_of(DOC, offset);
    assert_eq!(
        typed_engine.hover("file:///card.rs", line, character),
        default_engine.hover("file:///card.rs", line, character),
        "the typed default answers the panicking one's questions identically"
    );
}

#[test]
fn try_default_rides_the_reexported_error() {
    // The signature pin: the fallible constructor returns the theme
    // pass's OWN error, re-exported beside the config (the
    // no-manifest-add discipline — the host names it through
    // `aeolus_lsp`).
    let constructor: fn() -> Result<EngineConfig, ThemeError> = EngineConfig::try_default;
    assert!(constructor().is_ok(), "the built-in bake parses");
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
/// spacing knob) — the engine never forks pipeline semantics — and the
/// goto-definition face's origin URI (`None` the honest dark until a
/// host or the wire half names one).
#[test]
fn engine_config_covers_the_compile_knob() {
    let config = EngineConfig {
        tokens: None,
        compile: Config::default(),
        theme_origin: None,
    };
    let _ = AeolusEngine::new(config);
}
// ---------------------------------------------------------------------------
// The completion answers (the engine's third question: what could go
// here? — the candidate fragment under the cursor, completed from the
// SAME audited sources the compiler lowers against)
// ---------------------------------------------------------------------------

/// A synthetic .rs document carrying the three completion shapes: a
/// variant fragment, a utility fragment, an unknown root.
const COMPLETION_DOC: &str = "fn card() {\n    let a = \"hover:bg-pri\";\n    let b = \"flex-\";\n    let c = \"not-a-root\";\n}\n";

/// Byte offset of `needle`'s first occurrence, as (line, character).
fn position_after(doc: &str, needle: &str) -> (u32, u32) {
    let offset = doc.find(needle).expect("the fixture holds") + needle.len();
    position_of(doc, offset)
}

#[test]
fn a_variant_fragment_completes_from_the_closed_set() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", COMPLETION_DOC, 1);
    // The cursor right after `hov` — no ':' typed yet, so the variant
    // stack and the utility root are both in play.
    let (line, character) = position_after(COMPLETION_DOC, "hov");
    let items = lsp.complete("file:///card.rs", line, character);
    assert!(
        !items.is_empty(),
        "the fragment completes: {:?}",
        items.iter().map(|i| &i.label).collect::<Vec<_>>()
    );
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
    assert!(
        labels.contains(&"hover"),
        "the state spelling rides: {labels:?}"
    );
    for item in items.iter().filter(|i| i.label == "hover") {
        assert!(
            matches!(item.kind, EngineCompletionKind::Variant),
            "a variant rides the variant kind: {:?}",
            item.kind
        );
    }
}

#[test]
fn a_utility_fragment_completes_the_keyword_spellings() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", COMPLETION_DOC, 1);
    // The cursor right after `flex-` — the suffix half is empty, the
    // flex rows' keyword tables are the audited source.
    let (line, character) = position_after(COMPLETION_DOC, "flex-");
    let items = lsp.complete("file:///card.rs", line, character);
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
    assert!(
        labels.contains(&"flex-col") && labels.contains(&"flex-row"),
        "the flex-direction spellings ride: {labels:?}"
    );
    for item in items.iter().filter(|i| i.label == "flex-col") {
        assert!(
            matches!(item.kind, EngineCompletionKind::Utility),
            "a registry spelling rides the utility kind: {:?}",
            item.kind
        );
        assert!(
            item.detail.contains("flex-direction"),
            "the detail names the longhand the class lowers to: {}",
            item.detail
        );
    }
}

#[test]
fn a_utility_fragment_completes_the_theme_tokens() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", COMPLETION_DOC, 1);
    // The cursor right after `bg-pri` — the colour rows' token names
    // are the theme pass's own table (the default shadcn bake).
    let (line, character) = position_after(COMPLETION_DOC, "bg-pri");
    let items = lsp.complete("file:///card.rs", line, character);
    let hits: Vec<&EngineCompletion> = items
        .iter()
        .filter(|item| item.label == "bg-primary")
        .collect();
    assert_eq!(hits.len(), 1, "one bg-primary item, deduped");
    let item = hits[0];
    assert!(
        matches!(item.kind, EngineCompletionKind::Token),
        "a theme token rides the token kind: {:?}",
        item.kind
    );
    assert!(
        item.detail.contains("primary"),
        "the detail names the token: {}",
        item.detail
    );
}

#[test]
fn a_utility_fragment_completes_the_bare_roots() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", COMPLETION_DOC, 1);
    // `not-a-root` completes nothing — the unknown root is the honest
    // empty answer, never a guess.
    let (line, character) = position_after(COMPLETION_DOC, "not-a-root");
    let items = lsp.complete("file:///card.rs", line, character);
    assert!(
        items.is_empty(),
        "the unknown root completes nothing: {:?}",
        items.iter().map(|i| &i.label).collect::<Vec<_>>()
    );

    // A bare-root prefix completes the complete class (the root IS the
    // class — the display row's `hidden`). The fixture string carries a
    // dash token so the extraction seam's prose pre-filter (a string
    // with no `-`/`:` token is not a class string) keeps it — the same
    // rule the diagnostics keep, the completion firing where the
    // pipeline looks.
    let mut lsp2 = engine();
    let doc = "fn ui() {\n    let s = \"hid flex-col\";\n}\n";
    lsp2.open("file:///ui.rs", doc, 1);
    let hid_offset = doc.find("hid").expect("the fixture holds") + 3;
    let (line, character) = position_of(doc, hid_offset);
    let items = lsp2.complete("file:///ui.rs", line, character);
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
    assert!(
        labels.contains(&"hidden"),
        "the bare root completes: {labels:?}"
    );
    assert!(
        !labels.contains(&"flex-col"),
        "the fragment is `hid` — the row sibling does not ride: {labels:?}"
    );
}

#[test]
fn after_a_variant_stack_the_bare_roots_answer() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", COMPLETION_DOC, 1);
    // The cursor right after `hover:` — the utility half is empty; the
    // complete classes that need no suffix are the bounded answer.
    let (line, character) = position_after(COMPLETION_DOC, "hover:");
    let items = lsp.complete("file:///card.rs", line, character);
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
    assert!(
        labels.contains(&"hidden") && labels.contains(&"block"),
        "the bare utilities ride: {labels:?}"
    );
    // The bounded answer is the BARE roots only: no keyword-suffix
    // spelling, no token class (the utility half is empty — the suffix
    // sources have nothing to filter).
    assert!(
        !labels.contains(&"flex-col") && !labels.contains(&"bg-primary"),
        "the empty utility half answers complete classes only: {labels:?}"
    );
}

#[test]
fn completion_ranks_and_dedupes_deterministically() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", COMPLETION_DOC, 1);
    let (line, character) = position_after(COMPLETION_DOC, "flex-");
    let first = lsp.complete("file:///card.rs", line, character);
    let second = lsp.complete("file:///card.rs", line, character);
    assert_eq!(first, second, "complete is pure over its document map");
    // The registry order IS the rank order: non-decreasing, no repeats
    // (the editor's popup renders it verbatim).
    let ranks: Vec<usize> = first.iter().map(|i| i.rank).collect();
    assert!(
        ranks.windows(2).all(|pair| pair[0] < pair[1]),
        "ranks strictly increase along the answer: {ranks:?}"
    );
    let labels: std::collections::HashSet<&str> = first.iter().map(|i| i.label.as_str()).collect();
    assert_eq!(
        labels.len(),
        first.len(),
        "no duplicate labels: {:?}",
        first.iter().map(|i| &i.label).collect::<Vec<_>>()
    );
}

#[test]
fn the_themeless_config_completes_no_tokens() {
    // The theme-less boundary: the token source is the config's own
    // table — `None` answers registry spellings only, never a guessed
    // token name.
    let mut lsp = AeolusEngine::new(EngineConfig::new(None));
    lsp.open("file:///card.rs", COMPLETION_DOC, 1);
    let (line, character) = position_after(COMPLETION_DOC, "bg-pri");
    let items = lsp.complete("file:///card.rs", line, character);
    assert!(
        items
            .iter()
            .all(|i| !matches!(i.kind, EngineCompletionKind::Token)),
        "no token rides the theme-less config: {:?}",
        items.iter().map(|i| &i.label).collect::<Vec<_>>()
    );
    let registry_only: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
    assert!(
        registry_only.contains(&"bg-primary"),
        "the registry's own keyword spellings still ride: {registry_only:?}"
    );
    for item in items.iter().filter(|item| item.label == "bg-primary") {
        assert!(
            matches!(item.kind, EngineCompletionKind::Utility),
            "the themeless table renders the keyword half: {:?}",
            item.kind
        );
    }
}

#[test]
fn completion_outside_any_string_is_empty() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", COMPLETION_DOC, 1);
    // Line 0 is Rust code — no candidate string under the cursor.
    let items = lsp.complete("file:///card.rs", 0, 3);
    assert!(items.is_empty(), "no string, no completions");
    // An unknown URI answers empty too (the lifecycle contract).
    let (line, character) = position_after(COMPLETION_DOC, "flex-");
    assert!(
        lsp.complete("file:///never-opened.rs", line, character)
            .is_empty(),
        "an unknown URI answers empty"
    );
}
// ---------------------------------------------------------------------------
// The colour-swatch answers (the engine's fourth question: what does
// the editor paint here? — textDocument/documentColor, the per-candidate
// compile walk resolving the IR's three table-resolvable colour slots)
// ---------------------------------------------------------------------------

const COLOUR_DOC: &str = "fn card() {\n    let a = \"bg-primary\";\n    let b = \"p-4 text-secondary\";\n    let c = \"bg-transparent\";\n}\n";

#[test]
fn colour_swatches_resolve_the_token_slots() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", COLOUR_DOC, 1);
    let colours = lsp.document_colours("file:///card.rs");
    // The two token classes answer; `p-4` (no colour) and the keyword
    // slot (`bg-transparent` — a CSS keyword, not a swatch) stay quiet.
    assert_eq!(colours.len(), 2, "{colours:?}");
    for colour in &colours {
        assert_eq!(colour.rgba[3], 255, "the light tier's swatch is opaque");
    }
    // The two token classes name the SAME underlying token pair — the
    // swatches must agree (the table's own consistency).
    let primary = colours.iter().find(|c| {
        c.range.start.line == line_of(COLOUR_DOC, COLOUR_DOC.find("bg-primary").unwrap())
    });
    let secondary = colours.iter().find(|c| {
        c.range.start.line == line_of(COLOUR_DOC, COLOUR_DOC.find("text-secondary").unwrap())
    });
    let (Some(primary), Some(secondary)) = (primary, secondary) else {
        panic!("both token classes answer: {colours:?}");
    };
    assert_ne!(
        primary.rgba, secondary.rgba,
        "distinct tokens resolve distinct swatches"
    );
}

#[test]
fn colour_swatch_ranges_are_the_class_spans() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", COLOUR_DOC, 1);
    let colours = lsp.document_colours("file:///card.rs");
    let start = COLOUR_DOC.find("bg-primary").expect("the fixture holds");
    let (line, character) = position_of(COLOUR_DOC, start);
    let (end_line, end_character) = position_of(COLOUR_DOC, start + "bg-primary".len());
    let primary = colours
        .iter()
        .find(|c| c.range.start.line == line)
        .expect("the primary swatch rides");
    assert_eq!(primary.range.start.line, line);
    assert_eq!(primary.range.start.character, character);
    assert_eq!(primary.range.end.line, end_line);
    assert_eq!(primary.range.end.character, end_character);
}

#[test]
fn colour_swatches_agree_across_the_roots() {
    // The SAME token on two roots — one swatch: the table resolves it,
    // the walk does not re-derive it per root.
    let mut lsp = engine();
    let doc = "fn ui() {\n    let s = \"bg-primary border-primary\";\n}\n";
    lsp.open("file:///ui.rs", doc, 1);
    let colours = lsp.document_colours("file:///ui.rs");
    assert_eq!(colours.len(), 2, "{colours:?}");
    assert_eq!(
        colours[0].rgba, colours[1].rgba,
        "one token, one swatch: {colours:?}"
    );
}

#[test]
fn colour_swatches_are_deterministic_and_pure() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", COLOUR_DOC, 1);
    let first = lsp.document_colours("file:///card.rs");
    let second = lsp.document_colours("file:///card.rs");
    assert_eq!(first, second, "document_colours is pure over its map");
}

#[test]
fn colour_swatches_answer_empty_outside_the_pipeline() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", COLOUR_DOC, 1);
    assert!(
        lsp.document_colours("file:///never-opened.rs").is_empty(),
        "an unknown URI answers empty"
    );
    let plain = "fn clean() {}\n";
    lsp.open("file:///clean.rs", plain, 1);
    assert!(
        lsp.document_colours("file:///clean.rs").is_empty(),
        "a document with no strings answers empty"
    );
}

// ---------------------------------------------------------------------------
// The pull-diagnostics answer (the documentDiagnostic wire's engine
// half): `analyze` IS the pull report's source — the same walk the
// push channel publishes. The battery pins the answer's SHAPE the wire
// maps: one coded finding per miss, the range byte-true.
// ---------------------------------------------------------------------------

#[test]
fn analyze_is_the_pull_diagnostics_answer() {
    let mut engine = AeolusEngine::new(EngineConfig::default());
    engine.open(
        "file:///pull.rs",
        "fn card() {\n    let s = \"p-4 not-a-root\";\n}\n",
        1,
    );
    let findings = engine.analyze("file:///pull.rs");
    assert_eq!(findings.len(), 1, "the one miss answers: {findings:?}");
    let finding = &findings[0];
    assert!(
        finding.code.starts_with("aeolus::compile::"),
        "the stable machine code rides: {}",
        finding.code
    );
    assert!(
        finding.message.contains("not-a-root"),
        "the message names the token: {}",
        finding.message
    );
    assert_eq!(finding.range.start.line, 1, "the range is byte-true");
    // The clean document answers empty — the pull's empty report.
    engine.open("file:///clean.rs", "fn clean() {}\n", 1);
    assert!(
        engine.analyze("file:///clean.rs").is_empty(),
        "a clean document pulls an empty report"
    );
}

// ---------------------------------------------------------------------------
// The semantic-token answer (the engine's fifth question: what is this
// text to the editor's highlighter? — textDocument/semanticTokens/full,
// the per-candidate compile walk classifying every candidate's outcome
// onto the wire's legend)
// ---------------------------------------------------------------------------

#[test]
fn semantic_tokens_classify_every_candidate() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    let tokens = lsp.semantic_tokens("file:///card.rs");
    // Three candidates, three tokens: `p-4` and `flex-col` lower, the
    // unknown root refuses — the classification IS the compile walk's
    // own outcome, never a guess.
    assert_eq!(tokens.len(), 3, "one token per candidate: {tokens:?}");
    let lowered = tokens
        .iter()
        .filter(|t| t.kind == EngineSemanticKind::Lowered)
        .count();
    let refused = tokens
        .iter()
        .filter(|t| t.kind == EngineSemanticKind::Refused)
        .count();
    assert_eq!(
        (lowered, refused),
        (2, 1),
        "the lowered candidates highlight, the miss refuses: {tokens:?}"
    );
    // The refused token's range IS the miss's own span, byte-true (the
    // diagnostics' own range shape, one conversion).
    let start = DOC.find("not-a-root").expect("the fixture holds");
    let (line, character) = position_of(DOC, start);
    let (end_line, end_character) = position_of(DOC, start + "not-a-root".len());
    let miss = tokens
        .iter()
        .find(|t| t.kind == EngineSemanticKind::Refused)
        .expect("the miss rides");
    assert_eq!(miss.range.start.line, line);
    assert_eq!(miss.range.start.character, character);
    assert_eq!(miss.range.end.line, end_line);
    assert_eq!(miss.range.end.character, end_character);
    // A lowered token spans its own candidate (`p-4`, line 1).
    let p4 = DOC.find("p-4").expect("the fixture holds");
    let (p4_line, p4_character) = position_of(DOC, p4);
    let (_, p4_end) = position_of(DOC, p4 + "p-4".len());
    let lowered_token = tokens
        .iter()
        .find(|t| t.kind == EngineSemanticKind::Lowered)
        .expect("the lowered class rides");
    assert_eq!(lowered_token.range.start.line, p4_line);
    assert_eq!(lowered_token.range.start.character, p4_character);
    assert_eq!(lowered_token.range.end.character, p4_end);
}

#[test]
fn semantic_tokens_answer_in_document_order() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    let tokens = lsp.semantic_tokens("file:///card.rs");
    // The scan order IS the answer's order: ascending positions, the
    // delta encoder's own precondition (the wire test pins the deltas).
    let ascending = tokens.windows(2).all(|pair| {
        (pair[0].range.start.line, pair[0].range.start.character)
            < (pair[1].range.start.line, pair[1].range.start.character)
    });
    assert!(ascending, "the tokens answer in document order: {tokens:?}");
}

#[test]
fn semantic_tokens_are_pure_and_empty_at_the_boundaries() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    let first = lsp.semantic_tokens("file:///card.rs");
    let second = lsp.semantic_tokens("file:///card.rs");
    assert_eq!(first, second, "semantic_tokens is pure over its map");
    assert!(
        lsp.semantic_tokens("file:///never-opened.rs").is_empty(),
        "an unknown URI answers empty"
    );
    lsp.open("file:///clean.rs", "fn clean() {}\n", 1);
    assert!(
        lsp.semantic_tokens("file:///clean.rs").is_empty(),
        "a document with no strings answers empty"
    );
    // A candidate that is not a class at all (a plain prose string
    // without a `-`/`:` token) never reaches the walk — the extraction
    // seam's own pre-filter, the same rule the diagnostics keep.
    lsp.open(
        "file:///prose.rs",
        "fn p() { let s = \"some prose\"; }\n",
        1,
    );
    assert!(
        lsp.semantic_tokens("file:///prose.rs").is_empty(),
        "the prose string stays out of the highlighter"
    );
}

// ---------------------------------------------------------------------------
// The colour-presentation answer (the engine's sixth question: how would
// the editor SPELL this colour? — textDocument/colorPresentation, the
// request the colorProvider capability already advertises beside the
// swatches, answered by the config's own table walked backwards)
// ---------------------------------------------------------------------------

#[test]
fn colour_presentations_spell_the_resolved_tokens() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", COLOUR_DOC, 1);
    // The differential round trip: the swatch the walk answers is a
    // colour the spellings answer back — one table, both directions.
    let swatches = lsp.document_colours("file:///card.rs");
    assert!(!swatches.is_empty(), "the fixture carries swatches");
    let rgba = swatches[0].rgba;
    let presentations = lsp.colour_presentations(rgba);
    assert!(
        presentations
            .iter()
            .any(|p| p.label == "bg-primary" && p.token == "primary" && p.alpha == 100),
        "the opaque token spelling rides: {presentations:?}"
    );
    // The colour-group surface is the completion source's own (one
    // walk, all roots): the same token spells on every colour root.
    assert!(
        presentations.iter().any(|p| p.label == "text-primary"),
        "the text root rides the same token: {presentations:?}"
    );
    for presentation in &presentations {
        assert_eq!(
            presentation.alpha, 100,
            "the opaque colour carries no modifier: {presentations:?}"
        );
        assert!(
            !presentation.label.starts_with('-')
                && !presentation.label.contains('/')
                && presentation.label.contains(presentation.token.as_str()),
            "the label spells a root over the token, no modifier: {}",
            presentation.label
        );
    }
}

#[test]
fn colour_presentations_spell_the_alpha_steps() {
    // The differential round trip at an alpha step: the table's own
    // resolve at /50 is a colour the spellings answer back — the same
    // premultiplied identity both directions.
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::baked(&source).expect("the baked shadcn table builds");
    let id = table
        .id("primary")
        .expect("the shadcn bake carries primary");
    let rgba = table
        .resolve_rgba(id, 0.5, false)
        .expect("the tier resolves");
    let lsp = AeolusEngine::new(EngineConfig::default());
    let presentations = lsp.colour_presentations(rgba);
    assert!(
        presentations
            .iter()
            .any(|p| p.label == "bg-primary/50" && p.token == "primary" && p.alpha == 50),
        "the /50 spelling rides: {presentations:?}"
    );
    assert!(
        presentations.iter().all(|p| p.alpha != 100),
        "an alpha'd colour never spells bare: {presentations:?}"
    );
}

#[test]
fn colour_presentations_answer_empty_at_the_boundaries() {
    // The theme-less boundary: no table, no spellings (the same honest
    // answer the completion keeps).
    let themeless = AeolusEngine::new(EngineConfig::new(None));
    assert!(
        themeless
            .colour_presentations([93, 90, 208, 255])
            .is_empty(),
        "no table, no presentations"
    );
    // A colour no token resolves: the honest empty answer, never a
    // guessed spelling.
    let lsp = AeolusEngine::new(EngineConfig::default());
    assert!(
        lsp.colour_presentations([1, 2, 3, 4]).is_empty(),
        "an unmatched colour answers empty"
    );
}

#[test]
fn colour_presentations_are_pure_and_deduped() {
    let lsp = AeolusEngine::new(EngineConfig::default());
    let first = lsp.colour_presentations([93, 90, 208, 255]);
    let second = lsp.colour_presentations([93, 90, 208, 255]);
    assert_eq!(
        first, second,
        "colour_presentations is pure over its config"
    );
    let labels: std::collections::HashSet<&str> = first.iter().map(|p| p.label.as_str()).collect();
    assert_eq!(
        labels.len(),
        first.len(),
        "no duplicate spellings: {:?}",
        first.iter().map(|p| &p.label).collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// The inlay-hint answer (the engine's seventh question: what does this
// class RESOLVE to, inline? — textDocument/inlayHint, the per-candidate
// compile walk rendering the CSS arm's own declarations compactly at the
// class's end, the refused candidates staying quiet — the diagnostics'
// and semantic tokens' story)
// ---------------------------------------------------------------------------

#[test]
fn inlay_hints_render_the_lowered_css() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    let hints = lsp.inlay_hints("file:///card.rs");
    // Two hints for two lowered classes; the refused candidate never
    // hints (its story is the diagnostics' and the legend's).
    assert_eq!(hints.len(), 2, "{hints:?}");
    // `p-4`: the hint rides at the class's END (byte-true, UTF-16), the
    // label is the CSS arm's own declaration, single-line, and the
    // tooltip carries the hover's markdown (the class, the CSS block).
    let p4_start = DOC.find("\"p-4 flex-col\"").expect("the fixture holds") + 1;
    let (p4_line, _) = position_of(DOC, p4_start);
    let (_, p4_end_character) = position_of(DOC, p4_start + "p-4".len());
    let p4 = hints
        .iter()
        .find(|hint| hint.position.line == p4_line)
        .expect("the p-4 hint rides");
    assert_eq!(
        p4.position.character, p4_end_character,
        "the hint rides at the class's end"
    );
    assert!(
        p4.label.contains("padding"),
        "the CSS arm's own declaration renders: {}",
        p4.label
    );
    assert!(
        !p4.label.contains('\n'),
        "the label is the wire's single-line form: {}",
        p4.label
    );
    assert!(
        p4.tooltip.contains("p-4") && p4.tooltip.contains("```css"),
        "the tooltip is the hover's markdown: {}",
        p4.tooltip
    );
    // `flex-col`: the sibling class hints its own longhand.
    let flex_start = DOC.find("flex-col").expect("the fixture holds");
    let (_, flex_end_character) = position_of(DOC, flex_start + "flex-col".len());
    let flex = hints
        .iter()
        .find(|hint| hint.position != p4.position)
        .expect("the flex-col hint rides");
    assert_eq!(flex.position.character, flex_end_character);
    assert!(
        flex.label.contains("flex-direction"),
        "the longhand renders: {}",
        flex.label
    );
    // The p-4 hint's own position is the first hint's (not the flex
    // one's) — the find above keys on the line, the pairs stay distinct.
    assert_ne!(
        p4.position, flex.position,
        "one hint per class, at its own end"
    );
}

#[test]
fn inlay_hints_resolve_the_theme_tokens() {
    // The themed walk (the default shadcn bake): a token class's hint
    // renders the baked tier's resolved bytes, the CSS arm's own form.
    let mut lsp = engine();
    let doc = "fn ui() {\n    let s = \"bg-primary\";\n}\n";
    lsp.open("file:///ui.rs", doc, 1);
    let hints = lsp.inlay_hints("file:///ui.rs");
    assert_eq!(hints.len(), 1, "{hints:?}");
    let hint = &hints[0];
    assert!(
        hint.label.contains("background-color"),
        "the longhand renders: {}",
        hint.label
    );
    assert!(
        hint.label.contains("rgb("),
        "the baked tier's resolved bytes render: {}",
        hint.label
    );
    let start = doc.find("bg-primary").expect("the fixture holds");
    let (_, end_character) = position_of(doc, start + "bg-primary".len());
    assert_eq!(
        hint.position.character, end_character,
        "the hint rides at the class's end"
    );
}

#[test]
fn inlay_hints_are_pure_and_empty_at_the_boundaries() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    let first = lsp.inlay_hints("file:///card.rs");
    let second = lsp.inlay_hints("file:///card.rs");
    assert_eq!(first, second, "inlay_hints is pure over its map");
    assert!(
        lsp.inlay_hints("file:///never-opened.rs").is_empty(),
        "an unknown URI answers empty"
    );
    lsp.open("file:///clean.rs", "fn clean() {}\n", 1);
    assert!(
        lsp.inlay_hints("file:///clean.rs").is_empty(),
        "a document with no strings answers empty"
    );
    // A prose string never reaches the walk (the extraction seam's own
    // pre-filter), and a refused-only document answers empty — the
    // refusal's story is the diagnostics', never a guessed hint.
    lsp.open(
        "file:///prose.rs",
        "fn p() { let s = \"some prose\"; }\n",
        1,
    );
    assert!(
        lsp.inlay_hints("file:///prose.rs").is_empty(),
        "the prose string stays out of the hints"
    );
    lsp.open("file:///bad.rs", "fn b() { let s = \"not-a-root\"; }\n", 1);
    assert!(
        lsp.inlay_hints("file:///bad.rs").is_empty(),
        "a refused class never hints"
    );
    // The theme-less boundary: the lowered class's CSS arm rendering
    // rides either way — the hint is the pipeline's own answer, not the
    // theme tier's alone.
    let mut themeless = AeolusEngine::new(EngineConfig::new(None));
    themeless.open("file:///t.rs", "fn t() { let s = \"p-4\"; }\n", 1);
    let hints = themeless.inlay_hints("file:///t.rs");
    assert_eq!(hints.len(), 1, "{hints:?}");
    assert!(
        hints[0].label.contains("padding"),
        "the theme-less walk still renders the declaration: {}",
        hints[0].label
    );
}

// ---------------------------------------------------------------------------
// The code-action answer (the quick fixes, the suggestion seam's editor
// face): every action repairs one finding with a replacement the
// pipeline LOWERS — the audited tables are the sources, never a guessed
// word.
// ---------------------------------------------------------------------------

/// The document the quick-fix battery rides: one unknown root, one
/// keyword-table miss, one variant miss — the three folds the seam
/// answers.
const ACTIONS_DOC: &str = "fn card() {\n    let a = \"not-a-root\";\n    let b = \"flex-clo\";\n    let c = \"havor:p-4\";\n}\n";

#[test]
fn code_actions_answer_the_cursor_s_findings() {
    let mut lsp = engine();
    lsp.open("file:///actions.rs", ACTIONS_DOC, 1);
    // The cursor inside the keyword miss: the fold's registry answer,
    // the finding riding beside the edit.
    let offset = ACTIONS_DOC.find("flex-clo").expect("the fixture holds") + 2;
    let (line, character) = line_character_of(ACTIONS_DOC, offset);
    let actions = lsp.code_actions("file:///actions.rs", line, character);
    assert!(
        actions
            .iter()
            .any(|action| action.replacement == "flex-col"
                && action.title == "did you mean `flex-col`?"),
        "the registry sibling answers: {actions:?}"
    );
    for action in &actions {
        assert_eq!(
            action.diagnostic.range.start.line, line,
            "the finding rides the cursor's own line"
        );
        assert!(
            action.diagnostic.message.contains("help:"),
            "the finding carries the rendered help"
        );
    }
}

#[test]
fn code_actions_compose_the_variant_prefix_back_on() {
    let mut lsp = engine();
    lsp.open("file:///actions.rs", ACTIONS_DOC, 1);
    let offset = ACTIONS_DOC.find("havor:p-4").expect("the fixture holds") + 2;
    let (line, character) = line_character_of(ACTIONS_DOC, offset);
    let actions = lsp.code_actions("file:///actions.rs", line, character);
    assert!(
        actions
            .iter()
            .any(|action| action.replacement == "hover:p-4"),
        "the closed variant set answers with the utility half intact: {actions:?}"
    );
}

#[test]
fn code_actions_outside_a_candidate_answer_empty() {
    let mut lsp = engine();
    lsp.open("file:///actions.rs", ACTIONS_DOC, 1);
    // The cursor on the `fn` keyword (no string, no candidate).
    let actions = lsp.code_actions("file:///actions.rs", 0, 2);
    assert!(actions.is_empty(), "outside any candidate: {actions:?}");
    // The unknown URI: the lifecycle's own empty answer.
    assert!(lsp.code_actions("file:///never-opened.rs", 0, 0).is_empty());
}

// ---------------------------------------------------------------------------
// The document-highlight answer (the click-to-find occurrences): the
// TEXTUAL equality the wire contract rides, document order, the
// cursor's own occurrence included.
// ---------------------------------------------------------------------------

#[test]
fn document_highlights_answer_every_occurrence() {
    // The same class twice as its own token (the second beside a
    // refused neighbour), and once under a variant prefix — a DIFFERENT
    // raw text, no highlight.
    const DOC_HL: &str = "fn h() {\n    let a = \"flex-col\";\n    let b = \"hover:flex-col\";\n    let c = \"not-a-root flex-col\";\n}\n";
    let mut lsp = engine();
    lsp.open("file:///highlights.rs", DOC_HL, 1);
    let offset = DOC_HL.find("flex-col").expect("the fixture holds") + 2;
    let (line, character) = line_character_of(DOC_HL, offset);
    let highlights = lsp.document_highlights("file:///highlights.rs", line, character);
    assert_eq!(
        highlights.len(),
        2,
        "the two exact occurrences (the hover:-prefixed one is a different token): {highlights:?}"
    );
    // Document order, first hit first.
    assert!(
        highlights[0].range.start.line < highlights[1].range.start.line,
        "the answer rides the scan order: {highlights:?}"
    );
}

#[test]
fn document_highlights_are_textual_not_semantic() {
    let mut lsp = engine();
    lsp.open(
        "file:///bad.rs",
        "fn b() { let s = \"not-a-root not-a-root\"; }\n",
        1,
    );
    let offset = 22; // inside the first occurrence
    let (line, character) =
        line_character_of("fn b() { let s = \"not-a-root not-a-root\"; }\n", offset);
    let highlights = lsp.document_highlights("file:///bad.rs", line, character);
    assert_eq!(
        highlights.len(),
        2,
        "a refused class's occurrences highlight like a lowered one's: {highlights:?}"
    );
}

#[test]
fn document_highlights_outside_a_candidate_answer_empty() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    assert!(
        lsp.document_highlights("file:///card.rs", 0, 2).is_empty(),
        "the cursor on `fn` answers empty"
    );
    assert!(
        lsp.document_highlights("file:///never-opened.rs", 0, 0)
            .is_empty(),
        "the unknown URI answers empty"
    );
}

// ---------------------------------------------------------------------------
// The formatting answer (the class-sort seam's editor face):
// textDocument/formatting — the registry's canonical order, one edit per
// out-of-order class string
// ---------------------------------------------------------------------------

/// The formatter's document: one multi-byte identifier before the
/// out-of-order string (the UTF-16 column fixture), one already-sorted
/// string, one unknown-root string (its single class is trivially in
/// order — the formatter stays quiet).
const FMT_DOC: &str = "fn card() {\n    let héllo = \"hover:flex-col p-4 flex-col\";\n    let clean = \"p-4 flex-col\";\n    let bad = \"not-a-root\";\n}\n";

/// The canonical form the sort seam answers for [`FMT_DOC`]'s first
/// string (pinned against `aeolus sort`: the rows' progression rides
/// before the variant bucket).
const FMT_CANONICAL: &str = "p-4 flex-col hover:flex-col";

#[test]
fn format_answers_the_canonical_order() {
    let mut lsp = engine();
    lsp.open("file:///fmt.rs", FMT_DOC, 1);
    let edits = lsp.format("file:///fmt.rs");
    assert_eq!(
        edits.len(),
        1,
        "one out-of-order string, one edit — the clean and refused strings stay quiet: {edits:?}"
    );
    let edit = &edits[0];
    assert_eq!(edit.new_text, FMT_CANONICAL);
    // The edit's range is the STRING CONTENT's span (the extractor's
    // own trimmed hit, byte-true), never the quotes around it.
    let content_start = FMT_DOC.find("hover:flex-col").expect("the fixture holds");
    let content_end = content_start + "hover:flex-col p-4 flex-col".len();
    let (line, character) = line_character_of(FMT_DOC, content_start);
    let (end_line, end_character) = line_character_of(FMT_DOC, content_end);
    assert_eq!(edit.range.start.line, line);
    assert_eq!(edit.range.start.character, character);
    assert_eq!(edit.range.end.line, end_line);
    assert_eq!(edit.range.end.character, end_character);
}

#[test]
fn format_is_byte_true_over_the_multi_byte_column() {
    // `é` is two UTF-8 bytes but ONE UTF-16 unit: the edit's start
    // column counts code units, never bytes — the range must land on
    // the first class character, not inside the identifier.
    let mut lsp = engine();
    lsp.open("file:///fmt.rs", FMT_DOC, 1);
    let edit = &lsp.format("file:///fmt.rs")[0];
    let content_start = FMT_DOC.find("hover:flex-col").expect("the fixture holds");
    let (_, character) = line_character_of(FMT_DOC, content_start);
    let line_start = FMT_DOC[..content_start].rfind('\n').map_or(0, |i| i + 1);
    let byte_column = content_start - line_start;
    assert_ne!(
        byte_column, character as usize,
        "the fixture must be multi-byte (é shifts the byte column off the UTF-16 column)"
    );
    assert_eq!(
        edit.range.start.character, character,
        "the UTF-16 column, byte-true"
    );
}

#[test]
fn format_noops_an_already_ordered_document() {
    let mut lsp = engine();
    // Every string already rides the canonical order (and the
    // single-class unknown root is trivially in order).
    lsp.open("file:///fmt.rs", FMT_DOC, 1);
    let edits = lsp.format("file:///fmt.rs");
    assert_eq!(edits.len(), 1, "only the out-of-order string edits");
    // A document with NO class strings at all: the honest empty.
    lsp.change("file:///fmt.rs", "fn clean() {}\n", 2);
    assert!(
        lsp.format("file:///fmt.rs").is_empty(),
        "a document with no strings answers empty"
    );
}

#[test]
fn format_normalizes_the_whitespace_run() {
    // The canonical form is the sort seam's own join: the input's
    // whitespace run does not survive it (the seam's documented
    // formatter contract).
    let mut lsp = engine();
    lsp.open(
        "file:///ws.rs",
        "fn card() {\n    let s = \"p-4  flex-col\";\n}\n",
        1,
    );
    let edits = lsp.format("file:///ws.rs");
    assert_eq!(edits.len(), 1, "the whitespace run alone is an edit");
    assert_eq!(edits[0].new_text, "p-4 flex-col");
}

#[test]
fn format_known_classes_reorder_the_tail_rides_source_order() {
    // The honest tail: classes the seam cannot order never block the
    // formatter — the known prefix reorders, the tail rides its source
    // order after it (the sort seam's own contract), and the MISS
    // stays the diagnostics channels' story (the formatter's answer
    // carries edits, never findings).
    let mut lsp = engine();
    let doc = "fn card() {\n    let s = \"justify-betwen p-4 not-a-class\";\n}\n";
    lsp.open("file:///tail.rs", doc, 1);
    let edits = lsp.format("file:///tail.rs");
    assert_eq!(edits.len(), 1, "the known class reorders: {edits:?}");
    assert_eq!(edits[0].new_text, "p-4 justify-betwen not-a-class");
}

#[test]
fn format_is_pure_and_gates_on_the_document_map() {
    let mut lsp = engine();
    lsp.open("file:///fmt.rs", FMT_DOC, 1);
    let first = lsp.format("file:///fmt.rs");
    let second = lsp.format("file:///fmt.rs");
    assert_eq!(first, second, "format is pure over its document map");
    assert!(
        lsp.format("file:///never-opened.rs").is_empty(),
        "the unknown URI answers empty"
    );
    lsp.close("file:///fmt.rs");
    assert!(
        lsp.format("file:///fmt.rs").is_empty(),
        "a closed document answers empty"
    );
}

// ---------------------------------------------------------------------------
// The rename face (the wire's prepareRename + rename): the candidate
// span under the cursor, the textual occurrences' edits
// ---------------------------------------------------------------------------

#[test]
fn prepare_rename_answers_the_candidates_span() {
    // The cursor inside a candidate answers its byte-true range (the
    // full spelling — the highlights' own raw-text contract).
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    let start = DOC.find("p-4").expect("the fixture holds");
    let (line, character) = line_character_of(DOC, start + 1);
    let range = lsp
        .prepare_rename("file:///card.rs", line, character)
        .expect("the cursor inside a candidate answers the span");
    let (start_line, start_character) = line_character_of(DOC, start);
    let (end_line, end_character) = line_character_of(DOC, start + "p-4".len());
    assert_eq!(range.start.line, start_line, "{range:?}");
    assert_eq!(range.start.character, start_character, "{range:?}");
    assert_eq!(range.end.line, end_line, "{range:?}");
    assert_eq!(range.end.character, end_character, "{range:?}");
}

#[test]
fn prepare_rename_outside_a_candidate_answers_none() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    assert!(
        lsp.prepare_rename("file:///card.rs", 0, 2).is_none(),
        "the cursor on `fn` answers none (the protocol's cannot-rename)"
    );
    assert!(
        lsp.prepare_rename("file:///never-opened.rs", 0, 0)
            .is_none(),
        "the unknown URI answers none"
    );
}

#[test]
fn rename_edits_every_textual_occurrence() {
    // One edit per occurrence of the cursor's candidate (the
    // references' own fold, the workspace upgrade), the new spelling
    // riding every edit beside the carrying document's URI.
    const DOC_RN: &str =
        "fn r() {\n    let a = \"flex-col\";\n    let b = \"not-a-root flex-col\";\n}\n";
    let mut lsp = engine();
    lsp.open("file:///rename.rs", DOC_RN, 1);
    let offset = DOC_RN.find("flex-col").expect("the fixture holds") + 2;
    let (line, character) = line_character_of(DOC_RN, offset);
    let edits = lsp.rename("file:///rename.rs", line, character, "flex-row");
    assert_eq!(edits.len(), 2, "one edit per textual occurrence: {edits:?}");
    assert!(
        edits.iter().all(|edit| edit.new_text == "flex-row"),
        "every occurrence rides the new spelling: {edits:?}"
    );
    assert!(
        edits.iter().all(|edit| edit.uri == "file:///rename.rs"),
        "every edit rides its document's URI: {edits:?}"
    );
}

#[test]
fn rename_under_a_variant_replaces_the_whole_spelling() {
    // The candidate span IS the whole spelling: a cursor inside a
    // variant tail renames the full token (`hover:flex-col` is one
    // raw text, the highlights' own equality).
    const DOC_RN: &str = "fn r() {\n    let b = \"hover:flex-col\";\n}\n";
    let mut lsp = engine();
    lsp.open("file:///variant.rs", DOC_RN, 1);
    let hit = DOC_RN.find("hover:flex-col").expect("the fixture holds");
    let offset = DOC_RN.find("flex-col").expect("the fixture holds") + 2;
    let (line, character) = line_character_of(DOC_RN, offset);
    let edits = lsp.rename("file:///variant.rs", line, character, "flex-row");
    assert_eq!(edits.len(), 1, "{edits:?}");
    let (start_line, start_character) = line_character_of(DOC_RN, hit);
    let (end_line, end_character) = line_character_of(DOC_RN, hit + "hover:flex-col".len());
    assert_eq!(edits[0].uri, "file:///variant.rs", "{edits:?}");
    assert_eq!(edits[0].range.start.line, start_line, "{edits:?}");
    assert_eq!(edits[0].range.start.character, start_character, "{edits:?}");
    assert_eq!(edits[0].range.end.line, end_line, "{edits:?}");
    assert_eq!(edits[0].range.end.character, end_character, "{edits:?}");
}

#[test]
fn rename_guards_the_empty_name_and_the_unknown_uri() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    let offset = DOC.find("p-4").expect("the fixture holds") + 1;
    let (line, character) = line_character_of(DOC, offset);
    assert!(
        lsp.rename("file:///card.rs", line, character, "")
            .is_empty(),
        "the empty name answers no edits (never a deletion walk)"
    );
    assert!(
        lsp.rename("file:///never-opened.rs", line, character, "px-8")
            .is_empty(),
        "the unknown URI answers empty"
    );
}

// ---------------------------------------------------------------------------
// The document-symbol battery: the outline face (the wire's
// `textDocument/documentSymbol`) — one symbol per candidate class the
// extraction seam's strings carry, the semantic legend's own verdict
// riding beside the name
// ---------------------------------------------------------------------------

#[test]
fn document_symbols_answer_one_per_candidate_in_document_order() {
    // The outline walks every candidate class the extraction seam's
    // strings carry — one symbol per candidate, document order (the
    // scan order), the spelling the NAME.
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    let symbols = lsp.document_symbols("file:///card.rs");
    let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["p-4", "flex-col", "not-a-root"], "{names:?}");
}

#[test]
fn document_symbols_carry_the_byte_true_ranges_and_the_verdict() {
    // The range is the candidate's own span (the same byte-true
    // translation the swatches and the legend ride — the multi-byte
    // identifier BEFORE the string shifts the UTF-16 column, never the
    // line), the verdict the semantic legend's own two answers.
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    let symbols = lsp.document_symbols("file:///card.rs");
    assert_eq!(symbols.len(), 3, "{symbols:?}");
    let p4 = &symbols[0];
    assert_eq!(p4.range.start.line, 1, "{p4:?}");
    assert!(p4.range.end.character > p4.range.start.character, "{p4:?}");
    assert!(matches!(p4.kind, EngineSemanticKind::Lowered), "{p4:?}");
    let refused = symbols.last().expect("the fixture carries the miss");
    assert_eq!(refused.name, "not-a-root");
    assert!(
        matches!(refused.kind, EngineSemanticKind::Refused),
        "{refused:?}"
    );
}

#[test]
fn document_symbols_render_the_lowered_css_and_the_refused_code() {
    // The detail line is the pipeline's own output: a lowered class
    // renders the compact CSS the inlay hint's label carries (never a
    // paraphrase), a refused one the stable code the diagnostics key
    // on.
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    let symbols = lsp.document_symbols("file:///card.rs");
    assert!(
        symbols[0].detail.contains("padding-top"),
        "the lowered detail is the CSS arm's own declarations: {:?}",
        symbols[0].detail
    );
    assert_eq!(symbols[2].detail, "aeolus::compile::unknown_root");
}

#[test]
fn document_symbols_are_pure_and_unknown_uris_answer_empty() {
    let mut lsp = engine();
    lsp.open("file:///card.rs", DOC, 1);
    let first = lsp.document_symbols("file:///card.rs");
    let second = lsp.document_symbols("file:///card.rs");
    assert_eq!(first, second, "document_symbols is pure over its map");
    assert!(
        lsp.document_symbols("file:///never-opened.rs").is_empty(),
        "the unknown URI answers empty"
    );
}

// ---------------------------------------------------------------------------
// The helpers
// ---------------------------------------------------------------------------

/// A byte offset's `(line, character)` in the engine's own units (the
/// harness-side twin of the conversions the engine owns — one call
/// site per fixture).
fn line_character_of(text: &str, offset: usize) -> (u32, u32) {
    let line = text[..offset].matches('\n').count() as u32;
    let line_start = text[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = text[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    (line, character)
}
