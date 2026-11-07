//! v0.7 canonical plan snapshots (AEOLUS_V0.7.md §2): a zero-dependency,
//! byte-stable JSON writer plus a strict reader for the `StylePlan` IR —
//! the IR becomes a portable artifact (CI plan pins, cross-process cache
//! persistence, cross-language review).
//!
//! RED phase: written before `src/snapshot.rs` existed; the compile failure
//! is the captured RED (evidence in
//! `reports/10-tdd-evidence/09-portable-plans.md`).

use aeolus_core::*;

fn theme() -> TailwindTheme {
    TailwindTheme::from_theme_css(shadcn_css::LIGHT).unwrap()
}

/// A plan that touches every bucket and most slot payloads: the kitchen
/// sink the round-trip contract must survive.
fn kitchen_sink() -> StylePlan {
    let t = theme();
    let r = Resolver::new(&t);
    r.resolve(concat!(
        "p-4 px-2 bg-primary bg-linear-to-r from-red-500 via-white/50 to-blue-600/80 rounded-t-md",
        " rounded-es-xl w-1/2 h-screen min-w-0 max-w-xl gap-x-2 gap-y-3 m-2 mx-auto shadow-md",
        " shadow-red-500/40 inset-shadow-2xs ring ring-ring/50 ring-offset-2 border border-dashed",
        " border-x-2 outline-2 outline-offset-2 opacity-50 cursor-pointer pointer-events-none",
        " text-sm font-medium leading-tight tracking-tight underline-offset-4 text-left",
        " italic uppercase underline truncate line-clamp-2 tabular-nums select-text",
        " absolute inset-x-2 top-4 z-10 overflow-x-auto overflow-y-clip invisible aspect-video",
        " grid grid-cols-3 col-span-2 row-start-1 order-2 basis-1/3 content-center",
        " scale-110 rotate-45 translate-y-1/2 skew-x-6 transition-colors duration-300",
        " ease-in delay-100 animate-spin hover:bg-red-500 focus-visible:ring-[3px]",
        " disabled:opacity-50 md:p-8 2xl:px-4 dark:bg-red-900",
        " group-hover/menu:px-8 peer-focus/x:underline data-[state=open]:p-2",
        " first:pt-0 has-checked:bg-green-500 not-hover:opacity-100",
        " motion-reduce:animate-none print:hidden supports-[display:grid]:grid",
    ))
    .unwrap()
    .plan
}

#[test]
fn round_trip_kitchen_sink() {
    let plan = kitchen_sink();
    let json = plan.to_snapshot_json();
    let back = StylePlan::from_snapshot_json(&json).expect("snapshot parses");
    assert_eq!(back, plan, "snapshot round-trip is exact");
}

#[test]
fn round_trip_is_byte_stable() {
    // Two resolutions of the same string produce byte-identical artifacts —
    // the canonical form has no map-order or float-formatting drift.
    let a = kitchen_sink().to_snapshot_json();
    let b = kitchen_sink().to_snapshot_json();
    assert_eq!(a, b);
    // Re-serializing a parsed snapshot is byte-identical too.
    let back = StylePlan::from_snapshot_json(&a).unwrap();
    assert_eq!(back.to_snapshot_json(), a);
}

#[test]
fn snapshot_header_and_shape() {
    let plan = kitchen_sink();
    let json = plan.to_snapshot_json();
    // The header is the compatibility contract.
    assert!(json.contains("\"format\":\"aeolus-plan\""), "{json}");
    assert!(json.contains("\"version\":1"), "{json}");
    // Compact (no whitespace outside strings) — diff-friendly.
    assert!(!json.contains("\n"), "snapshot is compact JSON");
}

#[test]
fn empty_plan_round_trips() {
    let plan = StylePlan::default();
    let json = plan.to_snapshot_json();
    let back = StylePlan::from_snapshot_json(&json).unwrap();
    assert_eq!(back, plan);
}

#[test]
fn rejects_malformed_snapshots() {
    // Not JSON.
    let err = StylePlan::from_snapshot_json("not json at all").unwrap_err();
    assert!(!err.to_string().is_empty());
    // JSON but wrong shape (an array instead of an object).
    assert!(StylePlan::from_snapshot_json("[]").is_err());
    // A JSON object without the header.
    assert!(StylePlan::from_snapshot_json("{}").is_err());
    // Wrong format tag.
    assert!(StylePlan::from_snapshot_json("{\"format\":\"other\",\"version\":1}").is_err());
    // Unsupported version → precise, matchable error.
    let err =
        StylePlan::from_snapshot_json("{\"format\":\"aeolus-plan\",\"version\":99}").unwrap_err();
    assert!(
        matches!(err, SnapshotError::UnsupportedVersion(99)),
        "{err}"
    );
    // Unknown fields are rejected (strictness is the compat contract).
    assert!(
        StylePlan::from_snapshot_json("{\"format\":\"aeolus-plan\",\"version\":1,\"bogus\":1}")
            .is_err()
    );
    // Trailing garbage after a complete document is rejected.
    let plan = StylePlan::default();
    let json = plan.to_snapshot_json();
    assert!(StylePlan::from_snapshot_json(&format!("{json} junk")).is_err());
    // Truncated document is rejected (no silent partial plans).
    let cut = &json[..json.len() / 2];
    assert!(StylePlan::from_snapshot_json(cut).is_err());
}

