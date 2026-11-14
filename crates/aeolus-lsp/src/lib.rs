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
//!   plan §3.6 capability-split rule), and answers two questions:
//!   * *what is wrong here?* — the per-location diagnostics, each
//!     carrying its stable machine code and help text, its byte-true
//!     range already translated into the LSP's UTF-16 line/character
//!     pairs ([`EngineDiagnostic`]);
//!   * *what does this class mean?* — the hover resolution keyed by
//!     the candidate under the cursor: a lowered class renders its
//!     folded condition and the CSS arm's own declaration texts; a
//!     refused class renders the typed refusal (code + help); a cursor
//!     outside any candidate string answers None ([`EngineHover`]).
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
