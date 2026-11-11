//! Consumer demonstration: the plan-lowering emit arm (`Target::Plan`
//! — architecture plan §7's migration seam). The V1 pipeline's compiled
//! artifact lowered into the v0.x [`StylePlan`] the five backend crates
//! already consume — the seam that lets aeolus-xilem / -floem / -iced /
//! -egui / -slint style from `aeolus-compile` output while the P3
//! codegen crates assemble.
//!
//! ```sh
//! cargo run -p aeolus-compile --example plan_bridge
//! ```
//!
//! The honesty contract rides the artifact: the same spanned, coded,
//! helped diagnostics the IR walk produces for every class that does
//! not lower, plus the arm's own typed notes for every class that
//! lowers but has no v0.x slot — never a silent drop. The DIFFERENTIAL
//! oracle (`tests/plan_bridge.rs`) pins the plans slot-for-slot against
//! the v0.x `Resolver` fed the same theme CSS.

use aeolus_compile::{Config, PlanNoteKind, Target, emit_plan_theme};
use aeolus_core::plan::Slot;
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    // The same shadcn CSS the v0.x `shadcn_default()` resolver reads —
    // one theme input, two generations in agreement (the differential's
    // premise).
    let source = ThemeSource::shadcn().expect("the embedded palette parses");
    let tokens = TokenTable::baked(&source).expect("the source builds a table");

    // The shadcn Button's core classes through the V1 pipeline:
    let input = "inline-flex items-center justify-center gap-2 whitespace-nowrap \
                 rounded-md text-sm font-medium bg-primary text-primary-foreground \
                 px-4 py-2 h-9 hover:bg-primary/90 focus-visible:ring-2 \
                 disabled:opacity-50 transition-colors";
    let sheet = emit_plan_theme(input, &tokens, &Config::default())
        .expect("the class string emits (only the empty input hard-fails)");

    println!(
        "Target::Plan — {} classes in, one StylePlan out:\n",
        input.split_whitespace().count()
    );

    // The base layer: the slots the backends read, in the plan shape
    // every backend lowering already consumes.
    let base = &sheet.plan.base;
    if let Some(layout) = &base.layout {
        println!("  base.layout       = {layout:?}");
    }
    if let Some(padding) = &base.padding {
        println!(
            "  base.padding      = {padding:?} (edges set: {:?})",
            (
                padding.set.top,
                padding.set.right,
                padding.set.bottom,
                padding.set.left
            )
        );
    }
    if let Some(background) = &base.background {
        let srgba = background.0.to_srgba();
        println!(
            "  base.background   = srgba({:.3}, {:.3}, {:.3}) alpha {:.2} (bg-primary through the token table)",
            srgba[0], srgba[1], srgba[2], srgba[3]
        );
    }
    if !base.text.is_empty() {
        println!("  base.text         = {:?}", base.text);
    }

    // The conditional buckets: the classic states landed as layers, in
    // the bucket shape the backends' Catalog closures / status machineries
    // resolve per frame.
    println!();
    for layer in &sheet.plan.layers {
        println!(
            "  layer {:?} — {} slot(s) set",
            layer.condition,
            layer.style.slots().len()
        );
        if let Some(background) = &layer.style.background {
            let srgba = background.0.to_srgba();
            println!(
                "      background srgba({:.3}, {:.3}, {:.3}) alpha {:.2}",
                srgba[0], srgba[1], srgba[2], srgba[3]
            );
        }
    }

    // The honest reports: `transition-colors` has no V1 registry row
    // yet, and the arm's notes name every longhand the v0.x plan shape
    // cannot carry (none for this string — the diagnostics carry the
    // registry gap).
    println!();
    for diagnostic in &sheet.diagnostics {
        println!(
            "  diagnostic [{}] {} — {}",
            diagnostic.kind.code(),
            diagnostic.raw,
            diagnostic.kind.help()
        );
    }
    for note in &sheet.notes {
        println!("  note: {note}");
    }

    // The arm surface: the fifth emit form beside the four the Target
    // enum named before it.
    println!(
        "\nthe arm surface: {:?} — Target::Plan renders the v0.x plan shape",
        [
            Target::Css,
            Target::Rust,
            Target::Interp,
            Target::Static,
            Target::Plan
        ]
    );
    let _ = Slot::Background; // the slot set the diff/caps machinery keys on
    let _ = PlanNoteKind::GradientRow; // the typed boundary set, importable
}
