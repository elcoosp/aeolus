//! Consumer demonstration: the audit seam (the §5.1 capability-audit
//! face at the LIBRARY level — the same per-class report the CLI's
//! `audit` command rides, consumed the way a backend author or a
//! conformance dashboard does). A [`Capability`] is the target's own
//! tier table (one verdict per longhand, the emit gates' own truth —
//! never a parallel claim); [`ClassAudit::of`] walks one compiled
//! program's distinct longhands against it, deduped in first-seen
//! cascade order; [`AuditReport`] aggregates the classes and renders
//! the deterministic table (the same-input-same-bytes contract the
//! committed conformance pages ride).
//!
//! The demo program is hand-built from the IR's own constructors
//! (`Layer::new`), exactly the shape the static-data arm freezes — a
//! backend author audits THEIR capability table against any program
//! they can name, no compile step needed.
//!
//! ```sh
//! cargo run -p aeolus-codegen --example audit
//! ```

use aeolus_codegen::audit::{AuditReport, ClassAudit};
use aeolus_codegen::conformance::{Capability, Tier};
use aeolus_ir::{Cond, Decl, Layer, Program, Prop, Val};

/// The demo programs — one per tier story, each hand-built over the
/// IR's own `Layer::new` constructors (masks derived at construction):
///
/// * the native shape: padding + width, lengths the demo backend
///   expresses directly;
/// * the polyfill shape: the ring family, the `--tw-ring-*` custom
///   properties the demo backend composes;
/// * the refusal shape: `z-index`, outside the demo backend's express
///   set (the typed boundary, never a silent drop).
static NATIVE_PROGRAM: Program = Program::new(&[Layer::new(
    Cond::Base,
    &[
        Decl {
            prop: Prop::PaddingTop,
            val: Val::Rem(1.0),
        },
        Decl {
            prop: Prop::Width,
            val: Val::Rem(16.0),
        },
    ],
)]);

static POLYFILL_PROGRAM: Program = Program::new(&[Layer::new(
    Cond::Base,
    &[Decl {
        prop: Prop::RingWidth,
        val: Val::Px(1.0),
    }],
)]);

static REFUSED_PROGRAM: Program = Program::new(&[Layer::new(
    Cond::Base,
    &[Decl {
        prop: Prop::ZIndex,
        val: Val::Num(10.0),
    }],
)]);

/// The demo backend's capability table: lengths and padding native,
/// the ring family polyfilled through the `--tw-ring-*` custom
/// properties, everything else refused — the same shape a real
/// backend's verdicts take (one tier per longhand, the boundary
/// reasons typed).
struct DemoTarget;

impl Capability for DemoTarget {
    fn name(&self) -> &'static str {
        "demo"
    }

    fn tier(&self, prop: Prop) -> Tier {
        match prop {
            Prop::RingWidth | Prop::RingColor => Tier::Polyfill,
            Prop::PaddingTop | Prop::Width => Tier::Native,
            _ => Tier::Refused,
        }
    }

    fn reason(&self, prop: Prop) -> Option<&'static str> {
        match prop {
            Prop::RingWidth | Prop::RingColor => {
                Some("the ring family rides the --tw-ring-* custom properties")
            }
            Prop::PaddingTop | Prop::Width => None,
            _ => Some("the demo capability carries lengths and padding only"),
        }
    }
}

fn main() {
    // One class audit per tier story: the distinct longhands each
    // program sets, in first-seen cascade order.
    let native = ClassAudit::of("p-4 w-16", &NATIVE_PROGRAM, &DemoTarget);
    let polyfill = ClassAudit::of("ring-1", &POLYFILL_PROGRAM, &DemoTarget);
    let refused = ClassAudit::of("z-10", &REFUSED_PROGRAM, &DemoTarget);
    println!("== the class audits ==");
    for audit in [&native, &polyfill, &refused] {
        println!("raw: {}", audit.raw());
        for row in audit.rows() {
            let tier = match row.tier {
                Tier::Native => "native",
                Tier::Polyfill => "polyfill",
                Tier::Refused => "refused",
            };
            match row.reason {
                Some(reason) => println!("  {}: {tier} — {reason}", row.prop.name()),
                None => println!("  {}: {tier}", row.prop.name()),
            }
        }
        println!(
            "worst: {:?}, native rows: {}\n",
            audit.worst(),
            audit.count(Tier::Native)
        );
    }

    // The project report: one audit per class string (the caller's
    // scan order), the aggregate score over the observed rows and the
    // deterministic rendered table — the same bytes the committed
    // conformance pages ride.
    let report = AuditReport::new(DemoTarget.name(), vec![native, polyfill, refused]);
    println!("== the report ==");
    println!(
        "backend {}, {} classes, {} observed rows, score {:.2}%, native {:.2}%",
        report.backend(),
        report.classes().len(),
        report.total_rows(),
        report.score(),
        report.native_score(),
    );
    println!("worst overall: {:?}", report.worst());
    println!("\n{report}");
}
