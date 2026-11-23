//! The P0 performance-contract harness (architecture plan §4's "real
//! numbers replace §4 targets" — the P0 phase's exit criteria): the
//! static-program walk measured BESIDE the hand-written builder chain
//! it replaces, the freeze-once-clone-per-element host pattern, the
//! v0.x lane's own static/dynamic paths and the V1 dynamic lane's
//! warm/cold shapes — five representative strings, one harness, the
//! measured numbers the README P0 row and §4 carry.
//!
//! # The parity pins
//!
//! The harness's setup asserts, for the four hand-writable strings,
//! that the walk's refinement EQUALS the hand-written chain's on every
//! field the string sets — the comparison is only fair when the two
//! chains do the same work (`hand == walked`, field by field). The
//! `tw!` string and the hand chain share the SAME gpui calls and
//! constants (the frozen palette's own bytes), so the measured delta is
//! the pipeline's cost, not a work difference. The stateful string has
//! no hand equivalent (the hover/focus layers need the host's
//! interactivity) and rides the walk solo; the ring string measures the
//! accumulator's compose (the bands it builds ARE the hand-written
//! output, so the solo number is the polyfill's own cost).
//!
//! # The lanes
//!
//! * `hand_*` — the builder chain a gpui author writes by hand (the
//!   §4 row's "hand-written builder chain");
//! * `static_walk_*` — `tw!("…")` frozen program applied under the
//!   default `Env` (the static hot path);
//! * `static_clone_*` — `clone()` of the walked refinement (the host
//!   freeze-once pattern, the §4 row's own shape);
//! * `v0x_static_lane` — the plan cache's `resolve_static` (the v0.x
//!   lane's ptr-keyed path);
//! * `v0x_dynamic_warm` / `v0x_dynamic_fresh_cache` — the plan cache's
//!   L1 hit and a fresh L2 cache; the lane's L1 is THREAD-LOCAL and
//!   survives the fresh cache by design (the lane's own hot path), so
//!   the fresh-cache shape is the L1-hit-plus-reinsert, not a cold
//!   parse — the true cold parse+compile is the V1 lane's
//!   `cell_cold`;
//! * `cell_warm` / `cell_cold` — the V1 dynamic lane's hit and a fresh
//!   compile (`ClassCell`).
//!
//! The iai-callgrind instruction-count golden (`tw!(s)` vs hand-written,
//! the ±2 % gate) is the PLANNED nightly half — not an in-tree target
//! yet (no iai-callgrind dependency rides the workspace). This harness
//! is the stable half, wall-clock (divan), the numbers it produces
//! recorded beside the §4 targets they replace.
//!
//! ```sh
//! cargo bench -p aeolus-gpui --bench perf_contract
//! ```

use divan::Bencher;

use aeolus_gpui::prelude::*;
use gpui::{FontWeight, Styled, px, relative, rems};

// The frozen palette's red-500 — the baked oklch token's OWN constant
// (extracted from the walk once: the literal the hand author would have
// picked, the bake's own rounding included, so the parity pins compare
// one identical constant on both sides).
fn red() -> gpui::Hsla {
    use std::sync::OnceLock;
    static BAKED: OnceLock<gpui::Hsla> = OnceLock::new();
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    *BAKED.get_or_init(|| {
        let program = tw!("bg-red-500");
        let style = gpui::StyleRefinement::default().tw(program);
        match style.background {
            Some(gpui::Fill::Color(color)) => color
                .as_solid()
                .expect("the baked red-500 is a solid colour"),
            _ => panic!("bg-red-500 walks to a solid background"),
        }
    })
}

// The button string the v0.x lane benches resolve (the lane's
// `resolve_static` takes the &'static str the call site carries).
const BUTTON: &str = "inline-flex items-center justify-center gap-2 rounded-md \
     px-4 py-2 font-medium border border-red-500 bg-red-500 text-red-500";

// ---------------------------------------------------------------------------
// The parity pins (setup-time, never measured)
// ---------------------------------------------------------------------------

/// The colour fields' parity: the composed colours reach the walk
/// through DIFFERENT conversion paths (the hand literal's sRGB bake vs
/// the frozen theme's oklch bake) that round-trip to the SAME `Rgba`
/// bytes — what renders. The parity compares the rendered bytes, the
/// colour-identity contract the pipeline pins elsewhere.
fn colour_close(a: gpui::Hsla, b: gpui::Hsla) -> bool {
    u32::from(gpui::Rgba::from(a)) == u32::from(gpui::Rgba::from(b))
}

