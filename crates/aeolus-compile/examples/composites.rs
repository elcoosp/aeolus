//! The composite grammar demo (the value pass's verbatim slot): the
//! `calc()`/`min()`/`max()`/`clamp()` sized functions, the
//! underscore-joined track lists, the url/image functions and the bare
//! `n/d` ratios through every surface the pipeline arms them on — the
//! kind-gated row eligibility, the decoded verbatim text (the
//! underscores ARE spaces), the interp arm's effective declarations,
//! the CSS arm's rule text, the static arm's frozen literal, the
//! IR-level `cn()` merge and the honest boundaries (what refuses, with
//! its typed report).
//!
//! Run: `cargo run -p aeolus-compile --example composites`

use aeolus_compile::{Config, Env, cn, compile, emit_css, emit_rust, interp};

fn main() {
    let config = Config::default();

    // 1. The eligible shapes lower through the verbatim slot — one
    //    declaration per row, the decoded text riding as typed (v4's
    //    own arbitrary contract: the author owns the CSS validity).
    println!("== the composite shapes lower onto the verbatim slot ==");
    let compiled = compile(
        "w-[calc(100%-2rem)] grid-cols-[auto_1fr_auto] grid-rows-[minmax(0,1fr)_2fr] \
         bg-[url(/img/hero.png)] aspect-[16/9]",
        &config,
    )
    .expect("non-empty input compiles");
    println!(
        "diagnostics: {:?} (every eligible shape lowered)",
        compiled.diagnostics.len()
    );
    for decl in compiled.program.layers[0].decls {
        let text = match decl.val {
            aeolus_ir::Val::Verbatim(text) => format!("Verbatim({text:?})"),
            other => format!("{other:?}"),
        };
        println!("  {:?} = {text}", decl.prop);
    }

    // 2. The decode grammar: the underscores ARE spaces (`\_` a literal
    //    underscore), the function forms' parens must balance.
    println!("\n== the underscore decode and the balance boundary ==");
    let decoded = compile("grid-cols-[minmax(0,1fr)_auto]", &config).expect("compiles");
    for decl in decoded.program.layers[0].decls {
        println!("  decoded: {:?}", decl.val);
    }
    for truncated in ["w-[calc(]", "grid-cols-[repeat(2,"] {
        let compiled = compile(truncated, &config).expect("the walk is total");
        println!("  {truncated} refuses: {}", compiled.diagnostics[0]);
    }

    // 3. The kind gate: a composite refuses TYPED on every row whose
    //    slots cannot carry its shape — the shorthand fans, the filter
    //    handles, the shadow lists, the negative flag, the modifier.
    println!("\n== the typed boundaries ==");
    for refused in [
        "p-[calc(100%-2rem)]",
        "blur-[2px_4px]",
        "shadow-[0_1px_2px_#000]",
        "w-[url(x.png)]",
        "-w-[calc(100%-2rem)]",
    ] {
        let compiled = compile(refused, &config).expect("the walk is total");
        println!("  {refused} refuses: {}", compiled.diagnostics[0]);
    }

    // 4. The interp arm: the effective declarations under one probe env
    //    — the per-frame shape a host walks.
    println!("\n== the interp arm: the effective declarations ==");
    let interped = interp(&compiled.program, &Env::default()).expect("all values resolve");
    for decl in &interped.decls {
        println!("  {:?} under {:?}", decl.prop, decl.val);
    }

    // 5. The CSS arm: the rule text the browser cascade consumes.
    println!("\n== the CSS arm ==");
    let sheet = emit_css("w-[calc(100%-2rem)] grid-cols-[auto_1fr_auto]", &config)
        .expect("non-empty input renders");
    println!("{sheet}");

    // 6. The IR-level cn() merge: the verbatim slot folds per longhand,
    //    last occurrence wins.
    println!("== the cn() merge: the verbatim slot per longhand ==");
    let calc = compile("w-[calc(100%-2rem)]", &config).expect("compiles");
    let half = compile("w-1/2", &config).expect("compiles");
    let merged = cn(&[&half, &calc]).expect("merges");
    for decl in merged.program.layers[0].decls {
        println!("  {:?} = {:?}", decl.prop, decl.val);
    }

    // 7. The rust-tokens arm: the per-class fragment carries the
    //    verbatim text the frozen expression embeds.
    println!("\n== the rust-tokens arm ==");
    let fragments = emit_rust("w-[calc(100%-2rem)]", &config).expect("the fragments");
    println!(
        "{}",
        fragments
            .to_string()
            .lines()
            .take(12)
            .collect::<Vec<_>>()
            .join("\n")
    );
}
