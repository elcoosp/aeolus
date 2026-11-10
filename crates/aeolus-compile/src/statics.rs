//! The static-data emit arm (architecture plan §3.1 emit (b) — the
//! second arm of the `Target` surface): a compiled [`Program`] emits as
//! the IR's OWN static form — one `static` item over
//! [`Layer::new`](aeolus_ir::Layer::new)/[`Decl`](aeolus_ir::Decl)
//! constructors, `const`-constructible end to end, compilable anywhere
//! `aeolus-ir` is a dependency.
//!
//! ```text
//! emit_static(program) -> String
//!
//! static PROGRAM: &[aeolus_ir::Layer] = &[
//!     aeolus_ir::Layer::new(
//!         aeolus_ir::Cond::Base,
//!         &[
//!             aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingTop, val: aeolus_ir::Val::Rem(1.0) },
//!         ],
//!     ),
//! ];
//! ```
//!
//! This is the bridge between the pipeline's dynamic form (the interned
//! `&'static` slices) and the plan's STATIC form (§3.3: "the static form
//! is a `static` item"): a program lowered once can be frozen into
//! source and compiled into the consumer — the artifact the future
//! `tw!`-style macro writes out. The emitter is TOTAL over the IR and
//! the text is DETERMINISTIC (byte-stable for equal programs — the same
//! artifact contract the v0.7 plan snapshots pin):
//!
//! * every [`Val`], [`Prop`] and [`Cond`] variant has an emission arm —
//!   `Prop`/`Cond`/`Bp` render through the IR's own derived `Debug`
//!   (the compiler IS the variant-name table: a new longhand or
//!   condition cannot drift the emitter), `Val` is one arm per variant
//!   (the auditable-wall convention);
//! * the mask is NEVER emitted — the construction calls
//!   [`Layer::new`], which recomputes it, so the emitter cannot drift
//!   the mask invariant the IR documents;
//! * floats emit as valid `f32` literals (whole values keep `.0`; the
//!   shortest round-trip form otherwise), and the non-finite slots the
//!   pipeline's finite guard refuses emit as the `f32` constructors —
//!   defensive totality, never `unreachable!`;
//! * [`Val::Tok`]/[`Val::ModePair`] ride VERBATIM: both are
//!   `Copy`, `const`-constructible slots the static form is allowed to
//!   carry (the runtime table resolves them — the interpreter arm's
//!   business).
//!
//! The name `PROGRAM` is the emitter's one fixed choice — a host
//! emitting several programs renames or wraps the item. What this slice
//! deliberately is NOT: the Rust-TOKENS arm (plan §3.1 emit (a) — the
//! backend method calls are the P3 codegen crate's surface) and the
//! xtask bake that turns this output into committed source.

use aeolus_ir::{Program, Val};

