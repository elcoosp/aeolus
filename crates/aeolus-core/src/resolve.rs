//! Resolve parsed class tokens into a [`StylePlan`] against a [`TailwindTheme`].
//!
//! The utility table covers the Shadcn v4 utility set (the v1 scope locked in
//! the mission brief). Utilities outside it produce typed **warnings**, never
//! silent mis-styling (transitions/animations are recognized-and-dropped;
//! unknown tokens get a "did you mean" suggestion).

use crate::ast::{Breakpoint, ClassToken, NamedVariant, ParentKind, Variant};
use crate::color::Color;
use crate::parser::parse_class_string;
use crate::plan::*;
use crate::theme::TailwindTheme;
use std::collections::BTreeMap;

/// A non-fatal resolution outcome for one class.
#[derive(Clone, Debug, PartialEq)]
pub enum Warning {
    /// Recognized but not expressible in v1 (transitions, positioning, ...).
    Unsupported {
        /// The raw class that triggered the warning.
        class: String,
        /// Why it cannot be lowered.
        reason: &'static str,
    },
    /// No utility table entry matched.
    Unknown {
        /// The raw class that triggered the warning.
        class: String,
        /// A close known utility, when one is within edit distance.
        suggestion: Option<String>,
    },
    /// Parsed but the theme token is missing (`bg-notacolor`).
    MissingThemeToken {
        /// The raw class that triggered the warning.
        class: String,
        /// The token name that was not found.
        token: String,
    },
    /// The class failed the v1 grammar (unknown variant, malformed value).
    /// The rest of the string is unaffected; the class is skipped.
    Unparseable {
        /// The raw class that failed to parse.
        class: String,
        /// The parser's error message.
        message: String,
    },
}

/// Result of resolving a class string.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Resolution {
    /// The resolved plan.
    pub plan: StylePlan,
    /// Non-fatal diagnostics (unsupported/unknown/unparseable classes).
    pub warnings: Vec<Warning>,
}

/// The resolver: one per theme snapshot.
pub struct Resolver<'t> {
    /// The theme tokens utilities resolve against.
    pub theme: &'t TailwindTheme,
}

/// A single utility's effect on a bucket.
#[derive(Clone, Debug, PartialEq)]
enum Applied {
    Replace(Slot, PartialValue),
    /// Per-edge families accumulate (`p-4` then `px-2` keeps top/bottom).
    Padding {
        top: Option<f32>,
        right: Option<f32>,
        bottom: Option<f32>,
        left: Option<f32>,
    },
    Margin {
        top: Option<f32>,
        right: Option<f32>,
        bottom: Option<f32>,
        left: Option<f32>,
    },
    BorderWidth {
        top: Option<f32>,
        right: Option<f32>,
        bottom: Option<f32>,
        left: Option<f32>,
    },
    Radius {
        tl: Option<f32>,
        tr: Option<f32>,
        br: Option<f32>,
        bl: Option<f32>,
    },
    Text(TextProperty, TextValue),
    /// font-size + paired line-height (`text-sm` sets both)
    Texts(Vec<(TextProperty, TextValue)>),
}

#[derive(Clone, Debug, PartialEq)]
enum PartialValue {
    Background(Background),
    ContentColor(ContentColor),
    BorderColor(BorderColor),
    Dimensions(Dimensions),
    MinDimensions(Dimensions),
    MaxDimensions(Dimensions),
    Gap(Gap),
    Shadow(BoxShadow),
    Layout(Layout),
    Opacity(Opacity),
    Ring(Ring),
    Outline(Outline),
    Cursor(Cursor),
    PointerEvents(PointerEvents),
}

/// Per-condition accumulator: raw applied list finalized by overlay order.
#[derive(Default)]
struct Bucket {
    applied: Vec<Applied>,
}

impl<'t> Resolver<'t> {
    /// A resolver bound to one theme snapshot.
    pub fn new(theme: &'t TailwindTheme) -> Self {
        Self { theme }
    }

    /// Parse + resolve a class string into a plan.
    pub fn resolve(&self, classes: &str) -> Result<Resolution, Vec<crate::color::ParseError>> {
        let (tokens, unparseable) = parse_class_string(classes);
        let mut resolution = self.resolve_tokens(tokens);
        // Unknown SYNTAX is a warning, not a failure: shadcn v4 strings
        // legitimately contain variants we do not model (has-*, data-*,
        // aria-*), and the rest of the string must still style.
        if !unparseable.is_empty() {
            resolution.warnings.extend(
                unparseable
                    .into_iter()
                    .map(|(class, message)| Warning::Unparseable { class, message }),
            );
        }
        Ok(resolution)
    }

