//! Consumer demonstration: the attribute-probe variant fold (the P2
//! slice the variant docs carried as "the next slice"). The `data-*`,
//! `aria-*` and `has-*` spellings a real component table uses fold onto
//! the typed `Cond` surface and lower, render and emit through the SAME
//! pipeline as the built-in spellings — the differential oracle is the
//! v0.x resolver, pinned against the real tailwindcss v4.1.17 output in
//! the golden corpus (`[data-state="open"]`, `[aria-busy="true"]`,
//! `:has(*:focus)`, `:has(*[data-slot="card-action"])`).
//!
//! ```sh
//! cargo run -p aeolus-compile --example attr_variants
//! ```
//!
//! The interpreter arm's honest boundary shows here too: the probe
//! layers never fire under the element-state `Env` (the plan §3.5
//! state-bits surface is the follow-up) — they come back as typed
//! notes, never silent, never an error.

use aeolus_compile::{Config, DynCache, Env, emit_css_theme, interp_theme};
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    // The theme: the same CSS a web Tailwind v4 project uses, tiered.
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::runtime(&source).expect("default-seeded source builds");

    // The probe spellings a real component table carries — the corpus's
    // own classes plus the shadcn-ish bare aria form. Compile ONCE (the
    // dyn-cache's story): the fold interns each distinct probe payload
    // once.
    let cache = DynCache::new();
    let compiled = cache
        .compile_theme(
            "data-[state=open]:bg-primary data-open:ring-2 aria-busy:opacity-50 \
             has-focus:border-ring has-data-[slot=card-action]:grid-cols-2 p-4",
            &table,
            &Config::default(),
        )
        .expect("non-empty input compiles");

    println!(
        "── one program, {} layers (diagnostics: {}) ──",
        compiled.program.layers.len(),
        compiled.diagnostics.len()
    );
    for layer in compiled.program.layers {
        println!("  {:?} → {} decl(s)", layer.cond, layer.decls.len());
    }

    // The CSS form: the probes compound onto the escaped class exactly
    // as the real compiler renders them.
    let sheet = emit_css_theme(
        "data-[state=open]:bg-primary aria-busy:opacity-50 has-focus:ring-2",
        &table,
        &Config::default(),
    )
    .expect("non-empty input renders");
    println!("── the CSS form ──");
    print!("{sheet}");

    // The runtime form: the probe layers report as typed notes (the Env
    // answers element states, not DOM probes); the base layer resolves.
    let interped = interp_theme(&compiled.program, &table, &Env::default())
        .expect("the pipeline never emits an unresolvable value");
    println!("── the runtime form (resting env) ──");
    println!("  {} resolved decl(s)", interped.decls.len());
    for note in &interped.notes {
        println!("  note: {note}");
    }

    // The static form: the payload conds emit as constructors (the
    // derived Debug IS the constructor text).
    println!("── the static form (head) ──");
    for line in compiled.emit_static().lines().take(6) {
        println!("  {line}");
    }
}
