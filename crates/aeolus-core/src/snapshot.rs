//! Canonical plan snapshots (v0.7, plan of record `AEOLUS_V0.7.md` §2).
//!
//! The `StylePlan` IR is the project's central artifact — v0.7 makes it a
//! PORTABLE one: [`StylePlan::to_snapshot_json`] serializes a plan into a
//! canonical, byte-stable, dependency-free JSON document and
//! [`StylePlan::from_snapshot_json`] reads it back EXACTLY
//! (`from(to(p)) == p`, pinned by `tests/snapshot_roundtrip.rs`).
//!
//! What this unlocks:
//! - **CI plan pins**: a plan snapshot checked into a repo is a diffable
//!   golden — any resolver/IR change that shifts a resolved value shows up
//!   as a one-line diff.
//! - **Cross-process persistence**: a build step can resolve class strings
//!   once and ship the plans; an app loads them without re-resolving.
//! - **Cross-language review**: the artifact is plain JSON; the schema is
//!   the IR, field-for-field.
//!
//! Canonicality rules:
//! - Fixed field order (the schema order below), compact separators, no
//!   map-order drift (`responsive` is a `BTreeMap`, `layers` a `Vec`).
//! - `f32` values print Rust's shortest round-trip form (`0.9` stays
//!   `0.9`); the reader parses them back exactly.
//! - Non-finite `f32` values (never produced by the resolver — `rounded-full`
//!   is the finite 9999px sentinel) are encoded as the strings `"NaN"`,
//!   `"inf"`, `"-inf"` and decoded symmetrically, so the writer is total
//!   and never panics on a hand-built plan.
//!
//! Strictness rules (the compatibility contract): unknown fields, unknown
//! enum names, duplicate keys, non-integral integers, truncated documents
//! and trailing content are ALL errors with a precise `path`
//! (`"base.extras.grid.cols"`), so a reader never silently drops or
//! defaults a field it does not understand — the no-silent-failures rule,
//! applied to the artifact format.

use crate::ast::{Breakpoint, Media, PseudoElement, State, Structural};
use crate::color::Color;
use crate::cond::{Atom, Cond, Rule, Test};
use crate::plan::*;
use std::fmt;

/// The snapshot format tag (the `format` header field).
pub const PLAN_SNAPSHOT_FORMAT: &str = "aeolus-plan";
/// The current snapshot schema version. Readers reject other versions with
/// [`SnapshotError::UnsupportedVersion`] — new IR fields bump this.
pub const PLAN_SNAPSHOT_VERSION: u32 = 2;

/// Why a snapshot failed to parse. `Display` renders a precise,
/// path-qualified message (the no-silent-failures rule for artifacts).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapshotError {
    /// The document is not a JSON object carrying the `format`/`version`
    /// header (or the header fields have the wrong types).
    InvalidHeader(&'static str),
    /// The `format` tag is not [`PLAN_SNAPSHOT_FORMAT`]. Carries what was
    /// found.
    WrongFormat(String),
    /// A future (or corrupt) schema version. Carries the number found.
    UnsupportedVersion(u32),
    /// A field this schema version does not define. Strict by design:
    /// silently ignoring fields would corrupt plans across versions.
    UnknownField {
        /// Dotted path to the offending field.
        path: String,
    },
    /// A value with the wrong type/range for its field.
    InvalidValue {
        /// Dotted path to the offending field.
        path: String,
        /// What was wrong and what the field requires.
        msg: String,
    },
    /// The document ends mid-value (JSON syntax or truncation).
    Truncated {
        /// Dotted path where the input ran out.
        path: String,
    },
    /// Content after the complete top-level document.
    Trailing,
    /// Nesting exceeded the parser's depth limit (hostile input guard;
    /// the schema itself nests ~8 levels).
    DepthLimit,
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SnapshotError::InvalidHeader(why) => write!(f, "invalid snapshot header: {why}"),
            SnapshotError::WrongFormat(found) => {
                write!(
                    f,
                    "snapshot format is {found:?}, expected {PLAN_SNAPSHOT_FORMAT:?}"
                )
            }
            SnapshotError::UnsupportedVersion(v) => write!(
                f,
                "snapshot schema version {v} is not supported (this build reads version {PLAN_SNAPSHOT_VERSION})"
            ),
            SnapshotError::UnknownField { path } => {
                write!(
                    f,
                    "unknown field `{path}` — snapshots are read strictly (schema version {PLAN_SNAPSHOT_VERSION})"
                )
            }
            SnapshotError::InvalidValue { path, msg } => {
                write!(f, "invalid value at `{path}`: {msg}")
            }
            SnapshotError::Truncated { path } => {
                write!(f, "truncated snapshot at `{path}`")
            }
            SnapshotError::Trailing => write!(f, "content after the snapshot document"),
            SnapshotError::DepthLimit => {
                write!(f, "snapshot nesting exceeds the depth limit")
            }
        }
    }
}

impl std::error::Error for SnapshotError {}

// ---------------------------------------------------------------------------
// Minimal JSON tree + parser (zero dependencies by contract)
// ---------------------------------------------------------------------------

/// A parsed JSON value. Objects keep their pair order so duplicate-key and
/// unknown-field detection are exact.
#[derive(Clone, Debug, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

/// Parser recursion cap (the schema nests ~8 levels; 64 is generous).
const JSON_MAX_DEPTH: usize = 64;

fn parse_json(input: &str) -> Result<Json, SnapshotError> {
    let bytes = input.as_bytes();
    let mut pos = 0usize;
    skip_ws(bytes, &mut pos);
    let v = parse_value(bytes, &mut pos, 0, "$")?;
    skip_ws(bytes, &mut pos);
    if pos != bytes.len() {
        return Err(SnapshotError::Trailing);
    }
    Ok(v)
}

fn skip_ws(b: &[u8], pos: &mut usize) {
    while *pos < b.len() && matches!(b[*pos], b' ' | b'\t' | b'\n' | b'\r') {
        *pos += 1;
    }
}

fn parse_value(b: &[u8], pos: &mut usize, depth: usize, path: &str) -> Result<Json, SnapshotError> {
    if depth > JSON_MAX_DEPTH {
        return Err(SnapshotError::DepthLimit);
    }
    skip_ws(b, pos);
    let Some(&c) = b.get(*pos) else {
        return Err(SnapshotError::Truncated {
            path: path.to_string(),
        });
    };
    match c {
        b'{' => parse_obj(b, pos, depth, path),
        b'[' => parse_arr(b, pos, depth, path),
        b'"' => Ok(Json::Str(parse_string(b, pos, path)?)),
        b't' => expect_lit(b, pos, "true", path).map(|_| Json::Bool(true)),
        b'f' => expect_lit(b, pos, "false", path).map(|_| Json::Bool(false)),
        b'n' => expect_lit(b, pos, "null", path).map(|_| Json::Null),
        b'-' | b'0'..=b'9' => parse_num(b, pos, path),
        _ => Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: format!("unexpected character {:?}", c as char),
        }),
    }
}

fn expect_lit(b: &[u8], pos: &mut usize, lit: &str, path: &str) -> Result<(), SnapshotError> {
    if b[*pos..].starts_with(lit.as_bytes()) {
        *pos += lit.len();
        Ok(())
    } else {
        Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: format!("expected `{lit}`"),
        })
    }
}

fn parse_num(b: &[u8], pos: &mut usize, path: &str) -> Result<Json, SnapshotError> {
    let start = *pos;
    if b.get(*pos) == Some(&b'-') {
        *pos += 1;
    }
    // Integer part: 0 | [1-9][0-9]*
    match b.get(*pos) {
        Some(&c @ b'0') => {
            *pos += 1;
            if matches!(b.get(*pos), Some(c) if c.is_ascii_digit()) {
                return Err(SnapshotError::InvalidValue {
                    path: path.to_string(),
                    msg: "leading zero in number".to_string(),
                });
            }
            let _ = c;
        }
        Some(c) if c.is_ascii_digit() => {
            while matches!(b.get(*pos), Some(c) if c.is_ascii_digit()) {
                *pos += 1;
            }
        }
        _ => {
            return Err(SnapshotError::InvalidValue {
                path: path.to_string(),
                msg: "expected a number".to_string(),
            });
        }
    }
    // Fraction
    if b.get(*pos) == Some(&b'.') {
        *pos += 1;
        if !matches!(b.get(*pos), Some(c) if c.is_ascii_digit()) {
            return Err(SnapshotError::Truncated {
                path: path.to_string(),
            });
        }
        while matches!(b.get(*pos), Some(c) if c.is_ascii_digit()) {
            *pos += 1;
        }
    }
    // Exponent
    if matches!(b.get(*pos), Some(&c) if c == b'e' || c == b'E') {
        *pos += 1;
        if matches!(b.get(*pos), Some(&c) if c == b'+' || c == b'-') {
            *pos += 1;
        }
        if !matches!(b.get(*pos), Some(c) if c.is_ascii_digit()) {
            return Err(SnapshotError::Truncated {
                path: path.to_string(),
            });
        }
        while matches!(b.get(*pos), Some(c) if c.is_ascii_digit()) {
            *pos += 1;
        }
    }
    let text = std::str::from_utf8(&b[start..*pos]).map_err(|_| SnapshotError::InvalidValue {
        path: path.to_string(),
        msg: "number is not valid UTF-8".to_string(),
    })?;
    let n: f64 = text.parse().map_err(|_| SnapshotError::InvalidValue {
        path: path.to_string(),
        msg: format!("number {text:?} is out of range"),
    })?;
    Ok(Json::Num(n))
}

fn parse_string(b: &[u8], pos: &mut usize, path: &str) -> Result<String, SnapshotError> {
    // b[*pos] == b'"' (guaranteed by the caller).
    *pos += 1;
    let mut out = String::new();
    loop {
        let Some(&c) = b.get(*pos) else {
            return Err(SnapshotError::Truncated {
                path: path.to_string(),
            });
        };
        match c {
            b'"' => {
                *pos += 1;
                return Ok(out);
            }
            b'\\' => {
                *pos += 1;
                let Some(&esc) = b.get(*pos) else {
                    return Err(SnapshotError::Truncated {
                        path: path.to_string(),
                    });
                };
                *pos += 1;
                match esc {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'/' => out.push('/'),
                    b'b' => out.push('\u{0008}'),
                    b'f' => out.push('\u{000C}'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    b'u' => {
                        let hi = parse_hex4(b, pos, path)?;
                        let ch = if (0xD800..0xDC00).contains(&hi) {
                            // High surrogate: require a following \uXXXX low surrogate.
                            if b.get(*pos) == Some(&b'\\') && b.get(*pos + 1) == Some(&b'u') {
                                *pos += 2;
                                let lo = parse_hex4(b, pos, path)?;
                                if !(0xDC00..0xE000).contains(&lo) {
                                    return Err(SnapshotError::InvalidValue {
                                        path: path.to_string(),
                                        msg: "invalid low surrogate".to_string(),
                                    });
                                }
                                let cp = 0x1_0000 + ((hi - 0xD800) << 10) + (lo - 0xDC00);
                                char::from_u32(cp).ok_or_else(|| SnapshotError::InvalidValue {
                                    path: path.to_string(),
                                    msg: "invalid surrogate pair".to_string(),
                                })?
                            } else {
                                return Err(SnapshotError::InvalidValue {
                                    path: path.to_string(),
                                    msg: "lone high surrogate".to_string(),
                                });
                            }
                        } else if (0xDC00..0xE000).contains(&hi) {
                            return Err(SnapshotError::InvalidValue {
                                path: path.to_string(),
                                msg: "lone low surrogate".to_string(),
                            });
                        } else {
                            char::from_u32(hi).ok_or_else(|| SnapshotError::InvalidValue {
                                path: path.to_string(),
                                msg: "invalid \\u escape".to_string(),
                            })?
                        };
                        out.push(ch);
                    }
                    _ => {
                        return Err(SnapshotError::InvalidValue {
                            path: path.to_string(),
                            msg: format!("unknown escape \\{}", esc as char),
                        });
                    }
                }
            }
            c if c < 0x20 => {
                return Err(SnapshotError::InvalidValue {
                    path: path.to_string(),
                    msg: "raw control character in string".to_string(),
                });
            }
            _ => {
                // Copy one UTF-8 scalar (input is &str — always valid).
                let s = std::str::from_utf8(&b[*pos..]).unwrap_or("");
                let ch = s.chars().next().ok_or_else(|| SnapshotError::Truncated {
                    path: path.to_string(),
                })?;
                out.push(ch);
                *pos += ch.len_utf8();
            }
        }
    }
}

fn parse_hex4(b: &[u8], pos: &mut usize, path: &str) -> Result<u32, SnapshotError> {
    if *pos + 4 > b.len() {
        return Err(SnapshotError::Truncated {
            path: path.to_string(),
        });
    }
    let hex = std::str::from_utf8(&b[*pos..*pos + 4]).map_err(|_| SnapshotError::InvalidValue {
        path: path.to_string(),
        msg: "invalid \\u escape".to_string(),
    })?;
    let v = u32::from_str_radix(hex, 16).map_err(|_| SnapshotError::InvalidValue {
        path: path.to_string(),
        msg: "invalid \\u escape".to_string(),
    })?;
    *pos += 4;
    Ok(v)
}

fn parse_obj(b: &[u8], pos: &mut usize, depth: usize, path: &str) -> Result<Json, SnapshotError> {
    *pos += 1; // '{'
    let mut fields: Vec<(String, Json)> = Vec::new();
    skip_ws(b, pos);
    if b.get(*pos) == Some(&b'}') {
        *pos += 1;
        return Ok(Json::Obj(fields));
    }
    loop {
        skip_ws(b, pos);
        if b.get(*pos) != Some(&b'"') {
            return Err(trunc_or_val(b, *pos, path));
        }
        let key = parse_string(b, pos, path)?;
        skip_ws(b, pos);
        if b.get(*pos) != Some(&b':') {
            return Err(SnapshotError::InvalidValue {
                path: path.to_string(),
                msg: "expected `:`".to_string(),
            });
        }
        *pos += 1;
        let child_path = join_path(path, &key);
        let value = parse_value(b, pos, depth + 1, &child_path)?;
        if fields.iter().any(|(k, _)| *k == key) {
            return Err(SnapshotError::InvalidValue {
                path: child_path,
                msg: "duplicate object key".to_string(),
            });
        }
        fields.push((key, value));
        skip_ws(b, pos);
        match b.get(*pos) {
            Some(&b',') => {
                *pos += 1;
            }
            Some(&b'}') => {
                *pos += 1;
                return Ok(Json::Obj(fields));
            }
            _ => return Err(trunc_or_val(b, *pos, path)),
        }
    }
}

fn parse_arr(b: &[u8], pos: &mut usize, depth: usize, path: &str) -> Result<Json, SnapshotError> {
    *pos += 1; // '['
    let mut items = Vec::new();
    skip_ws(b, pos);
    if b.get(*pos) == Some(&b']') {
        *pos += 1;
        return Ok(Json::Arr(items));
    }
    loop {
        let child_path = format!("{path}[{}]", items.len());
        let value = parse_value(b, pos, depth + 1, &child_path)?;
        items.push(value);
        skip_ws(b, pos);
        match b.get(*pos) {
            Some(&b',') => {
                *pos += 1;
            }
            Some(&b']') => {
                *pos += 1;
                return Ok(Json::Arr(items));
            }
            _ => return Err(trunc_or_val(b, *pos, path)),
        }
    }
}

fn trunc_or_val(b: &[u8], pos: usize, path: &str) -> SnapshotError {
    if pos >= b.len() {
        SnapshotError::Truncated {
            path: path.to_string(),
        }
    } else {
        SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: format!("unexpected character {:?}", b[pos] as char),
        }
    }
}

