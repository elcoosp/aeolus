//! The Aeolus V1 compile seam (architecture plan §3.1/§3.2, phase P2):
//! class strings in, cascade-sorted [`Program`]s out — one pure
//! function, no global state.
//!
//! ```text
//! compile(input, &Config)                        -> Result<Compiled, CompileError>
//! compile_theme(input, &TokenTable, &Config)     -> Result<Compiled, CompileError>
//!   ├─ 1. lex      aeolus_syntax::parse_class_string — borrowed
//!   │              candidates with byte spans; per-token failures
//!   │              degrade, never fail the string
//!   ├─ 2. fold     the variant stack → the single dominant Cond
//!   │              (the flat-IR model, cond.rs)
//!   ├─ 3. read     aeolus_data::registry::lookup_detailed — the typed
//!   │              registry read side (unknown root vs rejected shape)
//!   ├─ 4. lower    row + suffix → longhand Decl fan-outs (the value
//!   │              pass); with a TokenTable, colour tokens and the
//!   │              /opacity modifier lower through the theme pass —
//!   │              without one they are typed boundaries, not truncations
//!   └─ 5. assemble stable specificity sort → Layer masks derived →
//!                  interned as the IR's documented dynamic form
//! ```
//!
//! Everything a class does NOT lower to comes back in
//! [`Compiled::diagnostics`] with a byte span, a stable code and a help
//! text — the no-silent-failures rule (CONTRIBUTING §2) applied to the
//! pipeline: importance, arbitrary values, untyped variants, and (only
//! where no theme pass is supplied) colour tokens and modifiers are
//! typed boundaries, never dropped classes.
//!
//! What this slice deliberately is NOT (each is the next P2 slice, and
//! the seams say so in their rustdocs): the IR-level `cn()` merge (today
//! the raw cascade is emitted and the overlap is exactly
//! `Program::base_conflicts()`'s bits), `@utility`/`@custom-variant`,
//! the `Target` half of the plan §3.2 signature, and the xtask
//! generator story. The THEME half landed with this slice:
//! [`compile_theme`] takes the plan's second argument — the tiered
//! `TokenTable` from `aeolus-theme` — and colour tokens plus opacity
//! modifiers lower through it (`aeolus-compile` depends on
//! `aeolus-theme`, which wraps the audited v0.x theme reader).
//!
//! Storage note: [`Program`] holds `&'static` slices, so each compile
//! call interns its program into the leak pool the IR documents as the
//! dynamic form's storage ("one interned `Box<[Decl]>` shared by
//! reference"). P2's `DynCache` dedupes the pool later; until then one
//! compile per distinct class string is the intended call pattern.
//!
//! The registry semantics are pinned at compile time: the crate refuses
//! to build against a `REGISTRY_VERSION` other than the one its
//! lowering was audited against.

mod lower;
mod variant;

use std::fmt;

use aeolus_data::registry::lookup_detailed;
use aeolus_data::LookupError;
use aeolus_ir::{Cond, Decl, Layer, Program};
use aeolus_syntax::parse_class_string;
use aeolus_syntax::Candidate;
use aeolus_theme::TokenTable;

// The lowering audits the rows' meaning; a registry semantics bump must
// re-audit it. `matches!` on the &str literal is const-evaluable.
const _: () = assert!(
    matches!(aeolus_data::REGISTRY_VERSION, "1"),
    "aeolus-compile pins the registry semantics at REGISTRY_VERSION \"1\"; \
     re-audit the value pass before bumping the registry"
);

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
        Self { spacing: Spacing::Tailwind, rem_px: 16.0 }
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
    fn new(span: (usize, usize), raw: &str, kind: DiagnosticKind) -> Self {
        Self { span, raw: raw.to_string(), kind }
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
    /// crate's typed miss, carried through with its span.
    Lex {
        /// The stable lexer message (`LexErrorKind::message`).
        message: String,
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
    /// (`has-data-[x]:`, `[@media(print)]:`) — the next P2 slice.
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
    /// The value half does not lower in this slice — arbitrary values
    /// wait for the P2 value pass, oversized keyword tables (colour
    /// tokens) for the theme pass. The reason string says which.
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
                 breakpoints, containers and the group/peer probes; \
                 has-/data-/aria-/arbitrary variants fold in the next P2 \
                 slice"
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
                "the value half is the next P2 slice (unit grammar and type \
                 hints); colour tokens lower through compile_theme's \
                 TokenTable (aeolus-theme, plan §3.4)"
            }
        }
    }
}

