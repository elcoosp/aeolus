//! The GPUI runtime for the Aeolus V1 static program (architecture plan
//! §3.1 emit (a)/(b), §3.2 `aeolus-codegen-gpui`, P3).
//!
//! ```ignore
//! use aeolus_gpui::prelude::*;
//!
//! div()
//!     .tw(tw!("flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground"))
//!     .child("Save")
//! ```
//!
//! `tw!` expands to an [`aeolus_rt::StaticProgram`] — the IR layers plus
//! the row-keyword texts — and this crate maps those declarations onto
//! GPUI's native style fields. State layers (`hover:`, `focus:`, ...) go
//! through [`TwStateExt`], which wires GPUI's own interaction closures;
//! [`TwExt::tw`] applies the base + environment-matched layers (dark,
//! breakpoints) for any `Styled` element.
//!
//! Attribute layers (`data-*:`/`aria-*:`) answer through the host state
//! seam (plan §3.5's "typed state bits passed WITH the element", P4's
//! last named piece): [`TwExt::tw_probes`] walks the base + environment
//! layers with the element's attribute bits in hand — the GPUI half of
//! the interp arm's `interp_probes` entry point. The plain [`TwExt::tw`]
//! keeps those layers quiet (the sourceless skip the interp reports as
//! `AttributeUnqueried` notes), and the descendant `has-*` shapes stay
//! the diagnosed boundary with or without bits.
//!
//! The keyword texts are why the program is sufficient: `hidden` and
//! `block` are both `Prop::Display` + `Val::Kw(0)` in the bare IR, so
//! the runtime reads the row text frozen beside the layer.

use std::cell::RefCell;

use aeolus_compile::Env;
use aeolus_ir::{Cond, Prop, Val};
use aeolus_rt::StaticProgram;
use gpui::{
    AbsoluteLength, DefiniteLength, Div, FontWeight, Hsla, IntoElement, Length,
    StatefulInteractiveElement, Styled, TextAlign, px, relative, rems, rgba,
};

pub use aeolus_macro::{cn, theme, tw, tw_cn, variants};
pub use aeolus_rt::{Env as Environment, StaticProgram as Program, theme_from_css};
// The attribute-bits seam (plan §3.5): the element's own `data-`/`aria-`
// bits ride the host state surface, so the prelude carries the probe
// trait, the built-in bit table and the bit/kind constructors.
pub use aeolus_rt::{AttrBit, AttrKind, AttrProbe, AttrState};

/// The V1 consumption surface: macros, the program type and the two
/// extension traits.
pub mod prelude {
    pub use crate::{
        AeolusRoot, TwExt, TwStateExt, begin_frame, cn, set_env, theme, theme_from_css, tw, tw_cn,
        variants,
    };
    pub use aeolus_compile::Env;
    pub use aeolus_rt::{AttrBit, AttrKind, AttrProbe, AttrState, StaticProgram};
}

thread_local! {
    static ENV: RefCell<Env> = RefCell::new(Env::default());
}

/// Replace the process-wide probe environment used by [`TwExt::tw`].
pub fn set_env(env: Env) {
    ENV.with(|current| *current.borrow_mut() = env);
}

/// The current probe environment.
pub fn env() -> Env {
    ENV.with(|current| *current.borrow())
}

/// Run `f` with `env` installed, restoring the previous value after.
pub fn with_env<R>(env: Env, f: impl FnOnce() -> R) -> R {
    let previous = ENV.with(|current| current.replace(env));
    let result = f();
    ENV.with(|current| *current.borrow_mut() = previous);
    result
}

/// Apply a static program's base + environment layers to any styled
/// element.
pub trait TwExt: Styled + Sized {
    /// Apply the program under the current [`env`].
    fn tw(self, program: StaticProgram) -> Self {
        self.tw_env(program, env())
    }