/// The walk and the hand chain land the SAME refinement on every field
/// the string sets — the fairness check the measured comparison needs.
fn assert_button_parity(hand: &gpui::StyleRefinement, walked: &gpui::StyleRefinement) {
    assert_eq!(hand.display, walked.display, "button display");
    assert_eq!(hand.align_items, walked.align_items, "button align_items");
    assert_eq!(
        hand.justify_content, walked.justify_content,
        "button justify_content"
    );
    assert_eq!(hand.gap, walked.gap, "button gap");
    assert_eq!(
        hand.corner_radii, walked.corner_radii,
        "button corner_radii"
    );
    assert_eq!(hand.padding, walked.padding, "button padding");
    assert_eq!(hand.border_widths, walked.border_widths, "button borders");
    assert!(
        match (&hand.background, &walked.background) {
            (Some(a), Some(b)) => colour_close_solid(a, b),
            _ => false,
        },
        "button background: {:?} vs {:?}",
        hand.background,
        walked.background
    );
    assert!(
        match (&hand.text.color, &walked.text.color) {
            (Some(a), Some(b)) => colour_close(*a, *b),
            _ => false,
        },
        "button text colour: {:?} vs {:?}",
        hand.text.color,
        walked.text.color
    );
    assert_eq!(
        hand.text.font_weight, walked.text.font_weight,
        "button weight"
    );
}

fn assert_layout_parity(hand: &gpui::StyleRefinement, walked: &gpui::StyleRefinement) {
    assert_eq!(hand.size, walked.size, "layout size");
    assert_eq!(hand.padding, walked.padding, "layout padding");
    assert_eq!(hand.border_widths, walked.border_widths, "layout borders");
    assert!(
        match (&hand.border_color, &walked.border_color) {
            (Some(a), Some(b)) => colour_close(*a, *b),
            _ => false,
        },
        "layout border colour: {:?} vs {:?}",
        hand.border_color,
        walked.border_color
    );
    assert_eq!(
        hand.corner_radii, walked.corner_radii,
        "layout corner_radii"
    );
}

fn assert_typo_parity(hand: &gpui::StyleRefinement, walked: &gpui::StyleRefinement) {
    assert_eq!(
        hand.text.font_weight, walked.text.font_weight,
        "typo weight"
    );
    assert_eq!(hand.text.underline, walked.text.underline, "typo underline");
    assert_eq!(
        hand.text.strikethrough, walked.text.strikethrough,
        "typo strikethrough (the colour alone never creates one)"
    );
}

fn assert_grid_parity(hand: &gpui::StyleRefinement, walked: &gpui::StyleRefinement) {
    assert_eq!(hand.display, walked.display, "grid display");
    assert_eq!(hand.grid_cols, walked.grid_cols, "grid cols");
    assert_eq!(hand.gap, walked.gap, "grid gap");
    assert_eq!(hand.size, walked.size, "grid size");
}

fn assert_state_parity(hand: &gpui::StyleRefinement, walked: &gpui::StyleRefinement) {
    assert!(
        match (&hand.background, &walked.background) {
            (Some(a), Some(b)) => colour_close_solid(a, b),
            _ => false,
        },
        "state base background: {:?} vs {:?}",
        hand.background,
        walked.background
    );
}

/// The `Fill` forms the colour fields carry (the walk's own tag — the
/// pinned `Fill` carries the one `Color` variant): both sides solid
/// colours, compared as the rendered bytes.
fn colour_close_solid(a: &gpui::Fill, b: &gpui::Fill) -> bool {
    let (gpui::Fill::Color(a), gpui::Fill::Color(b)) = (a, b);
    match (a.as_solid(), b.as_solid()) {
        (Some(x), Some(y)) => colour_close(x, y),
        _ => false,
    }
}

/// The walked refinement wrapper (built once, outside the measured
/// loop — the clone benches' source and the parity pins' walked half).
fn walked_inline(program: StaticProgram) -> gpui::StyleRefinement {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    gpui::StyleRefinement::default().tw(program)
}

/// The hand-written button chain (the §4 row's own half — the SAME
/// builder calls the parity pins compare the walk against).
fn button_hand_chain() -> gpui::StyleRefinement {
    gpui::StyleRefinement::default()
        .flex()
        .items_center()
        .justify_center()
        .gap_x(rems(0.5))
        .gap_y(rems(0.5))
        .rounded_tl(rems(0.375))
        .rounded_tr(rems(0.375))
        .rounded_br(rems(0.375))
        .rounded_bl(rems(0.375))
        .pl(rems(1.0))
        .pr(rems(1.0))
        .pt(rems(0.5))
        .pb(rems(0.5))
        .font_weight(FontWeight::from(500.0))
        .border_t(px(1.0))
        .border_r(px(1.0))
        .border_b(px(1.0))
        .border_l(px(1.0))
        .border_color(red())
        .bg(red())
        .text_color(red())
}

