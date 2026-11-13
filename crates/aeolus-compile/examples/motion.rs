//! The motion layer demo (the plan P6 row): the `transition-*`/
//! `duration-*`/`ease-*`/`delay-*`/`animate-*` families through every
//! surface the pipeline arms them on — the typed longhands, the interp
//! arm's effective declarations, the CSS arm's v4 property forms, the
//! static arm's frozen item and the honest boundaries (what refuses,
//! with its typed report).
//!
//! Run: `cargo run -p aeolus-compile --example motion`

use aeolus_compile::{Config, compile, emit_css, emit_rust, interp, sort};

fn main() {
    let config = Config::default();

    // 1. The five families compile into the IR's motion longhands — one
    //    declaration per family, the keyword slots indexing the row
    //    tables (the v4 default theme's own values).
    println!("== the motion families lower onto the typed longhands ==");
    let compiled = compile(
        "transition-colors duration-300 ease-in-out delay-150 animate-pulse",
        &config,
    )
    .expect("non-empty input compiles");
    println!(
        "diagnostics: {:?} (every family lowered)",
        compiled.diagnostics.len()
    );
    for decl in compiled.program.layers[0].decls {
        let slot = match decl.val {
            aeolus_ir::Val::Kw(slot) => format!("Kw({slot})"),
            aeolus_ir::Val::Num(n) => format!("Num({n})"),
            other => format!("{other:?}"),
        };
        println!("  {:?} = {}", decl.prop, slot);
    }

    // 2. The interp arm: the effective declarations under one probe env
    //    — the per-frame shape a host walks.
    println!("\n== the interp arm: the effective declarations ==");
    let interped = interp(&compiled.program, &Default::default()).expect("all values resolve");
    for decl in &interped.decls {
        println!("  {:?} under {:?}", decl.prop, decl.val);
    }

    // 3. The CSS arm: the v4 property forms, verbatim row texts.
    println!("\n== the CSS arm: the rendered stylesheet ==");
    let sheet = emit_css(
        "transition-colors duration-300 ease-in-out delay-150 animate-spin",
        &config,
    )
    .expect("non-empty input renders");
    print!("{sheet}");

    // 4. The static arm: the byte-stable frozen item (the first lines).
    println!("== the static arm: the frozen item (head) ==");
    let frozen = compiled.emit_static();
    for line in frozen.lines().take(6) {
        println!("  {line}");
    }

    // 5. The rust-tokens arm: the row texts annotate the slots.
    println!("\n== the rust-tokens arm: the annotation rides ==");
    let sheet = emit_rust("animate-spin transition-colors", &config).expect("compiles");
    for line in sheet.to_string().lines().take(10) {
        println!("  {line}");
    }

    // 6. The honest boundaries: arbitrary timing values and unknown
    //    preset names refuse typed (the read side's shape rejection
    //    names the root and the shapes it tried), negation refuses on
    //    the motion roots.
    println!("\n== the honest boundaries ==");
    for class in ["duration-[2s]", "animate-wiggle", "-duration-100"] {
        let compiled = compile(class, &config).expect("compiles");
        let report = compiled
            .diagnostics
            .first()
            .map(|d| format!("{} [{}]", d.kind, d.kind.code()))
            .unwrap_or_else(|| "no report".to_string());
        println!("  {class:>28} -> {report}");
    }

    // 7. The class-sort seam rides the registry order: the motion
    //    families sort into the registry's interaction tail.
    println!("\n== the sort seam: the registry's canonical order ==");
    let sorted = sort("animate-spin p-4 transition-colors duration-300").expect("compiles");
    for class in &sorted.classes {
        println!("  {}", class.raw);
    }

    // 8. The GPUI boundary (the §3.7 motion row) is the codegen target's
    //    own gate: `aeolus-codegen-gpui` refuses the motion longhands per
    //    longhand (`with_animation` needs an element id + state the
    //    frozen program does not name) — its tests pin the refusal set,
    //    this crate stays free of the UI-facing dependency edge.
}
