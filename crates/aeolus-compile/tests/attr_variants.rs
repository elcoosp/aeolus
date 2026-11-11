//! The attribute-probe variant fold contracts (architecture plan §3.5's
//! data-/aria- rows and the §2/§5.4 has- family — the P2 slice the
//! variant.rs docs carried as "the next slice"): `data-*`, `aria-*` and
//! `has-*` variant segments fold onto the typed [`Cond`] surface and
//! lower, render, interp and emit through the SAME pipeline as the
//! built-in spellings.
//!
//! The differential oracle is IN this tree: the v0.x resolver's
//! `has_atom`/`parse_attr`/`attr_test` (aeolus-core), pinned against the
//! real tailwindcss v4.1.17 output in the golden corpus
//! (`tests/fixtures/golden.corpus`: `data-[state=open]` →
//! `[data-state="open"]`, bare `aria-busy:` → `[aria-busy="true"]`,
//! `has-focus:` → `:has(*:focus)`, `has-data-[slot=card-action]:` →
//! `:has(*[data-slot="card-action"])`). Every rendering form below is
//! the corpus's form.
//!
//! The pins, in the order the docs promise them:
//!
//! * the FOLD: `data-[state=open]:`/`data-open:` → `Cond::Data`,
//!   `aria-busy:`/`aria-[sort=asc]:` → `Cond::Aria` (bare aria names
//!   the "true" value, the corpus's own reading), `has-focus:`/
//!   `has-data-[…]:`/`has-aria-…:`/`has-[:focus]:` → `Cond::Has`;
//! * the flat fold still applies: stacked variants keep the dominant
//!   condition (the highest-specificity segment, source order breaking
//!   ties), so `hover:data-[x]:` folds to the probe;
//! * the GRAMMAR refuses honestly: empty names/values, selector-
//!   unsafe payloads, compound `has-` inners, structural inners (no
//!   `first:` surface yet), parent-relative compounds (`group-data-*`
//!   — the next slice) and arbitrary variants stay typed
//!   unsupported-variant reports, never silent;
//! * the CSS arm renders the corpus's selector forms: the attribute
//!   test compounds on the escaped class, the `:has(...)` fragment
//!   star-prefixes non-tag inners, and the probe rides BOTH rules of a
//!   baked dark-pair companion;
//! * the interpreter arm reports attribute probes as typed NOTES (the
//!   `Env` answers element states, not DOM probes — the plan §3.5
//!   state-bits surface is the follow-up), never silent, never an
//!   error;
//! * the static-data and rust-tokens arms emit the payload conds (the
//!   derived `Debug` IS the constructor text — the statics arm's
//!   documented name table);
//! * the custom seam stays generic: an alias may target the new conds,
//!   and the two walks' diagnostics stay EQUAL with probes in play.
//!
//! Written FIRST (the RED spec): the suite fails against the 0.16.0
//! seam and passes only once the fold satisfies every pin below.

use aeolus_compile::{
    Config, CustomVariant, Customs, DiagnosticKind, Env, InterpNote, cn, compile, compile_with,
    emit_css, emit_css_theme, emit_css_with, emit_rust, interp,
};
use aeolus_ir::{Cond, Prop, Val};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The baked-tier table (the dark-pair companion pins need it).
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::baked(&source).expect("default-seeded source builds")
}

/// The one layer a single-candidate string compiles to.
fn one_cond(input: &str) -> Cond {
    let compiled = compile(input, &Config::default()).expect("non-empty input compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "{input} lowered without reports: {:?}",
        compiled.diagnostics
    );
    compiled.program.layers[0].cond
}

// ---------------------------------------------------------------------------
// The fold: data-
// ---------------------------------------------------------------------------

#[test]
fn data_bracketed_value_folds_to_the_probe() {
    // The corpus's own class: `data-[state=open]:bg-green-500` renders
    // `&[data-state="open"]` — the (name, value) pair is the payload.
    let cond = one_cond("data-[state=open]:p-4");
    assert_eq!(
        cond,
        Cond::Data {
            name: "state",
            value: Some("open")
        }
    );
}

#[test]
fn data_bare_spelling_folds_to_the_presence_probe() {
    // Bare `data-*:` is the presence test — the corpus's `[data-open]`
    // form (the v0.x `attr_test`'s None arm).
    let cond = one_cond("data-open:p-4");
    assert_eq!(
        cond,
        Cond::Data {
            name: "open",
            value: None
        }
    );
}

