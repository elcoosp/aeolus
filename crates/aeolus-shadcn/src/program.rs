//! The shadcn PROGRAM TABLES (architecture plan §3.6's surface (2), the
//! "closed-world components — the shadcn win" row, and §10's "shadcn
//! components are `const` tables"): the verbatim v4 class strings
//! compile through the V1 pipeline ONCE per closed-world combo and
//! intern — [`ButtonPrograms::program`] is a table hit, never the
//! [`crate::classes::button_classes`] `String` build plus a re-resolve
//! per call (the F6 finding, gone).
//!
//! ```no_run
//! use aeolus_rt::{ThemeSource, TokenTable};
//! use aeolus_shadcn::classes::{ButtonSize, ButtonVariant};
//! use aeolus_shadcn::program::ButtonPrograms;
//!
//! let source = ThemeSource::shadcn().expect("the built-in shadcn source");
//! let tokens = TokenTable::baked(&source).expect("the bake");
//! let tables = ButtonPrograms::new(&tokens);
//! // view state: the handle holds the session's table and its cells
//! let program = tables.program(ButtonVariant::Default, ButtonSize::Default);
//! ```
//!
//! The handle is the [`crate::classes`] enums' compile-time twin: one
//! cell per closed-world combo (the axes are the crate's own enums, so
//! the table is bounded by construction), a hit hands back the SAME
//! interned [`StaticProgram`], a miss compiles once and fills the
//! cell. The token table is the HANDLE's (one table per session, the
//! [`aeolus_rt::ClassCell`] identity rule) — the built-in
//! [`aeolus_theme::ThemeSource::shadcn`] bake or a host's own table; the compiled
//! programs carry the `Val::Tok` slots, the backend lanes resolve
//! them.
//!
//! The interior is a [`RefCell`], not a lock — the UI thread renders
//! sequentially per window, the [`aeolus_rt::ClassCell`] precedent — so
//! the handle is `!Sync` by construction and the contract fails to
//! compile instead of waiting for a race.
//!
//! The first compile's diagnostics ride the table
//! ([`ButtonPrograms::diagnostics`]): every refusal is one of the
//! documented arbitrary-variant/attribute boundary families the
//! capability notes list — reported, never a silent drop. The base
//! parts compile their [`crate::classes`] constants VERBATIM (the
//! string surface's own granularity — a host composing
//! `SEPARATOR_BASE` with an orientation merges through `cn` and rides
//! the [`aeolus_rt::ClassCell`] lane for the composed string).

use std::cell::RefCell;

use aeolus_compile::{Config, Diagnostic, compile_theme, keyword_alignment, keyword_texts};
use aeolus_rt::StaticProgram;
use aeolus_theme::TokenTable;

use crate::classes::{
    ALERT_DESCRIPTION, ALERT_TITLE, AlertVariant, BadgeVariant, ButtonSize, ButtonVariant, CARD,
    CARD_CONTENT, CARD_DESCRIPTION, CARD_FOOTER, CARD_HEADER, CARD_TITLE, INPUT_BASE, LABEL_BASE,
    SEPARATOR_BASE, SEPARATOR_HORIZONTAL, SEPARATOR_VERTICAL, SKELETON_BASE, alert_classes,
    badge_classes, button_classes,
};

/// One interned combo: the static program plus the FIRST compile's
/// diagnostics (the honest refusals the string path reports per call;
/// the table reports them once, they do not change).
#[derive(Clone)]
struct Combo {
    program: StaticProgram,
    diagnostics: Vec<Diagnostic>,
}

/// Compile `classes` through the V1 pipeline under `tokens`, intern
/// the result into `cell` (a hit hands back the SAME program — the
/// pointer the tests pin), and carry the first compile's diagnostics.
fn ensure(cell: &RefCell<Option<Combo>>, classes: &str, tokens: &TokenTable) -> Combo {
    if let Some(combo) = cell.borrow().as_ref() {
        return combo.clone();
    }
    let config = Config::default();
    let compiled = compile_theme(classes, tokens, &config)
        .expect("the shipped v4 table compiles (the string surface's own tests pin it)");
    let annotations =
        keyword_texts(classes, &config).expect("the shipped v4 table carries keyword texts");
    let texts = keyword_alignment(&compiled, &annotations);
    let program = StaticProgram::new(
        compiled.program.layers,
        // The interned form: the compile stages leak the layers; the
        // aligned row texts leak beside them (the ClassCell's own
        // documented storage — one distinct string, one pool entry).
        Box::leak(texts.into_boxed_slice()),
    );
    let combo = Combo {
        program,
        diagnostics: compiled.diagnostics,
    };
    *cell.borrow_mut() = Some(combo.clone());
    combo
}

