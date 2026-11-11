//! The plan-lowering emit arm's contracts (`Target::Plan` — the
//! architecture plan §7 migration seam): the V1 pipeline's compiled
//! artifact lowered into the v0.x `StylePlan` the five backends
//! consume, with every unmappable longhand a TYPED note.
//!
//! The centrepiece is the DIFFERENTIAL ORACLE: for every class in the
//! battery, `emit_plan_theme`'s plan must agree — slot for slot, edge
//! mask for edge mask, colour within the baked-table quantization
//! tolerance — with the audited v0.x `Resolver` fed the SAME theme
//! CSS. That is the P1 exit criterion's plan-level form ("new compiler
//! == old resolver"), the bridge the §3.2 table's "re-targeted onto
//! `Program`" backend rows migrate through.

use aeolus_compile::{
    Config, CustomUtility, Customs, PlanNoteKind, PlanSheet, Target, compile, emit_plan,
    emit_plan_theme, emit_plan_theme_with, emit_plan_with,
};
use aeolus_core::{
    EdgeSet, Resolver, StylePlan, TAILWIND_DEFAULT_THEME, TailwindTheme, VariantCondition,
};
use aeolus_ir::{Cond, Prop};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// The differential harness: the semantic comparator
// ---------------------------------------------------------------------------

/// The sRGB quantization tolerance: the V1 side bakes colours through
/// the theme pass's `[u8; 4]` table (one `/255` step per channel), the
/// v0.x side carries f32 Oklab. Both mix opacity in Oklab (the audited
/// premultiplied identity), so the comparison converts BOTH to Oklab
/// and allows the baked table's rounding — ±0.5/255 per sRGB channel
/// plus the conversion slope, well inside 0.02 per Oklab axis.
const OKLAB_EPSILON: f32 = 0.02;
const ALPHA_EPSILON: f32 = 0.011;

fn colors_agree(a: &aeolus_core::Color, b: &aeolus_core::Color) -> bool {
    (a.l - b.l).abs() <= OKLAB_EPSILON
        && (a.a - b.a).abs() <= OKLAB_EPSILON
        && (a.b - b.b).abs() <= OKLAB_EPSILON
        && (a.alpha - b.alpha).abs() <= ALPHA_EPSILON
}

