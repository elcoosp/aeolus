//! Consumer demonstration: the custom-authoring seam — plan §5.4's
//! `@utility`/`@custom-variant` surface. Projects REGISTER custom
//! utilities (fixed literals and functional forms) and custom variant
//! aliases, and the pipeline lowers and renders them exactly like the
//! built-in surface: the same typed read side, the same value pass, the
//! same diagnostics, the same CSS emit paths.
//!
//! ```sh
//! cargo run -p aeolus-compile --example custom_seam
//! ```
//!
//! The three registrations below are the canonical shapes:
//!
//! * `brand-p-*` — a FUNCTIONAL spacing utility (`--spacing(n)`), the
//!   `@utility brand-p-* { padding: --spacing(n); }` spelling;
//! * `brand-r` — a FIXED literal utility (the v4 `@utility brand-r`
//!   spelling, a baked rem value);
//! * `hocus` — a custom VARIANT alias folding to the typed `Cond::Hover`
//!   probe (the flat model's honest reading of `&:hover`).

use aeolus_compile::{
    CustomForm, CustomUtility, CustomVariant, Customs, DiagnosticKind, compile_theme_with,
    emit_css_theme_with,
};
use aeolus_ir::Prop;
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    // The registrations: bounded, one-time (the roots intern into the
    // leak pool — the same storage story the compiled programs have).
    let customs = Customs::new()
        .utility(CustomUtility::functional(
            "brand-p",
            CustomForm::Spacing,
            &[
                Prop::PaddingTop,
                Prop::PaddingRight,
                Prop::PaddingBottom,
                Prop::PaddingLeft,
            ],
        ))
        .utility(CustomUtility::fixed(
            "brand-r",
            aeolus_ir::Val::Rem(0.375),
            &[
                Prop::BorderTopLeftRadius,
                Prop::BorderTopRightRadius,
                Prop::BorderBottomRightRadius,
                Prop::BorderBottomLeftRadius,
            ],
        ))
        .utility(CustomUtility::functional(
            "brand-ink",
            // The theme-token functional form: the suffix IS the token
            // name, resolved through the tiered table (`var()`, the
            // premultiplied `/opacity` bake, the dark pairs).
            CustomForm::Token,
            &[Prop::Color],
        ))
        .variant(CustomVariant::new("hocus", aeolus_ir::Cond::Hover));

    // 1. The IR walk: the custom utilities lower exactly like registry
    //    rows, and the alias folds to its typed condition. The built-in
    //    `p-4` rides alongside — the registry stays the oracle, the
    //    registrations are additive surface.
    let input = "brand-p-4 brand-r hocus:brand-p-2 p-4";
    println!("class string : {input}\n");
    let compiled = compile_theme_with(input, &tokens(), &customs, &Default::default())
        .expect("non-empty input compiles");
    println!(
        "── program ({} layers) ─────────────────────────────",
        compiled.program.layers.len()
    );
    for layer in compiled.program.layers {
        let cond = layer.cond.source_text().unwrap_or("base");
        println!(
            "  [{cond}] specificity {} ({} longhands)",
            layer.cond.specificity(),
            layer.decls.len()
        );
        for decl in layer.decls {
            println!("    {} = {:?}", decl.prop.name(), decl.val);
        }
    }
    print_diagnostics(&compiled.diagnostics);

    // 2. The CSS walk: the same registrations render real stylesheet
    //    rules — the alias renders its probe suffix, the token form
    //    renders the var() reference and the /opacity color-mix, the
    //    escaped selectors are the v4 forms.
    let sheet = emit_css_theme_with(
        "brand-p-4 hocus:brand-r brand-ink-primary brand-ink-primary/90",
        &tokens(),
        &customs,
        &Default::default(),
    )
    .expect("non-empty input renders");
    println!("\n── rendered sheet ─────────────────────────────────────");
    print!("{sheet}");

    // 3. The typed boundaries survive the seam: a suffix the registered
    //    form refuses reports with the custom root's name (the SAME
    //    LookupError shape the registry reports), and an unregistered
    //    alias stays the unsupported-variant boundary.
    let miss = compile_theme_with("brand-p-wide", &tokens(), &customs, &Default::default())
        .expect("non-empty input compiles");
    println!("── typed misses ───────────────────────────────────────");
    for d in &miss.diagnostics {
        println!("  {d}");
        println!("    code: {}", d.kind.code());
        if let DiagnosticKind::ShapeRejected { root, tried, .. } = &d.kind {
            println!(
                "    the CUSTOM root {root:?} refused; forms tried: {}",
                tried.join(", ")
            );
        }
    }
}

/// The runtime-tier table over the shadcn light+dark source (the theme
/// half rides the seam; `bg-primary` lowers through it unchanged).
fn tokens() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

/// The honest-boundary view (the editor-squiggle contract).
fn print_diagnostics(diagnostics: &[aeolus_compile::Diagnostic]) {
    if diagnostics.is_empty() {
        println!("  diagnostics: none — every class lowered");
    } else {
        println!("\n  diagnostics:");
        for d in diagnostics {
            println!("    {d}");
        }
    }
}
