//! The GPUI motion half (architecture plan §3.7's transition/animate
//! row): the stateful animation lane the `with_animation` boundary
//! resolves through. The frozen static program cannot carry an element
//! id or an animator closure — the two inputs GPUI's
//! [`gpui::AnimationExt::with_animation`] needs — so the lane takes
//! them from the HOST and builds the [`gpui::Animation`] from the
//! frozen row text: the v4 animation shorthand the pipeline lowers
//! (`spin 1s linear infinite`), parsed into duration, easing and
//! iteration. The static emit keeps its honest refusal (the macro's
//! gate); the dynamic entrances ([`crate::TwExt::tw_dyn`]'s cell) and
//! a host compile are the lane's program sources.
//!
//! The timing curves are the CSS ones the registry freezes (the
//! `--ease-*` cubic-béziers), solved deterministically — the same
//! shapes the CSS arm renders and the interp arm resolves.
//!
//! What stays the host's: the animator (GPUI's `Styled` surface
//! carries no transform slots — rotate/scale/translate are
//! element-specific), the element id, and the measurement-free render
//! loop GPUI drives. [`pulse_animator`] is the one preset `Styled` CAN
//! express (opacity 1 → 0.5) and ships built-in.

use std::rc::Rc;
use std::time::Duration;

use gpui::Styled;

use crate::cond_matches;
use aeolus_compile::Env;
use aeolus_ir::Prop;
use aeolus_rt::StaticProgram;

// ---------------------------------------------------------------------------
// The shorthand grammar
// ---------------------------------------------------------------------------

/// A parsed animation shorthand — the frozen row text's shape (`spin 1s
/// linear infinite`). `name` is the `animation-name` token; the rest is
/// what [`gpui::Animation`] carries: one duration, one timing curve,
/// one iteration story.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimSpec<'t> {
    /// The `animation-name` token — the first plain word the grammar's
    /// keyword/duration/easing forms cannot claim (`spin`).
    pub name: &'t str,
    /// The FIRST duration token; the shorthand's second time token is
    /// the delay, which GPUI's `Animation` has no slot for and the lane
    /// folds away.
    pub duration: Duration,
    /// The timing curve: an easing keyword or `cubic-bezier(...)`.
    /// A shorthand without one carries the CSS initial `ease`.
    pub easing: Easing,
    /// `infinite` — the looping form. A finite iteration count has no
    /// GPUI slot (`Animation` is one-shot or looping) and folds to the
    /// one-shot form.
    pub infinite: bool,
}

/// The CSS timing curve a shorthand or the `ease-*` families name —
/// the `--ease-*` cubic-béziers the registry freezes, solved
/// deterministically by [`Easing::curve`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    /// `linear` — the delta itself.
    Linear,
    /// `ease` — the CSS initial curve, `cubic-bezier(0.25, 0.1, 0.25, 1)`.
    Ease,
    /// `ease-in` — `cubic-bezier(0.4, 0, 1, 1)`.
    EaseIn,
    /// `ease-out` — `cubic-bezier(0, 0, 0.2, 1)`.
    EaseOut,
    /// `ease-in-out` — `cubic-bezier(0.4, 0, 0.6, 1)`.
    EaseInOut,
    /// A literal `cubic-bezier(x1, y1, x2, y2)` — the v4 theme's own
    /// spellings inside the animate shorthands.
    CubicBezier(f32, f32, f32, f32),
}

impl Easing {
    /// The curve as the one-argument function [`gpui::Animation`]
    /// stores — normalized time in, eased value out (the animator
    /// receives the EASED delta).
    pub fn curve(self) -> Rc<dyn Fn(f32) -> f32> {
        match self {
            Easing::Linear => Rc::new(|t| t),
            Easing::Ease => bezier_curve(0.25, 0.1, 0.25, 1.0),
            Easing::EaseIn => bezier_curve(0.4, 0.0, 1.0, 1.0),
            Easing::EaseOut => bezier_curve(0.0, 0.0, 0.2, 1.0),
            Easing::EaseInOut => bezier_curve(0.4, 0.0, 0.6, 1.0),
            Easing::CubicBezier(x1, y1, x2, y2) => bezier_curve(x1, y1, x2, y2),
        }
    }
}

