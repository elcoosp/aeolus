//! The Rust-tokens emit arm's contracts (architecture plan §3.1 emit
//! (a) — the LAST arm of the `Target` surface): class strings in, Rust
//! token fragments out — one `aeolus_ir::Layer::new` expression per
//! lowered class, the keyword slots resolved to their ROW text (the
//! names only the pipeline can see), the theme token slots annotated
//! with their table names, and the SAME honest diagnostics the IR walk
//! reports.
//!
//! The pins:
//!
//! * the sheet's `Display` is a compilable `static PROGRAM:
//!   &[aeolus_ir::Layer]` item — one layer per fragment,
//!   specificity-sorted (stable, source order breaking ties), the doc
//!   test in the module compiles AND runs it;
//! * fragments are per-candidate in source order (the `tw!`-style
//!   consumer's unit — one class, one fragment);
//! * the text is DETERMINISTIC (same input, same bytes — the artifact
//!   contract);
//! * keyword slots carry `// <row text>` annotations — the capability
//!   the program-only emitters (`emit_static`/`interp`) structurally
//!   lack, because the `Val::Kw` slot indexes the ROW'S OWN table and a
//!   bare `Program` cannot recover the text (the in-pipeline emit
//!   renders where the rows still talk, the CSS arm's stated rationale);
//! * theme token slots carry `// <token name>` annotations when a
//!   `TokenTable` rides (the id→name lookup only the table answers);
//! * diagnostics stay EQUAL to [`compile`]'s over a battery (the
//!   shared-walk contract — the same lexer channel, read side and
//!   boundary set);
//! * the custom-authoring seam renders through the same walk, and empty
//!   registrations are byte-identical to the plain entry points.
//!
//! Written FIRST (the RED spec): the suite fails against the 0.15.0
//! seam (the rust-tokens arm did not exist) and passes only once
//! `emit_rust` satisfies every pin below.

use aeolus_compile::{
    CompileError, Config, CustomForm, CustomUtility, Customs, Spacing, Target, compile, emit_rust,
    emit_rust_theme, emit_rust_with,
};
use aeolus_ir::{Cond, Prop};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The runtime-tier table over the shadcn light+dark source.
fn runtime_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

// ---------------------------------------------------------------------------
// 1. The artifact: the sheet, the fragments, the static item
// ---------------------------------------------------------------------------

#[test]
fn a_base_class_emits_the_rust_form() {
    // `p-4` under the default (tailwind) spacing bakes four rem
    // declarations, in the IR's Prop cascade order — the Display is the
    // compilable static item, the fragment the embeddable expression.
    let sheet = emit_rust("p-4", &Config::default()).expect("the string compiles");
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    assert_eq!(sheet.fragments.len(), 1);
    assert_eq!(sheet.fragments[0].class, "p-4");
    assert_eq!(sheet.fragments[0].cond, Cond::Base);
    assert_eq!(
        sheet.fragments[0].tokens,
        "aeolus_ir::Layer::new(\n\
         \x20   aeolus_ir::Cond::Base,\n\
         \x20   &[\n\
         \x20       aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingTop, val: aeolus_ir::Val::Rem(1.0) },\n\
         \x20       aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingRight, val: aeolus_ir::Val::Rem(1.0) },\n\
         \x20       aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingBottom, val: aeolus_ir::Val::Rem(1.0) },\n\
         \x20       aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingLeft, val: aeolus_ir::Val::Rem(1.0) },\n\
         \x20   ],\n\
         )",
        "the fragment's token expression, verbatim"
    );
    assert_eq!(
        sheet.to_string(),
        "static PROGRAM: &[aeolus_ir::Layer] = &[\n\
         \x20   // p-4\n\
         \x20   aeolus_ir::Layer::new(\n\
         \x20       aeolus_ir::Cond::Base,\n\
         \x20       &[\n\
         \x20           aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingTop, val: aeolus_ir::Val::Rem(1.0) },\n\
         \x20           aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingRight, val: aeolus_ir::Val::Rem(1.0) },\n\
         \x20           aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingBottom, val: aeolus_ir::Val::Rem(1.0) },\n\
         \x20           aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingLeft, val: aeolus_ir::Val::Rem(1.0) },\n\
         \x20       ],\n\
         \x20   ),\n\
         ];\n",
        "the sheet renders as the static item, verbatim"
    );
}