    /// Resolve pre-parsed tokens (used by the cache and by tests).
    pub fn resolve_tokens<'a>(&self, tokens: Vec<ClassToken<'a>>) -> Resolution {
        let mut warnings = Vec::new();
        let mut base = Bucket::default();
        let mut layers: Vec<(VariantCondition, Bucket)> = Vec::new();
        let mut responsive: BTreeMap<Breakpoint, Bucket> = BTreeMap::new();
        let mut container: BTreeMap<Breakpoint, Bucket> = BTreeMap::new();
        let mut dark = Bucket::default();
        let mut group: Vec<(String, Bucket)> = Vec::new();
        let mut peer: Vec<(String, Bucket)> = Vec::new();

        for token in tokens {
            let class = token.raw.to_string();
            // Classify the variant stack.
            let mut state: Option<VariantCondition> = None;
            let mut target: Target;
            target = Target::Base;
            let mut compound_unsupported = false;
            let mut breakpoint = None;
            let mut group_label: Option<String> = None;
            let mut peer_label: Option<String> = None;
            let mut n_states = 0;

            for variant in &token.variants {
                match variant {
                    Variant::Named(NamedVariant::Hover)
                    | Variant::Named(NamedVariant::Focus)
                    | Variant::Named(NamedVariant::FocusVisible)
                    | Variant::Named(NamedVariant::FocusWithin)
                    | Variant::Named(NamedVariant::Active)
                    | Variant::Named(NamedVariant::Disabled)
                    | Variant::Named(NamedVariant::Visited) => {
                        n_states += 1;
                        state = Some(condition_of(variant));
                        target = Target::Layer;
                    }
                    Variant::Named(NamedVariant::Breakpoint(b)) => {
                        if target != Target::Base {
                            compound_unsupported = true;
                        }
                        breakpoint = Some(*b);
                        target = Target::Responsive;
                    }
                    Variant::Named(NamedVariant::Container(b)) => {
                        if target != Target::Base {
                            compound_unsupported = true;
                        }
                        breakpoint = Some(*b);
                        target = Target::Container;
                    }
                    Variant::Named(NamedVariant::Dark) => {
                        if target != Target::Base {
                            compound_unsupported = true;
                        }
                        target = Target::Dark;
                    }
                    Variant::Grouped {
                        parent: ParentKind::Group,
                        name,
                        label,
                    } => {
                        if target != Target::Base || n_states > 0 {
                            compound_unsupported = true;
                        }
                        group_label = Some(format!("{}{}", label.unwrap_or(""), name_of(name)));
                        target = Target::Group;
                    }
                    Variant::Grouped {
                        parent: ParentKind::Peer,
                        name,
                        label,
                    } => {
                        if target != Target::Base || n_states > 0 {
                            compound_unsupported = true;
                        }
                        peer_label = Some(format!("{}{}", label.unwrap_or(""), name_of(name)));
                        target = Target::Peer;
                    }
                    Variant::Arbitrary(_) => {
                        warnings.push(Warning::Unsupported {
                            class: class.clone(),
                            reason: "arbitrary/structural variants ([&..], data-*, aria-*) are not modeled in v1",
                        });
                        target = Target::Skip;
                        break;
                    }
                }
            }

            if compound_unsupported || n_states > 1 {
                warnings.push(Warning::Unsupported {
                    class: class.clone(),
                    reason: "compound variant chains (e.g. dark:hover:*, group-hover:focus:*) need nested conditions; not in the v1 IR",
                });
                continue;
            }

            match self.apply_utility(&token) {
                Ok(applied) => {
                    let bucket: &mut Bucket = match (target, state) {
                        (Target::Base, _) => &mut base,
                        (Target::Layer, None) => unreachable!("Layer target always sets state"),
                        (Target::Layer, Some(c)) => {
                            let idx = layers.iter().position(|(k, _)| *k == c);
                            let idx = idx.unwrap_or_else(|| {
                                layers.push((c, Bucket::default()));
                                layers.len() - 1
                            });
                            &mut layers[idx].1
                        }
                        (Target::Responsive, Some(_)) | (Target::Responsive, None) => {
                            let b = breakpoint.expect("responsive sets breakpoint");
                            responsive.entry(b).or_default()
                        }
                        (Target::Container, _) => {
                            let b = breakpoint.expect("container sets breakpoint");
                            container.entry(b).or_default()
                        }
                        (Target::Dark, _) => &mut dark,
                        (Target::Group, _) => {
                            let label = group_label.clone().unwrap_or_default();
                            let idx = group.iter().position(|(l, _)| *l == label);
                            let idx = idx.unwrap_or_else(|| {
                                group.push((label, Bucket::default()));
                                group.len() - 1
                            });
                            &mut group[idx].1
                        }
                        (Target::Peer, _) => {
                            let label = peer_label.clone().unwrap_or_default();
                            let idx = peer.iter().position(|(l, _)| *l == label);
                            let idx = idx.unwrap_or_else(|| {
                                peer.push((label, Bucket::default()));
                                peer.len() - 1
                            });
                            &mut peer[idx].1
                        }
                        (Target::Skip, _) => continue,
                    };
                    bucket.applied.push(applied);
                }
                Err(w) => warnings.push(w),
            }
        }

        let plan = StylePlan {
            base: self.finalize(&base),
            layers: layers
                .into_iter()
                .map(|(condition, b)| ConditionalLayer {
                    condition,
                    style: self.finalize(&b),
                })
                .collect(),
            responsive: responsive
                .into_iter()
                .map(|(b, v)| (b, self.finalize(&v)))
                .collect(),
            container: container
                .into_iter()
                .map(|(b, v)| (b, self.finalize(&v)))
                .collect(),
            dark: (!dark.applied.is_empty()).then(|| self.finalize(&dark)),
            group: group
                .into_iter()
                .map(|(l, v)| (l.into_boxed_str(), self.finalize(&v)))
                .collect(),
            peer: peer
                .into_iter()
                .map(|(l, v)| (l.into_boxed_str(), self.finalize(&v)))
                .collect(),
        };

        Resolution { plan, warnings }
    }

    // ------------------------------------------------------------------
    // finalize: applied list -> ResolvedStyle (later entries win per slot)
    // ------------------------------------------------------------------
    fn finalize(&self, bucket: &Bucket) -> ResolvedStyle {
        let mut style = ResolvedStyle::default();
        // Edge families accumulate first so `p-4 px-2` merges correctly.
        let mut pad = Edges::NONE;
        let mut mar = Edges::NONE;
        let mut bw = Edges::NONE;
        let mut rad = Corners::NONE;

        for a in &bucket.applied {
            match a {
                Applied::Padding {
                    top,
                    right,
                    bottom,
                    left,
                } => {
                    pad = Edges {
                        top: top.or(pad.top),
                        right: right.or(pad.right),
                        bottom: bottom.or(pad.bottom),
                        left: left.or(pad.left),
                    };
                }
                Applied::Margin {
                    top,
                    right,
                    bottom,
                    left,
                } => {
                    mar = Edges {
                        top: top.or(mar.top),
                        right: right.or(mar.right),
                        bottom: bottom.or(mar.bottom),
                        left: left.or(mar.left),
                    };
                }
                Applied::BorderWidth {
                    top,
                    right,
                    bottom,
                    left,
                } => {
                    bw = Edges {
                        top: top.or(bw.top),
                        right: right.or(bw.right),
                        bottom: bottom.or(bw.bottom),
                        left: left.or(bw.left),
                    };
                }
                Applied::Radius { tl, tr, br, bl } => {
                    rad = Corners {
                        tl: tl.or(rad.tl),
                        tr: tr.or(rad.tr),
                        br: br.or(rad.br),
                        bl: bl.or(rad.bl),
                    };
                }
                _ => {}
            }
        }
        if pad != Edges::NONE {
            style.padding = Some(Padding {
                top: pad.top.unwrap_or(0.0),
                right: pad.right.unwrap_or(0.0),
                bottom: pad.bottom.unwrap_or(0.0),
                left: pad.left.unwrap_or(0.0),
            });
        }
        if mar != Edges::NONE {
            style.margin = Some(Margin {
                top: mar.top.unwrap_or(0.0),
                right: mar.right.unwrap_or(0.0),
                bottom: mar.bottom.unwrap_or(0.0),
                left: mar.left.unwrap_or(0.0),
            });
        }
        if bw != Edges::NONE {
            style.border_width = Some(BorderWidth {
                top: bw.top.unwrap_or(0.0),
                right: bw.right.unwrap_or(0.0),
                bottom: bw.bottom.unwrap_or(0.0),
                left: bw.left.unwrap_or(0.0),
            });
        }
        if rad != Corners::NONE {
            style.corner_radius = Some(CornerRadius {
                top_left: rad.tl.unwrap_or(0.0),
                top_right: rad.tr.unwrap_or(0.0),
                bottom_right: rad.br.unwrap_or(0.0),
                bottom_left: rad.bl.unwrap_or(0.0),
            });
        }

        // Layout and Dimensions accumulate per-field (`h-9 w-full`,
        // `inline-flex items-center justify-center`).
        let mut dims = Dimensions::default();
        let mut min_dims = Dimensions::default();
        let mut max_dims = Dimensions::default();
        let mut layout = Layout::default();
        let mut dims_set = false;
        let mut min_set = false;
        let mut max_set = false;
        let mut layout_set = false;
        for a in &bucket.applied {
            match a {
                Applied::Replace(Slot::Dimensions, PartialValue::Dimensions(d)) => {
                    if d.width.is_some() {
                        dims.width = d.width;
                    }
                    if d.height.is_some() {
                        dims.height = d.height;
                    }
                    dims_set = true;
                }
                Applied::Replace(Slot::MinDimensions, PartialValue::MinDimensions(d)) => {
                    if d.width.is_some() {
                        min_dims.width = d.width;
                    }
                    if d.height.is_some() {
                        min_dims.height = d.height;
                    }
                    min_set = true;
                }
                Applied::Replace(Slot::MaxDimensions, PartialValue::MaxDimensions(d)) => {
                    if d.width.is_some() {
                        max_dims.width = d.width;
                    }
                    if d.height.is_some() {
                        max_dims.height = d.height;
                    }
                    max_set = true;
                }
                Applied::Replace(Slot::Layout, PartialValue::Layout(l)) => {
                    merge_layout(&mut layout, l);
                    layout_set = true;
                }
                _ => {}
            }
        }
        if dims_set {
            style.dimensions = Some(dims);
        }
        if min_set {
            style.min_dimensions = Some(min_dims);
        }
        if max_set {
            style.max_dimensions = Some(max_dims);
        }
        if layout_set {
            style.layout = Some(layout);
        }

        for a in &bucket.applied {
            if let Applied::Replace(slot, val) = a {
                // Layout/Dimensions/Min/Max were already accumulated above.
                if matches!(
                    slot,
                    Slot::Layout | Slot::Dimensions | Slot::MinDimensions | Slot::MaxDimensions
                ) {
                    continue;
                }
                match (slot, val) {
                    (Slot::Background, PartialValue::Background(v)) => style.background = Some(*v),
                    (Slot::ContentColor, PartialValue::ContentColor(v)) => {
                        style.content_color = Some(*v)
                    }
                    (Slot::BorderColor, PartialValue::BorderColor(v)) => {
                        style.border_color = Some(*v)
                    }
                    (Slot::Dimensions, PartialValue::Dimensions(v)) => style.dimensions = Some(*v),
                    (Slot::MinDimensions, PartialValue::MinDimensions(v)) => {
                        style.min_dimensions = Some(*v)
                    }
                    (Slot::MaxDimensions, PartialValue::MaxDimensions(v)) => {
                        style.max_dimensions = Some(*v)
                    }
                    (Slot::Gap, PartialValue::Gap(v)) => style.gap = Some(*v),
                    (Slot::Shadow, PartialValue::Shadow(v)) => style.shadow = Some(v.clone()),
                    (Slot::Layout, PartialValue::Layout(v)) => style.layout = Some(*v),
                    (Slot::Opacity, PartialValue::Opacity(v)) => style.opacity = Some(*v),
                    (Slot::Ring(_), PartialValue::Ring(v)) => merge_ring(&mut style, v),
                    (Slot::Outline, PartialValue::Outline(v)) => merge_outline(&mut style, v),
                    (Slot::Cursor, PartialValue::Cursor(v)) => style.cursor = Some(*v),
                    (Slot::PointerEvents, PartialValue::PointerEvents(v)) => {
                        style.pointer_events = Some(*v)
                    }
                    _ => {}
                }
            } else if let Applied::Text(p, v) = a {
                if let Some(slot) = style.text.iter_mut().find(|(q, _)| q == p) {
                    slot.1 = *v;
                } else {
                    style.text.push((*p, *v));
                }
            } else if let Applied::Texts(items) = a {
                for (p, v) in items {
                    if let Some(slot) = style.text.iter_mut().find(|(q, _)| q == p) {
                        slot.1 = *v;
                    } else {
                        style.text.push((*p, *v));
                    }
                }
            }
        }
        style
    }

    // ------------------------------------------------------------------
    // utility table
    // ------------------------------------------------------------------
    fn apply_utility(&self, t: &ClassToken<'_>) -> Result<Applied, Warning> {
        let class = t.raw.to_string();
        let unknown = |suggestion: Option<String>| Warning::Unknown {
            class: class.clone(),
            suggestion,
        };
        let unsupported = |reason: &'static str| Warning::Unsupported {
            class: class.clone(),
            reason,
        };
        let missing = |token: String| Warning::MissingThemeToken {
            class: class.clone(),
            token,
        };

        // Two-word utility heads (`min-w`, `ring-offset`, ...) — the elements
        // are split on '-', so the head must be re-joined for these families.
        const TWO_WORD_HEADS: &[&[&str]] = &[
            &["min", "w"],
            &["min", "h"],
            &["max", "w"],
            &["max", "h"],
            &["ring", "offset"],
            &["underline", "offset"],
            &["pointer", "events"],
        ];
        let two_word = t.elements.len() >= 2
            && TWO_WORD_HEADS
                .iter()
                .any(|p| p[0] == t.elements[0] && p[1] == t.elements[1]);
        let head_len = if two_word { 2 } else { 1 };
        let head = t.elements[..head_len].join("-");
        let value_elems = &t.elements[head_len..];
        let spacing = self.theme.spacing();

        // Numeric scale value (n -> n * 4px), honoring the negative sign.
        let scale = |n: f32| n * spacing * if t.negative { -1.0 } else { 1.0 };
        let scale_unsigned = |n: f32| n * spacing;

        // Color spec resolver: arbitrary `[...]` wins, else theme token.
        let color_of = |name: &str| -> Result<Option<Color>, Warning> {
            if let Some(arb) = t.arbitrary {
                return Color::parse(arb)
                    .map(Some)
                    .map_err(|_| Warning::Unsupported {
                        class: class.clone(),
                        reason: "arbitrary value is not a parsable color",
                    });
            }
            match self.theme.color(name) {
                Some(c) => Ok(Some(c)),
                None => Ok(None),
            }
        };
        let with_opacity = |c: Color| match t.opacity {
            Some(p) => c.mix_with_transparent(p),
            None => c,
        };
        // Length spec resolver for arbitrary values: `[42px]`, `[1.5rem]`, `[50%]`.
        let arb_length = |arb: &str| -> Option<SizedValue> {
            let v = arb.trim();
            if let Some(p) = v.strip_suffix("px") {
                p.trim().parse::<f32>().ok().map(SizedValue::Px)
            } else if let Some(r) = v.strip_suffix("rem") {
                r.trim()
                    .parse::<f32>()
                    .ok()
                    .map(|n| SizedValue::Px(n * 16.0))
            } else if let Some(p) = v.strip_suffix('%') {
                p.trim()
                    .parse::<f32>()
                    .ok()
                    .map(|n| SizedValue::Fraction(n / 100.0))
            } else {
                None
            }
        };

        match head.as_str() {
            // ---------------- colors ----------------
            "bg" => {
                if let Some(arb) = t.arbitrary {
                    if let Ok(c) = Color::parse(arb) {
                        return Ok(Applied::Replace(
                            Slot::Background,
                            PartialValue::Background(Background(with_opacity(c))),
                        ));
                    }
                    return Err(unsupported(
                        "bg-[...] value is not a color (gradients/images not in v1)",
                    ));
                }
                let name = value_elems.join("-");
                match color_of(&name)? {
                    Some(c) => Ok(Applied::Replace(
                        Slot::Background,
                        PartialValue::Background(Background(with_opacity(c))),
                    )),
                    None if name == "transparent" => Ok(Applied::Replace(
                        Slot::Background,
                        PartialValue::Background(Background(Color::TRANSPARENT)),
                    )),
                    None => Err(unknown(suggest(&name, self.theme))),
                }
            }
            "text" if t.arbitrary.is_none() && is_size_name(value_elems) => {
                let name = value_elems.join("-");
                match self.theme.font_size(&name) {
                    Some((size, lh)) => Ok(Applied::Texts(match lh {
                        Some(lh) => vec![
                            (TextProperty::FontSize, TextValue::Px(size)),
                            (TextProperty::LineHeight, TextValue::Px(lh)),
                        ],
                        None => vec![(TextProperty::FontSize, TextValue::Px(size))],
                    })),
                    None => Err(unsupported("unknown text size token")),
                }
            }
            "text" => {
                // color (or arbitrary size/color disambiguation)
                if let Some(arb) = t.arbitrary {
                    if let Ok(c) = Color::parse(arb) {
                        return Ok(Applied::Replace(
                            Slot::ContentColor,
                            PartialValue::ContentColor(ContentColor(c)),
                        ));
                    }
                    if let Some(v) = arb_length(arb)
                        && let SizedValue::Px(px) = v
                    {
                        return Ok(Applied::Text(TextProperty::FontSize, TextValue::Px(px)));
                    }
                    return Err(unsupported("text-[...] value is neither color nor length"));
                }
                let name = value_elems.join("-");
                match color_of(&name)? {
                    Some(c) => Ok(Applied::Replace(
                        Slot::ContentColor,
                        PartialValue::ContentColor(ContentColor(with_opacity(c))),
                    )),
                    None => Err(unknown(suggest(&name, self.theme))),
                }
            }
            "border" => {
                // `border` | `border-2` | `border-t` | `border-{color}` | arbitrary
                if let Some(arb) = t.arbitrary {
                    if let Ok(c) = Color::parse(arb) {
                        return Ok(Applied::Replace(
                            Slot::BorderColor,
                            PartialValue::BorderColor(BorderColor(c)),
                        ));
                    }
                    if let Ok(px) = arb.parse::<f32>() {
                        return Ok(Applied::BorderWidth {
                            top: Some(px),
                            right: Some(px),
                            bottom: Some(px),
                            left: Some(px),
                        });
                    }
                    if let Some(p) = arb.strip_suffix("px").and_then(|v| v.parse::<f32>().ok()) {
                        return Ok(Applied::BorderWidth {
                            top: Some(p),
                            right: Some(p),
                            bottom: Some(p),
                            left: Some(p),
                        });
                    }
                    return Err(unsupported("border-[...] value is neither color nor width"));
                }
                if value_elems.is_empty() {
                    return Ok(Applied::BorderWidth {
                        top: Some(1.0),
                        right: Some(1.0),
                        bottom: Some(1.0),
                        left: Some(1.0),
                    });
                }
                let name = value_elems.join("-");
                // width edges first: border-t / border-x / border-2 ...
                if let Some(applied) = border_width_family(&name, spacing) {
                    return Ok(applied);
                }
                match color_of(&name)? {
                    Some(c) => Ok(Applied::Replace(
                        Slot::BorderColor,
                        PartialValue::BorderColor(BorderColor(with_opacity(c))),
                    )),
                    None if name == "transparent" => Ok(Applied::Replace(
                        Slot::BorderColor,
                        PartialValue::BorderColor(BorderColor(Color::TRANSPARENT)),
                    )),
                    None => Err(unknown(suggest(&name, self.theme))),
                }
            }
            "ring" => {
                let mut ring = Ring::default();
                if let Some(arb) = t.arbitrary {
                    let w = arb
                        .trim()
                        .strip_suffix("px")
                        .unwrap_or(arb.trim())
                        .parse::<f32>()
                        .ok();
                    ring.width = w.ok_or_else(|| unsupported("ring-[...] expects a width"))?;
                    return Ok(Applied::Replace(
                        Slot::Ring(RingPart::Width),
                        PartialValue::Ring(ring),
                    ));
                }
                if value_elems.is_empty() {
                    ring.width = scale_unsigned(1.0); // v4: ring = 1px
                    return Ok(Applied::Replace(
                        Slot::Ring(RingPart::Width),
                        PartialValue::Ring(ring),
                    ));
                }
                let name = value_elems.join("-");
                if let Ok(px) = name.parse::<f32>() {
                    ring.width = scale_unsigned(px);
                    return Ok(Applied::Replace(
                        Slot::Ring(RingPart::Width),
                        PartialValue::Ring(ring),
                    ));
                }
                match color_of(&name)? {
                    Some(c) => {
                        ring.color = Some(with_opacity(c));
                        Ok(Applied::Replace(
                            Slot::Ring(RingPart::Color),
                            PartialValue::Ring(ring),
                        ))
                    }
                    None => Err(unknown(suggest(&name, self.theme))),
                }
            }
            "ring-offset" => {
                let mut ring = Ring::default();
                if value_elems.is_empty() {
                    ring.offset_width = scale_unsigned(2.0); // v4 default 2px
                    return Ok(Applied::Replace(
                        Slot::Ring(RingPart::OffsetWidth),
                        PartialValue::Ring(ring),
                    ));
                }
                let name = value_elems.join("-");
                if let Ok(px) = name.parse::<f32>() {
                    ring.offset_width = scale_unsigned(px);
                    return Ok(Applied::Replace(
                        Slot::Ring(RingPart::OffsetWidth),
                        PartialValue::Ring(ring),
                    ));
                }
                match color_of(&name)? {
                    Some(c) => {
                        ring.offset_color = Some(c);
                        Ok(Applied::Replace(
                            Slot::Ring(RingPart::OffsetColor),
                            PartialValue::Ring(ring),
                        ))
                    }
                    None => Err(unknown(suggest(&name, self.theme))),
                }
            }
            "outline" => {
                if value_elems.is_empty() {
                    return Ok(Applied::Replace(
                        Slot::Outline,
                        PartialValue::Outline(Outline {
                            width: 2.0,
                            color: None,
                            none: false,
                        }),
                    ));
                }
                let name = value_elems.join("-");
                match name.as_str() {
                    "none" | "hidden" => Ok(Applied::Replace(
                        Slot::Outline,
                        PartialValue::Outline(Outline {
                            width: 0.0,
                            color: None,
                            none: true,
                        }),
                    )),
                    other => {
                        if let Ok(px) = other.parse::<f32>() {
                            Ok(Applied::Replace(
                                Slot::Outline,
                                PartialValue::Outline(Outline {
                                    width: px,
                                    color: None,
                                    none: false,
                                }),
                            ))
                        } else {
                            match color_of(other)? {
                                Some(c) => Ok(Applied::Replace(
                                    Slot::Outline,
                                    PartialValue::Outline(Outline {
                                        width: 0.0,
                                        color: Some(c),
                                        none: false,
                                    }),
                                )),
                                None => Err(unknown(suggest(other, self.theme))),
                            }
                        }
                    }
                }
            }

            // ---------------- spacing ----------------
            "p" | "px" | "py" | "pt" | "pr" | "pb" | "pl" | "ps" | "pe" => {
                let n = value_token(t, value_elems, unknown)?;
                let v = scale(n);
                Ok(padding_applied(&head, v))
            }
            "m" | "mx" | "my" | "mt" | "mr" | "mb" | "ml" | "ms" | "me" => {
                let n = value_token(t, value_elems, unknown)?;
                let v = scale(n);
                Ok(margin_applied(&head, v))
            }
            "gap" => {
                if value_elems.first().is_some_and(|e| *e == "x" || *e == "y") {
                    return Err(unsupported("axis gap (gap-x/gap-y) is not in the v1 IR"));
                }
                let n = value_token(t, value_elems, unknown)?;
                Ok(Applied::Replace(
                    Slot::Gap,
                    PartialValue::Gap(Gap(scale(n))),
                ))
            }

            // ---------------- sizing ----------------
            "w" | "h" | "size" | "min-w" | "min-h" | "max-w" | "max-h" => {
                let dim = size_value(t, value_elems, arb_length, spacing, &unknown, self.theme)?;
                let mut d = Dimensions::default();
                match head.as_str() {
                    "w" | "min-w" | "max-w" => d.width = Some(dim),
                    "h" | "min-h" | "max-h" => d.height = Some(dim),
                    "size" => {
                        d.width = Some(dim);
                        d.height = Some(dim);
                    }
                    _ => unreachable!(),
                }
                let slot = match head.as_str() {
                    "min-w" | "min-h" => Slot::MinDimensions,
                    "max-w" | "max-h" => Slot::MaxDimensions,
                    _ => Slot::Dimensions,
                };
                Ok(Applied::Replace(
                    slot,
                    match slot {
                        Slot::MinDimensions => PartialValue::MinDimensions(d),
                        Slot::MaxDimensions => PartialValue::MaxDimensions(d),
                        _ => PartialValue::Dimensions(d),
                    },
                ))
            }

            // ---------------- radius ----------------
            "rounded" => {
                let full = t.arbitrary.and_then(arb_length);
                let name = value_elems.join("-");
                let (edges, token) = radius_family(&name);
                let px = match full {
                    Some(SizedValue::Px(v)) => v,
                    _ => match token {
                        Some(tok) => self
                            .theme
                            .radius(tok)
                            .unwrap_or_else(|| default_radius(tok, spacing)),
                        None => self.theme.radius("DEFAULT").unwrap_or(4.0),
                    },
                };
                if name == "full" {
                    return Ok(Applied::Radius {
                        tl: Some(9999.0),
                        tr: Some(9999.0),
                        br: Some(9999.0),
                        bl: Some(9999.0),
                    });
                }
                Ok(match edges {
                    RadiusEdges::All => Applied::Radius {
                        tl: Some(px),
                        tr: Some(px),
                        br: Some(px),
                        bl: Some(px),
                    },
                    RadiusEdges::Top => Applied::Radius {
                        tl: Some(px),
                        tr: Some(px),
                        br: None,
                        bl: None,
                    },
                    RadiusEdges::Bottom => Applied::Radius {
                        tl: None,
                        tr: None,
                        br: Some(px),
                        bl: Some(px),
                    },
                    RadiusEdges::Left => Applied::Radius {
                        tl: Some(px),
                        tr: None,
                        br: None,
                        bl: Some(px),
                    },
                    RadiusEdges::Right => Applied::Radius {
                        tl: None,
                        tr: Some(px),
                        br: Some(px),
                        bl: None,
                    },
                    RadiusEdges::Tl => Applied::Radius {
                        tl: Some(px),
                        tr: None,
                        br: None,
                        bl: None,
                    },
                    RadiusEdges::Tr => Applied::Radius {
                        tl: None,
                        tr: Some(px),
                        br: None,
                        bl: None,
                    },
                    RadiusEdges::Br => Applied::Radius {
                        tl: None,
                        tr: None,
                        br: Some(px),
                        bl: None,
                    },
                    RadiusEdges::Bl => Applied::Radius {
                        tl: None,
                        tr: None,
                        br: None,
                        bl: Some(px),
                    },
                })
            }

            // ---------------- shadows ----------------
            "shadow" => {
                if value_elems.is_empty() {
                    let spec = self
                        .theme
                        .shadow("")
                        .unwrap_or_else(|| v4_default_shadow(""));
                    return Ok(Applied::Replace(
                        Slot::Shadow,
                        PartialValue::Shadow(to_shadow(&spec)),
                    ));
                }
                let name = value_elems.join("-");
                if name == "none" {
                    return Ok(Applied::Replace(
                        Slot::Shadow,
                        PartialValue::Shadow(BoxShadow {
                            offset_x: 0.0,
                            offset_y: 0.0,
                            blur: 0.0,
                            spread: 0.0,
                            color: Color::TRANSPARENT,
                            inset: false,
                        }),
                    ));
                }
                match self.theme.shadow(&name) {
                    Some(spec) => Ok(Applied::Replace(
                        Slot::Shadow,
                        PartialValue::Shadow(to_shadow(&spec)),
                    )),
                    None => Err(missing(format!("--shadow-{name}"))),
                }
            }

            // ---------------- layout ----------------
            "flex" => {
                let mut layout = Layout {
                    display: Some(Display::Flex),
                    ..Default::default()
                };
                match value_elems {
                    [] => Ok(Applied::Replace(Slot::Layout, PartialValue::Layout(layout))),
                    ["col"] => {
                        layout.flex_direction = Some(FlexDirection::Column);
                        Ok(Applied::Replace(Slot::Layout, PartialValue::Layout(layout)))
                    }
                    ["row"] => {
                        layout.flex_direction = Some(FlexDirection::Row);
                        Ok(Applied::Replace(Slot::Layout, PartialValue::Layout(layout)))
                    }
                    ["col-reverse"] => {
                        layout.flex_direction = Some(FlexDirection::ColumnReverse);
                        Ok(Applied::Replace(Slot::Layout, PartialValue::Layout(layout)))
                    }
                    ["row-reverse"] => {
                        layout.flex_direction = Some(FlexDirection::RowReverse);
                        Ok(Applied::Replace(Slot::Layout, PartialValue::Layout(layout)))
                    }
                    ["wrap"] => {
                        layout.flex_wrap = Some(true);
                        Ok(Applied::Replace(Slot::Layout, PartialValue::Layout(layout)))
                    }
                    ["1"] => {
                        layout.flex_grow = Some(1.0);
                        layout.flex_shrink = Some(1.0);
                        Ok(Applied::Replace(Slot::Layout, PartialValue::Layout(layout)))
                    }
                    _ => Err(unknown(suggest(&head, self.theme))),
                }
            }
            "inline" if value_elems == ["flex"] => Ok(Applied::Replace(
                Slot::Layout,
                PartialValue::Layout(Layout {
                    display: Some(Display::InlineFlex),
                    ..Default::default()
                }),
            )),
            "grid" if value_elems.is_empty() => Ok(Applied::Replace(
                Slot::Layout,
                PartialValue::Layout(Layout {
                    display: Some(Display::Grid),
                    ..Default::default()
                }),
            )),
            "block" if value_elems.is_empty() => Ok(Applied::Replace(
                Slot::Layout,
                PartialValue::Layout(Layout {
                    display: Some(Display::Block),
                    ..Default::default()
                }),
            )),
            "hidden" if value_elems.is_empty() => Ok(Applied::Replace(
                Slot::Layout,
                PartialValue::Layout(Layout {
                    display: Some(Display::None),
                    ..Default::default()
                }),
            )),
            "items" => {
                let v = match value_elems {
                    ["center"] => AlignItems::Center,
                    ["start"] => AlignItems::Start,
                    ["end"] => AlignItems::End,
                    ["stretch"] => AlignItems::Stretch,
                    ["baseline"] => AlignItems::Baseline,
                    _ => return Err(unknown(suggest(&head, self.theme))),
                };
                Ok(Applied::Replace(
                    Slot::Layout,
                    PartialValue::Layout(Layout {
                        align_items: Some(v),
                        ..Default::default()
                    }),
                ))
            }
            "justify" => {
                let v = match value_elems {
                    ["center"] => JustifyContent::Center,
                    ["start"] => JustifyContent::Start,
                    ["end"] => JustifyContent::End,
                    ["between"] => JustifyContent::SpaceBetween,
                    ["around"] => JustifyContent::SpaceAround,
                    ["evenly"] => JustifyContent::SpaceEvenly,
                    _ => return Err(unknown(suggest(&head, self.theme))),
                };
                Ok(Applied::Replace(
                    Slot::Layout,
                    PartialValue::Layout(Layout {
                        justify_content: Some(v),
                        ..Default::default()
                    }),
                ))
            }
            "self" => {
                let v = match value_elems {
                    ["center"] => AlignItems::Center,
                    ["start"] => AlignItems::Start,
                    ["end"] => AlignItems::End,
                    ["stretch"] => AlignItems::Stretch,
                    _ => return Err(unknown(suggest(&head, self.theme))),
                };
                Ok(Applied::Replace(
                    Slot::Layout,
                    PartialValue::Layout(Layout {
                        align_self: Some(v),
                        ..Default::default()
                    }),
                ))
            }
            "grow" | "shrink" => {
                let n = if value_elems.is_empty() {
                    1.0
                } else {
                    value_elems
                        .join("-")
                        .parse::<f32>()
                        .map_err(|_| unknown(None))?
                };
                let mut layout = Layout::default();
                if head == "grow" {
                    layout.flex_grow = Some(n);
                } else {
                    layout.flex_shrink = Some(n);
                }
                Ok(Applied::Replace(Slot::Layout, PartialValue::Layout(layout)))
            }

            // ---------------- typography ----------------
            "font" => {
                let name = value_elems.join("-");
                let w = match name.as_str() {
                    "thin" => 100,
                    "extralight" => 200,
                    "light" => 300,
                    "normal" => 400,
                    "medium" => 500,
                    "semibold" => 600,
                    "bold" => 700,
                    "extrabold" => 800,
                    "black" => 900,
                    _ => {
                        return Err(unsupported(
                            "font-family / font-stretch classes are not in v1",
                        ));
                    }
                };
                Ok(Applied::Text(
                    TextProperty::FontWeight,
                    TextValue::Weight(w),
                ))
            }
            "leading" => {
                let name = value_elems.join("-");
                if let Ok(n) = name.parse::<f32>() {
                    return Ok(Applied::Text(
                        TextProperty::LineHeight,
                        TextValue::Px(scale_unsigned(n)),
                    ));
                }
                let v = match name.as_str() {
                    "none" => 1.0,
                    "tight" => 1.25,
                    "snug" => 1.375,
                    "normal" => 1.5,
                    "relaxed" => 1.625,
                    "loose" => 2.0,
                    _ => return Err(missing(format!("--leading-{name}"))),
                };
                Ok(Applied::Text(
                    TextProperty::LineHeight,
                    TextValue::Px(v * 16.0),
                ))
            }
            "tracking" => {
                let em = match value_elems {
                    ["tighter"] => -0.05,
                    ["tight"] => -0.025,
                    ["normal"] => 0.0,
                    ["wide"] => 0.025,
                    ["wider"] => 0.05,
                    ["widest"] => 0.1,
                    _ => return Err(missing(format!("--tracking-{}", value_elems.join("-")))),
                };
                Ok(Applied::Text(
                    TextProperty::LetterSpacing,
                    TextValue::Spacing(em * 16.0),
                ))
            }
            "underline-offset" => {
                let n = value_token(t, value_elems, unknown)?;
                Ok(Applied::Text(
                    TextProperty::UnderlineOffset,
                    TextValue::Px(n),
                ))
            }
            "whitespace" if value_elems == ["nowrap"] => Ok(Applied::Text(
                TextProperty::Whitespace,
                TextValue::Keyword("nowrap"),
            )),
            "select" if value_elems == ["none"] => Ok(Applied::Text(
                TextProperty::Select,
                TextValue::Keyword("select-none"),
            )),

            // ---------------- misc ----------------
            "opacity" => {
                let n = value_token(t, value_elems, unknown)?;
                Ok(Applied::Replace(
                    Slot::Opacity,
                    PartialValue::Opacity(Opacity(n / 100.0)),
                ))
            }
            "cursor" => {
                let name = value_elems.join("-");
                let v = match name.as_str() {
                    "pointer" => Cursor::Pointer,
                    "default" => Cursor::Default,
                    "not-allowed" => Cursor::NotAllowed,
                    "text" => Cursor::Text,
                    "grab" => Cursor::Grab,
                    _ => return Err(unknown(suggest(&name, self.theme))),
                };
                Ok(Applied::Replace(Slot::Cursor, PartialValue::Cursor(v)))
            }
            "pointer-events" => {
                let v = match value_elems {
                    ["none"] => PointerEvents::None,
                    ["auto"] => PointerEvents::Auto,
                    _ => return Err(unknown(suggest(&head, self.theme))),
                };
                Ok(Applied::Replace(
                    Slot::PointerEvents,
                    PartialValue::PointerEvents(v),
                ))
            }

            // Recognized-but-dropped families (no transition/animation engine).
            "transition" | "animate" => Err(unsupported(match head.as_str() {
                "transition" => {
                    "CSS transitions need a transition engine (not in v1) — documented no-op"
                }
                _ => "CSS animations are not in v1 — documented no-op",
            })),
            "inset" | "top" | "bottom" | "left" | "right" | "z" | "absolute" | "fixed"
            | "sticky" | "relative" | "static" | "overflow" | "order" | "basis" | "col" | "row"
            | "space" | "divide" | "aspect" | "object" | "overscroll" | "scroll" | "place"
            | "content" | "grid" | "inline" | "outline-hidden" | "isolate" | "float" | "clear"
            | "blur" | "brightness" | "contrast" | "grayscale" | "hue-rotate" | "invert"
            | "saturate" | "sepia" | "drop-shadow" | "backdrop" | "sr" | "not-sr" | "truncate"
            | "accent" | "appearance" | "resize" | "touch" | "will-change" | "fill" | "stroke"
            | "decoration" | "align" | "vertical" | "list" | "text-wrap" | "text-pretty"
            | "hyphens" | "uppercase" | "lowercase" | "capitalize" | "normal-case" | "italic"
            | "underline" | "overline" | "line-through" | "no-underline" | "antialiased"
            | "subpixel" | "ordinal" | "slashed" | "lining" | "oldstyle" | "proportional"
            | "tabular" | "diagonal" | "stacked" | "scale" | "rotate" | "translate" | "skew"
            | "origin" | "perspective" | "transform" | "snap" | "forced" | "print" | "rtl"
            | "ltr" | "field-sizing" | "mask" | "filter" | "columns" | "break" | "box"
            | "container" | "has" | "group" | "peer" | "not" | "supports" | "min" | "max"
            | "clamp" | "fit" => Err(unsupported("utility family is outside the v1 Shadcn scope")),

            _ => Err(unknown(suggest(&head, self.theme))),
        }
    }
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
enum Target {
    Base,
    Layer,
    Responsive,
    Container,
    Dark,
    Group,
    Peer,
    Skip,
}

