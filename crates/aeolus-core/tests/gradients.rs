//! Gradient-background semantics (v0.6 §1/§2, plan `AEOLUS_V0.6.md`).
//!
//! Every composed value below is pinned against the REAL tailwindcss
//! 4.1.17 output captured by `tools/capture_gradients.mjs`:
//! - stop positions default to the `@property` initial values
//!   (from 0%, via 50%, to 100%);
//! - a missing `from`/`to` stop is `#0000` (transparent);
//! - `bg-linear-*` interpolates `in oklab`;
//! - `from-{P}%` patches ONLY the stop position;
//! - `bg-gradient-to-*` is the deprecated alias of `bg-linear-to-*`.

use aeolus_core::*;
use plan::{Gradient, GradientPart, GradientStop, Slot};

fn theme() -> TailwindTheme {
    TailwindTheme::default()
}

fn base(class: &str) -> ResolvedStyle {
    let t = theme();
    let resolver = Resolver::new(&t);
    let resolution = resolver.resolve(class).expect("parse");
    assert!(
        resolution.warnings.is_empty(),
        "unexpected warnings: {:?}",
        resolution.warnings
    );
    resolution.plan.base
}

fn color_of(name: &str, css: &str) -> Color {
    let mut t = TailwindTheme::default();
    t.absorb_css(css).unwrap();
    let resolver = Resolver::new(&t);
    let resolution = resolver.resolve(class_of(name)).expect("parse");
    resolution
        .plan
        .base
        .content_color
        .map(|c| c.0)
        .expect("color slot")
}

fn class_of(name: &str) -> &'static str {
    match name {
        "red-500" => "text-red-500",
        "blue-500" => "text-blue-500",
        "yellow-400" => "text-yellow-400",
        _ => unreachable!(),
    }
}

// The v4 default palette values (oklch, verbatim from the capture).
const RED_500: &str = "--color-red-500: oklch(63.7% 0.237 25.331);";
const BLUE_500: &str = "--color-blue-500: oklch(62.3% 0.214 259.815);";
const YELLOW_400: &str = "--color-yellow-400: oklch(85.2% 0.199 91.936);";

fn themed_theme(extra: &str) -> TailwindTheme {
    let mut t = TailwindTheme::default();
    t.absorb_css(extra).unwrap();
    t
}

// ---------------------------------------------------------------------------
// shapes
// ---------------------------------------------------------------------------

#[test]
fn linear_corners_are_css_angles() {
    // Captured: `--tw-gradient-position: to right` (+ `in oklab` in the
    // @supports branch) — the angle form of the same direction.
    for (class, deg) in [
        ("bg-linear-to-t", 0.0),
        ("bg-linear-to-tr", 45.0),
        ("bg-linear-to-r", 90.0),
        ("bg-linear-to-br", 135.0),
        ("bg-linear-to-b", 180.0),
        ("bg-linear-to-bl", 225.0),
        ("bg-linear-to-l", 270.0),
        ("bg-linear-to-tl", 315.0),
    ] {
        let s = base(class);
        match s.background_gradient {
            Some(Gradient::Linear { angle_deg, .. }) => {
                assert_eq!(angle_deg, deg, "{class}");
            }
            other => panic!("{class} → {:?}, want Linear", other),
        }
    }
}

#[test]
fn linear_numeric_and_arbitrary_angles() {
    let s = base("bg-linear-45");
    assert!(matches!(s.background_gradient,
        Some(Gradient::Linear { angle_deg, .. }) if angle_deg == 45.0));
    let s = base("bg-linear-[200deg]");
    assert!(matches!(s.background_gradient,
        Some(Gradient::Linear { angle_deg, .. }) if angle_deg == 200.0));
    // bare `bg-linear` is not a v4 utility
    let t = theme();
    let resolver = Resolver::new(&t);
    let r = resolver.resolve("bg-linear").unwrap();
    assert!(matches!(r.warnings[0], Warning::Unknown { .. }));
}

#[test]
fn gradient_to_is_the_deprecated_alias() {
    // Captured: `.bg-gradient-to-r { --tw-gradient-position: to right in oklab; … }`
    let alias = base("bg-gradient-to-r");
    let modern = base("bg-linear-to-r");
    assert_eq!(alias.background_gradient, modern.background_gradient);
}

