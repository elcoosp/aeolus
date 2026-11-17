//! The V1 static-program lane's microbenchmark (divan, stable — the
//! workspace bench idiom): the per-frame cost of walking a
//! `tw!`-compiled [`StaticProgram`] into the plan path's own
//! [`ResolvedStyle`] under one probe [`Env`], the number the re-target's
//! §7 story is judged by (the plan path's own per-state resolution
//! measures beside it as the differential baseline). The bench joins
//! the lane after its first landing — the Floem/Iced/egui lanes
//! measured their walks on day one; this one caught up.

use divan::Bencher;

use aeolus_compile::{Config, compile_theme, keyword_alignment, keyword_texts};
use aeolus_rt::{Env, PseudoTarget, StaticProgram};
use aeolus_theme::{ThemeSource, TokenTable};
use aeolus_xilem::widget::TwEnv;
use aeolus_xilem::{
    env_for_state, lower_program, lower_program_pseudo, lower_program_pseudo_theme,
};

/// The shadcn button corpus — the same string the plan path's benches
/// and the interp arm's measure, one compile frozen per bench process.
const BUTTON: &str = "inline-flex items-center justify-center gap-2 whitespace-nowrap \
                      rounded-md text-sm font-medium transition-all \
                      disabled:pointer-events-none disabled:opacity-50 \
                      bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2";

/// Compile a class string the way `tw!` does (the theme pass's baked
/// shadcn table) and freeze it into the static form the macro emits.
fn frozen(classes: &str) -> StaticProgram {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::baked(&source).expect("the baked shadcn table builds");
    let config = Config::default();
    let compiled = compile_theme(classes, &table, &config).expect("the class string lowers");
    let annotations = keyword_texts(classes, &config).expect("non-empty input annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    StaticProgram::new(
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    )
}

#[divan::bench]
fn program_walk_button(bencher: Bencher) {
    let program = frozen(BUTTON);
    let env = Env::default();
    bencher.bench(|| lower_program(&program, &env));
}

#[divan::bench]
fn program_walk_hover_frame(bencher: Bencher) {
    // The per-frame reactive shape: the widget re-resolves per state
    // flip, the walk under the fresh probe (the `env_for_state`
    // conversion's cost included).
    let program = frozen(BUTTON);
    bencher.bench(|| {
        lower_program(
            &program,
            &env_for_state(&TwEnv::default(), true, false, false, false),
        )
    });
}

#[divan::bench]
fn plan_walk_button(bencher: Bencher) {
    // The differential baseline: the plan path's shipped per-state
    // resolution over the SAME corpus (the bridge compiled once,
    // outside the timed loop — the widget resolves per state at
    // pre-paint, never per string).
    use aeolus_compile::emit_plan_theme;
    use aeolus_xilem::widget::resolved_for_state;
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::baked(&source).expect("the baked shadcn table builds");
    let sheet =
        emit_plan_theme(BUTTON, &table, &Config::default()).expect("the plan bridge lowers");
    let plan = sheet.plan;
    bencher.bench(|| resolved_for_state(&plan, true, false, false, false));
}

fn main() {
    divan::main();
}

#[divan::bench]
fn program_walk_pseudo_target(bencher: Bencher) {
    // The render-target face's own path (the backend half of the
    // pseudo-target seam): one `::selection` cascade lowered from the
    // frozen program through the shared walk's condition-equality
    // verdict and the backend's report translation — the number a
    // host that paints the target judges the face by.
    let program =
        frozen("selection:bg-red-500 selection:bg-primary placeholder:text-muted-foreground p-4");
    bencher.bench(|| lower_program_pseudo(&program, PseudoTarget::Selection));
}

#[divan::bench]
fn program_walk_pseudo_theme_face(bencher: Bencher) {
    // The themed face: the runtime tier's table as the TokenSource, the
    // env riding the resolution (the mode selects the pair halves) —
    // the same verdict walk, one theme ask per fired longhand.
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let runtime = TokenTable::runtime(&source).expect("the runtime table builds");
    let program =
        frozen("selection:bg-red-500 selection:bg-primary placeholder:text-muted-foreground p-4");
    let env = Env::default();
    bencher.bench(|| lower_program_pseudo_theme(&program, PseudoTarget::Selection, &runtime, &env));
}
