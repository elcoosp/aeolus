//! The allocation contract on the `.tw()` path (plan §4's enforcement
//! cell: "extend `alloc_contract.rs` to the `.tw()` path" — the perf
//! table's `Heap allocs/element (static): 0 (+GPUI's own boxing)` row).
//!
//! The v0.x battery (`aeolus-core/tests/alloc_contract.rs`, v0.5 §3)
//! pins the resolver lane; this file pins the V1 static application
//! path the backends ride every frame: the freeze
//! ([`StaticProgramBuilder::assemble`]), the element walk
//! ([`StaticProgram::style`] / [`program_style`]), the render-target
//! cascade ([`StaticProgram::style_pseudo`]) and the row-text reads.
//!
//! Policy (the v0.x battery's own, carried over):
//! * exact-zero claims are pinned EXACTLY (`assemble` is a `const fn`;
//!   the walk's machinery on answers whose slots are `Copy`; the quiet
//!   layers of a resting env);
//! * the answer's OWNED slots (`ResolvedStyle::text`, the shadow bands'
//!   `Vec`s) legitimately allocate when they fill — clone/shape-shaped
//!   claims are pinned as UPPER BOUNDS; the measured numbers are
//!   recorded in `reports/08-benchmarks/BENCHMARKS.md`.
//!
//! Sections are serialized by a mutex: tests in one binary share the
//! process allocator, so the counters are only meaningful while the
//! gate is open inside one `measured` section.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

use aeolus_compile::{Config, Env};
use aeolus_rt::{ClassCell, PseudoTarget, StaticProgram, StaticProgramBuilder};
use aeolus_theme::{ThemeSource, TokenTable};

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
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The baked shadcn LIGHT table (T0's shape) — the freeze walk's own
/// construction.
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the baked shadcn table builds")
}

/// Build a static program through the dynamic lane's own compile —
/// the ClassCell is the runtime path's constructor (the same compiler
/// the macro runs), so every pin's input is a REAL frozen program.
/// The compile itself allocates by design (interning) and runs OUTSIDE
/// every measurement window.
fn frozen(classes: &str) -> StaticProgram {
    let cell = ClassCell::new();
    let program = cell
        .get_theme(classes, &baked_table(), &Config::default())
        .expect("the cell compiles what lowers");
    assert!(
        !program.layers.is_empty(),
        "the corpus carries layers: {classes}"
    );
    program
}

// ---------------------------------------------------------------------------
// Exact-zero pins (the plan §4 row's `0` target)
// ---------------------------------------------------------------------------

#[test]
fn the_freeze_allocates_nothing() {
    let _g = lock();
    let program = frozen("p-4 bg-red-500 rounded-md hover:bg-red-500/80 text-sm font-medium");
    let layers: &[aeolus_ir::Layer] = program.layers;
    let keywords: &[&str] = program.keywords;
    let (allocs, _bytes, _assembled) =
        measured(|| StaticProgramBuilder::assemble(layers, keywords));
    assert_eq!(
        allocs, 0,
        "the assembly is the struct's own two-pointer construction (a const fn) — zero heap events"
    );
}

#[test]
fn the_layout_family_walk_allocates_nothing() {
    let _g = lock();
    let program = frozen("p-4 px-8 gap-4 rounded-lg border-2 items-center justify-center");
    let env = Env::default();
    let (allocs, _bytes, _style) = measured(|| program.style(&env));
    assert_eq!(
        allocs, 0,
        "every slot the layout families write is Copy — the walk's machinery allocates nothing"
    );
}

#[test]
fn the_colour_family_walk_allocates_nothing() {
    let _g = lock();
    // The baked colour reaches the Oklab slot DIRECTLY (the audited
    // `from_srgb` conversion) — no hex-string round-trip through the
    // parser on the hot path.
    let program = frozen("bg-red-500 text-red-500 border-red-500 bg-red-500/90");
    let env = Env::default();
    let (allocs, _bytes, (style, _walk)) = measured(|| program.style(&env));
    assert_eq!(
        allocs, 0,
        "the colour slots are Copy — the conversion is arithmetic, no per-frame heap events"
    );
    assert!(style.background.is_some(), "the colour landed");
}

