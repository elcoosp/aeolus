//! The Aeolus V1 zero-copy lexer (architecture plan §3.2) — Tailwind v4
//! class strings into borrowed candidates with spans.
//!
//! Zero dependencies, `&str` end to end: a [`Candidate`] borrows every
//! slice it reports (variants, body, arbitrary value, modifier) from the
//! input, and carries the byte [`Candidate::span`] it occupies — the
//! diagnostics surface (LSP hover, editor squiggles) hangs off that.
//!
//! The grammar mirrors the v0.x `aeolus-core::parser` exactly (it stays
//! the oracle until the registry-as-data pipeline replaces it — plan
//! §5.2); the differential tests in `tests/differential.rs` pin the two
//! implementations together over a tricky battery AND every golden-corpus
//! class. One documented divergence: the v0.x parser caps variants (8)
//! and elements (10) in stack-inline arrays; the V1 lexer has no inline
//! caps, so overflow classes that the oracle rejects as `too many …` are
//! core-only failures.
//!
//! What the lexer does NOT do: classify variant names (the typed
//! `NamedVariant` table), resolve values, or lower to the IR. The lexer
//! segments and spans; the registry (P2's compile stage) interprets.

mod candidate;
mod error;

pub use candidate::{Arb, Candidate, Form, HINTS, parse_candidate, parse_class_string};
pub use error::{LexError, LexErrorKind, LexFailure};

/// The lexer's semantic version, surfaced for the pipeline's
/// cross-crate assertions. Derived from the manifest so it cannot drift
/// from the release it shipped in (the "0.1.0" hardcode predated the
/// crate joining the versioned workspace).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::VERSION;

    /// The version pin: derived from the manifest, parseable as semver —
    /// the cross-crate assertions the P2 pipeline makes key on it.
    #[test]
    fn version_tracks_the_manifest() {
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
        let (major, rest) = VERSION.split_once('.').expect("semver major");
        assert!(!major.is_empty() && major.bytes().all(|b| b.is_ascii_digit()));
        assert!(rest.split_once('.').is_some(), "at least major.minor");
    }
}
