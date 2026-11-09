//! Consumer demonstration: the utility registry as data — the read side
//! of `aeolus-data`. One auditable row per utility pattern; the longest
//! prefix disambiguates (`rounded-tl-lg` → `rounded-tl`); the row names
//! the longhand targets, the `tw_merge` conflict group and the v4
//! source ref. Misses are TYPED (`registry::lookup_detailed`), so the
//! linter's "did you mean" surface can match on the failure mode.
//!
//! ```sh
//! cargo run -p aeolus-data --example registry_lookup
//! ```
//!
//! What the registry does NOT do: emit CSS or lower values to the IR —
//! that is P2's compile stage. The row is the WHAT (targets + shape);
//! the backend decides the HOW.

use aeolus_data::registry::{LookupError, generate_docs, lookup, lookup_detailed};

fn main() {
    // The hits: every row the longest-prefix rule selects, in registry
    // order (`bg` carries two arms — colour and gradient image).
    for body in ["p-4", "rounded-tl-lg", "min-w-0", "text-lg", "bg-primary"] {
        let rows: Vec<_> = lookup(body).collect();
        println!("{body:?} → {} row(s):", rows.len());
        for row in rows {
            println!(
                "  root {:?} kind {:?} → longhands [{}] group {:?} v4 {}",
                row.root,
                row.kind.label(),
                row.targets
                    .iter()
                    .map(|p| p.name())
                    .collect::<Vec<_>>()
                    .join(", "),
                row.group,
                row.v4,
            );
        }
    }

    // The typed misses: `lookup_detailed` explains WHY, carrying the
    // deduplicated value shapes the root tried (`AnyOf` arms flattened).
    println!("\nthe typed misses:");
    for body in ["not-a-utility", "p-red"] {
        match lookup_detailed(body) {
            Ok(rows) => println!("{body:?} unexpectedly hit {} row(s)", rows.len()),
            Err(LookupError::UnknownRoot { body: b }) => {
                println!("  {b:?}: no registered root prefixes it");
            }
            Err(LookupError::ShapeRejected {
                body: b,
                root,
                tried,
            }) => {
                println!(
                    "  {b:?}: root {root:?} matched, but none of its value shapes accepted the suffix (tried: {})",
                    tried.join(", ")
                );
            }
        }
    }

    // The one-source-of-truth claim: `generate_docs()` renders the
    // audit page from the SAME rows — here: its header + the `p` rows.
    let docs = generate_docs();
    let head = docs.lines().take(5).collect::<Vec<_>>().join("\n");
    println!("\ngenerate_docs() head:\n{head}");
}
