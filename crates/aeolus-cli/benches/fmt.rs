//! The `fmt` command's microbenchmarks (divan, stable — the workspace
//! bench idiom): the class-sort seam's FILE face, one file's class
//! strings through the extraction seam and the themeless sort walk.
//! Two arms pin the cost split: the whole-file walk (every string
//! through the extraction seam, each through the sort — the cost a
//! format or --check pass pays per file) and the single-string sort at
//! the file-face's real input shape (the shadcn Button slice the CLI
//! benches key on).

use aeolus_compile::{scan_rust_strings, sort};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The Button slice, verbatim (the shared fixture the pipeline benches
/// key on).
const BUTTON: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2";

/// A synthetic `widget.rs` the way `tw!` call sites look: three class
/// strings (one variant-heavy, one clean, one with a whitespace run),
/// the fmt walk's input.
const WIDGET: &str = "fn view() {\n    let a = \"hover:flex-col p-4 flex-col\";\n    let b = \"inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap outline-none\";\n    let c = \"gap-2  p-4\";\n}\n";

/// The whole-file walk: the extraction seam's strings, each through
/// the sort — the edit list the rewrite applies (and the --check
/// report prints).
#[divan::bench]
fn fmt_file_walk(bencher: Bencher) {
    bencher.bench(|| -> Vec<(usize, usize, String)> {
        let mut edits = Vec::new();
        for hit in scan_rust_strings(WIDGET) {
            let Ok(sorted) = sort(&hit.text) else {
                continue;
            };
            let canonical = sorted.to_class_string();
            if canonical != hit.text {
                edits.push((hit.offset, hit.offset + hit.text.len(), canonical));
            }
        }
        edits
    });
}

/// The single-string sort at the file-face's real input shape: the
/// 17-class Button slice through the registry's read side.
#[divan::bench]
fn fmt_button_string(bencher: Bencher) {
    bencher.bench(|| sort(BUTTON).expect("non-empty input sorts"));
}
