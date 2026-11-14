//! The V1 static-program lane (architecture plan §7's Floem re-target —
//! the second backend after GPUI): a `tw!`-compiled
//! [`StaticProgram`](aeolus_rt::StaticProgram) walks into a Floem
//! [`Style`] under the interp's [`Env`] probe semantics.
//!
//! ```text
//! StaticProgram (layers + the ROW TEXTS the Kw slots index)
//!   └─ front-to-back walk: a layer FIRES when its Cond matches the Env
//!      ├─ fired   → its declarations apply in order (the specificity
//!      │            sort makes later-beats-earlier the cascade)
//!      ├─ quiet   → skipped (the probe says no — CSS semantics)
//!      └─ excluded→ reported as a [`LoweringReport`] drop (never silent)
//! ```
//!
//! The walk is the interp's `decide` semantics applied at the host — the
//! same shape the GPUI backend's frame walk takes (plan §3.6's "the
//! interp's `decide` semantics applied at the host"), with one
//! improvement the Floem crate's own contract calls for: every EXCLUDED
//! layer and every unmapped longhand lands in the
//! [`LoweringReport`](crate::LoweringReport), the no-silent-failures
//! channel the plan path has used since v0.1. The element-own attribute
//! probes answer through the [`AttrProbe`] seam (plan §3.5's typed state
//! bits), the theme forms ([`Val::Tok`]/[`Val::ModePair`]) through the
//! [`TokenSource`] seam (the interp's own two questions), and the rem
//! geometry through [`REM_PX`] — §9's per-backend `rem_px` constant,
//! default 16, the same resolution the v0.x resolver's scale bake uses.
//!
//! The KEYWORD texts are what makes the walk sufficient where a bare IR
//! walk is not: `hidden` and `block` both lower to `Prop::Display` +
//! `Val::Kw(0)`, and the geometry rows (`rounded-md` → `Kw(2)` with the
//! row text `0.375rem`) bake their scale into the row table — the
//! [`StaticProgram::keyword`](aeolus_rt::StaticProgram::keyword)
//! annotations recover the text, the same read the GPUI walk and the
//! `keyword_alignment` seam standardize. What the bare IR genuinely
//! cannot recover stays the documented boundary, reported per longhand:
//! the grid PLACEMENT span-vs-line distinction (the plan bridge's v0.x
//! slots carry it), the shadow preset text and the em-tracked spacing
//! (the CSS emit arm renders them) — never a silent zero.

use floem::peniko::{Brush, Color as PenikoColor, kurbo::Stroke};
use floem::style::unit::{Length, LengthAuto, Pct, Pt};
use floem::style::{CursorStyle, LineHeightValue, Style};
use floem::taffy::prelude::{
    FromFr, FromLength, GridTemplateComponent, MaxTrackSizingFunction, MinTrackSizingFunction,
    RepetitionCount, TrackSizingFunction,
};
use floem::taffy::style::Overflow;

use aeolus_ir::{Cond, Prop, Val};
use aeolus_rt::{AttrKind, AttrProbe, Env, StaticProgram, TokenSource};

use crate::LoweringReport;

/// The backend's rem size, in px (plan §9's "per-backend `rem_px`
/// constant, default 16"): every [`Val::Rem`] and every `Nrem` row text
/// resolves through this one constant, the same 16px bake the v0.x
/// resolver's spacing scale applies (1 unit = 0.25 rem = 4px, so `p-4`
/// = 1rem = 16px on both lanes — the differential pins agree).
pub const REM_PX: f32 = 16.0;

/// Walk one compiled static program into a Floem [`Style`] under the
/// frame's probe [`Env`] — the §7 re-target's lane, the program form the
/// `tw!` macro freezes and [`ClassCell`](aeolus_rt::ClassCell) pools.
///
/// Theme-token longhands ([`Val::Tok`]/[`Val::ModePair`]) refuse into
/// the report here: pass a source through
/// [`lower_program_with`] when the program carries them (the runtime
/// tier's explicit slow lane). Attribute-probe layers stay unqueried and
/// reported for the same reason.
pub fn lower_program(program: &StaticProgram, env: &Env) -> (Style, LoweringReport) {
    lower_program_with(program, env, None, None)
}

/// [`lower_program`] with the interp's two seams wired: an
/// [`AttrProbe`] source for the element-own `data-*`/`aria-*` layers
/// (the `tw_probes` convention) and a [`TokenSource`] for the theme
/// forms (the `interp_theme` convention). `None` keeps the CSS-quiet
/// semantics — an unanswered probe never fires, an unresolvable token
/// refuses into the report.
pub fn lower_program_with(
    program: &StaticProgram,
    env: &Env,
    probes: Option<&dyn AttrProbe>,
    tokens: Option<&dyn TokenSource>,
) -> (Style, LoweringReport) {
    let mut report = LoweringReport::default();
    let mut style = Style::new();
    for (layer_index, layer) in program.layers.iter().enumerate() {
        match decide(layer.cond, env, probes) {
            Fire::Yes => {
                for (decl_index, decl) in layer.decls.iter().enumerate() {
                    let keyword = program.keyword(layer_index, decl_index).unwrap_or("");
                    style = apply_decl(
                        style,
                        decl.prop,
                        decl.val,
                        keyword,
                        env,
                        tokens,
                        &mut report,
                    );
                }
            }
            Fire::No => {}
            Fire::Excluded(reason) => {
                report.drop(exclusion_text(layer.cond), reason);
            }
        }
    }
    (style, report)
}

