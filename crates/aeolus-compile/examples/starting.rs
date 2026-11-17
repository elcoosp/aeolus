//! Consumer demonstration: the `starting:` variant (`@starting-style`,
//! the transition's first frame — the at-rule tier the media-feature
//! families ride, the pseudo-element landing's deliberately deferred
//! sibling now landed). The typed [`Cond::Starting`] at-rule, the ONE
//! affirmative `Env` frame fact (`starting_style` — the host's
//! first-render gate beside the transition) and the plain [`interp`]
//! walk: an unset bit is the QUIET miss a frame the element is not in
//! is (the media facts' own reading — no probe seam, no sourceless
//! note), the CSS arm renders the bare `@starting-style` head, and the
//! plan bridge reports the v0.x boundary through the typed
//! [`PlanNoteKind::StartingStyle`] note.
//!
//! ```sh
//! cargo run -p aeolus-compile --example starting
//! ```

use aeolus_compile::{Config, Env, PlanNoteKind, compile, emit_css, emit_plan, interp};
use aeolus_ir::{Prop, Val};

fn main() {
    let config = Config::default();

    println!("── the program: starting:p-8 p-4 ──");
    let compiled = compile("starting:p-8 p-4", &config).expect("compiles");
    for layer in compiled.program.layers {
        println!("  layer: {:?}", layer.cond);
    }
    assert!(
        compiled
            .program
            .layers
            .iter()
            .any(|layer| layer.cond == aeolus_ir::Cond::Starting),
        "the spelling folds to Cond::Starting (the at-rule tier, specificity 4)"
    );

    println!("\n── 1. the resting env: the layer is the quiet miss ──");
    let resting = interp(&compiled.program, &Env::default()).expect("the walk resolves");
    assert_eq!(
        resting
            .decls
            .iter()
            .find(|d| d.prop == Prop::PaddingTop)
            .map(|d| d.val),
        Some(Val::Rem(1.0)),
        "only the base layer's padding answers"
    );
    assert!(
        resting.notes.is_empty(),
        "the frame facts carry no sourceless note (the media tier's reading)"
    );
    println!("  padding = 1rem (the base layer); no notes");

    println!("\n── 2. the frame fact set: the starting-style layer fires ──");
    let env = Env {
        starting_style: true,
        ..Env::default()
    };
    let fired = interp(&compiled.program, &env).expect("the walk resolves");
    assert_eq!(
        fired
            .decls
            .iter()
            .find(|d| d.prop == Prop::PaddingTop)
            .map(|d| d.val),
        Some(Val::Rem(2.0)),
        "the starting-style layer wins the cascade last-wins"
    );
    println!("  padding = 2rem (the starting-style layer, the first frame's own reading)");

    println!("\n── 3. the CSS arm renders the bare at-rule head ──");
    let sheet = emit_css("starting:p-8", &config).expect("the emit resolves");
    let at = sheet.rules[0]
        .media
        .as_ref()
        .map(|at| at.text())
        .expect("the at-rule head rides");
    println!("  rule head: {at}");
    assert_eq!(at, "@starting-style", "the upstream registration text");

    println!("\n── 4. the plan bridge reports the v0.x boundary ──");
    let sheet = emit_plan("starting:p-8", &config).expect("the plan lowers");
    let note = sheet
        .notes
        .iter()
        .find(|note| matches!(note.kind, PlanNoteKind::StartingStyle))
        .expect("the typed note rides");
    println!("  note code: {}", note.kind.code());
    assert_eq!(
        note.kind.code(),
        "aeolus::compile::plan::starting_style",
        "the stable machine code"
    );

    println!("\nall starting-style demos held — the honest boundaries are the tests' business");
}
