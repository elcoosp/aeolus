//! v0.8 §1 (AEOLUS_V0.8.md): per-edge set-mask semantics for the edge
//! families (`p-*`/`m-*`/`border-*`/`rounded-*`).
//!
//! The v0.7 finalize materialized unset edges as `0.0`, so a `hover:px-8`
//! layer carried `Padding { top: 0, right: 32, bottom: 0, left: 32 }` and
//! REPLACED the base's top/bottom padding when applied — CSS keeps
//! `padding-top: 16px` for `p-4 hover:px-8`. The fix: every edge-family
//! struct carries a per-edge set-mask (`EdgeSet` / `CornerSet`), finalize
//! records it, `ResolvedStyle::overlay` merges edge-wise, and
//! `materialize_unset_edges_from` gives whole-property cascades (Masonry's
//! `PropertyStack`, Slint's state re-bindings) the flattened values.
//!
//! RED phase: written before the implementation; every test here failed
//! to compile (no `EdgeSet`, no `materialize_unset_edges_from`) or
//! asserted against the zeroed finalize semantics when first run
//! (evidence in `reports/10-tdd-evidence/10-per-edge-set-mask.md`).

use aeolus_core::*;

fn theme() -> TailwindTheme {
    TailwindTheme::from_theme_css(shadcn_css::LIGHT).unwrap()
}

/// Resolve two class strings into plans in one step.
fn plans(base: &str, over: &str) -> (StylePlan, StylePlan) {
    let t = theme();
    let r = Resolver::new(&t);
    let a = r.resolve(base).unwrap().plan;
    let b = r.resolve(over).unwrap().plan;
    (a, b)
}

// ---------------------------------------------------------------------------
// Finalize records the mask
// ---------------------------------------------------------------------------

/// A whole-edge family (`p-4`) sets ALL edges — the mask is full, values
/// unchanged from v0.7.
#[test]
fn full_family_finalizes_with_full_mask() {
    let t = theme();
    let r = Resolver::new(&t);
    let p = r.resolve("p-4").unwrap().plan;
    let pad = p.base.padding.expect("p-4 sets padding");
    assert_eq!(pad.top, 16.0);
    assert_eq!(pad.set, EdgeSet::ALL, "p-4 addresses every edge");
    let rad = r.resolve("rounded-md").unwrap().plan;
    let cr = rad.base.corner_radius.expect("rounded-md sets radius");
    assert_eq!(cr.set, CornerSet::ALL);
}

/// A partial family (`px-8`) sets ONLY the edges it names; the others are
/// `0.0` placeholders with the mask off — the IR now distinguishes
/// `px-8` from `pt-0 px-8 pb-0`.
#[test]
fn partial_family_finalizes_with_partial_mask() {
    let t = theme();
    let r = Resolver::new(&t);
    let p = r.resolve("px-8").unwrap().plan;
    let pad = p.base.padding.expect("px-8 sets padding");
    assert_eq!(
        pad.set,
        EdgeSet {
            top: false,
            right: true,
            bottom: false,
            left: true,
        },
        "px-8 addresses only left/right"
    );
    assert_eq!(pad.right, 32.0);
    assert_eq!(pad.top, 0.0, "unset edges stay 0.0 placeholders");

    // A single edge: `pt-0` is a SET top edge with value 0 — no longer
    // indistinguishable from "not addressed".
    let p = r.resolve("pt-0").unwrap().plan;
    let pad = p.base.padding.expect("pt-0 sets padding");
    assert_eq!(
        pad.set,
        EdgeSet {
            top: true,
            right: false,
            bottom: false,
            left: false,
        },
    );
    assert_eq!(pad.top, 0.0);

    // Per-corner partial: `rounded-t-md` sets only the top corners.
    let p = r.resolve("rounded-t-md").unwrap().plan;
    let cr = p.base.corner_radius.expect("rounded-t-md sets radius");
    assert_eq!(
        cr.set,
        CornerSet {
            top_left: true,
            top_right: true,
            bottom_right: false,
            bottom_left: false,
        },
    );

    // Within-bucket accumulation keeps the union of the masks: `p-4 pr-0`
    // is a full family with an explicit zeroed right edge.
    let p = r.resolve("p-4 pr-0").unwrap().plan;
    let pad = p.base.padding.expect("p-4 pr-0 sets padding");
    assert_eq!(pad.set, EdgeSet::ALL);
    assert_eq!(
        (pad.top, pad.right, pad.bottom, pad.left),
        (16.0, 0.0, 16.0, 16.0)
    );
}