fn condition_of(v: &Variant<'_>) -> VariantCondition {
    match v {
        Variant::Named(NamedVariant::Hover) => VariantCondition::Hover,
        Variant::Named(NamedVariant::Focus) => VariantCondition::Focus,
        Variant::Named(NamedVariant::FocusVisible) => VariantCondition::FocusVisible,
        Variant::Named(NamedVariant::FocusWithin) => VariantCondition::FocusWithin,
        Variant::Named(NamedVariant::Active) => VariantCondition::Active,
        Variant::Named(NamedVariant::Disabled) => VariantCondition::Disabled,
        Variant::Named(NamedVariant::Visited) => VariantCondition::Visited,
        _ => unreachable!("condition_of on non-state variant"),
    }
}

fn name_of(n: &NamedVariant) -> &'static str {
    match n {
        NamedVariant::Hover => "hover",
        NamedVariant::Focus => "focus",
        NamedVariant::FocusVisible => "focus-visible",
        NamedVariant::FocusWithin => "focus-within",
        NamedVariant::Active => "active",
        NamedVariant::Disabled => "disabled",
        NamedVariant::Visited => "visited",
        NamedVariant::Dark => "dark",
        NamedVariant::Breakpoint(b) | NamedVariant::Container(b) => b.name(),
    }
}

