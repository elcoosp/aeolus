//! The wider v4 hint table's contracts (architecture plan §3.1's value
//! pass — the arbitrary-grammar hint line): `angle:`, `time:`,
//! `position:`, `family-name:`, `generic-name:`, `absolute-size:`,
//! `relative-size:` and `string:` lower onto the row kinds that carry
//! their shape, through the SAME typed pipeline as the landed
//! length/percentage/number/color/url/image/ratio hints. The
//! unit-grammar's time forms ride the free grammar too (the `deg`
//! precedent: the row's own number form carries the unit).
//!
//! The eligibility tables are the design's centre — each hint gates to
//! the rows whose slots can carry the shape (the `COLOUR_GROUPS`
//! audit pattern): an angle lowers on the degree rows (`rotate`,
//! `skew-x`, `skew-y`, the hue-rotate handles), a time on the
//! millisecond rows (`duration`, `delay`), a position on the
//! object-position row, a family on the font-family row, a size
//! keyword on the font-size row, a string on the content root — and
//! refuses TYPED everywhere else (the sibling-row skip + the run's
//! catch-all), never a silent drop, never a guessed unit.
//!
//! The `string:` hint lowers onto the NEW `Prop::Content` longhand —
//! the CSS `content` property the `content-["…"]` arbitrary spelling
//! writes (v4's own reading; the root's keyword half stays
//! `align-content`, the tw_merge oracle's group name). The v0.x plan
//! has no content slot, so the plan bridge reports it honestly (the
//! `grid-auto-flow` precedent), and the GPUI emit gate refuses it per
//! longhand (no native slot — the default tier).
//!
//! Written FIRST (the RED spec): the suite fails until the wider hint
//! table lowers — the value pass answers every landed hint with the
//! typed hint-mismatch boundary before the milestone.

use aeolus_compile::{
    Config, Env, PlanNoteKind, cn, compile, emit_css, emit_plan, emit_rust, keyword_texts,
    metamorphic_check, sort,
};
use aeolus_ir::{Prop, Val};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn config() -> Config {
    Config::default()
}

