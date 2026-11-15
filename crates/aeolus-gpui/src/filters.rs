//! The GPUI filter tier's polyfill half (architecture plan §3.7's
//! filter row — "no filter/backdrop-filter slot at the pinned surface —
//! the polyfill-wrapper backlog is the remaining half of this row").
//! GPUI's style surface carries no `filter` property, but the element's
//! own colour slots ARE the paint a CSS filter recolours, and GPUI's
//! `Vec<BoxShadow>` carries the band shape the `drop-shadow()` function
//! composes — so the nine polyfillable filter longhands the registry
//! lowers into accumulate into a [`FilterState`] and render through
//! them instead of the silent no-op the un-mapped `_ => style` arm used
//! to be.
//!
//! The composition is the CSS shorthand's own (Filter Effects Level 1):
//!
//! * the seven colour-matrix functions (`brightness()`, `contrast()`,
//!   `grayscale()`, `hue-rotate()`, `invert()`, `saturate()`,
//!   `sepia()`) are the spec's fixed sRGB matrices — the state applies
//!   each recorded amount to the element's own declared colours
//!   (background, text, border, the composed shadow/ring bands, the
//!   gradient stops) in the var-canonical order v4's `cssFilterValue`
//!   emits, alpha riding untouched (the shorthand's matrices are
//!   colour-only, non-premultiplied sRGB);
//! * `drop-shadow(x y b color)` maps onto GPUI's own band — offset,
//!   blur radius, colour, no spread, not inset — appended AFTER the
//!   shadow tier's bands (v4's canonical order puts the drop-shadow
//!   var last, so the matrices never recolour the drop band's own
//!   colour);
//! * the bare `filter` root (v4's all-empty var chain) re-declares the
//!   chain — the primitives' vars persist beside it — while
//!   `filter-none` is the reset the cascade reads (a chain rule after
//!   the reset re-arms the chain, the source order the walk runs).
//!
//! The spatial and backdrop halves stay the reported boundary: `blur`
//! blurs the element's rendered output and the `backdrop-*` mirrors
//! paint BEHIND it — GPUI has no slot for either, the walk skips them
//! (the static gate refuses them typed), and the CSS/interp arms carry
//! the real output.
//!
//! The colour math is exact on [`Rgba`]'s f32 components (the bytes the
//! registry bakes divided by 255), clamped, and converted through the
//! same `rgba` construction every other accumulator runs — the pins in
//! `tests/filters.rs` quote the spec matrices' own bytes on the
//! registry palette.

use gpui::{BoxShadow, Hsla, Rgba, rgba};

use aeolus_ir::{Prop, Val};

use crate::shadow::{band_color, split_band_tokens};

/// One parsed `drop-shadow(x y b color)` function — GPUI's own band
/// shape minus the spread the function never carries.
#[derive(Clone, Copy, Debug, PartialEq)]
struct DropBand {
    dx: f32,
    dy: f32,
    blur: f32,
    color: Hsla,
}

/// The filter machinery one walk accumulates — the seven matrix
/// amounts, the drop-shadow bands and the colour slots the chain
/// recolours.
///
/// Every field is `Copy` and last-write-wins: the walk feeds the
/// accumulator in cascade order (base layers first, then the fired
/// variant layers), so a later `grayscale-25` replaces an earlier
/// `grayscale-50` exactly like every other longhand family, and the
/// state rides the interaction closures like the ring/gradient/shadow
/// states (the `hover:drop-shadow-lg` compose the base's chain the way
/// CSS's vars persist across state layers).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FilterState {
    /// The last filter-PROPERTY rule was a chain, not the `filter-none`
    /// reset — the source-order fact the composition reads.
    active: bool,
    brightness: Option<f32>,
    contrast: Option<f32>,
    grayscale: Option<f32>,
    /// Degrees (the registry's `FnDeg` form carries the raw number).
    hue_rotate: Option<f32>,
    invert: Option<f32>,
    saturate: Option<f32>,
    sepia: Option<f32>,
    drop_a: Option<DropBand>,
    drop_b: Option<DropBand>,
    /// The colour slots the chain recolours — recorded as the walk
    /// applies them, re-applied through the matrices after it.
    background: Option<Rgba>,
    text: Option<Rgba>,
    border: Option<Rgba>,
}