/// The two plans agree: every slot carries the same value (colours
/// within the quantization tolerance), the same bucket routing, the
/// same per-edge masks. A differential failure names the FIRST
/// disagreeing slot — the reviewable unit.
fn assert_plans_agree(v1: &StylePlan, v0: &StylePlan) {
    let a = &v1.base;
    let b = &v0.base;

    assert_eq!(
        a.padding, b.padding,
        "base padding disagrees (V1 {a:?} vs V0 {b:?})"
    );
    assert_eq!(a.margin, b.margin, "base margin disagrees");
    assert_eq!(
        a.extras.margin_auto, b.extras.margin_auto,
        "base margin_auto disagrees"
    );
    assert_eq!(
        a.dimensions, b.dimensions,
        "base dimensions disagree (V1 {:?} vs V0 {:?})",
        a.dimensions, b.dimensions
    );
    assert_eq!(
        a.min_dimensions, b.min_dimensions,
        "base min-dimensions disagree"
    );
    assert_eq!(
        a.max_dimensions, b.max_dimensions,
        "base max-dimensions disagree"
    );
    assert_eq!(
        a.corner_radius, b.corner_radius,
        "base corner radius disagrees"
    );
    assert_eq!(
        a.border_width, b.border_width,
        "base border width disagrees"
    );
    match (&a.background, &b.background) {
        (None, None) => {}
        (Some(x), Some(y)) => assert!(
            colors_agree(&x.0, &y.0),
            "base background disagrees: V1 {x:?} vs V0 {y:?}"
        ),
        (x, y) => panic!("base background presence disagrees: {x:?} vs {y:?}"),
    }
    match (&a.content_color, &b.content_color) {
        (None, None) => {}
        (Some(x), Some(y)) => assert!(
            colors_agree(&x.0, &y.0),
            "base content colour disagrees: V1 {x:?} vs V0 {y:?}"
        ),
        (x, y) => panic!("base content colour presence disagrees: {x:?} vs {y:?}"),
    }
    match (&a.border_color, &b.border_color) {
        (None, None) => {}
        (Some(x), Some(y)) => assert!(
            colors_agree(&x.0, &y.0),
            "base border colour disagrees: V1 {x:?} vs V0 {y:?}"
        ),
        (x, y) => panic!("base border colour presence disagrees: {x:?} vs {y:?}"),
    }
    assert_eq!(
        a.extras.border_style, b.extras.border_style,
        "border style disagrees"
    );
    assert_eq!(
        a.layout, b.layout,
        "base layout disagrees (V1 {:?} vs V0 {:?})",
        a.layout, b.layout
    );
    assert_eq!(a.gap, b.gap, "base gap disagrees");
    assert_eq!(
        a.extras.gap_axes, b.extras.gap_axes,
        "base gap axes disagree"
    );
    assert_eq!(a.opacity, b.opacity, "base opacity disagrees");
    match (&a.ring, &b.ring) {
        (None, None) => {}
        (Some(x), Some(y)) => {
            assert_eq!(x.width, y.width, "ring width disagrees");
            assert_eq!(
                x.offset_width, y.offset_width,
                "ring offset width disagrees"
            );
            assert_eq!(x.inset, y.inset, "ring inset disagrees");
            match (&x.color, &y.color) {
                (None, None) => {}
                (Some(c1), Some(c2)) => assert!(
                    colors_agree(c1, c2),
                    "ring colour disagrees: V1 {c1:?} vs V0 {c2:?}"
                ),
                (x, y) => panic!("ring colour presence disagrees: {x:?} vs {y:?}"),
            }
            match (&x.offset_color, &y.offset_color) {
                (None, None) => {}
                (Some(c1), Some(c2)) => assert!(
                    colors_agree(c1, c2),
                    "ring offset colour disagrees: V1 {c1:?} vs V0 {c2:?}"
                ),
                (x, y) => panic!("ring offset colour presence disagrees: {x:?} vs {y:?}"),
            }
        }
        (x, y) => panic!("ring presence disagrees: {x:?} vs {y:?}"),
    }
    match (&a.outline, &b.outline) {
        (None, None) => {}
        (Some(x), Some(y)) => {
            assert_eq!(x.width, y.width, "outline width disagrees");
            assert_eq!(x.none, y.none, "outline-none disagrees");
            assert_eq!(x.offset, y.offset, "outline offset disagrees");
            match (&x.color, &y.color) {
                (None, None) => {}
                (Some(c1), Some(c2)) => assert!(
                    colors_agree(c1, c2),
                    "outline colour disagrees: V1 {c1:?} vs V0 {c2:?}"
                ),
                (x, y) => panic!("outline colour presence disagrees: {x:?} vs {y:?}"),
            }
        }
        (x, y) => panic!("outline presence disagrees: {x:?} vs {y:?}"),
    }
    assert_eq!(a.cursor, b.cursor, "base cursor disagrees");
    assert_eq!(
        a.pointer_events, b.pointer_events,
        "pointer events disagree"
    );
    assert_eq!(a.text.len(), b.text.len(), "text property count disagrees");
    for (i, (p1, v1)) in a.text.iter().enumerate() {
        let (p2, v2) = &b.text[i];
        assert_eq!(p1, p2, "text property {i} differs: {p1:?} vs {p2:?}");
        assert_eq!(v1, v2, "text value for {p1:?} differs: {v1:?} vs {v2:?}");
    }
    let (ea, eb) = (&a.extras, &b.extras);
    assert_eq!(ea.position, eb.position, "position disagrees");
    assert_eq!(ea.inset, eb.inset, "inset disagrees");
    assert_eq!(ea.z_index, eb.z_index, "z-index disagrees");
    assert_eq!(ea.order, eb.order, "order disagrees");
    assert_eq!(ea.overflow, eb.overflow, "overflow disagrees");
    assert_eq!(ea.visibility, eb.visibility, "visibility disagrees");
    assert_eq!(
        ea.aspect_ratio, eb.aspect_ratio,
        "aspect ratio disagrees (V1 {:?} vs V0 {:?})",
        ea.aspect_ratio, eb.aspect_ratio
    );
    assert_eq!(ea.flex_basis, eb.flex_basis, "flex basis disagrees");
    assert_eq!(ea.transform, eb.transform, "transform disagrees");
    assert_eq!(ea.grid, eb.grid, "grid disagrees");

    // The buckets: same routing, same layer conditions, same dark/responsive
    // presence. Rule conditions compare through their Debug shape (the
    // v0.x Cond carries owned Strings; the text is the identity).
    assert_eq!(v1.layers.len(), v0.layers.len(), "layer count disagrees");
    for (i, (l1, l2)) in v1.layers.iter().zip(v0.layers.iter()).enumerate() {
        assert_eq!(l1.condition, l2.condition, "layer {i} condition disagrees");
        assert_plans_agree(
            &StylePlan {
                base: l1.style.clone(),
                ..StylePlan::default()
            },
            &StylePlan {
                base: l2.style.clone(),
                ..StylePlan::default()
            },
        );
    }
    match (&v1.dark, &v0.dark) {
        (None, None) => {}
        (Some(d1), Some(d2)) => assert_plans_agree(
            &StylePlan {
                base: d1.clone(),
                ..StylePlan::default()
            },
            &StylePlan {
                base: d2.clone(),
                ..StylePlan::default()
            },
        ),
        (x, y) => panic!("dark bucket presence disagrees: {x:?} vs {y:?}"),
    }
    assert_eq!(
        v1.responsive.len(),
        v0.responsive.len(),
        "responsive key count disagrees"
    );
    for (bp, style1) in &v1.responsive {
        let style2 = v0
            .responsive
            .get(bp)
            .unwrap_or_else(|| panic!("responsive {bp:?} missing on the v0.x side"));
        assert_plans_agree(
            &StylePlan {
                base: style1.clone(),
                ..StylePlan::default()
            },
            &StylePlan {
                base: style2.clone(),
                ..StylePlan::default()
            },
        );
    }
    assert_eq!(v1.group.len(), v0.group.len(), "group count disagrees");
    assert_eq!(v1.peer.len(), v0.peer.len(), "peer count disagrees");
    assert_eq!(v1.rules.len(), v0.rules.len(), "rule count disagrees");
    for (i, (r1, r2)) in v1.rules.iter().zip(v0.rules.iter()).enumerate() {
        assert_eq!(
            format!("{:?}", r1.when),
            format!("{:?}", r2.when),
            "rule {i} condition disagrees"
        );
        assert_plans_agree(
            &StylePlan {
                base: r1.style.clone(),
                ..StylePlan::default()
            },
            &StylePlan {
                base: r2.style.clone(),
                ..StylePlan::default()
            },
        );
    }
}