#[test]
fn fragments_are_per_candidate_in_source_order() {
    let sheet = emit_rust("p-4 flex-col", &Config::default()).expect("the string compiles");
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    assert_eq!(sheet.fragments.len(), 2, "one fragment per lowered class");
    assert_eq!(sheet.fragments[0].class, "p-4");
    assert_eq!(sheet.fragments[1].class, "flex-col");
    assert_eq!(sheet.fragments[0].cond, Cond::Base);
    assert_eq!(sheet.fragments[1].cond, Cond::Base);
}

#[test]
fn the_emit_is_deterministic() {
    let input = "p-4 flex-col hover:bg-primary/50 md:hidden";
    let table = runtime_table();
    let a = emit_rust_theme(input, &table, &Config::default()).expect("compiles");
    let b = emit_rust_theme(input, &table, &Config::default()).expect("compiles");
    assert_eq!(
        a.to_string(),
        b.to_string(),
        "same input, same bytes — the artifact contract"
    );
    assert_eq!(a.fragments, b.fragments);
}

#[test]
fn the_display_sorts_layers_by_specificity() {
    // Fragments stay in SOURCE order (the per-class consumption unit);
    // the static item renders them specificity-sorted (stable — the
    // source order breaks ties), so the Display is a sorted program.
    let sheet = emit_rust("hover:p-4 p-4", &Config::default()).expect("compiles");
    assert_eq!(sheet.fragments[0].class, "hover:p-4", "source order");
    assert_eq!(sheet.fragments[0].cond, Cond::Hover);
    assert_eq!(sheet.fragments[1].class, "p-4");
    let text = sheet.to_string();
    let base = text.find("// p-4\n").expect("the base comment renders");
    let hover = text
        .find("// hover:p-4\n")
        .expect("the hover comment renders");
    assert!(
        base < hover,
        "the Display renders Base before Hover (specificity order): {text}"
    );
    assert!(
        text.starts_with("static PROGRAM: &[aeolus_ir::Layer] = &[\n") && text.ends_with("];\n"),
        "the Display is the static item shape: {text}"
    );
}

#[test]
fn an_all_refused_string_renders_the_empty_sheet() {
    let sheet = emit_rust("not-a-utility", &Config::default()).expect("compiles");
    assert_eq!(sheet.diagnostics.len(), 1, "the report rides");
    assert!(sheet.fragments.is_empty(), "nothing lowered, nothing emits");
    assert_eq!(
        sheet.to_string(),
        "static PROGRAM: &[aeolus_ir::Layer] = &[\n];\n",
        "the empty program is still a compilable item"
    );
}

#[test]
fn empty_input_is_the_compile_error() {
    assert_eq!(
        emit_rust("", &Config::default()),
        Err(CompileError::Empty),
        "the only whole-string failure, shared with compile"
    );
    assert_eq!(
        emit_rust("   ", &Config::default()),
        Err(CompileError::Empty),
        "whitespace-only is the same failure"
    );
}

// ---------------------------------------------------------------------------
// 2. The keyword resolution: the names only the pipeline can see
// ---------------------------------------------------------------------------

#[test]
fn keyword_slots_render_their_row_text() {
    // `flex-col` lowers to Val::Kw(slot) — slot indexes the row's OWN
    // FLEX_DIRECTIONS table ("col" is entry 0) — and the annotation
    // carries the row's CSS text (`column`), which a bare Program
    // CANNOT recover (the static-data arm emits the slot verbatim).
    let sheet = emit_rust("flex-col", &Config::default()).expect("compiles");
    let tokens = &sheet.fragments[0].tokens;
    assert!(
        tokens.contains("aeolus_ir::Prop::FlexDirection, val: aeolus_ir::Val::Kw(0) }, // column"),
        "the keyword slot renders with its row text: {tokens}"
    );
}

#[test]
fn bare_roots_render_their_row_text() {
    // `hidden` is a bare root: the whole utility IS the value
    // (`display: none`), slot 0, the row's own text as the annotation.
    let sheet = emit_rust("hidden", &Config::default()).expect("compiles");
    let tokens = &sheet.fragments[0].tokens;
    assert!(
        tokens.contains("aeolus_ir::Prop::Display, val: aeolus_ir::Val::Kw(0) }, // none"),
        "the bare value renders with its row text: {tokens}"
    );
}

#[test]
fn composite_rows_render_per_target_texts() {
    // `text-sm` is the composite pair (font-size, line-height): the two
    // targets carry DIFFERENT row texts, each annotated on its own decl.
    let sheet = emit_rust("text-sm", &Config::default()).expect("compiles");
    let tokens = &sheet.fragments[0].tokens;
    assert!(
        tokens.contains("aeolus_ir::Prop::FontSize, val: aeolus_ir::Val::Kw(1) }, // 0.875rem"),
        "the primary text annotates the font-size decl: {tokens}"
    );
    assert!(
        tokens.contains(
            "aeolus_ir::Prop::LineHeight, val: aeolus_ir::Val::Kw(1) }, // calc(1.25 / 0.875)"
        ),
        "the secondary text annotates the line-height decl: {tokens}"
    );
}

