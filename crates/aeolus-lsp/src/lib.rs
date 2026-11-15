//! The Aeolus LSP surface (architecture plan §5.5's tooling-parity
//! line, the P5 row's "LSP hover/diagnostics"): hover and diagnostics
//! for editors, answered from the SAME pipeline the CLI runs.
//!
//! The crate splits in two halves, one dependency boundary:
//!
//! * **The engine** ([`AeolusEngine`]) is the headless core: it keeps
//!   the open documents, runs the extraction seam
//!   ([`scan_rust_strings`](aeolus_compile::scan_rust_strings)) over a
//!   document's text, walks the hits through the V1 compile walk (the
//!   same [`compile`](aeolus_compile::compile)/`compile_theme` entry
//!   points the CLI and the macros use — never a parallel matcher, the
//!   plan §3.6 capability-split rule), and answers the editor's
//!   questions:
//!   * *what is wrong here?* — the per-location diagnostics, each
//!     carrying its stable machine code and help text, its byte-true
//!     range already translated into the LSP's UTF-16 line/character
//!     pairs ([`EngineDiagnostic`]);
//!   * *what does this class mean?* — the hover resolution keyed by
//!     the candidate under the cursor: a lowered class renders its
//!     folded condition and the CSS arm's own declaration texts; a
//!     refused class renders the typed refusal (code + help); a cursor
//!     outside any candidate string answers None ([`EngineHover`]);
//!   * *what could go here?* — the completion for the fragment under
//!     the cursor: registry spellings, theme-token classes and closed
//!     variant spellings, each item drawn from one of the compiler's
//!     OWN audited tables (never a guess), in the registry's canonical
//!     order ([`EngineCompletion`]);
//!   * *what does the editor paint here?* — the colour swatches: every
//!     candidate whose program carries a colour declaration, its
//!     byte-true range and the table's resolved light-tier rgba
//!     ([`EngineColour`]);
//!   * *what does the editor highlight here?* — the semantic tokens:
//!     every candidate's compile outcome onto the wire's legend, a
//!     lowered class as `class`, a refused one as the custom `refused`
//!     type ([`EngineSemanticToken`]);
//!   * *how would the editor spell this colour?* — the colour
//!     presentations: the config's own table walked backwards, every
//!     (root, token, opacity step) spelling that re-resolves to the
//!     requested bytes ([`EngineColourPresentation`]);
//!   * *what does this class resolve to, inline?* — the inlay hints:
//!     every lowered candidate's CSS-arm declarations rendered compactly
//!     at the class's end, the refused candidates staying quiet (the
//!     diagnostics' and the legend's story) ([`EngineInlayHint`]);
//!   * *how would you spell it right?* — the quick fixes: every finding
//!     the cursor rests on carries the "did you mean" corrections the
//!     pipeline's suggestion seam folds from its own audited tables, each
//!     riding the edit that repairs it ([`EngineCodeAction`]);
//!   * *where else is this class?* — the document highlights: every
//!     occurrence of the candidate under the cursor, document order, the
//!     editor's click-to-find ([`EngineHighlight`]).
//! * **The server** ([`Session`] + [`handle_message`] + [`run_stdio`])
//!   is the wire half: a `handle`-driven JSON-RPC state machine
//!   (initialize capabilities, didOpen/didChange/didClose publish
//!   diagnostics, hover requests answer from the engine,
//!   shutdown/exit close cleanly) that the stdio transport's thin loop
//!   ([`run_stdio`]) feeds. The state machine is testable without a
//!   pipe — the tests drive `handle_message` message by message.
//!
//! The engine is pure over its document map: open → change → analyze
//! is deterministic, an unknown URI answers empty, and every position
//! conversion (bytes ↔ UTF-16 pairs) lives HERE — a host never
//! re-derives the LSP's column model.
//!
//! ```no_run
//! use aeolus_lsp::{AeolusEngine, EngineConfig};
//!
//! let mut lsp = AeolusEngine::new(EngineConfig::default());
//! lsp.open("file:///card.rs", "fn f() { let s = \"p-4 flex-col\"; }\n", 1);
//! for diagnostic in lsp.analyze("file:///card.rs") {
//!     println!("{} [{}]", diagnostic.message, diagnostic.code);
//! }
//! if let Some(hover) = lsp.hover("file:///card.rs", 0, 22) {
//!     println!("{}", hover.contents);
//! }
//! ```

