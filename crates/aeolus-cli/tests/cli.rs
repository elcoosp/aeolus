//! CLI behavior contract (v0.9 §3, TDD: these were written FIRST — the RED
//! capture is the initial "no such crate" compile failure, recorded in
//! `reports/10-tdd-evidence/09-portable-plans.md`).

use aeolus_cli::{ThemeSpecError, check_string, extract_candidate_strings, load_theme, run};
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

// ---------------------------------------------------------------------------
// Typed theme-spec errors (v0.9.1 consumer-handoff): `load_theme` is a
// PUBLIC seam of the headless surface; per the no-weak-errors rule it
// returns a typed error (not `String`) whose Display keeps the historical
// run() stderr contract (`cannot read theme CSS …`) pinned above.
// ---------------------------------------------------------------------------

#[test]
fn load_theme_reports_unknown_spec_as_typed_error() {
    let e = load_theme("nope").unwrap_err();
    match &e {
        ThemeSpecError::UnknownSpec { spec } => assert_eq!(spec, "nope"),
        other => panic!("expected UnknownSpec, got {other:?}"),
    }
    let msg = e.to_string();
    assert!(msg.contains("nope"), "names the offending spec: {msg}");
    assert!(
        msg.contains("shadcn") && msg.contains("default") && msg.contains("<path"),
        "display carries the accepted forms: {msg}"
    );
}

#[test]
fn load_theme_reports_missing_file_as_typed_io_error() {
    let e = load_theme("/nonexistent/theme.css").unwrap_err();
    match &e {
        ThemeSpecError::ReadFile { path, kind } => {
            assert_eq!(path, "/nonexistent/theme.css");
            assert_eq!(*kind, std::io::ErrorKind::NotFound);
        }
        other => panic!("expected ReadFile, got {other:?}"),
    }
    // The rendered text keeps the historical prefix the run() contract
    // (and resolve_theme_file_and_specs above) already pin.
    assert!(e.to_string().contains("cannot read theme CSS"), "{e}");
}

#[test]
fn load_theme_reports_unparseable_css_as_typed_parse_error() {
    let path = std::env::temp_dir().join("aeolus-cli-bad-theme.css");
    // A var() that never resolves is the documented theme-parse failure
    // (calc entries whose dependencies never appear are rejected).
    std::fs::write(
        &path,
        ":root { --primary: calc(var(--never-defined) * 2); }",
    )
    .unwrap();
    let e = load_theme(path.to_str().unwrap()).unwrap_err();
    match &e {
        ThemeSpecError::ParseCss { path, .. } => {
            assert!(path.ends_with("aeolus-cli-bad-theme.css"), "{path}");
        }
        other => panic!("expected ParseCss, got {other:?}"),
    }
    let _ = std::fs::remove_file(&path);
}

#[test]
fn load_theme_built_in_specs_still_resolve() {
    assert!(load_theme("shadcn").is_ok());
    assert!(load_theme("shadcn-dark").is_ok());
    assert!(load_theme("default").is_ok());
}

#[test]
fn run_reports_the_typed_theme_error_on_stderr() {
    let (code, _, err) = run_cmd(&["resolve", "--theme", "/nonexistent/theme.css", "p-4"]);
    assert_eq!(code, 2);
    assert!(err.contains("cannot read theme CSS"), "{err}");
}

#[cfg(feature = "miette")]
#[test]
fn theme_spec_error_carries_miette_help() {
    use miette::Diagnostic;
    let e = load_theme("nope").unwrap_err();
    let help = e.help().expect("unknown-spec carries help").to_string();
    assert!(
        help.contains("shadcn"),
        "help names the accepted specs: {help}"
    );
    assert!(e.code().is_some(), "every variant carries a code");
    let e = load_theme("/nonexistent/theme.css").unwrap_err();
    let help = e.help().expect("read-file carries help").to_string();
    assert!(
        help.contains("built-in"),
        "help suggests the fallback: {help}"
    );
}

#[cfg(feature = "tracing")]
#[test]
fn run_with_tracing_spans_keeps_the_contract() {
    // No global subscriber is installed: the spans compile and are no-ops.
    // The observable contract (exit code, stderr text) must be unchanged.
    let (code, out, err) = run_cmd(&["resolve", "p-4"]);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains("\"base\""), "snapshot artifact intact: {out}");
    let (code, _, err) = run_cmd(&["resolve", "--theme", "/nonexistent/theme.css", "p-4"]);
    assert_eq!(code, 2);
    assert!(err.contains("cannot read theme CSS"), "{err}");
}

