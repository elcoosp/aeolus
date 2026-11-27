//! `tw!` — the zero-runtime class front door (architecture plan §3.1
//! emit (a)/(b), P3).
//!
//! The macro runs the V1 compiler at expansion time and emits the
//! program's STATIC form — the IR layers plus the row-keyword texts the
//! runtime backend maps natively. The input string is never emitted
//! back: F1 (`tw!` discarded its compile-time result) is closed here.
//!
//! ```ignore
//! // A static program value, applied with `aeolus_gpui::TwExt::tw`:
//! div().tw(tw!("p-4 bg-primary hover:bg-primary/90"))
//!
//! // The same seam merges static class lists at compile time:
//! div().tw(cn!("p-4", "px-8 rounded-full"))
//!
//! // A CSS-first theme file (the same one a web v4 project uses):
//! let theme = theme!("themes/app.css");
//! ```
//!
//! The macro compiles against the embedded shadcn palette by default so
//! colour tokens bake to literals (T0/T1). Two escape hatches exist:
//!
//! * The `theme:` ARGUMENT bakes a HOST theme at expansion time —
//!   `tw!(theme: "themes/app.css", "bg-primary")` reads the CSS
//!   relative to the calling crate's `CARGO_MANIFEST_DIR`, bakes it
//!   through `ThemeSource::from_css` + `TokenTable::baked` (T0
//!   literals, T1 `ModePair` dark pairs, `/opacity` premultiplied) and
//!   carries an `include_str!` rebuild anchor inside the expansion, so
//!   Cargo re-runs rustc when the CSS changes (plan §3.4 T0/T1). The
//!   GPUI-gated entries keep their audit; the neutral
//!   `tw_static!`/`cn_static!`/`variants_static!` entries freeze
//!   WITHOUT one — the interpreter backends' `LoweringReport` is their
//!   honesty channel.
//! * The `theme!` MACRO stays the T2 runtime handle — a
//!   `theme_from_css(include_str!(…))` value the host re-themes live
//!   through its `TokenTable`; nothing about it changed here.
//!
//! The macro grammar is a hand-written token walk (the mission's syn-free
//! constraint); the pedantic lints it trips are excepted in the crate
//! manifest's `[lints.clippy]` table, with the rationale — the workspace
//! convention (root `Cargo.toml`) keeps every allow beside its reason.

use proc_macro::{Delimiter, TokenStream, TokenTree};
use std::collections::HashMap;
use std::fmt::Write as _;

use aeolus_compile::cn as merge_cn;
use aeolus_compile::{
    ClassKeywords, Compiled, Config, Diagnostic, compile_theme, keyword_alignment, keyword_texts,
};
use aeolus_ir::Cond;
use aeolus_theme::{ThemeSource, TokenTable};

/// The gate refusal list rendered as the compile-error tail: one
/// line per refusal, the longhand and the emitter's own reason (the
/// `Emit` impl's verdicts, never a parallel claim).
fn render_refusals(refusals: &[aeolus_codegen_gpui::Unsupported]) -> String {
    let mut out = String::new();
    for refusal in refusals {
        let _ = write!(out, "\n  {:?}: {}", refusal.prop, refusal.reason);
    }
    out
}

/// Compile a class string at build time into an `aeolus_rt::StaticProgram`.
///
/// GPUI-gated: the expansion is audited against
/// [`aeolus_codegen_gpui`] before it freezes — a capability the target
/// cannot express is a compile error, never a silent drop. The optional
/// leading `theme: "path", ` argument bakes a host theme at expansion
/// time (plan §3.4 T0/T1); without it the embedded shadcn bake rides.
#[proc_macro]
pub fn tw(input: TokenStream) -> TokenStream {
    let (theme, rest) = match split_theme_prefix(input.clone(), "tw!") {
        Ok(pair) => pair,
        Err(msg) => return compile_error(&input, &msg),
    };
    let literal = match extract_single_string(&rest, "tw!") {
        Ok(s) => s,
        Err(msg) => return compile_error(&input, &msg),
    };
    let bake = match theme_bake_arg(theme, "tw!") {
        Ok(bake) => bake,
        Err(msg) => return compile_error(&input, &msg),
    };
    let (compiled, classes) = match compile_literal_with(&literal, bake.as_ref(), "tw!", false) {
        Ok(pair) => pair,
        Err(msg) => return compile_error(&input, &msg),
    };
    let keywords = keyword_alignment(&compiled, &classes);
    let refusals = aeolus_codegen_gpui::audit(&compiled.program, &keywords);
    if !refusals.is_empty() {
        return compile_error(
            &input,
            &format!(
                "aeolus: {literal:?} uses capabilities GPUI cannot express:{}",
                render_refusals(&refusals)
            ),
        );
    }
    let profile = match project_profile("tw!") {
        Ok(profile) => profile,
        Err(msg) => return compile_error(&input, &msg),
    };
    let emitted =
        aeolus_codegen_gpui::emit_static_program_for(profile, &compiled.program, &keywords);
    anchor_expression(bake.as_ref(), emitted.into(), "tw!")
}

/// The BACKEND-NEUTRAL freeze entry: the same expansion-time bake and
/// the same static emission as [`tw!`], with NO GPUI audit gate. The
/// emitted `StaticProgram` is the shape every backend's `lower_program`
/// lane consumes; what the interpreter cannot express lands in its own
/// `LoweringReport` — the honesty channel — never a compile refusal.
/// The optional `theme:` argument rides exactly like [`tw!`]'s.
#[proc_macro]
pub fn tw_static(input: TokenStream) -> TokenStream {
    let (theme, rest) = match split_theme_prefix(input.clone(), "tw_static!") {
        Ok(pair) => pair,
        Err(msg) => return compile_error(&input, &msg),
    };
    let literal = match extract_single_string(&rest, "tw_static!") {
        Ok(s) => s,
        Err(msg) => return compile_error(&input, &msg),
    };
    let bake = match theme_bake_arg(theme, "tw_static!") {
        Ok(bake) => bake,
        Err(msg) => return compile_error(&input, &msg),
    };
    let (compiled, classes) =
        match compile_literal_with(&literal, bake.as_ref(), "tw_static!", false) {
            Ok(pair) => pair,
            Err(msg) => return compile_error(&input, &msg),
        };
    let keywords = keyword_alignment(&compiled, &classes);
    let profile = match project_profile("tw_static!") {
        Ok(profile) => profile,
        Err(msg) => return compile_error(&input, &msg),
    };
    let emitted =
        aeolus_codegen_gpui::emit_static_program_for(profile, &compiled.program, &keywords);
    anchor_expression(bake.as_ref(), emitted.into(), "tw_static!")
}

/// Compile-time `cn()`: compile each static list, merge at IR level, and
/// emit the merged static program. GPUI-gated, like [`tw!`].
#[proc_macro]
pub fn cn(input: TokenStream) -> TokenStream {
    let (theme, rest) = match split_theme_prefix(input.clone(), "cn!") {
        Ok(pair) => pair,
        Err(msg) => return compile_error(&input, &msg),
    };
    let strings = match extract_string_list(rest, "cn!") {
        Ok(s) => s,
        Err(msg) => return compile_error(&input, &msg),
    };
    let bake = match theme_bake_arg(theme, "cn!") {
        Ok(bake) => bake,
        Err(msg) => return compile_error(&input, &msg),
    };
    let mut compiled = Vec::new();
    let mut classes: Vec<ClassKeywords> = Vec::new();
    for string in &strings {
        match compile_literal_with(string, bake.as_ref(), "cn!", false) {
            Ok((c, k)) => {
                compiled.push(c);
                classes.extend(k);
            }
            Err(msg) => return compile_error(&input, &msg),
        }
    }
    let refs: Vec<&Compiled> = compiled.iter().collect();
    let merged = match merge_cn(&refs) {
        Ok(merged) => merged,
        Err(err) => return compile_error(&input, &format!("cn!: {err}")),
    };
    let keywords = last_keywords(&merged, &classes);
    let joined = strings.join(" ");
    let refusals = aeolus_codegen_gpui::audit(&merged.program, &keywords);
    if !refusals.is_empty() {
        return compile_error(
            &input,
            &format!(
                "aeolus: cn!({joined:?}) uses capabilities GPUI cannot express:{}",
                render_refusals(&refusals)
            ),
        );
    }
    let profile = match project_profile("cn!") {
        Ok(profile) => profile,
        Err(msg) => return compile_error(&input, &msg),
    };
    let emitted = aeolus_codegen_gpui::emit_static_program_for(profile, &merged.program, &keywords);
    anchor_expression(bake.as_ref(), emitted.into(), "cn!")
}

