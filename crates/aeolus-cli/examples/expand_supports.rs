//! Consumer demonstration: the supports-variant seam's CLI face (the
//! `expand --supports` facts). The flag normalizes each condition
//! through the fold's own grammar (the
//! [`aeolus_compile::supports_condition`] face — the bare
//! `display:grid` spelling and the paren-wrapped `(display:grid)` land
//! the SAME payload the variant folded), the walk answers the
//! `supports-[…]` layers from the fact table, and the sourceless shape
//! reports through the dedicated note.
//!
//! ```sh
//! cargo run -p aeolus-cli --example expand_supports
//! ```
//!
//! This is the headless form of:
//!
//! ```sh
//! aeolus expand --supports 'display:grid' --supports 'selector(:hover)' \
//!   'supports-[display:grid]:p-8 supports-[selector(:hover)]:mt-2'
//! ```

use aeolus_compile::{Config, Env, SupportsState, compile, interp_supports};

fn main() {
    // The render target's capability facts, normalized through the
    // fold's grammar — the CLI validates the same way (one grammar, two
    // consumers; a fact no `@supports` grammar accepts is exit 2).
    let facts: &[Option<&'static str>] = &[
        aeolus_compile::supports_condition("display:grid"),
        aeolus_compile::supports_condition("(display:grid)"),
        aeolus_compile::supports_condition("selector(:hover)"),
    ];
    let mut conditions: Vec<&str> = Vec::new();
    println!("normalized facts (one payload per distinct condition):");
    for fact in facts.iter().flatten() {
        if !conditions.contains(fact) {
            conditions.push(fact);
            println!("  {fact:?}");
        }
    }
    let target = SupportsState::new(&conditions);

    // The walk answers: both supports layers fire, the cascade hands
    // the longhands over.
    let compiled = compile(
        "supports-[display:grid]:p-8 supports-[selector(:hover)]:mt-2",
        &Config::default(),
    )
    .expect("the class string compiles");
    let interped =
        interp_supports(&compiled.program, &Env::default(), &target).expect("the program resolves");
    println!("\nfired:");
    for decl in &interped.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }
    println!("  notes: {}", interped.notes.len());
}
