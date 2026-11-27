//! The `rsx.toml` import's wire-up battery (architecture plan §5,
//! "Integrating with `gpui-rsx`", config-bridge bullet): the imported
//! sheet hands its [`ThemeSource`] to the compile seam and the imported
//! tokens colour REAL classes. Written FIRST (the RED spec): the suite
//! fails before the theme crate carries the import seam.
//!
//! The pin is the migration's core promise: a gpui-rsx host's theme
//! data reaches the SAME pipeline `tw!`/`tw_static!` compile through —
//! one import, no parallel theme path. Consumption follows the
//! registry's own two paths: a sheet OVERRIDE of a closed colour name
//! (`red-500`) rewrites `bg-red-500`'s value, and a brand-new imported
//! name (`demo-accent`) lowers through the bare-var arbitrary form
//! (`bg-(--color-demo-accent)`) — the class-grammar's closed suffix
//! set staying the honest boundary it always was.

use aeolus_compile::{Config, DiagnosticKind, compile_theme, emit_css_theme};
use aeolus_ir::Val;
use aeolus_theme::{TokenTable, import_rsx};

const SHEET: &str = "[colors]\ndemo-accent = \"#8bddc5\"\nred-500 = \"#8bddc5\"\n";

/// The base layer's first declaration (a one-class program's answer).
fn base_val(compiled: &aeolus_compile::Compiled) -> Val {
    let decl = compiled
        .program
        .layers
        .first()
        .and_then(|layer| layer.decls.first())
        .expect("the class lowers");
    decl.val
}

/// The BAKED table over the imported sheet (T0: the literals the
/// sheet's own values bake to, no dark half).
fn baked_table() -> TokenTable {
    let import = import_rsx(SHEET).expect("the sheet imports");
    let source = import.theme_source().expect("the sheet bakes");
    TokenTable::baked(&source).expect("the table builds")
}

/// The RUNTIME table over the imported sheet (T2: ids, walk-time
/// resolution — the tier a host theme swap rides).
fn imported_table() -> TokenTable {
    let import = import_rsx(SHEET).expect("the sheet imports");
    let source = import.theme_source().expect("the sheet bakes");
    TokenTable::runtime(&source).expect("the table builds")
}

#[test]
fn the_imported_override_rewrites_a_closed_colour_class() {
    let table = baked_table();
    let compiled = compile_theme("bg-red-500", &table, &Config::default())
        .expect("the overridden colour lowers");
    assert!(compiled.diagnostics.is_empty(), "no refusals ride");
    assert_eq!(
        base_val(&compiled),
        Val::Rgba([0x8b, 0xdd, 0xc5, 0xff]),
        "the import's #8bddc5 rides bg-red-500 bit-true"
    );
}

#[test]
fn the_runtime_tier_carries_the_override_as_a_token() {
    let table = imported_table();
    let compiled = compile_theme("bg-red-500", &table, &Config::default())
        .expect("the overridden colour lowers");
    assert!(compiled.diagnostics.is_empty(), "no refusals ride");
    match base_val(&compiled) {
        Val::Tok { id, alpha } => {
            assert_eq!(alpha, 1.0, "no modifier, no alpha");
            assert_eq!(
                table.name(id),
                Some("red-500"),
                "the token is the overridden name"
            );
            assert_eq!(
                table.resolve_rgba(id, 1.0, false).expect("resolves"),
                [0x8b, 0xdd, 0xc5, 0xff],
                "the walk-time value is the import's"
            );
        }
        other => panic!("the runtime tier lowers to a Tok, got {other:?}"),
    }
}

#[test]
fn the_imported_name_lowers_through_the_bare_var_form() {
    let table = baked_table();
    let compiled = compile_theme("bg-(--color-demo-accent)", &table, &Config::default())
        .expect("the bare-var form lowers");
    assert!(compiled.diagnostics.is_empty(), "no refusals ride");
    assert_eq!(
        base_val(&compiled),
        Val::Rgba([0x8b, 0xdd, 0xc5, 0xff]),
        "the imported token resolves bit-true through the arbitrary form"
    );
}

#[test]
fn the_imported_override_answers_the_opacity_modifier() {
    let table = baked_table();
    let compiled =
        compile_theme("bg-red-500/90", &table, &Config::default()).expect("the modifier lowers");
    assert!(compiled.diagnostics.is_empty(), "no refusals ride");
    // The §2 identity: alpha scales, the OKLab channels hold (the sRGB
    // bytes re-encode from the preserved Oklab — the same shape the
    // theme pass's identity pins carry).
    match base_val(&compiled) {
        Val::Rgba([r, g, b, a]) => {
            assert_eq!((r, g, b), (147, 219, 197), "the Oklab-preserved bytes");
            let alpha = f32::from(a) / 255.0;
            assert!((alpha - 0.9).abs() < 0.01, "90% of 255 bakes, got {alpha}");
        }
        other => panic!("the token lowers to a literal, got {other:?}"),
    }
}

#[test]
fn the_imported_sheet_renders_the_css_arm() {
    let table = baked_table();
    let sheet = emit_css_theme("bg-red-500", &table, &Config::default()).expect("renders");
    let text = sheet.to_string();
    assert!(
        text.contains("background-color: rgb(139 221 197)"),
        "the CSS arm carries the imported value (v4's space-separated form): {text}"
    );
}

#[test]
fn an_unconsumed_imported_name_refuses_honestly() {
    let table = imported_table();
    let compiled = compile_theme("bg-demo-accent", &table, &Config::default())
        .expect("the compile degrades per class, never per string");
    let raws: Vec<&str> = compiled
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.raw.as_str())
        .collect();
    assert_eq!(
        raws,
        vec!["bg-demo-accent"],
        "the miss rides the honest per-class report: {:?}",
        compiled.diagnostics
    );
    assert!(
        matches!(
            compiled.diagnostics[0].kind,
            DiagnosticKind::ShapeRejected { .. }
        ),
        "the suffix is outside the closed colour set (the bare-var form is the \
         consumption path): {:?}",
        compiled.diagnostics[0].kind
    );
}
