//! The `variants!` compound-variant contracts (the §9 open question's
//! decision — "does `variants!` need slot-based multi-part components
//! (`cva` "compound variants", `slots`)?"): a `compound:` LIST applies
//! extra classes to every combo that matches ALL of an entry's
//! constraints — the cva shape, resolved at EXPANSION time (the entries
//! fold into the cartesian match arms the macro already renders, zero
//! runtime cost beyond the dispatch the table always had).
//!
//! The pins: a compound entry rides every combo it constrains and none
//! it does not, a one-group constraint applies across the other groups,
//! multiple entries compose in order, and the plain table is unchanged.
//!
//! Written FIRST (the RED spec): the suite fails while `compound:` is
//! no part of the macro grammar.

use aeolus_gpui::prelude::*;

variants! { pub Badge {
    base: "inline-flex items-center rounded-md px-2 py-1 text-xs font-medium",
    variant: {
        default: "bg-primary text-primary-foreground",
        outline: "border border-gray-200 text-foreground",
    },
    size: { default: "h-9 px-4", sm: "h-8 px-3" },
    compound: [
        { variant: outline, size: sm, class: "ring-2 ring-red-500" },
    ],
} }

#[test]
fn the_compound_entry_rides_only_the_matching_combos() {
    use aeolus_ir::Prop;

    let has_ring = |program: StaticProgram| {
        program
            .layers
            .iter()
            .flat_map(|layer| layer.decls.iter())
            .any(|decl| decl.prop == Prop::RingWidth)
    };

    // (outline, sm) — the constrained combo: the ring rides.
    assert!(
        has_ring(Badge::style(BadgeVariant::Outline, BadgeSize::Sm)),
        "the matching combo carries the compound's classes"
    );
    // The other three combos: no ring.
    assert!(
        !has_ring(Badge::style(BadgeVariant::Default, BadgeSize::Sm)),
        "the wrong variant leaves the compound out"
    );
    assert!(
        !has_ring(Badge::style(BadgeVariant::Outline, BadgeSize::Default)),
        "the wrong size leaves the compound out"
    );
    assert!(
        !has_ring(Badge::style(BadgeVariant::Default, BadgeSize::Default)),
        "both wrong leaves the compound out"
    );
}

variants! { pub Banner {
    base: "flex items-center gap-2 rounded-md px-4 py-2 text-sm",
    variant: { default: "bg-primary text-primary-foreground", outline: "border text-foreground" },
    size: { default: "h-9", sm: "h-8" },
    compound: [
        { variant: outline, class: "ring-2 ring-blue-500" },
        { variant: outline, size: sm, class: "shadow-md" },
    ],
} }

#[test]
fn a_one_group_constraint_applies_across_the_other_groups() {
    use aeolus_ir::Prop;

    let has_prop = |program: StaticProgram, prop: Prop| {
        program
            .layers
            .iter()
            .flat_map(|layer| layer.decls.iter())
            .any(|decl| decl.prop == prop)
    };

    // `variant: outline` alone: BOTH sizes carry the ring; only the sm
    // size carries the shadow (its entry constrains both groups).
    for size in [BannerSize::Default, BannerSize::Sm] {
        assert!(
            has_prop(Banner::style(BannerVariant::Outline, size), Prop::RingWidth),
            "the one-group constraint rides every size: {size:?}"
        );
    }
    assert!(
        has_prop(
            Banner::style(BannerVariant::Outline, BannerSize::Sm),
            Prop::BoxShadow
        ),
        "the two-group entry rides its combo"
    );
    assert!(
        !has_prop(
            Banner::style(BannerVariant::Outline, BannerSize::Default),
            Prop::BoxShadow
        ),
        "the two-group entry stays out of the other size"
    );
    assert!(
        !has_prop(
            Banner::style(BannerVariant::Default, BannerSize::Sm),
            Prop::RingWidth
        ),
        "the default variant takes no compound class"
    );
}

variants! { pub Chip {
    base: "inline-flex items-center rounded-full px-2 text-xs",
    label: { solid: "bg-primary text-primary-foreground", soft: "bg-secondary text-secondary-foreground" },
    compound: [
        { label: solid, class: "ring-1 ring-primary" },
    ],
} }

#[test]
fn the_plain_grammar_is_unchanged_and_compound_names_any_group() {
    use aeolus_ir::Prop;

    // The compound's constraint keys are the table's OWN group names
    // (any of them, not just `variant`/`size`), and a table whose only
    // group carries compound entries compiles and dispatches as before.
    let solid = Chip::style(ChipLabel::Solid);
    let soft = Chip::style(ChipLabel::Soft);
    let has_ring = |program: StaticProgram| {
        program
            .layers
            .iter()
            .flat_map(|layer| layer.decls.iter())
            .any(|decl| decl.prop == Prop::RingWidth)
    };
    assert!(has_ring(solid), "the matching key takes the entry");
    assert!(!has_ring(soft), "the other key does not");
}

variants! { pub Plain {
    base: "inline-flex items-center rounded-md px-4 py-2",
} }

#[test]
fn the_base_only_table_still_compiles() {
    // The pre-compound grammar keeps its shape (the static_program
    // suite's own pins cover the grouped forms; this pins the bare one
    // beside the new field).
    let program = Plain::style();
    assert!(!program.layers.is_empty());
}