impl FilterState {
    /// Record one longhand the walk saw, with its frozen row text (the
    /// annotation a bare [`Val::Kw`] slot cannot recover — the bare
    /// `grayscale` lowers to `Val::Kw(0)` whose row text is
    /// `"grayscale(100%)"`, v4's own default). Filter longhands mutate
    /// the machinery; the three colour slots record beside them (the
    /// matrices recolour them after the walk). Every other prop is a
    /// no-op here (the caller applies those through the per-decl
    /// mapping as before).
    pub fn apply(&mut self, prop: Prop, val: Val, keyword: &str) {
        match prop {
            Prop::FilterBrightness => {
                self.active = true;
                self.brightness = amount_of(val, keyword);
            }
            Prop::FilterContrast => {
                self.active = true;
                self.contrast = amount_of(val, keyword);
            }
            Prop::FilterGrayscale => {
                self.active = true;
                self.grayscale = amount_of(val, keyword);
            }
            Prop::FilterHueRotate => {
                self.active = true;
                self.hue_rotate = degrees_of(val, keyword);
            }
            Prop::FilterInvert => {
                self.active = true;
                self.invert = amount_of(val, keyword);
            }
            Prop::FilterSaturate => {
                self.active = true;
                self.saturate = amount_of(val, keyword);
            }
            Prop::FilterSepia => {
                self.active = true;
                self.sepia = amount_of(val, keyword);
            }
            Prop::FilterDropShadow => {
                // Each drop-shadow rule re-declares the chain beside its
                // own var (v4's emission form): the chain re-arms.
                self.active = true;
                let (a, b) = parse_drop_bands(keyword);
                self.drop_a = a;
                self.drop_b = b;
            }
            Prop::Filter => {
                // The composed root: `filter-none` is the reset the
                // cascade reads (the chain deactivates), any chain text
                // re-arms it (the bare root's all-empty chain reads the
                // primitives' vars the other rules set).
                self.active = keyword != "none";
            }
            Prop::Background => self.background = rgba_of(val),
            Prop::Color => self.text = rgba_of(val),
            Prop::BorderColor => self.border = rgba_of(val),
            _ => {}
        }
    }

    /// The chain recolours: a live chain with at least one matrix
    /// amount recorded. A drop-shadow-only chain recolours nothing
    /// (the matrices are their own identity) — the caller skips the
    /// colour re-application and composes the bands alone.
    pub fn has_matrix(&self) -> bool {
        self.active
            && (self.brightness.is_some()
                || self.contrast.is_some()
                || self.grayscale.is_some()
                || self.hue_rotate.is_some()
                || self.invert.is_some()
                || self.saturate.is_some()
                || self.sepia.is_some())
    }

    /// The chain's recolour of one colour — the seven spec matrices
    /// applied in v4's var-canonical order (brightness, contrast,
    /// grayscale, hue-rotate, invert, saturate, sepia), each recorded
    /// amount once, on the non-premultiplied sRGB channels, alpha
    /// riding untouched. `None` = the chain recolours nothing (the
    /// caller keeps the colour as applied).
    pub fn shade(&self, color: Hsla) -> Option<Hsla> {
        if !self.has_matrix() {
            return None;
        }
        let baked = Rgba::from(color);
        let mut r = baked.r;
        let mut g = baked.g;
        let mut b = baked.b;
        if let Some(amount) = self.brightness {
            (r, g, b) = brightness(r, g, b, amount);
        }
        if let Some(amount) = self.contrast {
            (r, g, b) = contrast(r, g, b, amount);
        }
        if let Some(amount) = self.grayscale {
            (r, g, b) = grayscale(r, g, b, amount);
        }
        if let Some(degrees) = self.hue_rotate {
            (r, g, b) = hue_rotate(r, g, b, degrees);
        }
        if let Some(amount) = self.invert {
            (r, g, b) = invert(r, g, b, amount);
        }
        if let Some(amount) = self.saturate {
            (r, g, b) = saturate(r, g, b, amount);
        }
        if let Some(amount) = self.sepia {
            (r, g, b) = sepia(r, g, b, amount);
        }
        // gpui's own `u32::from(Rgba)` truncates (`as u32`), so the
        // lattice rounding happens here — the CSS byte the spec math
        // rounds to (the parsed bands' own `.round()` convention).
        let round = |x: f32| (x * 255.0).round() / 255.0;
        Some(
            rgba(u32::from(Rgba {
                r: round(r.clamp(0.0, 1.0)),
                g: round(g.clamp(0.0, 1.0)),
                b: round(b.clamp(0.0, 1.0)),
                a: baked.a,
            }))
            .into(),
        )
    }