fn join_path(parent: &str, key: &str) -> String {
    format!("{parent}.{key}")
}

// ---------------------------------------------------------------------------
// Writer
// ---------------------------------------------------------------------------

/// The canonical writer: compact, fixed schema order, shortest round-trip
/// floats.
struct W {
    out: String,
}

impl W {
    fn new() -> Self {
        W {
            out: String::with_capacity(4096),
        }
    }
    fn raw(&mut self, s: &str) {
        self.out.push_str(s);
    }
    fn key(&mut self, k: &str) {
        self.s(k);
        self.out.push(':');
    }
    fn sep(&mut self) {
        self.out.push(',');
    }
    fn str_val(&mut self, v: &str) {
        self.s(v);
    }
    fn s(&mut self, v: &str) {
        self.out.push('"');
        for ch in v.chars() {
            match ch {
                '"' => self.out.push_str("\\\""),
                '\\' => self.out.push_str("\\\\"),
                '\n' => self.out.push_str("\\n"),
                '\r' => self.out.push_str("\\r"),
                '\t' => self.out.push_str("\\t"),
                '\u{0008}' => self.out.push_str("\\b"),
                '\u{000C}' => self.out.push_str("\\f"),
                c if (c as u32) < 0x20 => {
                    self.out.push_str(&format!("\\u{:04x}", c as u32));
                }
                c => self.out.push(c),
            }
        }
        self.out.push('"');
    }
    /// Shortest round-trip float; non-finite values become their string
    /// encodings (the reader maps them back — the writer is total).
    fn f32(&mut self, v: f32) {
        if v.is_nan() {
            self.s("NaN");
        } else if v.is_infinite() {
            self.s(if v > 0.0 { "inf" } else { "-inf" });
        } else {
            self.out.push_str(&format!("{}", v));
        }
    }
    fn opt_f32(&mut self, v: Option<f32>) {
        match v {
            Some(v) => self.f32(v),
            None => self.raw("null"),
        }
    }
    fn boolean(&mut self, v: bool) {
        self.raw(if v { "true" } else { "false" });
    }
    fn color(&mut self, c: &Color) {
        self.raw("[");
        self.f32(c.l);
        self.sep();
        self.f32(c.a);
        self.sep();
        self.f32(c.b);
        self.sep();
        self.f32(c.alpha);
        self.raw("]");
    }
}

// ---------------------------------------------------------------------------
// Plan writer (schema order is the compatibility contract)
// ---------------------------------------------------------------------------

impl StylePlan {
    /// Serialize this plan into the canonical snapshot JSON.
    ///
    /// Byte-stable for equal plans (pinned by `tests/snapshot_roundtrip.rs`):
    /// fixed field order, compact separators, shortest round-trip floats.
    pub fn to_snapshot_json(&self) -> String {
        let mut w = W::new();
        w.raw("{");
        w.key("format");
        w.str_val(PLAN_SNAPSHOT_FORMAT);
        w.sep();
        w.key("version");
        w.raw(&PLAN_SNAPSHOT_VERSION.to_string());
        w.sep();
        w.key("base");
        write_style(&mut w, &self.base);
        w.sep();
        w.key("layers");
        w.raw("[");
        for (i, layer) in self.layers.iter().enumerate() {
            if i > 0 {
                w.sep();
            }
            w.raw("{");
            w.key("condition");
            w.str_val(variant_condition_name(layer.condition));
            w.sep();
            w.key("style");
            write_style(&mut w, &layer.style);
            w.raw("}");
        }
        w.raw("]");
        w.sep();
        w.key("responsive");
        write_bucket_map(&mut w, &self.responsive);
        w.sep();
        w.key("container");
        write_bucket_map(&mut w, &self.container);
        w.sep();
        w.key("dark");
        match &self.dark {
            Some(style) => write_style(&mut w, style),
            None => w.raw("null"),
        }
        w.sep();
        w.key("group");
        write_labeled(&mut w, &self.group);
        w.sep();
        w.key("peer");
        write_labeled(&mut w, &self.peer);
        w.sep();
        w.key("rules");
        w.raw("[");
        for (i, rule) in self.rules.iter().enumerate() {
            if i > 0 {
                w.sep();
            }
            w.raw("{");
            w.key("when");
            write_cond(&mut w, &rule.when);
            w.sep();
            w.key("style");
            write_style(&mut w, &rule.style);
            w.raw("}");
        }
        w.raw("]");
        w.sep();
        w.key("breakpoints");
        w.raw("[");
        for (i, bp) in self.breakpoints.iter().enumerate() {
            if i > 0 {
                w.sep();
            }
            w.f32(*bp);
        }
        w.raw("]");
        w.raw("}");
        w.out
    }
}

fn write_bucket_map(w: &mut W, map: &std::collections::BTreeMap<Breakpoint, ResolvedStyle>) {
    // BTreeMap iteration is Ord order (Sm < Md < Lg < Xl < Xxl) — canonical.
    w.raw("{");
    let mut first = true;
    for (bp, style) in map {
        if !first {
            w.sep();
        }
        first = false;
        w.key(breakpoint_name(*bp));
        write_style(w, style);
    }
    w.raw("}");
}

fn write_labeled(w: &mut W, entries: &[(Box<str>, ResolvedStyle)]) {
    w.raw("[");
    for (i, (id, style)) in entries.iter().enumerate() {
        if i > 0 {
            w.sep();
        }
        w.raw("{");
        w.key("id");
        w.str_val(id);
        w.sep();
        w.key("style");
        write_style(w, style);
        w.raw("}");
    }
    w.raw("]");
}

fn write_cond(w: &mut W, cond: &Cond) {
    w.raw("[");
    for (i, atom) in cond.0.iter().enumerate() {
        if i > 0 {
            w.sep();
        }
        write_atom(w, atom);
    }
    w.raw("]");
}

fn write_atom(w: &mut W, atom: &Atom) {
    match atom {
        Atom::On(t) => {
            w.raw("{");
            w.key("on");
            write_test(w, t);
            w.raw("}");
        }
        Atom::Not(t) => {
            w.raw("{");
            w.key("not");
            write_test(w, t);
            w.raw("}");
        }
        Atom::Group(t, label) => {
            w.raw("{");
            w.key("group");
            write_labeled_test(w, t, label.as_deref());
            w.raw("}");
        }
        Atom::Peer(t, label) => {
            w.raw("{");
            w.key("peer");
            write_labeled_test(w, t, label.as_deref());
            w.raw("}");
        }
        Atom::Has(test) => {
            w.raw("{");
            w.key("has");
            write_test(w, test);
            w.raw("}");
        }
        Atom::Media(media) => {
            w.raw("{");
            w.key("media");
            w.str_val(media_name(*media));
            w.raw("}");
        }
        Atom::Viewport { min, max } => {
            w.raw("{");
            w.key("viewport");
            write_min_max(w, *min, *max, None);
            w.raw("}");
        }
        Atom::Container { min, max, name } => {
            w.raw("{");
            w.key("container");
            write_min_max(w, *min, *max, name.as_deref());
            w.raw("}");
        }
    }
}

fn write_labeled_test(w: &mut W, t: &Test, label: Option<&str>) {
    w.raw("{");
    w.key("test");
    write_test(w, t);
    w.sep();
    w.key("label");
    match label {
        Some(l) => w.str_val(l),
        None => w.raw("null"),
    }
    w.raw("}");
}

fn write_min_max(w: &mut W, min: Option<f32>, max: Option<f32>, name: Option<&str>) {
    w.raw("{");
    w.key("min");
    w.opt_f32(min);
    w.sep();
    w.key("max");
    w.opt_f32(max);
    if let Some(name) = name {
        w.sep();
        w.key("name");
        w.str_val(name);
    }
    w.raw("}");
}

fn write_test(w: &mut W, t: &Test) {
    match t {
        Test::State(s) => {
            w.raw("{");
            w.key("state");
            w.str_val(state_name(*s));
            w.raw("}");
        }
        Test::Data(name, value) | Test::Aria(name, value) => {
            w.raw("{");
            match t {
                Test::Data(..) => w.key("data"),
                _ => w.key("aria"),
            }
            w.raw("{");
            w.key("name");
            w.str_val(name);
            w.sep();
            w.key("value");
            match value {
                Some(v) => w.str_val(v),
                None => w.raw("null"),
            }
            w.raw("}");
            w.raw("}");
        }
        Test::Structural(s) => {
            w.raw("{");
            w.key("structural");
            match s {
                Structural::Nth(n) => {
                    w.raw("{");
                    w.key("nth");
                    w.raw(&n.to_string());
                    w.raw("}");
                }
                Structural::NthLast(n) => {
                    w.raw("{");
                    w.key("nth_last");
                    w.raw(&n.to_string());
                    w.raw("}");
                }
                other => w.str_val(structural_name(other)),
            }
            w.raw("}");
        }
        Test::Tag(tag) => {
            w.raw("{");
            w.key("tag");
            w.str_val(tag);
            w.raw("}");
        }
        Test::Pseudo(pe) => {
            w.raw("{");
            w.key("pseudo");
            w.str_val(pe.name());
            w.raw("}");
        }
    }
}

/// The v0.7 §4 arbitrary grid templates. `None` writes `null` (the field
/// is always present — fixed schema order); a template is an array of
/// single-key components (`track` / `repeat`).
fn write_tracks(w: &mut W, tracks: Option<&[TrackComponent]>) {
    match tracks {
        None => w.raw("null"),
        Some(ts) => {
            w.raw("[");
            for (i, t) in ts.iter().enumerate() {
                if i > 0 {
                    w.raw(",");
                }
                write_track_component(w, t);
            }
            w.raw("]");
        }
    }
}

fn write_track_component(w: &mut W, t: &TrackComponent) {
    w.raw("{");
    match t {
        TrackComponent::Track(tr) => {
            w.key("track");
            write_grid_track(w, tr);
        }
        TrackComponent::Repeat { count, track } => {
            w.key("repeat");
            w.raw("{");
            w.key("count");
            w.raw(&count.to_string());
            w.sep();
            w.key("track");
            write_grid_track(w, track);
            w.raw("}");
        }
    }
    w.raw("}");
}

fn write_grid_track(w: &mut W, t: &GridTrack) {
    w.raw("{");
    w.key("min");
    write_track_sizing(w, t.min);
    w.sep();
    w.key("max");
    write_track_sizing(w, t.max);
    w.raw("}");
}

fn write_track_sizing(w: &mut W, s: TrackSizing) {
    match s {
        TrackSizing::Px(v) => {
            w.raw("{");
            w.key("px");
            w.f32(v);
            w.raw("}");
        }
        TrackSizing::Percent(v) => {
            w.raw("{");
            w.key("percent");
            w.f32(v);
            w.raw("}");
        }
        TrackSizing::Fr(v) => {
            w.raw("{");
            w.key("fr");
            w.f32(v);
            w.raw("}");
        }
        TrackSizing::Auto => w.str_val("auto"),
        TrackSizing::MinContent => w.str_val("min_content"),
        TrackSizing::MaxContent => w.str_val("max_content"),
    }
}

fn write_style(w: &mut W, s: &ResolvedStyle) {
    w.raw("{");
    w.key("background");
    write_opt_color_newtype(w, &s.background, |b| &b.0);
    w.sep();
    w.key("background_gradient");
    match &s.background_gradient {
        Some(g) => write_gradient(w, g),
        None => w.raw("null"),
    }
    w.sep();
    w.key("content_color");
    write_opt_color_newtype(w, &s.content_color, |c| &c.0);
    w.sep();
    w.key("padding");
    write_opt_edges(w, &s.padding, |p| (&p.top, &p.right, &p.bottom, &p.left));
    w.sep();
    w.key("border_color");
    write_opt_color_newtype(w, &s.border_color, |b| &b.0);
    w.sep();
    w.key("border_width");
    write_opt_edges(w, &s.border_width, |b| {
        (&b.top, &b.right, &b.bottom, &b.left)
    });
    w.sep();
    w.key("corner_radius");
    match &s.corner_radius {
        Some(r) => {
            w.raw("{");
            write_corner_fields(
                w,
                (&r.top_left, &r.top_right, &r.bottom_right, &r.bottom_left),
            );
            w.raw("}");
        }
        None => w.raw("null"),
    }
    w.sep();
    w.key("dimensions");
    write_opt_dims(w, &s.dimensions);
    w.sep();
    w.key("min_dimensions");
    write_opt_dims(w, &s.min_dimensions);
    w.sep();
    w.key("max_dimensions");
    write_opt_dims(w, &s.max_dimensions);
    w.sep();
    w.key("gap");
    match &s.gap {
        Some(Gap(g)) => w.f32(*g),
        None => w.raw("null"),
    }
    w.sep();
    w.key("shadow");
    write_opt_shadow(w, &s.shadow);
    w.sep();
    w.key("margin");
    write_opt_edges(w, &s.margin, |m| (&m.top, &m.right, &m.bottom, &m.left));
    w.sep();
    w.key("layout");
    match &s.layout {
        Some(l) => {
            w.raw("{");
            w.key("display");
            write_enum_opt(w, l.display.as_ref(), display_name);
            w.sep();
            w.key("flex_direction");
            write_enum_opt(w, l.flex_direction.as_ref(), flex_direction_name);
            w.sep();
            w.key("align_items");
            write_enum_opt(w, l.align_items.as_ref(), align_items_name);
            w.sep();
            w.key("justify_content");
            write_enum_opt(w, l.justify_content.as_ref(), justify_content_name);
            w.sep();
            w.key("justify_items");
            write_enum_opt(w, l.justify_items.as_ref(), align_items_name);
            w.sep();
            w.key("flex_wrap");
            match l.flex_wrap {
                Some(b) => w.boolean(b),
                None => w.raw("null"),
            }
            w.sep();
            w.key("flex_grow");
            w.opt_f32(l.flex_grow);
            w.sep();
            w.key("flex_shrink");
            w.opt_f32(l.flex_shrink);
            w.sep();
            w.key("align_self");
            write_enum_opt(w, l.align_self.as_ref(), align_items_name);
            w.raw("}");
        }
        None => w.raw("null"),
    }
    w.sep();
    w.key("opacity");
    match &s.opacity {
        Some(Opacity(o)) => w.f32(*o),
        None => w.raw("null"),
    }
    w.sep();
    w.key("ring");
    match &s.ring {
        Some(r) => {
            w.raw("{");
            w.key("width");
            w.f32(r.width);
            w.sep();
            w.key("color");
            write_opt_color(w, r.color);
            w.sep();
            w.key("offset_width");
            w.f32(r.offset_width);
            w.sep();
            w.key("offset_color");
            write_opt_color(w, r.offset_color);
            w.sep();
            w.key("inset");
            w.boolean(r.inset);
            w.raw("}");
        }
        None => w.raw("null"),
    }
    w.sep();
    w.key("outline");
    match &s.outline {
        Some(o) => {
            w.raw("{");
            w.key("width");
            w.f32(o.width);
            w.sep();
            w.key("color");
            write_opt_color(w, o.color);
            w.sep();
            w.key("none");
            w.boolean(o.none);
            w.sep();
            w.key("offset");
            w.f32(o.offset);
            w.raw("}");
        }
        None => w.raw("null"),
    }
    w.sep();
    w.key("cursor");
    write_enum_opt(w, s.cursor.as_ref(), cursor_name);
    w.sep();
    w.key("pointer_events");
    write_enum_opt(w, s.pointer_events.as_ref(), pointer_events_name);
    w.sep();
    w.key("text");
    w.raw("[");
    for (i, (prop, value)) in s.text.iter().enumerate() {
        if i > 0 {
            w.sep();
        }
        w.raw("[");
        w.str_val(text_property_name(*prop));
        w.sep();
        write_text_value(w, value);
        w.raw("]");
    }
    w.raw("]");
    w.sep();
    w.key("extras");
    write_extras(w, &s.extras);
    w.raw("}");
}