/// The CSS cubic-bézier timing solve, bisection on the x parameter (32
/// halvings — deterministic, always converges for the CSS grammar's
/// monotonic x, and never needs Newton's divergence guards). `t` is the
/// normalized time in; the returned value is the eased progress the
/// animator maps onto the element.
fn bezier_curve(x1: f32, y1: f32, x2: f32, y2: f32) -> std::rc::Rc<dyn Fn(f32) -> f32> {
    std::rc::Rc::new(move |t| {
        if t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return 1.0;
        }
        // Solve x(u) = t by bisection (x is monotonic for x1, x2 in
        // [0, 1] — the CSS grammar's own constraint), then y(u).
        let mut low = 0.0_f32;
        let mut high = 1.0_f32;
        let mut u = t;
        for _ in 0..32 {
            let x = bezier_axis(u, x1, x2);
            if (x - t).abs() < 1e-6 {
                break;
            }
            if x > t {
                high = u;
            } else {
                low = u;
            }
            u = low.midpoint(high);
        }
        bezier_axis(u, y1, y2)
    })
}

/// One axis of the cubic-bézier at parameter `u`: the Bernstein form
/// `3u(1-u)²·a + 3u²(1-u)·b + u³`.
fn bezier_axis(u: f32, a: f32, b: f32) -> f32 {
    let one_minus = 1.0 - u;
    3.0 * u * one_minus * one_minus * a + 3.0 * u * u * one_minus * b + u * u * u
}

/// Parse a duration token (`1s`, `0.5s`, `700ms`) — the shorthand's
/// time units, non-negative by the grammar. The math is f64 so whole
/// milliseconds survive the seconds conversion exactly (`from_secs_f32`
/// would round `700ms` to 699.999988ms).
fn parse_duration(token: &str) -> Option<Duration> {
    let (value, unit_millis) = token
        .strip_suffix("ms")
        .map(|v| (v, 1.0_f64))
        .or_else(|| token.strip_suffix('s').map(|v| (v, 1000.0)))?;
    let n: f64 = value.parse().ok()?;
    if n.is_finite() && n >= 0.0 {
        Some(Duration::from_secs_f64(n * unit_millis / 1000.0))
    } else {
        None
    }
}

/// Parse an easing keyword — the named forms the CSS grammar carries.
/// A parenthesised function (`steps(...)`) is never a keyword: the
/// caller skips it so it cannot steal the animation-name slot.
fn parse_easing(token: &str) -> Option<Easing> {
    match token {
        "linear" => Some(Easing::Linear),
        "ease" => Some(Easing::Ease),
        "ease-in" => Some(Easing::EaseIn),
        "ease-out" => Some(Easing::EaseOut),
        "ease-in-out" => Some(Easing::EaseInOut),
        _ => None,
    }
}

