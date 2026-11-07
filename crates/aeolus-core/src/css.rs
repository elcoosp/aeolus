//! The verification CSS emitter (v0.4 §1, `AEOLUS_V0.4.md`).
//!
//! [`Resolver::to_css`] derives the Tailwind-v4-equivalent CSS of a class
//! string from the SAME resolution the backends consume: routing
//! ([`crate::cond`] conditions) and application (`Applied` values) feed a
//! per-token rule emitter — one [`CssRule`] per class token, v4's
//! one-rule-per-candidate shape.
//!
//! This is a VERIFICATION and debugging tool: it proves the pipeline
//! (parse → theme → plan) against genuine `tailwindcss` output via the
//! golden corpus (`tests/parity_golden.rs`), and it answers "what did the
//! compiler understand?" for debugging. Native backends keep consuming the
//! `StylePlan` directly; CSS emission is not a styling path.
//!
//! Honest scope: a class the resolver cannot apply produces the same
//! `Warning` it always did — nothing is invented here. The value shapes
//! mirror v4's output (`calc(var(--spacing) * n)`, oklch colors,
//! `color-mix(in oklab, …)` for opacity modifiers, composed `box-shadow`
//! for rings and shadow presets) so the corpus comparison is semantic, not
//! byte-oriented. Where the IR resolves past the CSS form (spacing scales
//! become px, `rounded-full` becomes the finite 9999px sentinel), the
//! token text restores the v4 shape.

use crate::ast::{ClassToken, Media, State, Structural};
use crate::color::Color;
use crate::cond::Test;
use crate::plan::{
    AlignItems, AutoEdges, BoxShadow, ContentAlign, Cursor, Display, Ease, FlexDirection,
    JustifyContent, OverflowKind, PointerEvents, Position, Ring, SizedValue, TextProperty,
    TextValue, TransitionSet, VariantCondition, Visibility,
};
use crate::resolve::{
    Applied, GradientEdge, GradientShapeApplied, PartialValue, Resolver, Route, Warning,
};

/// One emitted CSS rule: `.selector` under optional `@media`/`@container`
/// conditions with an ordered declaration list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRule {
    /// Fully-qualified selector, e.g. `.hover\:bg-red-600:hover`.
    pub selector: String,
    /// The `@media` condition without the keyword, e.g. `(width >= 48rem)`.
    pub media: Option<String>,
    /// The `@container` condition without the keyword.
    pub container: Option<String>,
    /// The named container (`@md/name`), when any.
    pub container_name: Option<String>,
    /// `(property, value)` pairs in emission order.
    pub declarations: Vec<(String, String)>,
}

/// The result of [`Resolver::to_css`]: one rule per resolvable class token
/// plus the usual non-fatal warnings.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CssOutput {
    /// The emitted rules (one per class token, source order).
    pub rules: Vec<CssRule>,
    /// Unsupported/unknown/unparseable classes — the resolver's own
    /// diagnostics, never suppressed (the no-silent-failures rule).
    pub warnings: Vec<Warning>,
}

impl CssRule {
    /// Pretty-print this rule (debug output; the corpus harness parses the
    /// structured fields instead).
    pub fn to_css(&self) -> String {
        let mut s = String::new();
        if let Some(c) = &self.container {
            s.push_str("@container");
            if let Some(n) = &self.container_name {
                s.push('/');
                s.push_str(n);
            }
            s.push_str(&format!(" {c} {{\n"));
        }
        if let Some(m) = &self.media {
            s.push_str(&format!("@media {m} {{\n"));
        }
        s.push_str(&format!("  {} {{\n", self.selector));
        for (p, v) in &self.declarations {
            s.push_str(&format!("    {p}: {v};\n"));
        }
        s.push_str("  }\n");
        if self.media.is_some() {
            s.push_str("}\n");
        }
        if self.container.is_some() {
            s.push_str("}\n");
        }
        s
    }
}

/// Pretty-print a whole [`CssOutput`].
pub fn format_rules(out: &CssOutput) -> String {
    let mut s = String::new();
    for r in &out.rules {
        s.push_str(&r.to_css());
    }
    s
}

impl<'t> Resolver<'t> {
    /// Emit the Tailwind-v4-equivalent CSS for a class string (v0.4 §1).
    ///
    /// One [`CssRule`] per class token, derived from the resolver's own
    /// routing and application — the same resolution backends consume.
    /// Classes the resolver reports (`Unsupported`/`Unknown`/…) produce no
    /// rule and keep their warning.
    pub fn to_css(
        &self,
        classes: &str,
    ) -> Result<crate::css::CssOutput, Vec<crate::color::ParseError>> {
        let expanded = self.expand_utilities(classes, 0);
        let (tokens, unparseable) = crate::parser::parse_class_string(&expanded);
        let mut out = CssOutput::default();
        for (class, message) in unparseable {
            out.warnings.push(Warning::Unparseable { class, message });
        }
        for token in tokens {
            let route = self.route(&token);
            if let Route::Skip(w) = route {
                out.warnings.push(w);
                continue;
            }
            let applied = match self.apply_utility(&token) {
                Ok(a) => a,
                Err(w) => {
                    out.warnings.push(w);
                    continue;
                }
            };
            let mut rule = self.css_rule(&token, &route, &applied);
            if token.important {
                rule.declarations = rule
                    .declarations
                    .into_iter()
                    .map(|(p, v)| (p, format!("{v} !important")))
                    .collect();
            }
            out.rules.push(rule);
        }
        Ok(out)
    }

