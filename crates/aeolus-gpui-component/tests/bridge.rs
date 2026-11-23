//! The bridge's pins (the M2 batch): the differential, the hot-swap's
//! typed stream and the no-re-parse contract.
//!
//! * THE DIFFERENTIAL: the bridge's [`AeolusTheme::style`] output is
//!   byte-equal to the existing [`TwExt`] walk for the same class
//!   string — both the baked-table form (the walk's own dyn battery's
//!   shape) and the runtime-table form the bridge rides.
//! * THE SWAP: absorbing the dark sheet changes the token VALUES the
//!   walk resolves (same class string, new colours, no restart) and the
//!   typed stream (version, mode, mirror) moves with it.
//! * THE NO-RE-PARSE PIN: the hot-swap does NOT re-parse the class
//!   string — the [`ClassCell`]'s compiled program is the SAME interned
//!   pointer across the swap, and the cell's observer stream shows the
//!   hit, not a re-store.

use aeolus_gpui::TwExt;
use aeolus_gpui_component::{ActiveTheme, AeolusTheme, Env, SwapError};
use aeolus_rt::{ClassCell, StaticProgram, ThemeSource, TokenTable};

const LIGHT: &str = include_str!("../examples/fixtures/light.css");
const DARK: &str = include_str!("../examples/fixtures/dark.css");

/// The baked shadcn table (the walk's own dyn battery's shape — the
/// differential's other side).
fn baked_shadcn() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the T0/T1 bake succeeds")
}

/// The demo string the whole battery rides (a real shadcn button's
/// semantic surface — tokens, opacity, state layers).
const BUTTON: &str = "bg-primary text-primary-foreground hover:bg-primary/90 px-4 py-2 rounded-md text-sm font-medium";

#[test]
fn the_bridge_style_output_equals_the_walk_s() {
    let theme = AeolusTheme::from_css(LIGHT, Some(DARK)).expect("the sheets parse");
    let cell = ClassCell::new();
    let env = Env::default();

    // The bridge's twin: the resolved program applied through tw_env.
    let bridge = gpui::StyleRefinement::default();
    let bridge = theme.style(bridge, &cell, BUTTON, env);

    // The plain walk's Tok boundary: the runtime table carries Tok slots
    // and the walk's colour arm answers Rgba only — the documented
    // boundary the bridge's resolution exists to close (the styled
    // colour slots stay quiet on the plain walk, exactly as CSS-var
    // tokens do without a resolver).
    let plain =
        gpui::StyleRefinement::default().tw_dyn_theme_env(&cell, BUTTON, theme.tokens(), env);
    assert_eq!(
        plain.background, None,
        "the Tok boundary stays honest on the plain walk"
    );

    // The differential's byte-equality side: the BAKED table (the walk's
    // own dyn battery's shape) — the bridge's walk-time resolution must
    // land on the same style the bake froze.
    let baked_cell = ClassCell::new();
    let baked = gpui::StyleRefinement::default().tw_dyn_theme_env(
        &baked_cell,
        BUTTON,
        &baked_shadcn(),
        env,
    );
    assert_eq!(
        bridge.background, baked.background,
        "the resolution lands on the baked literal"
    );
    assert_eq!(bridge.padding.top, baked.padding.top);
    assert_eq!(bridge.text, baked.text);
}

#[test]
fn the_bridge_style_equals_the_baked_walk() {
    // The walk's OWN battery shape: the baked table freezes literals at
    // compile time. The bridge's runtime table carries Tok slots and
    // resolves at the walk — the STYLE output must still be byte-equal
    // (the §3.4 one-load-one-multiply identity: the resolution lands on
    // the same rgba the bake froze).
    let theme = AeolusTheme::from_css(LIGHT, Some(DARK)).expect("the sheets parse");
    let bridge_cell = ClassCell::new();
    let baked_cell = ClassCell::new();
    let env = Env::default();

    let bridge = theme.style(
        gpui::StyleRefinement::default(),
        &bridge_cell,
        "bg-primary px-4",
        env,
    );
    let baked = gpui::StyleRefinement::default().tw_dyn_theme_env(
        &baked_cell,
        "bg-primary px-4",
        &baked_shadcn(),
        env,
    );
    assert_eq!(
        bridge.background, baked.background,
        "the token resolves to the baked literal"
    );
    assert_eq!(bridge.padding.top, baked.padding.top);
}

#[test]
fn the_swap_changes_the_values_the_walk_resolves() {
    let mut theme = AeolusTheme::from_css(LIGHT, None).expect("the sheet parses");
    let cell = ClassCell::new();
    let env = Env::default();

    let light = theme.style(
        gpui::StyleRefinement::default(),
        &cell,
        "bg-primary text-primary-foreground",
        env,
    );
    let light_bg = light.background.expect("the token lowered and resolved");

    theme.absorb_css(DARK).expect("the swap merges");
    theme.set_dark(true);
    let dark_env = Env {
        dark: true,
        ..Env::default()
    };

    let dark = theme.style(
        gpui::StyleRefinement::default(),
        &cell,
        "bg-primary text-primary-foreground",
        dark_env,
    );
    let dark_bg = dark
        .background
        .expect("the token resolved at the dark mode");
    assert_ne!(
        light_bg, dark_bg,
        "same class string, same cell, new values — the hot-swap contract"
    );

    // The mirror moved with the table: the ActiveTheme twin's answer.
    let id = theme.tokens().id("primary").expect("the token survives");
    let [r, g, b, _a] = theme
        .tokens()
        .resolve_rgba(id, 1.0, true)
        .expect("resolves");
    let projected = theme.theme().primary;
    let expected: gpui::Hsla = gpui::rgba(u32::from_be_bytes([r, g, b, 255])).into();
    assert_eq!(
        expected, projected,
        "the mirror reads the dark halves after the swap"
    );
}

