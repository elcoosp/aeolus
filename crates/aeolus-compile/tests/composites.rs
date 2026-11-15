//! The composite arbitrary-value grammar's contracts (architecture plan
//! §3.1's value pass, the "multi-token composite grammar (`calc()`,
//! underscore joins, url/image values …)" boundary line): the composite
//! spellings lower through the SAME typed pipeline as every unit and
//! colour — the IR's verbatim [`Val`] slot, the kind-gated row eligibility,
//! the CSS arm's verbatim rule text, the interp arm, the IR-level `cn()`
//! merge, the metamorphic triangle, the static and rust-tokens arms — and
//! the honest boundaries stay typed (the shorthand rows' per-longhand fan,
//! the filter/backdrop handles, the shadow lists, the malformed
//! functions, the negative flag, the opacity modifier).
//!
//! The kind gate is the design's centre: a composite lowers onto the rows
//! whose slots can carry it (the sized functions on the single-target
//! scalar rows, the track lists on the grid-template/auto groups, the
//! image functions on the bg-image group, the bare `n/d` ratio on the
//! aspect row) and refuses TYPED everywhere else — never a silent drop,
//! never an invalid-CSS passthrough on a slot that cannot hold the value.
//!
//! Written FIRST (the RED spec): the suite fails until the composite
//! grammar lowers — the IR carries no verbatim slot and the value pass
//! refuses every composite before the milestone.

use aeolus_compile::{
    Config, Env, PlanNoteKind, cn, compile, emit_css, emit_plan, emit_rust, interp,
    keyword_alignment, keyword_texts, metamorphic_check, sort,
};
use aeolus_ir::{Prop, Val};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn config() -> Config {
    Config::default()
}

/// The one base declaration `class` lowers to (the composite families'
/// eligible rows are single-longhand rows).
fn base_decl_of(class: &str) -> (Prop, Val) {
    let compiled = compile(class, &config()).expect("non-empty input compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "{class}: unexpected diagnostics {:?}",
        compiled.diagnostics
    );
    let layers = compiled.program.layers;
    assert_eq!(
        layers.len(),
        1,
        "{class}: one base layer, got {:?}",
        layers.iter().map(|l| l.cond).collect::<Vec<_>>()
    );
    let decls = layers[0].decls;
    assert_eq!(decls.len(), 1, "{class}: one declaration, got {decls:?}");
    (decls[0].prop, decls[0].val)
}

fn refused(class: &str) -> String {
    let compiled = compile(class, &config()).expect("the walk is total");
    assert!(
        !compiled.diagnostics.is_empty(),
        "{class}: expected a typed refusal, got none"
    );
    compiled.diagnostics[0].to_string()
}

// ---------------------------------------------------------------------------
// 1. The sized functions: calc()/min()/max()/clamp() on the scalar rows
// ---------------------------------------------------------------------------

#[test]
fn calc_lowers_verbatim_on_the_width_row() {
    let (prop, val) = base_decl_of("w-[calc(100%-2rem)]");
    assert_eq!(prop, Prop::Width);
    assert_eq!(
        val,
        Val::Verbatim("calc(100%-2rem)"),
        "the value text rides verbatim (v4's own arbitrary contract — the \
         author owns the CSS validity)"
    );
}

#[test]
fn the_calc_family_lowers_on_the_sized_rows() {
    for (class, prop) in [
        ("min-w-[min(50%,300px)]", Prop::MinWidth),
        ("max-w-[max(50%,300px)]", Prop::MaxWidth),
        ("w-[clamp(1rem,50%,3rem)]", Prop::Width),
    ] {
        let (p, val) = base_decl_of(class);
        assert_eq!(p, prop, "{class}");
        assert!(
            matches!(val, Val::Verbatim(_)),
            "{class}: the sized function lowers verbatim, got {val:?}"
        );
    }
}

#[test]
fn the_hinted_length_contract_takes_the_composite() {
    // The `length:` hint is the caller's explicit type assertion — it
    // widens the unit grammar to the sized functions.
    let (prop, val) = base_decl_of("w-[length:calc(100%-2rem)]");
    assert_eq!(prop, Prop::Width);
    assert_eq!(val, Val::Verbatim("calc(100%-2rem)"));
}

// ---------------------------------------------------------------------------
// 2. The underscore joins: the track lists on the grid rows
// ---------------------------------------------------------------------------

