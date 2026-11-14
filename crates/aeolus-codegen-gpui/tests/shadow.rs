//! The §3.7 shadow tier's GATE half (architecture plan §3.7's shadow row
//! — "`shadow-*` (multi-layer v4 presets) → `box_shadow: Vec<BoxShadow>`
//! → direct" — the tier the ring polyfill's landing left refused):
//! `supports`/`expressible` take the two shadow longhands, the per-decl
//! emit gate reads the frozen row texts, and the published conformance
//! report reads the same tables.
//!
//! The pins:
//!
//! * `supports()` — `box-shadow` (`Prop::BoxShadow`) and
//!   `--tw-shadow-color` (`Prop::ShadowColor`) are NATIVE (the pinned
//!   `gpui-pre` surface carries `box_shadow: Vec<BoxShadow>`, and the
//!   pinned `BoxShadow` struct itself carries the `inset` flag the
//!   `shadow-inner` band needs);
//! * the per-decl gate — the closed-world row texts the registry's
//!   shadow row lowers into (the `var(--tw-shadow-{name})` preset
//!   references, the `0 0 #0000` clear, the `inset 0 0 0 0 currentColor`
//!   placeholder) pass; an empty annotation (the bare-IR form) refuses
//!   with the text the gate has always named;
//! * `lower_class` — the third-party seam lowers the shadow families
//!   (RED first: the gate refused them before the flip);
//! * the audit — the macro gate's shape over a compiled shadow program:
//!   no `Unsupported` entries;
//! * the conformance report — the partition over `Prop::ALL` reads the
//!   same tables (the report is the emitter's verdicts, never a
//!   parallel claim), and the refused set shrinks by exactly the two.

use aeolus_codegen::conformance::Conformance;
use aeolus_codegen_gpui::{
    Caps, audit, emit_static_program, expressible, lower_class, polyfilled, supports,
    unsupported_reason,
};
use aeolus_compile::{Config, compile_theme, keyword_alignment, keyword_texts};
use aeolus_ir::{Program, Prop};
use aeolus_theme::{ThemeSource, TokenTable};

fn the_two() -> [Prop; 2] {
    [Prop::BoxShadow, Prop::ShadowColor]
}

/// The compiled shadow program plus its frozen row texts (the
/// annotation slice the emit gate's text-dependent verdicts read).
fn compiled_shadow_program(input: &str) -> (Program, Vec<&'static str>) {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::baked(&source).expect("the built-in bake");
    let compiled =
        compile_theme(input, &table, &Config::default()).expect("the shadow families lower");
    let annotations = keyword_texts(input, &Config::default()).expect("the row texts");
    let keywords = keyword_alignment(&compiled, &annotations);
    (compiled.program, keywords)
}

#[test]
fn the_two_shadow_longhands_are_native() {
    for prop in the_two() {
        assert!(
            supports(prop),
            "{prop:?} is a native slot at the pinned rev"
        );
        assert!(expressible(prop), "{prop:?} is expressible");
        assert!(!polyfilled(prop), "{prop:?} is native, not a polyfill");
    }
}

#[test]
fn the_closed_world_row_texts_pass_the_per_decl_gate() {
    // The registry's shadow row lowers exactly these texts (the
    // `SHADOW_PRESETS` table's var references and the two structural
    // forms; multi-token arbitrary lists refuse at compile, so no
    // other text reaches the gate).
    for text in [
        "var(--tw-shadow-2xs)",
        "var(--tw-shadow-xs)",
        "var(--tw-shadow-sm)",
        "var(--tw-shadow-md)",
        "var(--tw-shadow-lg)",
        "var(--tw-shadow-xl)",
        "var(--tw-shadow-2xl)",
        "0 0 #0000",
        "inset 0 0 0 0 currentColor",
    ] {
        assert_eq!(
            unsupported_reason(Prop::BoxShadow, text),
            None,
            "{text} composes into GPUI's BoxShadow list"
        );
    }
    // The bare-IR form (no annotation) keeps the annotation error the
    // gradient direction's bare form carries — the frozen text is the
    // recovery the bare `Val::Kw` slot needs.
    assert!(unsupported_reason(Prop::BoxShadow, "").is_some());
}

#[test]
fn the_audit_is_empty_for_the_shadow_families() {
    for input in [
        "shadow-2xs",
        "shadow-sm",
        "shadow-lg",
        "shadow-2xl",
        "shadow-none",
        "shadow-inner",
        "shadow-lg shadow-red-500",
        "shadow-lg shadow-red-500/50",
        "shadow-red-500",
    ] {
        let (program, keywords) = compiled_shadow_program(input);
        let unsupported = audit(&program, &keywords);
        assert!(
            unsupported.is_empty(),
            "{input} expresses on the GPUI target: {unsupported:?}"
        );
    }
}

#[test]
fn lower_class_accepts_the_shadow_families() {
    for input in ["shadow-lg", "shadow-2xl shadow-red-500/50", "shadow-none"] {
        let tokens = lower_class(input, &Config::default())
            .expect("the third-party seam lowers the shadow families");
        let rendered = tokens.to_string();
        assert!(
            rendered.contains("StaticProgram"),
            "the frozen program renders: {rendered}"
        );
    }
}

#[test]
fn the_frozen_emit_carries_the_shadow_declarations() {
    let (program, keywords) = compiled_shadow_program("shadow-lg");
    let tokens = emit_static_program(&program, &keywords).to_string();
    assert!(
        tokens.contains("BoxShadow"),
        "the frozen program carries the longhand: {tokens}"
    );
}

#[test]
fn the_conformance_partition_reads_the_same_tables() {
    let report = Conformance::of(&Caps);
    // The two longhands move refused → native: the tiers read the emit
    // gates' own tables, so the published score is the emitter's
    // verdict.
    for prop in the_two() {
        assert!(
            report.native().contains(&prop),
            "{prop:?} is native in the published partition"
        );
    }
    // The refused set shrinks by exactly the two (no other longhand
    // moved — the flip is the shadow tier's alone).
    for prop in the_two() {
        assert!(
            !report.refused().contains(&prop),
            "{prop:?} left the refused set"
        );
    }
    assert_eq!(
        report.total(),
        report.native().len() + report.polyfill().len() + report.refused().len(),
        "the partition stays total over Prop::ALL"
    );
}