/// Parse the frozen animation shorthand the pipeline lowers (the
/// `--animate-*` texts: `spin 1s linear infinite`, `ping 1s
/// cubic-bezier(0, 0, 0.2, 1) infinite`, ...) — the v4 animation
/// shorthand grammar, one token at a time:
///
/// * the FIRST duration token wins (a second is the delay, folded
///   away — GPUI's `Animation` has no delay slot);
/// * an easing keyword or `cubic-bezier(x1, y1, x2, y2)` (spaces and
///   all — the registry texts carry them) fills the curve slot; a
///   shorthand without one keeps the CSS initial `ease`;
/// * `infinite` marks the looping form; a finite iteration count has
///   no GPUI slot and folds to the one-shot form;
/// * any other parenthesised function (`steps(...)`) is skipped, never
///   a name; the first plain leftover token is the `animation-name`.
///
/// `none` and empty texts are no animation ([`None`]) — the lane
/// renders the element statically.
pub fn parse_animation(keyword: &str) -> Option<AnimSpec<'_>> {
    let keyword = keyword.trim();
    if keyword.is_empty() || keyword == "none" {
        return None;
    }
    let mut name: Option<&str> = None;
    let mut duration = Duration::ZERO;
    let mut easing = Easing::Ease;
    let mut infinite = false;
    let mut rest = keyword;
    while !rest.is_empty() {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        if let Some(after_open) = rest.strip_prefix("cubic-bezier(") {
            // The curve may carry spaces (`cubic-bezier(0, 0, 0.2, 1)`)
            // — consume the whole parenthesised group as one token. A
            // group that is not exactly four parseable axes is consumed
            // without claiming the curve slot (the CSS initial stays).
            let Some(close) = after_open.find(')') else {
                break;
            };
            let inner = &after_open[..close];
            let mut axes = inner.split(',');
            if let (Some(a), Some(b), Some(c), Some(d)) =
                (axes.next(), axes.next(), axes.next(), axes.next())
            {
                if axes.next().is_none() {
                    if let (Ok(x1), Ok(y1), Ok(x2), Ok(y2)) = (
                        a.trim().parse::<f32>(),
                        b.trim().parse::<f32>(),
                        c.trim().parse::<f32>(),
                        d.trim().parse::<f32>(),
                    ) {
                        easing = Easing::CubicBezier(x1, y1, x2, y2);
                    }
                }
            }
            rest = &after_open[close + 1..];
            continue;
        }
        let token = rest.split_whitespace().next().unwrap_or("");
        rest = &rest[token.len()..];
        if let Some(parsed) = parse_duration(token) {
            // The FIRST duration wins; a second time token is the
            // shorthand's delay, folded away (GPUI's `Animation` has
            // no delay slot).
            if duration == Duration::ZERO {
                duration = parsed;
            }
        } else if token == "infinite" {
            infinite = true;
        } else if let Some(parsed) = parse_easing(token) {
            easing = parsed;
        } else if token.contains('(') {
            // An unknown function (`steps(4, end)`) — skipped, never a
            // name (the lane folds it away; the CSS arm renders it
            // verbatim).
        } else if name.is_none() {
            name = Some(token);
        }
    }
    Some(AnimSpec {
        name: name?,
        duration,
        easing,
        infinite,
    })
}

// ---------------------------------------------------------------------------
// The built-in animation
// ---------------------------------------------------------------------------

/// Build the [`gpui::Animation`] a parsed [`AnimSpec`] describes: the
/// parsed duration, the looping form for `infinite` (one-shot
/// otherwise), the parsed curve as the easing slot. The animator stays
/// the host's — [`gpui::AnimationExt::with_animation`] takes it beside
/// the id, the two inputs the frozen static program cannot carry.
pub fn animation(spec: AnimSpec<'_>) -> gpui::Animation {
    gpui::Animation {
        duration: spec.duration,
        oneshot: !spec.infinite,
        synced: false,
        easing: spec.easing.curve(),
        max_fps: None,
    }
}

/// Parse AND build in one step — the lane's own path
/// ([`parse_animation`] then [`animation`]). `none`/empty/unparseable
/// texts build no animation ([`None`]).
pub fn animation_for(keyword: &str) -> Option<gpui::Animation> {
    parse_animation(keyword).map(animation)
}

/// The winning `Animation` longhand's frozen row text under `env` —
/// the cascade question the lane's walks answer: the LAST matching
/// `Animation` declaration in layer order (the specificity sort's own
/// winner rule), `None` when no layer carries one or none matches.
///
/// The condition test is the walk's own [`crate::TwExt`] semantics
/// (`dark:animate-pulse` answers under a dark env, named container
/// layers stay quiet without a bounds source) — one semantics, four
/// walks.
pub fn animation_keyword(program: &StaticProgram, env: &Env) -> Option<&'static str> {
    let mut winner = None;
    for (layer_index, layer) in program.layers.iter().enumerate() {
        if !cond_matches(layer.cond, env, None, None, None) {
            continue;
        }
        for (decl_index, decl) in layer.decls.iter().enumerate() {
            if decl.prop != Prop::Animation {
                continue;
            }
            winner = program.keyword(layer_index, decl_index);
        }
    }
    winner
}

// ---------------------------------------------------------------------------
// The built-in animator
// ---------------------------------------------------------------------------

/// The pulse preset's animator — the one v4 animation `Styled` can
/// express: opacity 1 → 0.5 (the keyframes' own range), mapped over the
/// EASED delta the animation element passes. The other presets need
/// transform slots (`spin` rotates, `ping` scales, `bounce`
/// translates) — GPUI's `Styled` surface carries none, so their
/// animators stay the host's element-specific closures.
pub fn pulse_animator<E: Styled>(element: E, t: f32) -> E {
    element.opacity(1.0 - 0.5 * t)
}