use std::collections::HashMap;
use std::fmt;

mod engine;
mod server;

pub use crate::engine::{AeolusEngine, EngineConfig};
pub use crate::server::{Handled, Session, handle_message, run_stdio};

// ---------------------------------------------------------------------------
// The wire-free types (the server maps these onto lsp_types)
// ---------------------------------------------------------------------------

/// An LSP position in the protocol's own units: the zero-based line and
/// the zero-based UTF-16 code-unit column. The engine converts BOTH
/// directions (bytes ↔ pairs) so a host never re-derives the model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnginePosition {
    /// The zero-based line.
    pub line: u32,
    /// The zero-based UTF-16 code-unit column.
    pub character: u32,
}

/// A half-open byte-true range in the LSP's position units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EngineRange {
    /// The range's first position.
    pub start: EnginePosition,
    /// One past the range's last position.
    pub end: EnginePosition,
}

/// One finding the engine publishes: the pipeline's typed diagnostic
/// with its stable machine code and help text, its range already
/// translated into the LSP's UTF-16 pairs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineDiagnostic {
    /// The finding's range (the offending region, byte-true).
    pub range: EngineRange,
    /// The human message — the diagnostic's own text plus its help
    /// line (the CLI's `check` rendering, editor-shaped).
    pub message: String,
    /// The stable machine code (`aeolus::compile::*`) a linter keys on.
    pub code: String,
}

impl fmt::Display for EngineDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}]", self.message, self.code)
    }
}

/// The hover answer: the class under the cursor rendered as markdown —
/// the lowered class's folded condition and the CSS arm's own
/// declaration texts, or the typed refusal (code + help).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineHover {
    /// The markdown contents (the editor renders the fenced block).
    pub contents: String,
    /// The hovered candidate's range, when the resolution came from one
    /// (the editor highlights the class).
    pub range: Option<EngineRange>,
}

/// Which audited surface a completion item comes from — the editor's
/// rendering key (the server maps each onto its protocol kind) and the
/// source's honesty tag: every item is a spelling one of the compiler's
/// OWN tables names, never a guess.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineCompletionKind {
    /// A registry spelling (a bare root, a keyword/composite suffix, an
    /// audited default-step scale) — the row's longhands are the detail.
    Utility,
    /// A theme-token spelling on a colour root — the config's own
    /// `TokenTable` names, the table's sorted order the rank.
    Token,
    /// A closed variant spelling from the compile crate's audited
    /// [`KNOWN_VARIANTS`](aeolus_compile::KNOWN_VARIANTS) set.
    Variant,
}

/// One completion item: the spelling to insert (no variant stack — the
/// fragment's already-typed prefix rides in the editor's own word), the
/// detail line (the longhands / the token / the source family), the
/// kind, and the rank — the registry's canonical order as a stable sort
/// key (strictly increasing along the answer, never repeated).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineCompletion {
    /// The class or variant spelling (`"flex-col"`, `"bg-primary"`,
    /// `"hover"`).
    pub label: String,
    /// The detail line: a utility's longhand names, a token's custom
    /// property, a variant's source family.
    pub detail: String,
    /// The audited surface the spelling comes from.
    pub kind: EngineCompletionKind,
    /// The canonical rank (the registry's row order, the token table's
    /// sorted names, the variant set's semantic order).
    pub rank: usize,
}

/// One colour swatch the editor paints beside a class: the candidate's
/// byte-true range and the RESOLVED rgba (the table's own light-tier
/// bytes — the alpha modifier baked, the same premultiplied identity
/// the token path lowers through). The keyword slots (`transparent`,
/// `currentColor`) stay unnamed: a CSS keyword is not a swatch, the
/// honest boundary the engine documents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EngineColour {
    /// The class's range (the editor paints beside the token).
    pub range: EngineRange,
    /// The resolved light-tier rgba (alpha 255 = opaque).
    pub rgba: [u8; 4],
}