    fn css_rule(&self, token: &ClassToken<'_>, route: &Route, applied: &Applied) -> CssRule {
        let class_sel = format!(".{}", escape_class(token.raw));
        let mut rule = CssRule {
            selector: class_sel.clone(),
            media: None,
            container: None,
            container_name: None,
            declarations: self.declarations(token, applied),
        };
        match route {
            Route::Base => {}
            Route::Layer(c) => rule.selector.push_str(state_suffix_css(*c)),
            Route::Responsive(bp) => {
                let px = self.theme.breakpoint_px(*bp);
                rule.media = Some(min_width(px));
            }
            Route::Dark => {
                rule.media = Some("(prefers-color-scheme: dark)".into());
            }
            Route::Group(label) => {
                let (label, state) = split_group(label);
                rule.selector
                    .push_str(&group_suffix("group", &label, &state, ""));
            }
            Route::Peer(label) => {
                let (label, state) = split_group(label);
                rule.selector
                    .push_str(&group_suffix("peer", &label, &state, " ~"));
            }
            Route::Rules(cond) => {
                let mut media: Vec<String> = Vec::new();
                let mut container: Vec<String> = Vec::new();
                let mut name: Option<String> = None;
                let mut tag: Option<String> = None;
                for atom in &cond.0 {
                    match atom {
                        crate::cond::Atom::On(Test::State(s)) => {
                            rule.selector.push_str(state_pseudo(*s));
                        }
                        crate::cond::Atom::On(Test::Data(n, v)) => {
                            rule.selector.push_str(&attr_test("data", n, v.as_deref()));
                        }
                        crate::cond::Atom::On(Test::Aria(n, v)) => {
                            rule.selector.push_str(&attr_test("aria", n, v.as_deref()));
                        }
                        crate::cond::Atom::On(Test::Structural(s)) => {
                            rule.selector.push_str(&structural_pseudo(*s));
                        }
                        crate::cond::Atom::On(Test::Tag(t)) => {
                            // v4 emits the tag as a selector PREFIX: `a.\[a\&\]`.
                            tag = Some(t.to_string());
                        }
                        crate::cond::Atom::Not(inner) => {
                            rule.selector.push_str(&not_test(inner));
                        }
                        crate::cond::Atom::Has(s) => {
                            rule.selector
                                .push_str(&format!(":has(*{})", state_pseudo(*s)));
                        }
                        crate::cond::Atom::Group(test, label) => {
                            let state = match test {
                                Test::State(s) => state_pseudo(*s).to_string(),
                                _ => String::new(),
                            };
                            let l = label.as_deref().unwrap_or("");
                            rule.selector
                                .push_str(&group_suffix("group", l, &state, ""));
                        }
                        crate::cond::Atom::Peer(test, label) => {
                            let state = match test {
                                Test::State(s) => state_pseudo(*s).to_string(),
                                _ => String::new(),
                            };
                            let l = label.as_deref().unwrap_or("");
                            rule.selector
                                .push_str(&group_suffix("peer", l, &state, " ~"));
                        }
                        // `rtl:`/`ltr:` are selector-based variants in v4,
                        // not media queries.
                        crate::cond::Atom::Media(Media::Rtl) => {
                            rule.selector
                                .push_str(":where(:dir(rtl), [dir=\"rtl\"], [dir=\"rtl\"] *)");
                        }
                        crate::cond::Atom::Media(Media::Ltr) => {
                            rule.selector
                                .push_str(":where(:dir(ltr), [dir=\"ltr\"], [dir=\"ltr\"] *)");
                        }
                        crate::cond::Atom::Media(m) => media.push(media_cond(*m)),
                        crate::cond::Atom::Viewport { min, max } => {
                            let mut parts: Vec<String> = Vec::new();
                            if let Some(px) = min {
                                parts.push(min_width(*px));
                            }
                            if let Some(px) = max {
                                parts.push(max_width(*px));
                            }
                            media.push(parts.join(" and "));
                        }
                        crate::cond::Atom::Container { min, max, name: n } => {
                            let mut parts: Vec<String> = Vec::new();
                            if let Some(px) = min {
                                parts.push(min_width(*px));
                            }
                            if let Some(px) = max {
                                parts.push(max_width(*px));
                            }
                            container.push(parts.join(" and "));
                            if let Some(n) = n {
                                name = Some(n.to_string());
                            }
                        }
                    }
                }
                if let Some(t) = tag {
                    rule.selector = format!("{t}{}", rule.selector);
                }
                rule.media = non_empty(media.join(" and "));
                rule.container = non_empty(container.join(" and "));
                rule.container_name = name;
            }
            Route::Skip(_) => unreachable!("skipped routes never reach the emitter"),
        }
        rule
    }