#[test]
fn data_bracketed_presence_folds_like_the_bare_spelling() {
    // `data-[open]` (bracketed, no `=`) parses to the same presence
    // probe the bare spelling folds to — one condition, one layer.
    assert_eq!(
        one_cond("data-[open]:p-4"),
        one_cond("data-open:p-4"),
        "the bracketed and bare presence spellings are one condition"
    );
}

// ---------------------------------------------------------------------------
// The fold: aria-
// ---------------------------------------------------------------------------

#[test]
fn aria_bare_spelling_names_the_true_value() {
    // The corpus: `aria-busy:opacity-50` renders `&[aria-busy="true"]`.
    // Bare aria means "true" — the fold normalizes so both spellings
    // are ONE condition.
    let cond = one_cond("aria-busy:p-4");
    assert_eq!(
        cond,
        Cond::Aria {
            name: "busy",
            value: Some("true")
        }
    );
}

#[test]
fn aria_bracketed_value_folds_to_the_probe() {
    let cond = one_cond("aria-[sort=asc]:p-4");
    assert_eq!(
        cond,
        Cond::Aria {
            name: "sort",
            value: Some("asc")
        }
    );
}

#[test]
fn aria_bare_and_bracketed_true_are_one_condition() {
    assert_eq!(
        one_cond("aria-[busy=true]:p-4"),
        one_cond("aria-busy:p-4"),
        "the bracketed true and the bare spelling are one condition"
    );
}

// ---------------------------------------------------------------------------
// The fold: has-
// ---------------------------------------------------------------------------

#[test]
fn has_named_state_folds_to_the_star_prefixed_fragment() {
    // The corpus: `has-focus:ring-2` renders `&:has(*:focus)`.
    assert_eq!(one_cond("has-focus:p-4"), Cond::Has("*:focus"));
    assert_eq!(one_cond("has-hover:p-4"), Cond::Has("*:hover"));
}

#[test]
fn has_bracketed_state_folds_like_the_named_spelling() {
    // The v0.x oracle's `has_atom`: the bracketed state form strips to
    // the same state test (`has-[:focus]` ≡ `has-focus`).
    assert_eq!(
        one_cond("has-[:focus]:p-4"),
        one_cond("has-focus:p-4"),
        "the bracketed and named state spellings are one condition"
    );
}

#[test]
fn has_data_compound_folds_to_the_star_prefixed_attr_fragment() {
    // The corpus: `has-data-[slot=card-action]:grid-cols-2` renders
    // `&:has(*[data-slot="card-action"])`.
    assert_eq!(
        one_cond("has-data-[slot=card-action]:p-4"),
        Cond::Has("*[data-slot=\"card-action\"]")
    );
}

#[test]
fn has_aria_compound_names_the_true_value() {
    // The corpus: `has-aria-invalid:border-red-500` renders
    // `&:has(*[aria-invalid="true"])`.
    assert_eq!(
        one_cond("has-aria-invalid:p-4"),
        Cond::Has("*[aria-invalid=\"true\"]")
    );
}

// ---------------------------------------------------------------------------
// The flat fold: stacking and ordering
// ---------------------------------------------------------------------------

#[test]
fn stacked_variants_keep_the_dominant_condition() {
    // The flat-IR model: one condition per layer, the highest-
    // specificity segment wins, source order breaking ties. A probe is
    // the state tier (2), dark is the mode tier (1): the probe stays.
    let cond = one_cond("dark:data-[state=open]:p-4");
    assert_eq!(
        cond,
        Cond::Data {
            name: "state",
            value: Some("open")
        },
        "the attribute probe outranks the mode tier"
    );
}

#[test]
fn probe_and_state_stack_in_source_order_at_equal_rank() {
    // Two rank-2 conditions stay separate layers in source order (the
    // stable sort keeps first-seen order for equal specificity).
    let compiled = compile("hover:gap-2 data-[state=open]:p-4", &Config::default())
        .expect("non-empty input compiles");
    assert!(compiled.diagnostics.is_empty(), "{compiled:?}");
    let conds: Vec<Cond> = compiled.program.layers.iter().map(|l| l.cond).collect();
    assert_eq!(conds, vec![Cond::Hover, one_cond("data-[state=open]:p-4")]);
}

