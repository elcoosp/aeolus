//! Consumer demonstration: the STRUCTURAL-variant probe fold (plan
//! §3.5's probe-row successor). The element's own DOM-position variants
//! — `first:`, `last:`, `odd:`, `even:`, `only:`, `empty:`, `nth-N:`,
//! `nth-last-N:` — fold onto the typed [`Cond`] surface, and the
//! [`StructProbe`] seam carries the element's DOM-position facts into
//! the walk (the built-in [`StructFacts`] is one Copy value; a host
//! wires its own layout bridge through the same one question).
//!
//! The demo walks the same program four ways: sourceless (the
//! structural layers REPORT — the honest boundary, the
//! `StructuralUnqueried` note), facts-answered at the first child (the
//! first and odd layers fire — index 0 IS `:nth-child(odd)`), facts-
//! answered at the last child (a QUIET miss for first/odd — a
//! positional selector that does not match, no note either), and the
//! unknown-facts value (`StructFacts::UNKNOWN` — every index-based test
//! quietly skips). The base layer keeps answering the whole time. The
//! themed structural lane (`interp_theme_struct`) resolves token
//! colours through the same seam family — see `tests/structural.rs`.
//!
//! ```sh
//! cargo run -p aeolus-compile --example struct_probe
//! ```

use aeolus_compile::{Config, Env, StructFacts, compile, interp, interp_struct};
use aeolus_ir::{Cond, Prop, Structural, Val};

fn main() {
    let config = Config::default();

    println!("── the program: three structural layers + one base ──");
    let compiled = compile("first:w-64 last:hidden odd:hidden p-4", &config).expect("compiles");
    for layer in compiled.program.layers {
        println!("  layer: {:?}", layer.cond);
    }

    println!("\n── 1. sourceless: the structural layers report, the base answers ──");
    let sourceless = interp(&compiled.program, &Env::default()).expect("the walk resolves");
    for note in &sourceless.notes {
        println!("  note: {note}");
    }
    assert_eq!(
        sourceless.notes.len(),
        3,
        "all three structural layers report"
    );
    assert_eq!(
        sourceless
            .decls
            .iter()
            .find(|d| d.prop == Prop::PaddingTop)
            .map(|d| d.val),
        Some(Val::Rem(1.0)),
        "the base layer still answers"
    );

    println!("\n── 2. facts answered: the element is the FIRST child of three ──");
    let facts = StructFacts::new(0, 3);
    let hot = interp_struct(&compiled.program, &Env::default(), &facts).expect("the walk resolves");
    for note in &hot.notes {
        println!("  note: {note}");
    }
    assert!(
        hot.notes.is_empty(),
        "a source-answered walk reports nothing"
    );
    assert_eq!(
        hot.decls
            .iter()
            .find(|d| d.prop == Prop::Width)
            .map(|d| d.val),
        Some(Val::Rem(16.0)),
        "the first layer fires at index 0"
    );
    assert!(
        hot.decls.iter().any(|d| d.prop == Prop::Display),
        "the odd layer fires too — index 0 IS :nth-child(odd)"
    );

    println!("\n── 3. facts answered at the LAST child: first/odd quietly skip ──");
    let facts = StructFacts::new(2, 3);
    let tail =
        interp_struct(&compiled.program, &Env::default(), &facts).expect("the walk resolves");
    assert!(
        tail.notes.is_empty(),
        "a source-answered miss is CSS-quiet, no note"
    );
    assert!(
        tail.decls.iter().all(|d| d.prop != Prop::Width),
        "the first layer quietly skips (a selector that does not match)"
    );
    assert!(
        tail.decls.iter().any(|d| d.prop == Prop::Display),
        "the last layer fires at the final child"
    );

    println!("\n── 4. the CSS arm renders the same positions as selectors ──");
    let sheet = aeolus_compile::emit_css("first:w-64 last:hidden nth-3:w-10", &config)
        .expect("the emit resolves");
    for rule in &sheet.rules {
        println!("  rule: {}", rule.selector);
    }
    assert!(
        sheet.to_string().contains(".first\\:w-64:first-child"),
        "the positional selector renders"
    );

    println!("\n── 5. the typed fold, one condition per position test ──");
    let conditions = [
        ("first:w-64", Cond::Structural(Structural::First)),
        ("nth-last-2:w-64", Cond::Structural(Structural::NthLast(2))),
    ];
    for (class, cond) in conditions {
        let compiled = compile(class, &config).expect("compiles");
        assert!(
            compiled
                .program
                .layers
                .iter()
                .any(|layer| layer.cond == cond),
            "{class} folds to {cond:?}"
        );
        println!("  {class} → {cond:?}");
    }

    println!("\n── 6. the unknown-facts value: every positional test quietly skips ──");
    let unknown = interp_struct(&compiled.program, &Env::default(), &StructFacts::UNKNOWN)
        .expect("the walk resolves");
    assert!(
        unknown.notes.is_empty(),
        "UNKNOWN facts are a quiet miss, not a note"
    );
    assert!(
        unknown.decls.iter().all(|d| matches!(
            d.prop,
            Prop::PaddingTop | Prop::PaddingRight | Prop::PaddingBottom | Prop::PaddingLeft
        )),
        "only the base layer fired (the four padding longhands)"
    );
    println!("  notes: none (the built-in answered \"unknown\" — CSS-quiet)");

    println!("\nall structural seam demos held — the honest boundaries are the tests' business");
}
