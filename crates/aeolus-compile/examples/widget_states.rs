//! Consumer demonstration: the widget-state variant interp surface
//! (`open:`/`inert:`/`default:` — the v0.x-gap closure in the other
//! direction, tailwindcss v4.1.17's own static variants, the
//! selector-side tier beside the element-state family). The typed
//! conditions, the THREE [`Env`] facts (`opened`, `inert`,
//! `form_default`) and the plain [`interp`] walk.
//!
//! ```sh
//! cargo run -p aeolus-compile --example widget_states
//! ```
//!
//! The CSS semantics the walk encodes: the widget-own answers are
//! element state the host answers — every family question is
//! AFFIRMATIVE (the `target:`/`autofill:` shape), so each layer fires
//! on its ONE fact and an unset fact is a QUIET miss (a selector that
//! does not match, never an exclusion). `inert:`'s compound
//! `:is([inert], [inert] *)` head shares the one per-element fact: the
//! host computes "inside (or carrying) an `[inert]` root" per element.

use aeolus_compile::{Config, Env, compile, interp};

fn main() {
    // The element's own facts (one `Env` per frame, the
    // --open/--inert/--default flags' runtime shape): the element is an
    // open disclosure inside a frozen subtree, and the form group's
    // default control.
    let env = Env {
        opened: true,
        inert: true,
        form_default: true,
        ..Env::default()
    };

    // The layers fire from the facts: open/inert/default win the
    // cascade over the base p-4.
    let compiled = compile("p-4 open:p-8 inert:mt-2 default:ring-2", &Config::default())
        .expect("the class string compiles");
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    println!("fired (opened, inert, form_default):");
    for decl in &interped.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }
    println!("  notes: {}", interped.notes.len());

    // The complement: the same string under a resting env — every
    // widget-state layer stays quiet (the unset facts are the misses a
    // selector that does not match is; no note rides).
    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    println!("\nfired (default env):");
    for decl in &interped.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }
    println!("  notes: {}", interped.notes.len());

    // The CSS arm renders the upstream selector texts (variants.ts's
    // own `staticVariant` texts, the compound `:is(...)` heads
    // verbatim) — the head rides the element's selector, never an
    // at-rule.
    let sheet =
        aeolus_compile::emit_css("open:hidden inert:ring-2 default:mt-3", &Config::default())
            .expect("emits");
    println!("\ncss head:\n{sheet}");
}