#[test]
fn radial_and_conic_shapes() {
    let s = base("bg-radial");
    assert!(matches!(
        s.background_gradient,
        Some(Gradient::Radial { .. })
    ));
    let s = base("bg-conic");
    assert!(matches!(s.background_gradient,
        Some(Gradient::Conic { from_angle_deg, .. }) if from_angle_deg == 0.0));
    let s = base("bg-conic-45");
    assert!(matches!(s.background_gradient,
        Some(Gradient::Conic { from_angle_deg, .. }) if from_angle_deg == 45.0));
}

#[test]
fn bg_none_clears_like_a_value() {
    let s = base("bg-linear-to-r from-red-500 to-blue-500 bg-none");
    assert_eq!(s.background_gradient, Some(Gradient::None));
    // …and per layer: `dark:bg-none` clears the DARK layer's image while
    // the base keeps its own (each bucket composes independently).
    let t = theme();
    let resolver = Resolver::new(&t);
    let r = resolver
        .resolve("bg-linear-to-r from-red-500 to-blue-500 dark:bg-none")
        .unwrap();
    assert!(r.warnings.is_empty());
    assert!(matches!(
        r.plan.base.background_gradient,
        Some(Gradient::Linear { .. })
    ));
    let dark = r.plan.dark.clone().expect("dark layer exists");
    assert_eq!(dark.background_gradient, Some(Gradient::None));
}

#[test]
fn solid_and_gradient_coexist() {
    // Captured: `.bg-red-500` emits `background-color`, `.bg-linear-to-r`
    // emits `background-image` — separate declarations, both apply.
    let s = base("bg-red-500 bg-linear-to-r from-red-500 to-blue-500");
    assert!(s.background.is_some(), "solid survives");
    assert!(s.background_gradient.is_some(), "gradient coexists");
    // …in either class order.
    let s = base("bg-linear-to-r from-red-500 to-blue-500 bg-red-500");
    assert!(s.background.is_some() && s.background_gradient.is_some());
}

// ---------------------------------------------------------------------------
// stops
// ---------------------------------------------------------------------------

#[test]
fn stops_compose_with_v4_defaults() {
    let t = themed_theme(&format!("{RED_500}{BLUE_500}"));
    let resolver = Resolver::new(&t);
    let r = resolver
        .resolve("bg-linear-to-r from-red-500 to-blue-500")
        .unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let Some(Gradient::Linear { angle_deg, stops }) = r.plan.base.background_gradient else {
        panic!("no composed linear gradient");
    };
    assert_eq!(angle_deg, 90.0);
    assert_eq!(
        stops.from,
        GradientStop {
            color: color_of("red-500", RED_500),
            pos_pct: None
        },
        "from = red-500 @ default 0%"
    );
    assert_eq!(stops.via, None);
    assert_eq!(
        stops.to,
        GradientStop {
            color: color_of("blue-500", BLUE_500),
            pos_pct: None
        },
        "to = blue-500 @ default 100%"
    );
}

