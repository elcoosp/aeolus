//! The coverage guard (plan §3.2/§5.3): feeds every Tailwind v4 utility
//! family (a representative class per family/shape) and asserts the result
//! is NEVER `Warning::Unknown`. A family must resolve to a typed slot or
//! produce a precise `Unsupported` with a reason — this closes the
//! "recognized but undefined" hole permanently.

use aeolus_core::*;

fn theme() -> TailwindTheme {
    TailwindTheme::from_theme_css(shadcn_css::LIGHT).unwrap()
}

/// Every v4 utility family, with a representative value shape.
const V4_FAMILIES: &[&str] = &[
    // colors
    "bg-primary",
    "bg-red-500",
    "bg-[#fff]",
    "bg-(--brand)",
    "bg-transparent",
    "text-primary",
    "text-red-500",
    "border-primary",
    "ring-ring",
    "ring-offset-background",
    "outline-primary",
    "shadow-red-500",
    "decoration-primary",
    "accent-primary",
    "caret-primary",
    "fill-primary",
    "stroke-primary",
    // spacing + sizing
    "p-4",
    "px-4",
    "py-4",
    "pt-4",
    "pr-4",
    "pb-4",
    "pl-4",
    "ps-4",
    "pe-4",
    "m-4",
    "mx-4",
    "my-4",
    "mt-4",
    "mr-4",
    "mb-4",
    "ml-4",
    "ms-4",
    "me-4",
    "mx-auto",
    "m-auto",
    "ml-auto",
    "w-4",
    "h-4",
    "size-4",
    "w-full",
    "w-fit",
    "w-auto",
    "w-px",
    "w-screen",
    "w-1/2",
    "h-dvh",
    "min-w-0",
    "min-h-0",
    "max-w-xl",
    "max-h-xl",
    "min-w-full",
    "max-w-7xl",
    "basis-4",
    "basis-1/3",
    "gap-4",
    "gap-x-2",
    "gap-y-2",
    "space-x-4",
    // layout
    "flex",
    "inline-flex",
    "flex-col",
    "flex-row",
    "flex-wrap",
    "flex-nowrap",
    "grow",
    "shrink",
    "grid",
    "grid-cols-3",
    "grid-rows-2",
    "col-span-2",
    "row-span-2",
    "col-start-1",
    "row-start-1",
    "auto-rows-min",
    "items-center",
    "items-start",
    "justify-center",
    "justify-between",
    "self-center",
    "content-center",
    "order-1",
    "order-first",
    "basis-px",
    "aspect-square",
    "aspect-video",
    // position
    "static",
    "fixed",
    "absolute",
    "relative",
    "sticky",
    "inset-0",
    "inset-x-2",
    "inset-y-2",
    "top-4",
    "bottom-4",
    "left-4",
    "right-4",
    "top-1/2",
    "start-2",
    "end-2",
    "z-10",
    "z-auto",
    "overflow-hidden",
    "overflow-x-auto",
    "overflow-y-scroll",
    "overflow-clip",
    "visible",
    "invisible",
    // typography
    "text-sm",
    "text-base",
    "text-2xl",
    "text-primary",
    "font-medium",
    "font-sans",
    "font-serif",
    "font-mono",
    "leading-tight",
    "leading-6",
    "tracking-tight",
    "tracking-wide",
    "underline-offset-4",
    "whitespace-nowrap",
    "text-left",
    "text-center",
    "italic",
    "not-italic",
    "uppercase",
    "lowercase",
    "capitalize",
    "normal-case",
    "underline",
    "overline",
    "line-through",
    "no-underline",
    "truncate",
    "text-ellipsis",
    "text-clip",
    "line-clamp-2",
    "tabular-nums",
    "ordinal",
    "select-none",
    "select-text",
    // effects
    "shadow",
    "shadow-sm",
    "shadow-md",
    "shadow-none",
    "inset-shadow-sm",
    "transition",
    "transition-all",
    "transition-colors",
    "transition-opacity",
    "transition-shadow",
    "transition-transform",
    "transition-none",
    "duration-300",
    "delay-100",
    "ease-in",
    "ease-out",
    "ease-linear",
    "ease-in-out",
    "animate-spin",
    "animate-pulse",
    "animate-none",
    "scale-105",
    "scale-x-50",
    "rotate-45",
    "translate-x-4",
    "translate-y-1/2",
    "skew-x-6",
    "border",
    "border-2",
    "border-t",
    "border-s",
    "border-e",
    "border-x",
    "border-y",
    "border-dashed",
    "border-dotted",
    "border-solid",
    "rounded",
    "rounded-md",
    "rounded-t-lg",
    "rounded-s-lg",
    "rounded-e-lg",
    "rounded-ss-xl",
    "rounded-se-xl",
    "rounded-ee-xl",
    "rounded-es-xl",
    "rounded-tl-xl",
    "rounded-full",
    "ring",
    "ring-2",
    "ring-inset",
    "outline",
    "outline-2",
    "outline-none",
    "outline-offset-2",
    "opacity-50",
    // interaction
    "cursor-pointer",
    "cursor-wait",
    "cursor-grab",
    "cursor-not-allowed",
    "cursor-zoom-in",
    "pointer-events-none",
    "pointer-events-auto",
    // misc
    "resize",
    "appearance-none",
    "invisible",
    "sr-only",
];