/// The walk's three-way layer verdict — the interp's `Fire` shape, with
/// the note channel folded into the report's `(what, reason)` pair.
enum Fire {
    /// The layer matches; its declarations apply.
    Yes,
    /// The layer does not match — CSS-quiet, nothing reported (a
    /// selector that misses is CSS semantics, not a drop).
    No,
    /// The layer can never fold into the element's own cascade under
    /// this walk's inputs — reported, honest failure kept.
    Excluded(&'static str),
}

/// The interp's `decide` semantics applied at the host: the same verdict
/// per [`Cond`] the interpreter arm walks with (pseudo-elements are
/// render targets, containers without a context never match, attribute
/// probes answer through the seam, the descendant `has-*` shapes stay
/// the diagnosed boundary).
fn decide(cond: Cond, env: &Env, probes: Option<&dyn AttrProbe>) -> Fire {
    let fires = |yes: bool| if yes { Fire::Yes } else { Fire::No };
    match cond {
        Cond::Base => Fire::Yes,
        Cond::Dark => fires(env.dark),
        Cond::Hover => fires(env.hover),
        Cond::Focus => fires(env.focus),
        Cond::FocusVisible => fires(env.focus_visible),
        Cond::FocusWithin => fires(env.focus_within),
        Cond::Active => fires(env.active),
        Cond::Disabled => fires(env.disabled),
        Cond::Visited => fires(env.visited),
        Cond::Checked => fires(env.checked),
        Cond::PlaceholderShown => fires(env.placeholder_shown),
        Cond::GroupHover => fires(env.group_hover),
        Cond::GroupFocus => fires(env.group_focus),
        Cond::PeerHover => fires(env.peer_hover),
        Cond::PeerFocus => fires(env.peer_focus),
        Cond::Bp(bp) | Cond::Min(bp) => fires(env.viewport_rem >= bp.rem()),
        Cond::Max(bp) => fires(env.viewport_rem < bp.rem()),
        Cond::Container { bp, name } => match (name, env.container_rem) {
            // The unnamed question answers from the nearest eligible
            // container; a NAMED container needs THAT container's width
            // — the Env answers the nearest-container question only, so
            // the named layer is excluded with its name in the reason
            // (the interp's ContainerUnqueried boundary).
            (None, Some(width)) => fires(width >= bp.rem()),
            (Some(_), _) => Fire::Excluded(
                "the named container query needs the container's own width — the Env answers \
                 the nearest-container question only (the interp reports ContainerUnqueried)",
            ),
            (None, None) => Fire::Excluded(
                "no container context in the Env — the @container layer never fires (the \
                 interp reports ContainerUnqueried)",
            ),
        },
        Cond::MaxContainer { bp, name } => match (name, env.container_rem) {
            (None, Some(width)) => fires(width < bp.rem()),
            (Some(_), _) => Fire::Excluded(
                "the named container query needs the container's own width — the Env answers \
                 the nearest-container question only (the interp reports ContainerUnqueried)",
            ),
            (None, None) => Fire::Excluded(
                "no container context in the Env — the @max-* layer never fires (the interp \
                 reports ContainerUnqueried)",
            ),
        },
        // The element-own probes answer through the seam: presence fires
        // on any value, the value spelling on equality, an ANSWERED miss
        // is CSS-quiet. Without a source the layer is unqueried —
        // reported, never silently skipped (the interp's
        // `AttributeUnqueried` note on the walk's report channel).
        Cond::Data { name, value } => match probes {
            Some(source) => fires(match source.probe(AttrKind::Data, name) {
                Some(answer) => value.is_none_or(|folded| answer == folded),
                None => false,
            }),
            None => Fire::Excluded(
                "no AttrProbe source — pass one via lower_program_with (the interp reports \
                 AttributeUnqueried)",
            ),
        },
        Cond::Aria { name, value } => match probes {
            Some(source) => fires(match source.probe(AttrKind::Aria, name) {
                Some(answer) => value.is_none_or(|folded| answer == folded),
                None => false,
            }),
            None => Fire::Excluded(
                "no AttrProbe source — pass one via lower_program_with (the interp reports \
                 AttributeUnqueried)",
            ),
        },
        // The descendant shapes are NOT element-own attribute bits — the
        // plan §3.5 row keeps them the diagnosed boundary, with or
        // without a source.
        Cond::Has(_) | Cond::GroupHas(_) | Cond::PeerHas(_) => Fire::Excluded(
            "the descendant has-* shape is not an attribute bit — the plan §3.5 boundary (the \
             interp reports AttributeUnqueried)",
        ),
        // The structural layers ask the interp arm's StructProbe seam —
        // the element's DOM-position facts are not frame state (the Env
        // cannot answer them), and this walk consults no structural
        // source: the layer is excluded and reported (the interp's
        // `StructuralUnqueried` note on the walk's report channel).
        Cond::Structural(_) => Fire::Excluded(
            "the structural query needs the element's DOM-position facts — the interp's \
             StructProbe seam answers them (the interp reports StructuralUnqueried)",
        ),
        // The pseudo-elements style a DIFFERENT render target, never the
        // element's own cascade.
        Cond::Selection | Cond::Placeholder => Fire::Excluded(
            "a pseudo-element styles a different render target — the CSS emit arm renders it",
        ),
    }
}

/// The `what` half of an exclusion's report entry: the condition's
/// canonical text where one exists, the debug shape otherwise.
fn exclusion_text(cond: Cond) -> String {
    match cond.source() {
        Some(variant) => format!("{variant}: layer"),
        None => format!("{cond:?} layer"),
    }
}

// ---------------------------------------------------------------------------
// Declaration mapping: the V1 longhand surface → Floem setters
// ---------------------------------------------------------------------------

fn apply_decl(
    mut style: Style,
    prop: Prop,
    val: Val,
    keyword: &str,
    env: &Env,
    tokens: Option<&dyn TokenSource>,
    report: &mut LoweringReport,
) -> Style {
    match prop {
        // ---- display & positioning ------------------------------------
        Prop::Display => display(style, keyword),
        Prop::Position => position(style, keyword, report),
        Prop::Top | Prop::Right | Prop::Bottom | Prop::Left => {
            if let Some(px) = absolute_px(val, keyword, report, prop) {
                style = apply_inset(style, prop, px);
            }
            style
        }
        Prop::InsetInlineStart | Prop::InsetInlineEnd => {
            report.drop(
                format!("{prop:?}"),
                "no Floem logical-inset prop — resolve the writing mode at the call site",
            );
            style
        }
        Prop::ZIndex => match val.as_num() {
            Some(n) => style.z_index(n as i32),
            None => {
                report.drop("z-index", "the walk maps numeric z-index only");
                style
            }
        },
        Prop::Order => {
            report.drop("order", "no Floem order prop at the pinned rev");
            style
        }

        // ---- flex / grid -----------------------------------------------
        Prop::FlexDirection => flex_direction(style, keyword),
        Prop::FlexWrap => flex_wrap(style, keyword),
        Prop::FlexGrow => match val.as_num() {
            Some(n) => style.flex_grow(n),
            None => style,
        },
        Prop::FlexShrink => match shrink_value(val, keyword) {
            Some(n) => style.flex_shrink(n),
            None => style,
        },
        Prop::FlexBasis => match basis(val, keyword, report) {
            Some(v) => style.flex_basis(v),
            None => style,
        },
        Prop::JustifyContent => justify_content(style, keyword),
        Prop::JustifyItems | Prop::JustifySelf => align_keyword(style, keyword, report, prop),
        Prop::AlignItems => align_keyword(style, keyword, report, prop),
        Prop::AlignContent => align_content(style, keyword, report),
        Prop::AlignSelf => align_keyword(style, keyword, report, prop),
        Prop::PlaceContent | Prop::PlaceItems | Prop::PlaceSelf => {
            report.drop(
                format!("{prop:?}"),
                "the folded place-* pair has no two-axis Floem prop — spell the align/justify \
                 longhands",
            );
            style
        }
        Prop::GridTemplateColumns | Prop::GridTemplateRows => match val.as_num() {
            Some(count) if count > 0.0 => {
                // `grid-cols-N`: N equal `minmax(0, 1fr)` tracks in one
                // repeat — the v0.x grid-cols lowering's own shape (the
                // plan path's own test pins `repeat(3, minmax(0,1fr))`).
                let track = TrackSizingFunction {
                    min: MinTrackSizingFunction::from_length(0.0_f32),
                    max: MaxTrackSizingFunction::from_fr(1.0_f32),
                };
                let component =
                    GridTemplateComponent::Repeat(floem::taffy::style::GridTemplateRepetition {
                        count: RepetitionCount::Count(count as u16),
                        tracks: vec![track],
                        line_names: vec![],
                    });
                if prop == Prop::GridTemplateColumns {
                    style.grid_template_columns(vec![component])
                } else {
                    style.grid_template_rows(vec![component])
                }
            }
            _ => {
                report.drop(
                    format!("{prop:?}"),
                    "the walk maps the numeric N-track form; the row-text templates ride the \
                     CSS emit arm",
                );
                style
            }
        },
        Prop::GridAutoColumns | Prop::GridAutoRows => {
            report.drop(
                format!("{prop:?}"),
                "no auto-track row text in the walk's surface",
            );
            style
        }
        Prop::GridRowStart | Prop::GridRowEnd | Prop::GridColumnStart | Prop::GridColumnEnd => {
            report.drop(
                format!("{prop:?}"),
                "the span-vs-line distinction rides the row text the IR's numeric slot cannot \
                 recover — the plan bridge's v0.x slots and the CSS emit arm carry it",
            );
            style
        }
        Prop::GapRow | Prop::GapCol => match definite_px(val, keyword, report, prop) {
            Some(px) => {
                if prop == Prop::GapRow {
                    style.row_gap(pt(px))
                } else {
                    style.col_gap(pt(px))
                }
            }
            None => style,
        },
        Prop::Columns => {
            report.drop("columns", "no Floem multicol prop");
            style
        }

        // ---- box: size, margin, padding --------------------------------
        Prop::AspectRatio => match val.as_num() {
            Some(n) => style.aspect_ratio(n),
            None => style,
        },
        Prop::Width
        | Prop::Height
        | Prop::MinWidth
        | Prop::MinHeight
        | Prop::MaxWidth
        | Prop::MaxHeight => match dimension(val, keyword, report, prop) {
            Some(v) => apply_dimension(style, prop, v),
            None => style,
        },
        Prop::PaddingTop | Prop::PaddingRight | Prop::PaddingBottom | Prop::PaddingLeft => {
            match definite_px(val, keyword, report, prop) {
                Some(px) => apply_padding(style, prop, px),
                None => style,
            }
        }
        Prop::MarginTop | Prop::MarginRight | Prop::MarginBottom | Prop::MarginLeft => {
            match margin_value(val, keyword, report, prop) {
                Some(v) => apply_margin(style, prop, v),
                None => style,
            }
        }

        // ---- border & radius -------------------------------------------
        Prop::BorderTopWidth
        | Prop::BorderRightWidth
        | Prop::BorderBottomWidth
        | Prop::BorderLeftWidth => match absolute_px(val, keyword, report, prop) {
            Some(px) => apply_border_width(style, prop, px),
            None => style,
        },
        Prop::BorderColor => match color_of(val, env, tokens, prop, report) {
            Some(rgba) => {
                let c = Brush::Solid(rgba);
                style
                    .border_left_color(c.clone())
                    .border_top_color(c.clone())
                    .border_right_color(c.clone())
                    .border_bottom_color(c)
            }
            None => style,
        },
        Prop::BorderStyle => border_style(style, keyword, report),
        Prop::BorderTopLeftRadius
        | Prop::BorderTopRightRadius
        | Prop::BorderBottomRightRadius
        | Prop::BorderBottomLeftRadius => match radius_value(val, keyword, report, prop) {
            Some(length) => apply_radius(style, prop, length),
            None => style,
        },
        Prop::OutlineWidth => match outline_width(val, keyword, report) {
            Some(px) => style.outline(Stroke::new(px)),
            None => style,
        },
        Prop::OutlineColor => match color_of(val, env, tokens, prop, report) {
            Some(rgba) => style.outline_color(Brush::Solid(rgba)),
            None => style,
        },
        Prop::OutlineStyle | Prop::OutlineOffset => {
            report.drop(
                format!("{prop:?}"),
                "Floem's outline is one kurbo Stroke — style and offset have no separate prop",
            );
            style
        }
        Prop::RingOffsetWidth | Prop::RingOffsetColor => {
            report.drop(
                format!("{prop:?}"),
                "Floem's outline has no offset — the plan path reports the same boundary",
            );
            style
        }
        Prop::RingColor => match color_of(val, env, tokens, prop, report) {
            Some(rgba) => style.outline_color(Brush::Solid(rgba)),
            None => style,
        },
        Prop::RingWidth => match val.as_num() {
            // The v4 ring is the plan path's outline polyfill: ring width
            // → outline stroke spread.
            Some(n) => style.outline(Stroke::new(f64::from(n))),
            None => style,
        },

        // ---- background --------------------------------------------------
        Prop::Background => match color_of(val, env, tokens, prop, report) {
            Some(rgba) => style.background(Brush::Solid(rgba)),
            None => style,
        },
        Prop::BackgroundImage => {
            report.drop(
                "background-image",
                "gradients stay reported on floem (Brush::Gradient needs concrete geometry) — \
                 the CSS emit arm renders the stops",
            );
            style
        }
        Prop::GradientFrom
        | Prop::GradientVia
        | Prop::GradientTo
        | Prop::GradientFromPosition
        | Prop::GradientViaPosition
        | Prop::GradientToPosition => {
            report.drop(
                format!("{prop:?}"),
                "the gradient stops compose with the direction row; floem's Brush::Gradient \
                 needs concrete geometry — the plan path's Gradient slot and the CSS emit arm \
                 carry them",
            );
            style
        }
        Prop::ObjectFit | Prop::ObjectPosition => {
            report.drop(
                format!("{prop:?}"),
                "no Floem object-* prop at the pinned rev",
            );
            style
        }

        // ---- typography ---------------------------------------------------
        Prop::Color => match color_of(val, env, tokens, prop, report) {
            Some(rgba) => style.color(rgba),
            None => style,
        },
        Prop::FontSize => match absolute_px(val, keyword, report, prop) {
            Some(px) => style.font_size(Pt(px)),
            None => style,
        },
        Prop::LineHeight => match line_height(val, keyword, report) {
            Some(v) => style.line_height(v),
            None => style,
        },
        Prop::FontWeight => match weight_value(val, keyword) {
            Some(w) => style.font_weight(floem::text::FontWeight::new(w)),
            None => style,
        },
        Prop::FontFamily => {
            report.drop(
                format!("font-{keyword}"),
                "Floem font-family needs concrete family names — set via the text props",
            );
            style
        }
        Prop::FontStyle => match keyword {
            "italic" => style.font_style(floem::text::FontStyle::Italic),
            "normal" | "not-italic" => style.font_style(floem::text::FontStyle::Normal),
            other => {
                report.drop(format!("font-style {other}"), "unknown font-style row text");
                style
            }
        },
        Prop::LetterSpacing => {
            report.drop(
                "letter-spacing",
                "no Floem letter-spacing prop (the em-tracked value rides the CSS emit arm)",
            );
            style
        }
        Prop::TextAlign => text_align(style, keyword, report),
        Prop::TextTransform => {
            report.drop("text-transform", "no Floem text-transform prop");
            style
        }
        Prop::TextDecorationLine => {
            report.drop(
                "text-decoration-line",
                "no Floem text-decoration prop at the pinned rev",
            );
            style
        }
        Prop::TextUnderlineOffset => {
            report.drop("underline-offset", "no Floem underline-offset prop");
            style
        }
        Prop::WhiteSpace => whitespace(style, keyword, report),
        Prop::WordBreak | Prop::VerticalAlign => {
            report.drop(format!("{prop:?}"), "no Floem prop at the pinned rev");
            style
        }

        // ---- visual --------------------------------------------------------
        Prop::OverflowX | Prop::OverflowY => match overflow(keyword, report, prop) {
            Some(kind) => {
                if prop == Prop::OverflowX {
                    style.overflow_x(kind)
                } else {
                    style.overflow_y(kind)
                }
            }
            None => style,
        },
        Prop::OverscrollX | Prop::OverscrollY => {
            report.drop(
                format!("{prop:?}"),
                "no Floem overscroll prop at the pinned rev",
            );
            style
        }
        Prop::Visibility => {
            report.drop(
                "visibility",
                "Floem has no visibility prop — use display:none or opacity",
            );
            style
        }
        Prop::Opacity => match val.as_num() {
            // The IR carries the utility's own scale (`opacity-50` =
            // 50.0); CSS opacity is the 0..1 fraction, the same /100 the
            // v0.x resolver bakes and the GPUI walk applies.
            Some(n) => style.opacity(if n > 1.0 { n / 100.0 } else { n }),
            None => style,
        },
        Prop::Isolation => {
            report.drop("isolation", "no Floem isolation prop");
            style
        }
        Prop::BoxShadow => {
            report.drop(
                "box-shadow",
                "the preset text rides the CSS emit arm — no shadow-text parser in the walk",
            );
            style
        }
        Prop::ShadowColor => {
            report.drop(
                "shadow-color",
                "the --tw-shadow-color machinery rides the CSS emit arm",
            );
            style
        }

        // ---- interaction -----------------------------------------------------
        Prop::ContainerType => {
            report.drop(
                "container-type",
                "container queries stay the documented backend boundary (plan §3.5)",
            );
            style
        }
        Prop::Resize => {
            report.drop("resize", "no Floem resize prop");
            style
        }
        Prop::Cursor => cursor(style, keyword, report),
        Prop::PointerEvents => match keyword {
            "none" => style.pointer_events_none(),
            "auto" => style,
            other => {
                report.drop(format!("pointer-events-{other}"), "unknown row text");
                style
            }
        },
        Prop::UserSelect => match keyword {
            "none" => style.selectable(false),
            "text" | "auto" | "all" => style.selectable(true),
            other => {
                report.drop(format!("select-{other}"), "unknown user-select row text");
                style
            }
        },
        Prop::Fill | Prop::Stroke => {
            report.drop(format!("{prop:?}"), "no Floem svg paint prop in the walk");
            style
        }
        Prop::TranslateX | Prop::TranslateY => match absolute_px(val, keyword, report, prop) {
            Some(px) => {
                if prop == Prop::TranslateX {
                    style.translate_x(Length::Pt(px))
                } else {
                    style.translate_y(Length::Pt(px))
                }
            }
            None => style,
        },
        Prop::Rotate => match val.as_num() {
            Some(deg) => style.rotate(floem::style::unit::Angle::Deg(f64::from(deg))),
            None => style,
        },
        Prop::ScaleX | Prop::ScaleY => match val.as_num() {
            Some(n) => {
                let pct = Pct(f64::from(n) * 100.0);
                if prop == Prop::ScaleX {
                    style.scale_x(pct)
                } else {
                    style.scale_y(pct)
                }
            }
            None => style,
        },
        Prop::TextOverflow => match keyword {
            "ellipsis" => style.text_overflow(floem::style::TextOverflow::NoWrap(
                floem::style::NoWrapOverflow::Ellipsis,
            )),
            "clip" => style.text_overflow(floem::style::TextOverflow::NoWrap(
                floem::style::NoWrapOverflow::Clip,
            )),
            other => {
                report.drop(
                    format!("text-overflow {other}"),
                    "unknown text-overflow row text",
                );
                style
            }
        },
        Prop::SkewX | Prop::SkewY => {
            report.drop(
                format!("{prop:?}"),
                "the --tw-* machinery value rides the CSS emit arm (the plan path reports the \
                 same boundary)",
            );
            style
        }
        // ---- motion (the plan P6 row) ----------------------------------
        //
        // Floem's pinned-rev Style carries no transition or animation
        // surface — the motion longhands ride the CSS emit arm (the
        // plan path reports the same boundary; the plan bridge folds
        // them into the oracle's Transition/Animation extras).
        Prop::TransitionProperty
        | Prop::TransitionDuration
        | Prop::TransitionTimingFunction
        | Prop::TransitionDelay
        | Prop::Animation => {
            report.drop(
                format!("{prop:?}"),
                "no transition/animation surface at the pinned rev — the motion families ride                  the CSS emit arm",
            );
            style
        }
    }
}

// ---------------------------------------------------------------------------
// Value helpers
// ---------------------------------------------------------------------------

/// The px value a `Val` (or its row text) carries, when it is an
/// ABSOLUTE length: rem resolves through [`REM_PX`], px rides verbatim,
/// and a `Val::Kw`'s row text parses in the registry's own spellings
/// (`0.375rem`, `1px`). Percentages and relative units return `None`
/// here — the per-prop mappings that accept them ask separately.
fn absolute_px(val: Val, keyword: &str, report: &mut LoweringReport, prop: Prop) -> Option<f64> {
    let what = format!("{prop:?}");
    match val {
        Val::Rem(n) => Some(f64::from(n * REM_PX)),
        Val::Px(n) => Some(f64::from(n)),
        // The registry's px-shaped numerics (border-2 → Num(2.0)) ride
        // as px — the same reading the plan bridge's width pass takes.
        Val::Num(n) => Some(f64::from(n)),
        Val::Kw(_) => match parse_length_text(keyword) {
            ParsedLength::Px(px) => Some(px),
            ParsedLength::Pct(_) | ParsedLength::Other => {
                report.drop(
                    what,
                    format!("the row text {keyword:?} is not an absolute length"),
                );
                None
            }
        },
        other => {
            report.drop(
                what,
                format!("the value {other:?} is not an absolute length"),
            );
            None
        }
    }
}

/// The DEFINITE px a spacing-family `Val` carries (padding, gap): rem
/// through [`REM_PX`], px verbatim, keyword row text parsed. Percentages
/// report — the walk's spacing setters take points, and a percentage
/// spacing class (`p-1/2`) is not a registry form.
fn definite_px(val: Val, keyword: &str, report: &mut LoweringReport, prop: Prop) -> Option<f64> {
    match val {
        Val::Rem(n) => Some(f64::from(n * REM_PX)),
        Val::Px(n) => Some(f64::from(n)),
        Val::Kw(_) => match parse_length_text(keyword) {
            ParsedLength::Px(px) => Some(px),
            ParsedLength::Pct(_) | ParsedLength::Other => {
                report.drop(
                    format!("{prop:?}"),
                    format!("the row text {keyword:?} is not a definite length"),
                );
                None
            }
        },
        other => {
            report.drop(
                format!("{prop:?}"),
                format!("the value {other:?} is not a definite length"),
            );
            None
        }
    }
}

/// A sizing dimension in Floem's [`LengthAuto`] form: lengths resolve to
/// points, percentages ride, keyword row text parses (`auto` included).
fn dimension(
    val: Val,
    keyword: &str,
    report: &mut LoweringReport,
    prop: Prop,
) -> Option<LengthAuto> {
    let what = format!("{prop:?}");
    match val {
        Val::Rem(n) => Some(LengthAuto::Pt(f64::from(n * REM_PX))),
        Val::Px(n) => Some(LengthAuto::Pt(f64::from(n))),
        Val::Pct(n) => Some(LengthAuto::Pct(f64::from(n))),
        Val::Kw(_) => match keyword {
            "auto" => Some(LengthAuto::Auto),
            _ => match parse_length_text(keyword) {
                ParsedLength::Px(px) => Some(LengthAuto::Pt(px)),
                ParsedLength::Pct(p) => Some(LengthAuto::Pct(p)),
                ParsedLength::Other => {
                    report.drop(what, format!("the row text {keyword:?} is not a size"));
                    None
                }
            },
        },
        other => {
            report.drop(what, format!("the value {other:?} is not a size"));
            None
        }
    }
}

/// A margin value: lengths and percentages ride, and the `auto` row text
/// is Floem's own [`LengthAuto::Auto`] (`mx-auto`).
fn margin_value(
    val: Val,
    keyword: &str,
    report: &mut LoweringReport,
    prop: Prop,
) -> Option<LengthAuto> {
    match val {
        Val::Kw(_) if keyword == "auto" => Some(LengthAuto::Auto),
        _ => dimension(val, keyword, report, prop),
    }
}

/// A flex-basis value: the `auto` keyword plus the size forms.
fn basis(val: Val, keyword: &str, report: &mut LoweringReport) -> Option<LengthAuto> {
    margin_value(val, keyword, report, Prop::FlexBasis)
}

/// A radius in Floem's [`Length`] form: lengths resolve, the registry's
/// percentage text (`50%`) rides, `9999px` stays the rounded-full
/// sentinel it is.
fn radius_value(
    val: Val,
    keyword: &str,
    report: &mut LoweringReport,
    prop: Prop,
) -> Option<Length> {
    match val {
        Val::Rem(n) => Some(Length::Pt(f64::from(n * REM_PX))),
        Val::Px(n) => Some(Length::Pt(f64::from(n))),
        Val::Pct(n) => Some(Length::Pct(f64::from(n))),
        Val::Kw(_) => match parse_length_text(keyword) {
            ParsedLength::Px(px) => Some(Length::Pt(px)),
            ParsedLength::Pct(p) => Some(Length::Pct(p)),
            ParsedLength::Other => {
                report.drop(
                    format!("{prop:?}"),
                    format!("the radius row text {keyword:?} is not a length"),
                );
                None
            }
        },
        other => {
            report.drop(
                format!("{prop:?}"),
                format!("the value {other:?} is not a radius"),
            );
            None
        }
    }
}

/// A font-weight value: the row's numeric text (`font-medium` → `500`)
/// or a bare [`Val::Num`].
fn weight_value(val: Val, keyword: &str) -> Option<f32> {
    match val {
        Val::Num(n) => Some(n),
        Val::Kw(_) => keyword.parse::<f32>().ok(),
        _ => None,
    }
}

/// A flex-shrink value: `shrink`'s keyword text is `1` (the bare class
/// fans to the numeric default), `shrink-0` a [`Val::Num`].
fn shrink_value(val: Val, keyword: &str) -> Option<f32> {
    match val {
        Val::Num(n) => Some(n),
        Val::Kw(_) => keyword.parse::<f32>().ok(),
        _ => None,
    }
}

/// A line-height in Floem's [`LineHeightValue`] form: rem text resolves
/// to points, a bare number is the unitless ratio, and the composite
/// `text-*` rows' `calc(a / b)` ratio text evaluates — the one row-text
/// shape the registry's font-size rows carry. Anything else reports.
fn line_height(val: Val, keyword: &str, report: &mut LoweringReport) -> Option<LineHeightValue> {
    match val {
        Val::Rem(n) => Some(LineHeightValue::Pt(n * REM_PX)),
        Val::Num(n) => Some(LineHeightValue::Normal(n)),
        Val::Kw(_) => {
            if let Some(ratio) = parse_ratio_text(keyword) {
                return Some(LineHeightValue::Normal(ratio));
            }
            if let ParsedLength::Px(px) = parse_length_text(keyword) {
                return Some(LineHeightValue::Pt(px as f32));
            }
            report.drop(
                "line-height",
                format!("the row text {keyword:?} is not a ratio or length"),
            );
            None
        }
        other => {
            report.drop(
                "line-height",
                format!("the value {other:?} is not a line-height"),
            );
            None
        }
    }
}

/// The concrete colour a value slot carries, through the theme seam when
/// the program rides the runtime tiers. [`Val::Rgba`] answers directly;
/// [`Val::Tok`]/[`Val::ModePair`] ask the [`TokenSource`] (the interp's
/// own two questions, the mode riding every ask); a refusal — no source
/// wired, an unknown slot — reports with the prop named and returns
/// `None`, so the longhand stays visibly unset rather than transparent.
fn color_of(
    val: Val,
    env: &Env,
    tokens: Option<&dyn TokenSource>,
    prop: Prop,
    report: &mut LoweringReport,
) -> Option<PenikoColor> {
    let what = format!("{prop:?}");
    let resolved: Result<[u8; 4], String> = match val {
        Val::Rgba(rgba) => Ok(rgba),
        Val::Tok { id, alpha } => match tokens {
            Some(source) => match source.resolve(id, alpha, env.dark) {
                Ok(Val::Rgba(rgba)) => Ok(rgba),
                Ok(other) => Err(format!("the source answered {other:?}, not a colour")),
                Err(err) => Err(err.to_string()),
            },
            None => Err(
                "no TokenSource — pass one via lower_program_with (the runtime tier's \
                         explicit slow lane)"
                    .to_string(),
            ),
        },
        Val::ModePair(slot) => match tokens {
            Some(source) => match source.resolve_pair(slot, env.dark) {
                Ok(Val::Rgba(rgba)) => Ok(rgba),
                Ok(other) => Err(format!("the source answered {other:?}, not a colour")),
                Err(err) => Err(err.to_string()),
            },
            None => Err(
                "no TokenSource — pass one via lower_program_with (the static dark \
                         pair rides the baked table)"
                    .to_string(),
            ),
        },
        other => Err(format!("the value {other:?} is not a colour")),
    };
    match resolved {
        Ok(rgba) => Some(PenikoColor::new([
            f32::from(rgba[0]) / 255.0,
            f32::from(rgba[1]) / 255.0,
            f32::from(rgba[2]) / 255.0,
            f32::from(rgba[3]) / 255.0,
        ])),
        Err(reason) => {
            report.drop(what, reason);
            None
        }
    }
}

/// The row-text length spellings the registry's keyword tables carry.
enum ParsedLength {
    /// An absolute length in px (`0.375rem` under [`REM_PX`], `1px`).
    Px(f64),
    /// A percentage (`50%`).
    Pct(f64),
    /// Anything else (em-tracked text, `var(--…)` presets, `auto`).
    Other,
}

/// Parse one row-text length: `Nrem` through [`REM_PX`], `Npx` verbatim,
/// `N%` as a percentage. A bare number is the registry's px spelling.
fn parse_length_text(text: &str) -> ParsedLength {
    let text = text.trim();
    // Slice at the first non-numeric char — `find`, not `split_once`
    // (split_once CONSUMES the separator, which would eat the unit's
    // first letter: "1px" would read unit "x").
    let (number, unit) = match text.find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-') {
        Some(at) => (&text[..at], &text[at..]),
        None => (text, ""),
    };
    let Ok(value) = number.parse::<f64>() else {
        return ParsedLength::Other;
    };
    match unit {
        "rem" => ParsedLength::Px(value * f64::from(REM_PX)),
        "px" | "" => ParsedLength::Px(value),
        "%" => ParsedLength::Pct(value),
        _ => ParsedLength::Other,
    }
}

