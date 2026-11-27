//! The builder-chain emitter — the V1 architecture plan's emit (a) END
//! FORM (`docs/AEOLUS_V1_ARCHITECTURE.md` §3.2 status, the rust-tokens
//! paragraph: "The backend METHOD calls the emit (a) end form implies
//! (`FlexDirection::Column`, builder chains) remain the P3 codegen
//! crate's surface"; the same deferral stands in the §8 P2 row and in
//! `reports/10-tdd-evidence/19-rust-tokens-arm.md`'s boundary list).
//!
//! Where [`emit_static_program`](crate::emit_static_program) renders
//! the frozen `aeolus_rt::StaticProgram` the walk applies per frame,
//! this module renders the program AS the GPUI builder-call chain —
//! the code a developer would have written by hand, the `Emit`
//! trait's own documented shape ("renders the target's builder
//! calls"). One compiled program in, one closure literal out:
//!
//! ```text
//! |s| s.flex().flex_col().pt(::aeolus_gpui::chain::rems(1.0)).hover(|s| s.bg(..rgba(0x..)))
//! ```
//!
//! The contract, complete for its stated scope:
//!
//! * **Base cascade.** The unconditional layers fold in cascade order
//!   (last call wins per longhand — the walk's own front-to-back
//!   reading; the `!important` origin's layers sort AFTER their normal
//!   siblings, so order carries the win here too). The four native
//!   state variants (`hover:`/`active:`/`focus:`/`focus-visible:`) are
//!   the conditional face the plan §3.5 table names
//!   (`.hover/.active/.focus/.focus_visible(|s| …)`); every other
//!   conditional family refuses typed — the frozen-program walk's
//!   probe seams answer them.
//! * **Compile-time values.** `Val::Rgba` renders as a `rgba(0x..)`
//!   literal (the walk's own conversion), the numeric/length slots
//!   render their `px`/`rems`/`relative` literals, and the row-text
//!   keyword slots resolve exactly the walk's row-length resolution.
//!   A `Val::Tok`/`Val::ModePair` slot resolves under the runtime
//!   `TokenTable` — the chain refuses typed (bake the theme instead:
//!   the T0/T1 tiers freeze literals).
//! * **Plain builder methods only.** The walk expresses the shadow,
//!   ring, gradient, filter and space/divide families through its
//!   STATE ACCUMULATORS (`ShadowState`/`RingState`/`GradState`/
//!   `FilterState`/`TwSpaceExt`) and the decoration colour/thickness
//!   longhands through text-style mutation — no plain `Styled` method
//!   carries them, so the chain refuses with the lane's name (the
//!   frozen-program lane keeps the real output).
//! * **All-or-nothing.** One refused declaration refuses the whole
//!   emit (the honest-failure contract, [`GpuiEmit`]'s shape): never a
//!   partial chain, never a silent drop.
//!
//! The differential target: for a string the chain CAN express, the
//! emitted method/argument sequence mirrors `aeolus_gpui`'s walk arms
//! (`apply_decl` and its helpers) one for one — the parity battery in
//! `crates/aeolus-gpui/tests/builder_parity.rs` pins the applied
//! `StyleRefinement` field-by-field against the walk's answer, and the
//! constructor paths ride `aeolus_gpui::chain` (the crate users depend
//! on) so a consumer needs no `gpui` naming of its own.

use proc_macro2::TokenStream;
use quote::quote;

use crate::Diagnostics;
use crate::decl_unsupported_reason;
use aeolus_codegen::Emit;
use aeolus_codegen::Unsupported;
use aeolus_compile::Config;
use aeolus_compile::keyword_texts;
use aeolus_ir::{Cond, Program, Prop, Val};
use aeolus_theme::{ThemeSource, TokenTable};

/// The conditional-layer refusal: the four native state methods are
/// the chain's cond face (§3.5's own GPUI mapping), every other
/// conditional family rides the frozen-program walk's probe seams.
const COND_UNSUPPORTED: &str = "the conditional layer rides the frozen-program walk's probe seams \
     (the AttrProbe/ContainerProbe/DescendantProbe/SupportsProbe lanes and the frame Env) — the \
     builder chain renders the unconditional cascade and the four native state methods (plan §3.5)";

