//! The `expand` command's microbenchmarks (divan, stable — the
//! workspace bench idiom): the interp arm's human-readable form, §3.2's
//! CLI row. Four arms pin the cost split: the themeless effective walk
//! (the keyword texts riding the winner attribution), the themed walk
//! (the tiered table resolving the token forms) and the keyword-heavy
//! render (every slot reading its row text through the `(cond, prop)`
//! annotation table).
//!
//! The benches measure the WALK the command drives (compile → interp →
//! annotation table), not the render's `writeln!` — the sinks are the
//! host's, the same split the `explain` bench keeps.

use aeolus_compile::{Config, Env, compile, compile_theme, interp_theme};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The lowered arm's input: the same shadcn v4 Button slice the CLI
/// pipeline benchmarks key on (all classes lower on the built-in
/// source, the themeless token boundary untouched).
const BUTTON: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2";

/// The keyword-heavy arm's input: every slot is a row-relative `Val::Kw`
/// the render reads its row text for.
const KEYWORDS: &str = "hidden block flex-col items-center justify-center uppercase font-bold rounded-md text-sm whitespace-nowrap";

#[divan::bench]
fn expand_themeless_effective(bencher: Bencher) {
    let compiled = compile(BUTTON, &Config::default()).expect("non-empty input compiles");
    bencher.bench(|| {
        compiled
            .interp(&Env::default())
            .expect("no theme forms in the themeless walk")
    });
}

#[divan::bench]
fn expand_themed_effective(bencher: Bencher) {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let tokens = TokenTable::runtime(&source).expect("default-seeded source builds");
    let compiled =
        compile_theme(BUTTON, &tokens, &Config::default()).expect("non-empty input compiles");
    bencher.bench(|| {
        interp_theme(&compiled.program, &tokens, &Env::default())
            .expect("the themed walk resolves the tokens")
    });
}

#[divan::bench]
fn expand_keyword_slots(bencher: Bencher) {
    let compiled = compile(KEYWORDS, &Config::default()).expect("non-empty input compiles");
    bencher.bench(|| compiled.interp(&Env::default()).expect("no theme forms"));
}

/// The interaction-state flags' CLI face (the `--hover`/`--focus-visible`/
/// `--disabled` facts): the same Button-shaped program — whose base/
/// state layers the string already names — walked with the element-own
/// facts asserted, the state layers riding the same front-to-back read
/// (the winners replace the base per longhand, no extra pass).
#[divan::bench]
fn expand_state_facts(bencher: Bencher) {
    let compiled = compile(BUTTON, &Config::default()).expect("non-empty input compiles");
    let env = Env {
        hover: true,
        focus_visible: true,
        disabled: true,
        ..Env::default()
    };
    bencher.bench(|| compiled.interp(&env).expect("no theme forms"));
}

/// The breakpoint families' CLI face (the `--width rem` viewport fact):
/// the base/state/breakpoint mix the cascade composes — the min forms
/// fire at the asserted width, the max form rides its strictly-below
/// edge, the unasserted tier waits — the same AT/strictly-below
/// boundary read the other fact walks keep.
const BREAKPOINTS: &str = "p-4 md:p-8 hover:p-12 xl:p-16 max-md:p-2";

#[divan::bench]
fn expand_viewport_facts(bencher: Bencher) {
    let compiled = compile(BREAKPOINTS, &Config::default()).expect("non-empty input compiles");
    let env = Env {
        viewport_rem: 48.0,
        ..Env::default()
    };
    bencher.bench(|| compiled.interp(&env).expect("no theme forms"));
}

/// The descendant-probe seam's CLI face (the `--has` facts): the same
/// Button-shaped program through the `*_descendants` walk, the fact
/// table hit and the note path beside it.
const HAS_BUTTON: &str =
    "inline-flex items-center gap-2 has-[>svg]:px-3 has-data-[slot=card-action]:p-8 has-focus:p-8";

#[divan::bench]
fn expand_descendant_facts(bencher: Bencher) {
    use aeolus_compile::{DescendantFact, DescendantKind, DescendantState, interp_descendants};
    let compiled = compile(HAS_BUTTON, &Config::default()).expect("non-empty input compiles");
    let tree = DescendantState::new(&[
        DescendantFact {
            kind: DescendantKind::Element,
            fragment: "*:focus",
        },
        DescendantFact {
            kind: DescendantKind::Element,
            fragment: "*[data-slot=\"card-action\"]",
        },
    ]);
    let env = Env::default();
    bencher.bench(|| {
        interp_descendants(&compiled.program, &env, &tree).expect("the program resolves")
    });
}

/// The supports-variant seam's CLI face (the `--supports` facts): the
/// same Button-shaped program through the `*_supports` walk, the
/// capability table hit and the note path beside it.
const SUPPORTS_BUTTON: &str =
    "inline-flex items-center gap-2 supports-[display:grid]:p-8 supports-[selector(:hover)]:px-3";

#[divan::bench]
fn expand_supports_facts(bencher: Bencher) {
    use aeolus_compile::{SupportsState, interp_supports};
    let compiled = compile(SUPPORTS_BUTTON, &Config::default()).expect("non-empty input compiles");
    let target = SupportsState::new(&["(display:grid)", "selector(:hover)"]);
    let env = Env::default();
    bencher
        .bench(|| interp_supports(&compiled.program, &env, &target).expect("the program resolves"));
}

/// The attribute-probe seam's CLI face (the `--data`/`--aria` facts):
/// the same Button-shaped program through the `*_probes` walk, the
/// bit-table hit (the fired `data-[state=open]`/`aria-busy` layers)
/// beside the base-only arms — the widest dispatch the attribute facts
/// ride on the CLI face.
const ATTR_BUTTON: &str =
    "inline-flex items-center gap-2 data-[state=open]:px-3 aria-busy:opacity-50";

#[divan::bench]
fn expand_attr_facts(bencher: Bencher) {
    use aeolus_compile::{AttrBit, AttrKind, AttrState, interp_probes};
    let compiled = compile(ATTR_BUTTON, &Config::default()).expect("non-empty input compiles");
    let bits = [
        AttrBit {
            kind: AttrKind::Data,
            name: "state",
            value: "open",
        },
        AttrBit {
            kind: AttrKind::Aria,
            name: "busy",
            value: "true",
        },
    ];
    let attrs = AttrState::new(&bits);
    let env = Env::default();
    bencher.bench(|| interp_probes(&compiled.program, &env, &attrs).expect("the program resolves"));
}

/// The custom-authoring seam's CLI face (the `--custom <path>`
/// vocabulary): the registered utility's class through the
/// `compile_with` entry — the composite body's longhands riding the
/// same lookup the registry rows take, the per-invocation compile the
/// CLI's own shape (the expand face compiles fresh every run).
#[divan::bench]
fn expand_custom_utility(bencher: Bencher) {
    use aeolus_compile::{compile_with, customs_from_css};
    let css = "@utility brand-badge { @apply rounded-md px-2 py-0.5; }\n";
    let customs = customs_from_css(css, &Config::default()).expect("the statements register");
    bencher.bench(|| {
        compile_with(
            "brand-badge bg-white rounded-md",
            &customs,
            &Config::default(),
        )
        .expect("non-empty input compiles")
    });
}
