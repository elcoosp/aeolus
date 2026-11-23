//! The M3 differential: the closed-world CONST tables
//! (`static_tables`, the `variants_static!` expansion) must produce
//! byte-identical programs to the pre-migration runtime-interned path
//! (`program`, the RefCell `Combo` handle) for EVERY closed-world
//! combo — layers, keyword row texts and pointer identity. The §10
//! headline claim pins here at EXPANSION time: the static arms return
//! the same `&'static` slices every call (a call is the dispatch, not
//! a compile and not a first-call intern).

use aeolus_compile::{Config, compile_theme, keyword_alignment, keyword_texts};
use aeolus_rt::StaticProgram;
use aeolus_shadcn::classes::{AlertVariant, BadgeVariant, ButtonSize, ButtonVariant};
use aeolus_shadcn::program::{AlertPrograms, BadgePrograms, BasePrograms, ButtonPrograms};
use aeolus_shadcn::static_tables::{Alert, Badge, Base, Button};
use aeolus_theme::{ThemeSource, TokenTable};

fn shadcn_table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the baked shadcn table builds")
}

/// The concatenation oracle: a fresh compile of the joined class
/// string — the same walk the macro's arm assembly runs. The static
/// arms pin THIS byte shape (the macro grammar's arm assembly joins
/// base + variant + size; the `cn()` merge is the runtime handle's
/// composition lane). The cascade's last-wins makes the two program
/// forms style-identical — pinned beside by the walk-identity test.
fn concat_bytes(classes: &str, tokens: &TokenTable) -> StaticProgram {
    let config = Config::default();
    let compiled = compile_theme(classes, tokens, &config).expect("the shipped v4 table compiles");
    let annotations =
        keyword_texts(classes, &config).expect("the shipped v4 table carries keyword texts");
    let keywords = keyword_alignment(&compiled, &annotations);
    StaticProgram::new(
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    )
}

/// The variants' class strings the runtime handle builds (the
/// `classes` module's own composition — the verbatim tables).
fn button_classes(variant: ButtonVariant, size: ButtonSize) -> String {
    aeolus_shadcn::classes::button_classes(variant, size)
}

#[test]
fn every_button_combo_is_byte_identical_across_both_paths() {
    let tokens = shadcn_table();
    let runtime = ButtonPrograms::new(&tokens);
    let variants = [
        ButtonVariant::Default,
        ButtonVariant::Destructive,
        ButtonVariant::Outline,
        ButtonVariant::Secondary,
        ButtonVariant::Ghost,
        ButtonVariant::Link,
    ];
    let sizes = [
        ButtonSize::Default,
        ButtonSize::Xs,
        ButtonSize::Sm,
        ButtonSize::Lg,
        ButtonSize::Icon,
        ButtonSize::IconXs,
        ButtonSize::IconSm,
        ButtonSize::IconLg,
    ];
    let static_variants = [
        aeolus_shadcn::static_tables::ButtonVariant::Default,
        aeolus_shadcn::static_tables::ButtonVariant::Destructive,
        aeolus_shadcn::static_tables::ButtonVariant::Outline,
        aeolus_shadcn::static_tables::ButtonVariant::Secondary,
        aeolus_shadcn::static_tables::ButtonVariant::Ghost,
        aeolus_shadcn::static_tables::ButtonVariant::Link,
    ];
    let static_sizes = [
        aeolus_shadcn::static_tables::ButtonSize::Default,
        aeolus_shadcn::static_tables::ButtonSize::Xs,
        aeolus_shadcn::static_tables::ButtonSize::Sm,
        aeolus_shadcn::static_tables::ButtonSize::Lg,
        aeolus_shadcn::static_tables::ButtonSize::Icon,
        aeolus_shadcn::static_tables::ButtonSize::IconXs,
        aeolus_shadcn::static_tables::ButtonSize::IconSm,
        aeolus_shadcn::static_tables::ButtonSize::IconLg,
    ];
    for (vi, variant) in variants.iter().enumerate() {
        for (si, size) in sizes.iter().enumerate() {
            let runtime_program = runtime.program(*variant, *size);
            let static_program = Button::style(static_variants[vi], static_sizes[si]);
            // THE merged-arm pin: the static arm freezes the SAME cn()-merge
            // composition the runtime handle's cn-merge lane runs (the
            // macro's expansion-time `cn()` over the arm's pieces) —
            // byte-identical to the RefCell path's interned program,
            // keyword slices included.
            assert_eq!(
                aeolus_compile::emit_static(&static_program.program()),
                aeolus_compile::emit_static(&runtime_program.program()),
                "the static arm must be byte-identical to the RefCell path's merged program ({variant:?}/{size:?})"
            );
            assert_eq!(
                static_program.keywords, runtime_program.keywords,
                "the keyword row texts ride beside the layers ({variant:?}/{size:?})"
            );
            // The runtime handle stays byte-identical to ITS own fresh
            // compile (the cn-merge lane's drift pin).
            let runtime_oracle = concat_bytes(&button_classes(*variant, *size), &tokens);
            assert_eq!(
                aeolus_compile::emit_static(&runtime_program.program()),
                aeolus_compile::emit_static(&runtime_oracle.program()),
                "the cn-merge handle must stay byte-identical to a fresh merge ({variant:?}/{size:?})"
            );
        }
    }
}