/// The one base declaration `class` lowers to (the hint families'
/// eligible rows are single-longhand rows).
fn base_decl_of(class: &str) -> (Prop, Val) {
    let compiled = compile(class, &config()).expect("non-empty input compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "{class}: unexpected diagnostics {:?}",
        compiled.diagnostics
    );
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

fn refused(class: &str) -> String {
    let compiled = compile(class, &config()).expect("the walk is total");
    assert!(
        !compiled.diagnostics.is_empty(),
        "{class}: expected a typed refusal, got none"
    );
    compiled.diagnostics[0].to_string()
}

/// The CSS sheet renders exactly one rule carrying `line`.
fn css_line_of(class: &str) -> String {
    let sheet = emit_css(class, &config()).expect("the CSS walk is total");
    assert!(
        sheet.diagnostics.is_empty(),
        "{class}: unexpected diagnostics {:?}",
        sheet.diagnostics
    );
    assert_eq!(sheet.rules.len(), 1, "{class}: one rule");
    sheet.rules[0].body.clone()
}

// ---------------------------------------------------------------------------
// 1. The `angle:` hint: the degree rows
// ---------------------------------------------------------------------------

#[test]
fn the_angle_hint_lowers_on_the_degree_rows() {
    // rotate/skew carry the deg number form — the hint's Num rides the
    // same slot the free `45deg` spelling lowers into.
    for (class, prop) in [
        ("rotate-[angle:45deg]", Prop::Rotate),
        ("skew-x-[angle:30deg]", Prop::SkewX),
        ("skew-y-[angle:-12deg]", Prop::SkewY),
    ] {
        let (p, val) = base_decl_of(class);
        assert_eq!(p, prop, "{class}");
        assert_eq!(val, Val::Num(expected_deg(class)), "{class}");
    }
}

fn expected_deg(class: &str) -> f32 {
    // The number between `angle:` and `deg`, sign included.
    let start = class.find("angle:").expect("hinted") + "angle:".len();
    let end = class.rfind("deg]").expect("deg form");
    class[start..end].parse().expect("finite deg")
}

#[test]
fn the_free_time_and_angle_forms_still_lower() {
    // The landed unit grammar's own spellings are untouched.
    let (p, val) = base_decl_of("rotate-[45deg]");
    assert_eq!(p, Prop::Rotate);
    assert_eq!(val, Val::Num(45.0));
}

#[test]
fn the_angle_hint_refuses_non_degree_spellings() {
    // The hint IS the type contract: rad/grad/turn and bare numbers
    // are not deg spellings — the typed hint mismatch, never a
    // conversion the IR cannot name.
    for class in [
        "rotate-[angle:0.5turn]",
        "rotate-[angle:1.2rad]",
        "rotate-[angle:50grad]",
        "rotate-[angle:45]",
    ] {
        let text = refused(class);
        assert!(text.contains("hint"), "{class}: {text}");
    }
}

#[test]
fn an_angle_on_a_non_degree_row_refuses() {
    // `z` takes numbers, not angles; no row of the z root carries a
    // degree form — the catch-all boundary, never a wrong lower.
    let text = refused("z-[angle:45deg]");
    assert!(!text.is_empty(), "{text}");
}

#[test]
fn the_angle_hint_css_arm_renders_deg() {
    let line = css_line_of("rotate-[angle:45deg]");
    assert!(line.contains("rotate: 45deg"), "{line}");
}

// ---------------------------------------------------------------------------
// 2. The `time:` hint: the millisecond rows
// ---------------------------------------------------------------------------

#[test]
fn the_time_hint_lowers_on_the_time_rows() {
    // The duration/delay slots are MILLISECONDS (the motion layer's
    // number form): `2s` bakes ×1000, `300ms` rides as-is.
    let (p, val) = base_decl_of("duration-[time:2s]");
    assert_eq!(p, Prop::TransitionDuration);
    assert_eq!(val, Val::Num(2000.0));

    let (p, val) = base_decl_of("delay-[time:300ms]");
    assert_eq!(p, Prop::TransitionDelay);
    assert_eq!(val, Val::Num(300.0));
}

#[test]
fn the_free_time_units_lower_on_the_time_rows() {
    // The deg precedent: the row's own number form carries the unit —
    // the free spelling lowers too (the motion battery's old refusal
    // retires for the landed semantics).
    let (p, val) = base_decl_of("duration-[2s]");
    assert_eq!(p, Prop::TransitionDuration);
    assert_eq!(val, Val::Num(2000.0));
}

#[test]
fn the_time_hint_refuses_non_time_spellings() {
    for class in [
        "duration-[time:2]",
        "duration-[time:2deg]",
        "delay-[time:fast]",
    ] {
        let text = refused(class);
        assert!(text.contains("hint"), "{class}: {text}");
    }
}

#[test]
fn a_time_on_a_non_time_row_refuses() {
    let text = refused("w-[time:2s]");
    assert!(!text.is_empty(), "{text}");
}

#[test]
fn the_time_css_arm_renders_ms() {
    let line = css_line_of("duration-[time:2s]");
    assert!(line.contains("transition-duration: 2000ms"), "{line}");
}

// ---------------------------------------------------------------------------
// 3. The `position:` hint: the object-position row
// ---------------------------------------------------------------------------

#[test]
fn the_position_hint_lowers_on_the_object_row() {
    let (p, val) = base_decl_of("object-[position:50%_25%]");
    assert_eq!(p, Prop::ObjectPosition);
    assert_eq!(val, Val::Verbatim("50% 25%"), "the underscore IS a space");

    let (p, val) = base_decl_of("object-[position:left_top]");
    assert_eq!(p, Prop::ObjectPosition);
    assert_eq!(val, Val::Verbatim("left top"));
}

#[test]
fn a_position_on_other_rows_refuses() {
    for class in ["w-[position:center]", "shadow-[position:center]"] {
        let text = refused(class);
        assert!(!text.is_empty(), "{class}: {text}");
    }
}

// ---------------------------------------------------------------------------
// 4. The `family-name:` / `generic-name:` hints: the font-family row
// ---------------------------------------------------------------------------

#[test]
fn the_family_hints_lower_on_the_font_family_row() {
    // Quoted family names keep their quotes (v4's own emission — the
    // family name is a CSS string); underscores decode to spaces.
    let (p, val) = base_decl_of("font-[family-name:\"Inter_Sans\"]");
    assert_eq!(p, Prop::FontFamily);
    assert_eq!(val, Val::Verbatim("\"Inter Sans\""));

    // The generic names ride the same slot (the CSS generic keywords).
    let (p, val) = base_decl_of("font-[generic-name:serif]");
    assert_eq!(p, Prop::FontFamily);
    assert_eq!(val, Val::Verbatim("serif"));
}

#[test]
fn the_family_css_arm_renders_the_name() {
    let line = css_line_of("font-[family-name:\"Inter_Sans\"]");
    assert!(line.contains("font-family: \"Inter Sans\""), "{line}");
}

#[test]
fn a_family_hint_on_the_weight_row_skips_to_the_family_row() {
    // The `font` root's weight sibling must NOT take the family hint —
    // the sibling-row skip; the family row lowers it.
    let (p, val) = base_decl_of("font-[family-name:\"Inter_Sans\"]");
    assert_eq!(p, Prop::FontFamily);
    assert!(matches!(val, Val::Verbatim(_)));
}

#[test]
fn a_family_hint_elsewhere_refuses() {
    let text = refused("w-[family-name:\"Inter\"]");
    assert!(!text.is_empty(), "{text}");
}

// ---------------------------------------------------------------------------
// 5. The `absolute-size:` / `relative-size:` hints: the font-size row
// ---------------------------------------------------------------------------

#[test]
fn the_size_keyword_hints_lower_on_the_font_size_row() {
    let (p, val) = base_decl_of("text-[absolute-size:larger]");
    assert_eq!(p, Prop::FontSize);
    assert_eq!(val, Val::Verbatim("larger"));

    let (p, val) = base_decl_of("text-[relative-size:smaller]");
    assert_eq!(p, Prop::FontSize);
    assert_eq!(val, Val::Verbatim("smaller"));
}

#[test]
fn a_size_keyword_hint_elsewhere_refuses() {
    let text = refused("w-[absolute-size:larger]");
    assert!(!text.is_empty(), "{text}");
}

// ---------------------------------------------------------------------------
// 6. The `string:` hint: the content root's CSS content property
// ---------------------------------------------------------------------------

#[test]
fn the_string_hint_lowers_onto_the_content_longhand() {
    // v4's own reading: `content-["…"]` writes the CSS `content`
    // property (the pseudo-element paint), NOT align-content.
    let (p, val) = base_decl_of("content-[\"hello_world\"]");
    assert_eq!(p, Prop::Content, "the new longhand");
    assert_eq!(val, Val::Verbatim("\"hello world\""), "quotes kept");
}

#[test]
fn the_content_css_arm_renders_the_string() {
    let line = css_line_of("content-[\"hello_world\"]");
    assert!(line.contains("content: \"hello world\""), "{line}");
}

#[test]
fn the_content_interp_answers_the_string() {
    let compiled = compile("content-[\"hello_world\"]", &config()).expect("compiles");
    let interped = compiled
        .interp(&Env::default())
        .expect("the base situation resolves");
    assert_eq!(interped.decls.len(), 1);
    assert_eq!(interped.decls[0].prop, Prop::Content);
    assert_eq!(interped.decls[0].val, Val::Verbatim("\"hello world\""));
}

#[test]
fn the_content_plan_bridge_reports_the_value_honestly() {
    // The v0.x plan has no content slot — the bridge's verbatim
    // short-circuit reports the CompositeValue note naming the
    // longhand (the slot left unset, never a silent zero; the plan
    // note's own contract).
    let sheet = emit_plan("content-[\"hello_world\"]", &config()).expect("compiles");
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    let note = sheet
        .notes
        .iter()
        .find(|note| matches!(note.kind, PlanNoteKind::CompositeValue { .. }))
        .expect("the bridge reports the content value");
    assert!(
        matches!(
            &note.kind,
            PlanNoteKind::CompositeValue {
                prop: Prop::Content,
                ..
            }
        ),
        "the note names the content longhand: {:?}",
        note.kind
    );
}

#[test]
fn the_content_keyword_half_stays_align_content() {
    // `content-center` is the align-content keyword (the group the
    // tw_merge oracle names) — the string form did not move it.
    let (p, _) = base_decl_of("content-center");
    assert_eq!(p, Prop::AlignContent);
}

#[test]
fn a_string_on_other_rows_refuses() {
    let text = refused("w-[string:\"x\"]");
    assert!(!text.is_empty(), "{text}");
}

// ---------------------------------------------------------------------------
// 7. The merge, the sort, the metamorphic triangle, the freezes
// ---------------------------------------------------------------------------

#[test]
fn the_merge_folds_the_hinted_forms_per_longhand() {
    // Same condition, same longhand — the LAST occurrence wins, the
    // same fold every family takes.
    let a = compile("content-[\"first\"]", &config()).expect("compiles");
    let b = compile("content-[\"second\"]", &config()).expect("compiles");
    let merged = cn(&[&a, &b]).expect("the merge is total");
    let layers = merged.program.layers;
    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0].decls.len(), 1);
    assert_eq!(layers[0].decls[0].val, Val::Verbatim("\"second\""));
}

