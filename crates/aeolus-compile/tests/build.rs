//! The project-build seam's contracts (architecture plan §2's approach
//! C — "whole-project scan + one generated artifact: scan sources, emit
//! one global artifact, dedupe globally" — and the P6 row's `cargo
//! aeolus build`), pinned end to end: the string-literal extraction
//! with its locations, the GLOBAL dedupe (one class string compiles
//! once no matter how many files say it), the registry-ordered rule
//! progression (the sort seam's own canonical order), the
//! self-contained theme-vars block (the `:root`/`.dark` half the
//! `var(--color-…)` references resolve against), the at-rule grouping,
//! the honest per-location diagnostics, and the differential pin
//! against the per-string `emit_css_theme` walk.
//!
//! Written FIRST (the RED spec): the suite fails until `scan`/`build`
//! exist.

use aeolus_compile::{Config, RustStrings, SourceFile, build, build_with, scan};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The runtime table over the shadcn light+dark palette (T2).
fn runtime_tokens() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("the embedded shadcn source parses");
    TokenTable::runtime(&source).expect("the pair surface fits the u16 slot")
}

/// Two "source files" the tests scan: one Rust view file, one HTML
/// template — the extractor is file-format-agnostic (a quoted string is
/// a quoted string), the shapes mirror a real consumer project.
const VIEW_RS: &str = "\
use aeolus_gpui::prelude::*;

