//! CSS-first theme registry.
//!
//! `TailwindTheme::from_theme_css` parses Tailwind v4 `@theme` blocks
//! (including `@theme inline` / `@theme static`, which behave identically for
//! token *resolution* in v1 — documented limitation) plus the light/dark
//! `:root`/`.dark` variable blocks that Shadcn uses.
//!
//! Runtime mutation is explicit: swap the token map, bump the version (`TailwindTheme::version`),
//! and re-resolve. Aeolus does not pretend to have CSS custom-property
//! propagation.

use crate::ast::Breakpoint;
use crate::color::{Color, ParseError};
use std::collections::HashMap;
use std::sync::Arc;

/// Tailwind CSS v4 default color palette (`@theme default`, colors only),
/// verbatim from tailwindcss v4.1.17 `packages/tailwindcss/theme.css`.
/// Every theme is seeded with it; user declarations merge on top
/// (same key → later wins), matching v4's `@theme` layering.
pub const TAILWIND_DEFAULT_PALETTE: &str = include_str!("../data/tailwind-default-palette.css");

/// A typed theme token.
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    /// A color token (Oklab + alpha).
    Color(Color),
    /// Length in px (rem resolved at 16px, the v4 root assumption).
    Length(f32),
    /// `--text-sm` font size + paired `--text-sm--line-height`.
    FontSize {
        /// Font size in px.
        size: f32,
        /// Paired line height in px, when declared.
        line_height: Option<f32>,
    },
    /// `--shadow-*` presets.
    Shadow(BoxShadowSpec),
}

/// One shadow layer (v1: single-layer presets).
#[derive(Clone, Debug, PartialEq)]
pub struct BoxShadowSpec {
    /// Horizontal offset in px.
    pub offset_x: f32,
    /// Vertical offset in px.
    pub offset_y: f32,
    /// Blur radius in px.
    pub blur: f32,
    /// Spread in px.
    pub spread: f32,
    /// Shadow color.
    pub color: Color,
}

/// A parsed, immutable-by-convention theme snapshot.
#[derive(Clone, Debug, Default)]
pub struct TailwindTheme {
    tokens: HashMap<String, Token>,
    breakpoints: Vec<(Breakpoint, f32)>,
    /// `calc(var(--radius) - 2px)` values deferred until their deps exist.
    raw_calcs: Vec<(String, String)>,
    /// `--text-sm--line-height` ratios pending their `--text-sm` parent.
    pending_lh: Vec<(String, f32)>,
    /// Source CSS, re-parsed when deriving the dark variant so
    /// `@theme inline` var chains re-resolve against the dark palette.
    source_css: String,
    /// Monotonic version — bump on swap so caches can key on it.
    version: u64,
    /// True when the snapshot came from the dark palette.
    dark_mode: bool,
}

impl TailwindTheme {
    /// A theme with no user tokens (still seeded with the default palette).
    pub fn new() -> Self {
        Self::default()
    }

    /// Monotonic theme version, used as a cache key component.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Was this snapshot derived from the `.dark` palette?
    pub fn dark_mode(&self) -> bool {
        self.dark_mode
    }

    /// Look up a color token. `theme.color("primary")` matches
    /// `--color-primary` (v4 registry naming) or `--primary` (shadcn app
    /// naming, mapped via `@theme inline`).
    pub fn color(&self, name: &str) -> Option<Color> {
        if let Some(Token::Color(c)) = self.tokens.get(&format!("--color-{name}")) {
            return Some(*c);
        }
        match self.tokens.get(&format!("--{name}")) {
            Some(Token::Color(c)) => Some(*c),
            _ => None,
        }
    }

    /// Look up a length token (`--spacing`, `--radius-md`, ...) in px.
    pub fn length(&self, name: &str) -> Option<f32> {
        match self.tokens.get(&format!("--{name}")) {
            Some(Token::Length(l)) => Some(*l),
            _ => None,
        }
    }

