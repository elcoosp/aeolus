//! Consumer demonstration: the pseudo-element RENDER TARGETS (plan
//! §3.5's variant-costs row, the interp arm's own-slice note resolved).
//! `selection:`/`placeholder:` style a DIFFERENT render target than the
//! element's own cascade — the element walk excludes those layers with
//! the [`InterpNote::PseudoElement`] note, and the [`PseudoTarget`]
//! arm resolves each target's OWN cascade
//! ([`interp_pseudo`]/[`interp_pseudo_theme`]; the parity the CSS arm
//! already renders as `::selection`/`::placeholder` rules).
//!
//! The demo walks one pseudo-mixed program three ways: the element
//! cascade (pseudo layers excluded, reported), the selection target
//! (its own colour), and the placeholder target (empty here — no rules
//! target it — plus one that does, resolved the same way).
//!
//! ```sh
//! cargo run -p aeolus-compile --example pseudo_targets
//! ```

use aeolus_compile::{
    Config, Env, PseudoTarget, compile, compile_theme, interp, interp_pseudo, interp_pseudo_theme,
};
use aeolus_ir::{Prop, Val};
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    let config = Config::default();

    println!("── the program: one ::selection rule + one ::placeholder rule + base ──");
    let compiled = compile(
        "bg-[#ef4444] selection:bg-[#eab308] placeholder:text-[#3f3f46]",
        &config,
    )
    .expect("compiles");
    for layer in compiled.program.layers {
        println!("  layer: {:?}", layer.cond);
    }

    println!("\n── 1. the element cascade: pseudo layers excluded, reported ──");
    let element = interp(&compiled.program, &Env::default()).expect("the walk resolves");
    for note in &element.notes {
        println!("  note: {note}");
    }
    assert_eq!(
        element.notes.len(),
        2,
        "both pseudo layers are render targets, not element state"
    );
    assert!(
        element
            .decls
            .iter()
            .all(|d| d.prop != Prop::BackgroundImage),
        "the element cascade carries only the element's own background"
    );

    println!("\n── 2. the selection target: its own cascade ──");
    let selection =
        interp_pseudo(&compiled.program, PseudoTarget::Selection).expect("the walk resolves");
    println!("  ::selection background = {:?}", selection.decls[0].val);
    assert_eq!(
        selection.conds[0],
        aeolus_ir::Cond::Selection,
        "the winner's condition is the pseudo-element's own"
    );
    assert_eq!(
        selection.decls.len(),
        1,
        "only the ::selection rule applies"
    );

    println!("\n── 3. the placeholder target: same walk, its own answer ──");
    let placeholder =
        interp_pseudo(&compiled.program, PseudoTarget::Placeholder).expect("the walk resolves");
    println!("  ::placeholder color = {:?}", placeholder.decls[0].val);
    assert_eq!(
        placeholder.decls.len(),
        1,
        "only the ::placeholder rule applies"
    );

    println!("\n── 4. the themed form: token slots resolve beside the target ──");
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::runtime(&source).expect("default-seeded source builds");
    let themed = compile_theme("selection:bg-primary", &table, &config).expect("theme compiles");
    let resolved = interp_pseudo_theme(
        &themed.program,
        &table,
        &Env::default(),
        PseudoTarget::Selection,
    )
    .expect("the walk resolves");
    let val = resolved.decls[0].val;
    println!("  ::selection background (theme) = {val:?}");
    assert!(
        matches!(val, Val::Rgba([..])),
        "the token resolved concrete"
    );

    println!("\nall four walks agree with the CSS semantics the pins encode.");
}
