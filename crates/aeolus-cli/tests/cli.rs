//! CLI behavior contract (v0.9 §3, TDD: these were written FIRST — the RED
//! capture is the initial "no such crate" compile failure, recorded in
//! `reports/10-tdd-evidence/09-portable-plans.md`).

use aeolus_cli::{check_string, extract_candidate_strings, run};
use aeolus_core::{StylePlan, TailwindTheme, shadcn_css};

fn theme() -> TailwindTheme {
    TailwindTheme::from_theme_css(shadcn_css::LIGHT).unwrap()
}

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

#[test]
fn resolve_emits_a_parseable_plan_snapshot() {
    let (code, out, err) = run_cmd(&["resolve", "p-4", "bg-primary", "hover:bg-primary/90"]);
    assert_eq!(code, 0, "stderr: {err}");
    let plan = StylePlan::from_snapshot_json(out.trim()).expect("snapshot parses");
    assert!(plan.base.background.is_some());
    assert_eq!(plan.layers.len(), 1);
    assert!(plan.layers[0].style.background.is_some());
    // The artifact is canonical: re-resolving produces byte-identical output.
    let (code2, out2, _) = run_cmd(&["resolve", "hover:bg-primary/90", "bg-primary", "p-4"]);
    assert_eq!(code2, 0);
    assert_eq!(out2, out, "class order does not change the plan artifact");
}

#[test]
fn resolve_summary_lists_slots_and_layers() {
    let (code, out, _) = run_cmd(&[
        "resolve",
        "--summary",
        "p-4",
        "bg-primary",
        "md:p-8",
        "dark:bg-red-900",
    ]);
    assert_eq!(code, 0);
    assert!(out.contains("base:"), "{out}");
    assert!(out.contains("Background"), "{out}");
    assert!(out.contains("Padding"), "{out}");
    assert!(out.contains("responsive: md"), "{out}");
    assert!(out.contains("dark: yes"), "{out}");
}

#[test]
fn resolve_reports_warnings_on_stderr_with_exit_zero() {
    let (code, _out, err) = run_cmd(&["resolve", "p-4", "bg-notacolor"]);
    assert_eq!(code, 0, "warnings are non-fatal");
    assert!(err.contains("bg-notacolor"), "{err}");
    assert!(err.starts_with("warning:"), "{err}");
}