    /// Apply the program under an explicit probe environment.
    fn tw_env(self, program: StaticProgram, env: Env) -> Self {
        apply_matching(self, &program, &env, None)
    }

    /// Apply the program under the current [`env`], with the element's
    /// attribute bits answering the `data-*:`/`aria-*:` layers — the
    /// GPUI half of the interp arm's `interp_probes` entry point (plan
    /// §3.5's "typed state bits passed WITH the element", one mask test
    /// per probe).
    ///
    /// `probes` is the [`AttrProbe`] seam, the same shape as the token
    /// seam: the built-in [`AttrState`] is a static bit table, and a
    /// host implements the one question itself (a component-state map,
    /// a bridge over the window's real attribute surface, a test
    /// double). A presence spelling (`data-open:`) fires on any value,
    /// a value spelling (`data-[state=open]:`) on equality, and a miss
    /// is CSS-quiet; the descendant `has-*` shapes stay the diagnosed
    /// boundary (they are not element-own bits) with or without bits.
    fn tw_probes(self, program: StaticProgram, probes: &impl AttrProbe) -> Self {
        self.tw_env_probes(program, env(), probes)
    }

    /// [`TwExt::tw_probes`] under an explicit probe environment — the
    /// `tw_env` pairing, with the attribute bits riding along.
    fn tw_env_probes(self, program: StaticProgram, env: Env, probes: &impl AttrProbe) -> Self {
        apply_matching(self, &program, &env, Some(probes as &dyn AttrProbe))
    }
}

impl<T: Styled> TwExt for T {}

/// Apply a static program's interaction layers through GPUI's native
/// `hover`/`focus`/... closures. Use on a stateful element:
///
/// ```ignore
/// div().id("save").tw(tw!("...")).tw_state(tw!("hover:bg-primary/90"))
/// ```
pub trait TwStateExt: StatefulInteractiveElement + Styled + Sized {
    /// Wire every state layer in `program` to its GPUI interaction slot.
    fn tw_state(self, program: StaticProgram) -> Self {
        let env = env();
        // The base+env walk stays sourceless: the interaction closures
        // are GPUI's own state, the element-own attribute bits ride the
        // `tw_probes` walk instead.
        let mut this = apply_matching(self, &program, &env, None);
        this = this.hover(|style| apply_state(style, &program, is_hover));
        this = this.focus(|style| apply_state(style, &program, is_focus));
        this = this.focus_visible(|style| apply_state(style, &program, is_focus_visible));
        this = this.active(|style| apply_state(style, &program, is_active));
        this
    }
}

impl<T: StatefulInteractiveElement + Styled> TwStateExt for T {}

// ---------------------------------------------------------------------------
// Condition matching
// ---------------------------------------------------------------------------

fn is_hover(cond: Cond) -> bool {
    matches!(cond, Cond::Hover | Cond::GroupHover | Cond::PeerHover)
}

fn is_focus(cond: Cond) -> bool {
    matches!(cond, Cond::Focus | Cond::GroupFocus | Cond::PeerFocus)
}

fn is_focus_visible(cond: Cond) -> bool {
    matches!(cond, Cond::FocusVisible)
}

fn is_active(cond: Cond) -> bool {
    matches!(cond, Cond::Active)
}