// ---------------------------------------------------------------------------
// The grammar's honest refusals
// ---------------------------------------------------------------------------

#[test]
fn empty_attr_fields_refuse() {
    for input in ["data-:p-4", "data-[state=]:p-4", "data-[:x]:p-4"] {
        let compiled = compile(input, &Config::default()).expect("non-empty input compiles");
        assert!(
            compiled
                .diagnostics
                .iter()
                .any(|d| matches!(d.kind, DiagnosticKind::UnsupportedVariant { .. })),
            "{input} refuses typed: {:?}",
            compiled.diagnostics
        );
    }
}

#[test]
fn selector_unsafe_payloads_refuse() {
    // Quotes and backslashes would break the rendered selector text —
    // the grammar refuses them instead of emitting broken CSS.
    for input in ["data-[x=\"a\"]:p-4", "aria-[x=\\a]:p-4"] {
        let compiled = compile(input, &Config::default()).expect("non-empty input compiles");
        assert!(
            compiled
                .diagnostics
                .iter()
                .any(|d| matches!(d.kind, DiagnosticKind::UnsupportedVariant { .. })),
            "{input} refuses typed: {:?}",
            compiled.diagnostics
        );
    }
}

#[test]
fn compound_and_structural_has_inners_refuse() {
    // The v0.x oracle's own boundary (`has_atom`): one simple test per
    // has- inner — compound selectors and the structural family (no
    // `first:` surface in the compile stage yet) stay typed refusals.
    for input in ["has-[p:hover]:p-4", "has-[>svg]:p-4", "has-first:p-4"] {
        let compiled = compile(input, &Config::default()).expect("non-empty input compiles");
        assert!(
            compiled
                .diagnostics
                .iter()
                .any(|d| matches!(d.kind, DiagnosticKind::UnsupportedVariant { .. })),
            "{input} refuses typed: {:?}",
            compiled.diagnostics
        );
    }
}

#[test]
fn unknown_bare_has_inner_refuses() {
    // `has-[x]` was the 0.16.0 refusal pin and STAYS one: a bare inner
    // that names no state and no data-/aria- compound is not a probe.
    let compiled = compile("has-[x]:p-4", &Config::default()).expect("compiles");
    assert_eq!(
        compiled.diagnostics[0].kind,
        DiagnosticKind::UnsupportedVariant {
            variant: "has-[x]".to_string()
        }
    );
}

#[test]
fn parent_relative_compounds_refuse_until_the_next_slice() {
    // `group-data-*` composes a parent probe WITH an attribute probe —
    // the parent-relative ATTRIBUTE composition (group-data-*, peer-
    // aria-*) stays the typed boundary even after the parent-relative
    // has-probes land; only the has- family composes today.
    for input in ["group-data-open:p-4", "peer-aria-busy:p-4"] {
        let compiled = compile(input, &Config::default()).expect("non-empty input compiles");
        assert!(
            compiled
                .diagnostics
                .iter()
                .any(|d| matches!(d.kind, DiagnosticKind::UnsupportedVariant { .. })),
            "{input} refuses typed: {:?}",
            compiled.diagnostics
        );
    }
}

#[test]
fn refusal_spans_the_segment() {
    let compiled = compile("data-[x=]:p-4", &Config::default()).expect("compiles");
    let d = &compiled.diagnostics[0];
    assert_eq!(d.span, (0, 9), "the variant segment's own span");
    assert_eq!(d.kind.code(), "aeolus::compile::unsupported_variant");
    assert!(!d.kind.help().is_empty());
}

// ---------------------------------------------------------------------------
// The CSS arm: the corpus's selector forms
// ---------------------------------------------------------------------------

#[test]
fn css_renders_the_corpus_attr_selector() {
    let sheet =
        emit_css("data-[state=open]:p-4", &Config::default()).expect("non-empty input renders");
    assert!(sheet.diagnostics.is_empty(), "{sheet:?}");
    assert_eq!(sheet.rules.len(), 1);
    assert_eq!(
        sheet.rules[0].selector, ".data-\\[state\\=open\\]\\:p-4[data-state=\"open\"]",
        "the escaped class compounds the attribute test"
    );
    assert_eq!(
        sheet.rules[0].body,
        "padding-top: 1rem;\npadding-right: 1rem;\npadding-bottom: 1rem;\npadding-left: 1rem;"
    );
}

