//! The generated `.slint` code is REAL slint code.
//!
//! These tests do two things the string assertions alone cannot:
//!
//! 1. Pin the exact property bindings the generator emits (hex literals
//!    with alpha LAST, per-corner radii, `states [ … ]` gates).
//! 2. **Compile the assembled component with slint's own compiler** (the
//!    interpreter, headless — compilation never instantiates anything), so
//!    every property name, keyword and gradient grammar the generator
//!    emits is proven valid against Slint 1.18 itself.

use aeolus_core::{Resolver, cn, shadcn_default};
use aeolus_slint::{StyleBook, brush_snippet, color_snippet};
use std::fmt::Write as _;

/// Indent a generated block by `n` spaces (the expected-component builder).
fn indent_block(snippet: &str, n: usize) -> String {
    let pad = " ".repeat(n);
    let mut out = String::new();
    for l in snippet.lines() {
        let _ = writeln!(out, "{pad}{l}");
    }
    out
}

fn resolver() -> Resolver<'static> {
    let theme = shadcn_default().unwrap();
    Resolver::new(Box::leak(Box::new(theme)))
}

/// Compile a `.slint` source with slint's own compiler and panic with the
/// diagnostics on any error (the honest failure — the tests must fail
/// loudly when the generator emits something Slint rejects).
fn assert_compiles(source: &str) {
    let compiler = slint_interpreter::Compiler::new();
    let result = futures::executor::block_on(
        compiler.build_from_source(source.to_string(), "aeolus-test.slint".into()),
    );
    let errors: Vec<String> = result
        .diagnostics()
        .filter(|d| d.level() == slint_interpreter::DiagnosticLevel::Error)
        .map(|d| {
            format!(
                "{:?} @ {}",
                d.message(),
                d.source_file()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default()
            )
        })
        .collect();
    assert!(
        errors.is_empty(),
        "generated .slint did not compile:\n{source}\n--- errors ---\n{}",
        errors.join("\n")
    );
}

#[test]
fn color_literals_use_slint_hex_order() {
    // Opaque → #RRGGBB; alpha carried LAST (slint's hex grammar), unlike
    // the Rust API's 0xAARRGGBB.
    let c = aeolus_core::Color::from_srgb(0.1, 0.2, 0.3, 1.0);
    // (the oklab→sRGB round trip is stable here; the last digit is what
    // the transfer function actually produces)
    assert_eq!(color_snippet(c), "#1A334C");
    let translucent = c.with_alpha(0.5);
    assert!(
        color_snippet(translucent).ends_with("80"),
        "alpha last: {}",
        color_snippet(translucent)
    );
}

#[test]
fn gradient_brushes_render_as_at_gradient_calls() {
    let r = resolver();
    let book = StyleBook::tw(&r, "bg-linear-to-r from-red-500 to-blue-500");
    let snippet = brush_snippet(book.base.background_gradient.as_ref().unwrap());
    assert!(
        snippet.starts_with("@linear-gradient(90deg,"),
        "got {snippet}"
    );
    assert!(snippet.contains('%'), "stops carry percent positions");

    let book = StyleBook::tw(&r, "bg-radial from-white to-black");
    let snippet = brush_snippet(book.base.background_gradient.as_ref().unwrap());
    assert!(
        snippet.starts_with("@radial-gradient(circle,"),
        "got {snippet}"
    );
}

#[test]
fn base_snippet_emits_the_documented_property_names() {
    let r = resolver();
    let book = StyleBook::tw(
        &r,
        "bg-primary rounded-md border border-input shadow-xs opacity-50 overflow-hidden w-full h-10 px-4 cursor-pointer",
    );
    let s = book.base_snippet();
    for prop in [
        "background:",
        "border-radius: 8px;",
        "border-width: 1px;",
        "border-color:",
        "drop-shadow-blur:",
        "drop-shadow-spread:",
        "drop-shadow-color:",
        "opacity: 0.5;",
        "clip: true;",
        "width: 100%;",
        "height: 40px;",
        // px-4 alone → per-edge padding (top/bottom stay 0).
        "padding-left: 16px;",
        "padding-right: 16px;",
        "mouse-cursor: pointer;",
    ] {
        assert!(s.contains(prop), "missing {prop:?} in:\n{s}");
    }
}

#[test]
fn per_corner_radii_emit_the_four_corner_properties() {
    let r = resolver();
    let book = StyleBook::tw(&r, "rounded-tl-lg rounded-br-sm");
    let s = book.base_snippet();
    assert!(s.contains("border-top-left-radius: 10px;"), "{s}");
    assert!(s.contains("border-bottom-right-radius: 6px;"), "{s}");
    assert!(
        !s.contains("border-radius:"),
        "no uniform shorthand when corners differ"
    );
}