#[test]
fn numeric_values_render_without_annotations() {
    // Spacing/fraction/number values are concrete — no row-text
    // annotation rides (the value IS the data, the static arm's form).
    let sheet = emit_rust("w-1/2 z-10", &Config::default()).expect("compiles");
    let text = sheet.to_string();
    assert!(text.contains("aeolus_ir::Val::Pct(50.0) },\n"), "{text}");
    assert!(text.contains("aeolus_ir::Val::Num(10.0) },\n"), "{text}");
    assert!(
        !text.contains("}, //"),
        "no annotation on concrete values: {text}"
    );
}

#[test]
fn the_px_config_bakes_pixels() {
    let config = Config {
        spacing: Spacing::Px,
        rem_px: 16.0,
    };
    let sheet = emit_rust("p-4", &config).expect("compiles");
    assert!(
        sheet.fragments[0]
            .tokens
            .contains("aeolus_ir::Val::Px(16.0)"),
        "the px bake rides the config: {}",
        sheet.fragments[0].tokens
    );
}

#[test]
fn negation_and_the_px_step_render() {
    let sheet = emit_rust("-mt-2 pb-px", &Config::default()).expect("compiles");
    let text = sheet.to_string();
    assert!(text.contains("aeolus_ir::Val::Rem(-0.5)"), "{text}");
    assert!(text.contains("aeolus_ir::Val::Px(1.0)"), "{text}");
}

// ---------------------------------------------------------------------------
// 3. The theme annotations: the table names only the source answers
// ---------------------------------------------------------------------------

#[test]
fn theme_tokens_render_their_table_names() {
    // The runtime tier lowers colour tokens to `Val::Tok { id, alpha }`
    // — the annotation carries the token NAME the table answers
    // (`TokenTable::name(id)` — the table's canonical, unprefixed key,
    // the same string `TokenTable::id` accepts), which a bare Program
    // cannot recover.
    let table = runtime_table();
    let sheet = emit_rust_theme("bg-primary/50", &table, &Config::default()).expect("compiles");
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    let tokens = &sheet.fragments[0].tokens;
    let id = table
        .id("primary")
        .expect("the shadcn table carries the primary token");
    assert_eq!(table.name(id), Some("primary"));
    // The Tok literal text (Tok's own brace, then the Decl's):
    let slot = format!("aeolus_ir::Val::Tok {{ id: {id}, alpha: 0.5 }} }}, // primary");
    assert!(
        tokens.contains(&slot),
        "the token slot renders with its table name: {tokens}"
    );
}

#[test]
fn baked_pairs_ride_without_a_name_annotation() {
    // The baked tier's dark pair has no name table (the slot indexes the
    // const pair table) — the slot rides verbatim, honestly unannotated.
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::baked(&source).expect("default-seeded source builds");
    let sheet = emit_rust_theme("bg-primary", &table, &Config::default()).expect("compiles");
    let tokens = &sheet.fragments[0].tokens;
    assert!(
        tokens.contains("aeolus_ir::Val::ModePair("),
        "the pair slot rides as the IR carries it: {tokens}"
    );
    assert!(
        !tokens.contains("}, //"),
        "no name annotation exists for a pair slot: {tokens}"
    );
}

// ---------------------------------------------------------------------------
// 4. Totality: every Cond form has an emission arm
// ---------------------------------------------------------------------------