/// Which compile outcome a semantic token highlights — the wire
/// legend's own two entries, the classification the compile walk
/// answers and nothing else (no guessed shape, no parallel matcher).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineSemanticKind {
    /// A candidate that lowers cleanly — the protocol's own `class`
    /// token type (the legend's index 0).
    Lowered,
    /// A candidate the compile walk refused (a typed refusal, code +
    /// help) — the server's custom `refused` token type (the legend's
    /// index 1; an editor that does not know it renders neutral, the
    /// honest degradation).
    Refused,
}

/// One semantic token: the candidate's byte-true range and its compile
/// outcome. The answer rides in document order (the extraction seam's
/// scan order, candidates within a string left to right) — the delta
/// encoding's own precondition, the wire half keeps it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EngineSemanticToken {
    /// The candidate's range (the editor highlights the spelling).
    pub range: EngineRange,
    /// The compile outcome the legend indexes.
    pub kind: EngineSemanticKind,
}

/// One colour presentation: the spelling the config's own table
/// resolves to the requested bytes, the token it names and the opacity
/// step the spelling carries — the picker's swap writes the class, the
/// table's own round trip guarantees the colour the editor picked is
/// the colour the pipeline lowers. The surface is the completion
/// source's own: every colour-group root the registry names, one walk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineColourPresentation {
    /// The class spelling (`"bg-primary"`, `"text-primary/50"`).
    pub label: String,
    /// The table token the spelling rides (`"primary"`).
    pub token: String,
    /// The opacity step the spelling carries (the audited
    /// `OPACITY_STEPS` scale; 100 is the bare spelling, no modifier).
    pub alpha: u16,
}

/// One inline hint the editor renders after a class: the lowered
/// candidate's CSS-arm declarations, compacted to the wire's single-line
/// label, at the class's END position (the hint rides where the class's
/// spelling stops). The tooltip carries the hover's markdown (the class,
/// its folded condition, the CSS block) — the inline label is the
/// summary, the tooltip is the WHY. A refused candidate never hints: its
/// story is the diagnostics' and the semantic legend's, the honest
/// boundary the answer keeps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineInlayHint {
    /// The position the hint rides at (the class's end, UTF-16 pairs).
    pub position: EnginePosition,
    /// The compact CSS the class lowers to (the CSS arm's own
    /// declarations, `"; "`-joined — one line, never a paraphrase).
    pub label: String,
    /// The markdown tooltip (the class, its folded condition and the
    /// CSS block — the hover rendering's own shape).
    pub tooltip: String,
}

/// One quick fix the editor offers beside a finding: the finding it
/// repairs (the wire maps the diagnostic back so the editor can filter),
/// the corrected token text (the edit's new content — the variant
/// prefix and the `/modifier` tail composed back on by the pipeline's
/// suggestion seam), and the finding's range (the edit's target). Every
/// replacement is a spelling the pipeline LOWERS — the audited tables
/// are the sources, never a guessed word (the completion's own
/// one-source-of-truth rule, at the miss side).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineCodeAction {
    /// The finding the action repairs (the editor's filter key).
    pub diagnostic: EngineDiagnostic,
    /// The action's menu title (`did you mean \`bg-primary\`?` — the
    /// CLI's own phrasing, editor-shaped).
    pub title: String,
    /// The corrected token text — the `TextEdit`'s new content over
    /// the finding's range.
    pub replacement: String,
}

/// One occurrence the editor highlights: the byte-true range of a
/// candidate class whose text equals the one under the cursor (the
/// document's own occurrences, document order — the wire's
/// documentHighlight answer). The highlight is TEXTUAL, not semantic: a
/// refused class's occurrences highlight exactly like a lowered one's
/// (the editor's own click-to-find, no compile verdict riding).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EngineHighlight {
    /// The occurrence's range (the editor highlights the spelling).
    pub range: EngineRange,
}

/// One open document the engine keeps: the current text and the LSP's
/// version tag (the change-detection key the wire contract rides).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    /// The document's current text, verbatim.
    pub text: String,
    /// The LSP's version tag for the text.
    pub version: i32,
}

/// The document map — `uri → Document`. The engine's own storage; the
/// `#[doc(hidden)]` export keeps the type nameable without growing the
/// documented surface.
#[doc(hidden)]
pub type DocumentMap = HashMap<String, Document>;
