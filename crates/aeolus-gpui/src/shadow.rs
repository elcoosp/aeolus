//! The GPUI shadow tier (architecture plan §3.7's `shadow-*` row:
//! "`shadow-*` (multi-layer v4 presets) → `box_shadow: Vec<BoxShadow>`
//! → direct; note: 1 `Vec` alloc/element"). GPUI's own box-shadow list
//! carries exactly the shape v4's shadow machinery composes — offset,
//! blur, spread, colour, and the `inset` flag the pinned surface has —
//! so the `shadow-*` families the registry lowers into
//! [`Prop::BoxShadow`] / [`Prop::ShadowColor`] accumulate into a
//! [`ShadowState`] and render as the v4 band list instead of the silent
//! no-op the un-mapped `_ => style` arm used to be.
//!
//! The lane's data is CLOSED-WORLD: the registry's shadow row lowers
//! exactly the `SHADOW_PRESETS` table's texts — the seven preset
//! `var(--tw-shadow-{name})` references, the `0 0 #0000` clear and the
//! `inset 0 0 0 0 currentColor` placeholder (multi-token arbitrary
//! lists refuse at compile, so nothing else reaches the walk). The
//! preset band texts quote the audited v4 default theme
//! (`aeolus-core/data/tailwind-default-theme.css`'s `--shadow-*`
//! tokens) verbatim — the same texts the CSS emit arm renders and the
//! plan bridge resolves through the oracle's own theme reader — and a
//! differential pin walks the two sources against each other, the
//! same drift discipline the registry tables run.
//!
//! The composition is v4's own (`--tw-shadow-colored`, utilities.ts):
//! a `shadow-{color}` recolour replaces EVERY band colour with the
//! shadow colour (the premultiplied `/opacity` modifier riding the
//! baked colour), the bands' geometry unchanged. The `currentColor`
//! band (the `shadow-inner` placeholder) reads the element's own text
//! colour, the same cascade the ring's `currentColor` default walks.

use gpui::{BoxShadow, Hsla, hsla, px, rgba};

use aeolus_ir::{Prop, Val};

/// The audited v4 default shadow presets: preset name → the composed
/// band text, quoted verbatim from the theme file's `--shadow-*`
/// tokens (the differential pin walks this table against that file).
/// The names match the registry row's own `SHADOW_PRESETS` keys.
pub const SHADOW_PRESET_TEXTS: &[(&str, &str)] = &[
    ("2xs", "0 1px rgb(0 0 0 / 0.05)"),
    ("xs", "0 1px 2px 0 rgb(0 0 0 / 0.05)"),
    (
        "sm",
        "0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)",
    ),
    (
        "md",
        "0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)",
    ),
    (
        "lg",
        "0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)",
    ),
    (
        "xl",
        "0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)",
    ),
    ("2xl", "0 25px 50px -12px rgb(0 0 0 / 0.25)"),
];

/// The static stand-in for a `currentColor` band when the program
/// carries no text colour (the `shadow-inner` placeholder alone): the
/// inherited colour is unknowable in a frozen refinement, so the
/// neutral black paints (documented, unlike a silent drop) — the
/// ring's own `currentColor` fallback.
const CURRENT_COLOR_FALLBACK: Hsla = hsla(0.0, 0.0, 0.0, 1.0);

/// One composed shadow band, in GPUI's own units. `Copy` so the
/// accumulator rides the interaction closures like every other state
/// (the `RingState` shape); `current_color` marks the band whose
/// colour is the element's own text colour (the `currentColor`
/// keyword), resolved at composition time.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Band {
    dx: f32,
    dy: f32,
    blur: f32,
    spread: f32,
    inset: bool,
    color: Hsla,
    current_color: bool,
}