/// Parse the composite `text-*` rows' line-height ratio text: the bare
/// number (`1.5`) and the `calc(a / b)` shape (`calc(1.25 / 0.875)`).
fn parse_ratio_text(text: &str) -> Option<f32> {
    let text = text.trim();
    if let Ok(ratio) = text.parse::<f32>() {
        return Some(ratio);
    }
    let inner = text.strip_prefix("calc(")?.strip_suffix(')')?;
    let (a, b) = inner.split_once('/')?;
    let a = a.trim().parse::<f32>().ok()?;
    let b = b.trim().parse::<f32>().ok()?;
    (b != 0.0).then(|| a / b)
}

// ---------------------------------------------------------------------------
// Keyword mappings (the enum rows the backends map natively)
// ---------------------------------------------------------------------------

fn display(style: Style, keyword: &str) -> Style {
    match keyword {
        "none" => style.display(floem::style::Display::None),
        "grid" | "inline-grid" => style.display(floem::style::Display::Grid),
        // Block/inline have no dedicated Floem display; the plan path's
        // documented normalization maps them onto the flex flow.
        _ => style.display(floem::style::Display::Flex),
    }
}

fn position(style: Style, keyword: &str, report: &mut LoweringReport) -> Style {
    match keyword {
        "absolute" | "fixed" => style.absolute(),
        "relative" | "static" | "sticky" => style,
        other => {
            report.drop(format!("position {other}"), "unknown position row text");
            style
        }
    }
}