/// The theme-token slot's refusal: the runtime `TokenTable` resolves
/// `Val::Tok`/`Val::ModePair` under the frame env — the chain carries
/// compile-time colour literals only.
const TOKEN_UNSUPPORTED: &str = "the theme-token slot resolves under the runtime TokenTable (the \
     walk's tw_env_theme lane, the interp's TokenSource seam) — the builder chain carries \
     compile-time colour literals only (bake the theme: the T0/T1 tiers freeze literals)";

/// The state-accumulator families' refusal: the walk composes them
/// through its own state (the accumulators' names ride the reason —
/// the message points at the lane that works, the filter families'
/// convention).
const ACCUMULATOR_UNSUPPORTED: &str = "the walk composes this family through its state \
     accumulators (ShadowState/RingState/GradState/FilterState/TwSpaceExt) or the text-style \
     mutation the plain builder methods do not carry — the frozen-program lane keeps the real \
     output";

/// The builder-chain emit target: one program in, the builder-call
/// closure out ([`Emit`]'s second GPUI shape beside [`GpuiEmit`]).
///
/// `keywords` is parallel to the flattened declarations (layer-major
/// order — the annotation convention [`GpuiEmit`] freezes); a missing
/// entry renders as `""` (no row text, the row-length miss).
#[derive(Clone, Copy, Debug, Default)]
pub struct GpuiBuilderEmit<'a> {
    /// Row texts parallel to the flattened declarations (layer-major
    /// order — the keyword-slot resolution reads them).
    pub keywords: &'a [&'a str],
}

impl<'a> GpuiBuilderEmit<'a> {
    /// Build the target over the annotation slice.
    pub const fn new(keywords: &'a [&'a str]) -> Self {
        Self { keywords }
    }
}

impl Emit for GpuiBuilderEmit<'_> {
    type Output = TokenStream;

    fn emit(&self, program: &Program) -> Result<Self::Output, Vec<Unsupported>> {
        emit_builder(program, self.keywords)
    }
}

