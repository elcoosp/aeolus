//! The V1 static-program lane's microbenchmark (divan, stable — the
//! workspace bench idiom): the per-frame cost of walking a
//! `tw!`-compiled [`StaticProgram`] into a Floem [`Style`] under one
//! [`Env`], the number the re-target's §7 story is judged by (the plan
//! path's own walk measures beside it as the differential baseline).

use divan::Bencher;

use aeolus_compile::{Config, compile_theme, keyword_alignment, keyword_texts};
use aeolus_floem::{lower_program, lower_with_transitions};
use aeolus_rt::{Env, StaticProgram};
use aeolus_theme::{ThemeSource, TokenTable};

/// The shadcn button corpus — the same string the plan path's benches
/// and the interp arm's measure, one compile frozen per bench process.
const BUTTON: &str = "inline-flex items-center justify-center gap-2 whitespace-nowrap \
                      rounded-md text-sm font-medium transition-all \
                      disabled:pointer-events-none disabled:opacity-50 \
                      bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2";

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
    // The per-frame reactive shape: the style closure re-walks under the
    // fresh probe (the `tw_program_env` wire-up's cost).
    let program = frozen(BUTTON);
    bencher.bench(|| {
        lower_program(
            &program,
            &Env {
                hover: true,
                ..Env::default()
            },
        )
    });
}

#[divan::bench]
fn plan_walk_button(bencher: Bencher) {
    // The differential baseline: the plan path's shipped walk over the
    // SAME corpus (the bridge compiled once, outside the timed loop —
    // the lane comparison is the walk itself).
    let sheet = aeolus_compile::emit_plan_theme(
        BUTTON,
        &{
            let source = ThemeSource::shadcn().expect("parses");
            TokenTable::baked(&source).expect("builds")
        },
        &Config::default(),
    )
    .expect("the plan bridge");
    bencher.bench(|| lower_with_transitions(&sheet.plan, None));
}

fn main() {
    divan::main();
}

#[divan::bench]
fn program_walk_probes_wired(bencher: Bencher) {
    // The widest entry's own path (the walk's full probe surface): a
    // measured named container, structural facts, the subtree fact and
    // the capability set answering the layers they know, the
    // sourceless forms keeping their typed reports — the number a host
    // that walks the lane WITH its probe sources judges the face by.
    use aeolus_floem::lower_program_probes_containers_struct_descendants_supports;
    use aeolus_rt::{
        ContainerBound, ContainerState, DescendantFact, DescendantKind, DescendantState,
        StructFacts, SupportsState,
    };
    let program = frozen(
        "p-4 bg-primary @md/card:bg-red-500 first:bg-accent \
         has-focus:shadow-lg supports-[display:grid]:grid",
    );
    let containers = ContainerState::new(&[ContainerBound {
        name: "card",
        rem: 64.0,
    }]);
    let structs = StructFacts::new(0, 3);
    let descendants = DescendantState::new(&[DescendantFact {
        kind: DescendantKind::Element,
        fragment: "*:focus",
    }]);
    let supports = SupportsState::new(&["(display:grid)"]);
    let env = Env::default();
    bencher.bench(|| {
        lower_program_probes_containers_struct_descendants_supports(
            &program,
            &env,
            None,
            Some(&containers),
            Some(&structs),
            Some(&descendants),
            Some(&supports),
            None,
        )
    });
}
