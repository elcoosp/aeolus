//! Consumer demonstration: the capability-variant interp surface (the
//! `pointer-*`/`any-pointer-*` input tier and the `portrait`/`landscape`
//! orientation forms, the media-feature tier's remaining v4.1.17
//! spellings beside the preference families). The typed
//! [`aeolus_ir::Pref`] bits, the [`Env`] facts (the user agent's own
//! media-query answers — frame state, the tier `dark:` rides) and the
//! plain [`interp`] walk.
//!
//! ```sh
//! cargo run -p aeolus-compile --example capabilities
//! ```
//!
//! The CSS semantics the walk encodes: the [`Env`] answers every
//! capability question, an unset bit is a QUIET miss (a media query
//! that does not hold, never an exclusion), and `landscape:` is the
//! COMPLEMENT of `portrait:` — the feature's two v4 values share one
//! fact, the orientation query always holding an answer the way the
//! reduced-motion one does.

use aeolus_compile::{Config, Env, compile, interp};

fn main() {
    // The user agent's capability facts (one `Env` per frame, the
    // --pointer-fine/--portrait flags' runtime shape): this user's
    // primary pointer is precise and the viewport is portrait.
    let env = Env {
        pointer_fine: true,
        portrait: true,
        ..Env::default()
    };

    // The layers fire from the facts: pointer-fine and portrait win
    // the cascade over the base p-4; pointer-coarse misses (the
    // sibling query does not hold); landscape misses (the complement
    // question).
    let compiled = compile(
        "p-4 pointer-fine:p-8 pointer-coarse:mt-2 landscape:flex",
        &Config::default(),
    )
    .expect("the class string compiles");
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    println!("fired (pointer-fine + portrait):");
    for decl in &interped.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }
    println!(
        "  notes: {} (unset bits are quiet misses)",
        interped.notes.len()
    );

    // The complement: the same string under a resting env — landscape
    // fires instead, the pointer layers all miss.
    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    println!("\nfired (default env):");
    for decl in &interped.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }
    println!("  notes: {}", interped.notes.len());
}
