//! Consumer demonstration: the `check` command's library path (the P5
//! row's "`aeolus check` in CI" face) — the same lint a CI job
//! runs over the sources, driven in-process the way a wrapper or a
//! pre-commit hook would. The extraction seam's strings from a source
//! text, each candidate string resolved into typed [`Finding`]s, and
//! the `--fail-on` gate keyed on the KIND (never on the prose) — the
//! exit codes the CI job maps.

use aeolus_cli::{FindingKind, check_string, extract_candidate_strings, load_theme};

/// A synthetic source the way a component file looks: one clean class
/// string, one unknown root, one unsupported form — the findings the
/// CI gate reads, one kind each.
const SOURCE: &str = r#"fn card() {
    let ok = "p-4 flex-col gap-2 rounded-md";
    let typo = "not-a-root p-4";
    let shape = "w-[calc(100%-2rem)]";
}
"#;

fn main() {
    // The theme: the built-in shadcn spec (the CLI's own default) — the
    // same `load_theme` seam the `--theme` flag drives.
    let theme = load_theme("shadcn").expect("the built-in spec loads");

    // The extraction seam: the candidate strings the linter resolves
    // (the SAME walk the build artifact rides — one implementation, two
    // consumers).
    let candidates = extract_candidate_strings(SOURCE);
    println!("── candidates (extract_candidate_strings) ──");
    for candidate in &candidates {
        println!("  {candidate:?}");
    }

    // The lint: one typed finding per miss — the kind is the contract,
    // the `--fail-on` modes key on it (never on a detail substring).
    println!("\n── findings (check_string) ──");
    let findings: Vec<_> = candidates
        .iter()
        .flat_map(|candidate| check_string(candidate, &theme))
        .collect();
    for finding in &findings {
        println!(
            "  {:<10} [{}] {}",
            finding.token,
            finding.kind.name(),
            finding.detail
        );
    }

    // The fail-on gate: `unknown` fires when an Unknown finding rides
    // (the CLI's default mode), `any` fires on everything, `none` never
    // does (the emit-first semantics the build/plan commands keep).
    let fires = |mode: &str| match mode {
        "unknown" => findings
            .iter()
            .any(|finding| matches!(finding.kind, FindingKind::Unknown)),
        "unsupported" => findings
            .iter()
            .any(|finding| matches!(finding.kind, FindingKind::Unsupported)),
        "any" => !findings.is_empty(),
        _ => false,
    };
    println!("\n── the fail-on gate ──");
    for mode in ["none", "unknown", "unsupported", "any"] {
        println!("  --fail-on {mode:<12} exit {}", i32::from(fires(mode)));
    }
}
