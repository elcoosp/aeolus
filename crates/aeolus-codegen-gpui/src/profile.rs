//! The §4 code-size knob: the project-level `aeolus.toml` profile and
//! the emit-face dispatch it selects.
//!
//! The plan line this module delivers — docs/AEOLUS_V1_ARCHITECTURE.md
//! §4:
//!
//! > Code-size knob in `aeolus.toml`: `profile = "speed" | "size"`
//! > (inline vs one outlined `fn` per distinct program).
//!
//! The knob maps the plan's two shapes onto the crate's two LANDED
//! emit faces, whose runtime answers are byte-identical
//! (`tests/outlined_face.rs` pins the frozen content):
//!
//! * `speed` — the INLINE reference emit
//!   ([`emit_static_program`]): the call site's direct
//!   `StaticProgram::new` construction over inline `const` items, the
//!   plan's "inline" shape and the differential emit.
//! * `size` — the OUTLINED emit ([`emit_static_program_outlined`]):
//!   one `StaticProgramBuilder::assemble` call over `static` slices —
//!   the plan's "one outlined `fn` per distinct program" shape, the
//!   ICF-friendly form §4's binary-growth row names ("`size` profile
//!   outlines identical programs (ICF-friendly)"), and the face the
//!   macro has shipped since the outlined-face perf switch
//!   (`perf(macro)` `919716b`, "to eliminate per-call-site const
//!   items").
//!
//! The DEFAULT is `Size` — the shipped face — so introducing the knob
//! changes nothing for any consumer that has no `aeolus.toml`; a
//! project opts into the inline shape explicitly. `profile = "size"`
//! names the shipped face in the knob's own vocabulary.
//!
//! # The file contract
//!
//! The macro reads the calling crate's manifest directory (the
//! `CARGO_MANIFEST_DIR` anchor the `theme:` bake rides) and joins
//! `aeolus.toml`. The parser reads the ONE key it owns and never the
//! whole file:
//!
//! * no file, or no `profile` key — [`Profile::default`];
//! * `profile = "speed"` / `profile = "size"` — the named face (the
//!   quotes are optional; the value is trimmed);
//! * lines starting with `#` (comments) and `[section]` headers are
//!   skipped; unknown keys are ignored — the file may grow other keys
//!   without breaking this one;
//! * the FIRST `profile` line answers;
//! * a `profile` key whose value is not one of the two spellings is a
//!   typed [`ProfileError::InvalidValue`] naming the path and the
//!   value — the operator named the knob, the parser never guesses;
//! * a present-but-unreadable file is a typed
//!   [`ProfileError::Unreadable`] — never a silent default.
//!
//! The macro renders both refusals as compile errors (the
//! `theme:` bake's own error path), so a mis-set knob fails the build
//! naming the file, the value and the accepted spellings.

use std::fmt;
use std::path::{Path, PathBuf};

use aeolus_ir::Program;
use proc_macro2::TokenStream;

use crate::{emit_static_program, emit_static_program_outlined};

/// The §4 code-size profile: which of the two LANDED emit faces the
/// macro expansion renders.
///
/// Runtime answers are byte-identical across the faces
/// (`tests/outlined_face.rs`); the knob selects the EXPANSION SHAPE —
/// the plan's inline-vs-outlined axis (§4's knob row, §9's bloat
/// mitigation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Profile {
    /// The inline reference emit — the call site's direct
    /// `StaticProgram::new` construction over inline `const` items
    /// (the plan's "inline" shape; the differential emit).
    Speed,
    /// The outlined emit — one `StaticProgramBuilder::assemble` call
    /// over `static` slices (the plan's "one outlined `fn` per
    /// distinct program" shape, the ICF-friendly form §4's
    /// binary-growth row names). The SHIPPED face: the macro's
    /// expansion since the outlined-face perf switch, and the knob's
    /// default.
    #[default]
    Size,
}

/// The knob's typed refusals — rendered verbatim into the macro's
/// compile error, never a silent guess or a silent default.
#[derive(Debug)]
pub enum ProfileError {
    /// A `profile` key whose value is not `"speed"` or `"size"`.
    InvalidValue {
        /// The `aeolus.toml` the value was read from.
        path: PathBuf,
        /// The value as written (trimmed, quotes stripped).
        value: String,
    },
    /// The `aeolus.toml` exists but could not be read.
    Unreadable {
        /// The `aeolus.toml` that refused.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidValue { path, value } => write!(
                f,
                "aeolus.toml at {}: profile = {:?} — the knob accepts \
                 \"speed\" (the inline reference emit) or \"size\" \
                 (the outlined emit, the shipped default)",
                path.display(),
                value
            ),
            Self::Unreadable { path, source } => {
                write!(
                    f,
                    "aeolus.toml at {} could not be read: {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for ProfileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidValue { .. } => None,
            Self::Unreadable { source, .. } => Some(source),
        }
    }
}

/// Read the manifest directory's `aeolus.toml` and resolve its
/// `profile` key (see the [module contract](self)).
///
/// A missing file answers [`Profile::default`] — the knob is opt-in.
pub fn profile_from_manifest(manifest_dir: &Path) -> Result<Profile, ProfileError> {
    let path = manifest_dir.join("aeolus.toml");
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Profile::default()),
        Err(err) => return Err(ProfileError::Unreadable { path, source: err }),
    };
    profile_from_str(&text).map_err(|value| ProfileError::InvalidValue { path, value })
}

/// Resolve the `profile` key out of an `aeolus.toml` body (the file
/// contract's parse half; [`profile_from_manifest`] is the file half).
fn profile_from_str(text: &str) -> Result<Profile, String> {
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != "profile" {
            continue;
        }
        let value = value.trim();
        let value = value
            .strip_prefix('"')
            .and_then(|rest| rest.strip_suffix('"'))
            .unwrap_or(value);
        return match value {
            "speed" => Ok(Profile::Speed),
            "size" => Ok(Profile::Size),
            other => Err(other.to_string()),
        };
    }
    Ok(Profile::default())
}

/// Emit one compiled program's static form under the project's
/// profile — the knob's dispatch over the two landed faces. The
/// `size` dispatch is [`emit_static_program_outlined`] byte for byte
/// (the shipped expansion) and the `speed` dispatch is
/// [`emit_static_program`] byte for byte (the reference emit).
pub fn emit_static_program_for(
    profile: Profile,
    program: &Program,
    keywords: &[&str],
) -> TokenStream {
    match profile {
        Profile::Speed => emit_static_program(program, keywords),
        Profile::Size => emit_static_program_outlined(program, keywords),
    }
}