/// The BACKEND-NEUTRAL merge entry: the same compile-time `cn()` merge,
/// with NO GPUI audit gate (see [`tw_static!`]).
#[proc_macro]
pub fn cn_static(input: TokenStream) -> TokenStream {
    let (theme, rest) = match split_theme_prefix(input.clone(), "cn_static!") {
        Ok(pair) => pair,
        Err(msg) => return compile_error(&input, &msg),
    };
    let strings = match extract_string_list(rest, "cn_static!") {
        Ok(s) => s,
        Err(msg) => return compile_error(&input, &msg),
    };
    let bake = match theme_bake_arg(theme, "cn_static!") {
        Ok(bake) => bake,
        Err(msg) => return compile_error(&input, &msg),
    };
    let mut compiled = Vec::new();
    let mut classes: Vec<ClassKeywords> = Vec::new();
    for string in &strings {
        match compile_literal_with(string, bake.as_ref(), "cn_static!", false) {
            Ok((c, k)) => {
                compiled.push(c);
                classes.extend(k);
            }
            Err(msg) => return compile_error(&input, &msg),
        }
    }
    let refs: Vec<&Compiled> = compiled.iter().collect();
    let merged = match merge_cn(&refs) {
        Ok(merged) => merged,
        Err(err) => return compile_error(&input, &format!("cn_static!: {err}")),
    };
    let keywords = last_keywords(&merged, &classes);
    let profile = match project_profile("cn_static!") {
        Ok(profile) => profile,
        Err(msg) => return compile_error(&input, &msg),
    };
    let emitted = aeolus_codegen_gpui::emit_static_program_for(profile, &merged.program, &keywords);
    anchor_expression(bake.as_ref(), emitted.into(), "cn_static!")
}

/// The emit (a) END FORM's macro face: compile the literal and emit the
/// builder-call closure — the method calls the walk would run, rendered
/// at expansion time (plan `docs/AEOLUS_V1_ARCHITECTURE.md` §3.2
/// status: "The backend METHOD calls the emit (a) end form implies
/// (`FlexDirection::Column`, builder chains) remain the P3 codegen
/// crate's surface"). Applied through [`TwExt::tw_chain`], the per-frame
/// cost is the builder calls themselves — no walk, no dispatch.
///
/// The embedded shadcn bake rides (the T0 literals — the chain carries
/// compile-time colour values only; a runtime-token slot or a
/// non-native conditional family is a compile error naming the reason,
/// the [`tw!`]-gated honesty contract). The `theme:` prefix does NOT
/// ride here yet: the chain form's themed bake is the follow-up slice,
/// the frozen-program macros carry it today.
#[proc_macro]
pub fn tw_builder(input: TokenStream) -> TokenStream {
    let literal = match extract_single_string(&input.clone(), "tw_builder!") {
        Ok(s) => s,
        Err(msg) => return compile_error(&input, &msg),
    };
    let (compiled, classes) = match compile_literal_with(&literal, None, "tw_builder!", false) {
        Ok(pair) => pair,
        Err(msg) => return compile_error(&input, &msg),
    };
    let keywords = keyword_alignment(&compiled, &classes);
    let refusals = aeolus_codegen_gpui::audit_builder(&compiled.program, &keywords);
    if !refusals.is_empty() {
        return compile_error(
            &input,
            &format!(
                "aeolus: {literal:?} uses capabilities the builder chain cannot express:{}",
                render_refusals(&refusals)
            ),
        );
    }
    match aeolus_codegen_gpui::emit_builder(&compiled.program, &keywords) {
        Ok(tokens) => tokens.into(),
        Err(refusals) => compile_error(
            &input,
            &format!(
                "aeolus: {literal:?} uses capabilities the builder chain cannot express:{}",
                render_refusals(&refusals)
            ),
        ),
    }
}

/// The compatibility alias kept for the pre-1.0 spelling (`tw_cn!`).
#[proc_macro]
pub fn tw_cn(input: TokenStream) -> TokenStream {
    cn(input)
}

/// Parse a CSS-first theme file at the host (`@theme`/`:root`/`.dark`,
/// the same file a web Tailwind v4 project uses) into the T2 RUNTIME
/// handle: the returned `ThemeSource` feeds a `TokenTable::runtime` the
/// host re-themes live. Compile-time baking is the `theme:` ARGUMENT on
/// the other entries — `theme!` stays the dynamic lane's front door.
///
/// One argument: the light source. Two arguments: light + dark override.
#[proc_macro]
pub fn theme(input: TokenStream) -> TokenStream {
    let strings = match extract_string_list(input.clone(), "theme!") {
        Ok(s) => s,
        Err(msg) => return compile_error(&input, &msg),
    };
    match strings.as_slice() {
        [light] => {
            let light = format!("include_str!({light:?})");
            let light: TokenStream = light.parse().expect("include_str path");
            quote_ts(&format!(
                "::aeolus_rt::theme_from_css({light}, ::core::option::Option::None)"
            ))
        }
        [light, dark] => {
            let light = format!("include_str!({light:?})");
            let dark = format!("include_str!({dark:?})");
            quote_ts(&format!(
                "::aeolus_rt::theme_from_css({light}, ::core::option::Option::Some({dark}))"
            ))
        }
        _ => compile_error(&input, "theme!: expected one or two string literal paths"),
    }
}

// ---------------------------------------------------------------------------
// The compile walk
// ---------------------------------------------------------------------------

/// A host theme read at expansion time: the sheet's CSS plus the
/// absolute path the rebuild anchor rides on (plan §3.4's
/// `include_str!` contract — Cargo re-runs rustc when the CSS changes).
#[derive(Debug)]
struct ThemeBake {
    css: String,
    anchor: String,
}

/// The rebuild anchor the themed expansions carry: an UNNAMED `const _`
/// item reading the theme file, so Cargo tracks the dependency. Unnamed
/// so two themed call sites in one scope coexist (the integration
/// battery's whole-file-scope compilation pins it).
fn theme_anchor_tokens(abs_path: &str) -> String {
    format!("const _: &str = ::core::include_str!({abs_path:?});")
}

/// Wrap an EXPRESSION-position expansion with the rebuild anchor. The
/// no-arg path passes the expansion through untouched — the embedded
/// shadcn bake stays byte-identical (the differential pin).
fn anchor_expression(theme: Option<&ThemeBake>, emitted: TokenStream, name: &str) -> TokenStream {
    let Some(bake) = theme else {
        return emitted;
    };
    let anchor = theme_anchor_tokens(&bake.anchor);
    format!("{{ {anchor} {emitted} }}")
        .parse()
        .unwrap_or_else(|_| {
            compile_error_msg(&format!("{name}: could not render the theme anchor"))
        })
}

