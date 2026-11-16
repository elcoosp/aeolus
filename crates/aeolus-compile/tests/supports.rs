//! The supports-variant interp surface (the `supports-*` family, the
//! at-rule seam beside the media tier). The seam:
//!
//! * [`Cond::Supports`] — the typed condition whose payload is the
//!   rendered `@supports` condition text (`(display:grid)`,
//!   `selector(:hover)`), interned like the attribute-probe payloads;
//! * [`SupportsProbe`] — the one question a host answers (the same
//!   shape as `AttrProbe`/`ContainerProbe`/`StructProbe`/
//!   `DescendantProbe`: the walk is generic, the host wires its own);
//! * [`SupportsState`] — the built-in source: the supported conditions
//!   as a static table (presence IS the answer, the zero-allocation
//!   form);
//! * the `*_supports` entry points — the seam-family convention's fifth
//!   member (the `*_probes`/`*_containers`/`*_struct`/`*_descendants`
//!   precedent, the full composed matrix included).
//!
//! The payload grammar is the real tailwindcss v4.1.17 supports
//! normalization (packages/tailwindcss/src/variants.ts, the
//! `functional('supports', …)` arm), pinned as the differential source:
//!
//! * `supports-[(display:grid)]` / any function-prefixed inner — the
//!   condition passes through verbatim, `and`/`or`/`not` connectives
//!   spaced (`(display:grid) or (hover:hover)`);
//! * `supports-[display:grid]` — a colon-bearing declaration wraps in
//!   parens (`(display:grid)`);
//! * `supports-[display]` — the bare shorthand expands to the
//!   declaration probe (`(display: var(--tw))`, tailwind's own words);
//!
//! and the honesty tightening: an empty inner, braces, `;`, unbalanced
//! parens and bare (non-bracketed) spellings refuse — a condition no
//! `@supports` grammar accepts never folds, the typed
//! unsupported-variant diagnostic is the report.
//!
//! The interp semantics the pins encode: a supports layer FIRES when
//! the source answers yes for ITS condition text; a source-answered no
//! is a QUIET skip (a CSS `@supports` block the render target fails,
//! not an exclusion) — the note stays for the UNQUERIED cases only (no
//! source rides), where it becomes the dedicated
//! [`InterpNote::SupportsUnqueried`] (the per-seam note convention the
//! container/structural/descendant seams follow). The render target's
//! capability set is not frame state — the [`Env`] cannot answer it
//! (the structural seam's own reading), so there is no sourceless
//! fallback value.
//!
//! Written FIRST (the RED spec): the suite fails against the 0.19.0
//! seam — the condition, the seam types and the entry points do not
//! exist yet (the fold refuses every `supports-*` spelling as
//! unsupported-variant, pinned in `the_refusal_grammar_stays_honest`
//! until the arm lands).

use aeolus_compile::{
    AttrBit, AttrKind, AttrState, Compiled, Config, ContainerBound, ContainerState, DescendantFact,
    DescendantKind, DescendantState, Env, InterpNote, Interped, StructFacts, SupportsState,
    compile, compile_theme, interp, interp_containers_descendants_supports,
    interp_containers_struct_descendants_supports, interp_containers_struct_supports,
    interp_containers_supports, interp_descendants_supports,
    interp_probes_containers_descendants_supports,
    interp_probes_containers_struct_descendants_supports, interp_probes_containers_struct_supports,
    interp_probes_containers_supports, interp_probes_descendants_supports,
    interp_probes_struct_descendants_supports, interp_probes_struct_supports,
    interp_probes_supports, interp_struct_descendants_supports, interp_struct_supports,
    interp_supports, interp_theme_containers_struct_descendants_supports,
    interp_theme_containers_struct_supports, interp_theme_containers_supports,
    interp_theme_descendants_supports, interp_theme_probes_containers_struct_descendants_supports,
    interp_theme_probes_containers_supports, interp_theme_probes_struct_descendants_supports,
    interp_theme_probes_struct_supports, interp_theme_probes_supports,
    interp_theme_struct_supports, interp_theme_supports, supports_condition,
};
use aeolus_ir::{Cond, Prop, Val};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn runtime_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

/// The target's capability facts: the render target supports the grid
/// declaration probe and the `selector(:hover)` function form; it does
/// NOT support the hover-declaration compound or the container-type
/// shorthand (the answered-miss rows).
fn target_state() -> SupportsState<'static> {
    SupportsState::new(&["(display:grid)", "selector(:hover)"])
}