#[test]
fn every_badge_combo_is_byte_identical_across_both_paths() {
    let tokens = shadcn_table();
    let runtime = BadgePrograms::new(&tokens);
    let pairs = [
        (
            BadgeVariant::Default,
            aeolus_shadcn::static_tables::BadgeVariant::Default,
        ),
        (
            BadgeVariant::Secondary,
            aeolus_shadcn::static_tables::BadgeVariant::Secondary,
        ),
        (
            BadgeVariant::Destructive,
            aeolus_shadcn::static_tables::BadgeVariant::Destructive,
        ),
        (
            BadgeVariant::Outline,
            aeolus_shadcn::static_tables::BadgeVariant::Outline,
        ),
        (
            BadgeVariant::Ghost,
            aeolus_shadcn::static_tables::BadgeVariant::Ghost,
        ),
        (
            BadgeVariant::Link,
            aeolus_shadcn::static_tables::BadgeVariant::Link,
        ),
    ];
    for (variant, static_variant) in pairs {
        let runtime_program = runtime.program(variant);
        let static_program = Badge::style(static_variant);
        // THE merged-arm pin: the static arm freezes the RefCell path's
        // cn()-merged program (the expansion-time `cn()` over the arm's
        // pieces), keyword slices included.
        assert_eq!(
            aeolus_compile::emit_static(&static_program.program()),
            aeolus_compile::emit_static(&runtime_program.program()),
            "badge {variant:?}: the static arm must be byte-identical to the RefCell path's merged program"
        );
        assert_eq!(
            static_program.keywords, runtime_program.keywords,
            "badge {variant:?}: the keyword row texts ride beside the layers"
        );
    }
}

#[test]
fn every_alert_combo_is_byte_identical_across_both_paths() {
    let tokens = shadcn_table();
    let runtime = AlertPrograms::new(&tokens);
    let pairs = [
        (
            AlertVariant::Default,
            aeolus_shadcn::static_tables::AlertVariant::Default,
        ),
        (
            AlertVariant::Destructive,
            aeolus_shadcn::static_tables::AlertVariant::Destructive,
        ),
    ];
    for (variant, static_variant) in pairs {
        let runtime_program = runtime.program(variant);
        let static_program = Alert::style(static_variant);
        // THE merged-arm pin: the static arm freezes the RefCell path's
        // cn()-merged program (the expansion-time `cn()` over the arm's
        // pieces), keyword slices included.
        assert_eq!(
            aeolus_compile::emit_static(&static_program.program()),
            aeolus_compile::emit_static(&runtime_program.program()),
            "alert {variant:?}: the static arm must be byte-identical to the RefCell path's merged program"
        );
        assert_eq!(
            static_program.keywords, runtime_program.keywords,
            "alert {variant:?}: the keyword row texts ride beside the layers"
        );
    }
}

