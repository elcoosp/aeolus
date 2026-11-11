//! The driver's own contract tests — the `run(args, out, err) -> i32`
//! surface (the aeolus-cli precedent), every exit code, and the
//! write-against-a-temp-root path (a test never mutates the real
//! tree; the real tree's drift-free state is pinned read-only by the
//! final test).

use std::path::PathBuf;
use std::process::Command;

use xtask::run;

fn argv(args: &[&str]) -> Vec<String> {
    args.iter().map(|a| (*a).to_string()).collect()
}

/// A unique temp root per test (the artifact paths resolve against
/// it); removed on drop so a failed assert does not leak the tree.
struct TempRoot(PathBuf);

impl TempRoot {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "xtask-driver-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock is after the epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("temp root creates");
        Self(dir)
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn usage_errors_exit_two_with_the_usage_text() {
    let mut out = Vec::new();
    let mut err = Vec::new();
    assert_eq!(run(argv(&[]), &mut out, &mut err), 2);
    assert!(out.is_empty());
    assert!(String::from_utf8(err).expect("utf8").contains("USAGE"));
}

#[test]
fn help_exits_zero_on_stdout() {
    let mut out = Vec::new();
    let mut err = Vec::new();
    assert_eq!(run(argv(&["--help"]), &mut out, &mut err), 0);
    assert!(err.is_empty());
    assert!(String::from_utf8(out).expect("utf8").contains("gen-tables"));
}

#[test]
fn unknown_commands_exit_two() {
    for cmd in ["bogus", "gen", "verify"] {
        let mut out = Vec::new();
        let mut err = Vec::new();
        assert_eq!(run(argv(&[cmd]), &mut out, &mut err), 2, "{cmd}");
        assert!(
            String::from_utf8(err)
                .expect("utf8")
                .contains("unknown command")
        );
    }
}

#[test]
fn unexpected_flags_exit_two() {
    let mut out = Vec::new();
    let mut err = Vec::new();
    assert_eq!(run(argv(&["list", "--check"]), &mut out, &mut err), 2);
    let mut out = Vec::new();
    let mut err = Vec::new();
    assert_eq!(run(argv(&["gen-tables", "--bogus"]), &mut out, &mut err), 2);
}

#[test]
fn list_names_every_artifact_and_path() {
    let mut out = Vec::new();
    let mut err = Vec::new();
    assert_eq!(run(argv(&["list"]), &mut out, &mut err), 0);
    let text = String::from_utf8(out).expect("utf8");
    assert!(text.contains("roots-index"), "{text}");
    assert!(
        text.contains("crates/aeolus-data/src/roots_index.rs"),
        "{text}"
    );
    assert!(text.contains("registry-docs"), "{text}");
    assert!(text.contains("docs/REGISTRY.md"), "{text}");
}

#[test]
fn write_mode_populates_a_temp_root_and_verify_stays_clean() {
    let temp = TempRoot::new("write-then-verify");
    let root = temp.0.to_string_lossy().to_string();

    // Write mode against an empty root: both artifacts land, exit 0.
    let mut out = Vec::new();
    let mut err = Vec::new();
    assert_eq!(
        run(argv(&["gen-tables", "--root", &root]), &mut out, &mut err),
        0
    );
    let text = String::from_utf8(out).expect("utf8");
    assert!(text.contains("wrote "), "{text}");
    assert!(
        temp.0
            .join("crates/aeolus-data/src/roots_index.rs")
            .exists()
    );
    assert!(temp.0.join("docs/REGISTRY.md").exists());

    // Verify mode over the same root: clean, exit 0.
    let mut out = Vec::new();
    let mut err = Vec::new();
    assert_eq!(
        run(
            argv(&["verify-tables", "--root", &root]),
            &mut out,
            &mut err
        ),
        0
    );
    let text = String::from_utf8(out).expect("utf8");
    assert_eq!(text.matches("clean").count(), 2, "{text}");
}

#[test]
fn verify_reports_drift_with_the_drift_exit_code() {
    let temp = TempRoot::new("drift");
    let root = temp.0.to_string_lossy().to_string();

    let mut out = Vec::new();
    let mut err = Vec::new();
    run(argv(&["gen-tables", "--root", &root]), &mut out, &mut err);

    // Corrupt one committed byte: verify must catch exactly that file.
    let index_path = temp.0.join("crates/aeolus-data/src/roots_index.rs");
    let text = std::fs::read_to_string(&index_path).expect("index written");
    std::fs::write(
        &index_path,
        text.replacen("PHF_SEED: u32", "PHF_SEED: u8 ", 1),
    )
    .expect("corruption writes");

    let mut out = Vec::new();
    let mut err = Vec::new();
    assert_eq!(
        run(
            argv(&["verify-tables", "--root", &root]),
            &mut out,
            &mut err
        ),
        1
    );
    let text = String::from_utf8(out).expect("utf8");
    assert!(text.contains("DRIFTED"), "{text}");
    assert!(text.contains("roots-index"), "{text}");
    assert!(text.contains("clean"), "{text}: the sibling stays clean");
}

#[test]
fn verify_reports_missing_files_as_drift() {
    let temp = TempRoot::new("missing");
    let root = temp.0.to_string_lossy().to_string();
    let mut out = Vec::new();
    let mut err = Vec::new();
    assert_eq!(
        run(
            argv(&["gen-tables", "--check", "--root", &root]),
            &mut out,
            &mut err
        ),
        1
    );
    let text = String::from_utf8(out).expect("utf8");
    assert_eq!(text.matches("MISSING").count(), 2, "{text}");
}

#[test]
fn the_committed_workspace_verifies_clean() {
    // The real tree, read-only: the same check `cargo xtask
    // verify-tables` runs as the pre-commit drift gate.
    let bin = std::env!("CARGO_BIN_EXE_xtask");
    let output = Command::new(bin)
        .arg("verify-tables")
        .output()
        .expect("driver runs");
    assert!(
        output.status.success(),
        "the committed artifacts drifted — run `cargo xtask gen-tables` (stdout: {})",
        String::from_utf8_lossy(&output.stdout)
    );
}
