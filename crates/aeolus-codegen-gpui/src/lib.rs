//! The GPUI codegen target (architecture plan §3.2's
//! `aeolus-codegen-gpui`).
//!
//! The arm emits the `aeolus_rt::StaticProgram` expression a `tw!`
//! expansion embeds: the IR's static layers plus the ROW TEXTS the
//! row-relative keyword slots index ([`aeolus_compile::keyword_texts`]).
//! The runtime half (`aeolus-gpui`) maps that form to GPUI's builder
//! methods; the split keeps this crate free of the GPUI dependency, so
//! a macro expansion can render the artifact without linking the UI
//! framework.
//!
//! The capability table ([`supports`]) is the §3.7 matrix in code:
//! every longhand GPUI's `StyleRefinement` expresses natively. The
//! reported longhands are the documented boundaries (outline,
//! transforms, filters), not silent drops — a host renders the returns
//! of [`Emit::emit`] as diagnostics. The ring family rides the §3.7
//! POLYFILL tier ([`polyfilled`]): no native slot, but the runtime
//! composes the v4 box-shadow list from GPUI's own `Vec<BoxShadow>`
//! (`aeolus_gpui::RingState`), so a program whose only refusals are
//! ring longhands emits.

use aeolus_codegen::conformance::{Capability, Tier};
use aeolus_codegen::{Emit, ir_path};

/// The emit gate's refusal type, re-exported for the macro surface's
/// `audit` gate (the compile-error renderer reads `prop` + `reason`).
pub use aeolus_codegen::Unsupported;
use aeolus_compile::{Config, keyword_texts};
use aeolus_ir::{Bp, Cond, Program, Prop, Val};
use aeolus_theme::{ThemeSource, TokenTable};
use proc_macro2::TokenStream;
use quote::quote;

/// The GPUI target: one program plus its parallel keyword annotations.
#[derive(Clone, Copy, Debug, Default)]
pub struct GpuiEmit<'a> {
    /// Row texts parallel to the flattened declarations, layer order
    /// (the `aeolus_rt::StaticProgram` contract).
    pub keywords: &'a [&'a str],
}

impl<'a> GpuiEmit<'a> {
    /// Build the target over the macro's annotation slice.
    pub const fn new(keywords: &'a [&'a str]) -> Self {
        Self { keywords }
    }
}

impl Emit for GpuiEmit<'_> {
    type Output = TokenStream;

    fn emit(&self, program: &Program) -> Result<Self::Output, Vec<Unsupported>> {
        // The keyword annotations are parallel to the flattened
        // declarations (layer-major order — the `emit_static_program`
        // contract), so the text-dependent verdicts read the same
        // slice the frozen program carries.
        let mut flat = 0usize;
        let refused: Vec<Unsupported> = program
            .layers
            .iter()
            .flat_map(|layer| layer.decls.iter())
            .filter_map(|decl| {
                let keyword = self.keywords.get(flat).copied().unwrap_or_default();
                flat += 1;
                unsupported_reason(decl.prop, keyword)
                    .map(|reason| Unsupported::new(decl.prop, reason))
            })
            .collect();
        if !refused.is_empty() {
            return Err(refused);
        }
        Ok(emit_static_program(program, self.keywords))
    }
}

/// The diagnostics a third-party `lower_class` call reports: compile
/// failures and capability refusals in one shape (the gpui-rsx bridge
/// consumes this for its LSP/preview surface).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Diagnostics {
    /// Human-readable compile errors (spanned in the compiler's own
    /// `Display` form).
    pub messages: Vec<String>,
    /// Longhands the GPUI target cannot express.
    pub unsupported: Vec<Unsupported>,
}

impl Diagnostics {
    /// True when no message and no refusal was recorded.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty() && self.unsupported.is_empty()
    }
}

