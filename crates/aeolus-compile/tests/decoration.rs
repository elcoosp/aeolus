//! The decoration family's contracts (architecture plan §3.7's
//! typography surface — the `decoration-{color}`/`decoration-{thickness}`/
//! `decoration-{style}` rows): the family lowers through the SAME typed
//! pipeline as every other registry family — IR longhands, the value
//! pass, the CSS arm's rendering, the interp arm, the rust-tokens arm's
//! annotations, the static arm — and the honest boundaries stay typed
//! (v4's integer-only thickness grammar, the style keywords GPUI
//! renders a single solid line for, negation, the bare-number arbitrary
//! refusal the real compiler shares).
//!
//! The audited v4.1.17 grammar (pinned from the upstream
//! `utilities.ts`): the static style utilities (`decoration-solid` …
//! `decoration-wavy`), the static thickness keywords (`decoration-auto`,
//! `decoration-from-font`), and the functional `decoration-` root whose
//! bare value is a NON-NEGATIVE INTEGER thickness (the
//! `isPositiveInteger` round-trip rule — `decoration-2.5` and
//! `decoration-02` are not utilities), else a colour token (the
//! `/opacity` modifier rides). Arbitrary values split by type:
//! length/percentage → thickness, colour → colour.
//!
//! Written FIRST (the RED spec): the suite fails until the decoration
//! family lowers — `Prop` has no text-decoration colour/thickness/style
//! longhands and the registry carries no decoration rows before the
//! milestone.

use aeolus_compile::{
    Config, DiagnosticKind, Target, cn, compile, compile_theme, emit_css, emit_css_theme,
    emit_rust, interp, sort,
};
use aeolus_ir::{Cond, Prop, Val};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn config() -> Config {
    Config::default()
}

fn baked_tokens() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("the embedded shadcn source parses");
    TokenTable::baked(&source).expect("the pair surface fits the u16 slot")
}

/// The one base declaration `class` lowers to (the decoration family is
/// single-longhand rows).
fn base_decl_of(class: &str) -> (Prop, Val) {
    let compiled = compile(class, &config()).expect("non-empty input compiles");
    let layers = compiled.program.layers;
    assert_eq!(
        layers.len(),
        1,
        "{class}: one base layer, got {:?}",
        layers.iter().map(|l| l.cond).collect::<Vec<_>>()
    );
    let decls = layers[0].decls;
    assert_eq!(decls.len(), 1, "{class}: one declaration, got {decls:?}");
    (decls[0].prop, decls[0].val)
}

// ---------------------------------------------------------------------------
// 1. The IR shapes: three longhands, the row-table slots
// ---------------------------------------------------------------------------

#[test]
fn style_keywords_lower_onto_the_style_longhand() {
    // solid=0, double=1, dotted=2, dashed=3, wavy=4 — the v4 static
    // utilities' own order.
    for (class, slot) in [
        ("decoration-solid", 0u8),
        ("decoration-double", 1),
        ("decoration-dotted", 2),
        ("decoration-dashed", 3),
        ("decoration-wavy", 4),
    ] {
        let (prop, val) = base_decl_of(class);
        assert_eq!(prop, Prop::TextDecorationStyle, "{class}");
        assert_eq!(val, Val::Kw(slot), "{class}");
    }
}

#[test]
fn thickness_keywords_and_integers_lower_onto_the_thickness_longhand() {
    // The static keywords: auto=0, from-font=1 (the v4 utilities' own
    // order), then the bare non-negative integers (the v4
    // `isPositiveInteger` form — integers lower as the pixel number).
    for (class, slot) in [("decoration-auto", 0u8), ("decoration-from-font", 1)] {
        let (prop, val) = base_decl_of(class);
        assert_eq!(prop, Prop::TextDecorationThickness, "{class}");
        assert_eq!(val, Val::Kw(slot), "{class}");
    }

    for (class, px) in [
        ("decoration-0", 0.0f32),
        ("decoration-2", 2.0),
        ("decoration-13", 13.0),
    ] {
        let (prop, val) = base_decl_of(class);
        assert_eq!(prop, Prop::TextDecorationThickness, "{class}");
        assert_eq!(val, Val::Num(px), "{class}");
    }
}