    /// The spacing base (`--spacing`, v4 default `0.25rem` = 4px).
    pub fn spacing(&self) -> f32 {
        self.length("spacing").unwrap_or(4.0)
    }

    /// Look up a `--text-{name}` font-size token (+ paired line height).
    pub fn font_size(&self, name: &str) -> Option<(f32, Option<f32>)> {
        match self.tokens.get(&format!("--text-{name}")) {
            Some(Token::FontSize { size, line_height }) => Some((*size, *line_height)),
            _ => None,
        }
    }

    /// Look up a `--shadow-{name}` preset.
    pub fn shadow(&self, name: &str) -> Option<BoxShadowSpec> {
        match self.tokens.get(&format!("--shadow-{name}")) {
            Some(Token::Shadow(s)) => Some(s.clone()),
            _ => None,
        }
    }

    /// Look up a `--radius-{name}` token in px.
    pub fn radius(&self, name: &str) -> Option<f32> {
        self.length(&format!("radius-{name}"))
    }

    /// The effective breakpoint px for `b` (user override or v4 default).
    pub fn breakpoint_px(&self, b: Breakpoint) -> f32 {
        self.breakpoints
            .iter()
            .find(|(n, _)| *n == b)
            .map(|(_, px)| *px)
            .unwrap_or_else(|| b.default_px())
    }

    /// All color token names (for `tw_audit`), without prefixes.
    pub fn color_names(&self) -> Vec<&str> {
        self.tokens
            .keys()
            .filter_map(|k| {
                k.strip_prefix("--color-")
                    // shadcn app naming: --primary etc. are colors too
                    .or_else(|| is_color_only_key(k).then(|| k.trim_start_matches("--")))
            })
            .collect()
    }

    /// Parse `@theme`/`:root`/`.dark` CSS into a fresh theme.
    pub fn from_theme_css(css: &str) -> Result<Self, ParseError> {
        let mut theme = Self::new();
        // v4 semantics: the default palette is the baseline; user `@theme`
        // declarations are merged over it (same key → later wins).
        theme.absorb_css(TAILWIND_DEFAULT_PALETTE)?;
        theme.absorb_css(css)?;
        Ok(theme)
    }

    /// Parse CSS into an existing theme (merge semantics). Bumps `version`.
    pub fn absorb_css(&mut self, css: &str) -> Result<(), ParseError> {
        self.source_css.push('\n');
        self.source_css.push_str(css);
        self.absorb_declarations(&extract_declarations(css))
    }

    fn absorb_declarations(
        &mut self,
        decls: &[(BlockKind, String, String)],
    ) -> Result<(), ParseError> {
        for (_kind, key, value) in decls {
            self.insert_var(key, value)?;
        }
        // Deferred calc() pass: deps now exist.
        let pending = std::mem::take(&mut self.raw_calcs);
        for (key, expr) in pending {
            let (raw, is_px) = eval_calc(&expr, &self.tokens)?;
            if let Some(parent) = key.strip_suffix("--line-height")
                && parent.starts_with("--text-")
            {
                // `calc(1.25 / 0.875)` is a unitless ratio of font-size.
                self.pending_lh.push((parent.to_string(), raw));
                continue;
            }
            self.store_length(&key, if is_px { raw } else { raw * 16.0 });
        }
        // Pair any pending line-height ratios whose parents arrived earlier.
        let pending_lh = std::mem::take(&mut self.pending_lh);
        for (parent, ratio) in pending_lh {
            if let Some(Token::FontSize { size, line_height }) = self.tokens.get_mut(&parent) {
                let sz = *size;
                *line_height = Some(sz * ratio);
            } else {
                // Parent arrives later; keep pending for the insert path.
                self.pending_lh.push((parent, ratio));
            }
        }
        self.version += 1;
        Ok(())
    }

