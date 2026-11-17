//! Consumer demonstration: the `expand --pseudo` face (the CLI's
//! pseudo-element render-target surface, the [`PseudoTarget`] seam's
//! consumer half). The element walk reports the pseudo layers as its
//! exclusions; `--pseudo` answers the NAMED render target's OWN
//! cascade beside the element answer — the closed set the flag
//! parses (`selection` | `placeholder` | `before` | `after` |
//! `marker` | `first-letter` | `first-line` | `file` | `backdrop` |
//! `details-content`).
//!
//! The demo resolves the library face directly (the same
//! [`interp_pseudo`] walk the flag runs) over one pseudo-mixed
//! program, then prints the tier's whole set — the output the
//! `aeolus expand --pseudo <target>` command renders.
//!
//! ```sh
//! cargo run -p aeolus-cli --example expand_pseudo
//! ```

use aeolus_compile::{Config, Env, PseudoTarget, compile, interp, interp_pseudo};

fn main() {
    let config = Config::default();
    let classes = "bg-[#ef4444] selection:bg-[#eab308] before:p-8 marker:text-[#3f3f46]";
    println!("── the program: {classes} ──");
    let compiled = compile(classes, &config).expect("compiles");

    println!(
        "\n── 1. the element cascade: the pseudo layers report, the answer is the element's ──"
    );
    let element = interp(&compiled.program, &Env::default()).expect("the walk resolves");
    for decl in &element.decls {
        println!("  {} = {:?}", decl.prop.name(), decl.val);
    }
    for note in &element.notes {
        println!("  note: {note}");
    }

    println!("\n── 2. the render targets: one cascade each (the --pseudo face's answer) ──");
    for target in PseudoTarget::ALL {
        let interped = interp_pseudo(&compiled.program, *target).expect("the walk resolves");
        print!("  ::{}", target.spelling());
        if interped.decls.is_empty() {
            println!(" — no rules target it (the quiet, not noted, answer)");
        } else {
            println!();
            for decl in &interped.decls {
                println!("    {} = {:?}", decl.prop.name(), decl.val);
            }
        }
    }

    println!(
        "\nthe closed set carries {} targets; every one resolves through the same walk.",
        PseudoTarget::ALL.len()
    );
}