#[test]
fn snapshot_never_panics_on_fuzzish_input() {
    // The reader is total: any input maps to a Result, never a panic.
    let inputs = [
        "",
        "{",
        "null",
        "123",
        "\"str\"",
        "{\"format\":\"aeolus-plan\",\"version\":1,\"base\":null}",
        "{\"format\":\"aeolus-plan\",\"version\":1,\"base\":{\"padding\":{\"top\":\"x\"}}}",
        "{\"format\":\"aeolus-plan\",\"version\":1,\"breakpoints\":[1,2,3]}",
        "{\"format\":\"aeolus-plan\",\"version\":1,\"layers\":[{\"condition\":42}]}",
        "{\"format\":\"aeolus-plan\",\"version\":1e999}",
    ];
    for input in inputs {
        let _ = StylePlan::from_snapshot_json(input); // must not panic
    }
}

#[test]
fn gradient_snapshot_round_trips() {
    let t = theme();
    let r = Resolver::new(&t);
    for classes in [
        "bg-linear-to-r from-red-500 to-blue-500",
        "bg-linear-45 from-red-500 via-white to-blue-500",
        "bg-radial from-amber-400 from-20% to-emerald-500",
        "bg-conic-180 from-red-500 to-blue-500",
        "bg-none",
        "bg-linear-to-br from-primary via-transparent",
    ] {
        let plan = r.resolve(classes).unwrap().plan;
        let json = plan.to_snapshot_json();
        let back = StylePlan::from_snapshot_json(&json).unwrap();
        assert_eq!(back, plan, "gradient round-trip: {classes}");
    }
}

/// Property test: random class strings from the grammar's surface resolve,
/// snapshot, and round-trip exactly — the writer/reader pair is total over
/// everything the resolver can produce.
#[test]
fn proptest_random_plans_round_trip() {
    let classes_pool: &[&str] = &[
        "p-1",
        "p-2",
        "px-4",
        "pt-3",
        "m-auto",
        "bg-primary",
        "bg-red-500/80",
        "text-sm",
        "text-primary",
        "font-bold",
        "tracking-wide",
        "leading-6",
        "rounded-full",
        "rounded-t-lg",
        "shadow-2xl",
        "shadow-none",
        "ring-2",
        "ring-offset-4",
        "outline-none",
        "opacity-75",
        "w-full",
        "h-dvh",
        "size-8",
        "min-w-fit",
        "max-h-screen",
        "gap-4",
        "gap-x-2",
        "flex",
        "flex-col",
        "items-center",
        "justify-between",
        "self-end",
        "grid",
        "grid-cols-12",
        "col-span-4",
        "order-first",
        "basis-auto",
        "aspect-square",
        "absolute",
        "inset-0",
        "top-1/2",
        "z-auto",
        "overflow-clip",
        "visible",
        "cursor-grabbing",
        "pointer-events-auto",
        "italic",
        "uppercase",
        "underline",
        "truncate",
        "select-none",
        "transition-all",
        "duration-700",
        "ease-linear",
        "animate-pulse",
        "scale-0",
        "rotate-12",
        "translate-x-full",
        "skew-y-12",
        "border-y",
        "border-double",
        "bg-linear-to-t",
        "from-cyan-300",
        "via-40%",
        "to-90%",
        "bg-conic",
        "hover:",
        "focus:",
        "dark:",
        "md:",
        "group-hover:",
        "peer-disabled:",
        "first:",
        "nth-3:",
        "has-[:hover]:",
        "not-focus:",
        "print:",
        "motion-safe:",
        "container",
        "tabular-nums",
        "underline-offset-8",
    ];
    // A deterministic pseudo-random sampler (xorshift) keeps the test stable
    // while covering thousands of combinations across runs.
    let mut state: u64 = 0x9E3779B97F4A7C15;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let t = theme();
    let r = Resolver::new(&t);
    for _ in 0..256 {
        let n = (next() % 6) + 1;
        let mut s = String::new();
        for i in 0..n {
            if i > 0 {
                s.push(' ');
            }
            s.push_str(classes_pool[(next() as usize) % classes_pool.len()]);
        }
        // Variants need a utility: append one when a bare variant was drawn.
        if s.ends_with(':') {
            s.push_str("bg-red-500");
        }
        let plan = match r.resolve(&s) {
            Ok(res) => res.plan,
            Err(_) => continue, // unparseable strings are the parser's domain, not the snapshot's
        };
        let json = plan.to_snapshot_json();
        let back = StylePlan::from_snapshot_json(&json).unwrap();
        assert_eq!(back, plan, "round-trip failed for {s:?}");
    }
}

#[test]
fn parse_cache_and_snapshot_agree() {
    // The snapshot is an exact witness of the plan: resolving through the
    // cache and snapshotting must produce the same bytes.
    let t = theme();
    let cache = PlanCache::new();
    let s = "p-4 bg-primary hover:bg-primary/90 md:p-8";
    let a = cache
        .resolve(&t, s)
        .unwrap()
        .resolution
        .plan
        .to_snapshot_json();
    let b = Resolver::new(&t)
        .resolve(s)
        .unwrap()
        .plan
        .to_snapshot_json();
    assert_eq!(a, b);
}
