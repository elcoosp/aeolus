//! The committed capability table's drift pin (the conformance moat's
//! publish-the-emitter's-verdicts rule, plan §5.1): a fresh
//! `walk_capabilities()` replay must equal the committed
//! `caps_tables::XILEM_CAPABILITIES` rows — prop for prop, tier for tier, reason
//! for reason. A walk-verdict edit that forgets the regeneration fails
//! here first; the committed bytes are the SAME freeze `cargo xtask
//! gen-tables` writes and `cargo xtask verify-tables` gates.

use aeolus_codegen::caps_tables::XILEM_CAPABILITIES;
use aeolus_codegen::conformance::Tier;

#[test]
fn the_committed_table_is_the_walk_s_verdicts() {
    let walk = aeolus_xilem::caps::walk_capabilities();
    let table = XILEM_CAPABILITIES;
    assert_eq!(
        walk.len(),
        table.len(),
        "the row count is the total partition over Prop::ALL"
    );
    for ((prop, verdict, reason), row) in walk.iter().zip(table.iter()) {
        assert_eq!(prop, &row.prop, "the cascade order holds");
        let tier = match verdict {
            aeolus_xilem::caps::WalkVerdict::Native => Tier::Native,
            aeolus_xilem::caps::WalkVerdict::Polyfill => Tier::Polyfill,
            aeolus_xilem::caps::WalkVerdict::Refused => Tier::Refused,
        };
        assert_eq!(tier, row.tier, "{prop:?}: the tier");
        let frozen = match verdict {
            aeolus_xilem::caps::WalkVerdict::Refused => reason.as_str(),
            _ => "",
        };
        assert_eq!(frozen, row.reason, "{prop:?}: the frozen reason");
    }
}

/// The wave-1 verdicts at THIS lane (the parity wave's honest reading,
/// frozen beside the walk): the between-child families refuse with the
/// precise no-child-access reason, the clamp count and the
/// variant-numeric family ride their honest tiers, the clip leg
/// refuses.
#[test]
fn the_wave1_verdicts_are_the_honest_ones() {
    use aeolus_ir::Prop;
    use aeolus_xilem::caps::WalkVerdict;
    let walk = aeolus_xilem::caps::walk_capabilities();
    let tier_of = |prop: aeolus_ir::Prop| {
        walk.iter()
            .find(|(p, _, _)| *p == prop)
            .map(|(_, v, _)| *v)
            .expect("the probe corpus covers every longhand")
    };
    // The between-child families: no child access in a per-view
    // lowering — the precise boundary, whichever lane.
    for prop in [
        Prop::SpaceX,
        Prop::SpaceXReverse,
        Prop::SpaceY,
        Prop::SpaceYReverse,
        Prop::DivideX,
        Prop::DivideXReverse,
        Prop::DivideY,
        Prop::DivideYReverse,
        Prop::DivideColor,
        Prop::DivideStyle,
    ] {
        assert_eq!(
            tier_of(prop),
            WalkVerdict::Refused,
            "{prop:?}: the between-child boundary"
        );
    }
    // The clip leg: no framework carries clip-path.
    assert_eq!(tier_of(Prop::ClipPath), WalkVerdict::Refused);
}
