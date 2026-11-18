//! The observed-capability audit (the §5.1 conformance score's
//! project-sized twin): the [`Capability`] seam's per-longhand ceiling
//! applied to the longhands a real class set actually names.
//!
//! ```text
//! a class string's compiled Program (the compile walk's own artifact)
//!   └─ ClassAudit::of(raw, &program, &caps)   — one tiered row per observed longhand
//!        └─ AuditReport::new(backend, classes)
//!             ├─ score()        — the expressible share over the OBSERVED rows
//!             ├─ native_score() — the strict native-only share
//!             ├─ worst()        — the gate tier a CI `--fail-on` keys on
//!             └─ Display        — the deterministic audit table
//! ```
//!
//! The seam's honesty contract is the conformance report's own, pointed
//! at a project instead of a backend: the rows are a FUNCTION of the
//! capability's own predicates (`tier`/`reason` — the GPUI impl reads
//! the generated table the emit gates run), so an audit can never drift
//! from what the emitter actually does. Where [`Conformance::of`] walks
//! [`Prop::ALL`] (the backend-wide ceiling the docs page publishes),
//! this module walks a program's OWN longhands: a project that never
//! names a refused longhand scores full, and the report names the
//! boundary per class — the "which of my classes degrade and why" answer.
//!
//! What stays deliberately out: the per-declaration refinements (the
//! emit gates' text-dependent verdicts — the gradient direction row,
//! the composite-value refusals). Those live at the gates, exactly as
//! the conformance module documents; the audit is the per-longhand
//! ceiling over an observed set, the shape a linter renders.

use std::fmt;

use aeolus_ir::{Program, Prop, PropMask};

use crate::conformance::{Capability, Tier};

/// One observed longhand's verdict in a class audit: the capability's
/// own tier for the longhand, with the typed boundary reason when the
/// capability names one (the generated tables' refused rows — an
/// expressible longhand answers `None`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PropAudit {
    /// The observed longhand.
    pub prop: Prop,
    /// The capability's own tier (the emit gates' verdict, never a
    /// parallel claim).
    pub tier: Tier,
    /// The capability's own boundary reason — `Some` exactly where the
    /// capability's tables name one.
    pub reason: Option<&'static str>,
}

/// One class string's audit: the distinct longhands its compiled
/// program sets, in first-seen cascade order (the compile stage emits
/// the layers specificity-sorted, so the order is deterministic), one
/// [`PropAudit`] row each — the per-longhand ceiling, NOT the
/// per-declaration count (a longhand set by two layers keeps one row).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassAudit {
    raw: String,
    rows: Vec<PropAudit>,
}

impl ClassAudit {
    /// Audit one class string's compiled program against one
    /// capability: every longhand the program's layers set, deduped
    /// (the mask's bitset walk), tiered through the capability's own
    /// predicates. An empty program (the empty-string compile's shape)
    /// observes nothing and carries no rows.
    pub fn of(raw: &str, program: &Program, capability: &impl Capability) -> Self {
        let mut rows: Vec<PropAudit> = Vec::new();
        let mut seen = PropMask::EMPTY;
        for layer in program.layers {
            for decl in layer.decls {
                if seen.contains(decl.prop) {
                    continue;
                }
                seen = seen.insert(decl.prop);
                rows.push(PropAudit {
                    prop: decl.prop,
                    tier: capability.tier(decl.prop),
                    reason: capability.reason(decl.prop),
                });
            }
        }
        Self {
            raw: raw.to_string(),
            rows,
        }
    }

    /// The audited class string (the raw input, byte-true).
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// The observed rows, first-seen cascade order.
    pub fn rows(&self) -> &[PropAudit] {
        &self.rows
    }

    /// The class's worst tier — the gate reading (`refused` outranks
    /// `polyfill` outranks `native`); `None` when the class set no
    /// longhand at all (nothing to gate on).
    pub fn worst(&self) -> Option<Tier> {
        let mut worst: Option<Tier> = None;
        for row in &self.rows {
            if worst.is_none_or(|seen| rank(row.tier) > rank(seen)) {
                worst = Some(row.tier);
            }
        }
        worst
    }

    /// How many observed rows ride one tier.
    pub fn count(&self, tier: Tier) -> usize {
        self.rows.iter().filter(|row| row.tier == tier).count()
    }
}

/// The tier's gate ranking: the worse a tier is, the higher it ranks.
fn rank(tier: Tier) -> u8 {
    match tier {
        Tier::Native => 0,
        Tier::Polyfill => 1,
        Tier::Refused => 2,
    }
}

/// A project audit: one backend's name (the capability's own) over the
/// classes' audits — the aggregate counts, the observed-set score and
/// the deterministic rendered table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditReport {
    backend: &'static str,
    classes: Vec<ClassAudit>,
}