fn env_matches(cond: Cond, env: &Env) -> bool {
    match cond {
        Cond::Base => true,
        Cond::Dark => env.dark,
        Cond::Hover => env.hover,
        Cond::Focus => env.focus,
        Cond::FocusVisible => env.focus_visible,
        Cond::FocusWithin => env.focus_within,
        Cond::Active => env.active,
        Cond::Disabled => env.disabled,
        Cond::Visited => env.visited,
        Cond::Checked => env.checked,
        Cond::PlaceholderShown => env.placeholder_shown,
        Cond::Selection | Cond::Placeholder => false,
        Cond::GroupHover => env.group_hover,
        Cond::GroupFocus => env.group_focus,
        Cond::PeerHover => env.peer_hover,
        Cond::PeerFocus => env.peer_focus,
        // The attribute shapes route through `cond_matches`'s probe
        // arms (the env alone has no answer for them); the guard keeps
        // the match total so a new `Cond` variant fails to compile here
        // instead of silently skipping.
        Cond::Data { .. } | Cond::Aria { .. } | Cond::Has(_) => {
            unreachable!("cond_matches routes the attribute shapes to the probe arms")
        }
        Cond::GroupHas(_) | Cond::PeerHas(_) => {
            unreachable!("cond_matches routes the attribute shapes to the probe arms")
        }
        Cond::Bp(bp) | Cond::Min(bp) => env.viewport_rem >= bp.rem(),
        Cond::Max(bp) => env.viewport_rem < bp.rem(),
        Cond::Container(bp) => env.container_rem.is_some_and(|rem| rem >= bp.rem()),
        Cond::MaxContainer(bp) => env.container_rem.is_some_and(|rem| rem < bp.rem()),
    }
}

// ---------------------------------------------------------------------------
// The walk
// ---------------------------------------------------------------------------

/// The one condition test the walk applies: the element-own attribute
/// probes answer through the host's probe source (plan §3.5's "typed
/// state bits passed WITH the element"), everything else through the
/// env.
fn cond_matches(cond: Cond, env: &Env, probes: Option<&dyn AttrProbe>) -> bool {
    match cond {
        // The element-own probes answer through the seam — the interp's
        // `decide` semantics applied at the host: a presence spelling
        // fires on any value, a value spelling on equality, a miss is
        // CSS-quiet. With no source the layers stay quiet (the interp
        // reports `AttributeUnqueried` notes; this walk has no note
        // channel, so the selector-miss skip stands).
        Cond::Data { name, value } => match probes {
            Some(source) => match source.probe(AttrKind::Data, name) {
                Some(answer) => value.is_none_or(|folded| answer == folded),
                None => false,
            },
            None => false,
        },
        Cond::Aria { name, value } => match probes {
            Some(source) => match source.probe(AttrKind::Aria, name) {
                Some(answer) => value.is_none_or(|folded| answer == folded),
                None => false,
            },
            None => false,
        },
        // The descendant shapes (`has-`/`group-has-`/`peer-has-`) are
        // NOT element-own attribute bits — the plan §3.5 row keeps them
        // the diagnosed boundary (the interp reports them as notes), so
        // the walk skips them with or without a source.
        Cond::Has(_) | Cond::GroupHas(_) | Cond::PeerHas(_) => false,
        other => env_matches(other, env),
    }
}

fn apply_matching<S: Styled>(
    mut style: S,
    program: &StaticProgram,
    env: &Env,
    probes: Option<&dyn AttrProbe>,
) -> S {
    for (layer_index, layer) in program.layers.iter().enumerate() {
        if !cond_matches(layer.cond, env, probes) {
            continue;
        }
        for (decl_index, decl) in layer.decls.iter().enumerate() {
            let keyword = program.keyword(layer_index, decl_index).unwrap_or("");
            style = apply_decl(style, decl.prop, decl.val, keyword);
        }
    }
    style
}

fn apply_state<S: Styled>(style: S, program: &StaticProgram, pred: fn(Cond) -> bool) -> S {
    let mut style = style;
    for (layer_index, layer) in program.layers.iter().enumerate() {
        if !pred(layer.cond) {
            continue;
        }
        for (decl_index, decl) in layer.decls.iter().enumerate() {
            let keyword = program.keyword(layer_index, decl_index).unwrap_or("");
            style = apply_decl(style, decl.prop, decl.val, keyword);
        }
    }
    style
}

// ---------------------------------------------------------------------------
// Declaration mapping
// ---------------------------------------------------------------------------

