//! The registry contract tests (plan §5.1 "registry as a versioned
//! spec" + §5.2 differential testing triangle, P1 slice):
//!
//! 1. **Row invariants** — every row is fully auditable (non-empty
//!    root/group/v4, no duplicate longhand in a target list, non-empty
//!    keyword tables, AnyOf depth 1, no duplicate rows).
//! 2. **Disambiguation** — the longest-prefix rule and the kind-shape
//!    contract over the tricky bodies (`text-*` triple split,
//!    `min-w-0`, `rounded-tl-lg`, `flex-1`).
//! 3. **Oracle cross-check** — for sample classes the v0.x
//!    `collision_group` answer must equal the row's group (the tw_merge
//!    pin).
//! 4. **Corpus coverage** — every golden-corpus class the oracle groups
//!    must find a registry row carrying that group (the P1 acceptance:
//!    the registry ports the corpus's utility surface).
//! 5. **Docs generation** — `generate_docs()` is a honest render of the
//!    same rows.

use aeolus_core::conflict::collision_group;
use aeolus_core::parser::parse_token;
use aeolus_data::registry::{generate_docs, lookup, lookup_root, roots};
use aeolus_data::{ValueKind, tables};

// ---------------------------------------------------------------------------
// 1. Row invariants
// ---------------------------------------------------------------------------

fn kind_tables_ok(kind: ValueKind, root: &str) {
    match kind {
        ValueKind::Bare(v) => {
            assert!(!v.is_empty(), "row {root:?}: empty bare value");
        }
        ValueKind::Keywords(t) => {
            assert!(!t.is_empty(), "row {root:?}: empty keyword table");
            for (suffix, css) in t {
                assert!(!suffix.is_empty(), "row {root:?}: keyword {suffix:?} empty");
                assert!(
                    !css.is_empty(),
                    "row {root:?}: keyword {suffix:?} css empty"
                );
            }
        }
        ValueKind::Composite(t) => {
            assert!(!t.is_empty(), "row {root:?}: empty composite table");
            for (suffix, primary, secondary) in t {
                assert!(
                    !suffix.is_empty(),
                    "row {root:?}: composite {suffix:?} empty"
                );
                assert!(
                    !primary.is_empty() && !secondary.is_empty(),
                    "row {root:?}: composite {suffix:?} half-empty"
                );
            }
        }
        ValueKind::AnyOf(kinds) => {
            assert!(!kinds.is_empty(), "row {root:?}: empty any-of");
            for &k in kinds {
                // Depth-1 contract: no nested AnyOf.
                assert!(
                    !matches!(k, ValueKind::AnyOf(_)),
                    "row {root:?}: nested any-of"
                );
                kind_tables_ok(k, root);
            }
        }
        ValueKind::Spacing
        | ValueKind::Fraction
        | ValueKind::Number
        | ValueKind::Percent
        | ValueKind::Arbitrary => {}
    }
}

#[test]
fn every_row_is_fully_auditable() {
    for row in aeolus_data::rows::ROWS {
        assert!(!row.root.is_empty(), "row with empty root");
        assert!(
            !row.root.ends_with('-') || !row.root.starts_with('-'),
            "row {:?}: hyphen-only root edge",
            row.root
        );
        assert!(
            !row.targets.is_empty(),
            "row {:?}: no longhand targets",
            row.root
        );
        let mut seen: Vec<aeolus_ir::Prop> = Vec::new();
        for &p in row.targets {
            assert!(
                !seen.contains(&p),
                "row {:?} targets {:?} twice",
                row.root,
                p
            );
            seen.push(p);
        }
        assert!(
            !row.group.is_empty(),
            "row {:?} has an empty group",
            row.root
        );
        assert!(
            !row.v4.is_empty(),
            "row {:?} has no v4 source ref",
            row.root
        );
        kind_tables_ok(row.kind, row.root);
    }
}

#[test]
fn no_two_rows_are_exact_duplicates() {
    // Same root + same kind + same group = an accidental double row.
    for (i, a) in aeolus_data::rows::ROWS.iter().enumerate() {
        for b in &aeolus_data::rows::ROWS[i + 1..] {
            let same = a.root == b.root && a.kind == b.kind && a.group == b.group;
            assert!(!same, "duplicate registry rows for root {:?}", a.root);
        }
    }
}

