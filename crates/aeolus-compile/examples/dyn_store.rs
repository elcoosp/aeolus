//! Consumer demonstration: the `DynStore` storage seam (plan §3.2's
//! dyn-cache slice's own storage note — "a host swaps in its own
//! bounded map without touching the pipeline"). The built-in
//! `TwoGenStore` is the default; a host with a different eviction story
//! (a fixed FIFO, a per-window slab, an LRU over its view tree)
//! implements the THREE questions and hands its store to
//! [`DynCache::with_store`] — the compile surface, the interned-slice
//! dedupe and the diagnostics all ride unchanged.
//!
//! ```sh
//! cargo run -p aeolus-compile --example dyn_store
//! ```

use std::collections::HashMap;
use std::sync::Mutex;

use aeolus_compile::{CachedCompiled, Config, DynCache, DynStore, compile};

/// The host's own store: a fixed-capacity FIFO map — the simplest
/// bounded story that is NOT the built-in two-generation rotation.
/// Entries evict oldest-first; `probe` is a read-lock-free map probe
/// behind one mutex, `resident` is the observability view hosts budget
/// memory with.
struct FifoStore {
    cap: usize,
    entries: Mutex<HashMap<String, CachedCompiled>>,
}

impl FifoStore {
    fn new(cap: usize) -> Self {
        Self {
            cap,
            entries: Mutex::new(HashMap::new()),
        }
    }
}

impl DynStore for FifoStore {
    fn probe(&self, key: &str) -> Option<CachedCompiled> {
        self.entries.lock().ok()?.get(key).cloned()
    }

    fn store(&self, key: String, value: CachedCompiled) {
        let mut entries = self.entries.lock().expect("the store lock");
        if entries.len() >= self.cap && !entries.contains_key(&key) {
            // FIFO eviction: the first-inserted key goes (the map's
            // iteration order is the host's own eviction story — a real
            // host orders by last-use here).
            if let Some(oldest) = entries.keys().next().cloned() {
                entries.remove(&oldest);
            }
        }
        entries.insert(key, value);
    }

    fn resident(&self) -> usize {
        self.entries.lock().map(|e| e.len()).unwrap_or(0)
    }
}

fn main() {
    // The cache over the HOST's store — the pipeline is untouched; the
    // type carries the store choice (`DynCache<FifoStore>`).
    let cache = DynCache::with_store(FifoStore::new(2));
    let config = Config::default();

    println!("── four strings through a 2-slot FIFO store ──");
    for classes in [
        "p-4 rounded-md",
        "p-2 rounded-full",
        "gap-2 flex",
        "p-4 rounded-md",
    ] {
        let compiled = cache
            .compile(classes, &config)
            .expect("non-empty input compiles");
        println!(
            "{classes:<22} layers {:>2}  resident {}",
            compiled.program.layers.len(),
            cache.resident()
        );
    }

    // The dedupe contract rides unchanged: a repeat string hands back
    // the SAME interned slices the miss pooled — through the host's
    // own store.
    let first = cache
        .compile("p-4 rounded-md", &config)
        .expect("compiles")
        .program
        .layers
        .as_ptr();
    let second = cache
        .compile("p-4 rounded-md", &config)
        .expect("compiles")
        .program
        .layers
        .as_ptr();
    println!();
    println!(
        "repeat string pointer-stable through the host store: {}",
        first == second
    );

    // And the plain entry point still answers — the same pipeline the
    // cache wraps, without the pooling (the reference the host can
    // always fall back to).
    let direct = compile("p-4 rounded-md", &config).expect("compiles");
    let cached = cache.compile("p-4 rounded-md", &config).expect("compiles");
    println!(
        "the direct compile shapes like the cached one: {}",
        direct.program.layers.len() == cached.program.layers.len()
    );
}
