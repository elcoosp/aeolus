//! The Iced backend's V1 static-program lane (architecture plan §7's
//! re-target — the fourth backend on the lane after GPUI, Floem and
//! Xilem): a `tw!`-compiled [`StaticProgram`] resolves through the
//! pipeline's shared walk into the crate's own [`IcedStyle`] under one
//! probe [`Env`], and the result must AGREE with the plan path — the
//! same class string through the plan bridge and the
//! [`StyleBook`](aeolus_iced::StyleBook) resolution the crate has
//! lowered since v0.1. Two lanes, one compiler, one cascade.
//!
//! The differential pins compare the walk's Env-folded style against
//! the plan path's per-probe resolution — the same effective
//! `container::Style` a running Catalog closure would paint, from two
//! different program forms.

use aeolus_compile::{Config, compile_theme, emit_plan_theme, keyword_alignment, keyword_texts};
use aeolus_iced::{IcedProbe, REM_PX, StyleBook, env_for_probe, lower_program, lower_program_with};
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
/// crate's [`StyleBook`] resolves per state.
fn bridged(classes: &str) -> aeolus_core::StylePlan {
    let sheet = emit_plan_theme(classes, &baked_table(), &Config::default())
        .expect("the plan bridge lowers");
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    sheet.plan
}

/// The plan path's own lowering: bridge plan → per-probe resolve → the
/// crate's `lower_resolved` — exactly the resolution
/// [`StyleBook::resolve_at`] runs, minus the book.
fn planned(classes: &str, probe: &IcedProbe) -> aeolus_iced::IcedStyle {
    let plan = bridged(classes);
    let resolved = plan.resolve(probe);
    let mut report = aeolus_iced::LoweringReport::default();
    aeolus_iced::lower_resolved(&resolved, &mut report)
}

/// bg-primary under the shadcn light palette: oklch(0.205 0 0) ≈
/// #171717 — the constant the plan-bridge batteries pin.
const PRIMARY: [f32; 4] = [23.0 / 255.0, 23.0 / 255.0, 23.0 / 255.0, 1.0];

/// The straight-sRGBA an iced colour carries, for the
/// quantization-tolerant colour reads (both lanes convert through the
/// audited parser; the Oklab equality epsilon is far below one 8-bit
/// step).
fn rgba(c: iced_core::Color) -> [f32; 4] {
    [c.r, c.g, c.b, c.a]
}

fn bg_of(style: &aeolus_iced::IcedStyle) -> Option<[f32; 4]> {
    style.container.background.as_ref().map(|bg| match bg {
        iced_core::Background::Color(c) => rgba(*c),
        iced_core::Background::Gradient(_) => panic!("the corpus carries no gradients"),
    })
}

fn close(a: Option<[f32; 4]>, b: Option<[f32; 4]>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => a
            .iter()
            .zip(b.iter())
            .all(|(x, y)| (x - y).abs() <= 1.0 / 255.0),
        _ => false,
    }
}

#[test]
fn the_base_layer_lowers_from_the_static_program() {
    let program = frozen("p-4 bg-primary rounded-md");
    let (style, report) = lower_program(&program, &Env::default());
    assert!(report.is_empty(), "unexpected drops: {report:?}");

    // p-4 = 1rem = 16px under the backend's REM_PX constant.
    let padding = style.widget.padding.expect("p-4 lands");
    assert_eq!(padding.top, 16.0);
    assert_eq!(padding.left, 16.0);

    let bg = bg_of(&style).expect("bg-primary lands");
    for (got, want) in bg.iter().zip(PRIMARY.iter()) {
        assert!((got - want).abs() <= 1.0 / 255.0, "got {bg:?}");
    }

    // rounded-md = 0.375rem = 6px (the registry row's v4 default; the
    // shadcn-remapped scale is the THEME's answer, not the row's).
    let radius = style.container.border.radius;
    assert!((radius.top_left - 6.0).abs() < 1e-4, "got {radius:?}");
}

