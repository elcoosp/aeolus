//! Consumer demonstration: the V1 static-program lane (architecture
//! plan §7's egui re-target — the fifth backend after GPUI, Floem,
//! Xilem and Iced). A `tw!`-compiled [`StaticProgram`] walks through
//! the pipeline's shared seam into the crate's own [`EguiStyle`] under
//! one probe [`Env`] — the same [`lower_resolved`] mapping the plan
//! path's [`StyleBook`] runs — and the SAME class string goes through
//! the plan bridge to show the two lanes agree. Headless by
//! construction: the walk is pure and the `Frame` payload prints.
//!
//! ```sh
//! cargo run -p aeolus-egui --example program
//! ```

use aeolus_compile::{Config, compile_theme, emit_plan_theme, keyword_alignment, keyword_texts};
use aeolus_egui::{EguiProbe, StyleBook, env_for_probe, lower_program};
use aeolus_rt::StaticProgram;
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
                   font-medium gap-2 disabled:opacity-50 hover:bg-accent";
    let program = frozen(classes);

    // The program lane: the immediate-mode frame converts its EguiProbe
    // once per frame (`env_for_probe`) and the walk folds the frozen
    // layers under it — zero parse, zero hash, one style fold.
    let idle_probe = EguiProbe::default();
    let hover_probe = EguiProbe {
        hovered: true,
        ..EguiProbe::default()
    };
    let disabled_probe = EguiProbe {
        disabled: true,
        ..EguiProbe::default()
    };
    let (idle, report) = lower_program(&program, &env_for_probe(&idle_probe));
    let (hovered, _) = lower_program(&program, &env_for_probe(&hover_probe));
    let (disabled, _) = lower_program(&program, &env_for_probe(&disabled_probe));

    println!("── egui Frame from the V1 static program ──");
    println!("  layers walked:            {}", program.layers.len());
    println!("  fill (idle):              {:?}", idle.frame.fill);
    println!(
        "  text color (idle):        {:?}",
        idle.text.color.map(|c| c.to_array())
    );
    println!("  inner_margin (p-4):       {:?}", idle.frame.inner_margin);
    println!(
        "  width/height hints:       {:?} / {:?}",
        idle.width, idle.height
    );
    println!("  item_spacing (gap-2):     {:?}", idle.item_spacing);
    println!(
        "  corner_radius (rounded-md): {:?}",
        idle.frame.corner_radius
    );
    println!(
        "  hover:bg-accent fires:    {}",
        hovered.frame.fill != idle.frame.fill
    );
    println!(
        "  disabled:opacity-50 bakes: fill alpha {} (egui has no opacity primitive — the CSS \
         composite scales every painted alpha byte)",
        disabled.frame.fill.a()
    );
    println!(
        "  walk report:              {} drops (exclusions and unmapped longhands land here, \
         never a silent no-op)",
        report.dropped.len()
    );

    // The plan bridge: the SAME compiled artifact lowered into the v0.x
    // StylePlan the crate has consumed since v0.1 — the StyleBook the
    // probe path resolves per frame. Two lanes, one compiler, one
    // cascade.
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::baked(&source).expect("the baked shadcn table builds");
    let sheet = emit_plan_theme(classes, &table, &Config::default()).expect("the bridge lowers");
    let (book, book_report) = StyleBook::new(&sheet.plan);
    let dark_probe = EguiProbe {
        dark: true,
        ..EguiProbe::default()
    };
    println!("── the plan bridge agrees ──");
    println!(
        "  walk fill == book fill (idle, light): {}",
        idle.frame.fill == book.resolve_at(&idle_probe).frame.fill
    );
    println!(
        "  walk fill == book fill (dark):        {}",
        lower_program(&program, &env_for_probe(&dark_probe))
            .0
            .frame
            .fill
            == book.resolve_at(&dark_probe).frame.fill
    );
    println!("  book report drops:        {}", book_report.dropped.len());
}