#[test]
fn the_hinted_classes_sort_by_the_registry_order() {
    let sorted =
        sort("rotate-[angle:45deg] duration-[time:2s] content-[\"x\"]").expect("the walk is total");
    assert!(sorted.diagnostics.is_empty(), "{:?}", sorted.diagnostics);
    let raws: Vec<&str> = sorted.classes.iter().map(|c| c.raw.as_str()).collect();
    assert!(raws.contains(&"rotate-[angle:45deg]"), "{raws:?}");
    assert!(raws.contains(&"duration-[time:2s]"), "{raws:?}");
    assert!(raws.contains(&"content-[\"x\"]"), "{raws:?}");
    // Every hinted class is registry-known (the rows carry them).
    assert!(
        sorted.classes.iter().all(|c| c.known),
        "hinted classes are known: {:?}",
        sorted
            .classes
            .iter()
            .map(|c| (c.raw.as_str(), c.known))
            .collect::<Vec<_>>()
    );
}

#[test]
fn the_metamorphic_triangle_agrees_on_the_hinted_forms() {
    for class in [
        "rotate-[angle:45deg]",
        "duration-[time:2s]",
        "object-[position:50%_25%]",
        "font-[family-name:\"Inter_Sans\"]",
        "text-[absolute-size:larger]",
        "content-[\"hello_world\"]",
    ] {
        let report = metamorphic_check(class, &config()).expect("the walk is total");
        for situation in &report.situations {
            assert!(
                situation.agree,
                "{class}: {:?} — {:#?}",
                situation.mismatches, situation.label
            );
            assert!(
                situation.interp_error.is_none(),
                "{class}: {:?}",
                situation.interp_error
            );
        }
        assert!(
            report.agree(),
            "{class}: the triangle disagrees (diagnostics_equal={}, compositional={})",
            report.diagnostics_equal,
            report.compositional
        );
    }
}

#[test]
fn the_static_and_rust_tokens_arms_freeze_the_hinted_slots() {
    // The static-data arm: one compilable `static` item over the
    // landed forms.
    let compiled = compile("duration-[time:2s] content-[\"x\"]", &config()).expect("compiles");
    let text = compiled.emit_static();
    assert!(text.contains("TransitionDuration"), "{text}");
    assert!(text.contains("Content"), "{text}");
    assert!(text.contains("Verbatim(\"\\\"x\\\"\")"), "{text}");

    // The rust-tokens arm: one embeddable expression per class.
    let sheet = emit_rust("duration-[time:2s]", &config()).expect("the walk is total");
    let rendered = sheet.to_string();
    assert!(rendered.contains("TransitionDuration"), "{rendered}");
}

#[test]
fn the_keyword_annotation_seam_reads_the_font_row() {
    // The font-family row grew an arbitrary arm — the keyword texts
    // for the KEYWORD spellings must still answer (the AnyOf flatten).
    let texts = keyword_texts("font-sans", &config()).expect("the walk is total");
    assert_eq!(texts.len(), 1);
    assert!(
        !texts[0].keywords.is_empty(),
        "font-sans carries its row text"
    );
}