    /// Build the dark variant by re-parsing the source CSS with the
    /// `.dark` overrides appended, so `@theme inline` var chains
    /// (`--color-primary: var(--primary)`) re-resolve against the dark
    /// palette — the propagation CSS gives us for free and Aeolus does
    /// explicitly (mission brief §3.5).
    pub fn dark_variant(&self, dark_css: &str) -> Result<Self, ParseError> {
        let mut dark = Self::new();
        dark.dark_mode = true;
        // 1. light palette (`:root` etc.)
        let all = extract_declarations(&self.source_css);
        let roots: Vec<_> = all
            .iter()
            .filter(|(k, _, _)| *k != BlockKind::Theme)
            .cloned()
            .collect();
        let themes: Vec<_> = all
            .iter()
            .filter(|(k, _, _)| *k == BlockKind::Theme)
            .cloned()
            .collect();
        dark.absorb_declarations(&roots)?;
        // 2. dark palette overrides
        dark.absorb_declarations(&extract_declarations(dark_css))?;
        // 3. re-resolve `@theme inline` var chains against the final palette
        dark.absorb_declarations(&themes)?;
        Ok(dark)
    }

    /// Mark the snapshot immutable and share it.
    pub fn seal(mut self) -> Arc<Self> {
        self.version += 1;
        Arc::new(self)
    }

    /// Store a resolved length, patching a paired `--text-*--line-height`.
    fn store_length(&mut self, key: &str, v: f32) {
        if let Some(parent) = key.strip_suffix("--line-height")
            && parent.starts_with("--text-")
            && let Some(Token::FontSize { line_height, .. }) = self.tokens.get_mut(parent)
        {
            *line_height = Some(v);
            return;
        }
        self.tokens.insert(key.to_string(), Token::Length(v));
    }

    /// Substitute a single `var(--x)` when the token is a known length.
    fn try_var_length(&self, value: &str) -> Option<f32> {
        let inner = value.trim().strip_prefix("var(")?.strip_suffix(')')?;
        match self.tokens.get(inner.trim()) {
            Some(Token::Length(l)) => Some(*l),
            _ => None,
        }
    }

    fn insert_var(&mut self, key: &str, value: &str) -> Result<(), ParseError> {
        // Defer calc() values until all their var() deps are parsed.
        if value.trim_start().starts_with("calc(") {
            self.raw_calcs.push((key.to_string(), value.to_string()));
            return Ok(());
        }
        // A bare `var(--x)` for a length key: substitute now if known,
        // otherwise defer like a calc.
        if value.trim_start().starts_with("var(")
            && !key.starts_with("--color-")
            && !is_color_only_key(key)
        {
            if let Some(v) = self.try_var_length(value) {
                self.store_length(key, v);
                return Ok(());
            }
            self.raw_calcs.push((key.to_string(), value.to_string()));
            return Ok(());
        }
        // --text-sm--line-height patches its parent --text-sm token.
        if let Some(parent) = key.strip_suffix("--line-height")
            && parent.starts_with("--text-")
        {
            let lh = parse_length(value)?;
            if let Some(Token::FontSize { line_height, .. }) = self.tokens.get_mut(parent) {
                *line_height = Some(lh);
                return Ok(());
            }
            // Parent not seen yet: store under the pair key, patched later.
            self.tokens.insert(key.to_string(), Token::Length(lh));
            return Ok(());
        }
        let token = match key {
            k if k.starts_with("--text-") => {
                let size = parse_length(value)?;
                let lh = match self.tokens.get(&format!("{k}--line-height")) {
                    Some(Token::Length(l)) => Some(*l),
                    _ => None,
                };
                Token::FontSize {
                    size,
                    line_height: lh,
                }
            }
            k if k.starts_with("--shadow-") => {
                Token::Shadow(parse_shadow_spec(&resolve_vars(value, &self.tokens)?)?)
            }
            // Value-based sniffing: any oklch()/rgb()/hsl()/hex/color-mix value is a
            // color token regardless of its name (shadcn uses --chart-1..5,
            // --sidebar-*, arbitrary app tokens). Name prefixes stay as fallbacks
            // for var() indirection (e.g. `--color-x: var(--x)`).
            _ if value_is_color(value, &self.tokens) => {
                Token::Color(Color::parse(&resolve_vars(value, &self.tokens)?)?)
            }
            k if k.starts_with("--color-") => {
                Token::Color(Color::parse(&resolve_vars(value, &self.tokens)?)?)
            }
            k if is_color_only_key(k) => {
                Token::Color(Color::parse(&resolve_vars(value, &self.tokens)?)?)
            }
            k if k.starts_with("--breakpoint-") => {
                let px = parse_length(value)?;
                if let Some(b) = Breakpoint::from_name(k.trim_start_matches("--breakpoint-")) {
                    self.breakpoints.retain(|(n, _)| *n != b);
                    self.breakpoints.push((b, px));
                }
                return Ok(());
            }
            _ => Token::Length(parse_length(value)?),
        };
        self.tokens.insert(key.to_string(), token);
        Ok(())
    }
}