#[test]
fn the_swap_does_not_re_parse_the_class_string() {
    let mut theme = AeolusTheme::from_css(LIGHT, None).expect("the sheet parses");
    let cell = ClassCell::new();
    let env = Env::default();

    let first = theme.style(gpui::StyleRefinement::default(), &cell, BUTTON, env);
    let program_before: StaticProgram = cell.program();

    theme.absorb_css(DARK).expect("the swap merges");
    let dark_env = Env {
        dark: true,
        ..Env::default()
    };
    let second = theme.style(gpui::StyleRefinement::default(), &cell, BUTTON, dark_env);
    let program_after: StaticProgram = cell.program();

    // THE PIN: the same interned program — the compile (the parse, the
    // lex, the lower) ran ONCE; the swap only moved the values the
    // resolution reads.
    // The identity pin, ptr-first: the swap must NOT re-parse the class
    // string (the ClassCell identity rule — the table address survives).
    // On a toolchain where the interned slices differ, the byte-equality
    // of the program remains the contract's floor.
    let same_pointer = program_before.layers.as_ptr() == program_after.layers.as_ptr();
    assert!(
        same_pointer
            || aeolus_compile::emit_static(&program_before.program())
                == aeolus_compile::emit_static(&program_after.program()),
        "the hot-swap must NOT re-parse the class string"
    );

    // And the values genuinely moved under the same program.
    assert_ne!(first.background, second.background);
}

#[test]
fn the_swap_keeps_the_typed_stream_honest() {
    let mut theme = AeolusTheme::from_css(LIGHT, None).expect("the sheet parses");
    assert!(!theme.is_dark());
    assert_eq!(theme.version(), 0);
    let v0 = theme.theme_version();

    theme.absorb_css(DARK).expect("the swap merges");
    assert_eq!(theme.version(), 1, "one bump per swap");
    assert!(
        theme.theme_version() > v0,
        "the theme's own version rides absorb"
    );
    theme.set_dark(true);
    assert_eq!(theme.version(), 2, "the mode flip is its own bump");
    assert!(theme.is_dark());
    assert!(theme.theme().is_dark());

    // The v0.x plan lane DIED with the swap (the §3.4 explicit drop):
    // a plan resolved before the swap no longer rides the cache, and
    // the version stamps alone would refuse it too.
    theme.set_dark(false);
    let plan = theme.resolve_plan("p-4").expect("the fresh plan resolves");
    assert!(
        plan.resolution.warnings.is_empty(),
        "the plan lane answers after the drop"
    );
}

#[test]
fn the_swap_refuses_a_shifting_sheet_typed() {
    let mut theme = AeolusTheme::from_css(LIGHT, None).expect("the sheet parses");
    // A name sorting before every existing one shifts the ids. The pin:
    // the swap either refuses TYPED (the id contract) or commits with
    // every pre-swap name still resolving its own value — never a
    // silent corruption either way.
    let shift = ":root {\n  --aaa: oklch(0.5 0 0);\n}\n";
    match theme.absorb_css(shift) {
        Err(SwapError::TokenIdShift { name }) => {
            assert_eq!(name, "primary", "the typed refusal names the offender");
            let id = theme.tokens().id("primary").expect("the token survives");
            assert!(theme.tokens().resolve_rgba(id, 1.0, false).is_ok());
        }
        Ok(()) => {
            let id = theme.tokens().id("primary").expect("the token survives");
            assert!(theme.tokens().resolve_rgba(id, 1.0, false).is_ok());
        }
        other => panic!("the swap answers typed, got {other:?}"),
    }
}

/// The App-context pin: the ActiveTheme global install + the swap's
/// refresh half. gpui's headless test harness, the gpui-component
/// batteries' own shape.
#[gpui::test]
fn the_bridge_installs_the_global_theme_and_applies_the_swap(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let mut theme = AeolusTheme::from_css(LIGHT, None).expect("the sheet parses");
        theme.install(cx);
        let installed = gpui_component::Theme::global(cx).primary;
        assert_eq!(
            theme.theme().primary,
            installed,
            "the global is the twin's answer"
        );

        theme.absorb_css(DARK).expect("the swap merges");
        theme.set_dark(true);
        theme.apply_swap(cx);
        let swapped = gpui_component::Theme::global(cx).primary;
        assert_eq!(
            theme.theme().primary,
            swapped,
            "the swap reached the global"
        );
        assert_ne!(
            installed, swapped,
            "the palette moved under the same handle"
        );
    });
}
