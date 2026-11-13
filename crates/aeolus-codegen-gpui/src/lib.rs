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

use aeolus_codegen::{Emit, Unsupported, ir_path};
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
        let refused: Vec<Unsupported> = program
            .layers
            .iter()
            .flat_map(|layer| layer.decls.iter())
            .filter_map(|decl| {
                (!expressible(decl.prop))
                    .then_some(Unsupported::new(decl.prop, "GPUI has no native slot"))
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
        unsupported: compiled
            .program
            .layers
            .iter()
            .flat_map(|layer| layer.decls.iter())
            .filter_map(|decl| {
                (!expressible(decl.prop))
                    .then_some(Unsupported::new(decl.prop, "GPUI has no native slot"))
            })
            .collect(),
    };
    diagnostics.unsupported.sort_by_key(|entry| entry.prop);
    diagnostics.unsupported.dedup();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let annotations = keyword_texts(literal, config).map_err(|err| Diagnostics {
        messages: vec![err.to_string()],
        unsupported: Vec::new(),
    })?;
    let keywords = aeolus_compile::keyword_alignment(&compiled, &annotations);
    Ok(emit_static_program(&compiled.program, &keywords))
}

/// Render the `aeolus_rt::StaticProgram` expression for one compiled
/// program. `keywords` is parallel to the flattened declarations
/// (layer-major order); a missing entry renders as `""` (no row text).
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
        Cond::Container(bp) => {
            let bp = bp_tokens(bp, ir);
            quote! { #ir::Cond::Container(#bp) }
        }
        Cond::MaxContainer(bp) => {
            let bp = bp_tokens(bp, ir);
            quote! { #ir::Cond::MaxContainer(#bp) }
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
