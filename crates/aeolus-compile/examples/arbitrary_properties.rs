//! Consumer demonstration: the arbitrary-PROPERTY form (the bare
//! `[prop:value]` class — v4's arbitrary-property utility, the last
//! typed refusal of the arbitrary grammar). The bracket IS the
//! utility: the declaration lowers onto the appended
//! [`aeolus_ir::Prop::Arbitrary`] arm with the whole `name: value`
//! text interned in the IR's verbatim slot, the CSS arm renders it
//! verbatim (the author owns the CSS validity — the composite
//! grammar's own contract), the variant stack composes normally, and
//! the IR-level `cn()`/`overlay` composition folds per PROPERTY NAME
//! (the one key the longhand bitset cannot carry). The ROOTED
//! spelling (`w-[mask-type:luminance]`) keeps its typed refusal — the
//! row already names its own property — and the framework lanes keep
//! theirs (no per-property slot exists; the CSS/interp arms are the
//! honest output).
//!
//! ```sh
//! cargo run -p aeolus-compile --example arbitrary_properties
//! ```
//!
//! The battery: a plain property, the custom-property spelling, an
//! underscore-decoded value, the variant-composed form, the name-keyed
//! composition, the CSS text and the rooted refusal.

use aeolus_compile::{Config, cn, compile, emit_css, overlay};
use aeolus_ir::{Cond, Prop, Val};

fn main() {
    // The bare forms lower: the declaration rides the appended arm,
    // the text normalized to the CSS `name: value` shape.
    let compiled = compile(
        "[mask-type:luminance] [--brand:red] [--tw-x:1px_2px]",
        &Config::default(),
    )
    .expect("non-empty input compiles");
    println!(
        "── bare property declarations: {} diagnostics ──",
        compiled.diagnostics.len()
    );
    for decl in compiled.program.layers[0].decls {
        if decl.prop == Prop::Arbitrary {
            if let Val::Verbatim(text) = decl.val {
                println!("  {text}");
            }
        }
    }

    // The variant stack composes normally: the declaration rides the
    // folded condition like every other class's.
    let composed = compile(
        "hover:[mask-type:luminance] focus:[--tw-y:var(--brand)]",
        &Config::default(),
    )
    .expect("non-empty input compiles");
    println!("── variant-composed ──");
    for layer in composed.program.layers {
        if layer.cond == Cond::Base {
            continue;
        }
        for decl in layer.decls {
            if decl.prop == Prop::Arbitrary {
                if let Val::Verbatim(text) = decl.val {
                    println!("  {:?}: {text}", layer.cond);
                }
            }
        }
    }

    // The composition folds per PROPERTY NAME: same name last-wins,
    // distinct names coexist — the CSS cascade's own per-property rule.
    let base = compile("[mask-type:luminance] [--brand:red]", &Config::default())
        .expect("non-empty input compiles");
    let user = compile("[mask-type:alpha] [--brand:blue]", &Config::default())
        .expect("non-empty input compiles");
    let merged = cn(&[&base, &user]).expect("non-empty inputs merge");
    println!("── cn() folds per name ──");
    for decl in merged.program.layers[0].decls {
        if let Val::Verbatim(text) = decl.val {
            println!("  {text}");
        }
    }

    // The overlay seam answers the same question over two programs.
    let over = overlay(&base.program, &user.program);
    println!(
        "── overlay: {} property declarations survive ──",
        over.layers[0]
            .decls
            .iter()
            .filter(|decl| decl.prop == Prop::Arbitrary)
            .count()
    );

    // The CSS arm renders the declaration verbatim.
    let sheet =
        emit_css("[mask-type:luminance]", &Config::default()).expect("non-empty input compiles");
    println!("── css ──");
    println!("{sheet}");

    // The rooted spelling keeps its typed refusal: the row already
    // names its own property.
    let rooted =
        compile("w-[mask-type:luminance]", &Config::default()).expect("non-empty input compiles");
    println!("── the rooted spelling stays a typed boundary ──");
    for diagnostic in &rooted.diagnostics {
        println!("  ! {diagnostic} [{}]", diagnostic.kind.code());
    }
}