// ---------------------------------------------------------------------------
// `explain` — the §5.5 tooling-parity slice: class → the Rust tokens the
// emit (a) arm renders, one fragment per lowered class, diagnostics for
// every class that does not (the honest-failure contract, CLI-shaped).
// TDD: written FIRST, the red is the initial "unknown command" run.
// ---------------------------------------------------------------------------

#[test]
fn explain_renders_one_fragment_per_lowered_class() {
    let (code, out, err) = run_cmd(&["explain", "p-4", "flex-col"]);
    assert_eq!(code, 0, "stderr: {err}");
    // One header per lowered class, source order, with the embeddable
    // Layer::new expression and its longhand names.
    assert!(out.contains("p-4"), "{out}");
    assert!(out.contains("flex-col"), "{out}");
    assert!(out.contains("Layer::new"), "{out}");
    assert!(out.contains("PaddingTop"), "{out}");
    assert!(out.contains("FlexDirection"), "{out}");
}

#[test]
fn explain_reports_misses_as_diagnostics_and_exits_1() {
    let (code, out, err) = run_cmd(&["explain", "p-4", "definitely-not-a-class-xyz"]);
    assert_eq!(code, 1, "stderr: {err}");
    assert!(out.contains("p-4"), "{out}");
    assert!(
        !out.contains("definitely-not-a-class-xyz"),
        "a miss is never a fragment: {out}"
    );
    assert!(err.contains("definitely-not-a-class-xyz"), "{err}");
    assert!(
        err.contains('['),
        "the diagnostic carries its stable code: {err}"
    );
}

#[test]
fn explain_theme_flag_lowers_colour_tokens() {
    let (code, out, err) = run_cmd(&["explain", "--theme", "shadcn", "bg-primary"]);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains("bg-primary"), "{out}");
    assert!(out.contains("Background"), "{out}");
}

#[test]
fn explain_without_theme_reports_the_token_boundary() {
    // The themeless emit keeps colour tokens a typed boundary whose
    // diagnostic names the lowering path — the CLI reports it as a
    // finding, never a silent drop.
    let (code, out, err) = run_cmd(&["explain", "bg-primary"]);
    assert_eq!(code, 1);
    assert!(err.contains("bg-primary"), "{err}");
    assert!(!out.contains("Background"), "{out}");
}

#[test]
fn explain_rejects_unknown_flags() {
    let (code, _, err) = run_cmd(&["explain", "--wat", "p-4"]);
    assert_eq!(code, 2);
    assert!(err.contains("unknown flag"), "{err}");
}

#[test]
fn explain_needs_a_class_string() {
    let (code, _, err) = run_cmd(&["explain"]);
    assert_eq!(code, 2);
    assert!(!err.is_empty());
}

// ---------------------------------------------------------------------------
// `sort` — the §5.5 tooling-parity slice: the registry's official order.
// ---------------------------------------------------------------------------

#[test]
fn sort_orders_by_the_registry() {
    let (code, out, err) = run_cmd(&["sort", "block", "p-4", "mx-2"]);
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(
        out.trim(),
        "p-4 mx-2 block",
        "the rows' progression decides"
    );
}

#[test]
fn sort_variants_after_the_base() {
    let (code, out, err) = run_cmd(&["sort", "hover:block", "block"]);
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(out.trim(), "block hover:block");
    // The cascade rank orders the buckets: dark < hover < breakpoint.
    let (code, out, _) = run_cmd(&["sort", "md:p-4", "hover:p-3", "dark:p-2", "p-1"]);
    assert_eq!(code, 0);
    assert_eq!(out.trim(), "p-1 dark:p-2 hover:p-3 md:p-4");
}

#[test]
fn sort_reports_unsortable_classes_and_exits_1() {
    let (code, out, err) = run_cmd(&["sort", "p-4", "not-a-class-xyz"]);
    assert_eq!(code, 1, "a typed miss is a finding, never a silent drop");
    // The sorted string still renders — the unknown rides the tail.
    assert_eq!(out.trim(), "p-4 not-a-class-xyz");
    assert!(err.contains("not-a-class-xyz"), "{err}");
    assert!(err.contains("aeolus::compile::unknown_root"), "{err}");
}

