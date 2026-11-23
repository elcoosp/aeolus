//! Consumer demonstration: the allocation contract on the `.tw()` walk
//! path (plan §4's enforcement cell, the alloc-contract battery's
//! headless face). A counting global allocator wraps the walk the same
//! way the battery does — the printed totals are the per-element,
//! per-frame heap budget the backends inherit.
//!
//! ```sh
//! cargo run -p aeolus-rt --example alloc_contract
//! ```
//!
//! Read-out: the freeze and the Copy-slot walks answer ZERO (the plan
//! §4 row's target, "0 (+GPUI's own boxing)"); the answer's owned
//! `Vec` slots (`ResolvedStyle::text`, the shadow bands) fill with the
//! Vec's own growth pattern — the battery upper-bounds those, the
//! measured numbers recorded in `reports/08-benchmarks/BENCHMARKS.md`.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

use aeolus_compile::{Config, Env};
use aeolus_rt::{ClassCell, StaticProgram, StaticProgramBuilder};
use aeolus_theme::{ThemeSource, TokenTable};

static ALLOCS: AtomicU64 = AtomicU64::new(0);
static TRACKING: AtomicBool = AtomicBool::new(false);
static LOCK: Mutex<()> = Mutex::new(());

struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if TRACKING.load(Ordering::Relaxed) {
            ALLOCS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn lock() -> MutexGuard<'static, ()> {
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The minimum across three runs — the operation's true cost (the
/// print machinery allocates OUTSIDE the windows; noise only adds).
fn measured<T>(mut f: impl FnMut() -> T) -> (u64, T) {
    let mut out = None;
    let mut best = u64::MAX;
    for _ in 0..3 {
        ALLOCS.store(0, Ordering::Relaxed);
        TRACKING.store(true, Ordering::Relaxed);
        let o = f();
        TRACKING.store(false, Ordering::Relaxed);
        best = best.min(ALLOCS.load(Ordering::Relaxed));
        out = Some(o);
    }
    (best, out.unwrap())
}

fn main() {
    let _g = lock();
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::baked(&source).expect("the baked shadcn table builds");
    let env = Env::default();

    let frozen = |classes: &str| -> StaticProgram {
        let cell = ClassCell::new();
        cell.get_theme(classes, &table, &Config::default())
            .expect("the cell compiles what lowers")
    };

    // The freeze: the outlined emit face's assembly.
    let program = frozen("p-4 bg-red-500 rounded-md hover:bg-red-500/80 text-sm font-medium");
    let layers: &[aeolus_ir::Layer] = program.layers;
    let keywords: &[&str] = program.keywords;
    let (allocs, _p) = measured(|| StaticProgramBuilder::assemble(layers, keywords));
    println!("freeze (assemble)                       : {allocs} allocs");

    // The layout families — every slot Copy.
    let layout = frozen("p-4 px-8 gap-4 rounded-lg border-2 items-center justify-center");
    let (allocs, _s) = measured(|| layout.style(&env));
    println!("walk (layout families, Copy slots)      : {allocs} allocs");

    // The colour family — the direct conversion.
    let colour = frozen("bg-red-500 text-red-500 border-red-500");
    let (allocs, _s) = measured(|| colour.style(&env));
    println!("walk (colour family, direct conversion) : {allocs} allocs");

    // The realistic per-element budget: the shadcn Button base.
    let button = frozen(
        "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium \
         transition-colors bg-primary text-primary-foreground hover:bg-primary/90",
    );
    let (allocs, _s) = measured(|| button.style(&env));
    println!(
        "walk (shadcn Button base)               : {allocs} allocs (the answer's own Vec fills)"
    );

    // The render target's own cascade (frozen OUTSIDE the window — the
    // compile allocates by design, the walk must not).
    let pseudo = frozen("before:p-8 before:bg-red-500 selection:bg-red-500");
    let (allocs, _s) = measured(|| pseudo.style_pseudo(aeolus_rt::PseudoTarget::Before));
    println!("walk_pseudo (Before cascade)            : {allocs} allocs");
}
