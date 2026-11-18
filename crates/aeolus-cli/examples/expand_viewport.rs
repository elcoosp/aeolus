//! Consumer demonstration: the viewport-width fact (the CLI's
//! breakpoint-family face — the `--width rem` flag). The flag carries
//! the viewport width the [`Env::viewport_rem`] slot answers the
//! breakpoint families from: the min forms (`sm:`…`2xl:`) fire AT and
//! above their breakpoint's rem, the max forms (`max-sm:`…) strictly
//! below it, and the resting env stays the zero-width viewport (no min
//! breakpoint fires — the quiet miss a below-breakpoint viewport is).
//! The library face the flag runs is the plain [`interp`] walk the
//! interp arm carries.
//!
//! ```sh
//! cargo run -p aeolus-cli --example expand_viewport
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
    let classes = "p-4 sm:p-8 max-md:p-2 xl:p-16";
    println!("── the program: {classes} ──");

    println!("\n── 1. the resting env (`expand {classes}`) ──");
    // The zero-width viewport: the min forms miss, the max form rides
    // its strictly-below edge (0 < 48).
    show(classes, &Env::default());

    println!("\n── 2. the `--width 48` fact (`expand --width 48 …`) ──");
    // AT the md breakpoint: `sm:` fired already, `max-md:` stopped, the
    // unasserted `xl:` waits.
    show(
        classes,
        &Env {
            viewport_rem: 48.0,
            ..Env::default()
        },
    );

    println!("\n── 3. the `--width 39.9` fact (`expand --width 39.9 …`) ──");
    // Strictly below sm: every min form misses, `max-md:` still fires.
    show(
        classes,
        &Env {
            viewport_rem: 39.9,
            ..Env::default()
        },
    );

    println!("\n── 4. the composition (`expand --width 48 --hover …`) ──");
    // The breakpoint tier joins the cascade AFTER the state tier (the
    // sort order's own slots): at the md width the breakpoint's answer
    // wins the padding longhand over the hover state's.
    let composed = "p-4 sm:p-8 hover:p-12 md:p-8";
    println!("  the program: {composed}");
    show(
        composed,
        &Env {
            viewport_rem: 48.0,
            hover: true,
            ..Env::default()
        },
    );

    println!(
        "\nthe flag is the breakpoint families' CLI face — the AT/strictly-below boundary rules."
    );
}
