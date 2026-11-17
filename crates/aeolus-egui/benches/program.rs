//! The V1 static-program lane's microbenchmark (divan, stable — the
//! workspace bench idiom): the per-frame cost of walking a
//! `tw!`-compiled [`StaticProgram`] into the crate's [`EguiStyle`]
//! under one probe [`Env`], the number the re-target's §7 story is
//! judged by (the plan path's own resolution measures beside it as the
//! differential baseline).

use divan::Bencher;

use aeolus_compile::{Config, compile_theme, keyword_alignment, keyword_texts};
use aeolus_egui::{EguiProbe, env_for_probe, lower_program, lower_resolved};
use aeolus_rt::{Env, StaticProgram};
use aeolus_theme::{ThemeSource, TokenTable};

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

/// The gradient corpus — the mesh-painter seam's per-frame cost: the
/// payload bakes at lowering time (the stop conversions inside the
/// walk), the strip rebuilds per rect at paint time.
const GRADIENT: &str = "w-64 h-16 rounded-lg bg-linear-to-r from-red-500 \
                        via-white to-blue-500 opacity-90";

#[divan::bench]
fn program_walk_linear_gradient(bencher: Bencher) {
    let program = frozen(GRADIENT);
    let env = Env::default();
    bencher.bench(|| lower_program(&program, &env));
}

#[divan::bench]
fn gradient_mesh_to_r(bencher: Bencher) {
    // The per-frame mesh build the host runs beside the frame paint:
    // the payload baked ONCE outside the timed loop, the strip rebuilt
    // per rect every frame (immediate mode redraws everything).
    let program = frozen(GRADIENT);
    let (style, _) = lower_program(&program, &Env::default());
    let paint = style.gradient.expect("the gradient corpus lowers");
    let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(256.0, 64.0));
    bencher.bench(|| aeolus_egui::gradient_mesh(rect, &paint));
}

/// The radial corpus — the same stops over the ring-band shape (the
/// segmentation and rect clip ride the same seam).
const RADIAL: &str = "w-64 h-16 rounded-lg bg-radial from-red-500 \
                      via-white to-blue-500 opacity-90";

#[divan::bench]
fn gradient_mesh_radial(bencher: Bencher) {
    let program = frozen(RADIAL);
    let (style, _) = lower_program(&program, &Env::default());
    let paint = style.gradient.expect("the radial corpus lowers");
    let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(256.0, 64.0));
    bencher.bench(|| aeolus_egui::gradient_mesh(rect, &paint));
}

/// The conic corpus — the angular sectors (the circumscribed fan and
/// the rect clip ride the same seam).
const CONIC: &str = "w-64 h-16 rounded-lg bg-conic from-red-500 \
                     via-white to-blue-500 opacity-90";

#[divan::bench]
fn gradient_mesh_conic(bencher: Bencher) {
    let program = frozen(CONIC);
    let (style, _) = lower_program(&program, &Env::default());
    let paint = style.gradient.expect("the conic corpus lowers");
    let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(256.0, 64.0));
    bencher.bench(|| aeolus_egui::gradient_mesh(rect, &paint));
}

#[divan::bench]
fn program_walk_hover_frame(bencher: Bencher) {
    // The per-frame immediate-mode shape: the frame re-walks under the
    // fresh probe every pass (the `env_for_probe` conversion included).
    let program = frozen(BUTTON);
    let probe = EguiProbe {
        hovered: true,
        ..EguiProbe::default()
    };
    bencher.bench(|| lower_program(&program, &env_for_probe(&probe)));
}

#[divan::bench]
fn plan_walk_button(bencher: Bencher) {
    // The differential baseline: the plan path's shipped walk over the
    // SAME corpus (the bridge compiled once, outside the timed loop —
    // the book resolves per state at read time, never per string).
    use aeolus_compile::emit_plan_theme;
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::baked(&source).expect("the baked shadcn table builds");
    let sheet =
        emit_plan_theme(BUTTON, &table, &Config::default()).expect("the plan bridge lowers");
    let plan = sheet.plan;
    let probe = EguiProbe {
        hovered: true,
        ..EguiProbe::default()
    };
    bencher.bench(|| {
        let resolved = plan.resolve(&probe);
        let mut report = aeolus_egui::LoweringReport::default();
        lower_resolved(&resolved, &mut report)
    });
}

fn main() {
    divan::main();
}