/// The hand-written layout chain.
fn layout_hand_chain() -> gpui::StyleRefinement {
    gpui::StyleRefinement::default()
        .w(relative(1.0))
        .h(rems(2.5))
        .pl(rems(0.75))
        .pr(rems(0.75))
        .pt(rems(0.5))
        .pb(rems(0.5))
        .border_t(px(1.0))
        .border_r(px(1.0))
        .border_b(px(1.0))
        .border_l(px(1.0))
        .border_color(red())
        .rounded_tl(rems(0.5))
        .rounded_tr(rems(0.5))
        .rounded_br(rems(0.5))
        .rounded_bl(rems(0.5))
}

/// The hand-written typography chain (the decoration tier's own forms).
fn typo_hand_chain() -> gpui::StyleRefinement {
    gpui::StyleRefinement::default()
        .font_weight(FontWeight::from(700.0))
        .underline()
        .text_decoration_color(red())
        .text_decoration_2()
        .text_decoration_wavy()
}

/// The hand-written grid chain.
fn grid_hand_chain() -> gpui::StyleRefinement {
    gpui::StyleRefinement::default()
        .grid()
        .grid_cols(3)
        .gap_x(rems(1.0))
        .gap_y(rems(1.0))
        .h(rems(8.0))
}

/// The hand-written state base (the layers have no hand equivalent).
fn state_hand_chain() -> gpui::StyleRefinement {
    gpui::StyleRefinement::default().bg(red())
}

/// Build the five walked refinements ONCE and pin every parity pair
/// before the benches measure anything (a diverging chain fails the
/// harness at startup — the comparison is never unfair). The returns
/// carry the walked refinements the clone benches reuse:
/// `(button, layout, typo, grid, state, ring_walked)`.
fn pinned() -> (
    gpui::StyleRefinement,
    gpui::StyleRefinement,
    gpui::StyleRefinement,
    gpui::StyleRefinement,
    gpui::StyleRefinement,
    gpui::StyleRefinement,
) {
    let button_hand = button_hand_chain();
    let layout_hand = layout_hand_chain();
    let typo_hand = typo_hand_chain();
    let grid_hand = grid_hand_chain();
    let state_hand = state_hand_chain();
    let (button_walked, layout_walked, typo_walked, grid_walked, state_walked) = (
        walked_inline(tw!(
            "inline-flex items-center justify-center gap-2 rounded-md \
             px-4 py-2 font-medium border border-red-500 bg-red-500 text-red-500"
        )),
        walked_inline(tw!(
            "w-full h-10 px-3 py-2 border border-red-500 rounded-lg"
        )),
        walked_inline(tw!(
            "font-bold underline decoration-wavy decoration-2 decoration-red-500"
        )),
        walked_inline(tw!("grid grid-cols-3 gap-4 h-32")),
        walked_inline(tw!(
            "hover:bg-red-500/90 focus:ring-2 focus:ring-red-500 bg-red-500"
        )),
    );
    assert_button_parity(&button_hand, &button_walked);
    assert_layout_parity(&layout_hand, &layout_walked);
    assert_typo_parity(&typo_hand, &typo_walked);
    assert_grid_parity(&grid_hand, &grid_walked);
    assert_state_parity(&state_hand, &state_walked);
    let ring_walked = walked_inline(tw!(
        "ring-2 ring-red-500 ring-offset-2 ring-offset-white bg-red-500"
    ));
    (
        button_hand,
        layout_hand,
        typo_hand,
        grid_hand,
        state_hand,
        ring_walked,
    )
}

// ---------------------------------------------------------------------------
// The hand-written chains (the §4 row's own half)
// ---------------------------------------------------------------------------

#[divan::bench]
fn hand_button_chain(bencher: Bencher) {
    bencher.bench(button_hand_chain);
}

#[divan::bench]
fn hand_layout_chain(bencher: Bencher) {
    bencher.bench(layout_hand_chain);
}

#[divan::bench]
fn hand_typo_chain(bencher: Bencher) {
    bencher.bench(typo_hand_chain);
}

#[divan::bench]
fn hand_grid_chain(bencher: Bencher) {
    bencher.bench(grid_hand_chain);
}

