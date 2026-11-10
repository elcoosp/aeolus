//! Consumer demonstration: the P2 theme seam — `compile_theme`, the
//! compile signature's theme half (plan §3.2/§3.4). Colour tokens and
//! the `/opacity` modifier lower through the tiered `TokenTable`; the
//! same string WITHOUT a table keeps the typed boundaries the v0.10.0
//! seam pinned.
//!
//! ```sh
//! cargo run -p aeolus-compile --example theme_seam
//! ```
//!
//! The theme pass's own tour lives on `aeolus-theme`
//! (`examples/theme_tiers.rs`); this one is the pipeline view: lex →
//! lookup → lower, now with the colour half resolving.

use aeolus_compile::{Config, DiagnosticKind, compile, compile_theme};
use aeolus_ir::Val;
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    // The shadcn Button's own vocabulary — the string the v0.10.0 seam
    // reported four boundaries on.
    let classes = "inline-flex rounded-md p-4 px-8 hover:bg-primary/90 h-9 text-sm \
                   focus-visible:ring-[3px] disabled:opacity-50 bg-red-500";

    // 1. The theme-less default: colour tokens and modifiers stay typed
    //    boundaries (the honest v0.10.0 contract, unchanged).
    let compiled = compile(classes, &Config::default()).expect("non-empty input compiles");
    println!(
        "── theme-less ({} diagnostics name the theme pass) ──",
        compiled.diagnostics.len()
    );
    for d in &compiled.diagnostics {
        println!("  {d}");
        println!("    code: {}", d.kind.code());
    }

    // 2. The runtime tier (T2): the tokens lower as Tok slots, the /90
    //    modifier rides the alpha — one multiply at theme-resolution
    //    time, the host's business per frame.
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let runtime = TokenTable::runtime(&source).expect("default-seeded source builds");
    let compiled =
        compile_theme(classes, &runtime, &Config::default()).expect("non-empty input compiles");
    println!(
        "\n── runtime tier ({} diagnostics) ──────────────────────",
        compiled.diagnostics.len()
    );
    print_colour_decls(&compiled, &runtime);

    // 3. The baked tier (T0/T1): literals for light-only tokens, const
    //    (light, dark) pairs for dark-overridden ones — and the /90 on
    //    a dark pair is the typed boundary (the pair form has no alpha
    //    arm; the runtime tier above is the answer).
    let baked = TokenTable::baked(&source).expect("default-seeded source builds");
    let compiled = compile_theme(
        "bg-primary hover:bg-primary/90 dark:bg-primary",
        &baked,
        &Config::default(),
    )
    .expect("non-empty input compiles");
    println!(
        "\n── baked tier (tier {:?}) ─────────────────────────────",
        baked.tier()
    );
    for d in &compiled.diagnostics {
        println!("  {d} [{}]", d.kind.code());
    }
    for layer in compiled.program.layers {
        let cond = layer.cond.source_text().unwrap_or("base");
        for decl in layer.decls {
            if matches!(decl.val, Val::Rgba(_) | Val::ModePair(_) | Val::Tok { .. }) {
                println!("  [{cond}] {} = {:?}", decl.prop.name(), decl.val);
            }
        }
    }

    // 4. The checklist seam: a shape-legal but table-absent name reports
    //    the not-a-token reason, whose help names `TokenTable::names()`.
    let bare_source =
        ThemeSource::from_css("/* the v4 default seed only */", None).expect("parses");
    let bare = TokenTable::runtime(&bare_source).expect("builds");
    let compiled =
        compile_theme("bg-primary", &bare, &Config::default()).expect("non-empty input compiles");
    println!("\n── a name the bare-seed table lacks ───────────────────");
    for d in &compiled.diagnostics {
        if let DiagnosticKind::UnsupportedValue { reason, .. } = &d.kind {
            println!("  {d}\n    why: {reason}");
        }
    }
}

/// The colour-form decls of a compiled program, resolved back through
/// the table so the demo prints names, not bare slots.
fn print_colour_decls(compiled: &aeolus_compile::Compiled, table: &TokenTable) {
    for layer in compiled.program.layers {
        let cond = layer.cond.source_text().unwrap_or("base");
        for decl in layer.decls {
            match decl.val {
                Val::Tok { id, alpha } => {
                    let name = table.name(id).unwrap_or("<unknown>");
                    println!(
                        "  [{cond}] {} = Tok {{ {name}, alpha: {alpha} }}",
                        decl.prop.name()
                    );
                }
                Val::Rgba(rgba) => {
                    println!("  [{cond}] {} = Rgba {rgba:?}", decl.prop.name());
                }
                Val::ModePair(slot) => {
                    println!("  [{cond}] {} = ModePair({slot})", decl.prop.name());
                }
                _ => {}
            }
        }
    }
}