    /// The declaration list for one applied utility (v0.4 §1.2).
    fn declarations(&self, token: &ClassToken<'_>, applied: &Applied) -> Vec<(String, String)> {
        let spacing = self.theme.spacing();
        match applied {
            Applied::Replace(_, PartialValue::Background(bg)) => {
                vec![("background-color".into(), self.color_text(token, &bg.0))]
            }
            Applied::Replace(_, PartialValue::ContentColor(c)) => {
                vec![("color".into(), self.color_text(token, &c.0))]
            }
            Applied::Replace(_, PartialValue::BorderColor(c)) => {
                vec![("border-color".into(), self.color_text(token, &c.0))]
            }
            Applied::Replace(_, PartialValue::Dimensions(d))
            | Applied::Replace(_, PartialValue::MinDimensions(d))
            | Applied::Replace(_, PartialValue::MaxDimensions(d)) => {
                let (wprop, hprop) =
                    if token.body.starts_with("min-w-") || token.body.starts_with("min-h-") {
                        ("min-width", "min-height")
                    } else if token.body.starts_with("max-w-") || token.body.starts_with("max-h-") {
                        ("max-width", "max-height")
                    } else {
                        ("width", "height")
                    };
                let mut out = Vec::new();
                if let Some(w) = d.width {
                    out.push((wprop.to_string(), self.sized_text(token, &w)));
                }
                if let Some(h) = d.height {
                    out.push((hprop.to_string(), self.sized_text(token, &h)));
                }
                out
            }
            Applied::Replace(_, PartialValue::Gap(g)) => {
                vec![("gap".into(), spacing_text(token, g.0, spacing))]
            }
            Applied::Replace(_, PartialValue::Layout(l)) => layout_decls(token, l),
            Applied::Replace(_, PartialValue::Opacity(o)) => {
                vec![("opacity".into(), format!("{}%", trim_f32(o.0 * 100.0)))]
            }
            Applied::Replace(_, PartialValue::Ring(r)) => ring_decls(r),
            Applied::Replace(_, PartialValue::Outline(o)) => outline_decls(o),
            Applied::Replace(_, PartialValue::Cursor(c)) => {
                vec![("cursor".into(), cursor_text(*c).to_string())]
            }
            Applied::Replace(_, PartialValue::PointerEvents(p)) => vec![(
                "pointer-events".into(),
                match p {
                    PointerEvents::None => "none",
                    PointerEvents::Auto => "auto",
                }
                .to_string(),
            )],
            Applied::Replace(_, PartialValue::Shadow(s)) => shadow_decls([s].into_iter().collect()),
            Applied::Padding {
                top,
                right,
                bottom,
                left,
            } => edge_decls(token, *top, *right, *bottom, *left, spacing),
            Applied::Margin {
                top,
                right,
                bottom,
                left,
            } => edge_decls(token, *top, *right, *bottom, *left, spacing),
            Applied::BorderWidth {
                top,
                right,
                bottom,
                left,
            } => edge_decls(token, *top, *right, *bottom, *left, spacing),
            Applied::Radius { tl, tr, br, bl } => radius_decls(token, *tl, *tr, *br, *bl),
            Applied::Text(prop, val) => text_decls(*prop, *val),
            Applied::Texts(pairs) => {
                // Utility-shaped extras: `truncate` and `line-clamp-*` carry
                // overflow/-webkit-box machinery in v4's output.
                if token.body == "truncate" {
                    return vec![
                        ("overflow".into(), "hidden".into()),
                        ("text-overflow".into(), "ellipsis".into()),
                        ("white-space".into(), "nowrap".into()),
                    ];
                }
                if token.body.starts_with("line-clamp") {
                    let n = pairs
                        .iter()
                        .find_map(|(p, v)| match (p, v) {
                            (TextProperty::LineClamp, TextValue::Count(n)) => Some(*n),
                            _ => None,
                        })
                        .unwrap_or(0);
                    return vec![
                        ("overflow".into(), "hidden".into()),
                        ("display".into(), "-webkit-box".into()),
                        ("-webkit-box-orient".into(), "vertical".into()),
                        ("-webkit-line-clamp".into(), n.to_string()),
                    ];
                }
                let mut out = Vec::new();
                for (p, v) in pairs {
                    out.extend(text_decls(*p, *v));
                }
                out
            }
            Applied::Position(p) => vec![(
                "position".into(),
                match p {
                    Position::Static => "static",
                    Position::Fixed => "fixed",
                    Position::Absolute => "absolute",
                    Position::Relative => "relative",
                    Position::Sticky => "sticky",
                }
                .to_string(),
            )],
            Applied::Inset {
                top,
                right,
                bottom,
                left,
            } => {
                let head = token.elements.as_slice()[0];
                let value = top
                    .as_ref()
                    .or(right.as_ref())
                    .or(bottom.as_ref())
                    .or(left.as_ref())
                    .map(|v| self.sized_text(token, v));
                match (head, value) {
                    ("top", Some(v)) => vec![("top".into(), v)],
                    ("right", Some(v)) => vec![("right".into(), v)],
                    ("bottom", Some(v)) => vec![("bottom".into(), v)],
                    ("left", Some(v)) => vec![("left".into(), v)],
                    ("start", Some(v)) => vec![("inset-inline-start".into(), v)],
                    ("end", Some(v)) => vec![("inset-inline-end".into(), v)],
                    ("inset", Some(v)) => vec![("inset".into(), v)],
                    _ => Vec::new(),
                }
            }
            Applied::ZIndex(z) => vec![("z-index".into(), z.to_string())],
            Applied::Overflow(o) => overflow_decls(o),
            Applied::Visibility(v) => vec![(
                "visibility".into(),
                match v {
                    Visibility::Visible => "visible",
                    Visibility::Hidden => "hidden",
                }
                .to_string(),
            )],
            Applied::AspectRatio(_) => {
                // `aspect-video` etc. resolve from `--aspect-*` raw tokens —
                // emit the token text verbatim (`16 / 9`), else the float.
                let value = token.elements.as_slice().get(1).copied().unwrap_or("");
                let text = match self.theme.raw_var(&format!("--aspect-{value}")) {
                    Some(raw) if token.arbitrary.is_none() => raw.to_string(),
                    _ => match applied {
                        Applied::AspectRatio(r) => trim_f32(*r),
                        _ => unreachable!(),
                    },
                };
                vec![("aspect-ratio".into(), text)]
            }
            Applied::Grid(g) => {
                let mut out = Vec::new();
                if let Some(c) = g.cols {
                    out.push((
                        "grid-template-columns".into(),
                        format!("repeat({c}, minmax(0, 1fr))"),
                    ));
                }
                if let Some(r) = g.rows {
                    out.push((
                        "grid-template-rows".into(),
                        format!("repeat({r}, minmax(0, 1fr))"),
                    ));
                }
                if let Some(n) = g.col_span {
                    out.push(("grid-column".into(), format!("span {n} / span {n}")));
                }
                if let Some(n) = g.row_span {
                    out.push(("grid-row".into(), format!("span {n} / span {n}")));
                }
                if let Some(n) = g.col_start {
                    out.push(("grid-column".into(), n.to_string()));
                }
                if let Some(n) = g.row_start {
                    out.push(("grid-row".into(), n.to_string()));
                }
                out
            }
            Applied::Transform {
                scale_x,
                scale_y,
                rotate_deg,
                translate_x,
                translate_y,
                skew_x,
                skew_y,
            } => self.transform_decls(
                token,
                &crate::plan::Transform {
                    scale_x: *scale_x,
                    scale_y: *scale_y,
                    rotate_deg: *rotate_deg,
                    translate_x: *translate_x,
                    translate_y: *translate_y,
                    skew_x: *skew_x,
                    skew_y: *skew_y,
                },
            ),
            Applied::Transition {
                set,
                duration_ms,
                ease,
                delay_ms,
            } => self.transition_decls(*set, *duration_ms, *ease, *delay_ms),
            Applied::Animation(a) => {
                let mut out = vec![("animation".into(), a.name.to_string())];
                if let Some(ms) = a.duration_ms {
                    out.push(("animation-duration".into(), format!("{}ms", trim_f32(ms))));
                }
                out
            }
            Applied::BorderStyle(b) => vec![(
                "border-style".into(),
                match b {
                    crate::plan::BorderStyle::Solid => "solid",
                    crate::plan::BorderStyle::Dashed => "dashed",
                    crate::plan::BorderStyle::Dotted => "dotted",
                    crate::plan::BorderStyle::Double => "double",
                    crate::plan::BorderStyle::None => "none",
                    crate::plan::BorderStyle::Hidden => "hidden",
                }
                .to_string(),
            )],
            Applied::FlexBasis(v) => vec![("flex-basis".into(), self.sized_text(token, v))],
            Applied::Order(o) => vec![("order".into(), o.to_string())],
            Applied::AlignContent(c) => {
                vec![("align-content".into(), content_align_text(*c).to_string())]
            }
            Applied::MarginAuto(e) => margin_auto_decls(e),
            Applied::GapAxes(g) => {
                let mut out = Vec::new();
                if let Some(c) = g.col {
                    out.push(("column-gap".into(), spacing_text(token, c, spacing)));
                }
                if let Some(r) = g.row {
                    out.push(("row-gap".into(), spacing_text(token, r, spacing)));
                }
                out
            }
            Applied::ShadowColor(_) => {
                // Recolors a preset in the same bucket; standalone it has no
                // box-shadow effect (matches v4's inert `--tw-shadow-color`).
                Vec::new()
            }
            Applied::ShadowPreset { first, rest } => {
                let refs: Vec<&BoxShadow> = std::iter::once(first).chain(rest.iter()).collect();
                shadow_decls(refs)
            }
            Applied::InsetShadows(shadows) => shadow_decls(shadows.iter().collect()),
            // v0.6 gradients (AEOLUS_V0.6.md §1.4): the emitter reproduces
            // v4.1.17's VERBATIM per-class declaration shapes (captured by
            // `tools/capture_gradients.mjs`) — the `--tw-gradient-*` var
            // machinery on the classes that set it. The COMPOSED
            // `background-image` lives in the plan
            // (`ResolvedStyle::background_gradient`); the corpus does not
            // pin these (cross-rule var composition is a documented model
            // boundary), so the shapes below are unit-tested, not
            // corpus-tested.
            Applied::GradientShape(GradientShapeApplied::None) => {
                vec![("background-image".into(), "none".into())]
            }
            Applied::GradientShape(GradientShapeApplied::Linear(angle_deg)) => {
                let angle = format_position_angle(*angle_deg);
                vec![
                    ("--tw-gradient-position".into(), format!("{angle} in oklab")),
                    (
                        "background-image".into(),
                        "linear-gradient(var(--tw-gradient-stops))".into(),
                    ),
                ]
            }
            Applied::GradientShape(GradientShapeApplied::Radial) => vec![
                ("--tw-gradient-position".into(), "in oklab".into()),
                (
                    "background-image".into(),
                    "radial-gradient(var(--tw-gradient-stops))".into(),
                ),
            ],
            Applied::GradientShape(GradientShapeApplied::Conic(from_deg)) => {
                let angle = format_position_angle(*from_deg);
                vec![
                    (
                        "--tw-gradient-position".into(),
                        format!("from {angle} in oklab"),
                    ),
                    (
                        "background-image".into(),
                        "conic-gradient(var(--tw-gradient-stops))".into(),
                    ),
                ]
            }
            Applied::GradientStopColor { edge, color } => {
                let edge = gradient_edge_name(*edge);
                let mut out = vec![(
                    format!("--tw-gradient-{edge}"),
                    self.color_text(token, color),
                )];
                // v4's stops chain: via routes through `--tw-gradient-via-stops`.
                if edge == "via" {
                    out.push((
                        "--tw-gradient-via-stops".into(),
                        GRADIENT_VIA_STOPS_CHAIN.into(),
                    ));
                    out.push((
                        "--tw-gradient-stops".into(),
                        "var(--tw-gradient-via-stops)".into(),
                    ));
                } else {
                    out.push(("--tw-gradient-stops".into(), GRADIENT_STOPS_CHAIN.into()));
                }
                out
            }
            Applied::GradientStopPos { edge, pct } => {
                vec![(
                    format!("--tw-gradient-{}-position", gradient_edge_name(*edge)),
                    format!("{pct}%"),
                )]
            }
        }
    }