fn flex_direction(style: Style, keyword: &str) -> Style {
    match keyword {
        "column" => style.flex_direction(floem::style::FlexDirection::Column),
        "column-reverse" => style.flex_direction(floem::style::FlexDirection::ColumnReverse),
        "row-reverse" => style.flex_direction(floem::style::FlexDirection::RowReverse),
        _ => style.flex_direction(floem::style::FlexDirection::Row),
    }
}

fn flex_wrap(style: Style, keyword: &str) -> Style {
    match keyword {
        "wrap" => style.flex_wrap(floem::style::FlexWrap::Wrap),
        "wrap-reverse" => style.flex_wrap(floem::style::FlexWrap::WrapReverse),
        _ => style.flex_wrap(floem::style::FlexWrap::NoWrap),
    }
}

fn justify_content(style: Style, keyword: &str) -> Style {
    let j = match keyword {
        "center" => floem::style::JustifyContent::Center,
        "flex-end" | "end" => floem::style::JustifyContent::FlexEnd,
        "space-between" => floem::style::JustifyContent::SpaceBetween,
        "space-around" => floem::style::JustifyContent::SpaceAround,
        "space-evenly" => floem::style::JustifyContent::SpaceEvenly,
        _ => floem::style::JustifyContent::FlexStart,
    };
    style.justify_content(j)
}

