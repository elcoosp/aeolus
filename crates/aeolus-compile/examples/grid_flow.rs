//! The grid auto-flow family demo (the plan §3.7 grid row's auto-flow
//! half): the `grid-flow-*` classes through every surface the pipeline
//! arms them on — the `GridAutoFlow` longhand's keyword slots, the
//! annotation seam's v4 value texts, the interp arm, the CSS arm's rule
//! text, the static arm's frozen item and the honest boundaries (the
//! plan bridge's `SlotUnsupported` note, the grid-auto-rows precedent;
//! the auto-TRACK half of the boundary line rides the composite
//! grammar's verbatim slot — see the composites demo).
//!
//! Run: `cargo run -p aeolus-compile --example grid_flow`

use aeolus_compile::{Config, Env, compile, emit_css, emit_plan, interp, keyword_texts};

fn main() {
    let config = Config::default();

    // 1. The five keyword slots lower onto the typed longhand — the
    //    v4.1 scale in slot order.
    println!("== the flow scale lowers onto the GridAutoFlow longhand ==");
    let compiled = compile(
        "grid-flow-row grid-flow-col grid-flow-dense grid-flow-row-dense grid-flow-col-dense",
        &config,
    )
    .expect("non-empty input compiles");
    println!(
        "diagnostics: {:?} (every slot lowered)",
        compiled.diagnostics.len()
    );
    for decl in compiled.program.layers[0].decls {
        println!("  {:?} = {:?}", decl.prop, decl.val);
    }

    // 2. The annotation seam: the keyword texts ARE the CSS values
    //    (`col` renders `column`, the dense pairs the two-value form).
    println!("\n== the annotation seam: the v4 value texts ==");
    let classes = keyword_texts("grid-flow-col grid-flow-row-dense", &config).expect("annotates");
    for class in &classes {
        for (prop, text) in &class.keywords {
            println!("  {prop:?} = {text:?}");
        }
    }

    // 3. The interp arm: the effective declarations under one probe env.
    println!("\n== the interp arm ==");
    let interped = interp(&compiled.program, &Env::default()).expect("all values resolve");
    for decl in &interped.decls {
        println!("  {:?} under {:?}", decl.prop, decl.val);
    }

    // 4. The CSS arm: the rule text the cascade consumes.
    println!("\n== the CSS arm ==");
    let sheet = emit_css("grid grid-flow-col-dense", &config).expect("non-empty input renders");
    println!("{sheet}");

    // 5. The honest boundary: the v0.x plan carries no auto-flow slot —
    //    the typed note, never a silent drop. The auto-TRACK lists ride
    //    the composite grammar's verbatim slot (their own note kind).
    println!("== the plan bridge: the typed notes ==");
    let sheet = emit_plan("grid grid-flow-col-dense grid-cols-[auto_1fr]", &config)
        .expect("the plan sheet");
    for note in &sheet.notes {
        println!("  {note}");
    }
}
