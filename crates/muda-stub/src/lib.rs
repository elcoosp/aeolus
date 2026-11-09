//! No-op, API-compatible stand-in for the `muda` desktop-menu crate.
//!
//! WHY THIS EXISTS
//! ===============
//! floem (pinned @ 1351ffb) declares `muda = { version = "0.17.1",
//! default-features = false }` plus, on every non-wasm desktop target,
//! `muda = { ..., features = ["gtk"] }` (floem/Cargo.toml, target-specific
//! section). muda's `gtk` feature builds `gtk-sys`/`gdk-sys`, which require
//! the GTK3 **development** headers (`gdk-3.0.pc`) at build time. The AEOLUS
//! sandbox has no GTK3 dev libraries and installing system packages is not
//! permitted, so `cargo check -p floem` fails with:
//!
//!   error: failed to run custom build command for `gdk-sys v0.18.2`
//!   Package 'gdk-3.0' ... not found
//!
//! muda is the *only* floem dependency with this problem (rfd 0.17 defaults
//! to `xdg-portal` + `wayland`, both pure Rust; wgpu/winit/vello need no C
//! headers). floem only uses muda for native window menus, which the AEOLUS
//! Floem backend never touches: on Linux floem implements context menus with
//! its own Floem views (src/platform/context_menu.rs) and merely *stores*
//! muda `Menu` values.
//!
//! This crate therefore re-implements exactly the muda surface floem's code
//! paths touch (modeled on floem's own `src/platform/wasm_stubs.rs`), as
//! non-functional no-ops. It is wired in via `[patch.crates-io]` in the
//! aeolus workspace root. It MUST NOT be published or used outside the
//! AEOLUS build sandbox.
//!
//! If any floem code path actually *invokes* a menu at runtime it would
//! silently do nothing — acceptable for a style-lowering prototype whose
//! tests never open windows.

use std::fmt;

/// Error type returned by appending items. Never actually produced here.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MenuError;

pub mod accelerator {
    /// Keyboard accelerator placeholder (floem only stores `Option<Accelerator>`).
    #[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
    pub struct Accelerator;
}
pub use accelerator::Accelerator;

/// Identifier for a menu item.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct MenuId(pub String);

impl MenuId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl From<String> for MenuId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for MenuId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for MenuId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MenuId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Placeholder icon type (never constructed by floem).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Icon;

/// Native icon enum (floem never names a variant; mirrors floem's wasm stub).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NativeIcon {}

/// Light/dark menu theme (only used by floem behind `#[cfg(target_os = "windows")]`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuTheme {
    Light,
    Dark,
}

/// Trait marking types that can be appended to a [`Menu`] or [`Submenu`].
pub trait IsMenuItem {}

/// The kind of a menu item, as returned by [`Menu::items`].
#[derive(Clone, Debug)]
pub enum MenuItemKind {
    MenuItem(MenuItem),
    Check(CheckMenuItem),
    Icon(IconMenuItem),
    Submenu(Submenu),
    Predefined(PredefinedMenuItem),
}

macro_rules! item_common {
    ($ty:ident) => {
        impl $ty {
            pub fn id(&self) -> &MenuId {
                &self.id
            }

            pub fn is_enabled(&self) -> bool {
                self.enabled
            }

            pub fn text(&self) -> String {
                self.text.clone()
            }

            pub fn set_enabled(&self, _enabled: bool) {}

            pub fn set_icon(&self, _icon: Option<Icon>) {}
            pub fn set_native_icon(&self, _icon: Option<NativeIcon>) {}
        }
    };
}

/// A plain clickable menu item.
#[derive(Clone, Debug)]
pub struct MenuItem {
    id: MenuId,
    text: String,
    enabled: bool,
}

impl MenuItem {
    pub fn with_id(
        id: impl Into<MenuId>,
        text: impl AsRef<str>,
        enabled: bool,
        _accelerator: Option<Accelerator>,
    ) -> Self {
        Self {
            id: id.into(),
            text: text.as_ref().to_string(),
            enabled,
        }
    }
}
item_common!(MenuItem);
impl IsMenuItem for MenuItem {}