/// One routing case: the input class and the bucket predicate the
/// shared oracle runs against BOTH plans.
type RouteCase = (&'static str, fn(&StylePlan) -> bool);

// ---------------------------------------------------------------------------
// The differential battery
// ---------------------------------------------------------------------------

/// Every class the differential pins. All of them lower through the
/// registry AND resolve through the v0.x oracle warning-free (verified
/// against both sides before landing — a new entry here is a contract
/// both generations must keep).
const BATTERY: &[&str] = &[
    // spacing & margins (the per-edge set-mask families)
    "p-4",
    "px-8",
    "pt-2",
    "pb-0",
    "m-4",
    "mt-2",
    "-mt-2",
    "mx-auto",
    // sizing
    "w-10",
    "h-6",
    "w-full",
    "w-auto",
    "w-fit",
    "max-w-10",
    "min-h-6",
    // radii (the corner set-mask family; the v4-default radius scale)
    "rounded-md",
    "rounded-lg",
    "rounded-full",
    "rounded-t-lg",
    // colours (the baked palette) + the opacity modifier
    "bg-red-500",
    "bg-red-500/50",
    "text-red-500",
    "border-red-500",
    // borders
    "border-2",
    "border-x-2",
    "border-dashed",
    // typography
    "text-sm",
    "font-medium",
    "font-bold",
    "leading-tight",
    "tracking-tight",
    "text-center",
    "italic",
    "uppercase",
    "underline",
    "whitespace-nowrap",
    "select-none",
    "select-text",
    "select-all",
    "underline-offset-4",
    "font-sans",
    // layout
    "flex",
    "flex-col",
    "items-center",
    "justify-between",
    "gap-2",
    "gap-x-4",
    "grow",
    "shrink-0",
    "basis-4",
    // position & extras
    "absolute",
    "top-0",
    "inset-0",
    "z-10",
    "order-3",
    "overflow-hidden",
    "invisible",
    "aspect-video",
    "cursor-pointer",
    "pointer-events-none",
    // ring & outline
    "ring-2",
    "ring-red-500",
    "ring-red-500/50",
    "ring-offset-2",
    "outline-2",
    "outline-red-500",
    // opacity & transforms
    "opacity-50",
    "scale-50",
    "rotate-45",
    "translate-x-2",
    "skew-x-3",
    // grid (the numeric forms)
    "grid-cols-3",
];

#[test]
fn differential_battery_agrees_with_the_v0_resolver() {
    let theme = TailwindTheme::from_theme_css(TAILWIND_DEFAULT_THEME).unwrap();
    let resolver = Resolver::new(&theme);
    let source = ThemeSource::from_css(TAILWIND_DEFAULT_THEME, None).unwrap();
    let tokens = TokenTable::baked(&source).unwrap();

    for class in BATTERY {
        let sheet = emit_plan_theme(class, &tokens, &Config::default())
            .unwrap_or_else(|e| panic!("{class}: the emit refused: {e:?}"));
        assert!(
            sheet.diagnostics.is_empty(),
            "{class}: V1 lowered with diagnostics {:?}",
            sheet.diagnostics
        );
        assert!(
            sheet.notes.is_empty(),
            "{class}: V1 lowered with notes {:?}",
            sheet.notes
        );
        let resolution = resolver.resolve(class).unwrap();
        assert!(
            resolution.warnings.is_empty(),
            "{class}: the v0.x oracle warned {:?} — the battery entry is not oracle-clean",
            resolution.warnings
        );
        assert_plans_agree(&sheet.plan, &resolution.plan);
    }
}

/// The multi-class strings cascade the same way on both sides: source
/// order, later per-longhand wins, per-edge masks surviving (`pt-2
/// px-8` keeps the top edge), slot merges (`ring-2 ring-red-500` one
/// ring), bucket coexistence (`gap-2 gap-x-4`).
#[test]
fn differential_strings_agree_with_the_v0_resolver() {
    let theme = TailwindTheme::from_theme_css(TAILWIND_DEFAULT_THEME).unwrap();
    let resolver = Resolver::new(&theme);
    let source = ThemeSource::from_css(TAILWIND_DEFAULT_THEME, None).unwrap();
    let tokens = TokenTable::baked(&source).unwrap();

    let strings = [
        "pt-2 px-8",
        "p-4 px-8",
        "ring-2 ring-red-500",
        "gap-2 gap-x-4",
        "flex flex-col items-center gap-2",
        "p-4 bg-red-500 text-sm rounded-md hover:bg-red-500/90",
        "hover:bg-red-500 focus-visible:ring-2",
        "w-10 h-6 rounded-md bg-red-500 text-center",
    ];
    for input in strings {
        let sheet = emit_plan_theme(input, &tokens, &Config::default()).expect("the emit refused");
        assert!(
            sheet.diagnostics.is_empty(),
            "{input}: diagnostics {:?}",
            sheet.diagnostics
        );
        assert!(sheet.notes.is_empty(), "{input}: notes {:?}", sheet.notes);
        let resolution = resolver.resolve(input).unwrap();
        assert!(
            resolution.warnings.is_empty(),
            "{input}: v0.x warnings {:?}",
            resolution.warnings
        );
        assert_plans_agree(&sheet.plan, &resolution.plan);
    }
}

/// The shadcn token surface: the SAME shadcn light CSS feeds both
/// generations, so the runtime token slots agree too (`bg-primary`,
/// `text-primary-foreground`). Radii stay OUT of this battery: the
/// registry's radius row carries the v4 defaults, the shadcn theme
/// re-maps them (`--radius-md: calc(var(--radius) - 2px)`) — the
/// documented theme-remap boundary.
#[test]
fn differential_shadcn_tokens_agree() {
    let theme = TailwindTheme::from_theme_css(aeolus_core::shadcn_css::LIGHT).unwrap();
    let resolver = Resolver::new(&theme);
    let source = ThemeSource::shadcn().unwrap();
    let tokens = TokenTable::baked(&source).unwrap();

    let strings = [
        "bg-primary",
        "text-primary-foreground",
        "border-input",
        "bg-primary/90",
        "bg-accent text-accent-foreground",
        "bg-destructive text-white",
        "bg-muted text-muted-foreground",
        "ring-ring/50",
    ];
    for input in strings {
        let sheet = emit_plan_theme(input, &tokens, &Config::default()).expect("the emit refused");
        assert!(
            sheet.diagnostics.is_empty(),
            "{input}: diagnostics {:?}",
            sheet.diagnostics
        );
        let resolution = resolver.resolve(input).unwrap();
        assert!(
            resolution.warnings.is_empty(),
            "{input}: v0.x warnings {:?}",
            resolution.warnings
        );
        assert_plans_agree(&sheet.plan, &resolution.plan);
    }
}

/// The consumer story end to end: the shadcn Button's classes through
/// the V1 pipeline into the plan shape the backends already consume —
/// with the two classes the registry does not carry yet
/// (`transition-colors`, the bare `border` default) reported as the
/// typed diagnostics they are, never dropped silently.
#[test]
fn shadcn_button_lowers_into_a_backend_ready_plan() {
    let source = ThemeSource::shadcn().unwrap();
    let tokens = TokenTable::baked(&source).unwrap();
    let input = "inline-flex items-center justify-center gap-2 whitespace-nowrap \
                 rounded-md text-sm font-medium bg-primary text-primary-foreground \
                 hover:bg-primary/90 focus-visible:ring-2 disabled:opacity-50 \
                 transition-colors border";
    let sheet = emit_plan_theme(input, &tokens, &Config::default()).expect("the emit refused");

    let plan = &sheet.plan;
    let base = &plan.base;
    assert!(base.layout.is_some(), "the layout classes lowered");
    assert!(base.background.is_some(), "bg-primary lowered");
    // hover + focus-visible + disabled — the three classic-state layers
    // the string carries (the v0.x oracle routes the same three).
    assert_eq!(
        plan.layers.len(),
        3,
        "the three state layers, {:?}",
        plan.layers.iter().map(|l| l.condition).collect::<Vec<_>>()
    );
    assert_eq!(
        plan.layers[0].condition,
        VariantCondition::Hover,
        "the hover layer routes into the layer bucket"
    );
    assert!(plan.dark.is_none());

    // The two registry gaps surface as diagnostics, with spans:
    // `transition-colors` (no transition prop in the V1 registry yet)
    // and the bare `border` (no default-width arm yet).
    assert_eq!(
        sheet.diagnostics.len(),
        2,
        "the two registry gaps are the only reports: {:?}",
        sheet.diagnostics
    );
}

// ---------------------------------------------------------------------------
// The bucket routing contracts
// ---------------------------------------------------------------------------

/// The fold's conditions route into the v0.x buckets the backends read:
/// the seven classic states into `layers`, `dark:` into `dark`, `sm:`/
/// `min-md:` into `responsive`, `group-*:`/`peer-*:` into the label
/// vecs, and everything the v0.1 buckets cannot express (the attribute
/// probes, `max-*:`, `@*:` container queries, `has-*`) into `rules`.
#[test]
fn conditions_route_into_the_v0_buckets() {
    let theme = TailwindTheme::from_theme_css(TAILWIND_DEFAULT_THEME).unwrap();
    let resolver = Resolver::new(&theme);
    let source = ThemeSource::from_css(TAILWIND_DEFAULT_THEME, None).unwrap();
    let tokens = TokenTable::baked(&source).unwrap();

    // One routing case: the input class and the bucket predicate the
    // shared oracle runs against BOTH plans.
    let cases: &[RouteCase] = &[
        ("hover:bg-red-500", |p| {
            p.layers.len() == 1 && p.layers[0].condition == VariantCondition::Hover
        }),
        ("focus:bg-red-500", |p| {
            p.layers.len() == 1 && p.layers[0].condition == VariantCondition::Focus
        }),
        ("focus-visible:bg-red-500", |p| {
            p.layers.len() == 1 && p.layers[0].condition == VariantCondition::FocusVisible
        }),
        ("focus-within:bg-red-500", |p| {
            p.layers.len() == 1 && p.layers[0].condition == VariantCondition::FocusWithin
        }),
        ("active:bg-red-500", |p| {
            p.layers.len() == 1 && p.layers[0].condition == VariantCondition::Active
        }),
        ("disabled:opacity-50", |p| {
            p.layers.len() == 1 && p.layers[0].condition == VariantCondition::Disabled
        }),
        ("visited:bg-red-500", |p| {
            p.layers.len() == 1 && p.layers[0].condition == VariantCondition::Visited
        }),
        ("dark:bg-red-500", |p| p.dark.is_some()),
        ("sm:px-8", |p| {
            p.responsive.contains_key(&aeolus_core::Breakpoint::Sm)
        }),
        ("min-md:px-8", |p| {
            p.responsive.contains_key(&aeolus_core::Breakpoint::Md)
        }),
        ("group-hover:opacity-75", |p| p.group.len() == 1),
        ("peer-hover:opacity-75", |p| p.peer.len() == 1),
        ("data-open:bg-red-500", |p| p.rules.len() == 1),
        ("checked:bg-red-500", |p| p.rules.len() == 1),
        ("selection:bg-red-500", |p| p.rules.len() == 1),
        ("max-lg:px-8", |p| p.rules.len() == 1),
        ("@md:px-8", |p| p.rules.len() == 1),
        ("has-focus:bg-red-500", |p| p.rules.len() == 1),
    ];
    for (input, check) in cases {
        let sheet = emit_plan_theme(input, &tokens, &Config::default()).expect("the emit refused");
        assert!(
            sheet.diagnostics.is_empty() && sheet.notes.is_empty(),
            "{input}: reports {:?} / {:?}",
            sheet.diagnostics,
            sheet.notes
        );
        assert!(
            check(&sheet.plan),
            "{input}: routed wrong — plan summary: layers {} dark? {} responsive {} group {} peer {} rules {}",
            sheet.plan.layers.len(),
            sheet.plan.dark.is_some(),
            sheet.plan.responsive.len(),
            sheet.plan.group.len(),
            sheet.plan.peer.len(),
            sheet.plan.rules.len()
        );
        // And the oracle routes it the same way:
        let resolution = resolver.resolve(input).unwrap();
        assert!(
            check(&resolution.plan),
            "{input}: the oracle routes differently"
        );
    }
}

/// Two candidates folding to the same condition merge into ONE layer
/// (the v0.x bucket semantics): `hover:bg-red-500 hover:px-8` is one
/// hover layer carrying both the colour and the per-edge padding.
#[test]
fn same_condition_candidates_merge_into_one_layer() {
    let theme = TailwindTheme::from_theme_css(TAILWIND_DEFAULT_THEME).unwrap();
    let resolver = Resolver::new(&theme);
    let source = ThemeSource::from_css(TAILWIND_DEFAULT_THEME, None).unwrap();
    let tokens = TokenTable::baked(&source).unwrap();

    let input = "hover:bg-red-500 hover:px-8";
    let sheet = emit_plan_theme(input, &tokens, &Config::default()).expect("the emit refused");
    assert!(sheet.diagnostics.is_empty() && sheet.notes.is_empty());
    assert_eq!(sheet.plan.layers.len(), 1, "one hover layer, not two");
    let layer = &sheet.plan.layers[0];
    assert!(layer.style.background.is_some(), "the colour landed");
    assert!(layer.style.padding.is_some(), "the padding landed");

    let resolution = resolver.resolve(input).unwrap();
    assert_eq!(resolution.plan.layers.len(), 1);
    assert_plans_agree(&sheet.plan, &resolution.plan);
}

// ---------------------------------------------------------------------------
// The honest boundary set: the typed notes
// ---------------------------------------------------------------------------

/// The longhands the v0.x plan cannot carry come back as TYPED notes
/// with stable codes — never silent drops. The gradient and shadow
/// preset rows lower into the IR fine; the plan shape is where they
/// stop, and the note says so.
#[test]
fn unmappable_families_report_typed_notes() {
    let source = ThemeSource::from_css(TAILWIND_DEFAULT_THEME, None).unwrap();
    let tokens = TokenTable::baked(&source).unwrap();

    // `shadow-sm` lowers into the IR (BoxShadow preset); the v0.x plan
    // slot wants a parsed BoxShadow, the row carries the preset text —
    // the note names the boundary.
    let sheet = emit_plan_theme("shadow-sm", &tokens, &Config::default()).unwrap();
    assert!(sheet.diagnostics.is_empty(), "the class itself lowered");
    assert_eq!(sheet.notes.len(), 1, "one note for the one class");
    let note = &sheet.notes[0];
    assert!(matches!(note.kind, PlanNoteKind::ShadowPreset), "{note:?}");
    assert!(!note.kind.code().is_empty());
    assert!(!note.kind.help().is_empty());

    // `bg-linear-to-r` lowers into the IR (BackgroundImage); the v0.x
    // Gradient slot wants composed stops — the note names it.
    let sheet = emit_plan_theme("bg-linear-to-r", &tokens, &Config::default()).unwrap();
    assert!(sheet.diagnostics.is_empty());
    assert_eq!(sheet.notes.len(), 1);
    assert!(matches!(sheet.notes[0].kind, PlanNoteKind::GradientRow));

    // `outline-dashed` lowers (OutlineStyle); the v0.x Outline slot has
    // no style field — the slot note.
    let sheet = emit_plan_theme("outline-dashed", &tokens, &Config::default()).unwrap();
    assert!(sheet.diagnostics.is_empty());
    assert_eq!(sheet.notes.len(), 1);
    assert!(matches!(
        sheet.notes[0].kind,
        PlanNoteKind::SlotUnsupported { .. }
    ));
}

/// The theme-less arm lowers every non-token utility and refuses the
/// colour tokens with the SAME diagnostics `compile` produces (the
/// walks share the boundary set — the palette keywords are `var()`
/// references, the token path's inputs). The notes still fire.
#[test]
fn themeless_arm_keeps_the_compile_diagnostics() {
    let sheet = emit_plan("p-4 hover:px-8", &Config::default()).unwrap();
    assert!(sheet.diagnostics.is_empty());
    assert!(sheet.notes.is_empty());

    // The colour-token boundary, diagnostics-equal with `compile`:
    let sheet = emit_plan("bg-red-500", &Config::default()).unwrap();
    let compiled = compile("bg-red-500", &Config::default()).unwrap();
    assert!(
        !sheet.diagnostics.is_empty(),
        "a token value without the theme pass is the typed boundary"
    );
    assert_eq!(
        sheet.diagnostics, compiled.diagnostics,
        "the plan arm's diagnostics are the IR walk's"
    );
}

/// The custom-authoring seam rides the plan arm: a registered custom
/// utility lowers into the plan exactly like a registry row.
#[test]
fn custom_utilities_lower_into_the_plan() {
    let customs = Customs::new().utility(CustomUtility::fixed(
        "brand-box",
        aeolus_ir::Val::Rem(1.5),
        &[Prop::PaddingTop, Prop::PaddingBottom],
    ));
    let sheet = emit_plan_with("brand-box", &customs, &Config::default()).unwrap();
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    let padding = sheet.plan.base.padding.expect("the custom utility lowered");
    assert_eq!(padding.top, 24.0);
    assert_eq!(
        padding.set,
        EdgeSet {
            top: true,
            right: false,
            bottom: true,
            left: false
        }
    );

    // The theme pass rides the customs too (`emit_plan_theme_with`):
    // a fixed-literal custom value is theme-independent, the walk
    // identical.
    let source = ThemeSource::shadcn().unwrap();
    let tokens = TokenTable::baked(&source).unwrap();
    let themed = emit_plan_theme_with("brand-box", &tokens, &customs, &Config::default()).unwrap();
    assert!(themed.diagnostics.is_empty(), "{:?}", themed.diagnostics);
    assert_eq!(
        themed.plan.base.padding.expect("the themed custom lowered"),
        padding,
        "a fixed-literal custom is theme-independent"
    );
}

/// The ergonomic determinism contract: equal inputs produce equal
/// sheets (the plan artifact is byte-comparable, like every emit form).
#[test]
fn equal_inputs_produce_equal_sheets() {
    let source = ThemeSource::from_css(TAILWIND_DEFAULT_THEME, None).unwrap();
    let tokens = TokenTable::baked(&source).unwrap();
    let a = emit_plan_theme("p-4 bg-red-500 hover:px-8", &tokens, &Config::default()).unwrap();
    let b = emit_plan_theme("p-4 bg-red-500 hover:px-8", &tokens, &Config::default()).unwrap();
    assert_eq!(a, b);
}

/// The arm surface: `Target::Plan` completes the emit family (the
/// tests pin the five-arm set beside the other arms' own pins).
#[test]
fn target_plan_completes_the_arm_surface() {
    // Compile-time proof the variant exists and names well:
    let target = Target::Plan;
    assert_eq!(format!("{target:?}"), "Plan");
    let _ = PlanSheet {
        plan: StylePlan::default(),
        diagnostics: Vec::new(),
        notes: Vec::new(),
    };
    // The Cond → bucket routing is total over the folded surface: the
    // base cond routes to the base bucket.
    assert_eq!(Cond::Base.specificity(), 0);
}