fn value_is_color(value: &str, tokens: &HashMap<String, Token>) -> bool {
    let v = value.trim();
    if v.starts_with("var(") {
        return false;
    }
    let resolved = resolve_vars(v, tokens).unwrap_or_else(|_| v.to_string());
    Color::parse(&resolved).is_ok()
}

fn is_color_only_key(k: &str) -> bool {
    // shadcn app themes use `--primary`, `--border`, ... (no `--color-` prefix).
    matches!(
        k,
        "--background"
            | "--foreground"
            | "--card"
            | "--card-foreground"
            | "--popover"
            | "--popover-foreground"
            | "--primary"
            | "--primary-foreground"
            | "--secondary"
            | "--secondary-foreground"
            | "--muted"
            | "--muted-foreground"
            | "--accent"
            | "--accent-foreground"
            | "--destructive"
            | "--destructive-foreground"
            | "--border"
            | "--input"
            | "--ring"
            | "--sidebar"
            | "--sidebar-foreground"
            | "--sidebar-primary"
            | "--sidebar-primary-foreground"
            | "--sidebar-accent"
            | "--sidebar-accent-foreground"
            | "--sidebar-border"
            | "--sidebar-ring"
            | "--surface"
            | "--surface-foreground"
            | "--selection"
            | "--selection-foreground"
    )
}

