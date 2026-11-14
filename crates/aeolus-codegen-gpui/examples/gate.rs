//! Consumer demonstration: the capability GATE at the macro surface —
//! the same `unsupported_reason` verdicts the `Emit` impl renders,
//! run over frozen programs so a class the GPUI target cannot express
//! (the motion families, the gradient via stop) becomes a COMPILE
//! error at `tw!`/`cn!`/`variants!` instead of a silent runtime no-op.
//!
//! The demo audits four class strings and prints the gate's verdicts:
//! what passes (layout + colour + the ring polyfill) and what refuses
//! (motion, the via stop), each refusal naming its longhand and the
//! emitter's own reason.
//!
//! ```sh
//! cargo run -p aeolus-codegen-gpui --example gate
//! ```

use aeolus_codegen_gpui::audit;
use aeolus_compile::{Config, compile_theme, keyword_alignment, keyword_texts};
use aeolus_theme::{ThemeSource, TokenTable};

/// The frozen pair the macros compute (the `tw!` path, verbatim).
fn frozen(literal: &str) -> (aeolus_ir::Program, Vec<&'static str>) {
    let source = ThemeSource::shadcn().expect("embedded palette parses");
    let tokens = TokenTable::baked(&source).expect("the baked table builds");
    let compiled = compile_theme(literal, &tokens, &Config::default()).expect("compiles");
    let annotations = keyword_texts(literal, &Config::default()).expect("annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    (compiled.program, keywords)
}

fn main() {
    let cases = [
        "p-4 bg-red-500 rounded-md hover:ring-4 ring-red-500",
        "animate-spin",
        "transition-colors duration-300",
        "from-red-500 via-blue-500 to-green-500",
    ];
    for literal in cases {
        let (program, keywords) = frozen(literal);
        let refusals = audit(&program, &keywords);
        println!("── {literal} ──");
        if refusals.is_empty() {
            println!("  gate: PASS (the `tw!` freeze proceeds)");
        } else {
            println!("  gate: REFUSE (the `tw!` expansion is a compile error):");
            for refusal in &refusals {
                println!("    {:?}: {}", refusal.prop, refusal.reason);
            }
        }
    }
    println!("\nthe gate is the emitter's own verdicts — the publish rule, at the macro surface");
}
