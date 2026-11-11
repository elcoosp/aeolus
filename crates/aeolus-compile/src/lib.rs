//! The Aeolus V1 compile seam (architecture plan §3.1/§3.2, phase P2):
//! class strings in, cascade-sorted [`Program`]s out — one pure
//! function, no global state.
//!
//! ```text
//! compile(input, &Config)                        -> Result<Compiled, CompileError>
//! compile_theme(input, &TokenTable, &Config)     -> Result<Compiled, CompileError>
//! cn(&[&Compiled, &Compiled, ...])               -> Result<Compiled, CompileError>
//! emit_css(input, &Config)                       -> Result<CssSheet,  CompileError>
//! emit_css_theme(input, &TokenTable, &Config)    -> Result<CssSheet,  CompileError>
//! emit_rust(input, &Config)                      -> Result<RustSheet, CompileError>
//! emit_rust_theme(input, &TokenTable, &Config)   -> Result<RustSheet, CompileError>
//!   ├─ 1. lex      aeolus_syntax::parse_class_string — borrowed
//!   │              candidates with byte spans; per-token failures
//!   │              degrade as TYPED LexFailures, never fail the string
//!   ├─ 2. fold     the variant stack → the single dominant Cond
//!   │              (the flat-IR model, cond.rs); the CSS arm renders
//!   │              the segments' conjunctive selectors instead
//!   ├─ 3. read     aeolus_data::registry::lookup_detailed — the typed
//!   │              registry read side (unknown root vs rejected shape)
//!   ├─ 4. lower    row + suffix → longhand Decl fan-outs (the value
//!   │              pass); with a TokenTable, colour tokens and the
//!   │              /opacity modifier lower through the theme pass —
//!   │              without one they are typed boundaries, not truncations.
//!   │              The CSS arm renders the SAME resolution into text
//!   │              (the row tables' own CSS strings, the theme tiers)
//!   └─ 5. assemble stable specificity sort → Layer masks derived →
//!                  interned as the IR's documented dynamic form
//!                  (the CSS arm: rules + at-rule grouping instead)
//! ```
//!
//! Everything a class does NOT lower to comes back in
//! [`Compiled::diagnostics`] / [`CssSheet::diagnostics`] / [`RustSheet::diagnostics`]
//! with a byte
//! span, a stable code and a help text — the no-silent-failures rule
//! (CONTRIBUTING §2) applied to the pipeline: importance, arbitrary
//! values, untyped variants, and (only where no theme pass is supplied)
//! colour tokens and modifiers are typed boundaries, never dropped
//! classes. The two walks share the lexer channel, the variant refusal
//! set, the registry read side and the value pass's boundary set —
//! `tests/css.rs` pins their diagnostics EQUAL over a battery.
//!
//! What this slice deliberately is NOT (each is a remaining P2/P3
//! slice, and the seams say so in their rustdocs): the xtask generator
//! story. The has-/data-/aria- variant folding landed 2026-11 (the
//! attribute-probe families fold onto the typed `Cond` surface — see
//! the `variant` module's docs and `tests/attr_variants.rs`). What LANDED here beside the theme
//! half, the
//! IR-level [`cn()`] merge (plan §3.2's "hosts the `cn()` engine at IR
//! level" — same-condition layers merge by `Cond` equality,
//! declarations concatenate and the per-longhand LAST occurrence wins,
//! so the overlap the merge removes is exactly
//! [`aeolus_ir::Program::base_conflicts`]'s bits on the raw cascade)
//! and the TARGET half's CSS arm — [`emit_css`]/[`emit_css_theme`]
//! render the plan §3.1 emit (d) form in-pipeline (the [`Target`] docs
//! carry the why: the IR's row-relative `Kw` slot cannot recover the
//! row's CSS text, so the emit renders where the rows still talk) — is
//! the CUSTOM-AUTHORING seam (plan §5.4's `@utility`/`@custom-variant`
//! surface): [`Customs`] register custom utilities (fixed literals,
//! the functional `integer`/`number`/`percentage`/`spacing` forms and
//! the theme-token `--value(color)` form) and custom variant aliases,
//! and [`compile_with`]/[`compile_theme_with`]/[`emit_css_with`]/
//! [`emit_css_theme_with`] lower and render them through the SAME
//! typed read side, value pass, diagnostics and emit paths as the
//! built-in surface (the `custom` module's docs carry the design) —
//! and the DYN-CACHE seam: [`DynCache`] is the content-keyed, bounded,
//! thread-safe cache over this pipeline (the leak-pool dedupe the
//! storage note below promises), with the storage swappable through
//! [`DynStore`] and the built-in [`TwoGenStore`] — and the
//! INTERPRETER arm (plan §3.1 emit (c), the "interp" half the
//! [`Target`] enum named but never armed): [`interp`]/[`interp_theme`]
//! resolve a compiled [`Program`] to its EFFECTIVE declarations under
//! one probe [`Env`] — the per-longhand cascade the IR's specificity
//! sort encodes, theme dynamism through the [`TokenSource`] seam (the
//! built-in [`TokenTable`] answers with the T2 tier's one-load-one-
//! multiply; a host source wires in through the same two questions),
//! and the honest exclusions reported as typed notes
//! ([`InterpNote`]) — the runtime half the §3.2 table's
//! "interpreter otherwise" migration line needs; and the STATIC-DATA
//! arm (plan §3.1 emit (b)): [`emit_static`] freezes a compiled
//! [`Program`] into the IR's OWN static form — one compilable,
//! byte-stable `static` item over `Layer::new` constructors, the
//! artifact the plan §3.3 "the static form is a `static` item"
//! story ships — and the ARBITRARY-VALUE UNIT GRAMMAR (the value
//! pass's last named P2 slice): the bracket/paren groups the registry
//! rows always accepted now LOWER — px/rem/%/deg units into the IR's
//! own slots, colours through the audited [`aeolus_core::Color`]
//! parser, `/opacity` through the audited premultiplied bake,
//! `var()`/`(--var)` references through the theme pass's token path,
//! the `length:`/`percentage:`/`number:`/`color:` hints constraining
//! the parse — and every value the IR cannot carry (multi-token
//! composites, the arbitrary-property form, keyword values on the
//! row-relative `Val::Kw` slots) refuses TYPED, with the sibling-row
//! class gate keeping the diagnostics quiet where another row of the
//! root takes the class (`text-[13px]` is the font-size row's alone;
//! `text-[#0f0f0f]` the colour row's). The `arbitrary` module's docs
//! carry the grammar and the boundary set; `tests/arbitrary.rs` pins
//! the contracts — and the RUST-TOKENS arm (plan §3.1 emit (a), the
//! LAST arm of the [`Target`] surface):
//! [`emit_rust`]/[`emit_rust_theme`] render one
//! [`Layer::new`](aeolus_ir::Layer::new) expression per lowered class,
//! the keyword slots resolved to their ROW text and the token slots to
//! their TABLE names — the two annotations a bare `Program` cannot
//! recover (the in-pipeline emit renders where the rows still talk,
//! the CSS arm's stated rationale), the sheet assembling into the
//! compilable static item the static-data arm names. The `rust`
//! module's docs carry the design; `tests/rust_tokens.rs` pins the
//! contracts.
//!
//! Storage note: [`Program`] holds `&'static` slices, so each compile
//! call interns its program into the leak pool the IR documents as the
//! dynamic form's storage ("one interned `Box<[Decl]>` shared by
//! reference"). P2's `DynCache` keeps the pool's growth rate bounded:
//! [`DynCache::compile`] is the content-keyed, bounded, thread-safe
//! cache over the same pipeline — a hit hands back the SAME interned
//! slices the miss pooled (pointer-identical, the tests pin the
//! addresses), so a repeated class string pools exactly once while the
//! cache holds it (see the `dyn_cache` module's docs; the built-in
//! [`TwoGenStore`] is the v0.x `PlanCache`'s two-generation boundedness
//! story, and the storage is a SEAM — [`DynStore`]).
//!
//! The registry semantics are pinned at compile time: the crate refuses
//! to build against a `REGISTRY_VERSION` other than the one its
//! lowering was audited against.