/// The align-items/self/items keyword family — the same enum the plan
/// path's `map_align` targets.
fn align_keyword(style: Style, keyword: &str, report: &mut LoweringReport, prop: Prop) -> Style {
    let a = match keyword {
        "center" => floem::style::AlignItems::Center,
        "flex-end" | "end" => floem::style::AlignItems::FlexEnd,
        "baseline" => floem::style::AlignItems::Baseline,
        "stretch" => floem::style::AlignItems::Stretch,
        "flex-start" | "start" => floem::style::AlignItems::FlexStart,
        other => {
            report.drop(format!("{prop:?} {other}"), "unknown align row text");
            return style;
        }
    };
    match prop {
        Prop::JustifyItems => style.justify_items(a),
        Prop::JustifySelf | Prop::AlignSelf => style.align_self(a),
        _ => style.align_items(a),
    }
}

fn align_content(style: Style, keyword: &str, report: &mut LoweringReport) -> Style {
    let a = match keyword {
        "center" => floem::style::AlignContent::Center,
        "flex-end" | "end" => floem::style::AlignContent::FlexEnd,
        "space-between" => floem::style::AlignContent::SpaceBetween,
        "space-around" => floem::style::AlignContent::SpaceAround,
        "space-evenly" => floem::style::AlignContent::SpaceEvenly,
        "stretch" => floem::style::AlignContent::Stretch,
        "flex-start" | "start" => floem::style::AlignContent::FlexStart,
        // taffy has no baseline align-content — the plan path's boundary.
        "baseline" => {
            report.drop("content-baseline", "taffy has no baseline align-content");
            return style;
        }
        other => {
            report.drop(
                format!("align-content {other}"),
                "unknown align-content row text",
            );
            return style;
        }
    };
    style.align_content(a)
}

