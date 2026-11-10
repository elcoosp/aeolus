//! Consumer demonstration: the `DynCache` — the plan §3.2 leak-pool
//! dedupe. A UI loop recompiles the same dynamic class string every
//! frame; without the cache each frame pools a fresh interned copy of
//! the program (the storage note's "one compile per distinct class
//! string is the intended call pattern" constraint), with it the first
//! frame lowers and every later frame hands back the SAME interned
//! slices — one map probe, zero pipeline passes.
//!
//! ```sh
//! cargo run -p aeolus-compile --example dyn_cache
//! ```
//!
//! The string here is DYNAMIC (app-state-composed, the v0.x
//! `PlanCache`'s own subject): static class strings are `tw!`'s
//! compile-time surface and never touch this path.

use aeolus_compile::{Config, DynCache, compile};

fn main() {
    let config = Config::default();
    let cache = DynCache::new();

    // The app state drives the string: the same composition every frame
    // in the steady state (the case the cache exists for).
    let selected = true;
    let frame_string = if selected {
        "p-4 hover:bg-primary/90 rounded-md"
    } else {
        "p-4 rounded-md"
    };

    println!("── 60 frames of the same dynamic string ──");
    let mut first_layers: *const _ = std::ptr::null();
    for frame in 0..60 {
        let compiled = cache
            .compile(frame_string, &config)
            .expect("non-empty input compiles");
        if frame == 0 {
            first_layers = compiled.program.layers.as_ptr();
            println!(
                "  frame 0: lowered ({} layers), slices interned at {first_layers:p}",
                compiled.program.layers.len()
            );
        } else if frame == 59 {
            let same = compiled.program.layers.as_ptr() == first_layers;
            println!(
                "  frame 59: {} layers, same interned slices as frame 0: {same}",
                compiled.program.layers.len()
            );
        }
    }

    // The honest cross-check: the cached result IS the free compile's
    // result — byte-identical, not an approximation.
    let free = compile(frame_string, &config).expect("non-empty input compiles");
    let cached = cache
        .compile(frame_string, &config)
        .expect("non-empty input compiles");
    println!("  cached == free compile: {}", cached == free);

    // The observability view: one miss (the first frame), the rest
    // hits, and the resident key set the cache holds.
    println!(
        "  hits={} misses={} resident={}",
        cache.hits(),
        cache.misses(),
        cache.resident()
    );

    // The churn case: distinct strings rotate — the two-generation
    // rotation bounds the resident map while every result stays
    // correct (an evicted string re-lowers; its old slices stay in the
    // leak pool, the IR's documented 'static storage).
    println!("── churn: 128 distinct strings through a 32-slot cache ──");
    let small = DynCache::with_cap(32);
    for n in 0..128u32 {
        small
            .compile(&format!("p-{n} gap-{n}"), &config)
            .expect("non-empty input compiles");
    }
    println!(
        "  hits={} misses={} resident={} (bound: 2 x 32 = 64)",
        small.hits(),
        small.misses(),
        small.resident()
    );
}
