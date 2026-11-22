//! The walk's own capability verdicts (plan §5.1's conformance moat's
//! per-backend half): the registry-driven probe corpus
//! ([`aeolus_rt::probe_programs`]) replays through THIS backend's real
//! `lower_program` lane, and each longhand's verdict is the BEST tier
//! across its probes — native when a probe lands without a report
//! entry naming it, polyfill for the emulated slots this crate
//! composes, refused with the report's own reason text otherwise. The
//! committed table ([`aeolus_codegen::caps_tables::ICED_CAPABILITIES`]) is THIS
//! fn's freeze; the drift pin beside the crate (`tests/caps_table.rs`)
//! re-runs the replay and asserts the equality — the published score
//! is the emitter's behaviour, never a parallel claim.

use std::collections::BTreeMap;

use aeolus_ir::Prop;

use aeolus_rt::Env;

/// The emulated slots (the polyfill tier): opacity bakes into the painted surface's alpha (the slot ledger's own Emulated row).
pub const POLYFILLED: &[Prop] = &[Prop::Opacity];

/// One longhand's walk verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WalkVerdict {
    /// The longhand's probes land in the native style slots.
    Native,
    /// The longhand composes through the crate's own surfaces (the
    /// emulated ledger's reading).
    Polyfill,
    /// The walk's report drops the longhand — the honest boundary.
    Refused,
}

/// The per-longhand verdict rows: `(prop, verdict, reason)` — the
/// reason carries the report's own text for the refused rows (the
/// committed table freezes it verbatim), empty otherwise.
pub fn walk_capabilities() -> Vec<(Prop, WalkVerdict, String)> {
    let probes = aeolus_rt::probe_programs();
    // The best tier across the corpus: any landed probe answers the
    // expressible tier (a per-decl refusal — a row-text form the
    // surface lacks — does not demote the longhand another probe
    // lands); an all-dropped longhand refuses with its report reason.
    let mut landed: BTreeMap<Prop, bool> = BTreeMap::new();
    let mut refusals: BTreeMap<Prop, String> = BTreeMap::new();
    for probe in &probes {
        let (_style, report) = crate::program::lower_program(&probe.program, &Env::default());
        for &prop in probe.targets {
            let dropped = report.dropped.iter().find(|(what, _)| {
                what == prop.name() || what.starts_with(&format!("{} ", prop.name()))
            });
            match dropped {
                None => {
                    landed.insert(prop, true);
                }
                Some((_, reason)) => {
                    let slot = refusals.entry(prop).or_insert_with(|| reason.clone());
                    if slot.is_empty() {
                        slot.clone_from(reason);
                    }
                }
            }
        }
    }
    Prop::ALL
        .iter()
        .map(|&prop| {
            let verdict = if landed.get(&prop).copied().unwrap_or(false) {
                if POLYFILLED.contains(&prop) {
                    WalkVerdict::Polyfill
                } else {
                    WalkVerdict::Native
                }
            } else {
                WalkVerdict::Refused
            };
            let reason = refusals.get(&prop).cloned().unwrap_or_default();
            (prop, verdict, reason)
        })
        .collect()
}

/// The plain verdict flags the committed table renders: `(prop, landed,
/// polyfill, reason)` — the uniform seam the xtask generator freezes
/// (native = landed ∧ ¬polyfill, polyfill = landed ∧ polyfill, refused
/// = ¬landed) and the drift pins re-derive from a fresh
/// [`walk_capabilities()`] replay.
pub fn walk_capability_flags() -> Vec<(Prop, bool, bool, String)> {
    walk_capabilities()
        .into_iter()
        .map(|(prop, verdict, reason)| {
            let (landed, polyfill) = match verdict {
                WalkVerdict::Native => (true, false),
                WalkVerdict::Polyfill => (true, true),
                WalkVerdict::Refused => (false, false),
            };
            (prop, landed, polyfill, reason)
        })
        .collect()
}
