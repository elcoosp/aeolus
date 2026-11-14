//! The shadcn PROGRAM TABLES (architecture plan §3.6's surface (2),
//! §10's "shadcn components are `const` tables"): the verbatim v4
//! class strings compiled through the V1 pipeline once per closed-world
//! combo and interned — the F6 finding (`button_classes() -> String`
//! per call, re-resolved by the consumer) gone.
//!
//! This demo walks the two arms side by side: the string path (the
//! `String` build the wrappers re-resolve per call) and the program
//! path (the table hit), the interning proof (the same pointer on a
//! repeat call), the first compile's diagnostics (the documented
//! arbitrary-variant refusals) and the effective declarations resolved
//! through the session's token table.
//!
//! ```sh
//! cargo run -p aeolus-shadcn --example program_tables
//! ```

use aeolus_compile::{Env, interp_theme};
use aeolus_rt::{ThemeSource, TokenTable};
use aeolus_shadcn::classes::{ButtonSize, ButtonVariant};
use aeolus_shadcn::program::ButtonPrograms;

fn main() {
    // The session's table: the built-in shadcn theme baked (T0
    // literals) — the same source a `theme!` project would ship.
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let tokens = TokenTable::baked(&source).expect("the T0/T1 bake succeeds");

    // The handle is view state: one table per session, the ClassCell's
    // own identity rule.
    let tables = ButtonPrograms::new(&tokens);

    // ── the string path (F6 as measured) ───────────────────────────
    let (variant, size) = (ButtonVariant::Default, ButtonSize::Default);
    let classes = aeolus_shadcn::classes::button_classes(variant, size);
    println!(
        "string path per call: {} chars built fresh (cn over base + variant + size), re-resolved by the consumer",
        classes.len(),
    );

    // ── the program path (the const-table seam) ────────────────────
    let program = tables.program(variant, size);
    println!(
        "program path: {} layers interned (compile once per closed-world combo)",
        program.layers.len(),
    );

    // The interning proof: a repeat call is the SAME pointer.
    let again = tables.program(variant, size);
    println!(
        "repeat call: {} — the table hit, not a second compile",
        if program.program().layers.as_ptr() == again.program().layers.as_ptr() {
            "SAME interned program"
        } else {
            "DIVERGED (a bug)"
        },
    );

    // The first compile's diagnostics: the documented
    // arbitrary-variant refusals, reported once — never a silent drop.
    let refusals = tables.diagnostics(variant, size);
    println!(
        "first compile's refusals: {} — {}",
        refusals.len(),
        refusals
            .iter()
            .map(|d| match &d.kind {
                aeolus_compile::DiagnosticKind::UnsupportedVariant { variant } => variant.clone(),
                other => format!("{other:?}"),
            })
            .collect::<Vec<_>>()
            .join(", "),
    );

    // The effective declarations: the token slots resolve through the
    // session's table (the T0 tier's one lookup).
    let effective =
        interp_theme(&program.program(), &tokens, &Env::default()).expect("the interp resolves");
    println!("\neffective declarations ({}):", effective.decls.len());
    for decl in &effective.decls {
        println!("  {:?} {:?}", decl.prop, decl.val);
    }
}
