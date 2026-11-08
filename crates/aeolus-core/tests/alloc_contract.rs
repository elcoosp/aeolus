//! The allocation contract (v0.5 §3, `AEOLUS_V0.5.md`) — report 08's
//! allocation-counter follow-up, closed in-repo.
//!
//! divan 0.1.x cannot carry per-benchmark allocation counters (its
//! `IntoCounter` is a fixed per-iteration value, not a sampled delta —
//! re-verified against the vendored 0.1.21 source), so the harness the
//! report asked for is a counting global allocator + this contract test:
//! deterministic, machine-independent numbers that pin the report's
//! structural claims.
//!
//! Policy (v0.5 §3.3):
//! * exact-zero claims are pinned EXACTLY (`diff` of equal plans);
//! * clone-shaped claims are pinned as UPPER BOUNDS (allocation counts
//!   legitimately churn across rustc versions); the measured numbers are
//!   recorded in `reports/08-benchmarks/BENCHMARKS.md`.
//!
//! Sections are serialized by a mutex: tests in one binary share the
//! process allocator, so the counters are only meaningful while the gate
//! is open inside one `measured` section.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

use aeolus_core::*;

static ALLOCS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);
static TRACKING: AtomicBool = AtomicBool::new(false);
static LOCK: Mutex<()> = Mutex::new(());

struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // One allocation EVENT (realloc's default impl counts as
        // alloc + dealloc — the dealloc side is not tracked).
        if TRACKING.load(Ordering::Relaxed) {
            ALLOCS.fetch_add(1, Ordering::Relaxed);
            BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

/// Run `f` three times with the counters open; returns the MINIMUM
/// `(allocs, bytes)` across runs plus the last result. The minimum is the
/// operation's true cost: the test harness's own bookkeeping (output
/// capture, thread timing) can allocate INSIDE a measurement window —
/// noise only ever ADDS counts, so the minimum is deterministic even
/// under the multi-threaded harness.
fn measured<T>(mut f: impl FnMut() -> T) -> (u64, u64, T) {
    let mut out = None;
    let (mut best_a, mut best_b) = (u64::MAX, u64::MAX);
    for _ in 0..3 {
        ALLOCS.store(0, Ordering::Relaxed);
        BYTES.store(0, Ordering::Relaxed);
        TRACKING.store(true, Ordering::Relaxed);
        let o = f();
        TRACKING.store(false, Ordering::Relaxed);
        best_a = best_a.min(ALLOCS.load(Ordering::Relaxed));
        best_b = best_b.min(BYTES.load(Ordering::Relaxed));
        out = Some(o);
    }
    (best_a, best_b, out.unwrap())
}

fn lock() -> MutexGuard<'static, ()> {
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn theme() -> TailwindTheme {
    TailwindTheme::from_theme_css(shadcn_css::LIGHT).unwrap()
}

const BUTTON: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90";

/// The report-08 claim, pinned EXACTLY: diffing two EQUAL fully-populated
/// plans allocates nothing (the changed-slot `Vec` is empty — no alloc).
#[test]
fn diff_of_equal_plans_allocates_zero() {
    let _g = lock();
    let t = theme();
    let a = Resolver::new(&t).resolve(BUTTON).unwrap().plan.base;
    let b = a.clone();
    let (allocs, _bytes, diff) = measured(|| plan::diff(&a, &b));
    assert_eq!(
        allocs, 0,
        "diff of equal plans must allocate nothing (report 08 structural claim)"
    );
    assert!(diff.is_empty(), "equal plans must diff to no changes");
}

/// The report-08 claim, measured PRECISELY: the warm cache hit is one
/// hash lookup + an `Arc` refcount bump of the cached `CachedPlan` — the
/// plan itself is NEVER deep-cloned, so the hit path allocates NOTHING.
/// Pinned exactly (the report's "plan clone" wording was an Arc bump all
/// along; the harness made that visible).
#[test]
fn cached_hit_allocates_nothing() {
    let _g = lock();
    let t = theme();
    let cache = PlanCache::new();
    let _ = cache.resolve(&t, BUTTON).unwrap();
    let (allocs, bytes, _) = measured(|| cache.resolve(&t, BUTTON).unwrap());
    assert_eq!(
        allocs, 0,
        "a warm hit returns an Arc clone of the cached plan — {bytes} B leaked into the path"
    );
}

/// The report-08 claim, bounded: the interner hit path is a hash lookup
/// returning the stored id — no allocation.
#[test]
fn intern_hit_allocates_nothing() {
    let _g = lock();
    let interner = Interner::new();
    let _ = interner.intern(BUTTON);
    let (allocs, _bytes, _) = measured(|| interner.intern(BUTTON));
    assert_eq!(
        allocs, 0,
        "an intern hit is a hash lookup — the string is already stored"
    );
}

/// Counted for report 08: `cn()` over the 33-class Button + one override
/// (conflict resolution = parse to tokens + trie scan + one output
/// String), and the cold resolve (parse + theme + conflict + plan build).
#[test]
fn cn_and_cold_resolve_are_counted() {
    let _g = lock();
    let t = theme();
    let (cn_allocs, cn_bytes, merged) =
        measured(|| cn(&[BUTTON, "rounded-lg text-base bg-primary shadow-md"]));
    assert!(!merged.is_empty());
    // Measured 61 allocs / 46,218 B on rustc 1.99.1 (report 08).
    assert!(
        cn_allocs <= 128 && cn_bytes <= 65536,
        "cn() allocations regressed: {cn_allocs} allocs / {cn_bytes} B"
    );

    let (cold_allocs, cold_bytes, _) = measured(|| Resolver::new(&t).resolve(BUTTON).unwrap());
    assert!(
        cold_allocs > 0,
        "a cold resolve builds the plan — it must allocate"
    );
    // Measured 82 allocs / 44,112 B on rustc 1.99.1 (report 08).
    assert!(
        cold_allocs <= 160 && cold_bytes <= 65536,
        "cold-resolve allocations regressed: {cold_allocs} allocs / {cold_bytes} B"
    );
}