#[test]
fn underscore_joins_decode_to_spaces_on_the_track_rows() {
    let (prop, val) = base_decl_of("grid-cols-[auto_1fr_auto]");
    assert_eq!(prop, Prop::GridTemplateColumns);
    assert_eq!(
        val,
        Val::Verbatim("auto 1fr auto"),
        "the underscore IS a space"
    );

    let (prop, val) = base_decl_of("grid-rows-[minmax(0,1fr)_2fr]");
    assert_eq!(prop, Prop::GridTemplateRows);
    assert_eq!(val, Val::Verbatim("minmax(0,1fr) 2fr"));
}

#[test]
fn the_track_functions_lower_on_the_track_rows() {
    let (prop, val) = base_decl_of("grid-cols-[repeat(2,minmax(0,1fr))]");
    assert_eq!(prop, Prop::GridTemplateColumns);
    assert_eq!(val, Val::Verbatim("repeat(2,minmax(0,1fr))"));

    let (prop, val) = base_decl_of("auto-cols-[minmax(0,1fr)_auto]");
    assert_eq!(prop, Prop::GridAutoColumns);
    assert_eq!(val, Val::Verbatim("minmax(0,1fr) auto"));
}

#[test]
fn an_underscore_list_on_a_scalar_row_refuses_typed() {
    // `w` takes ONE length — the underscore list is a track-shape value
    // the width slot cannot carry. The composite boundary names it.
    let text = refused("w-[3px_4px]");
    assert!(
        text.contains("composite"),
        "the boundary names the composite grammar: {text}"
    );
}

// ---------------------------------------------------------------------------
// 3. The image functions: url() on the bg-image row
// ---------------------------------------------------------------------------

#[test]
fn url_lowers_on_the_image_row_and_the_colour_sibling_skips() {
    let (prop, val) = base_decl_of("bg-[url(/img/hero.png)]");
    assert_eq!(prop, Prop::BackgroundImage);
    assert_eq!(val, Val::Verbatim("url(/img/hero.png)"));
}

#[test]
fn the_hinted_url_and_image_forms_lower_on_the_image_row() {
    // The `url:` hint wraps the bare path in the function (v4's own
    // reading); the `image:` hint's value is already a function and
    // rides verbatim.
    let (prop, val) = base_decl_of("bg-[url:/img/hero.png]");
    assert_eq!(prop, Prop::BackgroundImage);
    assert_eq!(val, Val::Verbatim("url(/img/hero.png)"));

    let (prop, val) = base_decl_of("bg-[image:linear-gradient(to_right,#333,#666)]");
    assert_eq!(prop, Prop::BackgroundImage);
    assert_eq!(
        val,
        Val::Verbatim("linear-gradient(to right,#333,#666)"),
        "the underscores decode everywhere (v4's own grammar)"
    );
}

#[test]
fn a_url_on_a_sized_row_refuses_typed() {
    // The image functions are image-slot values — the width row takes
    // none (v4's own type gate).
    let text = refused("w-[url(x.png)]");
    assert!(
        text.contains("composite"),
        "the boundary names the gate: {text}"
    );
}

// ---------------------------------------------------------------------------
// 4. The bare ratio: aspect-[16/9] on the aspect row
// ---------------------------------------------------------------------------

#[test]
fn the_bare_slash_ratio_lowers_on_the_aspect_row() {
    let (prop, val) = base_decl_of("aspect-[16/9]");
    assert_eq!(prop, Prop::AspectRatio);
    assert_eq!(val, Val::Verbatim("16/9"));
}

#[test]
fn the_hinted_ratio_form_lowers_on_the_aspect_row() {
    let (prop, val) = base_decl_of("aspect-[ratio:16/9]");
    assert_eq!(prop, Prop::AspectRatio);
    assert_eq!(val, Val::Verbatim("16/9"));
}

#[test]
fn a_bare_ratio_on_other_rows_refuses_typed() {
    // `16/9` is an aspect-shape value — the width row takes no ratio.
    let text = refused("w-[16/9]");
    assert!(
        text.contains("composite"),
        "the boundary names the gate: {text}"
    );
}

// ---------------------------------------------------------------------------
// 5. The typed boundaries: shorthand fans, filter handles, shadow lists,
//    malformed functions, the negative flag, the opacity modifier
// ---------------------------------------------------------------------------

