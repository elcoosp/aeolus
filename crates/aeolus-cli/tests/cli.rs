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
fn expand_reports_named_containers_without_bounds() {
    // The sourceless shape: the named layer reports (the honest
    // boundary the fold's own pins carry), nothing fires.
    let (code, out, _) = run_cmd(&["expand", "@md/card:w-64"]);
    assert_eq!(code, 0);
    assert!(out.contains("never matched"), "{out}");
    assert!(!out.contains("width = 16rem"), "{out}");
}

#[test]
fn expand_container_bounds_answer_the_named_layers() {
    // The seam wired through the CLI: `card` at exactly md's 48rem —
    // the min form fires AT the boundary (the pinned CSS semantics).
    let (code, out, _) = run_cmd(&["expand", "--container", "card=48", "@md/card:w-64"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("width = 16rem"), "{out}");
}

#[test]
fn expand_container_bounds_miss_quietly_below_the_boundary() {
    // Below the named container's width the layer does not fire — and
    // a source-ANSWERED miss is CSS-quiet, not an exclusion note.
    let (code, out, _) = run_cmd(&["expand", "--container", "card=40", "@md/card:w-64"]);
    assert_eq!(code, 0);
    assert!(!out.contains("width = 16rem"), "{out}");
    assert!(
        !out.contains("never matched"),
        "the answered miss is a quiet skip: {out}"
    );
}

#[test]
fn expand_rejects_malformed_container_bounds() {
    let (code, _out, err) = run_cmd(&["expand", "--container", "card", "w-64"]);
    assert_eq!(code, 2, "a malformed bound is a CLI input error");
    assert!(err.contains("name=rem"), "{err}");
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

// ---------------------------------------------------------------------------
// The @source registration (plan §5.4's CSS-first compat line): a theme
// CSS PATH can register scan content — inline literals join the
// candidate set, path entries join the scan set (relative to the theme
// CSS's directory), negated paths prune — and the directives are typed
// end to end.
// ---------------------------------------------------------------------------

fn source_fixture_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("aeolus-cli-src-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn check_scans_source_inline_entries_from_the_theme_css() {
    let dir = source_fixture_dir("inline");
    let theme = dir.join("theme.css");
    std::fs::write(
        &theme,
        "@theme {\n  --color-primary: #0ea5e9;\n}\n@source inline(\"p-4 bg-notacolor\");\n",
    )
    .unwrap();
    let file = dir.join("widget.rs");
    std::fs::write(&file, "let s = \"p-4\";").unwrap();
    let theme_path = theme.to_string_lossy().to_string();
    let file_path = file.to_string_lossy().to_string();
    // The inline candidate is linted beside the passed files; the
    // finding names the theme CSS as its origin.
    let (code, out, _) = run_cmd(&["check", "--theme", &theme_path, &file_path]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("bg-notacolor"), "{out}");
    assert!(out.contains(&theme_path), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn check_source_path_entries_join_the_scan_set() {
    let dir = source_fixture_dir("path");
    let theme = dir.join("theme.css");
    std::fs::write(
        &theme,
        "@theme {\n  --color-primary: #0ea5e9;\n}\n@source \"widget.rs\";\n",
    )
    .unwrap();
    let file = dir.join("widget.rs");
    std::fs::write(&file, "let s = \"bg-typosaurus\";").unwrap();
    let anchor = dir.join("anchor.rs");
    std::fs::write(&anchor, "let s = \"p-4\";").unwrap();
    let theme_path = theme.to_string_lossy().to_string();
    let anchor_path = anchor.to_string_lossy().to_string();
    // The registered file is scanned even though only the anchor was
    // passed: the directive joins the scan set.
    let (code, out, _) = run_cmd(&["check", "--theme", &theme_path, &anchor_path]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("bg-typosaurus"), "{out}");
    assert!(out.contains("widget.rs"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn check_source_not_prunes_but_explicit_files_survive() {
    let dir = source_fixture_dir("not");
    let theme = dir.join("theme.css");
    std::fs::write(
        &theme,
        "@theme {\n  --color-primary: #0ea5e9;\n}\n@source \"widget.rs\";\n@source not \"widget.rs\";\n",
    )
    .unwrap();
    let file = dir.join("widget.rs");
    std::fs::write(&file, "let s = \"bg-typosaurus\";").unwrap();
    let anchor = dir.join("anchor.rs");
    std::fs::write(&anchor, "let s = \"p-4\";").unwrap();
    let theme_path = theme.to_string_lossy().to_string();
    let anchor_path = anchor.to_string_lossy().to_string();
    // The negation prunes the registration: no findings, exit 0.
    let (code, out, _) = run_cmd(&["check", "--theme", &theme_path, &anchor_path]);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("bg-typosaurus"), "{out}");
    // An EXPLICIT command-line file survives the negation (the
    // operator's list wins over the theme's prune).
    let file_path = file.to_string_lossy().to_string();
    let (code, out, _) = run_cmd(&["check", "--theme", &theme_path, &file_path]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("bg-typosaurus"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn check_reports_malformed_source_directives_as_usage_errors() {
    let dir = source_fixture_dir("malformed");
    let theme = dir.join("theme.css");
    std::fs::write(
        &theme,
        "@theme {\n  --color-primary: #0ea5e9;\n}\n@source \"unterminated.rs;\n",
    )
    .unwrap();
    let file = dir.join("widget.rs");
    std::fs::write(&file, "let s = \"p-4\";").unwrap();
    let theme_path = theme.to_string_lossy().to_string();
    let file_path = file.to_string_lossy().to_string();
    let (code, _, err) = run_cmd(&["check", "--theme", &theme_path, &file_path]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("unterminated"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn check_with_a_builtin_theme_never_scans_sources() {
    // The built-in specs carry no CSS path to register from: the
    // behavior is exactly the pre-@source surface.
    let dir = source_fixture_dir("builtin");
    let file = dir.join("widget.rs");
    std::fs::write(&file, "let s = \"bg-typosaurus\";").unwrap();
    let file_path = file.to_string_lossy().to_string();
    let (code, out, _) = run_cmd(&["check", "--theme", "shadcn", &file_path]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(out.matches("bg-typosaurus").count(), 1, "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// build (§2 approach C / the P6 row): the whole-project artifact
// ---------------------------------------------------------------------------

/// A fixture project for the build tests: one view file, one HTML
/// template, one ignored directory (target/), under one temp root.
fn build_fixture_dir(tag: &str) -> std::path::PathBuf {
    let dir = source_fixture_dir(tag);
    let src = dir.join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(
        src.join("view.rs"),
        "let a = tw!(\"flex flex-col gap-2 rounded-md bg-primary text-primary-foreground\");\nlet b = tw!(\"px-8\");\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("page.html"),
        "<button class=\"rounded-md px-8\">Save</button>\n",
    )
    .unwrap();
    // The walk never enters target/ — a vendored copy of the same
    // class strings that must not reach the artifact.
    std::fs::create_dir_all(dir.join("target")).unwrap();
    std::fs::write(
        dir.join("target").join("view.rs"),
        "let a = tw!(\"bg-not-a-token-ever\");\n",
    )
    .unwrap();
    dir
}

#[test]
fn build_renders_one_deduped_artifact_to_stdout() {
    let dir = build_fixture_dir("stdout");
    let root = dir.to_string_lossy().to_string();
    let (code, out, err) = run_cmd(&["build", &root]);
    assert_eq!(code, 0, "findings are non-fatal: {err}");
    // The global dedupe: px-8 appears in BOTH files, one rule.
    assert_eq!(out.matches(".px-8 {").count(), 1, "{out}");
    assert_eq!(out.matches(".rounded-md {").count(), 1, "{out}");
    // The themeless default: the colour token stays the typed boundary.
    assert!(err.contains("bg-primary"), "{err}");
    assert!(err.contains("src/view.rs"), "{err}");
    assert!(
        !out.contains(":root"),
        "no vars block without a table: {out}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_theme_renders_the_vars_block_and_lowers_tokens() {
    let dir = build_fixture_dir("theme");
    let root = dir.to_string_lossy().to_string();
    let (code, out, err) = run_cmd(&["build", "--theme", "shadcn-dark", &root]);
    assert_eq!(code, 0, "the tokens lowered: {err}");
    assert!(out.contains(":root {"), "{out}");
    assert!(out.contains("--color-primary:"), "{out}");
    assert!(out.contains(".dark {"), "{out}");
    assert!(out.contains("var(--color-primary)"), "{out}");
    assert_eq!(err, "", "no findings when the theme lowers everything");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_out_writes_the_artifact_and_reports_the_summary() {
    let dir = build_fixture_dir("out");
    let root = dir.to_string_lossy().to_string();
    let artifact = dir.join("out.css");
    let artifact_path = artifact.to_string_lossy().to_string();
    let (code, out, err) = run_cmd(&["build", "--out", &artifact_path, &root]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("rule(s)"), "{out}");
    let written = std::fs::read_to_string(&artifact).unwrap();
    assert!(written.contains(".px-8 {"), "{written}");
    assert!(written.contains(".flex {"), "{written}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_json_carries_the_summary_and_the_diagnostics() {
    let dir = build_fixture_dir("json");
    let root = dir.to_string_lossy().to_string();
    let artifact = dir.join("out.css");
    let artifact_path = artifact.to_string_lossy().to_string();
    let (code, out, err) = run_cmd(&["build", "--json", "--out", &artifact_path, &root]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(err, "", "under --json the findings ride the object");
    assert!(out.contains("\"files\":2"), "{out}");
    assert!(out.contains("\"rules\":"), "{out}");
    // The themeless boundary on the fixture's colour tokens rides the
    // object with its typed code (the keyword-table boundary the
    // themeless walk reports, per location).
    assert!(
        out.contains("\"code\":\"aeolus::compile::unsupported_value\""),
        "{out}"
    );
    // The artifact still rode --out beside the summary.
    let written = std::fs::read_to_string(&artifact).unwrap();
    assert!(written.contains(".px-8 {"), "{written}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_fail_on_maps_the_typed_codes() {
    let dir = build_fixture_dir("failon");
    let root = dir.to_string_lossy().to_string();
    // The themeless build's ONLY finding is the token boundary — an
    // UNSUPPORTED code, so `--fail-on unknown` passes (exit 0) while
    // `--fail-on unsupported` fails (exit 1) and `any` too.
    let (code, _, _) = run_cmd(&["build", "--fail-on", "unknown", &root]);
    assert_eq!(code, 0);
    let (code, _, _) = run_cmd(&["build", "--fail-on", "unsupported", &root]);
    assert_eq!(code, 1);
    let (code, _, _) = run_cmd(&["build", "--fail-on", "any", &root]);
    assert_eq!(code, 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_unknown_flag_and_missing_paths_are_usage_errors() {
    let (code, _, err) = run_cmd(&["build", "--nope", "x.rs"]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("--nope"), "{err}");
    let (code, _, err) = run_cmd(&["build"]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("at least one file"), "{err}");
    let (code, _, err) = run_cmd(&["build", "/nonexistent-path-aeolus"]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("cannot read"), "{err}");
}

#[test]
fn build_explicit_files_ignore_the_extension_filter() {
    // The filter applies to the DIRECTORY walk only: a file the
    // operator named rides verbatim (the check contract).
    let dir = build_fixture_dir("explicit");
    let other = dir.join("classes.txt");
    std::fs::write(&other, "let a = tw!(\"font-bold\");\n").unwrap();
    let other_path = other.to_string_lossy().to_string();
    let (code, out, _) = run_cmd(&["build", &other_path]);
    assert_eq!(code, 0);
    assert!(out.contains(".font-bold {"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// `plan` — the plan bridge's CLI form (the Target::Plan arm, the §7
// migration seam's consumer face). The battery was written FIRST; the
// RED capture is the "unknown command" dispatch failure.
// ---------------------------------------------------------------------------

fn plan_snapshot(args: &[&str]) -> StylePlan {
    let (code, out, err) = run_cmd(args);
    assert_eq!(code, 0, "stderr: {err}");
    StylePlan::from_snapshot_json(out.trim()).expect("the plan artifact parses as a snapshot")
}

#[test]
fn plan_emits_the_plan_bridge_snapshot() {
    let plan = plan_snapshot(&[
        "plan",
        "--theme",
        "shadcn",
        "p-4",
        "bg-primary",
        "hover:bg-primary/90",
    ]);
    // The same shape `resolve` emits: base slots, one state layer.
    assert!(plan.base.padding.is_some(), "p-4 lowers through the bridge");
    assert!(
        plan.base.background.is_some(),
        "bg-primary lowers through the bridge"
    );
    assert_eq!(plan.layers.len(), 1, "hover lands in the layers bucket");
    assert!(plan.layers[0].style.background.is_some());
}

#[test]
fn plan_artifact_is_canonical_across_class_order() {
    // The snapshot is the canonical artifact: class order cannot change it.
    let (code, out, err) = run_cmd(&["plan", "--theme", "shadcn", "p-4", "bg-primary"]);
    assert_eq!(code, 0, "{err}");
    let (code2, out2, _) = run_cmd(&["plan", "--theme", "shadcn", "bg-primary", "p-4"]);
    assert_eq!(code2, 0);
    assert_eq!(out2, out, "class order does not change the plan artifact");
}

#[test]
fn plan_themed_agrees_with_the_v0_resolver_slot_for_slot() {
    // The differential contract the plan arm's own battery pins: for a
    // themed string both generations lower the SAME plan shape.
    let v1 = plan_snapshot(&["plan", "--theme", "shadcn", "p-4", "hover:px-8"]);
    let v0 = {
        let (code, out, err) = run_cmd(&["resolve", "--theme", "shadcn", "p-4", "hover:px-8"]);
        assert_eq!(code, 0, "{err}");
        StylePlan::from_snapshot_json(out.trim()).expect("the resolve snapshot parses")
    };
    assert_eq!(v1.base.padding, v0.base.padding);
    assert_eq!(v1.layers.len(), v0.layers.len(), "one hover layer each");
    assert_eq!(
        v1.layers[0].style.padding, v0.layers[0].style.padding,
        "the hover px-8 slot agrees across the two generations"
    );
}

#[test]
fn plan_summary_lists_slots_layers_and_notes() {
    // place-content lowers into the IR but has no v0.x slot: the note
    // rides stderr, never the plan, and never silences.
    let (code, out, err) = run_cmd(&[
        "plan",
        "--theme",
        "shadcn",
        "--summary",
        "p-4",
        "md:p-8",
        "dark:bg-red-900",
        "place-content-center",
    ]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("base:"), "{out}");
    assert!(out.contains("Padding"), "{out}");
    assert!(out.contains("responsive: md"), "{out}");
    assert!(out.contains("dark: yes"), "{out}");
    assert!(
        err.contains("place-content-center"),
        "the note names the class: {err}"
    );
    assert!(
        err.contains("aeolus::compile::plan::"),
        "the note carries its stable code: {err}"
    );
}

#[test]
fn plan_reports_diagnostics_and_keeps_the_artifact_honest() {
    // Themeless: bg-primary's colour token is a typed boundary — it
    // reports, the snapshot stays without the background slot, and the
    // artifact still rides stdout (p-4 lowers).
    let (code, out, err) = run_cmd(&["plan", "p-4", "bg-primary"]);
    assert_eq!(code, 0, "findings are non-fatal without --fail-on: {err}");
    assert!(err.contains("bg-primary"), "{err}");
    let plan = StylePlan::from_snapshot_json(out.trim()).expect("snapshot parses");
    assert!(plan.base.padding.is_some());
    assert!(
        plan.base.background.is_none(),
        "the token boundary is honest"
    );
    // The unknown root reports too (`bg` is a registered root — the
    // suffix refuses, so the typed kind is the shape rejection, the
    // `FindingKind::Unknown` half of the gate).
    let (code, _, err) = run_cmd(&["plan", "p-4", "bg-notacolor"]);
    assert_eq!(code, 0);
    assert!(err.contains("bg-notacolor"), "{err}");
    assert!(err.contains("aeolus::compile::shape_rejected"), "{err}");
}

#[test]
fn plan_json_embeds_the_plan_and_the_reports() {
    let (code, out, err) = run_cmd(&["plan", "--theme", "shadcn", "--json", "p-4", "bg-primary"]);
    assert_eq!(code, 0, "{err}");
    assert!(
        out.starts_with("{\"plan\":{\"format\":\"aeolus-plan\""),
        "{out}"
    );
    assert!(out.contains("\"diagnostics\":[]"), "{out}");
    assert!(out.contains("\"notes\":[]"), "{out}");
    // The note channel rides the JSON envelope too.
    let (code, out, _) = run_cmd(&[
        "plan",
        "--theme",
        "shadcn",
        "--json",
        "place-content-center",
    ]);
    assert_eq!(code, 0);
    assert!(out.contains("slot_unsupported"), "{out}");
    assert!(
        !out.contains("\"diagnostics\":[{"),
        "a note is not a diagnostic: {out}"
    );
}

#[test]
fn plan_fail_on_maps_the_typed_codes() {
    let (code, _, _) = run_cmd(&["plan", "--fail-on", "unknown", "bg-notacolor"]);
    assert_eq!(
        code, 1,
        "an unknown root is a finding under --fail-on unknown"
    );
    let (code, _, _) = run_cmd(&["plan", "--fail-on", "none", "bg-notacolor"]);
    assert_eq!(code, 0, "default-off: the artifact still emitted");
    let (code, _, _) = run_cmd(&["plan", "--fail-on", "any", "!p-4"]);
    assert_eq!(
        code, 1,
        "the important-flag refusal is a finding under --fail-on any"
    );
}

#[test]
fn plan_usage_errors_are_precise() {
    // No classes: the usage error names the command.
    let (code, _, err) = run_cmd(&["plan"]);
    assert_eq!(code, 2);
    assert!(err.contains("plan needs a class string"), "{err}");
    // The V1 theme pass takes shadcn | shadcn-dark | <path>: the v0.x
    // `default` spec is a typed refusal.
    let (code, _, err) = run_cmd(&["plan", "--theme", "default", "p-4"]);
    assert_eq!(code, 2);
    assert!(err.contains("shadcn | shadcn-dark | <path>"), "{err}");
    let (code, _, err) = run_cmd(&["plan", "--bogus", "p-4"]);
    assert_eq!(code, 2);
    assert!(err.contains("unknown flag"), "{err}");
    let (code, _, err) = run_cmd(&["plan", "--theme", "/nonexistent/theme.css", "p-4"]);
    assert_eq!(code, 2);
    assert!(err.contains("cannot read theme CSS"), "{err}");
}

#[test]
fn plan_is_a_documented_command() {
    let (code, out, _) = run_cmd(&["--help"]);
    assert_eq!(code, 0);
    assert!(out.contains("aeolus plan"), "{out}");
    assert!(out.contains("plan bridge"), "{out}");
    // The FAIL-ON line now names the per-command defaults (the usage
    // drift fix: build/plan default `none`, check `unknown`).
    assert!(
        out.contains("default: check unknown; build/plan none"),
        "{out}"
    );
}

// ---------------------------------------------------------------------------
// fmt: the class-sort seam's file face — one file's class strings into
// the registry's canonical order, --check as the CI drift gate
// ---------------------------------------------------------------------------

/// A unique temp dir per test: the caller's own label keys the path
/// (the check battery's pid+line pattern, label-spelled — the helper's
/// own `line!()` would name every caller alike, and parallel tests
/// would share a directory).
fn temp_dir(label: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("aeolus-cli-fmt-{}-{label}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn fmt_rewrites_the_out_of_order_string_in_place() {
    let dir = temp_dir("fmt_rewrites");
    let file = dir.join("widget.rs");
    std::fs::write(
        &file,
        "fn view() {\n    let a = \"hover:flex-col p-4 flex-col\";\n    let b = \"p-4 flex-col\";\n}\n",
    )
    .unwrap();
    let path = file.to_string_lossy().to_string();
    let (code, out, err) = run_cmd(&["fmt", &path]);
    assert_eq!(code, 0, "a clean sort formats and passes: {err}");
    assert_eq!(
        out,
        "fn view() {\n    let a = \"p-4 flex-col hover:flex-col\";\n    let b = \"p-4 flex-col\";\n}\n",
        "the out-of-order string rewrites in place; the canonical string stays byte-identical"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fmt_preserves_the_rest_of_the_file_byte_for_byte() {
    let dir = temp_dir("fmt_preserves");
    let file = dir.join("widget.rs");
    let source = "// a comment, untouched\nfn view() {\n    let prose = \"the quick brown fox\";\n    let a = \"md:p-4 dark:bg-primary/90 bg-primary p-1 hover:bg-primary\";\n}\n";
    std::fs::write(&file, source).unwrap();
    let path = file.to_string_lossy().to_string();
    let (code, out, _) = run_cmd(&["fmt", &path]);
    assert_eq!(code, 0);
    // The multi-byte-safe expectation: only the string CONTENT's span
    // moves — the comment, the prose string and the structure ride
    // byte-for-byte.
    let expected = "// a comment, untouched\nfn view() {\n    let prose = \"the quick brown fox\";\n    let a = \"p-1 bg-primary dark:bg-primary/90 hover:bg-primary md:p-4\";\n}\n";
    assert_eq!(out, expected);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fmt_noops_a_clean_file() {
    let dir = temp_dir("fmt_noops");
    let file = dir.join("clean.rs");
    let source = "fn view() {\n    let a = \"p-4 flex-col\";\n}\n";
    std::fs::write(&file, source).unwrap();
    let path = file.to_string_lossy().to_string();
    let (code, out, err) = run_cmd(&["fmt", &path]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out, source, "a clean file prints byte-identical");
    assert!(err.is_empty(), "a clean file keeps stderr quiet: {err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fmt_check_reports_the_lines_and_exits_1() {
    let dir = temp_dir("fmt_check_reports");
    let file = dir.join("widget.rs");
    std::fs::write(
        &file,
        "fn view() {\n    let a = \"hover:flex-col p-4 flex-col\";\n}\n",
    )
    .unwrap();
    let path = file.to_string_lossy().to_string();
    let (code, out, _) = run_cmd(&["fmt", "--check", &path]);
    assert_eq!(code, 1, "an out-of-order string is the drift exit");
    // The report names the file, the ONE-BASED line (the lint
    // convention's caret form), the UTF-16 column of the string
    // content's start, and the before → after pair.
    assert!(
        out.contains(&format!("{path}:2:13")),
        "the content span's own position: {out}"
    );
    assert!(out.contains("hover:flex-col p-4 flex-col"), "{out}");
    assert!(out.contains("p-4 flex-col hover:flex-col"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fmt_check_passes_a_clean_file() {
    let dir = temp_dir("fmt_check_passes");
    let file = dir.join("clean.rs");
    std::fs::write(&file, "fn view() {\n    let a = \"p-4 flex-col\";\n}\n").unwrap();
    let path = file.to_string_lossy().to_string();
    let (code, out, err) = run_cmd(&["fmt", "--check", &path]);
    assert_eq!(code, 0, "{err}");
    assert!(out.is_empty(), "a clean check prints nothing: {out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fmt_reports_misses_on_stderr_without_failing() {
    let dir = temp_dir("fmt_reports_misses");
    let file = dir.join("tail.rs");
    std::fs::write(&file, "let s = \"justify-betwen p-4 not-a-class\";\n").unwrap();
    let path = file.to_string_lossy().to_string();
    let (code, out, err) = run_cmd(&["fmt", &path]);
    // The formatter's job is formatting: the honest tail rides source
    // order, the miss reports on stderr, the exit stays 0 (the lint
    // gate is `check`).
    assert_eq!(code, 0, "{err}");
    assert_eq!(out, "let s = \"p-4 justify-betwen not-a-class\";\n");
    assert!(err.contains("aeolus::compile::shape_rejected"), "{err}");
    assert!(err.contains("aeolus::compile::unknown_root"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fmt_needs_exactly_one_file() {
    let (code, _, err) = run_cmd(&["fmt"]);
    assert_eq!(code, 2);
    assert!(err.contains("fmt takes exactly one file"), "{err}");
    let dir = temp_dir("fmt_needs_exactly_one");
    let a = dir.join("a.rs");
    let b = dir.join("b.rs");
    std::fs::write(&a, "let s = \"p-4\";\n").unwrap();
    std::fs::write(&b, "let s = \"p-4\";\n").unwrap();
    let (code, _, err) = run_cmd(&["fmt", &a.to_string_lossy(), &b.to_string_lossy()]);
    assert_eq!(code, 2);
    assert!(err.contains("fmt takes exactly one file"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fmt_rejects_unknown_flags() {
    let (code, _, err) = run_cmd(&["fmt", "--wat", "x.rs"]);
    assert_eq!(code, 2);
    assert!(err.contains("unknown flag"), "{err}");
}

#[test]
fn fmt_reports_an_unreadable_file() {
    let (code, _, err) = run_cmd(&["fmt", "/nonexistent/path/widget.rs"]);
    assert_eq!(code, 2);
    assert!(err.contains("cannot read"), "{err}");
    assert!(err.contains("/nonexistent/path/widget.rs"), "{err}");
}

#[test]
fn fmt_is_a_documented_command() {
    let (code, out, _) = run_cmd(&["--help"]);
    assert_eq!(code, 0);
    assert!(out.contains("aeolus fmt"), "{out}");
    assert!(out.contains("canonical order"), "{out}");
}

// ---------------------------------------------------------------------------
// expand --has: the descendant-probe seam's CLI face
// ---------------------------------------------------------------------------

#[test]
fn expand_reports_descendant_layers_without_facts() {
    // The sourceless shape: the descendant layer reports (the honest
    // boundary the seam's own pins carry), nothing fires.
    let (code, out, _) = run_cmd(&["expand", "has-focus:p-8"]);
    assert_eq!(code, 0);
    assert!(out.contains("never matched"), "{out}");
    assert!(!out.contains("padding-top = 2rem"), "{out}");
}

#[test]
fn expand_descendant_facts_answer_the_layers() {
    // The seam wired through the CLI: the element's own subtree
    // matches `*:focus` — the layer fires, the cascade hands it the
    // longhands.
    let (code, out, _) = run_cmd(&["expand", "--has", "element=*:focus", "has-focus:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");
}

#[test]
fn expand_descendant_facts_discriminate_the_scope() {
    // The scope discriminates the question: the GROUP scope answers,
    // the PEER question does not ride the group answer — a
    // source-answered miss is a quiet skip, not an exclusion note.
    let (code, out, _) = run_cmd(&["expand", "--has", "group=*:focus", "peer-has-focus:p-8"]);
    assert_eq!(code, 0);
    assert!(!out.contains("padding-top = 2rem"), "{out}");
    assert!(
        !out.contains("never matched"),
        "the answered miss is a quiet skip: {out}"
    );
}

#[test]
fn expand_rejects_malformed_has_facts() {
    let (code, _out, err) = run_cmd(&["expand", "--has", "element", "w-64"]);
    assert_eq!(code, 2, "a malformed fact is a CLI input error");
    assert!(err.contains("kind=fragment"), "{err}");

    let (code, _out, err) = run_cmd(&["expand", "--has", "ancestor=*:focus", "w-64"]);
    assert_eq!(code, 2, "an unknown kind is a CLI input error");
    assert!(err.contains("element|group|peer"), "{err}");
}

// ---------------------------------------------------------------------------
// expand --supports: the supports-variant seam's CLI face
// ---------------------------------------------------------------------------

#[test]
fn expand_reports_supports_layers_without_facts() {
    // The sourceless shape: the supports layer reports (the honest
    // boundary the seam's own pins carry), nothing fires.
    let (code, out, _) = run_cmd(&["expand", "supports-[display:grid]:p-8"]);
    assert_eq!(code, 0);
    assert!(out.contains("never matched"), "{out}");
    assert!(!out.contains("padding-top = 2rem"), "{out}");
}

#[test]
fn expand_supports_facts_answer_the_layers() {
    // The seam wired through the CLI: the render target supports the
    // grid probe — the layer fires, the cascade hands it the
    // longhands. The fact normalizes through the fold's own grammar:
    // the bare spelling and the paren-wrapped spelling land the SAME
    // payload the variant folded.
    let (code, out, _) = run_cmd(&[
        "expand",
        "--supports",
        "(display:grid)",
        "supports-[display:grid]:p-8",
    ]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");

    let (code, out, _) = run_cmd(&[
        "expand",
        "--supports",
        "display:grid",
        "supports-[display:grid]:p-8",
    ]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");
}

#[test]
fn expand_supports_facts_answer_the_function_form() {
    // The function form rides the same payload equality: the fact's
    // text answers the `supports-[selector(:hover)]` layer, and the
    // compound the table does NOT carry is an answered miss — a quiet
    // skip, not a note.
    let (code, out, _) = run_cmd(&[
        "expand",
        "--supports",
        "selector(:hover)",
        "supports-[selector(:hover)]:p-8 supports-[hover:hover]:p-8",
    ]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");
    assert!(
        !out.contains("never matched"),
        "the answered miss is a quiet skip: {out}"
    );
}

#[test]
fn expand_rejects_malformed_supports_facts() {
    // The fact grammar is the fold's own (one grammar, two consumers):
    // an empty, brace-carrying or unbalanced condition is a CLI input
    // error, exit 2, the refusal names the grammar.
    let (code, _out, err) = run_cmd(&["expand", "--supports", "", "w-64"]);
    assert_eq!(code, 2, "an empty condition is a CLI input error");
    assert!(err.contains("condition"), "{err}");

    let (code, _out, err) = run_cmd(&["expand", "--supports", "(display:grid", "w-64"]);
    assert_eq!(code, 2, "an unbalanced condition is a CLI input error");
    assert!(err.contains("condition"), "{err}");
}

// ---------------------------------------------------------------------------
// expand --reduced-motion etc.: the preference families' CLI face
// ---------------------------------------------------------------------------

#[test]
fn expand_preference_flags_answer_the_layers() {
    // The media-feature facts are frame state: the flags set the Env
    // bits, the layers fire (the --dark precedent).
    let (code, out, _) = run_cmd(&["expand", "--reduced-motion", "motion-reduce:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--print", "print:hidden"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("display = none"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--forced-colors", "forced-colors:hidden"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("display = none"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--more-contrast", "contrast-more:ring-2"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("--tw-ring"), "{out}");
}

#[test]
fn expand_without_preference_flags_is_a_quiet_miss() {
    // The default Env answers every preference question false — the
    // media query does not hold, the layer misses QUIETLY (never a
    // reported exclusion).
    let (code, out, _) = run_cmd(&["expand", "motion-reduce:p-8"]);
    assert_eq!(code, 0);
    assert!(!out.contains("padding-top = 2rem"), "{out}");
    assert!(
        !out.contains("never matched"),
        "a quiet miss reports nothing: {out}"
    );
}

#[test]
fn expand_motion_safe_asks_the_complement() {
    // The user agent's no-preference answer: motion-safe fires while
    // the reduced-motion bit is off, misses when --reduced-motion sets
    // it.
    let (code, out, _) = run_cmd(&["expand", "motion-safe:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--reduced-motion", "motion-safe:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("padding-top = 2rem"), "{out}");
}

// ---------------------------------------------------------------------------
// expand --pointer-fine etc.: the capability families' CLI face
// ---------------------------------------------------------------------------

#[test]
fn expand_capability_flags_answer_the_layers() {
    // The capability facts are frame state like the preferences: the
    // flags set the Env bits, the layers fire (the --reduced-motion
    // precedent).
    let (code, out, _) = run_cmd(&["expand", "--pointer-fine", "pointer-fine:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");

    let (code, out, _) = run_cmd(&[
        "expand",
        "--any-pointer-coarse",
        "any-pointer-coarse:hidden",
    ]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("display = none"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--pointer-none", "pointer-none:hidden"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("display = none"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--portrait", "portrait:p-4"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 1rem"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--any-pointer-fine", "any-pointer-fine:ring-2"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("--tw-ring"), "{out}");
}

#[test]
fn expand_without_capability_flags_is_a_quiet_miss() {
    // The default Env asserts no capability fact — the pointer queries
    // do not hold, the layers miss QUIETLY (never a reported
    // exclusion).
    let (code, out, _) = run_cmd(&["expand", "pointer-fine:p-8"]);
    assert_eq!(code, 0);
    assert!(!out.contains("padding-top = 2rem"), "{out}");
    assert!(
        !out.contains("never matched"),
        "a quiet miss reports nothing: {out}"
    );
}

#[test]
fn expand_landscape_asks_the_complement() {
    // The orientation query always holds an answer: landscape fires
    // while the portrait bit is off (the resting env's own answer),
    // misses when --portrait sets it.
    let (code, out, _) = run_cmd(&["expand", "landscape:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--portrait", "landscape:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("padding-top = 2rem"), "{out}");
}

#[test]
fn expand_portrait_asks_the_complement_pair() {
    // The same fact from the portrait side: --portrait fires the
    // portrait layer, the resting env misses it.
    let (code, out, _) = run_cmd(&["expand", "--portrait", "portrait:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "portrait:p-8"]);
    assert_eq!(code, 0);
    assert!(!out.contains("padding-top = 2rem"), "{out}");
}

// ---------------------------------------------------------------------------
// expand --rtl: the direction family's CLI face
// ---------------------------------------------------------------------------

#[test]
fn expand_rtl_flag_answers_the_layers() {
    // The direction fact is frame state like the capability bits: the
    // flag sets the Env bit, the rtl layers fire (the --portrait
    // precedent; one flag per FACT — `ltr:` answers from the same bit).
    let (code, out, _) = run_cmd(&["expand", "--rtl", "rtl:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--rtl", "rtl:hidden"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("display = none"), "{out}");
}

#[test]
fn expand_without_rtl_flag_is_a_quiet_miss() {
    // The default Env asserts no rtl fact — the direction is not rtl,
    // the rtl layers miss QUIETLY (never a reported exclusion).
    let (code, out, _) = run_cmd(&["expand", "rtl:p-8"]);
    assert_eq!(code, 0);
    assert!(!out.contains("padding-top = 2rem"), "{out}");
    assert!(
        !out.contains("never matched"),
        "a quiet miss reports nothing: {out}"
    );
}

#[test]
fn expand_ltr_answers_the_complement() {
    // The direction query always holds an answer: ltr fires while the
    // rtl bit is off (the resting env's own answer — CSS's initial
    // `direction` is ltr, the landscape precedent), misses when --rtl
    // sets it.
    let (code, out, _) = run_cmd(&["expand", "ltr:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--rtl", "ltr:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("padding-top = 2rem"), "{out}");
}

// ---------------------------------------------------------------------------
// expand --invalid/--user-valid/--user-invalid: the validation family's
// CLI face
// ---------------------------------------------------------------------------

#[test]
fn expand_invalid_flag_answers_the_layers() {
    // The constraint fact is element state like the state bits: the
    // flag sets the Env bit, the invalid layers fire (the --rtl
    // precedent; one flag per SPELLING — `valid:` answers from the
    // same bit).
    let (code, out, _) = run_cmd(&["expand", "--invalid", "invalid:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--invalid", "invalid:hidden"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("display = none"), "{out}");
}

#[test]
fn expand_without_validation_flags_is_a_quiet_miss() {
    // The default Env asserts no failing-constraints fact — the
    // invalid layers miss QUIETLY (never a reported exclusion).
    let (code, out, _) = run_cmd(&["expand", "invalid:p-8"]);
    assert_eq!(code, 0);
    assert!(!out.contains("padding-top = 2rem"), "{out}");
    assert!(
        !out.contains("never matched"),
        "a quiet miss reports nothing: {out}"
    );
}

#[test]
fn expand_valid_answers_the_complement() {
    // The constraint query always holds an answer: valid fires while
    // the failing bit is off (the resting env's own answer — CSS's
    // pristine-field reading is :valid, the rtl/ltr precedent),
    // misses when --invalid sets it.
    let (code, out, _) = run_cmd(&["expand", "valid:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--invalid", "valid:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("padding-top = 2rem"), "{out}");
}

#[test]
fn expand_user_flags_answer_past_the_gate() {
    // The `user-` pair gates on the interaction fact the HTML spec
    // requires: at the resting env BOTH spellings miss quietly; the
    // flags assert the gate AND the matching constraint fact, past it
    // the pair answers from the same fact the plain pair shares.
    let (code, out, _) = run_cmd(&["expand", "user-valid:p-8"]);
    assert_eq!(code, 0);
    assert!(
        !out.contains("padding-top = 2rem"),
        "no gate, no fire: {out}"
    );

    let (code, out, _) = run_cmd(&["expand", "--user-valid", "user-valid:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--user-invalid", "user-invalid:hidden"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("display = none"), "{out}");

    // The gate alone is not the constraint fact: --user-valid asserts
    // (gate AND not-failing), so the user-invalid layer misses.
    let (code, out, _) = run_cmd(&["expand", "--user-valid", "user-invalid:hidden"]);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("display = none"), "{out}");
}

// ---------------------------------------------------------------------------
// expand --target/--indeterminate/--autofill/--read-write: the element
// state family's CLI face
// ---------------------------------------------------------------------------

#[test]
fn expand_target_flag_answers_the_layers() {
    // The fragment-target fact is element state like the state bits: the
    // flag sets the Env bit, the target layers fire (the --invalid
    // precedent; one flag per FACT).
    let (code, out, _) = run_cmd(&["expand", "--target", "target:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--target", "target:hidden"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("display = none"), "{out}");
}

#[test]
fn expand_without_element_state_flags_is_a_quiet_miss() {
    // The default Env asserts no element-state fact — the target and
    // autofill layers miss QUIETLY (never a reported exclusion).
    let (code, out, _) = run_cmd(&["expand", "target:p-8"]);
    assert_eq!(code, 0);
    assert!(!out.contains("padding-top = 2rem"), "{out}");
    assert!(
        !out.contains("never matched"),
        "a quiet miss reports nothing: {out}"
    );

    let (code, out, _) = run_cmd(&["expand", "autofill:p-8"]);
    assert_eq!(code, 0);
    assert!(!out.contains("padding-top = 2rem"), "{out}");
}

#[test]
fn expand_indeterminate_flag_answers_the_layers() {
    // The indeterminate fact is the checkbox/radio's own answer: the
    // flag sets the Env bit, the indeterminate layers fire.
    let (code, out, _) = run_cmd(&["expand", "--indeterminate", "indeterminate:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");
}

#[test]
fn expand_autofill_flag_answers_the_layers() {
    // The autofill fact is the browser's filled-form answer: the flag
    // sets the Env bit, the autofill layers fire.
    let (code, out, _) = run_cmd(&["expand", "--autofill", "autofill:hidden"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("display = none"), "{out}");
}

#[test]
fn expand_read_only_answers_the_complement() {
    // The editability query always holds an answer: read-only fires
    // while the read-write fact is off (the resting env's own answer —
    // CSS's reading for a non-editable element is :read-only, the
    // valid/ltr complement precedent), misses when --read-write sets
    // the editability fact.
    let (code, out, _) = run_cmd(&["expand", "read-only:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--read-write", "read-only:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("padding-top = 2rem"), "{out}");
}

// ---------------------------------------------------------------------------
// The widget-state battery: the family's headless facts (M2)
// ---------------------------------------------------------------------------

#[test]
fn expand_open_flag_answers_the_layers() {
    // The disclosure/popover fact is element state like the state bits:
    // the flag sets the Env bit, the open layers fire (the --target
    // precedent; one flag per FACT).
    let (code, out, _) = run_cmd(&["expand", "--open", "open:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--open", "open:hidden"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("display = none"), "{out}");
}

#[test]
fn expand_inert_flag_answers_the_layers() {
    // The frozen-subtree fact is the per-element answer the compound
    // `:is([inert], [inert] *)` head reads: the flag sets the Env bit,
    // the inert layers fire.
    let (code, out, _) = run_cmd(&["expand", "--inert", "inert:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");
}

#[test]
fn expand_default_flag_answers_the_layers() {
    // The form group's default-control fact is the `:default` answer
    // the host widget tracks beside checked: the flag sets the Env
    // bit, the default layers fire.
    let (code, out, _) = run_cmd(&["expand", "--default", "default:hidden"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("display = none"), "{out}");
}

#[test]
fn expand_without_widget_state_flags_is_a_quiet_miss() {
    // The default Env asserts no widget-state fact — every family
    // question is AFFIRMATIVE, so the layers miss QUIETLY (never a
    // reported exclusion; the element-state battery's own shape).
    let (code, out, _) = run_cmd(&["expand", "open:p-8"]);
    assert_eq!(code, 0);
    assert!(!out.contains("padding-top = 2rem"), "{out}");
    assert!(
        !out.contains("never matched"),
        "a quiet miss reports nothing: {out}"
    );

    let (code, out, _) = run_cmd(&["expand", "inert:hidden"]);
    assert_eq!(code, 0);
    assert!(!out.contains("display = none"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "default:hidden"]);
    assert_eq!(code, 0);
    assert!(!out.contains("display = none"), "{out}");
}

#[test]
fn expand_widget_state_flags_mix_with_the_facts() {
    // The family rides the same Env the other fact flags build: the
    // widget-state flags compose with the base layer (the facts'
    // cascade, the element-state battery's own shape).
    let (code, out, _) = run_cmd(&["expand", "--open", "--default", "p-4 open:p-8 default:mt-2"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");
    assert!(out.contains("margin-top = 0.5rem"), "{out}");
}

// ---------------------------------------------------------------------------
// The form-state battery: the family's headless facts (M2)
// ---------------------------------------------------------------------------

#[test]
fn expand_form_associated_flag_answers_the_enabled_layer() {
    // The form-membership fact gates the matches-neither halves: the
    // flag sets the Env bit, the enabled layer fires on the membership
    // beside the unset disabled bit (the --target precedent; one flag
    // per FACT).
    let (code, out, _) = run_cmd(&["expand", "--form-associated", "enabled:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");
}

#[test]
fn expand_required_flags_answer_the_layer() {
    // The required half rides BOTH facts — the membership beside the
    // required attribute: the flags together fire the layer, the
    // attribute flag ALONE never fires it (a non-form element matches
    // neither half, the matches-neither boundary the family closes).
    let (code, out, _) = run_cmd(&[
        "expand",
        "--form-associated",
        "--required",
        "required:hidden",
    ]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("display = none"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--required", "required:hidden"]);
    assert_eq!(code, 0);
    assert!(!out.contains("display = none"), "{out}");
}

#[test]
fn expand_optional_flag_answers_the_unmarked_field() {
    // The optional half is the required attribute's complement WITHIN
    // the form universe: the membership flag alone fires it, the
    // attribute beside it keeps it quiet (the `validation_invalid`
    // precedent — within the membership the pair shares one state
    // fact).
    let (code, out, _) = run_cmd(&["expand", "--form-associated", "optional:mt-2"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("margin-top = 0.5rem"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--form-associated", "--required", "optional:mt-2"]);
    assert_eq!(code, 0);
    assert!(!out.contains("margin-top = 0.5rem"), "{out}");
}

#[test]
fn expand_range_flags_answer_the_layers() {
    // The range pair rides the bounded universe's membership beside
    // the value's position: bounded + within fires `in-range:`,
    // bounded alone fires its `out-of-range:` complement.
    let (code, out, _) = run_cmd(&["expand", "--range-bounded", "--range-in", "in-range:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--range-bounded", "out-of-range:hidden"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("display = none"), "{out}");
}

#[test]
fn expand_without_form_state_flags_is_a_quiet_miss() {
    // The default Env asserts no form-state fact — every family
    // question gates on the membership, so the layers miss QUIETLY
    // (never a reported exclusion; the widget-state battery's own
    // shape).
    for class in [
        "enabled:p-8",
        "required:hidden",
        "optional:mt-2",
        "in-range:p-8",
        "out-of-range:hidden",
    ] {
        let (code, out, _) = run_cmd(&["expand", class]);
        assert_eq!(code, 0, "{out}");
        assert!(
            !out.contains("never matched"),
            "a quiet miss reports nothing: {out}"
        );
    }

    let (code, out, _) = run_cmd(&["expand", "enabled:p-8"]);
    assert_eq!(code, 0);
    assert!(!out.contains("padding-top = 2rem"), "{out}");
}

#[test]
fn expand_form_state_flags_mix_with_the_facts() {
    // The family rides the same Env the other fact flags build: the
    // form-state flags compose with the base layer and with each
    // other (the facts' cascade; the bounded value's in-bounds bit
    // keeps the out-of-range layer quiet while the in-range layer
    // fires).
    let (code, out, _) = run_cmd(&[
        "expand",
        "--form-associated",
        "--range-bounded",
        "--range-in",
        "p-4 enabled:p-8 in-range:mt-2 out-of-range:hidden",
    ]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");
    assert!(out.contains("margin-top = 0.5rem"), "{out}");
    assert!(!out.contains("display = none"), "{out}");
}

// ---------------------------------------------------------------------------
// The pseudo-element tier: the `--pseudo` render-target face
// ---------------------------------------------------------------------------

#[test]
fn expand_pseudo_resolves_the_target_cascade() {
    // The element walk reports the pseudo layer as its exclusion note;
    // the `--pseudo` face answers the render target's OWN cascade (the
    // interp_pseudo arm) — the section rides after the element answer.
    let (code, out, err) = run_cmd(&["expand", "--pseudo", "before", "before:p-8 p-4"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("padding-top = 1rem"), "{out}");
    assert!(out.contains("note:"), "{out}");
    // The pseudo section: the target's own effective declarations.
    assert!(out.contains("::before"), "{out}");
    let pseudo_half = out.rsplit("::before").next().unwrap_or("");
    assert!(pseudo_half.contains("padding-top = 2rem"), "{out}");
}

#[test]
fn expand_pseudo_answers_the_whole_upstream_set() {
    for spelling in [
        "selection",
        "placeholder",
        "before",
        "after",
        "marker",
        "first-letter",
        "first-line",
        "file",
        "backdrop",
        "details-content",
    ] {
        let (code, out, err) =
            run_cmd(&["expand", "--pseudo", spelling, &format!("{spelling}:mt-2")]);
        assert_eq!(code, 0, "{spelling}: {err}");
        assert!(
            out.contains(&format!("::{spelling} ")),
            "{spelling}: the target header names the spelling: {out}"
        );
        assert!(
            out.contains("margin-top = 0.5rem"),
            "{spelling}: the target's own value: {out}"
        );
    }
}

#[test]
fn expand_pseudo_refuses_the_unknown_target_as_a_usage_error() {
    let (code, _out, err) = run_cmd(&["expand", "--pseudo", "no-such-target", "p-4"]);
    assert_eq!(code, 2, "usage errors exit 2");
    assert!(err.contains("--pseudo"), "{err}");
}

#[test]
fn expand_pseudo_json_carries_the_target_answer() {
    let (code, out, err) = run_cmd(&["expand", "--json", "--pseudo", "file", "file:p-8"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("\"pseudo\""), "{out}");
    assert!(out.contains("\"target\":\"file\""), "{out}");
    assert!(out.contains("padding-top"), "{out}");
}

// ---------------------------------------------------------------------------
// The media-feature remainder: the `--inverted-colors`/`--noscript` flags
// ---------------------------------------------------------------------------

#[test]
fn expand_media_remainder_flags_answer_the_layers() {
    // The frame facts answer their layers beside the resting env's
    // quiet miss (the capability flags' sibling — one flag per FACT).
    let (code, out, err) = run_cmd(&["expand", "--inverted-colors", "inverted-colors:p-8"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("padding-top = 2rem"), "{out}");

    let (code, out, err) = run_cmd(&["expand", "--noscript", "noscript:mt-2"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("margin-top = 0.5rem"), "{out}");
}

#[test]
fn expand_media_remainder_unset_flags_stay_the_quiet_miss() {
    let (code, out, _) = run_cmd(&["expand", "inverted-colors:p-8 noscript:mt-2"]);
    assert_eq!(code, 0);
    assert!(
        !out.contains("padding-top = 2rem") && !out.contains("margin-top = 0.5rem"),
        "an unset frame fact is the quiet miss a media query that does not hold is: {out}"
    );
    assert!(!out.contains("note:"), "no exclusion note rides: {out}");
}

// ---------------------------------------------------------------------------
// The of-type structural tier: the `--child-of-type` facts flag
// ---------------------------------------------------------------------------

#[test]
fn expand_child_of_type_answers_the_of_type_layers() {
    // The type-relative facts answer their layers beside the resting
    // walk's reported boundary (the `--child` flag's sibling — the
    // facts value the StructProbe seam documents, the type dimension
    // beside the child-index one).
    let (code, out, err) = run_cmd(&[
        "expand",
        "--child",
        "1/3",
        "--child-of-type",
        "0/2",
        "first-of-type:w-64 first:hidden",
    ]);
    assert_eq!(code, 0, "{err}");
    // The of-type layer fires (type index 0); the child-index layer
    // quietly misses (index 1 is not the first child) — the two
    // dimensions ride their own facts.
    assert!(out.contains("width = 16rem"), "{out}");
    assert!(!out.contains("display = none"), "{out}");

    let (code, out, err) = run_cmd(&[
        "expand",
        "--child",
        "2/3",
        "--child-of-type",
        "1/3",
        "nth-of-type-2:w-10 nth-last-of-type-2:p-2",
    ]);
    assert_eq!(code, 0, "{err}");
    // The second of three same-type siblings: `nth-of-type-2` fires
    // (1-based), and so does `nth-last-of-type-2` (the second from the
    // type's end of three).
    assert!(out.contains("width = 2.5rem"), "{out}");
    assert!(out.contains("padding-top = 0.5rem"), "{out}");
}

#[test]
fn expand_child_of_type_unset_stays_the_reported_boundary() {
    let (code, out, _) = run_cmd(&["expand", "first-of-type:w-64"]);
    assert_eq!(code, 0);
    assert!(
        !out.contains("width = 16rem"),
        "no type facts, no fire: {out}"
    );
    assert!(
        out.contains("note:"),
        "the sourceless form reports the structural boundary: {out}"
    );
}

// ---------------------------------------------------------------------------
// The `starting:` variant: the `--starting` fact flag
// ---------------------------------------------------------------------------

#[test]
fn expand_starting_flag_answers_the_layer() {
    // The frame fact answers its layer beside the resting env's quiet
    // miss (the capability flags' sibling — one flag per FACT).
    let (code, out, err) = run_cmd(&["expand", "--starting", "starting:p-8"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("padding-top = 2rem"), "{out}");
}

#[test]
fn expand_starting_unset_stays_the_quiet_miss() {
    let (code, out, _) = run_cmd(&["expand", "starting:p-8"]);
    assert_eq!(code, 0);
    assert!(
        !out.contains("padding-top = 2rem"),
        "an unset frame fact is the quiet miss a frame the element is not in is: {out}"
    );
    assert!(!out.contains("note:"), "no exclusion note rides: {out}");
}

// ---------------------------------------------------------------------------
// The interaction-state battery: the founding element-state family's
// CLI face (the `--hover`/`--focus`/`--active`… flags)
// ---------------------------------------------------------------------------

#[test]
fn expand_hover_flag_answers_the_layers() {
    // The hover fact is the element's own answer: the flag sets the
    // Env bit, the hover layers fire (one flag per FACT, the
    // `--indeterminate` precedent).
    let (code, out, err) = run_cmd(&["expand", "--hover", "p-4 hover:p-8"]);
    assert_eq!(code, 0, "{err}");
    // The fired layer wins the walk's per-prop read (the base 1rem is
    // replaced, never duplicated — the winner read the interp keeps).
    assert!(out.contains("padding-top = 2rem"), "{out}");
    assert!(!out.contains("padding-top = 1rem"), "{out}");
}

#[test]
fn expand_focus_flags_answer_the_layers() {
    // The focus pair and the within sibling: each flag asserts its own
    // fact, its layers fire beside the resting base.
    for (flag, class) in [
        ("--focus", "focus:p-8"),
        ("--focus-visible", "focus-visible:p-8"),
        ("--focus-within", "focus-within:p-8"),
    ] {
        let (code, out, err) = run_cmd(&["expand", flag, class]);
        assert_eq!(code, 0, "{flag}: {err}");
        assert!(out.contains("padding-top = 2rem"), "{flag}: {out}");
    }
}

#[test]
fn expand_active_disabled_flags_answer_the_layers() {
    // The pressed and disabled facts: the flag sets the Env bit, the
    // layer fires.
    for (flag, class) in [("--active", "active:p-8"), ("--disabled", "disabled:p-8")] {
        let (code, out, err) = run_cmd(&["expand", flag, class]);
        assert_eq!(code, 0, "{flag}: {err}");
        assert!(out.contains("padding-top = 2rem"), "{flag}: {out}");
    }
}

#[test]
fn expand_visited_checked_placeholder_flags_answer_the_layers() {
    // The visited, checked and placeholder-shown facts: the remaining
    // element-own bits, one flag per FACT (the closed Env set).
    for (flag, class) in [
        ("--visited", "visited:p-8"),
        ("--checked", "checked:p-8"),
        ("--placeholder-shown", "placeholder-shown:p-8"),
    ] {
        let (code, out, err) = run_cmd(&["expand", flag, class]);
        assert_eq!(code, 0, "{flag}: {err}");
        assert!(out.contains("padding-top = 2rem"), "{flag}: {out}");
    }
}

#[test]
fn expand_without_interaction_flags_stay_the_quiet_miss() {
    // The default Env asserts no interaction fact — the founding
    // family's layers miss QUIETLY (never a reported exclusion; the
    // element-state battery's own shape, unchanged by the flags).
    for class in [
        "hover:p-8",
        "focus:p-8",
        "active:p-8",
        "disabled:p-8",
        "placeholder-shown:p-8",
    ] {
        let (code, out, _) = run_cmd(&["expand", class]);
        assert_eq!(code, 0, "{out}");
        assert!(
            !out.contains("never matched"),
            "a quiet miss reports nothing: {out}"
        );
    }

    let (code, out, _) = run_cmd(&["expand", "hover:p-8"]);
    assert_eq!(code, 0);
    assert!(!out.contains("padding-top = 2rem"), "{out}");
}

#[test]
fn expand_interaction_flags_mix_with_the_base_and_each_other() {
    // The flags ride the same Env the other fact flags build: the
    // interaction layers compose with the base layer and with each
    // other (two asserted facts fire two layers; the unasserted one
    // waits).
    let (code, out, _) = run_cmd(&[
        "expand",
        "--hover",
        "--focus",
        "p-4 hover:p-8 focus:mt-2 active:hidden",
    ]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");
    assert!(out.contains("margin-top = 0.5rem"), "{out}");
    assert!(!out.contains("display = none"), "{out}");
}

// ---------------------------------------------------------------------------
// The breakpoint battery: the viewport-width fact's CLI face (the
// `--width rem` flag)
// ---------------------------------------------------------------------------

#[test]
fn expand_width_flag_answers_the_min_breakpoints() {
    // The viewport width is the frame fact the breakpoint families
    // answer from (`Env::viewport_rem`, the AT/strictly-below boundary
    // rules the interp pins): the flag carries the width in rem, the
    // min forms fire AT and above their breakpoint's rem.
    for (bp, rem, class) in [
        ("sm", "40", "sm:p-8"),
        ("md", "48", "md:p-8"),
        ("lg", "64", "lg:p-8"),
        ("xl", "80", "xl:p-8"),
        ("2xl", "96", "2xl:p-8"),
    ] {
        let (code, out, err) = run_cmd(&["expand", "--width", rem, class]);
        assert_eq!(code, 0, "{bp}: {err}");
        assert!(out.contains("padding-top = 2rem"), "{bp}: {out}");
    }
}

#[test]
fn expand_width_below_the_breakpoint_is_the_quiet_miss() {
    // Strictly below the breakpoint the min form misses — the same
    // quiet miss an unset state fact is (a media query that does not
    // hold reports nothing). The resting env (no flag) stays the
    // zero-width viewport: no min breakpoint fires.
    let (code, out, _) = run_cmd(&["expand", "--width", "39.9", "sm:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("padding-top = 2rem"), "{out}");
    assert!(
        !out.contains("never matched"),
        "a quiet miss reports nothing: {out}"
    );

    let (code, out, _) = run_cmd(&["expand", "md:p-8"]);
    assert_eq!(code, 0);
    assert!(!out.contains("padding-top = 2rem"), "{out}");
    assert!(!out.contains("never matched"), "{out}");
}

#[test]
fn expand_width_max_forms_answer_strictly_below() {
    // The max forms read the SAME fact off the complementary boundary:
    // strictly below the breakpoint the layer fires, at it the layer
    // stops (the interp's `Cond::Max` arm, the AT boundary's other
    // edge).
    let (code, out, err) = run_cmd(&["expand", "--width", "39.9", "max-md:p-8"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("padding-top = 2rem"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--width", "48", "max-md:p-8"]);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("padding-top = 2rem"), "{out}");
    assert!(
        !out.contains("never matched"),
        "a quiet miss reports nothing: {out}"
    );
}

#[test]
fn expand_width_composes_with_the_state_flags() {
    // The width rides the same Env the other fact flags build: the
    // breakpoint layer joins the cascade (responsive ranks AFTER the
    // state tier, the sort order's own slots) — at the md width the
    // breakpoint's answer wins the padding longhand, below it the
    // hover state's does, and the unasserted xl layer waits either
    // way.
    let (code, out, _) = run_cmd(&[
        "expand",
        "--width",
        "48",
        "--hover",
        "p-4 md:p-8 hover:p-12 xl:p-16",
    ]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 2rem"), "{out}");
    assert!(!out.contains("padding-top = 4rem"), "{out}");

    let (code, out, _) = run_cmd(&["expand", "--hover", "p-4 md:p-8 hover:p-12 xl:p-16"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("padding-top = 3rem"), "{out}");
    assert!(!out.contains("padding-top = 2rem"), "{out}");
}

#[test]
fn expand_width_json_carries_the_answer() {
    // The JSON shape carries the width-fired layer like any effective
    // declaration (the `--json` face reads the same walk).
    let (code, out, err) = run_cmd(&["expand", "--json", "--width", "48", "md:p-8"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("\"prop\":\"padding-top\""), "{out}");
    assert!(out.contains("\"value\":\"2rem\""), "{out}");
}

#[test]
fn expand_width_usage_errors_are_the_honest_refusal() {
    // The width rides the walk's own f32 slot: a malformed, negative
    // or non-finite spelling is a CLI input error (exit 2), never a
    // half-answered layer (the `--container` arm's honest-refusal
    // shape).
    for args in [
        vec!["expand", "--width"],               // missing value
        vec!["expand", "--width", "abc", "p-4"], // unparseable
        vec!["expand", "--width", "-4", "p-4"],  // negative
        vec!["expand", "--width", "inf", "p-4"], // non-finite
    ] {
        let (code, _, err) = run_cmd(&args);
        assert_eq!(code, 2, "args {args:?}: {err}");
        assert!(err.contains("--width"), "the refusal names the flag: {err}");
    }
}

#[test]
fn expand_width_is_the_expand_face_only() {
    // The flag is the expand face's (the allow-list closed set): the
    // other class-string commands refuse it as an unknown flag, the
    // usage-error exit.
    let (code, _, err) = run_cmd(&["resolve", "--width", "48", "p-4"]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("unknown flag"), "{err}");
}
