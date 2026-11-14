//! The GPUI grid-placement + flex-basis tier's GATE half (architecture
//! plan §3.7's grid row — "native subset; rest diagnosed" — finished for
//! every longhand the pinned GPUI surface carries a slot for):
//! `supports`/`expressible` take the five placement longhands, the emit
//! gate and the third-party `lower_class` seam accept their classes, and
//! the published conformance report reads the same tables.
//!
//! The pins:
//!
//! * `supports()` — `flex-basis`, `grid-row-start/end`,
//!   `grid-column-start/end` are NATIVE (the pinned `gpui-pre` surface
//!   carries `flex_basis`, `col_start`/`col_end`, `row_start`/`row_end`);
//! * `expressible()` — the emit gate's verdict accepts them; the
//!   polyfill tier stays untouched (the ring family remains the whole
//!   tier);
//! * `lower_class` — the third-party seam lowers the classes (RED first:
//!   the gate refused them before the flip);
//! * the audit — the macro gate's shape over a compiled placement
//!   program: no `Unsupported` entries;
//! * the conformance report — the partition over `Prop::ALL` reads the
//!   same tables (the report is the emitter's verdicts, never a
//!   parallel claim), and the refused set shrinks by exactly the five;
//! * the frozen emit — the `Emit` impl renders the placement program's
//!   declarations (the data the walk applies).

use aeolus_codegen::conformance::Conformance;
use aeolus_codegen_gpui::{
    Caps, audit, emit_static_program, expressible, lower_class, polyfilled, supports,
};
use aeolus_compile::{Config, compile, keyword_alignment, keyword_texts};

fn the_five() -> [aeolus_ir::Prop; 5] {
    [
        aeolus_ir::Prop::FlexBasis,
        aeolus_ir::Prop::GridRowStart,
        aeolus_ir::Prop::GridRowEnd,
        aeolus_ir::Prop::GridColumnStart,
        aeolus_ir::Prop::GridColumnEnd,
    ]
}

#[test]
fn the_five_placement_longhands_are_native() {
    for prop in the_five() {
        assert!(
            supports(prop),
            "{prop:?} is a native slot at the pinned rev"
        );
        assert!(expressible(prop), "{prop:?} is expressible");
        assert!(!polyfilled(prop), "{prop:?} is native, not a polyfill");
    }
}

#[test]
fn the_emit_gate_accepts_the_placement_classes() {
    let classes = [
        "basis-16",
        "basis-1/2",
        "basis-[30%]",
        "col-span-3",
        "col-start-2",
        "col-end-5",
        "row-span-2",
        "row-start-3",
        "row-end-4",
        "-col-start-2",
    ];
    for class in classes {
        let lowered = lower_class(class, &Config::default());
        assert!(
            lowered.is_ok(),
            "{class} lowers through the third-party seam: {:?}",
            lowered.err()
        );
    }
}

#[test]
fn the_audit_verdict_is_empty_for_a_placement_program() {
    // The macro gate's shape: a program whose every declaration the
    // target takes audits EMPTY — no `Unsupported` entries.
    let compiled =
        compile("col-span-3 basis-16 row-start-2", &Config::default()).expect("compiles");
    let classes =
        keyword_texts("col-span-3 basis-16 row-start-2", &Config::default()).expect("row texts");
    let keywords = keyword_alignment(&compiled, &classes);
    let unsupported = audit(&compiled.program, &keywords);
    assert!(
        unsupported.is_empty(),
        "a placement program audits clean: {unsupported:?}"
    );
}

#[test]
fn the_conformance_report_reads_the_same_tables() {
    let report = Conformance::of(&Caps);
    for prop in the_five() {
        assert!(
            report.native().contains(&prop),
            "{prop:?} sits in the report's native tier"
        );
        assert!(
            !report.refused().contains(&prop),
            "{prop:?} left the refused tier"
        );
    }
    // The ring family is still the whole polyfill tier (the report's
    // own documented pin).
    assert_eq!(report.polyfill().len(), 4, "the ring family's four slots");
}

#[test]
fn the_gpui_emit_renders_the_frozen_placement_decls() {
    // The emit's own path: the frozen program embeds the Num
    // declarations the walk applies (the emit is data, the walk is the
    // application — this pin is the emit's half).
    let compiled = compile("col-span-3", &Config::default()).expect("compiles");
    let classes = keyword_texts("col-span-3", &Config::default()).expect("row texts");
    let keywords = keyword_alignment(&compiled, &classes);
    let text = emit_static_program(&compiled.program, &keywords).to_string();
    assert!(
        text.contains("Decl"),
        "the emit freezes declarations: {text}"
    );
}
