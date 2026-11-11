//! Consumer demonstration: the attribute-probe interp surface (plan
//! §3.5 — "the element's own attributes answer through typed state
//! bits passed with the element"). The seam the `AttributeUnqueried`
//! note named, landed: [`AttrProbe`] (the one question), [`AttrState`]
//! (the built-in static-bit table) and the
//! [`interp_probes`]/[`interp_theme_probes`] entry points.
//!
//! ```sh
//! cargo run -p aeolus-compile --example probes
//! ```
//!
//! The CSS semantics the walk encodes: a probe layer FIRES when the
//! element carries the attribute (presence) or carries it with the
//! folded value (equality); a miss is CSS-quiet — a selector that does
//! not match, not an exclusion; the descendant probes (`has-*`) stay
//! the reported diagnosed boundary, with or without a source.

use aeolus_compile::{
    AttrBit, AttrKind, AttrState, Config, Env, InterpNote, compile, compile_theme, interp,
    interp_probes, interp_theme_probes,
};
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    // The element's typed attribute bits (plan §3.5): the card carries
    // data-state="open", data-slot="card-action" and aria-busy="false".
    let state = AttrState::new(&[
        AttrBit {
            kind: AttrKind::Data,
            name: "state",
            value: "open",
        },
        AttrBit {
            kind: AttrKind::Data,
            name: "slot",
            value: "card-action",
        },
        AttrBit {
            kind: AttrKind::Aria,
            name: "busy",
            value: "false",
        },
    ]);

    // A probe layer FIRES under the state: data-[state=open]:p-8 wins
    // the longhand cascade over the base p-4.
    let compiled = compile(
        "p-4 data-[state=open]:p-8 aria-busy:p-8",
        &Config::default(),
    )
    .expect("the class string compiles");
    let probed =
        interp_probes(&compiled.program, &Env::default(), &state).expect("the program resolves");
    println!("fired (data-state=open wins the cascade):");
    for decl in &probed.decls {
        println!("  {:?} -> {:?}", decl.prop, decl.val);
    }
    println!(
        "  notes: {} (answered misses are CSS-quiet)",
        probed.notes.len()
    );

    // The SAME program through the sourceless walk: the attribute
    // layers are the reported exclusion (the honest boundary the note
    // keeps for the plain entry points).
    let sourceless = interp(&compiled.program, &Env::default()).expect("the program resolves");
    println!(
        "\nsourceless (the plain interp): {} note(s)",
        sourceless.notes.len()
    );
    for note in &sourceless.notes {
        match note {
            InterpNote::AttributeUnqueried { cond, .. } => println!("  - {cond:?} unqueried"),
            _ => println!("  - {note}"),
        }
    }

    // The descendant probes stay the diagnosed boundary even WITH a
    // source: the plan §3.5 row keeps has-* honest failure.
    let compiled = compile("has-focus:p-8 data-[state=open]:p-8", &Config::default())
        .expect("the class string compiles");
    let probed =
        interp_probes(&compiled.program, &Env::default(), &state).expect("the program resolves");
    println!(
        "\nwith a source: {} fired, {} note(s) (the has- descendant probe reports)",
        probed.decls.len(),
        probed.notes.len()
    );

    // Theme dynamism rides the same walk: tokens through the
    // TokenSource, probes through the AttrProbe — both seams, one pass.
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::runtime(&source).expect("default-seeded source builds");
    let compiled = compile_theme(
        "bg-primary data-[state=open]:p-8",
        &table,
        &Config::default(),
    )
    .expect("the class string compiles");
    let themed = interp_theme_probes(&compiled.program, &table, &Env::default(), &state)
        .expect("the program resolves");
    println!(
        "\nthemed + probed: {} declaration(s), {} note(s)",
        themed.decls.len(),
        themed.notes.len()
    );
}