    /// The recorded slot colours, for the caller's re-application
    /// through [`FilterState::shade`] after the walk (the walk applied
    /// them inline unfiltered; the chain's re-application is the last
    /// write, the cascade's own order).
    pub fn background(&self) -> Option<Rgba> {
        self.background
    }

    /// The text colour slot ([`FilterState::background`]'s pairing).
    pub fn text(&self) -> Option<Rgba> {
        self.text
    }

    /// The border colour slot ([`FilterState::background`]'s pairing).
    pub fn border(&self) -> Option<Rgba> {
        self.border
    }

    /// The parsed drop-shadow bands as GPUI's own box-shadow list —
    /// `None` when the chain is reset or no band parses (the caller
    /// composes them AFTER the shadow tier's bands, v4's canonical
    /// order).
    pub fn drop_shadows(&self) -> Option<Vec<BoxShadow>> {
        if !self.active {
            return None;
        }
        let bands = [self.drop_a, self.drop_b].into_iter().flatten();
        let out: Vec<BoxShadow> = bands
            .map(|band| {
                BoxShadow::new(gpui::px(band.dx), gpui::px(band.dy), band.color)
                    .blur_radius(gpui::px(band.blur))
            })
            .collect();
        if out.is_empty() { None } else { Some(out) }
    }
}

/// The recorded [`Rgba`] a colour-valued declaration carries — the
/// baked bytes the theme forms produce, through the crate's own
/// `rgba` construction. Non-colour values (the runtime token slot the
/// static walk never resolves) leave the slot untouched.
fn rgba_of(val: Val) -> Option<Rgba> {
    match val {
        Val::Rgba([r, g, b, a]) => Some(rgba(u32::from_be_bytes([r, g, b, a]))),
        _ => None,
    }
}

/// The sRGB amount a `FnPct` primitive carries — the registry's
/// bare-integer percent form (`brightness-110` → 110) through the same
/// pragmatic read the opacity mapping runs (`n > 1.0` scales, else the
/// fraction an arbitrary spelling carried), the percentage slot's own
/// scale beside it, and the bare root's composed text (`grayscale`
/// → `"grayscale(100%"`) parsed last.
fn amount_of(val: Val, keyword: &str) -> Option<f32> {
    match val {
        Val::Num(n) => Some(if n > 1.0 { n / 100.0 } else { n }),
        Val::Pct(n) => Some(n / 100.0),
        Val::Kw(_) => fn_amount(keyword),
        _ => None,
    }
}

/// The degrees a `FnDeg` primitive carries — the raw number the
/// registry's `hue-rotate-*` rows lower (`hue-rotate-90` → 90).
fn degrees_of(val: Val, keyword: &str) -> Option<f32> {
    match val {
        Val::Num(n) => Some(n),
        Val::Kw(_) => fn_amount(keyword),
        _ => None,
    }
}

/// The amount inside the bare root's composed text —
/// `"grayscale(100%)"` → 1.0 (the registry's bare arms carry the v4
/// 100% defaults). Anything else is the honest `None`.
fn fn_amount(keyword: &str) -> Option<f32> {
    let inner = keyword
        .split_once('(')
        .and_then(|(_, rest)| rest.strip_suffix(')'))?;
    let number = inner.trim().strip_suffix('%')?.trim();
    number.parse::<f32>().ok().map(|n| n / 100.0)
}

/// Parse the audited drop-shadow texts — the registry's preset table
/// (`drop-shadow(0 1px 1px rgb(0 0 0 / 0.05))`, one band) and the bare
/// root's two-band `--drop-shadow` token. Each function's inner text
/// is `dx dy blur color` (the px numbers, the colour last); an
/// unparseable text is the honest `None` (the walk's documented skip).
fn parse_drop_bands(keyword: &str) -> (Option<DropBand>, Option<DropBand>) {
    let mut bands = [None, None];
    let mut slot = 0usize;
    let mut rest = keyword;
    while let Some(start) = rest.find("drop-shadow(") {
        let inner_start = start + "drop-shadow(".len();
        let bytes = rest.as_bytes();
        let mut depth = 1usize;
        let mut end = inner_start;
        for (offset, ch) in bytes[inner_start..].iter().enumerate() {
            match ch {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = inner_start + offset;
                        break;
                    }
                }
                _ => {}
            }
        }
        if depth != 0 {
            break;
        }
        let inner = &rest[inner_start..end];
        if slot < bands.len()
            && let Some(band) = parse_drop_band(inner)
        {
            bands[slot] = Some(band);
            slot += 1;
        }
        rest = &rest[(end + 1).min(rest.len())..];
    }
    (bands[0], bands[1])
}

