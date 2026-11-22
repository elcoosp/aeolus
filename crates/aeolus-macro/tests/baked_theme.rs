//! The macro's first integration battery: the baked-theme seam (plan
//! §3.4's T0/T1 literals at expansion time) and the backend-neutral
//! static front door.
//!
//! Every expansion in this file compiles THIS crate's own fixture
//! (`tests/fixtures/app.css` — read at expansion time relative to this
//! crate's `CARGO_MANIFEST_DIR`) through the real pipeline; the runtime
//! half re-compiles the same CSS through [`aeolus_compile`]'s public
//! face and pins byte-equality of the frozen layers AND the keyword
//! slices. The whole suite sharing one file scope is itself a pin: each
//! themed expansion carries its `include_str!` rebuild anchor as an
//! unnamed `const _` item — a named anchor would collide across the
//! expansions and the suite would not compile.
//!
//! ```sh
//! cargo test -p aeolus-macro
//! ```

use aeolus_compile::{
    Config, cn as merge_cn, compile_theme, emit_static, keyword_alignment, keyword_texts,
};
use aeolus_ir::{Cond, Prop, Val};
use aeolus_macro::{cn_static, tw, tw_static, variants, variants_static};
use aeolus_theme::{ThemeSource, TokenTable};

use std::collections::HashMap;

/// The fixture, read the runtime half's way (the same bytes the macro's
/// expansion-time read picks up).
const FIXTURE: &str = include_str!("fixtures/app.css");

/// The full surface: layout, spacing, a T1 pair token, a T0 literal.
const FULL: &str = "flex items-center gap-2 p-4 bg-primary text-primary-foreground rounded-lg";

/// The paired string: both tokens carry `.dark` overrides — the T1
/// pair surface (plan §3.4: `[Rgba; 2]` const pairs).
const PAIRED: &str = "bg-primary text-primary-foreground";

// The closed-world tables the variants battery pins. `variants_static!`
// has NO GPUI gate — `blur-md` is the documented tw!/variants! refusal
// (the filter family's emitter verdict), frozen here without one.
variants_static! {
    pub Effect {
        base: "p-4",
        variant: { blur: "blur-md", none: "shadow-none" },
    }
}

// The themed GPUI-gated table: the `theme:` argument rides the same
// grammar, the audit gate stays.
variants! {
    theme: "tests/fixtures/app.css",
    pub Tag {
        base: "bg-primary",
        variant: { solid: "text-primary-foreground" },
    }
}

