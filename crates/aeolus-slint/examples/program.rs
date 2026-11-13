//! Consumer demonstration: the V1 static-program lane (architecture
//! plan §7's slint re-target — the sixth backend after GPUI, Floem,
//! Xilem, Iced and egui). A `tw!`-compiled [`StaticProgram`] walks
//! through the pipeline's shared seam into the crate's own
//! [`SlintStyle`] under one probe [`Env`] — the same [`lower_resolved`]
//! mapping the plan path's [`StyleBook`] runs — and the SAME class
//! string goes through the plan bridge to show the two lanes agree.
//! Headless by construction: the walk is pure and the property payload
//! prints.
//!
//! ```sh
//! cargo run -p aeolus-slint --example program
//! ```

use aeolus_compile::{Config, compile_theme, emit_plan_theme, keyword_alignment, keyword_texts};
use aeolus_rt::StaticProgram;
use aeolus_slint::{SlintProbe, StyleBook, env_for_probe, lower_program};
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
    // The shadcn button surface, as a `tw!` call site would compile it.
    let classes = "p-4 bg-primary text-primary-foreground rounded-md w-24 h-9 \
                   font-medium disabled:opacity-50 hover:bg-accent";
    let program = frozen(classes);

    // The program lane: the element converts its SlintProbe once per
    // state flip (`env_for_probe`) and the walk folds the frozen layers
    // under it — zero parse, zero hash, one style fold.
    let idle_probe = SlintProbe::default();
    let hover_probe = SlintProbe {
        has_hover: true,
        ..SlintProbe::default()
    };
    let disabled_probe = SlintProbe {
        enabled: false,
        ..SlintProbe::default()
    };
    let (idle, report) = lower_program(&program, &env_for_probe(&idle_probe));
    let (hovered, _) = lower_program(&program, &env_for_probe(&hover_probe));
    let (disabled, _) = lower_program(&program, &env_for_probe(&disabled_probe));

    println!("── SlintStyle from the V1 static program ──");
    println!("  layers walked:            {}", program.layers.len());
    println!(
        "  background (idle):        {}",
        idle.background
            .as_ref()
            .map_or_else(|| "none".to_string(), aeolus_slint::brush_snippet,)
    );
    println!(
        "  text color (idle):        {}",
        idle.text
            .color
            .as_ref()
            .map_or_else(|| "none".to_string(), aeolus_slint::brush_snippet)
    );
    println!(
        "  padding hint (p-4):       {:?}",
        idle.padding.map(|p| (p.top, p.right, p.bottom, p.left))
    );
    println!(
        "  width/height:             {:?} / {:?}",
        idle.width.map(aeolus_slint::SlintLength::snippet),
        idle.height.map(aeolus_slint::SlintLength::snippet)
    );
    println!(
        "  corner_radius (rounded-md): {:?}",
        idle.border_radius.map(|r| r.top_left)
    );
    println!(
        "  hover:bg-accent fires:    {}",
        hovered.background != idle.background
    );
    println!(
        "  disabled:opacity-50:      opacity {:?} (Slint's native per-element property)",
        disabled.opacity
    );
    println!(
        "  base snippet:             {} bindings",
        idle.snippet().lines().count()
    );
    println!(
        "  walk report:              {} drops (exclusions and unmapped longhands land here, \
         never a silent no-op)",
        report.dropped.len()
    );

    // The plan bridge: the SAME compiled artifact lowered into the v0.x
    // StylePlan the crate has consumed since v0.1 — the StyleBook the
    // probe path resolves per state. Two lanes, one compiler, one
    // cascade.
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::baked(&source).expect("the baked shadcn table builds");
    let sheet = emit_plan_theme(classes, &table, &Config::default()).expect("the bridge lowers");
    let (book, book_report) = StyleBook::new(&sheet.plan);
    let dark_probe = SlintProbe {
        dark: true,
        ..SlintProbe::default()
    };
    println!("── the plan bridge agrees ──");
    println!(
        "  walk bg == book bg (idle, light): {}",
        idle.background == book.resolve_at(&idle_probe).background
    );
    println!(
        "  walk bg == book bg (dark):        {}",
        lower_program(&program, &env_for_probe(&dark_probe))
            .0
            .background
            == book.resolve_at(&dark_probe).background
    );
    println!("  book report drops:        {}", book_report.dropped.len());
}