fn text_align(style: Style, keyword: &str, report: &mut LoweringReport) -> Style {
    let a = match keyword {
        "left" | "start" => floem::text::Alignment::Start,
        "center" => floem::text::Alignment::Center,
        "right" | "end" => floem::text::Alignment::End,
        "justify" => floem::text::Alignment::Justify,
        other => {
            report.drop(format!("text-align {other}"), "unknown alignment row text");
            return style;
        }
    };
    style.text_align(a)
}

/// The single `TextOverflow` prop carries the wrap mode (the plan path's
/// v0.6 §2.3 mapping): `whitespace-normal` wraps, `nowrap` is floem's
/// default, the `pre*` forms report the missing preservation control.
fn whitespace(style: Style, keyword: &str, report: &mut LoweringReport) -> Style {
    match keyword {
        "normal" => style.text_overflow(floem::style::TextOverflow::Wrap {
            overflow_wrap: floem::text::OverflowWrap::Normal,
            word_break: floem::text::WordBreakStrength::Normal,
        }),
        "nowrap" => style,
        other => {
            report.drop(
                format!("whitespace-{other}"),
                "no whitespace-preservation control in floem (the nowrap half is floem's \
                 default)",
            );
            style
        }
    }
}

fn overflow(keyword: &str, report: &mut LoweringReport, prop: Prop) -> Option<Overflow> {
    match keyword {
        "visible" => Some(Overflow::Visible),
        "hidden" => Some(Overflow::Hidden),
        "clip" => Some(Overflow::Clip),
        // taffy has no auto; the plan path's documented normalization.
        "scroll" | "auto" => Some(Overflow::Scroll),
        other => {
            report.drop(format!("{prop:?} {other}"), "unknown overflow row text");
            None
        }
    }
}