fn card(classes: &str) -> Div {
    div().tw(tw!(\"flex flex-col gap-2 rounded-md bg-primary text-primary-foreground\"))
}

fn badge() -> Div {
    div().tw(tw!(\"inline-flex items-center rounded-full bg-secondary px-2 py-0.5\"))
        .tw(tw!(\"md:p-6 hover:bg-primary/90\"))
}
";

const TEMPLATE_HTML: &str = "\
<button class=\"inline-flex items-center rounded-full bg-secondary px-2 py-0.5\">
  Save
</button>
<p data-note=\"prose with a dash-separated phrase stays out\">
  body
</p>
";

fn files() -> Vec<SourceFile<'static>> {
    vec![
        SourceFile::new("src/view.rs", VIEW_RS),
        SourceFile::new("assets/template.html", TEMPLATE_HTML),
    ]
}

/// The byte offset of a needle inside a fixture, for the location pins.
fn offset_of(haystack: &str, needle: &str) -> usize {
    haystack
        .find(needle)
        .expect("needle present in the fixture")
}

fn hit_for<'a>(scanned: &'a aeolus_compile::Scanned, needle: &str) -> &'a aeolus_compile::Located {
    scanned
        .hits
        .iter()
        .find(|h| h.text.contains(needle))
        .unwrap_or_else(|| panic!("hit carrying {needle:?} present"))
}

// ---------------------------------------------------------------------------
// The extraction half
// ---------------------------------------------------------------------------

#[test]
fn scan_extracts_quoted_class_strings_with_locations() {
    let scanned = scan(&files(), &RustStrings);
    // One hit per candidate literal: the three tw! literals and the two
    // template strings (the prose data-note carries a dash, so the
    // pre-filter passes it — the compile walk is the precise filter,
    // exactly the CLI's own two-stage shape).
    assert_eq!(scanned.hits.len(), 5, "{:?}", scanned.hits);
    let first = hit_for(&scanned, "flex flex-col");
    assert_eq!(first.file, "src/view.rs");
    assert_eq!(
        first.text,
        "flex flex-col gap-2 rounded-md bg-primary text-primary-foreground"
    );
    assert_eq!(first.offset, offset_of(VIEW_RS, "flex flex-col"));
    // `    div().tw(tw!"` is 18 bytes into line 4; line/col point at the
    // literal's content, one-based.
    assert_eq!(first.line, 4);
    assert_eq!(first.col, 19);
}

#[test]
fn scan_locations_count_lines_and_columns_one_based() {
    let scanned = scan(&files(), &RustStrings);
    // The badge literal exists in BOTH files; the template's own copy
    // is the hit this pin reads. `<button class="` is 15 bytes into
    // line 1 of the template.
    let html = scanned
        .hits
        .iter()
        .find(|h| h.file == "assets/template.html" && h.text.contains("rounded-full"))
        .expect("the template's badge hit present");
    assert_eq!(html.line, 1);
    assert_eq!(html.col, 16);
    // The offset is within the hit's OWN file: `<button class="` spans
    // 15 bytes.
    assert_eq!(html.offset, offset_of(TEMPLATE_HTML, "inline-flex"));
}

#[test]
fn scan_raw_strings_and_single_quotes_extract() {
    let source =
        "let long = r#\"grid grid-cols-[0_1fr] gap-4\"#;\nlet short = 'italic font-semibold';\n";
    let files = [SourceFile::new("raw.rs", source)];
    let scanned = scan(&files, &RustStrings);
    let texts: Vec<&str> = scanned.hits.iter().map(|h| h.text.as_str()).collect();
    assert_eq!(
        texts,
        ["grid grid-cols-[0_1fr] gap-4", "italic font-semibold"]
    );
}

#[test]
fn scan_prose_literals_stay_out() {
    let source = "let note = \"a plain sentence without the utility shapes\";\n";
    let files = [SourceFile::new("prose.rs", source)];
    let scanned = scan(&files, &RustStrings);
    assert!(
        scanned.hits.is_empty(),
        "the class-shaped pre-filter rides prose out: {:?}",
        scanned.hits
    );
}

#[test]
fn scan_hits_are_stable_in_source_order() {
    // Determinism contract: two scans of the same inputs produce the
    // same hit list (file order, then in-file offset order) — the
    // artifact's first-seen ties ride this order.
    let a = scan(&files(), &RustStrings);
    let b = scan(&files(), &RustStrings);
    assert_eq!(a, b);
}

// ---------------------------------------------------------------------------
// The build half
// ---------------------------------------------------------------------------

#[test]
fn build_dedupes_class_strings_across_files() {
    // The same badge string appears in BOTH files; the artifact carries
    // its rule ONCE (global dedupe, approach C's whole point).
    let scanned = scan(&files(), &RustStrings);
    let artifact = build(&scanned, Some(&runtime_tokens()), &Config::default());
    let badge_rules = artifact
        .rules
        .iter()
        .filter(|r| r.selector.contains("inline-flex"))
        .count();
    assert_eq!(badge_rules, 1, "one badge rule despite two occurrences");
}

#[test]
fn build_orders_rules_by_the_registry_progression() {
    // The sort seam's canonical order decides the candidate order —
    // the artifact's global rule order is THE SORT SEAM'S OWN
    // DECISION, derived here from the public seam (the differential
    // form: the artifact never re-derives a position the registry
    // already answered). Within one candidate the rules ride the
    // sheet's source order.
    use std::collections::HashMap;

    use aeolus_compile::{emit_css, sort};

    let a = "flex flex-col p-4";
    let b = "block rounded-md";
    let source = format!("let a = tw!(\"{a}\");\nlet b = tw!(\"{b}\");\n");
    let files = [SourceFile::new("a.rs", source.as_str())];
    let scanned = scan(&files, &RustStrings);
    let artifact = build(&scanned, None, &Config::default());

    let sorted = sort(&format!("{a} {b}")).expect("non-empty input sorts");
    let mut keys: HashMap<&str, (u8, usize)> = HashMap::new();
    for class in &sorted.classes {
        keys.entry(class.raw.as_str()).or_insert_with(|| {
            if class.known {
                (class.key.bucket, class.key.row)
            } else {
                (u8::MAX, usize::MAX)
            }
        });
    }
    let class_key = |class: &str| keys.get(class).copied().unwrap_or((u8::MAX, usize::MAX));
    let head_key = |classes: &[&str]| {
        classes
            .iter()
            .map(|class| class_key(class))
            .min()
            .unwrap_or((u8::MAX, usize::MAX))
    };
    let (first, second) = if head_key(a.split_whitespace().collect::<Vec<_>>().as_slice())
        <= head_key(b.split_whitespace().collect::<Vec<_>>().as_slice())
    {
        (a, b)
    } else {
        (b, a)
    };
    // The expected rule order: the winning candidate's sheet first
    // (source order), then the other's.
    let expected_first = emit_css(first, &Config::default())
        .expect("non-empty input renders")
        .rules;
    let expected_second = emit_css(second, &Config::default())
        .expect("non-empty input renders")
        .rules;
    let expected: Vec<_> = expected_first
        .iter()
        .chain(expected_second.iter())
        .collect();
    assert_eq!(
        artifact.rules.len(),
        expected.len(),
        "{:?} vs {expected:?}",
        artifact.rules
    );
    for (rule, expected) in artifact.rules.iter().zip(expected) {
        assert_eq!(rule, expected, "the sort seam's own order decided");
    }
}

#[test]
fn build_variant_buckets_sort_after_the_base_across_candidates() {
    // The cascade specificity the sort seam buckets by decides the
    // candidate order too: the hover-only candidate renders after the
    // base candidate despite being first-seen in the scan.
    let source = "let a = tw!(\"hover:bg-primary/90\");\nlet b = tw!(\"bg-primary\");\n";
    let files = [SourceFile::new("a.rs", source)];
    let scanned = scan(&files, &RustStrings);
    let tokens = runtime_tokens();
    let artifact = build(&scanned, Some(&tokens), &Config::default());
    let bg_idx = artifact
        .rules
        .iter()
        .position(|r| r.selector == ".bg-primary")
        .expect("base rule present");
    let hover_idx = artifact
        .rules
        .iter()
        .position(|r| r.selector.contains("hover"))
        .expect("hover rule present");
    assert!(hover_idx > bg_idx, "{:?}", artifact.rules);
}

#[test]
fn build_renders_a_self_contained_theme_vars_block() {
    let scanned = scan(&files(), &RustStrings);
    let artifact = build(&scanned, Some(&runtime_tokens()), &Config::default());
    // Every token the table carries renders as a var the rules'
    // `var(--color-…)` references resolve against; dark overrides sit
    // in the `.dark` block. The names use the emitter's own
    // `--color-` prefix.
    assert!(!artifact.vars.is_empty());
    let primary = artifact
        .vars
        .iter()
        .find(|v| v.name == "primary")
        .expect("primary token present");
    assert!(primary.light.starts_with("rgb("), "{}", primary.light);
    assert!(
        artifact
            .vars
            .iter()
            .any(|v| v.name == "destructive" && v.dark.is_some()),
        "dark overrides ride their tokens"
    );
    let rendered = artifact.to_string();
    assert!(rendered.contains(":root {"), "{rendered}");
    assert!(rendered.contains("--color-primary:"), "{rendered}");
    assert!(rendered.contains(".dark {"), "{rendered}");
    // The vars block PRECEDES the rules (a stylesheet resolves its own
    // references regardless, but the reading order is the artifact's).
    let root = rendered.find(":root {").expect("root block");
    let dark = rendered.find(".dark {").expect("dark block");
    let first_rule = rendered.find(".flex {").or_else(|| rendered.find(".p-4 {"));
    assert!(
        first_rule.is_none_or(|pos| pos > root),
        "vars before rules: {rendered}"
    );
    assert!(dark > root, "dark block after the light block");
}

#[test]
fn build_without_a_table_keeps_the_token_boundary_honest() {
    let scanned = scan(&files(), &RustStrings);
    let artifact = build(&scanned, None, &Config::default());
    // `bg-primary` lowers only through the theme pass: the themeless
    // build reports the boundary — per LOCATION (two files say it).
    let token_misses = artifact
        .diagnostics
        .iter()
        .filter(|d| d.diagnostic.kind.code() == "aeolus::compile::unknown_root")
        .count();
    assert!(token_misses >= 2, "{:?}", artifact.diagnostics);
    assert!(
        artifact
            .rules
            .iter()
            .all(|r| !r.body.contains("var(--color-")),
        "no theme forms render without a table"
    );
}

#[test]
fn build_diagnostics_carry_every_location_of_the_offender() {
    // The badge string (with its `bg-secondary` token) appears in both
    // files; without a table each occurrence reports where it sits.
    let scanned = scan(&files(), &RustStrings);
    let artifact = build(&scanned, None, &Config::default());
    let badge_locations: Vec<&str> = artifact
        .diagnostics
        .iter()
        .filter(|d| d.location.text.contains("bg-secondary"))
        .map(|d| d.location.file.as_str())
        .collect();
    assert_eq!(badge_locations, ["src/view.rs", "assets/template.html"]);
}

#[test]
fn build_is_differential_with_the_per_string_emit() {
    // The artifact's rule for one candidate is byte-equal to the CSS
    // arm's own sheet for the same string — the scan seam ADDS the
    // global walk (dedupe, order, vars, locations) and changes no
    // per-string semantics.
    let scanned = scan(&files(), &RustStrings);
    let tokens = runtime_tokens();
    let artifact = build(&scanned, Some(&tokens), &Config::default());
    let sheet =
        aeolus_compile::emit_css_theme("md:p-6 hover:bg-primary/90", &tokens, &Config::default())
            .expect("non-empty input renders");
    for rule in &sheet.rules {
        assert!(
            artifact.rules.iter().any(|r| r == rule),
            "the sheet's rule rides the artifact: {rule:?}"
        );
    }
}

#[test]
fn build_groups_at_rules_like_the_sheet() {
    // The sheet's Display groups consecutive same-media rules into one
    // @media block; the artifact's Display keeps the same contract —
    // the md:p-6 rule renders INSIDE the block, not as a bare
    // `@media`-prefixed line.
    let scanned = scan(&files(), &RustStrings);
    let artifact = build(&scanned, None, &Config::default());
    let rendered = artifact.to_string();
    assert!(
        rendered.contains("@media (min-width: 48rem) {"),
        "{rendered}"
    );
    assert!(
        rendered.contains("  .md\\:p-6 {"),
        "the rule sits indented inside the at-rule block: {rendered}"
    );
}

#[test]
fn build_with_registers_customs_and_the_registry_wins() {
    use aeolus_compile::{CustomForm, CustomUtility, Customs};
    use aeolus_ir::Prop;

    let source = "let a = tw!(\"brand-p-4 p-4\");\n";
    let files = [SourceFile::new("a.rs", source)];
    let scanned = scan(&files, &RustStrings);
    let customs = Customs::new().utility(CustomUtility::functional(
        "brand-p",
        CustomForm::Spacing,
        &[
            Prop::PaddingTop,
            Prop::PaddingRight,
            Prop::PaddingBottom,
            Prop::PaddingLeft,
        ],
    ));
    let artifact = build_with(&scanned, None, &customs, &Config::default());
    assert!(
        artifact
            .rules
            .iter()
            .any(|r| r.selector.contains(r"brand-p-4")),
        "the registration renders: {:?}",
        artifact
            .rules
            .iter()
            .map(|r| &r.selector)
            .collect::<Vec<_>>()
    );
    assert!(artifact.diagnostics.is_empty());
}

#[test]
fn build_of_an_empty_scan_is_an_empty_artifact() {
    let files = [SourceFile::new("empty.rs", "let x = 1;\n")];
    let scanned = scan(&files, &RustStrings);
    assert!(scanned.hits.is_empty());
    let artifact = build(&scanned, None, &Config::default());
    assert!(artifact.rules.is_empty());
    assert!(artifact.diagnostics.is_empty());
    // No table, no candidates: the empty stylesheet — not an error.
    assert!(artifact.vars.is_empty());
    assert_eq!(artifact.to_string(), "");
}

#[test]
fn theme_vars_render_rgb_forms() {
    // The vars block reuses the emitter's rgb text forms (full opacity
    // `rgb(r g b)`, alpha `rgb(r g b / a%)`); the values come from the
    // table, never a hardcode.
    let tokens = runtime_tokens();
    let scanned = aeolus_compile::Scanned::default();
    let artifact = build(&scanned, Some(&tokens), &Config::default());
    // The vars block is the table's own emission, candidates or not.
    assert!(!artifact.vars.is_empty());
    let rendered = artifact.to_string();
    assert!(rendered.contains("--color-primary: rgb("), "{rendered}");
}
