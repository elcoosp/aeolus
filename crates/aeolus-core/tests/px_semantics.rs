//! px-vs-spacing-multiple semantics for the width/offset families, verified
//! against the tailwindcss v4.1.17 source (`packages/tailwindcss/src/utilities.ts`):
//!
//! * `ring-<number>` → `<number>px` (and bare `ring` → 1px);
//! * `ring-offset-<number>` → `<number>px`;
//! * bare `outline` → 1px, `outline-offset-<number>` → `<number>px`;
//! * `underline-offset-<number>` → `<number>px` (negatives supported);
//! * only the spacing families (`p-*`, `m-*`, `gap-*`, `leading-<number>`, …)
//!   are `--spacing` multiples.
//!
//! The pre-fix behavior scaled all of these by `--spacing` (4px), silently
//! mis-styling every ring/outline on the Shadcn focus idioms.

use aeolus_core::*;

fn theme() -> TailwindTheme {
    TailwindTheme::from_theme_css(shadcn_css::LIGHT).unwrap()
}

#[test]
fn ring_widths_are_plain_px() {
    let t = theme();
    let r = Resolver::new(&t).resolve("ring-2").unwrap();
    assert_eq!(
        r.plan.base.ring.unwrap().width,
        2.0,
        "ring-2 = 2px, not 8px"
    );

    let r = Resolver::new(&t).resolve("ring-4").unwrap();
    assert_eq!(r.plan.base.ring.unwrap().width, 4.0);

    // Bare `ring` = --default-ring-width fallback 1px.
    let r = Resolver::new(&t).resolve("ring").unwrap();
    assert_eq!(r.plan.base.ring.unwrap().width, 1.0);

    // Arbitrary widths stay absolute.
    let r = Resolver::new(&t).resolve("ring-[3px]").unwrap();
    assert_eq!(r.plan.base.ring.unwrap().width, 3.0);
}

#[test]
fn ring_offset_widths_are_plain_px() {
    let t = theme();
    let r = Resolver::new(&t).resolve("ring-offset-2").unwrap();
    assert_eq!(
        r.plan.base.ring.unwrap().offset_width,
        2.0,
        "ring-offset-2 = 2px, not 8px"
    );
}

#[test]
fn outline_widths_and_offsets_are_plain_px() {
    let t = theme();
    // Bare `outline` = --default-outline-width fallback 1px.
    let r = Resolver::new(&t).resolve("outline").unwrap();
    assert_eq!(r.plan.base.outline.unwrap().width, 1.0, "outline = 1px");

    let r = Resolver::new(&t).resolve("outline-2").unwrap();
    assert_eq!(r.plan.base.outline.unwrap().width, 2.0);

    let r = Resolver::new(&t).resolve("outline-offset-2").unwrap();
    assert_eq!(r.plan.base.outline.unwrap().offset, 2.0, "offset-2 = 2px");
    let r = Resolver::new(&t).resolve("outline-offset-4").unwrap();
    assert_eq!(r.plan.base.outline.unwrap().offset, 4.0);

    // Negatives are supported for offsets in v4.
    let r = Resolver::new(&t).resolve("-outline-offset-2").unwrap();
    assert_eq!(r.plan.base.outline.unwrap().offset, -2.0);
}

#[test]
fn underline_offsets_are_plain_px() {
    let t = theme();
    let r = Resolver::new(&t).resolve("underline-offset-4").unwrap();
    assert_eq!(
        r.plan.base.text,
        vec![(TextProperty::UnderlineOffset, TextValue::Px(4.0))],
        "underline-offset-4 = 4px, not 16px"
    );
    let r = Resolver::new(&t).resolve("underline-offset-2").unwrap();
    assert_eq!(r.plan.base.text[0].1, TextValue::Px(2.0));
    // Negatives are supported in v4.
    let r = Resolver::new(&t).resolve("-underline-offset-2").unwrap();
    assert_eq!(r.plan.base.text[0].1, TextValue::Px(-2.0));
}

/// The spacing families keep their `--spacing` multiples (guard against
/// overcorrecting the px fix).
#[test]
fn spacing_families_still_scale() {
    let t = theme();
    let r = Resolver::new(&t).resolve("p-2").unwrap();
    assert_eq!(r.plan.base.padding.unwrap().top, 8.0);
    let r = Resolver::new(&t).resolve("gap-2").unwrap();
    assert_eq!(r.plan.base.gap.unwrap().0, 8.0);
    let r = Resolver::new(&t).resolve("w-4").unwrap();
    assert!(matches!(
        r.plan.base.dimensions.unwrap().width,
        Some(SizedValue::Px(16.0))
    ));
}

/// The canonical v4 state cascade: when several state layers match, the
/// condition rank (hover < focus < focus-visible < focus-within < active <
/// disabled) decides — NOT the source order of the class string. Tailwind
/// sorts generated CSS by variant order, so `focus:` beats `hover:` when both
/// flags are set regardless of how the author listed the classes.
#[test]
fn state_layers_cascade_by_condition_rank_not_source_order() {
    use eval::Probe;

    let t = theme();
    // focus listed FIRST, hover second.
    let r = Resolver::new(&t)
        .resolve("focus:bg-blue-500 hover:bg-red-500")
        .unwrap();
    let probe = BothStatesProbe;
    let out = r.plan.resolve(&probe);
    assert_eq!(
        out.background.unwrap().0.to_oklch_string(),
        t.color("blue-500").unwrap().to_oklch_string(),
        "focus must beat hover even when hover is listed last"
    );

    // And the reverse listing resolves identically.
    let r = Resolver::new(&t)
        .resolve("hover:bg-red-500 focus:bg-blue-500")
        .unwrap();
    let out = r.plan.resolve(&probe);
    assert_eq!(
        out.background.unwrap().0.to_oklch_string(),
        t.color("blue-500").unwrap().to_oklch_string(),
    );

    // Disabled ranks after active: a disabled+active button shows disabled.
    let r = Resolver::new(&t)
        .resolve("active:bg-green-500 disabled:bg-gray-300")
        .unwrap();
    struct ActiveDisabledProbe;
    impl Probe for ActiveDisabledProbe {
        fn test(&self, t: &Test) -> bool {
            matches!(t, Test::State(State::Active) | Test::State(State::Disabled))
        }
    }
    let out = r.plan.resolve(&ActiveDisabledProbe);
    assert_eq!(
        out.background.unwrap().0.to_oklch_string(),
        t.color("gray-300").unwrap().to_oklch_string(),
        "disabled must beat active per the canonical variant order"
    );
}

/// Probe that reports the element as BOTH hovered and focused.
struct BothStatesProbe;
impl Probe for BothStatesProbe {
    fn test(&self, t: &Test) -> bool {
        matches!(t, Test::State(State::Hover) | Test::State(State::Focus))
    }
    fn related(&self, _rel: Rel, _label: Option<&str>, _t: &Test) -> bool {
        false
    }
}
