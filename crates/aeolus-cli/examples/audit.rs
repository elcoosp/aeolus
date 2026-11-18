//! Consumer demonstration: the `audit` command's library path (the §5.1
//! capability-audit face) — the same per-class report the command runs
//! over project sources, driven in-process the way a build script or a
//! dashboard would. One class per tier story: a native shape, the ring
//! family's polyfill half, the z-index and blur refusals (the generated
//! table's own typed boundaries), and the report's score arithmetic
//! over the observed set.

use aeolus_cli::audit_string;
use aeolus_theme::{ThemeSource, TokenTable};

/// The audit's driver table: the built-in shadcn bake (the CLI's own
/// default theme spec) — the same `--theme shadcn` source the command
/// rides.
fn table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

/// One class per tier story (the command battery's own shape).
const CLASSES: [&str; 4] = [
    "p-4 flex items-center",
    "ring-2 ring-offset-2",
    "z-10",
    "blur-md",
];

fn main() {
    let tokens = table();
    println!("── per-class audits (audit_string) ──");
    let mut audits = Vec::new();
    for raw in CLASSES {
        let audit = audit_string(raw, &tokens).expect("non-empty input compiles");
        let worst = match audit.worst() {
            Some(tier) => match tier {
                aeolus_codegen::conformance::Tier::Native => "native",
                aeolus_codegen::conformance::Tier::Polyfill => "polyfill",
                aeolus_codegen::conformance::Tier::Refused => "refused",
            },
            None => "—",
        };
        println!(
            "  \"{raw}\" — {} longhand{}, worst {worst}",
            audit.rows().len(),
            if audit.rows().len() == 1 { "" } else { "s" }
        );
        for row in audit.rows() {
            if row.tier == aeolus_codegen::conformance::Tier::Native {
                continue;
            }
            let tier = match row.tier {
                aeolus_codegen::conformance::Tier::Native => "native",
                aeolus_codegen::conformance::Tier::Polyfill => "polyfill",
                aeolus_codegen::conformance::Tier::Refused => "refused",
            };
            match row.reason {
                Some(reason) => println!("    {}: {tier} — {reason}", row.prop.name()),
                None => println!("    {}: {tier}", row.prop.name()),
            }
        }
        audits.push(audit);
    }

    // The aggregate: the report's score arithmetic over the observed
    // rows (the §5.1 conformance arithmetic pointed at the project's
    // own longhand set) and the deterministic rendered table.
    let report = aeolus_codegen::audit::AuditReport::new("gpui", audits);
    println!(
        "\nscore: {:.2}% expressible ({:.2}% native) over {} observed longhands",
        report.score(),
        report.native_score(),
        report.total_rows()
    );
    println!("\n── the rendered table (Display) ──");
    print!("{report}");
}