fn apply_decl<S: Styled>(style: S, prop: Prop, val: Val, keyword: &str) -> S {
    match prop {
        Prop::Display => display(style, keyword),
        Prop::Position => position(style, keyword),
        Prop::Visibility => visibility(style, keyword),
        Prop::FlexDirection => flex_direction(style, keyword),
        Prop::FlexWrap => flex_wrap(style, keyword),
        Prop::JustifyContent => justify(style, keyword),
        Prop::JustifyItems | Prop::JustifySelf => justify(style, keyword),
        Prop::AlignItems => align_items(style, keyword),
        Prop::AlignContent => align_content(style, keyword),
        Prop::AlignSelf => align_self(style, keyword),
        Prop::FlexGrow => with_num(style, val, Styled::flex_grow),
        Prop::FlexShrink => with_num(style, val, Styled::flex_shrink),
        Prop::GridTemplateColumns => match val {
            Val::Num(n) => style.grid_cols(n as u16),
            _ => style,
        },
        Prop::GridTemplateRows => match val {
            Val::Num(n) => style.grid_rows(n as u16),
            _ => style,
        },
        Prop::GapRow => with_definite(style, val, Styled::gap_y),
        Prop::GapCol => with_definite(style, val, Styled::gap_x),
        Prop::Width => with_length(style, val, Styled::w),
        Prop::Height => with_length(style, val, Styled::h),
        Prop::MinWidth => with_length(style, val, Styled::min_w),
        Prop::MinHeight => with_length(style, val, Styled::min_h),
        Prop::MaxWidth => with_length(style, val, Styled::max_w),
        Prop::MaxHeight => with_length(style, val, Styled::max_h),
        Prop::AspectRatio => with_num(style, val, Styled::aspect_ratio),
        Prop::Top => with_length(style, val, Styled::top),
        Prop::Right => with_length(style, val, Styled::right),
        Prop::Bottom => with_length(style, val, Styled::bottom),
        Prop::Left => with_length(style, val, Styled::left),
        Prop::PaddingTop => with_definite(style, val, Styled::pt),
        Prop::PaddingRight => with_definite(style, val, Styled::pr),
        Prop::PaddingBottom => with_definite(style, val, Styled::pb),
        Prop::PaddingLeft => with_definite(style, val, Styled::pl),
        Prop::MarginTop => with_length(style, val, Styled::mt),
        Prop::MarginRight => with_length(style, val, Styled::mr),
        Prop::MarginBottom => with_length(style, val, Styled::mb),
        Prop::MarginLeft => with_length(style, val, Styled::ml),
        Prop::BorderTopWidth => with_absolute(style, val, Styled::border_t),
        Prop::BorderRightWidth => with_absolute(style, val, Styled::border_r),
        Prop::BorderBottomWidth => with_absolute(style, val, Styled::border_b),
        Prop::BorderLeftWidth => with_absolute(style, val, Styled::border_l),
        Prop::BorderColor => with_color(style, val, Styled::border_color),
        Prop::BorderStyle => border_style(style, keyword),
        Prop::BorderTopLeftRadius => with_absolute(style, val, Styled::rounded_tl),
        Prop::BorderTopRightRadius => with_absolute(style, val, Styled::rounded_tr),
        Prop::BorderBottomRightRadius => with_absolute(style, val, Styled::rounded_br),
        Prop::BorderBottomLeftRadius => with_absolute(style, val, Styled::rounded_bl),
        Prop::Background => with_color(style, val, Styled::bg),
        Prop::Color => with_color(style, val, Styled::text_color),
        Prop::FontSize => with_absolute(style, val, Styled::text_size),
        Prop::LineHeight => with_definite(style, val, Styled::line_height),
        Prop::FontWeight => match keyword.parse::<f32>() {
            Ok(weight) => style.font_weight(FontWeight::from(weight)),
            Err(_) => style,
        },
        Prop::FontFamily => font_family(style, keyword),
        Prop::FontStyle => match keyword {
            "italic" => style.italic(),
            "normal" | "not-italic" => style.not_italic(),
            _ => style,
        },
        Prop::TextAlign => text_align(style, keyword),
        Prop::TextDecorationLine => text_decoration(style, keyword),
        Prop::WhiteSpace => match keyword {
            "nowrap" | "pre" | "pre-line" | "pre-wrap" | "break-spaces" => {
                style.whitespace_nowrap()
            }
            _ => style.whitespace_normal(),
        },
        Prop::TextOverflow => match keyword {
            "ellipsis" | "truncate" => style.text_ellipsis(),
            _ => style,
        },
        Prop::OverflowX => overflow_x(style, keyword),
        Prop::OverflowY => overflow_y(style, keyword),
        Prop::Opacity => with_num(style, val, |style, n| {
            style.opacity(if n > 1.0 { n / 100.0 } else { n })
        }),
        Prop::Cursor => cursor(style, keyword),
        _ => style,
    }
}