/// The runtime half of the differential: the same walk the macro's
/// themed bake performs, through the pipeline's public face.
fn runtime_program(css: &str, classes: &str) -> (String, Vec<&'static str>) {
    let source = ThemeSource::from_app_css(css).expect("the fixture CSS parses");
    let table = TokenTable::baked(&source).expect("the fixture table bakes");
    let config = Config::default();
    let compiled = compile_theme(classes, &table, &config).expect("the class string lowers");
    assert!(
        compiled.diagnostics.is_empty(),
        "the fixture class string must lower clean: {:?}",
        compiled.diagnostics
    );
    let annotations = keyword_texts(classes, &config).expect("the class string annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    (emit_static(&compiled.program), keywords)
}

/// The runtime half for merged lists: the same `cn` merge plus the
/// last-occurrence keyword fold (rightmost wins per `(cond, prop)`).
fn runtime_merged(strings: &[&str]) -> (String, Vec<&'static str>) {
    let source = ThemeSource::from_app_css(FIXTURE).expect("the fixture CSS parses");
    let table = TokenTable::baked(&source).expect("the fixture table bakes");
    let config = Config::default();
    let mut compiled = Vec::new();
    let mut classes = Vec::new();
    for string in strings {
        let one = compile_theme(string, &table, &config).expect("each list lowers");
        assert!(
            one.diagnostics.is_empty(),
            "each list must lower clean: {:?}",
            one.diagnostics
        );
        classes.extend(keyword_texts(string, &config).expect("each list annotates"));
        compiled.push(one);
    }
    let refs: Vec<&aeolus_compile::Compiled> = compiled.iter().collect();
    let merged = merge_cn(&refs).expect("the merge succeeds");
    let mut last: HashMap<(Cond, Prop), &'static str> = HashMap::new();
    for class in &classes {
        for &(prop, text) in &class.keywords {
            last.insert((class.cond, prop), text);
        }
    }
    let mut keywords = Vec::new();
    for layer in merged.program.layers {
        for decl in layer.decls {
            keywords.push(last.get(&(layer.cond, decl.prop)).copied().unwrap_or(""));
        }
    }
    (emit_static(&merged.program), keywords)
}

/// The fixture table, for the literal-form pins.
fn fixture_table() -> TokenTable {
    let source = ThemeSource::from_app_css(FIXTURE).expect("the fixture CSS parses");
    TokenTable::baked(&source).expect("the fixture table bakes")
}

/// The background declaration of a single-class program, for the
/// literal-form pins.
fn background_of(program: aeolus_rt::StaticProgram) -> Val {
    let decls: Vec<Val> = program
        .program()
        .layers
        .iter()
        .flat_map(|layer| layer.decls.iter())
        .filter(|decl| decl.prop == Prop::Background)
        .map(|decl| decl.val)
        .collect();
    assert_eq!(decls.len(), 1, "one background declaration, got {decls:?}");
    decls[0]
}

#[test]
fn the_themed_bake_matches_the_pipeline_under_the_same_css() {
    let baked = tw_static!(theme: "tests/fixtures/app.css", "flex items-center gap-2 p-4 bg-primary text-primary-foreground rounded-lg");
    let (expected_static, expected_keywords) = runtime_program(FIXTURE, FULL);
    assert_eq!(
        emit_static(&baked.program()),
        expected_static,
        "the themed bake's layers must be byte-equal to the pipeline's"
    );
    assert_eq!(
        baked.keywords,
        expected_keywords.as_slice(),
        "the themed bake's keyword slices must be byte-equal to the pipeline's"
    );
}

#[test]
fn the_no_arg_bake_stays_the_embedded_shadcn_bake() {
    let classes = "p-4 bg-primary text-primary-foreground rounded-md w-1/2 h-9 font-medium \
                   gap-2 disabled:opacity-50 hover:bg-accent";
    let baked = tw_static!(
        "p-4 bg-primary text-primary-foreground rounded-md w-1/2 h-9 \
                            font-medium gap-2 disabled:opacity-50 hover:bg-accent"
    );
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::baked(&source).expect("the baked shadcn table builds");
    let config = Config::default();
    let compiled = compile_theme(classes, &table, &config).expect("the class string lowers");
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let annotations = keyword_texts(classes, &config).expect("the class string annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    assert_eq!(
        emit_static(&baked.program()),
        emit_static(&compiled.program),
        "the no-arg freeze must stay the embedded shadcn bake (the differential pin)"
    );
    assert_eq!(baked.keywords, keywords.as_slice());
}

#[test]
fn the_dark_pair_bakes_a_mode_pair_literal() {
    let baked = tw_static!(theme: "tests/fixtures/app.css", "bg-primary");
    let val = background_of(baked);
    let Val::ModePair(slot) = val else {
        panic!("the dark-carrying token must bake a ModePair, got {val:?}");
    };
    let table = fixture_table();
    let (light, dark) = table.pair_at(slot).expect("the pair slot resolves");
    assert_eq!(light, [14, 165, 233, 255], "the fixture's light #0ea5e9");
    assert_eq!(dark, [239, 68, 68, 255], "the fixture's dark #ef4444");
}

#[test]
fn the_opacity_modifier_premultiplies_a_baked_literal() {
    let baked = tw_static!(theme: "tests/fixtures/app.css", "bg-accent/50");
    let val = background_of(baked);
    let Val::Rgba(_) = val else {
        panic!("the light-only token's /50 must bake a premultiplied literal, got {val:?}");
    };
    let table = fixture_table();
    let id = table.id("accent").expect("the fixture declares accent");
    let expected = table.resolve(id, Some(0.5)).expect("the literal resolves");
    assert_eq!(val, expected, "the bake must ride the table's own math");
}

#[test]
fn the_themed_bake_carries_no_runtime_tokens() {
    let baked = tw_static!(theme: "tests/fixtures/app.css", "flex items-center gap-2 p-4 bg-primary text-primary-foreground rounded-lg");
    for layer in baked.program().layers {
        for decl in layer.decls {
            assert!(
                !matches!(decl.val, Val::Tok { .. }),
                "a baked theme freezes T0/T1 literals — no T2 Tok slot may ride: {:?}",
                decl.val
            );
        }
    }
}

#[test]
fn the_themed_gpui_lane_bakes_the_same_layers() {
    // The GPUI-gated entry with a theme argument: the same themed bake,
    // the audit gate still riding (these classes express fine on GPUI).
    let baked = tw!(theme: "tests/fixtures/app.css", "bg-primary text-primary-foreground");
    let (expected_static, expected_keywords) = runtime_program(FIXTURE, PAIRED);
    assert_eq!(emit_static(&baked.program()), expected_static);
    assert_eq!(baked.keywords, expected_keywords.as_slice());
}

#[test]
fn the_cn_static_merge_freezes_the_merged_program() {
    let baked = cn_static!(theme: "tests/fixtures/app.css", "p-4", "px-8 bg-accent/50");
    let (expected_static, expected_keywords) = runtime_merged(&["p-4", "px-8 bg-accent/50"]);
    assert_eq!(
        emit_static(&baked.program()),
        expected_static,
        "the merged themed freeze must be byte-equal to the pipeline's cn()"
    );
    assert_eq!(baked.keywords, expected_keywords.as_slice());
}

#[test]
fn the_variants_static_table_freezes_without_the_gate() {
    let program = Effect::style(EffectVariant::Blur);
    assert!(
        !program.program().layers.is_empty(),
        "blur-md freezes on the neutral lane — the gate's refusal is tw!'s, not the freeze's"
    );
    // The plain arm stays the ordinary bake (the pipeline's own walk).
    let (expected_static, _) = runtime_program(FIXTURE, "p-4 shadow-none");
    assert_eq!(
        emit_static(&Effect::style(EffectVariant::None).program()),
        expected_static,
        "the non-refused arm must stay the pipeline's own bake"
    );
}

#[test]
fn the_themed_variants_table_bakes_the_theme() {
    let program = Tag::style(TagVariant::Solid);
    let (expected_static, _) = runtime_program(FIXTURE, "bg-primary text-primary-foreground");
    assert_eq!(
        emit_static(&program.program()),
        expected_static,
        "the themed variants table bakes the fixture's own tokens"
    );
}
