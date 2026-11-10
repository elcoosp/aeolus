//! Consumer demonstration: the arbitrary-value unit grammar (the value
//! pass's last named P2 slice). Bracket/paren values that the registry
//! rows always ACCEPTED now LOWER: units into the IR's own slots,
//! colours through the audited parser, `/opacity` through the audited
//! premultiplied bake, `var()` references through the theme pass's
//! token path — and every value the IR cannot carry refuses TYPED, with
//! a stable code and help text (the no-silent-failures rule).
//!
//! ```sh
//! cargo run -p aeolus-compile --example arbitrary_values
//! ```
//!
//! The lowering is theme-aware: run the same string through
//! `compile` (theme-less) and `compile_theme` (the tiered table) and
//! compare what lowered and what stayed a typed boundary.

use aeolus_compile::{Config, compile, compile_theme, emit_css};
use aeolus_ir::Val;
use aeolus_theme::{ThemeSource, TokenTable};

/// The battery: units, hints, colours, modifiers, var references, and
/// the typed boundaries the grammar refuses.
const CLASSES: &str = "w-[32px] p-[1.5rem] w-[50%] opacity-[0.37] rotate-[45deg] \
     rounded-[3px] bg-[#0ea5e9] bg-[#0ea5e9]/50 text-[13px] w-[length:32px] \
     bg-[var(--color-primary)] w-[calc(100%-2rem)] grid-cols-[0_1fr]";

fn main() {
    // The theme-less default: concrete values lower, tokens and vars
    // stay typed boundaries.
    let themeless = compile(CLASSES, &Config::default()).expect("non-empty input compiles");
    println!(
        "── theme-less: {} layers, {} diagnostics ──",
        themeless.program.layers.len(),
        themeless.diagnostics.len()
    );
    for layer in themeless.program.layers {
        if layer.cond != aeolus_ir::Cond::Base {
            continue;
        }
        for decl in layer.decls {
            println!("  {:?} = {:?}", decl.prop, describe(&decl.val));
        }
    }
    for diagnostic in &themeless.diagnostics {
        println!("  ! {diagnostic} [{}]", diagnostic.kind.code());
    }

    // The themed form: the var reference lowers through the token path.
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::runtime(&source).expect("default-seeded source builds");
    let themed =
        compile_theme(CLASSES, &table, &Config::default()).expect("non-empty input compiles");
    println!(
        "\n── themed: {} diagnostics (the var reference lowered) ──",
        themed.diagnostics.len()
    );
    for diagnostic in &themed.diagnostics {
        println!("  ! {diagnostic} [{}]", diagnostic.kind.code());
    }

    // The CSS arm renders the same resolution — the corpus's own forms:
    let sheet = emit_css(
        "pl-[7px] bg-[#0ea5e9] w-[1.5rem] text-[13px] bg-[#0ea5e9]/50",
        &Config::default(),
    )
    .expect("non-empty input compiles");
    println!("\n── the CSS arm ──\n{sheet}");
}

/// The IR value in the demo's own shorthand (the full `Debug` is the
/// IR's; this keeps the printout readable).
fn describe(val: &Val) -> String {
    match val {
        Val::Px(v) => format!("Px({v})"),
        Val::Rem(v) => format!("Rem({v})"),
        Val::Pct(v) => format!("Pct({v})"),
        Val::Num(v) => format!("Num({v})"),
        Val::Rgba([r, g, b, a]) => format!("Rgba(#{r:02x}{g:02x}{b:02x}{a:02x})"),
        Val::Tok { id, alpha } => format!("Tok(id {id}, alpha {alpha})"),
        Val::ModePair(slot) => format!("ModePair({slot})"),
        Val::Kw(slot) => format!("Kw({slot})"),
    }
}