#[test]
fn the_shorthand_fans_refuse_the_composite() {
    // `p` fans ONE value over four longhands — a composite cannot fan
    // (each longhand would carry the whole list). Typed, never fanned.
    let text = refused("p-[calc(100%-2rem)]");
    assert!(text.contains("composite"), "{text}");
}

#[test]
fn the_filter_handles_refuse_the_composite() {
    // The filter/backdrop handles take ONE value (the `${fn}(${value})`
    // form) — the composite stays the documented boundary (the §3.7
    // row's own line).
    for class in ["blur-[2px_4px]", "brightness-[calc(100%+10%)]"] {
        let text = refused(class);
        assert!(text.contains("composite"), "{class}: {text}");
    }
}

#[test]
fn the_shadow_lists_refuse_typed() {
    // The arbitrary shadow lists stay the codegen host's surface (the
    // closed-world preset table owns the shadow row).
    let text = refused("shadow-[0_1px_2px_#000]");
    assert!(text.contains("composite"), "{text}");
}

#[test]
fn a_malformed_function_refuses_typed() {
    // The parens must balance — a truncated function is a boundary, not
    // a verbatim passthrough of broken CSS.
    let text = refused("w-[calc(]");
    assert!(
        text.contains("balance") || text.contains("malformed"),
        "{text}"
    );
}

#[test]
fn the_negative_flag_has_no_composite_form() {
    // The scalar slots flip their number; the verbatim text carries its
    // own signs — the flag composes nothing here.
    let text = refused("-w-[calc(100%-2rem)]");
    assert!(text.contains("negative"), "{text}");
}

#[test]
fn the_opacity_modifier_refuses_the_composite() {
    // The modifier rides colour-form values only — the existing
    // post-check, now pinned over the verbatim slot.
    refused("w-[calc(100%-2rem)]/50");
}

// ---------------------------------------------------------------------------
// 6. The CSS arm: the verbatim rule text
// ---------------------------------------------------------------------------

#[test]
fn the_css_arm_renders_the_verbatim_rule() {
    let sheet = emit_css("w-[calc(100%-2rem)]", &config()).expect("non-empty input renders");
    let text = sheet.to_string();
    assert!(text.contains("width: calc(100%-2rem);"), "{text}");
}

#[test]
fn the_css_arm_renders_the_track_rule() {
    let sheet = emit_css("grid-cols-[auto_1fr_auto]", &config()).expect("renders");
    let text = sheet.to_string();
    assert!(
        text.contains("grid-template-columns: auto 1fr auto;"),
        "{text}"
    );
}

#[test]
fn the_css_arm_renders_the_image_rule() {
    let sheet = emit_css("bg-[url(/img/hero.png)]", &config()).expect("renders");
    let text = sheet.to_string();
    assert!(
        text.contains("background-image: url(/img/hero.png);"),
        "{text}"
    );
}

#[test]
fn the_css_arm_renders_the_ratio_rule() {
    let sheet = emit_css("aspect-[16/9]", &config()).expect("renders");
    let text = sheet.to_string();
    assert!(text.contains("aspect-ratio: 16/9;"), "{text}");
}

// ---------------------------------------------------------------------------
// 7. The IR-level cn() merge: the verbatim slot folds per longhand
// ---------------------------------------------------------------------------

#[test]
fn cn_folds_the_verbatim_slot_per_longhand() {
    let calc = compile("w-[calc(100%-2rem)]", &config()).expect("compiles");
    let half = compile("w-1/2", &config()).expect("compiles");
    // Last occurrence wins per longhand — the verbatim slot is one more
    // value form in the same fold.
    let merged = cn(&[&calc, &half]).expect("merges");
    let width = merged
        .program
        .layers
        .iter()
        .flat_map(|layer| layer.decls.iter())
        .find(|decl| decl.prop == Prop::Width)
        .expect("the width longhand survives");
    assert_eq!(width.val, Val::Pct(50.0));

    let merged = cn(&[&half, &calc]).expect("merges");
    let width = merged
        .program
        .layers
        .iter()
        .flat_map(|layer| layer.decls.iter())
        .find(|decl| decl.prop == Prop::Width)
        .expect("the width longhand survives");
    assert_eq!(width.val, Val::Verbatim("calc(100%-2rem)"));
}

