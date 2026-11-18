//! The theme lifecycle's loader seam: where a session's theme comes
//! from, and the disk form the watched-files wire half drives. The
//! engine answers over ONE table at a time ([`AeolusEngine::retheme`]
//! swaps it); a loader is the host's own statement of what the NEXT
//! table is — a file on disk (the built-in [`FileThemeLoader`]), a
//! virtualFS read, a baked string. The trait is the seam the tests and
//! examples implement inline (the [`DynStore`](aeolus_compile::DynStore)
//! precedent: the built-in is a consumer of the seam, never its
//! definition).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use aeolus_theme::{ThemeError, ThemeSource, TokenTable};

/// Where a session's next theme comes from. The wire half owns ONE
/// loader per session (the `Session::with_theme_loader` face): when
/// the watched-files notification names the loader's URI, the session
/// calls [`ThemeLoader::load`] and hands the result to the engine's
/// retheme — a `Swapped` answer republishes
/// every open document's diagnostics (URI order), an `Unchanged` one
/// (the no-op save: the re-baked table equals the riding one) answers
/// nothing, and a failed load keeps the last good table (the honest
/// degradation the pool's own faces ride).
///
/// The bounds are the observer faces' own (`Send + Sync + 'static`):
/// the loader rides the session across the stdio loop's threads.
pub trait ThemeLoader: Send + Sync + 'static {
    /// The watched URI this loader owns — the identity the
    /// `workspace/didChangeWatchedFiles` events key on. One loader,
    /// one file: a host with several theme files wires the loader
    /// that owns the session's actual table.
    fn uri(&self) -> &str;

    /// The next table. `Err` keeps the riding table untouched (the
    /// session publishes nothing — nothing changed), `Ok` rides the
    /// engine's [`crate::AeolusEngine::retheme`] contract (equal content
    /// the byte-neutral no-op).
    ///
    /// # Errors
    /// Whatever the source's own construction carries
    /// ([`ThemeError`]): the parse refusal, the bake failure — the
    /// typed boundary a disk read's `?` chain lands in.
    fn load(&self) -> Result<TokenTable, ThemeError>;
}

/// The disk form: the watched theme file's URI beside the path the
/// bytes read from. `load` reads the file FRESH every call (no cache
/// between loads — the notification IS the freshness signal; caching
/// here would re-theme from a stale body), and the read error rides
/// [`ThemeError::Source`] (the construction's own typed refusal).
///
/// ```no_run
/// use aeolus_lsp::{EngineConfig, FileThemeLoader, Session};
///
/// let session = Session::new(EngineConfig::try_default().unwrap())
///     .with_theme_loader(FileThemeLoader::new(
///         "file:///project/app.css",
///         "/project/app.css",
///     ));
/// // The editor's next app.css save re-themes the session.
/// ```
pub struct FileThemeLoader {
    uri: String,
    path: PathBuf,
}

impl FileThemeLoader {
    /// The loader for one watched theme file: `uri` the identity the
    /// watched-files events name, `path` the bytes `load` reads.
    pub fn new(uri: impl Into<String>, path: impl AsRef<Path>) -> Self {
        Self {
            uri: uri.into(),
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl ThemeLoader for FileThemeLoader {
    fn uri(&self) -> &str {
        &self.uri
    }

    fn load(&self) -> Result<TokenTable, ThemeError> {
        let css = std::fs::read_to_string(&self.path).map_err(|error| ThemeError::Source {
            message: format!("{}: {error}", self.path.display()),
        })?;
        let source = ThemeSource::from_css(&css, None)?;
        TokenTable::baked(&source)
    }
}

/// The observer faces' sharing blanket: one `Arc` handle totals several
/// sessions (the host keeps the loader, the sessions borrow it).
impl<O: ThemeLoader + ?Sized> ThemeLoader for Arc<O> {
    fn uri(&self) -> &str {
        (**self).uri()
    }

    fn load(&self) -> Result<TokenTable, ThemeError> {
        (**self).load()
    }
}
