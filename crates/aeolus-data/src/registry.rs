//! The registry read side: root listing, longest-prefix lookup and the
//! docs generator that proves the one-source-of-truth claim.
//!
//! The read path consults the GENERATED roots index first
//! (`crate::roots_index` — plan §3.2's generator story: the phf table
//! `cargo xtask gen-tables` derives from these rows): the longest
//! dash-aligned prefix probe answers WHICH root a body addresses, the
//! per-row shape contract (`ValueKind::accepts`) stays here. The
//! pre-index linear scan is kept VERBATIM below as
//! [`lookup_linear`] — the documented differential oracle the tests
//! and the lookup bench battery the indexed path against; the two
//! cannot drift apart silently. The same discipline carries the
//! SORT-ORDER seam: [`sort_index`] is the registry's canonical order
//! position for a body (the FIRST matching row's index — plan §5.5's
//! class-sorting source), with the linear scan as its oracle too.

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
/// The winner comes from the GENERATED roots index
/// ([`crate::roots_index::longest_prefix`] — the phf probe), the
/// per-row shape filter from here; [`lookup_linear`] is the same
/// answer from the pre-index scan, the differential oracle.
///
/// The value half (does `4` fit the spacing scale? which keyword table
/// holds `full`?) is the shape contract in `ValueKind::accepts`; the
/// lowering to `aeolus_ir::Val` is P2's compile stage.
pub fn lookup(body: &str) -> impl Iterator<Item = &'static UtilityRow> {
    // The index hands back the winning root's row indices in registry
    // order (the scan's own order); the shape filter is unchanged.
    let matched: Vec<&'static UtilityRow> = match crate::roots_index::longest_prefix(body) {
        Some(entry) => entry
            .rows
            .iter()
            .map(|&index| &ROWS[index as usize])
            .filter(|row| kind_accepts(row.kind, body, row.root))
            .collect(),
        None => Vec::new(),
    };
    matched.into_iter()
}

/// The pre-index linear scan — the documented DIFFERENTIAL ORACLE.
/// Byte-for-byte the read path the registry shipped before the
/// generated roots index landed: the longest root that prefixes the
/// body wins, every winning-root row whose shape accepts the suffix
/// returns in registry order. Production reads go through
/// [`lookup`]/[`lookup_detailed`] (the index's fast path); the oracle
/// stays public so the tests and the lookup bench can battery the two
/// against each other over the roots, the composites, the adversarial
/// misses and the deterministic fuzz — the semantics cannot drift
/// with the fast path.
pub fn lookup_linear(body: &str) -> impl Iterator<Item = &'static UtilityRow> {
    // Longest root that prefixes the body wins; ties (several rows of
    // the same root) all return, in registry order.
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

/// [`lookup`] with a typed WHY: the same rows on a hit (pinned equal by
/// the registry tests — the silent seam stays the zero-alloc hot path,
/// this one adds the failure MODE), and a [`LookupError`] on a miss:
/// no root at all vs. the longest root's arms all rejecting the suffix
/// shape. The P2 compile stage and the linter's "did you mean" surface
/// match on that mode instead of probing an empty iterator.
///
/// # Errors
/// [`LookupError::UnknownRoot`] when no registered root prefixes
/// `body`; [`LookupError::ShapeRejected`] when the longest matching
/// root's value shapes all reject the suffix (the error carries the
/// deduplicated labels that were tried, in registry order).
pub fn lookup_detailed(body: &str) -> Result<Vec<&'static UtilityRow>, LookupError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(body = %body, "registry lookup (detailed)");
    // The winning root from the generated index (the linear scan's
    // `best` computation is the oracle's job now); the rest of the
    // typed seam — the shape filter, the tried-labels error — reads
    // the winning root's rows exactly as before.
    let Some(entry) = crate::roots_index::longest_prefix(body) else {
        #[cfg(feature = "tracing")]
        tracing::debug!(body = %body, "no registered root prefixes the body");
        return Err(LookupError::UnknownRoot {
            body: body.to_string(),
        });
    };
    let root = entry.root;
    let root_rows: Vec<&'static UtilityRow> = entry
        .rows
        .iter()
        .map(|&index| &ROWS[index as usize])
        .collect();
    let rows: Vec<&'static UtilityRow> = root_rows
        .iter()
        .copied()
        .filter(|row| kind_accepts(row.kind, body, row.root))
        .collect();
    if rows.is_empty() {
        let mut tried: Vec<&'static str> = Vec::new();
        for row in &root_rows {
            shape_labels(row.kind, &mut tried);
        }
        #[cfg(feature = "tracing")]
        tracing::debug!(
            root,
            body = %body,
            tried = ?tried,
            "root matched but no value shape accepted the suffix"
        );
        return Err(LookupError::ShapeRejected {
            body: body.to_string(),
            root,
            tried,
        });
    }
    #[cfg(feature = "tracing")]
    tracing::trace!(rows = rows.len(), body = %body, "registry rows matched");
    Ok(rows)
}