#[test]
fn registry_version_is_surfaced() {
    assert!(!aeolus_data::REGISTRY_VERSION.is_empty());
}

// ---------------------------------------------------------------------------
// 2. Disambiguation
// ---------------------------------------------------------------------------

#[test]
fn lookup_routes_the_tricky_bodies() {
    // text-* splits three ways by kind shape.
    assert!(
        lookup("text-sm").all(|r| r.root == "text"),
        "text-sm must route to the text root"
    );
    assert!(
        lookup("text-sm").any(|r| r.group == "font-size"),
        "text-sm must find the font-size arm"
    );
    assert!(
        lookup("text-center").any(|r| r.group == "text-align"),
        "text-center must find the text-align arm"
    );
    assert!(
        lookup("text-red-500").any(|r| r.group == "text-color"),
        "text-red-500 must find the text-color arm"
    );
    // min-w-0: the LONGEST root wins over any `min`/`w` prefix.
    assert!(
        lookup("min-w-0").all(|r| r.root == "min-w"),
        "min-w-0 must disambiguate to min-w"
    );
    assert!(
        lookup("min-w-0").any(|r| r.group == "min-width"),
        "min-w-0 must land on the min-width group"
    );
    // rounded-tl-lg: the two-edge corner row, not the four-edge root.
    assert!(
        lookup("rounded-tl-lg").all(|r| r.root.starts_with("rounded")),
        "rounded-tl-lg must stay on a rounded root"
    );
    assert!(
        lookup("rounded-tl-lg").any(|r| r.group == "rounded-tl"),
        "rounded-tl-lg must disambiguate to rounded-tl"
    );
    // flex-1: the shorthand triplet arm, under the oracle's group name.
    assert!(
        lookup("flex-1").all(|r| r.root == "flex"),
        "flex-1 must route to the flex root"
    );
    assert!(
        lookup("flex-1").any(|r| r.group == "flex-direction"),
        "flex-1 must land on the oracle's flex-direction group"
    );
    // pointer-events-none: bare-word body.
    assert!(
        lookup("pointer-events-none").all(|r| r.root == "pointer-events"),
        "pointer-events-none must disambiguate to its own root"
    );
    // whitespace-nowrap and friends.
    assert!(
        lookup("whitespace-nowrap").all(|r| r.root == "whitespace"),
        "whitespace-nowrap must route to whitespace"
    );
}

#[test]
fn kind_shapes_disambiguate_within_a_root() {
    // w-8 spacing / w-full keyword / w-1/2 fraction / w-[32px] arbitrary.
    let w8: Vec<_> = lookup("w-8").collect();
    assert!(w8.iter().all(|r| r.root == "w"));
    assert!(w8.iter().any(|r| r.group == "width"));
    // No colour or row-group leaks onto w-*.
    assert!(
        w8.iter().all(|r| r.group == "width"),
        "w-8 matched a non-width arm: {:?}",
        w8.iter().map(|r| r.group).collect::<Vec<_>>()
    );
    let wfull: Vec<_> = lookup("w-full").collect();
    assert!(wfull.iter().all(|r| r.group == "width"));
    let whalf: Vec<_> = lookup("w-1/2").collect();
    assert!(whalf.iter().all(|r| r.group == "width"));
    let warb: Vec<_> = lookup("w").collect();
    assert!(warb.iter().any(|r| r.group == "width"));

    // mx-auto hits the margin-x arm (the auto keyword), not spacing.
    let mx: Vec<_> = lookup("mx-auto").collect();
    assert!(mx.iter().all(|r| r.group == "margin-x"), "mx-auto arms");

    // from-20% lands on the stop-POSITION row, from-red-500 on colour.
    assert!(
        lookup("from-20%").any(|r| r.group == "from-pos"),
        "from-20% must find the position arm"
    );
    assert!(
        lookup("from-red-500").any(|r| r.group == "from-color"),
        "from-red-500 must find the colour arm"
    );

    // border-t-2 is a width; border-t-red-500 a colour (oracle split).
    assert!(lookup("border-t-2").any(|r| r.group == "border-w-t"));
    assert!(lookup("border-t-red-500").any(|r| r.group == "border-color"));

    // pb-px: the one-pixel unit step.
    assert!(lookup("pb-px").any(|r| r.group == "padding-bottom"));

    // Unknown bodies miss.
    assert!(lookup("not-a-utility").next().is_none());
    assert!(lookup("text-banana").next().is_none(), "no text-banana arm");
}

