//! The per-backend conformance score demo (architecture plan §5.1:
//! "Publish a per-backend conformance score (WPT / caniuse-style
//! badge)"): the [`Capability`] seam read two ways — the GPUI target's
//! own [`Caps`] impl (the emit gates' tables, so the score IS the
//! emitter's verdicts) and a hand-rolled text target defined right
//! here, showing how a third-party backend publishes its own report
//! from its own capability tables.
//!
//! Run: `cargo run -p aeolus-codegen-gpui --example conformance`

use aeolus_codegen::conformance::{Capability, Conformance, Tier};
use aeolus_codegen_gpui::Caps;
use aeolus_ir::Prop;

/// The demo third-party target: a terminal renderer whose surface
/// carries the layout/box/typography core natively, the ring family
/// composed (the §3.7 polyfill precedent), everything else refused —
/// the three-tier shape a real backend's tables take.
struct TextTarget;

impl Capability for TextTarget {
    fn name(&self) -> &'static str {
        "text-target"
    }

    fn tier(&self, prop: Prop) -> Tier {
        if supports(prop) {
            Tier::Native
        } else if matches!(prop, Prop::RingWidth | Prop::RingColor) {
            Tier::Polyfill
        } else {
            Tier::Refused
        }
    }
}

/// The demo's native set: the positioning/flex/box/typography core the
/// target renders (a deliberate subset — the boundaries stay honest).
fn supports(prop: Prop) -> bool {
    matches!(
        prop,
        Prop::Display
            | Prop::Position
            | Prop::Top
            | Prop::Right
            | Prop::Bottom
            | Prop::Left
            | Prop::FlexDirection
            | Prop::FlexWrap
            | Prop::FlexGrow
            | Prop::FlexShrink
            | Prop::JustifyContent
            | Prop::AlignItems
            | Prop::GapRow
            | Prop::GapCol
            | Prop::Width
            | Prop::Height
            | Prop::PaddingTop
            | Prop::PaddingRight
            | Prop::PaddingBottom
            | Prop::PaddingLeft
            | Prop::MarginTop
            | Prop::MarginRight
            | Prop::MarginBottom
            | Prop::MarginLeft
            | Prop::Background
            | Prop::Color
            | Prop::FontSize
            | Prop::FontWeight
            | Prop::TextAlign
    )
}

fn main() {
    println!("== the registered backends' conformance scores (plan §5.1) ==\n");

    // The GPUI target's score reads the emit gates' own tables — the
    // report cannot drift from what the emitter actually refuses.
    print!("{}", Conformance::of(&Caps));

    // A third-party backend publishes its own report from its own
    // tables, the same seam:
    let report = Conformance::of(&TextTarget);
    println!("\n--- head of the text target's table ---");
    for line in report.to_string().lines().take(10) {
        println!("{line}");
    }

    // The score arithmetic rides the partition:
    println!(
        "\nnative {} + polyfill {} = refused {} (total {})",
        report.native().len(),
        report.polyfill().len(),
        report.refused().len(),
        report.total(),
    );
}