#[test]
fn rem_bakes_through_the_backend_constant() {
    // Plan §9's per-backend rem_px constant: the lane's own REM_PX is
    // the 16px bake the v0.x resolver's spacing scale applies, so the
    // two lanes agree on every spacing class.
    assert_eq!(REM_PX, 16.0);
    let program = frozen("p-2 gap-2");
    let (style, _) = lower_program(&program, &Env::default());
    assert_eq!(style.widget.padding.expect("p-2").top, 8.0);
    assert_eq!(style.widget.spacing, Some(8.0), "gap-2 rides the hint");
}

#[test]
fn state_layers_walk_under_env_for_probe() {
    let program = frozen("bg-primary hover:bg-red-500 disabled:opacity-50");

    // The crate's own conversion: IcedProbe → the probe Env the lane
    // walks under.
    let idle_env = env_for_probe(&IcedProbe::default());
    let (idle, _) = lower_program(&program, &idle_env);
    assert!(
        close(bg_of(&idle), Some(PRIMARY)),
        "idle: {:?}",
        bg_of(&idle)
    );

    let hover_env = env_for_probe(&IcedProbe {
        hovered: true,
        ..IcedProbe::default()
    });
    let (hovered, _) = lower_program(&program, &hover_env);
    // red-500 = oklch(0.637 0.237 25.331), gamut-mapped by the audited
    // bake to #fb2c36.
    assert!(
        close(
            bg_of(&hovered),
            Some([251.0 / 255.0, 44.0 / 255.0, 54.0 / 255.0, 1.0])
        ),
        "hover:bg-red-500 fires under the probe: {:?}",
        bg_of(&hovered)
    );

    let disabled_env = env_for_probe(&IcedProbe {
        disabled: true,
        ..IcedProbe::default()
    });
    let (disabled, _) = lower_program(&program, &disabled_env);
    // Iced has no opacity primitive — the CSS composite bakes into
    // every painted alpha (the crate's own documented baking).
    let bg = bg_of(&disabled).expect("the base colour still paints");
    assert!(
        (bg[3] - 0.5).abs() <= 1.0 / 255.0,
        "disabled:opacity-50 bakes to alpha 0.5, got {bg:?}"
    );
}

#[test]
fn env_for_probe_maps_the_iced_probe() {
    let env = env_for_probe(&IcedProbe {
        dark: true,
        focused: true,
        width: Some(800.0),
        container_width: Some(400.0),
        ..IcedProbe::default()
    });
    assert!(env.dark, "the probe's dark flag rides");
    // Iced 0.14 has no focus-visible distinction — the focused flag
    // feeds BOTH focus conditions (the plan path's documented
    // normalization: IcedProbe::test maps State::Focus and
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
    // combination, the Catalog closures' two resolution paths.
    let corpus = "p-4 bg-primary text-primary-foreground rounded-md \
                  border-2 border-primary gap-2 w-1/2 h-9 text-sm \
                  hover:bg-red-500 active:opacity-60";
    let program = frozen(corpus);

    for probe in &[
        IcedProbe::default(),
        IcedProbe {
            hovered: true,
            ..IcedProbe::default()
        },
        IcedProbe {
            active: true,
            ..IcedProbe::default()
        },
        IcedProbe {
            disabled: true,
            ..IcedProbe::default()
        },
        IcedProbe {
            hovered: true,
            active: true,
            ..IcedProbe::default()
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
            walked.widget.padding, planned_style.widget.padding,
            "padding @ {probe:?}"
        );
        assert!(
            close(bg_of(&walked), bg_of(&planned_style)),
            "background @ {probe:?}: {:?} vs {:?}",
            bg_of(&walked),
            bg_of(&planned_style)
        );
        assert_eq!(
            walked.container.text_color, planned_style.container.text_color,
            "text colour @ {probe:?}"
        );
        assert_eq!(
            walked.container.border.radius, planned_style.container.border.radius,
            "radius @ {probe:?}"
        );
        assert_eq!(
            walked.container.border.width, planned_style.container.border.width,
            "border width @ {probe:?}"
        );
        assert_eq!(
            walked.container.border.color, planned_style.container.border.color,
            "border colour @ {probe:?}"
        );
        assert_eq!(
            walked.widget.width, planned_style.widget.width,
            "width hint @ {probe:?}"
        );
        assert_eq!(
            walked.widget.height, planned_style.widget.height,
            "height hint @ {probe:?}"
        );
        assert_eq!(
            walked.text.font_size, planned_style.text.font_size,
            "font size hint @ {probe:?}"
        );
    }
}