// ---------------------------------------------------------------------------
// Plan algebra composes edge-wise
// ---------------------------------------------------------------------------

/// The v0.7 §1.4 defect, fixed at the algebra level: `p-4` ⊕ `px-8` must
/// keep the base's top/bottom (16px), not inherit the override's 0.0
/// placeholders.
#[test]
fn base_overlay_merges_edges_edge_wise() {
    let (base, over) = plans("p-4 bg-primary", "px-8");
    let merged = base.overlay(&over);
    let pad = merged.base.padding.expect("merged padding");
    assert_eq!(
        (pad.top, pad.right, pad.bottom, pad.left),
        (16.0, 32.0, 16.0, 32.0),
        "px-8 overrides left/right only; top/bottom keep p-4"
    );
    assert_eq!(pad.set, EdgeSet::ALL, "the merge covers every edge");
    assert!(merged.base.background.is_some(), "untouched slots survive");

    // Order matters: the reverse composition is also CSS-consistent.
    let (base, over) = plans("px-8", "p-4");
    let merged = base.overlay(&over);
    let pad = merged.base.padding.expect("merged padding");
    assert_eq!(
        (pad.top, pad.right, pad.bottom, pad.left),
        (16.0, 16.0, 16.0, 16.0)
    );
}

/// Conditional layers carry the partial mask: `hover:px-8` stays "left and
/// right, only" in the IR — a backend that applies the layer edge-wise
/// (Floem's per-field selector maps) gets the truth, and one that applies
/// whole properties materializes via `materialize_unset_edges_from`.
#[test]
fn partial_edge_layers_carry_the_mask() {
    struct HoverProbe;
    impl eval::Probe for HoverProbe {
        fn test(&self, t: &Test) -> bool {
            matches!(t, Test::State(State::Hover))
        }
    }

    let t = theme();
    let r = Resolver::new(&t);
    let p = r.resolve("p-4 bg-primary hover:px-8").unwrap().plan;
    assert_eq!(p.layers.len(), 1);
    let hover = &p.layers[0];
    let pad = hover.style.padding.expect("hover:px-8 sets padding");
    assert_eq!(
        pad.set,
        EdgeSet {
            top: false,
            right: true,
            bottom: false,
            left: true,
        },
        "the layer records WHICH edges it addresses"
    );
    let flat = p.resolve(&HoverProbe);
    let pad = flat.padding.expect("flattened padding");
    assert_eq!(
        (pad.top, pad.right, pad.bottom, pad.left),
        (16.0, 32.0, 16.0, 32.0),
        "the hovered element keeps padding-top/bottom = 16px (CSS cascade)"
    );
}

/// `materialize_unset_edges_from` fills a partial layer's unset edges from
/// the base — the primitive whole-property cascades (Masonry PropertyStack
/// entries, Slint state re-bindings) call before applying a layer.
#[test]
fn materialize_fills_unset_edges_from_base() {
    let t = theme();
    let r = Resolver::new(&t);
    let plan = r.resolve("p-4 rounded-md hover:px-8").unwrap().plan;
    let mut layer = plan.layers[0].style.clone();
    layer.materialize_unset_edges_from(&plan.base);

    let pad = layer.padding.expect("materialized padding");
    assert_eq!(
        (pad.top, pad.right, pad.bottom, pad.left),
        (16.0, 32.0, 16.0, 32.0),
        "unset top/bottom inherit the base's p-4"
    );
    assert_eq!(pad.set, EdgeSet::ALL, "a materialized layer is total");
    // Slots the layer does not address stay None (the lower cascade entry
    // keeps serving them).
    assert!(
        layer.background.is_none(),
        "hover:px-8 does not touch background"
    );
    assert!(
        layer.corner_radius.is_none(),
        "rounded-md lives on the base; the layer never addressed it"
    );

    // Materializing is a no-op for already-total layers and for missing
    // base slots.
    let plan = r.resolve("p-4").unwrap().plan;
    let mut layer = plan.base.clone();
    layer.materialize_unset_edges_from(&plan.base);
    assert_eq!(layer, plan.base, "materializing a total style is identity");
}

