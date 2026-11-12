//! Consumer demonstration: the V1 static-program lane (architecture
//! plan §7's Floem re-target — the second backend after GPUI). A
//! `tw!`-compiled [`StaticProgram`] walks into a Floem [`Style`] under
//! the interp's [`Env`] probe semantics, and the SAME class string goes
//! through the plan bridge to show the two lanes agree. Headless by
//! construction: the walk is pure.
//!
//! ```sh
//! cargo run -p aeolus-floem --example static_program
//! ```

use floem::style::{Background, PaddingLeft};

use aeolus_compile::{Config, compile_theme, emit_plan_theme, keyword_alignment, keyword_texts};
use aeolus_floem::{lower, lower_program};
use aeolus_rt::{Env, StaticProgram};
use aeolus_theme::{ThemeSource, TokenTable};

/// Compile a class string the way `tw!` does (the theme pass's baked
/// shadcn table) and freeze it into the static form the macro emits:
/// the interned layers plus the row texts the keyword slots index.
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

fn main() {
    // The shadcn card surface, as a `tw!` call site would compile it.
    let classes = "p-4 bg-card text-card-foreground rounded-md w-1/2 h-9 \
                   font-medium items-center gap-2 disabled:opacity-50 hover:bg-accent";
    let program = frozen(classes);

    // The program lane: the walk folds the layers under one probe Env —
    // the frame situation `tw_program_env` re-reads per style pass.
    let idle = lower_program(&program, &Env::default());
    let hovered = lower_program(
        &program,
        &Env {
            hover: true,
            ..Env::default()
        },
    );
    let disabled = lower_program(
        &program,
        &Env {
            disabled: true,
            ..Env::default()
        },
    );

    println!("── floem Style from the V1 static program ──");
    println!("  layers walked:         {}", program.layers.len());
    println!("  padding-left (p-4):    {:?}", idle.0.get(PaddingLeft));
    println!(
        "  background (brush):    {:?}",
        idle.0.get(Background).is_some()
    );
    println!(
        "  hover:bg-accent fires: {}",
        hovered.0.get(Background) != idle.0.get(Background)
    );
    println!(
        "  disabled:opacity-50:   {}",
        disabled.0.get(floem::style::Opacity) < 1.0
    );
    println!(
        "  walk report: {} drops (exclusions and unmapped longhands land here, never a silent no-op)",
        idle.1.dropped.len()
    );

    // The plan bridge: the SAME compiled artifact lowered into the v0.x
    // StylePlan the crate has consumed since v0.1 — two lanes, one
    // compiler, one cascade (tests/static_program.rs pins the agreement).
    let sheet =
        emit_plan_theme(classes, &frozen_table(), &Config::default()).expect("the plan bridge");
    let (bridged, report) = lower(&sheet.plan);
    println!("── the plan bridge, same classes ──");
    println!(
        "  padding-left agrees:   {}",
        bridged.get(PaddingLeft) == idle.0.get(PaddingLeft)
    );
    println!(
        "  background agrees:     {}",
        bridged.get(Background).is_some() == idle.0.get(Background).is_some()
    );
    println!("  bridge report: {} drops", report.dropped.len());
}

fn frozen_table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the baked shadcn table builds")
}