fn value_token<'a>(
    t: &ClassToken<'a>,
    elems: &[&'a str],
    unknown: impl Fn(Option<String>) -> Warning,
) -> Result<f32, Warning> {
    if let Some(arb) = t.arbitrary {
        return arb
            .trim()
            .trim_end_matches("px")
            .parse::<f32>()
            .map_err(|_| unknown(None));
    }
    let joined = elems.join("-");
    if joined.is_empty() {
        return Err(unknown(None));
    }
    joined.parse::<f32>().map_err(|_| unknown(None))
}

fn padding_applied(head: &str, v: f32) -> Applied {
    use Applied::Padding as P;
    match head {
        "p" => P {
            top: Some(v),
            right: Some(v),
            bottom: Some(v),
            left: Some(v),
        },
        "px" => P {
            top: None,
            right: Some(v),
            bottom: None,
            left: Some(v),
        },
        "py" => P {
            top: Some(v),
            right: None,
            bottom: Some(v),
            left: None,
        },
        "pt" => P {
            top: Some(v),
            right: None,
            bottom: None,
            left: None,
        },
        "pr" | "pe" => P {
            top: None,
            right: Some(v),
            bottom: None,
            left: None,
        },
        "pb" => P {
            top: None,
            right: None,
            bottom: Some(v),
            left: None,
        },
        "pl" | "ps" => P {
            top: None,
            right: None,
            bottom: None,
            left: Some(v),
        },
        _ => unreachable!(),
    }
}

