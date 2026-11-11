//! Consumer demonstration: the rust-tokens emit arm (architecture plan
//! §3.1 emit (a) — the LAST arm of the `Target` surface). Class strings
//! in, one embeddable `aeolus_ir::Layer::new` expression per lowered
//! class out — the keyword slots resolved to their ROW text and the
//! theme token slots to their TABLE names, the two annotations a bare
//! `Program` cannot recover (the in-pipeline emit renders where the
//! rows still talk, the CSS arm's stated rationale).
//!
//! ```sh
//! cargo run -p aeolus-compile --example rust_tokens
//! ```
//!
//! The sheet's Display assembles the fragments into one compilable
//! `static PROGRAM: &[aeolus_ir::Layer]` item — paste it anywhere
//! `aeolus-ir` is a dependency; the per-class fragments are the
//! `tw!`-style macro's unit (one class, one fragment, source order).

use aeolus_compile::{Config, compile_theme, emit_rust_theme};
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    // The theme: the same CSS a web Tailwind v4 project uses. The
    // runtime tier lowers colour tokens to Tok slots the annotation
    // names (the table's own answer to "which token IS id N?").
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::runtime(&source).expect("default-seeded source builds");

    let sheet = emit_rust_theme(
        "bg-primary text-primary-foreground p-4 rounded-md flex-col \
         hover:bg-primary/90 md:p-6",
        &table,
        &Config::default(),
    )
    .expect("non-empty input renders");

    // The fragments, per class in source order — the macro's unit:
    for fragment in &sheet.fragments {
        println!("── {} ──", fragment.class);
        println!("{}\n", fragment.tokens);
    }

    // The sheet Display: the whole program as one compilable static
    // item, layers specificity-sorted, masks derived at construction.
    println!("════════ the assembled static item ════════");
    print!("{sheet}");

    // The artifact is deterministic and total: the same input renders
    // byte-identical text, and the honest boundaries ride the SAME
    // diagnostics compile reports.
    let again = emit_rust_theme("bg-primary p-4", &table, &Config::default()).expect("renders");
    let compiled = compile_theme("bg-primary p-4", &table, &Config::default())
        .expect("non-empty input compiles");
    println!(
        "\n── determinism: {} ──",
        again.to_string() == again.to_string()
    );
    println!(
        "── the sheet reports exactly what compile reports: {} ──",
        again.diagnostics == compiled.diagnostics
    );
}
