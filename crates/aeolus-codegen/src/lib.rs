//! The backend-neutral codegen seam (architecture plan §3.2's
//! `aeolus-codegen`: `Program → TokenStream`, an `Emit` trait used by
//! the macros and by third parties).
//!
//! A target crate implements [`Emit`]: it walks the IR's layers and
//! declarations and renders the target's builder calls. Everything the
//! target cannot express comes back as [`Unsupported`] — the honest
//! failure contract applies to code generation exactly as it does to
//! the compiler (`CONTRIBUTING §2`: never a silent drop).
//!
//! The trait deliberately stays small: the IR is the contract, and a
//! backend that needs row-keyword names reads them from
//! [`aeolus_compile::keyword_texts`] (the annotation seam the P3
//! surface keys on) rather than re-deriving the registry.

use aeolus_ir::{Program, Prop};
use proc_macro2::TokenStream;

/// One longhand a target cannot express — the capability report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unsupported {
    /// The longhand the target refused.
    pub prop: Prop,
    /// The stable reason text a linter/diagnostic renders.
    pub reason: &'static str,
}

impl Unsupported {
    /// Build a report entry.
    pub const fn new(prop: Prop, reason: &'static str) -> Self {
        Self { prop, reason }
    }
}

/// The codegen target seam: one program in, one token stream out,
/// refusing per-longhand rather than per-program.
pub trait Emit {
    /// What the target renders (usually a [`TokenStream`], but a text
    /// target can produce a `String`).
    type Output;

    /// Render one compiled program.
    ///
    /// # Errors
    /// The longhands the target cannot express, in cascade order.
    fn emit(&self, program: &Program) -> Result<Self::Output, Vec<Unsupported>>;
}

/// True when this crate's token helpers can render the longhand at all.
/// Targets override this with their own capability table; the default
/// is deliberately conservative (nothing).
pub fn supports(_prop: Prop) -> bool {
    false
}

/// The fully-qualified IR path a generated fragment uses, so an emitted
/// token stream compiles wherever `aeolus-ir` is a dependency without
/// touching the host's imports.
pub fn ir_path() -> TokenStream {
    quote::quote!(::aeolus_ir)
}