/// Render one compiled program as the GPUI builder-call closure.
/// `keywords` is parallel to the flattened declarations (layer-major
/// order). The static emit's per-declaration gate
/// ([`decl_unsupported_reason`]) runs FIRST — its refusals stand —
/// then the chain's own gate refuses the conditional families, the
/// theme-token slots and the state-accumulator longhands; any refusal
/// refuses the whole emit.
///
/// # Errors
/// Every longhand the chain cannot express, in cascade order (the
/// static gate's verdicts beside the chain gate's own).
pub fn emit_builder(program: &Program, keywords: &[&str]) -> Result<TokenStream, Vec<Unsupported>> {
    // One flat pass in layer-major order (the keyword alignment's own
    // convention): gate every declaration, collect the renderable
    // tails grouped by their layer's condition.
    let mut refusals: Vec<Unsupported> = Vec::new();
    let mut groups: Vec<(Cond, Vec<TokenStream>)> = Vec::new();
    let mut flat = 0usize;
    for layer in program.layers {
        for decl in layer.decls {
            let keyword = keywords.get(flat).copied().unwrap_or_default();
            flat += 1;
            if let Some(reason) = cond_unsupported_reason(layer.cond) {
                refusals.push(Unsupported::new(decl.prop, reason));
                continue;
            }
            if let Some(reason) = decl_unsupported_reason(decl.prop, decl.val, keyword) {
                refusals.push(Unsupported::new(decl.prop, reason));
                continue;
            }
            if is_theme_token(decl.val) {
                refusals.push(Unsupported::new(decl.prop, TOKEN_UNSUPPORTED));
                continue;
            }
            match render_decl(decl.prop, decl.val, keyword) {
                Ok(Some(tail)) => match groups.iter_mut().find(|(cond, _)| *cond == layer.cond) {
                    Some(group) => group.1.push(tail),
                    None => groups.push((layer.cond, vec![tail])),
                },
                Ok(None) => {}
                Err(reason) => refusals.push(Unsupported::new(decl.prop, reason)),
            }
        }
    }
    if !refusals.is_empty() {
        refusals.sort_by_key(|entry| entry.prop);
        refusals.dedup();
        return Err(refusals);
    }
    // The closure: the base group is the outer chain, each native
    // state group its `.method(|s| …)` closure appended in
    // first-appearance (cascade) order.
    let mut chain: TokenStream = quote!(s);
    for (cond, tails) in &groups {
        if *cond == Cond::Base {
            for tail in tails {
                chain.extend(tail.clone());
            }
        } else {
            let inner = quote!(s #(#tails)*);
            let method = state_method(*cond);
            chain.extend(quote!(.#method(|s| #inner)));
        }
    }
    Ok(quote!(|s| #chain))
}

/// The chain's conditional face: the four native state methods §3.5's
/// GPUI mapping column names. `None` = the condition is the base
/// cascade (the outer chain). Some other `Cond` arm = the typed
/// refusal ([`COND_UNSUPPORTED`]).
fn cond_unsupported_reason(cond: Cond) -> Option<&'static str> {
    match cond {
        Cond::Base | Cond::Hover | Cond::Active | Cond::Focus | Cond::FocusVisible => None,
        _ => Some(COND_UNSUPPORTED),
    }
}

/// True when the value resolves under the runtime `TokenTable` — the
/// `Val::is_token` reading (the 0-alloc contract tests key off the
/// same slots).
fn is_theme_token(val: Val) -> bool {
    val.is_token()
}

/// The native state method's name for one carried `Cond` (the
/// `.hover/.active/.focus/.focus_visible` set).
fn state_method(cond: Cond) -> proc_macro2::Ident {
    let name = match cond {
        Cond::Active => "active",
        Cond::Focus => "focus",
        Cond::FocusVisible => "focus_visible",
        _ => "hover",
    };
    quote::format_ident!("{}", name)
}

/// Render ONE declaration as its method-call tail (`.method(args)`) —
/// the walk arm's own call, mirrored. `Ok(None)` = the walk's no-op
/// arm (the value shape the helpers do not resolve: the chain skips it
/// exactly like the walk). `Err` = the chain-form refusal.
fn render_decl(prop: Prop, val: Val, keyword: &str) -> Result<Option<TokenStream>, &'static str> {
    Ok(match prop {
        Prop::Display => keyword_method(
            keyword,
            &[
                ("none", "hidden"),
                ("flex", "flex"),
                ("inline-flex", "flex"),
                ("grid", "grid"),
                ("inline-grid", "grid"),
            ],
        )
        .or_else(|| Some(quote::format_ident!("block")))
        .map(|m| quote!(.#m())),
        Prop::Position => keyword_method(keyword, &[("absolute", "absolute")])
            .or_else(|| Some(quote::format_ident!("relative")))
            .map(|m| quote!(.#m())),
        Prop::Visibility => keyword_method(
            keyword,
            &[("hidden", "invisible"), ("collapse", "invisible")],
        )
        .or_else(|| Some(quote::format_ident!("visible")))
        .map(|m| quote!(.#m())),
        Prop::FlexDirection => keyword_method(
            keyword,
            &[
                ("column", "flex_col"),
                ("column-reverse", "flex_col_reverse"),
                ("row-reverse", "flex_row_reverse"),
            ],
        )
        .or_else(|| Some(quote::format_ident!("flex_row")))
        .map(|m| quote!(.#m())),
        Prop::FlexWrap => keyword_method(
            keyword,
            &[("wrap", "flex_wrap"), ("wrap-reverse", "flex_wrap_reverse")],
        )
        .or_else(|| Some(quote::format_ident!("flex_nowrap")))
        .map(|m| quote!(.#m())),
        Prop::JustifyContent => keyword_method(
            keyword,
            &[
                ("center", "justify_center"),
                ("flex-end", "justify_end"),
                ("end", "justify_end"),
                ("space-between", "justify_between"),
                ("space-around", "justify_around"),
                ("space-evenly", "justify_evenly"),
            ],
        )
        .or_else(|| Some(quote::format_ident!("justify_start")))
        .map(|m| quote!(.#m())),
        Prop::AlignItems => keyword_method(
            keyword,
            &[
                ("center", "items_center"),
                ("flex-end", "items_end"),
                ("end", "items_end"),
                ("baseline", "items_baseline"),
                ("stretch", "items_stretch"),
            ],
        )
        .or_else(|| Some(quote::format_ident!("items_start")))
        .map(|m| quote!(.#m())),
        Prop::AlignContent => keyword_method(
            keyword,
            &[
                ("center", "content_center"),
                ("flex-end", "content_end"),
                ("end", "content_end"),
                ("space-between", "content_between"),
                ("space-around", "content_around"),
                ("space-evenly", "content_evenly"),
                ("stretch", "content_stretch"),
            ],
        )
        .or_else(|| Some(quote::format_ident!("content_start")))
        .map(|m| quote!(.#m())),
        Prop::AlignSelf => keyword_method(
            keyword,
            &[
                ("center", "self_center"),
                ("flex-start", "self_start"),
                ("start", "self_start"),
                ("flex-end", "self_end"),
                ("end", "self_end"),
                ("baseline", "self_baseline"),
                ("stretch", "self_stretch"),
            ],
        )
        .or_else(|| Some(quote::format_ident!("self_start")))
        .map(|m| quote!(.#m())),
        Prop::FlexGrow => num_method(val, "flex_grow"),
        Prop::FlexShrink => num_method(val, "flex_shrink"),
        Prop::FlexBasis => length_arg(val, keyword).map(|arg| quote!(.flex_basis(#arg))),
        Prop::GridTemplateColumns => grid_count(val, "grid_cols"),
        Prop::GridTemplateRows => grid_count(val, "grid_rows"),
        Prop::GapRow => definite_arg(val, keyword).map(|arg| quote!(.gap_y(#arg))),
        Prop::GapCol => definite_arg(val, keyword).map(|arg| quote!(.gap_x(#arg))),
        Prop::Width => length_arg(val, keyword).map(|arg| quote!(.w(#arg))),
        Prop::Height => length_arg(val, keyword).map(|arg| quote!(.h(#arg))),
        Prop::MinWidth => length_arg(val, keyword).map(|arg| quote!(.min_w(#arg))),
        Prop::MinHeight => length_arg(val, keyword).map(|arg| quote!(.min_h(#arg))),
        Prop::MaxWidth => length_arg(val, keyword).map(|arg| quote!(.max_w(#arg))),
        Prop::MaxHeight => length_arg(val, keyword).map(|arg| quote!(.max_h(#arg))),
        Prop::AspectRatio => num_method(val, "aspect_ratio"),
        Prop::Top => length_arg(val, keyword).map(|arg| quote!(.top(#arg))),
        Prop::Right => length_arg(val, keyword).map(|arg| quote!(.right(#arg))),
        Prop::Bottom => length_arg(val, keyword).map(|arg| quote!(.bottom(#arg))),
        Prop::Left => length_arg(val, keyword).map(|arg| quote!(.left(#arg))),
        Prop::PaddingTop => definite_arg(val, keyword).map(|arg| quote!(.pt(#arg))),
        Prop::PaddingRight => definite_arg(val, keyword).map(|arg| quote!(.pr(#arg))),
        Prop::PaddingBottom => definite_arg(val, keyword).map(|arg| quote!(.pb(#arg))),
        Prop::PaddingLeft => definite_arg(val, keyword).map(|arg| quote!(.pl(#arg))),
        Prop::MarginTop => length_arg(val, keyword).map(|arg| quote!(.mt(#arg))),
        Prop::MarginRight => length_arg(val, keyword).map(|arg| quote!(.mr(#arg))),
        Prop::MarginBottom => length_arg(val, keyword).map(|arg| quote!(.mb(#arg))),
        Prop::MarginLeft => length_arg(val, keyword).map(|arg| quote!(.ml(#arg))),
        Prop::BorderTopWidth => absolute_arg(val, keyword).map(|arg| quote!(.border_t(#arg))),
        Prop::BorderRightWidth => absolute_arg(val, keyword).map(|arg| quote!(.border_r(#arg))),
        Prop::BorderBottomWidth => absolute_arg(val, keyword).map(|arg| quote!(.border_b(#arg))),
        Prop::BorderLeftWidth => absolute_arg(val, keyword).map(|arg| quote!(.border_l(#arg))),
        Prop::BorderColor => color_arg(val).map(|arg| quote!(.border_color(#arg))),
        Prop::BorderStyle => {
            keyword_method(keyword, &[("dashed", "border_dashed")]).map(|m| quote!(.#m()))
        }
        Prop::BorderTopLeftRadius => {
            absolute_arg(val, keyword).map(|arg| quote!(.rounded_tl(#arg)))
        }
        Prop::BorderTopRightRadius => {
            absolute_arg(val, keyword).map(|arg| quote!(.rounded_tr(#arg)))
        }
        Prop::BorderBottomRightRadius => {
            absolute_arg(val, keyword).map(|arg| quote!(.rounded_br(#arg)))
        }
        Prop::BorderBottomLeftRadius => {
            absolute_arg(val, keyword).map(|arg| quote!(.rounded_bl(#arg)))
        }
        Prop::Background => color_arg(val).map(|arg| quote!(.bg(#arg))),
        Prop::Color => color_arg(val).map(|arg| quote!(.text_color(#arg))),
        Prop::FontSize => absolute_arg(val, keyword).map(|arg| quote!(.text_size(#arg))),
        Prop::LineHeight => definite_arg(val, keyword).map(|arg| quote!(.line_height(#arg))),
        Prop::FontWeight => match keyword.parse::<f32>() {
            Ok(weight) => {
                let lit = f32_lit(weight);
                Some(quote!(.font_weight(::aeolus_gpui::chain::FontWeight::from(#lit))))
            }
            Err(_) => None,
        },
        Prop::FontFamily => {
            let family = if keyword.contains("mono") {
                "monospace"
            } else if keyword.contains("serif") {
                "serif"
            } else {
                "sans-serif"
            };
            Some(quote!(.font_family(#family)))
        }
        Prop::FontStyle => match keyword {
            "italic" => Some(quote!(.italic())),
            "normal" | "not-italic" => Some(quote!(.not_italic())),
            _ => None,
        },
        Prop::TextAlign => Some(quote!(.text_align(::aeolus_gpui::text_align_value(#keyword)))),
        Prop::TextDecorationLine => keyword_method(
            keyword,
            &[
                ("underline", "underline"),
                ("line-through", "line_through"),
                ("none", "text_decoration_none"),
                ("no-underline", "text_decoration_none"),
            ],
        )
        .map(|m| quote!(.#m())),
        Prop::TextDecorationStyle => keyword_method(
            keyword,
            &[
                ("wavy", "text_decoration_wavy"),
                ("solid", "text_decoration_solid"),
            ],
        )
        .map(|m| quote!(.#m())),
        Prop::WhiteSpace => match keyword {
            "nowrap" | "pre" | "pre-line" | "pre-wrap" | "break-spaces" => {
                Some(quote!(.whitespace_nowrap()))
            }
            _ => Some(quote!(.whitespace_normal())),
        },
        Prop::TextOverflow => match keyword {
            "ellipsis" | "truncate" => Some(quote!(.text_ellipsis())),
            _ => None,
        },
        Prop::OverflowX => keyword_method(
            keyword,
            &[
                ("hidden", "overflow_x_hidden"),
                ("clip", "overflow_x_hidden"),
                ("scroll", "overflow_x_hidden"),
                ("auto", "overflow_x_hidden"),
            ],
        )
        .map(|m| quote!(.#m())),
        Prop::OverflowY => keyword_method(
            keyword,
            &[
                ("hidden", "overflow_y_hidden"),
                ("clip", "overflow_y_hidden"),
                ("scroll", "overflow_y_hidden"),
                ("auto", "overflow_y_hidden"),
            ],
        )
        .map(|m| quote!(.#m())),
        Prop::Opacity => match val.as_num() {
            Some(n) => {
                let scaled = if n > 1.0 { n / 100.0 } else { n };
                let lit = f32_lit(scaled);
                Some(quote!(.opacity(#lit)))
            }
            None => None,
        },
        Prop::Cursor => keyword_method(
            keyword,
            &[
                ("pointer", "cursor_pointer"),
                ("text", "cursor_text"),
                ("move", "cursor_move"),
                ("not-allowed", "cursor_not_allowed"),
                ("crosshair", "cursor_crosshair"),
            ],
        )
        .map(|m| quote!(.#m())),
        Prop::LineClamp => match val.as_num() {
            Some(n) => {
                // The walk's own count reading: the non-negative floor,
                // the usize overflow the saturating answer.
                let count = usize::try_from(n.max(0.0) as u64).unwrap_or(usize::MAX);
                let count = proc_macro2::Literal::usize_unsuffixed(count);
                Some(quote!(.line_clamp(#count)))
            }
            None => None,
        },
        // The state-accumulator longhands (the static gate passes the
        // closed-world forms; the walk composes them through its own
        // state): the chain refuses with the lane's name.
        _ => return Err(ACCUMULATOR_UNSUPPORTED),
    })
}

/// One keyword-family arm: the row text's method name, the walk
/// helper's own mapping. `None` = the keyword outside the closed
/// family (the walk's no-op arm).
fn keyword_method(keyword: &str, arms: &[(&str, &str)]) -> Option<proc_macro2::Ident> {
    arms.iter()
        .find(|(text, _)| *text == keyword)
        .map(|(_, method)| quote::format_ident!("{}", method))
}

/// The plain-`f32` method arms (`flex_grow`, `aspect_ratio`, …) — the
/// walk's `Val::as_num` numeric reading.
fn num_method(val: Val, method: &str) -> Option<TokenStream> {
    let method = quote::format_ident!("{}", method);
    val.as_num().map(|n| {
        let lit = f32_lit(n);
        quote!(.#method(#lit))
    })
}

/// The grid track-count arms — the walk's own `Val::Num` reading
/// (`.grid_cols(n)`/`.grid_rows(n)`, the u16 cast).
fn grid_count(val: Val, method: &str) -> Option<TokenStream> {
    let method = quote::format_ident!("{}", method);
    match val {
        Val::Num(n) => {
            let count = proc_macro2::Literal::usize_unsuffixed(n as u16 as usize);
            Some(quote!(.#method(#count)))
        }
        _ => None,
    }
}

/// The length-value constructors, mirroring the walk's `length`
/// helper: `rems`/`px`/`relative` literals, the row-text resolution
/// ([`row_length`]) for the `Val::Kw` slots. `None` = the walk's no-op
/// arm.
fn length_arg(val: Val, keyword: &str) -> Option<TokenStream> {
    match val {
        Val::Rem(n) => Some(rems(n)),
        Val::Px(n) => Some(px(n)),
        Val::Pct(n) => Some(relative(n / 100.0)),
        Val::Kw(_) => match row_length(keyword) {
            Val::Rem(n) => Some(rems(n)),
            Val::Px(n) => Some(px(n)),
            Val::Pct(n) => Some(relative(n / 100.0)),
            _ => None,
        },
        _ => None,
    }
}

/// The definite-length arms (`gap_*`, `padding_*`, `line_height`) —
/// the same constructors, the `Into<DefiniteLength>` at the consumer.
fn definite_arg(val: Val, keyword: &str) -> Option<TokenStream> {
    length_arg(val, keyword)
}

/// The absolute-length arms (`border-*`, `text_size`, the radii) — the
/// walk's `absolute` helper: the bare-integer px scale (`border-2` is
/// 2px) beside the rem/px forms.
fn absolute_arg(val: Val, keyword: &str) -> Option<TokenStream> {
    match val {
        Val::Rem(n) => Some(rems(n)),
        Val::Px(n) => Some(px(n)),
        Val::Num(n) => Some(px(n)),
        Val::Kw(_) => match row_length(keyword) {
            Val::Rem(n) => Some(rems(n)),
            Val::Px(n) => Some(px(n)),
            _ => None,
        },
        _ => None,
    }
}

/// The colour arm — the walk's own conversion: the baked `[u8; 4]`
/// bytes as one `rgba(0x..)` literal.
fn color_arg(val: Val) -> Option<TokenStream> {
    match val {
        Val::Rgba([r, g, b, a]) => {
            let hex = u32::from_be_bytes([r, g, b, a]);
            let hex = format!("0x{hex:08X}");
            let hex: TokenStream = hex.parse().expect("a hex literal is a valid token");
            Some(quote!(::aeolus_gpui::chain::rgba(#hex)))
        }
        _ => None,
    }
}

/// The row text's own length form — the walk helper's parse, mirrored:
/// the plain px/rem/% spellings the row tables carry beside the
/// `Val::Kw` slots. `Val::Num(0.0)` = the CALC texts (the CSS arm's
/// own output — the walk does not evaluate expressions and returns no
/// value).
fn row_length(keyword: &str) -> Val {
    if let Some(n) = keyword.strip_suffix("px") {
        n.trim()
            .parse::<f32>()
            .map(Val::Px)
            .unwrap_or(Val::Num(0.0))
    } else if let Some(n) = keyword.strip_suffix("rem") {
        n.trim()
            .parse::<f32>()
            .map(Val::Rem)
            .unwrap_or(Val::Num(0.0))
    } else if let Some(n) = keyword.strip_suffix('%') {
        n.trim()
            .parse::<f32>()
            .map(Val::Pct)
            .unwrap_or(Val::Num(0.0))
    } else {
        // The CALC texts: not a value the chain evaluates — the walk's
        // own `None` arm (never a guessed value).
        Val::Num(0.0)
    }
}

fn rems(n: f32) -> TokenStream {
    let lit = f32_lit(n);
    quote!(::aeolus_gpui::chain::rems(#lit))
}

fn px(n: f32) -> TokenStream {
    let lit = f32_lit(n);
    quote!(::aeolus_gpui::chain::px(#lit))
}

fn relative(n: f32) -> TokenStream {
    let lit = f32_lit(n);
    quote!(::aeolus_gpui::chain::relative(#lit))
}

/// The shortest-round-trip float literal the emitted chain carries
/// (the exact `f32` the walk resolves — the debug form's decimal
/// point keeps it a float literal under inference).
fn f32_lit(n: f32) -> TokenStream {
    format!("{n:?}")
        .parse()
        .expect("the formatted f32 is a valid literal")
}

/// The third-party seam: `lower_class`'s builder twin — the §5
/// integration bullet's `resolve_style()` equivalent. One class
/// literal in, the builder-call closure out (the embedded shadcn bake
/// rides, exactly as [`crate::lower_class`]); a compile failure or a
/// chain refusal lands in the same [`Diagnostics`] shape.
///
/// # Errors
/// [`Diagnostics`] carrying the compile messages and/or the chain's
/// per-longhand refusals.
pub fn lower_class_builder(literal: &str, config: &Config) -> Result<TokenStream, Diagnostics> {
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
    let annotations = keyword_texts(literal, config).map_err(|err| Diagnostics {
        messages: vec![err.to_string()],
        unsupported: Vec::new(),
    })?;
    let keywords = aeolus_compile::keyword_alignment(&compiled, &annotations);
    emit_builder(&compiled.program, &keywords).map_err(|unsupported| Diagnostics {
        messages: Vec::new(),
        unsupported,
    })
}

/// The macro surface's gate: [`GpuiBuilderEmit`]'s refusals over one
/// compiled program (the `audit` twin — the macro calls this before
/// expanding `tw_builder!`, and a non-empty list is a compile error
/// naming the class and the reason).
pub fn audit_builder(program: &Program, keywords: &[&str]) -> Vec<Unsupported> {
    emit_builder(program, keywords).err().unwrap_or_default()
}