    fn transform_decls(
        &self,
        token: &ClassToken<'_>,
        t: &crate::plan::Transform,
    ) -> Vec<(String, String)> {
        let (scale_x, scale_y, rotate_deg, translate_x, translate_y, skew_x, skew_y) = (
            t.scale_x,
            t.scale_y,
            t.rotate_deg,
            t.translate_x,
            t.translate_y,
            t.skew_x,
            t.skew_y,
        );
        let mut out = Vec::new();
        if scale_x.is_some() || scale_y.is_some() {
            // Unset axes compose to the `--tw-scale-{x,y}` initial value `1`.
            let x = scale_x
                .map(|v| format!("{}%", trim_f32(v * 100.0)))
                .unwrap_or_else(|| "1".into());
            let y = scale_y
                .map(|v| format!("{}%", trim_f32(v * 100.0)))
                .unwrap_or_else(|| "1".into());
            out.push(("scale".into(), format!("{x} {y}")));
        }
        if let Some(deg) = rotate_deg {
            out.push(("rotate".into(), format!("{}deg", trim_f32(deg))));
        }
        if translate_x.is_some() || translate_y.is_some() {
            let x = translate_x
                .as_ref()
                .map(|v| self.sized_text(token, v))
                .unwrap_or_else(|| "0".into());
            let y = translate_y
                .as_ref()
                .map(|v| self.sized_text(token, v))
                .unwrap_or_else(|| "0".into());
            out.push(("translate".into(), format!("{x} {y}")));
        }
        if skew_x.is_some() || skew_y.is_some() {
            let mut parts: Vec<String> = Vec::new();
            if let Some(s) = skew_x {
                parts.push(format!("skewX({}deg)", trim_f32(s)));
            }
            if let Some(s) = skew_y {
                parts.push(format!("skewY({}deg)", trim_f32(s)));
            }
            out.push(("transform".into(), parts.join(" ")));
        }
        out
    }

    fn transition_decls(
        &self,
        set: Option<TransitionSet>,
        duration_ms: Option<f32>,
        ease: Option<Ease>,
        delay_ms: Option<f32>,
    ) -> Vec<(String, String)> {
        let mut out = Vec::new();
        if let Some(set) = set {
            out.push((
                "transition-property".into(),
                transition_property_list(set).to_string(),
            ));
            if set != TransitionSet::None {
                // v4's `transition-*` templates carry the default duration
                // and timing function (resolved via the `--default-*` vars).
                let ms = duration_ms.unwrap_or_else(|| self.theme.default_transition_ms());
                out.push(("transition-duration".into(), format!("{}ms", trim_f32(ms))));
                let timing = match ease {
                    Some(e) => self.ease_text(e),
                    None => self
                        .theme
                        .raw_var("--default-transition-timing-function")
                        .unwrap_or("cubic-bezier(0.4, 0, 0.2, 1)")
                        .to_string(),
                };
                out.push(("transition-timing-function".into(), timing));
            }
        }
        if let Some(ms) = duration_ms {
            out.push(("transition-duration".into(), format!("{}ms", trim_f32(ms))));
        }
        if let Some(e) = ease {
            out.push(("transition-timing-function".into(), self.ease_text(e)));
        }
        if let Some(ms) = delay_ms {
            out.push(("transition-delay".into(), format!("{}ms", trim_f32(ms))));
        }
        out
    }

    fn ease_text(&self, e: Ease) -> String {
        let (name, fallback): (&str, &str) = match e {
            Ease::Linear => ("--ease-linear", "linear"),
            Ease::In => ("--ease-in", "cubic-bezier(0.4, 0, 1, 1)"),
            Ease::Out => ("--ease-out", "cubic-bezier(0, 0, 0.2, 1)"),
            Ease::InOut => ("--ease-in-out", "cubic-bezier(0.4, 0, 0.2, 1)"),
        };
        self.theme.raw_var(name).unwrap_or(fallback).to_string()
    }

    /// The color value text; opacity modifiers emit v4's modern
    /// `color-mix(in oklab, …)` form (the harness evaluates it numerically).
    fn color_text(&self, token: &ClassToken<'_>, c: &Color) -> String {
        if token.elements.as_slice().last() == Some(&"transparent") {
            return "transparent".into();
        }
        if let Some(Some(pct)) = token.opacity_pct(self.theme)
            && pct > 0.0
            && pct < 100.0
            && c.alpha > 0.0
        {
            // Un-mix the resolved color (chroma scales with alpha) so the
            // emitted color-mix re-resolves to the same value.
            let f = pct / 100.0;
            let orig = Color {
                l: c.l,
                a: c.a / f,
                b: c.b / f,
                alpha: 1.0,
            };
            return format!(
                "color-mix(in oklab, {} {}%, transparent)",
                oklch_text(&orig),
                trim_f32(pct)
            );
        }
        if c.alpha < 1.0 {
            oklch_alpha_text(c)
        } else {
            oklch_text(c)
        }
    }

    /// The text of one dimension value. Scale values were resolved to px —
    /// the token text reconstructs v4's `calc(var(--spacing) * n)` shape.
    fn sized_text(&self, token: &ClassToken<'_>, v: &SizedValue) -> String {
        match v {
            SizedValue::Px(px) => spacing_text(token, *px, self.theme.spacing()),
            SizedValue::Scale(n) => format!("calc(var(--spacing) * {})", trim_f32(*n)),
            SizedValue::Fraction(f) => format!("{}%", trim_f32(*f * 100.0)),
            SizedValue::Full => "100%".into(),
            SizedValue::FitContent => "fit-content".into(),
            SizedValue::Auto => "auto".into(),
            SizedValue::Viewport => viewport_unit(token),
        }
    }
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

fn non_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

// v0.6 gradient emitter helpers (AEOLUS_V0.6.md §1.4). The chains below are
// the v4.1.17 var machinery captured verbatim by `tools/capture_gradients.mjs`
// (the emitter's color values resolve like every other color declaration;
// v4's `var(--color-*)` references compare equal after the parity
// normalizer's var substitution — the same documented normalization the
// corpus already relies on).

/// The `--tw-gradient-stops` chain a `from-*`/`to-*` class sets.
const GRADIENT_STOPS_CHAIN: &str = "var(--tw-gradient-via-stops, var(--tw-gradient-position), \
var(--tw-gradient-from) var(--tw-gradient-from-position), \
var(--tw-gradient-to) var(--tw-gradient-to-position))";

/// The `--tw-gradient-via-stops` chain a `via-*` class sets.
const GRADIENT_VIA_STOPS_CHAIN: &str = "var(--tw-gradient-position), \
var(--tw-gradient-from) var(--tw-gradient-from-position), \
var(--tw-gradient-via) var(--tw-gradient-via-position), \
var(--tw-gradient-to) var(--tw-gradient-to-position)";

/// `from`/`via`/`to` edge name for the `--tw-gradient-*` custom properties.
fn gradient_edge_name(e: GradientEdge) -> &'static str {
    match e {
        GradientEdge::From => "from",
        GradientEdge::Via => "via",
        GradientEdge::To => "to",
    }
}