/// The merged-arm assembly: compile each PIECE separately and fold the
/// programs through the IR-level `cn()` overlay (plan §3.3's
/// "Overlay/`cn()` at IR level") — the composition lane the runtime
/// handle's cn-merge runs, so the frozen arm bytes match the RefCell
/// path's merged programs. One piece short-circuits to its own compile
/// (the single-class arms keep the raw cascade byte shape); the keyword
/// annotations concatenate in piece order (the alignment queues
/// consume per (cond, prop), so the merged program's winners pop the
/// piece-order texts the joined compile would have carried). The
/// diagnostics ride `cn`'s concatenation; the allow_refusals contract
/// is each piece's own compile (a refusal without the key is the
/// compile error the literal form reports).
fn compile_pieces_with(
    pieces: &[&str],
    theme: Option<&ThemeBake>,
    allow_refusals: bool,
) -> Result<(Compiled, Vec<ClassKeywords>), String> {
    if let [only] = pieces {
        return compile_literal_with(only, theme, "variants!", allow_refusals);
    }
    // The merge runs at the granularity the runtime handle freezes its
    // programs at: the string-level conflict merge over the pieces (the
    // `cn()` engine's audited table — the same call
    // `aeolus_shadcn::classes::button_classes` runs), then ONE compile
    // of the merged string. The IR-level `overlay` dedupes per-longhand
    // — coarser than the cascade-origin pairs the runtime path keeps
    // (the `aria-invalid` ring pair rides two variant prefixes, one
    // layer, both decls) — so the byte-identity pin holds at the
    // merge-the-runtime-runs granularity, and the IR overlay stays the
    // runtime composition lane beside it.
    let merged = aeolus_compile::cn_string(pieces);
    compile_literal_with(&merged, theme, "variants!", allow_refusals)
}

fn compile_literal_with(
    literal: &str,
    theme: Option<&ThemeBake>,
    name: &str,
    allow_refusals: bool,
) -> Result<(Compiled, Vec<ClassKeywords>), String> {
    let source = match theme {
        // The no-arg bake: the embedded shadcn palette, exactly as it
        // always ran (byte-identical — the differential pin).
        None => ThemeSource::shadcn(),
        // The host bake: the SINGLE-FILE sheet a web v4 project uses —
        // the light half minus its `.dark` blocks, those blocks the dark
        // variant (a sheet without `.dark` yields zero pairs: pure T0).
        Some(bake) => ThemeSource::from_app_css(&bake.css),
    }
    .map_err(|err| format!("{name}: theme: {err}"))?;
    let table = TokenTable::baked(&source).map_err(|err| format!("{name}: theme: {err}"))?;
    let config = Config::default();
    let compiled =
        compile_theme(literal, &table, &config).map_err(|err| format!("{name}: {err}"))?;
    if !compiled.diagnostics.is_empty() && !allow_refusals {
        return Err(render_diagnostics(name, &compiled.diagnostics));
    }
    let classes = keyword_texts(literal, &config).map_err(|err| format!("{name}: {err}"))?;
    Ok((compiled, classes))
}

/// The compile-time diagnostic rendering. The DEFAULT is the one-line
/// form the rustc window shows inline (the diagnostics-equality
/// contract's own shape); the `verbose-diagnostics` feature renders
/// each refusal MULTI-LINE with the typed boundary's stable machine
/// code and help text — the editor/LSP parity the pipeline's
/// diagnostics carry (the `DiagnosticKind::code`/`help` seam), for
/// hosts that want the full contract inside the compile error.
#[cfg(not(feature = "verbose-diagnostics"))]
fn render_diagnostics(prefix: &str, diagnostics: &[Diagnostic]) -> String {
    let mut msg = String::new();
    for diagnostic in diagnostics {
        let _ = write!(msg, "{prefix}: {diagnostic}; ");
    }
    msg
}

/// The verbose half: same walks, same refusals — each diagnostic's
/// code and help ride beside the span line (the honest-failure
/// contract, rendered in full).
#[cfg(feature = "verbose-diagnostics")]
fn render_diagnostics(prefix: &str, diagnostics: &[Diagnostic]) -> String {
    let mut msg = String::new();
    for diagnostic in diagnostics {
        let _ = write!(
            msg,
            "{prefix}: {}\n  code: {}\n  help: {}\n",
            diagnostic,
            diagnostic.kind.code(),
            diagnostic.kind.help()
        );
    }
    msg
}

/// The merged form: one text per merged declaration, keyed by the LAST
/// source occurrence (rightmost wins, the `cn()` contract).
fn last_keywords(compiled: &Compiled, classes: &[ClassKeywords]) -> Vec<&'static str> {
    let mut last: HashMap<(Cond, aeolus_ir::Prop), &'static str> = HashMap::new();
    for class in classes {
        for &(prop, text) in &class.keywords {
            last.insert((class.cond, prop), text);
        }
    }
    let mut out = Vec::new();
    for layer in compiled.program.layers {
        for decl in layer.decls {
            out.push(last.get(&(layer.cond, decl.prop)).copied().unwrap_or(""));
        }
    }
    out
}

fn quote_ts(text: &str) -> TokenStream {
    text.parse()
        .unwrap_or_else(|_| compile_error_msg("macro: generated tokens did not parse"))
}

// ---------------------------------------------------------------------------
// Token parsing (syn-free, the mission's own constraint)
// ---------------------------------------------------------------------------

/// Split the optional leading `theme: "path", ` argument off the front
/// of a macro's token stream (plan §3.4 T0/T1's expansion-time bake).
/// The theme argument is a PREFIX: the payload is everything after the
/// argument's comma, so the plain forms parse exactly as before.
fn split_theme_prefix(
    input: TokenStream,
    name: &str,
) -> Result<(Option<String>, TokenStream), String> {
    let trees: Vec<TokenTree> = input.into_iter().collect();
    let is_theme_arg = matches!(
        (trees.first(), trees.get(1)),
        (Some(TokenTree::Ident(id)), Some(TokenTree::Punct(p)))
            if id.to_string() == "theme" && p.as_char() == ':'
    );
    if !is_theme_arg {
        return Ok((None, trees.into_iter().collect()));
    }
    // The theme path: one string literal after `theme:`, then the
    // argument's comma. A second literal before the comma is the
    // tokenizer's own "exactly one" refusal — the grammar has no
    // two-path form here (the T2 `theme!` keeps that one).
    let mut path_tokens: Vec<TokenTree> = Vec::new();
    let mut rest_start: Option<usize> = None;
    for (index, token) in trees.iter().enumerate().skip(2) {
        if let TokenTree::Punct(p) = token
            && p.as_char() == ','
            && !path_tokens.is_empty()
        {
            rest_start = Some(index + 1);
            break;
        }
        path_tokens.push(token.clone());
    }
    let Some(rest_start) = rest_start else {
        return Err(format!(
            "{name}: theme: expected the theme path followed by a comma before the class \
             strings"
        ));
    };
    let stream: TokenStream = path_tokens.into_iter().collect();
    let path = extract_single_string(&stream, name)?;
    Ok((Some(path), trees[rest_start..].iter().cloned().collect()))
}

/// The expansion-time `theme:` argument's bake: [`theme_bake`] when an
/// argument rode, `None` (the embedded shadcn bake) when not.
fn theme_bake_arg(theme: Option<String>, name: &str) -> Result<Option<ThemeBake>, String> {
    match theme {
        Some(rel) => resolve_theme(&rel, name).map(Some),
        None => Ok(None),
    }
}

