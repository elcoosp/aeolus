//! The `audit` command's behavior contract (TDD: written FIRST — the RED
//! capture is the compile failure before the face exists, recorded in
//! `reports/10-tdd-evidence/40-capability-audit.md`).
//!
//! The battery pins the capability-audit face end-to-end: the scan seam's
//! strings through the V1 compile walk into the codegen seam's per-longhand
//! tiered rows against the GPUI backend, the text and JSON renderings, the
//! `--fail-on` gate keyed on the TIER (never on the prose), the usage
//! errors and the theme ride — the same battery shape the other command
//! suites pin.

use aeolus_cli::run;

fn run_cmd(args: &[&str]) -> (i32, String, String) {
    let owned: Vec<String> = args.iter().map(std::string::ToString::to_string).collect();
    let mut out: Vec<u8> = Vec::new();
    let mut err: Vec<u8> = Vec::new();
    let code = run(owned, &mut out, &mut err);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

/// A source file the audit reads: every tier at least once — two native
/// shapes, the ring family's polyfill half, the z-index and blur
/// refusals (the generated table's own typed boundaries) — plus a
/// repeat occurrence the dedupe folds into the first.
fn write_source(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("aeolus-cli-audit-{tag}-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let file = dir.join("component.rs");
    std::fs::write(
        &file,
        r#"let clean = "p-4 flex items-center";
let stacked = "z-10";
let halo = "ring-2 ring-offset-2";
let soft = "blur-md";
let again = "z-10";
"#,
    )
    .expect("the fixture writes");
    file
}

#[test]
fn audit_reports_tiers_per_class_with_exit_zero_by_default() {
    let file = write_source("text");
    let path = file.to_str().expect("utf-8 path");
    let (code, out, err) = run_cmd(&["audit", path]);
    assert_eq!(
        code, 0,
        "the report face never gates by default; stderr: {err}"
    );
    // Each class string rides the report, the non-native rows name the
    // longhand, the tier word and the generated table's own reason.
    assert!(out.contains("\"z-10\""), "{out}");
    assert!(out.contains("refused"), "{out}");
    assert!(out.contains("GPUI has no native slot"), "{out}");
    assert!(out.contains("\"blur-md\""), "{out}");
    assert!(
        out.contains("GPUI has no blur/backdrop-filter slot"),
        "{out}"
    );
    assert!(out.contains("polyfill"), "{out}");
    assert!(out.contains("\"ring-2 ring-offset-2\""), "{out}");
    // The summary carries the backend name and the observed-set score.
    assert!(out.contains("gpui"), "{out}");
    assert!(out.contains("expressible"), "{out}");
    // The repeat occurrence dedupes: `z-10` appears once as a class row.
    assert_eq!(out.matches("\"z-10\"").count(), 1, "{out}");
}

#[test]
fn audit_fail_on_keys_on_the_tier() {
    let file = write_source("gate");
    let path = file.to_str().expect("utf-8 path");
    let cases = [
        (&["audit", "--fail-on", "refused", path] as &[&str], 1),
        (&["audit", "--fail-on", "polyfill", path], 1),
        (&["audit", "--fail-on", "any", path], 1),
        (&["audit", "--fail-on", "none", path], 0),
        (&["audit", path], 0),
    ];
    for (args, expected) in cases {
        let (code, _out, err) = run_cmd(args);
        assert_eq!(code, expected, "args {args:?}: stderr {err}");
    }
}

#[test]
fn audit_native_only_source_passes_every_gate() {
    let dir = std::env::temp_dir().join(format!("aeolus-cli-audit-clean-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let file = dir.join("clean.rs");
    std::fs::write(&file, r#"let s = "p-4 flex items-center";"#).expect("fixture writes");
    let path = file.to_str().expect("utf-8 path");
    for mode in ["none", "polyfill", "refused", "any"] {
        let (code, out, err) = run_cmd(&["audit", "--fail-on", mode, path]);
        assert_eq!(code, 0, "mode {mode}: stderr {err}");
        assert!(out.contains("native"), "{out}");
    }
}

#[test]
fn audit_json_renders_the_same_report() {
    let file = write_source("json");
    let path = file.to_str().expect("utf-8 path");
    let (code, out, err) = run_cmd(&["audit", "--json", path]);
    assert_eq!(code, 0, "{err}");
    // One object: the backend, the observed-set score and one entry per
    // class with its rows (the same arithmetic the text form prints).
    assert!(out.starts_with('{'), "{out}");
    assert!(out.contains("\"backend\":\"gpui\""), "{out}");
    assert!(out.contains("\"score\":"), "{out}");
    assert!(out.contains("\"raw\":\"z-10\""), "{out}");
    assert!(out.contains("\"tier\":\"refused\""), "{out}");
    assert!(out.contains("\"prop\":\"z-index\""), "{out}");
    assert!(
        out.contains("\"reason\":\"GPUI has no native slot\""),
        "{out}"
    );
    assert!(out.contains("\"worst\":\"polyfill\""), "{out}");
}

#[test]
fn audit_theme_flag_lowers_the_tokens() {
    let dir = std::env::temp_dir().join(format!("aeolus-cli-audit-theme-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let file = dir.join("themed.rs");
    std::fs::write(&file, r#"let s = "bg-primary text-primary-foreground";"#)
        .expect("fixture writes");
    let path = file.to_str().expect("utf-8 path");
    // The themed walk lowers the token slots; the longhands are native,
    // so every gate passes and the summary counts both rows (the text
    // report names only the classes that degrade — the good news rides
    // the summary line).
    let (code, out, err) = run_cmd(&["audit", "--theme", "shadcn", "--fail-on", "any", path]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("2 longhand rows — 2 native"), "{out}");
    assert!(out.contains("100.00% expressible"), "{out}");
    // A bad spec stays the typed usage error (exit 2), the `check` shape.
    let (code, _out, err) = run_cmd(&["audit", "--theme", "default", path]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("no V1 source"), "{err}");
}

#[test]
fn audit_usage_errors_match_the_command_grammar() {
    let (code, _out, err) = run_cmd(&["audit"]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("audit takes at least one file"), "{err}");
    let (code, _out, err) = run_cmd(&["audit", "--fail-on", "sometimes"]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("refused|polyfill|any|none"), "{err}");
    let (code, _out, err) = run_cmd(&["audit", "--unknown-flag", "x.rs"]);
    assert_eq!(code, 2, "{err}");
    let (code, _out, err) = run_cmd(&["audit", "/definitely/missing/file.rs"]);
    assert_eq!(code, 2, "{err}");
}

#[test]
fn audit_empty_observation_set_reports_vacuous_full() {
    // A file whose strings the extractor's prose pre-filter passes out
    // (no Tailwind-looking token): no rows, no refusals — the gate
    // stays quiet even at `refused`, the score prints the documented
    // 100%.
    let dir = std::env::temp_dir().join(format!("aeolus-cli-audit-prose-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let file = dir.join("prose.rs");
    std::fs::write(&file, r#"let story = "once upon a time";"#).expect("fixture writes");
    let path = file.to_str().expect("utf-8 path");
    let (code, out, err) = run_cmd(&["audit", "--fail-on", "refused", path]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("100.00%"), "{out}");
}