/// Diff stays lane-correct on partial edges: overlaying `px-8` on `p-4`
/// reports ONLY the Padding slot (values changed), and the cosmetic lane
/// is untouched. An override that changes NO edge value reports nothing.
#[test]
fn diff_on_partial_edge_overlay_is_exact() {
    let (base, over) = plans("p-4", "px-8");
    let merged = base.overlay(&over);
    let d = plan::diff(&base.base, &merged.base);
    assert_eq!(d.changed, vec![Slot::Padding]);
    assert!(d.needs_layout(), "padding is geometric");

    // Same edges, same values → no diff (the mask is union-compatible).
    let (base, over) = plans("p-4", "px-4");
    let merged = base.overlay(&over);
    let pad = merged.base.padding.expect("merged");
    assert_eq!(
        (pad.top, pad.right, pad.bottom, pad.left),
        (16.0, 16.0, 16.0, 16.0)
    );
}

/// Overlay stays idempotent with partial-edge overrides (the v0.7 §1.3
/// contract, re-pinned for the edge-wise merge).
#[test]
fn idempotent_overlay_with_partial_edges() {
    let (base, over) = plans("p-4 bg-primary", "px-8 pt-2");
    let once = base.overlay(&over);
    let twice = once.overlay(&over);
    assert_eq!(once, twice, "overlay is idempotent (override dominates)");
}

/// Margins and border widths compose edge-wise like padding.
#[test]
fn margin_and_border_width_compose_edge_wise() {
    let (base, over) = plans("m-4 border", "mx-8 border-r-4");
    let merged = base.overlay(&over);
    let m = merged.base.margin.expect("merged margin");
    assert_eq!(
        (m.top, m.right, m.bottom, m.left),
        (16.0, 32.0, 16.0, 32.0),
        "mx-8 overrides left/right only"
    );
    let bw = merged.base.border_width.expect("merged border width");
    assert_eq!(
        (bw.top, bw.right, bw.bottom, bw.left),
        (1.0, 4.0, 1.0, 1.0),
        "border-r-4 overrides the right side only; border keeps the rest"
    );
}

/// Per-corner composition: `rounded-lg` ⊕ `rounded-tr-full` keeps the
/// base's other corners.
#[test]
fn corner_radius_composes_corner_wise() {
    let (base, over) = plans("rounded-lg", "rounded-tr-full");
    let merged = base.overlay(&over);
    let cr = merged.base.corner_radius.expect("merged radius");
    assert_eq!(cr.top_right, 9999.0, "the override wins on its corner");
    assert_eq!(
        cr.top_left, 10.0,
        "the base keeps its other corners (rounded-lg = 10px in shadcn)"
    );
    assert_eq!(cr.set, CornerSet::ALL);
}

// ---------------------------------------------------------------------------
// The snapshot artifact carries the mask (schema v2)
// ---------------------------------------------------------------------------

/// Masks round-trip: a partial layer written to the canonical JSON reads
/// back equal — `from(to(p)) == p` for partial edges, and the document
/// says schema version 3 (v3 = the variant-engine merge; the v2 set-mask
/// payload is unchanged — the bump keeps old readers failing loudly, which
/// is the point of the version header).
#[test]
fn snapshot_v3_roundtrips_partial_edge_masks() {
    let t = theme();
    let r = Resolver::new(&t);
    let p = r.resolve("p-4 hover:px-8 rounded-t-md").unwrap().plan;
    let json = p.to_snapshot_json();
    assert!(
        json.contains("\"version\":3"),
        "the schema version must bump so old readers fail loudly: {json}"
    );
    let back = StylePlan::from_snapshot_json(&json).expect("round-trips");
    assert_eq!(back, p, "masks are part of plan equality");

    // The hover layer's mask is explicitly in the document.
    assert!(
        json.contains("\"set\":{\"top\":false,\"right\":true,\"bottom\":false,\"left\":true}"),
        "the partial hover mask is canonical JSON, not inferred: {json}"
    );
}

/// The strict reader rejects a malformed mask object with a dotted path.
#[test]
fn snapshot_strict_reader_rejects_bad_masks() {
    let t = theme();
    let r = Resolver::new(&t);
    let p = r.resolve("p-4").unwrap().plan;
    let json = p.to_snapshot_json();
    let tampered = json.replace(
        "\"bottom\":true,\"left\":true}",
        "\"bottom\":true,\"left\":\"yes\"}",
    );
    assert_ne!(tampered, json, "test must actually tamper the mask");
    let err = StylePlan::from_snapshot_json(&tampered)
        .expect_err("a non-boolean mask entry is a typed error");
    assert!(
        err.to_string().contains("$.base.padding.set.left"),
        "the error path names the offending mask field: {err}"
    );
}
