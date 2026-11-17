//! Consumer demonstration: the `expand --starting` face (the CLI's
//! starting-style frame-fact flag — one flag per FACT, the capability
//! flags' sibling). The flag answers the `starting:` layer's own
//! effective declarations beside the resting env's quiet miss; the
//! library face the flag runs is the plain [`interp`] walk the interp
//! arm carries.
//!
//! ```sh
//! cargo run -p aeolus-cli --example expand_starting
//! ```

use aeolus_compile::{Config, Env, compile, interp};

fn main() {
    let config = Config::default();
    let classes = "starting:p-8 p-4";
    println!("── the program: {classes} ──");
    let compiled = compile(classes, &config).expect("compiles");

    println!("\n── 1. the resting env (`expand starting:p-8 p-4`) ──");
    let resting = interp(&compiled.program, &Env::default()).expect("the walk resolves");
    for decl in &resting.decls {
        println!("  {} = {:?}", decl.prop.name(), decl.val);
    }
    for note in &resting.notes {
        println!("  note: {note}");
    }

    println!("\n── 2. the `--starting` frame fact (`expand --starting …`) ──");
    let env = Env {
        starting_style: true,
        ..Env::default()
    };
    let fired = interp(&compiled.program, &env).expect("the walk resolves");
    for decl in &fired.decls {
        println!("  {} = {:?}", decl.prop.name(), decl.val);
    }

    println!("\nthe flag is the frame fact's CLI face — one flag per FACT.");
}
