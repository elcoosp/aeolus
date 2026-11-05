//! The backend parity test (plan §4.1/§5.5): every slot the corpus sets is
//! exactly one of native / emulated / reported — "dropped without notice"
//! is not a state. This walks a corpus of real Shadcn v4 classes plus the
//! v0.2 utility expansion, lowers each plan, and asserts the parity ledger
//! covers every set slot.

use aeolus_core::Warning;
use aeolus_core::plan::Slot;
use aeolus_floem::{Coverage, slot_coverage};

const THEME: &str = include_str!("../../aeolus-core/data/shadcn-light.css");

/// A corpus spanning every family the Floem backend lowers.
const CORPUS: &[&str] = &[
    // shadcn button (all variants)
    "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50",
    "bg-primary text-primary-foreground hover:bg-primary/90",
    "border bg-background shadow-xs hover:bg-accent dark:border-input dark:bg-input/30 dark:hover:bg-input/50",
    "hover:bg-accent hover:text-accent-foreground dark:hover:bg-accent/50",
    "h-9 px-4 py-2",
    "size-9",
    // v0.2 utility expansion
    "mx-auto absolute z-10 top-2 left-4 overflow-hidden invisible aspect-video",
    "grid grid-cols-3 gap-x-2 gap-y-3 order-2 basis-4",
    "scale-105 rotate-45 translate-x-4",
    "italic uppercase underline truncate line-clamp-2 tabular-nums font-sans",
    "transition-colors duration-300 ease-in-out delay-100 animate-pulse",
    "border-dashed border-e-2 ring-inset outline-offset-2 shadow-sm inset-shadow-sm",
    "cursor-pointer cursor-wait select-none w-screen max-w-xl text-left text-ellipsis",
    // compound + structural variants (rules → emulated)
    "dark:hover:bg-accent/50",
    "md:hover:p-2",
    "group-hover:bg-accent peer-disabled:opacity-50",
    "data-[state=open]:opacity-100 first:bg-transparent",
    "@md:gap-4",
];

#[test]
fn every_corpus_slot_is_native_emulated_or_reported() {
    let theme = aeolus_core::TailwindTheme::from_theme_css(THEME).unwrap();
    let resolver = aeolus_core::Resolver::new(&theme);
    for classes in CORPUS {
        let r = resolver.resolve(classes).unwrap();
        // Every rule atom is EMULATED by the probe path (tw_plan); legacy
        // buckets are native selector maps; everything else must appear in
        // the coverage ledger below.
        let mut slots: Vec<Slot> = Vec::new();
        slots.extend(r.plan.base.slots());
        for l in &r.plan.layers {
            slots.extend(l.style.slots());
        }
        if let Some(d) = &r.plan.dark {
            slots.extend(d.slots());
        }
        for s in r.plan.responsive.values() {
            slots.extend(s.slots());
        }
        for rule in &r.plan.rules {
            slots.extend(rule.style.slots());
        }
        slots.sort_by_key(|s| s.index());
        slots.dedup();

        // The coverage ledger from the FINAL resolved style (the union the
        // backend actually applies at runtime is base + hit conditions; the
        // ledger classifies every slot shape the corpus can produce).
        let mut coverage: Vec<(Slot, Coverage)> = Vec::new();
        coverage.extend(slot_coverage(&r.plan.base));
        for l in &r.plan.layers {
            coverage.extend(slot_coverage(&l.style));
        }
        if let Some(d) = &r.plan.dark {
            coverage.extend(slot_coverage(d));
        }
        for s in r.plan.responsive.values() {
            coverage.extend(slot_coverage(s));
        }
        for rule in &r.plan.rules {
            coverage.extend(slot_coverage(&rule.style));
        }
        coverage.sort_by_key(|(s, _)| s.index());
        coverage.dedup();

        for slot in &slots {
            let covered = coverage.iter().any(|(s, _)| s == slot);
            assert!(
                covered,
                "{classes:?}: slot {slot:?} is set but has no coverage entry \
                 (neither native, emulated, nor reported) — the parity \
                 contract (plan §4.1) forbids silent drops"
            );
        }
    }
}

/// The rules bucket is the EMULATED lane — lowering must say so, not drop.
#[test]
fn rules_are_emulated_not_dropped() {
    let theme = aeolus_core::TailwindTheme::from_theme_css(THEME).unwrap();
    let resolver = aeolus_core::Resolver::new(&theme);
    let r = resolver.resolve("dark:hover:bg-accent/50").unwrap();
    assert!(!r.plan.rules.is_empty(), "the compound variant is a rule");
    let (_, report) = aeolus_floem::lower(&r.plan);
    assert!(
        report
            .dropped
            .iter()
            .any(|(what, _)| what.starts_with("rules:")),
        "the lowering must RECORD that rules need the reactive probe path: {report:?}"
    );
}

/// Theme-aware breakpoints reach the lowering (plan §4.2 fixes
/// `Breakpoint::default_px` ignoring theme overrides).
#[test]
fn breakpoints_are_theme_aware() {
    let css = format!("{THEME} @theme {{ --breakpoint-md: 900px; }}");
    let theme = aeolus_core::TailwindTheme::from_theme_css(&css).unwrap();
    let resolver = aeolus_core::Resolver::new(&theme);
    let r = resolver.resolve("md:p-2").unwrap();
    assert_eq!(
        r.plan.breakpoints[aeolus_core::Breakpoint::Md.index()],
        900.0
    );
    assert!(
        r.warnings
            .iter()
            .all(|w| !matches!(w, Warning::Unknown { .. } | Warning::Unparseable { .. }))
    );
}

/// The v0.2 transition slot lowers into real Floem transitions.
#[test]
fn transitions_lower_to_floem_transitions() {
    let theme = aeolus_core::TailwindTheme::from_theme_css(THEME).unwrap();
    let resolver = aeolus_core::Resolver::new(&theme);
    let r = resolver.resolve("transition-colors duration-300").unwrap();
    let (style, report) = aeolus_floem::lower(&r.plan);
    assert!(
        !report
            .dropped
            .iter()
            .any(|(what, _)| what.contains("transition")),
        "transition-colors is native now: {report:?}"
    );
    // Style is opaque (map-based); the absence of a drop entry IS the
    // contract here — the prop was applied.
    let _ = style;
}