mod arbitrary;
mod css;
mod custom;
mod dyn_cache;
mod interp;
mod lower;
mod rust;
mod statics;
mod variant;

use std::fmt;

use aeolus_data::LookupError;
use aeolus_data::registry::lookup_detailed;
use aeolus_ir::{Cond, Decl, Layer, Program};
use aeolus_syntax::Candidate;
use aeolus_syntax::LexErrorKind;
use aeolus_syntax::parse_class_string;
use aeolus_theme::TokenTable;

pub use css::{
    CssAt, CssRule, CssSheet, emit_css, emit_css_theme, emit_css_theme_with, emit_css_with,
};
pub use custom::{ClassifyVariant, CustomForm, CustomUtility, CustomValue, CustomVariant, Customs};
pub use dyn_cache::{
    CachedCompiled, DEFAULT_DYN_CACHE_CAP, DynCache, DynIdentity, DynStore, TwoGenStore,
};
pub use interp::{Env, InterpError, InterpNote, Interped, TokenSource, interp, interp_theme};
pub use rust::{
    RustFragment, RustSheet, emit_rust, emit_rust_theme, emit_rust_theme_with, emit_rust_with,
};
pub use statics::emit_static;

// The lowering audits the rows' meaning; a registry semantics bump must
// re-audit it. Byte-wise const compare — `&str` equality is not a const
// trait, so the pin walks the bytes (const-safe on the MSRV).
const fn registry_version_is_pinned() -> bool {
    let v = aeolus_data::REGISTRY_VERSION.as_bytes();
    if v.len() != 1 {
        return false;
    }
    v[0] == b'1'
}
const _: () = assert!(
    registry_version_is_pinned(),
    "aeolus-compile pins the registry semantics at REGISTRY_VERSION \"1\"; \
     re-audit the value pass before bumping the registry"
);

/// The emit half of the plan §3.2 compile signature
/// (`compile(&str, &ThemeSource, &Target) -> Result<Program, Diagnostics>`):
/// what ARTIFACT the pipeline's emit stage produces from the program.
/// All four of the plan §3.1 diagram's emit forms are armed —
/// [`Target::Css`] (slice 4, rendered by [`emit_css`]/[`emit_css_theme`]),
/// [`Target::Rust`] (the Rust-tokens arm, [`emit_rust`]/[`emit_rust_theme`]),
/// [`Target::Interp`] (the interpreter, [`interp`]/[`interp_theme`]) and
/// [`Target::Static`] (the static-data form, [`emit_static`]). The
/// backend METHOD calls the Rust-tokens arm's end form implies are the
/// P3 codegen crate's surface — the arm renders the IR's own token
/// shape, annotated where only the pipeline can see.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// The CSS text form (plan §3.1 emit (d)): a rendered stylesheet per
    /// class string — [`emit_css`]/[`emit_css_theme`] are this arm's
    /// entry points.
    Css,
    /// The Rust-tokens form (plan §3.1 emit (a)): one embeddable
    /// [`Layer::new`](aeolus_ir::Layer::new) expression per lowered
    /// class, the keyword/token slots annotated with the names only the
    /// pipeline can see — [`emit_rust`]/[`emit_rust_theme`] are this
    /// arm's entry points.
    Rust,
    /// The resolved-declaration form (plan §3.1 emit (c)): the program's
    /// EFFECTIVE declarations under one probe situation and one token
    /// source — [`interp`]/[`interp_theme`] are this arm's entry points.
    Interp,
    /// The static-data form (plan §3.1 emit (b)): the program as the
    /// IR's own `static` item, byte-stable, compilable anywhere
    /// `aeolus-ir` is a dependency — [`emit_static`] is this arm's
    /// entry point.
    Static,
}