/// Lower one class literal into the static-program token expression the
/// `tw!` macro embeds — the public seam for third parties (gpui-rsx).
///
/// # Errors
/// [`Diagnostics`] carrying compile messages and/or capability refusals.
pub fn lower_class(literal: &str, config: &Config) -> Result<TokenStream, Diagnostics> {
    let source = ThemeSource::shadcn().map_err(|err| Diagnostics {
        messages: vec![err.to_string()],
        unsupported: Vec::new(),
    })?;
    let table = TokenTable::baked(&source).map_err(|err| Diagnostics {
        messages: vec![err.to_string()],
        unsupported: Vec::new(),
    })?;
    let compiled =
        aeolus_compile::compile_theme(literal, &table, config).map_err(|err| Diagnostics {
            messages: vec![err.to_string()],
            unsupported: Vec::new(),
        })?;
    let mut diagnostics = Diagnostics {
        messages: compiled
            .diagnostics
            .iter()
            .map(ToString::to_string)
            .collect(),
        unsupported: Vec::new(),
    };
    // The annotations first: the text-dependent verdicts (the gradient
    // direction's row text) read the same slice the emit freezes.
    let annotations = keyword_texts(literal, config).map_err(|err| Diagnostics {
        messages: vec![err.to_string()],
        unsupported: Vec::new(),
    })?;
    let keywords = aeolus_compile::keyword_alignment(&compiled, &annotations);
    let mut flat = 0usize;
    for layer in compiled.program.layers {
        for decl in layer.decls {
            let keyword = keywords.get(flat).copied().unwrap_or_default();
            flat += 1;
            if let Some(reason) = unsupported_reason(decl.prop, keyword) {
                diagnostics
                    .unsupported
                    .push(Unsupported::new(decl.prop, reason));
            }
        }
    }
    diagnostics.unsupported.sort_by_key(|entry| entry.prop);
    diagnostics.unsupported.dedup();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    Ok(emit_static_program(&compiled.program, &keywords))
}

/// Render the `aeolus_rt::StaticProgram` expression for one compiled
/// program. `keywords` is parallel to the flattened declarations
/// (layer-major order); a missing entry renders as `""` (no row text).
/// The capability GATE the macro surface runs: walk the frozen
/// program's declarations through the SAME [`GpuiEmit`]'s
/// `unsupported_reason` gates the emit renders — the
/// publish-the-emitter's-verdicts rule — and collect every refusal.
/// The macro calls this before freezing, and a non-empty list is a
/// compile error naming the class and the reason (plan §9's
/// per-class diagnostic story): the honest-failure contract reaches
/// `tw!`/`cn!`/`variants!`, so a class the GPUI target cannot express
/// (the motion families, the gradient via stop) refuses at expansion
/// time instead of freezing into a silent runtime no-op.
pub fn audit(program: &Program, keywords: &[&str]) -> Vec<Unsupported> {
    GpuiEmit { keywords }
        .emit(program)
        .err()
        .unwrap_or_default()
}

