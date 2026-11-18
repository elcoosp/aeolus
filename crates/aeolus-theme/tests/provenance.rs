// ---------------------------------------------------------------------------
// The token-provenance contracts (the theme pass's origin scan): every
// colour token the source's own CSS declares answers WHERE it declares —
// the line, the UTF-16 column and the key's own width, byte-true — and
// the scan's answer set is CLOSED by the audited reader's own names (a
// token the file does not declare stays dark). The scan walks the SAME
// text `ThemeSource::from_css` parsed: a key mentioned in a comment, a
// quoted string or a `var()` reference is not a declaration, and the
// prefixed `--color-{name}` form wins over the bare shadcn `--{name}`
// form (the reader's own `color()` lookup order). The built-in palettes
// are the compiler's own data — no file to jump to, no origin answered.
//
// Written FIRST (the RED spec).
// ---------------------------------------------------------------------------

use aeolus_theme::{ThemeSource, TokenTable};

/// A v4-named theme plus two shadcn-named bare keys: every name the
/// battery's assertions ride, on known lines.
const FIXTURE: &str = concat!(
    "@theme {\n",                              // line 0
    "  --color-primary: oklch(0.205 0 0);\n",  // line 1, key at column 2
    "  --color-secondary: oklch(0.97 0 0);\n", // line 2
    "}\n",                                     // line 3
    ":root {\n",                               // line 4
    "  --muted: oklch(0.97 0 0);\n",           // line 5 (the shadcn bare form)
    "}\n",
);

#[test]
fn origins_land_on_their_declarations() {
    let source = ThemeSource::from_css(FIXTURE, None).expect("the fixture parses");
    let origin = source
        .provenance()
        .origin("primary")
        .expect("the declared token answers an origin");

    assert_eq!(origin.line, 1, "the key rides its own line");
    assert_eq!(
        origin.character, 2,
        "the key starts two columns in (the indentation)"
    );
    assert_eq!(
        origin.length, 15,
        "the range covers the key itself (`--color-primary`)"
    );
}

#[test]
fn the_bare_shadcn_key_answers() {
    let source = ThemeSource::from_css(FIXTURE, None).expect("the fixture parses");
    let origin = source
        .provenance()
        .origin("muted")
        .expect("the shadcn-named token answers an origin");

    assert_eq!(origin.line, 5, "the bare `--muted` form is the declaration");
    assert_eq!(
        origin.length, 7,
        "the range covers `--muted` (the bare key)"
    );
}

#[test]
fn the_prefixed_key_wins_over_the_bare() {
    let css = concat!(
        "@theme {\n",
        "  --color-primary: oklch(0.205 0 0);\n",
        "}\n",
        ":root {\n",
        "  --primary: oklch(0.205 0 0);\n",
        "}\n",
    );
    let source = ThemeSource::from_css(css, None).expect("the fixture parses");
    let origin = source
        .provenance()
        .origin("primary")
        .expect("both forms carry the name");

    assert_eq!(
        origin.line, 1,
        "the prefixed form is the declaration (the reader's own lookup order)"
    );
}

#[test]
fn a_token_the_file_does_not_declare_stays_dark() {
    let source = ThemeSource::from_css(FIXTURE, None).expect("the fixture parses");

    // The v4 palette seeds every table (the default theme is the
    // baseline); a seeded name the file never declares has no origin —
    // the honest dark, never a guessed span.
    assert!(
        source.provenance().origin("red-500").is_none(),
        "a seeded token the file does not declare answers no origin"
    );
}

#[test]
fn a_comment_mentioned_declaration_is_not_one() {
    let css = concat!(
        "@theme {\n",
        "  /* --color-primary: oklch(0.5 0 0); */\n",
        "  --color-primary: oklch(0.205 0 0);\n",
        "}\n",
    );
    let source = ThemeSource::from_css(css, None).expect("the fixture parses");
    let origin = source
        .provenance()
        .origin("primary")
        .expect("the real declaration answers");

    assert_eq!(
        origin.line, 2,
        "the commented mention is skipped; the real declaration is the origin"
    );
}

#[test]
fn a_quoted_mention_is_not_one() {
    // The quoted mention rides a font-family value (a string value on a
    // colour key is the reader's own typed refusal): the KEY the scan
    // looks for appears inside the string, and a string is not a
    // declaration.
    let css = concat!(
        "@theme {\n",
        "  --font-serif: \"not a --color-primary declaration\";\n",
        "  --color-primary: oklch(0.205 0 0);\n",
        "}\n",
    );
    let source = ThemeSource::from_css(css, None).expect("the fixture parses");
    let origin = source
        .provenance()
        .origin("primary")
        .expect("the real declaration answers");

    assert_eq!(
        origin.line, 2,
        "the quoted mention is skipped (a string is not a declaration)"
    );
}

