//! The generated capability table's contracts (the §3.7 generator
//! slice's drift pin): the committed `caps_generated.rs` is the emit
//! gates' OWN verdicts over `Prop::ALL` — every longhand's tier plus,
//! for a refused longhand, the typed boundary reason the emit gate
//! answers — rendered by `cargo xtask gen-gpui-caps` from
//! [`aeolus_codegen_gpui::Caps`], and pinned equal HERE in `cargo test`
//! (without the driver), the same shape the registry's generated index
//! takes. A capability-table edit that forgets the regeneration fails
//! this battery first.
//!
//! Written FIRST (the RED spec): the suite fails against the
//! not-yet-existing seam and passes only once the generated table
//! satisfies every pin below.

use aeolus_codegen::conformance::{Capability, Conformance, Tier};
use aeolus_codegen_gpui::{Caps, PROP_CAPABILITIES};
use aeolus_ir::Prop;

#[test]
fn the_table_covers_every_longhand_exactly_once_in_order() {
    assert_eq!(
        PROP_CAPABILITIES.len(),
        Prop::ALL.len(),
        "one row per IR longhand, no gaps: {} vs {}",
        PROP_CAPABILITIES.len(),
        Prop::ALL.len()
    );
    for (row, prop) in PROP_CAPABILITIES.iter().zip(Prop::ALL) {
        assert_eq!(
            row.prop, *prop,
            "the rows ride Prop::ALL's own (cascade) order"
        );
    }
}

#[test]
fn the_table_is_the_capability_impls_own_verdicts() {
    // The publish-the-emitter's-verdicts rule, pinned row for row: the
    // table's tier IS `Caps`' answer — never a parallel claim.
    for row in PROP_CAPABILITIES {
        assert_eq!(
            row.tier,
            Caps.tier(row.prop),
            "row {} drifted from the capability impl",
            row.prop.name()
        );
    }
}

#[test]
fn the_table_agrees_with_the_published_partition() {
    // The generated rows and the conformance report (the committed
    // docs page's source) are two renderings of ONE partition — they
    // must list the same longhands, in the same order.
    let report = Conformance::of(&Caps);
    let native: Vec<Prop> = PROP_CAPABILITIES
        .iter()
        .filter(|row| row.tier == Tier::Native)
        .map(|row| row.prop)
        .collect();
    let polyfill: Vec<Prop> = PROP_CAPABILITIES
        .iter()
        .filter(|row| row.tier == Tier::Polyfill)
        .map(|row| row.prop)
        .collect();
    let refused: Vec<Prop> = PROP_CAPABILITIES
        .iter()
        .filter(|row| row.tier == Tier::Refused)
        .map(|row| row.prop)
        .collect();
    assert_eq!(native, report.native(), "the native partition agrees");
    assert_eq!(polyfill, report.polyfill(), "the polyfill partition agrees");
    assert_eq!(refused, report.refused(), "the refused partition agrees");
}

#[test]
fn refused_rows_carry_the_typed_boundary_and_the_others_stay_empty() {
    for row in PROP_CAPABILITIES {
        match row.tier {
            Tier::Refused => assert!(
                !row.reason.is_empty(),
                "a refused longhand names its boundary: {}",
                row.prop.name()
            ),
            Tier::Native | Tier::Polyfill => assert_eq!(
                row.reason,
                "",
                "an expressible longhand carries no refusal text: {}",
                row.prop.name()
            ),
        }
    }
}

#[test]
fn the_boundaries_keep_their_typed_reasons() {
    // Spot pins, the §3.7 matrix's own rows: the ring family rides the
    // polyfill tier (the composed box-shadow list), a transition
    // longhand refuses with the motion row's own text (the static
    // emit's documented boundary), and the gradient direction row's
    // refusal names the annotation refinement (the verdict is the row
    // TEXT's — the per-longhand table says so honestly).
    let ring = PROP_CAPABILITIES
        .iter()
        .find(|row| row.prop == Prop::RingWidth)
        .expect("the ring longhand rides");
    assert_eq!(ring.tier, Tier::Polyfill, "the ring family polyfills");
    let transition = PROP_CAPABILITIES
        .iter()
        .find(|row| row.prop == Prop::TransitionProperty)
        .expect("the transition longhand rides");
    assert_eq!(transition.tier, Tier::Refused);
    assert!(
        transition.reason.contains("no transition slot"),
        "the motion row's own boundary rides: {}",
        transition.reason
    );
    let z_index = PROP_CAPABILITIES
        .iter()
        .find(|row| row.prop == Prop::ZIndex)
        .expect("the z-index longhand rides");
    assert_eq!(z_index.tier, Tier::Refused);
    assert!(
        z_index.reason.contains("no native slot"),
        "the per-prop fallback's own text rides: {}",
        z_index.reason
    );
    let direction = PROP_CAPABILITIES
        .iter()
        .find(|row| row.prop == Prop::BackgroundImage)
        .expect("the direction longhand rides");
    assert_eq!(direction.tier, Tier::Refused);
    assert!(
        direction.reason.contains("annotation"),
        "the row-TEXT refinement names itself: {}",
        direction.reason
    );
}
