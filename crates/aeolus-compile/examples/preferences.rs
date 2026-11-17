//! Consumer demonstration: the preference-variant interp surface (the
//! `motion-*`/`contrast-*`/`forced-colors:`/`print:` families, the
//! media-feature tier beside the breakpoint forms). The typed
//! [`aeolus_ir::Pref`] bits, the [`Env`] facts (the user agent's own
//! media-query answers — frame state, the tier `dark:` rides) and the
//! plain [`interp`] walk.
//!
//! ```sh
//! cargo run -p aeolus-compile --example preferences
//! ```
//!
//! The CSS semantics the walk encodes: the [`Env`] answers every
//! preference question, an unset bit is a QUIET miss (a media query
//! that does not hold, never an exclusion), and `motion-safe:` is the
//! COMPLEMENT of `motion-reduce:` — the feature's two v4 values share
//! one fact.

use aeolus_compile::{Config, Env, compile, interp};

fn main() {
    // The user agent's preference facts (one `Env` per frame, the
    // --reduced-motion/--print flags' runtime shape): this user prefers
    // reduced motion and prints.
    let env = Env {
        reduced_motion: true,
        print_media: true,
        ..Env::default()
    };

    // The layers fire from the facts: motion-reduce wins the cascade
    // over the base p-4; motion-safe misses (the complement question);
    // print fires (the media type holds).
    let compiled = compile(
        "p-4 motion-reduce:p-8 motion-safe:mt-2 print:hidden",
        &Config::default(),
    )
    .expect("the class string compiles");
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    println!("fired (reduced motion + print):");
    for decl in &interped.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }
    println!(
        "  notes: {} (unset bits are quiet misses)",
        interped.notes.len()
    );

    // The complement: the same string under a NO-preference user agent
    // — motion-safe fires instead, motion-reduce misses.
    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    println!("\nfired (no preferences):");
    for decl in &interped.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }
    println!("  notes: {}", interped.notes.len());

    // The media-feature REMAINDER (the v4.1 additions' tier): the
    // inverted-colours answer and the scripting feature's `none`
    // value — the same frame-state facts, the same env-field read.
    let compiled = compile("inverted-colors:p-8 noscript:mt-2", &Config::default())
        .expect("the class string compiles");
    let env = Env {
        inverted_colors: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    println!("\nfired (inverted colours, scripting available):");
    for decl in &interped.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }
    println!("  notes: {}", interped.notes.len());
}
