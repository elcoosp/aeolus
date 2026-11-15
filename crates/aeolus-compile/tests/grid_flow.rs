//! The grid auto-flow family's contracts (architecture plan §3.7's grid
//! row — the "auto-track/auto-flow slots stay the reported boundary"
//! line's second half): the `grid-flow-*` classes lower through the SAME
//! typed pipeline as every keyword family — the `GridAutoFlow` longhand,
//! the registry row's keyword table, the CSS arm's value form, the
//! interp arm, the static and rust-tokens arms — and the honest
//! boundaries stay typed (the v0.x plan has no auto-flow slot: the
//! `SlotUnsupported` note, the grid-auto-rows precedent; the GPUI lane
//! refuses per longhand with the reason pointing at the CSS arm).
//!
//! The auto-TRACK half of the boundary line landed with the composite
//! grammar (the `composites` battery: `grid-cols-[auto_1fr_auto]`,
//! `auto-cols-[minmax(0,1fr)_auto]`); this battery lands the auto-FLOW
//! half — the keyword family the registry carries no rows for before
//! the milestone.
//!
//! Written FIRST (the RED spec): the suite fails until the family
//! lowers — `Prop` has no auto-flow longhand and the registry carries
//! no grid-flow rows before the milestone.

use aeolus_compile::{
    Config, Env, PlanNoteKind, compile, emit_css, emit_plan, emit_rust, interp, keyword_texts, sort,
};
use aeolus_ir::{Prop, Val};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn config() -> Config {
    Config::default()
}

/// The one base declaration `class` lowers to (the family is a
/// single-longhand keyword row).
fn base_decl_of(class: &str) -> (Prop, Val) {
    let compiled = compile(class, &config()).expect("non-empty input compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "{class}: unexpected diagnostics {:?}",
        compiled.diagnostics
    );
    let layers = compiled.program.layers;
    assert_eq!(layers.len(), 1, "{class}: one base layer");
    let decls = layers[0].decls;
    assert_eq!(decls.len(), 1, "{class}: one declaration, got {decls:?}");
    (decls[0].prop, decls[0].val)
}

// ---------------------------------------------------------------------------
// 1. The keyword scale: the v4.1 grid-auto-flow values in slot order
// ---------------------------------------------------------------------------

#[test]
fn the_flow_scale_lowers_onto_the_auto_flow_longhand() {
    // The v4 default scale in keyword-slot order (the CSS values the
    // table carries): row=0, col=1, dense=2, row-dense=3, col-dense=4.
    for (class, slot) in [
        ("grid-flow-row", 0u8),
        ("grid-flow-col", 1),
        ("grid-flow-dense", 2),
        ("grid-flow-row-dense", 3),
        ("grid-flow-col-dense", 4),
    ] {
        let (prop, val) = base_decl_of(class);
        assert_eq!(prop, Prop::GridAutoFlow, "{class}");
        assert_eq!(val, Val::Kw(slot), "{class}");
    }
}

#[test]
fn the_annotation_seam_carries_the_css_values() {
    // The keyword texts ARE the CSS values (v4's own table): `col`
    // renders `column`, the dense pairs render the two-value form.
    let classes = keyword_texts(
        "grid-flow-row grid-flow-col grid-flow-dense grid-flow-row-dense grid-flow-col-dense",
        &config(),
    )
    .expect("annotates");
    let texts: Vec<&str> = classes
        .iter()
        .flat_map(|class| class.keywords.iter().map(|(_, text)| *text))
        .collect();
    assert_eq!(
        texts,
        ["row", "column", "dense", "row dense", "column dense"],
        "the v4 grid-auto-flow values verbatim"
    );
}

// ---------------------------------------------------------------------------
// 2. The CSS arm: the value form
// ---------------------------------------------------------------------------

#[test]
fn the_css_arm_renders_the_value_form() {
    let sheet = emit_css("grid grid-flow-col-dense", &config()).expect("non-empty input renders");
    let text = sheet.to_string();
    assert!(text.contains("grid-auto-flow: column dense;"), "{text}");
}

// ---------------------------------------------------------------------------
// 3. The variant composition: the family rides the typed conds
// ---------------------------------------------------------------------------

#[test]
fn the_family_rides_the_variant_conds() {
    let compiled = compile("md:grid-flow-row hover:grid-flow-col", &config()).expect("compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let conds: Vec<_> = compiled
        .program
        .layers
        .iter()
        .map(|layer| layer.cond)
        .collect();
    assert_eq!(conds.len(), 2, "two variant layers, got {conds:?}");
}

// ---------------------------------------------------------------------------
// 4. The interp + static + rust-tokens arms
// ---------------------------------------------------------------------------

#[test]
fn the_interp_answers_the_auto_flow_longhand() {
    let compiled = compile("grid-flow-dense", &config()).expect("compiles");
    let interped = interp(&compiled.program, &Env::default()).expect("resolves");
    assert!(interped.touched().contains(Prop::GridAutoFlow));
}

#[test]
fn the_static_arm_freezes_the_slot() {
    let compiled = compile("grid-flow-row-dense", &config()).expect("compiles");
    let text = compiled.emit_static();
    assert!(
        text.contains("Val::Kw(3)"),
        "the row-dense slot freezes: {text}"
    );
    let again = compile("grid-flow-row-dense", &config())
        .expect("compiles")
        .emit_static();
    assert_eq!(text, again, "the static text is deterministic");
}

#[test]
fn the_rust_tokens_arm_carries_the_value_text() {
    let sheet = emit_rust("grid-flow-col", &config()).expect("the fragments");
    assert!(
        sheet.to_string().contains("column"),
        "the annotation resolves the CSS value: {sheet}"
    );
}

// ---------------------------------------------------------------------------
// 5. The sort seam: the family rides the registry order
// ---------------------------------------------------------------------------

#[test]
fn the_sort_seam_orders_the_family() {
    let sorted = sort("grid-flow-col-dense grid").expect("sorts");
    let order: Vec<&str> = sorted.classes.iter().map(|c| c.raw.as_str()).collect();
    assert_eq!(
        order,
        ["grid-flow-col-dense", "grid"],
        "registry row order (the grid-* utility section precedes the display rows \
         in the committed row order — the registry's canonical positions)"
    );
}

// ---------------------------------------------------------------------------
// 6. The plan bridge: no v0.x auto-flow slot — the typed note
// ---------------------------------------------------------------------------

#[test]
fn the_plan_bridge_reports_the_auto_flow_slot_as_unsupported() {
    // The v0.x plan carries grid templates/starts but no auto-flow slot
    // (the oracle's own shape) — the `SlotUnsupported` note, the
    // grid-auto-rows precedent.
    let sheet = emit_plan("grid-flow-row-dense", &config()).expect("the plan sheet");
    let props: Vec<_> = sheet
        .notes
        .iter()
        .filter_map(|note| match &note.kind {
            PlanNoteKind::SlotUnsupported { prop } => Some(*prop),
            _ => None,
        })
        .collect();
    assert!(
        props.contains(&Prop::GridAutoFlow),
        "the auto-flow longhand reports: {props:?}"
    );
}