fn write_extras(w: &mut W, e: &Extras) {
    w.raw("{");
    w.key("position");
    write_enum_opt(w, e.position.as_ref(), position_name);
    w.sep();
    w.key("inset");
    match &e.inset {
        Some(i) => {
            w.raw("{");
            w.key("top");
            write_opt_sized(w, i.top);
            w.sep();
            w.key("right");
            write_opt_sized(w, i.right);
            w.sep();
            w.key("bottom");
            write_opt_sized(w, i.bottom);
            w.sep();
            w.key("left");
            write_opt_sized(w, i.left);
            w.raw("}");
        }
        None => w.raw("null"),
    }
    w.sep();
    w.key("z_index");
    match e.z_index {
        Some(z) => w.raw(&z.to_string()),
        None => w.raw("null"),
    }
    w.sep();
    w.key("overflow");
    match &e.overflow {
        Some(o) => {
            w.raw("{");
            w.key("x");
            write_enum_opt(w, o.x.as_ref(), overflow_kind_name);
            w.sep();
            w.key("y");
            write_enum_opt(w, o.y.as_ref(), overflow_kind_name);
            w.raw("}");
        }
        None => w.raw("null"),
    }
    w.sep();
    w.key("visibility");
    write_enum_opt(w, e.visibility.as_ref(), visibility_name);
    w.sep();
    w.key("aspect_ratio");
    w.opt_f32(e.aspect_ratio);
    w.sep();
    w.key("grid");
    match &e.grid {
        Some(g) => {
            w.raw("{");
            w.key("cols");
            write_opt_u16(w, g.cols);
            w.sep();
            w.key("rows");
            write_opt_u16(w, g.rows);
            w.sep();
            w.key("col_span");
            write_opt_u16(w, g.col_span);
            w.sep();
            w.key("row_span");
            write_opt_u16(w, g.row_span);
            w.sep();
            w.key("col_start");
            write_opt_i16(w, g.col_start);
            w.sep();
            w.key("row_start");
            write_opt_i16(w, g.row_start);
            w.sep();
            w.key("template_cols");
            write_tracks(w, g.template_cols.as_deref());
            w.sep();
            w.key("template_rows");
            write_tracks(w, g.template_rows.as_deref());
            w.sep();
            w.key("auto_rows");
            write_tracks(w, g.auto_rows.as_deref());
            w.sep();
            w.key("auto_cols");
            write_tracks(w, g.auto_cols.as_deref());
            w.raw("}");
        }
        None => w.raw("null"),
    }
    w.sep();
    w.key("transform");
    match &e.transform {
        Some(t) => {
            w.raw("{");
            w.key("scale_x");
            w.opt_f32(t.scale_x);
            w.sep();
            w.key("scale_y");
            w.opt_f32(t.scale_y);
            w.sep();
            w.key("rotate_deg");
            w.opt_f32(t.rotate_deg);
            w.sep();
            w.key("translate_x");
            write_opt_sized(w, t.translate_x);
            w.sep();
            w.key("translate_y");
            write_opt_sized(w, t.translate_y);
            w.sep();
            w.key("skew_x");
            w.opt_f32(t.skew_x);
            w.sep();
            w.key("skew_y");
            w.opt_f32(t.skew_y);
            w.raw("}");
        }
        None => w.raw("null"),
    }
    w.sep();
    w.key("transition");
    match &e.transition {
        Some(t) => {
            w.raw("{");
            w.key("set");
            w.str_val(transition_set_name(t.set));
            w.sep();
            w.key("duration_ms");
            w.f32(t.duration_ms);
            w.sep();
            w.key("ease");
            write_enum_opt(w, t.ease.as_ref(), ease_name);
            w.sep();
            w.key("delay_ms");
            w.f32(t.delay_ms);
            w.raw("}");
        }
        None => w.raw("null"),
    }
    w.sep();
    w.key("animation");
    match &e.animation {
        Some(a) => {
            w.raw("{");
            w.key("name");
            w.str_val(&a.name);
            w.sep();
            w.key("duration_ms");
            w.opt_f32(a.duration_ms);
            w.sep();
            w.key("ease");
            write_enum_opt(w, a.ease.as_ref(), ease_name);
            w.sep();
            w.key("infinite");
            w.boolean(a.infinite);
            w.raw("}");
        }
        None => w.raw("null"),
    }
    w.sep();
    w.key("border_style");
    write_enum_opt(w, e.border_style.as_ref(), border_style_name);
    w.sep();
    w.key("flex_basis");
    match &e.flex_basis {
        Some(v) => write_sized(w, v),
        None => w.raw("null"),
    }
    w.sep();
    w.key("order");
    match e.order {
        Some(o) => w.raw(&o.to_string()),
        None => w.raw("null"),
    }
    w.sep();
    w.key("align_content");
    write_enum_opt(w, e.align_content.as_ref(), content_align_name);
    w.sep();
    w.key("margin_auto");
    match &e.margin_auto {
        Some(m) => {
            w.raw("{");
            w.key("left");
            w.boolean(m.left);
            w.sep();
            w.key("right");
            w.boolean(m.right);
            w.sep();
            w.key("top");
            w.boolean(m.top);
            w.sep();
            w.key("bottom");
            w.boolean(m.bottom);
            w.raw("}");
        }
        None => w.raw("null"),
    }
    w.sep();
    w.key("gap_axes");
    match &e.gap_axes {
        Some(g) => {
            w.raw("{");
            w.key("col");
            w.opt_f32(g.col);
            w.sep();
            w.key("row");
            w.opt_f32(g.row);
            w.raw("}");
        }
        None => w.raw("null"),
    }
    w.sep();
    w.key("container_type");
    match &e.container_type {
        Some(c) => {
            w.raw("{");
            w.key("name");
            match &c.name {
                Some(n) => w.str_val(n),
                None => w.raw("null"),
            }
            w.raw("}");
        }
        None => w.raw("null"),
    }
    w.sep();
    w.key("shadow_rest");
    write_shadow_list(w, &e.shadow_rest);
    w.sep();
    w.key("inset_shadows");
    write_shadow_list(w, &e.inset_shadows);
    w.raw("}");
}

fn write_shadow_list(w: &mut W, shadows: &[BoxShadow]) {
    w.raw("[");
    for (i, sh) in shadows.iter().enumerate() {
        if i > 0 {
            w.sep();
        }
        write_shadow(w, sh);
    }
    w.raw("]");
}

fn write_shadow(w: &mut W, sh: &BoxShadow) {
    w.raw("{");
    w.key("offset_x");
    w.f32(sh.offset_x);
    w.sep();
    w.key("offset_y");
    w.f32(sh.offset_y);
    w.sep();
    w.key("blur");
    w.f32(sh.blur);
    w.sep();
    w.key("spread");
    w.f32(sh.spread);
    w.sep();
    w.key("color");
    w.color(&sh.color);
    w.sep();
    w.key("inset");
    w.boolean(sh.inset);
    w.raw("}");
}

fn write_opt_shadow(w: &mut W, sh: &Option<BoxShadow>) {
    match sh {
        Some(sh) => write_shadow(w, sh),
        None => w.raw("null"),
    }
}

fn write_opt_dims(w: &mut W, d: &Option<Dimensions>) {
    match d {
        Some(d) => {
            w.raw("{");
            w.key("width");
            write_opt_sized(w, d.width);
            w.sep();
            w.key("height");
            write_opt_sized(w, d.height);
            w.raw("}");
        }
        None => w.raw("null"),
    }
}

fn write_gradient(w: &mut W, g: &Gradient) {
    match g {
        Gradient::None => w.str_val("none"),
        Gradient::Linear { angle_deg, stops } => {
            w.raw("{");
            w.key("shape");
            w.str_val("linear");
            w.sep();
            w.key("angle_deg");
            w.f32(*angle_deg);
            w.sep();
            w.key("stops");
            write_stops(w, stops);
            w.raw("}");
        }
        Gradient::Radial { stops } => {
            w.raw("{");
            w.key("shape");
            w.str_val("radial");
            w.sep();
            w.key("stops");
            write_stops(w, stops);
            w.raw("}");
        }
        Gradient::Conic {
            from_angle_deg,
            stops,
        } => {
            w.raw("{");
            w.key("shape");
            w.str_val("conic");
            w.sep();
            w.key("from_angle_deg");
            w.f32(*from_angle_deg);
            w.sep();
            w.key("stops");
            write_stops(w, stops);
            w.raw("}");
        }
    }
}

fn write_stops(w: &mut W, stops: &GradientStops) {
    w.raw("{");
    w.key("from");
    write_stop(w, &stops.from);
    w.sep();
    w.key("via");
    match &stops.via {
        Some(v) => write_stop(w, v),
        None => w.raw("null"),
    }
    w.sep();
    w.key("to");
    write_stop(w, &stops.to);
    w.raw("}");
}

fn write_stop(w: &mut W, stop: &GradientStop) {
    w.raw("{");
    w.key("color");
    w.color(&stop.color);
    w.sep();
    w.key("pos_pct");
    w.opt_f32(stop.pos_pct);
    w.raw("}");
}

fn write_sized(w: &mut W, v: &SizedValue) {
    match v {
        SizedValue::Px(px) => {
            w.raw("{");
            w.key("px");
            w.f32(*px);
            w.raw("}");
        }
        SizedValue::Scale(n) => {
            w.raw("{");
            w.key("scale");
            w.f32(*n);
            w.raw("}");
        }
        SizedValue::Fraction(f) => {
            w.raw("{");
            w.key("fraction");
            w.f32(*f);
            w.raw("}");
        }
        SizedValue::Full => w.str_val("full"),
        SizedValue::FitContent => w.str_val("fit"),
        SizedValue::Auto => w.str_val("auto"),
        SizedValue::Viewport => w.str_val("viewport"),
    }
}

fn write_opt_sized(w: &mut W, v: Option<SizedValue>) {
    match v {
        Some(v) => write_sized(w, &v),
        None => w.raw("null"),
    }
}

fn write_text_value(w: &mut W, v: &TextValue) {
    match v {
        TextValue::Px(px) => {
            w.raw("{");
            w.key("px");
            w.f32(*px);
            w.raw("}");
        }
        TextValue::Weight(weight) => {
            w.raw("{");
            w.key("weight");
            w.raw(&weight.to_string());
            w.raw("}");
        }
        TextValue::Spacing(s) => {
            w.raw("{");
            w.key("spacing");
            w.f32(*s);
            w.raw("}");
        }
        TextValue::TrackingEm(em) => {
            w.raw("{");
            w.key("tracking_em");
            w.f32(*em);
            w.raw("}");
        }
        TextValue::LineHeightRatio(ratio) => {
            w.raw("{");
            w.key("line_height_ratio");
            w.f32(*ratio);
            w.raw("}");
        }
        TextValue::Keyword(kw) => {
            w.raw("{");
            w.key("keyword");
            w.str_val(kw);
            w.raw("}");
        }
        TextValue::Count(n) => {
            w.raw("{");
            w.key("count");
            w.raw(&n.to_string());
            w.raw("}");
        }
        TextValue::Color(c) => {
            w.raw("{");
            w.key("color");
            w.color(c);
            w.raw("}");
        }
    }
}

fn write_opt_color(w: &mut W, c: Option<Color>) {
    match c {
        Some(c) => w.color(&c),
        None => w.raw("null"),
    }
}

fn write_opt_color_newtype<T>(w: &mut W, slot: &Option<T>, get: impl Fn(&T) -> &Color) {
    match slot {
        Some(v) => w.color(get(v)),
        None => w.raw("null"),
    }
}

fn write_opt_edges<T>(w: &mut W, slot: &Option<T>, edges: impl Fn(&T) -> (&f32, &f32, &f32, &f32)) {
    match slot {
        Some(v) => {
            let (top, right, bottom, left) = edges(v);
            w.raw("{");
            w.key("top");
            w.f32(*top);
            w.sep();
            w.key("right");
            w.f32(*right);
            w.sep();
            w.key("bottom");
            w.f32(*bottom);
            w.sep();
            w.key("left");
            w.f32(*left);
            w.raw("}");
        }
        None => w.raw("null"),
    }
}

fn write_corner_fields(w: &mut W, (tl, tr, br, bl): (&f32, &f32, &f32, &f32)) {
    w.key("top_left");
    w.f32(*tl);
    w.sep();
    w.key("top_right");
    w.f32(*tr);
    w.sep();
    w.key("bottom_right");
    w.f32(*br);
    w.sep();
    w.key("bottom_left");
    w.f32(*bl);
}

fn write_enum_opt<T: Copy>(w: &mut W, value: Option<T>, name: impl Fn(T) -> &'static str) {
    match value {
        Some(v) => w.str_val(name(v)),
        None => w.raw("null"),
    }
}

fn write_opt_u16(w: &mut W, v: Option<u16>) {
    match v {
        Some(v) => w.raw(&v.to_string()),
        None => w.raw("null"),
    }
}

fn write_opt_i16(w: &mut W, v: Option<i16>) {
    match v {
        Some(v) => w.raw(&v.to_string()),
        None => w.raw("null"),
    }
}

// ---------------------------------------------------------------------------
// Reader (strict: unknown fields are errors, never defaults)
// ---------------------------------------------------------------------------