fn display<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "none" => style.hidden(),
        "flex" | "inline-flex" => style.flex(),
        "grid" | "inline-grid" => style.grid(),
        _ => style.block(),
    }
}

fn position<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "absolute" => style.absolute(),
        _ => style.relative(),
    }
}

fn visibility<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "hidden" | "collapse" => style.invisible(),
        _ => style.visible(),
    }
}

fn flex_direction<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "column" => style.flex_col(),
        "column-reverse" => style.flex_col_reverse(),
        "row-reverse" => style.flex_row_reverse(),
        _ => style.flex_row(),
    }
}

fn flex_wrap<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "wrap" => style.flex_wrap(),
        "wrap-reverse" => style.flex_wrap_reverse(),
        _ => style.flex_nowrap(),
    }
}

fn justify<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "center" => style.justify_center(),
        "flex-end" | "end" => style.justify_end(),
        "space-between" => style.justify_between(),
        "space-around" => style.justify_around(),
        "space-evenly" => style.justify_evenly(),
        _ => style.justify_start(),
    }
}

fn align_items<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "center" => style.items_center(),
        "flex-end" | "end" => style.items_end(),
        "baseline" => style.items_baseline(),
        "stretch" => style.items_stretch(),
        _ => style.items_start(),
    }
}

fn align_content<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "center" => style.content_center(),
        "flex-end" | "end" => style.content_end(),
        "space-between" => style.content_between(),
        "space-around" => style.content_around(),
        "space-evenly" => style.content_evenly(),
        "stretch" => style.content_stretch(),
        _ => style.content_start(),
    }
}

fn align_self<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "center" => style.self_center(),
        "flex-start" | "start" => style.self_start(),
        "flex-end" | "end" => style.self_end(),
        "baseline" => style.self_baseline(),
        "stretch" => style.self_stretch(),
        _ => style.self_start(),
    }
}

fn border_style<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "dashed" => style.border_dashed(),
        _ => style,
    }
}

fn text_align<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "center" => style.text_center(),
        "right" | "end" => style.text_right(),
        _ => style.text_left(),
    }
}

fn text_decoration<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "underline" => style.underline(),
        "line-through" => style.line_through(),
        "none" | "no-underline" => style.text_decoration_none(),
        _ => style,
    }
}

fn overflow_x<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "hidden" | "clip" | "scroll" | "auto" => style.overflow_x_hidden(),
        _ => style,
    }
}

fn overflow_y<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "hidden" | "clip" | "scroll" | "auto" => style.overflow_y_hidden(),
        _ => style,
    }
}

fn font_family<S: Styled>(style: S, keyword: &str) -> S {
    if keyword.contains("mono") {
        style.font_family("monospace")
    } else if keyword.contains("serif") {
        style.font_family("serif")
    } else {
        style.font_family("sans-serif")
    }
}

fn cursor<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "pointer" => style.cursor_pointer(),
        "text" => style.cursor_text(),
        "move" => style.cursor_move(),
        "not-allowed" => style.cursor_not_allowed(),
        "crosshair" => style.cursor_crosshair(),
        _ => style,
    }
}

