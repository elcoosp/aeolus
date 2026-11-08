//! Typed color model with a real OKLab/OKLCH pipeline.
//!
//! Tailwind v4 ships OKLCH colors and resolves opacity modifiers with
//! `color-mix(in oklab, <color> <p>%, transparent)`. Aeolus must therefore
//! parse, mix and emit colors in OKLab — not fake it with sRGB alpha.
//!
//! Conversions follow Björn Ottosson's reference implementation
//! (<https://bottosson.github.io/posts/oklab/>), which is also what
//! `culori` (used by Tailwind itself) implements.

/// A color stored in **Oklab** with straight (non-premultiplied) alpha.
///
/// Oklab is our canonical space because Tailwind v4 opacity modifiers and
/// palette interpolation are defined in it. Convert to sRGBA only at the
/// backend boundary.
#[derive(Clone, Copy, Debug)]
pub struct Color {
    /// L in [0, 1]
    pub l: f32,
    /// a axis (green–red)
    pub a: f32,
    /// b axis (blue–yellow)
    pub b: f32,
    /// alpha in [0, 1]
    pub alpha: f32,
}

impl Color {
    /// Fully transparent black.
    pub const TRANSPARENT: Color = Color {
        l: 0.0,
        a: 0.0,
        b: 0.0,
        alpha: 0.0,
    };

    /// Build directly from Oklab components.
    pub const fn oklab(l: f32, a: f32, b: f32, alpha: f32) -> Self {
        Self { l, a, b, alpha }
    }

    /// Build from OKLCH polar form (`oklch(L C H / alpha)`).
    ///
    /// The hue is clamped to [-360, 360]: hue is periodic, and at f32
    /// precision an argument of e.g. 1.9e83 loses every significant bit in
    /// `cos`'s argument reduction (producing NaN — fuzz finding). Any hue
    /// that large is undefined input; the clamp keeps the math finite.
    pub fn oklch(l: f32, c: f32, h_deg: f32, alpha: f32) -> Self {
        let h = h_deg.clamp(-360.0, 360.0).to_radians();
        Self {
            l,
            a: c * h.cos(),
            b: c * h.sin(),
            alpha,
        }
    }