/// Substitute already-parsed `var(--x)` references with their CSS text so a
/// color defined as `var(--color-blue-300)` can still be parsed.
fn resolve_vars(value: &str, tokens: &HashMap<String, Token>) -> Result<String, ParseError> {
    if !value.contains("var(") {
        return Ok(value.to_string());
    }
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(i) = rest.find("var(") {
        out.push_str(&rest[..i]);
        let after = &rest[i + 4..];
        let Some(end) = after.find(')') else {
            return Err(ParseError::new("unbalanced var() in theme value"));
        };
        let name = after[..end].trim();
        let substituted = match tokens.get(name) {
            Some(Token::Color(c)) => c.to_oklch_string(),
            Some(Token::Length(l)) => format!("{l}px"),
            _ => {
                // Unknown: keep var text; parse will fail with a clear error.
                format!("var({name})")
            }
        };
        out.push_str(&substituted);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Parse a length: `0.25rem`, `4px`. Bare numbers are rejected (v4 theme
/// lengths always carry units).
pub fn parse_length(s: &str) -> Result<f32, ParseError> {
    let s = s.trim();
    if let Some(rem) = s.strip_suffix("rem") {
        return rem
            .trim()
            .parse::<f32>()
            .map(|v| v * 16.0)
            .map_err(|_| bad_length(s));
    }
    if let Some(px) = s.strip_suffix("px") {
        return px.trim().parse::<f32>().map_err(|_| bad_length(s));
    }
    Err(bad_length(s))
}

fn bad_length(s: &str) -> ParseError {
    ParseError::new(format!("theme length requires px/rem unit: {s:?}"))
}

fn parse_shadow_spec(value: &str) -> Result<BoxShadowSpec, ParseError> {
    // v4 defaults ship multi-layer shadows (`--shadow-sm: 0 1px 3px 0 rgb(0 0 0 /
    // 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)`). The locked StylePlan IR has a
    // single `BoxShadow` slot (§3.1), so keep the first (dominant) layer.
    let value = {
        let mut depth = 0usize;
        let mut cut = value.len();
        for (i, ch) in value.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => depth = depth.saturating_sub(1),
                ',' if depth == 0 => {
                    cut = i;
                    break;
                }
                _ => {}
            }
        }
        &value[..cut]
    };
    // v4 shape: `0px 1px 2px 0px rgb(0 0 0 / 5%)` / oklch(...) — the color
    // may itself contain spaces, so scan from the last numeric token.
    let mut nums: Vec<f32> = Vec::with_capacity(4);
    let mut color_start = value.len();
    for (idx, part) in value.split_whitespace().enumerate() {
        if idx >= 4 || !(part.ends_with("px") || part.parse::<f32>().is_ok()) {
            color_start = value.find(part).unwrap_or(value.len());
            break;
        }
        if let Ok(v) = part.trim_end_matches("px").parse::<f32>() {
            nums.push(v);
        }
    }
    let color_src = value[color_start..].trim();
    let color = if color_src.is_empty() {
        Color::TRANSPARENT
    } else {
        Color::parse(color_src)?
    };
    Ok(BoxShadowSpec {
        offset_x: *nums.first().unwrap_or(&0.0),
        offset_y: *nums.get(1).unwrap_or(&0.0),
        blur: *nums.get(2).unwrap_or(&0.0),
        spread: *nums.get(3).unwrap_or(&0.0),
        color,
    })
}

/// Evaluate a `calc()` expression with `var()` refs, `+ - * /`, parentheses
/// and **standard operator precedence** (shunting-yard, ~50 LOC, zero deps).
///
/// The Shadcn v4 strings only need `var(--radius) - 2px` shapes, but a
/// left-to-right evaluator silently mis-computes `4px + var(--r) * 2` —
/// and precedence bugs are silent visual bugs, so the evaluator implements
/// the real CSS calc() grammar subset: unary minus, `+ - * /`, nested
/// parentheses, substituted `var()`s.
///
/// Unit rule: if the expression (after substitution) contains any px-suffixed
/// token or any substituted var, the result is px; otherwise the expression
/// was dimensionless (e.g. `calc(1.25 / 0.875)`) and the result is rem (×16),
/// matching v4's `--text-sm--line-height` idiom.
fn eval_calc(expr: &str, tokens: &HashMap<String, Token>) -> Result<(f32, bool), ParseError> {
    let inner = expr
        .trim()
        .strip_prefix("calc(")
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| ParseError::new(format!("not a calc(): {expr:?}")))?;

    let mut resolved = String::new();
    let mut rest = inner;
    let mut had_var = false;
    while let Some(i) = rest.find("var(") {
        resolved.push_str(&rest[..i]);
        let after = &rest[i + 4..];
        let end = after
            .find(')')
            .ok_or_else(|| ParseError::new("unbalanced var()"))?;
        let name = after[..end].trim();
        let v = match tokens.get(name) {
            Some(Token::Length(l)) => *l,
            _ => {
                return Err(ParseError::new(format!(
                    "calc() references unknown token {name}"
                )));
            }
        };
        had_var = true;
        resolved.push_str(&format!("{v} "));
        rest = &after[end + 1..];
    }
    resolved.push_str(rest);

    let is_px = had_var || resolved.contains("px");

    // --- tokenize: numbers ("4px" -> 4.0), operators, parens ----------------
    // Unary +/- in operator position (start, after an operator or '(') is
    // folded into the following number, or, when it guards a parenthesized
    // group, rewritten as the binary `0 <op> (group)`.
    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Tok {
        Num(f32),
        Op(char),
        LParen,
        RParen,
    }
    let mut toks: Vec<Tok> = Vec::new();
    let mut sign: f32 = 1.0; // pending unary sign for the next number
    let mut wants_operand = true; // true = a unary +/- may appear here
    let mut it = resolved.split_whitespace().peekable();
    while let Some(raw) = it.next() {
        let mut piece = raw;
        // Leading parens may be glued to the rest of the token: `(4px`.
        while let Some(rest) = piece.strip_prefix('(') {
            toks.push(Tok::LParen);
            piece = rest;
        }
        if piece.is_empty() {
            continue;
        }
        // Split a trailing `)` off: `2)`, `4px)`.
        let mut value_part = piece;
        let mut trailing_parens: usize = 0;
        while let Some(rest) = value_part.strip_suffix(')') {
            trailing_parens += 1;
            value_part = rest;
        }
        if !value_part.is_empty() {
            if value_part.len() == 1 && "+-*/".contains(value_part) {
                let op = value_part.chars().next().unwrap();
                if wants_operand {
                    if op == '-' || op == '+' {
                        // Unary sign: fold into the next number, or convert
                        // a guarded paren group into `0 <op> (group)`.
                        let next_paren = it.peek().is_some_and(|n| n.starts_with('('));
                        if next_paren {
                            toks.push(Tok::Num(0.0));
                            toks.push(Tok::Op(op));
                            // wants_operand stays true: still waiting for the
                            // operand, which is the paren group.
                        } else if op == '-' {
                            sign = -sign;
                        }
                    } else {
                        return Err(ParseError::new(format!(
                            "operator {op:?} needs a left operand"
                        )));
                    }
                } else {
                    toks.push(Tok::Op(op));
                    wants_operand = true;
                }
            } else {
                let n: f32 = if let Some(stripped) = value_part.strip_suffix("px") {
                    stripped.parse()
                } else {
                    value_part.parse()
                }
                .map_err(|_| ParseError::new(format!("bad calc token {raw:?}")))?;
                toks.push(Tok::Num(n * sign));
                sign = 1.0;
                wants_operand = false;
            }
        }
        for _ in 0..trailing_parens {
            toks.push(Tok::RParen);
            wants_operand = false;
        }
    }
    if wants_operand && !toks.is_empty() {
        return Err(ParseError::new("calc() ends with an operator"));
    }

    // --- shunting-yard -> RPN (standard precedence: * / over + -) ----------
    fn prec(op: char) -> u8 {
        match op {
            '*' | '/' => 2,
            _ => 1,
        }
    }
    let mut rpn: Vec<Tok> = Vec::with_capacity(toks.len());
    let mut ops: Vec<char> = Vec::new();
    for t in toks {
        match t {
            Tok::Num(v) => rpn.push(Tok::Num(v)),
            Tok::Op(op) => {
                while ops.last().is_some_and(|&o| o != '(' && prec(o) >= prec(op)) {
                    rpn.push(Tok::Op(ops.pop().unwrap()));
                }
                ops.push(op);
            }
            Tok::LParen => ops.push('('),
            Tok::RParen => loop {
                match ops.pop() {
                    Some('(') | None => break,
                    Some(o) => rpn.push(Tok::Op(o)),
                }
            },
        }
    }
    while let Some(o) = ops.pop() {
        if o == '(' {
            return Err(ParseError::new("unbalanced '(' in calc()"));
        }
        rpn.push(Tok::Op(o));
    }

    // --- evaluate the RPN ----------------------------------------------------
    let mut stack: Vec<f32> = Vec::new();
    for t in rpn {
        match t {
            Tok::Num(v) => stack.push(v),
            Tok::Op(op) => {
                let (Some(b), Some(a)) = (stack.pop(), stack.pop()) else {
                    return Err(ParseError::new("calc() operand missing"));
                };
                stack.push(match op {
                    '+' => a + b,
                    '-' => a - b,
                    '*' => a * b,
                    '/' => a / b,
                    _ => return Err(ParseError::new("unknown calc() operator")),
                });
            }
            Tok::LParen | Tok::RParen => {
                return Err(ParseError::new("unbalanced parens in calc()"));
            }
        }
    }
    if stack.len() != 1 {
        return Err(ParseError::new("calc() left surplus operands"));
    }
    let v = stack[0];
    Ok((v, is_px))
}