// ---------------------------------------------------------------------------
// Value helpers
// ---------------------------------------------------------------------------

fn with_num<S: Styled>(style: S, val: Val, f: impl FnOnce(S, f32) -> S) -> S {
    match val.as_num() {
        Some(n) => f(style, n),
        None => style,
    }
}

fn with_definite<S: Styled>(style: S, val: Val, f: impl FnOnce(S, DefiniteLength) -> S) -> S {
    match definite(val) {
        Some(length) => f(style, length),
        None => style,
    }
}

fn with_length<S: Styled>(style: S, val: Val, f: impl FnOnce(S, Length) -> S) -> S {
    match length(val) {
        Some(length) => f(style, length),
        None => style,
    }
}

fn with_absolute<S: Styled>(style: S, val: Val, f: impl FnOnce(S, AbsoluteLength) -> S) -> S {
    match absolute(val) {
        Some(length) => f(style, length),
        None => style,
    }
}

fn with_color<S: Styled>(style: S, val: Val, f: impl FnOnce(S, Hsla) -> S) -> S {
    match color(val) {
        Some(color) => f(style, color),
        None => style,
    }
}

fn definite(val: Val) -> Option<DefiniteLength> {
    match val {
        Val::Rem(n) => Some(rems(n).into()),
        Val::Px(n) => Some(px(n).into()),
        Val::Pct(n) => Some(relative(n / 100.0)),
        _ => None,
    }
}

fn length(val: Val) -> Option<Length> {
    match val {
        Val::Rem(n) => Some(rems(n).into()),
        Val::Px(n) => Some(px(n).into()),
        Val::Pct(n) => Some(relative(n / 100.0).into()),
        _ => None,
    }
}

fn absolute(val: Val) -> Option<AbsoluteLength> {
    match val {
        Val::Rem(n) => Some(rems(n).into()),
        Val::Px(n) => Some(px(n).into()),
        _ => None,
    }
}

fn color(val: Val) -> Option<Hsla> {
    match val {
        Val::Rgba([r, g, b, a]) => Some(rgba(u32::from_be_bytes([r, g, b, a])).into()),
        _ => None,
    }
}

/// The GPUI mapping for a bare `TextAlign` value, exposed for tests.
pub fn text_align_value(keyword: &str) -> TextAlign {
    match keyword {
        "center" => TextAlign::Center,
        "right" | "end" => TextAlign::Right,
        _ => TextAlign::Left,
    }
}

/// The `Div` alias hosts reach for when writing the README's example.
pub type TwDiv = Div;

// ---------------------------------------------------------------------------
// The frame environment (P4)
// ---------------------------------------------------------------------------

/// Refresh the probe environment from the window: dark mode and the
/// viewport width in rem. Call once per render; [`AeolusRoot`] does it
/// for you.
pub fn begin_frame(window: &gpui::Window) {
    let rem = window.rem_size();
    let viewport_rem = if rem > px(0.0) {
        window.viewport_size().width / rem
    } else {
        0.0
    };
    let dark = matches!(
        window.appearance(),
        gpui::WindowAppearance::Dark | gpui::WindowAppearance::VibrantDark
    );
    let mut current = env();
    current.dark = dark;
    current.viewport_rem = viewport_rem;
    set_env(current);
}

/// A wrapper view that refreshes [`Env`] once per render and then renders
/// the wrapped entity — the zero-argument `.tw()` ergonomics.
pub struct AeolusRoot<V: gpui::Render> {
    child: gpui::Entity<V>,
}

impl<V: gpui::Render> AeolusRoot<V> {
    /// Wrap a child view.
    pub const fn new(child: gpui::Entity<V>) -> Self {
        Self { child }
    }
}

impl<V: gpui::Render> gpui::Render for AeolusRoot<V> {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        begin_frame(window);
        self.child.clone().into_any_element()
    }
}
