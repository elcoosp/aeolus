//! The custom-authoring seam's CLI battery (the expand face's flag
//! pins): the `--custom <path>` fact loads a CSS file through the
//! `customs_from_css` seam — the `@custom-variant` statements fold
//! onto the typed `Cond` surface (the walk answers them from the same
//! probe situation the built-ins ride) and the `@utility` composite
//! bodies join the registry lookup — so the effective declarations
//! carry the project's own vocabulary. A missing or malformed file is
//! the usage error (exit 2), never a half-answered layer. Written
//! FIRST (the RED spec): the suite fails while the flag surface
//! carries no custom arm.

use aeolus_cli::run;

/// The expand face's effective-declaration lines for one run (exit 0
/// expected — the honest-refusal pins drive `run` directly).
fn expand_lines(args: &[&str]) -> Vec<String> {
    let (code, lines, err) = expand_raw(args);
    assert_eq!(code, 0, "expand exits clean: {err:?}");
    lines
}

/// The expand face's raw run: `(exit code, stdout lines, stderr text)`
/// — the honest-refusal pins read all three channels.
fn expand_raw(args: &[&str]) -> (i32, Vec<String>, String) {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let owned: Vec<String> = args.iter().map(std::string::ToString::to_string).collect();
    let code = run(owned, &mut out, &mut err);
    let lines = String::from_utf8(out)
        .expect("stdout is utf-8")
        .lines()
        .map(str::to_string)
        .collect();
    let err = String::from_utf8(err).expect("stderr is utf-8");
    (code, lines, err)
}

/// The temp CSS file the battery compiles against: one variant
/// statement folding onto the mode tier and one applied utility —
/// the theme CSS's own two statements.
fn custom_css_path(test: &str) -> (std::path::PathBuf, String) {
    let dir = std::env::temp_dir().join(format!(
        "aeolus-cli-custom-{test}-{}-{}",
        std::process::id(),
        line!()
    ));
    std::fs::create_dir_all(&dir).expect("the temp dir builds");
    let css = "@custom-variant night (&:where(.dark, .dark *));\n\
               @utility brand-badge { @apply rounded-md px-2; }\n";
    let path = dir.join("custom.css");
    std::fs::write(&path, css).expect("the temp css writes");
    (path, css.to_string())
}

#[test]
fn the_custom_variant_folds_onto_the_mode_tier() {
    // The `night:` alias folds to `Cond::Dark` (the `.dark` selector's
    // own reading): `--dark` fires it, the resting env stays the quiet
    // miss — the same probe situation the built-in `dark:` answers.
    let (path, _css) = custom_css_path("variant");
    let flag = path.display().to_string();
    let lines = expand_lines(&["expand", "--custom", &flag, "--dark", "p-4", "night:p-8"]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("padding-top = 2rem"),
        "the custom variant answers the asserted mode: {joined}"
    );
    let lines = expand_lines(&["expand", "--custom", &flag, "p-4", "night:p-8"]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("padding-top = 1rem"),
        "the resting env keeps the custom variant quiet: {joined}"
    );
}

#[test]
fn the_custom_utility_joins_the_lookup() {
    // The applied utility's composite body lowers at registration:
    // the class lands its longhands like any registry row's.
    let (path, _css) = custom_css_path("utility");
    let flag = path.display().to_string();
    let lines = expand_lines(&["expand", "--custom", &flag, "p-4", "brand-badge"]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("padding-left = 0.5rem") && joined.contains("padding-right = 0.5rem"),
        "the composite body's spacing longhands answer: {joined}"
    );
    assert!(
        joined.contains("border-") || joined.contains("radius"),
        "the composite body's radius longhand answers: {joined}"
    );
}

#[test]
fn the_custom_surface_rides_the_themed_walk() {
    // The themed dispatch composes with the customs: the utility's
    // class and the theme's token tiers ride ONE compile (the
    // `compile_theme_with` form), the token resolves.
    let (path, _css) = custom_css_path("themed");
    let flag = path.display().to_string();
    let lines = expand_lines(&[
        "expand",
        "--custom",
        &flag,
        "--theme",
        "shadcn",
        "p-4",
        "brand-badge",
        "bg-primary",
    ]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("background-color = #"),
        "the themed walk resolves the token beside the custom utility: {joined}"
    );
    assert!(
        joined.contains("padding-left = 0.5rem"),
        "the custom utility lowers beside the theme classes: {joined}"
    );
}

#[test]
fn the_keyword_rows_carry_the_custom_utility() {
    // The annotation table rides the customs too (`keyword_texts_with`):
    // the composite body's row texts render through the same
    // `(cond, prop)` table the built-ins use.
    let (path, _css) = custom_css_path("keywords");
    let flag = path.display().to_string();
    let lines = expand_lines(&["expand", "--custom", &flag, "brand-badge", "hidden"]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("display = none"),
        "the keyword slot renders its row text beside the custom utility: {joined}"
    );
    assert!(
        joined.contains("border-top-left-radius = 0.375rem"),
        "the composite body's radius keyword renders: {joined}"
    );
}

#[test]
fn a_missing_file_is_the_usage_error() {
    let (code, _lines, err) = expand_raw(&[
        "expand",
        "--custom",
        "/nonexistent/aeolus/custom.css",
        "p-4",
    ]);
    assert_eq!(code, 2, "a missing file is the honest usage error");
    assert!(
        err.contains("--custom"),
        "the refusal names the flag: {err:?}"
    );
}

#[test]
fn a_malformed_statement_is_the_usage_error() {
    // The typed `CustomCssError` boundary: the bare variant form is
    // the malformed statement, rendered at the CLI face (exit 2).
    let dir = std::env::temp_dir().join(format!(
        "aeolus-cli-custom-bad-{}-{}",
        std::process::id(),
        line!()
    ));
    std::fs::create_dir_all(&dir).expect("the temp dir builds");
    let path = dir.join("bad.css");
    std::fs::write(&path, "@custom-variant bare;\n").expect("the temp css writes");
    let flag = path.display().to_string();
    let (code, _lines, err) = expand_raw(&["expand", "--custom", &flag, "p-4"]);
    assert_eq!(code, 2, "a malformed statement is the honest usage error");
    assert!(
        err.contains("custom") || err.contains("bare"),
        "the refusal names the boundary: {err:?}"
    );
}

#[test]
fn a_refused_selector_is_the_usage_error() {
    // The selector grammar is the typed `Cond` surface's: an
    // arbitrary media selector has no flat slot, the registration
    // refuses (exit 2 at the CLI face).
    let dir = std::env::temp_dir().join(format!(
        "aeolus-cli-custom-selector-{}-{}",
        std::process::id(),
        line!()
    ));
    std::fs::create_dir_all(&dir).expect("the temp dir builds");
    let path = dir.join("wide.css");
    std::fs::write(&path, "@custom-variant wide (min-width: 40rem);\n")
        .expect("the temp css writes");
    let flag = path.display().to_string();
    let (code, _lines, err) = expand_raw(&["expand", "--custom", &flag, "p-4"]);
    assert_eq!(code, 2, "a refused selector is the honest usage error");
    assert!(
        err.contains("wide"),
        "the refusal names the refused variant: {err:?}"
    );
}