#[test]
fn css_renders_the_presence_form() {
    let sheet = emit_css("data-open:p-4", &Config::default()).expect("non-empty input renders");
    assert_eq!(sheet.rules[0].selector, ".data-open\\:p-4[data-open]");
}

#[test]
fn css_renders_the_bare_aria_true_form() {
    let sheet = emit_css("aria-busy:p-4", &Config::default()).expect("non-empty input renders");
    assert_eq!(
        sheet.rules[0].selector,
        ".aria-busy\\:p-4[aria-busy=\"true\"]"
    );
}

#[test]
fn css_renders_the_star_prefixed_has_fragment() {
    let sheet = emit_css("has-focus:p-4", &Config::default()).expect("non-empty input renders");
    assert_eq!(sheet.rules[0].selector, ".has-focus\\:p-4:has(*:focus)");
}

#[test]
fn css_renders_the_has_attr_compound() {
    let sheet = emit_css(
        "has-data-[slot=card-action]:grid-cols-2",
        &Config::default(),
    )
    .expect("non-empty input renders");
    assert!(sheet.diagnostics.is_empty(), "{sheet:?}");
    assert!(
        sheet.rules[0]
            .selector
            .ends_with(":has(*[data-slot=\"card-action\"])"),
        "the corpus's :has form: {}",
        sheet.rules[0].selector
    );
}

#[test]
fn css_compounds_probe_after_pseudo_in_source_stack() {
    // The flat arm's documented order: the pseudo suffixes ride first,
    // the attribute probes compound after, the dark scope last.
    let sheet = emit_css("hover:data-[state=open]:p-4", &Config::default())
        .expect("non-empty input renders");
    assert_eq!(
        sheet.rules[0].selector,
        ".hover\\:data-\\[state\\=open\\]\\:p-4:hover[data-state=\"open\"]"
    );
}

#[test]
fn css_media_wrap_carries_the_probe() {
    let sheet = emit_css("md:data-open:hidden", &Config::default()).expect("non-empty renders");
    assert!(sheet.diagnostics.is_empty(), "{sheet:?}");
    assert_eq!(
        sheet.rules[0].selector,
        ".md\\:data-open\\:hidden[data-open]"
    );
    assert!(sheet.rules[0].media.is_some(), "the md: at-rule wraps");
}

#[test]
fn css_dark_pair_companion_carries_the_probe() {
    let tokens = baked_table();
    let sheet = emit_css_theme("data-open:bg-primary", &tokens, &Config::default())
        .expect("non-empty input renders");
    assert!(sheet.diagnostics.is_empty(), "{sheet:?}");
    assert!(
        sheet.rules.len() >= 2,
        "the baked dark pair renders two rules: {sheet:?}"
    );
    for rule in &sheet.rules {
        assert!(
            rule.selector.ends_with("[data-open]"),
            "both halves keep the probe: {}",
            rule.selector
        );
    }
}

// ---------------------------------------------------------------------------
// The interpreter arm: typed exclusion notes
// ---------------------------------------------------------------------------

#[test]
fn interp_reports_attribute_probes_as_notes() {
    let compiled = compile("data-open:p-4", &Config::default()).expect("compiles");
    let interped =
        interp(&compiled.program, &Env::default()).expect("no theme forms in the program");
    assert!(
        interped.decls.is_empty(),
        "the probe never fires under a state-less Env"
    );
    assert_eq!(interped.notes.len(), 1, "{:?}", interped.notes);
    match &interped.notes[0] {
        InterpNote::AttributeUnqueried { cond, props } => {
            assert_eq!(
                *cond,
                Cond::Data {
                    name: "open",
                    value: None
                }
            );
            assert_eq!(props.len(), 4, "the four padding longhands ride the note");
        }
        other => panic!("the attribute note, not {other:?}"),
    }
    assert!(!interped.notes[0].to_string().is_empty());
}