/// How spacing multiples bake (plan §3.3's `spacing = "tailwind" | "px"`
/// config knob).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spacing {
    /// `1 unit = 0.25rem`, the v4 default: values emit as
    /// [`aeolus_ir::Val::Rem`] and honour the window's rem size.
    Tailwind,
    /// gpui-rsx compat: the rem value bakes against [`Config::rem_px`]
    /// and emits as [`aeolus_ir::Val::Px`].
    Px,
}

/// The compile-time configuration (plan §3.3/§9: the spacing knob and
/// the per-backend rem size).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    /// The spacing-multiple strategy ([`Spacing::Tailwind`] by default).
    pub spacing: Spacing,
    /// The rem size the [`Spacing::Px`] strategy bakes with (16 by
    /// default, plan §9's per-backend constant).
    pub rem_px: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            spacing: Spacing::Tailwind,
            rem_px: 16.0,
        }
    }
}

/// The compile result: the interned program plus the honest per-class
/// reports. Only an empty class string hard-fails; everything else
/// degrades per class, never per string.
#[derive(Clone, Debug, PartialEq)]
pub struct Compiled {
    /// The cascade-sorted program (layer 0 the base rule, conditional
    /// layers ascending by specificity), interned as the IR's
    /// documented dynamic form.
    pub program: Program,
    /// One entry per class that did NOT lower, in source order.
    pub diagnostics: Vec<Diagnostic>,
}

impl Compiled {
    /// Resolve this compile's program to its effective declarations
    /// under one probe situation — the interpreter arm's ergonomic half
    /// ([`interp`] on [`Compiled::program`]; the theme-LESS boundary:
    /// token values refuse typed, [`interp_theme`] carries the source).
    ///
    /// # Errors
    /// [`InterpError::ThemeRequired`] when a declaration's value is a
    /// theme form and no source rides along.
    pub fn interp(&self, env: &Env) -> Result<Interped, InterpError> {
        interp(&self.program, env)
    }

    /// Resolve this compile's program to its effective declarations
    /// under one probe situation and one [`TokenSource`] — the themed
    /// half (the built-in [`TokenTable`] answers with concrete colours;
    /// a host source wires in through the seam).
    ///
    /// # Errors
    /// The source's own typed failures (see [`interp_theme`]).
    pub fn interp_theme(
        &self,
        tokens: &impl TokenSource,
        env: &Env,
    ) -> Result<Interped, InterpError> {
        interp_theme(&self.program, tokens, env)
    }

    /// Emit this compile's program as the IR's static form — the
    /// static-data arm's ergonomic half ([`emit_static`] on
    /// [`Compiled::program`]): one compilable, byte-stable `static`
    /// item.
    pub fn emit_static(&self) -> String {
        emit_static(&self.program)
    }
}

/// Why one class did not lower, with the machine-readable code and the
/// help text a linter or editor renders. These are REPORTS, not errors:
/// the string's result is still a program (only an empty input is
/// [`CompileError`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// The offending region's byte range inside the INPUT string (the
    /// editor-squiggle span; the lexer's spans, carried through).
    pub span: (usize, usize),
    /// The raw token text, verbatim.
    pub raw: String,
    /// What refused, and why.
    pub kind: DiagnosticKind,
}

impl Diagnostic {
    pub(crate) fn new(span: (usize, usize), raw: &str, kind: DiagnosticKind) -> Self {
        Self {
            span,
            raw: raw.to_string(),
            kind,
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (token {:?}, bytes {}..{})",
            self.kind, self.raw, self.span.0, self.span.1
        )
    }
}

/// The typed boundary set — one variant per reason a class does not
/// lower. Every variant carries a stable [`DiagnosticKind::code`] and
/// [`DiagnosticKind::help`]; the optional `miette` feature wires the
/// same strings into the structured `Diagnostic` impls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// The lexer rejected the token (`p--4`, `w-[]`): the syntax
    /// crate's typed miss, carried through with its span — the KIND is
    /// the payload (the stable message renders through
    /// [`LexErrorKind::message`]); no stringly-typed contract crosses
    /// the crate boundary.
    Lex {
        /// Which grammar rule rejected the token.
        kind: LexErrorKind,
    },
    /// No registered root prefixes the body (`not-a-utility`) — the
    /// registry's [`LookupError::UnknownRoot`], spanned.
    UnknownRoot {
        /// The offending body, verbatim.
        body: String,
    },
    /// The longest root matched but every value shape rejected the
    /// suffix (`p-red`) — [`LookupError::ShapeRejected`], spanned.
    ShapeRejected {
        /// The offending body, verbatim.
        body: String,
        /// The longest root that matched.
        root: &'static str,
        /// The deduplicated value-shape labels that were tried.
        tried: Vec<&'static str>,
    },
    /// The variant segment is outside the typed `Cond` surface
    /// (`[@media(print)]:`, `group-data-open:`, `has-[p:hover]:`) — the
    /// element-own and parent-relative attribute probes fold (the
    /// `variant` module's docs carry the grammar); what remains outside
    /// is the arbitrary-variant form, the structural family and the
    /// parent×attribute compositions.
    UnsupportedVariant {
        /// The segment text, verbatim.
        variant: String,
    },
    /// The v4 `!` importance flag: the IR has no slot for it yet.
    UnsupportedImportant,
    /// The `/modifier` suffix: opacity on a known token lowers through
    /// [`compile_theme`]'s TokenTable (one f32 multiply, plan §3.4);
    /// leading and `var()` modifiers stay with the codegen host.
    /// Without a table the whole modifier is this boundary.
    UnsupportedModifier {
        /// The raw modifier text (`"50"`, `"[0.37]"`).
        modifier: String,
    },
    /// The `-` form on a root whose row says `negative = false`
    /// (`-p-4`) — the registry is the legality oracle.
    UnsupportedNegation {
        /// The body WITHOUT the sign (what refused it).
        body: String,
    },
    /// The value half does not lower in this slice — the numeric,
    /// keyword and arbitrary-unit halves landed (the unit grammar's
    /// boundary set lives in the `arbitrary` module), and oversized
    /// keyword tables (colour tokens) lower through the theme pass. The
    /// reason string says which boundary fired (the composite/
    /// property/keyword/hint/table refusals, the shape-mismatch drift
    /// arm, or the non-finite guard).
    UnsupportedValue {
        /// The offending body, verbatim.
        body: String,
        /// The stable reason (one of the `lower::REASON_*` strings).
        reason: &'static str,
    },
}