fn cursor(style: Style, keyword: &str, report: &mut LoweringReport) -> Style {
    let c = match keyword {
        "default" | "auto" => CursorStyle::Default,
        "pointer" => CursorStyle::Pointer,
        "wait" => CursorStyle::Wait,
        "progress" => CursorStyle::Progress,
        "text" | "vertical-text" => CursorStyle::Text,
        "move" | "all-scroll" => CursorStyle::Move,
        "grab" => CursorStyle::Grab,
        "grabbing" => CursorStyle::Grabbing,
        "col-resize" => CursorStyle::ColResize,
        "row-resize" => CursorStyle::RowResize,
        "n-resize" => CursorStyle::NResize,
        "e-resize" => CursorStyle::EResize,
        "s-resize" => CursorStyle::SResize,
        "w-resize" => CursorStyle::WResize,
        "nw-resize" => CursorStyle::NwResize,
        "ne-resize" => CursorStyle::NeResize,
        "sw-resize" => CursorStyle::SwResize,
        "se-resize" => CursorStyle::SeResize,
        "nesw-resize" => CursorStyle::NeswResize,
        "nwse-resize" => CursorStyle::NwseResize,
        "crosshair" => CursorStyle::Crosshair,
        // Floem's `CursorStyle` has no not-allowed/none/context-menu
        // shape — the plan path's own map_cursor normalizes them onto
        // `Default` (documented, a real cursor rather than a drop).
        "not-allowed" | "none" | "context-menu" | "help" | "cell" | "alias" | "copy"
        | "no-drop" | "zoom-in" | "zoom-out" => CursorStyle::Default,
        // The registry's cursor rows spell the CSS set; anything outside
        // the walk's table reports (never a silent default).
        other => {
            report.drop(
                format!("cursor-{other}"),
                "no Floem CursorStyle for this row text",
            );
            return style;
        }
    };
    style.cursor(c)
}