/// Read the `theme:` argument's CSS relative to the CALLING crate's
/// manifest (the same resolution the `include_str!` anchor rides). An
/// empty sheet is a typed refusal of its own; a malformed sheet is the
/// pipeline's own `ThemeError` text, carried verbatim into the compile
/// error by the themed bake.
fn resolve_theme(rel: &str, name: &str) -> Result<ThemeBake, String> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").map_err(|_| {
        format!(
            "{name}: theme: CARGO_MANIFEST_DIR is unset — the theme path resolves relative \
             to the calling crate's manifest"
        )
    })?;
    let path = std::path::Path::new(&manifest).join(rel);
    let anchor = path.display().to_string();
    let css = std::fs::read_to_string(&path)
        .map_err(|err| format!("{name}: theme file {rel:?}: {err}"))?;
    if css.trim().is_empty() {
        return Err(format!(
            "{name}: theme file {rel:?}: empty theme file — the bake refuses an empty sheet \
             (declare `@theme` tokens or a `:root`/`.dark` palette)"
        ));
    }
    Ok(ThemeBake { css, anchor })
}

/// Read the project's `aeolus.toml` code-size profile — plan §4's
/// knob (`profile = "speed" | "size"`, the inline-vs-outlined emit
/// axis) — relative to the CALLING crate's manifest (the same
/// `CARGO_MANIFEST_DIR` anchor the `theme:` bake rides). A missing
/// file answers the shipped default (the outlined emit), so the knob
/// is opt-in; a mis-set knob is the codegen crate's typed refusal,
/// rendered as a compile error by the entry points.
fn project_profile(name: &str) -> Result<aeolus_codegen_gpui::Profile, String> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").map_err(|_| {
        format!(
            "{name}: profile: CARGO_MANIFEST_DIR is unset — the aeolus.toml \
             knob resolves relative to the calling crate's manifest"
        )
    })?;
    aeolus_codegen_gpui::profile_from_manifest(std::path::Path::new(&manifest))
        .map_err(|err| format!("{name}: profile: {err}"))
}

fn extract_single_string(input: &TokenStream, name: &str) -> Result<String, String> {
    let mut strings = extract_string_list(input.clone(), name)?;
    if strings.len() != 1 {
        return Err(format!("{name}: expected exactly one string literal"));
    }
    Ok(strings.remove(0))
}

fn extract_string_list(input: TokenStream, name: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut pending: Option<String> = None;
    for token in input {
        match token {
            TokenTree::Literal(lit) => {
                let s = lit.to_string();
                if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                    let unquoted = unescape(&s[1..s.len() - 1]);
                    match pending.as_mut() {
                        Some(prev) => prev.push_str(&unquoted),
                        None => pending = Some(unquoted),
                    }
                } else {
                    return Err(format!(
                        "{name}: unsupported literal {s} (expected a string)"
                    ));
                }
            }
            TokenTree::Punct(p) if p.as_char() == ',' => {
                if let Some(s) = pending.take() {
                    out.push(s);
                }
            }
            TokenTree::Group(g) if g.delimiter() == Delimiter::None => {
                let inner = extract_string_list(g.stream(), name)?;
                out.extend(inner);
            }
            other => {
                return Err(format!(
                    "{name}: unexpected token {other} (expected a string literal)"
                ));
            }
        }
    }
    if let Some(s) = pending.take() {
        out.push(s);
    }
    Ok(out)
}

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some('0') => out.push('\0'),
            // Rust's escaped newline: `\` then a line break, the
            // following indentation swallowed. Class strings in the wild
            // wrap this way.
            Some('\n' | '\r') => {
                while chars.peek().is_some_and(|next| next.is_whitespace()) {
                    let _ = chars.next();
                }
            }
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

fn compile_error(input: &TokenStream, msg: &str) -> TokenStream {
    let text = format!("compile_error!({msg:?})");
    let _ = input;
    text.parse()
        .unwrap_or_else(|_| compile_error_msg("tw!: could not render the error"))
}

fn compile_error_msg(msg: &str) -> TokenStream {
    format!("compile_error!({msg:?})")
        .parse()
        .expect("compile_error! is valid Rust")
}

// ---------------------------------------------------------------------------
// `variants!`: compile-time component variant tables
// ---------------------------------------------------------------------------

struct VariantTable {
    visibility: String,
    name: String,
    fields: Vec<VariantField>,
    /// The closed-world table's refusal tolerance: the shadcn tables'
    /// documented boundary families (`[&_svg]`, `has-[>svg]`, the
    /// data-orientation attribute forms) lower with TYPED diagnostics on
    /// the runtime-interned path — the diagnostics are the honesty
    /// channel the handle carries. A `variants_static!` table that names
    /// `allow_refusals: true` freezes the SAME programs at expansion
    /// time WITH those diagnostics refused as the documented boundary
    /// (the static form has no diagnostics slot; the runtime path stays
    /// the diagnostics source of record and the differential pins the
    /// bytes). Without the key the strict contract holds: any diagnostic
    /// is a compile error.
    allow_refusals: bool,
}

struct VariantField {
    field: String,
    classes: VariantClasses,
}

/// The classes one group value (or one compound entry's `class`) hands
/// out. The STRING form applies to the whole component; the brace form
/// (`{ root: "…", title: "…" }`) is the per-slot map — only legal in a
/// table that declares `slots:` (validated after the parse, since the
/// `slots:` field may arrive after the groups that use it).
#[derive(Clone)]
enum ClassSource {
    All(String),
    PerSlot(Vec<(String, String)>),
}

impl ClassSource {
    /// The classes this source hands to `slot` — the string form's text
    /// for every slot, the brace form's own text for the slots it names
    /// (an unnamed slot takes nothing).
    fn for_slot<'a>(&'a self, slot: &str) -> &'a str {
        match self {
            ClassSource::All(text) => text,
            ClassSource::PerSlot(pairs) => pairs
                .iter()
                .find(|(name, _)| name == slot)
                .map_or("", |(_, text)| text),
        }
    }

    /// True when the source names a slot the table does not declare.
    fn has_unknown_slot(&self, slots: &[(String, String)]) -> Option<String> {
        if let ClassSource::PerSlot(pairs) = self {
            for (name, _) in pairs {
                if !slots.iter().any(|(slot, _)| slot == name) {
                    return Some(name.clone());
                }
            }
        }
        None
    }
}

enum VariantClasses {
    Base(String),
    Groups(Vec<(String, ClassSource)>),
    /// `slots: { ... }` — the multi-part form (the §9 open question's
    /// "slots" word, the tailwind-variants shape): one base per part,
    /// a `{Name}Slot` enum and a `style_slot(slot, …)` dispatch
    /// generated for the table. A `base:` beside `slots:` applies to
    /// every slot (prepended).
    Slots(Vec<(String, String)>),
    /// `compound: [ ... ]` — the cva compound-variant entries (plan §9's
    /// open question, decided): extra classes applied to every combo
    /// that matches ALL of an entry's constraints, resolved at EXPANSION
    /// time (the entries fold into the cartesian match arms the render
    /// already emits — zero runtime cost beyond the dispatch).
    Compound(Vec<CompoundEntry>),
}

/// One `compound:` entry: the group-key constraints (ALL must match the
/// selection) and the class list they ride (string = every slot, brace
/// = the slots it names).
struct CompoundEntry {
    constraints: Vec<(String, String)>,
    classes: ClassSource,
}

