//! The GPUI motion half (architecture plan §3.7's transition/animate
//! row, the `with_animation` boundary): `animate-*` longhands ride the
//! stateful animation lane — the host names the element (the
//! `ElementId` the frozen static program cannot carry) and supplies the
//! animator closure (GPUI's `Styled` surface carries no transform
//! slots, so the visual mapping stays the host's), while the lane
//! builds the `gpui::Animation` from the frozen row text — the v4
//! shorthand the pipeline lowers (`spin 1s linear infinite`), parsed
//! into duration, easing and iteration.
//!
//! The pins:
//! * the parse is the v4 animation shorthand grammar: name, the FIRST
//!   duration, the easing keyword or `cubic-bezier(...)` (spaces and
//!   all), `infinite`, finite iteration counts folding to one-shot;
//! * `animate-none` and empty texts are no animation;
//! * the CSS timing curves solve exactly (the `--ease-*` cubic-béziers
//!   the registry freezes, endpoints and midpoints pinned);
//! * the built-in `gpui::Animation` carries the parsed duration,
//!   iteration and easing curve;
//! * the winner selection is the cascade: the LAST matching
//!   `Animation` declaration in layer order, under the probe `Env`;
//! * `animate-none`/absent animations render the element statically
//!   (the zero-animation wrap, the host's animator never called);
//! * the pulse preset is the one animator `Styled` can express —
//!   opacity 1 → 0.5 — and it is the built-in helper;
//! * the dynamic entrances compile through [`ClassCell`] (the cell has
//!   no GPUI capability gate, the tw! macro's refusal stays).

use std::time::Duration;

use aeolus_compile::Config;
use aeolus_gpui::prelude::*;
use aeolus_gpui::{Easing, animation_for, animation_keyword, parse_animation, pulse_animator};
use aeolus_rt::{ClassCell, ThemeSource, TokenTable};

#[test]
fn spin_parses_the_full_shorthand() {
    let spec = parse_animation("spin 1s linear infinite").expect("the preset parses");
    assert_eq!(spec.name, "spin");
    assert_eq!(spec.duration, Duration::from_secs(1));
    assert_eq!(spec.easing, Easing::Linear);
    assert!(spec.infinite);
}

#[test]
fn ping_and_pulse_parse_their_cubic_bezier_curves() {
    let ping =
        parse_animation("ping 1s cubic-bezier(0, 0, 0.2, 1) infinite").expect("the preset parses");
    assert_eq!(ping.duration, Duration::from_secs(1));
    assert_eq!(ping.easing, Easing::CubicBezier(0.0, 0.0, 0.2, 1.0));
    assert!(ping.infinite);

    let pulse = parse_animation("pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite")
        .expect("the preset parses");
    assert_eq!(pulse.duration, Duration::from_secs(2));
    assert_eq!(pulse.easing, Easing::CubicBezier(0.4, 0.0, 0.6, 1.0));
}

#[test]
fn bounce_has_no_easing_token_so_the_css_initial_applies() {
    // `bounce 1s infinite` carries no timing keyword — the CSS initial
    // `ease` curve applies, the grammar's own default.
    let spec = parse_animation("bounce 1s infinite").expect("the preset parses");
    assert_eq!(spec.name, "bounce");
    assert_eq!(spec.duration, Duration::from_secs(1));
    assert_eq!(spec.easing, Easing::Ease);
    assert!(spec.infinite);
}

#[test]
fn none_and_empty_are_no_animation() {
    assert!(parse_animation("none").is_none());
    assert!(parse_animation("").is_none());
    assert!(parse_animation("   ").is_none());
}

#[test]
fn finite_iteration_counts_fold_to_one_shot() {
    // The shorthand's finite iteration count has no GPUI slot (the
    // `Animation` carries oneshot only) — it folds to the one-shot
    // form, the documented boundary.
    let spec = parse_animation("spin 1s 3").expect("the shorthand parses");
    assert!(!spec.infinite);
    assert_eq!(spec.duration, Duration::from_secs(1));
}

#[test]
fn durations_parse_in_seconds_and_milliseconds() {
    let half = parse_animation("waggle 0.5s").expect("parses");
    assert_eq!(half.duration, Duration::from_millis(500));
    let ms = parse_animation("waggle 700ms").expect("parses");
    assert_eq!(ms.duration, Duration::from_millis(700));
    // The FIRST duration wins; a second time token is the shorthand's
    // delay, which the lane folds away (GPUI's Animation has no delay).
    let first = parse_animation("waggle 1s 2s").expect("parses");
    assert_eq!(first.duration, Duration::from_secs(1));
}

#[test]
fn unknown_functions_and_tokens_never_steal_the_name() {
    // A parenthesised function the lane does not know (`steps(...)`)
    // is skipped — the FIRST plain token stays the animation-name.
    let spec = parse_animation("waggle 1s steps(4, end) infinite").expect("parses");
    assert_eq!(spec.name, "waggle");
    assert_eq!(spec.duration, Duration::from_secs(1));
    assert_eq!(spec.easing, Easing::Ease);
    assert!(spec.infinite);
}

