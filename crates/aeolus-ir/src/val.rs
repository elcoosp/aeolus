//! The 12-byte value type (architecture plan §3.3).
//!
//! Every variant is `Copy`, none touch the heap: the static form is a
//! `static` item, the dynamic form is one interned `Box<[Decl]>` shared by
//! reference. `Rem` is first-class on purpose — GPUI has `rems()`, so
//! emitting rem-based values honours the window's rem size (plan §3.3:
//! "more correct than gpui-rsx's px-only default").

/// One declaration value. The discriminant is the lowering strategy:
/// baked numerics for the static theme, [`Val::Tok`] for the runtime
/// theme tiers, [`Val::Kw`] for enum keywords that backends map natively.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub enum Val {
    /// A baked pixel length (`p-4` under the `px` spacing config).
    Px(f32),
    /// A rem length (`p-4` = `1rem` under the default `tailwind` config;
    /// 1 unit = 0.25 rem, the data side's `SPACING_STEP_REM` table).
    Rem(f32),
    /// A percentage (`w-1/2` → `50%`).
    Pct(f32),
    /// A unitless number (`z-10` → `10`; `opacity-50` scales to `0.5` on
    /// the data side).
    Num(f32),
    /// A baked RGBA colour (static theme, gamut-mapped once at build
    /// time — plan §3.4 / the oklch→sRGB note in §9).
    Rgba([u8; 4]),
    /// A runtime theme token + optional opacity modifier (`bg-primary/90`):
    /// `id` indexes the `TokenTable`, `alpha` is `1.0` when no `/…`
    /// modifier rode along.
    Tok { id: u16, alpha: f32 },
    /// An index into a `(light, dark)` const pair — the static form of the
    /// `dark:` tier (plan §3.4 T1). The pair itself lives with the baked
    /// theme; the IR only carries the slot.
    ModePair(u16),
    /// An enum keyword the backend maps natively (`flex-col`, `uppercase`,
    /// `absolute`, ...). The u8 indexes the per-prop keyword table the
    /// registry row's `ValueKind::Keywords` describes; the codegen crate
    /// owns the names.
    Kw(u8),
}

impl Val {
    /// The numeric view, for validators that need to know a value is a
    /// plain number (`z-10`, `opacity-*`).
    pub const fn as_num(self) -> Option<f32> {
        match self {
            Val::Num(n) | Val::Px(n) | Val::Rem(n) | Val::Pct(n) => Some(n),
            _ => None,
        }
    }

    /// True when the value references a runtime token (the 0-alloc/0-lock
    /// contract tests key off this).
    pub const fn is_token(self) -> bool {
        matches!(self, Val::Tok { .. } | Val::ModePair(_))
    }
}