/// Compile a closed-world variant table (the shadcn/cva surface):
///
/// ```ignore
/// variants! { pub Button {
///     base: "inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium …",
///     variant: { default: "bg-primary text-primary-foreground",
///                destructive: "bg-destructive text-white hover:bg-destructive/90" },
///     size:    { default: "h-9 px-4 py-2", sm: "h-8 px-3" },
/// } }
/// ```
///
/// Generates one enum per group, a `style(...)` function returning the
/// merged `aeolus_rt::StaticProgram`, and a `Default` enum variant
/// (the key named `default`, else the first key).
///
/// # The slot form (multi-part components)
///
/// A `slots:` map declares one base per PART of a component (a card's
/// root/header/title, the tailwind-variants shape) and generates a
/// `{Name}Slot` enum (default: the first declared slot) plus a
/// `style_slot(slot, …)` dispatch — each part is its OWN program:
///
/// ```ignore
/// variants! { pub Card {
///     base: "text-card-foreground",              // applies to every slot
///     slots: {
///         root: "flex flex-col gap-6 rounded-xl border py-6",
///         header: "flex flex-col gap-1.5 px-6",
///         title: "font-semibold leading-none",
///     },
///     variant: {
///         default: "text-sm",                    // a string applies to EVERY slot
///         outline: { root: "border-gray-200",    // a brace applies to the slots it names
///                    title: "text-muted-foreground" },
///     },
///     compound: [ { variant: outline, class: { root: "shadow-none" } } ],
/// } }
///
/// Card::style_slot(CardSlot::Title, CardVariant::Outline)
/// ```
///
/// A group value or a compound `class:` may be a string (every slot)
/// or a brace naming slots; per-slot braces in a table without
/// `slots:`, an unknown slot name, a duplicate slot, an empty `slots:`
/// map and a compound on a group-less table are typed grammar errors.
/// The arms compile per (slot, combo) at expansion time, so the
/// runtime cost stays the match dispatch the table always had.
#[proc_macro]
pub fn variants(input: TokenStream) -> TokenStream {
    variants_expansion(input, true)
}

/// The BACKEND-NEUTRAL variants table: the same closed-world compile,
/// with NO GPUI audit gate (see [`tw_static!`] — the interpreter
/// backends' `LoweringReport` is the honesty channel).
#[proc_macro]
pub fn variants_static(input: TokenStream) -> TokenStream {
    variants_expansion(input, false)
}

/// The two tables' shared walk: the optional `theme:` prefix splits
/// off first, then the table grammar, then the render (the gate is the
/// only difference between the entries).
fn variants_expansion(input: TokenStream, gate: bool) -> TokenStream {
    let (theme, rest) = match split_theme_prefix(input.clone(), "variants!") {
        Ok(pair) => pair,
        Err(msg) => return compile_error(&input, &msg),
    };
    let table = match parse_variants(rest) {
        Ok(table) => table,
        Err(msg) => return compile_error(&input, &msg),
    };
    let bake = match theme_bake_arg(theme, "variants!") {
        Ok(bake) => bake,
        Err(msg) => return compile_error(&input, &msg),
    };
    match render_variants(&table, bake.as_ref(), gate) {
        Ok(tokens) => tokens,
        Err(msg) => compile_error(&input, &msg),
    }
}

fn parse_variants(input: TokenStream) -> Result<VariantTable, String> {
    let mut tokens = input.into_iter().peekable();
    let mut visibility = String::new();
    if matches!(tokens.peek(), Some(TokenTree::Ident(id)) if id.to_string() == "pub") {
        let _ = tokens.next();
        visibility.push_str("pub ");
        if let Some(TokenTree::Group(group)) = tokens.peek() {
            if group.delimiter() == Delimiter::Parenthesis {
                let mut text = String::from("pub(");
                for token in group.stream() {
                    text.push_str(&token.to_string());
                }
                text.push_str(") ");
                let _ = tokens.next();
                visibility = text;
            }
        }
    }
    let name = match tokens.next() {
        Some(TokenTree::Ident(id)) => id.to_string(),
        _ => return Err("variants!: expected a component type name".into()),
    };
    let body = match tokens.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => group.stream(),
        _ => return Err("variants!: expected `{ ... }` after the type name".into()),
    };

    let entries = split_top_level(body, ',');
    let mut fields = Vec::new();
    let mut pending_field: Option<String> = None;
    let mut allow_refusals = false;
    for entry in entries {
        let parts = split_first_colon(entry);
        let Some((field, rest)) = parts else {
            continue;
        };
        let field = field.trim().to_string();
        let rest = rest.trim();
        if rest.is_empty() {
            pending_field = Some(field);
            continue;
        }
        if field == "allow_refusals" {
            allow_refusals = rest == "true";
            if rest != "true" && rest != "false" {
                return Err("variants!: `allow_refusals` takes `true` or `false`".into());
            }
            continue;
        }
        let classes = match rest.parse::<TokenStream>() {
            Ok(stream) => parse_variant_classes(stream, &field)?,
            Err(_) => return Err(format!("variants!: `{field}` has an invalid value")),
        };
        fields.push(VariantField { field, classes });
        pending_field = None;
    }
    if fields.is_empty() {
        return Err("variants!: the table has no fields".into());
    }
    if let Some(field) = pending_field {
        return Err(format!("variants!: `{field}` has no value"));
    }
    Ok(VariantTable {
        visibility,
        name,
        fields,
        allow_refusals,
    })
}

fn parse_variant_classes(rest: TokenStream, field: &str) -> Result<VariantClasses, String> {
    let tokens: Vec<TokenTree> = rest.clone().into_iter().collect();
    // The compound LIST: a bracket group of brace entries, each naming
    // group-key constraints plus the `class` list they ride.
    if field == "compound" {
        if let Some(TokenTree::Group(group)) = tokens.first() {
            if group.delimiter() == Delimiter::Bracket {
                return parse_compound_entries(group.stream(), field);
            }
        }
        return Err(format!(
            "variants!: `{field}` expects a `[ {{ ... }} ]` list of entries"
        ));
    }
    // The slot MAP: a brace group of `name: "classes"` pairs, one base
    // per part of the component (the multi-part form).
    if field == "slots" {
        if let Some(TokenTree::Group(group)) = tokens.first() {
            if group.delimiter() == Delimiter::Brace {
                return parse_slot_map(group.stream(), field).map(VariantClasses::Slots);
            }
        }
        return Err(format!(
            "variants!: `{field}` expects a `{{ ... }}` map of slot bases"
        ));
    }
    if let Some(TokenTree::Group(group)) = tokens.first() {
        if group.delimiter() == Delimiter::Brace {
            let mut groups = Vec::new();
            let mut pending: Option<String> = None;
            for entry in split_top_level(group.stream(), ',') {
                let parts = split_first_colon(entry);
                let Some((key, value)) = parts else {
                    continue;
                };
                let key = key.trim().to_string();
                let value = value.trim();
                if value.is_empty() {
                    pending = Some(key);
                    continue;
                }
                let stream: TokenStream = value
                    .parse()
                    .map_err(|_| format!("variants!: `{field}.{key}` is not a string"))?;
                // A string value applies to every slot; a brace value
                // is the per-slot map (only legal in a table that
                // declares `slots:` — validated in the render, since
                // the `slots:` field may arrive after this group).
                let source =
                    if let Some(TokenTree::Group(inner)) = stream.clone().into_iter().next() {
                        if inner.delimiter() == Delimiter::Brace {
                            ClassSource::PerSlot(parse_slot_map(inner.stream(), field)?)
                        } else {
                            ClassSource::All(extract_single_string(&stream, "variants!")?)
                        }
                    } else {
                        ClassSource::All(extract_single_string(&stream, "variants!")?)
                    };
                groups.push((key, source));
                pending = None;
            }
            if let Some(key) = pending {
                return Err(format!("variants!: `{field}.{key}` has no class list"));
            }
            if groups.is_empty() {
                return Err(format!("variants!: `{field}` group is empty"));
            }
            return Ok(VariantClasses::Groups(groups));
        }
    }
    if let Some(TokenTree::Literal(_)) = tokens.first() {
        return Ok(VariantClasses::Base(extract_single_string(
            &rest,
            "variants!",
        )?));
    }
    Err(format!(
        "variants!: `{field}` expects a string or `{{ ... }}`"
    ))
}