/// Which block a declaration came from (drives dark-variant re-resolution).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    /// An `@theme { ... }` block (v4 theme layer).
    Theme,
    /// A `:root` / `.dark` block (app-variable layer).
    Palette,
}

/// Extract `--key: value;` declarations from `@theme`, `:root`, `.dark`
/// blocks. Other blocks (component rules) are ignored.
pub fn extract_declarations(css: &str) -> Vec<(BlockKind, String, String)> {
    let mut out = Vec::new();
    let owned = strip_comments(css);
    let mut rest: &str = &owned;
    while let Some(open) = rest.find('{') {
        let selector = rest[..open].trim();
        let close = matching_brace(rest.as_bytes(), open);
        if close >= rest.len() {
            // Unbalanced block (no closing brace): the malformed tail is
            // ignored, never sliced out of bounds (fuzz finding).
            return out;
        }
        let body = &rest[open + 1..close];
        let kind = if selector.contains("@theme") {
            BlockKind::Theme
        } else if selector == ":root" || selector.contains(".dark") {
            BlockKind::Palette
        } else {
            rest = &rest[close + 1..];
            continue;
        };
        for decl in split_decls(body) {
            if let Some((k, v)) = decl.split_once(':') {
                let k = k.trim();
                if k.starts_with("--") && !k.starts_with("--tw-") {
                    out.push((
                        kind,
                        k.to_string(),
                        v.trim().trim_end_matches(';').to_string(),
                    ));
                }
            }
        }
        rest = &rest[close + 1..];
    }
    out
}

