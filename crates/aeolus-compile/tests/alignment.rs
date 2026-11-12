//! The keyword-alignment seam (`keyword_alignment`, the P3 read side's
//! second half): one row text per flattened declaration, in layer-major
//! order — the annotation table the `tw!` macro freezes beside the
//! layers and the dynamic lane (`aeolus_rt::ClassCell`) walks at
//! runtime.
//!
//! The pins:
//! * the alignment is TOTAL — one entry per flattened declaration,
//!   layer-major, the `StaticProgram` keyword slice's exact shape;
//! * duplicate `(cond, prop)` pairs consume in source order (the
//!   compile assembly's stable order — `hidden flex block` keeps the
//!   three `Display` texts in the class order);
//! * numeric/colour/arbitrary values align to `""` (no row text);
//! * the variant buckets align per layer (the `(cond, prop)` queue is
//!   per-condition, so an unfired layer's text can never shadow the
//!   winner's);
//! * a class that refuses is absent from the annotations and aligns to
//!   `""` everywhere it never contributed.

use aeolus_compile::{Config, compile, keyword_alignment, keyword_texts};
use aeolus_ir::Prop;

/// The layer-major total: the alignment output's length is the program's
/// flattened declaration count, and every keyword slot of the demo
/// string carries its row text.
#[test]
fn alignment_is_total_and_layer_major() {
    let classes = "hidden flex-col underline";
    let config = Config::default();
    let compiled = compile(classes, &config).expect("non-empty input compiles");
    let annotations = keyword_texts(classes, &config).expect("non-empty input annotates");
    let alignment = keyword_alignment(&compiled, &annotations);
    let total: usize = compiled
        .program
        .layers
        .iter()
        .map(|layer| layer.decls.len())
        .sum();
    assert_eq!(alignment.len(), total);
    // The three demo classes are all keyword rows: every slot carries a
    // non-empty text somewhere in the slice.
    assert!(alignment.contains(&"none"));
    assert!(alignment.contains(&"column"));
    assert!(alignment.contains(&"underline"));
}

/// Duplicate `(cond, prop)` pairs consume in source order: three display
/// classes in one base layer produce three `Display` declarations whose
/// aligned texts are the class order, not a scramble.
#[test]
fn duplicate_pairs_consume_in_source_order() {
    let classes = "hidden flex block";
    let config = Config::default();
    let compiled = compile(classes, &config).expect("non-empty input compiles");
    let annotations = keyword_texts(classes, &config).expect("non-empty input annotates");
    let alignment = keyword_alignment(&compiled, &annotations);
    let mut display_texts = Vec::new();
    let mut index = 0usize;
    for (layer_index, layer) in compiled.program.layers.iter().enumerate() {
        for decl in layer.decls {
            if layer_index == 0 && decl.prop == Prop::Display {
                display_texts.push(alignment[index]);
            }
            index += 1;
        }
    }
    assert_eq!(display_texts, ["none", "flex", "block"]);
}

/// Numeric, colour and arbitrary values carry their own value — no row
/// text — so their alignment entries are empty.
#[test]
fn non_keyword_values_align_empty() {
    let classes = "p-4 w-[32px]";
    let config = Config::default();
    let compiled = compile(classes, &config).expect("non-empty input compiles");
    let annotations = keyword_texts(classes, &config).expect("non-empty input annotates");
    let alignment = keyword_alignment(&compiled, &annotations);
    assert!(
        alignment.iter().all(|text| text.is_empty()),
        "spacing and arbitrary lengths have no row text: {alignment:?}"
    );
}

/// The `(cond, prop)` queue is per-condition: the base and hover layers
/// align independently, so a hover layer's padding text (still empty —
/// numeric) can never consume the base's.
#[test]
fn variant_buckets_align_per_layer() {
    let classes = "p-4 hover:p-8";
    let config = Config::default();
    let compiled = compile(classes, &config).expect("non-empty input compiles");
    let annotations = keyword_texts(classes, &config).expect("non-empty input annotates");
    let alignment = keyword_alignment(&compiled, &annotations);
    let total: usize = compiled
        .program
        .layers
        .iter()
        .map(|layer| layer.decls.len())
        .sum();
    assert_eq!(alignment.len(), total);
    assert_eq!(compiled.program.layers.len(), 2, "base + hover");
}

/// A class the pipeline refuses is absent from the annotations — the
/// alignment stays total over the program that DID lower, with no panic
/// and no phantom texts.
#[test]
fn refused_class_aligns_nothing() {
    let classes = "p-4 not-a-real-utility-xyz";
    let config = Config::default();
    let compiled = compile(classes, &config).expect("non-empty input compiles");
    assert!(
        !compiled.diagnostics.is_empty(),
        "the unknown class is a typed diagnostic, not a silent drop"
    );
    let annotations = keyword_texts(classes, &config).expect("non-empty input annotates");
    assert_eq!(annotations.len(), 1, "only the lowered class annotates");
    let alignment = keyword_alignment(&compiled, &annotations);
    let total: usize = compiled
        .program
        .layers
        .iter()
        .map(|layer| layer.decls.len())
        .sum();
    assert_eq!(alignment.len(), total);
}
