//! Consumer demonstration: the `tw!`/`cn!` macro surface (architecture
//! plan §3.6's surface (1)) — what the expansion actually EMITS. The
//! macro runs the full V1 pipeline at expansion time and freezes the
//! program's static form: the IR layers plus the row-keyword texts the
//! runtime backends map natively. The input string is never emitted
//! back (the F1 finding's fix) — this example prints the frozen
//! program a `tw!` expansion evaluates to, the compile-time merge a
//! `cn!` performs, and the capability gate's compile-time refusal the
//! un-expressible classes get, so a consumer can see exactly what data
//! lands in their binary.
//!
//! ```sh
//! cargo run -p aeolus-macro --example frozen_program
//! ```

use aeolus_macro::{cn, tw};

fn main() {
    // The static-flagship string from the README: one call site's
    // frozen program — base layer first, the fired variant layers
    // after, specificity-sorted.
    let program = tw!("flex items-center gap-2 p-4 rounded-md bg-primary \
         text-primary-foreground hover:bg-primary/90 dark:shadow-none md:p-6");

    println!("── the frozen program one tw! expansion evaluates to ──");
    println!("layers: {}", program.layers.len());
    for (i, layer) in program.layers.iter().enumerate() {
        println!("layer [{i}] cond {:?}:", layer.cond);
        for (j, decl) in layer.decls.iter().enumerate() {
            let text = program.keyword(i, j).map_or_else(
                || format!("{:?}", decl.val),
                |text| format!("{text:?} (row text)"),
            );
            println!("    {:?} = {text}", decl.prop);
        }
    }

    // The compile-time merge: `cn!` folds same-condition layers by
    // `Cond` equality and keeps the LAST occurrence per longhand —
    // `px-8` replacing `p-4`'s horizontal halves at EXPANSION time,
    // computed once, no runtime merge.
    let merged = cn!("p-4", "px-8");
    println!();
    println!("── cn!(\"p-4\", \"px-8\"): the merge the expansion computed ──");
    for (i, layer) in merged.layers.iter().enumerate() {
        for (j, decl) in layer.decls.iter().enumerate() {
            if matches!(
                decl.prop,
                aeolus_ir::Prop::PaddingLeft
                    | aeolus_ir::Prop::PaddingRight
                    | aeolus_ir::Prop::PaddingTop
                    | aeolus_ir::Prop::PaddingBottom
            ) {
                println!(
                    "layer [{i}] {:?} = {:?} (the row text {:?})",
                    decl.prop,
                    decl.val,
                    merged.keyword(i, j).unwrap_or("")
                );
            }
        }
    }

    // The capability gate: a class the GPUI target cannot express
    // refuses AT COMPILE TIME with the typed reason naming the lane
    // that works — `tw!("blur-md")` is a compile error, never a silent
    // no-op. (Uncomment to see the expansion fail:
    //
    //   let refused = tw!("blur-md");
    //
    // The error tail names the longhand and the emitter's own reason:
    // `GPUI has no blur/backdrop-filter slot — the spatial and backdrop
    // halves stay the CSS/interp arms' output`.)
    println!();
    println!("── the compile-time capability gate ──");
    println!("tw!(\"blur-md\") refuses at expansion: the filter tier's spatial half is");
    println!("the typed refusal; the polyfilled colour matrices emit (see the");
    println!("aeolus-gpui filters example for the walk half).");
}
