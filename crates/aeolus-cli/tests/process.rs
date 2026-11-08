//! Process-level end-to-end tests: the REAL binary, real pipes, real exit
//! codes (`tests/cli.rs` drives the library `run()` directly; this file
//! pins the layer above it — `main()`'s delegation, OS exit codes, and
//! stdout/stderr separation through actual pipes).
//!
//! The binary is `aeolus` (`[[bin]] name = "aeolus"`); Cargo exposes it to
//! integration tests as `CARGO_BIN_EXE_aeolus`.

use std::process::{Command, Output};

#[derive(Debug)]
struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str]) -> Run {
    let Output { status, stdout, stderr } = Command::new(env!("CARGO_BIN_EXE_aeolus"))
        .args(args)
        .output()
        .expect("spawn the aeolus binary");
    Run {
        code: status.code().unwrap_or(-1),
        stdout: String::from_utf8(stdout).expect("stdout is utf-8"),
        stderr: String::from_utf8(stderr).expect("stderr is utf-8"),
    }
}

/// `resolve` writes the plan snapshot artifact to STDOUT as valid JSON with
/// the current schema header, and exits 0.
#[test]
fn resolve_stdout_is_a_current_schema_snapshot() {
    let r = run(&["resolve", "bg-red-500", "p-4"]);
    assert_eq!(r.code, 0, "stderr: {}", r.stderr);
    assert_eq!(
        r.stderr, "",
        "warnings go to stderr; a clean class list must print none"
    );
    assert!(
        r.stdout.starts_with("{\"format\":\"aeolus-plan\",\"version\":2,"),
        "stdout must be the v2 snapshot artifact, got: {}",
        &r.stdout[..r.stdout.len().min(60)]
    );
    // The artifact parses back through the STRICT reader — the same
    // contract the snapshot round-trip tests pin, now through the binary.
    // (Cheap structural check here; the strict parse is covered by
    // tests/cli.rs against run() — this pins the BYTES match that path.)
    assert!(r.stdout.contains("\"base\""));
    assert!(r.stdout.contains("\"breakpoints\":[640,768,1024,1280,1536]"));
}

/// The CLI is deterministic: the same arguments produce byte-identical
/// stdout across runs (the artifact is the contract — build systems and
/// caches may hash it).
#[test]
fn resolve_is_byte_deterministic_across_processes() {
    let a = run(&["resolve", "hover:bg-blue-500", "dark:p-4", "@md:gap-2"]);
    let b = run(&["resolve", "hover:bg-blue-500", "dark:p-4", "@md:gap-2"]);
    assert_eq!(a.code, 0);
    assert_eq!(a.stdout, b.stdout, "equal inputs must hash-equal");
}

/// Warnings are NON-fatal: an unknown class resolves the rest, prints the
/// warning to STDERR (never stdout — the artifact stays parseable) and
/// exits 0.
#[test]
fn warnings_go_to_stderr_and_stay_non_fatal() {
    let r = run(&["resolve", "bg-red-500", "bg-notacolor"]);
    assert_eq!(r.code, 0, "warnings are non-fatal");
    assert!(
        r.stdout.starts_with("{\"format\":\"aeolus-plan\""),
        "stdout remains a parseable artifact"
    );
    assert!(
        r.stderr.contains("bg-notacolor"),
        "the warning names the offending token, got: {r:?}"
    );
    // The artifact still contains the good class.
    assert!(r.stdout.contains("\"background\""));
}

/// Every command answers its exit contract: success 0, unknown flag 2,
/// unknown command 2, missing `--theme` value 2. Exit codes are the
/// scripting contract (`CI`, `make`, `just`).
#[test]
fn exit_contracts_through_the_real_process() {
    assert_eq!(run(&["resolve", "p-4"]).code, 0);
    assert_eq!(run(&["css", "p-4"]).code, 0);
    assert_eq!(run(&["diff", "p-4", "p-8"]).code, 0);
    assert_eq!(run(&["cn", "p-4", "p"]).code, 0);
    assert_eq!(run(&["--version"]).code, 0);
    // Usage errors: unknown flag, unknown command, missing value.
    let r = run(&["resolve", "--bogus", "p-4"]);
    assert_eq!(r.code, 2);
    assert!(r.stderr.contains("unknown flag"), "{r:?}");
    let r = run(&["frobnicate"]);
    assert_eq!(r.code, 2);
    assert!(r.stderr.contains("unknown command"), "{r:?}");
    let r = run(&["resolve", "--theme"]);
    assert_eq!(r.code, 2);
    assert!(r.stderr.contains("--theme needs a value"), "{r:?}");
    // No arguments at all: usage on stderr, exit 2.
    assert_eq!(run(&[]).code, 2);
}

