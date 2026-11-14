//! Consumer demonstration: the custom-authoring seam's CSS-FILE form —
//! plan §5.4's "same file as web" line. The `@custom-variant` and
//! `@utility` statements a project's theme CSS carries register onto
//! the SAME typed `Customs` surface the API form uses, and the
//! `@utility name { @apply …; }` bodies compile AT REGISTRATION — the
//! multi-decl composite utilities the fixed/functional forms cannot
//! hold, keyword texts riding the registration beside the decls.
//!
//! ```sh
//! cargo run -p aeolus-compile --example custom_css
//! ```
//!
//! The sheet below is the canonical shape: one variant alias, one
//! composite badge utility (four radii + two paddings — different
//! values per longhand), one keyword-carrying utility, a later utility
//! whose `@apply` references an earlier custom root (the registration
//! order resolves), and a functional star utility (`--spacing(n)`) —
//! the typed grammar's own suffix form, authored in the file.

use aeolus_compile::{
    Config, compile_theme_with, compile_with, customs_from_css, emit_css_with, keyword_alignment,
    keyword_texts_with,
};
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    let sheet = "\
@custom-variant night (&:where(.dark, .dark *));
@utility brand-badge { @apply rounded-md px-2 py-0.5; }
@utility brand-gone { @apply hidden; }
@utility brand-mini { @apply brand-badge text-sm; }
@utility brand-pad-* { padding: --spacing(n); }
";
    let config = Config::default();

    // Registration: the statements parse, the bodies compile through
    // the registration-order customs, everything interns once. A
    // malformed statement, a refused selector or a body that does not
    // compile cleanly is a typed error here — never a quiet skip.
    let customs = customs_from_css(sheet, &config).expect("the sheet registers");
    println!(
        "registered: {} utilities, {} variants",
        customs.utilities().len(),
        customs.variants().len()
    );

    // Compile: the composite utility IS its apply body — the same
    // declarations the body's classes lower to, under the candidate's
    // own condition (`night:brand-badge` splices under the dark mode).
    let compiled = compile_with("night:brand-badge", &customs, &config).expect("compiles");
    for layer in compiled.program.layers {
        println!("layer {:?}:", layer.cond);
        for decl in layer.decls {
            println!("  {:?} = {:?}", decl.prop, decl.val);
        }
    }

    // The annotations: the applied keyword texts ride the registration,
    // so the alignment stays row-true for the body's keyword classes
    // (`hidden` → `none`).
    let annotations = keyword_texts_with("brand-gone", &customs, &config).expect("annotates");
    let texts = keyword_alignment(&compiled, &annotations);
    println!("aligned texts for the badge walk: {:?}", texts.len());

    // The CSS arm: the composite utility renders its apply body's
    // declarations in its own rule, the keyword slots rendering their
    // registration-time row texts.
    let sheet = emit_css_with("brand-badge brand-gone", &customs, &config).expect("renders");
    println!("{sheet}");

    // The functional star form: the suffix lowers through the SAME
    // spacing grammar the registry rows ride; the token form resolves
    // through the tiered tables (the themed lane).
    let compiled = compile_with("brand-pad-6", &customs, &config).expect("compiles");
    println!("brand-pad-6 → {:?}", compiled.program.layers[0].decls[0]);
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let tokens = TokenTable::baked(&source).expect("the T0/T1 bake succeeds");
    let themed =
        compile_theme_with("night:brand-pad-2", &tokens, &customs, &config).expect("compiles");
    for layer in themed.program.layers {
        println!("layer {:?}:", layer.cond);
        for decl in layer.decls {
            println!("  {:?} = {:?}", decl.prop, decl.val);
        }
    }
}