/// Emit a compiled program as the IR's static form: one compilable
/// `static PROGRAM: &[aeolus_ir::Layer]` item, byte-stable for equal
/// programs, total over every `Val`/`Prop`/`Cond` variant.
///
/// The emitted text uses fully-qualified `aeolus_ir::` paths, so the
/// fragment compiles anywhere `aeolus-ir` is a dependency without
/// touching the host's imports.
///
/// # The emitted shape, compiled
///
/// The emitted shape IS the static form — this doc test compiles and
/// RUNS it, asserting the static item reconstructs a sorted program
/// whose masks the [`Layer::new`](aeolus_ir::Layer::new) derivation
/// filled in:
///
/// ```rust
/// use aeolus_ir::{Cond, Decl, Layer, Program, Prop, Val};
///
/// // The emitter's output for `compile("p-4")`, verbatim (the shape
/// // tests/statics.rs pins byte for byte):
/// static PROGRAM: &[aeolus_ir::Layer] = &[
///     aeolus_ir::Layer::new(
///         aeolus_ir::Cond::Base,
///         &[
///             aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingTop, val: aeolus_ir::Val::Rem(1.0) },
///             aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingRight, val: aeolus_ir::Val::Rem(1.0) },
///             aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingBottom, val: aeolus_ir::Val::Rem(1.0) },
///             aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingLeft, val: aeolus_ir::Val::Rem(1.0) },
///         ],
///     ),
/// ];
///
/// let program = Program::new(PROGRAM);
/// assert!(program.is_sorted(), "the static form is specificity-sorted");
/// assert_eq!(program.layers[0].mask.len(), 4, "Layer::new derived the mask");
/// let _ = Decl { prop: Prop::Top, val: Val::Px(16.0) };
/// let _ = Cond::Base;
/// ```
pub fn emit_static(program: &Program) -> String {
    use std::fmt::Write as _;
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("emit_static", layers = program.layers.len()).entered();
    // A one-allocation walk: every piece goes through `write!` into the
    // same String (the workspace's `format_push_string` rule — the
    // emitter is a build-time path, but the shape stays allocation-lean).
    let mut out = String::from("static PROGRAM: &[aeolus_ir::Layer] = &[\n");
    for layer in program.layers {
        let _ = write!(
            out,
            "    aeolus_ir::Layer::new(\n        aeolus_ir::Cond::{:?},\n        &[",
            layer.cond
        );
        out.push('\n');
        for decl in layer.decls {
            let _ = write!(
                out,
                "            aeolus_ir::Decl {{ prop: aeolus_ir::Prop::{:?}, val: {} }},",
                decl.prop,
                val_literal(decl.val)
            );
            out.push('\n');
        }
        out.push_str("        ],\n    ),\n");
    }
    out.push_str("];\n");
    out
}

/// One `Val` variant's constructor text — the auditable wall: one arm
/// per variant, none wildcarded (a new `Val` variant is a compile
/// error here, by design).
fn val_literal(val: Val) -> String {
    // Whole floats keep `.0` (`1` is not a valid f32 literal); the
    // shortest round-trip form otherwise; the non-finite slots the
    // pipeline's finite guard refuses emit as the f32 constructors —
    // defensive totality, never `unreachable!`.
    fn float(v: f32) -> String {
        if v.is_nan() {
            "f32::NAN".to_string()
        } else if v == f32::INFINITY {
            "f32::INFINITY".to_string()
        } else if v == f32::NEG_INFINITY {
            "f32::NEG_INFINITY".to_string()
        } else if v.fract() == 0.0 {
            format!("{v:.1}")
        } else {
            format!("{v}")
        }
    }
    match val {
        Val::Px(v) => format!("aeolus_ir::Val::Px({})", float(v)),
        Val::Rem(v) => format!("aeolus_ir::Val::Rem({})", float(v)),
        Val::Pct(v) => format!("aeolus_ir::Val::Pct({})", float(v)),
        Val::Num(v) => format!("aeolus_ir::Val::Num({})", float(v)),
        Val::Rgba([r, g, b, a]) => format!("aeolus_ir::Val::Rgba([{r}, {g}, {b}, {a}])"),
        Val::Tok { id, alpha } => {
            format!(
                "aeolus_ir::Val::Tok {{ id: {id}, alpha: {} }}",
                float(alpha)
            )
        }
        Val::ModePair(slot) => format!("aeolus_ir::Val::ModePair({slot})"),
        Val::Kw(slot) => format!("aeolus_ir::Val::Kw({slot})"),
    }
}

/// The `Prop` emitter rides the IR's derived `Debug` — the compiler is
/// the variant-name table (a new longhand cannot drift this emitter).
/// The pin lives in the module tests: if the derive ever renders a
/// prefix/suffix, the emitter's format must move with it, and the test
/// fails instead of emitting unparsable text.
#[cfg(test)]
mod tests {
    use aeolus_ir::Prop;

    /// `Debug` renders the bare variant name — the emitter's name table.
    #[test]
    fn prop_debug_is_the_bare_variant_name() {
        assert_eq!(format!("{:?}", Prop::PaddingTop), "PaddingTop");
        assert_eq!(format!("{:?}", Prop::Background), "Background");
    }
}