#[test]
fn a_resting_env_never_walks_a_quiet_layer_into_the_heap() {
    let _g = lock();
    // The frame-state layers (hover/focus-visible/dark) do not fire
    // under the resting env — the walk must neither consult nor
    // allocate for them. (The probe tiers — data-*/has-*/supports-* —
    // are NOT here: a sourceless walk reports their boundary note, the
    // honesty channel's own report, which is a different contract.)
    let program = frozen("hover:bg-red-500 focus-visible:ring-2 dark:text-red-500");
    let env = Env::default();
    let (allocs, _bytes, (style, walk)) = measured(|| program.style(&env));
    assert_eq!(allocs, 0, "quiet layers stay quiet");
    assert!(walk.is_empty(), "a clean walk on the resting env");
    assert!(style.background.is_none(), "nothing fired");
}

#[test]
fn the_pseudo_target_cascade_allocates_nothing() {
    let _g = lock();
    let program = frozen("before:p-8 before:bg-red-500 selection:bg-red-500");
    let env = Env::default();
    let (allocs, _bytes, (_style, _walk)) = measured(|| program.style_pseudo(PseudoTarget::Before));
    assert_eq!(
        allocs, 0,
        "the render target's own cascade is Copy-slot writes"
    );
    let _ = env;
}

#[test]
fn the_row_text_reads_allocate_nothing() {
    let _g = lock();
    let program = frozen("border-2 rounded-md text-sm w-full");
    let (allocs, _bytes, _text) =
        measured(|| program.keyword_for(0, aeolus_ir::Prop::BorderTopWidth));
    assert_eq!(allocs, 0, "the row-text lookups slice the frozen keywords");
}

#[test]
fn the_walk_is_allocation_idempotent() {
    let _g = lock();
    // Re-walking the same program never grows the cost — no hidden
    // accumulation across frames.
    let program = frozen("p-4 bg-red-500 text-sm");
    let env = Env::default();
    let (first, _b1, _s1) = measured(|| program.style(&env));
    let (second, _b2, _s2) = measured(|| program.style(&env));
    assert_eq!(first, second, "the per-frame cost is constant");
}

// ---------------------------------------------------------------------------
// Upper-bound pins (the answer's owned slots)
// ---------------------------------------------------------------------------

#[test]
fn the_typography_slot_grows_within_its_bound() {
    let _g = lock();
    // `ResolvedStyle::text` owns its `Vec<(TextProperty, TextValue)>` —
    // two typography entries fill it with the Vec's own growth pattern
    // (the precedent's clone-shaped upper bound, BENCHMARKS.md records
    // the measured numbers).
    // `text-sm` lands its line-height companion (the v0.x text-sm →
    // 20px semantics), so two classes are three entries.
    let program = frozen("text-sm font-medium");
    let env = Env::default();
    let (allocs, _bytes, (style, _walk)) = measured(|| program.style(&env));
    assert_eq!(style.text.len(), 3, "both typography entries landed");
    assert!(
        allocs <= 4,
        "the typography slot's own growth only ({allocs} allocs for three entries)"
    );
}

#[test]
fn the_shadcn_button_walks_within_its_bound() {
    let _g = lock();
    // The realistic per-element budget: the shadcn Button base string's
    // walk, the answer's owned slots included. The plan §4 row's `0`
    // target names the walk's own machinery; the owned Vec fills are
    // the answer's materialization (recorded in BENCHMARKS.md).
    let program = frozen(
        "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium \
         transition-colors bg-primary text-primary-foreground hover:bg-primary/90",
    );
    let env = Env::default();
    let (allocs, _bytes, (style, _walk)) = measured(|| program.style(&env));
    assert!(style.background.is_some(), "the button's fill landed");
    assert_eq!(
        style.text.len(),
        4,
        "typography landed (whitespace-nowrap rides too)"
    );
    assert!(
        allocs <= 4,
        "the button's walk pays only the answer's own Vec fills ({allocs} allocs)"
    );
}