#[test]
fn interp_fires_the_base_layers_and_notes_the_probes() {
    let compiled = compile("p-2 data-open:p-4", &Config::default()).expect("compiles");
    let interped =
        interp(&compiled.program, &Env::default()).expect("no theme forms in the program");
    assert_eq!(interped.decls.len(), 4, "the base layer's paddings resolve");
    for decl in &interped.decls {
        assert_eq!(decl.val, Val::Rem(0.5), "p-2 is 0.5rem");
    }
    assert_eq!(interped.notes.len(), 1, "the data layer reports, not fires");
}

// ---------------------------------------------------------------------------
// The static-data + rust-tokens arms: the payload conds emit
// ---------------------------------------------------------------------------

#[test]
fn static_form_emits_the_payload_conds() {
    let compiled = compile("data-open:p-4 has-focus:gap-1", &Config::default()).expect("compiles");
    let text = compiled.emit_static();
    assert!(
        text.contains("aeolus_ir::Cond::Data { name: \"open\", value: None }"),
        "the Data payload emits as the constructor: {text}"
    );
    assert!(
        text.contains("aeolus_ir::Cond::Has(\"*:focus\")"),
        "the Has fragment emits as the constructor: {text}"
    );
}

#[test]
fn rust_fragments_carry_the_payload_cond() {
    let sheet = emit_rust("data-open:p-4", &Config::default()).expect("non-empty input renders");
    assert!(sheet.diagnostics.is_empty(), "{sheet:?}");
    assert_eq!(sheet.fragments.len(), 1);
    assert!(
        sheet.fragments[0]
            .tokens
            .contains("aeolus_ir::Cond::Data { name: \"open\", value: None }"),
        "the fragment embeds the payload cond: {}",
        sheet.fragments[0].tokens
    );
}

// ---------------------------------------------------------------------------
// The merge and the custom seam stay generic
// ---------------------------------------------------------------------------

#[test]
fn cn_merges_same_probe_layers_and_keeps_disjoint_ones() {
    let a = compile("data-[state=open]:p-4", &Config::default()).expect("compiles");
    let b = compile("data-[state=open]:px-8", &Config::default()).expect("compiles");
    let merged = cn(&[&a, &b]).expect("non-empty inputs merge");
    assert!(merged.diagnostics.is_empty());
    assert_eq!(merged.program.layers.len(), 1, "same cond: one layer");
    // px-8 wins left/right over p-4 (the last declaration per longhand).
    let left = merged.program.layers[0]
        .decls
        .iter()
        .find(|d| d.prop == Prop::PaddingLeft)
        .expect("padding-left survives");
    assert_eq!(left.val, Val::Rem(2.0), "px-8 is 2rem");

    let c = compile("data-open:p-4", &Config::default()).expect("compiles");
    let disjoint = cn(&[&a, &c]).expect("merge");
    assert_eq!(
        disjoint.program.layers.len(),
        2,
        "different probes stay separate layers"
    );
}

#[test]
fn custom_alias_may_target_the_probe_conds() {
    // The seam is generic over `Cond`: a registered alias can name the
    // new conditions, and both walks fold and render it identically.
    let customs = Customs::new().variant(CustomVariant::new(
        "flagged",
        Cond::Data {
            name: "flag",
            value: Some("on"),
        },
    ));
    let compiled = compile_with("flagged:p-4", &customs, &Config::default())
        .expect("non-empty input compiles");
    assert!(compiled.diagnostics.is_empty(), "{compiled:?}");
    assert_eq!(
        compiled.program.layers[0].cond,
        Cond::Data {
            name: "flag",
            value: Some("on")
        }
    );

    let sheet = emit_css_with("flagged:p-4", &customs, &Config::default())
        .expect("non-empty input renders");
    assert!(sheet.diagnostics.is_empty(), "{sheet:?}");
    assert_eq!(sheet.rules[0].selector, ".flagged\\:p-4[data-flag=\"on\"]");
}

#[test]
fn the_two_walks_report_identically_with_probes_in_play() {
    let battery = [
        "data-[state=open]:p-4",
        "data-open:p-4",
        "data-[x=]:p-4",
        "data-:p-4",
        "aria-busy:p-4",
        "aria-[sort=asc]:p-4",
        "has-focus:p-4",
        "has-data-[slot=card-action]:p-4",
        "has-aria-invalid:p-4",
        "has-[:focus]:p-4",
        "has-[x]:p-4",
        "has-[p:hover]:p-4",
        "has-first:p-4",
        "group-data-open:p-4",
        "hover:data-[state=open]:p-4",
        "md:data-open:hidden",
        "p-4 data-open:gap-2",
        "group-has-focus:p-4",
        "peer-has-data-[slot=x]:p-4",
        "group-has-focus/menu:p-4",
        "group-has-[p:hover]:p-4",
    ];
    for input in battery {
        let compiled = compile(input, &Config::default()).expect("non-empty input compiles");
        let sheet = emit_css(input, &Config::default()).expect("non-empty input renders");
        assert_eq!(
            compiled.diagnostics, sheet.diagnostics,
            "the walks agree on {input}"
        );
    }
}

