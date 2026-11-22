//! Consumer demonstration: the parent-relative composition tier (the
//! `group-*:`/`peer-*:` spellings over the FULL probe-state set — the
//! tailwindcss v4.1.17 `group`/`peer` compound mechanisms compose with
//! every StyleRules-eligible variant; where only hover/focus folded
//! before, `group-active:`, `peer-disabled:`, `group-focus-visible:`
//! and friends now fold beside them). Each composition is its own
//! typed [`Cond`] probe and its own [`Env`] frame fact — a `.group`
//! ancestor (or `.peer` sibling) IN the composed state; the unset bit
//! is the QUIET miss a frame the composition does not hold in (the
//! media facts' own reading — no probe seam, no sourceless note), the
//! CSS arm renders the composed state on the relation's own head
//! (`.group:active`, the sibling `.peer:disabled ~`), and the plan
//! bridge lands the layer in the same labelled group/peer buckets the
//! landed pair rides.
//!
//! ```sh
//! cargo run -p aeolus-compile --example group_peer_states
//! ```

use aeolus_compile::{Config, Env, compile, emit_css, interp};
use aeolus_ir::{Decl, Prop, Val};

/// The winning declaration for ONE longhand (the interp tests' own
/// projection — `Interped::decls` is `Prop`-sorted, one per longhand).
fn resolved(decls: &[Decl], prop: Prop) -> Val {
    decls
        .iter()
        .find(|d| d.prop == prop)
        .unwrap_or_else(|| panic!("{prop:?} is in the resolved declarations"))
        .val
}

fn main() {
    let config = Config::default();

    println!("── the program: p-4 group-active:p-8 peer-disabled:p-2 ──");
    let compiled = compile("p-4 group-active:p-8 peer-disabled:p-2", &config).expect("compiles");
    for layer in compiled.program.layers {
        println!("  layer: {:?}", layer.cond);
    }
    assert!(
        compiled.diagnostics.is_empty(),
        "the compositions fold clean: {:?}",
        compiled.diagnostics
    );

    println!();
    println!("── the interp walk: a .group ancestor IS active ──");
    let interped = interp(
        &compiled.program,
        &Env {
            group_active: true,
            ..Env::default()
        },
    )
    .expect("resolves");
    assert_eq!(
        resolved(&interped.decls, Prop::PaddingTop),
        Val::Rem(2.0),
        "the group-active layer answers its frame fact"
    );
    println!(
        "  padding-top: {:?}",
        resolved(&interped.decls, Prop::PaddingTop)
    );

    println!();
    println!("── the interp walk: a .peer sibling IS disabled ──");
    let interped = interp(
        &compiled.program,
        &Env {
            peer_disabled: true,
            ..Env::default()
        },
    )
    .expect("resolves");
    assert_eq!(
        resolved(&interped.decls, Prop::PaddingTop),
        Val::Rem(0.5),
        "the peer-disabled layer answers its frame fact"
    );
    println!(
        "  padding-top: {:?}",
        resolved(&interped.decls, Prop::PaddingTop)
    );

    println!();
    println!("── the resting env: the compositions stay the quiet miss ──");
    let resting = interp(&compiled.program, &Env::default()).expect("resolves");
    assert_eq!(
        resolved(&resting.decls, Prop::PaddingTop),
        Val::Rem(1.0),
        "the base declaration answers; the composition layers wait for their facts"
    );
    println!(
        "  padding-top: {:?}",
        resolved(&resting.decls, Prop::PaddingTop)
    );

    println!();
    println!("── the CSS arm: the composed state rides the relation's head ──");
    let sheet = emit_css(
        "group-active:p-4 peer-disabled:p-2 group-focus-visible/menu:p-1",
        &config,
    )
    .expect("the sheet renders");
    let text = sheet.to_string();
    println!("{text}");
    assert!(
        text.contains(".group:active .group-active\\:p-4 {"),
        "the group head renders the composed state"
    );
    assert!(
        text.contains(".peer:disabled ~ .peer-disabled\\:p-2 {"),
        "the peer head renders the composed state (the sibling relation)"
    );
    assert!(
        text.contains(".group\\/menu:focus-visible .group-focus-visible\\/menu\\:p-1 {"),
        "the label form rides the head (the group-hover/menu precedent)"
    );

    println!();
    println!("── the label form rides the IR slot (the named condition) ──");
    let labelled = compile("group-active/menu:p-0", &config).expect("compiles");
    println!("  layer: {:?}", labelled.program.layers[0].cond);
    assert!(matches!(
        labelled.program.layers[0].cond,
        aeolus_ir::Cond::GroupActive { name: Some("menu") }
    ));

    println!();
    println!("── every composition spelling folds (the closed set) ──");
    let spellings = [
        "group-focus-visible",
        "group-focus-within",
        "group-active",
        "group-disabled",
        "group-visited",
        "group-checked",
        "peer-focus-visible",
        "peer-focus-within",
        "peer-active",
        "peer-disabled",
        "peer-visited",
        "peer-checked",
    ];
    for spelling in spellings {
        let input = format!("{spelling}:p-0");
        let compiled = compile(&input, &config).expect("compiles");
        assert!(
            compiled.diagnostics.is_empty(),
            "{spelling}: reports {:?}",
            compiled.diagnostics
        );
        println!("  {spelling}: → {:?}", compiled.program.layers[0].cond);
    }

    println!();
    println!(
        "the composition tier: each probe its own fact, the quiet miss a frame does not hold."
    );
}
