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
//!     type ([`EngineSemanticToken`]); the range-scoped form beside it
//!     ([`AeolusEngine::semantic_tokens_range`], the wire's
//!     `textDocument/semanticTokens/range`): the same walk restricted
//!     to the candidates the requested range touches — a partial
//!     selection still names the token whole, a candidate wholly
//!     outside stays out AND never compiles, the whole-document range
//!     the full answer byte for byte;
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
//!     editor's click-to-find ([`EngineHighlight`]);
//!   * *what order should this string ride?* — the formatting edits:
//!     every class string not yet in the registry's canonical order
//!     rides one edit over its content span, the class-sort seam's own
//!     canonical join as the new text — the no-op contract a
//!     format-on-save rides, an already-canonical string answering
//!     nothing ([`EngineTextEdit`]); the SELECTION-SCOPED form beside
//!     it ([`AeolusEngine::format_range`], the wire's
//!     `textDocument/rangeFormatting`): the same walk restricted to the
//!     strings the requested range touches — a partial selection still
//!     names the string (the whole string reorders, never a fragment),
//!     the whole-document range the formatting request's own answer
//!     byte for byte;
//!   * *what classes live here?* — the document symbols: every
//!     candidate class as an outline entry — the spelling the name, its
//!     byte-true range, the legend's own verdict and the pipeline's own
//!     detail line ([`EngineSymbol`]);
//!   * *where else is this spelled, workspace-wide?* — the cross-document
//!     references: the highlights' textual equality over EVERY open
//!     document, each occurrence riding its document's URI, URI order
//!     across the map (the engine's first cross-document answer)
//!     ([`EngineReference`]);
//!   * *which spellings does the workspace carry?* — the workspace
//!     symbols: the document outline generalized over EVERY open
//!     document, the query the filter (case-insensitive substring, the
//!     empty query riding every open candidate), URI order across the
//!     map ([`EngineWorkspaceSymbol`]);
//!   * *how would every document spell it?* — the cross-document
//!     rename: the references' own fold over the rename edits — the
//!     cursor's candidate's textual equality over EVERY open document,
//!     one edit per occurrence, each riding its document's URI and the
//!     new spelling, URI order across the map ([`EngineRenameEdit`]);
//!   * *what does the cursor expand to?* — the selection ranges: one
//!     parent chain per requested position, the candidate span keyed
//!     by the shared locator with the string's content span riding as
//!     its parent, the caret's zero-width range the chain-less answer
//!     outside any string ([`EngineSelectionRange`]);
//!   * *what folds here?* — the folding ranges: one fold per class
//!     string the extraction seam names — the string's own content
//!     span, document order, no compile verdict riding (the
//!     highlights' textual family; the multi-line raw-string call
//!     sites the folds the editor actually collapses)
//!     ([`EngineRange`]);
//!   * *what does the whole workspace report?* — the workspace pull:
//!     every open document's findings beside its result identity, a
//!     matching previous id the unchanged verdict the client's cache
//!     answers from — the document pull generalized over the map
//!     ([`EngineWorkspaceDiagnostic`]);
//!   * *where does this token live?* — the token goto-definition face:
//!     the cursor's token class answers the theme file's own
//!     declaration — the `--color-{name}` (or bare `--{name}`) key the
//!     theme pass's provenance scan locates (byte-true, UTF-16 columns,
//!     a comment or quoted string or `var()` reference never a
//!     declaration, a seeded token the file does not declare answering
//!     no origin), the config's named origin URI the site rides under,
//!     a wired host store ([`TokenOrigins`]) the answer that rides
//!     FIRST ([`EngineDefinition`]);
//!   * *how would every document spell the TOKEN?* — the token-rename
//!     fold: the cursor's candidate decomposed onto its colour-group
//!     token (the definition face's own walk, factored once with the
//!     token's own byte span), every candidate over EVERY open document
//!     whose decomposition names the SAME token riding one edit over
//!     the token's own span — `bg-primary` and `dark:text-primary` are
//!     different spellings and the same token, so one rename takes them
//!     together, the variant prefixes and `/opacity` tails preserved
//!     ([`EngineRenameEdit`], through [`AeolusEngine::rename_token`]).
//! * **The server** ([`Session`] + [`handle_message`] + [`run_stdio`])
//!   is the wire half: a `handle`-driven JSON-RPC state machine
//!   (initialize capabilities, didOpen/didChange/didClose publish
//!   diagnostics, hover requests answer from the engine,
//!   shutdown/exit close cleanly) that the stdio transport's thin loop
//!   ([`run_stdio`]) feeds. The state machine is testable without a
//!   pipe — the tests drive `handle_message` message by message. The
//!   transport observability face ([`TransportObserver`], wired
//!   through [`Session::with_observer`] — the loop's observed twin is
//!   [`run_stdio_observed`]) streams the machine's own per-message
//!   verdicts (the answers, the quiet drops, the publishes, the exit)
//!   to a host handle; the answers themselves stay byte-identical. The
//!   command surface rides beside the request arms:
//!   `workspace/executeCommand` answers the CLOSED command table the
//!   capabilities advertise (`aeolus.sortDocument` — the class-sort
//!   seam's editor face invoked as a command, the same edits the
//!   formatting request answers; `aeolus.renameToken` — the token-rename
//!   fold invoked as a command, the rename request's own changes-map
//!   shape over the theme TOKEN's identity; an unknown command or a
//!   malformed argument the protocol's own null).
//!
//! The engine is pure over its document map: open → change → analyze
//! is deterministic, an unknown URI answers empty, and every position
//! conversion (bytes ↔ UTF-16 pairs) lives HERE — a host never
//! re-derives the LSP's column model. The change half rides the wire's
//! INCREMENTAL sync: [`EngineTextChange`] patches the map through the
//! ranged didChange shapes ([`AeolusEngine::change_incremental`]), each
//! change spliced sequentially against the text the previous change
//! left, a range-less change the full-text replace the full-sync
//! clients keep sending.
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
mod theme_loader;
mod token_origins;
mod transport_observer;

