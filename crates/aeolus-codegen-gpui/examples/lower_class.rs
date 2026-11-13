//! Consumer demonstration: the `lower_class` third-party seam
//! (architecture plan §5's "Expose `aeolus_codegen_gpui::lower_class`"
//! — the gpui-rsx integration surface). One class literal lowers into
//! the `aeolus_rt::StaticProgram` expression a `tw!` expansion embeds,
//! with the §3.7 capability verdicts (native / polyfilled / reported)
//! rendered beside it. Headless by construction: the emit is pure
//! token rendering.
//!
//! ```sh
//! cargo run -p aeolus-codegen-gpui --example lower_class
//! ```

use aeolus_codegen::Emit;
use aeolus_codegen_gpui::{GpuiEmit, expressible, polyfilled, supports};
use aeolus_compile::{Config, compile_theme};
use aeolus_ir::Prop;
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::baked(&source).expect("the baked shadcn table builds");

    // The class literal a gpui-rsx `class="…"` attribute would carry.
    let literal = "ring-2 ring-red-500 ring-offset-2 ring-offset-white bg-primary rounded-md";
    let compiled = compile_theme(literal, &table, &Config::default()).expect("the class lowers");
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );

    // The capability verdicts, straight from the §3.7 table in code.
    println!("── capability verdicts for {literal} ──");
    let mut props: Vec<Prop> = compiled
        .program
        .layers
        .iter()
        .flat_map(|layer| layer.decls.iter().map(|decl| decl.prop))
        .collect();
    props.sort_by_key(|prop| *prop);
    props.dedup();
    for prop in props {
        let verdict = if supports(prop) {
            "native"
        } else if polyfilled(prop) {
            "polyfilled (the runtime composes it)"
        } else {
            "reported (the documented boundary)"
        };
        println!("  {:<24} {verdict}", prop.name());
    }

    // The emit: the static-program expression the macro embeds — ring
    // longhands ride the polyfill tier (`expressible`), so they emit
    // where the native-only gate used to refuse.
    let tokens = GpuiEmit::default()
        .emit(&compiled.program)
        .expect("every longhand is expressible");
    let text = tokens.to_string();
    println!("── the emitted StaticProgram expression ──");
    println!("  {} chars of token stream", text.len());
    println!(
        "  carries StaticProgram::new: {}",
        text.replace(' ', "").contains("StaticProgram::new")
    );
    println!(
        "  ring longhands accepted:    {}",
        expressible(Prop::RingWidth)
    );
}
