//! The GPUI capability's `reason` half (TDD: written FIRST — the RED
//! capture is the compile failure before `Capability::reason` exists,
//! recorded in `reports/10-tdd-evidence/40-capability-audit.md`).
//!
//! The contract: [`Caps::reason`] reads the GENERATED table
//! (`caps_generated.rs`) — the same one-source-of-truth rows the tier
//! impl reads — so an audit report's boundary reasons can never drift
//! from the emit gates' own verdicts (the conformance seam's honesty
//! contract, applied to the reason column). A refused longhand carries
//! its non-empty typed boundary; an expressible longhand answers none.

use aeolus_codegen::conformance::{Capability, Tier};
use aeolus_codegen_gpui::{Caps, PropCapability};
use aeolus_ir::Prop;

#[test]
fn every_refused_row_names_its_typed_boundary() {
    for row in aeolus_codegen_gpui::PROP_CAPABILITIES {
        let reason = Caps.reason(row.prop);
        if row.tier == Tier::Refused {
            let reason = reason.unwrap_or_else(|| {
                panic!(
                    "{}: a refused longhand must name its boundary (the audit renders it)",
                    row.prop.name()
                )
            });
            assert!(
                !reason.is_empty(),
                "{}: the reason is non-empty",
                row.prop.name()
            );
        } else {
            assert_eq!(
                reason,
                None,
                "{}: an expressible longhand carries no reason",
                row.prop.name()
            );
        }
    }
}

#[test]
fn the_reason_table_agrees_with_the_tier_table() {
    // Parity pin: the impl's verdicts and the committed rows answer the
    // same question identically (the drift gate the generator's own
    // test rides, from the reason side).
    for row in aeolus_codegen_gpui::PROP_CAPABILITIES {
        assert_eq!(Caps.tier(row.prop), row.tier, "{}", row.prop.name());
    }
    assert_eq!(Caps.name(), "gpui");
}

#[test]
fn the_public_row_shape_stays_stable() {
    // The audit's input: one row per IR longhand, in cascade order.
    let rows: &[PropCapability] = aeolus_codegen_gpui::PROP_CAPABILITIES;
    assert_eq!(rows.len(), Prop::ALL.len());
    for (row, prop) in rows.iter().zip(Prop::ALL) {
        assert_eq!(row.prop, *prop, "the table rides Prop::ALL's order");
    }
}