fn margin_applied(head: &str, v: f32) -> Applied {
    use Applied::Margin as M;
    match head {
        "m" => M {
            top: Some(v),
            right: Some(v),
            bottom: Some(v),
            left: Some(v),
        },
        "mx" => M {
            top: None,
            right: Some(v),
            bottom: None,
            left: Some(v),
        },
        "my" => M {
            top: Some(v),
            right: None,
            bottom: Some(v),
            left: None,
        },
        "mt" => M {
            top: Some(v),
            right: None,
            bottom: None,
            left: None,
        },
        "mr" | "me" => M {
            top: None,
            right: Some(v),
            bottom: None,
            left: None,
        },
        "mb" => M {
            top: None,
            right: None,
            bottom: Some(v),
            left: None,
        },
        "ml" | "ms" => M {
            top: None,
            right: None,
            bottom: None,
            left: Some(v),
        },
        _ => unreachable!(),
    }
}

fn border_width_family(name: &str, _spacing: f32) -> Option<Applied> {
    // v4 border widths are px, not spacing-scale multiples.
    // Non-target edges are `None` (not `Some(0.0)`) so per-side classes
    // COMPOSE like the padding/margin families do (`border-t border-b`
    // must keep the top edge set by the earlier class).
    let w = |n: f32| Some(n);
    use Applied::BorderWidth as B;
    Some(match name {
        "0" => B {
            top: Some(0.0),
            right: Some(0.0),
            bottom: Some(0.0),
            left: Some(0.0),
        },
        "t" => B {
            top: w(1.0),
            right: None,
            bottom: None,
            left: None,
        },
        "r" => B {
            top: None,
            right: w(1.0),
            bottom: None,
            left: None,
        },
        "b" => B {
            top: None,
            right: None,
            bottom: w(1.0),
            left: None,
        },
        "l" => B {
            top: None,
            right: None,
            bottom: None,
            left: w(1.0),
        },
        "x" => B {
            top: None,
            right: w(1.0),
            bottom: None,
            left: w(1.0),
        },
        "y" => B {
            top: w(1.0),
            right: None,
            bottom: w(1.0),
            left: None,
        },
        other => {
            let n: f32 = other.parse().ok()?;
            B {
                top: w(n),
                right: w(n),
                bottom: w(n),
                left: w(n),
            }
        }
    })
}