/// The interned program a cell already holds — the hot path: a Copy
/// out of the interned static form, never a diagnostics vector clone.
fn program_in(cell: &RefCell<Option<Combo>>) -> Option<StaticProgram> {
    cell.borrow().as_ref().map(|combo| combo.program)
}

/// The Button program table: one interned cell per closed-world
/// (variant, size) combo — the §3.6 surface (2) sketch's
/// `Button::style(Variant, Size)` shape.
pub struct ButtonPrograms<'t> {
    tokens: &'t TokenTable,
    cells: [RefCell<Option<Combo>>; ButtonVariant::COUNT * ButtonSize::COUNT],
}

impl<'t> ButtonPrograms<'t> {
    /// The table over one session's token table (the built-in
    /// [`aeolus_theme::ThemeSource::shadcn`] bake or a host's own).
    pub fn new(tokens: &'t TokenTable) -> Self {
        Self {
            tokens,
            cells: std::array::from_fn(|_| RefCell::new(None)),
        }
    }

    /// The interned program for a (variant, size) combo — a hit is the
    /// same pointer every time, a miss compiles once.
    pub fn program(&self, variant: ButtonVariant, size: ButtonSize) -> StaticProgram {
        let cell = &self.cells[variant as usize * ButtonSize::COUNT + size as usize];
        program_in(cell)
            .unwrap_or_else(|| ensure(cell, &button_classes(variant, size), self.tokens).program)
    }

    /// The FIRST compile's diagnostics for a combo — the documented
    /// refusals (the arbitrary-variant/attribute boundary families),
    /// never an unknown root. (A copy: the per-frame path is
    /// [`ButtonPrograms::program`], diagnostics are read once.)
    pub fn diagnostics(&self, variant: ButtonVariant, size: ButtonSize) -> Vec<Diagnostic> {
        let cell = &self.cells[variant as usize * ButtonSize::COUNT + size as usize];
        ensure(cell, &button_classes(variant, size), self.tokens).diagnostics
    }
}

/// The Badge program table: one interned cell per variant.
pub struct BadgePrograms<'t> {
    tokens: &'t TokenTable,
    cells: [RefCell<Option<Combo>>; BadgeVariant::COUNT],
}

impl<'t> BadgePrograms<'t> {
    /// The table over one session's token table.
    pub fn new(tokens: &'t TokenTable) -> Self {
        Self {
            tokens,
            cells: std::array::from_fn(|_| RefCell::new(None)),
        }
    }

    /// The interned program for a variant — the canonical string
    /// `badge_classes` ships.
    pub fn program(&self, variant: BadgeVariant) -> StaticProgram {
        let cell = &self.cells[variant as usize];
        program_in(cell)
            .unwrap_or_else(|| ensure(cell, &badge_classes(variant), self.tokens).program)
    }
}

/// The Alert program table: one interned cell per variant.
pub struct AlertPrograms<'t> {
    tokens: &'t TokenTable,
    cells: [RefCell<Option<Combo>>; AlertVariant::COUNT],
}

impl<'t> AlertPrograms<'t> {
    /// The table over one session's token table.
    pub fn new(tokens: &'t TokenTable) -> Self {
        Self {
            tokens,
            cells: std::array::from_fn(|_| RefCell::new(None)),
        }
    }

    /// The interned program for a variant — the canonical string
    /// `alert_classes` ships.
    pub fn program(&self, variant: AlertVariant) -> StaticProgram {
        let cell = &self.cells[variant as usize];
        program_in(cell)
            .unwrap_or_else(|| ensure(cell, &alert_classes(variant), self.tokens).program)
    }
}

/// The base-only components' programs: the single-string components
/// (input, label, the card family, the separators, skeleton, the alert
/// title/description) interned the same one-cell-per-component way.
pub struct BasePrograms<'t> {
    tokens: &'t TokenTable,
    cells: [RefCell<Option<Combo>>; BasePart::COUNT],
}

