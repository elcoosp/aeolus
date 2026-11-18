//! Consumer demonstration: the interaction-state flags (the CLI's
//! founding element-state family face — `--hover`/`--focus`/`--active`
//! beside the composition tier's founding `--group-hover`/`--peer-focus`
//! members). The flags assert the element-own facts the [`Env`] carries;
//! the interp answers the asserted layers beside the resting env's quiet
//! miss. The library face the flags run is the plain [`interp`] walk the
//! interp arm carries.
//!
//! ```sh
//! cargo run -p aeolus-cli --example expand_interaction_states
//! ```

use aeolus_compile::{Config, Env, compile, interp};

fn show(classes: &str, env: &Env) {
    let compiled = compile(classes, &Config::default()).expect("compiles");
    let interped = interp(&compiled.program, env).expect("the walk resolves");
    for decl in &interped.decls {
        println!("  {} = {:?}", decl.prop.name(), decl.val);
    }
}

fn main() {
    let classes = "p-4 hover:p-8 focus-visible:mt-2 disabled:opacity-50";
    println!("── the program: {classes} ──");

    println!("\n── 1. the resting env (`expand {classes}`) ──");
    show(classes, &Env::default());

    println!("\n── 2. the `--hover` fact (`expand --hover …`) ──");
    show(
        classes,
        &Env {
            hover: true,
            ..Env::default()
        },
    );

    println!("\n── 3. two asserted facts (`expand --hover --focus-visible …`) ──");
    show(
        classes,
        &Env {
            hover: true,
            focus_visible: true,
            ..Env::default()
        },
    );

    println!("\n── 4. the composition founders (`expand --group-hover …`) ──");
    let composed = "p-4 group-hover:p-8 peer-focus:mt-2";
    println!("  the program: {composed}");
    show(
        composed,
        &Env {
            group_hover: true,
            peer_focus: true,
            ..Env::default()
        },
    );

    println!("\nthe flags are the founding family's CLI face — one flag per FACT.");
}
