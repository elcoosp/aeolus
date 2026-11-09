//! The Aeolus V1 longhand IR (architecture plan §3.3) — the heart of the
//! zero-runtime revamp.
//!
//! Three moving parts, all `Copy`, all `const`-constructible, none of them
//! touching the heap in the static form:
//!
//! * [`Prop`] — the LONGHAND list. `#[repr(u16)]`; the discriminant order
//!   IS the cascade/sort order, so "sort declarations" is a integer
//!   compare and nothing else.
//! * [`PropMask`] — one `u128` bitset over the longhands. The per-edge
//!   set-mask model that v0.8 shipped for padding/margin/border/radius
//!   generalizes here to the whole surface: `p-4 ⊕ px-8` and
//!   `hover:px-8` dominance are bit operations, not table walks.
//! * [`Val`] — the 12-byte value: baked lengths/colours for the static
//!   form, token references for the runtime form (theme tiers, plan §3.4).
//!
//! [`Decl`]/[`Layer`]/[`Program`] (module [`program`]) assemble those into
//! the compile output: layer 0 is the base rule, the rest are sorted by
//! condition specificity — the exact shape `aeolus-codegen` emits and the
//! backends interpret.
//!
//! The v0.x `StylePlan` stays the migration oracle (plan §7): P2's
//! `aeolus-compile` proves `Program == StylePlan` on the golden corpus
//! before any backend flips over.

pub mod cond;
pub mod program;
pub mod prop;
pub mod val;

pub use cond::{Bp, Cond};
pub use program::{Decl, Layer, Program};
pub use prop::{Prop, PropMask};
pub use val::Val;

/// The IR's semantic version, surfaced for diagnostics and the P2
/// pipeline's cross-crate assertions.
pub const VERSION: &str = "0.1.0";

#[cfg(test)]
mod tests {
    use super::*;

    /// The static form must never touch the heap: every IR type is `Copy`
    /// and the value type fits the plan's 12-byte budget (§3.3).
    #[test]
    fn ir_types_are_copy_and_sized() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<Prop>();
        assert_copy::<PropMask>();
        assert_copy::<Val>();
        assert_copy::<Cond>();
        assert_copy::<Bp>();
        assert_copy::<Decl>();
        assert_copy::<Layer>();
        // The plan sketch budgets Val at 12 bytes (`Tok` dominates: u16 +
        // padding + f32). A regression here is a design regression.
        assert_eq!(std::mem::size_of::<Val>(), 12);
        assert_eq!(std::mem::size_of::<Decl>(), 16);
    }
}