/// Parse one colour token in the audited band grammar: the slash form
/// (`rgb(0 0 0 / 0.1)`), a hex spelling (`#0000`, `#fb2c36`) or
/// `currentColor` (the marker the composition resolves against the
/// text colour). Everything else is the honest `None`.
fn band_color(token: &str) -> Option<(Hsla, bool)> {
    if token == "currentColor" {
        return Some((CURRENT_COLOR_FALLBACK, true));
    }
    if let Some(hex) = token.strip_prefix('#') {
        let byte = |part: &str| u8::from_str_radix(part, 16).ok();
        return match hex.len() {
            6 => {
                let r = byte(&hex[0..2])?;
                let g = byte(&hex[2..4])?;
                let b = byte(&hex[4..6])?;
                Some((rgba(u32::from_be_bytes([r, g, b, 255])).into(), false))
            }
            // The shorthand spellings widen each nibble (v4's `#0000`
            // clear rides the 4-digit form).
            4 => {
                let widen = |n: u8| u16::from(n) * 17;
                let r = byte(&hex[0..1])?;
                let g = byte(&hex[1..2])?;
                let b = byte(&hex[2..3])?;
                let a = byte(&hex[3..4])?;
                Some((
                    rgba(u32::from_be_bytes([
                        widen(r) as u8,
                        widen(g) as u8,
                        widen(b) as u8,
                        widen(a) as u8,
                    ]))
                    .into(),
                    false,
                ))
            }
            8 => {
                let r = byte(&hex[0..2])?;
                let g = byte(&hex[2..4])?;
                let b = byte(&hex[4..6])?;
                let a = byte(&hex[6..8])?;
                Some((rgba(u32::from_be_bytes([r, g, b, a])).into(), false))
            }
            _ => None,
        };
    }
    // The slash form: `rgb(r g b / a)` (the audited texts' own
    // spelling; `rgba(r g b / a)` rides the same read).
    let inner = match (token.strip_prefix("rgb("), token.strip_prefix("rgba(")) {
        (Some(rest), _) | (None, Some(rest)) => rest.strip_suffix(')')?,
        _ => return None,
    };
    let mut parts = inner.split('/');
    let channels = parts.next()?.trim();
    let alpha_text = parts.next()?.trim();
    if parts.next().is_some() {
        return None;
    }
    let mut numbers = channels.split_whitespace();
    let r = numbers.next()?.parse::<f32>().ok()?;
    let g = numbers.next()?.parse::<f32>().ok()?;
    let b = numbers.next()?.parse::<f32>().ok()?;
    if numbers.next().is_some() {
        return None;
    }
    let clamp_byte = |v: f32| v.round().clamp(0.0, 255.0) as u8;
    let a = alpha_text.parse::<f32>().ok()?;
    Some((
        rgba(u32::from_be_bytes([
            clamp_byte(r),
            clamp_byte(g),
            clamp_byte(b),
            (a * 255.0).round().clamp(0.0, 255.0) as u8,
        ]))
        .into(),
        false,
    ))
}

/// The band's tokens, whitespace-split OUTSIDE the function parens —
/// the audited texts' `rgb(0 0 0 / 0.1)` spelling carries internal
/// spaces the naive split would tear apart.
fn split_band_tokens(text: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut depth = 0usize;
    let mut start: Option<usize> = None;
    for (index, ch) in text.char_indices() {
        match ch {
            '(' => {
                depth += 1;
                if start.is_none() {
                    start = Some(index);
                }
            }
            ')' => depth = depth.saturating_sub(1),
            c if c.is_whitespace() && depth == 0 => {
                if let Some(s) = start.take() {
                    tokens.push(&text[s..index]);
                }
            }
            _ if start.is_none() => start = Some(index),
            _ => {}
        }
    }
    if let Some(s) = start {
        tokens.push(&text[s..]);
    }
    tokens
}

/// Parse one band: `[inset] dx dy [blur] [spread] color` — the px
/// numbers first (unitless zero is v4's own spelling for the
/// structural bands), the colour token last. Anything else is the
/// honest `None`.
fn parse_band(text: &str) -> Option<Band> {
    let tokens = split_band_tokens(text.trim());
    let (inset, tokens) = match tokens.split_first() {
        Some((first, rest)) if *first == "inset" => (true, rest),
        _ => (false, &tokens[..]),
    };
    let mut numbers = Vec::new();
    let mut color = None;
    for token in tokens {
        if color.is_some() {
            return None;
        }
        let number = token
            .strip_suffix("px")
            .unwrap_or(token)
            .parse::<f32>()
            .ok()
            .filter(|_| token.strip_suffix("px").is_some() || *token == "0");
        match number {
            Some(n) if numbers.len() < 4 => numbers.push(n),
            _ => color = Some(band_color(token)?),
        }
    }
    let color = color?;
    let (dx, dy) = (numbers.first().copied()?, numbers.get(1).copied()?);
    Some(Band {
        dx,
        dy,
        blur: numbers.get(2).copied().unwrap_or(0.0),
        spread: numbers.get(3).copied().unwrap_or(0.0),
        inset,
        color: color.0,
        current_color: color.1,
    })
}

/// Parse the composed band text into the lane's own bands — the
/// closed-world forms the registry's shadow row lowers:
///
/// * a `var(--tw-shadow-{name})` preset reference resolves through the
///   audited [`SHADOW_PRESET_TEXTS`] table;
/// * a literal band list (`0 0 #0000`, `inset 0 0 0 0 currentColor`)
///   parses directly, `currentColor` bands carrying the marker the
///   composition resolves against the text colour;
///
/// an unknown text is the honest `None` (the walk's documented skip —
/// this path carries no note channel).
fn parse_bands(text: &str) -> Option<Vec<Band>> {
    let resolved: &str = if let Some(inner) = text
        .strip_prefix("var(--tw-shadow-")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let (_, value) = SHADOW_PRESET_TEXTS
            .iter()
            .find(|(name, _)| *name == inner)?;
        value
    } else {
        text
    };
    resolved.split(',').map(parse_band).collect()
}