    /// Parse any CSS color syntax Aeolus supports:
    /// `oklch(L C H[/a])`, `oklab(L a b[/a])`, `#rgb|#rgba|#rrggbb|#rrggbbaa`,
    /// `rgb(r g b[/a] | r, g, b, a)` with optional `%`.
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let s = s.trim();
        if let Some(rest) = s.strip_prefix('#') {
            return Self::parse_hex(rest);
        }
        let (head, tail) = split_fn(s)
            .ok_or_else(|| ParseError::new(format!("unsupported color syntax: {s:?}")))?;
        match head {
            "oklch" => Self::parse_oklch(tail),
            "oklab" => Self::parse_oklab(tail),
            "rgb" | "rgba" => Self::parse_rgb(tail),
            other => Err(ParseError::new(format!(
                "unsupported color function {other:?} (supported: oklch, oklab, rgb, #hex)"
            ))),
        }
    }

    fn parse_hex(s: &str) -> Result<Self, ParseError> {
        let bad = || ParseError::new(format!("invalid hex color #{s}"));
        // Hex digits are ASCII. The length checks below count BYTES, so a
        // multibyte string of the right byte length would panic mid-char
        // when sliced (fuzz finding: "#ab\u{1401}").
        if !s.is_ascii() {
            return Err(bad());
        }
        // Support 3/4/6/8 digit hex.
        let srgb = match s.len() {
            3 | 4 => {
                let mut v = String::with_capacity(s.len() * 2);
                for ch in s.chars() {
                    let lo = ch.to_digit(16).ok_or_else(bad)?;
                    v.push(char::from_digit(lo, 16).unwrap());
                    v.push(char::from_digit(lo, 16).unwrap());
                }
                v
            }
            6 | 8 => s.to_string(),
            _ => return Err(bad()),
        };
        let num = |r: std::ops::Range<usize>| -> Result<f32, ParseError> {
            u8::from_str_radix(&srgb[r], 16)
                .map(|v| f32::from(v) / 255.0)
                .map_err(|_| bad())
        };
        let r = num(0..2)?;
        let g = num(2..4)?;
        let b = num(4..6)?;
        let a = if srgb.len() == 8 { num(6..8)? } else { 1.0 };
        Ok(Self::from_srgb(r, g, b, a))
    }

    fn parse_oklch(s: &str) -> Result<Self, ParseError> {
        let (parts, alpha) = split_alpha(s)?;
        if parts.len() != 3 {
            return Err(ParseError::new(format!(
                "oklch() expects 3 components, got {}",
                parts.len()
            )));
        }
        let l = parse_component(parts[0], 1.0)?; // L may be % of 1
        let c = parse_component(parts[1], f32::INFINITY)?;
        let h = parse_component(parts[2], 360.0)?; // hue may be deg
        Ok(Self::oklch(l, c, h, alpha))
    }

    fn parse_oklab(s: &str) -> Result<Self, ParseError> {
        let (parts, alpha) = split_alpha(s)?;
        if parts.len() != 3 {
            return Err(ParseError::new(format!(
                "oklab() expects 3 components, got {}",
                parts.len()
            )));
        }
        let l = parse_component(parts[0], 1.0)?;
        let a = parse_component(parts[1], f32::INFINITY)?;
        let b = parse_component(parts[2], f32::INFINITY)?;
        Ok(Self { l, a, b, alpha })
    }

    fn parse_rgb(s: &str) -> Result<Self, ParseError> {
        // CSS Color 4 legacy comma syntax — `rgb(251, 44, 54)` /
        // `rgba(251, 44, 54, 0.5)` (percentages allowed, alpha is an optional
        // 4th component). Documented in [`Color::parse`]; common in arbitrary
        // values (`bg-[rgba(0,0,0,0.5)]`, `shadow-[0_0_0_1px_rgb(0,0,0,0.05)]`).
        if s.contains(',') {
            let parts: Vec<&str> = s.split(',').map(str::trim).collect();
            if parts.len() != 3 && parts.len() != 4 {
                return Err(ParseError::new(format!(
                    "rgb() expects 3 comma-separated components (+ optional alpha), got {}",
                    parts.len()
                )));
            }
            let ch = |p: &str| parse_component(p, 255.0);
            let (r, g, b) = (ch(parts[0])?, ch(parts[1])?, ch(parts[2])?);
            let alpha = match parts.get(3) {
                Some(a) => parse_component(a, 1.0)?.clamp(0.0, 1.0),
                None => 1.0,
            };
            return Ok(Self::from_srgb(r / 255.0, g / 255.0, b / 255.0, alpha));
        }
        let (parts, alpha) = split_alpha(s)?;
        if parts.len() != 3 {
            return Err(ParseError::new(format!(
                "rgb() expects 3 components, got {}",
                parts.len()
            )));
        }
        // rgb() components are 0-255 numbers or percentages of 255.
        let ch = |p: &str| -> Result<f32, ParseError> { parse_component(p, 255.0) };
        let (r, g, b) = (ch(parts[0])?, ch(parts[1])?, ch(parts[2])?);
        Ok(Self::from_srgb(r / 255.0, g / 255.0, b / 255.0, alpha))
    }

    /// Build from linear-light-independent sRGB (0..1) + alpha by converting
    /// sRGB -> linear sRGB -> Oklab.
    ///
    /// Inputs are clamped to 0..=1: the transfer function's `powf(2.4)` is
    /// NaN for negative bases (defense in depth; `to_srgba` never emits
    /// out-of-range values).
    pub fn from_srgb(r: f32, g: f32, b: f32, alpha: f32) -> Self {
        let clamp01 = |v: f32| v.clamp(0.0, 1.0);
        let (r, g, b) = (clamp01(r), clamp01(g), clamp01(b));
        let (l, a, b) =
            oklab_from_linear_srgb(srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b));
        Self { l, a, b, alpha }
    }

    /// Convert to sRGB (0..1) + alpha, clamping out-of-gamut values.
    ///
    /// No gamut mapping beyond clamping in v1 (documented limitation;
    /// Tailwind's own palette is gamut-safe). The clamp is NOT cosmetic:
    /// out-of-gamut Oklab can produce *negative* linear components, and
    /// `powf(1/2.4)` on a negative base is NaN — found by the Phase-4
    /// property test (`oklab_srgb_roundtrip`, minimal case oklab(0, 0,
    /// 0.2738)).
    pub fn to_srgba(self) -> [f32; 4] {
        // Clamp Oklab components to the physically meaningful range BEFORE
        // the matrix: no real color has L outside [0,1] or chroma axes
        // beyond ±0.5 (Oklab's sRGB gamut lives well inside that), and
        // unbounded inputs overflow the f32 cubes to inf (fuzz finding:
        // oklch(3 3.3e40 1) -> inf -> NaN). This keeps the arithmetic
        // finite for ANY parsed value, then the linear clamp handles the
        // (much smaller) out-of-gamut remainder.
        const CHROMA_MAX: f32 = 0.5;
        let c = Color {
            l: self.l.clamp(0.0, 1.0),
            a: self.a.clamp(-CHROMA_MAX, CHROMA_MAX),
            b: self.b.clamp(-CHROMA_MAX, CHROMA_MAX),
            alpha: self.alpha,
        };
        let (lr, lg, lb) = linear_srgb_from_oklab(c.l, c.a, c.b);
        let clamp01 = |v: f32| v.clamp(0.0, 1.0);
        [
            linear_to_srgb(clamp01(lr)),
            linear_to_srgb(clamp01(lg)),
            linear_to_srgb(clamp01(lb)),
            clamp01(self.alpha), // defense: parse paths clamp too
        ]
    }

    /// Pack into 0xRRGGBBAA (alpha is rounded, not premultiplied).
    pub fn to_rgba8(self) -> [u8; 4] {
        let [r, g, b, a] = self.to_srgba();
        let f = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        [f(r), f(g), f(b), f(a)]
    }

    /// CSS `color-mix(in oklab, self p%, transparent)`.
    ///
    /// Mixing with transparent in OKLab premultiplies: both chroma and alpha
    /// scale by `p/100`, hue/lightness are preserved. This is exactly the
    /// Tailwind v4 `/opacity` modifier semantics.
    pub fn mix_with_transparent(self, pct: f32) -> Self {
        let f = (pct / 100.0).clamp(0.0, 1.0);
        Self {
            l: self.l,
            a: self.a * f,
            b: self.b * f,
            alpha: self.alpha * f,
        }
    }

    /// Return a copy with a new alpha (clamped to 0..=1).
    pub fn with_alpha(self, alpha: f32) -> Self {
        Self {
            alpha: alpha.clamp(0.0, 1.0),
            ..self
        }
    }

    /// Human-readable debug value, e.g. `oklch(0.63 0.26 29.2 / 0.9)`.
    pub fn to_oklch_string(self) -> String {
        let c = (self.a * self.a + self.b * self.b).sqrt();
        let h = self.b.atan2(self.a).to_degrees();
        let h = if h < 0.0 { h + 360.0 } else { h };
        if self.alpha >= 1.0 {
            format!("oklch({:.3} {:.3} {:.1})", self.l, c, h)
        } else {
            format!("oklch({:.3} {:.3} {:.1} / {:.2})", self.l, c, h, self.alpha)
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

/// Approximate equality, quantized to ~1e-4 per Oklab component.
///
/// WHY: exact f32 equality split colors that differ only by float noise
/// (an OKLab→sRGB→OKLab round-trip shifts the a-axis by ~3e-7), which
/// surfaced as spurious `diff()` churn and false cache misses. 1e-4 is far
/// below one sRGB 8-bit step (~4e-3 in Oklab) so no visual difference is
/// ever masked. Known tradeoff: tolerance equality is not transitive
/// (`a≈b`, `b≈c` does not imply `a≈c`) — acceptable for diffing/caching,
/// which is what colors are compared for.
impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        const EPS: f32 = 1e-4;
        (self.l - other.l).abs() <= EPS
            && (self.a - other.a).abs() <= EPS
            && (self.b - other.b).abs() <= EPS
            && (self.alpha - other.alpha).abs() <= EPS
    }
}

