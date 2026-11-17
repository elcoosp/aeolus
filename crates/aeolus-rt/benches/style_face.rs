//! The style face's microbenchmarks (divan, stable — the workspace
//! bench idiom): the `StaticProgram::style*` methods, the free walk
//! entries' delegates on the host surface. The delegation cost is the
//! number the face exists to bound — the host-surface call must be the
//! free walk's own cost, the slices threaded once by the type.
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them.

use aeolus_compile::{Config, Env, PseudoTarget, program_style, program_style_pseudo};
use aeolus_rt::{ClassCell, StaticProgram};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The baked shadcn LIGHT table (T0's shape) — the freeze walk's own
/// construction.
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the baked shadcn table builds")
}

/// Build a static program through the dynamic lane's own compile (the
/// ClassCell is the runtime path's constructor): the shadcn Button
/// string grown by a selection layer — one program, both faces.
fn frozen(classes: &str) -> StaticProgram {
    let cell = ClassCell::new();
    cell.get(classes, &Config::default())
        .expect("the cell compiles what lowers")
}

/// The shadcn v4 Button core grown by a selection recolour — one
/// program, the element walk and the pseudo-target cascade both
/// answer it.
const CORPUS: &str = "inline-flex items-center justify-center gap-2 rounded-md bg-primary h-9 px-4 py-2 selection:bg-primary/90";

/// The element walk through the host surface's method.
#[divan::bench]
fn style_face_element(bencher: Bencher) {
    let program = frozen(CORPUS);
    let env = Env::default();
    bencher.bench(|| program.style(&env));
}

/// The element walk through the free entry the method delegates to:
/// the delegation-parity pin's other half (the same slices, threaded
/// by hand).
#[divan::bench]
fn style_free_element(bencher: Bencher) {
    let program = frozen(CORPUS);
    let env = Env::default();
    bencher.bench(|| program_style(program.layers, program.keywords, &env));
}

/// The pseudo-target cascade through the host surface's method.
#[divan::bench]
fn style_face_pseudo(bencher: Bencher) {
    let program = frozen(CORPUS);
    bencher.bench(|| program.style_pseudo(PseudoTarget::Selection));
}

/// The pseudo-target cascade through the free entry.
#[divan::bench]
fn style_free_pseudo(bencher: Bencher) {
    let program = frozen(CORPUS);
    bencher
        .bench(|| program_style_pseudo(PseudoTarget::Selection, program.layers, program.keywords));
}

/// The themed pseudo cascade: the runtime-tier slots resolve through
/// the source (T2's one load + one multiply on the host face).
#[divan::bench]
fn style_face_pseudo_themed(bencher: Bencher) {
    let cell = ClassCell::new();
    let table = baked_table();
    let program = cell
        .get_theme(CORPUS, &table, &Config::default())
        .expect("the themed cell compiles");
    let env = Env::default();
    bencher.bench(|| program.style_pseudo_theme(PseudoTarget::Selection, &table, &env));
}

// ---------------------------------------------------------------------------
// The probe face: the structural and widest walk entries' delegates
// ---------------------------------------------------------------------------

/// The probe face's corpus — the Button core grown by one layer per
/// seam family (attribute, container, structural, descendant,
/// supports), so the widest walk consults every seam it names.
const PROBE_CORPUS: &str = "inline-flex items-center rounded-md bg-red-500 h-9 px-4 \
     data-[state=open]:bg-red-500/90 @md/card:w-64 first:w-40 has-focus:opacity-40 \
     supports-[display:grid]:hidden";

/// The probe corpus's wired seam sources: the attribute, container and
/// walk answers built once per bench fn (the walk consumes the
/// answers, not their construction).
macro_rules! probe_sources {
    () => {{
        use aeolus_compile::{AttrBit, AttrKind, ContainerBound, DescendantFact, DescendantKind};
        let attrs = aeolus_compile::AttrState::new(&[AttrBit {
            kind: AttrKind::Data,
            name: "state",
            value: "open",
        }]);
        let card = aeolus_compile::ContainerState::new(&[ContainerBound {
            name: "card",
            rem: 64.0,
        }]);
        let facts = aeolus_compile::StructFacts::new(0, 2);
        let focused = aeolus_compile::DescendantState::new(&[DescendantFact {
            kind: DescendantKind::Element,
            fragment: "*:focus",
        }]);
        let grid = aeolus_compile::SupportsState::new(&["(display:grid)"]);
        (attrs, card, facts, focused, grid)
    }};
}

/// The structural face through the host surface's method.
#[divan::bench]
fn style_face_struct(bencher: Bencher) {
    let program = frozen(PROBE_CORPUS);
    let facts = aeolus_compile::StructFacts::new(0, 2);
    bencher.bench(|| program.style_struct(&Env::default(), Some(&facts)));
}

/// The structural face through the free entry the method delegates to:
/// the delegation-parity pin's other half (the same slices, threaded
/// by hand).
#[divan::bench]
fn style_free_struct(bencher: Bencher) {
    let program = frozen(PROBE_CORPUS);
    let facts = aeolus_compile::StructFacts::new(0, 2);
    bencher.bench(|| {
        aeolus_compile::program_style_struct(
            program.layers,
            program.keywords,
            &Env::default(),
            Some(&facts),
        )
    });
}

/// The widest walk through the host surface's method: every seam wired,
/// the sources built once outside the benched closure.
#[divan::bench]
fn style_face_widest(bencher: Bencher) {
    let program = frozen(PROBE_CORPUS);
    let env = Env::default();
    let (attrs, card, facts, focused, grid) = probe_sources!();
    bencher.bench(|| {
        program.style_probes_containers_struct_descendants_supports(
            &env,
            Some(&attrs),
            Some(&card),
            Some(&facts),
            Some(&focused),
            Some(&grid),
            None,
        )
    });
}

/// The widest walk through the free entry the method delegates to.
#[divan::bench]
fn style_free_widest(bencher: Bencher) {
    let program = frozen(PROBE_CORPUS);
    let env = Env::default();
    let (attrs, card, facts, focused, grid) = probe_sources!();
    bencher.bench(|| {
        aeolus_compile::program_style_probes_containers_struct_descendants_supports(
            program.layers,
            program.keywords,
            &env,
            Some(&attrs),
            Some(&card),
            Some(&facts),
            Some(&focused),
            Some(&grid),
            None,
        )
    });
}

/// The narrow named-container face through the host surface's method —
/// the host naming ONE seam threads one Option, not six.
#[divan::bench]
fn style_face_containers(bencher: Bencher) {
    use aeolus_compile::ContainerBound;
    let program = frozen(PROBE_CORPUS);
    let env = Env::default();
    let card = aeolus_compile::ContainerState::new(&[ContainerBound {
        name: "card",
        rem: 64.0,
    }]);
    bencher.bench(|| program.style_containers(&env, Some(&card), None));
}

/// The narrow named-container face through the free entry the method
/// delegates to.
#[divan::bench]
fn style_free_containers(bencher: Bencher) {
    use aeolus_compile::ContainerBound;
    let program = frozen(PROBE_CORPUS);
    let env = Env::default();
    let card = aeolus_compile::ContainerState::new(&[ContainerBound {
        name: "card",
        rem: 64.0,
    }]);
    bencher.bench(|| {
        aeolus_compile::program_style_containers(
            program.layers,
            program.keywords,
            &env,
            Some(&card),
            None,
        )
    });
}
