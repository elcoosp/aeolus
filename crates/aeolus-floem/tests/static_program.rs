//! The V1 static-program lane's correctness (architecture plan §7's
//! Floem re-target, the second backend after GPUI): a `tw!`-compiled
//! [`StaticProgram`] walks into a Floem [`Style`] under the interp's
//! [`Env`] semantics, and the result must AGREE with the plan bridge —
//! the same compiled artifact lowered through
//! [`aeolus_compile::emit_plan_theme`] and the v0.x
//! [`lower`](aeolus_floem::lower) the crate has shipped since v0.1.
//! Two lanes, one compiler, one cascade.
//!
//! Floem's `Style::apply_selectors` is the plan path's cascade evaluator,
//! so the differential pins compare the walk's Env-folded style against
//! the plan path's selector-resolved style — the same effective values a
//! running app would compute, from two different program forms.

use floem::peniko::{Brush, Color as PenikoColor};
use floem::style::unit::Length;
use floem::style::{
    Background, LineHeight, LineHeightValue, PaddingLeft, Style, StyleSelector, TextColor,
};

use aeolus_compile::{Config, compile_theme, emit_plan_theme, keyword_alignment, keyword_texts};
use aeolus_floem::{lower, lower_program, lower_program_with};
use aeolus_rt::{AttrBit, AttrKind, AttrState, Env, StaticProgram};
use aeolus_theme::{ThemeSource, TokenTable};

/// The baked shadcn LIGHT table (T0's shape: every colour token bakes to
/// a literal) — the same construction the `tw!` macro's freeze walk runs.
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the baked shadcn table builds")
}

