//! The egui backend's V1 static-program lane (architecture plan §7's
//! re-target — the fifth backend on the lane after GPUI, Floem, Xilem
//! and Iced): a `tw!`-compiled [`StaticProgram`] resolves through the
//! pipeline's shared walk into the crate's own [`EguiStyle`] under one
//! probe [`Env`], and the result must AGREE with the plan path — the
//! same class string through the plan bridge and the
//! [`StyleBook`](aeolus_egui::StyleBook) resolution the crate has
//! lowered since v0.1. Two lanes, one compiler, one cascade.
//!
//! The differential pins compare the walk's Env-folded style against
//! the plan path's per-probe resolution — the same effective `Frame`
//! a running immediate-mode frame would paint, from two different
//! program forms.

use aeolus_compile::{Config, compile_theme, emit_plan_theme, keyword_alignment, keyword_texts};
use aeolus_egui::{EguiProbe, EguiStyle, REM_PX, env_for_probe, lower_program, lower_program_with};
use aeolus_rt::{AttrBit, AttrKind, AttrState, Env, StaticProgram, TokenSource};
use aeolus_theme::{ThemeSource, TokenTable};

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
/// crate's [`StyleBook`](aeolus_egui::StyleBook) resolves per state.
fn bridged(classes: &str) -> aeolus_core::StylePlan {
    let sheet = emit_plan_theme(classes, &baked_table(), &Config::default())
        .expect("the plan bridge lowers");
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    sheet.plan
}

/// The plan path's own lowering: bridge plan → per-probe resolve → the
/// crate's `lower_resolved` — exactly the resolution
/// [`StyleBook::resolve_at`](aeolus_egui::StyleBook::resolve_at) runs,
/// minus the book.
fn planned(classes: &str, probe: &EguiProbe) -> EguiStyle {
    let plan = bridged(classes);
    let resolved = plan.resolve(probe);
    let mut report = aeolus_egui::LoweringReport::default();
    aeolus_egui::lower_resolved(&resolved, &mut report)
}

/// bg-primary under the shadcn light palette: oklch(0.205 0 0) ≈
/// #171717 — the constant the plan-bridge batteries pin. egui colours
/// are straight-alpha sRGB bytes.
const PRIMARY: [u8; 4] = [23, 23, 23, 255];

/// The straight-alpha bytes an egui colour carries, for the
/// quantization-tolerant colour reads (both lanes convert through the
/// audited parser; the Oklab equality epsilon is far below one 8-bit
/// step).
fn rgba(c: egui::Color32) -> [u8; 4] {
    c.to_array()
}

fn fill_of(style: &EguiStyle) -> [u8; 4] {
    rgba(style.frame.fill)
}

/// Byte-wise ±1 comparison — the Oklab equality epsilon is far below one
/// 8-bit step, and egui's `Frame` fill is always present (a refusal keeps
/// the `Color32::default()` the no-source pin reads).
fn close(a: [u8; 4], b: [u8; 4]) -> bool {
    a.iter()
        .zip(b.iter())
        .all(|(x, y)| i16::from(*x) - i16::from(*y) <= 1 && i16::from(*y) - i16::from(*x) <= 1)
}

#[test]
fn the_base_layer_lowers_from_the_static_program() {
    let program = frozen("p-4 bg-primary rounded-md");
    let (style, report) = lower_program(&program, &Env::default());
    assert!(report.is_empty(), "unexpected drops: {report:?}");

    // p-4 = 1rem = 16px under the backend's REM_PX constant (CSS
    // padding is egui's inner_margin, the crate's own mapping).
    let margin = style.frame.inner_margin;
    assert_eq!(margin.left, 16);
    assert_eq!(margin.top, 16);

    let fill = fill_of(&style);
    for (got, want) in fill.iter().zip(PRIMARY.iter()) {
        assert!(i16::from(*got) - i16::from(*want) <= 1, "got {fill:?}");
    }

    // rounded-md = 0.375rem = 6px (the registry row's v4 default; the
    // shadcn-remapped scale is the THEME's answer, not the row's).
    assert_eq!(style.frame.corner_radius.nw, 6);
}