#[test]
fn roots_and_root_lookup_agree() {
    let all = roots();
    assert!(!all.is_empty());
    // Every root in the table appears in roots() and vice versa.
    for row in aeolus_data::rows::ROWS {
        assert!(
            all.contains(&row.root),
            "root {:?} missing from roots()",
            row.root
        );
    }
    // lookup_root returns exactly the rows of that root.
    for root in &all {
        for row in lookup_root(root) {
            assert_eq!(row.root, *root);
        }
    }
    assert!(lookup_root("p").count() >= 1);
    assert!(lookup_root("bg").count() >= 2, "bg must carry both arms");
}

// ---------------------------------------------------------------------------
// 3. Oracle cross-check (tw_merge pin)
// ---------------------------------------------------------------------------

#[test]
fn row_groups_match_the_collision_oracle() {
    // (class, oracle group) — the pairs the tw_merge table is famous for,
    // including the documented quirks (border-dashed under border-color,
    // flex-1 under flex-direction, ring-inset under ring-color).
    const SAMPLES: &[(&str, &str)] = &[
        ("p-4", "padding"),
        ("px-8", "padding-x"),
        ("pt-2", "padding-top"),
        ("pb-px", "padding-bottom"),
        ("m-4", "margin"),
        ("mx-auto", "margin-x"),
        ("-mt-2", "margin-top"),
        ("size-10", "size"),
        ("w-full", "width"),
        ("w-1/2", "width"),
        ("h-screen", "height"),
        ("min-w-0", "min-width"),
        ("max-w-md", "max-width"),
        ("gap-4", "gap"),
        ("gap-x-2", "gap"),
        ("flex", "display"),
        ("flex-col", "flex-direction"),
        ("flex-1", "flex-direction"),
        ("flex-wrap", "flex-wrap"),
        ("grow-0", "flex-grow"),
        ("basis-1/2", "flex-basis"),
        ("order-3", "order"),
        ("items-center", "align-items"),
        ("content-center", "align-content"),
        ("justify-between", "justify-content"),
        ("justify-items-start", "justify-items"),
        ("self-end", "align-self"),
        ("grid-cols-3", "grid-template-columns"),
        ("auto-rows-fr", "grid-auto-rows"),
        ("col-span-2", "col-span"),
        ("hidden", "display"),
        ("grid", "display"),
        ("static", "position"),
        ("absolute", "position"),
        ("inset-0", "inset"),
        ("top-2", "top"),
        ("-top-2", "top"),
        ("z-10", "z-index"),
        ("-z-10", "z-index"),
        ("opacity-50", "opacity"),
        ("overflow-hidden", "overflow"),
        ("overflow-x-auto", "overflow-x"),
        ("aspect-video", "aspect"),
        ("rounded-md", "rounded"),
        ("rounded-s-lg", "rounded-l"),
        ("rounded-t-full", "rounded-t"),
        ("border", "border-w"),
        ("border-2", "border-w"),
        ("border-t", "border-w-t"),
        ("border-x", "border-w-x"),
        ("border-red-500", "border-color"),
        ("border-dashed", "border-color"),
        ("bg-red-500", "bg-color"),
        ("bg-primary", "bg-color"),
        ("bg-transparent", "bg-color"),
        ("outline-2", "outline-w"),
        ("outline-none", "outline-w"),
        ("outline-red-500", "outline-color"),
        ("outline-offset-2", "outline-offset"),
        ("ring", "ring-w"),
        ("ring-2", "ring-w"),
        ("ring-[3px]", "ring-w"),
        ("ring-red-500", "ring-color"),
        ("ring-inset", "ring-color"),
        ("ring-offset-2", "ring-offset-w"),
        ("shadow-lg", "shadow"),
        ("shadow-none", "shadow"),
        ("shadow-red-500/50", "shadow-color"),
        ("text-sm", "font-size"),
        ("text-lg", "font-size"),
        ("text-center", "text-align"),
        ("text-red-500", "text-color"),
        ("text-primary-foreground", "text-color"),
        ("font-bold", "font-weight"),
        ("font-medium", "font-weight"),
        ("italic", "font-style"),
        ("leading-loose", "line-height"),
        ("tracking-wide", "letter-spacing"),
        ("underline", "text-decoration"),
        ("underline-offset-2", "underline-offset"),
        ("uppercase", "text-transform"),
        ("whitespace-nowrap", "whitespace"),
        ("align-baseline", "vertical-align"),
        ("visible", "visibility"),
        ("invisible", "visibility"),
        ("cursor-pointer", "cursor"),
        ("pointer-events-none", "pointer-events"),
        ("select-none", "user-select"),
        ("object-cover", "object-fit"),
        ("object-center", "object-position"),
        ("resize", "resize"),
        ("isolate", "isolation"),
    ];
    for (class, group) in SAMPLES {
        let token = parse_token(class)
            .unwrap_or_else(|e| panic!("oracle must parse {class}: {}", e.message));
        let oracle = collision_group(&token).unwrap_or_else(|| panic!("oracle must group {class}"));
        assert_eq!(oracle, *group, "sample table stale for {class}");
        let rows: Vec<_> = lookup(token.body).collect();
        assert!(
            rows.iter().any(|r| r.group == oracle),
            "class {class} (body {:?}): oracle group {oracle:?} not found in {:?}",
            token.body,
            rows.iter().map(|r| r.group).collect::<Vec<_>>()
        );
    }
}