impl StylePlan {
    /// Parse a snapshot produced by [`StylePlan::to_snapshot_json`] (or any
    /// byte-equal document) back into an exactly equal plan.
    ///
    /// Strict: unknown fields, unknown enum names, duplicate keys, wrong
    /// versions and trailing content are all
    /// [`SnapshotError`]s — never defaults.
    pub fn from_snapshot_json(json: &str) -> Result<StylePlan, SnapshotError> {
        let doc = parse_json(json)?;
        let Json::Obj(fields) = &doc else {
            return Err(SnapshotError::InvalidHeader(
                "the document is not a JSON object",
            ));
        };
        // Header first: format + version gate everything else. Unknown keys
        // are left for the body loop, which owns the field whitelist.
        let mut format_seen = false;
        let mut version_seen = false;
        let mut format = String::new();
        let mut version = 0u64;
        for (key, value) in fields {
            match key.as_str() {
                "format" => {
                    format_seen = true;
                    format = read_string(value, "$.format")?;
                }
                "version" => {
                    version_seen = true;
                    version = read_u64(value, "$.version")?;
                }
                _ => {}
            }
        }
        if !format_seen || !version_seen {
            return Err(SnapshotError::InvalidHeader(if format_seen {
                "missing `version`"
            } else {
                "missing `format`"
            }));
        }
        if format != PLAN_SNAPSHOT_FORMAT {
            return Err(SnapshotError::WrongFormat(format));
        }
        if version != PLAN_SNAPSHOT_VERSION as u64 {
            return Err(SnapshotError::UnsupportedVersion(
                u32::try_from(version).unwrap_or(u32::MAX),
            ));
        }
        let mut plan = StylePlan::default();
        let mut seen = [false; 8];
        for (key, value) in fields {
            match key.as_str() {
                "format" | "version" => {}
                "base" => {
                    seen[0] = true;
                    plan.base = read_style(value, "$.base")?;
                }
                "layers" => {
                    seen[1] = true;
                    plan.layers = read_arr(value, "$.layers", read_layer)?;
                }
                "responsive" => {
                    seen[2] = true;
                    plan.responsive = read_bucket_map(value, "$.responsive")?;
                }
                "container" => {
                    seen[3] = true;
                    plan.container = read_bucket_map(value, "$.container")?;
                }
                "dark" => {
                    seen[4] = true;
                    plan.dark = match value {
                        Json::Null => None,
                        v => Some(read_style(v, "$.dark")?),
                    };
                }
                "group" => {
                    seen[5] = true;
                    plan.group = read_labeled(value, "$.group")?;
                }
                "peer" => {
                    seen[6] = true;
                    plan.peer = read_labeled(value, "$.peer")?;
                }
                "rules" => {
                    seen[7] = true;
                    plan.rules = read_arr(value, "$.rules", read_rule)?;
                }
                "breakpoints" => {
                    let arr = read_arr(value, "$.breakpoints", read_f32)?;
                    let n = match value {
                        Json::Arr(items) => items.len(),
                        _ => 0,
                    };
                    let bps: [f32; 5] =
                        arr.try_into().map_err(|_| SnapshotError::InvalidValue {
                            path: "$.breakpoints".to_string(),
                            msg: format!("expected 5 breakpoint values, found {n}"),
                        })?;
                    plan.breakpoints = bps;
                }
                _ => {
                    return Err(SnapshotError::UnknownField {
                        path: join_path("$", key),
                    });
                }
            }
        }
        // Every schema field is required (the writer always emits them).
        if seen.iter().any(|s| !s) {
            return Err(SnapshotError::InvalidHeader(
                "missing a required plan field (base/layers/responsive/container/dark/group/peer/rules/breakpoints)",
            ));
        }
        Ok(plan)
    }
}

fn read_arr<T>(
    j: &Json,
    path: &str,
    mut item: impl FnMut(&Json, &str) -> Result<T, SnapshotError>,
) -> Result<Vec<T>, SnapshotError> {
    let Json::Arr(items) = j else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected an array".to_string(),
        });
    };
    let mut out = Vec::with_capacity(items.len());
    for (i, it) in items.iter().enumerate() {
        out.push(item(it, &format!("{path}[{i}]"))?);
    }
    Ok(out)
}

fn read_bucket_map(
    j: &Json,
    path: &str,
) -> Result<std::collections::BTreeMap<Breakpoint, ResolvedStyle>, SnapshotError> {
    let Json::Obj(fields) = j else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected an object".to_string(),
        });
    };
    let mut map = std::collections::BTreeMap::new();
    for (key, value) in fields {
        let bp = parse_breakpoint(key).ok_or_else(|| SnapshotError::UnknownField {
            path: join_path(path, key),
        })?;
        map.insert(bp, read_style(value, &join_path(path, key))?);
    }
    Ok(map)
}

fn read_labeled(j: &Json, path: &str) -> Result<Vec<(Box<str>, ResolvedStyle)>, SnapshotError> {
    read_arr(j, path, |item, p| {
        let Json::Obj(fields) = item else {
            return Err(SnapshotError::InvalidValue {
                path: p.to_string(),
                msg: "expected an object".to_string(),
            });
        };
        let mut id: Option<Box<str>> = None;
        let mut style: Option<ResolvedStyle> = None;
        for (key, value) in fields {
            match key.as_str() {
                "id" => id = Some(read_string(value, &join_path(p, "id"))?.into()),
                "style" => style = Some(read_style(value, &join_path(p, "style"))?),
                _ => {
                    return Err(SnapshotError::UnknownField {
                        path: join_path(p, key),
                    });
                }
            }
        }
        match (id, style) {
            (Some(id), Some(style)) => Ok((id, style)),
            (None, _) => Err(SnapshotError::InvalidValue {
                path: p.to_string(),
                msg: "missing `id`".to_string(),
            }),
            (_, None) => Err(SnapshotError::InvalidValue {
                path: p.to_string(),
                msg: "missing `style`".to_string(),
            }),
        }
    })
}

fn read_layer(j: &Json, path: &str) -> Result<ConditionalLayer, SnapshotError> {
    let Json::Obj(fields) = j else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected an object".to_string(),
        });
    };
    let mut condition: Option<VariantCondition> = None;
    let mut style: Option<ResolvedStyle> = None;
    for (key, value) in fields {
        match key.as_str() {
            "condition" => {
                condition = Some(
                    parse_variant_condition(&read_string(value, &join_path(path, "condition"))?)
                        .ok_or_else(|| SnapshotError::InvalidValue {
                            path: join_path(path, "condition"),
                            msg: "unknown variant condition".to_string(),
                        })?,
                )
            }
            "style" => style = Some(read_style(value, &join_path(path, "style"))?),
            _ => {
                return Err(SnapshotError::UnknownField {
                    path: join_path(path, key),
                });
            }
        }
    }
    match (condition, style) {
        (Some(condition), Some(style)) => Ok(ConditionalLayer { condition, style }),
        (None, _) => Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "missing `condition`".to_string(),
        }),
        (_, None) => Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "missing `style`".to_string(),
        }),
    }
}

fn read_rule(j: &Json, path: &str) -> Result<Rule, SnapshotError> {
    let Json::Obj(fields) = j else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected an object".to_string(),
        });
    };
    let mut when: Option<Cond> = None;
    let mut style: Option<ResolvedStyle> = None;
    for (key, value) in fields {
        match key.as_str() {
            "when" => when = Some(read_cond(value, &join_path(path, "when"))?),
            "style" => style = Some(read_style(value, &join_path(path, "style"))?),
            _ => {
                return Err(SnapshotError::UnknownField {
                    path: join_path(path, key),
                });
            }
        }
    }
    match (when, style) {
        (Some(when), Some(style)) => Ok(Rule { when, style }),
        (None, _) => Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "missing `when`".to_string(),
        }),
        (_, None) => Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "missing `style`".to_string(),
        }),
    }
}

fn read_cond(j: &Json, path: &str) -> Result<Cond, SnapshotError> {
    Ok(Cond(read_arr(j, path, read_atom)?))
}

fn read_atom(j: &Json, path: &str) -> Result<Atom, SnapshotError> {
    let Json::Obj(fields) = j else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected an object".to_string(),
        });
    };
    if fields.len() != 1 {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "an atom is a single-key object (on/not/group/peer/has/media/viewport/container)"
                .to_string(),
        });
    }
    let (key, value) = &fields[0];
    let child = join_path(path, key);
    match key.as_str() {
        "on" => Ok(Atom::On(read_test(value, &child)?)),
        "not" => Ok(Atom::Not(read_test(value, &child)?)),
        "group" | "peer" => {
            let Json::Obj(inner) = value else {
                return Err(SnapshotError::InvalidValue {
                    path: child,
                    msg: "expected an object".to_string(),
                });
            };
            let mut test: Option<Test> = None;
            let mut label: Option<Option<Box<str>>> = None;
            for (k, v) in inner {
                match k.as_str() {
                    "test" => test = Some(read_test(v, &join_path(&child, "test"))?),
                    "label" => {
                        label = Some(match v {
                            Json::Null => None,
                            _ => Some(read_string(v, &join_path(&child, "label"))?.into()),
                        })
                    }
                    _ => {
                        return Err(SnapshotError::UnknownField {
                            path: join_path(&child, k),
                        });
                    }
                }
            }
            let Some(test) = test else {
                return Err(SnapshotError::InvalidValue {
                    path: child,
                    msg: "missing `test`".to_string(),
                });
            };
            let label = label.unwrap_or(None);
            if key == "group" {
                Ok(Atom::Group(test, label))
            } else {
                Ok(Atom::Peer(test, label))
            }
        }
        "has" => Ok(Atom::Has(read_test(value, &child)?)),
        "media" => {
            let media = parse_media(&read_string(value, &child)?).ok_or_else(|| {
                SnapshotError::InvalidValue {
                    path: child.clone(),
                    msg: "unknown media".to_string(),
                }
            })?;
            Ok(Atom::Media(media))
        }
        "viewport" => {
            let (min, max) = read_min_max(value, &child)?;
            Ok(Atom::Viewport { min, max })
        }
        "container" => {
            let Json::Obj(inner) = value else {
                return Err(SnapshotError::InvalidValue {
                    path: child,
                    msg: "expected an object".to_string(),
                });
            };
            let mut min = None;
            let mut max = None;
            let mut name: Option<Option<Box<str>>> = None;
            for (k, v) in inner {
                match k.as_str() {
                    "min" => min = read_opt_f32(v, &join_path(&child, "min"))?,
                    "max" => max = read_opt_f32(v, &join_path(&child, "max"))?,
                    "name" => {
                        name = Some(match v {
                            Json::Null => None,
                            _ => Some(read_string(v, &join_path(&child, "name"))?.into()),
                        })
                    }
                    _ => {
                        return Err(SnapshotError::UnknownField {
                            path: join_path(&child, k),
                        });
                    }
                }
            }
            let name = name.unwrap_or(None);
            Ok(Atom::Container { min, max, name })
        }
        _ => Err(SnapshotError::UnknownField {
            path: path.to_string(),
        }),
    }
}

/// The v0.7 §4 grid templates: `null` or an array of single-key
/// components (`track` / `repeat{count,track}`). Strict: unknown keys,
/// missing fields and non-canonical shapes are typed errors.
fn read_tracks(j: &Json, path: &str) -> Result<Option<Vec<TrackComponent>>, SnapshotError> {
    if matches!(j, Json::Null) {
        return Ok(None);
    }
    let Json::Arr(items) = j else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected an array or null".to_string(),
        });
    };
    let mut tracks = Vec::with_capacity(items.len());
    for item in items {
        let Json::Obj(fields) = item else {
            return Err(SnapshotError::InvalidValue {
                path: path.to_string(),
                msg: "expected a single-key object".to_string(),
            });
        };
        if fields.len() != 1 {
            return Err(SnapshotError::InvalidValue {
                path: path.to_string(),
                msg: "a track component is a single-key object (track/repeat)".to_string(),
            });
        }
        let (key, value) = &fields[0];
        let child = join_path(path, key);
        match key.as_str() {
            "track" => tracks.push(TrackComponent::Track(read_grid_track(value, &child)?)),
            "repeat" => {
                let Json::Obj(inner) = value else {
                    return Err(SnapshotError::InvalidValue {
                        path: child,
                        msg: "expected an object".to_string(),
                    });
                };
                let mut count: Option<u16> = None;
                let mut track: Option<GridTrack> = None;
                for (k, v) in inner {
                    let ip = join_path(&child, k);
                    match k.as_str() {
                        "count" => count = Some(read_u16(v, &ip)?),
                        "track" => track = Some(read_grid_track(v, &ip)?),
                        _ => return Err(SnapshotError::UnknownField { path: ip }),
                    }
                }
                let Some(count) = count else {
                    return Err(SnapshotError::InvalidValue {
                        path: child,
                        msg: "missing `count`".to_string(),
                    });
                };
                let Some(track) = track else {
                    return Err(SnapshotError::InvalidValue {
                        path: child,
                        msg: "missing `track`".to_string(),
                    });
                };
                tracks.push(TrackComponent::Repeat { count, track });
            }
            _ => {
                return Err(SnapshotError::UnknownField {
                    path: path.to_string(),
                });
            }
        }
    }
    Ok(Some(tracks))
}

fn read_grid_track(j: &Json, path: &str) -> Result<GridTrack, SnapshotError> {
    let Json::Obj(fields) = j else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected an object".to_string(),
        });
    };
    let mut min: Option<TrackSizing> = None;
    let mut max: Option<TrackSizing> = None;
    for (k, v) in fields {
        let ip = join_path(path, k);
        match k.as_str() {
            "min" => min = Some(read_track_sizing(v, &ip)?),
            "max" => max = Some(read_track_sizing(v, &ip)?),
            _ => return Err(SnapshotError::UnknownField { path: ip }),
        }
    }
    let Some(min) = min else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "missing `min`".to_string(),
        });
    };
    let Some(max) = max else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "missing `max`".to_string(),
        });
    };
    Ok(GridTrack { min, max })
}

fn read_track_sizing(j: &Json, path: &str) -> Result<TrackSizing, SnapshotError> {
    match j {
        Json::Str(s) => match s.as_str() {
            "auto" => Ok(TrackSizing::Auto),
            "min_content" => Ok(TrackSizing::MinContent),
            "max_content" => Ok(TrackSizing::MaxContent),
            _ => Err(SnapshotError::InvalidValue {
                path: path.to_string(),
                msg: "unknown track sizing keyword".to_string(),
            }),
        },
        Json::Obj(fields) if fields.len() == 1 => {
            let (key, value) = &fields[0];
            let child = join_path(path, key);
            match key.as_str() {
                "px" => Ok(TrackSizing::Px(read_f32(value, &child)?)),
                "percent" => Ok(TrackSizing::Percent(read_f32(value, &child)?)),
                "fr" => Ok(TrackSizing::Fr(read_f32(value, &child)?)),
                _ => Err(SnapshotError::UnknownField { path: child }),
            }
        }
        _ => Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected a keyword or single-key object".to_string(),
        }),
    }
}

fn read_min_max(j: &Json, path: &str) -> Result<(Option<f32>, Option<f32>), SnapshotError> {
    let Json::Obj(fields) = j else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected an object".to_string(),
        });
    };
    let mut min = None;
    let mut max = None;
    for (k, v) in fields {
        match k.as_str() {
            "min" => min = read_opt_f32(v, &join_path(path, "min"))?,
            "max" => max = read_opt_f32(v, &join_path(path, "max"))?,
            _ => {
                return Err(SnapshotError::UnknownField {
                    path: join_path(path, k),
                });
            }
        }
    }
    Ok((min, max))
}

