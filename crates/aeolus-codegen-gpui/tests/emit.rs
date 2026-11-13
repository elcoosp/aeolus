use aeolus_codegen::Emit;
use aeolus_codegen_gpui::{Diagnostics, GpuiEmit, lower_class, supports};
use aeolus_compile::{Config, compile_theme};
use aeolus_ir::Prop;
use aeolus_theme::{ThemeSource, TokenTable};

fn table() -> TokenTable {
    let source = ThemeSource::shadcn().unwrap();
    TokenTable::baked(&source).unwrap()
}

#[test]
fn supports_matches_the_runtime_mapping() {
    assert!(supports(Prop::PaddingTop));
    assert!(supports(Prop::Background));
    assert!(supports(Prop::Display));
    assert!(!supports(Prop::BoxShadow));
    assert!(!supports(Prop::Rotate));
}

#[test]
fn emit_refuses_unsupported_longhands() {
    let compiled = compile_theme("shadow-md", &table(), &Config::default()).unwrap();
    let err = GpuiEmit::default().emit(&compiled.program).unwrap_err();
    assert!(err.iter().any(|entry| entry.prop == Prop::BoxShadow));
}

#[test]
fn emit_renders_a_static_program_expression() {
    let compiled = compile_theme("p-4 bg-primary", &table(), &Config::default()).unwrap();
    let tokens = GpuiEmit::default()
        .emit(&compiled.program)
        .unwrap()
        .to_string();
    let compact = tokens.replace(' ', "");
    assert!(compact.contains("StaticProgram::new"));
    assert!(compact.contains("DECLS_0"));
}

#[test]
fn lower_class_is_the_public_seam() {
    let tokens = lower_class("p-4 bg-primary", &Config::default()).unwrap();
    assert!(
        tokens
            .to_string()
            .replace(' ', "")
            .contains("StaticProgram::new")
    );
    let diagnostics: Diagnostics = lower_class("shadow-md", &Config::default()).unwrap_err();
    assert!(
        diagnostics
            .unsupported
            .iter()
            .any(|entry| entry.prop == Prop::BoxShadow)
    );
}

#[test]
fn the_ring_family_rides_the_polyfill_tier() {
    // §3.7's ring row: no native slot, but the runtime composes the v4
    // box-shadow list (aeolus_gpui::RingState) — the emit gates accept
    // the ring longhands, the truly-native table stays honest.
    assert!(!supports(Prop::RingWidth));
    assert!(aeolus_codegen_gpui::polyfilled(Prop::RingWidth));
    assert!(aeolus_codegen_gpui::polyfilled(Prop::RingColor));
    assert!(aeolus_codegen_gpui::polyfilled(Prop::RingOffsetWidth));
    assert!(aeolus_codegen_gpui::polyfilled(Prop::RingOffsetColor));
    assert!(aeolus_codegen_gpui::expressible(Prop::RingWidth));
    // The ring program emits where it used to refuse.
    let compiled = compile_theme("ring-2 ring-red-500", &table(), &Config::default()).unwrap();
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let tokens = GpuiEmit::default().emit(&compiled.program).unwrap();
    assert!(
        tokens
            .to_string()
            .replace(' ', "")
            .contains("StaticProgram::new")
    );
    let tokens = lower_class(
        "ring-2 ring-red-500 ring-offset-2 ring-offset-white",
        &Config::default(),
    )
    .expect("the ring program emits through the public seam");
    assert!(
        tokens
            .to_string()
            .replace(' ', "")
            .contains("StaticProgram::new")
    );
}
