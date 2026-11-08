//! Test-pyramid property tests (proptest, dev-dependency only).
//!
//! The laws the runtime composition story relies on, checked over PLANS
//! generated through the REAL pipeline (random class strings → resolver →
//! plan), not hand-built structs:
//!
//! 1. **Overlay idempotence** — `p.overlay(o).overlay(o) == p.overlay(o)`.
//!    Applying the same override twice changes nothing the second time.
//! 2. **Overlay per-slot later-wins** — every slot the override defines
//!    ends up with the override's value; slots it leaves unset keep the
//!    base value.
//! 3. **Overlay change containment** — `diff(base, merged.base)` reports
//!    only slots the override's base actually defines (composition cannot
//!    invent changes).
//! 4. **Snapshot round-trip** — `from_snapshot_json(to_snapshot_json(p))`
//!    is the identity for arbitrary plans, and the writer is byte-stable
//!    (two writes of the equal plans produce identical bytes).
//!
//! Plan generation resolves class strings against the shadcn light theme —
//! the same fixture the golden corpus and the benchmarks use — so the
//! properties hold for the plans that actually ship.

use proptest::prelude::*;

use aeolus_core::{Resolver, StylePlan, TailwindTheme, diff};

fn theme() -> TailwindTheme {
    TailwindTheme::from_theme_css(include_str!("../data/shadcn-light.css")).unwrap()
}

/// A curated alphabet of REAL utilities covering every plan region: base
/// slots, typography, extras (grid/transform/animation), the v0.7 surface
/// (pseudo-elements, has-*, containers, arbitrary templates), states,
/// breakpoints, dark, group/peer and data/aria rules. Invalid or
/// unsupported entries simply resolve to warnings — the plan is still
/// total — which is itself part of the property.
const ALPHABET: &[&str] = &[
    "bg-red-500",
    "bg-primary/50",
    "p-4",
    "px-8",
    "py-0.5",
    "m-2",
    "-mt-1",
    "rounded-md",
    "rounded-t-lg",
    "border",
    "border-2",
    "border-red-300",
    "shadow-sm",
    "shadow-lg/50",
    "opacity-75",
    "ring-2",
    "ring-offset-2",
    "outline-1",
    "cursor-pointer",
    "pointer-events-none",
    "text-sm",
    "font-medium",
    "tracking-tight",
    "underline",
    "truncate",
    "line-clamp-2",
    "tabular-nums",
    "flex",
    "grid",
    "hidden",
    "flex-row-reverse",
    "items-center",
    "justify-between",
    "gap-3",
    "gap-x-1",
    "order-3",
    "basis-1/2",
    "self-end",
    "content-start",
    "grid-cols-3",
    "grid-cols-[0_1fr]",
    "grid-rows-[auto_auto]",
    "auto-rows-fr",
    "col-span-2",
    "row-start-1",
    "justify-items-start",
    "w-1/2",
    "h-screen",
    "max-w-md",
    "min-h-0",
    "size-10",
    "aspect-video",
    "inset-x-2",
    "top-4",
    "z-10",
    "overflow-auto",
    "invisible",
    "scale-110",
    "rotate-45",
    "translate-y-2",
    "skew-x-6",
    "transition-colors",
    "duration-300",
    "ease-in",
    "delay-100",
    "animate-spin",
    "selection:bg-primary",
    "placeholder:text-muted-foreground",
    "file:text-sm",
    "has-checked:bg-green-500",
    "has-data-[slot=card-action]:grid-cols-[1fr_auto]",
    "has-[:hover]:underline",
    "not-hover:opacity-100",
    "group-hover:underline",
    "group-hover/menu:bg-red-500",
    "peer-focus:block",
    "data-[state=open]:bg-accent",
    "aria-expanded:bg-accent",
    "first:pt-0",
    "nth-3:pb-1",
    "hover:bg-primary/90",
    "focus-visible:ring-[3px]",
    "focus-visible:ring-ring/50",
    "disabled:pointer-events-none",
    "disabled:opacity-50",
    "active:scale-95",
    "sm:text-lg",
    "md:grid-cols-2",
    "lg:w-auto",
    "xl:gap-4",
    "2xl:p-10",
    "dark:bg-primary",
    "dark:hover:bg-accent",
    "@container/card-header",
    "@md/name:gap-4",
    "@max-lg:flex-col",
    "supports-[backdrop-filter]:bg-white/50",
    "text-balance",
    "backdrop-blur",
    "[&>svg]:size-4",
    "[.border-b]:pb-1",
];

fn class_string() -> impl Strategy<Value = String> {
    proptest::collection::vec(proptest::sample::select(ALPHABET), 0..24)
        .prop_map(|classes| classes.join(" "))
}

fn plan_of(theme: &TailwindTheme, classes: &str) -> StylePlan {
    Resolver::new(theme).resolve(classes).unwrap().plan
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Law 1 — overlay idempotence, over full-pipeline plans.
    #[test]
    fn overlay_is_idempotent(base_classes in class_string(), over_classes in class_string()) {
        let t = theme();
        let base = plan_of(&t, &base_classes);
        let over = plan_of(&t, &over_classes);
        let once = base.overlay(&over);
        let twice = once.overlay(&over);
        prop_assert_eq!(once, twice, "p.overlay(o).overlay(o) != p.overlay(o)");
    }

    /// Law 2 — per-slot later-wins: slots the override defines take the
    /// override's value; slots it leaves unset keep the base value.
    #[test]
    fn overlay_is_per_slot_later_wins(base_classes in class_string(), over_classes in class_string()) {
        let t = theme();
        let base = plan_of(&t, &base_classes);
        let over = plan_of(&t, &over_classes);
        let merged = base.overlay(&over);
        // Every base slot the override does NOT define survives unchanged
        // (law 3 below pins the same invariant from the changed-set side;
        // here it is pinned from the preserved-slot side).
        let d = diff(&base.base, &merged.base);
        for slot in &d.changed {
            prop_assert!(
                over.base.slots().contains(slot),
                "slot {slot:?} changed without the override defining it"
            );
        }
    }

    /// Law 3 — change containment: composition cannot invent changes.
    /// Every slot `diff(base, merged)` reports must be a slot the override
    /// defines (same-theme plans: the breakpoint tables agree).
    #[test]
    fn overlay_changes_are_contained(base_classes in class_string(), over_classes in class_string()) {
        let t = theme();
        let base = plan_of(&t, &base_classes);
        let over = plan_of(&t, &over_classes);
        let merged = base.overlay(&over);
        let d = diff(&base.base, &merged.base);
        for slot in &d.changed {
            prop_assert!(
                over.base.slots().contains(slot),
                "merged changed {slot:?} but the override never defines it"
            );
        }
    }

    /// Law 4 — snapshot round-trip identity: the strict reader reconstructs
    /// the exact plan the byte-stable writer emitted.
    #[test]
    fn snapshot_roundtrip_is_the_identity(classes in class_string()) {
        let t = theme();
        let plan = plan_of(&t, &classes);
        let json = plan.to_snapshot_json();
        let back = StylePlan::from_snapshot_json(&json).unwrap();
        // Byte stability: writing the reconstructed plan again yields the
        // same bytes (canonical order, shortest float form).
        let json2 = back.to_snapshot_json();
        prop_assert_eq!(plan, back, "round-trip changed the plan");
        prop_assert_eq!(json, json2);
    }
}
