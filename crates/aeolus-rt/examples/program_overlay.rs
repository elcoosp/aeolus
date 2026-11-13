//! Consumer demonstration: the §3.6 surface (2) note's non-compile-time
//! composition path — a `tw!`-frozen static program overlaid with a
//! RUNTIME-compiled user override ([`overlay`], rightmost wins per
//! longhand, mask-algebra conflict-freedom out).
//!
//! ```sh
//! cargo run -p aeolus-rt --example program_overlay
//! ```
//!
//! The story: the component ships its frozen base (the macro's own
//! `&'static` form); the user's runtime class string compiles through
//! the same pipeline (the dynamic lane's compile), and the two compose
//! at the IR level — no re-parse of the frozen half, no string-level
//! merge, the cascade the longhands encode.

use aeolus_compile::{Config, compile, keyword_alignment, keyword_texts};
use aeolus_rt::{Env, StaticProgram, overlay};
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    // The component's frozen base: compiled once at macro expansion
    // (this example walks the macro's own compile + alignment + freeze
    // — the `static_walk` demo's first half) and shipped as the static
    // form.
    let source = ThemeSource::shadcn().expect("the shadcn source parses");
    let tokens = TokenTable::baked(&source).expect("the T0/T1 bake succeeds");
    let config = Config::default();

    let base_classes = "inline-flex h-9 px-4 rounded-md text-sm";
    let base_compiled = compile_theme(&tokens, base_classes, config);
    let base_annotations = keyword_texts(base_classes, &config).expect("non-empty input annotates");
    let base_keywords = keyword_alignment(&base_compiled, &base_annotations);
    let frozen = StaticProgram::new(
        base_compiled.program.layers,
        Box::leak(base_keywords.into()),
    );

    // The user's runtime override: compiled through the same pipeline
    // (the dynamic lane's lane — here the plain free function; the
    // ClassCell demo carries the caching story).
    let override_classes = "px-8 py-3";
    let override_compiled = compile(override_classes, &config).expect("non-empty input compiles");
    assert!(override_compiled.diagnostics.is_empty());

    // The composition: the frozen base UNDER the runtime override,
    // rightmost wins per longhand — px-8 re-sets left/right over px-4,
    // py-3 adds the vertical half, the frozen half's untouched
    // longhands stand. Conflict-freedom is a mask question
    // (`base_conflicts()` empty), not a table walk.
    let merged = overlay(&frozen.program(), &override_compiled.program);
    println!("layers: {}", merged.layers.len());
    let base = merged.layers[0];
    for decl in base.decls {
        println!("  {:?}: {:?}", decl.prop, decl.val);
    }
    assert!(
        merged.base_conflicts().is_empty(),
        "the composition's conflict-freedom contract"
    );

    // The resolved read (the interp arm's effective form under one
    // probe `Env`): what a backend would walk per frame — the merged
    // program's own declarations, no re-derivation.
    let interped = aeolus_rt::interp(&merged, &Env::default()).expect("the probe interp resolves");
    println!(
        "effective declarations under the resting env: {}",
        interped.decls.len()
    );
}

/// The component base's compile (the macro's own themed walk).
fn compile_theme(tokens: &TokenTable, classes: &str, config: Config) -> aeolus_compile::Compiled {
    let compiled =
        aeolus_compile::compile_theme(classes, tokens, &config).expect("non-empty input compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "the demo string lowers clean on the built-in source"
    );
    compiled
}