impl DiagnosticKind {
    /// The stable machine code (`aeolus::compile::*`) a linter keys on —
    /// the same strings the `miette` feature exposes.
    pub fn code(&self) -> &'static str {
        match self {
            DiagnosticKind::Lex { .. } => "aeolus::compile::lex",
            DiagnosticKind::UnknownRoot { .. } => "aeolus::compile::unknown_root",
            DiagnosticKind::ShapeRejected { .. } => "aeolus::compile::shape_rejected",
            DiagnosticKind::UnsupportedVariant { .. } => "aeolus::compile::unsupported_variant",
            DiagnosticKind::UnsupportedImportant => "aeolus::compile::unsupported_important",
            DiagnosticKind::UnsupportedModifier { .. } => "aeolus::compile::unsupported_modifier",
            DiagnosticKind::UnsupportedNegation { .. } => "aeolus::compile::unsupported_negation",
            DiagnosticKind::UnsupportedValue { .. } => "aeolus::compile::unsupported_value",
        }
    }

    /// The actionable help text, stable per kind.
    pub fn help(&self) -> &'static str {
        match self {
            DiagnosticKind::Lex { .. } => {
                "the token breaks the v4 grammar; the rest of the string \
                 still compiles (honest degradation)"
            }
            DiagnosticKind::UnknownRoot { .. } => {
                "check the spelling against `registry::roots()`; the \
                 registry covers the golden corpus's utility surface, not \
                 arbitrary CSS"
            }
            DiagnosticKind::ShapeRejected { .. } => {
                "the root exists but the suffix fits none of its value \
                 shapes; a bracketed arbitrary value may be the legal \
                 spelling"
            }
            DiagnosticKind::UnsupportedVariant { .. } => {
                "the typed Cond surface covers probe states, dark, \
                 breakpoints, containers, the group/peer probes and the \
                 data-/aria-/has- attribute probes (the `variant` module's \
                 docs carry the fold's grammar); arbitrary variants, the \
                 structural family and the parent×attribute compositions \
                 stay typed refusals"
            }
            DiagnosticKind::UnsupportedImportant => {
                "the IR has no importance slot; the `!` flag rides the \
                 codegen pass (P3)"
            }
            DiagnosticKind::UnsupportedModifier { .. } => {
                "the modifier arithmetic is the theme pass: compile_theme() \
                 lowers opacity on a known token (one f32 multiply, plan \
                 §3.4); leading and var() modifiers stay with the codegen host"
            }
            DiagnosticKind::UnsupportedNegation { .. } => {
                "drop the leading `-`; the root's row carries `negative = \
                 false` (the registry is the legality oracle)"
            }
            DiagnosticKind::UnsupportedValue { .. } => {
                "the numeric, keyword and arbitrary-unit value halves lower today (px/rem/%/deg \
                 units, colours, the length:/percentage:/number:/color: hints, var() token \
                 references through the theme pass); the remaining boundaries are the multi-token \
                 composite forms (calc(), underscore joins, url/image values), the arbitrary-property \
                 form and keyword values on the row-relative keyword slots — the codegen host's surface"
            }
        }
    }
}

impl fmt::Display for DiagnosticKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticKind::Lex { kind } => f.write_str(kind.message()),
            DiagnosticKind::UnknownRoot { body } => {
                write!(f, "no registered utility root prefixes {body:?}")
            }
            DiagnosticKind::ShapeRejected { body, root, tried } => write!(
                f,
                "root {root:?} matched {body:?} but no value shape accepted \
                 the suffix (tried: {})",
                tried.join(", ")
            ),
            DiagnosticKind::UnsupportedVariant { variant } => {
                write!(f, "variant {variant:?} is outside the typed Cond surface")
            }
            DiagnosticKind::UnsupportedImportant => {
                f.write_str("the v4 important flag has no IR slot in this slice")
            }
            DiagnosticKind::UnsupportedModifier { modifier } => {
                write!(
                    f,
                    "modifier {modifier:?} lowers in the theme pass, not the compile seam"
                )
            }
            DiagnosticKind::UnsupportedNegation { body } => {
                write!(
                    f,
                    "{body:?} refuses the negated form (the row's negative flag)"
                )
            }
            DiagnosticKind::UnsupportedValue { body, reason } => {
                write!(f, "{body:?} does not lower in this slice: {reason}")
            }
        }
    }
}

/// Why the whole compile or merge failed. Per-class failures are
/// [`Diagnostic`]s; the only whole-string failures are caller bugs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompileError {
    /// The input holds no candidate at all: an empty/whitespace-only
    /// class string.
    Empty,
    /// [`cn`] was called with no programs to merge — the merge needs at
    /// least one compiled input.
    NoInputs,
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::Empty => {
                f.write_str("empty class string: compile needs at least one non-whitespace token")
            }
            CompileError::NoInputs => {
                f.write_str("no inputs: cn needs at least one compiled program to merge")
            }
        }
    }
}

impl std::error::Error for CompileError {}

#[cfg(feature = "miette")]
impl miette::Diagnostic for CompileError {
    fn code(&self) -> Option<Box<dyn fmt::Display + '_>> {
        Some(Box::new(match self {
            CompileError::Empty => "aeolus::compile::empty_input",
            CompileError::NoInputs => "aeolus::compile::cn_no_inputs",
        }))
    }

    fn help(&self) -> Option<Box<dyn fmt::Display + '_>> {
        Some(Box::new(match self {
            CompileError::Empty => {
                "a class string must contain at least one non-whitespace token; \
                 an empty string is a caller bug, not a styling result"
            }
            CompileError::NoInputs => {
                "cn merges one or more already-compiled programs; an empty \
                 input slice is a caller bug, not a styling result"
            }
        }))
    }
}

