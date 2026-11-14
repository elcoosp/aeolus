//! The Aeolus V1 runtime seam (architecture plan §3.2's `aeolus-rt`):
//! the small, host-facing half of the pipeline.
//!
//! ```text
//! StaticProgram ─ the static form a `tw!`-style macro emits:
//!                 the IR's own layers plus the ROW TEXTS its
//!                 row-relative `Val::Kw` slots index
//! ClassCell     ─ the dynamic-strings slow lane's view-state cell
//!                 (plan §3.6's surface (3)): one slot keyed on the
//!                 string's exact content and compile identity — the
//!                 same compiler as `tw!`, runtime-driven
//! Env           ─ the probe situation (re-exported from the interp arm)
//! TokenTable    ─ the runtime theme tier (re-exported from the theme pass)
//! DynCache      ─ the bounded dynamic-string cache (the interp/compile seam)
//! overlay       ─ the IR-level composition seam (plan §3.6's surface (2):
//!                 the frozen base under the runtime override, rightmost wins)
//! ```
//!
//! The keyword annotations are what makes a `static` program sufficient
//! at the host: `hidden` and `block` both lower to `Prop::Display` +
//! `Val::Kw(0)` (the slot is row-relative), so the bare IR cannot
//! recover the text the backend maps natively. The compile pipeline
//! already reads those texts for the Rust-tokens emit arm
//! ([`aeolus_compile::keyword_texts`]); the macro freezes them beside
//! the layers and [`StaticProgram::keyword`] reads them back — and the
//! dynamic lane aligns them at runtime through
//! [`aeolus_compile::keyword_alignment`].
//!
//! A STATIC program is built once at macro expansion and never parses
//! classes again. A DYNAMIC string goes through [`ClassCell`] (the
//! per-view slow lane — the full V1 compile on a content change, a
//! compare per frame otherwise) or the host-owned [`DynCache`]; both
//! compile through the same pipeline the macro runs, no smaller
//! matcher.

use aeolus_ir::{Layer, Program, Prop};

mod class_cell;

pub use aeolus_compile::{
    AttrBit, AttrKind, AttrProbe, AttrState, CachedCompiled, CompileError, ContainerBound,
    ContainerProbe, ContainerState, Customs, DEFAULT_DYN_CACHE_CAP, Diagnostic, DynCache, DynStore,
    Env, InterpError, InterpNote, Interped, PlanNoteKind, StructFacts, StructProbe, StyleWalk,
    TokenSource, TwoGenStore, interp, interp_containers, interp_probes, interp_probes_containers,
    interp_struct, interp_theme, interp_theme_containers, interp_theme_probes,
    interp_theme_probes_containers, interp_theme_struct, overlay, program_style,
    program_style_struct, program_style_with,
};
pub use aeolus_theme::{ThemeError, ThemeSource, TokenTable};
pub use class_cell::ClassCell;

/// The static form a `tw!`-style macro emits.
///
/// `layers` is the program's own `&'static [Layer]` and `keywords` is
/// parallel to the declarations in layer order — the row text for a
/// [`aeolus_ir::Val::Kw`] slot, or `""` for values that carry their own
/// text (numbers, colours, tokens). Parallel (rather than keyed by
/// `Prop`) is deliberate: two classes can set the same longhand to two
/// different keyword texts (`hidden flex`), and only source order
/// disambiguates them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StaticProgram {
    /// The compiled layers, specificity-sorted.
    pub layers: &'static [Layer],
    /// Row texts parallel to the flattened declarations, layer order.
    pub keywords: &'static [&'static str],
}

impl StaticProgram {
    /// Build a static program from the macro's two slices.
    pub const fn new(layers: &'static [Layer], keywords: &'static [&'static str]) -> Self {
        Self { layers, keywords }
    }

    /// The IR program view of this static form.
    pub const fn program(&self) -> Program {
        Program::new(self.layers)
    }

    /// The row text for the declaration at `(layer, decl)`, if any.
    /// The annotation slice is flattened in the same layer-major order
    /// the macro emits it.
    pub fn keyword(&self, layer: usize, decl: usize) -> Option<&'static str> {
        let mut index = 0usize;
        for (li, l) in self.layers.iter().enumerate() {
            if li == layer {
                let text = *self.keywords.get(index + decl)?;
                return (!text.is_empty()).then_some(text);
            }
            index += l.decls.len();
        }
        None
    }

    /// The row text for one property in a layer whose value is a
    /// keyword — the ergonomic lookup a backend applies per layer.
    /// Returns the LAST matching declaration (the cascade's winner).
    pub fn keyword_for(&self, layer: usize, prop: Prop) -> Option<&'static str> {
        let mut index = 0usize;
        for (li, l) in self.layers.iter().enumerate() {
            if li == layer {
                let mut found = None;
                for (di, decl) in l.decls.iter().enumerate() {
                    if decl.prop == prop {
                        let text = *self.keywords.get(index + di)?;
                        if !text.is_empty() {
                            found = Some(text);
                        }
                    }
                }
                return found;
            }
            index += l.decls.len();
        }
        None
    }
}

/// Parse a CSS-first theme file at the host — the `theme!` macro's
/// runtime half (T2, the explicit slow lane the plan allows for
/// user-editable themes). The file text is the SAME `@theme`/`:root`/
/// `.dark` CSS a web Tailwind v4 project uses.
///
/// # Errors
/// [`ThemeError`] when either block does not parse.
pub fn theme_from_css(light_css: &str, dark_css: Option<&str>) -> Result<ThemeSource, ThemeError> {
    ThemeSource::from_css(light_css, dark_css)
}
