//! The class-sorting seam's contracts (architecture plan §5.5's
//! tooling-parity line, "class sorting (official order from the
//! registry)"; §3.2's CLI row names the surface `sort`). TDD: these
//! were written FIRST — the RED capture is the "no such function"
//! compile failure.
//!
//! The contracts:
//!
//! 1. **Registry order** — within the base bucket the sort key is the
//!    FIRST matching registry row's index ([`aeolus_data::registry::sort_index`]),
//!    so the sorted string reads in the canonical utility progression
//!    the rows audit (spacing → sizing → layout → colour → typography).
//! 2. **Variant buckets** — the folded `Cond`'s cascade specificity
//!    ([`aeolus_ir::Cond::specificity`]) buckets the variants AFTER the
//!    base: base(0) < dark(1) < probe states(2) < parent-relative(3) <
//!    media(4) < container(5) — the same rank the pipeline sorts layers
//!    by, so the sorted string reads in the order the CSS emit arm
//!    renders rules.
//! 3. **Honest tail** — a class the seam cannot ORDER (lex failure,
//!    variant fold miss, registry miss) is never reordered relative to
//!    its unsortable peers, rides AFTER the sorted classes in source
//!    order, and reports the SAME typed, coded diagnostic the compile
//!    walk reports. No silent drops, no guessed positions.
//! 4. **Additive customs** — registered custom utilities sort after all
//!    registry rows, in registration order (`sort_with`); the registry
//!    wins a name collision. Empty registrations are byte-identical to
//!    [`sort`] (the additive-seam precedent).
//! 5. **Totality** — the walk is deterministic, stable (equal keys keep
//!    source order), idempotent, and normalizes the join to single
//!    spaces; only the empty input hard-fails ([`CompileError::Empty`],
//!    the compile seam's own contract).

use aeolus_compile::{CompileError, CustomForm, CustomUtility, Customs, sort, sort_with};
use aeolus_ir::Prop;

/// The one-call helper: sort and render (the happy-path shape).
fn ordered(input: &str) -> String {
    sort(input)
        .expect("non-empty input sorts")
        .to_class_string()
}

// ---------------------------------------------------------------------------
// 1. Registry order
// ---------------------------------------------------------------------------

#[test]
fn registry_order_within_the_base_bucket() {
    // The rows' progression: padding(0) < margin(9) < display family.
    // `block` sits in the display family's own root rows, far after the
    // spacing families.
    assert_eq!(ordered("block p-4 mx-2"), "p-4 mx-2 block");
    // Sizing before layout, layout before colour.
    assert_eq!(ordered("bg-red-500 hidden w-8"), "w-8 hidden bg-red-500");
    // Typography after borders.
    assert_eq!(
        ordered("font-bold border rounded-md"),
        "border rounded-md font-bold"
    );
}

#[test]
fn every_registry_root_sorts_where_its_rows_say() {
    // A longer, realistic string: the shadcn Button's own vocabulary.
    // Pinned to the rows' indices: px(1) < py(2) < h < inline-flex <
    // bg < rounded < shadow < text < font, the hover bucket last.
    assert_eq!(
        ordered(
            "inline-flex h-9 px-4 py-2 rounded-md text-sm font-medium \
             bg-primary shadow-xs hover:bg-primary/90"
        ),
        "px-4 py-2 h-9 inline-flex bg-primary rounded-md shadow-xs text-sm font-medium hover:bg-primary/90"
    );
    // The point is the ORDER, not the echo: base classes sorted by row
    // index, the hover layer riding after every base class.
    let sorted = ordered("shadow-xs px-4 h-9 inline-flex hover:bg-primary/90");
    assert_eq!(
        sorted, "px-4 h-9 inline-flex shadow-xs hover:bg-primary/90",
        "spacing/sizing families precede the shadow family; the hover bucket rides last"
    );
}

// ---------------------------------------------------------------------------
// 2. Variant buckets
// ---------------------------------------------------------------------------

#[test]
fn variant_buckets_sort_after_the_base() {
    assert_eq!(ordered("hover:block block"), "block hover:block");
    assert_eq!(
        ordered("md:p-4 hover:p-3 dark:p-2 p-1"),
        // base(0) < dark(1) < hover(2) < media bp(4) — cond.rs's own
        // specificity contract, the cascade rank the layers sort by.
        "p-1 dark:p-2 hover:p-3 md:p-4"
    );
}

#[test]
fn stacked_variants_fold_to_their_dominant_bucket() {
    // The flat-IR model: `dark:hover:` folds to the hover probe (the
    // highest-specificity segment), so the pair sorts with the probe
    // tier, after the plain dark layer.
    assert_eq!(
        ordered("dark:hover:flex flex dark:flex"),
        "flex dark:flex dark:hover:flex"
    );
}

// ---------------------------------------------------------------------------
// 3. Honest tail
// ---------------------------------------------------------------------------

