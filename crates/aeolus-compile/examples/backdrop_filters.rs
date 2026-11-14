//! The backdrop-filter layer demo (the plan §3.7 backlog row's second
//! half): the `backdrop-blur-*`/`backdrop-brightness-*`/
//! `backdrop-contrast-*`/`backdrop-grayscale-*`/
//! `backdrop-hue-rotate-*`/`backdrop-invert-*`/`backdrop-opacity-*`/
//! `backdrop-saturate-*`/`backdrop-sepia-*`/`backdrop-filter` families
//! through every surface the pipeline arms them on — the typed
//! longhands, the interp arm's effective declarations, the CSS arm's
//! prefixed var-chain form (the primitive's var line beside BOTH
//! composed properties, v4's own emission order), the static arm's
//! frozen item and the honest boundaries.
//!
//! Run: `cargo run -p aeolus-compile --example backdrop_filters`

use aeolus_compile::{Config, Env, compile, emit_css, interp, keyword_texts};

fn main() {
    let config = Config::default();

    // 1. The ten families compile into the IR's backdrop longhands —
    //    one declaration per family; the backdrop-only ninth primitive
    //    (`backdrop-opacity-*`) beside the filter-half mirrors.
    println!("== the backdrop families lower onto the typed longhands ==");
    let compiled = compile(
        "backdrop-blur-md backdrop-brightness-90 backdrop-contrast-125 backdrop-grayscale \
         backdrop-invert backdrop-hue-rotate-30 backdrop-opacity-50 backdrop-saturate-150 \
         backdrop-sepia",
        &config,
    )
    .expect("non-empty input compiles");
    println!(
        "diagnostics: {:?} (every family lowered)",
        compiled.diagnostics.len()
    );
    for decl in compiled.program.layers[0].decls {
        let slot = match decl.val {
            aeolus_ir::Val::Kw(slot) => format!("Kw({slot})"),
            aeolus_ir::Val::Num(n) => format!("Num({n})"),
            other => format!("{other:?}"),
        };
        println!("  {:?} = {slot}", decl.prop);
    }

    // 2. The annotation seam: the row texts the slots ride — the shared
    //    blur scale (v4's `--backdrop-blur`→`--blur` theme fallback),
    //    the 100% bare defaults, the composed chain.
    println!("\n== the annotation seam: the row texts ==");
    let classes = keyword_texts(
        "backdrop-blur backdrop-blur-md backdrop-grayscale backdrop-filter backdrop-filter-none",
        &config,
    )
    .expect("annotates");
    for class in &classes {
        for (prop, text) in &class.keywords {
            let head = if text.len() > 72 {
                format!("{}…", &text[..72])
            } else {
                text.to_string()
            };
            println!("  {prop:?} = {head:?}");
        }
    }

    // 3. The interp arm: the effective declarations under one probe env
    //    — the per-frame shape a host walks.
    println!("\n== the interp arm: the effective declarations ==");
    let interped = interp(&compiled.program, &Env::default()).expect("all values resolve");
    for decl in &interped.decls {
        println!("  {:?} under {:?}", decl.prop, decl.val);
    }

    // 4. The CSS arm: the prefixed var-chain form — each primitive rule
    //    renders its var line beside BOTH composed properties (the
    //    `-webkit-` prefix first, v4's own emission order).
    println!("\n== the CSS arm: the rendered stylesheet ==");
    let sheet = emit_css(
        "backdrop-blur-md backdrop-brightness-90 backdrop-filter-none",
        &config,
    )
    .expect("non-empty input renders");
    print!("{sheet}");

    // 5. The static arm: the byte-stable frozen item (the first lines).
    println!("== the static arm: the frozen item (head) ==");
    let frozen = compiled.emit_static();
    for line in frozen.lines().take(6) {
        println!("  {line}");
    }

    // 6. The honest boundaries: bare `backdrop-brightness` (v4 has no
    //    default) and the composite arbitrary values.
    println!("\n== the honest boundaries ==");
    for class in ["backdrop-brightness", "backdrop-filter-[blur(2px)]"] {
        let compiled = compile(class, &config).expect("the walk is total");
        let kinds: Vec<_> = compiled.diagnostics.iter().map(|d| d.kind.code()).collect();
        println!("  {class:?} -> {kinds:?}");
    }
}
