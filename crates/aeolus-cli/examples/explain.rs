//! Consumer demonstration: the `explain` command's library path (plan
//! §5.5's tooling-parity line, "class → Rust it emits") — the same
//! render the `aeolus explain` subcommand prints, driven in-process the
//! way an editor plugin or build script would. One fragment per lowered
//! class (the embeddable `Layer::new` expression, keyword and token
//! slots annotated with the names only the pipeline can see), one
//! coded diagnostic per class that does not lower.

use aeolus_compile::{Config, emit_rust, emit_rust_theme};
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    // 1. The themeless emit: structural classes lower to their
    //    embeddable fragments; colour tokens are the typed boundary.
    let sheet = emit_rust(
        "p-4 flex-col hover:pointer-events-none bg-primary",
        &Config::default(),
    )
    .expect("non-empty input compiles");

    println!("── fragments (themeless) ──");
    for fragment in &sheet.fragments {
        println!("{}", fragment.class);
        for line in fragment.tokens.lines() {
            println!("  {line}");
        }
        println!();
    }

    println!("── diagnostics (themeless) ──");
    for diagnostic in &sheet.diagnostics {
        println!("error: {diagnostic} [{}]", diagnostic.kind.code());
    }

    // 2. The themed emit (`--theme shadcn`): the same string lowers
    //    further — the token boundary resolves through the runtime
    //    tier's table.
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let tokens = TokenTable::runtime(&source).expect("default-seeded source builds");
    let themed = emit_rust_theme("bg-primary", &tokens, &Config::default())
        .expect("non-empty input compiles");

    println!("── fragments (themed, shadcn) ──");
    for fragment in &themed.fragments {
        println!("{}", fragment.class);
        for line in fragment.tokens.lines() {
            println!("  {line}");
        }
    }
}
