//! The static-data emit arm's contracts (architecture plan §3.1 emit
//! (b) — the second arm of the `Target` surface): a compiled [`Program`]
//! emits as the IR's OWN static form — one `static` item over
//! `Layer::new`/`Decl` constructors, `const`-constructible end to end,
//! compilable anywhere `aeolus-ir` is a dependency.
//!
//! The pins:
//!
//! * the text is DETERMINISTIC (same program, same bytes — the artifact
//!   contract, byte-stable like the v0.7 plan snapshots);
//! * every [`Val`] variant and every [`Cond`] form has an emission arm
//!   (the emitter is TOTAL over the IR: no wildcard, one arm per
//!   variant, the auditable-wall convention);
//! * the emitted construction calls [`Layer::new`], so the mask is
//!   DERIVED at construction — the emitter cannot drift the mask
//!   invariant the IR documents;
//! * floats emit as valid `f32` literals (whole values keep `.0`), and
//!   the non-finite slots — which the pipeline's finite guard refuses —
//!   emit as the `f32` constructors rather than as unreachable code;
//! * the emitted shape IS the static form: the doc test in the module
//!   compiles and runs it against the IR.
//!
//! Written FIRST (the RED spec): the suite fails against the 0.13.0
//! seam (the static arm did not exist) and passes only once
//! `emit_static` satisfies every pin below.

use aeolus_compile::{Compiled, Config, Target, compile, compile_theme, emit_static};
use aeolus_ir::{Cond, Decl, Layer, Program, Prop, Val};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The runtime-tier table over the shadcn light+dark source.
fn runtime_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

/// The baked-tier table over the same source.
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::baked(&source).expect("default-seeded source builds")
}

/// A one-layer base program over leaked decls — the IR's own documented
/// dynamic-form storage, which is what lets a test hand-build programs
/// the pipeline would never emit.
fn program_of(decls: &[Decl]) -> Program {
    let decls: &'static [Decl] = Box::leak(decls.to_vec().into_boxed_slice());
    let layers: &'static [Layer] =
        Box::leak(vec![Layer::new(Cond::Base, decls)].into_boxed_slice());
    Program::new(layers)
}

// ---------------------------------------------------------------------------
// 1. The artifact: deterministic, byte-stable text
// ---------------------------------------------------------------------------

#[test]
fn a_base_program_emits_the_static_form() {
    // `p-4` under the default (tailwind) spacing bakes four rem
    // declarations, in the IR's Prop cascade order.
    let compiled = compile("p-4", &Config::default()).expect("the string compiles");
    let text = emit_static(&compiled.program);
    assert_eq!(
        text,
        "static PROGRAM: &[aeolus_ir::Layer] = &[\n\
         \x20   aeolus_ir::Layer::new(\n\
         \x20       aeolus_ir::Cond::Base,\n\
         \x20       &[\n\
         \x20           aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingTop, val: aeolus_ir::Val::Rem(1.0) },\n\
         \x20           aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingRight, val: aeolus_ir::Val::Rem(1.0) },\n\
         \x20           aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingBottom, val: aeolus_ir::Val::Rem(1.0) },\n\
         \x20           aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingLeft, val: aeolus_ir::Val::Rem(1.0) },\n\
         \x20       ],\n\
         \x20   ),\n\
         ];\n",
        "the emitted static form, verbatim"
    );
}

#[test]
fn the_emit_is_deterministic() {
    let compiled = compile("p-4 px-8 hover:py-2", &Config::default()).expect("the string compiles");
    assert_eq!(
        emit_static(&compiled.program),
        emit_static(&compiled.program),
        "same program, same bytes — the artifact contract"
    );
}

#[test]
fn an_empty_program_emits_an_empty_static() {
    let text = emit_static(&Program::new(&[]));
    assert_eq!(
        text, "static PROGRAM: &[aeolus_ir::Layer] = &[\n];\n",
        "the empty program is still a compilable item"
    );
}

#[test]
fn whole_floats_keep_their_point_zero() {
    // `p-4` is 1.0rem — `1` is not a valid f32 literal; the emitter
    // keeps every value a valid literal.
    let compiled = compile("p-4", &Config::default()).expect("compiles");
    let text = emit_static(&compiled.program);
    assert!(
        text.contains("Val::Rem(1.0)") && !text.contains("Val::Rem(1)"),
        "whole floats keep `.0`: {text}"
    );
}

// ---------------------------------------------------------------------------
// 2. Totality: every Val variant, every Cond form
// ---------------------------------------------------------------------------