/// The bands as GPUI's own box-shadow list (the `currentColor` bands
/// through the documented neutral stand-in — the stateful composition
/// resolves the real text colour).
fn bands_to_shadows(bands: &[Band]) -> Vec<BoxShadow> {
    bands
        .iter()
        .map(|band| BoxShadow {
            color: if band.current_color {
                CURRENT_COLOR_FALLBACK
            } else {
                band.color
            },
            offset: gpui::point(px(band.dx), px(band.dy)),
            blur_radius: px(band.blur),
            spread_radius: px(band.spread),
            inset: band.inset,
        })
        .collect()
}

/// Parse the composed band text into GPUI's own box-shadow list — the
/// lane's public parse, the [`parse_bands`] forms composed at the
/// documented stand-in colour.
pub fn parse_shadow_bands(text: &str) -> Option<Vec<BoxShadow>> {
    parse_bands(text).map(|bands| bands_to_shadows(&bands))
}

/// The `shadow-*` machinery one walk accumulates — the composed band
/// list (the preset winner, last-write-wins), the `shadow-{color}`
/// recolour and the text colour the `currentColor` band reads.
///
/// Every field is `Copy`, so the state rides the interaction closures
/// like the ring/gradient/grid states: the walk feeds the accumulator
/// in cascade order (base layers first, then the fired variant layers),
/// so a later `shadow-lg` replaces an earlier `shadow-sm` exactly like
/// every other longhand family, and `hover:shadow-md` composes with
/// the base's bands the way CSS's vars persist across state layers.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShadowState {
    band_a: Option<Band>,
    band_b: Option<Band>,
    recolor: Option<Hsla>,
    text_color: Option<Hsla>,
}

impl ShadowState {
    /// Record one longhand the walk saw, with its frozen row text (the
    /// annotation a bare [`Val::Kw`] slot cannot recover — the preset
    /// references are the row's own composed texts). Shadow longhands
    /// mutate the machinery; [`Prop::Color`] records the
    /// `currentColor` source. Every other prop is a no-op here (the
    /// caller applies those through the per-decl mapping as before).
    pub fn apply(&mut self, prop: Prop, val: Val, keyword: &str) {
        match prop {
            Prop::BoxShadow => {
                if let Some(bands) = parse_bands(keyword) {
                    self.band_a = bands.first().copied();
                    self.band_b = bands.get(1).copied();
                }
            }
            Prop::ShadowColor => {
                if let Some(color) = hsla_of(val) {
                    self.recolor = Some(color);
                }
            }
            Prop::Color => {
                if let Some(color) = hsla_of(val) {
                    self.text_color = Some(color);
                }
            }
            _ => {}
        }
    }

    /// The v4 band list the state composes — the winner preset's bands
    /// (the `currentColor` band through the text colour), the recolour
    /// applied over every band's colour (v4's `--tw-shadow-colored`).
    /// A recolour without bands composes NOTHING (v4's
    /// `--tw-shadow-color` colors an empty shadow), and an empty state
    /// never touches the refinement — the caller sets the whole list
    /// in one `shadow()` call beside the ring's bands (GPUI's
    /// `Vec<BoxShadow>` is the composed surface, the walk the only
    /// writer on this path).
    pub fn shadows(&self) -> Vec<BoxShadow> {
        let color_of = |band: &Band| -> Hsla {
            if band.current_color {
                self.text_color.unwrap_or(CURRENT_COLOR_FALLBACK)
            } else {
                self.recolor.unwrap_or(band.color)
            }
        };
        [self.band_a, self.band_b]
            .iter()
            .flatten()
            .map(|band| BoxShadow {
                color: color_of(band),
                offset: gpui::point(px(band.dx), px(band.dy)),
                blur_radius: px(band.blur),
                spread_radius: px(band.spread),
                inset: band.inset,
            })
            .collect()
    }
}

/// The [`Hsla`] a colour carries — the baked `Rgba` the theme pass's
/// T0/T1 forms produce (the `/opacity` modifier premultiplied in),
/// through the same conversion the crate's other colour slots run.
fn hsla_of(val: Val) -> Option<Hsla> {
    match val {
        Val::Rgba([r, g, b, a]) => Some(rgba(u32::from_be_bytes([r, g, b, a])).into()),
        _ => None,
    }
}