#[derive(Clone, Copy, PartialEq)]
struct Edges {
    top: Option<f32>,
    right: Option<f32>,
    bottom: Option<f32>,
    left: Option<f32>,
}

impl Edges {
    const NONE: Edges = Edges {
        top: None,
        right: None,
        bottom: None,
        left: None,
    };
}

#[derive(Clone, Copy, PartialEq)]
struct Corners {
    tl: Option<f32>,
    tr: Option<f32>,
    br: Option<f32>,
    bl: Option<f32>,
}

impl Corners {
    const NONE: Corners = Corners {
        tl: None,
        tr: None,
        br: None,
        bl: None,
    };
}

fn merge_layout(acc: &mut Layout, next: &Layout) {
    if next.display.is_some() {
        acc.display = next.display;
    }
    if next.flex_direction.is_some() {
        acc.flex_direction = next.flex_direction;
    }
    if next.align_items.is_some() {
        acc.align_items = next.align_items;
    }
    if next.justify_content.is_some() {
        acc.justify_content = next.justify_content;
    }
    if next.flex_wrap.is_some() {
        acc.flex_wrap = next.flex_wrap;
    }
    if next.flex_grow.is_some() {
        acc.flex_grow = next.flex_grow;
    }
    if next.flex_shrink.is_some() {
        acc.flex_shrink = next.flex_shrink;
    }
    if next.align_self.is_some() {
        acc.align_self = next.align_self;
    }
}

