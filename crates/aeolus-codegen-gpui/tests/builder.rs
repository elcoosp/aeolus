//! The builder-chain emitter's contract (TDD: written FIRST — the RED
//! spec fails to compile against the not-yet-existing seam).
//!
//! Plan fidelity: the pinned acceptance criterion is the V1
//! architecture plan's emit (a) landing note
//! (`docs/AEOLUS_V1_ARCHITECTURE.md`, §3.2 status, the rust-tokens
//! paragraph): "The backend METHOD calls the emit (a) end form implies
//! (`FlexDirection::Column`, builder chains) remain the P3 codegen
//! crate's surface." The same deferral stands in the §8 P2 row ("the
//! backend method calls staying the P3 codegen's surface") and in
//! `reports/10-tdd-evidence/19-rust-tokens-arm.md`'s boundary list.
//! These pins capture it: the emitter renders one compiled program as
//! the GPUI builder-call chain the walk would apply — the `Emit`
//! trait's own documented shape ("renders the target's builder calls")
//! — refusing, per declaration and per condition, everything the chain
//! form cannot carry (the frozen-program lanes keep the real output).
//!
//! The differential target: for a string the chain CAN express, the
//! emitted method/argument sequence mirrors `aeolus_gpui`'s walk arms
//! (`apply_decl` and its helpers) one for one — the parity battery in
//! `crates/aeolus-gpui/tests/builder_parity.rs` pins the applied
//! `StyleRefinement` field-by-field against the walk's answer.

use aeolus_codegen::Emit;
use aeolus_codegen_gpui::{Diagnostics, GpuiBuilderEmit, lower_class_builder};
use aeolus_compile::{Config, compile_theme};
use aeolus_ir::Prop;
use aeolus_theme::{ThemeSource, TokenTable};

fn table() -> TokenTable {
    let source = ThemeSource::shadcn().unwrap();
    TokenTable::baked(&source).unwrap()
}

fn runtime_table() -> TokenTable {
    let source = ThemeSource::shadcn().unwrap();
    TokenTable::runtime(&source).unwrap()
}

fn builder_tokens(literal: &str) -> String {
    let compiled = compile_theme(literal, &table(), &Config::default()).unwrap();
    // The annotation slice: the row texts the keyword-slot resolution
    // reads (the same alignment the macro surface freezes).
    let annotations = aeolus_compile::keyword_texts(literal, &Config::default()).unwrap();
    let keywords = aeolus_compile::keyword_alignment(&compiled, &annotations);
    GpuiBuilderEmit::new(&keywords)
        .emit(&compiled.program)
        .unwrap()
        .to_string()
}

fn compact(tokens: &str) -> String {
    tokens.replace(' ', "")
}

#[test]
fn the_button_string_renders_the_walks_method_calls() {
    // The plan §3.6 surface (1)'s own example string — every
    // declaration chain-expressible under the baked shadcn table: the
    // display/flex-direction/align keyword slots, the spacing scale,
    // the radii and the baked colour literals. The compacted text
    // carries each walk arm's method call.
    let tokens = builder_tokens(
        "flex flex-col items-center gap-2 p-4 rounded-md bg-primary text-primary-foreground",
    );
    let c = compact(&tokens);
    // The keyword slots render the walk helper's pinned methods.
    assert!(c.contains(".flex()"), "{c}");
    assert!(c.contains(".flex_col()"), "{c}");
    assert!(c.contains(".items_center()"), "{c}");
    // The spacing scale renders px-scale rems (p-4 → 1.0rem).
    assert!(c.contains(".pt("), "{c}");
    assert!(c.contains("rems(1.0)"), "{c}");
    // The gap family rides its two longhands.
    assert!(c.contains(".gap_x("), "{c}");
    assert!(c.contains(".gap_y("), "{c}");
    // The radii fan out per corner.
    assert!(c.contains(".rounded_tl("), "{c}");
    // The baked colours render as rgba literals (the walk's
    // `rgba(u32::from_be_bytes(..))` arm).
    assert!(c.contains(".bg("), "{c}");
    assert!(c.contains("rgba(0x"), "{c}");
    assert!(c.contains(".text_color("), "{c}");
    // The whole output is ONE closure over a styled receiver.
    assert!(c.starts_with("|s|"), "{c}");
}