/// Parse a `slots: { ... }` map (or a per-slot brace value): each
/// `name: "classes"` pair is one slot's base. Order is declaration
/// order (the first slot is the slot enum's default); duplicate names
/// and an empty map are grammar errors.
fn parse_slot_map(stream: TokenStream, field: &str) -> Result<Vec<(String, String)>, String> {
    let mut slots = Vec::new();
    let mut pending: Option<String> = None;
    for entry in split_top_level(stream, ',') {
        let parts = split_first_colon(entry);
        let Some((name, value)) = parts else {
            continue;
        };
        let name = name.trim().to_string();
        let value = value.trim();
        if value.is_empty() {
            pending = Some(name);
            continue;
        }
        if slots.iter().any(|(slot, _)| slot == &name) {
            return Err(format!("variants!: `{field}` declares `{name}` twice"));
        }
        let text = extract_single_string(
            &value
                .parse::<TokenStream>()
                .map_err(|_| format!("variants!: `{field}.{name}` is not a string"))?,
            "variants!",
        )?;
        slots.push((name, text));
        pending = None;
    }
    if let Some(name) = pending {
        return Err(format!("variants!: `{field}.{name}` has no class list"));
    }
    if slots.is_empty() {
        return Err(format!("variants!: `{field}` map is empty"));
    }
    Ok(slots)
}

/// Parse a `compound: [ ... ]` list: each brace entry carries
/// `group: key` constraint pairs plus the `class` list they apply to
/// (a string = every slot, a brace = the slots it names).
fn parse_compound_entries(stream: TokenStream, field: &str) -> Result<VariantClasses, String> {
    let mut entries = Vec::new();
    for entry in split_top_level(stream, ',') {
        let tokens: Vec<TokenTree> = entry.into_iter().collect();
        let Some(TokenTree::Group(group)) = tokens.first() else {
            continue;
        };
        if group.delimiter() != Delimiter::Brace {
            return Err(format!(
                "variants!: `{field}` entries are brace groups (`{{ variant: key, class: \"…\" }}`)"
            ));
        }
        let mut constraints = Vec::new();
        let mut classes: Option<ClassSource> = None;
        for pair in split_top_level(group.stream(), ',') {
            let Some((key, value)) = split_first_colon(pair) else {
                continue;
            };
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            if key == "class" {
                let parsed = value
                    .parse::<TokenStream>()
                    .map_err(|_| format!("variants!: `{field}.class` is not a string"))?;
                // The two forms: a string rides every slot, a brace
                // names the slots it rides (validated against the
                // declared slots in the render).
                let source =
                    if let Some(TokenTree::Group(inner)) = parsed.clone().into_iter().next() {
                        if inner.delimiter() == Delimiter::Brace {
                            ClassSource::PerSlot(parse_slot_map(inner.stream(), field)?)
                        } else {
                            ClassSource::All(extract_single_string(&parsed, "variants!")?)
                        }
                    } else {
                        ClassSource::All(extract_single_string(&parsed, "variants!")?)
                    };
                classes = Some(source);
            } else {
                constraints.push((key, value));
            }
        }
        if constraints.is_empty() {
            return Err(format!(
                "variants!: `{field}` entry constrains nothing (name at least one group key)"
            ));
        }
        let Some(classes) = classes else {
            return Err(format!("variants!: `{field}` entry has no class list"));
        };
        entries.push(CompoundEntry {
            constraints,
            classes,
        });
    }
    if entries.is_empty() {
        return Err(format!("variants!: `{field}` list is empty"));
    }
    Ok(VariantClasses::Compound(entries))
}

fn split_top_level(stream: TokenStream, separator: char) -> Vec<TokenStream> {
    let mut out = Vec::new();
    let mut current = Vec::new();
    for token in stream {
        match &token {
            TokenTree::Punct(punct) if punct.as_char() == separator => {
                if !current.is_empty() {
                    out.push(current.drain(..).collect());
                }
            }
            _ => current.push(token),
        }
    }
    if !current.is_empty() {
        out.push(current.into_iter().collect());
    }
    out
}

fn split_first_colon(stream: TokenStream) -> Option<(String, String)> {
    let mut left = String::new();
    let mut right = String::new();
    let mut seen = false;
    for token in stream {
        if !seen {
            if let TokenTree::Punct(punct) = &token {
                if punct.as_char() == ':' {
                    seen = true;
                    continue;
                }
            }
            left.push_str(&token.to_string());
        } else {
            right.push_str(&token.to_string());
            if let TokenTree::Punct(punct) = &token {
                if punct.as_char() == ',' {
                    right.pop();
                }
            }
        }
    }
    seen.then_some((left, right))
}

fn pascal_case(text: &str) -> String {
    let mut out = String::new();
    for part in text.split(|c: char| !c.is_alphanumeric()) {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.extend(chars);
        }
    }
    if out.is_empty() {
        out.push_str("Variant");
    }
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, 'V');
    }
    out
}