/// The base-only components, as table slots (the
/// [`crate::classes`] constants' compile-time twins).
#[derive(Clone, Copy)]
#[repr(usize)]
enum BasePart {
    Input,
    Label,
    Card,
    CardHeader,
    CardTitle,
    CardDescription,
    CardContent,
    CardFooter,
    Separator,
    SeparatorHorizontal,
    SeparatorVertical,
    Skeleton,
    AlertTitle,
    AlertDescription,
}

impl BasePart {
    const COUNT: usize = Self::AlertDescription as usize + 1;

    /// The canonical class string the part compiles from — the
    /// [`crate::classes`] constant VERBATIM.
    const fn classes(self) -> &'static str {
        match self {
            Self::Input => INPUT_BASE,
            Self::Label => LABEL_BASE,
            Self::Card => CARD,
            Self::CardHeader => CARD_HEADER,
            Self::CardTitle => CARD_TITLE,
            Self::CardDescription => CARD_DESCRIPTION,
            Self::CardContent => CARD_CONTENT,
            Self::CardFooter => CARD_FOOTER,
            Self::Separator => SEPARATOR_BASE,
            Self::SeparatorHorizontal => SEPARATOR_HORIZONTAL,
            Self::SeparatorVertical => SEPARATOR_VERTICAL,
            Self::Skeleton => SKELETON_BASE,
            Self::AlertTitle => ALERT_TITLE,
            Self::AlertDescription => ALERT_DESCRIPTION,
        }
    }
}

impl<'t> BasePrograms<'t> {
    /// The table over one session's token table.
    pub fn new(tokens: &'t TokenTable) -> Self {
        Self {
            tokens,
            cells: std::array::from_fn(|_| RefCell::new(None)),
        }
    }

    /// The interned program for one base part.
    fn part(&self, part: BasePart) -> StaticProgram {
        let cell = &self.cells[part as usize];
        program_in(cell).unwrap_or_else(|| ensure(cell, part.classes(), self.tokens).program)
    }

    /// The input's interned program.
    pub fn input(&self) -> StaticProgram {
        self.part(BasePart::Input)
    }

    /// The label's interned program.
    pub fn label(&self) -> StaticProgram {
        self.part(BasePart::Label)
    }

    /// The card's interned program.
    pub fn card(&self) -> StaticProgram {
        self.part(BasePart::Card)
    }

    /// The card header's interned program.
    pub fn card_header(&self) -> StaticProgram {
        self.part(BasePart::CardHeader)
    }

    /// The card title's interned program.
    pub fn card_title(&self) -> StaticProgram {
        self.part(BasePart::CardTitle)
    }

    /// The card description's interned program.
    pub fn card_description(&self) -> StaticProgram {
        self.part(BasePart::CardDescription)
    }

    /// The card content's interned program.
    pub fn card_content(&self) -> StaticProgram {
        self.part(BasePart::CardContent)
    }

    /// The card footer's interned program.
    pub fn card_footer(&self) -> StaticProgram {
        self.part(BasePart::CardFooter)
    }

    /// The base separator's interned program.
    pub fn separator(&self) -> StaticProgram {
        self.part(BasePart::Separator)
    }

    /// The horizontal separator's interned program.
    pub fn separator_horizontal(&self) -> StaticProgram {
        self.part(BasePart::SeparatorHorizontal)
    }

    /// The vertical separator's interned program.
    pub fn separator_vertical(&self) -> StaticProgram {
        self.part(BasePart::SeparatorVertical)
    }

    /// The skeleton's interned program.
    pub fn skeleton(&self) -> StaticProgram {
        self.part(BasePart::Skeleton)
    }

    /// The alert title's interned program.
    pub fn alert_title(&self) -> StaticProgram {
        self.part(BasePart::AlertTitle)
    }

    /// The alert description's interned program.
    pub fn alert_description(&self) -> StaticProgram {
        self.part(BasePart::AlertDescription)
    }
}

// The closed-world counts the cell arrays size against — the axes are
// the crate's own enums, so the tables are bounded by construction.
impl ButtonVariant {
    /// The closed-world variant count (the cell array's first axis).
    const COUNT: usize = Self::Link as usize + 1;
}

impl ButtonSize {
    /// The closed-world size count (the cell array's second axis).
    const COUNT: usize = Self::IconLg as usize + 1;
}

impl BadgeVariant {
    /// The closed-world variant count (the cell array's only axis).
    const COUNT: usize = Self::Link as usize + 1;
}

impl AlertVariant {
    /// The closed-world variant count (the cell array's only axis).
    const COUNT: usize = Self::Destructive as usize + 1;
}