#[test]
fn states_snippet_gates_on_touch_area_properties() {
    let r = resolver();
    let book = StyleBook::tw(
        &r,
        "bg-primary hover:bg-primary/90 active:bg-primary/80 disabled:opacity-50 focus-visible:opacity-60",
    );
    let states = book.to_states_snippet("ta");
    assert!(states.starts_with("states ["), "{states}");
    assert!(states.contains("hovered when ta.has-hover"), "{states}");
    assert!(states.contains("pressed when ta.pressed"), "{states}");
    assert!(states.contains("disabled when !ta.enabled"), "{states}");
    // has-focus is a FocusScope property, NOT a TouchArea one — the
    // TouchArea-only block must not emit it (the compiler enforces this).
    assert!(!states.contains("has-focus"), "{states}");
    assert!(
        states.contains("background: "),
        "state bodies carry property bindings"
    );

    // With a FocusScope named, the focused state appears.
    let focused = book.to_states_snippet_focused("ta", "fs");
    assert!(focused.contains("focused when fs.has-focus"), "{focused}");
    assert_compiles(&format!(
        "export component AeolusFocusTest {{\n    Rectangle {{\n        ta := TouchArea {{ }}\n        fs := FocusScope {{ }}\n{states}\n    }}\n}}",
        states = indent_block(&focused, 8),
    ));
}

/// The full assembly: shadcn Button table → book → component source →
/// slint's own compiler. The exact code the generator emits must compile.
#[test]
fn generated_button_component_compiles_with_slint() {
    let r = resolver();
    let classes = cn(&[&aeolus_shadcn_classes_button_default(), "shadow-lg"]);
    let book = StyleBook::tw(&r, &classes);
    let text_block = book.base.text.snippet();
    let source = format!(
        "export component AeolusButtonTest {{\n    width: 160px;\n    height: 40px;\n    Rectangle {{\n{base}        ta := TouchArea {{ }}\n        fs := FocusScope {{ }}\n        Text {{\n            text: \"Save changes\";\n{text}        }}\n{states}\n    }}\n}}",
        base = indent_block(&book.base_snippet(), 8),
        text = indent_block(&text_block, 12),
        states = indent_block(&book.to_states_snippet_focused("ta", "fs"), 8),
    );
    assert_compiles(&source);
}

#[test]
fn generated_gradient_and_geometry_component_compiles_with_slint() {
    let r = resolver();
    let book = StyleBook::tw(
        &r,
        "bg-linear-to-r from-red-500 to-blue-500 rounded-tl-lg rounded-br-sm border border-input min-w-20 max-h-96 opacity-80 overflow-hidden",
    );
    let source = format!(
        "export component AeolusGradientTest {{\n    width: 200px;\n    height: 60px;\n    Rectangle {{\n        {body}\n    }}\n}}",
        body = indent_block(&book.base_snippet(), 8),
    );
    assert_compiles(&source);
}

#[test]
fn typography_snippet_compiles_on_a_text_element() {
    let r = resolver();
    let book = StyleBook::tw(
        &r,
        "text-sm font-medium text-primary-foreground whitespace-nowrap truncate text-center",
    );
    let text_block = book.base.text.snippet();
    assert!(text_block.contains("font-size: 14px;"), "{text_block}");
    assert!(text_block.contains("font-weight: 500;"), "{text_block}");
    assert!(text_block.contains("wrap: no-wrap;"), "{text_block}");
    assert!(text_block.contains("overflow: elide;"), "{text_block}");
    assert!(
        text_block.contains("horizontal-alignment: center;"),
        "{text_block}"
    );
    let source = format!(
        "export component AeolusTextTest {{\n    Rectangle {{\n        Text {{\n            {body}\n        }}\n    }}\n}}",
        body = indent_block(&text_block, 12),
    );
    assert_compiles(&source);
}

#[test]
fn inner_shadow_and_visibility_compiles_with_slint() {
    let r = resolver();
    let book = StyleBook::tw(&r, "inset-shadow-2xs invisible");
    let body = book.base_snippet();
    assert!(body.contains("inner-shadow-blur:"), "{body}");
    assert!(body.contains("visible: false;"), "{body}");
    let source = format!(
        "export component AeolusInnerTest {{\n    Rectangle {{\n        {body}\n    }}\n}}",
        body = indent_block(&body, 8),
    );
    assert_compiles(&source);
}

/// The class table for the default Button variant + size, mirroring
/// `aeolus_shadcn::classes::button_classes` (kept inline so this test
/// compiles against aeolus-slint alone).
fn aeolus_shadcn_classes_button_default() -> String {
    const BUTTON_BASE: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4";
    const DEFAULT_VARIANT: &str =
        "bg-primary text-primary-foreground shadow-xs hover:bg-primary/90";
    const DEFAULT_SIZE: &str = "h-9 px-4 py-2 has-[>svg]:px-3";
    cn(&[BUTTON_BASE, DEFAULT_VARIANT, DEFAULT_SIZE])
}
