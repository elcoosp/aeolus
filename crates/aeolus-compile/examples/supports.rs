//! Consumer demonstration: the supports-variant interp surface (the
//! `supports-[…]` family, the at-rule seam beside the media tier).
//! [`SupportsProbe`] (the one question), [`SupportsState`] (the
//! built-in static-fact table) and the
//! [`interp_supports`]/[`interp_theme_supports`] entry points.
//!
//! ```sh
//! cargo run -p aeolus-compile --example supports
//! ```
//!
//! The CSS semantics the walk encodes: a supports layer FIRES when the
//! source answers yes for the layer's OWN condition text (the payload
//! the fold rendered from the `supports-[…]` spelling); a
//! source-answered no is CSS-quiet — an `@supports` block the render
//! target fails, not an exclusion; the sourceless walk keeps the
//! reported boundary (the dedicated note, the honest failure the
//! at-rule tier kept until this seam landed).

use aeolus_compile::{Config, Env, InterpNote, SupportsState, compile, interp, interp_supports};

fn main() {
    // The render target's capability facts: the target supports the
    // grid declaration probe and the selector function form; it does
    // NOT support the hover-declaration compound (the answered-miss
    // row). The facts are the fold's own normalized payloads — a host
    // builds them from its style system's feature set.
    let target = SupportsState::new(&["(display:grid)", "selector(:hover)"]);

    // A supports layer FIRES under the facts: supports-[display:grid]
    // wins the longhand cascade over the base p-4.
    let compiled = compile(
        "p-4 supports-[display:grid]:p-8 supports-[selector(:hover)]:mt-2",
        &Config::default(),
    )
    .expect("the class string compiles");
    let interped =
        interp_supports(&compiled.program, &Env::default(), &target).expect("the program resolves");
    println!("fired (the target answers yes for both conditions):");
    for decl in &interped.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }
    println!(
        "  notes: {} (answered misses are CSS-quiet)",
        interped.notes.len()
    );

    // A source-answered NO is a quiet skip: the target does not carry
    // the hover compound, so the layer never matches — no note, the
    // CSS selector semantics.
    let compiled = compile("supports-[hover:hover]:p-8", &Config::default())
        .expect("the class string compiles");
    let interped =
        interp_supports(&compiled.program, &Env::default(), &target).expect("the program resolves");
    println!("\nanswered miss (supports-[hover:hover] unsupported):");
    println!("  declarations: {}", interped.decls.len());
    println!(
        "  notes: {} (a failed block is quiet)",
        interped.notes.len()
    );

    // The sourceless walk keeps the reported boundary: the plain
    // `interp` carries no capability facts, the layer reports through
    // the dedicated note.
    let compiled = compile("supports-[display:grid]:p-8", &Config::default())
        .expect("the class string compiles");
    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    println!("\nsourceless walk (no SupportsProbe source):");
    for note in &interped.notes {
        match note {
            InterpNote::SupportsUnqueried { cond, .. } => {
                println!("  note: {note}");
                if let aeolus_ir::Cond::Supports(condition) = cond {
                    println!("    condition: {condition:?}");
                }
            }
            other => println!("  other: {other}"),
        }
    }
}