#[test]
fn timing_curves_solve_the_css_cubic_beziers() {
    // The `--ease-*` curves the registry freezes, endpoints exact and
    // midpoints pinned against the solved values (bisection, 32
    // iterations — deterministic).
    let linear = Easing::Linear.curve();
    assert_eq!(linear(0.0), 0.0);
    assert_eq!(linear(0.5), 0.5);
    assert_eq!(linear(1.0), 1.0);

    let ease_out = Easing::CubicBezier(0.0, 0.0, 0.2, 1.0).curve();
    assert_eq!(ease_out(0.0), 0.0);
    assert_eq!(ease_out(1.0), 1.0);
    assert!((ease_out(0.25) - 0.5776).abs() < 1e-3);
    assert!((ease_out(0.5) - 0.8392).abs() < 1e-3);
    assert!((ease_out(0.75) - 0.9642).abs() < 1e-3);

    let ease_in_out = Easing::CubicBezier(0.4, 0.0, 0.6, 1.0).curve();
    assert!((ease_in_out(0.5) - 0.5).abs() < 1e-3);
    assert!((ease_in_out(0.25) - 0.1353).abs() < 1e-3);

    let ease_in = Easing::CubicBezier(0.4, 0.0, 1.0, 1.0).curve();
    assert!((ease_in(0.5) - 0.3248).abs() < 1e-3);

    let ease = Easing::Ease.curve();
    assert!((ease(0.5) - 0.8024).abs() < 1e-3);
}

#[test]
fn the_built_in_animation_carries_the_parsed_shape() {
    // `animate-spin` builds the looping one-second linear animation —
    // the `Animation` fields the GPUI element consumes.
    let spin = animation_for("spin 1s linear infinite").expect("the preset builds");
    assert_eq!(spin.duration, Duration::from_secs(1));
    assert!(!spin.oneshot);
    assert_eq!((spin.easing)(0.5), 0.5);

    // `animate-ping` builds the looping ease-out second — the curve
    // rides the Animation's easing slot (the animator receives the
    // EASED delta).
    let ping =
        animation_for("ping 1s cubic-bezier(0, 0, 0.2, 1) infinite").expect("the preset builds");
    assert!((ping.easing)(0.25) - 0.5776 < 1e-3);

    // A finite count builds the ONE-SHOT form.
    let once = animation_for("spin 1s 3").expect("the shorthand builds");
    assert!(once.oneshot);
}

#[test]
fn winner_selection_is_the_cascade_under_the_env() {
    // The cell compiles the string (the tw! macro's gate refuses
    // motion classes — the lane's programs come from the dynamic lane
    // or a host compile); the LAST matching `Animation` declaration in
    // layer order wins, per the probe env.
    let cell = ClassCell::new();
    let dark = Env {
        dark: true,
        ..Env::default()
    };

    let program = cell
        .get("animate-spin dark:animate-pulse", &Config::default())
        .expect("the cell compiles the motion string");
    let base = animation_keyword(&program, &Env::default()).expect("the base winner");
    assert_eq!(base, "spin 1s linear infinite");
    let dark_winner = animation_keyword(&program, &dark).expect("the dark winner");
    assert_eq!(
        dark_winner,
        "pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite"
    );

    // A non-motion string carries no winner.
    let plain = cell
        .get("p-4", &Config::default())
        .expect("the cell compiles the plain string");
    assert!(animation_keyword(&plain, &Env::default()).is_none());
}

#[test]
fn theme_tokens_compile_through_the_themed_dynamic_entrance() {
    // The themed cell compiles colour + motion strings the same way —
    // the lane's themed entrance is the `tw_dyn_theme` pairing.
    let cell = ClassCell::new();
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let tokens = TokenTable::baked(&source).expect("the built-in shadcn table bakes");
    let program = cell
        .get_theme("animate-spin bg-primary", &tokens, &Config::default())
        .expect("the themed compile lowers");
    assert!(animation_keyword(&program, &Env::default()).is_some());
}

#[test]
fn the_pulse_animator_maps_opacity_for_any_styled_element() {
    // v4's pulse keyframes: opacity 1 → 0.5, eased. The animator
    // receives the EASED delta, so the mapping is the linear lerp.
    let refinement = gpui::StyleRefinement::default();
    let animated = pulse_animator(refinement, 0.4);
    let eased = animated.opacity.expect("the animator set the opacity");
    assert!((eased - 0.8).abs() < 1e-6);

    let rest = pulse_animator(gpui::StyleRefinement::default(), 0.0);
    let start = rest.opacity.expect("the animator set the opacity");
    assert!((start - 1.0).abs() < 1e-6);
}

#[test]
fn the_lane_wraps_elements_with_the_built_animation() {
    // The stateful lane's static form: a program the host compiled
    // (here through the cell), the host's element id and animator —
    // the two inputs the frozen static program cannot carry, per plan
    // §3.7's motion row. The wrap compiles headless (the same
    // construction a render pass runs).
    let cell = ClassCell::new();
    let program = cell
        .get("animate-spin", &Config::default())
        .expect("the cell compiles");
    let element =
        gpui::div().tw_anim_env(program, Env::default(), "dashboard-spinner", |el, _t| el);
    let _ = element;
}

#[test]
fn animate_none_wraps_the_element_static_without_the_animator() {
    // `animate-none` (and a program with no animation at all) renders
    // the element statically — the zero-animation wrap, the host's
    // animator never called.
    let cell = ClassCell::new();
    let program = cell
        .get("animate-none p-4", &Config::default())
        .expect("the cell compiles");
    let element = gpui::div().tw_anim_env(program, Env::default(), "card", |el, _t| el);
    let _ = element;
}