#[test]
fn the_keyword_slots_render_their_walk_methods() {
    // One representative per keyword family the walk maps (the
    // `apply_decl` arms): position, justify, overflow, cursor,
    // decoration line, white-space, text-align.
    let tokens = builder_tokens(
        "relative justify-between overflow-hidden cursor-pointer underline whitespace-nowrap text-center",
    );
    let c = compact(&tokens);
    assert!(c.contains(".relative()"), "{c}");
    assert!(c.contains(".justify_between()"), "{c}");
    assert!(c.contains(".overflow_x_hidden()"), "{c}");
    assert!(c.contains(".cursor_pointer()"), "{c}");
    assert!(c.contains(".underline()"), "{c}");
    assert!(c.contains(".whitespace_nowrap()"), "{c}");
    // The text-align value rides the walk's own pub mapping (one
    // source of truth for the LTR fold).
    assert!(
        c.contains("text_align(::aeolus_gpui::text_align_value(\"center\"))"),
        "{c}"
    );
}

#[test]
fn the_numeric_and_length_slots_render_their_value_forms() {
    // opacity-50 → the walk's /100 reading; w-full → the row text's
    // 100% fraction; border-2 → the bare-integer px scale;
    // line-clamp-2 → the usize count; font-medium → the weight number.
    let tokens = builder_tokens("opacity-50 w-full border-2 line-clamp-2 font-medium");
    let c = compact(&tokens);
    assert!(c.contains(".opacity(0.5)"), "{c}");
    assert!(c.contains(".w("), "{c}");
    assert!(c.contains("relative(1.0)"), "{c}");
    assert!(c.contains(".border_t("), "{c}");
    assert!(c.contains("px(2.0)"), "{c}");
    assert!(c.contains(".line_clamp(2)"), "{c}");
    assert!(c.contains(".font_weight("), "{c}");
    assert!(c.contains("FontWeight::from(500.0)"), "{c}");
}

#[test]
fn the_state_variants_render_their_native_methods() {
    // §3.5's GPUI mapping column: hover/active/focus/focus-visible are
    // the native `.hover/.active/.focus/.focus_visible(|s| …)` state
    // methods — the chain renders each conditional layer through its
    // own closure, the base cascade staying the outer chain.
    let tokens =
        builder_tokens("p-4 hover:bg-red-500 focus:p-2 active:opacity-90 focus-visible:underline");
    let c = compact(&tokens);
    assert!(c.contains(".pt("), "{c}");
    assert!(c.contains(".hover(|s|"), "{c}");
    assert!(c.contains(".focus(|s|"), "{c}");
    assert!(c.contains(".focus_visible(|s|"), "{c}");
    assert!(c.contains(".active(|s|"), "{c}");
    assert!(c.contains(".bg("), "{c}");
    assert!(c.contains(".underline()"), "{c}");
}

#[test]
fn the_emitter_is_the_emit_trait_impl() {
    // The seam rides the backend-neutral `Emit` trait (§3.2's row: one
    // trait, the macros and third parties call it).
    let compiled = compile_theme("p-4", &table(), &Config::default()).unwrap();
    let tokens = Emit::emit(&GpuiBuilderEmit::default(), &compiled.program).unwrap();
    assert!(compact(&tokens.to_string()).contains(".pt("));
}

#[test]
fn lower_class_builder_is_the_public_seam() {
    // `lower_class`'s twin: the §5 integration bullet's resolve_style()
    // equivalent — one literal in, the builder-call tokens out, the
    // typed refusals in the same `Diagnostics` shape.
    let tokens = lower_class_builder("p-4", &Config::default()).unwrap();
    assert!(compact(&tokens.to_string()).contains(".pt("));
    let diagnostics: Diagnostics =
        lower_class_builder("rotate-45", &Config::default()).unwrap_err();
    assert!(
        diagnostics
            .unsupported
            .iter()
            .any(|entry| entry.prop == Prop::Rotate),
        "{diagnostics:?}"
    );
}

