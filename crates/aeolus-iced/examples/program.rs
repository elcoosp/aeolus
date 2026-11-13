//! Consumer demonstration: the V1 static-program lane (architecture
//! plan §7's Iced re-target — the fourth backend after GPUI, Floem and
//! Xilem). A `tw!`-compiled [`StaticProgram`] walks through the
//! pipeline's shared seam into the crate's own [`IcedStyle`] under one
//! probe [`Env`] — the same [`lower_resolved`] mapping the plan path's
//! [`StyleBook`] runs — and the SAME class string goes through the plan
//! bridge to show the two lanes agree. Headless by construction: the
//! walk is pure and the Catalog payload prints.
//!
//! ```sh
//! cargo run -p aeolus-iced --example program
//! ```

use aeolus_compile::{Config, compile_theme, emit_plan_theme, keyword_alignment, keyword_texts};
use aeolus_iced::{IcedProbe, StyleBook, env_for_probe, lower_program};
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
    let classes = "p-4 bg-primary text-primary-foreground rounded-md w-1/2 h-9 \
                   font-medium items-center gap-2 disabled:opacity-50 hover:bg-accent";
    let program = frozen(classes);

    // The program lane: the Catalog closure converts its IcedProbe once
    // per frame (`env_for_probe`) and the walk folds the frozen layers
    // under it — zero parse, zero hash, one style fold.
    let idle_probe = IcedProbe::default();
    let hover_probe = IcedProbe {
        hovered: true,
        ..IcedProbe::default()
    };
    let disabled_probe = IcedProbe {
        disabled: true,
        ..IcedProbe::default()
    };
    let (idle, report) = lower_program(&program, &env_for_probe(&idle_probe));
    let (hovered, _) = lower_program(&program, &env_for_probe(&hover_probe));
    let (disabled, _) = lower_program(&program, &env_for_probe(&disabled_probe));

    println!("── iced container::Style from the V1 static program ──");
    println!("  layers walked:            {}", program.layers.len());
    println!(
        "  background (idle):        {:?}",
        idle.container.background
    );
    println!(
        "  text color (idle):        {:?}",
        idle.container.text_color.map(|c| (c.r, c.g, c.b, c.a))
    );
    println!("  padding (p-4):            {:?}", idle.widget.padding);
    println!("  width hint (w-1/2):       {:?}", idle.widget.width);
    println!("  spacing hint (gap-2):     {:?}", idle.widget.spacing);
    println!(
        "  border radius (rounded-md): {:?}",
        idle.container.border.radius
    );
    println!(
        "  hover:bg-accent fires:    {}",
        hovered.container.background != idle.container.background
    );
    println!(
        "  disabled:opacity-50 bakes: bg alpha {:.2} (iced has no opacity primitive — the CSS \
         composite scales every painted alpha)",
        disabled
            .container
            .background
            .map(|bg| match bg {
                iced_core::Background::Color(c) => c.a,
                iced_core::Background::Gradient(_) => f32::NAN,
            })
            .unwrap_or(f32::NAN)
    );
    println!(
        "  walk report:              {} drops (exclusions and unmapped longhands land here, \
         never a silent no-op)",
        report.dropped.len()
    );

    // The plan bridge: the SAME compiled artifact lowered into the v0.x
    // StylePlan the crate has consumed since v0.1 — the StyleBook the
    // Catalog closures resolve per state. Two lanes, one compiler, one
    // cascade.
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::baked(&source).expect("the baked shadcn table builds");
    let sheet = emit_plan_theme(classes, &table, &Config::default()).expect("the bridge lowers");
    let (book, book_report) = StyleBook::new(&sheet.plan);
    println!("── the plan bridge agrees ──");
    println!(
        "  walk bg == book bg (idle, light):   {}",
        idle.container.background == book.container_style(&iced_core::Theme::Light).background
    );
    println!(
        "  walk bg == book bg (dark):          {}",
        lower_program(
            &program,
            &env_for_probe(&IcedProbe {
                dark: true,
                ..IcedProbe::default()
            }),
        )
        .0
        .container
        .background
            == book.container_style(&iced_core::Theme::Dark).background
    );
    println!("  book report drops:        {}", book_report.dropped.len());
}