impl AuditReport {
    /// Assemble the report: the backend's stable name (the
    /// [`Capability::name`] answer) over one [`ClassAudit`] per audited
    /// class string, in the caller's order (the scan's source order —
    /// deterministic given a deterministic scan).
    pub fn new(backend: &'static str, classes: Vec<ClassAudit>) -> Self {
        Self { backend, classes }
    }

    /// The backend's name.
    pub fn backend(&self) -> &'static str {
        self.backend
    }

    /// The per-class audits, in the caller's order.
    pub fn classes(&self) -> &[ClassAudit] {
        &self.classes
    }

    /// The observed rows the score runs over (every class's rows).
    pub fn total_rows(&self) -> usize {
        self.classes.iter().map(|class| class.rows.len()).sum()
    }

    /// How many observed rows ride one tier, across every class.
    pub fn count(&self, tier: Tier) -> usize {
        self.classes.iter().map(|class| class.count(tier)).sum()
    }

    /// The report's worst tier across every class — the aggregate gate
    /// reading; `None` when nothing was observed.
    pub fn worst(&self) -> Option<Tier> {
        let mut worst: Option<Tier> = None;
        for class in &self.classes {
            if let Some(tier) = class.worst() {
                if worst.is_none_or(|seen| rank(tier) > rank(seen)) {
                    worst = Some(tier);
                }
            }
        }
        worst
    }

    /// The expressible share over the OBSERVED rows (native +
    /// polyfilled), in percent — the conformance arithmetic pointed at
    /// the project's own longhand set. An empty observation set scores
    /// the vacuous full 100% (nothing observed, nothing refused; the
    /// NaN shape is never a published number).
    pub fn score(&self) -> f32 {
        let total = self.total_rows() as f32;
        if total == 0.0 {
            return 100.0;
        }
        (self.count(Tier::Native) + self.count(Tier::Polyfill)) as f32 / total * 100.0
    }

    /// The strict native-only share over the observed rows, in percent
    /// (the polyfill tier excluded; the same empty-set answer as
    /// [`AuditReport::score`]).
    pub fn native_score(&self) -> f32 {
        let total = self.total_rows() as f32;
        if total == 0.0 {
            return 100.0;
        }
        self.count(Tier::Native) as f32 / total * 100.0
    }
}

/// The tier's word (the table's and the JSON's spelling).
fn tier_word(tier: Tier) -> &'static str {
    match tier {
        Tier::Native => "native",
        Tier::Polyfill => "polyfill",
        Tier::Refused => "refused",
    }
}

impl fmt::Display for AuditReport {
    /// The deterministic audit table (the same-input-same-bytes
    /// contract the committed artifact pages ride): the score line, one
    /// row per class, and the non-native longhands with their reasons —
    /// the "why" the audit exists for.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = self.total_rows();
        let polyfill = self.count(Tier::Polyfill);
        let score = format!("{:.2}", self.score());
        let native_share = format!("{:.2}", self.native_score());
        let polyfill_share = format!(
            "{:.2}",
            if total == 0 {
                100.0
            } else {
                polyfill as f32 / total as f32 * 100.0
            }
        );
        writeln!(f, "# Aeolus capability audit — {}\n", self.backend)?;
        writeln!(
            f,
            "**score: {score}% expressible** ({native_share}% native + {polyfill_share}% polyfilled) over {total} observed longhands\n"
        )?;
        writeln!(f, "| class | native | polyfill | refused | worst |")?;
        writeln!(f, "| --- | --- | --- | --- | --- |")?;
        for class in &self.classes {
            let worst = match class.worst() {
                Some(tier) => tier_word(tier),
                None => "—",
            };
            writeln!(
                f,
                "| `{}` | {} | {} | {} | {worst} |",
                class.raw(),
                class.count(Tier::Native),
                class.count(Tier::Polyfill),
                class.count(Tier::Refused),
            )?;
        }
        writeln!(
            f,
            "\nThe non-native rows — the longhand, the tier, the capability's own boundary reason:"
        )?;
        let mut named_any = false;
        for class in &self.classes {
            for row in &class.rows {
                if row.tier == Tier::Native {
                    continue;
                }
                named_any = true;
                let word = tier_word(row.tier);
                match row.reason {
                    Some(reason) => writeln!(
                        f,
                        "- `\"{}\"` — `{}` {word} — {reason}",
                        class.raw(),
                        row.prop.name(),
                    )?,
                    None => writeln!(f, "- `\"{}\"` — `{}` {word}", class.raw(), row.prop.name())?,
                }
            }
        }
        if !named_any {
            writeln!(f, "- none — every observed longhand is native")?;
        }
        Ok(())
    }
}