pub use crate::engine::{AeolusEngine, EngineConfig};
// The theme lifecycle's loader seam beside the retheme answer (the
// reload handoff): the host's own statement of where the NEXT table
// comes from (`ThemeLoader`, the disk form `FileThemeLoader`), wired
// through `Session::with_theme_loader` — the watched-files wire half
// drives it, the engine's `retheme` is the answer it lands on.
pub use crate::theme_loader::{FileThemeLoader, ThemeLoader};
// The goto-definition half's source seam beside the loader (the host's
// own statement of where a TOKEN lives — the built-in the engine's
// config path consults, the override a virtual-fs host wires):
// `TokenOrigins`, the table-provenance form `ProvenanceOrigins` beside.
pub use crate::token_origins::{ProvenanceOrigins, TokenOrigins};
// The theme pass's typed error beside the config that can carry it
// (the `EngineConfig::try_default` host's own boundary, nameable
// without adding `aeolus-theme` to its manifest — the faces'
// no-manifest-add discipline).
pub use crate::server::{
    Handled, Session, handle_message, run_stdio, run_stdio_observed, run_stdio_with_theme,
};
pub use aeolus_theme::ThemeError;
// The transport observability face (the wire half's own seam): the
// state machine's per-message verdicts — the answered/quiet requests,
// the published/exit/silent notifications — wired through
// `Session::with_observer` (the loop's face beside the pool/walk
// re-exports' no-manifest-add discipline).
pub use crate::transport_observer::{
    NoTransportObserver, NotificationOutcome, RequestOutcome, TransportObserver, TransportSnapshot,
    TransportStats,
};
// The observability face the engine's pool rides (the compile crate's
// seam, nameable HERE — the rt crate's re-export precedent): a host
// consuming the LSP crate implements `DynObserver`, totals the
// session's decisions with the built-in `CacheStats`/`CacheSnapshot`
// (the eviction face's typed `EvictReason` beside), and keeps the
// `NoObserver` default — without adding `aeolus-compile` to its
// manifest.
pub use aeolus_compile::{CacheSnapshot, CacheStats, DynObserver, EvictReason, NoObserver};
// The compile-walk observability face beside the pool's: the
// stage-boundary channel (the walk the engine's answers ride), the
// same no-manifest-add discipline.
pub use aeolus_compile::{CandidateOutcome, NoWalkObserver, WalkObserver, WalkSnapshot, WalkStats};

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