/// The CSS angle text of a gradient position (`90` → `90deg` — Rust's f32
/// Display drops the trailing `.0`).
fn format_position_angle(deg: f32) -> String {
    format!("{deg}deg")
}

/// Escape a class name the way tailwind escapes selectors: everything
/// outside `[a-zA-Z0-9_-]` gets a backslash.
pub(crate) fn escape_class(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 4);
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch);
        } else {
            out.push('\\');
            out.push(ch);
        }
    }
    out
}

fn state_suffix_css(c: VariantCondition) -> &'static str {
    match c {
        VariantCondition::Hover => ":hover",
        VariantCondition::Focus => ":focus",
        VariantCondition::FocusVisible => ":focus-visible",
        VariantCondition::FocusWithin => ":focus-within",
        VariantCondition::Active => ":active",
        VariantCondition::Disabled => ":disabled",
        VariantCondition::Visited => ":visited",
    }
}

fn state_pseudo(s: State) -> &'static str {
    match s {
        State::Hover => ":hover",
        State::Focus => ":focus",
        State::FocusVisible => ":focus-visible",
        State::FocusWithin => ":focus-within",
        State::Active => ":active",
        State::Disabled => ":disabled",
        State::Visited => ":visited",
        State::Target => ":target",
        State::Enabled => ":enabled",
        State::Checked => ":checked",
        State::Indeterminate => ":indeterminate",
        State::Required => ":required",
        State::Optional => ":optional",
        State::Valid => ":valid",
        State::Invalid => ":invalid",
        State::UserValid => ":user-valid",
        State::UserInvalid => ":user-invalid",
        State::PlaceholderShown => ":placeholder-shown",
        State::Autofill => ":autofill",
        State::ReadOnly => ":read-only",
        State::InRange => ":in-range",
        State::OutOfRange => ":out-of-range",
    }
}

fn structural_pseudo(s: Structural) -> String {
    match s {
        Structural::First => ":first-child".into(),
        Structural::Last => ":last-child".into(),
        Structural::Odd => ":nth-child(odd)".into(),
        Structural::Even => ":nth-child(even)".into(),
        Structural::Only => ":only-child".into(),
        Structural::Empty => ":empty".into(),
        Structural::Nth(n) => format!(":nth-child({n})"),
        Structural::NthLast(n) => format!(":nth-last-child({n})"),
    }
}

fn attr_test(kind: &str, name: &str, value: Option<&str>) -> String {
    match value {
        Some(v) => format!("[{kind}-{name}=\"{v}\"]"),
        // bare `data-*:` is a presence test; bare `aria-*:` means "true"
        None if kind == "aria" => format!("[{kind}-{name}=\"true\"]"),
        None => format!("[{kind}-{name}]"),
    }
}

fn not_test(inner: &Test) -> String {
    let inner_text = match inner {
        Test::State(s) => format!("*{}", state_pseudo(*s)),
        Test::Structural(s) => format!("*{}", structural_pseudo(*s)),
        Test::Data(n, v) => attr_test("data", n, v.as_deref()),
        Test::Aria(n, v) => attr_test("aria", n, v.as_deref()),
        Test::Tag(t) => t.to_string(),
    };
    format!(":not({inner_text})")
}

fn group_suffix(parent: &str, label: &str, state: &str, sibling: &str) -> String {
    if label.is_empty() {
        format!(":is(:where(.{parent}){state}{sibling} *)")
    } else {
        format!(
            ":is(:where(.{parent}\\/{}){state}{sibling} *)",
            escape_class(label)
        )
    }
}

fn media_cond(m: Media) -> String {
    match m {
        Media::Dark => "(prefers-color-scheme: dark)".into(),
        Media::Print => "(print)".into(),
        Media::MotionSafe => "(prefers-reduced-motion: no-preference)".into(),
        Media::MotionReduce => "(prefers-reduced-motion: reduce)".into(),
        Media::ContrastMore => "(prefers-contrast: more)".into(),
        Media::ContrastLess => "(prefers-contrast: less)".into(),
        Media::ForcedColors => "(forced-colors: active)".into(),
        Media::Portrait => "(orientation: portrait)".into(),
        Media::Landscape => "(orientation: landscape)".into(),
        Media::PointerFine => "(pointer: fine)".into(),
        Media::PointerCoarse => "(pointer: coarse)".into(),
        Media::PointerNone => "(pointer: none)".into(),
        Media::AnyPointerFine => "(any-pointer: fine)".into(),
        Media::AnyPointerCoarse => "(any-pointer: coarse)".into(),
        Media::AnyPointerNone => "(any-pointer: none)".into(),
        // rtl/ltr are selector-based and handled at the call site.
        Media::Rtl | Media::Ltr => String::new(),
    }
}

fn min_width(px: f32) -> String {
    format!("(width >= {}rem)", trim_f32(px / 16.0))
}

fn max_width(px: f32) -> String {
    format!("(width < {}rem)", trim_f32(px / 16.0))
}

fn oklch_text(c: &Color) -> String {
    format!(
        "oklch({} {} {})",
        trim_f32(c.l),
        trim_f32(chroma(c)),
        trim_f32(hue(c))
    )
}

fn oklch_alpha_text(c: &Color) -> String {
    format!(
        "oklch({} {} {} / {})",
        trim_f32(c.l),
        trim_f32(chroma(c)),
        trim_f32(hue(c)),
        trim_f32(c.alpha)
    )
}

fn chroma(c: &Color) -> f32 {
    (c.a * c.a + c.b * c.b).sqrt()
}

fn hue(c: &Color) -> f32 {
    let h = c.b.atan2(c.a).to_degrees();
    if h < 0.0 { h + 360.0 } else { h }
}