#[test]
fn every_val_variant_has_an_emission_arm() {
    let program = program_of(&[
        Decl {
            prop: Prop::Top,
            val: Val::Px(16.0),
        },
        Decl {
            prop: Prop::PaddingTop,
            val: Val::Rem(1.0),
        },
        Decl {
            prop: Prop::Width,
            val: Val::Pct(50.0),
        },
        Decl {
            prop: Prop::ZIndex,
            val: Val::Num(10.0),
        },
        Decl {
            prop: Prop::Background,
            val: Val::Rgba([23, 23, 23, 255]),
        },
        Decl {
            prop: Prop::Color,
            val: Val::Tok { id: 7, alpha: 0.5 },
        },
        Decl {
            prop: Prop::BorderColor,
            val: Val::ModePair(3),
        },
        Decl {
            prop: Prop::FlexDirection,
            val: Val::Kw(2),
        },
    ]);
    let text = emit_static(&program);
    assert!(text.contains("Val::Px(16.0)"), "{text}");
    assert!(text.contains("Val::Rem(1.0)"), "{text}");
    assert!(text.contains("Val::Pct(50.0)"), "{text}");
    assert!(text.contains("Val::Num(10.0)"), "{text}");
    assert!(text.contains("Val::Rgba([23, 23, 23, 255])"), "{text}");
    assert!(
        text.contains("Val::Tok { id: 7, alpha: 0.5 }"),
        "the runtime token rides as the IR carries it: {text}"
    );
    assert!(text.contains("Val::ModePair(3)"), "{text}");
    assert!(text.contains("Val::Kw(2)"), "{text}");
}

#[test]
fn every_cond_form_has_an_emission_arm() {
    // The REAL pipeline folds every Cond constructor — one string walks
    // the whole surface, the emitter's cond arms pinned end to end.
    let compiled = compile(
        "p-9 \
         hover:p-0 focus:p-1 focus-visible:p-2 focus-within:p-3 \
         active:p-4 disabled:p-5 visited:p-6 checked:p-7 \
         placeholder-shown:p-8 selection:p-0 placeholder:p-1 \
         group-hover:p-2 group-focus:p-3 peer-hover:p-4 peer-focus:p-5 \
         dark:p-6 sm:p-7 min-md:p-8 max-lg:p-0 @xl:p-1 @max-2xl:p-2",
        &Config::default(),
    )
    .expect("the string compiles");
    let text = emit_static(&compiled.program);
    for cond in [
        "Base",
        "Dark",
        "Hover",
        "Focus",
        "FocusVisible",
        "FocusWithin",
        "Active",
        "Disabled",
        "Visited",
        "Checked",
        "PlaceholderShown",
        "Selection",
        "Placeholder",
        "GroupHover",
        "GroupFocus",
        "PeerHover",
        "PeerFocus",
        "Bp(Sm)",
        "Min(Md)",
        "Max(Lg)",
        "Container(Xl)",
        "MaxContainer(Xl2)",
    ] {
        assert!(
            text.contains(&format!("aeolus_ir::Cond::{cond}")),
            "the emitted form carries {cond}: {text}"
        );
    }
}

#[test]
fn the_mask_is_derived_not_emitted() {
    // The emitted construction calls `Layer::new`, which recomputes the
    // mask — the emitter CANNOT drift the IR's mask invariant.
    let compiled = compile("p-4", &Config::default()).expect("compiles");
    let text = emit_static(&compiled.program);
    assert!(
        text.contains("aeolus_ir::Layer::new(") && !text.contains("mask"),
        "the mask rides Layer::new, never a literal: {text}"
    );
}

#[test]
fn non_finite_values_emit_as_constructors() {
    // The pipeline's finite guard refuses non-finite values, so the
    // emitter's non-finite arms are DEFENSIVE totality (a hand-built
    // program is the only way in): they emit the f32 constructors,
    // never `unreachable!`, never a NaN literal.
    let program = program_of(&[
        Decl {
            prop: Prop::PaddingTop,
            val: Val::Rem(f32::NAN),
        },
        Decl {
            prop: Prop::Width,
            val: Val::Pct(f32::INFINITY),
        },
        Decl {
            prop: Prop::Height,
            val: Val::Pct(f32::NEG_INFINITY),
        },
    ]);
    let text = emit_static(&program);
    assert!(text.contains("Val::Rem(f32::NAN)"), "{text}");
    assert!(text.contains("Val::Pct(f32::INFINITY)"), "{text}");
    assert!(text.contains("Val::Pct(f32::NEG_INFINITY)"), "{text}");
}

// ---------------------------------------------------------------------------
// 3. The arm: Target::Static and the compile-surface wiring
// ---------------------------------------------------------------------------

#[test]
fn the_static_arm_is_the_target_surface() {
    // The Target enum's second arm: the static-data form is an emit
    // artifact of the pipeline, named beside Css and Interp.
    let arms = [Target::Css, Target::Interp, Target::Static];
    assert_eq!(arms.len(), 3, "the plan §3.1 (b)/(c)/(d) arms are armed");
}

#[test]
fn compiled_convenience_emits_its_own_program() {
    let table = baked_table();
    let compiled = compile_theme("bg-primary", &table, &Config::default()).expect("compiles");
    // The ModePair slot rides as the IR carries it (the pair lives with
    // the baked theme; the static form carries the slot).
    let text: String = Compiled::emit_static(&compiled);
    assert!(
        text.contains("Val::ModePair("),
        "the baked tier's pair slot emits as the slot: {text}"
    );
}

#[test]
fn themed_tokens_emit_as_the_ir_carries_them() {
    // The runtime tier's Tok slot is static-form-legal: the value is
    // Copy, const-constructible — the static item carries the token
    // reference for a runtime table to resolve.
    let table = runtime_table();
    let compiled = compile_theme("bg-primary/50", &table, &Config::default()).expect("compiles");
    let text = emit_static(&compiled.program);
    assert!(
        text.contains("Val::Tok { id: ") && text.contains("alpha: 0.5 }"),
        "the token reference + alpha emit verbatim: {text}"
    );
}
