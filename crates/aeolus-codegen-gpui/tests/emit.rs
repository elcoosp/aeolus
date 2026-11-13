use aeolus_codegen::Emit;
use aeolus_codegen::conformance::Capability as _;
use aeolus_codegen_gpui::{Diagnostics, GpuiEmit, lower_class, polyfilled, supports};
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

#[test]
fn the_motion_layer_stays_the_reported_gpui_boundary() {
    // The plan §3.7 motion row: GPUI's style surface carries no
    // transition slot and `with_animation` needs an element id + state
    // the frozen static program does not name — the five motion
    // longhands refuse per longhand, the honest boundary the diagnostics
    // render (the CSS/interp/plan arms carry the real output).
    assert!(!supports(Prop::TransitionProperty));
    assert!(!supports(Prop::TransitionDuration));
    assert!(!supports(Prop::TransitionTimingFunction));
    assert!(!supports(Prop::TransitionDelay));
    assert!(!supports(Prop::Animation));
    assert!(!aeolus_codegen_gpui::polyfilled(Prop::TransitionProperty));
    assert!(!aeolus_codegen_gpui::polyfilled(Prop::Animation));

    // The emit gate refuses the program, naming each motion longhand:
    let compiled = compile_theme(
        "transition-colors duration-300 animate-spin",
        &table(),
        &Config::default(),
    )
    .unwrap();
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let err = GpuiEmit::default().emit(&compiled.program).unwrap_err();
    for prop in [
        Prop::TransitionProperty,
        Prop::TransitionDuration,
        Prop::Animation,
    ] {
        assert!(
            err.iter().any(|entry| entry.prop == prop),
            "{prop:?} missing from the refusal report: {err:?}"
        );
    }

    // The third-party seam reports the same refusals as diagnostics:
    let err = lower_class("transition-opacity ease-in", &Config::default()).unwrap_err();
    assert!(
        err.unsupported
            .iter()
            .any(|entry| entry.prop == Prop::TransitionProperty)
    );
    assert!(
        err.unsupported
            .iter()
            .any(|entry| entry.prop == Prop::TransitionTimingFunction)
    );
}

#[test]
fn the_conformance_seam_reads_the_capability_tables() {
    // The §5.1 conformance seam: the GPUI target's `Capability` impl
    // reads the SAME `supports`/`polyfilled` tables the emit gates run,
    // so the published score can never drift from the gate's verdicts.
    let caps = aeolus_codegen_gpui::Caps;
    assert_eq!(caps.name(), "gpui");
    // Spot checks against the predicates themselves (one per tier):
    assert_eq!(
        caps.tier(Prop::PaddingTop),
        aeolus_codegen::conformance::Tier::Native
    );
    assert_eq!(
        caps.tier(Prop::RingWidth),
        aeolus_codegen::conformance::Tier::Polyfill
    );
    assert_eq!(
        caps.tier(Prop::TransitionProperty),
        aeolus_codegen::conformance::Tier::Refused
    );
    assert_eq!(
        caps.tier(Prop::Animation),
        aeolus_codegen::conformance::Tier::Refused
    );

    // The report's partition is the predicates' own partition:
    let report = aeolus_codegen::conformance::Conformance::of(&caps);
    assert_eq!(
        report.total(),
        Prop::ALL.len(),
        "the tiers partition Prop::ALL exactly"
    );
    for &prop in report.refused() {
        assert!(
            !supports(prop),
            "{prop:?} refused in the report but supports() takes it"
        );
        assert!(!polyfilled(prop));
    }
    for &prop in report.native() {
        assert!(supports(prop));
    }
    for &prop in report.polyfill() {
        assert!(!supports(prop));
        assert!(polyfilled(prop));
    }
    // The ring family is the whole polyfill tier (the §3.7 ring row).
    // The ring family is the whole polyfill tier (the §3.7 ring row),
    // in Prop::ALL's own discriminant order.
    assert_eq!(
        report.polyfill(),
        &[
            Prop::RingOffsetWidth,
            Prop::RingOffsetColor,
            Prop::RingColor,
            Prop::RingWidth,
        ]
    );
}
