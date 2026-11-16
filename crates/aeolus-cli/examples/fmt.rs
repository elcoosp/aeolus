//! Consumer demonstration: the `fmt` command's file face (the class-sort
//! seam's batch form, beside `sort`'s literal-string form and the LSP's
//! `textDocument/formatting` editor face) — one file's class strings
//! into the registry's canonical order, driven in-process the way a
//! format-on-save hook or a CI drift gate would. The `--check` shape is
//! the same walk's report: one `path:line:col "before" -> "after"` line
//! per out-of-order string, exit 1 the drift gate maps.

use aeolus_compile::{scan_rust_strings, sort};

/// A synthetic source the way a component file looks: one out-of-order
/// string (the variant bucket rides after the base in the canonical
/// order), one already-canonical string, one whitespace run the
/// canonical join normalizes.
const SOURCE: &str = r#"fn card() {
    let a = "hover:flex-col p-4 flex-col";
    let b = "p-4 flex-col";
    let c = "gap-2  p-4";
}
"#;

/// The `(line, column-UTF16)` of one byte offset (the report's one-based
/// caret form).
fn line_character(text: &str, offset: usize) -> (usize, usize) {
    let head = &text[..offset];
    let line = head.matches('\n').count() + 1;
    let line_start = head.rfind('\n').map_or(0, |i| i + 1);
    let character = head[line_start..]
        .chars()
        .map(char::len_utf16)
        .sum::<usize>();
    (line, character)
}

fn main() {
    // The fmt walk: the extraction seam's strings, each through the
    // themeless sort — the order is a shape-level fact (the sort
    // module's own boundary), so the formatter needs no table.
    println!("── the drift report (the --check shape) ──");
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    for hit in scan_rust_strings(SOURCE) {
        let Ok(sorted) = sort(&hit.text) else {
            continue;
        };
        let canonical = sorted.to_class_string();
        if canonical == hit.text {
            continue;
        }
        let (line, col) = line_character(SOURCE, hit.offset);
        println!(
            "  widget.rs:{line}:{col} {:?} -> {:?}",
            &SOURCE[hit.offset..hit.offset + hit.text.len()],
            canonical
        );
        edits.push((hit.offset, hit.offset + hit.text.len(), canonical));
    }

    // The rewrite: every edit applied back to front (the hits ride in
    // source order, so later edits never shift earlier spans) — the
    // stdout the bare `fmt` command prints.
    let mut formatted = SOURCE.to_string();
    for (start, end, text) in edits.iter().rev() {
        formatted.replace_range(start..end, text);
    }
    println!("\n── the file after the walk (the bare fmt stdout) ──");
    print!("{formatted}");
}