fn merge_ring(style: &mut ResolvedStyle, ring: &Ring) {
    let r = style.ring.get_or_insert_with(Ring::default);
    if ring.width != 0.0 {
        r.width = ring.width;
    }
    if ring.color.is_some() {
        r.color = ring.color;
    }
    if ring.offset_width != 0.0 {
        r.offset_width = ring.offset_width;
    }
    if ring.offset_color.is_some() {
        r.offset_color = ring.offset_color;
    }
}

fn merge_outline(style: &mut ResolvedStyle, o: &Outline) {
    let cur = style.outline.get_or_insert_with(Outline::default);
    if o.none {
        cur.none = true;
        cur.width = 0.0;
    }
    if o.width != 0.0 {
        cur.width = o.width;
    }
    if o.color.is_some() {
        cur.color = o.color;
    }
}

fn to_shadow(spec: &crate::theme::BoxShadowSpec) -> BoxShadow {
    BoxShadow {
        offset_x: spec.offset_x,
        offset_y: spec.offset_y,
        blur: spec.blur,
        spread: spec.spread,
        color: spec.color,
        inset: false,
    }
}

/// v4 default shadow presets (used when the theme has no --shadow-* tokens).
fn v4_default_shadow(name: &str) -> crate::theme::BoxShadowSpec {
    use crate::color::Color;
    let black = Color::from_srgb(0.0, 0.0, 0.0, 0.05);
    match name {
        "sm" => crate::theme::BoxShadowSpec {
            offset_x: 0.0,
            offset_y: 1.0,
            blur: 3.0,
            spread: 0.0,
            color: black,
        },
        "md" => crate::theme::BoxShadowSpec {
            offset_x: 0.0,
            offset_y: 2.0,
            blur: 6.0,
            spread: -1.0,
            color: black,
        },
        "lg" => crate::theme::BoxShadowSpec {
            offset_x: 0.0,
            offset_y: 4.0,
            blur: 10.0,
            spread: -2.0,
            color: black,
        },
        "xl" => crate::theme::BoxShadowSpec {
            offset_x: 0.0,
            offset_y: 8.0,
            blur: 16.0,
            spread: -4.0,
            color: black,
        },
        // "xs" (v4 default for empty) — 0 1px 2px 0 rgb(0 0 0 / 0.05)
        _ => crate::theme::BoxShadowSpec {
            offset_x: 0.0,
            offset_y: 1.0,
            blur: 2.0,
            spread: 0.0,
            color: black,
        },
    }
}

