//! # Aeolus Shadcn — v1 component set
//!
//! Class tables for the eight v1 components (Button, Input, Label, Card,
//! Badge, Separator, Skeleton, Alert), **verbatim** from shadcn/ui's official
//! `new-york-v4` registry:
//!
//! ```text
//! shadcn-ui/ui @ 4aaea19d9cff3e0908e12a8c1d504b8c5ec34610 (2026-11-02)
//! apps/v4/registry/new-york-v4/ui/{button,input,label,card,badge,separator,skeleton,alert}.tsx
//! ```
//!
//! Three consumption layers:
//! - **[`classes`]** — pure `&str` tables + variant enums; backend-agnostic,
//!   suitable for `cn()` merging and for the `tw!` macro.
//! - **[`xilem`] interactive** — `button` (a real `StyledButton` widget:
//!   pointer/keyboard/accesskit activation, disabled state) and `input`
//!   (Masonry `TextInput` with the controlled-value contract) are REAL
//!   widgets wired to callbacks, styled from the class tables.
//! - **[`xilem`] / [`floem`] style-only** — badge, label, card family and
//!   alert are styled boxes around labels; they are NOT interactive and do
//!   not pretend to be.
//!
//! ## Capability notes (honest per-briefing §3)
//!
//! The v4 class strings contain utilities beyond the native backends'
//! reach. They are *parsed*, and *reported as dropped* by the lowering
//! (never silently ignored):
//!
//! | utility family | aeolus-xilem (fallback widget) | aeolus-floem |
//! |---|---|---|
//! | `bg-*`, `text-*`, `p-*`, `rounded-*`, `shadow-xs/sm`, `border`, `border-input`, `h-*`, `w-*`, `gap-*`, `font-medium/semibold`, `leading-none` | ✅ | ✅ |
//! | `hover:bg-*`, `focus-visible:*`, `disabled:*`, `dark:*` | ✅ via `Selector` flags / fallback state resolution | ✅ via nested selector maps |
//! | `ring-*`, `outline-none`, `transition-all`, `cursor-*`, `select-none` | ⚠️ dropped (reported) | ⚠️ ring dropped; `select-none` etc. dropped in v1 |
//! | `[&_svg]:*`, `[a&]:*`, `has-[>svg]:*`, `data-[orientation=*]:*`, `aria-invalid:*`, `[.border-b]:*` arbitrary/attribute variants | ⚠️ dropped (reported) | ⚠️ dropped (reported) |
//!
//! Layout utilities (`inline-flex items-center justify-center gap-2`) are
//! consumed by the *wrappers* (centered box), not the plan's layout slot.

pub mod classes;
pub mod floem;
pub mod xilem;

/// Re-export for callers building their own variants.
pub use aeolus_core::{Resolver, cn};
