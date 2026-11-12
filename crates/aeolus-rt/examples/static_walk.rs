//! Consumer demonstration: the host-facing runtime seam — how a backend
//! (or the `tw!` macro's expansion) builds a [`StaticProgram`] and walks
//! it without parsing classes, touching a lock, or allocating: the IR's
//! layers plus the row texts its row-relative `Val::Kw` slots index.
//!
//! ```sh
//! cargo run -p aeolus-rt --example static_walk
//! ```
//!
//! The example mirrors the macro's own freeze: compile once, align the
//! per-class keyword annotations with the compiled program (one text per
//! declaration, layer-major order), hand the two slices to
//! [`StaticProgram::new`] — then read it back the way a backend does
//! (`keyword`/`keyword_for`), and resolve the themed interpreter half
//! under one probe [`Env`].

use std::collections::HashMap;
use std::collections::VecDeque;

use aeolus_compile::{ClassKeywords, Config, compile_theme, keyword_texts};
use aeolus_ir::{Cond, Prop, Val};
use aeolus_rt::{Env, StaticProgram};
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    // The theme: the built-in shadcn source (the same `@theme`/`:root`
    // CSS a web project uses; `theme_from_css` is the file-facing form
    // the `theme!` macro's runtime half wraps).
    let source = ThemeSource::shadcn().expect("the shadcn source parses");
    let tokens = TokenTable::baked(&source).expect("the T0/T1 bake succeeds");
    let config = Config::default();

    // The compile walk (the macro's own): the class string lowers once.
    // (no /opacity modifier: an alpha on a BAKED colour is the theme
    // pass's typed boundary — the runtime tier is the modifier's lane)
    let classes = "inline-flex h-9 px-4 rounded-md text-sm bg-primary hover:bg-primary";
    let compiled = compile_theme(classes, &tokens, &config).expect("non-empty input compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "the demo string lowers clean on the built-in source"
    );
    let annotations: Vec<ClassKeywords> =
        keyword_texts(classes, &config).expect("non-empty input annotates");

    // The alignment: one text per declaration, in layer-major order —
    // the same queue the macro freezes beside the layers (duplicate
    // `(cond, prop)` pairs consume in source order, the assembly's
    // stable order).
    let mut queues: HashMap<(Cond, Prop), VecDeque<&'static str>> = HashMap::new();
    for class in &annotations {
        for &(prop, text) in &class.keywords {
            queues
                .entry((class.cond, prop))
                .or_default()
                .push_back(text);
        }
    }
    let mut keywords: Vec<&'static str> = Vec::new();
    for layer in compiled.program.layers {
        for decl in layer.decls {
            let text = queues
                .get_mut(&(layer.cond, decl.prop))
                .and_then(VecDeque::pop_front)
                .unwrap_or("");
            keywords.push(text);
        }
    }

    // The freeze: layers + parallel row texts — the static form the
    // macro emits and the host consumes. Both slices are `&'static` (the
    // compile interning into the leak pool this example reads through).
    let program = StaticProgram::new(compiled.program.layers, Box::leak(keywords.into()));
    println!("layers: {}", program.layers.len());
    for (li, layer) in program.layers.iter().enumerate() {
        let cond = layer.cond;
        let decls = layer.decls.len();
        println!("  layer {li}: cond {cond:?}, {decls} declaration(s)");
    }

    // The backend read: the row text behind a keyword slot — what the
    // bare IR cannot recover (`hidden` and `block` are both `Kw(0)`).
    let display = program.keyword_for(0, Prop::Display);
    println!("display slot text: {display:?}");
    let padding = program.keyword_for(0, Prop::PaddingTop);
    println!("padding-top slot text: {padding:?}"); // numeric — no text
    let _ = program.keyword(0, 0); // the positional form (layer, decl)

    // The themed half: the same program under one probe Env — the
    // interpreter resolves the effective declarations per the cascade.
    let env = Env {
        dark: true,
        ..Env::default()
    };
    let interped = aeolus_rt::interp(&program.program(), &env).expect("no theme forms");
    for decl in &interped.decls {
        let value = match decl.val {
            Val::Rgba(c) => format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2]),
            Val::Px(v) => format!("{v}px"),
            Val::Rem(v) => format!("{v}rem"),
            Val::Pct(v) => format!("{v}%"),
            Val::Num(v) => format!("{v}"),
            Val::Kw(_) => "(keyword slot — read the row text above)".to_string(),
            _ => "…".to_string(),
        };
        println!("  {} = {value}", decl.prop.name());
    }
    println!("notes: {}", interped.notes.len());
}