fn read_test(j: &Json, path: &str) -> Result<Test, SnapshotError> {
    let Json::Obj(fields) = j else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected an object".to_string(),
        });
    };
    if fields.len() != 1 {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "a test is a single-key object (state/data/aria/structural/tag/pseudo)"
                .to_string(),
        });
    }
    let (key, value) = &fields[0];
    let child = join_path(path, key);
    match key.as_str() {
        "state" => {
            let state = parse_state(&read_string(value, &child)?).ok_or_else(|| {
                SnapshotError::InvalidValue {
                    path: child.clone(),
                    msg: "unknown state".to_string(),
                }
            })?;
            Ok(Test::State(state))
        }
        "data" | "aria" => {
            let Json::Obj(inner) = value else {
                return Err(SnapshotError::InvalidValue {
                    path: child,
                    msg: "expected an object".to_string(),
                });
            };
            let mut name: Option<String> = None;
            let mut attr_value: Option<Option<Box<str>>> = None;
            for (k, v) in inner {
                match k.as_str() {
                    "name" => name = Some(read_string(v, &join_path(&child, "name"))?),
                    "value" => {
                        attr_value = Some(match v {
                            Json::Null => None,
                            _ => Some(read_string(v, &join_path(&child, "value"))?.into()),
                        })
                    }
                    _ => {
                        return Err(SnapshotError::UnknownField {
                            path: join_path(&child, k),
                        });
                    }
                }
            }
            let Some(name) = name else {
                return Err(SnapshotError::InvalidValue {
                    path: child,
                    msg: "missing `name`".to_string(),
                });
            };
            let value = attr_value.unwrap_or(None);
            if key == "data" {
                Ok(Test::Data(name.into(), value))
            } else {
                Ok(Test::Aria(name.into(), value))
            }
        }
        "structural" => {
            let s = read_structural(value, &child)?;
            Ok(Test::Structural(s))
        }
        "tag" => Ok(Test::Tag(read_string(value, &child)?.into())),
        "pseudo" => {
            let name = read_string(value, &child)?;
            let pe = parse_pseudo(&name).ok_or_else(|| SnapshotError::InvalidValue {
                path: child.clone(),
                msg: "unknown pseudo-element".to_string(),
            })?;
            Ok(Test::Pseudo(pe))
        }
        _ => Err(SnapshotError::UnknownField {
            path: path.to_string(),
        }),
    }
}

fn read_structural(j: &Json, path: &str) -> Result<Structural, SnapshotError> {
    match j {
        Json::Str(s) => parse_structural(s).ok_or_else(|| SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: format!("unknown structural test {s:?}"),
        }),
        Json::Obj(fields) if fields.len() == 1 => {
            let (key, value) = &fields[0];
            match key.as_str() {
                "nth" => Ok(Structural::Nth(read_i32(value, &join_path(path, "nth"))?)),
                "nth_last" => Ok(Structural::NthLast(read_i32(
                    value,
                    &join_path(path, "nth_last"),
                )?)),
                _ => Err(SnapshotError::UnknownField {
                    path: join_path(path, key),
                }),
            }
        }
        _ => Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected a structural name or {\"nth\": n}".to_string(),
        }),
    }
}

fn read_style(j: &Json, path: &str) -> Result<ResolvedStyle, SnapshotError> {
    let Json::Obj(fields) = j else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected an object".to_string(),
        });
    };
    let mut s = ResolvedStyle::default();
    for (key, value) in fields {
        let p = join_path(path, key);
        match key.as_str() {
            "background" => s.background = read_opt_color_newtype(value, &p)?.map(Background),
            "background_gradient" => {
                s.background_gradient = match value {
                    Json::Null => None,
                    v => Some(read_gradient(v, &p)?),
                }
            }
            "content_color" => {
                s.content_color = read_opt_color_newtype(value, &p)?.map(ContentColor)
            }
            "padding" => {
                s.padding = read_opt_edges(value, &p, |t, r, b, l| Padding {
                    top: t,
                    right: r,
                    bottom: b,
                    left: l,
                })?
            }
            "border_color" => s.border_color = read_opt_color_newtype(value, &p)?.map(BorderColor),
            "border_width" => {
                s.border_width = read_opt_edges(value, &p, |t, r, b, l| BorderWidth {
                    top: t,
                    right: r,
                    bottom: b,
                    left: l,
                })?
            }
            "corner_radius" => {
                s.corner_radius = match value {
                    Json::Null => None,
                    v => {
                        let (tl, tr, br, bl) = read_corner_obj(v, &p)?;
                        Some(CornerRadius {
                            top_left: tl,
                            top_right: tr,
                            bottom_right: br,
                            bottom_left: bl,
                        })
                    }
                }
            }
            "dimensions" => s.dimensions = read_opt_dims(value, &p)?,
            "min_dimensions" => s.min_dimensions = read_opt_dims(value, &p)?,
            "max_dimensions" => s.max_dimensions = read_opt_dims(value, &p)?,
            "gap" => s.gap = read_opt_f32(value, &p)?.map(Gap),
            "shadow" => s.shadow = read_opt_shadow(value, &p)?,
            "margin" => {
                s.margin = read_opt_edges(value, &p, |t, r, b, l| Margin {
                    top: t,
                    right: r,
                    bottom: b,
                    left: l,
                })?
            }
            "layout" => s.layout = read_opt_layout(value, &p)?,
            "opacity" => s.opacity = read_opt_f32(value, &p)?.map(Opacity),
            "ring" => s.ring = read_opt_ring(value, &p)?,
            "outline" => s.outline = read_opt_outline(value, &p)?,
            "cursor" => {
                s.cursor = match value {
                    Json::Null => None,
                    v => Some(parse_cursor(&read_string(v, &p)?).ok_or_else(|| {
                        SnapshotError::InvalidValue {
                            path: p.clone(),
                            msg: "unknown cursor".to_string(),
                        }
                    })?),
                }
            }
            "pointer_events" => {
                s.pointer_events = match value {
                    Json::Null => None,
                    v => Some(parse_pointer_events(&read_string(v, &p)?).ok_or_else(|| {
                        SnapshotError::InvalidValue {
                            path: p.clone(),
                            msg: "unknown pointer-events".to_string(),
                        }
                    })?),
                }
            }
            "text" => {
                s.text = read_arr(value, &p, |item, ip| {
                    let Json::Arr(pair) = item else {
                        return Err(SnapshotError::InvalidValue {
                            path: ip.to_string(),
                            msg: "expected a [property, value] pair".to_string(),
                        });
                    };
                    if pair.len() != 2 {
                        return Err(SnapshotError::InvalidValue {
                            path: ip.to_string(),
                            msg: "expected exactly 2 elements".to_string(),
                        });
                    }
                    let prop = parse_text_property(&read_string(&pair[0], &join_path(ip, "0"))?)
                        .ok_or_else(|| SnapshotError::InvalidValue {
                            path: join_path(ip, "0"),
                            msg: "unknown text property".to_string(),
                        })?;
                    let value = read_text_value(&pair[1], &join_path(ip, "1"))?;
                    Ok((prop, value))
                })?;
            }
            "extras" => read_extras(value, &p, &mut s.extras)?,
            _ => return Err(SnapshotError::UnknownField { path: p }),
        }
    }
    Ok(s)
}

fn read_extras(j: &Json, path: &str, e: &mut Extras) -> Result<(), SnapshotError> {
    let Json::Obj(fields) = j else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected an object".to_string(),
        });
    };
    for (key, value) in fields {
        let p = join_path(path, key);
        match key.as_str() {
            "position" => {
                e.position = match value {
                    Json::Null => None,
                    v => Some(parse_position(&read_string(v, &p)?).ok_or_else(|| {
                        SnapshotError::InvalidValue {
                            path: p.clone(),
                            msg: "unknown position".to_string(),
                        }
                    })?),
                }
            }
            "inset" => {
                e.inset = match value {
                    Json::Null => None,
                    v => {
                        let Json::Obj(inner) = v else {
                            return Err(SnapshotError::InvalidValue {
                                path: p.clone(),
                                msg: "expected an object".to_string(),
                            });
                        };
                        let mut inset = Inset::default();
                        for (k, val) in inner {
                            let ip = join_path(&p, k);
                            let edge = match k.as_str() {
                                "top" => &mut inset.top,
                                "right" => &mut inset.right,
                                "bottom" => &mut inset.bottom,
                                "left" => &mut inset.left,
                                _ => return Err(SnapshotError::UnknownField { path: ip }),
                            };
                            *edge = read_opt_sized(val, &ip)?;
                        }
                        Some(inset)
                    }
                }
            }
            "z_index" => e.z_index = read_opt_i32(value, &p)?,
            "overflow" => {
                e.overflow = match value {
                    Json::Null => None,
                    v => {
                        let Json::Obj(inner) = v else {
                            return Err(SnapshotError::InvalidValue {
                                path: p.clone(),
                                msg: "expected an object".to_string(),
                            });
                        };
                        let mut o = Overflow::default();
                        for (k, val) in inner {
                            let ip = join_path(&p, k);
                            let axis = match k.as_str() {
                                "x" => &mut o.x,
                                "y" => &mut o.y,
                                _ => return Err(SnapshotError::UnknownField { path: ip }),
                            };
                            *axis = match val {
                                Json::Null => None,
                                val => {
                                    Some(parse_overflow_kind(&read_string(val, &ip)?).ok_or_else(
                                        || SnapshotError::InvalidValue {
                                            path: ip.clone(),
                                            msg: "unknown overflow kind".to_string(),
                                        },
                                    )?)
                                }
                            };
                        }
                        Some(o)
                    }
                }
            }
            "visibility" => {
                e.visibility = match value {
                    Json::Null => None,
                    v => Some(parse_visibility(&read_string(v, &p)?).ok_or_else(|| {
                        SnapshotError::InvalidValue {
                            path: p.clone(),
                            msg: "unknown visibility".to_string(),
                        }
                    })?),
                }
            }
            "aspect_ratio" => e.aspect_ratio = read_opt_f32(value, &p)?,
            "grid" => {
                e.grid = match value {
                    Json::Null => None,
                    v => {
                        let Json::Obj(inner) = v else {
                            return Err(SnapshotError::InvalidValue {
                                path: p.clone(),
                                msg: "expected an object".to_string(),
                            });
                        };
                        let mut g = Grid::default();
                        for (k, val) in inner {
                            let ip = join_path(&p, k);
                            match k.as_str() {
                                "cols" => g.cols = read_opt_u16(val, &ip)?,
                                "rows" => g.rows = read_opt_u16(val, &ip)?,
                                "col_span" => g.col_span = read_opt_u16(val, &ip)?,
                                "row_span" => g.row_span = read_opt_u16(val, &ip)?,
                                "col_start" => g.col_start = read_opt_i16(val, &ip)?,
                                "row_start" => g.row_start = read_opt_i16(val, &ip)?,
                                "template_cols" => g.template_cols = read_tracks(val, &ip)?,
                                "template_rows" => g.template_rows = read_tracks(val, &ip)?,
                                "auto_rows" => g.auto_rows = read_tracks(val, &ip)?,
                                "auto_cols" => g.auto_cols = read_tracks(val, &ip)?,
                                _ => return Err(SnapshotError::UnknownField { path: ip }),
                            }
                        }
                        Some(g)
                    }
                }
            }
            "transform" => {
                e.transform = match value {
                    Json::Null => None,
                    v => {
                        let Json::Obj(inner) = v else {
                            return Err(SnapshotError::InvalidValue {
                                path: p.clone(),
                                msg: "expected an object".to_string(),
                            });
                        };
                        let mut t = Transform::default();
                        for (k, val) in inner {
                            let ip = join_path(&p, k);
                            match k.as_str() {
                                "scale_x" => t.scale_x = read_opt_f32(val, &ip)?,
                                "scale_y" => t.scale_y = read_opt_f32(val, &ip)?,
                                "rotate_deg" => t.rotate_deg = read_opt_f32(val, &ip)?,
                                "translate_x" => t.translate_x = read_opt_sized(val, &ip)?,
                                "translate_y" => t.translate_y = read_opt_sized(val, &ip)?,
                                "skew_x" => t.skew_x = read_opt_f32(val, &ip)?,
                                "skew_y" => t.skew_y = read_opt_f32(val, &ip)?,
                                _ => return Err(SnapshotError::UnknownField { path: ip }),
                            }
                        }
                        Some(t)
                    }
                }
            }
            "transition" => {
                e.transition = match value {
                    Json::Null => None,
                    v => {
                        let Json::Obj(inner) = v else {
                            return Err(SnapshotError::InvalidValue {
                                path: p.clone(),
                                msg: "expected an object".to_string(),
                            });
                        };
                        let mut set: Option<TransitionSet> = None;
                        let mut duration_ms = 0.0f32;
                        let mut ease: Option<Option<Ease>> = None;
                        let mut delay_ms = 0.0f32;
                        for (k, val) in inner {
                            let ip = join_path(&p, k);
                            match k.as_str() {
                                "set" => {
                                    set = Some(
                                        parse_transition_set(&read_string(val, &ip)?).ok_or_else(
                                            || SnapshotError::InvalidValue {
                                                path: ip.clone(),
                                                msg: "unknown transition set".to_string(),
                                            },
                                        )?,
                                    )
                                }
                                "duration_ms" => duration_ms = read_f32(val, &ip)?,
                                "ease" => {
                                    ease = Some(match val {
                                        Json::Null => None,
                                        val => {
                                            Some(parse_ease(&read_string(val, &ip)?).ok_or_else(
                                                || SnapshotError::InvalidValue {
                                                    path: ip.clone(),
                                                    msg: "unknown ease".to_string(),
                                                },
                                            )?)
                                        }
                                    })
                                }
                                "delay_ms" => delay_ms = read_f32(val, &ip)?,
                                _ => return Err(SnapshotError::UnknownField { path: ip }),
                            }
                        }
                        let Some(set) = set else {
                            return Err(SnapshotError::InvalidValue {
                                path: p.clone(),
                                msg: "missing `set`".to_string(),
                            });
                        };
                        Some(Transition {
                            set,
                            duration_ms,
                            ease: ease.unwrap_or(None),
                            delay_ms,
                        })
                    }
                }
            }
            "animation" => {
                e.animation = match value {
                    Json::Null => None,
                    v => {
                        let Json::Obj(inner) = v else {
                            return Err(SnapshotError::InvalidValue {
                                path: p.clone(),
                                msg: "expected an object".to_string(),
                            });
                        };
                        let mut name: Option<String> = None;
                        let mut duration_ms: Option<Option<f32>> = None;
                        let mut ease: Option<Option<Ease>> = None;
                        let mut infinite: Option<bool> = None;
                        for (k, val) in inner {
                            let ip = join_path(&p, k);
                            match k.as_str() {
                                "name" => name = Some(read_string(val, &ip)?),
                                "duration_ms" => duration_ms = Some(read_opt_f32(val, &ip)?),
                                "ease" => {
                                    ease = Some(match val {
                                        Json::Null => None,
                                        val => {
                                            Some(parse_ease(&read_string(val, &ip)?).ok_or_else(
                                                || SnapshotError::InvalidValue {
                                                    path: ip.clone(),
                                                    msg: "unknown ease".to_string(),
                                                },
                                            )?)
                                        }
                                    })
                                }
                                "infinite" => infinite = Some(read_bool(val, &ip)?),
                                _ => return Err(SnapshotError::UnknownField { path: ip }),
                            }
                        }
                        let Some(name) = name else {
                            return Err(SnapshotError::InvalidValue {
                                path: p.clone(),
                                msg: "missing `name`".to_string(),
                            });
                        };
                        let Some(duration_ms) = duration_ms else {
                            return Err(SnapshotError::InvalidValue {
                                path: p.clone(),
                                msg: "missing `duration_ms`".to_string(),
                            });
                        };
                        let Some(infinite) = infinite else {
                            return Err(SnapshotError::InvalidValue {
                                path: p.clone(),
                                msg: "missing `infinite`".to_string(),
                            });
                        };
                        Some(Animation {
                            name: name.into(),
                            duration_ms,
                            ease: ease.unwrap_or(None),
                            infinite,
                        })
                    }
                }
            }
            "border_style" => {
                e.border_style = match value {
                    Json::Null => None,
                    v => Some(parse_border_style(&read_string(v, &p)?).ok_or_else(|| {
                        SnapshotError::InvalidValue {
                            path: p.clone(),
                            msg: "unknown border style".to_string(),
                        }
                    })?),
                }
            }
            "flex_basis" => {
                e.flex_basis = match value {
                    Json::Null => None,
                    v => Some(read_sized(v, &p)?),
                }
            }
            "order" => e.order = read_opt_i16(value, &p)?,
            "align_content" => {
                e.align_content = match value {
                    Json::Null => None,
                    v => Some(parse_content_align(&read_string(v, &p)?).ok_or_else(|| {
                        SnapshotError::InvalidValue {
                            path: p.clone(),
                            msg: "unknown content alignment".to_string(),
                        }
                    })?),
                }
            }
            "margin_auto" => {
                e.margin_auto = match value {
                    Json::Null => None,
                    v => {
                        let Json::Obj(inner) = v else {
                            return Err(SnapshotError::InvalidValue {
                                path: p.clone(),
                                msg: "expected an object".to_string(),
                            });
                        };
                        let mut m = AutoEdges::default();
                        for (k, val) in inner {
                            let ip = join_path(&p, k);
                            let edge = match k.as_str() {
                                "left" => &mut m.left,
                                "right" => &mut m.right,
                                "top" => &mut m.top,
                                "bottom" => &mut m.bottom,
                                _ => return Err(SnapshotError::UnknownField { path: ip }),
                            };
                            *edge = read_bool(val, &ip)?;
                        }
                        Some(m)
                    }
                }
            }
            "gap_axes" => {
                e.gap_axes = match value {
                    Json::Null => None,
                    v => {
                        let Json::Obj(inner) = v else {
                            return Err(SnapshotError::InvalidValue {
                                path: p.clone(),
                                msg: "expected an object".to_string(),
                            });
                        };
                        let mut g = GapAxes::default();
                        for (k, val) in inner {
                            let ip = join_path(&p, k);
                            match k.as_str() {
                                "col" => g.col = read_opt_f32(val, &ip)?,
                                "row" => g.row = read_opt_f32(val, &ip)?,
                                _ => return Err(SnapshotError::UnknownField { path: ip }),
                            }
                        }
                        Some(g)
                    }
                }
            }
            "shadow_rest" => e.shadow_rest = read_arr(value, &p, read_shadow)?,
            "inset_shadows" => e.inset_shadows = read_arr(value, &p, read_shadow)?,
            "container_type" => {
                e.container_type = match value {
                    Json::Null => None,
                    v => {
                        let Json::Obj(inner) = v else {
                            return Err(SnapshotError::InvalidValue {
                                path: p.clone(),
                                msg: "expected an object".to_string(),
                            });
                        };
                        let mut name: Option<Option<Box<str>>> = None;
                        for (k, val) in inner {
                            let ip = join_path(&p, k);
                            match k.as_str() {
                                "name" => {
                                    name = Some(match val {
                                        Json::Null => None,
                                        _ => Some(read_string(val, &ip)?.into()),
                                    })
                                }
                                _ => return Err(SnapshotError::UnknownField { path: ip }),
                            }
                        }
                        Some(ContainerType {
                            name: name.unwrap_or(None),
                        })
                    }
                }
            }
            _ => return Err(SnapshotError::UnknownField { path: p }),
        }
    }
    Ok(())
}