#[test]
fn sort_json_carries_the_sorted_string_and_diagnostics() {
    let (code, out, _) = run_cmd(&["sort", "--json", "p-4", "block"]);
    assert_eq!(code, 0);
    assert!(out.contains("\"sorted\":\"p-4 block\""), "{out}");
    assert!(out.contains("\"diagnostics\":[]"), "{out}");
    let (code, out, _) = run_cmd(&["sort", "--json", "p-4", "not-a-class-xyz"]);
    assert_eq!(code, 1);
    assert!(out.contains("\"sorted\":\"p-4 not-a-class-xyz\""), "{out}");
    assert!(out.contains("unknown_root"), "{out}");
}

#[test]
fn sort_rejects_unknown_flags() {
    let (code, _, err) = run_cmd(&["sort", "--wat", "p-4"]);
    assert_eq!(code, 2);
    assert!(err.contains("unknown flag"), "{err}");
}

#[test]
fn sort_needs_a_class_string() {
    let (code, _, err) = run_cmd(&["sort"]);
    assert_eq!(code, 2);
    assert!(err.contains("sort needs a class string"), "{err}");
}

#[test]
fn sort_empty_input_is_a_usage_error() {
    let (code, _, err) = run_cmd(&["sort", "   "]);
    assert_eq!(code, 2, "the compile seam's typed Empty error");
    assert!(err.contains("empty class string"), "{err}");
}

// ---------------------------------------------------------------------------
// `expand` — the interp arm's human-readable form (§3.2's CLI row): the
// EFFECTIVE declarations under one probe Env, keyword slots resolved to
// their row texts, the honest exclusions as notes.
// ---------------------------------------------------------------------------

#[test]
fn expand_renders_effective_declarations() {
    let (code, out, err) = run_cmd(&["expand", "hidden p-4"]);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains("display = none"), "{out}");
    assert!(out.contains("padding-top = 1rem"), "{out}");
}

#[test]
fn expand_resolves_keyword_slots_to_row_texts() {
    let (code, out, _) = run_cmd(&["expand", "flex-col"]);
    assert_eq!(code, 0);
    assert!(out.contains("flex-direction = column"), "{out}");
}

#[test]
fn expand_state_layers_do_not_fire_under_the_resting_env() {
    // hover is a probe the resting Env answers false — the layer does
    // not fire (the interp's documented semantics, not a drop).
    let (code, out, _) = run_cmd(&["expand", "hidden hover:block"]);
    assert_eq!(code, 0);
    assert!(out.contains("display = none"), "{out}");
    assert!(
        !out.contains("= block"),
        "the hover layer never fired: {out}"
    );
}

#[test]
fn expand_dark_flag_lifts_the_mode_tier() {
    let (code, out, _) = run_cmd(&["expand", "--dark", "hidden dark:block"]);
    assert_eq!(code, 0);
    assert!(out.contains("display = block"), "{out}");
}

#[test]
fn expand_theme_flag_lowers_colour_tokens() {
    let (code, out, _) = run_cmd(&["expand", "--theme", "shadcn", "bg-primary"]);
    assert_eq!(code, 0);
    assert!(out.contains("background-color = #"), "{out}");
}

#[test]
fn expand_reports_the_themeless_token_boundary() {
    let (code, _out, err) = run_cmd(&["expand", "bg-primary"]);
    assert_eq!(
        code, 1,
        "a typed boundary is a finding, never a silent drop"
    );
    assert!(err.contains("aeolus::compile::"), "{err}");
}

#[test]
fn expand_json_carries_declarations_and_notes() {
    let (code, out, _) = run_cmd(&["expand", "--json", "hidden"]);
    assert_eq!(code, 0);
    assert!(out.contains("\"declarations\""), "{out}");
    assert!(out.contains("\"display\""), "{out}");
    assert!(out.contains("\"notes\":[]"), "{out}");
}

#[test]
fn expand_reports_the_pseudo_element_note() {
    // `selection:` styles a different render target — excluded AND
    // noted (the interp's typed note, the no-silent-exclusion rule).
    let (code, out, err) = run_cmd(&["expand", "selection:hidden"]);
    assert_eq!(code, 0, "the note is a report, not a failure: {err}");
    assert!(out.contains("note:"), "{out}");
}

#[test]
fn expand_rejects_unknown_flags() {
    let (code, _, err) = run_cmd(&["expand", "--wat", "hidden"]);
    assert_eq!(code, 2);
    assert!(err.contains("unknown flag"), "{err}");
}

#[test]
fn expand_needs_a_class_string() {
    let (code, _, err) = run_cmd(&["expand"]);
    assert_eq!(code, 2);
    assert!(err.contains("expand needs a class string"), "{err}");
}