pub fn emit_static_program(program: &Program, keywords: &[&str]) -> TokenStream {
    let ir = ir_path();
    let mut decl_consts = Vec::new();
    let mut layer_entries = Vec::new();
    for (layer_index, layer) in program.layers.iter().enumerate() {
        let cond = cond_tokens(layer.cond, &ir);
        let decls = layer.decls.iter().map(|decl| {
            let prop: TokenStream = format!("{:?}", decl.prop).parse().expect("variant name");
            let val = val_tokens(decl.val, &ir);
            quote! {
                #ir::Decl { prop: #ir::Prop::#prop, val: #val }
            }
        });
        let name = proc_macro2::Ident::new(
            &format!("DECLS_{layer_index}"),
            proc_macro2::Span::call_site(),
        );
        decl_consts.push(quote! {
            const #name: &[#ir::Decl] = &[#(#decls),*];
        });
        layer_entries.push(quote! {
            #ir::Layer::new(#cond, #name)
        });
    }
    let texts = keywords.iter().map(|text| {
        let text = *text;
        quote! { #text }
    });
    quote! {{
        #(#decl_consts)*
        const LAYERS: &[#ir::Layer] = &[#(#layer_entries),*];
        const KEYWORDS: &[&str] = &[#(#texts),*];
        ::aeolus_rt::StaticProgram::new(LAYERS, KEYWORDS)
    }}
}

/// Flatten per-class annotations into the declaration-parallel slice the
/// static program stores. Duplicate `(cond, prop)` pairs consume in
/// source order (the compile assembly's stable order).
fn bp_tokens(bp: Bp, ir: &TokenStream) -> TokenStream {
    let name: TokenStream = format!("{bp:?}").parse().expect("breakpoint name");
    quote! { #ir::Bp::#name }
}

/// The named-container payload's tokens (`@md/card:`'s `card` — the
/// `&'static str` the fold interned) — `None` for the bare forms.
fn name_tokens(name: Option<&str>) -> TokenStream {
    match name {
        Some(name) => quote! { ::core::option::Option::Some(#name) },
        None => quote! { ::core::option::Option::None },
    }
}

fn cond_tokens(cond: Cond, ir: &TokenStream) -> TokenStream {
    match cond {
        Cond::Data { name, value } => {
            let value = match value {
                Some(value) => quote! { ::core::option::Option::Some(#value) },
                None => quote! { ::core::option::Option::None },
            };
            quote! { #ir::Cond::Data { name: #name, value: #value } }
        }
        Cond::Aria { name, value } => {
            let value = match value {
                Some(value) => quote! { ::core::option::Option::Some(#value) },
                None => quote! { ::core::option::Option::None },
            };
            quote! { #ir::Cond::Aria { name: #name, value: #value } }
        }
        Cond::Has(inner) => quote! { #ir::Cond::Has(#inner) },
        Cond::GroupHas(inner) => quote! { #ir::Cond::GroupHas(#inner) },
        Cond::PeerHas(inner) => quote! { #ir::Cond::PeerHas(#inner) },
        Cond::Bp(bp) => {
            let bp = bp_tokens(bp, ir);
            quote! { #ir::Cond::Bp(#bp) }
        }
        Cond::Min(bp) => {
            let bp = bp_tokens(bp, ir);
            quote! { #ir::Cond::Min(#bp) }
        }
        Cond::Max(bp) => {
            let bp = bp_tokens(bp, ir);
            quote! { #ir::Cond::Max(#bp) }
        }
        Cond::Container { bp, name } => {
            let bp = bp_tokens(bp, ir);
            let name = name_tokens(name);
            quote! { #ir::Cond::Container { bp: #bp, name: #name } }
        }
        Cond::MaxContainer { bp, name } => {
            let bp = bp_tokens(bp, ir);
            let name = name_tokens(name);
            quote! { #ir::Cond::MaxContainer { bp: #bp, name: #name } }
        }
        // The structural probes: the payload is the test itself (the
        // ordinal rides the variant) — the frozen static item carries
        // it verbatim, the interp walk's `StructProbe` seam answering
        // at runtime.
        Cond::Structural(test) => {
            let name: TokenStream = format!("{test:?}").parse().expect("structural test name");
            quote! { #ir::Cond::Structural(#ir::Structural::#name) }
        }
        other => {
            let name: TokenStream = format!("{other:?}").parse().expect("condition name");
            quote! { #ir::Cond::#name }
        }
    }
}

fn val_tokens(val: Val, ir: &TokenStream) -> TokenStream {
    match val {
        Val::Px(n) => {
            let n = n as f64;
            quote! { #ir::Val::Px(#n as f32) }
        }
        Val::Rem(n) => {
            let n = n as f64;
            quote! { #ir::Val::Rem(#n as f32) }
        }
        Val::Pct(n) => {
            let n = n as f64;
            quote! { #ir::Val::Pct(#n as f32) }
        }
        Val::Num(n) => {
            let n = n as f64;
            quote! { #ir::Val::Num(#n as f32) }
        }
        Val::Rgba(c) => {
            let [r, g, b, a] = c;
            quote! { #ir::Val::Rgba([#r, #g, #b, #a]) }
        }
        Val::Tok { id, alpha } => {
            let alpha = alpha as f64;
            quote! { #ir::Val::Tok { id: #id, alpha: #alpha as f32 } }
        }
        Val::ModePair(id) => quote! { #ir::Val::ModePair(#id) },
        Val::Kw(slot) => quote! { #ir::Val::Kw(#slot) },
    }
}

/// The §3.7 capability table: the longhands GPUI's style API expresses
/// natively. Everything else is the honest, reported boundary — except
/// the [`polyfilled`] tier, which the runtime composes from native
/// surfaces (the emit gates accept both; see [`expressible`]).
pub const fn supports(prop: Prop) -> bool {
    matches!(
        prop,
        Prop::Display
            | Prop::Position
            | Prop::Top
            | Prop::Right
            | Prop::Bottom
            | Prop::Left
            | Prop::FlexDirection
            | Prop::FlexWrap
            | Prop::FlexGrow
            | Prop::FlexShrink
            | Prop::JustifyContent
            | Prop::AlignItems
            | Prop::AlignContent
            | Prop::AlignSelf
            | Prop::GridTemplateColumns
            | Prop::GridTemplateRows
            | Prop::GapRow
            | Prop::GapCol
            | Prop::AspectRatio
            | Prop::Width
            | Prop::Height
            | Prop::MinWidth
            | Prop::MinHeight
            | Prop::MaxWidth
            | Prop::MaxHeight
            | Prop::PaddingTop
            | Prop::PaddingRight
            | Prop::PaddingBottom
            | Prop::PaddingLeft
            | Prop::MarginTop
            | Prop::MarginRight
            | Prop::MarginBottom
            | Prop::MarginLeft
            | Prop::BorderTopWidth
            | Prop::BorderRightWidth
            | Prop::BorderBottomWidth
            | Prop::BorderLeftWidth
            | Prop::BorderColor
            | Prop::BorderStyle
            | Prop::BorderTopLeftRadius
            | Prop::BorderTopRightRadius
            | Prop::BorderBottomRightRadius
            | Prop::BorderBottomLeftRadius
            | Prop::Background
            | Prop::Color
            | Prop::FontSize
            | Prop::LineHeight
            | Prop::FontWeight
            | Prop::FontFamily
            | Prop::FontStyle
            | Prop::TextAlign
            | Prop::TextDecorationLine
            | Prop::WhiteSpace
            | Prop::OverflowX
            | Prop::OverflowY
            | Prop::Visibility
            | Prop::Opacity
            | Prop::Cursor
            | Prop::TextOverflow
            // The gradient stops and their positions: GPUI's
            // `linear_gradient` stops carry a colour and a percentage
            // each, the v4 two-stop form. The via stop (three stops)
            // and the direction longhand (whose verdict is the row
            // TEXT's, see [`unsupported_reason`]) stay out of the
            // per-prop table.
            | Prop::GradientFrom
            | Prop::GradientTo
            | Prop::GradientFromPosition
            | Prop::GradientToPosition
    )
}

/// The §3.7 POLYFILL tier: the longhands GPUI has no native slot for
/// but the runtime composes from native surfaces. Today that is the
/// ring family — no ring primitive, but the v4 ring machinery IS a
/// box-shadow list, and GPUI's `Vec<BoxShadow>` carries it
/// (`aeolus_gpui::RingState`, the spread-only bands). The emit gates
/// accept a program whose only non-native longhands are these: the
/// frozen program carries the same `Val`s the runtime's walk
/// polyfills, so nothing is lost in the static form.
pub const fn polyfilled(prop: Prop) -> bool {
    matches!(
        prop,
        Prop::RingWidth | Prop::RingColor | Prop::RingOffsetWidth | Prop::RingOffsetColor
    )
}

/// The emit gates' verdict: a longhand the target takes — native
/// ([`supports`]) or polyfilled ([`polyfilled`]). Everything else
/// refuses into the `Unsupported` diagnostics, the documented
/// boundary.
pub const fn expressible(prop: Prop) -> bool {
    supports(prop) || polyfilled(prop)
}

/// The §5.1 conformance seam's GPUI half: the capability impl reading
/// the SAME `supports`/`polyfilled` tables the emit gates run — the
/// published score is the emitter's own verdicts, never a parallel
/// claim. The per-declaration refinements (the text-dependent verdicts
/// `unsupported_reason` runs) stay at the gates; this is the
/// per-longhand ceiling the docs page renders.
pub struct Caps;

impl Capability for Caps {
    fn name(&self) -> &'static str {
        "gpui"
    }

    fn tier(&self, prop: Prop) -> Tier {
        if supports(prop) {
            Tier::Native
        } else if polyfilled(prop) {
            Tier::Polyfill
        } else {
            Tier::Refused
        }
    }
}

/// The per-DECLARATION verdict: the per-prop tiers ([`expressible`])
/// refined where the prop's answer is the ROW TEXT's. The direction
/// longhand ([`Prop::BackgroundImage`]) expresses exactly when its
/// composed row text names a shape GPUI's `Fill` carries — the
/// `linear-gradient(to …)` directions ([`aeolus_compile::
/// linear_gradient_angle`]) and the `bg-none` clear — while the
/// radial/conic/placeholder texts refuse with the reason the
/// diagnostic renders; the via stop refuses on its own slot shape
/// (GPUI's gradient carries two stops). `None` = expressible.
///
/// `keyword` is the declaration's row text (the annotation a bare
/// `Val::Kw` slot cannot recover): a direction decl with NO text
/// cannot be gate-checked and refuses rather than guessing.
pub fn unsupported_reason(prop: Prop, keyword: &str) -> Option<&'static str> {
    match prop {
        // ---- the motion layer (the plan §3.7 transition/animate row) ---
        //
        // GPUI's style surface carries no transition slot, and its
        // animation story (`with_animation`) needs an element id + state
        // the frozen static program does not name — the plan's own
        // "Phase 6 motion layer" boundary, reported per longhand, the
        // CSS/interp/plan arms rendering the real output.
        Prop::TransitionProperty | Prop::TransitionTimingFunction => Some(
            "GPUI's style surface carries no transition slot — the motion \
             families stay the CSS/interp/plan arms' output (plan §3.7)",
        ),
        Prop::TransitionDuration | Prop::TransitionDelay => Some(
            "GPUI has no transition duration/delay slot — the motion \
             families stay the CSS/interp/plan arms' output (plan §3.7)",
        ),
        Prop::Animation => Some(
            "GPUI's with_animation needs an element id + state the frozen \
             static program does not name (plan §3.7's motion row)",
        ),
        Prop::BackgroundImage => {
            if keyword.is_empty() {
                Some(
                    "the direction row's text is the annotation the bare IR cannot recover — \
                     pass the keyword slice the pipeline freezes beside the layers",
                )
            } else if keyword == "none" || aeolus_compile::linear_gradient_angle(keyword).is_some()
            {
                None
            } else {
                Some(
                    "GPUI's Fill carries only the linear-gradient directions (bg-linear-to-*) \
                     — the radial/conic/placeholder texts have no slot",
                )
            }
        }
        Prop::GradientVia => Some(
            "GPUI linear gradients carry two stops — the via stop has no slot (v4's \
             three-stop form stays the documented boundary)",
        ),
        _ => {
            if expressible(prop) {
                None
            } else {
                Some("GPUI has no native slot")
            }
        }
    }
}