/// Why the registry found no row for a utility BODY — the typed read
/// side ([`lookup_detailed`]) of the "did you mean" surface the crate
/// docs promise (the P2 linter and compile stage both consume it).
///
/// With the optional `miette` feature the variants additionally carry a
/// structured `Diagnostic` code + help text (the CLI's instrumentation
/// seam, off by default so the default graph stays dependency-light).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LookupError {
    /// No registered root prefixes the body (`"not-a-utility"`).
    UnknownRoot {
        /// The offending body, verbatim.
        body: String,
    },
    /// The longest root that prefixes the body has rows (`"p"` for
    /// `"p-red"`), but no row's [`crate::ValueKind`] accepted the
    /// suffix shape.
    ShapeRejected {
        /// The offending body, verbatim.
        body: String,
        /// The longest root that matched the body.
        root: &'static str,
        /// The deduplicated value-shape labels the root carries, in
        /// registry order (`["spacing", "arbitrary"]` for `p`) —
        /// [`crate::ValueKind::AnyOf`] arms flatten to their inner
        /// kinds, so the list says WHAT was possible.
        tried: Vec<&'static str>,
    },
}

impl std::fmt::Display for LookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LookupError::UnknownRoot { body } => {
                write!(f, "no registered utility root prefixes {body:?}")
            }
            LookupError::ShapeRejected { body, root, tried } => write!(
                f,
                "root {root:?} matched {body:?} but no value shape accepted the suffix (tried: {})",
                tried.join(", ")
            ),
        }
    }
}

impl std::error::Error for LookupError {}

#[cfg(feature = "miette")]
impl miette::Diagnostic for LookupError {
    fn code(&self) -> Option<Box<dyn std::fmt::Display + '_>> {
        Some(Box::new(match self {
            LookupError::UnknownRoot { .. } => "aeolus::registry::unknown_root",
            LookupError::ShapeRejected { .. } => "aeolus::registry::shape_rejected",
        }))
    }

    fn help(&self) -> Option<Box<dyn std::fmt::Display + '_>> {
        Some(Box::new(match self {
            LookupError::UnknownRoot { .. } => {
                "check the spelling against `registry::roots()` / `generate_docs()`'s root table; the registry covers the golden corpus's utility surface, not arbitrary CSS"
                    .to_string()
            }
            LookupError::ShapeRejected { root, .. } => format!(
                "the root {root:?} exists, but the suffix after `root-` fits none of its value shapes (see the row's ValueKind in `generate_docs()`); a bracketed arbitrary value may be the legal spelling"
            ),
        }))
    }
}

/// The deduplicated value-shape labels a root's rows carry, in
/// registry order — [`ValueKind::AnyOf`] arms FLATTEN to their inner
/// kinds (the audit view says WHAT was tried: `spacing`, `arbitrary`,
/// ... — never the uninformative `any-of` umbrella label).
fn shape_labels(kind: crate::ValueKind, out: &mut Vec<&'static str>) {
    match kind {
        crate::ValueKind::AnyOf(inner) => {
            for k in inner {
                shape_labels(*k, out);
            }
        }
        k => {
            let label = k.label();
            if !out.contains(&label) {
                out.push(label);
            }
        }
    }
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

/// The registry's canonical order position for a utility BODY — the
/// FIRST matching row's index in [`crate::rows::ROWS`] (plan §5.5's
/// "class sorting (official order from the registry)"; the crate and
/// rows docs name the read side). The rows are committed in the audited
/// tailwindcss v4 utilities progression (spacing → sizing → layout →
/// colour → borders → typography → interaction), so a sort keyed on
/// these indices reorders a class string the way the pipeline's own CSS
/// emit arm renders rules.
///
/// The match rules are [`lookup`]'s, byte for byte: the generated roots
/// index answers WHICH root (the longest-prefix probe), the shape
/// contract filters its rows, and the smallest index among the accepted
/// rows wins. A miss is `None` — no root, or a rejected suffix shape,
/// means NO order position, never a guessed one (the caller reports the
/// typed miss; the compile stage's sort seam does).
///
/// The differential oracle is the test battery's linear scan
/// (`tests/registry.rs`) — the same discipline [`lookup_linear`] keeps
/// for the lookup seam.
pub fn sort_index(body: &str) -> Option<usize> {
    let entry = crate::roots_index::longest_prefix(body)?;
    entry
        .rows
        .iter()
        .filter(|&&index| {
            let row = &ROWS[index as usize];
            kind_accepts(row.kind, body, row.root)
        })
        .map(|&index| index as usize)
        .min()
}

/// The total committed row count — the boundary the compile stage's
/// sort seam keys registered custom utilities AFTER (registration
/// order starts at this index; the registry always wins a name
/// collision by construction).
pub fn row_count() -> usize {
    ROWS.len()
}

/// Render the registry as the markdown audit page — the proof that the
/// committed rows are the single source the generated docs flow from.
/// The P2 xtask grows this into the full docs/linter generator story
/// (crate docs); the sort-order half landed as the [`sort_index`] read
/// side.
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