pub(crate) fn trim_f32(v: f32) -> String {
    if v.is_infinite() {
        return if v > 0.0 {
            "infinity".into()
        } else {
            "-infinity".into()
        };
    }
    let s = format!("{:.6}", v);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// `p-4` → `calc(var(--spacing) * 4)`; `pb-px` / arbitrary / named tokens →
/// absolute px; negatives carry the sign. `px` is the resolved f32.
fn spacing_text(token: &ClassToken<'_>, px: f32, spacing: f32) -> String {
    if token.arbitrary.is_some() || is_px_unit(token) || !is_spacing_number(token) {
        return format!("{}px", trim_f32(px));
    }
    let n = px / spacing;
    format!("calc(var(--spacing) * {})", trim_f32(n))
}

/// Does the token's utility value parse as a spacing-scale number (`4`,
/// `0.5`)? `pb-px`, `max-w-md`, `w-screen`… do not.
fn is_spacing_number(token: &ClassToken<'_>) -> bool {
    token
        .elements
        .as_slice()
        .last()
        .and_then(|v| v.parse::<f32>().ok())
        .is_some()
}

fn is_px_unit(token: &ClassToken<'_>) -> bool {
    token.elements.as_slice().last().is_some_and(|v| *v == "px")
}

fn viewport_unit(token: &ClassToken<'_>) -> String {
    // `w-screen`/`h-screen` → vw/vh; `h-dvh` → dynamic units. The plan
    // stores the abstract `Viewport`; the token text restores the unit.
    let head = token.elements.as_slice()[0];
    let vertical = matches!(head, "h" | "min-h" | "max-h");
    let value = token
        .elements
        .as_slice()
        .last()
        .copied()
        .unwrap_or("screen");
    match value {
        "dvh" => "100dvh".into(),
        "svh" => "100svh".into(),
        "lvh" => "100lvh".into(),
        "dvw" => "100dvw".into(),
        "svw" => "100svw".into(),
        "lvw" => "100lvw".into(),
        _ if vertical => "100vh".into(),
        _ => "100vw".into(),
    }
}

/// One per-edge utility head → its v4 property (`px-2` →
/// `padding-inline`, `border-t` → `border-top-width`). The payload only
/// carries values; the token names the property.
fn edge_stem(body: &str) -> Option<&'static str> {
    let (head, rest) = body.split_once('-').unwrap_or((body, ""));
    Some(match head {
        "p" => "padding",
        "px" => "padding-inline",
        "py" => "padding-block",
        "pt" => "padding-top",
        "pr" => "padding-right",
        "pb" => "padding-bottom",
        "pl" => "padding-left",
        "ps" => "padding-inline-start",
        "pe" => "padding-inline-end",
        "m" => "margin",
        "mx" => "margin-inline",
        "my" => "margin-block",
        "mt" => "margin-top",
        "mr" => "margin-right",
        "mb" => "margin-bottom",
        "ml" => "margin-left",
        "ms" => "margin-inline-start",
        "me" => "margin-inline-end",
        "border" => match rest.split('-').next().unwrap_or("") {
            "" => "border-width",
            "x" => "border-inline-width",
            "y" => "border-block-width",
            "t" => "border-top-width",
            "r" => "border-right-width",
            "b" => "border-bottom-width",
            "l" => "border-left-width",
            "s" => "border-inline-start-width",
            "e" => "border-inline-end-width",
            _ => "border-width",
        },
        _ => return None,
    })
}
fn edge_decls(
    token: &ClassToken<'_>,
    top: Option<f32>,
    right: Option<f32>,
    bottom: Option<f32>,
    left: Option<f32>,
    spacing: f32,
) -> Vec<(String, String)> {
    let Some(stem) = edge_stem(token.body) else {
        return Vec::new();
    };
    let value = top.or(right).or(bottom).or(left).unwrap_or(0.0);
    vec![(stem.to_string(), spacing_text(token, value, spacing))]
}

fn radius_decls(
    token: &ClassToken<'_>,
    tl: Option<f32>,
    tr: Option<f32>,
    br: Option<f32>,
    bl: Option<f32>,
) -> Vec<(String, String)> {
    let edge = token.elements.as_slice().get(1).copied();
    let radius_text = |v: f32| -> String {
        if v >= 9999.0 {
            // v4's `--radius-full: calc(infinity * 1px)`; the plan keeps a
            // finite 9999px sentinel — the token text restores the v4 shape.
            "calc(infinity * 1px)".into()
        } else {
            format!("{}px", trim_f32(v))
        }
    };
    let v = |x: Option<f32>| x.map(radius_text).unwrap_or_default();
    let (tl, tr, br, bl) = (v(tl), v(tr), v(br), v(bl));
    match edge {
        Some("t") => vec![
            ("border-top-left-radius".into(), tl),
            ("border-top-right-radius".into(), tr),
        ],
        Some("b") => vec![
            ("border-bottom-left-radius".into(), bl),
            ("border-bottom-right-radius".into(), br),
        ],
        Some("s") => vec![
            ("border-start-start-radius".into(), tl),
            ("border-end-start-radius".into(), bl),
        ],
        Some("e") => vec![
            ("border-start-end-radius".into(), tr),
            ("border-end-end-radius".into(), br),
        ],
        Some("ss") => vec![("border-start-start-radius".into(), tl)],
        Some("se") => vec![("border-start-end-radius".into(), tr)],
        Some("ee") => vec![("border-end-end-radius".into(), br)],
        Some("es") => vec![("border-end-start-radius".into(), bl)],
        _ => vec![("border-radius".into(), tl)],
    }
}

fn ring_decls(r: &Ring) -> Vec<(String, String)> {
    let mut layers: Vec<String> = Vec::new();
    // `ring-offset-*` alone assigns `--tw-ring-offset-width` in v4 — a var
    // no isolated rule references — so it emits no visible declaration here;
    // the ring layer's spread composes it when a ring exists.
    if r.width > 0.0 {
        let color = r
            .color
            .as_ref()
            .map(oklch_text)
            .unwrap_or_else(|| "currentcolor".into());
        let inset = if r.inset { "inset " } else { "" };
        layers.push(format!("{inset}0 0 0 {}px {}", trim_f32(r.width), color));
    }
    if layers.is_empty() {
        return Vec::new();
    }
    vec![("box-shadow".into(), layers.join(", "))]
}

fn outline_decls(o: &crate::plan::Outline) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if o.none {
        out.push(("outline-style".into(), "none".into()));
        return out;
    }
    if o.width > 0.0 {
        out.push(("outline-width".into(), format!("{}px", trim_f32(o.width))));
    }
    if let Some(c) = o.color {
        out.push(("outline-color".into(), oklch_text(&c)));
    }
    if o.offset != 0.0 {
        out.push(("outline-offset".into(), format!("{}px", trim_f32(o.offset))));
    }
    out
}

fn shadow_decls(layers: Vec<&BoxShadow>) -> Vec<(String, String)> {
    let texts: Vec<String> = layers
        .into_iter()
        .map(|l| {
            let inset = if l.inset { "inset " } else { "" };
            format!(
                "{inset}{}px {}px {}px {}px {}",
                trim_f32(l.offset_x),
                trim_f32(l.offset_y),
                trim_f32(l.blur),
                trim_f32(l.spread),
                oklch_alpha_text(&l.color)
            )
        })
        .filter(|t| !is_zero_transparent_layer(t))
        .collect();
    if texts.is_empty() {
        return Vec::new();
    }
    vec![("box-shadow".into(), texts.join(", "))]
}

fn is_zero_transparent_layer(layer: &str) -> bool {
    let Some((lens, color)) = split_layer(layer) else {
        return false;
    };
    if !color.contains("/ 0)") && color != "transparent" && color != "#0000" {
        return false;
    }
    lens.split_whitespace().count() >= 2
        && lens
            .split_whitespace()
            .all(|t| parse_len(t).is_some_and(|v| v == 0.0))
}

/// Split `0 1px 3px 0 oklch(0 0 0 / 0.1)` into lengths + color: the color
/// starts at the first color-function head (lengths never contain one).
fn split_layer(layer: &str) -> Option<(String, String)> {
    const HEADS: [&str; 6] = ["oklch(", "oklab(", "rgb(", "rgba(", "hsl(", "lab("];
    let mut best: Option<usize> = None;
    for h in HEADS {
        if let Some(i) = layer.find(h) {
            best = Some(best.map_or(i, |b: usize| b.min(i)));
        }
    }
    if layer.contains('#')
        && let Some(i) = layer.find('#')
    {
        best = Some(best.map_or(i, |b: usize| b.min(i)));
    }
    let idx = best?;
    Some((
        layer[..idx].trim().to_string(),
        layer[idx..].trim().to_string(),
    ))
}

fn parse_len(t: &str) -> Option<f32> {
    let t = t.trim().strip_suffix("px").unwrap_or(t.trim());
    t.parse().ok()
}