/// The element state the attribute probes answer for (the composed
/// walks' attribute half).
fn card_attrs() -> AttrState<'static> {
    AttrState::new(&[AttrBit {
        kind: AttrKind::Data,
        name: "state",
        value: "open",
    }])
}

/// The subtree facts the descendant probes answer for (the composed
/// walks' descendant half).
fn tree_state() -> DescendantState<'static> {
    DescendantState::new(&[DescendantFact {
        kind: DescendantKind::Element,
        fragment: "*:focus",
    }])
}

/// The value one longhand resolved to.
fn resolved(interp: &Interped, prop: Prop) -> Val {
    interp
        .decls
        .iter()
        .find(|d| d.prop == prop)
        .unwrap_or_else(|| panic!("{prop:?} is in the resolved declarations"))
        .val
}

fn note_for<'i>(interp: &'i Interped, cond: &Cond) -> Option<&'i InterpNote> {
    interp.notes.iter().find(|note| match note {
        InterpNote::SupportsUnqueried { cond: c, .. } => c == cond,
        _ => false,
    })
}

// ---------------------------------------------------------------------------
// The payload grammar: the corpus normalization, pinned
// ---------------------------------------------------------------------------

#[test]
fn the_declaration_form_wraps_in_parens() {
    // `supports-[display:grid]` — a colon-bearing declaration wraps
    // (tailwind's own wrap branch): the payload is the wrapped text,
    // verbatim, no space inserted.
    assert_eq!(
        supports_condition("display:grid").expect("a declaration folds"),
        "(display:grid)"
    );
}

#[test]
fn the_function_form_passes_through_verbatim() {
    // Function-prefixed inners are conditions already — the payload is
    // the inner text itself.
    assert_eq!(
        supports_condition("selector(:hover)").expect("a function form folds"),
        "selector(:hover)"
    );
    assert_eq!(
        supports_condition("(display:grid)").expect("a paren-prefixed form folds"),
        "(display:grid)"
    );
}

#[test]
fn the_connectives_carry_the_spacing_pass() {
    // The function branch spaces `and`/`or`/`not` (Chrome's
    // `(a)or(b)` bug is tailwind's own normalization reason).
    assert_eq!(
        supports_condition("(display:grid)or(hover:hover)").expect("a compound folds"),
        "(display:grid) or (hover:hover)"
    );
    assert_eq!(
        supports_condition("not (display:grid)").expect("a negation folds"),
        "not (display:grid)"
    );
}

#[test]
fn the_shorthand_expands_to_the_declaration_probe() {
    // A colon-less bare feature is the declaration shorthand:
    // `supports-[display]` -> `(display: var(--tw))`, tailwind's own
    // words for the form.
    assert_eq!(
        supports_condition("display").expect("the shorthand folds"),
        "(display: var(--tw))"
    );
}

#[test]
fn the_two_spellings_of_one_condition_are_one_cond() {
    // `[(display:grid)]` and `[display:grid]` normalize to the SAME
    // payload — one condition, one layer, the attribute-probe
    // equality contract (the payload text compares, never the pointer).
    assert_eq!(
        supports_condition("(display:grid)"),
        supports_condition("display:grid"),
        "the wrapped and wrapped-for-you forms are one condition"
    );
}

#[test]
fn the_refusal_grammar_stays_honest() {
    // The empty inner, braces, the rule separator, unbalanced parens
    // and the bare (non-bracketed) spellings refuse — the typed
    // unsupported-variant diagnostic is the report, never a broken
    // `@supports` head.
    assert!(supports_condition("").is_none(), "empty refuses");
    assert!(
        supports_condition("display{grid}").is_none(),
        "braces refuse"
    );
    assert!(supports_condition("display;grid").is_none(), "; refuses");
    assert!(
        supports_condition("(display:grid").is_none(),
        "an unbalanced wrap refuses"
    );
    assert!(
        supports_condition("selector(:hover").is_none(),
        "an unbalanced function form refuses"
    );
}

#[test]
fn the_variant_folds_onto_the_typed_cond() {
    // The whole variant segment folds onto `Cond::Supports` with the
    // interned payload — the CSS arm renders the head from the SAME
    // text (one grammar, the render cannot drift the fold).
    let compiled = compile("supports-[display:grid]:p-8", &Config::default()).expect("compiles");
    let cond = compiled
        .program
        .layers
        .iter()
        .find_map(|layer| match layer.cond {
            Cond::Supports(cond) => Some(cond),
            _ => None,
        })
        .expect("the supports layer is in the program");
    assert_eq!(cond, "(display:grid)");
}

