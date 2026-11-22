//! Consumer demonstration: the V1 static-program lane (architecture
//! plan §7's Xilem re-target — the third backend after GPUI and Floem).
//! A `tw_static!`-frozen [`StaticProgram`] resolves through the pipeline's
//! shared walk into the plan path's own [`ResolvedStyle`] under one
//! probe [`Env`] — the SAME per-state resolution the crate's
//! [`StyledWidget`](aeolus_xilem::widget::StyledWidget) consumes — and
//! the SAME class string goes through the plan bridge to show the two
//! lanes agree. Headless by construction: the walk is pure (no Masonry
//! runtime needed to print the resolved slots).
//!
//! ```sh
//! cargo run -p aeolus-xilem --example program
//! ```

use aeolus_compile::{Config, emit_plan_theme};
use aeolus_macro::tw_static;
use aeolus_theme::{ThemeSource, TokenTable};
use aeolus_xilem::widget::{TwEnv, resolved_for_state};
use aeolus_xilem::{env_for_state, lower_program};

fn main() {
    // The shadcn button surface, as a `tw!` call site would compile it.
    let classes = "p-4 bg-primary text-primary-foreground rounded-md w-1/2 h-9 \
                   font-medium gap-2 disabled:opacity-50 hover:bg-accent";
    let program = tw_static!(
        "p-4 bg-primary text-primary-foreground rounded-md w-1/2 h-9 font-medium items-center gap-2 disabled:opacity-50 hover:bg-accent"
    );

    // The program lane: the widget converts its situation once per
    // state flip (`env_for_state`) and the walk folds the macro-frozen layers
    // under it — zero parse, zero hash, one style fold.
    let idle_env = env_for_state(&TwEnv::default(), false, false, false, false);
    let hover_env = env_for_state(&TwEnv::default(), true, false, false, false);
    let disabled_env = env_for_state(&TwEnv::default(), false, false, true, false);
    let (idle, report) = lower_program(&program, &idle_env);
    let (hovered, _) = lower_program(&program, &hover_env);
    let (disabled, _) = lower_program(&program, &disabled_env);

    println!("── Masonry ResolvedStyle from the V1 static program ──");
    println!("  layers walked:            {}", program.layers.len());
    println!(
        "  background (idle):        {:?}",
        idle.background.as_ref().map(|b| b.0.to_srgba())
    );
    println!(
        "  content colour (idle):    {:?}",
        idle.content_color.as_ref().map(|c| c.0.to_srgba())
    );
    println!("  padding (p-4):            {:?}", idle.padding);
    println!("  dimensions (w-1/2 h-9):   {:?}", idle.dimensions);
    println!("  gap (gap-2):              {:?}", idle.gap);
    println!(
        "  corner radius (rounded-md): {:?}",
        idle.corner_radius.map(|r| r.top_left)
    );
    println!(
        "  hover:bg-accent fires:    {}",
        hovered.background != idle.background
    );
    println!(
        "  disabled:opacity-50:      {:?}",
        disabled.opacity.map(|o| o.0)
    );
    println!(
        "  walk report:              {} drops (exclusions and unmapped longhands land here, \
         never a silent no-op)",
        report.dropped.len()
    );

    // The plan bridge: the SAME compiled artifact lowered into the v0.x
    // StylePlan the crate has consumed since v0.1 — the per-state
    // resolution the StyledWidget runs at pre-paint. Two lanes, one
    // compiler, one cascade.
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::baked(&source).expect("the baked shadcn table builds");
    let sheet = emit_plan_theme(classes, &table, &Config::default()).expect("the bridge lowers");
    let planned_idle = resolved_for_state(&sheet.plan, false, false, false, false);
    println!("── the plan bridge agrees ──");
    println!(
        "  walk bg == plan bg (idle):          {}",
        idle.background == planned_idle.background
    );
    println!(
        "  walk padding == plan padding:       {}",
        idle.padding == planned_idle.padding
    );
    println!(
        "  walk radius == plan radius:         {}",
        idle.corner_radius == planned_idle.corner_radius
    );
}
