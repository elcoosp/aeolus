//! The slint backend's V1 static-program lane (architecture plan §7's
//! re-target — the sixth backend on the lane after GPUI, Floem, Xilem,
//! Iced and egui): a `tw!`-compiled [`StaticProgram`] resolves through
//! the pipeline's shared walk into the crate's own [`SlintStyle`] under
//! one probe [`Env`], and the result must AGREE with the plan path —
//! the same class string through the plan bridge and the
//! [`StyleBook`](aeolus_slint::StyleBook) resolution the crate has
//! lowered since v0.1. Two lanes, one compiler, one cascade.
//!
//! The differential pins compare the walk's Env-folded style against
//! the plan path's per-probe resolution — the same effective element
//! properties a running Slint app would set, from two different
//! program forms.

use aeolus_compile::{Config, compile_theme, emit_plan_theme, keyword_alignment, keyword_texts};
use aeolus_rt::{AttrBit, AttrKind, AttrState, Env, StaticProgram, TokenSource};
use aeolus_slint::{
    LoweringReport, REM_PX, SlintProbe, SlintStyle, env_for_probe, lower_program,
    lower_program_with, lower_resolved,
};
use aeolus_theme::{ThemeSource, TokenTable};
use i_slint_core::graphics::{Brush, Color};

/// The baked shadcn LIGHT table (T0's shape: every colour token bakes
/// to a literal) — the same construction the `tw!` macro's freeze walk
/// runs.
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the baked shadcn table builds")
}

/// Compile a class string the way `tw!` does and freeze it into the
/// static form the macro emits: the interned layers plus the row texts
/// the keyword slots index.
fn frozen(classes: &str) -> StaticProgram {
    let config = Config::default();
    let compiled = compile_theme(classes, &baked_table(), &config).expect("the class lowers");
    assert!(
        compiled.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        compiled.diagnostics
    );
    let annotations = keyword_texts(classes, &config).expect("non-empty input annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    StaticProgram::new(
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    )
}

/// The plan-bridge plan for the SAME class string — the v0.x plan the
/// crate's [`StyleBook`](aeolus_slint::StyleBook) resolves per state.
fn bridged(classes: &str) -> aeolus_core::StylePlan {
    let sheet = emit_plan_theme(classes, &baked_table(), &Config::default())
        .expect("the plan bridge lowers");
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    sheet.plan
}

/// The plan path's own lowering: bridge plan → per-probe resolve → the
/// crate's `lower_resolved` — exactly the resolution
/// [`StyleBook::resolve_at`](aeolus_slint::StyleBook::resolve_at) runs,
/// minus the book.
fn planned(classes: &str, probe: &SlintProbe) -> SlintStyle {
    let plan = bridged(classes);
    let resolved = plan.resolve(probe);
    let mut report = LoweringReport::default();
    lower_resolved(&resolved, &mut report)
}

/// bg-primary under the shadcn light palette: oklch(0.205 0 0) ≈
/// #171717 — the constant the plan-bridge batteries pin. Slint colours
/// are straight-alpha sRGB bytes (`0xAARRGGBB`).
const PRIMARY: [u8; 4] = [23, 23, 23, 255];

/// The straight-alpha bytes a Slint [`Color`] carries, for the
/// quantization-tolerant colour reads (both lanes convert through the
/// audited parser; the Oklab equality epsilon is far below one 8-bit
/// step).
fn rgba(c: Color) -> [u8; 4] {
    let argb = c.as_argb_encoded();
    [
        ((argb >> 16) & 0xFF) as u8,
        ((argb >> 8) & 0xFF) as u8,
        (argb & 0xFF) as u8,
        ((argb >> 24) & 0xFF) as u8,
    ]
}

/// The solid fill of the style's background slot, when it is one.
fn fill_of(style: &SlintStyle) -> Option<[u8; 4]> {
    match &style.background {
        Some(Brush::SolidColor(c)) => Some(rgba(*c)),
        _ => None,
    }
}

/// Byte-wise ±1 comparison — the Oklab equality epsilon is far below one
/// 8-bit step, and both lanes bake through the audited parser.
fn close(a: [u8; 4], b: [u8; 4]) -> bool {
    a.iter()
        .zip(b.iter())
        .all(|(x, y)| i16::from(*x) - i16::from(*y) <= 1 && i16::from(*y) - i16::from(*x) <= 1)
}