#[test]
fn the_condition_rides_the_at_rule_tier() {
    // The specificity contract: `@supports` sorts BESIDE the media
    // tier (tailwind's own conditional-rule order — media, supports,
    // container — maps onto the flat tier model's 4 < 5).
    let supports = Cond::Supports("(display:grid)");
    let media = Cond::Bp(aeolus_ir::Bp::Md);
    let container = Cond::Container {
        bp: aeolus_ir::Bp::Md,
        name: None,
    };
    assert_eq!(supports.specificity(), media.specificity());
    assert!(media.specificity() < container.specificity());
}

// ---------------------------------------------------------------------------
// The element-own half: the walk fires and skips
// ---------------------------------------------------------------------------

#[test]
fn supports_layer_fires_when_the_target_answers_yes() {
    let compiled = compile("supports-[display:grid]:p-8", &Config::default()).expect("compiles");
    let interped =
        interp_supports(&compiled.program, &Env::default(), &target_state()).expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "a fire is not a note");
}

#[test]
fn supports_layer_misses_quietly_when_the_target_answers_no() {
    // The target supports grid and the selector form, NOT the
    // hover-declaration compound — the `supports-[hover:hover]` layer
    // misses, and an ANSWERED miss is CSS-quiet (an `@supports` block
    // the render target fails), never a note.
    let compiled = compile("supports-[hover:hover]:p-8", &Config::default()).expect("compiles");
    let interped =
        interp_supports(&compiled.program, &Env::default(), &target_state()).expect("resolves");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);
    assert!(interped.notes.is_empty(), "{:?}", interped.notes);
}

#[test]
fn the_function_form_asks_the_same_seam() {
    // The selector function form rides the same payload equality: the
    // table's `selector(:hover)` row answers the `supports-[selector(:hover)]`
    // layer.
    let compiled =
        compile("supports-[selector(:hover)]:p-8", &Config::default()).expect("compiles");
    let interped =
        interp_supports(&compiled.program, &Env::default(), &target_state()).expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
}

#[test]
fn sourceless_supports_layers_stay_reported() {
    // The plain `interp` carries no source: the layer reports through
    // the DEDICATED note (the per-seam convention), never silently —
    // and the note's help names the CLI fact flag (the observability
    // contract the other per-seam notes keep).
    let compiled = compile("supports-[display:grid]:p-8", &Config::default()).expect("compiles");
    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    assert!(interped.decls.is_empty());
    let note = note_for(&interped, &Cond::Supports("(display:grid)")).expect("the note rides");
    match note {
        InterpNote::SupportsUnqueried { cond, props } => {
            assert_eq!(*cond, Cond::Supports("(display:grid)"));
            assert_eq!(props.len(), 4, "the four padding longhands ride the note");
        }
        other => panic!("the supports note, not {other:?}"),
    }
    let text = note.to_string();
    assert!(!text.is_empty(), "the note reads for humans");
    assert!(text.contains("never matched"), "the note reports: {text}");
}

// ---------------------------------------------------------------------------
// The CSS arm: the at-rule head renders the payload
// ---------------------------------------------------------------------------

#[test]
fn the_css_arm_renders_the_at_rule_head() {
    let sheet =
        aeolus_compile::emit_css("supports-[display:grid]:p-8", &Config::default()).expect("emits");
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    let rendered = sheet.to_string();
    assert!(
        rendered.contains("@supports (display:grid) {"),
        "the at-rule head is the payload verbatim: {rendered}"
    );
    assert!(rendered.contains("padding-top: 2rem"), "{rendered}");
}

#[test]
fn the_css_arm_renders_the_function_form_head() {
    let sheet = aeolus_compile::emit_css("supports-[selector(:hover)]:p-8", &Config::default())
        .expect("emits");
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    let rendered = sheet.to_string();
    assert!(
        rendered.contains("@supports selector(:hover) {"),
        "the function form renders unwrapped: {rendered}"
    );
}

// ---------------------------------------------------------------------------
// The composed matrix: the seam joins the walk family
// ---------------------------------------------------------------------------

