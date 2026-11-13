//! Consumer demonstration: the named-container BOUNDS seam (plan §3.5's
//! `@container` row, the resolution half the named-container fold left
//! open). A NAMED container query (`@md/card:`) needs THAT container's
//! width, and one [`Env::container_rem`] slot cannot answer per name —
//! the [`ContainerProbe`] seam carries the named containers' widths
//! into the walk (the built-in [`ContainerState`] static table; a host
//! wires its own layout-phase bridge through the same one question).
//!
//! The demo walks the same program three ways: sourceless (the named
//! layer REPORTS — the honest boundary), bounds-answered (the layer
//! FIRES at the boundary), and bounds-answered below it (a QUIET miss —
//! a CSS selector that does not match, no note either). The unnamed
//! layer keeps answering from the Env beside the seam the whole time.
//!
//! ```sh
//! cargo run -p aeolus-compile --example container_bounds
//! ```

use aeolus_compile::{
    Config, ContainerBound, ContainerState, Env, InterpNote, compile, interp, interp_containers,
};
use aeolus_ir::{Prop, Val};

fn main() {
    let config = Config::default();

    println!("── the program: two named containers + one unnamed ──");
    let compiled =
        compile("@md/card:w-64 @max-lg/sidebar:hidden @md:flex", &config).expect("compiles");
    for layer in compiled.program.layers {
        println!("  layer: {:?}", layer.cond);
    }

    println!("\n── 1. sourceless: the named layers report, the unnamed answers ──");
    let env = Env {
        container_rem: Some(48.0),
        ..Env::default()
    };
    let sourceless = interp(&compiled.program, &env).expect("the walk resolves");
    for note in &sourceless.notes {
        println!("  note: {note}");
    }
    assert_eq!(sourceless.notes.len(), 2, "both named layers report");
    assert!(
        sourceless.decls.iter().any(|d| d.prop == Prop::Display),
        "the unnamed layer still answers from Env::container_rem"
    );

    println!("\n── 2. bounds answered: card at exactly md's 48rem ──");
    let bounds = [
        ContainerBound {
            name: "card",
            rem: 48.0,
        },
        ContainerBound {
            name: "sidebar",
            rem: 30.0,
        },
    ];
    let state = ContainerState::new(&bounds);
    let hot = interp_containers(&compiled.program, &env, &state).expect("the walk resolves");
    let width = hot
        .decls
        .iter()
        .find(|d| d.prop == Prop::Width)
        .expect("the named layer fired at the boundary");
    println!("  w-64 fires: width = {:?}", width.val);
    assert_eq!(width.val, Val::Rem(16.0), "w-64 lowers to 16rem");
    assert!(
        hot.notes.is_empty(),
        "every container question was answered: {:?}",
        hot.notes
    );

    println!("\n── 3. below the boundary: a QUIET miss, not an exclusion ──");
    let cold_bounds = [ContainerBound {
        name: "card",
        rem: 47.9,
    }];
    let cold_state = ContainerState::new(&cold_bounds);
    let cold = interp_containers(&compiled.program, &env, &cold_state).expect("the walk resolves");
    assert!(
        cold.decls.iter().all(|d| d.prop != Prop::Width),
        "below the named width the layer never fires"
    );
    assert!(
        cold.notes
            .iter()
            .all(|note| !matches!(note, InterpNote::ContainerUnqueried { .. })),
        "a source-answered miss is CSS-quiet, never an exclusion note"
    );
    println!("  width unset, no notes — the selector simply does not match");

    println!("\nall three walks agree with the CSS semantics the pins encode.");
}