#[test]
fn every_base_part_is_byte_identical_across_both_paths() {
    let tokens = shadcn_table();
    let runtime = BasePrograms::new(&tokens);
    let parts = [
        (
            concat_bytes(aeolus_shadcn::classes::INPUT_BASE, &tokens),
            Base::style_slot(aeolus_shadcn::static_tables::BaseSlot::Input),
        ),
        (
            concat_bytes(aeolus_shadcn::classes::LABEL_BASE, &tokens),
            Base::style_slot(aeolus_shadcn::static_tables::BaseSlot::Label),
        ),
        (
            concat_bytes(aeolus_shadcn::classes::CARD, &tokens),
            Base::style_slot(aeolus_shadcn::static_tables::BaseSlot::Card),
        ),
        (
            runtime.card_header(),
            Base::style_slot(aeolus_shadcn::static_tables::BaseSlot::CardHeader),
        ),
        (
            runtime.card_title(),
            Base::style_slot(aeolus_shadcn::static_tables::BaseSlot::CardTitle),
        ),
        (
            runtime.card_description(),
            Base::style_slot(aeolus_shadcn::static_tables::BaseSlot::CardDescription),
        ),
        (
            runtime.card_content(),
            Base::style_slot(aeolus_shadcn::static_tables::BaseSlot::CardContent),
        ),
        (
            runtime.card_footer(),
            Base::style_slot(aeolus_shadcn::static_tables::BaseSlot::CardFooter),
        ),
        (
            runtime.separator(),
            Base::style_slot(aeolus_shadcn::static_tables::BaseSlot::Separator),
        ),
        (
            runtime.separator_horizontal(),
            Base::style_slot(aeolus_shadcn::static_tables::BaseSlot::SeparatorHorizontal),
        ),
        (
            runtime.separator_vertical(),
            Base::style_slot(aeolus_shadcn::static_tables::BaseSlot::SeparatorVertical),
        ),
        (
            runtime.skeleton(),
            Base::style_slot(aeolus_shadcn::static_tables::BaseSlot::Skeleton),
        ),
        (
            runtime.alert_title(),
            Base::style_slot(aeolus_shadcn::static_tables::BaseSlot::AlertTitle),
        ),
        (
            runtime.alert_description(),
            Base::style_slot(aeolus_shadcn::static_tables::BaseSlot::AlertDescription),
        ),
    ];
    for (index, (oracle, static_program)) in parts.iter().enumerate() {
        assert_eq!(
            aeolus_compile::emit_static(&static_program.program()),
            aeolus_compile::emit_static(&oracle.program()),
            "base part {index}: the static arm must be byte-identical to the classes constant's compile"
        );
    }
}

#[test]
fn the_static_arms_are_the_const_tables_the_headline_names() {
    // The §10 claim at expansion time: a call returns the SAME &'static
    // slices every time — the dispatch is the whole cost, the program is
    // not re-assembled per call and not interned at first call.
    let first = Button::style(
        aeolus_shadcn::static_tables::ButtonVariant::Default,
        aeolus_shadcn::static_tables::ButtonSize::Default,
    );
    let second = Button::style(
        aeolus_shadcn::static_tables::ButtonVariant::Default,
        aeolus_shadcn::static_tables::ButtonSize::Default,
    );
    assert_eq!(
        first.layers.as_ptr(),
        second.layers.as_ptr(),
        "the closed-world arm's layers are one &'static table"
    );
    assert_eq!(first.keywords.as_ptr(), second.keywords.as_ptr());
    // And the runtime fallback stays reachable for the dynamic strings —
    // the ClassCell lane the host composition rides.
    let tokens = shadcn_table();
    let runtime = ButtonPrograms::new(&tokens);
    let _ = runtime.program(ButtonVariant::Default, ButtonSize::Default);
}