#[test]
fn attribute_and_supports_probes_compose() {
    // Both probes ride one walk: the data- layer answers from the
    // attribute seam, the supports- layer from the supports seam.
    let compiled = compile(
        "data-[state=open]:pt-2 supports-[display:grid]:pb-8",
        &Config::default(),
    )
    .expect("compiles");
    let interped = interp_probes_supports(
        &compiled.program,
        &Env::default(),
        &card_attrs(),
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(0.5));
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "{:?}", interped.notes);
}

#[test]
fn container_and_supports_probes_compose() {
    // The named container answers from the container seam, the
    // supports- layer from the supports seam.
    let compiled = compile(
        "@md/card:w-64 supports-[display:grid]:pb-8",
        &Config::default(),
    )
    .expect("compiles");
    let bounds = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let interped = interp_containers_supports(
        &compiled.program,
        &Env::default(),
        &ContainerState::new(&bounds),
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::Width), Val::Rem(16.0));
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(2.0));
}

#[test]
fn structural_and_supports_probes_compose() {
    let compiled = compile(
        "first:pt-2 supports-[display:grid]:pb-8",
        &Config::default(),
    )
    .expect("compiles");
    let interped = interp_struct_supports(
        &compiled.program,
        &Env::default(),
        &StructFacts {
            index: Some(0),
            sibling_count: Some(3),
            empty: false,
        },
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(0.5));
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(2.0));
}

#[test]
fn descendant_and_supports_probes_compose() {
    let compiled = compile(
        "has-focus:pt-2 supports-[display:grid]:pb-8",
        &Config::default(),
    )
    .expect("compiles");
    let interped = interp_descendants_supports(
        &compiled.program,
        &Env::default(),
        &tree_state(),
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(0.5));
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(2.0));
}

#[test]
fn the_themeless_full_form_rides_every_seam() {
    // Attribute + container + structural + descendant + supports
    // probes, one walk: the FULL composed form the matrix completion
    // lands — the pipeline's fifth seam dimension beside the four.
    let compiled = compile(
        "data-[state=open]:pt-2 @md/card:mt-4 first:ml-1 has-focus:mb-1 supports-[display:grid]:pb-8",
        &Config::default(),
    )
    .expect("compiles");
    let bounds = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let interped = interp_probes_containers_struct_descendants_supports(
        &compiled.program,
        &Env::default(),
        &card_attrs(),
        &ContainerState::new(&bounds),
        &StructFacts {
            index: Some(0),
            sibling_count: Some(3),
            empty: false,
        },
        &tree_state(),
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(0.5));
    assert_eq!(resolved(&interped, Prop::MarginTop), Val::Rem(1.0));
    assert_eq!(resolved(&interped, Prop::MarginBottom), Val::Rem(0.25));
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "{:?}", interped.notes);
}

#[test]
fn the_themed_full_form_resolves_tokens_too() {
    // The theme forms ride the same walk: one token value resolves
    // through the source while every seam answers.
    let compiled = compile_theme(
        "bg-background supports-[display:grid]:p-8 data-[state=open]:pt-2 @md/card:mt-4 first:ml-1",
        &runtime_table(),
        &Config::default(),
    )
    .expect("compiles");
    let bounds = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let interped = interp_theme_probes_containers_struct_descendants_supports(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &card_attrs(),
        &ContainerState::new(&bounds),
        &StructFacts {
            index: Some(0),
            sibling_count: Some(3),
            empty: false,
        },
        &DescendantState::new(&[]),
        &target_state(),
    )
    .expect("resolves");
    assert!(matches!(
        resolved(&interped, Prop::Background),
        Val::Rgba(_)
    ));
    // The supports layer rides the AT-RULE tier (4) ABOVE the
    // data probe's tier (2): its p-8 wins every longhand the two
    // layers share, the named container's mt-4 (tier 5) sorts after
    // it and keeps the top margin.
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(2.0));
    assert_eq!(resolved(&interped, Prop::MarginTop), Val::Rem(1.0));
    assert!(interped.notes.is_empty(), "{:?}", interped.notes);
}