#[test]
fn the_base_layer_lowers_from_the_static_program() {
    let program = frozen("p-4 bg-primary text-primary-foreground rounded-md");
    let (style, report) = lower_program(&program, &Env::default());
    assert!(report.is_empty(), "unexpected drops: {report:?}");

    // p-4 = 1rem = 16px under the backend's REM_PX constant (CSS
    // padding is the layout hint the crate lowers to per-edge px).
    let padding = style.padding.expect("p-4 lowers into the padding hint");
    assert_eq!(padding.top, 16.0);
    assert_eq!(padding.left, 16.0);

    let fill = fill_of(&style).expect("bg-primary lowers into a solid brush");
    for (got, want) in fill.iter().zip(PRIMARY.iter()) {
        assert!(i16::from(*got) - i16::from(*want) <= 1, "got {fill:?}");
    }

    // rounded-md = 0.375rem = 6px (the registry row's v4 default; the
    // shadcn-remapped scale is the THEME's answer, not the row's).
    let radius = style.border_radius.expect("rounded-md lowers natively");
    assert_eq!(radius.top_left, 6.0);

    // text-primary-foreground rides Prop::Color → the v0.x
    // content_color slot → the Text item's colour: the pin that keeps
    // the lane's text surface non-vacuous (primary-foreground light =
    // oklch(0.985 0 0) ≈ #fafafa).
    let text = style
        .text
        .color
        .expect("text-primary-foreground lowers into the Text colour");
    let fill = rgba(match text {
        Brush::SolidColor(c) => c,
        other => panic!("expected a solid text brush, got {other:?}"),
    });
    for (got, want) in fill.iter().zip([250u8, 250, 250, 255].iter()) {
        assert!(i16::from(*got) - i16::from(*want) <= 1, "got {fill:?}");
    }
}

#[test]
fn rem_bakes_through_the_backend_constant() {
    // Plan §9's per-backend rem_px constant: the lane's own REM_PX is
    // the 16px bake the v0.x resolver's spacing scale applies, so the
    // two lanes agree on every spacing class.
    assert_eq!(REM_PX, 16.0);
    let program = frozen("p-2 w-24");
    let (style, _) = lower_program(&program, &Env::default());
    assert_eq!(style.padding.expect("p-2 lowers").top, 8.0);
    assert_eq!(
        style.width,
        Some(aeolus_slint::SlintLength::Px(96.0)),
        "w-24 = 6rem = 96px"
    );
}

#[test]
fn state_layers_walk_under_env_for_probe() {
    let program = frozen("bg-primary hover:bg-red-500 disabled:opacity-50");

    // The crate's own conversion: SlintProbe → the probe Env the lane
    // walks under.
    let idle_env = env_for_probe(&SlintProbe::default());
    let (idle, _) = lower_program(&program, &idle_env);
    let idle_fill = fill_of(&idle).expect("the base fill");
    assert!(close(idle_fill, PRIMARY), "idle: {idle_fill:?}");

    let hover_env = env_for_probe(&SlintProbe {
        has_hover: true,
        ..SlintProbe::default()
    });
    let (hovered, _) = lower_program(&program, &hover_env);
    // red-500 = oklch(0.637 0.237 25.331), gamut-mapped by the audited
    // bake to #fb2c36.
    let fill = fill_of(&hovered).expect("the hover fill");
    assert!(
        close(fill, [251, 44, 54, 255]),
        "hover:bg-red-500 fires under the probe: {fill:?}"
    );

    let disabled_env = env_for_probe(&SlintProbe {
        enabled: false,
        ..SlintProbe::default()
    });
    let (disabled, _) = lower_program(&program, &disabled_env);
    // Slint has a NATIVE opacity property (unlike egui's baked alpha):
    // the disabled layer lands as the 0.5 factor, the fill stays.
    assert_eq!(
        disabled.opacity,
        Some(0.5),
        "disabled:opacity-50 lowers into the opacity property"
    );
    let fill = fill_of(&disabled).expect("the disabled fill stays opaque");
    assert!(close(fill, PRIMARY), "the fill is untouched: {fill:?}");
}

