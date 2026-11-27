//! The `rsx.toml` import battery (architecture plan §5, "Integrating
//! with `gpui-rsx`", config-bridge bullet: *"an `rsx.toml → aeolus
//! theme` importer"*). Written FIRST (the RED spec): the suite fails
//! against the pre-import theme crate and passes only once
//! `import_rsx` satisfies every pin below.
//!
//! Seven families:
//! 1. **The demo sheet** — the gpui-rsx demo `rsx.toml` imports whole: the
//!    colour/size mappings ride the sheet, the scale defaults to Tailwind,
//!    and every unmappable entry rides a typed note.
//! 2. **The theme round-trip** — the imported sheet builds a
//!    [`ThemeSource`] whose table resolves the imported tokens.
//! 3. **The namespace map** — radius/text-size/line-height/spacing bake
//!    their v4 namespaces; a line-height that shares a text-size's name
//!    pairs onto it (the reader's `--text-{name}--line-height` form), a
//!    lone one takes `--leading-{name}`.
//! 4. **The config bridge** — the `spacing-scale` option rides the report
//!    (`Px` is gpui-rsx's 1-unit-1-px scale, NOT the compile crate's
//!    absolute-px bake — that difference is the migration guide's scale row).
//! 5. **The honest notes** — Rust-expression values, the runtime preset,
//!    the runtime options and named opacities never invent a CSS line; the
//!    note channel answers what LEFT and why.
//! 6. **The typed errors** — the grammar's refusals are line-numbered
//!    variants, one per upstream failure mode, never a stringly panic.
//! 7. **The determinism** — the same input emits byte-identical CSS (and
//!    the quote-aware comment strip keeps `#hex` values whole).

use aeolus_theme::{RsxError, RsxNoteKind, RsxSpacingScale, import_rsx};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// The gpui-rsx demo sheet, verbatim (`gpui-rsx/demo/rsx.toml`): every
/// section but `[radius]`/`[spacing]` rides it, so one import exercises
/// the mappings AND the note channel.
const DEMO: &str = "# Semantic colors resolve through Kit's component theme at render time.
[options]
dynamic-runtime = \"rsx_runtime\"
preset = \"gpui-component\"

[colors]
demo-canvas = \"#111a16\"
demo-surface = \"#19241f\"
demo-raised = \"#24352b\"
demo-border = \"#2b3b33\"
demo-text = \"#edf3ef\"
demo-muted = \"#bac8bf\"
demo-accent = \"#8bddc5\"

[text-size]
body = \"14px\"

[line-height]
body = \"20px\"

[opacity]
muted = \"60%\"