#[test]
fn missing_to_stop_is_transparent() {
    // Captured @property: `--tw-gradient-to { initial-value: #0000; }`
    let t = themed_theme(RED_500);
    let resolver = Resolver::new(&t);
    let r = resolver.resolve("bg-linear-to-r from-red-500").unwrap();
    assert!(r.warnings.is_empty());
    match r.plan.base.background_gradient {
        Some(Gradient::Linear { stops, .. }) => {
            assert_eq!(stops.to.color, Color::TRANSPARENT, "missing to = #0000");
            assert_eq!(stops.from.color, color_of("red-500", RED_500));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn via_composes_between_from_and_to() {
    let t = themed_theme(&format!("{RED_500}{YELLOW_400}{BLUE_500}"));
    let resolver = Resolver::new(&t);
    let r = resolver
        .resolve("bg-linear-to-r from-red-500 via-yellow-400 to-blue-500")
        .unwrap();
    assert!(r.warnings.is_empty());
    match r.plan.base.background_gradient {
        Some(Gradient::Linear { stops, .. }) => {
            let via = stops.via.expect("via composed");
            assert_eq!(via.color, color_of("yellow-400", YELLOW_400));
            assert_eq!(via.pos_pct, None, "via default = 50% (pos_pct None)");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn position_patches_patch_only_the_position() {
    // Captured: `.from-20\% { --tw-gradient-from-position: 20%; }` — the
    // color var is untouched.
    let t = themed_theme(RED_500);
    let resolver = Resolver::new(&t);
    let r = resolver
        .resolve("bg-linear-to-r from-red-500 from-20% to-blue-500 to-80%")
        .unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    match r.plan.base.background_gradient {
        Some(Gradient::Linear { stops, .. }) => {
            assert_eq!(stops.from.color, color_of("red-500", RED_500));
            assert_eq!(stops.from.pos_pct, Some(20.0));
            assert_eq!(stops.to.pos_pct, Some(80.0));
        }
        other => panic!("{other:?}"),
    }
    // A position patch with no color composes the patch alone (v4 keeps
    // `--tw-gradient-from-position` independent of the color var).
    let r = resolver.resolve("bg-linear-to-r from-20%").unwrap();
    match r.plan.base.background_gradient {
        Some(Gradient::Linear { stops, .. }) => {
            assert_eq!(stops.from.color, Color::TRANSPARENT);
            assert_eq!(stops.from.pos_pct, Some(20.0));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn stop_opacity_modifier_uses_the_v4_color_mix_shape() {
    // Captured: `--tw-gradient-from: color-mix(in oklab, var(--color-red-500) 50%, transparent)`
    // — the same shape `with_opacity` produces for every other color slot.
    let t = themed_theme(RED_500);
    let resolver = Resolver::new(&t);
    let r = resolver.resolve("bg-linear-to-r from-red-500/50").unwrap();
    assert!(r.warnings.is_empty());
    match r.plan.base.background_gradient {
        Some(Gradient::Linear { stops, .. }) => {
            assert!(
                stops.from.color != color_of("red-500", RED_500),
                "alpha applied"
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn stops_without_a_shape_compose_nothing() {
    // v4: `--tw-gradient-position` unset → the gradient function has no
    // position → no image. Stops alone are carried but produce no gradient.
    let t = themed_theme(RED_500);
    let resolver = Resolver::new(&t);
    let r = resolver.resolve("from-red-500 to-blue-500").unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    assert_eq!(r.plan.base.background_gradient, None);
}

// ---------------------------------------------------------------------------
// the v0.6 floem unwired surface (§2) — resolver-side pieces live with the
// backends' lowering tests; here: the slots the new lowerings consume.
// ---------------------------------------------------------------------------

#[test]
fn overflow_slots_carry_per_axis_kinds() {
    let s = base("overflow-hidden overflow-x-auto");
    let o = s.extras.overflow.expect("overflow slot");
    assert_eq!(o.y, Some(plan::OverflowKind::Hidden));
    assert_eq!(o.x, Some(plan::OverflowKind::Auto));
}

#[test]
fn grid_slot_carries_template_and_placement() {
    let s = base("grid grid-cols-4 col-span-2 row-start-3");
    let g = s.extras.grid.expect("grid slot");
    assert_eq!(g.cols, Some(4));
    assert_eq!(g.col_span, Some(2));
    assert_eq!(g.row_start, Some(3));
}

#[test]
fn select_and_whitespace_keywords() {
    let s = base("select-none whitespace-normal whitespace-nowrap");
    let (_, v) = s
        .text
        .iter()
        .find(|(p, _)| *p == plan::TextProperty::Select)
        .unwrap();
    assert!(matches!(v, plan::TextValue::Keyword("select-none")));
    let (_, w) = s
        .text
        .iter()
        .find(|(p, _)| *p == plan::TextProperty::Whitespace)
        .unwrap();
    assert!(matches!(w, plan::TextValue::Keyword("nowrap")));
}

// ---------------------------------------------------------------------------
// diffs / lanes / conflict groups
// ---------------------------------------------------------------------------

#[test]
fn gradient_changes_diff_through_the_cosmetic_lane() {
    let a = base("bg-linear-to-r from-red-500 to-blue-500");
    let b = base("bg-linear-to-r from-blue-500 to-blue-500");
    let d = plan::diff(&a, &b);
    assert!(
        d.changed
            .iter()
            .any(|s| matches!(s, Slot::Gradient(GradientPart::Shape))),
        "{:?}",
        d.changed
    );
    for s in &d.changed {
        assert_eq!(s.lane(), plan::Lane::Cosmetic, "{s:?}");
    }
    // Shape flip (linear → radial) diffs too.
    let c = base("bg-radial from-red-500 to-blue-500");
    let d = plan::diff(&a, &c);
    assert!(d.changed.iter().any(|s| matches!(s, Slot::Gradient(_))));
}

#[test]
fn cn_keeps_solid_and_gradient_but_merges_per_group() {
    // different groups (bg-color / bg-image): BOTH survive, in either order.
    assert_eq!(
        cn(&["bg-red-500", "bg-linear-to-r"]),
        "bg-red-500 bg-linear-to-r"
    );
    assert_eq!(
        cn(&["bg-linear-to-r", "bg-red-500"]),
        "bg-linear-to-r bg-red-500"
    );
    // same image group: later shape wins.
    assert_eq!(cn(&["bg-linear-to-r", "bg-radial"]), "bg-radial");
    assert_eq!(cn(&["bg-radial", "bg-none"]), "bg-none");
    // per-edge color groups: later color wins.
    assert_eq!(cn(&["from-red-500", "from-blue-500"]), "from-blue-500");
    assert_eq!(cn(&["to-blue-500", "to-red-500"]), "to-red-500");
    // color + position COMPOSE (different groups): both survive.
    assert_eq!(cn(&["from-red-500", "from-20%"]), "from-red-500 from-20%");
}

// ---------------------------------------------------------------------------
// the verification emitter (§1.4) — v4-verbatim per-class shapes
// ---------------------------------------------------------------------------

fn one_rule(class: &str) -> CssRule {
    let t = themed_theme(&format!("{RED_500}{BLUE_500}{YELLOW_400}"));
    let out = Resolver::new(&t).to_css(class).unwrap();
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    out.rules.into_iter().next().expect("one rule")
}

#[test]
fn emitter_linear_shape_is_the_v4_var_machinery() {
    let r = one_rule("bg-linear-to-r");
    assert_eq!(
        r.declarations,
        vec![
            (
                "--tw-gradient-position".to_string(),
                "90deg in oklab".to_string()
            ),
            (
                "background-image".to_string(),
                "linear-gradient(var(--tw-gradient-stops))".to_string()
            ),
        ]
    );
}

#[test]
fn emitter_stop_classes_set_the_captured_chains() {
    let r = one_rule("from-red-500");
    assert_eq!(
        r.declarations,
        vec![
            // The resolved oklch (the emitter resolves like every other
            // color declaration; v4's `var(--color-red-500)` compares
            // equal after var substitution).
            (
                "--tw-gradient-from".to_string(),
                "oklch(0.637 0.237 25.330997)".to_string()
            ),
            (
                "--tw-gradient-stops".to_string(),
                "var(--tw-gradient-via-stops, var(--tw-gradient-position), \
var(--tw-gradient-from) var(--tw-gradient-from-position), \
var(--tw-gradient-to) var(--tw-gradient-to-position))"
                    .to_string()
            ),
        ]
    );
    let r = one_rule("via-yellow-400");
    assert_eq!(r.declarations[0].0, "--tw-gradient-via");
    assert_eq!(r.declarations[1].0, "--tw-gradient-via-stops");
    assert_eq!(
        r.declarations[2],
        (
            "--tw-gradient-stops".to_string(),
            "var(--tw-gradient-via-stops)".to_string()
        )
    );
    let r = one_rule("from-20%");
    assert_eq!(
        r.declarations,
        vec![("--tw-gradient-from-position".to_string(), "20%".to_string())]
    );
}

#[test]
fn emitter_bg_none_is_verbatim() {
    let r = one_rule("bg-none");
    assert_eq!(
        r.declarations,
        vec![("background-image".to_string(), "none".to_string())]
    );
}

#[test]
fn currentcolor_stops_report_precisely() {
    let t = theme();
    let resolver = Resolver::new(&t);
    let r = resolver.resolve("from-current").unwrap();
    assert!(matches!(r.warnings[0], Warning::Unsupported { .. }));
}
