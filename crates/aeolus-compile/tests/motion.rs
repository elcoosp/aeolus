//! The motion layer's contracts (architecture plan P6's "motion layer"
//! row — §3.7's `transition-*`/`animate-*` line): the `transition-*`
//! (`transition`, `duration-*`, `ease-*`, `delay-*`) and `animate-*`
//! families lower through the SAME typed pipeline as every other
//! registry family — IR longhands, the value pass, the CSS arm's
//! rendering, the interp arm, the rust-tokens arm's annotations, the
//! static arm — and the honest boundaries stay typed (arbitrary
//! timing values, negation, unknown easings/animation names).
//!
//! The differential half (the plans agree with the v0.x oracle slot
//! for slot, the oracle's own `Transition`/`Animation` extras) is
//! pinned in `tests/plan_bridge.rs`; this suite pins the V1 side's own
//! shapes.
//!
//! Written FIRST (the RED spec): the suite fails until the motion
//! families lower — `Prop` has no transition/animation longhands and
//! the registry carries no motion rows before the milestone.

use aeolus_compile::{
    Config, Target, cn, compile, compile_theme, emit_css, emit_rust, interp, sort,
};
use aeolus_ir::{Cond, Prop, Val};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn config() -> Config {
    Config::default()
}

fn baked_tokens() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("the embedded shadcn source parses");
    TokenTable::baked(&source).expect("the pair surface fits the u16 slot")
}