impl fmt::Display for DiagnosticKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticKind::Lex { message } => f.write_str(message),
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
                write!(f, "modifier {modifier:?} lowers in the theme pass, not the compile seam")
            }
            DiagnosticKind::UnsupportedNegation { body } => {
                write!(f, "{body:?} refuses the negated form (the row's negative flag)")
            }
            DiagnosticKind::UnsupportedValue { body, reason } => {
                write!(f, "{body:?} does not lower in this slice: {reason}")
            }
        }
    }
}

/// Why the whole compile failed. Per-class failures are
/// [`Diagnostic`]s; the only whole-string failure is a caller bug.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompileError {
    /// The input holds no candidate at all: an empty/whitespace-only
    /// class string.
    Empty,
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::Empty => {
                f.write_str("empty class string: compile needs at least one non-whitespace token")
            }
        }
    }
}

impl std::error::Error for CompileError {}

#[cfg(feature = "miette")]
impl miette::Diagnostic for CompileError {
    fn code(&self) -> Option<Box<dyn fmt::Display + '_>> {
        Some(Box::new("aeolus::compile::empty_input"))
    }

    fn help(&self) -> Option<Box<dyn fmt::Display + '_>> {
        Some(Box::new(
            "a class string must contain at least one non-whitespace token; \
             an empty string is a caller bug, not a styling result",
        ))
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
    run(input, config, None)
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
    run(input, config, Some(tokens))
}

/// The whole pipeline, theme-less or themed. `tokens` selects whether
/// the colour/modifier surfaces lower (the theme pass) or stay typed
/// boundaries (the honest default).
fn run(
    input: &str,
    config: &Config,
    tokens: Option<&TokenTable>,
) -> Result<Compiled, CompileError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(input_len = input.len(), "compile: lex -> lookup -> lower");

    if input.trim().is_empty() {
        return Err(CompileError::Empty);
    }

    // Stage 1: the zero-copy lexer. Per-token failures degrade; the
    // stage never fails the string (the syntax crate's contract).
    let (candidates, degraded) = parse_class_string(input);
    #[cfg(feature = "tracing")]
    tracing::debug!(tokens = candidates.len(), degraded = degraded.len(), "lexer done");

    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    // The lexer's degraded pairs report (raw, message) without spans;
    // re-derive them with the SAME sequential find the lexer uses to
    // span its tokens, so the squiggle lands on the right occurrence.
    let mut cursor = 0usize;
    for (raw, message) in &degraded {
        let start = input[cursor..].find(raw.as_str()).map_or(cursor, |i| cursor + i);
        cursor = start + raw.len();
        // The pair text is `{raw}: {stable message}`; keep the message.
        let text = message.strip_prefix(raw.as_str()).unwrap_or(message);
        let text = text.strip_prefix(": ").unwrap_or(text);
        diagnostics.push(Diagnostic {
            span: (start, cursor),
            raw: raw.clone(),
            kind: DiagnosticKind::Lex { message: text.to_string() },
        });
    }

    // Stages 2-4: fold the variant stack, read the registry, lower each
    // matched row. Conditions group first-seen; the specificity sort is
    // the assembly step's. The modifier normalizes once per candidate:
    // a modifier that does not parse (or names a var) stays the typed
    // boundary — with or without the theme pass.
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
        let alpha = match candidate.modifier {
            None => None,
            Some(modifier) => match normalize_modifier(modifier).filter(|_| tokens.is_some()) {
                Some(alpha) => Some(alpha),
                None => {
                    // Unparseable/var-named modifiers always; parseable
                    // modifiers without the theme pass too (the boundary
                    // is the theme-less default).
                    diagnostics.push(Diagnostic::new(
                        candidate.span,
                        candidate.raw,
                        DiagnosticKind::UnsupportedModifier { modifier: modifier.to_string() },
                    ));
                    continue;
                }
            },
        };
        let cond = match variant::fold(&candidate.variants) {
            Ok(cond) => cond,
            Err(miss) => {
                diagnostics.push(variant_diagnostic(candidate, &miss));
                continue;
            }
        };
        let rows = match lookup_detailed(candidate.body) {
            Ok(rows) => rows,
            Err(err) => {
                diagnostics.push(lookup_diagnostic(candidate, err));
                continue;
            }
        };
        for row in rows {
            match lower::row_to_decls(*row, candidate.body, candidate.negative, alpha, tokens, config)
            {
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
                    diagnostics.push(reject_diagnostic(candidate, reject, candidate.modifier));
                }
            }
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
    Ok(Compiled { program: Program::new(layers), diagnostics })
}

