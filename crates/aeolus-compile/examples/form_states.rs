//! Consumer demonstration: the form-state variant interp surface
//! (`enabled:`/`required:`/`optional:`/`in-range:`/`out-of-range:` —
//! the matches-neither closure, tailwindcss v4.1.17's own static
//! variants, the selector-side tier beside the widget-state family).
//! The typed conditions, the FOUR [`Env`] facts (`form_associated`,
//! `required_field`, `range_bounded`, `range_in`) and the plain
//! [`interp`] walk.
//!
//! ```sh
//! cargo run -p aeolus-compile --example form_states
//! ```
//!
//! The CSS semantics the walk encodes: the form-own answers are
//! element state the host answers — each pair gates on a MEMBERSHIP
//! fact (`form_associated`, `range_bounded` — a non-form element
//! matches NEITHER half, the matches-neither boundary one Env bit
//! could not encode) beside the pair's shared state fact
//! (`required_field`, `range_in`; the complement half answers within
//! the universe, the `validation_invalid` precedent). An unset
//! membership is a QUIET miss (a selector that does not match, never
//! an exclusion).

use aeolus_compile::{Config, Env, compile, interp};

fn main() {
    // The element's own facts (one `Env` per frame, the
    // --form-associated/--required/--range-bounded/--range-in flags'
    // runtime shape): the element is a required form control whose
    // bounded value sits within its declared min/max bounds.
    let env = Env {
        form_associated: true,
        required_field: true,
        range_bounded: true,
        range_in: true,
        ..Env::default()
    };

    // The layers fire from the facts: enabled/required/in-range win
    // the cascade over the base p-4.
    let compiled = compile(
        "p-4 enabled:p-8 required:mt-2 optional:hidden in-range:ring-2",
        &Config::default(),
    )
    .expect("the class string compiles");
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    println!("fired (form-associated, required_field, range_bounded, range_in):");
    for decl in &interped.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }
    println!("  notes: {}", interped.notes.len());

    // The complement: the same string under a resting env — a
    // non-form element matches NEITHER half (the membership facts are
    // unset, every form-state layer stays quiet; no note rides).
    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    println!("\nfired (default env):");
    for decl in &interped.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }
    println!("  notes: {}", interped.notes.len());

    // The bounded complement: membership without the in-bounds bit —
    // `out-of-range:` answers within the bounded universe.
    let env = Env {
        range_bounded: true,
        ..Env::default()
    };
    let compiled =
        compile("out-of-range:hidden", &Config::default()).expect("the class string compiles");
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    println!("\nfired (range_bounded only):");
    for decl in &interped.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }

    // The CSS arm renders the upstream selector texts (variants.ts's
    // own `staticVariant` texts verbatim) — the head rides the
    // element's selector, never an at-rule.
    let sheet = aeolus_compile::emit_css(
        "enabled:hidden required:ring-2 optional:mt-3 in-range:p-8 out-of-range:hidden",
        &Config::default(),
    )
    .expect("emits");
    println!("\ncss head:\n{sheet}");
}
