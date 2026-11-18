//! The goto-definition half's source seam: where a token's declaration
//! lives, answered by the host's own store. The engine's BUILT-IN
//! source is the riding table's own provenance (the theme pass's scan)
//! under the config's named origin URI — the config's own data, kept in
//! step by every retheme. A host whose tokens live somewhere the
//! pipeline cannot know (a virtual FS, a second pipeline's bake, a
//! LSP-side merged map) wires THIS trait instead: the store rides
//! FIRST, the built-in path the fallback — the [`ThemeLoader`]
//! precedent (the host's own statement of where a resource lives), the
//! built-in a consumer of the seam, never its definition.

use std::sync::Arc;

use aeolus_theme::{TokenOrigin, TokenTable};

use crate::{EngineDefinition, EnginePosition, EngineRange};

/// The goto-definition half's source: the definition site one token
/// name answers, in the engine's own wire-free shape. `None` — the
/// honest dark — for a token the store does not carry (never a guessed
/// span, the no-silent-failures rule applied to positions).
///
/// The bounds are the observer faces' own (`Send + Sync + 'static`):
/// the store rides the engine across the session's threads.
pub trait TokenOrigins: Send + Sync + 'static {
    /// The definition site a token name answers.
    fn definition(&self, token: &str) -> Option<EngineDefinition>;
}

/// The built-in source over a host-owned table: the table's own
/// provenance (the theme pass's scan) under the URI the host names.
/// A host that OWNS its table outside the engine — the embedding host
/// that never rethemes, or rethemes by rebuilding this store beside —
/// wires it through [`AeolusEngine::with_token_origins`](crate::AeolusEngine::with_token_origins);
/// the engine's own config path ([`EngineConfig::theme_origin`](crate::EngineConfig::theme_origin)
/// over the riding table) consults the same provenance without one.
pub struct ProvenanceOrigins {
    uri: String,
    table: Arc<TokenTable>,
}

impl ProvenanceOrigins {
    /// The source over one host-owned table: `uri` the origin URI the
    /// answers name, `table` the provenance they consult.
    pub fn new(uri: impl Into<String>, table: Arc<TokenTable>) -> Self {
        Self {
            uri: uri.into(),
            table,
        }
    }
}

impl TokenOrigins for ProvenanceOrigins {
    fn definition(&self, token: &str) -> Option<EngineDefinition> {
        let origin = self.table.origin(token)?;
        Some(EngineDefinition {
            uri: self.uri.clone(),
            range: origin_range(origin),
        })
    }
}

/// The observer faces' sharing blanket: one `Arc` handle totals several
/// sessions (the host keeps the store, the sessions borrow it).
impl<O: TokenOrigins + ?Sized> TokenOrigins for Arc<O> {
    fn definition(&self, token: &str) -> Option<EngineDefinition> {
        (**self).definition(token)
    }
}

/// One origin site onto the engine's wire-free range: the same line at
/// `character` → `character + length` (a key never spans lines).
pub(crate) fn origin_range(origin: &TokenOrigin) -> EngineRange {
    EngineRange {
        start: EnginePosition {
            line: origin.line,
            character: origin.character,
        },
        end: EnginePosition {
            line: origin.line,
            character: origin.character + origin.length,
        },
    }
}
