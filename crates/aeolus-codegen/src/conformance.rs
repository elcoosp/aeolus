//! The per-backend conformance score (architecture plan §5.1: "Publish
//! a per-backend conformance score (WPT / caniuse-style badge)"):
//! the [`Capability`] seam — one tiered verdict per IR longhand — and
//! the [`Conformance`] report that turns a backend's capability tables
//! into the publishable score.
//!
//! ```text
//! a backend's capability tables (supports/polyfilled predicates)
//!   └─ impl Capability for ItsCaps    — one tier per Prop
//!        └─ Conformance::of(&caps)    — the partition over Prop::ALL
//!             ├─ score()        — expressible (native + polyfill) %, the badge number
//!             ├─ native_score() — the strict native-only %
//!             └─ Display        — the audit table the committed docs page renders
//! ```
//!
//! The seam's contract is the SAME honesty contract every surface
//! here carries: the report is a FUNCTION of the backend's own
//! predicates (the GPUI impl reads `supports`/`polyfilled` — the exact
//! tables the emit gates run), so a published score can never drift
//! from what the emitter actually does. The partition is total and
//! disjoint over `Prop::ALL` (test-pinned): a capability table that
//! forgets a longhand cannot inflate the score, and the polyfill tier
//! is counted SEPARATELY — the §3.7 ring row's lesson, a composed
//! surface is not a native slot.
//!
//! What the per-prop score deliberately does NOT model: the
//! per-DECLARATION refinement (the emit gates' text-dependent verdicts
//! — the gradient direction row expresses exactly the linear-gradient
//! directions, the via stop refuses on GPUI's two-stop shape). Those
//! live at the emit gate, where the keyword annotations ride; the
//! score is the per-longhand ceiling a linter or docs page renders.
//! A backend that needs the finer verdict grows its own
//! per-declaration report beside the gate (the `lower_class`
//! diagnostics are that surface for GPUI).

use std::fmt;

use aeolus_ir::Prop;

/// One longhand's verdict in a backend's capability tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    /// The backend's native surface carries the longhand directly.
    Native,
    /// No native slot, but the runtime composes it from native
    /// surfaces (the §3.7 ring row's `RingState` precedent).
    Polyfill,
    /// The backend cannot express the longhand — the honest, reported
    /// boundary (the emit gate refuses it with a named reason).
    Refused,
}

/// The per-backend capability seam: one question per IR longhand —
/// what tier does the backend carry it at? The impl reads the
/// backend's OWN capability predicates (one source of truth — the
/// GPUI impl maps `supports`/`polyfilled`, the same tables the emit
/// gates run), so the published score is the emitter's behaviour, not
/// a parallel claim.
pub trait Capability {
    /// The backend's stable name — the report's key and the docs
    /// page's section header (`"gpui"`).
    fn name(&self) -> &'static str;

    /// The verdict for one longhand.
    fn tier(&self, prop: Prop) -> Tier;
}

/// The §5.1 conformance report: one backend's tier partition over
/// [`Prop::ALL`], the score arithmetic and the rendered audit table.
/// Deterministic by construction — the same capability reports the
/// same bytes, which is what makes the committed `docs/CONFORMANCE.md`
/// page a drift-pinned artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conformance {
    backend: &'static str,
    native: Vec<Prop>,
    polyfill: Vec<Prop>,
    refused: Vec<Prop>,
}

impl Conformance {
    /// Build the report by walking [`Prop::ALL`] through the
    /// capability's verdicts, in discriminant order (the IR's own
    /// cascade order — the rendered table reads in the same order the
    /// compiler sorts declarations).
    pub fn of(capability: &impl Capability) -> Self {
        let mut report = Self {
            backend: capability.name(),
            native: Vec::new(),
            polyfill: Vec::new(),
            refused: Vec::new(),
        };
        for prop in Prop::ALL {
            match capability.tier(*prop) {
                Tier::Native => report.native.push(*prop),
                Tier::Polyfill => report.polyfill.push(*prop),
                Tier::Refused => report.refused.push(*prop),
            }
        }
        report
    }