#[test]
fn unknown_classes_ride_the_tail_with_diagnostics() {
    let sorted = sort("p-4 not-a-utility mx-2").expect("sorts with the tail");
    assert_eq!(sorted.to_class_string(), "p-4 mx-2 not-a-utility");
    assert_eq!(sorted.diagnostics.len(), 1, "one typed miss");
    let diagnostic = &sorted.diagnostics[0];
    assert!(
        matches!(
            diagnostic.kind,
            aeolus_compile::DiagnosticKind::UnknownRoot { .. }
        ),
        "the registry's own typed miss: {:?}",
        diagnostic.kind
    );
    assert_eq!(
        diagnostic.kind.code(),
        "aeolus::compile::unknown_root",
        "the stable code a linter keys on"
    );
    // The tail classes carry their source spans and the unsortable mark.
    let tail = sorted.classes.last().expect("the unknown class rides last");
    assert!(!tail.known, "the unknown class is marked unsortable");
    assert_eq!(tail.raw, "not-a-utility");
    assert_eq!(tail.span, (4, 17), "the token's byte range in the input");
}

#[test]
fn shape_rejections_ride_the_tail_too() {
    let sorted = sort("p-red p-4").expect("sorts with the tail");
    assert_eq!(sorted.to_class_string(), "p-4 p-red");
    assert!(matches!(
        sorted.diagnostics[0].kind,
        aeolus_compile::DiagnosticKind::ShapeRejected { root: "p", .. }
    ));
}

#[test]
fn lex_failures_ride_the_tail() {
    let sorted = sort("p-4 w-[]").expect("the string still sorts");
    assert_eq!(sorted.to_class_string(), "p-4 w-[]");
    assert!(matches!(
        sorted.diagnostics[0].kind,
        aeolus_compile::DiagnosticKind::Lex { .. }
    ));
}

#[test]
fn unsupported_variants_ride_the_tail() {
    let sorted = sort("p-4 [@media(print)]:flex").expect("the string still sorts");
    assert_eq!(sorted.to_class_string(), "p-4 [@media(print)]:flex");
    assert!(matches!(
        sorted.diagnostics[0].kind,
        aeolus_compile::DiagnosticKind::UnsupportedVariant { .. }
    ));
}

// ---------------------------------------------------------------------------
// 4. Additive customs
// ---------------------------------------------------------------------------

#[test]
fn custom_utilities_sort_after_the_registry_rows() {
    let customs = Customs::new().utility(CustomUtility::functional(
        "brand-p",
        CustomForm::Spacing,
        &[
            Prop::PaddingTop,
            Prop::PaddingRight,
            Prop::PaddingBottom,
            Prop::PaddingLeft,
        ],
    ));
    let sorted = sort_with("brand-p-2 p-4", &customs).expect("sorts");
    assert_eq!(
        sorted.to_class_string(),
        "p-4 brand-p-2",
        "every registry row precedes every custom registration"
    );
    // Two registrations keep THEIR order (registration order after the
    // registry).
    let customs = customs.utility(CustomUtility::functional(
        "brand-m",
        CustomForm::Spacing,
        &[
            Prop::MarginTop,
            Prop::MarginRight,
            Prop::MarginBottom,
            Prop::MarginLeft,
        ],
    ));
    let sorted = sort_with("brand-m-2 brand-p-2", &customs).expect("sorts");
    assert_eq!(sorted.to_class_string(), "brand-p-2 brand-m-2");
}

#[test]
fn the_registry_wins_a_name_collision() {
    let customs = Customs::new().utility(CustomUtility::functional(
        "p",
        CustomForm::Spacing,
        &[
            Prop::PaddingTop,
            Prop::PaddingRight,
            Prop::PaddingBottom,
            Prop::PaddingLeft,
        ],
    ));
    let with_customs = sort_with("p-4", &customs).expect("sorts");
    let plain = sort("p-4").expect("sorts");
    assert_eq!(
        with_customs.classes[0].key, plain.classes[0].key,
        "the registered custom under a registry root never shifts the key"
    );
}

// ---------------------------------------------------------------------------
// 5. Totality
// ---------------------------------------------------------------------------

#[test]
fn equal_keys_keep_source_order() {
    // Same root, same key: the merge is cn()'s business — the sort is
    // stable, never reordering equals.
    assert_eq!(ordered("mx-8 mx-2"), "mx-8 mx-2");
    assert_eq!(ordered("p-4 p-8"), "p-4 p-8");
}

#[test]
fn modifiers_and_negations_do_not_disturb_the_order() {
    // The modifier is a value concern (opacity/denominator), not an
    // order concern; the negated form sorts under its root.
    assert_eq!(ordered("bg-primary/90 p-4"), "p-4 bg-primary/90");
    // Same root, same key → stable source order, sign and all.
    assert_eq!(ordered("mt-2 -mt-4"), "mt-2 -mt-4");
    assert_eq!(ordered("-mt-4 mt-2"), "-mt-4 mt-2");
}

#[test]
fn the_walk_is_idempotent() {
    let once = ordered("hover:p-4 block p-2 mx-8 not-a-real-class-anywhere");
    let twice = ordered(&once);
    assert_eq!(once, twice, "sorting a sorted string is the identity");
}

#[test]
fn the_join_normalizes_to_single_spaces() {
    assert_eq!(ordered("  p-4     block  "), "p-4 block");
    assert_eq!(ordered("p-4\tblock\nmx-2"), "p-4 mx-2 block");
}

#[test]
fn empty_input_is_the_typed_error() {
    assert_eq!(sort("   "), Err(CompileError::Empty));
    assert_eq!(sort(""), Err(CompileError::Empty));
}
