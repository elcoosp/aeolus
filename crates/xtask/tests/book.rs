//! The docs book's contract battery — plan §8's P5 row and §5.8's
//! governance line are the plan source this pins:
//!
//! > gpui-rsx integration PR, LSP/CLI (…), gpui-component bridge, **docs
//! > book**, public benchmarks, `1.0-rc`
//!
//! (`docs/AEOLUS_V1_ARCHITECTURE.md` §8, the P5 row — the P5 phase's
//! earliest deliverable the tree did not carry: the docs lived as
//! scattered pages — README, the architecture plan, MIGRATION, the
//! generated REGISTRY/CONFORMANCE pages, the reports — with no single
//! ordered reader artifact, `find . -iname '*book*'` empty.)
//!
//! The book lands as the driver's own artifact family: the generated
//! chapters ride the `GeneratedArtifact` seam the other committed
//! artifacts follow, `cargo xtask gen-book` writes them, and `cargo
//! xtask verify-book` gates BOTH halves — the generated chapters' byte
//! drift AND the book's structural contract (every `SUMMARY.md` entry
//! resolves to a chapter file, every `{{#include}}` target exists) —
//! with the same exit-code contract as `verify-tables` (0 clean, 1
//! drift, 2 usage) and the same drift-pin discipline: a chapter edit
//! that forgets the regeneration fails HERE, in plain `cargo test`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use xtask::run;

/// The scratch-root counter: one unique temp root per test, cleaned up
/// on drop (the driver tests never mutate the real tree).
static SCRATCH: AtomicU32 = AtomicU32::new(0);

/// A writable stand-in workspace root holding a minimal book skeleton —
/// the negative tests' subject (the real tree is never touched).
struct Scratch {
    root: PathBuf,
}

impl Scratch {
    fn new(label: &str) -> Self {
        let n = SCRATCH.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "aeolus-book-{}-{n}",
            label.replace([' ', '/'], "-")
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs/book/src")).expect("scratch book dir");
        Self { root }
    }

    /// Write one file under the scratch root (workspace-relative path).
    fn write(&self, rel: &str, body: &str) {
        let path = self.root.join(rel);
        fs::create_dir_all(path.parent().expect("parent")).expect("parent dir");
        fs::write(path, body).expect("write");
    }

    /// Read one file back (workspace-relative path).
    fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.root.join(rel)).expect("read back")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// The minimal book skeleton the negative tests start from: a well-formed
/// `book.toml`, a two-entry `SUMMARY.md`, both chapters present.
fn skeleton(s: &Scratch) {
    s.write(
        "docs/book/book.toml",
        "[book]\ntitle = \"The Aeolus Book\"\nsrc = \"src\"\n",
    );
    s.write(
        "docs/book/src/SUMMARY.md",
        "# Summary\n\n[Introduction](introduction.md)\n\n- [The pipeline](pipeline.md)\n",
    );
    s.write(
        "docs/book/src/introduction.md",
        "# Introduction\n\nThe book.\n",
    );
    s.write(
        "docs/book/src/pipeline.md",
        "# The pipeline\n\nLex, fold, read, lower.\n",
    );
}

/// The two sinks the driver writes into — `run` takes `&mut dyn
/// io::Write`, the tests read the bytes back as text.
fn out_err() -> (Vec<u8>, Vec<u8>) {
    (Vec::new(), Vec::new())
}

/// The sink's text (the assertions' reading).
fn text(v: &[u8]) -> String {
    String::from_utf8_lossy(v).into_owned()
}

// ---------------------------------------------------------------------------
// The P5 row's committed book
// ---------------------------------------------------------------------------

/// The P5 row's acceptance criterion, pinned over the REAL tree: the
/// committed book passes the whole gate — the generated chapters byte-match
/// their regeneration and the structure walk finds nothing broken.
#[test]
fn verify_book_runs_clean_over_the_committed_book() {
    let (mut out, mut err) = out_err();
    let exit = run(["verify-book".into()], &mut out, &mut err);
    assert_eq!(
        exit,
        0,
        "verify-book over the committed book: {}",
        text(&err)
    );
    // The gate reports per-check lines and the summary count — the driver's
    // report shape (`name  path  clean`).
    assert!(text(&out).contains("book-structure"), "{}", text(&out));
    assert!(text(&out).contains("clean"), "{}", text(&out));
}

/// The generated CLI-reference chapter is the CLI's OWN usage text —
/// the single source the `aeolus-cli` crate carries, rendered into the
/// book's chapter path. The real tree's committed chapter is pinned
/// byte-equal to the regeneration (the drift-pin discipline).
#[test]
fn the_cli_reference_chapter_matches_the_cli_usage() {
    let (mut out, mut err) = out_err();
    let exit = run(["verify-book".into()], &mut out, &mut err);
    assert_eq!(exit, 0, "{}", text(&err));
    assert!(
        text(&out).contains("book-cli-reference"),
        "the generated chapter's report line: {}",
        text(&out)
    );
}