/// The `/modifier` text → the alpha fraction the theme pass carries:
/// bare digits are percent (`"90"` → `0.9`), a `[...]`-bracketed value
/// is the literal fraction (`"[0.37]"` → `0.37`). Non-finite or
/// unparseable modifiers are `None` — the caller keeps the typed
/// boundary. The clamp itself lives in the audited bake
/// (`mix_with_transparent`) and the table's resolve.
fn normalize_modifier(modifier: &str) -> Option<f32> {
    let bracketed = modifier.starts_with('[') && modifier.ends_with(']');
    let raw = if bracketed { &modifier[1..modifier.len() - 1] } else { modifier };
    let value = raw.parse::<f32>().ok()?;
    if !value.is_finite() {
        return None;
    }
    Some(if bracketed { value } else { value / 100.0 })
}

/// The unsupported-variant report, spanned to the SEGMENT (the same
/// sequential find the lexer uses; repeated identical segments report
/// the first occurrence).
fn variant_diagnostic(candidate: &Candidate<'_>, miss: &str) -> Diagnostic {
    let offset = candidate.raw.find(miss).unwrap_or(0);
    let start = candidate.span.0 + offset;
    Diagnostic {
        span: (start, start + miss.len()),
        raw: candidate.raw.to_string(),
        kind: DiagnosticKind::UnsupportedVariant { variant: miss.to_string() },
    }
}

/// The registry read side's typed misses, carried through spanned.
fn lookup_diagnostic(candidate: &Candidate<'_>, err: LookupError) -> Diagnostic {
    let kind = match err {
        LookupError::UnknownRoot { body } => DiagnosticKind::UnknownRoot { body },
        LookupError::ShapeRejected { body, root, tried } => {
            DiagnosticKind::ShapeRejected { body, root, tried }
        }
    };
    Diagnostic::new(candidate.span, candidate.raw, kind)
}

/// The value pass's typed refusals, carried through spanned. The
/// modifier boundary renders the candidate's raw modifier text (the
/// value pass refuses the WHOLE row when an alpha cannot ride).
fn reject_diagnostic(
    candidate: &Candidate<'_>,
    reject: lower::Reject,
    modifier: Option<&str>,
) -> Diagnostic {
    let kind = match reject {
        lower::Reject::Negation => {
            DiagnosticKind::UnsupportedNegation { body: candidate.body.to_string() }
        }
        lower::Reject::Value { reason } => {
            DiagnosticKind::UnsupportedValue { body: candidate.body.to_string(), reason }
        }
        lower::Reject::Modifier => DiagnosticKind::UnsupportedModifier {
            modifier: modifier.unwrap_or_default().to_string(),
        },
        // Internal marker: `row_to_decls` resolves it (the token path or
        // the typed REASON_TOKEN_TABLE value) before returning.
        lower::Reject::TokenOverflow => DiagnosticKind::UnsupportedValue {
            body: candidate.body.to_string(),
            reason: lower::REASON_TOKEN_TABLE,
        },
    };
    Diagnostic::new(candidate.span, candidate.raw, kind)
}