/// The border STYLE strokes (the plan path's v0.6 §2.6 kurbo mapping):
/// dashes on the current side strokes, the width preserved from the
/// width longhands the cascade applied before the style.
fn border_style(style: Style, keyword: &str, report: &mut LoweringReport) -> Style {
    use floem::peniko::kurbo::Cap;
    let dashed = |w: f64| Stroke::new(w).with_dashes(0., [6., 6.]);
    let dotted = |w: f64| {
        Stroke::new(w)
            .with_caps(Cap::Round)
            .with_dashes(0., [0.01, (w * 2.0).max(0.1)])
    };
    match keyword {
        "solid" => style,
        "dashed" | "dotted" => {
            let paint = |s: Style, get: fn(&Style) -> Stroke, set: fn(Style, Stroke) -> Style| {
                let width = get(&s).width;
                let stroke = if keyword == "dashed" {
                    dashed(width)
                } else {
                    dotted(width)
                };
                set(s, stroke)
            };
            let s = paint(
                style,
                |st| st.get(floem::style::BorderLeft),
                Style::border_left,
            );
            let s = paint(s, |st| st.get(floem::style::BorderTop), Style::border_top);
            let s = paint(
                s,
                |st| st.get(floem::style::BorderRight),
                Style::border_right,
            );
            paint(
                s,
                |st| st.get(floem::style::BorderBottom),
                Style::border_bottom,
            )
        }
        "none" | "hidden" => {
            let zero = Stroke::new(0.0);
            style
                .border_left(zero.clone())
                .border_top(zero.clone())
                .border_right(zero.clone())
                .border_bottom(zero)
        }
        other => {
            report.drop(
                format!("border-{other}"),
                "the walk maps solid/dashed/dotted/none/hidden (double reports on the plan \
                 path too)",
            );
            style
        }
    }
}

// ---------------------------------------------------------------------------
// Setter shims (one per side/edge family)
// ---------------------------------------------------------------------------

fn apply_inset(style: Style, prop: Prop, px: f64) -> Style {
    match prop {
        Prop::Top => style.inset_top(LengthAuto::Pt(px)),
        Prop::Right => style.inset_right(LengthAuto::Pt(px)),
        Prop::Bottom => style.inset_bottom(LengthAuto::Pt(px)),
        _ => style.inset_left(LengthAuto::Pt(px)),
    }
}

fn apply_dimension(style: Style, prop: Prop, v: LengthAuto) -> Style {
    match prop {
        Prop::Width => style.width(v),
        Prop::Height => style.height(v),
        Prop::MinWidth => style.min_width(v),
        Prop::MinHeight => style.min_height(v),
        Prop::MaxWidth => style.max_width(v),
        _ => style.max_height(v),
    }
}

fn apply_padding(style: Style, prop: Prop, px: f64) -> Style {
    let v = pt(px);
    match prop {
        Prop::PaddingTop => style.padding_top(v),
        Prop::PaddingRight => style.padding_right(v),
        Prop::PaddingBottom => style.padding_bottom(v),
        _ => style.padding_left(v),
    }
}

fn apply_margin(style: Style, prop: Prop, v: LengthAuto) -> Style {
    match prop {
        Prop::MarginTop => style.margin_top(v),
        Prop::MarginRight => style.margin_right(v),
        Prop::MarginBottom => style.margin_bottom(v),
        _ => style.margin_left(v),
    }
}

/// One side stroke at `px`, PRESERVING the dash pattern a border-style
/// longhand painted earlier in the cascade (`border-dashed border-2`
/// keeps its dashes). The longhand names ONE side — the IR's `border`
/// shorthand fans to four longhand declarations, so the per-side setter
/// is the whole contract.
fn apply_border_width(style: Style, prop: Prop, px: f64) -> Style {
    let mut stroke = match prop {
        Prop::BorderTopWidth => style.get(floem::style::BorderTop),
        Prop::BorderRightWidth => style.get(floem::style::BorderRight),
        Prop::BorderBottomWidth => style.get(floem::style::BorderBottom),
        _ => style.get(floem::style::BorderLeft),
    };
    stroke.width = px;
    match prop {
        Prop::BorderTopWidth => style.border_top(stroke),
        Prop::BorderRightWidth => style.border_right(stroke),
        Prop::BorderBottomWidth => style.border_bottom(stroke),
        _ => style.border_left(stroke),
    }
}

fn apply_radius(style: Style, prop: Prop, v: Length) -> Style {
    match prop {
        Prop::BorderTopLeftRadius => style.border_top_left_radius(v),
        Prop::BorderTopRightRadius => style.border_top_right_radius(v),
        Prop::BorderBottomRightRadius => style.border_bottom_right_radius(v),
        _ => style.border_bottom_left_radius(v),
    }
}

/// Tailwind px → floem [`Pt`], the plan path's own unit audit applying:
/// the pinned rev's `Pt` is a logical layout point, so `p-4` = 16px at
/// 96dpi — see the plan path's `pt` documentation.
fn pt(px: f64) -> Length {
    Length::Pt(px)
}

/// The outline-width value: `outline-none`'s keyword text is `none` (a
/// zero stroke, the plan path's own arm), numeric widths ride.
fn outline_width(val: Val, keyword: &str, report: &mut LoweringReport) -> Option<f64> {
    match val {
        Val::Num(n) => Some(f64::from(n)),
        Val::Kw(_) if keyword == "none" => Some(0.0),
        Val::Kw(_) => match parse_length_text(keyword) {
            ParsedLength::Px(px) => Some(px),
            _ => {
                report.drop(
                    "outline-width",
                    format!("the row text {keyword:?} is not an outline width"),
                );
                None
            }
        },
        other => {
            report.drop(
                "outline-width",
                format!("the value {other:?} is not an outline width"),
            );
            None
        }
    }
}
