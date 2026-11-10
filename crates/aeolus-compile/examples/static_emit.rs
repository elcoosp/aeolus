//! Consumer demonstration: the static-data emit arm (architecture plan
//! §3.1 emit (b) — the `static` half of the `Target` surface). A
//! compiled program freezes into the IR's OWN static form: one
//! compilable, byte-stable `static PROGRAM: &[aeolus_ir::Layer]` item —
//! the bridge between the pipeline's interned dynamic form and the
//! plan §3.3 "the static form is a `static` item" artifact (the thing a
//! build step writes out once and compiles into the consumer).
//!
//! ```sh
//! cargo run -p aeolus-compile --example static_emit
//! ```
//!
//! The emitted text uses fully-qualified `aeolus_ir::` paths and calls
//! `Layer::new`, so the mask invariant derives at construction — paste
//! it anywhere `aeolus-ir` is a dependency.

use aeolus_compile::{Config, compile_theme, emit_static};
use aeolus_ir::Program;
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    // The theme: the same CSS a web Tailwind v4 project uses. The baked
    // tier freezes colours as literals and dark pairs as ModePair slots;
    // the runtime tier freezes Tok references a runtime table resolves.
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::baked(&source).expect("default-seeded source builds");

    let compiled = compile_theme(
        "bg-primary text-primary-foreground p-4 rounded-md \
         hover:bg-primary/90 md:p-6 dark:shadow-none",
        &table,
        &Config::default(),
    )
    .expect("non-empty input compiles");

    let artifact = compiled.emit_static();
    print!("{artifact}");

    // The artifact is total: feed it back through the IR and the same
    // program stands — masks derived, specificity sorted.
    println!(
        "\n── the static form carries {} layers; emit_static is byte-stable: {} ──",
        compiled.program.layers.len(),
        artifact == emit_static(&compiled.program)
    );

    // The IR-level entry point takes any program, compiled or
    // hand-built (the IR is const-constructible end to end):
    let _ = emit_static(&Program::new(&[]));
}