#[test]
fn rem_bakes_through_the_backend_constant() {
    // Plan §9's per-backend rem_px constant: the lane's own REM_PX is
    // the 16px bake the v0.x resolver's spacing scale applies, so the
    // two lanes agree on every spacing class.
    assert_eq!(REM_PX, 16.0);
    let program = frozen("p-2 gap-2");
    let (style, _) = lower_program(&program, &Env::default());
    assert_eq!(style.frame.inner_margin.top, 8);
    assert_eq!(
        style.item_spacing,
        Some(egui::Vec2::splat(8.0)),
        "gap-2 rides the hint"
    );
}

#[test]
fn state_layers_walk_under_env_for_probe() {
    let program = frozen("bg-primary hover:bg-red-500 disabled:opacity-50");

    // The crate's own conversion: EguiProbe → the probe Env the lane
    // walks under.
    let idle_env = env_for_probe(&EguiProbe::default());
    let (idle, _) = lower_program(&program, &idle_env);
    assert!(close(fill_of(&idle), PRIMARY), "idle: {:?}", fill_of(&idle));

    let hover_env = env_for_probe(&EguiProbe {
        hovered: true,
        ..EguiProbe::default()
    });
    let (hovered, _) = lower_program(&program, &hover_env);
    // red-500 = oklch(0.637 0.237 25.331), gamut-mapped by the audited
    // bake to #fb2c36.
    assert!(
        close(fill_of(&hovered), [251, 44, 54, 255]),
        "hover:bg-red-500 fires under the probe: {:?}",
        fill_of(&hovered)
    );

    let disabled_env = env_for_probe(&EguiProbe {
        disabled: true,
        ..EguiProbe::default()
    });
    let (disabled, _) = lower_program(&program, &disabled_env);
    // egui has no opacity primitive — the CSS composite bakes into
    // every painted alpha byte (the crate's own documented baking).
    let fill = fill_of(&disabled);
    assert!(
        (f32::from(fill[3]) - 255.0 * 0.5).abs() <= 1.5,
        "disabled:opacity-50 bakes to alpha 0.5, got {fill:?}"
    );
}

#[test]
fn env_for_probe_maps_the_egui_probe() {
    let env = env_for_probe(&EguiProbe {
        dark: true,
        focused: true,
        width: Some(800.0),
        container_width: Some(400.0),
        ..EguiProbe::default()
    });
    assert!(env.dark, "the probe's dark flag rides");
    // egui's Response::has_focus carries no focus-visible distinction —
    // the focused flag feeds BOTH focus conditions (the plan path's
    // documented normalization: EguiProbe::test maps State::Focus and
    // State::FocusVisible onto the same bool).
    assert!(env.focus && env.focus_visible);
    assert!((env.viewport_rem - 50.0).abs() < 1e-4, "800px / 16 = 50rem");
    assert_eq!(env.container_rem, Some(25.0), "400px / 16 = 25rem");
    // The probe cannot answer group/peer/visited/checked/placeholder —
    // they stay off (CSS: a selector that does not match).
    assert!(
        !env.group_hover
            && !env.peer_focus
            && !env.visited
            && !env.checked
            && !env.placeholder_shown
    );
}