// ---------------------------------------------------------------------------
// 8. The interp arm: the verbatim decl rides
// ---------------------------------------------------------------------------

#[test]
fn the_interp_answers_the_verbatim_decl() {
    let compiled = compile("w-[calc(100%-2rem)]", &config()).expect("compiles");
    let interped = interp(&compiled.program, &Env::default()).expect("resolves");
    let width = interped
        .decls
        .iter()
        .find(|decl| decl.prop == Prop::Width)
        .expect("the effective declaration carries the width");
    assert_eq!(width.val, Val::Verbatim("calc(100%-2rem)"));
}

// ---------------------------------------------------------------------------
// 9. The metamorphic triangle: the three answers agree on the verbatim text
// ---------------------------------------------------------------------------

#[test]
fn the_metamorphic_walk_agrees_on_the_composites() {
    for input in [
        "w-[calc(100%-2rem)]",
        "grid-cols-[auto_1fr_auto]",
        "bg-[url(/img/hero.png)]",
        "aspect-[16/9]",
    ] {
        let report = metamorphic_check(input, &config()).expect("the walk");
        assert!(
            report.agree(),
            "{input}: the three answers agree: {report:?}"
        );
        for situation in &report.situations {
            assert!(
                situation.mismatches.is_empty(),
                "{input}: {:?}: {:?}",
                situation.label,
                situation.mismatches
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 10. The plan bridge: the composite value is a typed note, never a drop
// ---------------------------------------------------------------------------

#[test]
fn the_plan_bridge_reports_the_composite_value() {
    // The v0.x plan slots carry baked numerics and row keywords — the
    // verbatim text has no slot. The note names the longhand and the
    // text; the plan's width slot stays unset (never a silent zero).
    let sheet = emit_plan("w-[calc(100%-2rem)]", &config()).expect("the plan sheet");
    let reported = sheet
        .notes
        .iter()
        .any(|note| matches!(note.kind, PlanNoteKind::CompositeValue { .. }));
    assert!(reported, "the composite note rides: {:?}", sheet.notes);
}

// ---------------------------------------------------------------------------
// 11. The static + rust-tokens arms freeze the verbatim slot
// ---------------------------------------------------------------------------

#[test]
fn the_static_arm_freezes_the_verbatim_literal() {
    let compiled = compile("w-[calc(100%-2rem)]", &config()).expect("compiles");
    let text = compiled.emit_static();
    assert!(
        text.contains(r#"Val::Verbatim("calc(100%-2rem)")"#),
        "the verbatim literal freezes: {text}"
    );
    let again = compile("w-[calc(100%-2rem)]", &config())
        .expect("compiles")
        .emit_static();
    assert_eq!(text, again, "the static text is deterministic");
}

#[test]
fn the_rust_tokens_arm_carries_the_verbatim_text() {
    let sheet = emit_rust("w-[calc(100%-2rem)]", &config()).expect("the fragments");
    let rendered = sheet.to_string();
    assert!(
        rendered.contains("calc(100%-2rem)"),
        "the verbatim text renders in the fragment: {rendered}"
    );
}

// ---------------------------------------------------------------------------
// 12. The sort seam and the annotation alignment: composites ride the
//     registry order; the alignment answers "" (the value carries its
//     own text, the numbers' convention)
// ---------------------------------------------------------------------------

#[test]
fn the_sort_seam_orders_the_composites_by_registry_order() {
    let sorted = sort("flex w-[calc(100%-2rem)] grid-cols-[auto_1fr_auto]").expect("sorts");
    assert!(
        sorted
            .classes
            .iter()
            .filter(|class| class.raw == "w-[calc(100%-2rem)]")
            .count()
            == 1,
        "the composite sorts with the rest: {:?}",
        sorted
            .classes
            .iter()
            .map(|c| c.raw.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn the_annotation_alignment_answers_empty_for_the_verbatim_slot() {
    // A value that carries its own text aligns to "" — the numbers',
    // colours' and tokens' own convention (the seam's documented rule).
    let compiled = compile("w-[calc(100%-2rem)]", &config()).expect("compiles");
    let annotations = keyword_texts("w-[calc(100%-2rem)]", &config()).expect("annotates");
    let texts = keyword_alignment(&compiled, &annotations);
    assert_eq!(
        texts,
        [""],
        "the verbatim decl needs no row text: {texts:?}"
    );
}