// ---------------------------------------------------------------------------
// parsing helpers
// ---------------------------------------------------------------------------

/// An error produced while parsing a class string or a color.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    /// Human-readable description of what failed.
    pub message: String,
}

impl ParseError {
    /// A parse error carrying a human-readable message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ParseError {}

fn split_fn(s: &str) -> Option<(&str, &str)> {
    let open = s.find('(')?;
    let close = s.rfind(')')?;
    if close < open {
        return None;
    }
    Some((&s[..open], &s[open + 1..close]))
}

/// Split `a b c / 50%` into components and a parsed alpha.
///
/// CSS clamps alpha to [0, 1]: an absurd component (1e95 -> f32 inf) must
/// come out as 1.0, never inf (fuzz finding).
fn split_alpha(s: &str) -> Result<(Vec<&str>, f32), ParseError> {
    let (body, alpha) = match s.split_once('/') {
        Some((b, a)) => (b, parse_component(a.trim(), 1.0)?.clamp(0.0, 1.0)),
        None => (s, 1.0),
    };
    let parts: Vec<&str> = body.split_whitespace().collect();
    Ok((parts, alpha))
}

/// Parse a CSS component: `0.5`, `50%` (relative to `scale`), `none` -> 0.
fn parse_component(s: &str, scale: f32) -> Result<f32, ParseError> {
    let s = s.trim();
    if s == "none" {
        return Ok(0.0);
    }
    if let Some(pct) = s.strip_suffix('%') {
        let v: f32 = pct
            .trim()
            .parse()
            .map_err(|_| ParseError::new(format!("bad percentage {s:?}")))?;
        // Percentages need a FINITE reference range. For unbounded
        // components (Oklab a/b, Oklch chroma — `scale = INFINITY`) the CSS
        // spec's percentage reference is ±0.4; `v% of INFINITY` would
        // produce NaN/inf and silently poison the whole conversion
        // (fuzz-found: `oklab(0% 0% 3)` used to yield NaN chroma).
        let scale = if scale.is_infinite() { 0.4 } else { scale };
        return Ok(v / 100.0 * scale);
    }
    s.parse::<f32>()
        .map_err(|_| ParseError::new(format!("bad number {s:?}")))
}

// ---------------------------------------------------------------------------
// color space math (Ottosson reference matrices, f32)
// ---------------------------------------------------------------------------

fn srgb_to_linear(c: f32) -> f32 {
    if c.abs() <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f32) -> f32 {
    if c.abs() <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn oklab_from_linear_srgb(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let l = 0.412_221_4 * r + 0.536_332_5 * g + 0.051_445_99 * b;
    let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
    let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    (
        0.210_454_26 * l_ + 0.793_617_8 * m_ - 0.004_072_046 * s_,
        1.977_998_5 * l_ - 2.428_592_2 * m_ + 0.450_593_7 * s_,
        0.025_904_04 * l_ + 0.782_771_77 * m_ - 0.808_675_77 * s_,
    )
}

fn linear_srgb_from_oklab(l: f32, a: f32, b: f32) -> (f32, f32, f32) {
    let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
    let m_ = l - 0.105_561_35 * a - 0.063_854_17 * b;
    let s_ = l - 0.089_484_18 * a - 1.291_485_6 * b;

    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;

    (
        4.076_741_7 * l3 - 3.307_711_6 * m3 + 0.230_969_94 * s3,
        -1.268_438 * l3 + 2.609_757_4 * m3 - 0.341_319_4 * s3,
        -0.004_196_08 * l3 - 0.703_418_6 * m3 + 1.707_614_7 * s3,
    )
}

#[cfg(test)]
mod tests {
    /// Fuzz regression (v0.2): percentages on UNBOUNDED components
    /// (Oklab a/b, Oklch chroma) used to produce NaN via `pct% of INFINITY`.
    /// The CSS percentage reference for those components is ±0.4.
    #[test]
    fn percentage_on_unbounded_components_is_finite() {
        for s in [
            "oklab(0% 0% 3)",
            "   oklab(   0%  \n 0%  \n\n\n\n3  )  ",
            "oklch(50% 50% 90)",
        ] {
            let c = Color::parse(s).unwrap();
            let [r, g, b, a] = c.to_srgba();
            assert!(
                r.is_finite() && g.is_finite() && b.is_finite() && a.is_finite(),
                "{s} must convert to finite sRGB"
            );
        }
    }

    use super::*;

    /// Tailwind v4's own palette is the oracle: `oklch(0.637 0.237 25.331)` is
    /// `--color-red-500`, which is `#fb2c36` in the v4 default theme.
    #[test]
    fn red500_roundtrip() {
        let c = Color::parse("oklch(0.637 0.237 25.331)").unwrap();
        let [r, g, b, _] = c.to_srgba();
        // Tailwind publishes #fb2c36 (their values are gamut-mapped lightly).
        assert!((r - 0.984).abs() < 0.02, "r={r}");
        assert!((g - 0.173).abs() < 0.05, "g={g}");
        assert!((b - 0.212).abs() < 0.05, "b={b}");
    }

    #[test]
    fn white_black() {
        let w = Color::parse("oklch(1 0 0)").unwrap().to_rgba8();
        assert_eq!(w, [255, 255, 255, 255]);
        let k = Color::parse("oklch(0 0 0)").unwrap().to_rgba8();
        assert_eq!(k, [0, 0, 0, 255]);
    }

    #[test]
    fn hex_and_rgb() {
        assert_eq!(
            Color::parse("#fb2c36").unwrap().to_rgba8(),
            [251, 44, 54, 255]
        );
        assert_eq!(
            Color::parse("#09f8").unwrap().to_rgba8(),
            [0, 153, 255, 136]
        );
        assert_eq!(
            Color::parse("rgb(251 44 54)").unwrap().to_rgba8(),
            [251, 44, 54, 255]
        );
        assert_eq!(
            Color::parse("rgb(98.4% 17.3% 21.2% / 50%)")
                .unwrap()
                .to_rgba8()[3],
            128
        );
    }

    #[test]
    fn opacity_modifier_is_premultiplied_chroma() {
        let c = Color::parse("oklch(0.637 0.237 25.331)").unwrap();
        let mixed = c.mix_with_transparent(50.0);
        let hue = |c: Color| c.b.atan2(c.a);
        assert_eq!(mixed.alpha, 0.5);
        assert_eq!(mixed.l, c.l, "lightness preserved");
        assert!((mixed.a - c.a * 0.5).abs() < 1e-6, "chroma scaled");
        assert!((hue(mixed) - hue(c)).abs() < 1e-6, "hue preserved");
    }

    /// Exact f32 equality splits colors that differ only by float noise
    /// (e.g. an sRGB round-trip), causing spurious plan diffs and false
    /// cache misses. Equality must be approximate (~1e-4).
    #[test]
    fn approximate_equality_ignores_float_noise() {
        use crate::plan::{Background, ResolvedStyle};

        let a = Color::parse("oklch(0.637 0.237 25.331)").unwrap();
        let [r, g, b, al] = a.to_srgba();
        let roundtrip = Color::from_srgb(r, g, b, al);
        assert_eq!(a, roundtrip, "sRGB round-trip noise must not split colors");
        // The plan-level diff must agree (a ≈ b → no changed slots).
        let mut prev = ResolvedStyle::default();
        let mut next = ResolvedStyle::default();
        prev.background = Some(Background(a));
        next.background = Some(Background(roundtrip));
        assert!(
            crate::diff(&prev, &next).is_empty(),
            "a ≈ b → diff(a,b).is_empty()"
        );
    }

    /// Fuzz finding: non-ASCII hex must be rejected, not panic on a
    /// non-char-boundary slice (`#όόό` is 6 BYTES, passing the old
    /// byte-length check).
    #[test]
    fn non_ascii_hex_is_rejected_not_panic() {
        // 'ᐁ' is 3 bytes: "#abᐁ" is 6 BYTES (passes the old byte-length
        // check) and slicing its 6-byte form at [2..4] panics mid-char.
        assert!(Color::parse("#ab\u{1401}").is_err());
        assert!(Color::parse("#όόό").is_err());
        assert!(Color::parse("#\u{0}").is_err());
        // Real hex still parses.
        assert!(Color::parse("#fb2c36").is_ok());
        assert!(Color::parse("#09f8").is_ok());
    }

    /// Fuzz finding: an absurd alpha component (1e95) parsed to f32 inf —
    /// alpha is clamped to [0,1] by CSS, so parse must clamp (and sRGB
    /// conversion must stay finite for ANY parsed value).
    #[test]
    fn absurd_alpha_is_clamped_to_unit_range() {
        let c = Color::parse("rgb(1 2 2/66666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666660)").unwrap();
        assert_eq!(c.alpha, 1.0, "alpha > 1 clamps to opaque");
        let [_, _, _, a] = c.to_srgba();
        assert!(a.is_finite() && a <= 1.0);
        let c = Color::parse("oklch(0.5 0.1 180 / -42)").unwrap();
        assert_eq!(c.alpha, 0.0, "negative alpha clamps to transparent");
    }

    /// Over-tolerant equality would merge genuinely different colors.
    #[test]
    fn approximate_equality_still_rejects_real_differences() {
        let a = Color::parse("oklch(0.637 0.237 25.331)").unwrap();
        let b = Color::parse("oklch(0.637 0.237 26)").unwrap();
        assert_ne!(a, b, "hue delta 0.669deg ≈ 0.003 on a/b axes > tolerance");
        let c = Color::parse("oklch(0.637 0.237 25.331 / 0.9)").unwrap();
        assert_ne!(a, c, "alpha is part of equality");
    }

    /// Legacy comma syntax is documented in `Color::parse` and common in
    /// arbitrary values — it must parse exactly like the space form
    /// (regression: `rgb(251, 44, 54)` used to fail with `bad number "251,"`).
    #[test]
    fn legacy_comma_rgb_parses() {
        assert_eq!(
            Color::parse("rgb(251, 44, 54)").unwrap().to_rgba8(),
            [251, 44, 54, 255]
        );
        assert_eq!(
            Color::parse("rgb(251,44,54)").unwrap().to_rgba8(),
            [251, 44, 54, 255]
        );
        // rgba() with a 4th alpha component (number and percent forms).
        assert_eq!(
            Color::parse("rgba(251, 44, 54, 0.5)").unwrap().to_rgba8()[3],
            128
        );
        assert_eq!(
            Color::parse("rgb(251, 44, 54, 50%)").unwrap().to_rgba8()[3],
            128
        );
        // Percentage components + commas.
        assert_eq!(
            Color::parse("rgba(98.4%, 17.3%, 21.2%, 1)")
                .unwrap()
                .to_rgba8(),
            [251, 44, 54, 255]
        );
        // Wrong arity stays a precise error, never a panic.
        assert!(Color::parse("rgb(1,2)").is_err());
        assert!(Color::parse("rgb(1,2,3,4,5)").is_err());
        assert!(Color::parse("rgb(1,,3)").is_err());
    }

    #[test]
    fn alpha_syntax() {
        let c = Color::parse("oklch(0.5 0.1 180 / 40%)").unwrap();
        assert!((c.alpha - 0.4).abs() < 1e-6);
        let c = Color::parse("oklch(0.5 0.1 180 / none)").unwrap();
        assert_eq!(c.alpha, 0.0);
    }
}