/// A checkable menu item.
#[derive(Clone, Debug)]
pub struct CheckMenuItem {
    id: MenuId,
    text: String,
    enabled: bool,
    checked: bool,
}

impl CheckMenuItem {
    pub fn with_id(
        id: impl Into<MenuId>,
        text: impl AsRef<str>,
        enabled: bool,
        checked: bool,
        _accelerator: Option<Accelerator>,
    ) -> Self {
        Self {
            id: id.into(),
            text: text.as_ref().to_string(),
            enabled,
            checked,
        }
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }
}
item_common!(CheckMenuItem);
impl IsMenuItem for CheckMenuItem {}

/// A menu item with an icon.
#[derive(Clone, Debug)]
pub struct IconMenuItem {
    id: MenuId,
    text: String,
    enabled: bool,
}

impl IconMenuItem {
    pub fn with_id(
        id: impl Into<MenuId>,
        text: impl AsRef<str>,
        enabled: bool,
        _icon: Option<Icon>,
        _accelerator: Option<Accelerator>,
    ) -> Self {
        Self {
            id: id.into(),
            text: text.as_ref().to_string(),
            enabled,
        }
    }
}
item_common!(IconMenuItem);
impl IsMenuItem for IconMenuItem {}

/// A nested submenu.
#[derive(Clone, Debug)]
pub struct Submenu {
    id: MenuId,
    text: String,
    enabled: bool,
    items: Vec<MenuItemKind>,
}

impl Submenu {
    pub fn new(text: impl AsRef<str>, enabled: bool) -> Self {
        Self {
            id: MenuId::new(String::new()),
            text: text.as_ref().to_string(),
            enabled,
            items: Vec::new(),
        }
    }

    pub fn append(&self, _item: &dyn IsMenuItem) -> Result<(), MenuError> {
        Ok(())
    }

    pub fn items(&self) -> Vec<MenuItemKind> {
        self.items.clone()
    }
}
item_common!(Submenu);
impl IsMenuItem for Submenu {}

/// A predefined (platform-provided) item such as a separator.
#[derive(Clone, Debug)]
pub struct PredefinedMenuItem;

impl PredefinedMenuItem {
    pub fn separator() -> Self {
        Self
    }

    pub fn fullscreen(_detached: bool) -> Self {
        Self
    }
}
impl IsMenuItem for PredefinedMenuItem {}

/// Emitted menu click event (only referenced by floem behind
/// `#[cfg(any(target_os = "windows", target_os = "macos"))]`).
#[derive(Clone, Debug)]
pub struct MenuEvent {
    pub id: MenuId,
}

impl MenuEvent {
    pub fn set_event_handler<F>(_handler: Option<F>)
    where
        F: Fn(MenuEvent) + Send + 'static,
    {
        // No-op: menus never emit events in the stub.
    }
}

/// A top-level menu.
#[derive(Clone, Debug, Default)]
pub struct Menu {
    items: Vec<MenuItemKind>,
}

impl Menu {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn append(&self, _item: &dyn IsMenuItem) -> Result<(), MenuError> {
        Ok(())
    }

    pub fn append_items(&self, _items: &[&dyn IsMenuItem]) -> Result<(), MenuError> {
        Ok(())
    }

    pub fn items(&self) -> Vec<MenuItemKind> {
        self.items.clone()
    }

    pub fn set_icon(&self, _icon: Option<Icon>) {}
    pub fn set_native_icon(&self, _icon: Option<NativeIcon>) {}
    pub fn set_enabled(&self, _enabled: bool) {}
    pub fn is_enabled(&self) -> bool {
        true
    }
    pub fn text(&self) -> String {
        String::new()
    }
    pub fn id(&self) -> &MenuId {
        static DEFAULT: std::sync::OnceLock<MenuId> = std::sync::OnceLock::new();
        DEFAULT.get_or_init(|| MenuId::new(String::new()))
    }
}
