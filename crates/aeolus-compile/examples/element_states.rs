//! Consumer demonstration: the element-state variant interp surface
//! (`target:`/`indeterminate:`/`autofill:`/`read-only:` — the v0.x-gap
//! closure, tailwindcss v4.1.17's own static variants, the selector-side
//! tier beside the validation states). The typed conditions and the
//! FOUR [`Env`] facts (`url_target`, `indeterminate`, `autofilled`,
//! `read_write`) and the plain [`interp`] walk.
//!
//! ```sh
//! cargo run -p aeolus-compile --example element_states
//! ```
//!
//! The CSS semantics the walk encodes: the element-own answers are
//! element state the host answers — the target/indeterminate/autofill
//! layers fire on their ONE fact each, and `read-only:` rides the
//! editability fact's COMPLEMENT (CSS's own reading for a
//! non-editable element IS :read-only, the `ltr:` precedent). An unset
//! fact is a QUIET miss (a selector that does not match, never an
//! exclusion).

use aeolus_compile::{Config, Env, compile, interp};

fn main() {
    // The element's own facts (one `Env` per frame, the
    // --target/--indeterminate/--autofill/--read-write flags' runtime
    // shape): the element is the document's fragment target and is NOT
    // user-editable (the resting read_write fact — the complement the
    // read-only query answers from).
    let env = Env {
        url_target: true,
        indeterminate: true,
        ..Env::default()
    };

    // The layers fire from the facts: target/indeterminate win the
    // cascade over the base p-4; autofill misses (the one-fact question
    // under the unset answer).
    let compiled = compile(
        "p-4 target:p-8 indeterminate:mt-2 autofill:mt-3 read-only:ring-2",
        &Config::default(),
    )
    .expect("the class string compiles");
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    println!("fired (url_target, indeterminate, resting read_write):");
    for decl in &interped.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }
    println!(
        "  notes: {} (the autofill miss is quiet, not a note)",
        interped.notes.len()
    );

    // The complement: the same string under a resting env — the
    // read-only layer fires (CSS's non-editable-element reading), the
    // target/indeterminate/autofill layers all stay quiet (the unset
    // facts are the misses a selector that does not match is).
    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    println!("\nfired (default env):");
    for decl in &interped.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }
    println!("  notes: {}", interped.notes.len());

    // The CSS arm renders the upstream pseudo-classes (variants.ts's
    // own `staticVariant` texts) — the head rides the element's
    // selector, never an at-rule.
    let sheet = aeolus_compile::emit_css("target:hidden read-only:ring-2", &Config::default())
        .expect("emits");
    println!("\ncss head:\n{sheet}");
}