/// Compile a class string into a cascade-sorted [`Program`] plus the
/// honest per-class [`Diagnostic`]s.
///
/// The pipeline is deterministic and total on any input (plan §3.1):
/// lexing degrades per token, the registry read side types its misses,
/// the value pass types its boundaries, and the only whole-string
/// failure is the empty input. Each compile call interns its program
/// into the leak pool (see the crate docs) — one compile per distinct
/// class string is the intended call pattern.
///
/// This is the theme-less form: colour tokens and opacity modifiers
/// stay typed boundaries (their diagnostics name
/// [`compile_theme`] as the lowering path). Pass a `TokenTable` to
/// lower them.
///
/// ```no_run
/// use aeolus_compile::{compile, Config};
///
/// let compiled = compile("p-4 hover:bg-primary/90", &Config::default())?;
/// assert_eq!(compiled.program.layers.len(), 2); // base + hover
/// // Every class the seam does not lower comes back with a span,
/// // a stable code and help text — never a silent drop.
/// for diagnostic in &compiled.diagnostics {
///     println!("{diagnostic} [{}]", diagnostic.kind.code());
/// }
/// # Ok::<(), aeolus_compile::CompileError>(())
/// ```
pub fn compile(input: &str, config: &Config) -> Result<Compiled, CompileError> {
    run(input, *config, None, &Customs::new())
}

/// [`compile`] with the custom-authoring seam supplied (plan §5.4's
/// `@utility`/`@custom-variant` surface): registered custom utilities
/// lower exactly like registry rows and registered variant aliases
/// fold like built-in spellings — the registry always wins a name
/// collision, and the typed read side (unknown root vs rejected shape)
/// is byte-shared.
///
/// With EMPTY registrations every contract is byte-identical to
/// [`compile`] (the differential pins it over a battery): the seam is
/// additive, never a behaviour switch.
///
/// ```no_run
/// use aeolus_compile::{compile_with, Config, CustomForm, CustomUtility, Customs};
/// use aeolus_ir::Prop;
///
/// let customs = Customs::new().utility(CustomUtility::functional(
///     "brand-p",
///     CustomForm::Spacing,
///     &[Prop::PaddingTop, Prop::PaddingRight, Prop::PaddingBottom, Prop::PaddingLeft],
/// ));
/// let compiled = compile_with("brand-p-4", &customs, &Config::default())?;
/// assert!(compiled.diagnostics.is_empty()); // the registration lowered
/// # Ok::<(), aeolus_compile::CompileError>(())
/// ```
pub fn compile_with(
    input: &str,
    customs: &Customs,
    config: &Config,
) -> Result<Compiled, CompileError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        utilities = customs.utilities().len(),
        variants = customs.variants().len(),
        "compile_with: the custom seam is supplied"
    );
    run(input, *config, None, customs)
}

/// Compile a class string with the theme pass supplied (plan §3.2's
/// signature gaining its second half): colour tokens lower through the
/// tiered `TokenTable` (`bg-red-500` → the tier's `Val` form), and the
/// `/opacity` modifier rides colour-form values — the premultiplied
/// bake for a baked literal, the slot's alpha for a runtime token
/// (plan §3.4, the audited `mix_with_transparent` identity).
///
/// Without a table every other contract is byte-identical to
/// [`compile`]: the same diagnostics, the same spans, the same program
/// shape. A modifier that cannot ride (a non-colour lowering, or an
/// alpha on a baked dark pair) stays the typed `UnsupportedModifier`
/// boundary — never a half-applied modifier.
///
/// ```no_run
/// use aeolus_compile::{compile_theme, Config};
/// use aeolus_theme::{ThemeSource, TokenTable};
///
/// let source = ThemeSource::shadcn_dark()?;
/// let tokens = TokenTable::runtime(&source)?;
/// let compiled = compile_theme("bg-primary/90", &tokens, &Config::default())?;
/// assert!(compiled.diagnostics.is_empty()); // the token lowered
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
/// The same [`CompileError`] as [`compile`] (only the empty input).
pub fn compile_theme(
    input: &str,
    tokens: &TokenTable,
    config: &Config,
) -> Result<Compiled, CompileError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        tokens = tokens.len(),
        tier = ?tokens.tier(),
        "compile_theme: the theme pass is supplied"
    );
    run(input, *config, Some(tokens), &Customs::new())
}

/// [`compile_theme`] with the custom-authoring seam supplied: colour
/// tokens lower through the theme pass AND registered custom
/// utilities/variants lower through the same pipeline (see
/// [`compile_with`] for the seam's contract).
///
/// ```no_run
/// use aeolus_compile::{compile_theme_with, Config, CustomVariant, Customs};
/// use aeolus_ir::Cond;
/// use aeolus_theme::{ThemeSource, TokenTable};
///
/// let source = ThemeSource::shadcn_dark()?;
/// let tokens = TokenTable::runtime(&source)?;
/// let customs = Customs::new().variant(CustomVariant::new("night", Cond::Dark));
/// let compiled = compile_theme_with("night:bg-primary/90", &tokens, &customs, &Config::default())?;
/// assert!(compiled.diagnostics.is_empty()); // the alias folded, the token lowered
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn compile_theme_with(
    input: &str,
    tokens: &TokenTable,
    customs: &Customs,
    config: &Config,
) -> Result<Compiled, CompileError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        tokens = tokens.len(),
        tier = ?tokens.tier(),
        utilities = customs.utilities().len(),
        variants = customs.variants().len(),
        "compile_theme_with: the theme pass and the custom seam are supplied"
    );
    run(input, *config, Some(tokens), customs)
}