#[test]
fn the_lane_agrees_with_the_plan_path_under_dark_and_wide() {
    let corpus = "p-4 md:p-6 dark:bg-white bg-primary";
    let program = frozen(corpus);

    // Wide viewport: the responsive bucket's padding — the walk under
    // the env's rem, the plan path through the probe's width.
    let wide = IcedProbe {
        width: Some(800.0),
        ..IcedProbe::default()
    };
    let (walked, _) = lower_program(&program, &env_for_probe(&wide));
    assert_eq!(
        walked.widget.padding.expect("p-6").left,
        24.0,
        "md:p-6 at 50rem"
    );

    // Dark mode: the dark layer's background, both lanes.
    let dark = IcedProbe {
        dark: true,
        ..IcedProbe::default()
    };
    let (walked, _) = lower_program(&program, &env_for_probe(&dark));
    let planned_style = planned(corpus, &dark);
    assert!(
        close(bg_of(&walked), bg_of(&planned_style)),
        "dark background agrees: {:?} vs {:?}",
        bg_of(&walked),
        bg_of(&planned_style)
    );
}

#[test]
fn the_book_and_the_lane_agree_on_the_catalog_payload() {
    // The StyleBook the plan path ships and the program lane must paint
    // the same container payload for the same theme situation — the
    // two consumption paths of one class string.
    let corpus = "p-4 bg-primary text-primary-foreground rounded-md hover:bg-red-500";
    let program = frozen(corpus);
    let plan = bridged(corpus);
    let (book, book_report) = StyleBook::new(&plan);
    assert!(
        book_report.is_empty(),
        "the corpus fully maps: {book_report:?}"
    );

    let dark_probe = IcedProbe {
        dark: true,
        ..IcedProbe::default()
    };
    let (walked, _) = lower_program(&program, &env_for_probe(&dark_probe));
    let book_style = book.container_style(&iced_core::Theme::Dark);
    assert!(
        close(
            bg_of(&walked),
            book_style.background.map(|bg| match bg {
                iced_core::Background::Color(c) => rgba(c),
                iced_core::Background::Gradient(_) => panic!("no gradients in the corpus"),
            })
        ),
        "the book's dark payload matches the walk: {:?} vs {:?}",
        bg_of(&walked),
        book_style.background
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
        style.container.background.is_none(),
        "a token slot without a source must not render a colour"
    );
    assert!(!report.is_empty(), "the refusal is reported: {report:?}");

    // With the table as the token source: the T2 one-load-one-multiply
    // answers, the /90 alpha premultiplied onto the colour.
    let (style, report) = lower_program_with(&program, &Env::default(), None, Some(&table));
    assert!(report.is_empty(), "unexpected drops: {report:?}");
    let bg = bg_of(&style).expect("bg-primary/90 lands");
    assert!(
        (bg[3] - 0.9).abs() <= 1.0 / 255.0,
        "channels unchanged, alpha scaled: got {bg:?}"
    );
}

#[test]
fn attr_probe_layers_fire_through_the_seam() {
    let program = frozen("bg-primary data-[state=open]:bg-red-500");

    // No probe source: the layer is CSS-quiet AND reported (the interp's
    // AttributeUnqueried note, carried on the backend report channel).
    let (idle, report) = lower_program(&program, &Env::default());
    assert!(close(bg_of(&idle), Some(PRIMARY)));
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
        close(
            bg_of(&fired),
            Some([251.0 / 255.0, 44.0 / 255.0, 54.0 / 255.0, 1.0])
        ),
        "data-[state=open]:bg-red-500 fires through the seam: {:?}",
        bg_of(&fired)
    );
}

#[test]
fn unmapped_longhands_are_reported_never_dropped() {
    // shadow-sm lowers into the IR (Prop::BoxShadow, the preset's var
    // text) but the walk's slot writer reports the preset family — the
    // report names it.
    let program = frozen("shadow-sm bg-primary");
    let (style, report) = lower_program(&program, &Env::default());
    assert!(
        close(bg_of(&style), Some(PRIMARY)),
        "the mapped longhand lands"
    );
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
