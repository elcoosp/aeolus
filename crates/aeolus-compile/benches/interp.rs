//! The interpreter-arm microbenchmarks (divan, stable — the workspace
//! bench idiom): the plan §3.1 emit (c) walk over the same shadcn Button
//! string the compile-seam and dyn-cache benches key on. The per-frame
//! loop's cost is the number this arm exists for: compile ONCE (the
//! dyn-cache's story), interp per env flip — one pass over the interned
//! layers, one map-free binary-search overwrite per declaration.
//!
//! The bench bodies `expect` on the interp: a compiled program resolves
//! under any env (errors are caller bugs — unknown ids, non-finite
//! alphas — and none of those can come out of the pipeline).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (see `reports/10-tdd-evidence/17-emit-arms.md`
//! for the red-to-green record).

use aeolus_compile::{Config, DynCache, Env, interp, interp_theme};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The shadcn v4 Button table verbatim (base + default variant + default
/// size) — the same 33-class string the v0.x hot-path benchmarks, the
/// compile-seam benches and the dyn-cache benches key on.
const BUTTON_CLASSES: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2 has-[>svg]:px-3";

/// The baked-tier table (pairs + literals): the program carries
/// `ModePair` slots the interp selects by mode.
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::baked(&source).expect("default-seeded source builds")
}

/// The runtime-tier table: the program carries `Tok` slots the interp
/// resolves through the one-load-one-multiply.
fn runtime_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

/// The resting env: no probe answers, only the base rule fires.
fn resting_env() -> Env {
    Env::default()
}

/// The busy env: hover, focus-visible, disabled, dark and a wide
/// viewport all answer — every conditional layer the button string
/// folds fires at once.
fn busy_env() -> Env {
    Env {
        dark: true,
        hover: true,
        focus_visible: true,
        disabled: true,
        viewport_rem: 64.0,
        ..Env::default()
    }
}

/// Compile once (outside the measured loop) — the interp's input is the
/// interned program, exactly the cached-loop shape.
fn button_program(table: &TokenTable) -> aeolus_compile::Compiled {
    let cache = DynCache::new();
    cache
        .compile_theme(BUTTON_CLASSES, table, &Config::default())
        .expect("non-empty input compiles")
}

/// The baked tier, resting env: base rule only, pairs select the light
/// half.
#[divan::bench]
fn interp_baked_button_resting(bencher: Bencher) {
    let table = baked_table();
    let compiled = button_program(&table);
    let env = resting_env();
    bencher.bench(|| interp_theme(&compiled.program, &table, &env).expect("the program resolves"));
}

/// The baked tier, busy env: every conditional layer fires — the full
/// cascade read.
#[divan::bench]
fn interp_baked_button_busy(bencher: Bencher) {
    let table = baked_table();
    let compiled = button_program(&table);
    let env = busy_env();
    bencher.bench(|| interp_theme(&compiled.program, &table, &env).expect("the program resolves"));
}

/// The runtime tier, busy env: the T2 multiply rides every token
/// declaration (the plan §3.4 runtime cost, measured).
#[divan::bench]
fn interp_runtime_button_busy(bencher: Bencher) {
    let table = runtime_table();
    let compiled = button_program(&table);
    let env = busy_env();
    bencher.bench(|| interp_theme(&compiled.program, &table, &env).expect("the program resolves"));
}

/// The theme-less walk over the same program: the token boundary stops
/// it immediately — the typed refusal's cost, for the record.
#[divan::bench]
fn interp_themeless_button_busy(bencher: Bencher) {
    let table = baked_table();
    let compiled = button_program(&table);
    let env = busy_env();
    bencher.bench(|| {
        let _ = interp(&compiled.program, &env);
    });
}

// ---------------------------------------------------------------------------
// The attribute-probe seam (the plan §3.5 state-bits surface)
// ---------------------------------------------------------------------------

use aeolus_compile::{AttrBit, AttrKind, AttrState, interp_probes, interp_theme_probes};

/// The element's typed attribute bits: the aria-invalid state the
/// Button string's own layers ask for (`aria-invalid:border-…`,
/// `aria-invalid:ring-…`) plus a data bit nothing fires on — the
/// source-answered miss rides the same walk.
fn invalid_state() -> AttrState<'static> {
    AttrState::new(&[
        AttrBit {
            kind: AttrKind::Aria,
            name: "invalid",
            value: "true",
        },
        AttrBit {
            kind: AttrKind::Data,
            name: "state",
            value: "closed",
        },
    ])
}

/// The themed walk WITH the probe seam: the Button's aria-invalid
/// layers fire under the element's typed bits (the per-frame resolve
/// carries both sources — the DynCache loop's real shape).
#[divan::bench]
fn interp_theme_probes_button_busy(bencher: Bencher) {
    let table = runtime_table();
    let compiled = button_program(&table);
    let env = busy_env();
    bencher.bench(|| {
        interp_theme_probes(&compiled.program, &table, &env, &invalid_state())
            .expect("the program resolves")
    });
}

/// The probe walk over a token-free program (the theme-less probe
/// form): one probe layer fires, one source-answered miss stays quiet,
/// one descendant probe reports — the seam's per-layer costs in one
/// pass.
#[divan::bench]
fn interp_probes_mixed(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(
            "p-4 data-[state=open]:p-8 data-[state=closed]:py-2 has-focus:p-8",
            &Config::default(),
        )
        .expect("non-empty input compiles");
    let env = Env::default();
    bencher.bench(|| {
        interp_probes(&compiled.program, &env, &invalid_state()).expect("the program resolves")
    });
}

/// The same program through the SOURCELESS walk: every attribute layer
/// is the reported exclusion — the note path's cost, for the record.
#[divan::bench]
fn interp_sourceless_mixed(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(
            "p-4 data-[state=open]:p-8 data-[state=closed]:py-2 has-focus:p-8",
            &Config::default(),
        )
        .expect("non-empty input compiles");
    let env = Env::default();
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}