// ---------------------------------------------------------------------------
// The static-program walk (the lane the macro replaces the chain with)
// ---------------------------------------------------------------------------

#[divan::bench]
fn static_walk_button(bencher: Bencher) {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let program = tw!("inline-flex items-center justify-center gap-2 rounded-md \
         px-4 py-2 font-medium border border-red-500 bg-red-500 text-red-500");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

#[divan::bench]
fn static_walk_layout(bencher: Bencher) {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let program = tw!("w-full h-10 px-3 py-2 border border-red-500 rounded-lg");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

#[divan::bench]
fn static_walk_typo(bencher: Bencher) {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let program = tw!("font-bold underline decoration-wavy decoration-2 decoration-red-500");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

#[divan::bench]
fn static_walk_grid(bencher: Bencher) {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let program = tw!("grid grid-cols-3 gap-4 h-32");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

#[divan::bench]
fn static_walk_state(bencher: Bencher) {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    // The stateful representative rides the walk solo (the hover/focus
    // layers need the host's interactivity — no hand equivalent).
    let program = tw!("hover:bg-red-500/90 focus:ring-2 focus:ring-red-500 bg-red-500");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

#[divan::bench]
fn ring_walk(bencher: Bencher) {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    // The polyfill representative solo: the bands the accumulator builds
    // ARE the hand-written output, so this number is the polyfill's own
    // compose cost.
    let program = tw!("ring-2 ring-red-500 ring-offset-2 ring-offset-white bg-red-500");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

// ---------------------------------------------------------------------------
// The freeze-once-clone-per-element host pattern (the §4 row's own shape)
// ---------------------------------------------------------------------------

#[divan::bench]
fn static_clone_button(bencher: Bencher) {
    let (button, ..) = pinned();
    bencher.bench(|| button.clone());
}

#[divan::bench]
fn static_clone_layout(bencher: Bencher) {
    let (_, layout, ..) = pinned();
    bencher.bench(|| layout.clone());
}

// ---------------------------------------------------------------------------
// The v0.x lane (the plan cache's own static/dynamic paths)
// ---------------------------------------------------------------------------

#[divan::bench]
fn v0x_static_lane(bencher: Bencher) {
    use aeolus_core::PlanCache;
    let theme = aeolus_core::shadcn_default().expect("the embedded theme parses");
    let cache = PlanCache::new();
    bencher.bench(|| cache.resolve_static(&theme, BUTTON).expect("resolves"));
}

#[divan::bench]
fn v0x_dynamic_warm(bencher: Bencher) {
    use aeolus_core::PlanCache;
    let theme = aeolus_core::shadcn_default().expect("the embedded theme parses");
    let cache = PlanCache::new();
    let _ = cache.resolve(&theme, BUTTON).expect("resolves");
    bencher.bench(|| cache.resolve(&theme, BUTTON).expect("resolves"));
}

#[divan::bench]
fn v0x_dynamic_fresh_cache(bencher: Bencher) {
    // A fresh L2 cache per iteration; the thread-local L1 persists (the
    // lane's own warm path by design — the doc comment says why this is
    // not a cold parse).
    use aeolus_core::PlanCache;
    let theme = aeolus_core::shadcn_default().expect("the embedded theme parses");
    bencher.bench(|| {
        let cache = PlanCache::new();
        cache.resolve(&theme, BUTTON).expect("resolves")
    });
}

// ---------------------------------------------------------------------------
// The V1 dynamic lane (the ClassCell's hit and a fresh compile)
// ---------------------------------------------------------------------------

#[divan::bench]
fn cell_warm(bencher: Bencher) {
    // The cell is the UI-thread shape (`RefCell`/`Rc`, !Sync — the
    // §3.6 surface's own contract): each bench thread carries its own.
    thread_local! {
        static CELL: aeolus_rt::ClassCell = aeolus_rt::ClassCell::new();
    }
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    CELL.with(|cell| {
        let _ = cell.get(BUTTON, &Default::default()).expect("compiles");
    });
    bencher.bench(|| {
        CELL.with(|cell| {
            let program = cell.get(BUTTON, &Default::default()).expect("compiles");
            gpui::StyleRefinement::default().tw(program)
        })
    });
}

#[divan::bench]
fn cell_cold(bencher: Bencher) {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    bencher.bench(|| {
        let cell = aeolus_rt::ClassCell::new();
        let program = cell.get(BUTTON, &Default::default()).expect("compiles");
        gpui::StyleRefinement::default().tw(program)
    });
}

fn main() {
    divan::main();
}