    /// The backend's name.
    pub fn backend(&self) -> &'static str {
        self.backend
    }

    /// The natively-carried longhands, in `Prop::ALL` order.
    pub fn native(&self) -> &[Prop] {
        &self.native
    }

    /// The polyfilled longhands, in `Prop::ALL` order.
    pub fn polyfill(&self) -> &[Prop] {
        &self.polyfill
    }

    /// The refused longhands — the documented boundaries — in
    /// `Prop::ALL` order.
    pub fn refused(&self) -> &[Prop] {
        &self.refused
    }

    /// The longhand count the score runs over (`Prop::ALL.len()`).
    pub fn total(&self) -> usize {
        Prop::ALL.len()
    }

    /// The §5.1 badge number: the EXPRESSIBLE share (native +
    /// polyfilled) over the total, in percent. The polyfill tier
    /// counts — a composed surface styles the element — but the
    /// strict native-only share rides beside it
    /// ([`Conformance::native_score`]), so the two numbers never
    /// blur the tier line.
    pub fn score(&self) -> f32 {
        let total = self.total() as f32;
        (self.native.len() + self.polyfill.len()) as f32 / total * 100.0
    }

    /// The strict native-only share, in percent (the polyfill tier
    /// excluded).
    pub fn native_score(&self) -> f32 {
        self.native.len() as f32 / self.total() as f32 * 100.0
    }
}

impl fmt::Display for Conformance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Two decimals: deterministic across platforms (the artifact
        // contract — same capability, same bytes), precise enough for
        // a 115-longhand surface where one longhand moves the score by
        // less than a point.
        let badge = format!("{:.2}", self.score());
        let native = format!("{:.2}", self.native_score());
        let polyfill = format!(
            "{:.2}",
            self.polyfill.len() as f32 / self.total() as f32 * 100.0
        );
        writeln!(f, "# Aeolus conformance — {}\n", self.backend)?;
        writeln!(
            f,
            "**score: {badge}% expressible** ({native}% native + {polyfill}% polyfilled) over {} longhands\n",
            self.total()
        )?;
        writeln!(
            f,
            "The per-longhand tiers, in the IR's cascade order. The \
             per-declaration refinements (the emit gates' text-dependent \
             verdicts) live at the gates themselves — this is the \
             per-longhand ceiling.\n"
        )?;
        writeln!(f, "| longhand | tier |")?;
        writeln!(f, "| --- | --- |")?;
        let mut rows = Vec::new();
        for prop in &self.native {
            rows.push((*prop, "native"));
        }
        for prop in &self.polyfill {
            rows.push((*prop, "polyfill"));
        }
        for prop in &self.refused {
            rows.push((*prop, "refused"));
        }
        for (prop, tier) in rows {
            writeln!(f, "| `{}` | {tier} |", prop.name())?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// The seam's own contract tests (the RED spec — the integration suite
// in tests/conformance.rs pins the same shape against a local fixture;
// these pin it against the REAL partition invariants)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// The all-native capability: the partition's edge case.
    struct AllNative;
    impl Capability for AllNative {
        fn name(&self) -> &'static str {
            "all-native"
        }
        fn tier(&self, _prop: Prop) -> Tier {
            Tier::Native
        }
    }

    /// The all-refused capability: the other edge.
    struct AllRefused;
    impl Capability for AllRefused {
        fn name(&self) -> &'static str {
            "all-refused"
        }
        fn tier(&self, _prop: Prop) -> Tier {
            Tier::Refused
        }
    }

    #[test]
    fn the_edges_score_zero_and_full() {
        let full = Conformance::of(&AllNative);
        assert_eq!(full.score(), 100.0);
        assert_eq!(full.native_score(), 100.0);
        assert!(full.refused().is_empty());

        let zero = Conformance::of(&AllRefused);
        assert_eq!(zero.score(), 0.0);
        assert_eq!(zero.native_score(), 0.0);
        assert!(zero.native().is_empty());
    }

    #[test]
    fn the_partition_is_total_for_any_capability() {
        for (report, name) in [
            (Conformance::of(&AllNative), "all-native"),
            (Conformance::of(&AllRefused), "all-refused"),
        ] {
            assert_eq!(report.backend(), name);
            assert_eq!(
                report.native().len() + report.polyfill().len() + report.refused().len(),
                Prop::ALL.len(),
                "{name}: the tiers partition Prop::ALL exactly"
            );
        }
    }
}