fn read_gradient(j: &Json, path: &str) -> Result<Gradient, SnapshotError> {
    if let Json::Str(s) = j {
        if s == "none" {
            return Ok(Gradient::None);
        }
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: format!("unknown gradient form {s:?} (expected \"none\" or an object)"),
        });
    }
    let Json::Obj(fields) = j else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected \"none\" or an object".to_string(),
        });
    };
    let mut shape = String::new();
    let mut angle_deg = 0.0f32;
    let mut from_angle_deg = 0.0f32;
    let mut stops: Option<GradientStops> = None;
    for (key, value) in fields {
        let p = join_path(path, key);
        match key.as_str() {
            "shape" => shape = read_string(value, &p)?,
            "angle_deg" => angle_deg = read_f32(value, &p)?,
            "from_angle_deg" => from_angle_deg = read_f32(value, &p)?,
            "stops" => stops = Some(read_stops(value, &p)?),
            _ => return Err(SnapshotError::UnknownField { path: p }),
        }
    }
    let Some(stops) = stops else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "missing `stops`".to_string(),
        });
    };
    match shape.as_str() {
        "linear" => Ok(Gradient::Linear { angle_deg, stops }),
        "radial" => Ok(Gradient::Radial { stops }),
        "conic" => Ok(Gradient::Conic {
            from_angle_deg,
            stops,
        }),
        other => Err(SnapshotError::InvalidValue {
            path: join_path(path, "shape"),
            msg: format!("unknown gradient shape {other:?}"),
        }),
    }
}

fn read_stops(j: &Json, path: &str) -> Result<GradientStops, SnapshotError> {
    let Json::Obj(fields) = j else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected an object".to_string(),
        });
    };
    let mut from: Option<GradientStop> = None;
    let mut via: Option<Option<GradientStop>> = None;
    let mut to: Option<GradientStop> = None;
    for (key, value) in fields {
        let p = join_path(path, key);
        match key.as_str() {
            "from" => from = Some(read_stop(value, &p)?),
            "via" => {
                via = Some(match value {
                    Json::Null => None,
                    v => Some(read_stop(v, &p)?),
                })
            }
            "to" => to = Some(read_stop(value, &p)?),
            _ => return Err(SnapshotError::UnknownField { path: p }),
        }
    }
    Ok(GradientStops {
        from: from.ok_or_else(|| SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "missing `from`".to_string(),
        })?,
        via: via.unwrap_or(None),
        to: to.ok_or_else(|| SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "missing `to`".to_string(),
        })?,
    })
}

fn read_stop(j: &Json, path: &str) -> Result<GradientStop, SnapshotError> {
    let Json::Obj(fields) = j else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected an object".to_string(),
        });
    };
    let mut color: Option<Color> = None;
    let mut pos_pct: Option<Option<f32>> = None;
    for (key, value) in fields {
        let p = join_path(path, key);
        match key.as_str() {
            "color" => color = Some(read_color(value, &p)?),
            "pos_pct" => pos_pct = Some(read_opt_f32(value, &p)?),
            _ => return Err(SnapshotError::UnknownField { path: p }),
        }
    }
    Ok(GradientStop {
        color: color.ok_or_else(|| SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "missing `color`".to_string(),
        })?,
        pos_pct: pos_pct.unwrap_or(None),
    })
}

fn read_shadow(j: &Json, path: &str) -> Result<BoxShadow, SnapshotError> {
    let Json::Obj(fields) = j else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected an object".to_string(),
        });
    };
    let mut sh = BoxShadow {
        offset_x: 0.0,
        offset_y: 0.0,
        blur: 0.0,
        spread: 0.0,
        color: Color::TRANSPARENT,
        inset: false,
    };
    let mut seen = [false; 6];
    for (key, value) in fields {
        let p = join_path(path, key);
        match key.as_str() {
            "offset_x" => {
                seen[0] = true;
                sh.offset_x = read_f32(value, &p)?;
            }
            "offset_y" => {
                seen[1] = true;
                sh.offset_y = read_f32(value, &p)?;
            }
            "blur" => {
                seen[2] = true;
                sh.blur = read_f32(value, &p)?;
            }
            "spread" => {
                seen[3] = true;
                sh.spread = read_f32(value, &p)?;
            }
            "color" => {
                seen[4] = true;
                sh.color = read_color(value, &p)?;
            }
            "inset" => {
                seen[5] = true;
                sh.inset = read_bool(value, &p)?;
            }
            _ => return Err(SnapshotError::UnknownField { path: p }),
        }
    }
    if seen.iter().any(|s| !s) {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "a shadow requires offset_x/offset_y/blur/spread/color/inset".to_string(),
        });
    }
    Ok(sh)
}

fn read_opt_shadow(j: &Json, path: &str) -> Result<Option<BoxShadow>, SnapshotError> {
    match j {
        Json::Null => Ok(None),
        v => Ok(Some(read_shadow(v, path)?)),
    }
}

fn read_opt_dims(j: &Json, path: &str) -> Result<Option<Dimensions>, SnapshotError> {
    match j {
        Json::Null => Ok(None),
        v => {
            let Json::Obj(fields) = v else {
                return Err(SnapshotError::InvalidValue {
                    path: path.to_string(),
                    msg: "expected an object".to_string(),
                });
            };
            let mut width = None;
            let mut height = None;
            for (key, value) in fields {
                let p = join_path(path, key);
                match key.as_str() {
                    "width" => width = read_opt_sized(value, &p)?,
                    "height" => height = read_opt_sized(value, &p)?,
                    _ => return Err(SnapshotError::UnknownField { path: p }),
                }
            }
            Ok(Some(Dimensions { width, height }))
        }
    }
}

fn read_opt_layout(j: &Json, path: &str) -> Result<Option<Layout>, SnapshotError> {
    match j {
        Json::Null => Ok(None),
        v => {
            let Json::Obj(fields) = v else {
                return Err(SnapshotError::InvalidValue {
                    path: path.to_string(),
                    msg: "expected an object".to_string(),
                });
            };
            let mut l = Layout::default();
            for (key, value) in fields {
                let p = join_path(path, key);
                match key.as_str() {
                    "display" => l.display = read_named_opt(value, &p, parse_display)?,
                    "flex_direction" => {
                        l.flex_direction = read_named_opt(value, &p, parse_flex_direction)?
                    }
                    "align_items" => l.align_items = read_named_opt(value, &p, parse_align_items)?,
                    "justify_content" => {
                        l.justify_content = read_named_opt(value, &p, parse_justify_content)?
                    }
                    "justify_items" => {
                        l.justify_items = read_named_opt(value, &p, parse_align_items)?
                    }
                    "flex_wrap" => l.flex_wrap = read_opt_bool(value, &p)?,
                    "flex_grow" => l.flex_grow = read_opt_f32(value, &p)?,
                    "flex_shrink" => l.flex_shrink = read_opt_f32(value, &p)?,
                    "align_self" => l.align_self = read_named_opt(value, &p, parse_align_items)?,
                    _ => return Err(SnapshotError::UnknownField { path: p }),
                }
            }
            Ok(Some(l))
        }
    }
}

fn read_opt_ring(j: &Json, path: &str) -> Result<Option<Ring>, SnapshotError> {
    match j {
        Json::Null => Ok(None),
        v => {
            let Json::Obj(fields) = v else {
                return Err(SnapshotError::InvalidValue {
                    path: path.to_string(),
                    msg: "expected an object".to_string(),
                });
            };
            let mut r = Ring::default();
            for (key, value) in fields {
                let p = join_path(path, key);
                match key.as_str() {
                    "width" => r.width = read_f32(value, &p)?,
                    "color" => r.color = read_opt_color(value, &p)?,
                    "offset_width" => r.offset_width = read_f32(value, &p)?,
                    "offset_color" => r.offset_color = read_opt_color(value, &p)?,
                    "inset" => r.inset = read_bool(value, &p)?,
                    _ => return Err(SnapshotError::UnknownField { path: p }),
                }
            }
            Ok(Some(r))
        }
    }
}

fn read_opt_outline(j: &Json, path: &str) -> Result<Option<Outline>, SnapshotError> {
    match j {
        Json::Null => Ok(None),
        v => {
            let Json::Obj(fields) = v else {
                return Err(SnapshotError::InvalidValue {
                    path: path.to_string(),
                    msg: "expected an object".to_string(),
                });
            };
            let mut o = Outline::default();
            for (key, value) in fields {
                let p = join_path(path, key);
                match key.as_str() {
                    "width" => o.width = read_f32(value, &p)?,
                    "color" => o.color = read_opt_color(value, &p)?,
                    "none" => o.none = read_bool(value, &p)?,
                    "offset" => o.offset = read_f32(value, &p)?,
                    _ => return Err(SnapshotError::UnknownField { path: p }),
                }
            }
            Ok(Some(o))
        }
    }
}

fn read_sized(j: &Json, path: &str) -> Result<SizedValue, SnapshotError> {
    match j {
        Json::Str(s) => match s.as_str() {
            "full" => Ok(SizedValue::Full),
            "fit" => Ok(SizedValue::FitContent),
            "auto" => Ok(SizedValue::Auto),
            "viewport" => Ok(SizedValue::Viewport),
            other => Err(SnapshotError::InvalidValue {
                path: path.to_string(),
                msg: format!("unknown sized value {other:?}"),
            }),
        },
        Json::Obj(fields) if fields.len() == 1 => {
            let (key, value) = &fields[0];
            let p = join_path(path, key);
            match key.as_str() {
                "px" => Ok(SizedValue::Px(read_f32(value, &p)?)),
                "scale" => Ok(SizedValue::Scale(read_f32(value, &p)?)),
                "fraction" => Ok(SizedValue::Fraction(read_f32(value, &p)?)),
                _ => Err(SnapshotError::UnknownField {
                    path: path.to_string(),
                }),
            }
        }
        _ => Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected a sized value string or {px|scale|fraction: n}".to_string(),
        }),
    }
}

fn read_opt_sized(j: &Json, path: &str) -> Result<Option<SizedValue>, SnapshotError> {
    match j {
        Json::Null => Ok(None),
        v => Ok(Some(read_sized(v, path)?)),
    }
}

fn read_text_value(j: &Json, path: &str) -> Result<TextValue, SnapshotError> {
    let Json::Obj(fields) = j else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected an object".to_string(),
        });
    };
    if fields.len() != 1 {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "a text value is a single-key object".to_string(),
        });
    }
    let (key, value) = &fields[0];
    let p = join_path(path, key);
    match key.as_str() {
        "px" => Ok(TextValue::Px(read_f32(value, &p)?)),
        "weight" => Ok(TextValue::Weight(read_u16(value, &p)?)),
        "spacing" => Ok(TextValue::Spacing(read_f32(value, &p)?)),
        "tracking_em" => Ok(TextValue::TrackingEm(read_f32(value, &p)?)),
        "line_height_ratio" => Ok(TextValue::LineHeightRatio(read_f32(value, &p)?)),
        "keyword" => {
            let kw = read_string(value, &p)?;
            let kw_static = keyword_static(&kw).ok_or_else(|| SnapshotError::InvalidValue {
                path: p.clone(),
                msg: format!("unknown keyword {kw:?}"),
            })?;
            Ok(TextValue::Keyword(kw_static))
        }
        "count" => Ok(TextValue::Count(read_u16(value, &p)?)),
        "color" => Ok(TextValue::Color(read_color(value, &p)?)),
        _ => Err(SnapshotError::UnknownField {
            path: path.to_string(),
        }),
    }
}

// -- scalar readers ----------------------------------------------------------

fn read_string(j: &Json, path: &str) -> Result<String, SnapshotError> {
    match j {
        Json::Str(s) => Ok(s.clone()),
        _ => Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected a string".to_string(),
        }),
    }
}

fn read_bool(j: &Json, path: &str) -> Result<bool, SnapshotError> {
    match j {
        Json::Bool(b) => Ok(*b),
        _ => Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected a boolean".to_string(),
        }),
    }
}