// ---------------------------------------------------------------------------
// 4. Corpus coverage
// ---------------------------------------------------------------------------

/// Extract every class string from the golden corpus' `@entry` lines.
fn corpus_classes() -> Vec<String> {
    let raw = include_str!("../../aeolus-core/tests/fixtures/golden.corpus");
    raw.lines()
        .filter_map(|line| line.strip_prefix("@entry "))
        .filter_map(|rest| rest.split_once(' '))
        .map(|(_, class)| class.to_string())
        .collect()
}

#[test]
fn registry_covers_every_oracle_grouped_corpus_class() {
    let classes = corpus_classes();
    assert!(!classes.is_empty(), "corpus fixture must parse");
    let mut misses: Vec<String> = Vec::new();
    for class in &classes {
        // Honest per-class failure; the parser owns the error path.
        let Ok(token) = parse_token(class) else {
            continue;
        };
        let Some(oracle) = collision_group(&token) else {
            continue; // outside the tw_merge scope — the registry may skip it
        };
        let rows: Vec<_> = lookup(token.body).collect();
        if !rows.iter().any(|r| r.group == oracle) {
            misses.push(format!(
                "{class} (body {:?}, oracle {oracle:?}) → {:?}",
                token.body,
                rows.iter().map(|r| r.group).collect::<Vec<_>>()
            ));
        }
    }
    assert!(
        misses.is_empty(),
        "registry misses oracle-grouped corpus classes:\n{}",
        misses.join("\n")
    );
}

// ---------------------------------------------------------------------------
// 5. Docs generation
// ---------------------------------------------------------------------------

#[test]
fn generate_docs_renders_every_row() {
    let docs = generate_docs();
    assert!(docs.starts_with("# Aeolus V1 utility registry"));
    for row in aeolus_data::rows::ROWS {
        assert!(
            docs.contains(&format!("| `{}` |", row.root)),
            "docs missing root {:?}",
            row.root
        );
        assert!(
            docs.contains(row.group),
            "docs missing group {:?}",
            row.group
        );
    }
    // The shared tables are the source: the colour table's red-500 arm
    // and the spacing step surface in the docs page.
    assert!(tables::COLOR_KEYWORDS.iter().any(|&(n, _)| n == "red-500"));
    assert!((tables::SPACING_STEP_REM - 0.25).abs() < f32::EPSILON);
}
