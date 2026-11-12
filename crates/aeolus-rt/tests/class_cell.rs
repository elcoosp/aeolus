//! The dynamic-strings slow lane (architecture plan §3.6's surface (3),
//! the one user-facing surface the static `tw!`/`variants!` pair did not
//! carry): [`aeolus_rt::ClassCell`] is the per-view, UI-thread,
//! single-slot cell — "store in view state; compiled when the string
//! changes, a hash compare per frame otherwise".
//!
//! The pins:
//! * a hit hands back the SAME interned slices (the DynCache contract,
//!   pointer-stable) and costs no compile — the counters prove it;
//! * the slot keys on content AND identity (lane, config, table
//!   address — the v0.x collision-safety rule, never hash-only);
//! * an empty input errors and leaves the last good program standing
//!   (the honest per-frame degradation; errors are never cached);
//! * per-class misses ride the cell's typed diagnostics channel;
//! * a fresh cell renders nothing (the empty program is honest).
//!
//! The DIFFERENTIAL pin (the cell's program equals the `tw!` macro's
//! frozen program for the same string under the same table) rides the
//! gpui crate's tests (`aeolus-gpui/tests/dyn.rs`): the macro emits
//! `::aeolus_rt::…` paths, which resolve inside a crate that depends on
//! `aeolus-rt` — this crate's own integration tests cannot alias
//! themselves to their dependency's name.

use aeolus_compile::{CompileError, Config, Diagnostic};
use aeolus_rt::{ClassCell, ThemeSource, TokenTable};

fn baked_shadcn() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the T0/T1 bake succeeds")
}

/// A hit hands back the same interned slices (pointer-stable, the
/// DynCache contract) and does not compile — the hit/miss counters are
/// the observability seam.
#[test]
fn hit_is_pointer_stable_and_counts() {
    let classes = "p-4 flex-col";
    let config = Config::default();
    let cell = ClassCell::new();
    let first = cell.get(classes, &config).expect("compiles");
    assert_eq!(cell.misses(), 1);
    assert_eq!(cell.hits(), 0);
    let second = cell.get(classes, &config).expect("hits");
    assert_eq!(cell.hits(), 1, "the content match is the fast path");
    assert_eq!(cell.misses(), 1, "no recompile");
    assert_eq!(
        first.layers.as_ptr(),
        second.layers.as_ptr(),
        "the same interned slices"
    );
    assert_eq!(first, second);
}

/// The slot keys on content AND identity: the same string under a
/// DIFFERENT table identity is a miss, not a false hit (the v0.x
/// collision-safety rule — never hash-only).
#[test]
fn identity_change_is_a_miss() {
    let classes = "bg-primary";
    let config = Config::default();
    // Two LIVE tables: distinct addresses (a dropped temporary could
    // reuse the slot, and the identity is the address — the v0.x
    // (ptr, len) precedent).
    let first = baked_shadcn();
    let second = baked_shadcn();
    let cell = ClassCell::new();
    let _ = cell
        .get_theme(classes, &first, &config)
        .expect("first table compiles");
    let _ = cell
        .get_theme(classes, &second, &config)
        .expect("a second table is a distinct identity");
    assert_eq!(cell.misses(), 2, "two tables, two slots' worth of work");
    assert_eq!(cell.hits(), 0);
}

/// A string change swaps the program (the single slot; there is no
/// pool — the bounded pool is the host's `DynCache`, the cell is the
/// per-view fast path).
#[test]
fn string_change_swaps_the_slot() {
    let config = Config::default();
    let cell = ClassCell::new();
    let p4 = cell.get("p-4", &config).expect("compiles");
    let p8 = cell.get("p-8", &config).expect("compiles");
    assert_ne!(p4, p8);
    let again = cell.get("p-4", &config).expect("compiles");
    assert_ne!(again, p8);
    assert_eq!(cell.misses(), 3, "no memory: p-4 again is a fresh miss");
}

/// The empty input errors on EVERY call and the last good program
/// stands — the honest per-frame degradation (errors are never cached,
/// never overwrite the slot).
#[test]
fn empty_input_errors_and_keeps_the_last_good_program() {
    let config = Config::default();
    let cell = ClassCell::new();
    let good = cell.get("p-4", &config).expect("compiles");
    let err = cell
        .get("", &config)
        .expect_err("empty input is a typed error");
    assert_eq!(err, CompileError::Empty);
    assert_eq!(
        cell.misses(),
        2,
        "the miss is counted (the compile cache's contract)…"
    );
    let err2 = cell
        .get("   ", &config)
        .expect_err("whitespace is empty too");
    assert_eq!(err2, CompileError::Empty);
    assert_eq!(cell.misses(), 3);
    assert_eq!(cell.hits(), 0, "…but the ERROR is never cached as a hit");
    assert_eq!(cell.program(), good, "the last good program stands");
}

/// Per-class misses ride the cell's typed diagnostics channel — the
/// same coded diagnostics the compile walk reports, never a silent
/// drop.
#[test]
fn per_class_misses_ride_the_diagnostics_channel() {
    let config = Config::default();
    let cell = ClassCell::new();
    let program = cell
        .get("p-4 not-a-real-utility-xyz", &config)
        .expect("the string still lowers what it can");
    assert!(cell.diagnostics().len() == 1, "one typed miss");
    let diagnostic: &Diagnostic = &cell.diagnostics()[0];
    assert_eq!(diagnostic.raw, "not-a-real-utility-xyz");
    assert!(program.layers[0].mask.contains(aeolus_ir::Prop::PaddingTop));
    // A clean recompile clears the channel (the diagnostics belong to
    // the CURRENT string).
    let _ = cell.get("p-4", &config).expect("compiles");
    assert!(cell.diagnostics().is_empty());
}

/// A fresh cell renders nothing: the empty program is the honest start
/// (no layers, no keyword texts) — a view that never compiled still
/// lays out.
#[test]
fn fresh_cell_renders_nothing() {
    let cell = ClassCell::new();
    let program = cell.program();
    assert!(program.layers.is_empty());
    assert!(program.keywords.is_empty());
}

/// The themed and themeless lanes coexist through the same four entry
/// points the compile seam and the `DynCache` carry — the full compile
/// surface, cell-shaped.
#[test]
fn the_full_compile_surface_is_cell_shaped() {
    let config = Config::default();
    let table = baked_shadcn();
    // Themeless: colour tokens are the typed boundary (diagnostics).
    let cell = ClassCell::new();
    let _ = cell.get("bg-primary", &config).expect("lowers what it can");
    assert!(!cell.diagnostics().is_empty(), "the token boundary reports");
    // Themed: the same string lowers to the baked literal, no misses.
    let themed = ClassCell::new();
    let program = themed
        .get_theme("bg-primary", &table, &config)
        .expect("the baked table resolves the token");
    assert!(themed.diagnostics().is_empty());
    assert!(
        program.layers[0]
            .decls
            .iter()
            .any(|decl| matches!(decl.val, aeolus_ir::Val::Rgba(_)))
    );
}
