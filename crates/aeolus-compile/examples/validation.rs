//! Consumer demonstration: the validation-variant interp surface
//! (`valid:`/`invalid:`/`user-valid:`/`user-invalid:` — the form-state
//! family, tailwindcss v4.1.17's own static variants, the selector-side
//! tier beside `hover:`/`checked:`). The typed conditions and the TWO
//! [`Env`] facts (`validation_invalid`, `user_validated`) and the plain
//! [`interp`] walk.
//!
//! ```sh
//! cargo run -p aeolus-compile --example validation
//! ```
//!
//! The CSS semantics the walk encodes: the constraint state is element
//! state the host answers — `invalid:` on the fact, `valid:` its
//! COMPLEMENT (CSS's own reading for a pristine field is :valid, the
//! `ltr:` precedent) — and the `user-` pair additionally gates on the
//! interaction fact the HTML spec requires, so BOTH stay quiet until
//! the gate is set. An unset fact is a QUIET miss (a selector that does
//! not match, never an exclusion).

use aeolus_compile::{Config, Env, compile, interp};

fn main() {
    // The field's constraint facts (one `Env` per frame, the
    // --invalid/--user-* flags' runtime shape): the constraints fail
    // and the user has interacted with the field.
    let env = Env {
        validation_invalid: true,
        user_validated: true,
        ..Env::default()
    };

    // The layers fire from the facts: user-invalid wins the cascade
    // over the base p-4; valid misses (the complement question under
    // the failing fact).
    let compiled = compile(
        "p-4 invalid:p-8 valid:mt-2 user-valid:mt-3 user-invalid:hidden",
        &Config::default(),
    )
    .expect("the class string compiles");
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    println!("fired (validation_invalid, user_validated):");
    for decl in &interped.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }
    println!(
        "  notes: {} (the valid/user-valid misses are quiet, not notes)",
        interped.notes.len()
    );

    // The complement: the same string under a resting env — the valid
    // layer fires (CSS's pristine-field reading), the invalid and the
    // gated user- pair all stay quiet (the interaction gate is the
    // HTML spec's own).
    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    println!("\nfired (default env):");
    for decl in &interped.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }
    println!("  notes: {}", interped.notes.len());

    // The CSS arm renders the upstream pseudo-classes (variants.ts's
    // own `staticVariant` texts) — the head rides the element's
    // selector, never an at-rule.
    let sheet = aeolus_compile::emit_css("user-invalid:hidden", &Config::default()).expect("emits");
    println!("\ncss head:\n{sheet}");
}