/// Merge already-compiled programs, RIGHTMOST WINS per longhand — the
/// IR-level `cn()` (plan §3.2: "hosts the `cn()` engine at IR level";
/// §3.3: "mask algebra + copy").
///
/// The mechanics are the longhand IR's own algebra, no conflict table
/// needed at this level (the rows' fan-outs ARE the groups; the audited
/// `tw_merge` table stays pinned at the string level — the CLI's `cn`
/// and `aeolus_core::cn`):
///
/// * same-condition layers merge by `Cond` equality; disjoint
///   conditions stay separate layers, re-sorted by specificity (stable,
///   left-then-right source order breaking ties);
/// * within a layer the declarations concatenate left-then-right and
///   the LAST occurrence per longhand wins — `cn(compile("p-4"),
///   compile("px-8"))` is the four-longhand cascade `p-4 px-8`
///   resolves to, and the declarations the merge removes are exactly
///   the overlap [`aeolus_ir::Program::base_conflicts`] names on the
///   raw concatenation (which stays the [`compile`] contract,
///   byte-identical to v0.11.0 — the raw cascade keeps both fan-outs;
///   the MERGE is what dedupes);
/// * diagnostics concatenate in input order with their original spans
///   (the inputs are separate strings; the merge never offsets or
///   drops a report);
/// * the result is conflict-free (`base_conflicts()` empty),
///   specificity-sorted, and interned as the IR's documented dynamic
///   form — the leak-pool dedup (`DynCache`) is the remaining P2
///   slice's job, so one `cn` per distinct composition is the intended
///   call pattern, same as [`compile`].
///
/// The merge is idempotent and associative (the RED-pinned algebra:
/// "concatenate in order, last per longhand wins" groups freely).
///
/// ```no_run
/// use aeolus_compile::{cn, compile, Config};
///
/// let base = compile("p-4 rounded-md", &Config::default())?;
/// let user = compile("px-8 rounded-full", &Config::default())?;
/// let merged = cn(&[&base, &user])?;
/// // px-8 re-sets left/right over p-4; rounded-full wins the radii.
/// assert!(merged.diagnostics.is_empty());
/// # Ok::<(), aeolus_compile::CompileError>(())
/// ```
///
/// # Errors
/// [`CompileError::NoInputs`] when the slice is empty — the merge needs
/// at least one program. Per-class reports never fail a merge (they
/// ride [`Compiled::diagnostics`]).
pub fn cn(inputs: &[&Compiled]) -> Result<Compiled, CompileError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(inputs = inputs.len(), "cn: the ir-level merge");

    if inputs.is_empty() {
        return Err(CompileError::NoInputs);
    }

    // Conditions group first-seen (left then right); declarations
    // concatenate in the same order. The specificity sort + the
    // per-longhand last-wins dedupe in the assembly step do the rest.
    let mut conds: Vec<Cond> = Vec::new();
    let mut groups: Vec<Vec<Decl>> = Vec::new();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    for input in inputs {
        diagnostics.extend(input.diagnostics.iter().cloned());
        for layer in input.program.layers {
            let idx = match conds.iter().position(|&c| c == layer.cond) {
                Some(idx) => idx,
                None => {
                    conds.push(layer.cond);
                    groups.push(Vec::new());
                    conds.len() - 1
                }
            };
            groups[idx].extend_from_slice(layer.decls);
        }
    }

    // The assembly: stable specificity sort (first-seen order breaks
    // ties), per-group cascade order (Prop's discriminant IS the sort
    // order; the stable sort keeps source order within one longhand),
    // then the merge's own contract — the LAST declaration per longhand
    // wins, so the dedupe drops exactly the shadowed earlier decls (the
    // overlap `base_conflicts()` names on the raw concatenation).
    let mut order: Vec<(Cond, Vec<Decl>)> = conds.into_iter().zip(groups).collect();
    order.sort_by_key(|(cond, _)| cond.specificity());
    let mut layers: Vec<Layer> = Vec::with_capacity(order.len());
    for (cond, mut decls) in order {
        decls.sort_by_key(|decl: &Decl| decl.prop);
        let mut merged: Vec<Decl> = Vec::with_capacity(decls.len());
        for decl in decls {
            if merged
                .last()
                .is_some_and(|last: &Decl| last.prop == decl.prop)
            {
                merged.pop();
            }
            merged.push(decl);
        }
        let merged: &'static [Decl] = Box::leak(merged.into_boxed_slice());
        layers.push(Layer::new(cond, merged));
    }
    let layers: &'static [Layer] = Box::leak(layers.into_boxed_slice());
    #[cfg(feature = "tracing")]
    tracing::debug!(
        layers = layers.len(),
        diagnostics = diagnostics.len(),
        "cn: merged"
    );
    Ok(Compiled {
        program: Program::new(layers),
        diagnostics,
    })
}