fn read_opt_bool(j: &Json, path: &str) -> Result<Option<bool>, SnapshotError> {
    match j {
        Json::Null => Ok(None),
        v => Ok(Some(read_bool(v, path)?)),
    }
}

fn read_f32(j: &Json, path: &str) -> Result<f32, SnapshotError> {
    match j {
        Json::Num(n) => {
            // JSON numbers are finite by definition; the writer only ever
            // emits f32-representable values, so anything else is corrupt
            // input (or a value that would silently change in f32).
            if !n.is_finite() {
                return Err(SnapshotError::InvalidValue {
                    path: path.to_string(),
                    msg: "number is out of range".to_string(),
                });
            }
            let v = *n as f32;
            if !v.is_finite() {
                return Err(SnapshotError::InvalidValue {
                    path: path.to_string(),
                    msg: "number does not fit in f32".to_string(),
                });
            }
            Ok(v)
        }
        Json::Str(s) => match s.as_str() {
            "NaN" => Ok(f32::NAN),
            "inf" => Ok(f32::INFINITY),
            "-inf" => Ok(f32::NEG_INFINITY),
            other => Err(SnapshotError::InvalidValue {
                path: path.to_string(),
                msg: format!("expected a number, found {other:?}"),
            }),
        },
        _ => Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected a number".to_string(),
        }),
    }
}

fn read_opt_f32(j: &Json, path: &str) -> Result<Option<f32>, SnapshotError> {
    match j {
        Json::Null => Ok(None),
        v => Ok(Some(read_f32(v, path)?)),
    }
}

fn read_u64(j: &Json, path: &str) -> Result<u64, SnapshotError> {
    match j {
        Json::Num(n) if *n >= 0.0 && n.fract() == 0.0 && *n <= u32::MAX as f64 => Ok(*n as u64),
        _ => Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected a non-negative integer".to_string(),
        }),
    }
}

fn read_i32(j: &Json, path: &str) -> Result<i32, SnapshotError> {
    match j {
        Json::Num(n) if n.fract() == 0.0 && *n >= i32::MIN as f64 && *n <= i32::MAX as f64 => {
            Ok(*n as i32)
        }
        _ => Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected an integer".to_string(),
        }),
    }
}

fn read_opt_i32(j: &Json, path: &str) -> Result<Option<i32>, SnapshotError> {
    match j {
        Json::Null => Ok(None),
        v => Ok(Some(read_i32(v, path)?)),
    }
}

fn read_u16(j: &Json, path: &str) -> Result<u16, SnapshotError> {
    match j {
        Json::Num(n) if n.fract() == 0.0 && *n >= 0.0 && *n <= u16::MAX as f64 => Ok(*n as u16),
        _ => Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected an integer in 0..=65535".to_string(),
        }),
    }
}

fn read_opt_u16(j: &Json, path: &str) -> Result<Option<u16>, SnapshotError> {
    match j {
        Json::Null => Ok(None),
        v => Ok(Some(read_u16(v, path)?)),
    }
}

fn read_i16(j: &Json, path: &str) -> Result<i16, SnapshotError> {
    match j {
        Json::Num(n) if n.fract() == 0.0 && *n >= i16::MIN as f64 && *n <= i16::MAX as f64 => {
            Ok(*n as i16)
        }
        _ => Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected an integer in -32768..=32767".to_string(),
        }),
    }
}

fn read_opt_i16(j: &Json, path: &str) -> Result<Option<i16>, SnapshotError> {
    match j {
        Json::Null => Ok(None),
        v => Ok(Some(read_i16(v, path)?)),
    }
}

fn read_color(j: &Json, path: &str) -> Result<Color, SnapshotError> {
    let Json::Arr(items) = j else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected a color as [l, a, b, alpha]".to_string(),
        });
    };
    if items.len() != 4 {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "a color is exactly 4 numbers".to_string(),
        });
    }
    let mut c = [0.0f32; 4];
    for (i, item) in items.iter().enumerate() {
        c[i] = read_f32(item, &format!("{path}[{i}]"))?;
    }
    Ok(Color {
        l: c[0],
        a: c[1],
        b: c[2],
        alpha: c[3],
    })
}

fn read_opt_color(j: &Json, path: &str) -> Result<Option<Color>, SnapshotError> {
    match j {
        Json::Null => Ok(None),
        v => Ok(Some(read_color(v, path)?)),
    }
}

fn read_opt_color_newtype(j: &Json, path: &str) -> Result<Option<Color>, SnapshotError> {
    read_opt_color(j, path)
}

fn read_opt_edges<T>(
    j: &Json,
    path: &str,
    build: impl Fn(f32, f32, f32, f32) -> T,
) -> Result<Option<T>, SnapshotError> {
    match j {
        Json::Null => Ok(None),
        v => {
            let Json::Obj(fields) = v else {
                return Err(SnapshotError::InvalidValue {
                    path: path.to_string(),
                    msg: "expected an object".to_string(),
                });
            };
            let (mut top, mut right, mut bottom, mut left) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
            for (key, value) in fields {
                let p = join_path(path, key);
                match key.as_str() {
                    "top" => top = read_f32(value, &p)?,
                    "right" => right = read_f32(value, &p)?,
                    "bottom" => bottom = read_f32(value, &p)?,
                    "left" => left = read_f32(value, &p)?,
                    _ => return Err(SnapshotError::UnknownField { path: p }),
                }
            }
            Ok(Some(build(top, right, bottom, left)))
        }
    }
}

fn read_corner_obj(j: &Json, path: &str) -> Result<(f32, f32, f32, f32), SnapshotError> {
    let Json::Obj(fields) = j else {
        return Err(SnapshotError::InvalidValue {
            path: path.to_string(),
            msg: "expected an object".to_string(),
        });
    };
    let mut tl = 0.0f32;
    let mut tr = 0.0f32;
    let mut br = 0.0f32;
    let mut bl = 0.0f32;
    for (key, value) in fields {
        let p = join_path(path, key);
        match key.as_str() {
            "top_left" => tl = read_f32(value, &p)?,
            "top_right" => tr = read_f32(value, &p)?,
            "bottom_right" => br = read_f32(value, &p)?,
            "bottom_left" => bl = read_f32(value, &p)?,
            _ => return Err(SnapshotError::UnknownField { path: p }),
        }
    }
    Ok((tl, tr, br, bl))
}

fn read_named_opt<T: Copy>(
    j: &Json,
    path: &str,
    parse: impl Fn(&str) -> Option<T>,
) -> Result<Option<T>, SnapshotError> {
    match j {
        Json::Null => Ok(None),
        v => {
            let s = read_string(v, path)?;
            parse(&s)
                .map(Some)
                .ok_or_else(|| SnapshotError::InvalidValue {
                    path: path.to_string(),
                    msg: format!("unknown name {s:?}"),
                })
        }
    }
}

// -- enum name tables (writer side) ------------------------------------------

fn variant_condition_name(c: VariantCondition) -> &'static str {
    match c {
        VariantCondition::Hover => "hover",
        VariantCondition::Active => "active",
        VariantCondition::Disabled => "disabled",
        VariantCondition::Focus => "focus",
        VariantCondition::FocusVisible => "focus_visible",
        VariantCondition::FocusWithin => "focus_within",
        VariantCondition::Visited => "visited",
    }
}

fn parse_variant_condition(s: &str) -> Option<VariantCondition> {
    Some(match s {
        "hover" => VariantCondition::Hover,
        "active" => VariantCondition::Active,
        "disabled" => VariantCondition::Disabled,
        "focus" => VariantCondition::Focus,
        "focus_visible" => VariantCondition::FocusVisible,
        "focus_within" => VariantCondition::FocusWithin,
        "visited" => VariantCondition::Visited,
        _ => return None,
    })
}

fn breakpoint_name(b: Breakpoint) -> &'static str {
    match b {
        Breakpoint::Sm => "sm",
        Breakpoint::Md => "md",
        Breakpoint::Lg => "lg",
        Breakpoint::Xl => "xl",
        Breakpoint::Xxl => "2xl",
    }
}

fn parse_breakpoint(s: &str) -> Option<Breakpoint> {
    Some(match s {
        "sm" => Breakpoint::Sm,
        "md" => Breakpoint::Md,
        "lg" => Breakpoint::Lg,
        "xl" => Breakpoint::Xl,
        "2xl" => Breakpoint::Xxl,
        _ => return None,
    })
}

fn state_name(s: State) -> &'static str {
    match s {
        State::Hover => "hover",
        State::Focus => "focus",
        State::FocusVisible => "focus_visible",
        State::FocusWithin => "focus_within",
        State::Active => "active",
        State::Disabled => "disabled",
        State::Visited => "visited",
        State::Target => "target",
        State::Enabled => "enabled",
        State::Checked => "checked",
        State::Indeterminate => "indeterminate",
        State::Required => "required",
        State::Optional => "optional",
        State::Valid => "valid",
        State::Invalid => "invalid",
        State::UserValid => "user_valid",
        State::UserInvalid => "user_invalid",
        State::PlaceholderShown => "placeholder_shown",
        State::Autofill => "autofill",
        State::ReadOnly => "read_only",
        State::InRange => "in_range",
        State::OutOfRange => "out_of_range",
    }
}

fn parse_state(s: &str) -> Option<State> {
    Some(match s {
        "hover" => State::Hover,
        "focus" => State::Focus,
        "focus_visible" => State::FocusVisible,
        "focus_within" => State::FocusWithin,
        "active" => State::Active,
        "disabled" => State::Disabled,
        "visited" => State::Visited,
        "target" => State::Target,
        "enabled" => State::Enabled,
        "checked" => State::Checked,
        "indeterminate" => State::Indeterminate,
        "required" => State::Required,
        "optional" => State::Optional,
        "valid" => State::Valid,
        "invalid" => State::Invalid,
        "user_valid" => State::UserValid,
        "user_invalid" => State::UserInvalid,
        "placeholder_shown" => State::PlaceholderShown,
        "autofill" => State::Autofill,
        "read_only" => State::ReadOnly,
        "in_range" => State::InRange,
        "out_of_range" => State::OutOfRange,
        _ => return None,
    })
}

fn media_name(m: Media) -> &'static str {
    match m {
        Media::Dark => "dark",
        Media::Print => "print",
        Media::MotionSafe => "motion_safe",
        Media::MotionReduce => "motion_reduce",
        Media::ContrastMore => "contrast_more",
        Media::ContrastLess => "contrast_less",
        Media::ForcedColors => "forced_colors",
        Media::Portrait => "portrait",
        Media::Landscape => "landscape",
        Media::Rtl => "rtl",
        Media::Ltr => "ltr",
        Media::PointerFine => "pointer_fine",
        Media::PointerCoarse => "pointer_coarse",
        Media::PointerNone => "pointer_none",
        Media::AnyPointerFine => "any_pointer_fine",
        Media::AnyPointerCoarse => "any_pointer_coarse",
        Media::AnyPointerNone => "any_pointer_none",
    }
}

/// The pseudo-element variant names (v0.7 §1) as accepted by the reader —
/// the same names the parser registers for `selection:`/`placeholder:`/`file:`.
fn parse_pseudo(s: &str) -> Option<PseudoElement> {
    Some(match s {
        "selection" => PseudoElement::Selection,
        "placeholder" => PseudoElement::Placeholder,
        "file" => PseudoElement::FileSelectorButton,
        _ => return None,
    })
}

fn parse_media(s: &str) -> Option<Media> {
    Some(match s {
        "dark" => Media::Dark,
        "print" => Media::Print,
        "motion_safe" => Media::MotionSafe,
        "motion_reduce" => Media::MotionReduce,
        "contrast_more" => Media::ContrastMore,
        "contrast_less" => Media::ContrastLess,
        "forced_colors" => Media::ForcedColors,
        "portrait" => Media::Portrait,
        "landscape" => Media::Landscape,
        "rtl" => Media::Rtl,
        "ltr" => Media::Ltr,
        "pointer_fine" => Media::PointerFine,
        "pointer_coarse" => Media::PointerCoarse,
        "pointer_none" => Media::PointerNone,
        "any_pointer_fine" => Media::AnyPointerFine,
        "any_pointer_coarse" => Media::AnyPointerCoarse,
        "any_pointer_none" => Media::AnyPointerNone,
        _ => return None,
    })
}

fn structural_name(s: &Structural) -> &'static str {
    match s {
        Structural::First => "first",
        Structural::Last => "last",
        Structural::Odd => "odd",
        Structural::Even => "even",
        Structural::Only => "only",
        Structural::Empty => "empty",
        Structural::Nth(_) | Structural::NthLast(_) => "",
    }
}

fn parse_structural(s: &str) -> Option<Structural> {
    Some(match s {
        "first" => Structural::First,
        "last" => Structural::Last,
        "odd" => Structural::Odd,
        "even" => Structural::Even,
        "only" => Structural::Only,
        "empty" => Structural::Empty,
        _ => return None,
    })
}

fn display_name(d: &Display) -> &'static str {
    match d {
        Display::Flex => "flex",
        Display::InlineFlex => "inline_flex",
        Display::Grid => "grid",
        Display::Block => "block",
        Display::Inline => "inline",
        Display::None => "none",
    }
}

fn parse_display(s: &str) -> Option<Display> {
    Some(match s {
        "flex" => Display::Flex,
        "inline_flex" => Display::InlineFlex,
        "grid" => Display::Grid,
        "block" => Display::Block,
        "inline" => Display::Inline,
        "none" => Display::None,
        _ => return None,
    })
}

fn flex_direction_name(d: &FlexDirection) -> &'static str {
    match d {
        FlexDirection::Row => "row",
        FlexDirection::Column => "column",
        FlexDirection::RowReverse => "row_reverse",
        FlexDirection::ColumnReverse => "column_reverse",
    }
}

fn parse_flex_direction(s: &str) -> Option<FlexDirection> {
    Some(match s {
        "row" => FlexDirection::Row,
        "column" => FlexDirection::Column,
        "row_reverse" => FlexDirection::RowReverse,
        "column_reverse" => FlexDirection::ColumnReverse,
        _ => return None,
    })
}

fn align_items_name(a: &AlignItems) -> &'static str {
    match a {
        AlignItems::Start => "start",
        AlignItems::Center => "center",
        AlignItems::End => "end",
        AlignItems::Stretch => "stretch",
        AlignItems::Baseline => "baseline",
    }
}

fn parse_align_items(s: &str) -> Option<AlignItems> {
    Some(match s {
        "start" => AlignItems::Start,
        "center" => AlignItems::Center,
        "end" => AlignItems::End,
        "stretch" => AlignItems::Stretch,
        "baseline" => AlignItems::Baseline,
        _ => return None,
    })
}

fn justify_content_name(j: &JustifyContent) -> &'static str {
    match j {
        JustifyContent::Start => "start",
        JustifyContent::Center => "center",
        JustifyContent::End => "end",
        JustifyContent::SpaceBetween => "space_between",
        JustifyContent::SpaceAround => "space_around",
        JustifyContent::SpaceEvenly => "space_evenly",
    }
}

fn parse_justify_content(s: &str) -> Option<JustifyContent> {
    Some(match s {
        "start" => JustifyContent::Start,
        "center" => JustifyContent::Center,
        "end" => JustifyContent::End,
        "space_between" => JustifyContent::SpaceBetween,
        "space_around" => JustifyContent::SpaceAround,
        "space_evenly" => JustifyContent::SpaceEvenly,
        _ => return None,
    })
}