#[test]
fn the_lane_agrees_with_the_plan_path_per_state() {
    // The shadcn button corpus plus the family spread the plan path's
    // own battery covers — every mapped slot must agree per probe
    // combination, the immediate-mode frame's two resolution paths.
    // (The typography family rides `font-medium`: the composite
    // `text-sm` row's line-height half is egui's own reported boundary,
    // not a lane disagreement.)
    let corpus = "p-4 bg-primary text-primary-foreground rounded-md \
                  border-2 border-primary gap-2 w-24 h-9 font-medium \
                  hover:bg-red-500 active:opacity-60";
    let program = frozen(corpus);

    for probe in &[
        EguiProbe::default(),
        EguiProbe {
            hovered: true,
            ..EguiProbe::default()
        },
        EguiProbe {
            active: true,
            ..EguiProbe::default()
        },
        EguiProbe {
            disabled: true,
            ..EguiProbe::default()
        },
        EguiProbe {
            hovered: true,
            active: true,
            ..EguiProbe::default()
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
            walked.frame.inner_margin, planned_style.frame.inner_margin,
            "padding @ {probe:?}"
        );
        assert!(
            close(fill_of(&walked), fill_of(&planned_style)),
            "fill @ {probe:?}: {:?} vs {:?}",
            fill_of(&walked),
            fill_of(&planned_style)
        );
        assert_eq!(
            walked.text.color, planned_style.text.color,
            "text colour @ {probe:?}"
        );
        assert_eq!(
            walked.frame.corner_radius, planned_style.frame.corner_radius,
            "radius @ {probe:?}"
        );
        assert_eq!(
            walked.frame.stroke, planned_style.frame.stroke,
            "border stroke @ {probe:?}"
        );
        assert_eq!(
            walked.item_spacing, planned_style.item_spacing,
            "gap hint @ {probe:?}"
        );
        assert_eq!(walked.width, planned_style.width, "width hint @ {probe:?}");
        assert_eq!(
            walked.height, planned_style.height,
            "height hint @ {probe:?}"
        );
        assert_eq!(
            walked.text.weight, planned_style.text.weight,
            "font weight hint @ {probe:?}"
        );
    }
}

#[test]
fn the_lane_agrees_with_the_plan_path_under_dark_and_wide() {
    let corpus = "p-4 md:p-6 dark:bg-white bg-primary";
    let program = frozen(corpus);

    // Wide viewport: the responsive bucket's padding — the walk under
    // the env's rem, the plan path through the probe's width.
    let wide = EguiProbe {
        width: Some(800.0),
        ..EguiProbe::default()
    };
    let (walked, _) = lower_program(&program, &env_for_probe(&wide));
    assert_eq!(walked.frame.inner_margin.left, 24, "md:p-6 at 50rem");

    // Dark mode: the dark layer's fill, both lanes.
    let dark = EguiProbe {
        dark: true,
        ..EguiProbe::default()
    };
    let (walked, _) = lower_program(&program, &env_for_probe(&dark));
    let planned_style = planned(corpus, &dark);
    assert!(
        close(fill_of(&walked), fill_of(&planned_style)),
        "dark fill agrees: {:?} vs {:?}",
        fill_of(&walked),
        fill_of(&planned_style)
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
    let (book, book_report) = aeolus_egui::StyleBook::new(&plan);
    assert!(
        book_report.is_empty(),
        "the corpus fully maps: {book_report:?}"
    );

    let dark_probe = EguiProbe {
        dark: true,
        ..EguiProbe::default()
    };
    let (walked, _) = lower_program(&program, &env_for_probe(&dark_probe));
    let book_style = book.resolve_at(&dark_probe);
    assert!(
        close(fill_of(&walked), rgba(book_style.frame.fill)),
        "the book's dark payload matches the walk: {:?} vs {:?}",
        fill_of(&walked),
        book_style.frame.fill
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
        style.frame.fill == egui::Color32::default(),
        "a token slot without a source must not render a colour"
    );
    assert!(!report.is_empty(), "the refusal is reported: {report:?}");

    // With the table as the token source: the T2 one-load-one-multiply
    // answers, the /90 alpha premultiplied onto the colour.
    let (style, report) = lower_program_with(&program, &Env::default(), None, Some(&table));
    assert!(report.is_empty(), "unexpected drops: {report:?}");
    let fill = fill_of(&style);
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
    assert!(close(fill_of(&idle), PRIMARY));
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
    assert!(
        close(fill_of(&fired), [251, 44, 54, 255]),
        "data-[state=open]:bg-red-500 fires through the seam: {:?}",
        fill_of(&fired)
    );
}

#[test]
fn unmapped_longhands_are_reported_never_dropped() {
    // shadow-sm lowers into the IR (Prop::BoxShadow, the preset's var
    // text) but the walk's slot writer reports the preset family — the
    // report names it.
    let program = frozen("shadow-sm bg-primary");
    let (style, report) = lower_program(&program, &Env::default());
    assert!(close(fill_of(&style), PRIMARY), "the mapped longhand lands");
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