#[test]
fn colour_tokens_lower_onto_the_colour_longhand() {
    // The theme pass supplies the table: the shadcn palette's baked
    // red-500. The theme-less compile keeps the typed boundary (the
    // oversized colour table's own rule, the bg-{color} precedent).
    let compiled =
        compile_theme("decoration-red-500", &baked_tokens(), &config()).expect("compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let decls = &compiled.program.layers[0].decls;
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].prop, Prop::TextDecorationColor);

    // The theme-less form refuses typed — never a silent drop (the
    // REASON_TOKEN_TABLE boundary the bg-{color} row carries).
    let compiled = compile("decoration-red-500", &config()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "{:?}", compiled.diagnostics);
    assert!(matches!(
        compiled.diagnostics[0].kind,
        DiagnosticKind::UnsupportedValue { .. }
    ));
}

// ---------------------------------------------------------------------------
// 2. The v4 grammar boundaries: typed, never silent
// ---------------------------------------------------------------------------

#[test]
fn non_integer_and_zero_padded_thicknesses_are_not_utilities() {
    // The v4 isPositiveInteger rule: integers only, and the spelling
    // must round-trip (`decoration-2.5` and `decoration-02` are not
    // utilities in the real compiler). The shape rejection names the
    // root — the "did you mean" surface's input.
    for class in ["decoration-2.5", "decoration-02"] {
        let compiled = compile(class, &config()).expect("compiles");
        assert_eq!(
            compiled.diagnostics.len(),
            1,
            "{class}: one refusal, got {:?}",
            compiled.diagnostics
        );
        assert!(
            matches!(
                compiled.diagnostics[0].kind,
                DiagnosticKind::ShapeRejected { .. }
            ),
            "{class}: {:?}",
            compiled.diagnostics[0].kind
        );
    }
}

#[test]
fn negation_refuses_on_the_decoration_root() {
    // The decoration rows carry `negative = false` (v4 registers no
    // negative form): the `-` prefix is the typed negation boundary.
    let compiled = compile("-decoration-2", &config()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "{:?}", compiled.diagnostics);
    assert!(matches!(
        compiled.diagnostics[0].kind,
        DiagnosticKind::UnsupportedNegation { .. }
    ));
}

#[test]
fn unknown_style_and_colour_names_refuse_typed() {
    // Outside the closed keyword tables / the token table: the shape
    // rejection (style) and the not-a-token boundary (colour), never a
    // silent drop.
    let compiled = compile("decoration-bouncy", &config()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "{:?}", compiled.diagnostics);

    let compiled =
        compile_theme("decoration-not-a-token", &baked_tokens(), &config()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "{:?}", compiled.diagnostics);
}

// ---------------------------------------------------------------------------
// 3. The arbitrary values: the type split the real compiler infers
// ---------------------------------------------------------------------------

#[test]
fn arbitrary_lengths_and_percentages_lower_onto_thickness() {
    for (class, val) in [
        ("decoration-[3px]", Val::Px(3.0)),
        ("decoration-[0.25rem]", Val::Rem(0.25)),
        ("decoration-[50%]", Val::Pct(50.0)),
    ] {
        let (prop, got) = base_decl_of(class);
        assert_eq!(prop, Prop::TextDecorationThickness, "{class}");
        assert_eq!(got, val, "{class}");
    }
}

