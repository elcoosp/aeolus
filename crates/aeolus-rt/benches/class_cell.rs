//! The dynamic lane's microbenchmarks (divan, stable — the workspace
//! bench idiom): the view-state cell's two costs, split the way a host
//! experiences them.
//!
//! The HIT is the plan §3.6 contract — "compiled when the string
//! changes, a hash compare per frame otherwise" — one exact-content
//! compare plus the interned-program copy, no compile, no allocation.
//! The MISSES run the full V1 pipeline (the same compiler as `tw!`,
//! per the plan's no-smaller-matcher rule): a rotating battery keeps
//! every call a miss (the single slot has no pool — the bounded pool
//! is the host's `DynCache`), themeless and themed.

use aeolus_compile::Config;
use aeolus_rt::{ClassCell, ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The per-frame fast path: the same string every call — a content
/// compare, the counters, the program copy. (`bench_local`: the cell
/// is the UI-thread surface, `!Sync` by design — the plan's own note.)
#[divan::bench]
fn cell_hit(bencher: Bencher) {
    let config = Config::default();
    let cell = ClassCell::new();
    let _ = cell
        .get("p-4 flex-col gap-2 bg-white", &config)
        .expect("compiles");
    bencher.bench_local(|| {
        cell.get("p-4 flex-col gap-2 bg-white", &config)
            .expect("hits")
    });
}

/// The render-side read: what a view's draw call consumes — the last
/// good program, no compile, no probe.
#[divan::bench]
fn cell_program_read(bencher: Bencher) {
    let config = Config::default();
    let cell = ClassCell::new();
    let _ = cell.get("p-4 flex-col", &config).expect("compiles");
    bencher.bench_local(|| cell.program());
}

/// The miss path, themeless: a rotating battery keeps every call a
/// miss — the full lex → lookup → lower → align → intern walk per
/// call (the single slot remembers nothing beyond its own entry).
#[divan::bench]
fn cell_miss_rotating(bencher: Bencher) {
    let config = Config::default();
    let cell = ClassCell::new();
    let strings: Vec<String> = (0..16).map(|i| format!("p-{i} flex-col")).collect();
    let mut index = 0usize;
    bencher.bench_local(|| {
        let classes = strings[index % strings.len()].as_str();
        index += 1;
        let _ = cell.get(classes, &config).expect("compiles");
    });
}

/// The miss path, themed: the runtime token table joins the identity
/// and the lowering (`bg-primary` through the T2 tier's token path).
#[divan::bench]
fn cell_theme_miss(bencher: Bencher) {
    let source = ThemeSource::shadcn().expect("the built-in source parses");
    let tokens = TokenTable::runtime(&source).expect("the runtime table builds");
    let config = Config::default();
    let cell = ClassCell::new();
    let strings: Vec<String> = (0..8).map(|i| format!("bg-primary p-{i}")).collect();
    let mut index = 0usize;
    bencher.bench_local(|| {
        let classes = strings[index % strings.len()].as_str();
        index += 1;
        let _ = cell.get_theme(classes, &tokens, &config).expect("compiles");
    });
}