[shadow]
card = { expr = \"vec![BoxShadow::new(px(0.0), px(1.0), hsla(0.0, 0.0, 0.0, 0.2))]\" }
";

// ---------------------------------------------------------------------------
// 1. The demo sheet
// ---------------------------------------------------------------------------

#[test]
fn the_demo_sheet_imports_whole() {
    let import = import_rsx(DEMO).expect("the demo sheet imports");
    // The colour namespace, source order preserved.
    assert!(
        import.css().contains("--color-demo-canvas: #111a16;"),
        "{}",
        import.css()
    );
    assert!(
        import.css().contains("--color-demo-accent: #8bddc5;"),
        "{}",
        import.css()
    );
    // The text-size namespace.
    assert!(
        import.css().contains("--text-body: 14px;"),
        "{}",
        import.css()
    );
    // No invented lines: the unmappable entries stay out of the sheet.
    assert!(!import.css().contains("--shadow-card"), "{}", import.css());
    assert!(!import.css().contains("opacity"), "{}", import.css());
}

#[test]
fn the_demo_scale_defaults_to_tailwind() {
    let import = import_rsx(DEMO).expect("the demo sheet imports");
    assert_eq!(
        import.spacing_scale(),
        RsxSpacingScale::Tailwind,
        "no spacing-scale option: the v4 scale is the default"
    );
}

#[test]
fn the_demo_notes_answer_what_left() {
    let import = import_rsx(DEMO).expect("the demo sheet imports");
    // One note per unmappable entry, in source order: the two runtime
    // options, the runtime preset, the named opacity, the shadow
    // expression.
    let names: Vec<(&str, RsxNoteKind)> = import
        .notes()
        .iter()
        .map(|note| (note.name.as_str(), note.kind))
        .collect();
    assert_eq!(
        names,
        vec![
            ("dynamic-runtime", RsxNoteKind::RuntimeOption),
            ("preset", RsxNoteKind::RuntimePreset),
            ("muted", RsxNoteKind::NamedOpacity),
            ("card", RsxNoteKind::RustExpression),
        ],
        "the note channel is the honest-failure contract at the import seam"
    );
    // The notes carry their source lines (the demo's fixed layout).
    assert_eq!(
        import.notes()[0].line,
        3,
        "dynamic-runtime is the third line"
    );
    assert_eq!(import.notes()[3].line, 25, "the shadow card rides line 25");
}

// ---------------------------------------------------------------------------
// 2. The theme round-trip
// ---------------------------------------------------------------------------

#[test]
fn the_imported_sheet_builds_a_resolving_table() {
    let import = import_rsx(DEMO).expect("the demo sheet imports");
    let source = import.theme_source().expect("the imported sheet bakes");
    let table = aeolus_theme::TokenTable::runtime(&source).expect("the table builds");
    let id = table
        .id("demo-accent")
        .expect("the imported colour is a table token");
    let rgba = table
        .resolve_rgba(id, 1.0, false)
        .expect("the imported hex resolves");
    assert_eq!(rgba, [0x8b, 0xdd, 0xc5, 0xff], "#8bddc5 resolves bit-true");
}

// ---------------------------------------------------------------------------
// 3. The namespace map
// ---------------------------------------------------------------------------

#[test]
fn radius_bakes_the_radius_namespace() {
    let import = import_rsx("[radius]\ncard = \"12px\"\n").expect("imports");
    assert!(
        import.css().contains("--radius-card: 12px;"),
        "{}",
        import.css()
    );
}

#[test]
fn a_shared_line_height_pairs_onto_its_text_parent() {
    let import = import_rsx("[text-size]\nbody = \"14px\"\n\n[line-height]\nbody = \"20px\"\n")
        .expect("imports");
    assert!(
        import.css().contains("--text-body: 14px;"),
        "{}",
        import.css()
    );
    assert!(
        import.css().contains("--text-body--line-height: 20px;"),
        "{}",
        import.css()
    );
    assert!(
        !import.css().contains("--leading-body"),
        "the paired form replaces the leading namespace: {}",
        import.css()
    );
}

#[test]
fn a_lone_line_height_takes_the_leading_namespace() {
    let import = import_rsx("[line-height]\ntight = \"20px\"\n").expect("imports");
    assert!(
        import.css().contains("--leading-tight: 20px;"),
        "{}",
        import.css()
    );
}

#[test]
fn named_spacing_rides_the_sheet_and_the_note() {
    let import = import_rsx("[spacing]\ncard = \"12px\"\n").expect("imports");
    assert!(
        import.css().contains("--spacing-card: 12px;"),
        "the v4 functional namespace rides the sheet: {}",
        import.css()
    );
    let names: Vec<&str> = import
        .notes()
        .iter()
        .map(|note| note.name.as_str())
        .collect();
    assert_eq!(
        names,
        vec!["card"],
        "aeolus's spacing classes consume the numeric scale, so a named entry rides the note channel too"
    );
    assert_eq!(
        import.notes()[0].kind,
        RsxNoteKind::NamedSpacing,
        "the note names the boundary kind"
    );
}

#[test]
fn rem_lengths_pass_through_verbatim() {
    let import = import_rsx("[radius]\nfull = \"0.75rem\"\n").expect("imports");
    assert!(
        import.css().contains("--radius-full: 0.75rem;"),
        "{}",
        import.css()
    );
}

// ---------------------------------------------------------------------------
// 4. The config bridge
// ---------------------------------------------------------------------------

#[test]
fn the_spacing_scale_option_rides_the_report() {
    let px = import_rsx("[options]\nspacing-scale = \"px\"\n").expect("imports");
    assert_eq!(px.spacing_scale(), RsxSpacingScale::Px);
    assert!(px.spacing_scale().is_px());
    let tailwind = import_rsx("[options]\nspacing-scale = \"tailwind\"\n").expect("imports");
    assert_eq!(tailwind.spacing_scale(), RsxSpacingScale::Tailwind);
    assert!(!tailwind.spacing_scale().is_px());
}

// ---------------------------------------------------------------------------
// 5. The honest notes
// ---------------------------------------------------------------------------

#[test]
fn rust_expressions_ride_notes_not_the_sheet() {
    let import = import_rsx(
        "[colors]\nbrand = { expr = \"cx.theme().colors.primary\" }\n[shadow]\ncard = { expr = \"vec![]\" }\n",
    )
    .expect("imports");
    assert!(
        !import.css().contains("--color-brand"),
        "a Rust expression has no CSS form: {}",
        import.css()
    );
    let names: Vec<&str> = import
        .notes()
        .iter()
        .map(|note| note.name.as_str())
        .collect();
    assert_eq!(
        names,
        vec!["brand", "card"],
        "both expressions answer, in source order"
    );
    assert_eq!(
        import.notes()[0].kind,
        RsxNoteKind::RustExpression,
        "the note kind names the shape"
    );
}

#[test]
fn colour_and_length_grammar_stays_strict() {
    // The 4-digit hex form is NOT in the upstream grammar (#rgb/#rrggbb/
    // #rrggbbaa): the import refuses typed instead of guessing.
    let err = import_rsx("[colors]\nbrand = \"#abcd\"\n").expect_err("4-digit hex refuses");
    assert!(
        matches!(err, RsxError::BadColor { line: 2, .. }),
        "typed, line-numbered: {err:?}"
    );
    // Three-digit short form rides (the upstream grammar's own shape).
    let short = import_rsx("[colors]\nbrand = \"#abc\"\n").expect("short hex imports");
    assert!(
        short.css().contains("--color-brand: #abc;"),
        "{}",
        short.css()
    );
    // Alpha hex rides verbatim (the reader's 8-digit form).
    let alpha = import_rsx("[colors]\nbrand = \"#8bddc580\"\n").expect("alpha hex imports");
    assert!(
        alpha.css().contains("--color-brand: #8bddc580;"),
        "{}",
        alpha.css()
    );
    // A length outside px/rem refuses (the upstream grammar: px/rem only).
    let err = import_rsx("[radius]\nbig = \"3vh\"\n").expect_err("vh refuses");
    assert!(
        matches!(err, RsxError::BadLength { line: 2, .. }),
        "typed, line-numbered: {err:?}"
    );
}

#[test]
fn bad_opacity_values_refuse_typed() {
    let err = import_rsx("[opacity]\ntooMuch = \"1.5\"\n").expect_err("over 1 refuses");
    assert!(
        matches!(err, RsxError::BadOpacity { line: 2, .. }),
        "typed, line-numbered: {err:?}"
    );
    // The two upstream forms ride the note channel.
    let import = import_rsx("[opacity]\nfaint = \"0.6\"\nsoft = \"60%\"\n").expect("imports");
    let names: Vec<&str> = import
        .notes()
        .iter()
        .map(|note| note.name.as_str())
        .collect();
    assert_eq!(names, vec!["faint", "soft"], "both named opacities note");
    assert_eq!(import.notes()[0].kind, RsxNoteKind::NamedOpacity);
}

// ---------------------------------------------------------------------------
// 6. The typed errors
// ---------------------------------------------------------------------------

#[test]
fn the_grammar_refusals_are_typed_and_line_numbered() {
    let cases: Vec<(&str, RsxError)> = vec![
        (
            "[colours]\nx = \"#111111\"\n",
            RsxError::UnknownSection {
                line: 1,
                section: "colours".to_string(),
            },
        ),
        (
            "x = \"#111111\"\n",
            RsxError::KeyOutsideSection {
                line: 1,
                key: "x".to_string(),
            },
        ),
        ("[colors]\nno value\n", RsxError::MissingEquals { line: 2 }),
        (
            "[colors]\nbad key! = \"#111111\"\n",
            RsxError::InvalidKey {
                line: 2,
                key: "bad key!".to_string(),
            },
        ),
        (
            "[colors]\nbrand = \"#111111\"\nbrand = \"#222222\"\n",
            RsxError::DuplicateKey {
                line: 3,
                section: "colors".to_string(),
                key: "brand".to_string(),
            },
        ),
        (
            "[options]\nnope = \"x\"\n",
            RsxError::UnknownOption {
                line: 2,
                option: "nope".to_string(),
            },
        ),
        (
            "[options]\nspacing-scale = \"em\"\n",
            RsxError::BadSpacingScale {
                line: 2,
                value: "em".to_string(),
            },
        ),
        (
            "[options]\npreset = \"tailwindui\"\n",
            RsxError::BadPreset {
                line: 2,
                value: "tailwindui".to_string(),
            },
        ),
    ];
    for (source, expected) in cases {
        let err = import_rsx(source).expect_err("each malformed sheet refuses");
        assert_eq!(err, expected, "the refusal matches for {source:?}");
        let text = err.to_string();
        assert!(
            text.starts_with("rsx.toml:"),
            "the Display carries the file shape the upstream messages use: {text}"
        );
        assert!(
            text.contains("line") || text.contains(':'),
            "the line rides: {text}"
        );
    }
}

#[test]
fn a_shadow_string_refuses_upstream_shaped() {
    let err = import_rsx("[shadow]\ncard = \"0 1px 2px black\"\n").expect_err("string refuses");
    assert!(
        matches!(err, RsxError::ShadowNeedsExpression { line: 2, .. }),
        "the upstream grammar takes expressions only here: {err:?}"
    );
}

#[test]
fn an_unquoted_option_value_refuses() {
    let err = import_rsx("[options]\ngpui-path = crates\n").expect_err("bare value refuses");
    assert!(
        matches!(err, RsxError::BadValue { line: 2, .. }),
        "the value grammar takes quoted strings and inline tables: {err:?}"
    );
}

// ---------------------------------------------------------------------------
// 7. Determinism and quiet lines
// ---------------------------------------------------------------------------

#[test]
fn the_import_is_deterministic() {
    let first = import_rsx(DEMO).expect("imports");
    let second = import_rsx(DEMO).expect("imports");
    assert_eq!(first.css(), second.css(), "byte-identical sheets");
    assert_eq!(first.notes(), second.notes(), "byte-identical notes");
}

#[test]
fn comments_and_quoted_hashes_are_quiet() {
    let import =
        import_rsx("# a comment line\n\n[colors]\ncanvas = \"#111a16\" # an inline comment\n")
            .expect("imports");
    assert!(
        import.css().contains("--color-canvas: #111a16;"),
        "the # inside the quoted value survives the comment strip: {}",
        import.css()
    );
}

#[test]
fn an_empty_sheet_imports_to_an_empty_theme() {
    let import = import_rsx("").expect("an empty file is a valid (empty) sheet");
    assert!(
        import.css().contains("@theme"),
        "the sheet keeps the @theme shape"
    );
    assert!(import.notes().is_empty(), "nothing left, nothing noted");
    let source = import
        .theme_source()
        .expect("the empty sheet bakes to the v4 seed");
    let table = aeolus_theme::TokenTable::runtime(&source).expect("the seed builds");
    let id = table.id("red-500").expect("the v4 seed carries red-500");
    assert!(
        table.resolve_rgba(id, 1.0, false).is_ok(),
        "the seed resolves"
    );
}
