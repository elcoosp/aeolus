//! Typed lex errors with byte spans — the diagnostics surface.
//!
//! One error type, one `kind`, a span, and a stable message. The messages
//! mirror the v0.x oracle's wording (the differential harness compares
//! the failure SPLIT, not the text; the wording staying close keeps the
//! CLI's honest-degradation output familiar).

use core::fmt;

/// Which grammar rule rejected the token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LexErrorKind {
    /// An empty segment before a top-level `:` (`":p-4"`, `"hover::p-4"`).
    EmptyVariant,
    /// Nothing left after the variants/negation (`"p:"` bare bracket
    /// heads, garbage).
    EmptyHead,
    /// A double `-` inside the utility body (`"p--4"`... with the second
    /// `-` opening an empty element).
    EmptyElement,
    /// A `[`/`(` group never closed (or a `]`/`)` closed too much).
    UnbalancedGroup,
    /// The arbitrary group's brackets contain nothing (`"w-[]"`).
    EmptyArbitrary,
    /// The `[...]` group extended past the top-level `/` — impossible for
    /// a well-formed token.
    GroupCrossesSlash,
    /// Text rode along after the arbitrary group (`"w-[4px]x"`).
    TextAfterGroup,
}

impl LexErrorKind {
    /// The stable message text (mirrors the v0.x oracle's wording).
    pub const fn message(self) -> &'static str {
        match self {
            LexErrorKind::EmptyVariant => "empty variant before ':'",
            LexErrorKind::EmptyHead => "utility head is empty",
            LexErrorKind::EmptyElement => "empty utility element (double '-')",
            LexErrorKind::UnbalancedGroup => "unbalanced '[' or '(' in utility",
            LexErrorKind::EmptyArbitrary => "empty arbitrary value",
            LexErrorKind::GroupCrossesSlash => "arbitrary group crosses the '/'",
            LexErrorKind::TextAfterGroup => "unexpected text after arbitrary group",
        }
    }
}

/// A lex failure: what, and where (byte span within the TOKEN —
/// `parse_class_string` reports the token's own span alongside).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LexError {
    /// Byte offset of the offending region (inclusive).
    pub start: usize,
    /// Byte offset one past the offending region.
    pub end: usize,
    /// Which rule fired.
    pub kind: LexErrorKind,
}

impl LexError {
    /// Build an error spanning `start..end`.
    pub const fn new(start: usize, end: usize, kind: LexErrorKind) -> Self {
        Self { start, end, kind }
    }

    /// Build an error over the whole token (position-less rejections like
    /// `EmptyHead`).
    pub const fn whole(len: usize, kind: LexErrorKind) -> Self {
        Self {
            start: 0,
            end: len,
            kind,
        }
    }

    /// The stable message (the kind's wording).
    pub const fn message(&self) -> &'static str {
        self.kind.message()
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}..{})", self.kind.message(), self.start, self.end)
    }
}

impl std::error::Error for LexError {}
