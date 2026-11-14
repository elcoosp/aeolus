//! The filter layer demo (the plan §3.7 backlog row's pipeline half):
//! the `blur-*`/`brightness-*`/`contrast-*`/`grayscale-*`/
//! `hue-rotate-*`/`invert-*`/`saturate-*`/`sepia-*`/`drop-shadow-*`/
//! `filter` families through every surface the pipeline arms them on —
//! the typed longhands, the interp arm's effective declarations, the
//! CSS arm's v4 var-chain form (the primitive's var line beside the
//! composed chain), the static arm's frozen item and the honest
//! boundaries (what refuses, with its typed report).
//!
//! Run: `cargo run -p aeolus-compile --example filters`

use aeolus_compile::{Config, Env, compile, emit_css, interp, keyword_texts};

fn main() {
    let config = Config::default();

    // 1. The ten families compile into the IR's filter longhands — one
    //    declaration per family, the keyword slots indexing the row
    //    tables (the v4 default theme's own values), the number slots
    //    the raw percent/degree scale.
    println!("== the filter families lower onto the typed longhands ==");
    let compiled = compile(
        "blur-md brightness-110 contrast-125 grayscale invert hue-rotate-30 \
         saturate-150 sepia drop-shadow-md",
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
        println!("  {:?} = {slot}", decl.prop);
    }

    // 2. The annotation seam: the row texts the slots ride — the v4
    //    default theme's own values, the bare `blur` default included.
    println!("\n== the annotation seam: the row texts ==");
    let classes = keyword_texts(
        "blur blur-none blur-md drop-shadow drop-shadow-none grayscale invert sepia",
        &config,
    )
    .expect("annotates");
    for class in &classes {
        for (prop, text) in &class.keywords {
            println!("  {prop:?} = {text:?}");
        }
    }

    // 3. The interp arm: the effective declarations under one probe env
    //    — the per-frame shape a host walks.
    println!("\n== the interp arm: the effective declarations ==");
    let interped = interp(&compiled.program, &Env::default()).expect("all values resolve");
    for decl in &interped.decls {
        println!("  {:?} under {:?}", decl.prop, decl.val);
    }

    // 4. The CSS arm: the v4 var-chain form — each primitive rule
    //    renders its var line beside the composed chain (identical in
    //    every rule, so the cascade never loses a component).
    println!("\n== the CSS arm: the rendered stylesheet ==");
    let sheet = emit_css("blur-md brightness-110 drop-shadow-md filter-none", &config)
        .expect("non-empty input renders");
    print!("{sheet}");

    // 5. The static arm: the byte-stable frozen item (the first lines).
    println!("== the static arm: the frozen item (head) ==");
    let frozen = compiled.emit_static();
    for line in frozen.lines().take(6) {
        println!("  {line}");
    }

    // 6. The honest boundaries: bare `brightness` (v4 has no default),
    //    the composite arbitrary values (the codegen host's surface),
    //    and the GPUI emit gate (no filter slot — the CSS arm is the
    //    consumer).
    println!("\n== the honest boundaries ==");
    for class in ["brightness", "filter-[blur(2px)_brightness(0.5)]"] {
        let compiled = compile(class, &config).expect("the walk is total");
        let kinds: Vec<_> = compiled.diagnostics.iter().map(|d| d.kind.code()).collect();
        println!("  {class:?} -> {kinds:?} ({})", compiled.diagnostics.len());
    }
}