#[test]
fn the_theme_token_slots_refuse_typed() {
    // The chain form carries compile-time values only: the T2 tier's
    // runtime table lowers colour tokens to `Val::Tok`/`Val::ModePair`
    // slots — the walk's TokenTable lane resolves them under the frame
    // env, the chain refuses.
    let compiled = compile_theme("bg-primary", &runtime_table(), &Config::default()).unwrap();
    let err = GpuiBuilderEmit::default()
        .emit(&compiled.program)
        .unwrap_err();
    assert!(
        err.iter()
            .any(|entry| { entry.prop == Prop::Background && entry.reason.contains("theme") }),
        "{err:?}"
    );
}

#[test]
fn the_state_accumulator_families_refuse_typed() {
    // The walk expresses the shadow/ring/gradient/filter families
    // through its state accumulators (`ShadowState`/`RingState`/
    // `GradState`/`FilterState`) — no plain builder method carries
    // them, so the chain form refuses with the lane's name.
    let compiled = compile_theme(
        "shadow-md ring-2 bg-linear-to-r backdrop-blur-md space-x-2",
        &table(),
        &Config::default(),
    )
    .unwrap();
    let err = GpuiBuilderEmit::default()
        .emit(&compiled.program)
        .unwrap_err();
    let props: Vec<Prop> = err.iter().map(|entry| entry.prop).collect();
    assert!(props.contains(&Prop::BoxShadow), "{props:?}");
    assert!(props.contains(&Prop::RingWidth), "{props:?}");
    assert!(props.contains(&Prop::BackgroundImage), "{props:?}");
    assert!(props.contains(&Prop::BackdropBlur), "{props:?}");
    // The space/divide child-selector composite rides its own walk
    // state (`TwSpaceExt`), not a builder method.
    assert!(props.contains(&Prop::SpaceX), "{props:?}");
}

#[test]
fn the_non_native_cond_families_refuse_typed() {
    // The four native state methods carry their §3.5 row; every other
    // conditional family (dark, breakpoints, group-*, data-*, the
    // probe seams) rides the frozen-program walk and refuses here.
    let compiled = compile_theme(
        "dark:bg-red-500 md:p-6 group-hover:opacity-90 data-open:p-2",
        &table(),
        &Config::default(),
    )
    .unwrap();
    let err = GpuiBuilderEmit::default()
        .emit(&compiled.program)
        .unwrap_err();
    assert!(!err.is_empty(), "the conditional layers refuse typed");
    assert!(
        err.iter().all(|entry| entry.reason.contains("conditional")),
        "{err:?}"
    );
}

#[test]
fn the_refusal_is_all_or_nothing() {
    // The honest-failure contract reaches the chain form: one refused
    // declaration refuses the WHOLE emit (never a partial chain).
    let compiled = compile_theme("p-4 rotate-45", &table(), &Config::default()).unwrap();
    assert!(GpuiBuilderEmit::default().emit(&compiled.program).is_err());
}

#[test]
fn the_emit_is_deterministic() {
    // The static emit's determinism contract reaches the chain form:
    // the same program renders byte-identical tokens.
    let a = builder_tokens("p-4 bg-primary");
    let b = builder_tokens("p-4 bg-primary");
    assert_eq!(a, b);
}

#[test]
fn the_cascade_folds_per_condition_last_wins() {
    // Two same-condition layers fold the way the walk applies them:
    // the later declaration's method call overwrites the earlier
    // (last-wins per longhand, the cascade's own reading).
    let tokens = builder_tokens("p-4 p-8");
    let c = compact(&tokens);
    assert!(c.contains("rems(2.0)"), "{c}");
    assert!(c.contains("rems(1.0)"), "{c}");
    // p-8 renders after p-4 — the closure applies them in order.
    let p8 = c.find("rems(2.0)").unwrap();
    let p4 = c.find("rems(1.0)").unwrap();
    assert!(p4 < p8, "the later declaration renders later: {c}");
}