#[test]
fn resolve_theme_file_and_specs() {
    // Named specs work.
    let (code, out, err) = run_cmd(&["resolve", "--theme", "default", "p-4", "bg-black"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("\"format\":\"aeolus-plan\""));
    // A bad spec is a usage error (exit 2), reported precisely.
    let (code, _, err) = run_cmd(&["resolve", "--theme", "/nonexistent/theme.css", "p-4"]);
    assert_eq!(code, 2);
    assert!(err.contains("cannot read theme CSS"), "{err}");
}

#[test]
fn css_emits_the_verification_shapes() {
    let (code, out, err) = run_cmd(&["css", "p-4"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("padding:"), "{out}");
    assert!(out.contains("calc(var(--spacing) * 4)"), "{out}");
    let (code, out, _) = run_cmd(&["css", "bg-primary"]);
    assert_eq!(code, 0);
    assert!(out.contains("background-color"), "{out}");
}

#[test]
fn diff_reports_changed_slots_and_lanes() {
    // Each argument is ONE class string (quote it in a real shell).
    let (code, out, err) = run_cmd(&["diff", "p-4 bg-primary", "p-8 bg-primary"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("Padding"), "{out}");
    assert!(out.contains("geometric"), "{out}");
    assert!(out.contains("relayout"), "{out}");
    // JSON mode:
    let (code, out, _) = run_cmd(&["diff", "--json", "p-4", "p-4"]);
    assert_eq!(code, 0);
    assert!(out.contains("\"changed\":[]"), "{out}");
    // Cosmetic-only change:
    let (code, out, _) = run_cmd(&["diff", "bg-primary", "bg-red-500"]);
    assert_eq!(code, 0);
    assert!(out.contains("cosmetic"), "{out}");
    assert!(out.contains("repaint-only"), "{out}");
}

#[test]
fn cn_merges_and_accepts_multiple_groups() {
    // Later classes win their conflict group (tw_merge semantics): px-2 is
    // evicted by px-4; non-conflicting groups survive.
    let (code, out, _) = run_cmd(&["cn", "px-2 px-4", "rounded-md"]);
    assert_eq!(code, 0);
    let merged = out.trim();
    assert!(merged.contains("px-4"), "{merged}");
    assert!(merged.contains("rounded-md"), "{merged}");
    assert!(!merged.contains("px-2"), "{merged}");
    // A parent later in the string evicts its children (p-8 over px-4).
    let (code, out, _) = run_cmd(&["cn", "px-4", "p-8"]);
    assert_eq!(code, 0);
    assert_eq!(out.trim(), "p-8");
}

#[test]
fn check_flags_unknown_tokens_in_source_files() {
    let dir = std::env::temp_dir().join(format!(
        "aeolus-cli-test-{}-{}",
        std::process::id(),
        line!()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("widget.rs");
    std::fs::write(
        &file,
        r#"fn view() {
    let class_string = "p-4 bg-primary rounded-md hover:bg-notacolor";
    let prose = "The quick brown fox jumps over the lazy dog.";
}
"#,
    )
    .unwrap();
    let path = file.to_string_lossy().to_string();
    // Default fail-on=unknown: the unknown hover class fails the check.
    let (code, out, _) = run_cmd(&["check", &path]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("bg-notacolor"), "{out}");
    assert!(out.contains("1 finding(s)"), "{out}");
    // The prose string produced no findings.
    assert!(!out.contains("quick"), "{out}");
    // --fail-on none: findings reported, exit 0.
    let (code, out, _) = run_cmd(&["check", "--fail-on", "none", &path]);
    assert_eq!(code, 0, "{out}");
    // Clean file passes.
    let clean = dir.join("clean.rs");
    std::fs::write(&clean, r#"let s = "p-4 bg-primary";"#).unwrap();
    let (code, out, _) = run_cmd(&["check", &clean.to_string_lossy()]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("0 finding(s)"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn check_reports_conflicts() {
    let dir = std::env::temp_dir().join(format!(
        "aeolus-cli-test-{}-{}",
        std::process::id(),
        line!()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("c.rs");
    std::fs::write(&file, r#"let s = "px-2 px-4";"#).unwrap();
    let (code, out, _) = run_cmd(&["check", &file.to_string_lossy()]);
    // px-2 is dropped by cn() — with the default fail-on=unknown this is
    // NOT a failure (the classes are all known), but --fail-on conflict is.
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("px-2"), "{out}");
    assert!(out.contains("conflict"), "{out}");
    let (code, _, _) = run_cmd(&["check", "--fail-on", "conflict", &file.to_string_lossy()]);
    assert_eq!(code, 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn check_json_output() {
    let dir = std::env::temp_dir().join(format!(
        "aeolus-cli-test-{}-{}",
        std::process::id(),
        line!()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("j.rs");
    std::fs::write(&file, r#"let s = "bg-typosaurus";"#).unwrap();
    let (code, out, _) = run_cmd(&["check", "--json", &file.to_string_lossy()]);
    assert_eq!(code, 1, "{out}");
    assert!(out.starts_with('['), "{out}");
    assert!(out.contains("\"file\""), "{out}");
    assert!(out.contains("\"token\":\"bg-typosaurus\""), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn usage_version_and_errors() {
    let (code, out, _) = run_cmd(&["--help"]);
    assert_eq!(code, 0);
    assert!(out.contains("USAGE"));
    let (code, out, _) = run_cmd(&["--version"]);
    assert_eq!(code, 0);
    assert!(out.starts_with("aeolus "), "{out}");
    let (code, _, err) = run_cmd(&["frobnicate"]);
    assert_eq!(code, 2);
    assert!(err.contains("unknown command"), "{err}");
    let (code, _, _) = run_cmd(&[]);
    assert_eq!(code, 2);
    // Unknown flags are usage errors too.
    let (code, _, _) = run_cmd(&["cn", "--fancy", "p-4"]);
    assert_eq!(code, 2);
}

#[test]
fn extractor_finds_quoted_and_raw_strings() {
    let src = r##"let a = "p-4 bg-primary";
    let b = 'flex-col';
    let c = r#"gap-2 hover:bg-primary/90"#;
    let prose = "The quick brown fox";
    let no_class = "just some words here";
"##;
    let found = extract_candidate_strings(src);
    assert!(found.iter().any(|s| s == "p-4 bg-primary"), "{found:?}");
    assert!(found.iter().any(|s| s == "flex-col"), "{found:?}");
    assert!(found.iter().any(|s| s.contains("gap-2")), "{found:?}");
    // Prose is pre-filtered out of the extraction (the linter's input is
    // extractor output, so prose never reaches check_string).
    assert!(!found.iter().any(|s| s.contains("quick")), "{found:?}");
    assert_eq!(
        extract_candidate_strings("The quick brown fox jumps over the lazy dog"),
        [] as [std::string::String; 0]
    );
    // ...the extractor output feeds check_string; a real class string is
    // clean, and check_string is documented to take EXTRACTOR output only
    // (raw prose linted directly would flag every word as unknown).
    let t = theme();
    assert_eq!(
        check_string("p-4 bg-primary", &t),
        [] as [aeolus_cli::Finding; 0]
    );
}
