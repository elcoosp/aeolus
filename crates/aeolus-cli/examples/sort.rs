//! Consumer demonstration: the `sort` face (the class-sort seam's
//! literal-string form — the registry's canonical order as the new
//! text, per-class misses riding the typed diagnostics channel). The
//! file face is `fmt` (its own example beside it), the editor face the
//! LSP's `textDocument/formatting`; this is the batch stdin/stdout
//! shape a CI hook or a `tr`-style pipe drives.
//!
//! ```sh
//! cargo run -p aeolus-cli --example sort
//! ```
//!
//! This is the headless form of:
//!
//! ```sh
//! aeolus sort 'block p-4 mx-2 hover:block p-8'
//! ```

use aeolus_cli::run;

fn main() {
    // The canonical order: the registry's own cascade order as the new
    // text — `p-4` before `mx-2` (the spacing family's order), `block`
    // before `hover:block` (the variant layers sort after their base).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "sort".to_string(),
            "block p-4 mx-2 hover:block p-8".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("$ aeolus sort 'block p-4 mx-2 hover:block p-8' (exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());
    print!("{}", String::from_utf8(err).unwrap_or_default());

    // The JSON face: the same answer as the machine-readable form (the
    // sorted text beside the typed diagnostics array).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "sort".to_string(),
            "--json".to_string(),
            "hover:block flex p-4".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("\n$ aeolus sort --json 'hover:block flex p-4' (exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());
    print!("{}", String::from_utf8(err).unwrap_or_default());

    // The miss face: a class the registry does not know rides the
    // diagnostics channel — the sort still emits (the known classes
    // order, the miss reports), exit 1 (the findings code).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "sort".to_string(),
            "p-4 not-a-real-utility flex".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("\n$ aeolus sort 'p-4 not-a-real-utility flex' (exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());
    print!("{}", String::from_utf8(err).unwrap_or_default());
}