#[test]
fn every_v4_family_resolves_or_reports_precisely() {
    let theme = theme();
    let resolver = Resolver::new(&theme);
    let mut unknowns = Vec::new();
    for class in V4_FAMILIES {
        let r = resolver.resolve(class).unwrap();
        for w in &r.warnings {
            if let Warning::Unknown {
                class: c,
                suggestion,
            } = w
            {
                unknowns.push(format!(
                    "{c} (suggestion {suggestion:?}) — no typed slot, no precise reason"
                ));
            }
            // Unsupported MUST carry a specific reason (non-empty).
            if let Warning::Unsupported { reason, .. } = w {
                assert!(!reason.is_empty(), "{class}: Unsupported without a reason");
            }
        }
    }
    assert!(
        unknowns.is_empty(),
        "coverage guard violated — these v4 families resolved as Unknown:\n{}",
        unknowns.join("\n")
    );
}

/// The plan slots the families land in — spot-check the v0.2 additions.
#[test]
fn v02_slots_are_populated() {
    let theme = theme();
    let resolver = Resolver::new(&theme);

    let r = resolver.resolve("mx-auto absolute z-10 overflow-hidden invisible aspect-video grid-cols-3 order-2 basis-4 scale-110 rotate-45 translate-x-4").unwrap();
    let b = &r.plan.base;
    assert_eq!(
        b.extras.margin_auto,
        Some(AutoEdges {
            left: true,
            right: true,
            top: false,
            bottom: false
        })
    );
    assert_eq!(b.extras.position, Some(Position::Absolute));
    assert_eq!(b.extras.z_index, Some(10));
    assert_eq!(
        b.extras.overflow,
        Some(Overflow {
            x: Some(OverflowKind::Hidden),
            y: Some(OverflowKind::Hidden)
        })
    );
    assert_eq!(b.extras.visibility, Some(Visibility::Hidden));
    assert!((b.extras.aspect_ratio.unwrap() - 16.0 / 9.0).abs() < 1e-4);
    assert_eq!(b.extras.grid.as_ref().and_then(|g| g.cols), Some(3));
    assert_eq!(b.extras.order, Some(2));
    assert_eq!(b.extras.flex_basis, Some(SizedValue::Px(16.0)));
    let t = b.extras.transform.as_ref().expect("transform");
    assert_eq!(t.scale_x, Some(1.1));
    assert_eq!(t.rotate_deg, Some(45.0));
    assert_eq!(t.translate_x, Some(SizedValue::Px(16.0)));
    // transition + duration from the default theme (150ms).
    let r = resolver.resolve("transition-colors duration-300").unwrap();
    let tr = r.plan.base.extras.transition.as_ref().expect("transition");
    assert_eq!(tr.set, TransitionSet::Colors);
    assert_eq!(tr.duration_ms, 300.0);
    // gap axes.
    let r = resolver.resolve("gap-x-2 gap-y-3").unwrap();
    assert_eq!(
        r.plan.base.extras.gap_axes,
        Some(GapAxes {
            col: Some(8.0),
            row: Some(12.0)
        })
    );
    // border styles + logical sides.
    let r = resolver.resolve("border-dashed border-e-2").unwrap();
    assert_eq!(r.plan.base.extras.border_style, Some(BorderStyle::Dashed));
    assert_eq!(
        r.plan.base.border_width.as_ref().map(|b| b.right),
        Some(2.0)
    );
    // ring-inset + outline-offset (v4 px semantics: offset-2 = 2px).
    let r = resolver.resolve("ring-inset outline-offset-2").unwrap();
    assert_eq!(r.plan.base.ring.map(|r| r.inset), Some(true));
    assert_eq!(r.plan.base.outline.map(|o| o.offset), Some(2.0));
}

