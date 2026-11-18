//! Consumer demonstration: the workspace-wide pull — the wire's
//! `workspace/diagnostic` (LSP 3.17) answered headless. Every open
//! document's findings ride beside its result identity — the content
//! fingerprint the client caches against — and a matching previous id
//! answers the unchanged verdict: the client's cache is the report,
//! nothing recomputes (the retheme fast-path's equality shape, at the
//! pull channel).
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_workspace_diagnostics
//! ```

use aeolus_lsp::{AeolusEngine, EngineConfig};

/// A synthetic .rs document: one lowered string beside a refused
/// candidate — the findings the full report carries.
const DOC: &str = "fn card() {\n    let a = \"p-4 flex-col not-a-root\";\n}\n";

/// The workspace's second document (a rebrand table lives elsewhere in
/// a real session).
const DOC_BRAND: &str = "fn brand() {\n    let a = \"bg-primary hover:bg-primary/90\";\n}\n";

fn main() {
    let mut lsp = AeolusEngine::new(EngineConfig::default());
    lsp.open("file:///card.rs", DOC, 1);
    lsp.open("file:///brand.rs", DOC_BRAND, 1);

    // 1. The first pull (no client cache): every open document answers
    //    the full report — URI order across the map, the findings the
    //    refused candidate carries.
    println!("── the first pull (empty client cache) ──");
    for report in lsp.workspace_diagnostics(&[]) {
        println!(
            "    {} (v{}, result id {})",
            report.uri, report.version, report.result_id
        );
        if let aeolus_lsp::EngineWorkspaceOutcome::Full { diagnostics } = &report.outcome {
            for diagnostic in diagnostics {
                println!("        finding: {diagnostic}");
            }
        }
    }

    // 2. The quiet workspace: the SAME identities ride back — the
    //    unchanged verdict the client's cache answers from.
    println!("── the quiet pull (the client's ids cached) ──");
    let previous: Vec<(String, String)> = lsp
        .workspace_diagnostics(&[])
        .into_iter()
        .map(|report| (report.uri, report.result_id))
        .collect();
    let borrowed: Vec<(&str, &str)> = previous
        .iter()
        .map(|(uri, id)| (uri.as_str(), id.as_str()))
        .collect();
    for report in lsp.workspace_diagnostics(&borrowed) {
        println!(
            "    {}: {} — nothing recomputes",
            report.uri,
            match report.outcome {
                aeolus_lsp::EngineWorkspaceOutcome::Full { .. } => "full",
                aeolus_lsp::EngineWorkspaceOutcome::Unchanged => "unchanged",
            }
        );
    }

    // 3. A didChange moves one document: its cached id goes stale, its
    //    report recomputes; the other document keeps riding unchanged.
    println!("── the pull after one didChange ──");
    lsp.change(
        "file:///card.rs",
        "fn card() {\n    let a = \"p-4\";\n}\n",
        2,
    );
    for report in lsp.workspace_diagnostics(&borrowed) {
        println!(
            "    {}: {} (result id {})",
            report.uri,
            match report.outcome {
                aeolus_lsp::EngineWorkspaceOutcome::Full { diagnostics } => {
                    format!("full, {} finding(s)", diagnostics.len())
                }
                aeolus_lsp::EngineWorkspaceOutcome::Unchanged => "unchanged".to_string(),
            },
            report.result_id
        );
    }
}
