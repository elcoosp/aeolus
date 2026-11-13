//! The program lane's style form (the `program_style` seam this file
//! pins): a frozen static program — the two slices the `tw!` macro
//! emits — resolves into the v0.x [`ResolvedStyle`] under one probe
//! [`Env`], and the result must AGREE with the plan bridge — the same
//! class string lowered through [`aeolus_compile::emit_plan_theme`]
//! into the buckets the five backends consume. Two write paths, one
//! compiler, one cascade: the bridge routes per candidate into the
//! plan's buckets, the walk folds the fired layers in program order —
//! the same slot values must come out.

use aeolus_compile::{
    AttrBit, AttrKind, AttrState, Config, Env, InterpNote, PlanNoteKind, TokenSource,
    emit_plan_theme, keyword_alignment, keyword_texts, program_style, program_style_with,
};
use aeolus_core::Breakpoint;
use aeolus_core::plan::VariantCondition;
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
fn frozen(classes: &str) -> (&'static [aeolus_ir::Layer], &'static [&'static str]) {
    let config = Config::default();
    let compiled =
        aeolus_compile::compile_theme(classes, &baked_table(), &config).expect("the class lowers");
    assert!(
        compiled.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        compiled.diagnostics
    );
    let annotations = keyword_texts(classes, &config).expect("non-empty input annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    (
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    )
}

/// The plan-bridge sheet for the SAME class string — the v0.x plan the
/// five backends consume.
fn bridged(classes: &str) -> aeolus_compile::PlanSheet {
    let sheet = emit_plan_theme(classes, &baked_table(), &Config::default())
        .expect("the plan bridge lowers");
    assert!(
        sheet.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        sheet.diagnostics
    );
    sheet
}

/// bg-primary under the shadcn light palette: oklch(0.205 0 0) ≈
/// #171717 — the same constant the plan bridge battery pins.
const PRIMARY: [f32; 4] = [23.0 / 255.0, 23.0 / 255.0, 23.0 / 255.0, 1.0];

/// The sRGBA a baked background carries, for the quantization-tolerant
/// colour reads (both lanes convert through the audited parser, so the
/// agreement is exact up to the 1e-4 Oklab equality epsilon).
fn srgba(c: aeolus_core::Color) -> [f32; 4] {
    c.to_srgba()
}

#[test]
fn the_base_layer_resolves_the_style() {
    let (layers, keywords) = frozen("p-4 bg-primary rounded-md");
    let (style, walk) = program_style(layers, keywords, &Env::default());
    assert!(
        walk.notes.is_empty() && walk.boundaries.is_empty() && walk.refusals.is_empty(),
        "a fully mapped base must walk clean: {walk:?}"
    );

    // p-4 = 1rem = 16px on every edge.
    let padding = style.padding.expect("p-4 lands");
    assert_eq!(padding.left, 16.0);
    assert_eq!(padding.top, 16.0);

    // bg-primary bakes to the palette literal.
    let bg = style.background.expect("bg-primary lands");
    let rgba = srgba(bg.0);
    for (got, want) in rgba.iter().zip(PRIMARY.iter()) {
        assert!((got - want).abs() < 1.0 / 255.0, "got {rgba:?}");
    }

    // rounded-md = 0.375rem = 6px (the registry row's v4 default).
    let radius = style.corner_radius.expect("rounded-md lands");
    assert_eq!(radius.top_left, 6.0);
}

#[test]
fn the_walk_agrees_with_the_plan_bridge_on_the_base() {
    // The corpus the plan path's own battery covers — every class
    // lowers under BOTH forms, and the resolved slots must be EQUAL.
    let corpus = "p-4 bg-primary text-primary-foreground rounded-md \
                  border-2 border-primary gap-2 w-1/2 h-9 mx-auto";
    let (layers, keywords) = frozen(corpus);
    let (style, walk) = program_style(layers, keywords, &Env::default());
    assert!(
        walk.notes.is_empty() && walk.boundaries.is_empty() && walk.refusals.is_empty(),
        "the corpus fully maps: {walk:?}"
    );
    let base = &bridged(corpus).plan.base;

    assert_eq!(style.padding, base.padding, "padding agrees");
    assert_eq!(style.background, base.background, "background agrees");
    assert_eq!(
        style.content_color, base.content_color,
        "text colour agrees"
    );
    assert_eq!(
        style.corner_radius, base.corner_radius,
        "corner radius agrees"
    );
    assert_eq!(style.border_width, base.border_width, "border width agrees");
    assert_eq!(
        style.border_color, base.border_color,
        "border colour agrees"
    );
    assert_eq!(style.gap, base.gap, "the gap slot agrees");
    assert_eq!(
        style.extras.margin_auto, base.extras.margin_auto,
        "the auto-margin edges agree"
    );
    assert_eq!(style.dimensions, base.dimensions, "dimensions agree");
}

#[test]
fn state_layers_fold_under_the_env_like_the_bridge_buckets() {
    let corpus = "bg-primary hover:bg-red-500 active:opacity-60 disabled:opacity-50";
    let (layers, keywords) = frozen(corpus);
    let sheet = bridged(corpus);

    // Idle: the base bucket's background.
    let (idle, _) = program_style(layers, keywords, &Env::default());
    assert_eq!(
        idle.background.as_ref().map(|b| srgba(b.0)),
        sheet.plan.base.background.as_ref().map(|b| srgba(b.0)),
    );

    // Hover: the walk folds the hover layer; the bridge routed the
    // class into the Hover bucket. Same effective colour.
    let hover_layer = sheet
        .plan
        .layers
        .iter()
        .find(|l| l.condition == VariantCondition::Hover)
        .expect("the bridge carries a hover bucket");
    let (hovered, _) = program_style(
        layers,
        keywords,
        &Env {
            hover: true,
            ..Env::default()
        },
    );
    assert_eq!(
        hovered.background.as_ref().map(|b| srgba(b.0)),
        hover_layer.style.background.as_ref().map(|b| srgba(b.0)),
        "the hover fold agrees with the bridge's bucket"
    );

    // Disabled: the opacity scale halves.
    let (disabled, _) = program_style(
        layers,
        keywords,
        &Env {
            disabled: true,
            ..Env::default()
        },
    );
    let opacity = disabled.opacity.expect("disabled:opacity-50 lands");
    assert!((opacity.0 - 0.5).abs() < 1e-6, "got {}", opacity.0);
}

#[test]
fn mode_and_breakpoint_layers_fold_under_the_env() {
    let corpus = "p-4 md:p-6 dark:bg-white";
    let (layers, keywords) = frozen(corpus);
    let sheet = bridged(corpus);

    // Below md: base padding only.
    let (mobile, _) = program_style(layers, keywords, &Env::default());
    assert_eq!(mobile.padding.expect("p-4").left, 16.0);

    // At md (48rem): the responsive bucket's padding.
    let md = sheet
        .plan
        .responsive
        .get(&Breakpoint::Md)
        .expect("the bridge routes md: into the responsive bucket");
    let (wide, _) = program_style(
        layers,
        keywords,
        &Env {
            viewport_rem: 50.0,
            ..Env::default()
        },
    );
    assert_eq!(
        wide.padding, md.padding,
        "the md fold agrees with the bucket"
    );

    // Dark mode: the dark slot's background.
    let dark = sheet.plan.dark.as_ref().expect("the bridge routes dark:");
    let (darked, _) = program_style(
        layers,
        keywords,
        &Env {
            dark: true,
            ..Env::default()
        },
    );
    assert_eq!(
        darked.background.as_ref().map(|b| srgba(b.0)),
        dark.background.as_ref().map(|b| srgba(b.0)),
        "the dark fold agrees with the bridge's dark slot"
    );
}

#[test]
fn the_composite_text_pair_lowers_through_the_row_texts() {
    let (layers, keywords) = frozen("text-sm");
    let (style, walk) = program_style(layers, keywords, &Env::default());
    assert!(walk.notes.is_empty(), "{walk:?}");
    // font-size 0.875rem = 14px; the calc(1.25 / 0.875) ratio bakes
    // against the candidate's own font size (the v0.x 20px semantic).
    let size = style
        .text
        .iter()
        .find(|(p, _)| *p == aeolus_core::TextProperty::FontSize)
        .map(|(_, v)| *v)
        .expect("text-sm's font size lands");
    assert!(
        matches!(size, aeolus_core::TextValue::Px(px) if (px - 14.0).abs() < 1e-4),
        "got {size:?}"
    );
    let line = style
        .text
        .iter()
        .find(|(p, _)| *p == aeolus_core::TextProperty::LineHeight)
        .map(|(_, v)| *v)
        .expect("text-sm's line height lands");
    assert!(
        matches!(line, aeolus_core::TextValue::Px(px) if (px - 20.0).abs() < 1e-3),
        "got {line:?}"
    );
}

#[test]
fn the_gap_axes_split_like_the_bridge() {
    // `gap-x-4` alone: the axes slot (the v0.x oracle's own split).
    let (layers, keywords) = frozen("gap-x-4");
    let (style, walk) = program_style(layers, keywords, &Env::default());
    assert!(walk.notes.is_empty(), "{walk:?}");
    assert!(
        style.gap.is_none(),
        "a single-axis gap must not fill the whole Gap slot"
    );
    let axes = style
        .extras
        .gap_axes
        .expect("gap-x-4 lands on the axes slot");
    assert_eq!(axes.col, Some(16.0));
    assert_eq!(axes.row, None);

    // The bridge writes the same split.
    let base = &bridged("gap-x-4").plan.base;
    assert_eq!(style.extras.gap_axes, base.extras.gap_axes);
}

#[test]
fn token_slots_refuse_without_a_source_and_resolve_through_one() {
    // The RUNTIME table (T2's shape): bg-primary lowers to a token slot
    // the frozen program carries verbatim — the host resolves it per
    // walk through the TokenSource seam.
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::runtime(&source).expect("the runtime table builds");
    let config = Config::default();
    let compiled = aeolus_compile::compile_theme("bg-primary/90", &table, &config)
        .expect("the class string lowers");
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let annotations = keyword_texts("bg-primary/90", &config).expect("annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    let keywords: &'static [&'static str] = Box::leak(keywords.into_boxed_slice());

    // No source: the themed longhand refuses into the walk's report —
    // never a silent transparent background.
    let (style, walk) = program_style(compiled.program.layers, keywords, &Env::default());
    assert!(
        style.background.is_none(),
        "a token slot without a source must not render a colour"
    );
    assert!(
        !walk.refusals.is_empty(),
        "the refusal is reported, not silent: {walk:?}"
    );

    // With the table as the source: the T2 one-load-one-multiply
    // answers, the /90 alpha premultiplied onto the colour.
    let (style, walk) = program_style_with(
        compiled.program.layers,
        keywords,
        &Env::default(),
        None,
        Some(&table),
    );
    assert!(walk.refusals.is_empty(), "{walk:?}");
    let rgba = srgba(style.background.expect("bg-primary/90 lands").0);
    assert!(
        (rgba[3] - 0.9).abs() <= 1.0 / 255.0,
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
    let compiled = aeolus_compile::compile_theme("bg-background", &table, &config)
        .expect("the class string lowers");
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let annotations = keyword_texts("bg-background", &config).expect("annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    let keywords: &'static [&'static str] = Box::leak(keywords.into_boxed_slice());

    let (light, walk) = program_style_with(
        compiled.program.layers,
        keywords,
        &Env::default(),
        None,
        Some(&table),
    );
    assert!(walk.refusals.is_empty(), "{walk:?}");
    let (dark, _) = program_style_with(
        compiled.program.layers,
        keywords,
        &Env {
            dark: true,
            ..Env::default()
        },
        None,
        Some(&table),
    );
    let light_rgba = srgba(light.background.expect("bg-background lands").0);
    let dark_rgba = srgba(dark.background.expect("bg-background lands").0);
    assert_ne!(light_rgba, dark_rgba, "the mode tier selects the half");
    // shadcn light background = oklch(1 0 0) → #ffffff; dark = #0a0a0a.
    assert!(
        light_rgba
            .iter()
            .zip([1.0f32; 3].iter())
            .all(|(a, b)| (a - b) < 1.0 / 255.0)
            && (light_rgba[3] - 1.0).abs() < 1e-6,
        "the light half: {light_rgba:?}"
    );
    assert!(
        (dark_rgba[0] - 10.0 / 255.0).abs() < 1.0 / 255.0,
        "the dark half: {dark_rgba:?}"
    );
}

#[test]
fn attr_probe_layers_fire_through_the_seam() {
    let (layers, keywords) = frozen("bg-primary data-[state=open]:bg-red-500");

    // No probe source: the layer is CSS-quiet AND reported (the interp's
    // AttributeUnqueried note, carried on the walk's note channel).
    let (idle, walk) = program_style(layers, keywords, &Env::default());
    let bg = srgba(idle.background.expect("bg-primary lands").0);
    assert!(
        bg.iter()
            .zip(PRIMARY.iter())
            .all(|(a, b)| (a - b).abs() < 1.0 / 255.0),
        "the probe layer stayed quiet: {bg:?}"
    );
    assert!(
        walk.notes
            .iter()
            .any(|n| matches!(n, InterpNote::AttributeUnqueried { .. })),
        "the unqueried layer is reported: {walk:?}"
    );

    // The answered miss is CSS-quiet — and NOT a note.
    let closed = AttrState::new(&[AttrBit {
        kind: AttrKind::Data,
        name: "state",
        value: "closed",
    }]);
    let (_, walk) = program_style_with(layers, keywords, &Env::default(), Some(&closed), None);
    assert!(walk.notes.is_empty(), "an answered miss is quiet: {walk:?}");

    // The answered hit fires.
    let open = AttrState::new(&[AttrBit {
        kind: AttrKind::Data,
        name: "state",
        value: "open",
    }]);
    let (fired, _) = program_style_with(layers, keywords, &Env::default(), Some(&open), None);
    let bg = srgba(fired.background.expect("the probe hit lands").0);
    // red-500 = oklch(0.637 0.237 25.331), gamut-mapped to #fb2c36.
    assert!(
        (bg[0] - 251.0 / 255.0).abs() < 1.0 / 255.0 && (bg[1] - 44.0 / 255.0).abs() < 1.0 / 255.0,
        "data-[state=open]:bg-red-500 fires through the seam: {bg:?}"
    );
}

#[test]
fn boundary_longhands_report_as_typed_notes_never_silently() {
    // shadow-sm lowers into the IR (Prop::BoxShadow, the preset's var
    // text) but the v0.x slot wants a parsed layer — the SAME typed
    // boundary the plan bridge notes, carried on the walk.
    let (layers, keywords) = frozen("shadow-sm bg-primary");
    let (style, walk) = program_style(layers, keywords, &Env::default());
    assert!(
        style.background.is_some(),
        "the mapped longhand still lands"
    );
    assert!(
        walk.boundaries
            .iter()
            .any(|b| matches!(b, PlanNoteKind::ShadowPreset)),
        "the shadow preset is a typed boundary: {walk:?}"
    );

    // place-content has no v0.x slot either (the SlotUnsupported set).
    let (layers, keywords) = frozen("place-content-center");
    let (_, walk) = program_style(layers, keywords, &Env::default());
    assert!(
        walk.boundaries
            .iter()
            .any(|b| matches!(b, PlanNoteKind::SlotUnsupported { .. })),
        "the slot-less longhand is a typed boundary: {walk:?}"
    );
}

#[test]
fn pseudo_and_descendant_layers_stay_the_reported_boundary() {
    // `selection:` and `has-focus:` never fold into the element's own
    // cascade — the plan §3.5 boundary, carried as the interp's notes.
    let (layers, keywords) = frozen("bg-primary selection:bg-red-500");
    let (_, walk) = program_style(layers, keywords, &Env::default());
    assert!(
        walk.notes
            .iter()
            .any(|n| matches!(n, InterpNote::PseudoElement { .. })),
        "the pseudo-element layer is reported: {walk:?}"
    );

    let (layers, keywords) = frozen("has-focus:bg-red-500");
    let (_, walk) = program_style(layers, keywords, &Env::default());
    assert!(
        walk.notes
            .iter()
            .any(|n| matches!(n, InterpNote::AttributeUnqueried { .. })),
        "the descendant shape is reported: {walk:?}"
    );
}

#[test]
fn the_source_seam_is_object_safe_for_host_sources() {
    // The dyn-compat contract the backends' `*_with` entry points rely
    // on (the floem/gpui/xilem walks pass Option<&dyn TokenSource>).
    fn takes_source(_: Option<&dyn TokenSource>) {}
    let table = baked_table();
    takes_source(Some(&table));
}
