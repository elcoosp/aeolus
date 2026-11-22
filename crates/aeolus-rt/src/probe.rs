//! The conformance probe corpus (plan §5.1's per-backend score's WALK
//! half): one frozen program per registry row, the row's own target
//! longhands riding beside it. Each interpreter backend walks these
//! through its OWN `lower_program` lane (and reads its own
//! `slot_coverage` ledger over the same programs) to derive its
//! per-longhand verdicts — the published conformance tables are the
//! emitters' verdicts, never a parallel claim. The corpus itself is
//! the REGISTRY's grammar: the probe class for a row is synthesized
//! from the row's own root and value shape, so a registry edit moves
//! the corpus with it and no second table can drift.
//!
//! The class strings compile through the SAME bake the no-arg macros
//! run (the embedded shadcn table) — the probes are ordinary static
//! programs, exactly the shape every backend lane consumes.

use aeolus_compile::{Config, compile_theme, keyword_alignment, keyword_texts};
use aeolus_data::ValueKind;
use aeolus_data::rows::ROWS;
use aeolus_ir::Prop;
use aeolus_theme::{ThemeSource, TokenTable};

/// One probe: the row's class string compiled and frozen into the
/// static form, with the longhands the row targets (the verdict a
/// backend answers for the program fans out to ALL of them — a row's
/// targets share the mapping fate; the per-prop verdict is the BEST
/// tier across the corpus).
pub struct ProbeProgram {
    /// The frozen program (the shadcn bake, the no-arg macros' shape).
    pub program: StaticProgram,
    /// The longhands the row lowers to (the registry's own targets).
    pub targets: &'static [Prop],
}

use crate::StaticProgram;

/// The suffix one keyword-table entry spells. Colour-shaped tables mix
/// plain CSS keywords (`inherit` — the themed walk refuses them: the
/// suffix must be a TOKEN in the supplied table) with theme-referenced
/// entries (`black` → `var(--color-black)`); the probe takes the first
/// THEME-REFERENCED spelling when one exists, else the table's first.
/// The two- and three-tuple tables (keywords vs the text-size
/// composite) share the probe shape: the NAME is the spelling, so the
/// selector takes `(name, themed)` pairs.
fn pick_suffix<'a>(entries: impl Iterator<Item = (&'a str, bool)>) -> Option<&'a str> {
    let mut first: Option<&'a str> = None;
    for (name, themed) in entries {
        if first.is_none() {
            first = Some(name);
        }
        if themed {
            return Some(name);
        }
    }
    first
}

/// The class string one registry row probes with: the FIRST suffix the
/// row's own value shape accepts (the row's grammar, no parallel
/// table). Arbitrary rows carry a bracketed px value — the candidate's
/// arbitrary group is the passthrough's own shape.
fn probe_class(root: &str, kind: ValueKind) -> String {
    let suffix = match kind {
        ValueKind::Bare(_) => None,
        ValueKind::Keywords(table) => {
            pick_suffix(table.iter().map(|(n, v)| (*n, v.starts_with("var(--"))))
        }
        ValueKind::Composite(table) => {
            pick_suffix(table.iter().map(|(n, v, _)| (*n, v.starts_with("var(--"))))
        }
        ValueKind::Spacing => Some("4"),
        ValueKind::Fraction => Some("1/2"),
        ValueKind::Number => Some("10"),
        ValueKind::PositiveInt => Some("2"),
        ValueKind::Percent => Some("20%"),
        ValueKind::Arbitrary => Some("[2px]"),
        ValueKind::AnyOf(kinds) => {
            // The first sub-kind's spelling (the row's own preference
            // order; nesting is forbidden, the registry tests pin it).
            return match kinds.first().copied().expect("any-of rows are non-empty") {
                ValueKind::Spacing => format!("{root}-4"),
                ValueKind::Fraction => format!("{root}-1/2"),
                ValueKind::Number | ValueKind::PositiveInt => format!("{root}-10"),
                ValueKind::Percent => format!("{root}-20%"),
                ValueKind::Arbitrary => format!("{root}-[2px]"),
                ValueKind::Keywords(table) => {
                    let suffix =
                        pick_suffix(table.iter().map(|(n, v)| (*n, v.starts_with("var(--"))))
                            .expect("keyword rows are non-empty");
                    format!("{root}-{suffix}")
                }
                ValueKind::Composite(table) => {
                    let suffix =
                        pick_suffix(table.iter().map(|(n, v, _)| (*n, v.starts_with("var(--"))))
                            .expect("composite rows are non-empty");
                    format!("{root}-{suffix}")
                }
                ValueKind::Bare(_) => root.to_string(),
                ValueKind::AnyOf(_) => unreachable!("any-of nesting is forbidden"),
            };
        }
    };
    match suffix {
        Some(suffix) => format!("{root}-{suffix}"),
        None => root.to_string(),
    }
}

/// The hand probes: longhands NO registry row lowers to, spelled by
/// their own grammar (the `content-["…"]` string form routes to
/// `Prop::Content` through the value pass's hint targets — the root's
/// keyword half is `align-content`). The coverage pin below refuses a
/// corpus that misses a longhand, so this list grows with the IR.
const HAND_PROBES: &[(&str, &[Prop])] = &[("content-[\"Hello\"]", &[Prop::Content])];

/// The probe corpus: one entry per registry row whose class string
/// lowers clean (the registry is the compile walk's own source of
/// truth — a row that refuses is a registry bug the compile
/// diagnostics name, and the conformance coverage pin catches the
/// missing longhands), plus the hand probes.
pub fn probe_programs() -> Vec<ProbeProgram> {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::baked(&source).expect("the baked shadcn table builds");
    let config = Config::default();
    let mut out = Vec::with_capacity(ROWS.len() + HAND_PROBES.len());
    for (class, targets) in HAND_PROBES.iter().copied().chain(ROWS.iter().map(|row| {
        (
            probe_class(row.root, row.kind).leak() as &'static str,
            row.targets,
        )
    })) {
        let Ok(compiled) = compile_theme(class, &table, &config) else {
            continue;
        };
        if !compiled.diagnostics.is_empty() {
            continue;
        }
        let Ok(annotations) = keyword_texts(class, &config) else {
            continue;
        };
        let keywords = keyword_alignment(&compiled, &annotations);
        let program = StaticProgram::new(
            compiled.program.layers,
            Box::leak(keywords.into_boxed_slice()),
        );
        out.push(ProbeProgram { program, targets });
    }
    out
}

#[cfg(test)]
mod coverage_tests {
    use super::probe_programs;
    use aeolus_ir::Prop;

    /// The corpus covers EVERY IR longhand — the conformance tables'
    /// total partition is only honest when every longhand was actually
    /// probed. A registry row that stops lowering (or a new `Prop`
    /// variant without a probe) fails here, naming the gap.
    #[test]
    fn the_probe_corpus_covers_every_longhand() {
        let probes = probe_programs();
        let mut covered = std::collections::HashSet::new();
        for probe in &probes {
            covered.extend(probe.targets.iter().copied());
        }
        let missing: Vec<String> = Prop::ALL
            .iter()
            .filter(|prop| !covered.contains(prop))
            .map(|prop| format!("{prop:?}"))
            .collect();
        assert!(
            missing.is_empty(),
            "the probe corpus misses longhands — add the spelling to HAND_PROBES: {missing:?}"
        );
    }
}
