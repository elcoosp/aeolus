//! The `@source` directive seam (architecture plan §5.4's CSS-first
//! compatibility line, item 4 — the last of the four named forms: the
//! `@theme`/`@utility`/`@custom-variant` trio is in-tree, the scanner
//! registration is not): the SAME theme CSS a web Tailwind v4 project
//! uses can register the content the class scan walks — `@source
//! inline("…")` literals, `@source "path"` files and `@source not
//! "path"` prunes — and the parse is typed end to end.
//!
//! The pins:
//! * the three forms parse to their typed entries, in source order;
//! * a theme CSS with NO directives yields an empty list (the built-in
//!   palettes are byte-clean under the scan);
//! * malformed forms (unterminated string, unknown payload, bare
//!   `@source;`) are TYPED errors — the no-silent-failures contract,
//!   never a guessed entry;
//! * the directives ride `ThemeSource::from_css` too — a host reading
//!   the theme carries the registrations beside the token tables;
//! * comments and `@theme`/`:root` bodies never leak into the scan
//!   (only top-level `@source` statements count).

use aeolus_theme::{SourceDirective, ThemeSource, source_directives};

const THEME: &str = "\
@theme {\n  --color-primary: #0ea5e9;\n}\n\
:root {\n  --radius: 0.5rem;\n}\n\
";

#[test]
fn the_three_forms_parse_in_source_order() {
    let css = format!(
        "{THEME}@source inline(\"p-4 bg-primary\");\n@source \"src/views.rs\";\n@source not \"src/generated.rs\";\n"
    );
    let directives = source_directives(&css).expect("well-formed directives parse");
    assert_eq!(
        directives,
        vec![
            SourceDirective::Inline("p-4 bg-primary".to_string()),
            SourceDirective::Path {
                path: "src/views.rs".to_string(),
                negated: false,
            },
            SourceDirective::Path {
                path: "src/generated.rs".to_string(),
                negated: true,
            },
        ]
    );
}

/// The bare path form and the negated form share the typed `Path`
/// entry; `inline` carries its verbatim content.
#[test]
fn forms_are_typed_distinctly() {
    let css = "@source \"./a.css\";\n@source inline(\"hidden\");\n@source not \"./b.css\";\n";
    let directives = source_directives(css).expect("parses");
    assert!(matches!(
        &directives[0],
        SourceDirective::Path { negated: false, .. }
    ));
    assert!(matches!(&directives[1], SourceDirective::Inline(content) if content == "hidden"));
    assert!(matches!(
        &directives[2],
        SourceDirective::Path { negated: true, .. }
    ));
}

/// A theme CSS without directives scans to nothing.
#[test]
fn absence_yields_nothing() {
    assert!(source_directives(THEME).expect("no directives").is_empty());
    assert!(source_directives("").expect("no directives").is_empty());
}

/// Unterminated strings, unknown payloads and the bare form are typed
/// errors — the honest scan never guesses. (The unknown payload is
/// deliberately not comment-shaped: a quoted glob `@source "**/*.rs";`
/// is VALID payload — the string-aware comment walk protects it.)
#[test]
fn malformed_forms_are_typed_errors() {
    let unterminated = "@source \"src/views.rs;\n";
    let err = source_directives(unterminated).expect_err("unterminated string errors");
    assert!(err.to_string().contains("unterminated"), "{err}");
    let unknown = "@source bare-token;\n";
    let err = source_directives(unknown).expect_err("unknown payload errors");
    assert!(err.to_string().contains("unrecognized"), "{err}");
    let bare = "@source;\n";
    let err = source_directives(bare).expect_err("the bare form errors");
    assert!(err.to_string().contains("empty"), "{err}");
}

/// A quoted glob survives the comment walk — the real Tailwind form
/// (`@source "**/*.rs";`) is payload, never a comment opener.
#[test]
fn quoted_globs_survive_the_comment_walk() {
    let css = "/* header */\n@source \"src/**/*.rs\";\n@source inline(\"p-4\");\n";
    let directives = source_directives(css).expect("the glob is payload");
    assert_eq!(
        directives,
        vec![
            SourceDirective::Path {
                path: "src/**/*.rs".to_string(),
                negated: false,
            },
            SourceDirective::Inline("p-4".to_string()),
        ]
    );
}

/// The directives ride the theme source: `ThemeSource::from_css`
/// carries the registrations beside the token tables, and a source
/// with none reads empty.
#[test]
fn the_theme_source_carries_the_registrations() {
    let css = format!("{THEME}@source inline(\"p-4\");\n@source \"lib.rs\";\n");
    let source = ThemeSource::from_css(&css, None).expect("the source parses");
    assert_eq!(
        source.sources(),
        &[
            SourceDirective::Inline("p-4".to_string()),
            SourceDirective::Path {
                path: "lib.rs".to_string(),
                negated: false,
            },
        ]
    );
    let plain = ThemeSource::shadcn().expect("the built-in source parses");
    assert!(plain.sources().is_empty());
}

/// Comments and theme/`:root` bodies never leak: a `@source` mention
/// inside a comment is not a directive; a custom-property named like
/// the directive is not one either.
#[test]
fn comments_and_bodies_do_not_leak() {
    let css = "\
/* @source \"ghost.rs\"; */
@theme {
  /* @source inline(\"ghost\"); */
  --color-x: #fff;
}
:root { --y: 1rem; }
@source \"real.rs\";
";
    let directives = source_directives(css).expect("parses");
    assert_eq!(
        directives,
        vec![SourceDirective::Path {
            path: "real.rs".to_string(),
            negated: false,
        }]
    );
}