/// `gen-book` writes the generated chapters in write mode — proven on a
/// scratch root whose chapter file starts missing.
#[test]
fn gen_book_writes_the_generated_chapters() {
    let s = Scratch::new("gen-writes");
    skeleton(&s);
    let (mut out, mut err) = out_err();
    let exit = run(
        [
            "gen-book".into(),
            "--root".into(),
            s.root.display().to_string(),
        ],
        &mut out,
        &mut err,
    );
    assert_eq!(exit, 0, "{}", text(&err));
    assert!(text(&out).contains("wrote"), "{}", text(&out));
    let chapter = s.read("docs/book/src/cli-reference.md");
    assert!(chapter.contains("aeolus resolve"), "{chapter}");
}

// ---------------------------------------------------------------------------
// The gate catches the breakages
// ---------------------------------------------------------------------------

/// A `SUMMARY.md` entry pointing at a missing chapter file fails the gate.
#[test]
fn verify_book_catches_a_broken_summary_entry() {
    let s = Scratch::new("broken-summary");
    skeleton(&s);
    s.write(
        "docs/book/src/SUMMARY.md",
        "# Summary\n\n- [Missing chapter](nope.md)\n",
    );
    let (mut out, mut err) = out_err();
    let exit = run(
        [
            "verify-book".into(),
            "--root".into(),
            s.root.display().to_string(),
        ],
        &mut out,
        &mut err,
    );
    assert_eq!(exit, 1, "{}{}", text(&out), text(&err));
    assert!(
        text(&out).contains("nope.md"),
        "the report names the entry: {}",
        text(&out)
    );
}

/// A `{{#include}}` whose target does not exist fails the gate.
#[test]
fn verify_book_catches_a_missing_include_target() {
    let s = Scratch::new("broken-include");
    skeleton(&s);
    s.write(
        "docs/book/src/pipeline.md",
        "# The pipeline\n\n{{#include ../../NOPE.md}}\n",
    );
    let (mut out, mut err) = out_err();
    let exit = run(
        [
            "verify-book".into(),
            "--root".into(),
            s.root.display().to_string(),
        ],
        &mut out,
        &mut err,
    );
    assert_eq!(exit, 1, "{}{}", text(&out), text(&err));
    assert!(
        text(&out).contains("NOPE.md"),
        "the report names the target: {}",
        text(&out)
    );
}

/// A drifted generated chapter fails the gate (the byte-drift half).
#[test]
fn verify_book_catches_a_drifted_generated_chapter() {
    let s = Scratch::new("drifted-chapter");
    skeleton(&s);
    s.write("docs/book/src/cli-reference.md", "stale bytes");
    let (mut out, mut err) = out_err();
    let exit = run(
        [
            "verify-book".into(),
            "--root".into(),
            s.root.display().to_string(),
        ],
        &mut out,
        &mut err,
    );
    assert_eq!(exit, 1, "{}{}", text(&out), text(&err));
    assert!(
        text(&out).contains("DRIFTED"),
        "the report names the drift: {}",
        text(&out)
    );
}

/// A `book.toml` without the `[book]` title fails the gate — the config
/// the mdbook build needs is part of the structural contract.
#[test]
fn verify_book_catches_an_unusable_book_toml() {
    let s = Scratch::new("broken-config");
    skeleton(&s);
    s.write("docs/book/book.toml", "[book]\n");
    let (mut out, mut err) = out_err();
    let exit = run(
        [
            "verify-book".into(),
            "--root".into(),
            s.root.display().to_string(),
        ],
        &mut out,
        &mut err,
    );
    assert_eq!(exit, 1, "{}{}", text(&out), text(&err));
    assert!(
        text(&out).contains("title"),
        "the report names the gap: {}",
        text(&out)
    );
}

/// The book commands honour the driver's usage contract: no command or
/// an unknown command prints the usage and exits 2.
#[test]
fn the_book_commands_follow_the_usage_contract() {
    let (mut out, mut err) = out_err();
    let exit = run(Vec::<String>::new(), &mut out, &mut err);
    assert_eq!(exit, 2);
    assert!(
        text(&err).contains("gen-book"),
        "the usage lists the book pair: {}",
        text(&err)
    );
    assert!(
        text(&err).contains("verify-book"),
        "the usage lists the book pair: {}",
        text(&err)
    );

    let (mut out, mut err) = out_err();
    let exit = run(["gen-book-x".into()], &mut out, &mut err);
    assert_eq!(exit, 2);
}

/// The generated chapter's path is the book's own — the artifact commits
/// under `docs/book/src/`, never a second copy of another page.
#[test]
fn the_book_artifacts_commit_under_the_book_tree() {
    for artifact in xtask::book::artifacts() {
        let path = Path::new(artifact.path());
        assert!(
            path.starts_with("docs/book/"),
            "{}: the book artifacts live under docs/book/",
            artifact.name()
        );
    }
}