fn is_size_name(elems: &[&str]) -> bool {
    matches!(
        elems,
        ["xs"]
            | ["sm"]
            | ["base"]
            | ["lg"]
            | ["xl"]
            | ["2xl"]
            | ["3xl"]
            | ["4xl"]
            | ["5xl"]
            | ["6xl"]
            | ["7xl"]
            | ["8xl"]
            | ["9xl"]
    )
}

#[derive(Clone, Copy, PartialEq)]
enum RadiusEdges {
    All,
    Top,
    Bottom,
    Left,
    Right,
    Tl,
    Tr,
    Br,
    Bl,
}

/// Split `rounded-t-md` into edges + token name.
fn radius_family(name: &str) -> (RadiusEdges, Option<&str>) {
    match name {
        "" => (RadiusEdges::All, Some("DEFAULT")),
        "full" | "none" => (RadiusEdges::All, None),
        other => {
            let (edge, token) = match other.split_once('-') {
                Some((e, t)) => (e, Some(t)),
                None => (other, None),
            };
            let edges = match edge {
                "t" => RadiusEdges::Top,
                "b" => RadiusEdges::Bottom,
                "l" => RadiusEdges::Left,
                "r" => RadiusEdges::Right,
                "tl" => RadiusEdges::Tl,
                "tr" => RadiusEdges::Tr,
                "br" => RadiusEdges::Br,
                "bl" => RadiusEdges::Bl,
                _ => RadiusEdges::All,
            };
            if edges == RadiusEdges::All {
                (RadiusEdges::All, Some(other))
            } else {
                (edges, token)
            }
        }
    }
}

fn default_radius(token: &str, spacing: f32) -> f32 {
    match token {
        "DEFAULT" | "sm" => spacing,
        "md" => spacing * 1.5,
        "lg" => spacing * 2.0,
        "xl" => spacing * 3.0,
        "2xl" => spacing * 4.0,
        "3xl" => spacing * 6.0,
        _ => spacing * 1.5,
    }
}

fn size_value<'a>(
    t: &ClassToken<'a>,
    elems: &[&'a str],
    arb: impl Fn(&str) -> Option<SizedValue>,
    spacing: f32,
    unknown: &impl Fn(Option<String>) -> Warning,
    theme: &TailwindTheme,
) -> Result<SizedValue, Warning> {
    if let Some(a) = t.arbitrary {
        return arb(a).ok_or_else(|| unknown(None));
    }
    let name = elems.join("-");
    if name.is_empty() {
        return Err(unknown(None));
    }
    if let Some(frac) = name.strip_prefix("1/") {
        let d: f32 = frac.parse().map_err(|_| unknown(None))?;
        return Ok(SizedValue::Fraction(1.0 / d));
    }
    Ok(match name.as_str() {
        "full" => SizedValue::Full,
        "fit" => SizedValue::FitContent,
        "auto" => SizedValue::Auto,
        "px" => SizedValue::Px(1.0),
        "screen" => {
            return Err(Warning::Unsupported {
                class: t.raw.to_string(),
                reason: "viewport units are resolved by the backend, not at resolve time",
            });
        }
        other => {
            if let Ok(n) = other.parse::<f32>() {
                SizedValue::Px(n * spacing)
            } else if let Some(min) = theme.length(&format!("max-w-{other}")) {
                SizedValue::Px(min)
            } else {
                return Err(unknown(None));
            }
        }
    })
}

/// "did you mean" suggestion via bounded Levenshtein over known utility
/// prefixes and color tokens.
fn suggest(name: &str, theme: &TailwindTheme) -> Option<String> {
    const PREFIXES: &[&str] = &[
        "bg",
        "text",
        "border",
        "rounded",
        "p",
        "px",
        "py",
        "pt",
        "pr",
        "pb",
        "pl",
        "m",
        "mx",
        "my",
        "mt",
        "mr",
        "mb",
        "ml",
        "w",
        "h",
        "size",
        "min-w",
        "min-h",
        "max-w",
        "max-h",
        "gap",
        "flex",
        "grid",
        "items",
        "justify",
        "shadow",
        "ring",
        "ring-offset",
        "outline",
        "font",
        "leading",
        "tracking",
        "opacity",
        "cursor",
        "pointer-events",
        "transition",
        "animate",
        "select",
        "whitespace",
        "underline-offset",
        "grow",
        "shrink",
    ];
    let mut best: Option<(usize, String)> = None;
    let mut consider = |candidate: String| {
        let d = levenshtein(name, &candidate);
        if d <= 3 && best.as_ref().is_none_or(|(bd, _)| d < *bd) {
            best = Some((d, candidate));
        }
    };
    for p in PREFIXES {
        consider((*p).to_string());
    }
    // Full scan: suggestions run only on the cold error path, and the v4
    // default palette alone adds ~244 color names — truncating the list
    // made suggestions depend on hash order.
    for c in theme.color_names() {
        consider(c.to_string());
    }
    best.map(|(_, c)| c)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}
