//! Consumer demonstration: the interpreter arm (architecture plan §3.1
//! emit (c) — the `interp` half of the `Target` surface). Compile ONCE,
//! resolve per frame: the same interned program answers "what does this
//! element style RIGHT NOW" for every probe situation — hover flips,
//! dark flips, viewport resizes — without a recompile, a reparse or a
//! second interned copy (the dyn-cache's slices stand).
//!
//! ```sh
//! cargo run -p aeolus-compile --example interp
//! ```
//!
//! The theme dynamism resolves through the `TokenSource` seam: the
//! built-in `TokenTable` answers with the T2 tier's one-load-one-
//! multiply (the audited premultiplied identity); a host (a live theme
//! bridge, a test double) implements the same two questions.

use aeolus_compile::{Config, DynCache, Env, InterpError, interp_theme};
use aeolus_ir::Val;
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    // The theme: the same CSS a web Tailwind v4 project uses, tiered.
    // The baked tier resolves dark pairs by index; the runtime tier
    // carries `Tok` slots the interp multiplies per frame.
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::runtime(&source).expect("default-seeded source builds");

    // Compile ONCE — the DynCache dedupes the interned slices; the
    // interp borrows the program, never re-compiles it.
    let cache = DynCache::new();
    let compiled = cache
        .compile_theme(
            "bg-primary text-primary-foreground p-4 rounded-md \
             hover:bg-primary/90 focus-visible:ring-2 md:p-6 selection:bg-primary/20",
            &table,
            &Config::default(),
        )
        .expect("non-empty input compiles");
    println!(
        "── one program, {} layers interned (diagnostics: {}) ──",
        compiled.program.layers.len(),
        compiled.diagnostics.len()
    );

    // The probe situations a frame can ask about.
    let situations = [
        ("resting light", Env::default()),
        (
            "resting dark",
            Env {
                dark: true,
                ..Env::default()
            },
        ),
        (
            "hovered light",
            Env {
                hover: true,
                ..Env::default()
            },
        ),
        (
            "hovered dark, wide viewport",
            Env {
                dark: true,
                hover: true,
                viewport_rem: 64.0,
                ..Env::default()
            },
        ),
    ];

    for (name, env) in situations {
        // The themed interp: every token asks the table (one load, one
        // multiply for the runtime tier); the honest exclusions ride
        // back as notes.
        let resolved = interp_theme(&compiled.program, &table, &env)
            .expect("the compiled program resolves under any env");
        println!("\n· {name}: {} declarations", resolved.decls.len());
        for decl in &resolved.decls {
            // The interp hands back CONCRETE values: theme forms resolved
            // through the source, keywords riding as the IR slot.
            let value = match decl.val {
                Val::Rgba([r, g, b, a]) => format!("rgba({r}, {g}, {b}, {a})"),
                Val::Rem(rem) => format!("{rem}rem"),
                Val::Px(px) => format!("{px}px"),
                Val::Pct(pct) => format!("{pct}%"),
                Val::Num(n) => format!("{n}"),
                Val::Kw(slot) => format!("keyword #{slot}"),
                Val::Tok { .. } | Val::ModePair(_) => {
                    format!("theme form on {:?} — the interp resolves these", decl.prop)
                }
            };
            println!("    {:<28} {value}", decl.prop.name());
        }
        for note in &resolved.notes {
            println!("    note: {note}");
        }
    }

    // The theme-LESS boundary, for contrast: token values refuse typed,
    // naming the longhand that carried them — never a silent passthrough.
    if let Err(InterpError::ThemeRequired { prop }) = compiled.interp(&Env::default()) {
        println!(
            "\n· the theme-less interp refuses at the first token value: {prop:?} \
             carries a theme form (interp_theme carries the source)"
        );
    }
}
