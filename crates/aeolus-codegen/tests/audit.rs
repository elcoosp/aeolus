//! The observed-capability audit's contract tests (TDD: written FIRST —
//! the RED capture is the "no `audit` module / no `reason` method" compile
//! failure, recorded in `reports/10-tdd-evidence/40-capability-audit.md`).
//!
//! The seam under test is `aeolus_codegen::audit`: the §5.1 conformance
//! score's PROJECT-SIZED twin — the per-longhand ceiling applied to the
//! longhands a real class set actually names, one tiered row per observed
//! longhand with the capability's own boundary reason. The fixture
//! capability keeps the pins backend-independent: the REAL GPUI verdicts
//! are pinned beside their generated table in
//! `aeolus-codegen-gpui/tests/caps_reason.rs` and end-to-end through the
//! CLI battery (`aeolus-cli/tests/audit.rs`).

use aeolus_codegen::audit::{AuditReport, ClassAudit};
use aeolus_codegen::conformance::{Capability, Tier};
use aeolus_ir::{Cond, Decl, Layer, Program, Prop, Val};

/// A three-tier fixture: two native longhands, one polyfill, everything
/// else refused — enough shape to pin the partition, the worst-tier
/// ranking and the score arithmetic. One refused longhand carries a
/// reason, the rest refuse unnamed (the generated tables' shape).
struct Fixture {
    named: bool,
}

impl Capability for Fixture {
    fn name(&self) -> &'static str {
        "fixture"
    }

    fn tier(&self, prop: Prop) -> Tier {
        match prop {
            Prop::Display | Prop::PaddingTop => Tier::Native,
            Prop::RingWidth => Tier::Polyfill,
            _ => Tier::Refused,
        }
    }

    fn reason(&self, prop: Prop) -> Option<&'static str> {
        if self.named && prop == Prop::ZIndex {
            Some("the fixture's own typed boundary")
        } else {
            None
        }
    }
}

/// A capability WITHOUT a `reason` override — the provided default must
/// answer `None` everywhere (existing impls never ripple).
struct Bare;

impl Capability for Bare {
    fn name(&self) -> &'static str {
        "bare"
    }

    fn tier(&self, _prop: Prop) -> Tier {
        Tier::Refused
    }
}

/// One base layer over the given (longhand, value) pairs.
fn base_layer(decls: &[(Prop, Val)]) -> Layer {
    let owned: Vec<Decl> = decls
        .iter()
        .map(|(prop, val)| Decl {
            prop: *prop,
            val: *val,
        })
        .collect();
    Layer::new(Cond::Base, owned.leak())
}

fn program(layers: Vec<Layer>) -> Program {
    Program::new(layers.leak())
}

#[test]
fn rows_follow_first_seen_cascade_order_and_dedupe() {
    // `PaddingTop` names twice (two layers); the audit keeps ONE row,
    // the first occurrence's position — the per-longhand ceiling, not
    // the per-declaration count.
    let layer_one = base_layer(&[(Prop::Display, Val::Kw(0)), (Prop::ZIndex, Val::Num(10.0))]);
    let layer_two = base_layer(&[
        (Prop::PaddingTop, Val::Rem(1.0)),
        (Prop::PaddingTop, Val::Rem(2.0)),
    ]);
    let layer_three = base_layer(&[(Prop::RingWidth, Val::Rem(2.0))]);
    let audit = ClassAudit::of(
        "flex z-10 p-4 p-8 ring-2",
        &program(vec![layer_one, layer_two, layer_three]),
        &Fixture { named: true },
    );
    let props: Vec<Prop> = audit.rows().iter().map(|row| row.prop).collect();
    assert_eq!(
        props,
        vec![
            Prop::Display,
            Prop::ZIndex,
            Prop::PaddingTop,
            Prop::RingWidth
        ],
        "first-seen cascade order, deduped"
    );
    let tiers: Vec<Tier> = audit.rows().iter().map(|row| row.tier).collect();
    assert_eq!(
        tiers,
        vec![Tier::Native, Tier::Refused, Tier::Native, Tier::Polyfill]
    );
    // The capability's own reason rides the row; native rows answer none.
    assert_eq!(audit.rows()[0].reason, None);
    assert_eq!(
        audit.rows()[1].reason,
        Some("the fixture's own typed boundary")
    );
    assert_eq!(audit.raw(), "flex z-10 p-4 p-8 ring-2");
}

