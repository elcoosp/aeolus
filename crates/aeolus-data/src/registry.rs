//! The registry read side: root listing, longest-prefix lookup and the
//! docs generator that proves the one-source-of-truth claim.

use crate::UtilityRow;
use crate::rows::ROWS;
use std::fmt::Write as _;

/// Every distinct root, in registry order (deduplicated).
pub fn roots() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for row in ROWS {
        if !out.contains(&row.root) {
            out.push(row.root);
        }
    }
    out
}

/// The rows for a utility BODY (`"p-4"`, `"rounded-tl-lg"`, `"hidden"`),
/// under the longest-root-prefix rule: `rounded-tl-lg` disambiguates to
/// the `rounded-tl` rows, not `rounded`. Within the winning root, every
/// row whose [`crate::ValueKind`] accepts the suffix shape matches (a
/// root can carry several arms — `bg` colour vs gradient image).
///
/// The value half (does `4` fit the spacing scale? which keyword table
/// holds `full`?) is the shape contract in `ValueKind::accepts`; the
/// lowering to `aeolus_ir::Val` is P2's compile stage.
pub fn lookup(body: &str) -> impl Iterator<Item = &'static UtilityRow> {
    // Longest root that prefixes the body wins; ties (several rows of the
    // same root) all return, in registry order.
    let best = ROWS
        .iter()
        .filter(|row| body_matches_root(body, row.root))
        .map(|row| row.root.len())
        .max();
    let matched: Vec<&'static UtilityRow> = match best {
        Some(len) => ROWS
            .iter()
            .filter(|row| {
                row.root.len() == len
                    && body_matches_root(body, row.root)
                    && kind_accepts(row.kind, body, row.root)
            })
            .collect(),
        None => Vec::new(),
    };
    matched.into_iter()
}

/// `lookup_root("bg")` — every arm registered under one exact root.
pub fn lookup_root(root: &str) -> impl Iterator<Item = &'static UtilityRow> {
    ROWS.iter()
        .filter(move |row| row.root == root)
        .collect::<Vec<&'static UtilityRow>>()
        .into_iter()
}

/// Does `body` address `root`? Either exact (`hidden`) or
/// `root + '-' + suffix` (`p-4` under `p`).
fn body_matches_root(body: &str, root: &str) -> bool {
    body == root || (body.starts_with(root) && body.as_bytes().get(root.len()) == Some(&b'-'))
}

/// The suffix after the root (`"4"` for `p-4`, `None` for `hidden`).
fn suffix_of<'a>(body: &'a str, root: &str) -> Option<&'a str> {
    if body == root {
        None
    } else {
        Some(&body[root.len() + 1..])
    }
}

fn kind_accepts(kind: crate::ValueKind, body: &str, root: &str) -> bool {
    kind.accepts(suffix_of(body, root))
}

/// Render the registry as the markdown audit page — the proof that the
/// committed rows are the single source the generated docs flow from.
/// The P2 xtask grows this into the full docs/linter/sort-order
/// generator story (crate docs).
pub fn generate_docs() -> String {
    let mut out = String::with_capacity(ROWS.len() * 96);
    out.push_str("# Aeolus V1 utility registry\n\n");
    out.push_str("Generated from `aeolus-data`'s committed rows — do not edit.\n\n");
    out.push_str("| root | value kind | longhands | conflict group | neg | v4 ref |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- |\n");
    for row in ROWS {
        let targets: Vec<&str> = row.targets.iter().map(|p| p.name()).collect();
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | `{}` | {} | {} |",
            row.root,
            row.kind.label(),
            targets.join(", "),
            row.group,
            if row.negative { "yes" } else { "no" },
            row.v4
        );
    }
    out
}
