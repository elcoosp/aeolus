//! The `variants!` compound-variant contracts (the §9 open question's
//! decision —"does `variants!` need slot-based multi-part components
//! (`cva`"compound variants", `slots`)?"): a `compound:` LIST applies
//! extra classes to every combo that matches ALL of an entry's
//! constraints — the cva shape, resolved at EXPANSION time (the entries
//! fold into the cartesian match arms the macro already renders, zero
//! runtime cost beyond the dispatch the table always had).
//!
//! The pins: a compound entry rides every combo it constrains and none
//! it does not, a one-group constraint applies across the other groups,
//! multiple entries compose in order, and the plain table is unchanged.
//!
//! The SLOTS half pins the multi-part form (the same §9 row's"slots"
//! word, the tailwind-variants shape): a `slots:` map declares one base
//! per part and generates a `Slot` enum plus a `style_slot(slot, …)`
//! dispatch — a group value that is a STRING applies to every slot, a
//! brace `{ slot:"classes"}` applies to the slots it names, and the
//! compound `class:` takes both forms the same way. Written FIRST (the
//! RED spec): the suite fails while `slots:` is no part of the macro
//! grammar.

use aeolus_gpui::prelude::*;

variants! { pub Badge {
    base:"inline-flex items-center rounded-md px-2 py-1 text-xs font-medium",
    variant: {
        default:"bg-primary text-primary-foreground",
        outline:"border border-gray-200 text-foreground",
    },
    size: { default:"h-9 px-4", sm:"h-8 px-3"},
    compound: [
        { variant: outline, size: sm, class:"ring-2 ring-red-500"},
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
    base:"flex items-center gap-2 rounded-md px-4 py-2 text-sm",
    variant: { default:"bg-primary text-primary-foreground", outline:"border text-foreground"},
    size: { default:"h-9", sm:"h-8"},
    compound: [
        { variant: outline, class:"ring-2 ring-blue-500"},
        { variant: outline, size: sm, class:"w-24"},
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
    // size carries the extra width (its entry constrains both groups).
    for size in [BannerSize::Default, BannerSize::Sm] {
        assert!(
            has_prop(Banner::style(BannerVariant::Outline, size), Prop::RingWidth),
            "the one-group constraint rides every size: {size:?}"
        );
    }
    assert!(
        has_prop(
            Banner::style(BannerVariant::Outline, BannerSize::Sm),
            Prop::Width
        ),
        "the two-group entry rides its combo"
    );
    assert!(
        !has_prop(
            Banner::style(BannerVariant::Outline, BannerSize::Default),
            Prop::Width
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
    base:"inline-flex items-center rounded-full px-2 text-xs",
    label: { solid:"bg-primary text-primary-foreground", soft:"bg-secondary text-secondary-foreground"},
    compound: [
        { label: solid, class:"ring-1 ring-primary"},
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
    base:"inline-flex items-center rounded-md px-4 py-2",
} }

#[test]
fn the_base_only_table_still_compiles() {
    // The pre-compound grammar keeps its shape (the static_program
    // suite's own pins cover the grouped forms; this pins the bare one
    // beside the new field).
    let program = Plain::style();
    assert!(!program.layers.is_empty());
}

// ---------------------------------------------------------------------------
// The slots half: the multi-part form (the same §9 row's"slots"word)
// ---------------------------------------------------------------------------

variants! { pub Card {
    slots: {
        root:"flex flex-col gap-6 rounded-xl border py-6",
        header:"flex flex-col gap-1.5 px-6",
        title:"font-semibold leading-none",
    },
    variant: {
        default:"text-card-foreground",
        outline: { root:"border-gray-200", title:"text-muted-foreground"},
    },
    size: { default:"text-sm", lg:"text-lg"},
} }

#[test]
fn a_slotted_table_dispatches_per_slot() {
    use aeolus_ir::Prop;

    let has_prop = |program: StaticProgram, prop: Prop| {
        program
            .layers
            .iter()
            .flat_map(|layer| layer.decls.iter())
            .any(|decl| decl.prop == prop)
    };

    // The title slot carries ITS base's typography and none of the
    // root slot's layout (each slot is its own program — the multi-part
    // point of the form).
    let title = Card::style_slot(CardSlot::Title, CardVariant::Default, CardSize::Default);
    assert!(
        has_prop(title, Prop::FontWeight),
        "the title slot carries its own base's font weight"
    );
    assert!(
        !has_prop(title, Prop::PaddingTop),
        "the title slot carries none of the root slot's padding"
    );

    let root = Card::style_slot(CardSlot::Root, CardVariant::Default, CardSize::Default);
    assert!(
        has_prop(root, Prop::PaddingTop),
        "the root slot carries its own base's padding"
    );
    assert!(
        !has_prop(root, Prop::FontWeight),
        "the root slot carries none of the title slot's font weight"
    );
}

#[test]
fn a_group_string_applies_to_every_slot_and_a_brace_to_the_slots_it_names() {
    use aeolus_ir::Prop;

    let has_prop = |program: StaticProgram, prop: Prop| {
        program
            .layers
            .iter()
            .flat_map(|layer| layer.decls.iter())
            .any(|decl| decl.prop == prop)
    };

    // The `default` variant's value is a STRING: the colour rides every
    // slot, including the header slot whose own base carries no colour.
    let header = Card::style_slot(CardSlot::Header, CardVariant::Default, CardSize::Default);
    assert!(
        has_prop(header, Prop::Color),
        "the string form applies to every slot (header carries the colour)"
    );

    // The `outline` variant's value is a BRACE: only the named slots
    // take their classes (the header is unnamed — it stays as-is).
    let root_outline = Card::style_slot(CardSlot::Root, CardVariant::Outline, CardSize::Default);
    assert!(
        has_prop(root_outline, Prop::BorderColor),
        "the brace form applies to the slot it names (root's border colour)"
    );
    let header_outline =
        Card::style_slot(CardSlot::Header, CardVariant::Outline, CardSize::Default);
    assert!(
        !has_prop(header_outline, Prop::BorderColor),
        "the brace form skips the slots it does not name"
    );
    let title_outline = Card::style_slot(CardSlot::Title, CardVariant::Outline, CardSize::Default);
    assert!(
        has_prop(title_outline, Prop::Color),
        "the title slot's own brace colour rides with the string's"
    );
}

variants! { pub Notice {
    slots: { body:"rounded-md px-4 py-3 text-sm", icon:"shrink-0"},
    tone: {
        info:"bg-blue-500",
        warn: { body:"bg-amber-500", icon:"text-amber-500"},
    },
    compound: [
        { tone: info, class: { body:"ring-2 ring-blue-500"} },
        { tone: warn, class:"w-24"},
    ],
} }

#[test]
fn a_compound_class_takes_both_forms_per_slot() {
    use aeolus_ir::Prop;

    let has_prop = |program: StaticProgram, prop: Prop| {
        program
            .layers
            .iter()
            .flat_map(|layer| layer.decls.iter())
            .any(|decl| decl.prop == prop)
    };

    // The brace compound class rides ONLY the slot it names: the body
    // takes the ring, the icon does not.
    let body_info = Notice::style_slot(NoticeSlot::Body, NoticeTone::Info);
    let icon_info = Notice::style_slot(NoticeSlot::Icon, NoticeTone::Info);
    assert!(
        has_prop(body_info, Prop::RingWidth),
        "the brace compound class rides the slot it names (body)"
    );
    assert!(
        !has_prop(icon_info, Prop::RingWidth),
        "the brace compound class skips the slots it does not name (icon)"
    );

    // The string compound class rides EVERY slot: both parts take the
    // extra width.
    let body_warn = Notice::style_slot(NoticeSlot::Body, NoticeTone::Warn);
    let icon_warn = Notice::style_slot(NoticeSlot::Icon, NoticeTone::Warn);
    assert!(
        has_prop(body_warn, Prop::Width) && has_prop(icon_warn, Prop::Width),
        "the string compound class applies to every slot"
    );
}

variants! { pub Panel {
    slots: { shell:"rounded-lg border bg-card", inner:"p-4"},
} }

#[test]
fn the_groupless_slotted_table_and_the_first_slot_default() {
    use aeolus_ir::Prop;

    let has_prop = |program: StaticProgram, prop: Prop| {
        program
            .layers
            .iter()
            .flat_map(|layer| layer.decls.iter())
            .any(|decl| decl.prop == prop)
    };

    // A slotted table with no groups dispatches on the slot alone, and
    // the slot enum's default is the FIRST declared slot (the shell).
    assert_eq!(
        PanelSlot::default(),
        PanelSlot::Shell,
        "first slot defaults"
    );
    let shell = Panel::style_slot(PanelSlot::Shell);
    let inner = Panel::style_slot(PanelSlot::Inner);
    assert!(
        has_prop(shell, Prop::Background) && !has_prop(shell, Prop::PaddingTop),
        "the shell slot carries its own base only"
    );
    assert!(
        has_prop(inner, Prop::PaddingTop) && !has_prop(inner, Prop::Background),
        "the inner slot carries its own base only"
    );
}