/// What [`AeolusEngine::retheme`] did — the typed outcome the theme
/// lifecycle's callers key on (never prose to parse, the observer
/// faces' own rule).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RethemeOutcome {
    /// The table swapped: the pool retired ([`aeolus_compile::DynCache::clear`]
    /// — the
    /// compile crate's invalidation seam), every later answer rides
    /// the NEW table, and the wire half republishes every open
    /// document's diagnostics.
    Swapped,
    /// The candidate table equals the riding one (names, tiers and
    /// resolved colours — the no-op save's re-bake): nothing swaps,
    /// nothing republishes, the pool keeps serving — the byte-neutral
    /// no-op the differential pins.
    Unchanged,
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

/// One definition site the editor jumps to (the wire's
/// `textDocument/definition` — the token goto-definition face): the
/// theme file's own URI beside the declaration's byte-true range — the
/// `--color-{name}` (or bare `--{name}`) key the token's name rides,
/// the theme pass's own provenance scan (or a wired host store's)
/// answering the site. The range rides in the LSP's UTF-16 pairs like
/// every engine answer; the URI is the config's named origin (or the
/// host store's) — never a guessed span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineDefinition {
    /// The carrying file's URI (the wire's `Location.uri`).
    pub uri: String,
    /// The declaration's range (the editor jumps here).
    pub range: EngineRange,
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

/// One reference the workspace answers (the wire's
/// `textDocument/references` — the engine's first cross-document
/// answer): the byte-true range of a candidate class whose text equals
/// the cursor's candidate, beside the URI of the open document that
/// carries it. The identity is the highlights' own textual rule — a
/// refused class's occurrences answer exactly like a lowered one's (no
/// compile verdict riding) — and the scope is the WHOLE document map,
/// the highlights' per-document walk generalized: URI order across the
/// documents (sorted, whatever the open order was — the answer is pure
/// over the map), document order within each.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineReference {
    /// The carrying document's URI (the wire's `Location.uri`).
    pub uri: String,
    /// The occurrence's range (the editor jumps here).
    pub range: EngineRange,
}

/// One edit the formatter rides (the class-sort seam's editor face —
/// the wire's `textDocument/formatting` answer): the string content's
/// byte-true range and the sort seam's canonical join as the new text.
/// The range covers the class STRING's content span (the extractor's
/// own trimmed hit) — never the quotes around it, the editor's own
/// territory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineTextEdit {
    /// The content span the edit replaces (UTF-16 pairs, byte-true).
    pub range: EngineRange,
    /// The canonical order the sort seam answers (single-space join —
    /// the whitespace run does not survive the canonical form).
    pub new_text: String,
}

/// One outline symbol the editor's document tree rides (the wire's
/// `textDocument/documentSymbol` answer): one per candidate class the
/// extraction seam's strings carry — the spelling as the NAME, its
/// byte-true range, the semantic legend's own verdict (the same
/// classification the highlighter feeds: a clean lower, a typed
/// refusal) and the detail line the pipeline's own output carries (the
/// lowered class's compact CSS — the inlay hint's label — a refused
/// one's stable code beside). Document order: the extraction seam's
/// scan order, candidates within a string left to right.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineSymbol {
    /// The candidate's spelling (`"p-4"`, `"hover:bg-primary/90"`).
    pub name: String,
    /// The candidate's byte-true range (the editor's outline jumps
    /// here).
    pub range: EngineRange,
    /// The pipeline's own detail line: the compact CSS the class
    /// lowers to, or the refusal's stable machine code.
    pub detail: String,
    /// The compile verdict — the semantic legend's own two answers
    /// (the wire half maps each onto its protocol kind).
    pub kind: EngineSemanticKind,
}

/// One workspace symbol the engine answers (the wire's
/// `workspace/symbol` — the references fold's second consumer): the
/// document symbols' outline entry beside the URI of the open document
/// that carries it. The scope is the WHOLE document map — URI order
/// across the documents (sorted, whatever the open order was — the
/// answer is pure over the map), document order within each; the
/// query is the filter, case-insensitive substring on the spelling,
/// the empty query riding every open candidate (the protocol's own
/// shape).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineWorkspaceSymbol {
    /// The carrying document's URI (the wire's `Location.uri`).
    pub uri: String,
    /// The outline entry (the document symbols' own shape).
    pub symbol: EngineSymbol,
}

