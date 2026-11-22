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
    // The shadow tier is NATIVE at the pinned rev (§3.7's shadow row,
    // "direct": GPUI's `box_shadow: Vec<BoxShadow>` carries the v4 band
    // shape — the walk composes through `aeolus_gpui::ShadowState`).
    assert!(supports(Prop::BoxShadow));
    assert!(supports(Prop::ShadowColor));
    assert!(!supports(Prop::Rotate));
}

#[test]
fn emit_refuses_unsupported_longhands() {
    // The transform family stays the reported boundary (GPUI's `Styled`
    // carries no transform slots — plan §3.7's transforms row).
    let compiled = compile_theme("rotate-45", &table(), &Config::default()).unwrap();
    let err = GpuiEmit::default().emit(&compiled.program).unwrap_err();
    assert!(err.iter().any(|entry| entry.prop == Prop::Rotate));
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
    let diagnostics: Diagnostics = lower_class("rotate-45", &Config::default()).unwrap_err();
    assert!(
        diagnostics
            .unsupported
            .iter()
            .any(|entry| entry.prop == Prop::Rotate)
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
fn the_filter_tier_splits_the_polyfill_half_from_the_spatial_refusal() {
    // The plan §3.7 filter row's GPUI polyfill half: the seven
    // colour-matrix longhands, the `drop-shadow-*` presets and the
    // bare `filter`/`filter-none` reset arm ride the POLYFILL tier
    // (the ring row's precedent — the walk composes them into the
    // element's own colour slots and GPUI's `Vec<BoxShadow>`), while
    // the SPATIAL half (`blur-*`) and the backdrop mirrors keep the
    // typed refusal (no GPUI slot; the CSS/interp arms carry the real
    // output).
    for prop in [
        Prop::FilterBrightness,
        Prop::FilterContrast,
        Prop::FilterGrayscale,
        Prop::FilterHueRotate,
        Prop::FilterInvert,
        Prop::FilterSaturate,
        Prop::FilterSepia,
        Prop::FilterDropShadow,
        Prop::Filter,
    ] {
        assert!(!supports(prop), "{prop:?} is the polyfill tier, not native");
        assert!(polyfilled(prop), "{prop:?} rides the polyfill tier");
        assert_eq!(
            aeolus_codegen_gpui::Caps.tier(prop),
            aeolus_codegen::conformance::Tier::Polyfill
        );
    }

    // The colour-matrix program emits where it used to refuse:
    let compiled = compile_theme(
        "brightness-110 contrast-125 grayscale",
        &table(),
        &Config::default(),
    )
    .unwrap();
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
    let tokens = lower_class("grayscale invert saturate-0", &Config::default())
        .expect("the matrix program emits through the public seam");
    assert!(
        tokens
            .to_string()
            .replace(' ', "")
            .contains("StaticProgram::new")
    );

    // The drop-shadow presets emit (the audited band texts the walk
    // parses) — with the keyword slice the macro freezes, the
    // annotation the band gate reads:
    let compiled = aeolus_compile::compile("drop-shadow-md", &Config::default()).unwrap();
    let classes = aeolus_compile::keyword_texts("drop-shadow-md", &Config::default()).unwrap();
    let keywords = aeolus_compile::keyword_alignment(&compiled, &classes);
    assert!(
        GpuiEmit::new(&keywords).emit(&compiled.program).is_ok(),
        "the drop-shadow preset emits"
    );
    // A keyword-less emit (no annotation slice) refuses the band gate
    // rather than guessing:
    let err = GpuiEmit::default().emit(&compiled.program).unwrap_err();
    assert!(
        err.iter().any(|entry| entry.prop == Prop::FilterDropShadow),
        "the annotation-less band refuses"
    );

    // The spatial + backdrop halves keep the typed refusal:
    for prop in [
        Prop::FilterBlur,
        Prop::BackdropBlur,
        Prop::BackdropBrightness,
        Prop::BackdropContrast,
        Prop::BackdropGrayscale,
        Prop::BackdropHueRotate,
        Prop::BackdropInvert,
        Prop::BackdropOpacity,
        Prop::BackdropSaturate,
        Prop::BackdropSepia,
        Prop::BackdropFilter,
    ] {
        assert!(!supports(prop), "{prop:?} refused");
        assert!(!polyfilled(prop), "{prop:?} refused");
        assert_eq!(
            aeolus_codegen_gpui::Caps.tier(prop),
            aeolus_codegen::conformance::Tier::Refused
        );
    }
    let compiled = compile_theme("blur-md brightness-110", &table(), &Config::default()).unwrap();
    let err = GpuiEmit::default().emit(&compiled.program).unwrap_err();
    assert!(
        err.iter().any(|entry| entry.prop == Prop::FilterBlur),
        "the blur longhand in the refusal report: {err:?}"
    );
    assert!(
        !err.iter().any(|entry| entry.prop == Prop::FilterBrightness),
        "the brightness longhand rides the polyfill tier: {err:?}"
    );
    let err = lower_class("blur-md", &Config::default()).unwrap_err();
    assert!(
        err.unsupported
            .iter()
            .any(|entry| entry.prop == Prop::FilterBlur)
    );
    let err = lower_class("backdrop-blur-md", &Config::default()).unwrap_err();
    assert!(
        err.unsupported
            .iter()
            .any(|entry| entry.prop == Prop::BackdropBlur)
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
    // The ring family, the filter tier's polyfill half AND the wave-1
    // between-child families are the whole polyfill set (the §3.7 ring
    // row + the filter row's landed half + the space/divide wrapper
    // row), in Prop::ALL's own discriminant order.
    assert_eq!(
        report.polyfill(),
        &[
            Prop::RingOffsetWidth,
            Prop::RingOffsetColor,
            Prop::RingColor,
            Prop::RingWidth,
            Prop::FilterBrightness,
            Prop::FilterContrast,
            Prop::FilterGrayscale,
            Prop::FilterHueRotate,
            Prop::FilterInvert,
            Prop::FilterSaturate,
            Prop::FilterSepia,
            Prop::FilterDropShadow,
            Prop::Filter,
            Prop::SpaceXReverse,
            Prop::SpaceX,
            Prop::SpaceYReverse,
            Prop::SpaceY,
            Prop::DivideXReverse,
            Prop::DivideYReverse,
            Prop::DivideStyle,
            Prop::DivideX,
            Prop::DivideY,
            Prop::DivideColor,
        ]
    );
}

#[test]
fn the_composite_values_stay_the_reported_gpui_boundary() {
    // The value pass's verbatim slot (the composite grammar): the sized
    // functions, the track lists and the image functions lower through
    // the pipeline's typed IR — and the GPUI lane refuses the decl per
    // value, the honest boundary the diagnostics render (the CSS arm
    // carries the verbatim rule text, the interp answers under any env).
    let err = lower_class("w-[calc(100%-2rem)]", &Config::default()).unwrap_err();
    assert!(
        err.unsupported
            .iter()
            .any(|entry| entry.prop == Prop::Width),
        "the verbatim width in the refusal report: {err:?}"
    );
    let err = lower_class("grid-cols-[auto_1fr_auto]", &Config::default()).unwrap_err();
    assert!(
        err.unsupported
            .iter()
            .any(|entry| entry.prop == Prop::GridTemplateColumns),
        "the verbatim track list in the refusal report: {err:?}"
    );

    // The per-decl gate refuses the program the same way (the val-keyed
    // verdict, the per-prop tiers beside it):
    let compiled = compile_theme("w-[calc(100%-2rem)]", &table(), &Config::default()).unwrap();
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let err = GpuiEmit::default().emit(&compiled.program).unwrap_err();
    assert!(
        err.iter().any(|entry| entry.prop == Prop::Width),
        "the verbatim value in the emit refusal: {err:?}"
    );

    // The eligible shapes never turned the PROPS themselves off: a
    // baked width still emits (the conformance ceiling is per-longhand,
    // the composite verdict is per-VALUE).
    let tokens = lower_class("w-32", &Config::default()).unwrap();
    assert!(!tokens.to_string().is_empty());
}

#[test]
fn the_content_longhand_stays_the_reported_gpui_boundary() {
    // The wider hint table's string half: `content-["…"]` lowers onto
    // the pipeline's NEW `Prop::Content` longhand (the CSS arm renders
    // the string, the interp answers) — and the GPUI lane refuses the
    // longhand per prop (no native slot at the pinned surface, the
    // default tier's reason), the honest boundary the diagnostics
    // render.
    let err = lower_class("content-[\"hello_world\"]", &Config::default()).unwrap_err();
    assert!(
        err.unsupported
            .iter()
            .any(|entry| entry.prop == Prop::Content),
        "the content longhand in the refusal report: {err:?}"
    );

    // The conformance tiers read the same verdict: Content is neither
    // native nor polyfilled — the partition still covers ALL exactly.
    assert!(!supports(Prop::Content), "content is not a GPUI slot");
    assert!(!polyfilled(Prop::Content), "content has no polyfill");
}
