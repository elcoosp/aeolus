//! Consumer demonstration: the P2 compile seam — a class string becomes
//! a cascade-sorted `Program` of longhand declarations, and every class
//! the seam does not lower comes back as a spanned, coded, helped
//! diagnostic (never a silent drop).
//!
//! ```sh
//! cargo run -p aeolus-compile --example compile_pipeline
//! ```
//!
//! The pipeline reads the three P1 crates end to end: the lexer's
//! candidates (`lex_candidates` on `aeolus-syntax`), the registry's
//! rows (`registry_lookup` on `aeolus-data`), and the IR's masks
//! (`mask_algebra` on `aeolus-ir`) all feed this stage. The landed P2
//! halves consume the `Program` this seam emits: the theme tiers
//! (`theme_seam`), the IR-level `cn()` merge (`cn_merge`), the CSS
//! target (`css_emit`) and the custom-authoring seam
//! (`custom_seam`).

use aeolus_compile::{Config, Diagnostic, DiagnosticKind, Spacing, compile};

fn main() {
    // The shadcn Button's own vocabulary: scale utilities, keyword
    // utilities, a variant stack, colour tokens, arbitrary variants —
    // every boundary the seam reports honestly.
    let classes = "inline-flex rounded-md p-4 px-8 hover:bg-primary/90 h-9 text-sm \
                   focus-visible:ring-[3px] disabled:opacity-50";
    println!("class string : {classes}\n");

    // 1. The default (tailwind) config: rem-based spacing, typed
    //    diagnostics for everything the seam does not lower yet.
    let compiled = compile(classes, &Config::default()).expect("non-empty input compiles");
    print_program(&compiled.program);
    print_diagnostics(&compiled.diagnostics);

    // 2. The px compat config (plan §3.3): the SAME string bakes pixels
    //    against the configured rem size — `p-4` is Rem(1.0) above and
    //    Px(16.0) here.
    println!("\n── px compat config (rem_px = 16) ──────────────────────");
    let config = Config {
        spacing: Spacing::Px,
        ..Config::default()
    };
    let compiled = compile("p-4", &config).expect("non-empty input compiles");
    for decl in compiled.program.layers[0].decls {
        println!("  {} = {:?}", decl.prop.name(), decl.val);
    }

    // 3. The typed miss: an empty class string is a caller bug, not a
    //    styling result — the only whole-string failure.
    let err = compile("   ", &Config::default()).unwrap_err();
    println!("\nempty input → {err}");
}

/// The audit view of one program: layer by layer, condition,
/// specificity rank, then the longhand fan-out with the value slot.
fn print_program(program: &aeolus_ir::Program) {
    println!(
        "── program ({} layers) ─────────────────────────────────",
        program.layers.len()
    );
    for layer in program.layers {
        let cond = layer.cond.source_text().unwrap_or("base");
        println!(
            "  [{cond}] specificity {} ({} longhands)",
            layer.cond.specificity(),
            layer.decls.len()
        );
        for decl in layer.decls {
            println!("    {} = {:?}", decl.prop.name(), decl.val);
        }
    }
    println!("  touched mask: {} longhands", program.touched().len());
    let conflicts = program.base_conflicts();
    if !conflicts.is_empty() {
        println!("  base conflicts (the bits the IR-level cn() merge will take):");
        conflicts.for_each(|prop| println!("    {}", prop.name()));
    }
}

/// The honest-boundary view: every non-lowered class with its span,
/// stable code and help text — the editor-squiggle contract.
fn print_diagnostics(diagnostics: &[Diagnostic]) {
    println!(
        "\n── diagnostics ({} did not lower) ─────────────────────",
        diagnostics.len()
    );
    for d in diagnostics {
        println!("  {d}");
        println!("    code: {}", d.kind.code());
        match &d.kind {
            DiagnosticKind::UnsupportedValue { reason, .. } => {
                println!("    help: {}", d.kind.help());
                println!("    why : {reason}");
            }
            _ => println!("    help: {}", d.kind.help()),
        }
    }
}
