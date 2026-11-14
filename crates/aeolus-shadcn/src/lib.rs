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
//! Four consumption layers:
//! - **[`classes`]** — pure `&str` tables + variant enums; backend-agnostic,
//!   suitable for `cn()` merging and for the `tw!` macro.
//! - **[`program`]** — the closed-world PROGRAM TABLES (plan §3.6's surface
//!   (2), §10's "shadcn components are `const` tables"): the same verbatim
//!   strings compiled through the V1 pipeline ONCE per combo and interned —
//!   `ButtonPrograms::program(variant, size)` is a table hit, never a
//!   `String` build plus a re-resolve per call (the F6 finding, gone); the
//!   token table is the handle's (the built-in `ThemeSource::shadcn` bake
//!   or a host's own).
//! - **[`xilem`] interactive** — `button` (a real `StyledButton` widget:
//!   pointer/keyboard/accesskit activation, disabled state) and `input`
//!   (Masonry `TextInput` with the controlled-value contract) are REAL
//!   widgets wired to callbacks, styled from the class tables.
//! - **[`xilem`] / [`floem`] style-only** — badge, label, card family and
//!   alert are styled boxes around labels; they are NOT interactive and do
//!   not pretend to be.
//! - **egui/iced interactive** — the egui `button` wrapper lowers the book
//!   onto egui's `WidgetVisuals` cascade so a plain `egui::widgets::Button`
//!   restyles on hover/press by itself, and the iced `button_book` hands
//!   out the Catalog closures Iced resolves per `button::Status`/`&Theme`;
//!   the rest of the egui/iced wrappers are style-only frames and books,
//!   honestly labelled.
//! - **[`slint`] generated-code** — Slint has no runtime rule injection,
//!   so the `slint` wrappers hand out books whose
//!   `base_snippet()`/`to_states_snippet*()` emit the property bindings
//!   and `states [ … ]` blocks a `TouchArea`-driven element restyles
//!   itself from (hover/press/disabled/focus are the element's own state
//!   machinery); the emitted code is compiled by slint's own interpreter
//!   in the aeolus-slint test suite.
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
pub mod egui;
pub mod floem;
pub mod iced;
pub mod program;
pub mod slint;
pub mod xilem;

/// Re-export for callers building their own variants.
pub use aeolus_core::{Resolver, cn};