/// Parse one band's inner text: the px numbers first (the audited
/// preset texts spell them unitless-but-pixed — `1px`), the colour
/// token last — the shadow tier's own band grammar through its
/// paren-aware token split and colour parser.
fn parse_drop_band(inner: &str) -> Option<DropBand> {
    let tokens = split_band_tokens(inner.trim());
    let numbers: Vec<f32> = tokens
        .iter()
        .take(3)
        .map(|token| token.strip_suffix("px").unwrap_or(token))
        .filter_map(|token| token.parse::<f32>().ok())
        .collect();
    if numbers.len() != 3 {
        return None;
    }
    let color_token = *tokens.get(3)?;
    let (color, _) = band_color(color_token)?;
    Some(DropBand {
        dx: numbers[0],
        dy: numbers[1],
        blur: numbers[2],
        color,
    })
}

// ---------------------------------------------------------------------------
// The CSS Filter Effects Level 1 shorthand matrices (sRGB,
// non-premultiplied; each function per the spec's feColorMatrix /
// feComponentTransfer definitions, alpha untouched)
// ---------------------------------------------------------------------------

/// `brightness(a)`: `c' = c·a` (the linear transfer's zero intercept).
fn brightness(r: f32, g: f32, b: f32, a: f32) -> (f32, f32, f32) {
    (r * a, g * a, b * a)
}

/// `contrast(a)`: `c' = c·a + 0.5·(1 − a)` (slope `a`, the 0.5 pivot).
fn contrast(r: f32, g: f32, b: f32, a: f32) -> (f32, f32, f32) {
    let offset = 0.5 * (1.0 - a);
    (r * a + offset, g * a + offset, b * a + offset)
}

/// `grayscale(a)`: the identity interpolated toward the sRGB luminance
/// row (0.2126, 0.7152, 0.0722) by the amount.
fn grayscale(r: f32, g: f32, b: f32, a: f32) -> (f32, f32, f32) {
    let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let keep = 1.0 - a;
    (keep * r + a * lum, keep * g + a * lum, keep * b + a * lum)
}

/// `hue-rotate(θ)`: the spec's cos/sin matrix, θ in degrees.
fn hue_rotate(r: f32, g: f32, b: f32, degrees: f32) -> (f32, f32, f32) {
    let theta = degrees.to_radians();
    let (sin, cos) = theta.sin_cos();
    (
        (0.213 + cos * 0.787 - sin * 0.213) * r
            + (0.715 - cos * 0.715 - sin * 0.715) * g
            + (0.072 - cos * 0.072 + sin * 0.928) * b,
        (0.213 - cos * 0.213 + sin * 0.143) * r
            + (0.715 + cos * 0.285 + sin * 0.140) * g
            + (0.072 - cos * 0.072 - sin * 0.283) * b,
        (0.213 - cos * 0.213 - sin * 0.787) * r
            + (0.715 - cos * 0.715 + sin * 0.715) * g
            + (0.072 + cos * 0.928 + sin * 0.072) * b,
    )
}

/// `invert(a)`: `c' = c·(1 − 2a) + a` per channel.
fn invert(r: f32, g: f32, b: f32, a: f32) -> (f32, f32, f32) {
    let slope = 1.0 - 2.0 * a;
    (r * slope + a, g * slope + a, b * slope + a)
}

/// `saturate(s)`: the spec's fixed saturation matrix at scale `s`.
fn saturate(r: f32, g: f32, b: f32, s: f32) -> (f32, f32, f32) {
    (
        (0.213 + 0.787 * s) * r + (0.715 - 0.715 * s) * g + (0.072 - 0.072 * s) * b,
        (0.213 - 0.213 * s) * r + (0.715 + 0.285 * s) * g + (0.072 - 0.072 * s) * b,
        (0.213 - 0.213 * s) * r + (0.715 - 0.715 * s) * g + (0.072 + 0.928 * s) * b,
    )
}

/// `sepia(a)`: the identity interpolated toward the spec's fixed sepia
/// matrix by the amount.
fn sepia(r: f32, g: f32, b: f32, a: f32) -> (f32, f32, f32) {
    let keep = 1.0 - a;
    let mr = 0.393 * r + 0.769 * g + 0.189 * b;
    let mg = 0.349 * r + 0.686 * g + 0.168 * b;
    let mb = 0.272 * r + 0.534 * g + 0.131 * b;
    (keep * r + a * mr, keep * g + a * mg, keep * b + a * mb)
}