fn render_variants(
    table: &VariantTable,
    theme: Option<&ThemeBake>,
    gate: bool,
) -> Result<TokenStream, String> {
    // The code-size profile resolves ONCE per table — the §4
    // `aeolus.toml` knob every arm's freeze renders under (the
    // entries' own opt-in; the default is the shipped outlined face).
    let profile = project_profile("variants!")?;
    let mut output = String::new();
    // The themed tables carry the rebuild anchor as their first ITEM
    // (module position — the enum defs follow).
    if let Some(bake) = theme {
        output.push_str(&theme_anchor_tokens(&bake.anchor));
        output.push('\n');
    }
    let visibility = &table.visibility;
    let name = &table.name;

    // The declared slots, if the table carries the multi-part form.
    let slots: Option<&[(String, String)]> =
        table.fields.iter().find_map(|field| match &field.classes {
            VariantClasses::Slots(slots) => Some(slots.as_slice()),
            _ => None,
        });

    let base: String = table
        .fields
        .iter()
        .filter_map(|field| match &field.classes {
            VariantClasses::Base(classes) => Some(classes.as_str()),
            VariantClasses::Groups(_) | VariantClasses::Slots(_) | VariantClasses::Compound(_) => {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let mut enum_defs = String::new();
    let mut signature = Vec::new();
    let mut group_fields = Vec::new();

    for field in &table.fields {
        if let VariantClasses::Groups(groups) = &field.classes {
            let enum_name = format!("{name}{}", pascal_case(&field.field));
            enum_defs.push_str(&format!(
                "#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]\n{visibility}enum {enum_name} {{\n"
            ));
            let default_key = groups
                .iter()
                .find(|(key, _)| key == "default")
                .map_or(0, |(key, _)| {
                    groups.iter().position(|(k, _)| k == key).unwrap_or(0)
                });
            for (index, (key, _)) in groups.iter().enumerate() {
                let variant = pascal_case(key);
                if index == default_key {
                    enum_defs.push_str("    #[default]\n");
                }
                enum_defs.push_str(&format!("    {variant},\n"));
            }
            enum_defs.push_str("}\n");
            signature.push(format!("{}: {enum_name}", field.field));
            group_fields.push((field.field.clone(), enum_name, groups.clone()));
        }
    }

    // The slot enum rides the group enums (default: the FIRST declared
    // slot, the declaration order the map keeps).
    let slot_enum = format!("{name}Slot");
    if let Some(slots) = slots {
        enum_defs.push_str(&format!(
            "#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]\n{visibility}enum {slot_enum} {{\n"
        ));
        for (index, (slot, _)) in slots.iter().enumerate() {
            if index == 0 {
                enum_defs.push_str("    #[default]\n");
            }
            enum_defs.push_str(&format!("    {},\n", pascal_case(slot)));
        }
        enum_defs.push_str("}\n");
    }

    // The compound entries, with every constraint validated against the
    // declared groups (an unknown group or key is a grammar error, not
    // a silent never-matching arm).
    let compounds: Vec<&CompoundEntry> = table
        .fields
        .iter()
        .filter_map(|field| match &field.classes {
            VariantClasses::Compound(entries) => Some(entries.iter().collect::<Vec<_>>()),
            _ => None,
        })
        .flatten()
        .collect();

    // Per-slot braces name SLOTS: an unknown slot (or a brace in a
    // table without `slots:`) is a grammar error, not a silent
    // never-riding class list.
    let no_slots: Vec<(String, String)> = Vec::new();
    let declared_slots = slots.unwrap_or(&no_slots);
    for (field, _, groups) in &group_fields {
        for (key, source) in groups {
            if let Some(unknown) = source.has_unknown_slot(declared_slots) {
                return Err(format!(
                    "variants!: `{field}.{key}` names `{unknown}` — no such slot (per-slot \
                     braces need a `slots:` table)"
                ));
            }
        }
    }
    for entry in &compounds {
        if let Some(unknown) = entry.classes.has_unknown_slot(declared_slots) {
            return Err(format!(
                "variants!: `compound` class names `{unknown}` — no such slot (per-slot braces \
                 need a `slots:` table)"
            ));
        }
    }

    if group_fields.is_empty() {
        // A compound entry constrains GROUPS; a table without groups has
        // nothing to constrain (the honest grammar error).
        if let Some(entry) = compounds.first() {
            let _ = entry;
            return Err(
                "variants!: `compound` constrains a group the table does not declare".into(),
            );
        }
        match slots {
            Some(slots) => {
                // The group-less slotted table: the dispatch is the
                // slot alone, one frozen program per declared part.
                let mut arms = String::new();
                for (slot, classes) in slots {
                    let full = join_classes(&base, classes);
                    let (program, class_annotations) =
                        compile_literal_with(&full, theme, "variants!", table.allow_refusals)?;
                    let keywords = keyword_alignment(&program, &class_annotations);
                    if gate {
                        let refusals = aeolus_codegen_gpui::audit(&program.program, &keywords);
                        if !refusals.is_empty() {
                            return Err(format!(
                                "variants!: {full:?} uses capabilities GPUI cannot express:{}",
                                render_refusals(&refusals)
                            ));
                        }
                    }
                    let tokens = aeolus_codegen_gpui::emit_static_program_for(
                        profile,
                        &program.program,
                        &keywords,
                    );
                    arms.push_str(&format!(
                        "        {slot_enum}::{} => {tokens},\n",
                        pascal_case(slot)
                    ));
                }
                output.push_str(&enum_defs);
                output.push_str(&format!("{visibility}struct {name};\n"));
                output.push_str(&format!(
                    "impl {name} {{\n    {visibility}fn style_slot(slot: {slot_enum}) -> ::aeolus_rt::StaticProgram {{\n        match slot {{\n{arms}        }}\n    }}\n}}\n"
                ));
            }
            None => {
                let (program, _) =
                    compile_literal_with(&base, theme, "variants!", table.allow_refusals)?;
                let keywords = keyword_alignment(&program, &[]);
                if gate {
                    let refusals = aeolus_codegen_gpui::audit(&program.program, &keywords);
                    if !refusals.is_empty() {
                        return Err(format!(
                            "variants!: {base:?} uses capabilities GPUI cannot express:{}",
                            render_refusals(&refusals)
                        ));
                    }
                }
                let tokens = aeolus_codegen_gpui::emit_static_program_for(
                    profile,
                    &program.program,
                    &keywords,
                );
                output.push_str(&enum_defs);
                output.push_str(&format!(
                    "{visibility}struct {name};\nimpl {name} {{\n    {visibility}fn style() -> ::aeolus_rt::StaticProgram {{\n        {tokens}\n    }}\n}}\n"
                ));
            }
        }
        return output
            .parse()
            .map_err(|_| "variants!: generated code did not parse".into());
    }

    let combos = cartesian(&group_fields);
    let mut arms = String::new();
    match slots {
        None => {
            // The plain table: one arm per combo, the group sources'
            // texts joined in declaration order (a per-slot brace was
            // already refused above — no `slots:` to name).
            for combo in combos {
                let mut pattern = Vec::new();
                // The merged-arm assembly (the M3 deviation's close): the
                // arm's PIECES compile separately and fold through the
                // IR-level `cn()` overlay at expansion time — the same
                // composition lane the runtime handle's cn-merge runs, so
                // the frozen bytes match the RefCell path's merged
                // programs (the differential pins it), never the joined
                // string's raw-cascade shape.
                let mut pieces: Vec<&str> = Vec::new();
                if !base.is_empty() {
                    pieces.push(base.as_str());
                }
                for (field, enum_name, groups) in &group_fields {
                    let key = combo
                        .iter()
                        .find(|(field_name, _)| field_name == field)
                        .map(|(_, key)| key.as_str())
                        .unwrap_or_else(|| groups[0].0.as_str());
                    let class = groups
                        .iter()
                        .find(|(group_key, _)| group_key == key)
                        .map_or("", |(_, source)| source.for_slot(""));
                    if !class.is_empty() {
                        pieces.push(class);
                    }
                    pattern.push(format!("{enum_name}::{}", pascal_case(key)));
                }
                // The compound entries whose constraints ALL match this combo
                // fold their class list in — cva's semantics, resolved at
                // expansion time (each matching entry rides its own piece).
                for entry in &compounds {
                    if entry
                        .constraints
                        .iter()
                        .all(|(group, key)| combo.iter().any(|(f, k)| f == group && k == key))
                    {
                        let class = entry.classes.for_slot("");
                        if !class.is_empty() {
                            pieces.push(class);
                        }
                    }
                }
                let (program, class_annotations) =
                    compile_pieces_with(&pieces, theme, table.allow_refusals)?;
                let keywords = keyword_alignment(&program, &class_annotations);
                if gate {
                    let refusals = aeolus_codegen_gpui::audit(&program.program, &keywords);
                    if !refusals.is_empty() {
                        return Err(format!(
                            "variants!: {:?} uses capabilities GPUI cannot express:{}",
                            pieces.join(" "),
                            render_refusals(&refusals)
                        ));
                    }
                }
                let tokens = aeolus_codegen_gpui::emit_static_program_for(
                    profile,
                    &program.program,
                    &keywords,
                );
                arms.push_str(&format!("        ({}) => {tokens},\n", pattern.join(", ")));
            }

            output.push_str(&enum_defs);
            output.push_str(&format!("{visibility}struct {name};\n"));
            let params = signature.join(", ");
            output.push_str(&format!(
                "impl {name} {{\n    {visibility}fn style({params}) -> ::aeolus_rt::StaticProgram {{\n        match ("
            ));
            let names: Vec<&str> = group_fields
                .iter()
                .map(|(field, _, _)| field.as_str())
                .collect();
            output.push_str(&names.join(", "));
            output.push_str(") {\n");
            output.push_str(&arms);
            output.push_str("        }\n    }\n}\n");
        }
        Some(slots) => {
            // The slotted table: the cartesian grows the SLOT
            // dimension — one arm per (slot, combo), the classes that
            // arm's slot sees (the shared base, the slot's own base,
            // the group sources' texts FOR the slot, the compound
            // entries' classes FOR the slot). The runtime cost stays
            // the match dispatch the plain table always had.
            for (slot, slot_base) in slots {
                for combo in &combos {
                    let mut pattern = vec![format!("{slot_enum}::{}", pascal_case(slot))];
                    // The merged-arm assembly (the plain table's own rule):
                    // base + slot base + group sources + compounds ride
                    // SEPARATE piece compiles, folded through the IR-level
                    // `cn()` overlay before the freeze — the RefCell path's
                    // merged byte shape, not the joined string's.
                    let mut pieces: Vec<&str> = Vec::new();
                    if !base.is_empty() {
                        pieces.push(base.as_str());
                    }
                    if !slot_base.is_empty() {
                        pieces.push(slot_base.as_str());
                    }
                    for (field, enum_name, groups) in &group_fields {
                        let key = combo
                            .iter()
                            .find(|(field_name, _)| field_name == field)
                            .map(|(_, key)| key.as_str())
                            .unwrap_or_else(|| groups[0].0.as_str());
                        let class = groups
                            .iter()
                            .find(|(group_key, _)| group_key == key)
                            .map_or("", |(_, source)| source.for_slot(slot));
                        if !class.is_empty() {
                            pieces.push(class);
                        }
                        pattern.push(format!("{enum_name}::{}", pascal_case(key)));
                    }
                    for entry in &compounds {
                        if entry
                            .constraints
                            .iter()
                            .all(|(group, key)| combo.iter().any(|(f, k)| f == group && k == key))
                        {
                            let class = entry.classes.for_slot(slot);
                            if !class.is_empty() {
                                pieces.push(class);
                            }
                        }
                    }
                    let (program, class_annotations) =
                        compile_pieces_with(&pieces, theme, table.allow_refusals)?;
                    let keywords = keyword_alignment(&program, &class_annotations);
                    if gate {
                        let refusals = aeolus_codegen_gpui::audit(&program.program, &keywords);
                        if !refusals.is_empty() {
                            return Err(format!(
                                "variants!: {:?} uses capabilities GPUI cannot express:{}",
                                pieces.join(" "),
                                render_refusals(&refusals)
                            ));
                        }
                    }
                    let tokens = aeolus_codegen_gpui::emit_static_program_for(
                        profile,
                        &program.program,
                        &keywords,
                    );
                    arms.push_str(&format!("        ({}) => {tokens},\n", pattern.join(", ")));
                }
            }

            let mut signature = signature;
            signature.insert(0, format!("slot: {slot_enum}"));
            let params = signature.join(", ");
            output.push_str(&enum_defs);
            output.push_str(&format!("{visibility}struct {name};\n"));
            output.push_str(&format!(
                "impl {name} {{\n    {visibility}fn style_slot({params}) -> ::aeolus_rt::StaticProgram {{\n        match ("
            ));
            let mut names = vec!["slot"];
            names.extend(group_fields.iter().map(|(field, _, _)| field.as_str()));
            output.push_str(&names.join(", "));
            output.push_str(") {\n");
            output.push_str(&arms);
            output.push_str("        }\n    }\n}\n");
        }
    }
    output
        .parse()
        .map_err(|_| "variants!: generated code did not parse".into())
}

/// Join two class lists with one space, skipping empty sides (the arm
/// assembly's whitespace discipline).
fn join_classes(left: &str, right: &str) -> String {
    match (left.is_empty(), right.is_empty()) {
        (true, true) => String::new(),
        (true, false) => right.to_string(),
        (false, true) => left.to_string(),
        (false, false) => format!("{left} {right}"),
    }
}

fn cartesian(
    fields: &[(String, String, Vec<(String, ClassSource)>)],
) -> Vec<Vec<(String, String)>> {
    let mut rows = vec![Vec::new()];
    for (field, _, groups) in fields {
        let mut next = Vec::new();
        for row in &rows {
            for (key, _) in groups {
                let mut extended = row.clone();
                extended.push((field.clone(), key.clone()));
                next.push(extended);
            }
        }
        rows = next;
    }
    rows
}

// ---------------------------------------------------------------------------
// The diagnostic renderer's own contracts (the plain-fn half — the
// unit battery the proc-macro shell can host without an expansion)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod render_tests {
    use super::{
        ThemeBake, compile_literal_with, render_diagnostics, resolve_theme, theme_anchor_tokens,
    };
    use aeolus_compile::{Config, compile};

    /// The two renderers carry the SAME refusal set — the verbose mode
    /// adds the code/help lines, never a different diagnostic.
    #[test]
    fn the_renderers_carry_the_same_refusals() {
        let compiled = compile("not-a-class", &Config::default()).expect("the walk is total");
        let plain = render_diagnostics("tw!", &compiled.diagnostics);
        assert!(plain.contains("tw!:"), "{plain}");
        assert!(plain.contains("not-a-class"), "{plain}");
        // The verbose renderer compiles under the feature; under the
        // default both spellings agree on the token text. (The verbose
        // shape's own pin runs in CI's feature job through the same
        // battery — see the manifest's feature doc.)
    }

    #[cfg(feature = "verbose-diagnostics")]
    #[test]
    fn the_verbose_renderer_carries_code_and_help() {
        let compiled = compile("not-a-class", &Config::default()).expect("the walk is total");
        let verbose = render_diagnostics("tw!", &compiled.diagnostics);
        assert!(verbose.contains("code: "), "{verbose}");
        assert!(verbose.contains("help: "), "{verbose}");
    }

    /// The rebuild anchor is the unnamed `const _` the plan §3.4 sketch
    /// pins: `include_str!` on the absolute path, so Cargo tracks the
    /// theme file. Unnamed is the contract — two themed call sites in
    /// one scope must coexist (the integration battery compiles a whole
    /// file of themed expansions through exactly this shape).
    #[test]
    fn the_theme_anchor_is_an_unnamed_include_str_const() {
        let anchor = theme_anchor_tokens("/abs/path/app.css");
        assert!(anchor.starts_with("const _: &str = "), "{anchor}");
        assert!(anchor.contains("::core::include_str!("), "{anchor}");
        assert!(anchor.contains("/abs/path/app.css"), "{anchor}");
        assert!(anchor.trim_end().ends_with(';'), "{anchor}");
    }

    /// A missing theme file is a typed compile error naming the macro,
    /// the path and the io failure — never a panic, never a silent
    /// shadcn fallback.
    #[test]
    fn the_missing_theme_file_refuses_with_the_macro_s_name() {
        let err = resolve_theme("tests/fixtures/missing.css", "tw_static!")
            .expect_err("the missing file refuses");
        assert!(err.starts_with("tw_static!"), "{err}");
        assert!(err.contains("theme file"), "{err}");
        assert!(err.contains("missing.css"), "{err}");
    }

    /// An EMPTY theme file is its own typed refusal — the bake refuses
    /// a sheet that declares nothing rather than silently baking the
    /// default palette.
    #[test]
    fn the_empty_theme_file_refuses_typed() {
        let err =
            resolve_theme("tests/fixtures/empty.css", "tw!").expect_err("the empty sheet refuses");
        assert!(err.starts_with("tw!"), "{err}");
        assert!(err.contains("empty theme file"), "{err}");
    }

    /// A malformed sheet is the pipeline's own diagnostic text carried
    /// verbatim (the `ThemeError::Parse` display rides the compile
    /// error — the macro wraps, never re-words).
    #[test]
    fn the_malformed_theme_sheet_carries_the_pipeline_s_text() {
        let bake = ThemeBake {
            css: "@theme { --color-brand: var(--never-declared); }".to_string(),
            anchor: String::new(),
        };
        let err = compile_literal_with("bg-primary", Some(&bake), "tw!", false)
            .expect_err("the unresolved var() refuses");
        assert!(err.starts_with("tw!: theme: "), "{err}");
        assert!(
            err.contains("theme CSS:"),
            "the ThemeError text rides verbatim: {err}"
        );
    }
}