// ---------------------------------------------------------------------------
// M2: the parent-relative has-probes (group-has-* / peer-has-*)
// ---------------------------------------------------------------------------

#[test]
fn group_has_folds_to_the_parent_relative_probe() {
    // The v0.x oracle's own composition (`group-` wraps the inner
    // variant): the fold rides the parent tier, the label drops (the
    // group-hover precedent).
    assert_eq!(
        one_cond("group-has-focus:p-4"),
        Cond::GroupHas("*:focus"),
        "the parent-relative probe rides the parent tier"
    );
    assert_eq!(
        one_cond("peer-has-data-[slot=x]:p-4"),
        Cond::PeerHas("*[data-slot=\"x\"]")
    );
}

#[test]
fn group_has_label_form_folds_to_the_same_probe() {
    // `group-hover/menu` drops the label in the flat fold; the
    // group-has label form drops it the same way.
    assert_eq!(
        one_cond("group-has-focus/menu:p-4"),
        one_cond("group-has-focus:p-4"),
        "the label is the CSS arm's head decoration, not an IR slot"
    );
}

#[test]
fn parent_probe_outranks_the_element_probe_in_the_fold() {
    // The parent tier (3) outranks the element-own probe tier (2): the
    // dominant-condition fold keeps the parent-relative probe.
    assert_eq!(
        one_cond("data-open:group-has-focus:p-4"),
        Cond::GroupHas("*:focus"),
    );
}

#[test]
fn css_renders_the_group_has_head_forms() {
    // The arm's conjunctive parent form (the group-hover precedent):
    // `.group[/label]:has(...) .class`, the peer head the sibling `~`.
    let sheet =
        emit_css("group-has-focus:p-4", &Config::default()).expect("non-empty input renders");
    assert_eq!(
        sheet.rules[0].selector,
        ".group:has(*:focus) .group-has-focus\\:p-4"
    );

    let labelled =
        emit_css("group-has-focus/menu:p-4", &Config::default()).expect("non-empty input renders");
    assert_eq!(
        labelled.rules[0].selector, ".group\\/menu:has(*:focus) .group-has-focus\\/menu\\:p-4",
        "the label form escapes both class halves"
    );

    let peer =
        emit_css("peer-has-data-[slot=x]:p-4", &Config::default()).expect("non-empty renders");
    assert_eq!(
        peer.rules[0].selector,
        ".peer:has(*[data-slot=\"x\"]) ~ .peer-has-data-\\[slot\\=x\\]\\:p-4"
    );
}

#[test]
fn group_has_compound_inner_refuses_typed() {
    let compiled = compile("group-has-[p:hover]:p-4", &Config::default()).expect("compiles");
    assert_eq!(
        compiled.diagnostics[0].kind,
        DiagnosticKind::UnsupportedVariant {
            variant: "group-has-[p:hover]".to_string()
        }
    );
}

#[test]
fn interp_notes_the_parent_probes_too() {
    let compiled = compile("group-has-focus:p-4", &Config::default()).expect("compiles");
    let interped =
        interp(&compiled.program, &Env::default()).expect("no theme forms in the program");
    assert!(interped.decls.is_empty());
    assert_eq!(interped.notes.len(), 1, "{:?}", interped.notes);
    assert!(matches!(
        interped.notes[0],
        InterpNote::AttributeUnqueried { .. }
    ));
}

#[test]
fn static_form_emits_the_parent_probes() {
    let compiled = compile("group-has-focus:p-4", &Config::default()).expect("compiles");
    let text = compiled.emit_static();
    assert!(
        text.contains("aeolus_ir::Cond::GroupHas(\"*:focus\")"),
        "the parent probe emits as the constructor: {text}"
    );
}