#[test]
fn every_cond_form_has_an_emission_arm() {
    // The REAL pipeline folds every Cond constructor — one string walks
    // the whole surface, the arm's cond coverage pinned end to end.
    let sheet = emit_rust(
        "p-9 \
         hover:p-0 focus:p-1 focus-visible:p-2 focus-within:p-3 \
         active:p-4 disabled:p-5 visited:p-6 checked:p-7 \
         placeholder-shown:p-8 selection:p-0 placeholder:p-1 \
         group-hover:p-2 group-focus:p-3 peer-hover:p-4 peer-focus:p-5 \
         dark:p-6 sm:p-7 min-md:p-8 max-lg:p-0 @xl:p-1 @max-2xl:p-2",
        &Config::default(),
    )
    .expect("the string compiles");
    let conds: Vec<Cond> = sheet.fragments.iter().map(|f| f.cond).collect();
    for cond in [
        Cond::Base,
        Cond::Dark,
        Cond::Hover,
        Cond::Focus,
        Cond::FocusVisible,
        Cond::FocusWithin,
        Cond::Active,
        Cond::Disabled,
        Cond::Visited,
        Cond::Checked,
        Cond::PlaceholderShown,
        Cond::Selection,
        Cond::Placeholder,
        Cond::GroupHover,
        Cond::GroupFocus,
        Cond::PeerHover,
        Cond::PeerFocus,
        Cond::Bp(aeolus_ir::Bp::Sm),
        Cond::Min(aeolus_ir::Bp::Md),
        Cond::Max(aeolus_ir::Bp::Lg),
        Cond::Container {
            bp: aeolus_ir::Bp::Xl,
            name: None,
        },
        Cond::MaxContainer {
            bp: aeolus_ir::Bp::Xl2,
            name: None,
        },
    ] {
        assert!(
            conds.contains(&cond),
            "the folded surface carries {cond:?}: {conds:?}"
        );
        let text = format!("aeolus_ir::Cond::{cond:?}");
        assert!(
            sheet.to_string().contains(&text),
            "the emitted form carries {text}"
        );
    }
}

// ---------------------------------------------------------------------------
// 5. The shared walk: diagnostics equal to compile's
// ---------------------------------------------------------------------------

#[test]
fn the_diagnostics_stay_equal_to_compile_over_the_battery() {
    // The load-bearing agreement: the rust-tokens walk shares the lexer
    // channel, the read side and the value pass's boundary set with the
    // IR walk — the diagnostics come out EQUAL over the battery.
    let battery = [
        "p--4",                    // the lexer's typed miss
        "not-a-utility",           // unknown root
        "p-red",                   // shape rejected
        "hover:not-a-variant:p-4", // outside the typed Cond surface
        "!p-4",                    // the importance flag
        "bg-red-500/50",           // modifier without the theme pass
        "leading-[13px]",          // the sibling-row quiet gate
        "w-[calc(100%-2rem)]",     // the composite boundary
        "grid-cols-[0_1fr]",       // the underscore-join boundary
        "bg-[url(x.png)]",         // the image boundary
        "p-4 not-a-utility",       // mixed: one lowered, one reported
    ];
    for class in battery {
        let ir = compile(class, &Config::default()).expect("compiles");
        let sheet = emit_rust(class, &Config::default()).expect("compiles");
        assert_eq!(
            ir.diagnostics, sheet.diagnostics,
            "the two walks agree on {class}"
        );
    }
}

// ---------------------------------------------------------------------------
// 6. The custom seam: additive, byte-shared
// ---------------------------------------------------------------------------

#[test]
fn the_custom_seam_renders_through_the_same_walk() {
    let customs = Customs::new().utility(CustomUtility::functional(
        "brand-p",
        CustomForm::Spacing,
        &[
            Prop::PaddingTop,
            Prop::PaddingRight,
            Prop::PaddingBottom,
            Prop::PaddingLeft,
        ],
    ));
    let sheet = emit_rust_with("brand-p-4", &customs, &Config::default()).expect("compiles");
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    assert_eq!(sheet.fragments.len(), 1);
    assert!(
        sheet.fragments[0]
            .tokens
            .contains("aeolus_ir::Val::Rem(1.0)"),
        "the registration lowered through the shared value pass: {}",
        sheet.fragments[0].tokens
    );
}

#[test]
fn empty_customs_are_byte_identical_to_the_plain_entry_points() {
    // The seam is additive, never a behaviour switch — the differential
    // pin over a battery.
    let battery = [
        "p-4 flex-col",
        "hover:bg-red-500/50",
        "text-sm w-1/2",
        "not-a-utility p--4",
    ];
    for class in battery {
        let plain = emit_rust(class, &Config::default()).expect("compiles");
        let seamed = emit_rust_with(class, &Customs::new(), &Config::default()).expect("compiles");
        assert_eq!(
            plain.to_string(),
            seamed.to_string(),
            "empty registrations change nothing on {class}"
        );
    }
}

// ---------------------------------------------------------------------------
// 7. The arm: Target::Rust completes the plan §3.1 surface
// ---------------------------------------------------------------------------

#[test]
fn the_target_enum_carries_all_four_arms() {
    // The plan §3.1 diagram's four emit forms, all armed: (a) Rust
    // tokens, (b) static data, (c) interp, (d) CSS.
    let arms = [Target::Css, Target::Rust, Target::Interp, Target::Static];
    assert_eq!(arms.len(), 4, "the emit surface is complete");
}
