//! Consumer demonstration: the direction-variant interp surface
//! (`ltr:`/`rtl:` — the text-direction family, the selector-side
//! sibling of the media-feature tier, tailwindcss v4.1.17's own static
//! variants). The typed [`aeolus_ir::Direction`] bit, the ONE [`Env`]
//! fact (`direction_rtl`) and the plain [`interp`] walk.
//!
//! ```sh
//! cargo run -p aeolus-compile --example direction
//! ```
//!
//! The CSS semantics the walk encodes: the direction is frame state
//! (the element's resolved `direction`), the two spellings answer from
//! the ONE fact — `rtl:` on the bit, `ltr:` its COMPLEMENT (CSS's own
//! initial `direction` value is ltr, the `landscape:` precedent) — and
//! an unset fact is a QUIET miss (a selector that does not match,
//! never an exclusion).

use aeolus_compile::{Config, Env, compile, interp};

fn main() {
    // The element's direction fact (one `Env` per frame, the --rtl
    // flag's runtime shape): this document runs right-to-left.
    let env = Env {
        direction_rtl: true,
        ..Env::default()
    };

    // The layers fire from the fact: rtl wins the cascade over the
    // base p-4; ltr misses (the complement question under the rtl
    // fact).
    let compiled = compile("p-4 rtl:p-8 ltr:mt-2 rtl:hidden", &Config::default())
        .expect("the class string compiles");
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    println!("fired (direction_rtl):");
    for decl in &interped.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }
    println!(
        "  notes: {} (the ltr miss is quiet, not a note)",
        interped.notes.len()
    );

    // The complement: the same string under a resting env — the ltr
    // layers fire instead (CSS's initial direction is ltr), the rtl
    // layers all miss.
    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    println!("\nfired (default env):");
    for decl in &interped.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }
    println!("  notes: {}", interped.notes.len());

    // The CSS arm renders the upstream selectors (variants.ts's own
    // `staticVariant` texts) — the head rides the element's selector,
    // never an at-rule.
    let sheet = aeolus_compile::emit_css("rtl:p-8", &Config::default()).expect("emits");
    println!("\ncss head:\n{sheet}");
}