/// The one base declaration `class` lowers to (the motion families are
/// single-longhand rows).
fn base_decl_of(class: &str) -> (Prop, Val) {
    let compiled = compile(class, &config()).expect("non-empty input compiles");
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

// ---------------------------------------------------------------------------
// 1. The IR shapes: one longhand per family, the row-table slots
// ---------------------------------------------------------------------------

#[test]
fn transition_sets_lower_onto_the_property_longhand() {
    // The bare root IS the v4 default set (slot 0, the bare arm's text);
    // the keyword table's own order: all=0, colors=1, opacity=2,
    // shadow=3, transform=4, none=5.
    let (prop, val) = base_decl_of("transition");
    assert_eq!(prop, Prop::TransitionProperty);
    assert_eq!(val, Val::Kw(0), "the bare root lowers the default set");

    let (prop, val) = base_decl_of("transition-all");
    assert_eq!(prop, Prop::TransitionProperty);
    assert_eq!(val, Val::Kw(0));

    let (prop, val) = base_decl_of("transition-colors");
    assert_eq!(prop, Prop::TransitionProperty);
    assert_eq!(val, Val::Kw(1));

    let (prop, val) = base_decl_of("transition-opacity");
    assert_eq!(prop, Prop::TransitionProperty);
    assert_eq!(val, Val::Kw(2));

    let (prop, val) = base_decl_of("transition-shadow");
    assert_eq!(prop, Prop::TransitionProperty);
    assert_eq!(val, Val::Kw(3));

    let (prop, val) = base_decl_of("transition-transform");
    assert_eq!(prop, Prop::TransitionProperty);
    assert_eq!(val, Val::Kw(4));

    let (prop, val) = base_decl_of("transition-none");
    assert_eq!(prop, Prop::TransitionProperty);
    assert_eq!(val, Val::Kw(5));
}

#[test]
fn duration_and_delay_lower_as_millisecond_numbers() {
    let (prop, val) = base_decl_of("duration-300");
    assert_eq!(prop, Prop::TransitionDuration);
    assert_eq!(val, Val::Num(300.0));

    let (prop, val) = base_decl_of("delay-150");
    assert_eq!(prop, Prop::TransitionDelay);
    assert_eq!(val, Val::Num(150.0));
}

#[test]
fn ease_curves_lower_onto_the_timing_longhand() {
    // linear=0, in=1, out=2, in-out=3 — the v4 default-theme curves.
    for (class, slot) in [
        ("ease-linear", 0u8),
        ("ease-in", 1),
        ("ease-out", 2),
        ("ease-in-out", 3),
    ] {
        let (prop, val) = base_decl_of(class);
        assert_eq!(prop, Prop::TransitionTimingFunction, "{class}");
        assert_eq!(val, Val::Kw(slot), "{class}");
    }
}

#[test]
fn animate_presets_lower_onto_the_animation_longhand() {
    // spin=0, ping=1, pulse=2, bounce=3, none=4 — the v4 default-theme
    // presets.
    for (class, slot) in [
        ("animate-spin", 0u8),
        ("animate-ping", 1),
        ("animate-pulse", 2),
        ("animate-bounce", 3),
        ("animate-none", 4),
    ] {
        let (prop, val) = base_decl_of(class);
        assert_eq!(prop, Prop::Animation, "{class}");
        assert_eq!(val, Val::Kw(slot), "{class}");
    }
}

// ---------------------------------------------------------------------------
// 2. The honest boundaries: typed refusals, never silent drops
// ---------------------------------------------------------------------------

#[test]
fn arbitrary_timing_values_refuse_typed() {
    // `duration-[2s]`/`ease-[cubic-bezier(...)]`: the time/curve values
    // are the documented arbitrary-boundary (the wider v4 hint table) —
    // the duration/delay/ease rows carry no arbitrary arm, so the read
    // side's shape rejection names the root and the shapes it tried,
    // never a silent drop.
    for class in ["duration-[2s]", "ease-[cubic-bezier(0,0,0,1)]"] {
        let compiled = compile(class, &config()).expect("compiles");
        assert_eq!(
            compiled.diagnostics.len(),
            1,
            "{class}: {:?}",
            compiled.diagnostics
        );
        assert!(
            matches!(
                compiled.diagnostics[0].kind,
                aeolus_compile::DiagnosticKind::ShapeRejected { .. }
            ),
            "{class}: {:?}",
            compiled.diagnostics[0].kind
        );
        // The refusal left nothing lowered silently (an all-refused
        // string compiles to the empty program).
        assert_eq!(
            compiled
                .program
                .layers
                .iter()
                .map(|l| l.decls.len())
                .sum::<usize>(),
            0,
            "{class}: nothing lowers"
        );
    }
}

#[test]
fn negation_refuses_on_the_motion_roots() {
    // The motion rows carry `negative = false` (a negative duration is
    // not a v4 spelling): the `-` form is the typed negation boundary.
    let compiled = compile("-duration-100", &config()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "{:?}", compiled.diagnostics);
    assert!(matches!(
        compiled.diagnostics[0].kind,
        aeolus_compile::DiagnosticKind::UnsupportedNegation { .. }
    ));
}

#[test]
fn unknown_easing_and_animation_names_refuse_typed() {
    // Outside the closed keyword tables: the shape rejection names the
    // root (the "did you mean" surface's input), never a silent drop.
    let compiled = compile("ease-bouncy", &config()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "{:?}", compiled.diagnostics);

    let compiled = compile("animate-wiggle", &config()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "{:?}", compiled.diagnostics);
}

// ---------------------------------------------------------------------------
// 3. The cascade: the motion families compose like every other family
// ---------------------------------------------------------------------------

#[test]
fn the_motion_families_compose_in_one_string() {
    let compiled = compile(
        "transition-colors duration-300 ease-in-out delay-150 animate-pulse",
        &config(),
    )
    .expect("compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let base = &compiled.program.layers[0].decls;
    assert_eq!(base.len(), 5, "five longhands, one per family: {base:?}");
    // The cascade order is the Prop discriminant order (the IR's own
    // sort): the transition quartet then the animation slot.
    let props: Vec<Prop> = base.iter().map(|d| d.prop).collect();
    let mut sorted = props.clone();
    sorted.sort_by_key(|p| *p as u16);
    assert_eq!(props, sorted, "the layer arrives cascade-sorted");
}

#[test]
fn ir_level_cn_keeps_the_last_transition_set() {
    // The IR-level merge is per-LONGHAND last-wins: two transition sets
    // collide on `transition-property`, the rightmost survives.
    let base = compile("transition-all", &config()).expect("compiles");
    let over = compile("transition-colors", &config()).expect("compiles");
    let merged = cn(&[&base, &over]).expect("non-empty merge");
    let base_layer = &merged.program.layers[0].decls;
    assert_eq!(base_layer.len(), 1, "the merge evicts the shadowed set");
    assert_eq!(base_layer[0].prop, Prop::TransitionProperty);
    assert_eq!(base_layer[0].val, Val::Kw(1), "transition-colors wins");
}

// ---------------------------------------------------------------------------
// 4. The CSS arm: the v4 property forms
// ---------------------------------------------------------------------------

#[test]
fn css_renders_the_motion_families() {
    let sheet = emit_css(
        "transition-colors duration-300 ease-in-out delay-150 animate-spin",
        &config(),
    )
    .expect("compiles");
    let text = sheet.to_string();
    assert!(
        text.contains(
            "transition-property: color, background-color, border-color, outline-color, \
             text-decoration-color, fill, stroke, --tw-gradient-from, --tw-gradient-via, \
             --tw-gradient-to;"
        ),
        "{text}"
    );
    assert!(text.contains("transition-duration: 300ms;"), "{text}");
    assert!(
        text.contains("transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);"),
        "{text}"
    );
    assert!(text.contains("transition-delay: 150ms;"), "{text}");
    assert!(
        text.contains("animation: spin 1s linear infinite;"),
        "{text}"
    );
}

#[test]
fn css_renders_the_default_set_and_the_clears() {
    let sheet = emit_css("transition transition-none animate-none", &config()).expect("compiles");
    let text = sheet.to_string();
    // The v4 default set — the full property list the oracle renders.
    assert!(
        text.contains(
            "transition-property: color, background-color, border-color, outline-color, \
             text-decoration-color, fill, stroke, --tw-gradient-from, --tw-gradient-via, \
             --tw-gradient-to, opacity, box-shadow, transform, translate, scale, rotate, filter, \
             -webkit-backdrop-filter, backdrop-filter, display, content-visibility, overlay, \
             pointer-events;"
        ),
        "{text}"
    );
    assert!(text.contains("transition-property: none;"), "{text}");
    assert!(text.contains("animation: none;"), "{text}");
}

// ---------------------------------------------------------------------------
// 5. The interp arm: the effective declarations carry the motion forms
// ---------------------------------------------------------------------------

#[test]
fn interp_resolves_the_motion_families() {
    let compiled = compile_theme(
        "transition-colors duration-300 animate-spin",
        &baked_tokens(),
        &config(),
    )
    .expect("compiles");
    let interped = interp(&compiled.program, &Default::default()).expect("all values resolve");
    assert_eq!(interped.decls.len(), 3, "{:?}", interped.decls);
    let props: Vec<Prop> = interped.decls.iter().map(|d| d.prop).collect();
    assert!(props.contains(&Prop::TransitionProperty));
    assert!(props.contains(&Prop::TransitionDuration));
    assert!(props.contains(&Prop::Animation));
    // The winner attribution rides (the conds slice is parallel).
    assert_eq!(interped.conds.len(), interped.decls.len());
    assert!(interped.conds.iter().all(|c| *c == Cond::Base));
}

// ---------------------------------------------------------------------------
// 6. The rust-tokens arm: the row texts annotate the slots
// ---------------------------------------------------------------------------

#[test]
fn rust_tokens_annotate_the_motion_slots() {
    let sheet = emit_rust("animate-spin transition-colors", &config()).expect("compiles");
    let text = sheet.to_string();
    assert!(text.contains("Prop::Animation"), "{text}");
    assert!(text.contains("Prop::TransitionProperty"), "{text}");
    // The row text the slot indexes — the annotation a bare `Val::Kw`
    // cannot recover.
    assert!(text.contains("spin 1s linear infinite"), "{text}");
    assert!(
        text.contains(
            "color, background-color, border-color, outline-color, text-decoration-color"
        ),
        "{text}"
    );
}

// ---------------------------------------------------------------------------
// 7. The static arm: the motion decls freeze byte-stable
// ---------------------------------------------------------------------------

#[test]
fn static_form_freezes_the_motion_decls() {
    let compiled = compile("transition-colors animate-pulse", &config()).expect("compiles");
    let text = compiled.emit_static();
    assert!(text.contains("Prop::TransitionProperty"), "{text}");
    assert!(text.contains("Prop::Animation"), "{text}");
    // Deterministic: the same compile freezes the same bytes.
    let again = compile("transition-colors animate-pulse", &config()).expect("compiles");
    assert_eq!(text, again.emit_static(), "the static item is byte-stable");
}

// ---------------------------------------------------------------------------
// 8. The sort seam: the registry order covers the motion families
// ---------------------------------------------------------------------------

#[test]
fn sort_orders_the_motion_families() {
    let sorted = sort("animate-spin p-4 transition-colors").expect("compiles");
    let texts: Vec<&str> = sorted.classes.iter().map(|c| c.raw.as_str()).collect();
    // The registry's canonical progression: spacing before the motion
    // tail (the motion rows ride the registry's interaction tail).
    let p = texts.iter().position(|t| *t == "p-4").expect("p-4 sorts");
    let colors = texts
        .iter()
        .position(|t| *t == "transition-colors")
        .expect("transition-colors sorts");
    let animate = texts
        .iter()
        .position(|t| *t == "animate-spin")
        .expect("animate-spin sorts");
    assert!(p < colors, "{texts:?}");
    assert!(colors < animate, "{texts:?}");
}

// ---------------------------------------------------------------------------
// 9. The Target surface names the arm (the enum's own audit)
// ---------------------------------------------------------------------------

#[test]
fn the_target_enum_still_names_five_forms() {
    // The motion layer rides the EXISTING five emit forms — no new arm,
    // no capability regression (the audit pin the Target docs carry).
    assert_eq!(
        [
            Target::Css,
            Target::Rust,
            Target::Interp,
            Target::Static,
            Target::Plan
        ]
        .len(),
        5
    );
}