fn text_decls(prop: TextProperty, val: TextValue) -> Vec<(String, String)> {
    let one = |p: &str, v: String| vec![(p.to_string(), v)];
    match (prop, val) {
        (TextProperty::FontSize, TextValue::Px(px)) => {
            one("font-size", format!("{}px", trim_f32(px)))
        }
        (TextProperty::LineHeight, TextValue::Px(px)) => {
            one("line-height", format!("{}px", trim_f32(px)))
        }
        // v0.5 §1.3: the token form emits v4's unitless shape
        // (`line-height: 1.25` — element-relative at render time).
        (TextProperty::LineHeight, TextValue::LineHeightRatio(r)) => {
            one("line-height", trim_f32(r))
        }
        // v0.5 §1.3: the token form emits v4's em shape
        // (`letter-spacing: -0.025em`).
        (TextProperty::LetterSpacing, TextValue::TrackingEm(e)) => {
            one("letter-spacing", format!("{}em", trim_f32(e)))
        }
        (TextProperty::FontWeight, TextValue::Weight(w)) => one("font-weight", w.to_string()),
        (TextProperty::LetterSpacing, TextValue::Spacing(px)) => {
            one("letter-spacing", format!("{}px", trim_f32(px)))
        }
        (TextProperty::Color, TextValue::Color(c)) => one("color", oklch_text(&c)),
        (TextProperty::UnderlineOffset, TextValue::Px(px)) => {
            one("text-underline-offset", format!("{}px", trim_f32(px)))
        }
        (TextProperty::Whitespace, TextValue::Keyword(k)) => one("white-space", k.to_string()),
        (TextProperty::TextAlign, TextValue::Keyword(k)) => one("text-align", k.to_string()),
        (TextProperty::FontStyle, TextValue::Keyword(k)) => one("font-style", k.to_string()),
        (TextProperty::TextTransform, TextValue::Keyword(k)) => {
            one("text-transform", k.to_string())
        }
        (TextProperty::TextDecoration, TextValue::Keyword(k)) => {
            one("text-decoration-line", k.to_string())
        }
        (TextProperty::FontFamily, TextValue::Keyword(k)) => one("font-family", k.to_string()),
        (TextProperty::TextOverflow, TextValue::Keyword(k)) => one("text-overflow", k.to_string()),
        (TextProperty::LineClamp, TextValue::Count(n)) => one("-webkit-line-clamp", n.to_string()),
        (TextProperty::FontVariantNumeric, TextValue::Keyword(k)) => {
            one("font-variant-numeric", k.to_string())
        }
        // `select-none` etc.: v4 emits the -webkit- prefixed form alongside.
        (TextProperty::Select, TextValue::Keyword(k)) => {
            let kw = k.strip_prefix("select-").unwrap_or(k);
            vec![
                ("-webkit-user-select".into(), kw.to_string()),
                ("user-select".into(), kw.to_string()),
            ]
        }
        _ => Vec::new(),
    }
}

fn overflow_decls(o: &crate::plan::Overflow) -> Vec<(String, String)> {
    let kind = |k: &Option<OverflowKind>| {
        k.map(|k| match k {
            OverflowKind::Visible => "visible",
            OverflowKind::Hidden => "hidden",
            OverflowKind::Clip => "clip",
            OverflowKind::Scroll => "scroll",
            OverflowKind::Auto => "auto",
        })
    };
    let (x, y) = (kind(&o.x), kind(&o.y));
    match (x, y) {
        (Some(k), Some(_)) if o.x == o.y => vec![("overflow".into(), k.to_string())],
        (Some(k), _) => vec![("overflow-x".into(), k.to_string())],
        (_, Some(k)) => vec![("overflow-y".into(), k.to_string())],
        _ => Vec::new(),
    }
}

fn layout_decls(token: &ClassToken<'_>, l: &crate::plan::Layout) -> Vec<(String, String)> {
    let mut out = Vec::new();
    // `flex-col`/`flex-wrap` etc. set layout axes; only the display
    // utilities themselves emit `display` (v4's one-property-per-candidate).
    if let Some(d) = l.display {
        let is_display_utility = matches!(
            token.body,
            "flex" | "inline-flex" | "grid" | "block" | "inline" | "hidden"
        );
        if is_display_utility {
            out.push((
                "display".into(),
                match d {
                    Display::Flex => "flex",
                    Display::InlineFlex => "inline-flex",
                    Display::Grid => "grid",
                    Display::Block => "block",
                    Display::Inline => "inline",
                    Display::None => "none",
                }
                .to_string(),
            ));
        }
    }
    if let Some(d) = l.flex_direction {
        out.push((
            "flex-direction".into(),
            match d {
                FlexDirection::Row => "row",
                FlexDirection::Column => "column",
                FlexDirection::RowReverse => "row-reverse",
                FlexDirection::ColumnReverse => "column-reverse",
            }
            .to_string(),
        ));
    }
    if let Some(a) = l.align_items {
        out.push(("align-items".into(), align_text(a).to_string()));
    }
    if let Some(j) = l.justify_content {
        out.push(("justify-content".into(), justify_text(j).to_string()));
    }
    if let Some(w) = l.flex_wrap {
        out.push((
            "flex-wrap".into(),
            if w { "wrap" } else { "nowrap" }.to_string(),
        ));
    }
    if let Some(g) = l.flex_grow {
        out.push(("flex-grow".into(), trim_f32(g)));
    }
    if let Some(s) = l.flex_shrink {
        out.push(("flex-shrink".into(), trim_f32(s)));
    }
    if let Some(a) = l.align_self {
        out.push(("align-self".into(), align_text(a).to_string()));
    }
    out
}

fn align_text(a: AlignItems) -> &'static str {
    match a {
        AlignItems::Start => "flex-start",
        AlignItems::Center => "center",
        AlignItems::End => "flex-end",
        AlignItems::Stretch => "stretch",
        AlignItems::Baseline => "baseline",
    }
}

fn justify_text(j: JustifyContent) -> &'static str {
    match j {
        JustifyContent::Start => "flex-start",
        JustifyContent::Center => "center",
        JustifyContent::End => "flex-end",
        JustifyContent::SpaceBetween => "space-between",
        JustifyContent::SpaceAround => "space-around",
        JustifyContent::SpaceEvenly => "space-evenly",
    }
}

fn content_align_text(c: ContentAlign) -> &'static str {
    match c {
        ContentAlign::Start => "flex-start",
        ContentAlign::Center => "center",
        ContentAlign::End => "flex-end",
        ContentAlign::SpaceBetween => "space-between",
        ContentAlign::SpaceAround => "space-around",
        ContentAlign::SpaceEvenly => "space-evenly",
        ContentAlign::Baseline => "baseline",
        ContentAlign::Stretch => "stretch",
        ContentAlign::Normal => "normal",
    }
}

fn cursor_text(c: Cursor) -> &'static str {
    match c {
        Cursor::Auto => "auto",
        Cursor::Default => "default",
        Cursor::Pointer => "pointer",
        Cursor::Wait => "wait",
        Cursor::Text => "text",
        Cursor::Move => "move",
        Cursor::Help => "help",
        Cursor::NotAllowed => "not-allowed",
        Cursor::None => "none",
        Cursor::ContextMenu => "context-menu",
        Cursor::Progress => "progress",
        Cursor::Cell => "cell",
        Cursor::Crosshair => "crosshair",
        Cursor::VerticalText => "vertical-text",
        Cursor::Alias => "alias",
        Cursor::Copy => "copy",
        Cursor::NoDrop => "no-drop",
        Cursor::Grab => "grab",
        Cursor::Grabbing => "grabbing",
        Cursor::AllScroll => "all-scroll",
        Cursor::ColResize => "col-resize",
        Cursor::RowResize => "row-resize",
        Cursor::NResize => "n-resize",
        Cursor::EResize => "e-resize",
        Cursor::SResize => "s-resize",
        Cursor::WResize => "w-resize",
        Cursor::NeResize => "ne-resize",
        Cursor::NwResize => "nw-resize",
        Cursor::SeResize => "se-resize",
        Cursor::SwResize => "sw-resize",
        Cursor::EwResize => "ew-resize",
        Cursor::NsResize => "ns-resize",
        Cursor::NeswResize => "nesw-resize",
        Cursor::NwseResize => "nwse-resize",
        Cursor::ZoomIn => "zoom-in",
        Cursor::ZoomOut => "zoom-out",
    }
}