/// The whole pipeline, theme-less or themed. `tokens` selects whether
/// the colour/modifier surfaces lower (the theme pass) or stay typed
/// boundaries (the honest default); `customs` is the registered
/// custom-authoring surface (empty registrations change nothing — the
/// differential pins it).
fn run(
    input: &str,
    config: Config,
    tokens: Option<&TokenTable>,
    customs: &Customs,
) -> Result<Compiled, CompileError> {
    // The internal walk takes the Copy config by value (clippy's
    // trivially-copy pass-by-ref); the public API keeps the reference.
    #[cfg(feature = "tracing")]
    tracing::debug!(input_len = input.len(), "compile: lex -> lookup -> lower");

    if input.trim().is_empty() {
        return Err(CompileError::Empty);
    }

    // Stage 1: the zero-copy lexer. Per-token failures degrade; the
    // stage never fails the string (the syntax crate's contract). The
    // degraded channel is TYPED: the failure carries the token's own
    // span inside the input — no re-derivation, no format contract.
    let (candidates, degraded) = parse_class_string(input);
    #[cfg(feature = "tracing")]
    tracing::debug!(
        tokens = candidates.len(),
        degraded = degraded.len(),
        "lexer done"
    );

    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    for failure in &degraded {
        diagnostics.push(Diagnostic {
            span: failure.span,
            raw: failure.raw.to_string(),
            kind: DiagnosticKind::Lex { kind: failure.kind },
        });
    }

    // Stages 2-4: fold the variant stack, read the registry, lower each
    // matched row. Conditions group first-seen; the specificity sort is
    // the assembly step's. The modifier normalizes once per candidate:
    // a modifier that does not parse (or names a var) stays the typed
    // boundary — with or without the theme pass. Arbitrary-valued
    // candidates track their row outputs: the sibling rows that never
    // matched the value's class skip SILENTLY, and the boundary
    // reasons defer to one deduped report per candidate (one class,
    // one boundary — never a per-sibling diagnostic pile, never a
    // silent drop when NO row took the class).
    let mut conds: Vec<Cond> = Vec::new();
    let mut groups: Vec<Vec<Decl>> = Vec::new();
    for candidate in &candidates {
        #[cfg(feature = "tracing")]
        tracing::trace!(raw = %candidate.raw, "candidate");

        if candidate.important {
            diagnostics.push(Diagnostic::new(
                candidate.span,
                candidate.raw,
                DiagnosticKind::UnsupportedImportant,
            ));
            continue;
        }
        let cond = match variant::fold(&candidate.variants, customs) {
            Ok(cond) => cond,
            Err(miss) => {
                diagnostics.push(variant_diagnostic(candidate, &miss));
                continue;
            }
        };
        // The registry read side, with the value pass's fraction
        // re-attachment (the audited oracle's F1: the raw `/modifier`
        // is the DENOMINATOR for fraction-shaped rows — `w-1/2` lexes
        // body `w-1` plus raw modifier `2`, and the fraction is `1/2`).
        // The first lookup runs on the lexed body; on a miss with a raw
        // modifier riding, the JOINED body retries — the registry's own
        // fraction shape accepts the joined suffix, and the modifier is
        // SPENT (no opacity rides it). `effective_body` is the body the
        // value pass and the diagnostics read.
        let joined = candidate
            .modifier
            .map(|modifier| format!("{}/{}", candidate.body, modifier));
        let (rows, effective_body, fraction_ride) = match &joined {
            // The joined body reads FIRST (the sizing family's own
            // precedence — a bare `w-1` also reads as spacing, so the
            // denominator reading must win the arm choice when a raw
            // modifier rides); a joined miss falls back to the lexed
            // body and the modifier goes through the opacity gate.
            Some(text) => match lookup_detailed(text) {
                Ok(retried) => (Ok(retried), text.as_str(), true),
                Err(_) => (lookup_detailed(candidate.body), candidate.body, false),
            },
            None => (lookup_detailed(candidate.body), candidate.body, false),
        };
        // The modifier gate, AFTER the read side: a fraction ride is not
        // an opacity modifier. Everything else normalizes as before —
        // unparseable/var-named modifiers always, parseable modifiers
        // without the theme pass too (the boundary is the theme-less
        // default).
        let alpha = if fraction_ride {
            None
        } else {
            match candidate.modifier {
                None => None,
                Some(modifier) => match normalize_modifier(modifier).filter(|_| tokens.is_some()) {
                    Some(alpha) => Some(alpha),
                    None => {
                        diagnostics.push(Diagnostic::new(
                            candidate.span,
                            candidate.raw,
                            DiagnosticKind::UnsupportedModifier {
                                modifier: modifier.to_string(),
                            },
                        ));
                        continue;
                    }
                },
            }
        };
        let mut produced = 0usize;
        let mut deferred: Vec<&'static str> = Vec::new();
        let mut reported = false;
        match rows {
            Ok(rows) => {
                for row in rows {
                    match lower::row_to_decls(
                        *row,
                        effective_body,
                        candidate.negative,
                        alpha,
                        tokens,
                        config,
                        candidate.arbitrary,
                    ) {
                        Ok(decls) => {
                            let idx = match conds.iter().position(|&c| c == cond) {
                                Some(idx) => idx,
                                None => {
                                    conds.push(cond);
                                    groups.push(Vec::new());
                                    conds.len() - 1
                                }
                            };
                            produced += decls.len();
                            groups[idx].extend(decls);
                        }
                        Err(reject) => {
                            reported = true;
                            // The arbitrary boundary set defers: the
                            // sibling rows share one miss, and the
                            // report renders once, deduped, below.
                            if let (true, lower::Reject::Value { reason }) =
                                (candidate.arbitrary.is_some(), &reject)
                                && arbitrary::is_boundary_reason(reason)
                            {
                                if !deferred.contains(reason) {
                                    deferred.push(reason);
                                }
                                continue;
                            }
                            diagnostics.push(reject_diagnostic(
                                candidate,
                                effective_body,
                                reject,
                                candidate.modifier,
                            ));
                        }
                    }
                }
                // The catch-all: an arbitrary candidate whose NO row
                // took (every sibling skipped silently) and that no
                // other report answered still reports — the
                // no-silent-failures rule, applied to the class gate's
                // quiet half. Deferred boundaries render here, deduped
                // (one class, one boundary).
                if candidate.arbitrary.is_some() && produced == 0 {
                    if !deferred.is_empty() {
                        for reason in deferred {
                            diagnostics.push(Diagnostic::new(
                                candidate.span,
                                candidate.raw,
                                DiagnosticKind::UnsupportedValue {
                                    body: effective_body.to_string(),
                                    reason,
                                },
                            ));
                        }
                    } else if !reported {
                        diagnostics.push(Diagnostic::new(
                            candidate.span,
                            candidate.raw,
                            DiagnosticKind::UnsupportedValue {
                                body: effective_body.to_string(),
                                reason: arbitrary::REASON_ARBITRARY_UNIT,
                            },
                        ));
                    }
                }
            }
            // The registry is the oracle; the customs are additive
            // surface consulted on the registry's unknown-root miss
            // only. A registered custom utility lowers through the SAME
            // group/cond assembly and the SAME typed boundary set. On a
            // custom miss the MORE SPECIFIC report wins: a custom shape
            // rejection names the custom root and its forms (the
            // built-in side said unknown-root — it never saw the
            // registration), while a double miss keeps the built-in
            // UnknownRoot (the two are byte-identical; the differential
            // pins it).
            Err(builtin_err) => match customs.lookup(candidate.body) {
                Ok(utilities) => {
                    for utility in utilities {
                        match custom::utility_to_decls(
                            utility,
                            candidate.body,
                            candidate.negative,
                            alpha,
                            tokens,
                            config,
                        ) {
                            Ok(decls) => {
                                let idx = match conds.iter().position(|&c| c == cond) {
                                    Some(idx) => idx,
                                    None => {
                                        conds.push(cond);
                                        groups.push(Vec::new());
                                        conds.len() - 1
                                    }
                                };
                                groups[idx].extend(decls);
                            }
                            Err(reject) => {
                                diagnostics.push(reject_diagnostic(
                                    candidate,
                                    effective_body,
                                    reject,
                                    candidate.modifier,
                                ));
                            }
                        }
                    }
                }
                Err(custom_err) => {
                    let err = match custom_err {
                        LookupError::ShapeRejected { .. } => custom_err,
                        LookupError::UnknownRoot { .. } => builtin_err,
                    };
                    diagnostics.push(lookup_diagnostic(candidate, err));
                }
            },
        }
    }

    // Stage 5: assemble — stable specificity sort (first-seen order
    // breaks ties), the per-group cascade order (Prop's discriminant IS
    // the sort order; the stable sort keeps source order within one
    // longhand, so the later duplicate wins the front-to-back read),
    // then intern into the leak pool.
    let mut order: Vec<(Cond, Vec<Decl>)> = conds.into_iter().zip(groups).collect();
    order.sort_by_key(|(cond, _)| cond.specificity());
    let mut layers: Vec<Layer> = Vec::with_capacity(order.len());
    for (cond, mut decls) in order {
        decls.sort_by_key(|decl: &Decl| decl.prop);
        let decls: &'static [Decl] = Box::leak(decls.into_boxed_slice());
        layers.push(Layer::new(cond, decls));
    }
    let layers: &'static [Layer] = Box::leak(layers.into_boxed_slice());
    #[cfg(feature = "tracing")]
    tracing::debug!(layers = layers.len(), "program interned");
    Ok(Compiled {
        program: Program::new(layers),
        diagnostics,
    })
}