/// `diff` is lane-aware through the binary too: padding (geometric) is
/// reported with its lane, color (cosmetic) with its.
#[test]
fn diff_reports_lanes_through_the_process() {
    let r = run(&["diff", "bg-red-500", "bg-blue-500"]);
    assert_eq!(r.code, 0, "{r:?}");
    assert!(r.stdout.contains("Background"), "{r:?}");
    assert!(r.stdout.contains("cosmetic"), "{r:?}");

    let r = run(&["diff", "p-4", "p-8"]);
    assert_eq!(r.code, 0, "{r:?}");
    assert!(r.stdout.contains("Padding"), "{r:?}");
    assert!(r.stdout.contains("geometric"), "{r:?}");
}

/// `cn` merges conflicts and keeps the winner through the process boundary.
#[test]
fn cn_merge_is_stable_through_the_process() {
    let r = run(&["cn", "p-4", "p-8"]);
    assert_eq!(r.code, 0, "{r:?}");
    assert_eq!(r.stdout.trim(), "p-8", "later class wins the merge");
    // Same-family non-conflicting classes survive both.
    let r = run(&["cn", "p-4", "m-2"]);
    assert_eq!(r.code, 0);
    let out = r.stdout.trim();
    assert!(out.contains("p-4") && out.contains("m-2"), "{out:?}");
}

/// `check` exits 1 when findings exist and 0 on a clean file — the lint
/// gate contract — and names the offending token on stdout (findings are
/// the PRODUCT, not errors).
#[test]
fn check_exit_1_with_findings_and_zero_clean() {
    let dir = std::env::temp_dir().join(format!("aeolus-e2e-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("mkdir");
    let dirty = dir.join("dirty.html");
    std::fs::write(&dirty, "<div class=\"p-4 bg-notacolor\">x</div>").expect("write");
    let clean = dir.join("clean.html");
    std::fs::write(&clean, "<div class=\"p-4\">x</div>").expect("write");

    let dirty_s = dirty.to_str().unwrap();
    let clean_s = clean.to_str().unwrap();
    let r = run(&["check", clean_s]);
    assert_eq!(r.code, 0, "{r:?}");
    let r = run(&["check", dirty_s]);
    assert_eq!(r.code, 1, "findings → exit 1");
    assert!(r.stdout.contains("bg-notacolor"), "{r:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// `--theme` selects the theme family; a bogus theme file is a usage-level
/// error (exit 2), not a panic.
#[test]
fn theme_selection_and_error_paths() {
    let r = run(&["resolve", "--theme", "default", "p-4"]);
    assert_eq!(r.code, 0, "{r:?}");
    let r = run(&["resolve", "--theme", "/nonexistent/theme.css", "p-4"]);
    assert_eq!(r.code, 2, "a missing theme file is an IO error");
    assert!(!r.stderr.is_empty(), "the error is explained");
}

/// A large class list (the shadcn Button table plus variants) resolves in
/// one process without truncation — the artifact carries every layer.
#[test]
fn large_class_list_resolves_completely() {
    let button = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap outline-none focus-visible:ring-[3px] disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 selection:bg-primary has-checked:bg-green-500";
    let r = run(&["resolve", button]);
    assert_eq!(r.code, 0, "{r:?}");
    // Layers present: hover, focus-visible, disabled + the has-data rule.
    assert!(r.stdout.contains("\"hover\""), "{r:?}");
    assert!(r.stdout.contains("\"focus_visible\""));
    assert!(r.stdout.contains("\"disabled\""));
    // The v0.7 shapes survive the process boundary.
    assert!(r.stdout.contains("\"pseudo\""), "selection: pseudo-element test");
    assert!(r.stdout.contains("\"has\""), "has-checked rule");
}