/// The theme ships the v4 default scales the plan requires (§3.3).
#[test]
fn theme_scales_are_present() {
    let t = TailwindTheme::new();
    // Container scale (max-w-xl = 36rem = 576px).
    assert_eq!(t.container_px("xl"), Some(576.0));
    // Breakpoints are theme-aware.
    assert_eq!(t.breakpoint_px(Breakpoint::Md), 768.0);
    assert_eq!(t.breakpoints_px()[1], 768.0);
    // Multi-layer shadows.
    assert_eq!(t.shadows("sm").len(), 2);
    assert_eq!(t.shadows("2xl").len(), 1);
    // Easings/animations.
    assert!(t.animation("bounce").is_some());
    assert_eq!(t.ease("in"), Some(Ease::In));
}

/// `@utility` expansion works end-to-end (depth-limited).
#[test]
fn custom_utilities_expand() {
    let theme = TailwindTheme::from_theme_css(
        "@utility tab-nums { @apply tabular-nums tracking-tight; }\n@utility double { @apply tab-nums uppercase; }",
    )
    .unwrap();
    let r = Resolver::new(&theme).resolve("tab-nums").unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    assert!(r.plan.base.text.contains(&(
        TextProperty::FontVariantNumeric,
        TextValue::Keyword("tabular-nums")
    )));
    assert!(
        r.plan
            .base
            .text
            .contains(&(TextProperty::LetterSpacing, TextValue::Spacing(-0.4)))
    );
    // Two-level expansion (double → tab-nums → tabular-nums) + uppercase.
    let r = Resolver::new(&theme).resolve("double").unwrap();
    assert!(
        r.plan
            .base
            .text
            .contains(&(TextProperty::TextTransform, TextValue::Keyword("uppercase")))
    );
}

/// `@custom-variant` maps a name onto the dark strategy (§3.3).
#[test]
fn custom_variant_dark() {
    let theme = TailwindTheme::from_theme_css(
        ":root { --primary: oklch(0.205 0 0); } @theme inline { --color-primary: var(--primary); } @custom-variant dark (&:where(.dark, .dark *));",
    )
    .unwrap();
    let r = Resolver::new(&theme).resolve("dark:bg-primary").unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    assert!(
        r.plan.dark.is_some(),
        "custom dark variant maps to Media::Dark"
    );
}

/// Paren-shorthand arbitrary values (`bg-(--brand)`, `p-(--x)`) resolve
/// through the theme (§1.3 coverage gaps).
#[test]
fn paren_shorthand_arbitrary() {
    let theme = TailwindTheme::from_theme_css(
        ":root { --brand: oklch(0.5 0.1 20); --sp: 8px; } @theme inline { --color-brand: var(--brand); }",
    )
    .unwrap();
    let r = Resolver::new(&theme).resolve("bg-(--brand)").unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let bg = r.plan.base.background.expect("bg-(--brand)");
    assert!((bg.0.l - 0.5).abs() < 1e-3);
    let r = Resolver::new(&theme).resolve("p-(--sp)").unwrap();
    assert_eq!(
        r.plan.base.padding,
        Some(Padding {
            top: 8.0,
            right: 8.0,
            bottom: 8.0,
            left: 8.0
        })
    );
}

/// Bracket var() arbitrary values resolve through the theme too.
#[test]
fn bracket_var_arbitrary() {
    let theme = TailwindTheme::from_theme_css(
        ":root { --brand: oklch(0.5 0.1 20); } @theme inline { --color-brand: var(--brand); }",
    )
    .unwrap();
    let r = Resolver::new(&theme).resolve("bg-[var(--brand)]").unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    assert!(r.plan.base.background.is_some());
}
