//! Consumer demonstration: the descendant-probe interp surface (plan
//! §3.5's `has-*` row, the seam the `DescendantUnqueried` note carries
//! since the attribute-probe fold landed). [`DescendantProbe`] (the
//! one question), [`DescendantState`] (the built-in static-fact table)
//! and the [`interp_descendants`]/[`interp_theme_descendants`] entry
//! points.
//!
//! ```sh
//! cargo run -p aeolus-compile --example descendants
//! ```
//!
//! The CSS semantics the walk encodes: a descendant layer FIRES when
//! the source answers yes for the layer's OWN scope (the element's
//! subtree, the `.group` ancestor's, the `.peer` sibling's — the
//! [`DescendantKind`] discrimination); a source-answered no is
//! CSS-quiet — a `:has()` selector that does not match, not an
//! exclusion; the sourceless walk keeps the reported boundary (the
//! dedicated note, the honest failure the plan row kept until this
//! seam landed).

use aeolus_compile::{
    Config, DescendantFact, DescendantKind, DescendantState, Env, InterpNote, compile, interp,
    interp_descendants,
};
use aeolus_ir::Prop;

fn main() {
    // The element's subtree facts (plan §3.5): the element's own
    // subtree contains a focused child and the card-action data shape;
    // the `.group` ancestor's subtree contains a focused child. The
    // `.peer` sibling's subtree matches nothing.
    let tree = DescendantState::new(&[
        DescendantFact {
            kind: DescendantKind::Element,
            fragment: "*:focus",
        },
        DescendantFact {
            kind: DescendantKind::Element,
            fragment: "*[data-slot=\"card-action\"]",
        },
        DescendantFact {
            kind: DescendantKind::Group,
            fragment: "*:focus",
        },
    ]);

    // A descendant layer FIRES under the facts: has-focus:p-8 wins the
    // longhand cascade over the base p-4.
    let compiled =
        compile("p-4 has-focus:p-8", &Config::default()).expect("the class string compiles");
    let interped = interp_descendants(&compiled.program, &Env::default(), &tree)
        .expect("the program resolves");
    println!("fired (has-focus wins the cascade):");
    for decl in &interped.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }
    println!(
        "  notes: {} (answered misses are CSS-quiet)",
        interped.notes.len()
    );

    // The SCOPE discriminates the question: the peer sibling's subtree
    // answers nothing, so peer-has-focus quietly misses (a `:has()`
    // selector that does not match — not a note).
    let compiled = compile(
        "group-has-focus:pt-2 peer-has-focus:pb-8",
        &Config::default(),
    )
    .expect("the class string compiles");
    let interped = interp_descendants(&compiled.program, &Env::default(), &tree)
        .expect("the program resolves");
    let top = interped
        .decls
        .iter()
        .find(|d| d.prop == Prop::PaddingTop)
        .expect("group-has-focus fires under the group facts");
    println!("\nscope discrimination:");
    println!("  group-has-focus fired: PaddingTop -> {:?}", top.val);
    println!(
        "  peer-has-focus missed quietly: PaddingBottom present? {}",
        interped.decls.iter().any(|d| d.prop == Prop::PaddingBottom)
    );

    // The sourceless walk keeps the reported boundary: the dedicated
    // note replaces the blanket attribute spelling for the family.
    let compiled = compile("has-focus:p-8", &Config::default()).expect("the class string compiles");
    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    println!("\nsourceless walk:");
    for note in &interped.notes {
        match note {
            InterpNote::DescendantUnqueried { .. } => println!("  {note}"),
            other => println!("  {other}"),
        }
    }
}