#[test]
fn worst_ranks_refused_over_polyfill_over_native() {
    let native_only = base_layer(&[(Prop::Display, Val::Kw(0))]);
    let polyfill_only = base_layer(&[(Prop::RingWidth, Val::Rem(2.0))]);
    let refused_only = base_layer(&[(Prop::ZIndex, Val::Num(10.0))]);
    let audit = |layers: Vec<Layer>| {
        ClassAudit::of("the raw input", &program(layers), &Fixture { named: false })
    };

    assert_eq!(audit(vec![native_only]).worst(), Some(Tier::Native));
    assert_eq!(audit(vec![polyfill_only]).worst(), Some(Tier::Polyfill));
    assert_eq!(audit(vec![refused_only]).worst(), Some(Tier::Refused));
    // The mixed shape answers the WORST tier — the gate the CLI's
    // `--fail-on` keys on.
    let mixed = base_layer(&[
        (Prop::Display, Val::Kw(0)),
        (Prop::RingWidth, Val::Rem(2.0)),
    ]);
    assert_eq!(audit(vec![mixed]).worst(), Some(Tier::Polyfill));
    // An empty program (the empty-string compile's own shape) observes
    // nothing: no rows, no worst.
    assert_eq!(
        audit(vec![]).worst(),
        None,
        "an empty program has no verdict"
    );
}

#[test]
fn the_report_score_mirrors_the_conformance_arithmetic() {
    // Four observed rows: two native, one polyfill, one refused — the
    // §5.1 arithmetic over the OBSERVED set (75% expressible, 50%
    // native), never the backend-wide partition.
    let native = base_layer(&[
        (Prop::Display, Val::Kw(0)),
        (Prop::PaddingTop, Val::Rem(1.0)),
    ]);
    let rest = base_layer(&[
        (Prop::ZIndex, Val::Num(10.0)),
        (Prop::RingWidth, Val::Rem(2.0)),
    ]);
    let audit = ClassAudit::of(
        "flex p-4 z-10 ring-2",
        &program(vec![native, rest]),
        &Fixture { named: false },
    );
    assert_eq!(audit.count(Tier::Native), 2);
    assert_eq!(audit.count(Tier::Polyfill), 1);
    assert_eq!(audit.count(Tier::Refused), 1);

    let report = AuditReport::new("fixture", vec![audit]);
    assert_eq!(report.backend(), "fixture");
    assert_eq!(report.total_rows(), 4);
    assert!((report.score() - 75.0).abs() < 1e-4, "{}", report.score());
    assert!(
        (report.native_score() - 50.0).abs() < 1e-4,
        "{}",
        report.native_score()
    );

    // The empty observation set: nothing refused, nothing to score —
    // the vacuous-full answer (NaN is never a published number).
    let empty = AuditReport::new(
        "fixture",
        vec![ClassAudit::of(
            "",
            &program(vec![]),
            &Fixture { named: false },
        )],
    );
    assert_eq!(empty.total_rows(), 0);
    assert_eq!(empty.score(), 100.0);
    assert_eq!(empty.worst(), None);
}

#[test]
fn the_report_worst_aggregates_the_classes() {
    let native = base_layer(&[(Prop::Display, Val::Kw(0))]);
    let refused = base_layer(&[(Prop::ZIndex, Val::Num(10.0))]);
    let report = AuditReport::new(
        "fixture",
        vec![
            ClassAudit::of("flex", &program(vec![native]), &Fixture { named: false }),
            ClassAudit::of("z-10", &program(vec![refused]), &Fixture { named: false }),
        ],
    );
    assert_eq!(report.worst(), Some(Tier::Refused));
    assert_eq!(report.classes().len(), 2);
}

#[test]
fn the_reason_default_answers_none_without_an_override() {
    let refused = base_layer(&[(Prop::ZIndex, Val::Num(10.0))]);
    let audit = ClassAudit::of("z-10", &program(vec![refused]), &Bare);
    assert_eq!(audit.rows()[0].tier, Tier::Refused);
    assert_eq!(audit.rows()[0].reason, None, "no override, no reason");
}

#[test]
fn the_display_is_deterministic_and_names_the_boundaries() {
    let native = base_layer(&[
        (Prop::Display, Val::Kw(0)),
        (Prop::PaddingTop, Val::Rem(1.0)),
    ]);
    let rest = base_layer(&[
        (Prop::ZIndex, Val::Num(10.0)),
        (Prop::RingWidth, Val::Rem(2.0)),
    ]);
    let render = || {
        let audit = ClassAudit::of(
            "flex p-4 z-10 ring-2",
            &program(vec![native]),
            &Fixture { named: true },
        );
        let refused_audit = ClassAudit::of(
            "z-10 ring-2",
            &program(vec![rest]),
            &Fixture { named: true },
        );
        AuditReport::new("fixture", vec![audit, refused_audit]).to_string()
    };
    let first = render();
    let second = render();
    assert_eq!(first, second, "same capability, same bytes");
    assert!(first.contains("fixture"), "{first}");
    // The non-native rows name the longhand and the tier; the reason
    // rides when the capability supplies one.
    assert!(first.contains("z-index"), "{first}");
    assert!(first.contains("refused"), "{first}");
    assert!(first.contains("polyfill"), "{first}");
    assert!(
        first.contains("the fixture's own typed boundary"),
        "{first}"
    );
}