#[test]
fn env_for_probe_maps_the_slint_probe() {
    let env = env_for_probe(&SlintProbe {
        dark: true,
        has_focus: true,
        checked: true,
        width: Some(800.0),
        container_width: Some(400.0),
        ..SlintProbe::default()
    });
    assert!(env.dark, "the probe's dark flag rides");
    // Slint's `has-focus` carries no focus-visible distinction (the
    // TouchArea/FocusScope property is one bool) — the focused flag
    // feeds BOTH focus conditions, the plan path's documented
    // normalization (SlintProbe::test maps State::Focus and
    // State::FocusVisible onto the same bool).
    assert!(env.focus && env.focus_visible);
    assert!(env.checked, "the probe's checked flag rides");
    assert!((env.viewport_rem - 50.0).abs() < 1e-4, "800px / 16 = 50rem");
    assert_eq!(env.container_rem, Some(25.0), "400px / 16 = 25rem");
    // The probe cannot answer group/peer/visited/placeholder — they
    // stay off (CSS: a selector that does not match).
    assert!(!env.group_hover && !env.peer_focus && !env.visited && !env.placeholder_shown);
}

#[test]
fn the_lane_agrees_with_the_plan_path_per_state() {
    // The shadcn button corpus (minus `gap-2` — the layout hint is the
    // slint backend's own REPORTED boundary, not a lane disagreement)
    // — every mapped slot must agree per probe combination, the two
    // resolution paths of one class string.
    let corpus = "p-4 bg-primary text-primary-foreground rounded-md \
                  border-2 border-primary w-24 h-9 font-medium \
                  hover:bg-red-500 active:opacity-60";
    let program = frozen(corpus);

    for probe in &[
        SlintProbe::default(),
        SlintProbe {
            has_hover: true,
            ..SlintProbe::default()
        },
        SlintProbe {
            pressed: true,
            ..SlintProbe::default()
        },
        SlintProbe {
            enabled: false,
            ..SlintProbe::default()
        },
        SlintProbe {
            has_hover: true,
            pressed: true,
            ..SlintProbe::default()
        },
    ] {
        let env = env_for_probe(probe);
        let (walked, report) = lower_program(&program, &env);
        assert!(
            report.is_empty(),
            "the corpus fully maps on both lanes: {report:?}"
        );
        let planned_style = planned(corpus, probe);

        assert_eq!(
            walked.padding, planned_style.padding,
            "padding hint @ {probe:?}"
        );
        assert_eq!(
            walked.background, planned_style.background,
            "background brush @ {probe:?}"
        );
        assert_eq!(
            walked.text.color, planned_style.text.color,
            "text colour @ {probe:?}"
        );
        assert_eq!(
            walked.border_radius, planned_style.border_radius,
            "radius @ {probe:?}"
        );
        assert_eq!(
            walked.border_width, planned_style.border_width,
            "border width @ {probe:?}"
        );
        assert_eq!(
            walked.border_color, planned_style.border_color,
            "border colour @ {probe:?}"
        );
        assert_eq!(walked.width, planned_style.width, "width @ {probe:?}");
        assert_eq!(walked.height, planned_style.height, "height @ {probe:?}");
        assert_eq!(
            walked.text.font_weight, planned_style.text.font_weight,
            "font weight @ {probe:?}"
        );
        assert_eq!(walked.opacity, planned_style.opacity, "opacity @ {probe:?}");
    }
}

#[test]
fn the_lane_agrees_with_the_plan_path_under_dark_and_wide() {
    let corpus = "p-4 md:p-6 dark:bg-white bg-primary";
    let program = frozen(corpus);

    // Wide viewport: the responsive bucket's padding — the walk under
    // the env's rem, the plan path through the probe's width.
    let wide = SlintProbe {
        width: Some(800.0),
        ..SlintProbe::default()
    };
    let (walked, _) = lower_program(&program, &env_for_probe(&wide));
    assert_eq!(
        walked.padding.expect("md:p-6 at 50rem").left,
        24.0,
        "md:p-6 = 1.5rem = 24px"
    );

    // Dark mode: the dark layer's fill, both lanes.
    let dark = SlintProbe {
        dark: true,
        ..SlintProbe::default()
    };
    let (walked, _) = lower_program(&program, &env_for_probe(&dark));
    let planned_style = planned(corpus, &dark);
    assert_eq!(
        walked.background, planned_style.background,
        "dark fill agrees"
    );
}

#[test]
fn the_book_and_the_lane_agree_on_the_probe_path() {
    // The StyleBook the plan path ships and the program lane must paint
    // the same payload for the same probe situation — the two
    // consumption paths of one class string.
    let corpus = "p-4 bg-primary text-primary-foreground rounded-md hover:bg-red-500";
    let program = frozen(corpus);
    let plan = bridged(corpus);
    let (book, book_report) = aeolus_slint::StyleBook::new(&plan);
    assert!(
        book_report.is_empty(),
        "the corpus fully maps: {book_report:?}"
    );

    let dark_probe = SlintProbe {
        dark: true,
        ..SlintProbe::default()
    };
    let (walked, _) = lower_program(&program, &env_for_probe(&dark_probe));
    let book_style = book.resolve_at(&dark_probe);
    assert_eq!(
        walked.background, book_style.background,
        "the book's dark payload matches the walk"
    );
}