#[test]
fn the_matrix_themed_pairs_answer_their_seams() {
    // The remaining themed pairs: theme+containers+supports,
    // theme+struct+supports, theme+probes+supports,
    // theme+descendants+supports.
    let bounds = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let facts = StructFacts {
        index: Some(0),
        sibling_count: Some(3),
        empty: false,
    };

    let compiled = compile_theme(
        "@md/card:w-64 supports-[display:grid]:p-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("c");
    let interped = interp_theme_containers_supports(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &ContainerState::new(&bounds),
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    let compiled = compile_theme(
        "empty:pt-2 supports-[display:grid]:p-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("c");
    let interped = interp_theme_struct_supports(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &facts,
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    let compiled = compile_theme(
        "data-[state=open]:pt-2 supports-[display:grid]:p-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("c");
    let interped = interp_theme_probes_supports(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &card_attrs(),
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    let compiled = compile_theme(
        "has-focus:pt-2 supports-[display:grid]:p-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("c");
    let interped = interp_theme_descendants_supports(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &tree_state(),
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    // The themed D-alone form: the supports seam rides the themed
    // walk with nothing else composed.
    let compiled = compile_theme(
        "supports-[display:grid]:p-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("c");
    let interped = interp_theme_supports(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    // The themed triples: theme+probes+containers+supports and
    // theme+containers+struct+supports.
    let compiled = compile_theme(
        "data-[state=open]:pt-2 @md/card:mt-4 supports-[display:grid]:p-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("c");
    let interped = interp_theme_probes_containers_supports(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &card_attrs(),
        &ContainerState::new(&bounds),
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    let compiled = compile_theme(
        "@md/card:w-64 empty:pt-2 supports-[display:grid]:p-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("c");
    let interped = interp_theme_containers_struct_supports(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &ContainerState::new(&bounds),
        &facts,
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    // The themed triple: theme+probes+struct+supports.
    let compiled = compile_theme(
        "data-[state=open]:pt-2 first:mt-1 supports-[display:grid]:p-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("c");
    let interped = interp_theme_probes_struct_supports(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &card_attrs(),
        &facts,
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
}

#[test]
fn the_matrix_themeless_triples_and_quads_answer_their_seams() {
    let bounds = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let facts = StructFacts {
        index: Some(0),
        sibling_count: Some(3),
        empty: false,
    };

    // themeless: containers+struct+supports, probes+containers+supports.
    let compiled =
        compile("first:pt-2 supports-[display:grid]:p-8", &Config::default()).expect("c");
    let interped = interp_containers_struct_supports(
        &compiled.program,
        &Env::default(),
        &ContainerState::new(&bounds),
        &facts,
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    let compiled = compile(
        "data-[state=open]:pt-2 supports-[display:grid]:p-8",
        &Config::default(),
    )
    .expect("c");
    let interped = interp_probes_containers_supports(
        &compiled.program,
        &Env::default(),
        &card_attrs(),
        &ContainerState::new(&bounds),
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    // The quads beside the full form: probes+struct+supports,
    // containers+descendants+supports, probes+descendants+supports,
    // struct+descendants+supports, probes+struct+descendants+supports,
    // containers+struct+descendants+supports,
    // probes+containers+struct+supports,
    // probes+containers+descendants+supports,
    // theme+probes+struct+descendants+supports,
    // theme+containers+struct+descendants+supports.
    let compiled =
        compile("first:pt-2 supports-[display:grid]:p-8", &Config::default()).expect("c");
    let interped = interp_probes_struct_supports(
        &compiled.program,
        &Env::default(),
        &card_attrs(),
        &facts,
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    let compiled = compile(
        "@md/card:w-64 has-focus:pt-2 supports-[display:grid]:p-8",
        &Config::default(),
    )
    .expect("c");
    let interped = interp_containers_descendants_supports(
        &compiled.program,
        &Env::default(),
        &ContainerState::new(&bounds),
        &tree_state(),
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    let compiled = compile(
        "data-[state=open]:pt-2 has-focus:p-8 supports-[display:grid]:pb-8",
        &Config::default(),
    )
    .expect("c");
    let interped = interp_probes_descendants_supports(
        &compiled.program,
        &Env::default(),
        &card_attrs(),
        &tree_state(),
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(2.0));

    let compiled = compile(
        "nth-2:pt-2 has-focus:p-8 supports-[display:grid]:pb-8",
        &Config::default(),
    )
    .expect("c");
    let interped = interp_struct_descendants_supports(
        &compiled.program,
        &Env::default(),
        &facts,
        &tree_state(),
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(2.0));

    let compiled = compile(
        "data-[state=open]:pt-2 nth-2:mt-1 has-focus:p-8 supports-[display:grid]:pb-8",
        &Config::default(),
    )
    .expect("c");
    let interped = interp_probes_struct_descendants_supports(
        &compiled.program,
        &Env::default(),
        &card_attrs(),
        &facts,
        &tree_state(),
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(2.0));

    let compiled = compile(
        "@md/card:mt-4 nth-2:pt-2 has-focus:p-8 supports-[display:grid]:pb-8",
        &Config::default(),
    )
    .expect("c");
    let interped = interp_containers_struct_descendants_supports(
        &compiled.program,
        &Env::default(),
        &ContainerState::new(&bounds),
        &facts,
        &tree_state(),
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(2.0));

    let compiled = compile(
        "data-[state=open]:pt-2 @md/card:mt-4 first:ml-1 supports-[display:grid]:pb-8",
        &Config::default(),
    )
    .expect("c");
    let interped = interp_probes_containers_struct_supports(
        &compiled.program,
        &Env::default(),
        &card_attrs(),
        &ContainerState::new(&bounds),
        &facts,
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(2.0));

    let compiled = compile(
        "data-[state=open]:pt-2 @md/card:mt-4 has-focus:mb-1 supports-[display:grid]:pb-8",
        &Config::default(),
    )
    .expect("c");
    let interped = interp_probes_containers_descendants_supports(
        &compiled.program,
        &Env::default(),
        &card_attrs(),
        &ContainerState::new(&bounds),
        &tree_state(),
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(2.0));

    // The themed quads beside the full form.
    let compiled = compile_theme(
        "data-[state=open]:pt-2 nth-2:mt-1 has-focus:mb-1 supports-[display:grid]:pb-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("c");
    let interped = interp_theme_probes_struct_descendants_supports(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &card_attrs(),
        &facts,
        &tree_state(),
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(2.0));

    let compiled = compile_theme(
        "@md/card:mt-4 nth-2:pt-2 has-focus:mb-1 supports-[display:grid]:pb-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("c");
    let interped = interp_theme_containers_struct_descendants_supports(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &ContainerState::new(&bounds),
        &facts,
        &tree_state(),
        &target_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(2.0));
}

// ---------------------------------------------------------------------------
// The ergonomic half: the Compiled method face
// ---------------------------------------------------------------------------

#[test]
fn the_compiled_method_face_carries_the_seam() {
    let compiled: Compiled = compile("supports-[display:grid]:p-8", &Config::default())
        .expect("the class string compiles");
    let interped = compiled
        .interp_supports(&Env::default(), &target_state())
        .expect("the program resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    let themed = compile_theme(
        "bg-background supports-[display:grid]:p-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("c");
    let interped = themed
        .interp_theme_supports(&runtime_table(), &Env::default(), &target_state())
        .expect("resolves");
    assert!(matches!(
        resolved(&interped, Prop::Background),
        Val::Rgba(_)
    ));
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
}

// ---------------------------------------------------------------------------
// The built-in table: first-match semantics
// ---------------------------------------------------------------------------

#[test]
fn the_built_in_table_answers_presence() {
    // Presence in the table is the answer: a row fires the layer, its
    // absence quietly misses — the zero-allocation per-frame form.
    let table = SupportsState::new(&["(width > 400px)"]);
    // The paren-prefixed inner passes through verbatim (the fold's
    // function branch) — the payload the table row compares against.
    let hot = compile("supports-[(width_>_400px)]:p-8", &Config::default()).expect("compiles");
    let interped = interp_supports(&hot.program, &Env::default(), &table).expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
}

// ---------------------------------------------------------------------------
// The plan bridge: the v0.x surface's honest boundary
// ---------------------------------------------------------------------------

#[test]
fn the_plan_bridge_reports_the_condition_note() {
    // The v0.x StylePlan surface has no @supports slot: the layer's
    // style does not land, the typed note names the condition (the
    // same boundary the unmappable has- fragments keep).
    let sheet = aeolus_compile::emit_plan("supports-[display:grid]:p-8", &Config::default())
        .expect("emits");
    let note = sheet
        .notes
        .iter()
        .find(|note| {
            matches!(
                note.kind,
                aeolus_compile::PlanNoteKind::SupportsCondition { .. }
            )
        })
        .expect("the condition note rides");
    match &note.kind {
        aeolus_compile::PlanNoteKind::SupportsCondition { cond } => {
            assert_eq!(*cond, "(display:grid)");
        }
        other => panic!("the supports note, not {other:?}"),
    }
    let text = note.to_string();
    assert!(!text.is_empty(), "the note reads for humans");
    // The mapped half of the same string lowers note-free: the boundary
    // is per-layer, never per-string.
    let plain = aeolus_compile::emit_plan("p-8", &Config::default()).expect("emits");
    assert!(plain.notes.is_empty(), "{:?}", plain.notes);
}