fn transition_property_list(set: TransitionSet) -> &'static str {
    match set {
        TransitionSet::Default => {
            "color, background-color, border-color, outline-color, text-decoration-color, fill, stroke, --tw-gradient-from, --tw-gradient-via, --tw-gradient-to, opacity, box-shadow, transform, translate, scale, rotate, filter, -webkit-backdrop-filter, backdrop-filter, display, content-visibility, overlay, pointer-events"
        }
        TransitionSet::All => "all",
        TransitionSet::Colors => {
            "color, background-color, border-color, outline-color, text-decoration-color, fill, stroke, --tw-gradient-from, --tw-gradient-via, --tw-gradient-to"
        }
        TransitionSet::Opacity => "opacity",
        TransitionSet::Shadow => "box-shadow",
        TransitionSet::Transform => "transform, translate, scale, rotate",
        TransitionSet::None => "none",
    }
}

fn margin_auto_decls(e: &AutoEdges) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if e.left && e.right {
        out.push(("margin-inline".into(), "auto".into()));
    } else {
        if e.left {
            out.push(("margin-left".into(), "auto".into()));
        }
        if e.right {
            out.push(("margin-right".into(), "auto".into()));
        }
    }
    if e.top && e.bottom {
        out.push(("margin-block".into(), "auto".into()));
    } else {
        if e.top {
            out.push(("margin-top".into(), "auto".into()));
        }
        if e.bottom {
            out.push(("margin-bottom".into(), "auto".into()));
        }
    }
    out
}

/// Split a legacy group/peer bucket key (`menufocus`, `hover`) into
/// `(label, state-pseudo)` via the evaluator's longest-suffix matcher.
fn split_group(label: &str) -> (String, String) {
    match crate::eval::split_group_label(label) {
        Some((l, s)) => (
            l.unwrap_or_default().to_string(),
            state_pseudo(s).to_string(),
        ),
        None => (label.to_string(), String::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TailwindTheme;

    fn resolver() -> Resolver<'static> {
        theme()
    }

    fn theme() -> Resolver<'static> {
        // leak a theme so tests stay one-liners
        Box::leak(Box::new(TailwindTheme::default()));
        let t: &'static TailwindTheme = Box::leak(Box::new(TailwindTheme::default()));
        Resolver::new(t)
    }

    fn one_rule(css: &str) -> CssRule {
        let out = resolver().to_css(css).unwrap();
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
        assert_eq!(out.rules.len(), 1);
        out.rules[0].clone()
    }

    #[test]
    fn escape_class_corners() {
        assert_eq!(escape_class("p-4"), r"p-4");
        assert_eq!(escape_class("hover:bg-x"), r"hover\:bg-x");
        assert_eq!(escape_class("bg-red-500/90"), r"bg-red-500\/90");
        assert_eq!(escape_class("w-1/2"), r"w-1\/2");
        assert_eq!(escape_class("p-0.5"), r"p-0\.5");
        assert_eq!(escape_class("!p-4"), r"\!p-4");
        assert_eq!(escape_class("@md:bg-x"), r"\@md\:bg-x");
        assert_eq!(escape_class("bg-[#0ea5e9]"), r"bg-\[\#0ea5e9\]");
        assert_eq!(
            escape_class("data-[state=open]:bg-x"),
            r"data-\[state\=open\]\:bg-x"
        );
    }

    #[test]
    fn base_rule_shape() {
        let r = one_rule("p-4");
        assert_eq!(r.selector, ".p-4");
        assert_eq!(
            r.declarations,
            vec![("padding".into(), "calc(var(--spacing) * 4)".into())]
        );
    }

    #[test]
    fn state_and_compound_rules() {
        let out = resolver()
            .to_css("hover:bg-red-500 md:hover:block")
            .unwrap();
        assert!(out.warnings.is_empty());
        assert_eq!(out.rules.len(), 2);
        assert_eq!(out.rules[0].selector, ".hover\\:bg-red-500:hover");
        assert_eq!(out.rules[1].media.as_deref(), Some("(width >= 48rem)"));
        assert_eq!(out.rules[1].selector, ".md\\:hover\\:block:hover");
    }

    #[test]
    fn arbitrary_variant_hover_canonicalizes() {
        let r = one_rule("[&:hover]:bg-red-500");
        assert_eq!(r.selector, r".\[\&\:hover\]\:bg-red-500:hover");
    }

    #[test]
    fn tag_test_prefixes_the_selector() {
        // `[a&]` — v4 emits the tag as a selector PREFIX (`a.\[a\&\]`).
        let r = one_rule("[a&]:block");
        assert_eq!(r.selector, r"a.\[a\&\]\:block");
        assert_eq!(r.declarations, vec![("display".into(), "block".into())]);
    }

    #[test]
    fn group_and_peer_suffixes() {
        let r = one_rule("group-hover:underline");
        assert_eq!(
            r.selector,
            ".group-hover\\:underline:is(:where(.group):hover *)"
        );
        let r = one_rule("peer-checked:bg-red-500");
        assert_eq!(
            r.selector,
            ".peer-checked\\:bg-red-500:is(:where(.peer):checked ~ *)"
        );
        let r = one_rule("group-hover/menu:bg-red-500");
        assert_eq!(
            r.selector,
            ".group-hover\\/menu\\:bg-red-500:is(:where(.group\\/menu):hover *)"
        );
    }

    #[test]
    fn important_suffix() {
        let r = one_rule("!p-4");
        assert_eq!(r.declarations[0].1, "calc(var(--spacing) * 4) !important");
        let r = one_rule("p-4!");
        assert_eq!(r.declarations[0].1, "calc(var(--spacing) * 4) !important");
    }

    #[test]
    fn viewport_units_from_token_text() {
        assert_eq!(one_rule("h-screen").declarations[0].1, "100vh");
        assert_eq!(one_rule("w-screen").declarations[0].1, "100vw");
        assert_eq!(one_rule("h-dvh").declarations[0].1, "100dvh");
    }

    #[test]
    fn warnings_never_silenced() {
        let out = resolver().to_css("bg-not-a-color p-4").unwrap();
        assert_eq!(out.rules.len(), 1); // only p-4 emits
        assert!(!out.warnings.is_empty());
        let out = resolver().to_css("[&_svg]:block").unwrap();
        assert!(matches!(out.warnings[0], Warning::Unsupported { .. }));
        assert!(out.rules.is_empty());
    }

    #[test]
    fn empty_ring_variants_emit_nothing() {
        // `ring-inset`/`ring-{color}` alone are inert in v4's isolated rules.
        assert!(one_rule("ring-inset").declarations.is_empty());
    }

    #[test]
    fn format_rules_prints_media() {
        let out = resolver().to_css("md:block").unwrap();
        let text = format_rules(&out);
        assert!(text.contains("@media (width >= 48rem)"));
        assert!(text.contains(".md\\:block"));
    }
}