/// Compile a class string the way `tw!` does (the theme pass's baked
/// table) and freeze it into the static form the macro emits: the
/// interned layers plus the row texts the keyword slots index.
fn frozen(classes: &str) -> StaticProgram {
    let config = Config::default();
    let compiled =
        compile_theme(classes, &baked_table(), &config).expect("the class string lowers");
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

/// The plan-bridge style for the SAME class string — the v0.x plan the
/// five backends consume, lowered through the crate's shipped
/// [`lower`].
fn bridged(classes: &str) -> Style {
    let sheet =
        emit_plan_theme(classes, &baked_table(), &Config::default()).expect("the plan bridge");
    let (style, report) = lower(&sheet.plan);
    assert!(report.is_empty(), "plan bridge dropped: {report:?}");
    style
}

fn brush_rgba(b: &Brush) -> [u8; 4] {
    match b {
        // peniko 0.6 has no `[u8; 4]` compare for Rgba8 — raw components.
        Brush::Solid(c) => c.components.map(|v| (v * 255.0).round() as u8),
        _ => panic!("expected a solid brush"),
    }
}

/// The rgba a peniko colour carries, for the quantization-tolerant
/// comparison the two generations' colour paths agree within.
fn peniko_rgba(c: &PenikoColor) -> [u8; 4] {
    c.components.map(|v| (v * 255.0).round() as u8)
}

/// bg-primary under the shadcn light palette: oklch(0.205 0 0) ≈ #171717.
const PRIMARY: [u8; 4] = [23, 23, 23, 255];

#[test]
fn the_base_layer_lowers_from_the_static_program() {
    let program = frozen("p-4 bg-primary rounded-md");
    let (style, report) = lower_program(&program, &Env::default());
    assert!(report.is_empty(), "unexpected drops: {report:?}");

    let bg = style.get(Background);
    assert_eq!(brush_rgba(bg.as_ref().unwrap()), PRIMARY, "bg-primary");

    // p-4 = 1rem = 16px under the backend's REM_PX constant.
    assert_eq!(style.get(PaddingLeft), Length::Pt(16.0));

    // rounded-md = 0.375rem = 6px (the registry row's v4 default; the
    // shadcn-remapped 8px scale is the THEME's answer, not the row's).
    assert_eq!(
        style.get(floem::style::BorderTopLeftRadius),
        Length::Pt(6.0)
    );
}

#[test]
fn state_layers_walk_under_the_frame_env() {
    let program = frozen("bg-primary hover:bg-red-500 disabled:opacity-50");

    let (idle, _) = lower_program(&program, &Env::default());
    assert_eq!(brush_rgba(idle.get(Background).as_ref().unwrap()), PRIMARY);

    let hovered = lower_program(
        &program,
        &Env {
            hover: true,
            ..Env::default()
        },
    )
    .0;
    // red-500 = oklch(0.637 0.237 25.331), gamut-mapped by the audited
    // bake to #fb2c36.
    assert_eq!(
        brush_rgba(hovered.get(Background).as_ref().unwrap()),
        [251, 44, 54, 255],
        "hover:bg-red-500 fires under the probe"
    );

    let disabled = lower_program(
        &program,
        &Env {
            disabled: true,
            ..Env::default()
        },
    )
    .0;
    assert!(
        (disabled.get(floem::style::Opacity) - 0.5).abs() < 1e-6,
        "disabled:opacity-50 scales to 0.5, got {}",
        disabled.get(floem::style::Opacity)
    );
    assert_eq!(brush_rgba(idle.get(Background).as_ref().unwrap()), PRIMARY);
}

#[test]
fn mode_and_breakpoint_layers_walk_under_the_env() {
    let program = frozen("p-4 md:p-6");

    let (mobile, _) = lower_program(&program, &Env::default());
    assert_eq!(mobile.get(PaddingLeft), Length::Pt(16.0), "below md");

    let wide = lower_program(
        &program,
        &Env {
            viewport_rem: 50.0,
            ..Env::default()
        },
    )
    .0;
    assert_eq!(wide.get(PaddingLeft), Length::Pt(24.0), "at md (48rem)");
}

#[test]
fn typography_lowers_through_the_row_texts() {
    // The bare keyword row (font-medium's text is "500") and the
    // numeric rem row (leading-6 = 1.5rem) — two annotation shapes.
    let program = frozen("font-medium leading-6");
    let (style, report) = lower_program(&program, &Env::default());
    assert!(report.is_empty(), "unexpected drops: {report:?}");
    let weight = style.get(floem::style::FontWeight);
    assert_eq!(
        weight,
        Some(floem::text::FontWeight::new(500.0)),
        "font-medium lowers to the weight row text"
    );
    assert_eq!(style.get(LineHeight), LineHeightValue::Pt(24.0));

    // The composite pair (text-sm → font-size 0.875rem + the
    // calc-composed leading ratio): targets[0] takes the primary text,
    // targets[1] the secondary, one queue per (cond, prop).
    let program = frozen("text-sm");
    let (style, report) = lower_program(&program, &Env::default());
    assert!(report.is_empty(), "unexpected drops: {report:?}");
    assert_eq!(
        style.get(floem::style::FontSize),
        14.0,
        "text-sm's font-size row text resolves through REM_PX"
    );
    // calc(1.25 / 0.875) ≈ 1.4286 — the unitless ratio floem resolves
    // against the element's font-size at paint.
    let ratio = match style.get(LineHeight) {
        LineHeightValue::Normal(r) => r,
        other @ LineHeightValue::Pt(_) => {
            panic!("expected a ratio line-height, got {other:?}")
        }
    };
    assert!((ratio - 1.25 / 0.875).abs() < 1e-5, "got {ratio}");
}

#[test]
fn the_walk_agrees_with_the_plan_bridge() {
    // The shadcn button corpus plus the family spread the plan path's
    // own battery covers — every class lowers under BOTH program forms.
    let corpus = "p-4 bg-primary text-primary-foreground rounded-md \
                  flex items-center justify-center gap-2 w-1/2 h-9 \
                  font-medium leading-6 truncate overflow-hidden \
                  select-none cursor-pointer pointer-events-none z-10 \
                  border-2 border-primary mx-auto";
    let program = frozen(corpus);
    let (walked, report) = lower_program(&program, &Env::default());
    assert!(report.is_empty(), "walk dropped: {report:?}");
    let bridged_style = bridged(corpus);

    assert_eq!(
        walked.get(Background).as_ref().map(brush_rgba),
        bridged_style.get(Background).as_ref().map(brush_rgba),
        "background agrees across the two lanes"
    );
    let walk_fg = walked.get(TextColor).map(|c| peniko_rgba(&c));
    let bridge_fg = bridged_style.get(TextColor).map(|c| peniko_rgba(&c));
    assert!(
        matches!((walk_fg, bridge_fg), (Some(a), Some(b)) if
            a.iter().zip(b.iter()).all(|(x, y)| (i16::from(*x) - i16::from(*y)).abs() <= 1)),
        "text color agrees within the /255 quantization step: {walk_fg:?} vs {bridge_fg:?}"
    );
    assert_eq!(
        walked.get(PaddingLeft),
        bridged_style.get(PaddingLeft),
        "padding agrees"
    );
    assert_eq!(
        walked.get(floem::style::BorderTopLeftRadius),
        bridged_style.get(floem::style::BorderTopLeftRadius),
        "radius agrees (the row's v4 default, not the theme remap)"
    );
    assert_eq!(
        walked.get(floem::style::BorderLeft).width,
        bridged_style.get(floem::style::BorderLeft).width,
        "border width agrees"
    );
}

#[test]
fn the_walk_agrees_with_the_bridge_under_state_and_mode() {
    let corpus = "bg-primary hover:bg-red-500 dark:bg-white p-4";
    let program = frozen(corpus);

    // Hover: the plan path nests a Hover selector; the walk folds the
    // layer under env.hover. Same effective colour.
    let walked = lower_program(
        &program,
        &Env {
            hover: true,
            ..Env::default()
        },
    )
    .0;
    let bridged_style = bridged(corpus).apply_selectors(&[StyleSelector::Hover]);
    assert_eq!(
        walked.get(Background).as_ref().map(brush_rgba),
        bridged_style.get(Background).as_ref().map(brush_rgba),
        "hover layer agrees"
    );

    // Mode: `dark:` nests a DarkMode selector; the walk folds under
    // env.dark. (The breakpoint lanes differ by construction — the plan
    // path's responsive selectors evaluate against the real window at
    // layout time, the walk against the env's rem — so the breakpoint
    // agreement pins on the walk side alone.)
    let walked = lower_program(
        &program,
        &Env {
            dark: true,
            ..Env::default()
        },
    )
    .0;
    let bridged_style = bridged(corpus).apply_selectors(&[StyleSelector::DarkMode]);
    assert_eq!(
        walked.get(Background).as_ref().map(brush_rgba),
        bridged_style.get(Background).as_ref().map(brush_rgba),
        "dark layer agrees"
    );
    let idle_walked = lower_program(&program, &Env::default()).0;
    let idle_bridged = bridged(corpus).apply_selectors(&[]);
    assert_eq!(
        idle_walked.get(PaddingLeft),
        idle_bridged.get(PaddingLeft),
        "the base layer under the light mode agrees"
    );
}

#[test]
fn token_slots_refuse_without_a_source_and_resolve_through_one() {
    // The RUNTIME table (T2's shape): bg-primary lowers to a token slot
    // the IR carries verbatim — the host resolves it per frame.
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::runtime(&source).expect("the runtime table builds");
    let config = Config::default();
    let compiled =
        compile_theme("bg-primary/90", &table, &config).expect("the class string lowers");
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

    // No source: the themed longhand refuses into the report — never a
    // silent transparent background.
    let (style, report) = lower_program(&program, &Env::default());
    assert!(
        style.get(Background).is_none(),
        "a token slot without a source must not render a colour"
    );
    assert!(
        !report.is_empty(),
        "the refusal is reported, not silent: {report:?}"
    );

    // With the table as the token source: the T2 one-load-one-multiply
    // answers, the /90 alpha premultiplied onto the colour.
    let (style, report) = lower_program_with(&program, &Env::default(), None, Some(&table));
    assert!(report.is_empty(), "unexpected drops: {report:?}");
    let rgba = brush_rgba(style.get(Background).as_ref().unwrap());
    assert_eq!(rgba[..3], PRIMARY[..3], "channels unchanged, alpha scaled");
    assert!(
        (f32::from(rgba[3]) - 229.0).abs() <= 1.0,
        "alpha 0.9 baked: got {rgba:?}"
    );
}

#[test]
fn mode_pairs_resolve_through_a_baked_source() {
    // The T1 table (light AND dark): colour tokens lower to pair slots;
    // the walk selects the half the env's mode asks for.
    let source = ThemeSource::shadcn_dark().expect("the built-in shadcn-dark source parses");
    let table = TokenTable::baked(&source).expect("the baked pair table builds");
    let config = Config::default();
    let compiled =
        compile_theme("bg-background", &table, &config).expect("the class string lowers");
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let annotations = keyword_texts("bg-background", &config).expect("annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    let program = StaticProgram::new(
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    );

    let (light, report) = lower_program_with(&program, &Env::default(), None, Some(&table));
    assert!(report.is_empty(), "unexpected drops: {report:?}");
    let (dark, _) = lower_program_with(
        &program,
        &Env {
            dark: true,
            ..Env::default()
        },
        None,
        Some(&table),
    );
    let light_rgba = brush_rgba(light.get(Background).as_ref().unwrap());
    let dark_rgba = brush_rgba(dark.get(Background).as_ref().unwrap());
    assert_ne!(light_rgba, dark_rgba, "the mode tier selects the half");
    // shadcn light background = oklch(1 0 0) → #ffffff; dark = #0a0a0a.
    assert_eq!(light_rgba, [255, 255, 255, 255], "the light half");
    assert_eq!(dark_rgba, [10, 10, 10, 255], "the dark half");
}

#[test]
fn attr_probe_layers_fire_through_the_seam() {
    let program = frozen("bg-primary data-[state=open]:bg-red-500");

    // No probe source: the layer is CSS-quiet AND reported (the interp's
    // AttributeUnqueried note, carried on the walk's report channel).
    let (idle, report) = lower_program(&program, &Env::default());
    assert_eq!(brush_rgba(idle.get(Background).as_ref().unwrap()), PRIMARY);
    assert!(!report.is_empty(), "the unqueried layer is reported");

    // The answered miss is CSS-quiet — and NOT a note.
    let closed = AttrState::new(&[AttrBit {
        kind: AttrKind::Data,
        name: "state",
        value: "closed",
    }]);
    let (missed, report) = lower_program_with(&program, &Env::default(), Some(&closed), None);
    assert_eq!(
        brush_rgba(missed.get(Background).as_ref().unwrap()),
        PRIMARY
    );
    assert!(report.is_empty(), "an answered miss is quiet: {report:?}");

    // The answered hit fires.
    let open = AttrState::new(&[AttrBit {
        kind: AttrKind::Data,
        name: "state",
        value: "open",
    }]);
    let (fired, _) = lower_program_with(&program, &Env::default(), Some(&open), None);
    assert_eq!(
        brush_rgba(fired.get(Background).as_ref().unwrap()),
        [251, 44, 54, 255],
        "data-[state=open]:bg-red-500 fires through the seam"
    );
}

#[test]
fn unmapped_longhands_are_reported_never_dropped() {
    // shadow-sm lowers into the IR (Prop::BoxShadow, the preset's var
    // text) but has no Floem slot in the walk — the report names it.
    let program = frozen("shadow-sm bg-primary");
    let (style, report) = lower_program(&program, &Env::default());
    assert_eq!(
        brush_rgba(style.get(Background).as_ref().unwrap()),
        PRIMARY,
        "the mapped longhand still lands"
    );
    assert!(
        !report.is_empty(),
        "the unmapped longhand is reported: {report:?}"
    );
    let names = report
        .dropped
        .iter()
        .map(|(what, _)| what)
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        names.iter().any(|what| what.contains("box-shadow")),
        "the report names the longhand: {names:?}"
    );
}

#[test]
fn descendant_and_pseudo_layers_stay_the_reported_boundary() {
    // `has-focus:` and `selection:` never fold into the element's own
    // cascade — the plan §3.5/interp boundary, carried on the report.
    let program = frozen("bg-primary selection:bg-red-500");
    let (_, report) = lower_program(&program, &Env::default());
    assert!(
        !report.is_empty(),
        "the pseudo-element layer is reported: {report:?}"
    );
}