fn cursor_name(c: &Cursor) -> &'static str {
    match c {
        Cursor::Auto => "auto",
        Cursor::Default => "default",
        Cursor::Pointer => "pointer",
        Cursor::Wait => "wait",
        Cursor::Text => "text",
        Cursor::Move => "move",
        Cursor::Help => "help",
        Cursor::NotAllowed => "not_allowed",
        Cursor::None => "none",
        Cursor::ContextMenu => "context_menu",
        Cursor::Progress => "progress",
        Cursor::Cell => "cell",
        Cursor::Crosshair => "crosshair",
        Cursor::VerticalText => "vertical_text",
        Cursor::Alias => "alias",
        Cursor::Copy => "copy",
        Cursor::NoDrop => "no_drop",
        Cursor::Grab => "grab",
        Cursor::Grabbing => "grabbing",
        Cursor::AllScroll => "all_scroll",
        Cursor::ColResize => "col_resize",
        Cursor::RowResize => "row_resize",
        Cursor::NResize => "n_resize",
        Cursor::EResize => "e_resize",
        Cursor::SResize => "s_resize",
        Cursor::WResize => "w_resize",
        Cursor::NeResize => "ne_resize",
        Cursor::NwResize => "nw_resize",
        Cursor::SeResize => "se_resize",
        Cursor::SwResize => "sw_resize",
        Cursor::EwResize => "ew_resize",
        Cursor::NsResize => "ns_resize",
        Cursor::NeswResize => "nesw_resize",
        Cursor::NwseResize => "nwse_resize",
        Cursor::ZoomIn => "zoom_in",
        Cursor::ZoomOut => "zoom_out",
    }
}

fn parse_cursor(s: &str) -> Option<Cursor> {
    Some(match s {
        "auto" => Cursor::Auto,
        "default" => Cursor::Default,
        "pointer" => Cursor::Pointer,
        "wait" => Cursor::Wait,
        "text" => Cursor::Text,
        "move" => Cursor::Move,
        "help" => Cursor::Help,
        "not_allowed" => Cursor::NotAllowed,
        "none" => Cursor::None,
        "context_menu" => Cursor::ContextMenu,
        "progress" => Cursor::Progress,
        "cell" => Cursor::Cell,
        "crosshair" => Cursor::Crosshair,
        "vertical_text" => Cursor::VerticalText,
        "alias" => Cursor::Alias,
        "copy" => Cursor::Copy,
        "no_drop" => Cursor::NoDrop,
        "grab" => Cursor::Grab,
        "grabbing" => Cursor::Grabbing,
        "all_scroll" => Cursor::AllScroll,
        "col_resize" => Cursor::ColResize,
        "row_resize" => Cursor::RowResize,
        "n_resize" => Cursor::NResize,
        "e_resize" => Cursor::EResize,
        "s_resize" => Cursor::SResize,
        "w_resize" => Cursor::WResize,
        "ne_resize" => Cursor::NeResize,
        "nw_resize" => Cursor::NwResize,
        "se_resize" => Cursor::SeResize,
        "sw_resize" => Cursor::SwResize,
        "ew_resize" => Cursor::EwResize,
        "ns_resize" => Cursor::NsResize,
        "nesw_resize" => Cursor::NeswResize,
        "nwse_resize" => Cursor::NwseResize,
        "zoom_in" => Cursor::ZoomIn,
        "zoom_out" => Cursor::ZoomOut,
        _ => return None,
    })
}

fn pointer_events_name(p: &PointerEvents) -> &'static str {
    match p {
        PointerEvents::None => "none",
        PointerEvents::Auto => "auto",
    }
}

fn parse_pointer_events(s: &str) -> Option<PointerEvents> {
    Some(match s {
        "none" => PointerEvents::None,
        "auto" => PointerEvents::Auto,
        _ => return None,
    })
}

fn position_name(p: &Position) -> &'static str {
    match p {
        Position::Static => "static",
        Position::Fixed => "fixed",
        Position::Absolute => "absolute",
        Position::Relative => "relative",
        Position::Sticky => "sticky",
    }
}

fn parse_position(s: &str) -> Option<Position> {
    Some(match s {
        "static" => Position::Static,
        "fixed" => Position::Fixed,
        "absolute" => Position::Absolute,
        "relative" => Position::Relative,
        "sticky" => Position::Sticky,
        _ => return None,
    })
}

fn overflow_kind_name(o: &OverflowKind) -> &'static str {
    match o {
        OverflowKind::Visible => "visible",
        OverflowKind::Hidden => "hidden",
        OverflowKind::Clip => "clip",
        OverflowKind::Scroll => "scroll",
        OverflowKind::Auto => "auto",
    }
}

fn parse_overflow_kind(s: &str) -> Option<OverflowKind> {
    Some(match s {
        "visible" => OverflowKind::Visible,
        "hidden" => OverflowKind::Hidden,
        "clip" => OverflowKind::Clip,
        "scroll" => OverflowKind::Scroll,
        "auto" => OverflowKind::Auto,
        _ => return None,
    })
}

fn visibility_name(v: &Visibility) -> &'static str {
    match v {
        Visibility::Visible => "visible",
        Visibility::Hidden => "hidden",
    }
}

fn parse_visibility(s: &str) -> Option<Visibility> {
    Some(match s {
        "visible" => Visibility::Visible,
        "hidden" => Visibility::Hidden,
        _ => return None,
    })
}

fn border_style_name(b: &BorderStyle) -> &'static str {
    match b {
        BorderStyle::Solid => "solid",
        BorderStyle::Dashed => "dashed",
        BorderStyle::Dotted => "dotted",
        BorderStyle::Double => "double",
        BorderStyle::None => "none",
        BorderStyle::Hidden => "hidden",
    }
}

fn parse_border_style(s: &str) -> Option<BorderStyle> {
    Some(match s {
        "solid" => BorderStyle::Solid,
        "dashed" => BorderStyle::Dashed,
        "dotted" => BorderStyle::Dotted,
        "double" => BorderStyle::Double,
        "none" => BorderStyle::None,
        "hidden" => BorderStyle::Hidden,
        _ => return None,
    })
}

fn content_align_name(c: &ContentAlign) -> &'static str {
    match c {
        ContentAlign::Start => "start",
        ContentAlign::Center => "center",
        ContentAlign::End => "end",
        ContentAlign::SpaceBetween => "space_between",
        ContentAlign::SpaceAround => "space_around",
        ContentAlign::SpaceEvenly => "space_evenly",
        ContentAlign::Baseline => "baseline",
        ContentAlign::Stretch => "stretch",
        ContentAlign::Normal => "normal",
    }
}

fn parse_content_align(s: &str) -> Option<ContentAlign> {
    Some(match s {
        "start" => ContentAlign::Start,
        "center" => ContentAlign::Center,
        "end" => ContentAlign::End,
        "space_between" => ContentAlign::SpaceBetween,
        "space_around" => ContentAlign::SpaceAround,
        "space_evenly" => ContentAlign::SpaceEvenly,
        "baseline" => ContentAlign::Baseline,
        "stretch" => ContentAlign::Stretch,
        "normal" => ContentAlign::Normal,
        _ => return None,
    })
}

fn transition_set_name(t: TransitionSet) -> &'static str {
    match t {
        TransitionSet::Default => "default",
        TransitionSet::All => "all",
        TransitionSet::Colors => "colors",
        TransitionSet::Opacity => "opacity",
        TransitionSet::Shadow => "shadow",
        TransitionSet::Transform => "transform",
        TransitionSet::None => "none",
    }
}

fn parse_transition_set(s: &str) -> Option<TransitionSet> {
    Some(match s {
        "default" => TransitionSet::Default,
        "all" => TransitionSet::All,
        "colors" => TransitionSet::Colors,
        "opacity" => TransitionSet::Opacity,
        "shadow" => TransitionSet::Shadow,
        "transform" => TransitionSet::Transform,
        "none" => TransitionSet::None,
        _ => return None,
    })
}

fn ease_name(e: &Ease) -> &'static str {
    match e {
        Ease::Linear => "linear",
        Ease::In => "in",
        Ease::Out => "out",
        Ease::InOut => "in_out",
    }
}

fn parse_ease(s: &str) -> Option<Ease> {
    Some(match s {
        "linear" => Ease::Linear,
        "in" => Ease::In,
        "out" => Ease::Out,
        "in_out" => Ease::InOut,
        _ => return None,
    })
}

fn text_property_name(p: TextProperty) -> &'static str {
    match p {
        TextProperty::FontSize => "font_size",
        TextProperty::LineHeight => "line_height",
        TextProperty::FontWeight => "font_weight",
        TextProperty::LetterSpacing => "letter_spacing",
        TextProperty::Color => "color",
        TextProperty::UnderlineOffset => "underline_offset",
        TextProperty::Whitespace => "whitespace",
        TextProperty::TextAlign => "text_align",
        TextProperty::Select => "select",
        TextProperty::FontStyle => "font_style",
        TextProperty::TextTransform => "text_transform",
        TextProperty::TextDecoration => "text_decoration",
        TextProperty::FontFamily => "font_family",
        TextProperty::TextOverflow => "text_overflow",
        TextProperty::LineClamp => "line_clamp",
        TextProperty::FontVariantNumeric => "font_variant_numeric",
    }
}

fn parse_text_property(s: &str) -> Option<TextProperty> {
    Some(match s {
        "font_size" => TextProperty::FontSize,
        "line_height" => TextProperty::LineHeight,
        "font_weight" => TextProperty::FontWeight,
        "letter_spacing" => TextProperty::LetterSpacing,
        "color" => TextProperty::Color,
        "underline_offset" => TextProperty::UnderlineOffset,
        "whitespace" => TextProperty::Whitespace,
        "text_align" => TextProperty::TextAlign,
        "select" => TextProperty::Select,
        "font_style" => TextProperty::FontStyle,
        "text_transform" => TextProperty::TextTransform,
        "text_decoration" => TextProperty::TextDecoration,
        "font_family" => TextProperty::FontFamily,
        "text_overflow" => TextProperty::TextOverflow,
        "line_clamp" => TextProperty::LineClamp,
        "font_variant_numeric" => TextProperty::FontVariantNumeric,
        _ => return None,
    })
}

/// The closed set of `TextValue::Keyword` strings the resolver produces
/// (verified against `resolve.rs`); the reader maps back through this table
/// so `&'static str` stays sound.
const KEYWORDS: &[&str] = &[
    "left",
    "center",
    "right",
    "justify",
    "nowrap",
    "pre",
    "pre-line",
    "pre-wrap",
    "normal",
    "select-none",
    "select-text",
    "select-all",
    "select-auto",
    "sans",
    "serif",
    "mono",
    "italic",
    "uppercase",
    "lowercase",
    "capitalize",
    "none",
    "underline",
    "overline",
    "line-through",
    "ellipsis",
    "clip",
    "tabular-nums",
    "ordinal",
];

fn keyword_static(s: &str) -> Option<&'static str> {
    KEYWORDS.iter().copied().find(|k| *k == s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_parser_shapes() {
        assert_eq!(parse_json("null").unwrap(), Json::Null);
        assert_eq!(parse_json("true").unwrap(), Json::Bool(true));
        assert_eq!(parse_json("-12.5e2").unwrap(), Json::Num(-1250.0));
        assert_eq!(parse_json(r#""a\né""#).unwrap(), Json::Str("a\né".into()));
        // \u escapes incl. surrogate pairs.
        assert_eq!(
            parse_json(r#""😀""#).unwrap(),
            Json::Str("\u{1F600}".into())
        );
        assert!(parse_json("[1, 2, [3]]").is_ok());
        assert!(parse_json(r#"{"a":1,"b":{"c":null}}"#).is_ok());
    }

    #[test]
    fn json_parser_rejects_garbage() {
        for bad in [
            "",
            "{",
            "}",
            "[1,]",
            "{\"a\":}",
            "{\"a\" 1}",
            "01",
            "1.",
            "1e",
            "\"\\q\"",
            "\"\\u00\"",
            "tru",
            "[1] extra",
            "NaN",
            "'str'",
            "{\"a\":1,\"a\":2}",
        ] {
            assert!(parse_json(bad).is_err(), "expected error for {bad:?}");
        }
    }

    #[test]
    fn json_depth_limit() {
        let deep = format!("{}{}", "[".repeat(80), "]".repeat(80));
        assert_eq!(parse_json(&deep).unwrap_err(), SnapshotError::DepthLimit);
    }

    #[test]
    fn writer_float_formats() {
        let mut w = W::new();
        w.f32(0.9);
        assert_eq!(w.out, "0.9");
        let mut w = W::new();
        w.f32(f32::NAN);
        assert_eq!(w.out, "\"NaN\"");
        let mut w = W::new();
        w.f32(f32::NEG_INFINITY);
        assert_eq!(w.out, "\"-inf\"");
    }

    #[test]
    fn non_finite_round_trip() {
        // A hand-built plan with non-finite angles must survive the artifact
        // round-trip (the writer is total, never panics).
        let mut plan = StylePlan::default();
        plan.base.extras.transform = Some(Transform {
            rotate_deg: Some(f32::NAN),
            ..Default::default()
        });
        let json = plan.to_snapshot_json();
        let back = StylePlan::from_snapshot_json(&json).unwrap();
        assert!(
            back.base
                .extras
                .transform
                .unwrap()
                .rotate_deg
                .unwrap()
                .is_nan()
        );
        assert_eq!(back.to_snapshot_json(), json);
    }

    #[test]
    fn header_errors_are_precise() {
        let e = StylePlan::from_snapshot_json("{\"format\":\"other\",\"version\":1}").unwrap_err();
        assert_eq!(
            e.to_string(),
            "snapshot format is \"other\", expected \"aeolus-plan\""
        );
        let e = StylePlan::from_snapshot_json("{\"format\":\"aeolus-plan\",\"version\":7}")
            .unwrap_err();
        assert_eq!(
            e.to_string(),
            "snapshot schema version 7 is not supported (this build reads version 2)"
        );
    }

    #[test]
    fn unknown_field_paths_are_qualified() {
        let mut plan = StylePlan::default();
        plan.base.padding = Some(Padding {
            top: 4.0,
            right: 4.0,
            bottom: 4.0,
            left: 4.0,
        });
        let json = plan.to_snapshot_json();
        // Inject an unknown field at base.padding.
        let injected = json.replace(
            "\"padding\":{\"top\":4,",
            "\"padding\":{\"bogus\":1,\"top\":4,",
        );
        assert_ne!(injected, json);
        let err = StylePlan::from_snapshot_json(&injected).unwrap_err();
        assert!(
            err.to_string().contains("$.base.padding.bogus"),
            "path not qualified: {err}"
        );
    }

    #[test]
    fn escaped_strings_round_trip() {
        // group labels / animation names / tag tests are raw strings from
        // class text; they must survive escaping exactly.
        let mut plan = StylePlan::default();
        plan.base.extras.animation = Some(Animation {
            name: "quote\"back\\slash".into(),
            duration_ms: None,
            ease: None,
            infinite: true,
        });
        let json = plan.to_snapshot_json();
        assert!(json.contains("quote\\\"back\\\\slash"));
        let back = StylePlan::from_snapshot_json(&json).unwrap();
        assert_eq!(back, plan);
    }
}
