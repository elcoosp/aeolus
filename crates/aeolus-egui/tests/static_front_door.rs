//! The backend-neutral static front door, pinned from THIS backend's
//! side (plan §3.6 surface 1–2 for every backend): the prelude
//! re-exports freeze programs at EXPANSION time without the GPUI gate,
//! `lower_program` consumes them, and the `LoweringReport` stays the
//! honesty channel — the documented GPUI refusal (`blur-md`, the filter
//! family's emitter verdict) freezes here and lands in the report, never
//! a compile error and never a silent no-op.

use aeolus_egui::EguiProbe;
use aeolus_egui::{cn_static, tw_static, variants_static};
use aeolus_egui::{env_for_probe, lower_program};
use aeolus_rt::Env;

#[test]
fn the_prelude_re_export_freezes_and_lowers() {
    let program =
        tw_static!("p-4 bg-primary text-primary-foreground rounded-md w-1/2 hover:bg-accent");
    assert!(
        !program.layers.is_empty(),
        "the frozen program carries layers"
    );
    let (payload, report) = lower_program(&program, &env_for_probe(&EguiProbe::default()));
    let _ = payload;
    assert!(
        report.is_empty(),
        "the plain surface lowers clean on this backend: {:?}",
        report.dropped
    );
}

#[test]
fn the_gpui_refusal_freezes_on_the_neutral_lane() {
    // `blur-md` is the tw!/variants! compile error (the GPUI gate's
    // filter verdict). The neutral entry has no gate — the freeze is
    // the host's choice, the report the honesty channel.
    let program = tw_static!("blur-md");
    assert!(
        !program.layers.is_empty(),
        "the refused class still freezes"
    );
    let (_, report) = lower_program(&program, &env_for_probe(&EguiProbe::default()));
    assert!(
        !report.is_empty(),
        "what this backend cannot express lands in the report, never a silent no-op"
    );
}

#[test]
fn the_neutral_merge_freezes_the_merged_program() {
    let program = cn_static!("p-4", "px-8 bg-primary");
    assert!(!program.layers.is_empty());
    let (_, report) = lower_program(&program, &env_for_probe(&EguiProbe::default()));
    assert!(report.is_empty(), "{:?}", report.dropped);
}

#[test]
fn the_neutral_variants_table_freezes_without_the_gate() {
    variants_static! {
        pub Chip {
            base: "p-2 rounded-md",
            variant: { solid: "bg-primary text-primary-foreground", soft: "bg-accent/50" },
        }
    }
    let solid = Chip::style(ChipVariant::Solid);
    assert!(!solid.layers.is_empty());
    let (_, report) = lower_program(&solid, &Env::default());
    assert!(report.is_empty(), "{:?}", report.dropped);
    let soft = Chip::style(ChipVariant::Soft);
    assert!(!soft.layers.is_empty());
}