/// The `/modifier` text → the alpha fraction the theme pass carries:
/// bare digits are percent (`"90"` → `0.9`), a `[...]`-bracketed value
/// is the literal fraction (`"[0.37]"` → `0.37`). Non-finite or
/// unparseable modifiers are `None` — the caller keeps the typed
/// boundary. The clamp itself lives in the audited bake
/// (`mix_with_transparent`) and the table's resolve.
pub(crate) fn normalize_modifier(modifier: &str) -> Option<f32> {
    let bracketed = modifier.starts_with('[') && modifier.ends_with(']');
    let raw = if bracketed {
        &modifier[1..modifier.len() - 1]
    } else {
        modifier
    };
    let value = raw.parse::<f32>().ok()?;
    if !value.is_finite() {
        return None;
    }
    Some(if bracketed { value } else { value / 100.0 })
}

/// The unsupported-variant report, spanned to the SEGMENT (the same
/// sequential find the lexer uses; repeated identical segments report
/// the first occurrence).
pub(crate) fn variant_diagnostic(
    candidate: &Candidate<'_>,
    miss: &variant::VariantMiss,
) -> Diagnostic {
    let offset = candidate.raw.find(&miss.variant).unwrap_or(0);
    let start = candidate.span.0 + offset;
    Diagnostic {
        span: (start, start + miss.variant.len()),
        raw: candidate.raw.to_string(),
        kind: DiagnosticKind::UnsupportedVariant {
            variant: miss.variant.clone(),
        },
    }
}

/// The registry read side's typed misses, carried through spanned.
pub(crate) fn lookup_diagnostic(candidate: &Candidate<'_>, err: LookupError) -> Diagnostic {
    let kind = match err {
        LookupError::UnknownRoot { body } => DiagnosticKind::UnknownRoot { body },
        LookupError::ShapeRejected { body, root, tried } => {
            DiagnosticKind::ShapeRejected { body, root, tried }
        }
    };
    Diagnostic::new(candidate.span, candidate.raw, kind)
}

/// The value pass's typed refusals, carried through spanned. `body` is
/// the EFFECTIVE body the value pass read (the fraction re-attachment's
/// joined form when the raw modifier rode as a denominator); the
/// modifier boundary renders the candidate's raw modifier text (the
/// value pass refuses the WHOLE row when an alpha cannot ride).
pub(crate) fn reject_diagnostic(
    candidate: &Candidate<'_>,
    body: &str,
    reject: lower::Reject,
    modifier: Option<&str>,
) -> Diagnostic {
    let kind = match reject {
        lower::Reject::Negation => DiagnosticKind::UnsupportedNegation {
            body: body.to_string(),
        },
        lower::Reject::Value { reason } => DiagnosticKind::UnsupportedValue {
            body: body.to_string(),
            reason,
        },
        lower::Reject::Modifier => DiagnosticKind::UnsupportedModifier {
            modifier: modifier.unwrap_or_default().to_string(),
        },
        // Internal marker: `row_to_decls` resolves it (the token path or
        // the typed REASON_TOKEN_TABLE value) before returning.
        lower::Reject::TokenOverflow => DiagnosticKind::UnsupportedValue {
            body: body.to_string(),
            reason: lower::REASON_TOKEN_TABLE,
        },
    };
    Diagnostic::new(candidate.span, candidate.raw, kind)
}