/// One rename edit the workspace answers (the wire's
/// `textDocument/rename` — the rename face's workspace upgrade): the
/// byte-true range of a candidate class whose text equals the cursor's
/// candidate, beside the URI of the open document that carries it and
/// the new spelling the edit writes. The scope is the WHOLE document
/// map — the references' own fold over the rename edits: URI order
/// across the documents (sorted, whatever the open order was — the
/// answer is pure over the map), document order within each, the new
/// spelling riding every edit. The identity is the highlights' own
/// TEXTUAL rule — a refused class's occurrences edit exactly like a
/// lowered one's (no compile verdict riding: the diagnostics channel
/// tells the truth about the NEW spelling the moment the didChange
/// lands) — and a variant-prefixed sibling (`hover:flex-col` beside
/// `flex-col`) is a DIFFERENT spelling, renamed whole.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineRenameEdit {
    /// The carrying document's URI (the wire's `changes` map key).
    pub uri: String,
    /// The occurrence's range (the editor replaces the spelling).
    pub range: EngineRange,
    /// The new spelling (every occurrence rides the same one).
    pub new_text: String,
}

/// One selection range the editor expands to (the wire's
/// `textDocument/selectionRange`): the requested position's byte-true
/// range with the range CONTAINING it riding as the parent — the
/// expand-selection walk's own chain. The engine answers the chain
/// from the SAME locator the hover, the highlights, the references and
/// the rename walk (the candidate span keyed by the cursor), the
/// class string's own content span the chain's head (the extractor's
/// trimmed hit — the quotes stay the editor's territory). A cursor
/// inside the string but between classes answers the content span
/// alone (no candidate level, no invented inner range); a cursor
/// outside any string answers the protocol's own minimal range — the
/// position's zero-width caret, no parent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineSelectionRange {
    /// The selection's range (the editor selects it).
    pub range: EngineRange,
    /// The containing range (the next expand step), the chain's head
    /// answering `None`.
    pub parent: Option<Box<EngineSelectionRange>>,
}

/// One document's workspace-pull answer (the wire's
/// `workspace/diagnostic` — the pull channel generalized over the whole
/// document map): the URI beside the document's result identity — the
/// fingerprint the client caches against — and the pull outcome the
/// identity decides (the full report recomputed, or the unchanged
/// verdict the client's cache answers from). URI order across the map
/// (the cross-document answers' own determinism contract).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineWorkspaceDiagnostic {
    /// The reporting document's URI.
    pub uri: String,
    /// The document's own version tag (the didChange lifecycle's).
    pub version: i32,
    /// The document's CURRENT result identity: a 64-bit FNV-1a over the
    /// exact text — the same text answers the same id whatever the
    /// version tag rides (the equal-content re-save the byte-neutral
    /// no-op the retheme fast-path keeps), a text change answers a new
    /// one. The findings are pure over the text, so the content is the
    /// identity.
    pub result_id: String,
    /// The pull outcome the identity decided.
    pub outcome: EngineWorkspaceOutcome,
}

/// The workspace-pull outcome one document rides: the full report (the
/// document's own findings, the document pull's answer at the
/// workspace's scope) or the unchanged verdict (a previous result id
/// the client sent matches the current one — the report rides from the
/// client's cache, nothing recomputes).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EngineWorkspaceOutcome {
    /// The report recomputes: the document's findings ride.
    Full {
        /// The document's own findings (the push channel's and the
        /// document pull's — one walk, every transport).
        diagnostics: Vec<EngineDiagnostic>,
    },
    /// The client's cache is the report: a previous result id the
    /// client sent matches the document's current one.
    Unchanged,
}

/// One open document the engine keeps: the current text and the LSP's
/// version tag (the change-detection key the wire contract rides).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Document {
    /// The document's current text, verbatim.
    pub(crate) text: String,
    /// The LSP's version tag for the text.
    pub(crate) version: i32,
}

/// One incremental content change (the wire's `textDocument/didChange`
/// ranged shape, the engine's wire-free form): the replacement text
/// over the half-open range, [`EngineTextChange::range`] `None` — the
/// full-text replace (the backward-compat shape a full-sync client
/// sends, still legal under the incremental capability).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineTextChange {
    /// The replaced half-open range, the CURRENT text's UTF-16 pairs
    /// (each change of a didChange notification applies against the
    /// text the previous change left — the protocol's own sequential
    /// application order), `None` the full-text replace.
    pub range: Option<(EnginePosition, EnginePosition)>,
    /// The replacement text (a zero-width range inserts; the whole
    /// text's range deletes).
    pub text: String,
}

/// The document map — `uri → Document`. The engine's own storage.
pub(crate) type DocumentMap = HashMap<String, Document>;