#[test]
fn a_var_reference_is_not_a_declaration() {
    let css = concat!(
        "@theme {\n",
        "  --color-cta: var(--color-primary);\n",
        "  --color-primary: oklch(0.205 0 0);\n",
        "}\n",
    );
    let source = ThemeSource::from_css(css, None).expect("the fixture parses");
    let origin = source
        .provenance()
        .origin("primary")
        .expect("the real declaration answers");

    assert_eq!(
        origin.line, 2,
        "the var() reference's mention is not the declaration"
    );
}

#[test]
fn the_prefix_boundary_holds() {
    let css = concat!(
        "@theme {\n",
        "  --color-primary-foreground: oklch(0.985 0 0);\n",
        "}\n",
    );
    let source = ThemeSource::from_css(css, None).expect("the fixture parses");

    assert!(
        source.provenance().origin("primary").is_none(),
        "`--color-primary-foreground` is not `primary`'s declaration"
    );
    let foreground = source
        .provenance()
        .origin("primary-foreground")
        .expect("the full name answers");
    assert_eq!(foreground.line, 1);
}

#[test]
fn the_dark_sheet_rides_the_light_site() {
    let light = concat!(
        "@theme {\n",
        "  --color-primary: oklch(0.205 0 0);\n",
        "}\n",
    );
    let dark = concat!(".dark {\n", "  --color-primary: oklch(0.985 0 0);\n", "}\n",);
    let source = ThemeSource::from_css(light, Some(dark)).expect("the fixture parses");
    let origin = source
        .provenance()
        .origin("primary")
        .expect("the paired token answers an origin");

    assert_eq!(
        origin.line, 1,
        "the light sheet is the canonical site (the dark override re-answers the name)"
    );
}

#[test]
fn the_built_in_palettes_carry_no_origins() {
    let shadcn = ThemeSource::shadcn().expect("the embedded palette parses");
    let dark = ThemeSource::shadcn_dark().expect("the embedded palettes parse");

    assert!(
        shadcn.provenance().is_empty(),
        "the built-in data is no file: no origin answered"
    );
    assert!(
        dark.provenance().is_empty(),
        "the built-in pair answers no origin either"
    );
}

#[test]
fn the_table_rides_the_scan() {
    let source = ThemeSource::from_css(FIXTURE, None).expect("the fixture parses");
    let baked = TokenTable::baked(&source).expect("the bake succeeds");
    let runtime = TokenTable::runtime(&source).expect("the runtime bake succeeds");

    let expected = source
        .provenance()
        .origin("primary")
        .copied()
        .expect("the source answers");
    assert_eq!(
        baked.origin("primary").copied(),
        Some(expected),
        "the baked table carries the source's own scan"
    );
    assert_eq!(
        runtime.origin("secondary").copied(),
        source.provenance().origin("secondary").copied(),
        "the runtime table rides the same provenance"
    );
    assert!(
        TokenTable::baked(&ThemeSource::shadcn().expect("the built-in parses"))
            .expect("the bake succeeds")
            .origin("primary")
            .is_none(),
        "the built-in bake answers no origin"
    );
}

#[test]
fn the_column_counts_utf16_pairs_not_bytes() {
    // The wave emoji rides the declaration's line BEFORE the key: two
    // UTF-16 code units over four bytes — the column model the wire's
    // positions ride, not the byte model.
    let css = concat!(
        "@theme {\n",
        "  /* 🌊 */ --color-primary: oklch(0.205 0 0);\n",
        "}\n",
    );
    let source = ThemeSource::from_css(css, None).expect("the fixture parses");
    let origin = source
        .provenance()
        .origin("primary")
        .expect("the declaration answers");

    // "  /* 🌊 */ " = 2 + 9 + 2 = the emoji contributes 2 units over 4
    // bytes, so the UTF-16 column is the byte column minus 2.
    let byte_column = "  /* 🌊 */ ".len() as u32;
    assert_eq!(origin.line, 1);
    assert_eq!(
        origin.character,
        byte_column - 2,
        "the column counts the emoji's pairs, not its bytes"
    );
}

#[test]
fn the_scan_is_pure_over_its_text() {
    let first = ThemeSource::from_css(FIXTURE, None).expect("the fixture parses");
    let second = ThemeSource::from_css(FIXTURE, None).expect("the fixture parses");

    assert_eq!(
        first.provenance(),
        second.provenance(),
        "the same text answers the same origins"
    );
}