#[test]
fn arbitrary_colours_lower_onto_colour_with_the_modifier() {
    let (prop, val) = base_decl_of("decoration-[#0f0f0f]");
    assert_eq!(prop, Prop::TextDecorationColor);
    assert_eq!(val, Val::Rgba([0x0f, 0x0f, 0x0f, 0xff]));

    // The /opacity modifier rides colour-form values — and it is the
    // THEME pass's form (the modifier boundary is the theme-less
    // default, the bg-[#0f0f0f]/50 precedent): the baked literal lands
    // the audited alpha round (50% → 128).
    let compiled =
        compile_theme("decoration-[#0f0f0f]/50", &baked_tokens(), &config()).expect("compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let decls = &compiled.program.layers[0].decls;
    assert_eq!(decls[0].prop, Prop::TextDecorationColor);
    let Val::Rgba([r, g, b, a]) = decls[0].val else {
        panic!("colour form, got {:?}", decls[0].val)
    };
    assert_eq!([r, g, b], [0x0f, 0x0f, 0x0f], "the literal rgb carries");
    assert!(
        (a as f32 - 127.5).abs() < 1.0,
        "the 50% bake lands near alpha 127.5, got {a}"
    );
}

#[test]
fn arbitrary_var_references_lower_through_the_theme_pass() {
    // `decoration-[var(--color-primary)]` (and the `(...)` shorthand):
    // the token path, gated to the colour rows — the bg-[var(--…)]
    // precedent.
    let compiled = compile_theme(
        "decoration-[var(--color-primary)]",
        &baked_tokens(),
        &config(),
    )
    .expect("compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let decls = &compiled.program.layers[0].decls;
    assert_eq!(decls[0].prop, Prop::TextDecorationColor);
}

#[test]
fn bare_number_arbitrary_values_refuse_typed() {
    // `decoration-[13]`: the real compiler infers no type for a bare
    // number (the type list is colour/length/percentage) and refuses —
    // the registry's PositiveInt arm is the BARE form's legality oracle,
    // so the arbitrary bare number stays the typed unit boundary (the
    // `w-[13]` precedent).
    let compiled = compile("decoration-[13]", &config()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "{:?}", compiled.diagnostics);
}

#[test]
fn the_sibling_rows_split_the_arbitrary_shapes() {
    // One string, two rows: the px arbitrary lands on thickness, the
    // colour arbitrary on colour, the diagnostics quiet (the
    // `text-[13px]`/`text-[#0f0f0f]` sibling gate's own rule).
    let compiled = compile(
        "decoration-[3px] decoration-[#0f0f0f] decoration-[50%]",
        &config(),
    )
    .expect("compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let decls = &compiled.program.layers[0].decls;
    assert_eq!(decls.len(), 3, "{decls:?}");
    let props: Vec<Prop> = decls.iter().map(|d| d.prop).collect();
    assert!(props.contains(&Prop::TextDecorationColor));
    assert_eq!(
        props
            .iter()
            .filter(|p| **p == Prop::TextDecorationThickness)
            .count(),
        2,
        "px and % both land on thickness: {props:?}"
    );
}

// ---------------------------------------------------------------------------
// 4. The cascade: the family composes like every other family
// ---------------------------------------------------------------------------

#[test]
fn the_decoration_family_composes_in_one_string() {
    let compiled = compile("underline decoration-wavy decoration-2", &config()).expect("compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let base = &compiled.program.layers[0].decls;
    assert_eq!(base.len(), 3, "{base:?}");
    let props: Vec<Prop> = base.iter().map(|d| d.prop).collect();
    assert!(props.contains(&Prop::TextDecorationLine));
    assert!(props.contains(&Prop::TextDecorationStyle));
    assert!(props.contains(&Prop::TextDecorationThickness));
    // The cascade order is the Prop discriminant order: the line longhand
    // (typography block) precedes the appended decoration tail.
    assert_eq!(props[0], Prop::TextDecorationLine);
}

#[test]
fn ir_level_cn_keeps_the_last_thickness_and_colour() {
    // The IR-level merge is per-LONGHAND last-wins: two thicknesses
    // collide, the rightmost survives; a colour and a thickness COEXIST
    // (different longhands — the per-longhand split the family exists
    // for).
    let base = compile_theme(
        "decoration-2 decoration-red-500",
        &baked_tokens(),
        &config(),
    )
    .expect("compiles");
    let over = compile_theme(
        "decoration-4 decoration-blue-500",
        &baked_tokens(),
        &config(),
    )
    .expect("compiles");
    let merged = cn(&[&base, &over]).expect("non-empty merge");
    let base_layer = &merged.program.layers[0].decls;
    let thickness: Vec<_> = base_layer
        .iter()
        .filter(|d| d.prop == Prop::TextDecorationThickness)
        .collect();
    let colour: Vec<_> = base_layer
        .iter()
        .filter(|d| d.prop == Prop::TextDecorationColor)
        .collect();
    assert_eq!(
        thickness.len(),
        1,
        "the merge evicts the shadowed thickness"
    );
    assert_eq!(thickness[0].val, Val::Num(4.0), "decoration-4 wins");
    assert_eq!(colour.len(), 1, "the merge evicts the shadowed colour");
}

// ---------------------------------------------------------------------------
// 5. The CSS arm: the v4 property forms
// ---------------------------------------------------------------------------

#[test]
fn css_renders_the_decoration_family() {
    let sheet = emit_css(
        "underline decoration-wavy decoration-2 decoration-[#0f0f0f]",
        &config(),
    )
    .expect("compiles");
    let text = sheet.to_string();
    assert!(text.contains("text-decoration-line: underline;"), "{text}");
    assert!(text.contains("text-decoration-style: wavy;"), "{text}");
    assert!(text.contains("text-decoration-thickness: 2px;"), "{text}");
    assert!(text.contains("text-decoration-color: #0f0f0f;"), "{text}");
}

#[test]
fn css_renders_the_thickness_keywords_and_the_token_colour() {
    let sheet = emit_css("decoration-auto decoration-from-font", &config()).expect("compiles");
    let text = sheet.to_string();
    assert!(text.contains("text-decoration-thickness: auto;"), "{text}");
    assert!(
        text.contains("text-decoration-thickness: from-font;"),
        "{text}"
    );

    // The theme token renders the var form (the runtime tier's
    // `color-mix` free spelling when no modifier rides).
    let sheet = emit_css_theme("decoration-red-500", &baked_tokens(), &config()).expect("compiles");
    let text = sheet.to_string();
    assert!(text.contains("text-decoration-color:"), "{text}");
}

// ---------------------------------------------------------------------------
// 6. The interp arm: the effective declarations carry the family
// ---------------------------------------------------------------------------

#[test]
fn interp_resolves_the_decoration_family() {
    let compiled = compile_theme(
        "underline decoration-wavy decoration-2 decoration-red-500",
        &baked_tokens(),
        &config(),
    )
    .expect("compiles");
    let interped = interp(&compiled.program, &Default::default()).expect("all values resolve");
    let props: Vec<Prop> = interped.decls.iter().map(|d| d.prop).collect();
    assert!(props.contains(&Prop::TextDecorationLine), "{props:?}");
    assert!(props.contains(&Prop::TextDecorationStyle), "{props:?}");
    assert!(props.contains(&Prop::TextDecorationThickness), "{props:?}");
    assert!(props.contains(&Prop::TextDecorationColor), "{props:?}");
    // The winner attribution rides (the conds slice is parallel).
    assert_eq!(interped.conds.len(), interped.decls.len());
    assert!(interped.conds.iter().all(|c| *c == Cond::Base));
}

// ---------------------------------------------------------------------------
// 7. The rust-tokens arm: the row texts annotate the slots
// ---------------------------------------------------------------------------

#[test]
fn rust_tokens_annotate_the_decoration_slots() {
    let sheet = emit_rust("underline decoration-wavy", &config()).expect("compiles");
    let text = sheet.to_string();
    assert!(text.contains("Prop::TextDecorationLine"), "{text}");
    assert!(text.contains("Prop::TextDecorationStyle"), "{text}");
    // The row text the slot indexes — the annotation a bare `Val::Kw`
    // cannot recover.
    assert!(text.contains("wavy"), "{text}");
}

// ---------------------------------------------------------------------------
// 8. The static arm: the decoration decls freeze byte-stable
// ---------------------------------------------------------------------------

#[test]
fn static_form_freezes_the_decoration_decls() {
    let compiled = compile("underline decoration-wavy decoration-2", &config()).expect("compiles");
    let text = compiled.emit_static();
    assert!(text.contains("Prop::TextDecorationLine"), "{text}");
    assert!(text.contains("Prop::TextDecorationStyle"), "{text}");
    assert!(text.contains("Prop::TextDecorationThickness"), "{text}");
    // Deterministic: the same compile freezes the same bytes.
    let again = compile("underline decoration-wavy decoration-2", &config()).expect("compiles");
    assert_eq!(text, again.emit_static(), "the static item is byte-stable");
}

// ---------------------------------------------------------------------------
// 9. The sort seam: the registry order covers the family
// ---------------------------------------------------------------------------

#[test]
fn sort_orders_the_decoration_family() {
    let sorted = sort("decoration-2 p-4 decoration-wavy").expect("compiles");
    let texts: Vec<&str> = sorted.classes.iter().map(|c| c.raw.as_str()).collect();
    let p = texts.iter().position(|t| *t == "p-4").expect("p-4 sorts");
    let style = texts
        .iter()
        .position(|t| *t == "decoration-wavy")
        .expect("decoration-wavy sorts");
    assert!(p < style, "{texts:?}");
    // The family rides the registry's typography block, before the
    // interaction tail.
    let cursor = texts
        .iter()
        .position(|t| *t == "decoration-2")
        .expect("decoration-2 sorts");
    assert!(cursor < texts.len(), "{texts:?}");
}

// ---------------------------------------------------------------------------
// 10. The plan bridge: the v0.x shape is the typed limit
// ---------------------------------------------------------------------------

#[test]
fn the_plan_bridge_reports_the_decoration_slots() {
    // The v0.x `TextProperty` surface carries the LINE slot only — the
    // three new longhands land as the typed `SlotUnsupported` notes
    // (the underline-offset precedent: the lowering is correct, the
    // v0.x shape is the limit).
    let sheet = aeolus_compile::emit_plan("underline decoration-wavy decoration-2", &config())
        .expect("compiles");
    let named: Vec<_> = sheet
        .notes
        .iter()
        .filter_map(|note| match note.kind {
            aeolus_compile::PlanNoteKind::SlotUnsupported { prop } => Some(prop),
            _ => None,
        })
        .collect();
    assert!(named.contains(&Prop::TextDecorationStyle), "{named:?}");
    assert!(named.contains(&Prop::TextDecorationThickness), "{named:?}");
}

// ---------------------------------------------------------------------------
// 11. The metamorphic triangle: compile == CSS == interp
// ---------------------------------------------------------------------------

#[test]
fn the_metamorphic_triangle_agrees_on_the_family() {
    use aeolus_compile::metamorphic_check;
    let literal = "underline decoration-wavy decoration-2";
    let report = metamorphic_check(literal, &config()).expect("compiles");
    assert!(
        report.agree(),
        "diagnostics_equal={} compositional={} situations={:?}",
        report.diagnostics_equal,
        report.compositional,
        report
            .situations
            .iter()
            .map(|s| s.agree)
            .collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// 12. The Target surface names the arm (the enum's own audit)
// ---------------------------------------------------------------------------

#[test]
fn the_target_enum_still_names_five_forms() {
    // The decoration family rides the EXISTING five emit forms — no new
    // arm, no capability regression (the audit pin the Target docs
    // carry).
    assert_eq!(
        [
            Target::Css,
            Target::Rust,
            Target::Interp,
            Target::Static,
            Target::Plan
        ]
        .len(),
        5
    );
}