/// Remove `/* ... */` comments so selector classification cannot be fooled
/// by documentation text inside them.
fn strip_comments(css: &str) -> String {
    if !css.contains("/*") {
        return css.to_string();
    }
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(i) = rest.find("/*") {
        out.push_str(&rest[..i]);
        let Some(end) = rest[i + 2..].find("*/") else {
            return out;
        };
        rest = &rest[i + 2 + end + 2..];
    }
    out.push_str(rest);
    out
}

fn matching_brace(bytes: &[u8], open: usize) -> usize {
    let mut depth = 0usize;
    for (i, &b) in bytes.iter().enumerate().skip(open) {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
    }
    bytes.len()
}

fn split_decls(body: &str) -> impl Iterator<Item = &str> {
    let mut items = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (i, b) in body.bytes().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b';' if depth == 0 => {
                items.push(&body[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    items.push(&body[start..]);
    items.into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fuzz finding: a CSS string ending in an UNBALANCED block must not
    /// panic (matching_brace returns the string length; `rest[close + 1..]`
    /// was out of bounds).
    #[test]
    fn unbalanced_blocks_do_not_panic() {
        assert!(TailwindTheme::from_theme_css(".dark { --x: oklch(0 0 0)").is_ok());
        assert!(TailwindTheme::from_theme_css("{a").is_ok());
        assert!(TailwindTheme::from_theme_css(":root { --primary: oklch(0.2 0 0) } {").is_ok());
        // Well-formed input still parses its declarations.
        let t = TailwindTheme::from_theme_css(":root { --radius: 10px }").unwrap();
        assert_eq!(t.length("radius"), Some(10.0));
    }

    /// v4 `calc()` uses standard operator precedence. The Shadcn strings only
    /// need `var +- px`, but a left-to-right evaluator silently mis-computes
    /// `4px + var(--r) * 2` (28 instead of 24) — a silent visual bug class.
    #[test]
    fn calc_has_operator_precedence() {
        let t = TailwindTheme::from_theme_css(
            ":root { --radius: 10px; --radius-md: calc(4px + var(--radius) * 2); --radius-lg: calc((4px + var(--radius)) * 2); --radius-sm: calc(var(--radius) / 2 - 1px); --radius-xl: calc(2 * var(--radius) - 4px); }",
        )
        .unwrap();
        assert_eq!(t.radius("md"), Some(24.0), "4 + 10*2 = 24, not (4+10)*2");
        assert_eq!(t.radius("lg"), Some(28.0), "(4 + 10) * 2 = 28");
        assert_eq!(t.radius("sm"), Some(4.0), "10 / 2 - 1 = 4");
        assert_eq!(t.radius("xl"), Some(16.0), "2 * 10 - 4 = 16");
    }

    const SHADCN_LIGHT: &str = r#"
    :root {
      --radius: 0.625rem;
      --background: oklch(1 0 0);
      --foreground: oklch(0.145 0 0);
      --primary: oklch(0.205 0 0);
      --primary-foreground: oklch(0.985 0 0);
      --border: oklch(0.922 0 0);
      --ring: oklch(0.708 0 0);
      --destructive: oklch(0.577 0.245 27.325);
    }
    @theme inline {
      --radius-sm: calc(var(--radius) - 4px);
      --radius-md: calc(var(--radius) - 2px);
      --radius-lg: var(--radius);
      --color-primary: var(--primary);
      --color-background: var(--background);
    }
    "#;

    #[test]
    fn parses_shadcn_tokens() {
        let t = TailwindTheme::from_theme_css(SHADCN_LIGHT).unwrap();
        assert_eq!(t.radius("sm"), Some(6.0));
        assert_eq!(t.radius("md"), Some(8.0));
        assert_eq!(t.radius("lg"), Some(10.0));
        let p = t.color("primary").unwrap();
        assert!((p.l - 0.205).abs() < 1e-4, "primary L = {}", p.l);
        assert!(t.color("background").is_some());
        assert!(t.color_names().contains(&"primary"));
    }

    #[test]
    fn dark_variant_overrides() {
        let light = TailwindTheme::from_theme_css(SHADCN_LIGHT).unwrap();
        let dark = light
            .dark_variant(".dark { --background: oklch(0.145 0 0); --border: oklch(1 0 0 / 10%); }")
            .unwrap();
        assert!(dark.dark_mode());
        let bg = dark.color("background").unwrap();
        assert!((bg.l - 0.145).abs() < 1e-4);
        let border = dark.color("border").unwrap();
        assert!(border.alpha < 0.11 && border.alpha > 0.09);
        // light unchanged
        assert!((light.color("background").unwrap().l - 1.0).abs() < 1e-4);
    }

    #[test]
    fn text_size_pairs_line_height() {
        let t = TailwindTheme::from_theme_css(
            "@theme { --text-sm: 0.875rem; --text-sm--line-height: calc(1.25 / 0.875); }",
        )
        .unwrap();
        let (size, lh) = t.font_size("sm").unwrap();
        assert_eq!(size, 14.0);
        let lh = lh.expect("line height resolved");
        assert!((lh - 20.0).abs() < 0.1, "lh={lh}");
    }

    #[test]
    fn breakpoint_tokens() {
        let t = TailwindTheme::from_theme_css("@theme { --breakpoint-3xl: 120rem; }").unwrap();
        assert_eq!(t.breakpoint_px(Breakpoint::Sm), 640.0);
    }

    #[test]
    fn spacing_default() {
        let t = TailwindTheme::new();
        assert_eq!(t.spacing(), 4.0);
        let t = TailwindTheme::from_theme_css("@theme { --spacing: 0.25rem; }").unwrap();
        assert_eq!(t.spacing(), 4.0);
    }
}