#[test]
fn token_slots_refuse_without_a_source_and_resolve_through_one() {
    // The RUNTIME table (T2's shape): bg-primary lowers to a token slot
    // the frozen program carries verbatim — the host resolves it per
    // walk through the TokenSource seam.
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::runtime(&source).expect("the runtime table builds");
    let config = Config::default();
    let compiled = compile_theme("bg-primary/90", &table, &config).expect("the class lowers");
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let annotations = keyword_texts("bg-primary/90", &config).expect("annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    let program = StaticProgram::new(
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    );

    // No source: the themed longhand refuses into the backend report —
    // never a silent transparent background.
    let (style, report) = lower_program(&program, &Env::default());
    assert!(
        style.background.is_none(),
        "a token slot without a source must not render a colour"
    );
    assert!(!report.is_empty(), "the refusal is reported: {report:?}");

    // With the table as the token source: the T2 one-load-one-multiply
    // answers, the /90 alpha premultiplied onto the colour.
    let (style, report) = lower_program_with(&program, &Env::default(), None, Some(&table));
    assert!(report.is_empty(), "unexpected drops: {report:?}");
    let fill = fill_of(&style).expect("the token resolved into a solid brush");
    assert!(
        (f32::from(fill[3]) - 255.0 * 0.9).abs() <= 2.0,
        "channels unchanged, alpha scaled: got {fill:?}"
    );
}

#[test]
fn attr_probe_layers_fire_through_the_seam() {
    let program = frozen("bg-primary data-[state=open]:bg-red-500");

    // No probe source: the layer is CSS-quiet AND reported (the interp's
    // AttributeUnqueried note, carried on the backend report channel).
    let (idle, report) = lower_program(&program, &Env::default());
    let idle_fill = fill_of(&idle).expect("the base fill");
    assert!(close(idle_fill, PRIMARY));
    assert!(!report.is_empty(), "the unqueried layer is reported");

    // The answered miss is CSS-quiet — and NOT a note.
    let closed = AttrState::new(&[AttrBit {
        kind: AttrKind::Data,
        name: "state",
        value: "closed",
    }]);
    let (_, report) = lower_program_with(&program, &Env::default(), Some(&closed), None);
    assert!(report.is_empty(), "an answered miss is quiet: {report:?}");

    // The answered hit fires.
    let open = AttrState::new(&[AttrBit {
        kind: AttrKind::Data,
        name: "state",
        value: "open",
    }]);
    let (fired, _) = lower_program_with(&program, &Env::default(), Some(&open), None);
    let fill = fill_of(&fired).expect("the fired fill");
    assert!(
        close(fill, [251, 44, 54, 255]),
        "data-[state=open]:bg-red-500 fires through the seam: {fill:?}"
    );
}

#[test]
fn unmapped_longhands_are_reported_never_dropped() {
    // shadow-sm lowers into the IR (Prop::BoxShadow, the preset's var
    // text) but the walk's slot writer reports the preset family — the
    // report names it.
    let program = frozen("shadow-sm bg-primary");
    let (style, report) = lower_program(&program, &Env::default());
    let fill = fill_of(&style).expect("the mapped longhand lands");
    assert!(close(fill, PRIMARY), "the mapped longhand lands");
    assert!(!report.is_empty(), "the unmapped longhand is reported");
    assert!(
        report
            .dropped
            .iter()
            .any(|(what, _)| what.contains("box-shadow")),
        "the report names the longhand: {:?}",
        report.dropped
    );

    // `selection:` never folds into the element's own cascade — the
    // plan §3.5 boundary, carried on the report.
    let program = frozen("bg-primary selection:bg-red-500");
    let (_, report) = lower_program(&program, &Env::default());
    assert!(
        !report.is_empty(),
        "the pseudo-element layer is reported: {report:?}"
    );
}

#[test]
fn the_source_seam_is_object_safe_for_host_sources() {
    // The dyn-compat contract the `*_with` entry points rely on.
    fn takes_source(_: Option<&dyn TokenSource>) {}
    let table = baked_table();
    takes_source(Some(&table));
}
