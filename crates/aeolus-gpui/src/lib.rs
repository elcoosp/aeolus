//! The GPUI runtime for the Aeolus V1 static program (architecture plan
//! §3.1 emit (a)/(b), §3.2 `aeolus-codegen-gpui`, P3).
//!
//! ```ignore
//! use aeolus_gpui::prelude::*;
//!
//! div()
//!     .tw(tw!("flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground"))
//!     .child("Save")
//! ```
//!
//! `tw!` expands to an [`aeolus_rt::StaticProgram`] — the IR layers plus
//! the row-keyword texts — and this crate maps those declarations onto
//! GPUI's native style fields. State layers (`hover:`, `focus:`, ...) go
//! through [`TwStateExt`], which wires GPUI's own interaction closures;
//! [`TwExt::tw`] applies the base + environment-matched layers (dark,
//! breakpoints) for any `Styled` element.
//!
//! Attribute layers (`data-*:`/`aria-*:`) answer through the host state
//! seam (plan §3.5's "typed state bits passed WITH the element", P4's
//! last named piece): [`TwExt::tw_probes`] walks the base + environment
//! layers with the element's attribute bits in hand — the GPUI half of
//! the interp arm's `interp_probes` entry point. The plain [`TwExt::tw`]
//! keeps those layers quiet (the sourceless skip the interp reports as
//! `AttributeUnqueried` notes), and the descendant `has-*` shapes stay
//! the diagnosed boundary with or without bits.
//!
//! Named container queries (`@md/card:`, `@max-lg/sidebar:`) answer
//! through the bounds seam (plan §3.5's `@container` row, the
//! `ContainerProbe` half): [`TwExt::tw_containers`] walks with the
//! named containers' widths in hand — the GPUI half of the interp
//! arm's `interp_containers` entry point, the same AT/strictly-below
//! boundary rules. The unnamed forms (`@md:`) keep answering from the
//! [`Env`]'s nearest-container slot, and the element-bounds WRAPPER —
//! who MEASURES the widths — is the [`TwContainerExt::tw_container`]
//! element beside it: a stateless measuring relay that publishes the
//! width of its own box into the shared [`MeasuredContainers`]
//! hand-off at the layout phase (the prepaint bounds), re-rendering on
//! a measurement delta, while the walk consults the answers and never
//! measures.
//!
//! Motion (`animate-*`) rides the stateful animation lane (plan §3.7's
//! GPUI half, the `with_animation` boundary): [`TwAnimExt::tw_anim`] /
//! [`TwAnimExt::tw_anim_dyn`] take the element id and the animator
//! closure from the HOST — the two inputs the frozen static program
//! cannot carry — and build the [`gpui::Animation`] from the frozen
//! row text (the v4 shorthand `spin 1s linear infinite`). The `tw!`
//! macro's capability gate keeps refusing motion classes at compile
//! time; the lane's programs come from the dynamic lane or a host
//! compile, and `animate-none` renders the element statically.
//!
//! The keyword texts are why the program is sufficient: `hidden` and
//! `block` are both `Prop::Display` + `Val::Kw(0)` in the bare IR, so
//! the runtime reads the row text frozen beside the layer.
//!
//! The ring family rides the §3.7 polyfill: GPUI has no ring primitive,
//! so the four `--tw-ring-*` longhands the walk sees compose into the
//! v4 box-shadow list ([`RingState`], the spread-only `BoxShadow`
//! bands) instead of the silent no-op the un-mapped arm used to be.
//! The shadow family rides the §3.7 native tier beside it: GPUI's own
//! `box_shadow: Vec<BoxShadow>` carries the v4 band shape (the pinned
//! `BoxShadow` even carries the `inset` flag), so the `shadow-*`/
//! `shadow-{color}` families compose into the SAME list through the
//! [`ShadowState`] accumulator — the ring's bands and the preset's
//! bands in one v4-ordered call.

use std::cell::RefCell;

use aeolus_compile::{CompileError, Config, Env, structural_fires};
use aeolus_ir::{Cond, Prop, Val};
use aeolus_rt::{ClassCell, StaticProgram, TokenTable};
use gpui::{
    AbsoluteLength, Animation, AnimationElement, AnimationExt, DefiniteLength, ElementId,
    FontWeight, Hsla, IntoElement, Length, StatefulInteractiveElement, Styled, TextAlign, px,
    relative, rems, rgba,
};

mod containers;
mod filters;
mod grad;
mod motion;
mod ring;
mod shadow;
pub use containers::{ContainerQuery, TwContainerExt, container_rem};
pub use filters::FilterState;
pub use grad::GradState;
pub use motion::{
    AnimSpec, Easing, animation, animation_for, animation_keyword, parse_animation, pulse_animator,
};
pub use ring::RingState;
pub use shadow::{SHADOW_PRESET_TEXTS, ShadowState, parse_shadow_bands};

pub use aeolus_macro::{cn, theme, tw, tw_cn, variants};
pub use aeolus_rt::theme_from_css;
// The attribute-bits seam (plan §3.5): the element's own `data-`/`aria-`
// bits ride the host state surface, so the prelude carries the probe
// trait, the built-in bit table and the bit/kind constructors.
pub use aeolus_rt::{AttrBit, AttrKind, AttrProbe, AttrState, StructFacts, StructProbe};
// The container-bounds seam (plan §3.5's `@container` row): the NAMED
// container queries (`@md/card:`) answer through the host's
// [`ContainerProbe`] source, so the prelude carries the trait, the
// built-in static table and its bound row — plus the element-bounds
// WRAPPER's hand-off ([`MeasuredContainers`], what the
// [`TwContainerExt::tw_container`] measuring element publishes into).
pub use aeolus_rt::{ContainerBound, ContainerProbe, ContainerState, MeasuredContainers};

/// The V1 consumption surface: macros, the program type and the two
/// extension traits.
pub mod prelude {
    pub use crate::{
        AeolusRoot, ContainerQuery, FilterState, GradState, TwAnimExt, TwContainerExt, TwExt,
        TwStateExt, begin_frame, cn, container_rem, set_env, theme, theme_from_css, tw, tw_cn,
        variants,
    };
    pub use aeolus_compile::Env;
    pub use aeolus_rt::{
        AttrBit, AttrKind, AttrProbe, AttrState, ClassCell, ContainerBound, ContainerProbe,
        ContainerState, MeasuredContainers, StaticProgram, StructFacts, StructProbe,
    };
}

thread_local! {
    static ENV: RefCell<Env> = RefCell::new(Env::default());
}

/// Replace the process-wide probe environment used by [`TwExt::tw`].
pub fn set_env(env: Env) {
    ENV.with(|current| *current.borrow_mut() = env);
}

/// The current probe environment.
pub fn env() -> Env {
    ENV.with(|current| *current.borrow())
}

/// Run `f` with `env` installed, restoring the previous value after.
pub fn with_env<R>(env: Env, f: impl FnOnce() -> R) -> R {
    let previous = ENV.with(|current| current.replace(env));
    let result = f();
    ENV.with(|current| *current.borrow_mut() = previous);
    result
}

/// Apply a static program's base + environment layers to any styled
/// element.
pub trait TwExt: Styled + Sized {
    /// Apply the program under the current `env()`.
    fn tw(self, program: StaticProgram) -> Self {
        self.tw_env(program, env())
    }

    /// Apply the program under an explicit probe environment.
    fn tw_env(self, program: StaticProgram, env: Env) -> Self {
        apply_matching(self, &program, &env, None, None, None)
    }

    /// Apply the program under the current `env()`, with the element's
    /// attribute bits answering the `data-*:`/`aria-*:` layers — the
    /// GPUI half of the interp arm's `interp_probes` entry point (plan
    /// §3.5's "typed state bits passed WITH the element", one mask test
    /// per probe).
    ///
    /// `probes` is the [`AttrProbe`] seam, the same shape as the token
    /// seam: the built-in [`AttrState`] is a static bit table, and a
    /// host implements the one question itself (a component-state map,
    /// a bridge over the window's real attribute surface, a test
    /// double). A presence spelling (`data-open:`) fires on any value,
    /// a value spelling (`data-[state=open]:`) on equality, and a miss
    /// is CSS-quiet; the descendant `has-*` shapes stay the diagnosed
    /// boundary (they are not element-own bits) with or without bits.
    fn tw_probes(self, program: StaticProgram, probes: &impl AttrProbe) -> Self {
        self.tw_env_probes(program, env(), probes)
    }

    /// [`TwExt::tw_probes`] under an explicit probe environment — the
    /// `tw_env` pairing, with the attribute bits riding along.
    fn tw_env_probes(self, program: StaticProgram, env: Env, probes: &impl AttrProbe) -> Self {
        apply_matching(
            self,
            &program,
            &env,
            Some(probes as &dyn AttrProbe),
            None,
            None,
        )
    }

    /// Apply the program under the current `env()`, with the element's
    /// DOM-position facts answering the structural layers (`first:`,
    /// `last:`, `odd:`, `even:`, `only:`, `empty:`, `nth-N:`,
    /// `nth-last-N:`) — the GPUI half of the interp arm's
    /// `interp_struct` entry point. `structs` is the [`StructProbe`]
    /// seam (the built-in [`StructFacts`] is one Copy value): the
    /// verdicts are the interp's own `structural_fires`, shared
    /// verbatim, an unknown fact quietly missing its test, and a
    /// sourceless structural layer staying quiet (the interp reports
    /// `StructuralUnqueried`; this walk has no note channel).
    fn tw_struct(self, program: StaticProgram, structs: &impl StructProbe) -> Self {
        self.tw_env_struct(program, env(), structs)
    }

    /// [`TwExt::tw_struct`] under an explicit probe environment — the
    /// `tw_env` pairing, with the DOM-position facts riding along.
    fn tw_env_struct(self, program: StaticProgram, env: Env, structs: &impl StructProbe) -> Self {
        apply_matching(
            self,
            &program,
            &env,
            None,
            None,
            Some(structs as &dyn StructProbe),
        )
    }

    /// [`TwExt::tw_probes`] and [`TwExt::tw_struct`] composed: the
    /// attribute bits AND the DOM-position facts ride one walk (the
    /// interp's `interp_probes_struct` pairing).
    fn tw_probes_struct(
        self,
        program: StaticProgram,
        probes: &impl AttrProbe,
        structs: &impl StructProbe,
    ) -> Self {
        self.tw_env_probes_struct(program, env(), probes, structs)
    }

    /// [`TwExt::tw_probes_struct`] under an explicit probe environment.
    fn tw_env_probes_struct(
        self,
        program: StaticProgram,
        env: Env,
        probes: &impl AttrProbe,
        structs: &impl StructProbe,
    ) -> Self {
        apply_matching(
            self,
            &program,
            &env,
            Some(probes as &dyn AttrProbe),
            None,
            Some(structs as &dyn StructProbe),
        )
    }

    /// Apply the program under the current `env()`, with the named
    /// containers' widths answering the named container-query layers
    /// (`@md/card:`, `@max-lg/sidebar:`) — the GPUI half of the interp
    /// arm's `interp_containers` entry point (plan §3.5's `@container`
    /// row, the bounds seam's walk half). The UNNAMED forms (`@md:`)
    /// keep answering from the [`Env`]'s nearest-container slot; the
    /// NAMED question needs THAT container's width.
    ///
    /// `containers` is the [`ContainerProbe`] seam, the same shape as
    /// the token and attribute seams: the built-in [`ContainerState`]
    /// is a borrowed static table, and a host implements the ONE
    /// question itself (a layout-phase measurement cache, a
    /// component-tree walk, a test double). A hit fires by the same
    /// AT/strictly-below boundary rules the viewport breakpoints obey,
    /// a container the source does not know is a CSS-quiet miss, and a
    /// sourceless walk keeps the named layers quiet (the interp reports
    /// `ContainerUnqueried`; this walk has no note channel). The
    /// element-bounds WRAPPER — who MEASURES the widths — is the
    /// [`TwContainerExt::tw_container`] element beside this walk: it
    /// publishes its measured box into a [`MeasuredContainers`] hand-off,
    /// and this walk consults the answers, it never measures.
    fn tw_containers(self, program: StaticProgram, containers: &impl ContainerProbe) -> Self {
        self.tw_env_containers(program, env(), containers)
    }

    /// [`TwExt::tw_containers`] under an explicit probe environment —
    /// the `tw_env` pairing, with the container bounds riding along.
    fn tw_env_containers(
        self,
        program: StaticProgram,
        env: Env,
        containers: &impl ContainerProbe,
    ) -> Self {
        apply_matching(
            self,
            &program,
            &env,
            None,
            Some(containers as &dyn ContainerProbe),
            None,
        )
    }

    /// [`TwExt::tw_probes`] and [`TwExt::tw_containers`] composed: the
    /// attribute bits AND the named container bounds ride one walk (the
    /// interp's `interp_probes_containers` pairing).
    fn tw_probes_containers(
        self,
        program: StaticProgram,
        probes: &impl AttrProbe,
        containers: &impl ContainerProbe,
    ) -> Self {
        self.tw_env_probes_containers(program, env(), probes, containers)
    }

    /// [`TwExt::tw_probes_containers`] under an explicit probe
    /// environment.
    fn tw_env_probes_containers(
        self,
        program: StaticProgram,
        env: Env,
        probes: &impl AttrProbe,
        containers: &impl ContainerProbe,
    ) -> Self {
        apply_matching(
            self,
            &program,
            &env,
            Some(probes as &dyn AttrProbe),
            Some(containers as &dyn ContainerProbe),
            None,
        )
    }

    /// [`TwExt::tw_containers`] and [`TwExt::tw_struct`] composed: the
    /// named container bounds AND the DOM-position facts ride one walk
    /// (the interp's `interp_containers_struct` pairing).
    fn tw_containers_struct(
        self,
        program: StaticProgram,
        containers: &impl ContainerProbe,
        structs: &impl StructProbe,
    ) -> Self {
        self.tw_env_containers_struct(program, env(), containers, structs)
    }

    /// [`TwExt::tw_containers_struct`] under an explicit probe
    /// environment.
    fn tw_env_containers_struct(
        self,
        program: StaticProgram,
        env: Env,
        containers: &impl ContainerProbe,
        structs: &impl StructProbe,
    ) -> Self {
        apply_matching(
            self,
            &program,
            &env,
            None,
            Some(containers as &dyn ContainerProbe),
            Some(structs as &dyn StructProbe),
        )
    }

    /// [`TwExt::tw_probes`], [`TwExt::tw_containers`] and
    /// [`TwExt::tw_struct`] composed: the attribute bits, the named
    /// container bounds and the DOM-position facts ride ONE walk (the
    /// interp's `interp_probes_containers_struct` pairing).
    fn tw_probes_containers_struct(
        self,
        program: StaticProgram,
        probes: &impl AttrProbe,
        containers: &impl ContainerProbe,
        structs: &impl StructProbe,
    ) -> Self {
        self.tw_env_probes_containers_struct(program, env(), probes, containers, structs)
    }

    /// [`TwExt::tw_probes_containers_struct`] under an explicit probe
    /// environment — the full seam matrix's walk form.
    fn tw_env_probes_containers_struct(
        self,
        program: StaticProgram,
        env: Env,
        probes: &impl AttrProbe,
        containers: &impl ContainerProbe,
        structs: &impl StructProbe,
    ) -> Self {
        apply_matching(
            self,
            &program,
            &env,
            Some(probes as &dyn AttrProbe),
            Some(containers as &dyn ContainerProbe),
            Some(structs as &dyn StructProbe),
        )
    }

    /// The dynamic lane's one-call entrance (plan §3.6's surface (3)):
    /// compile `classes` through [`ClassCell`] — the full V1 pipeline,
    /// the SAME compiler as [`tw!`] — and apply the result under the
    /// current `env()`.
    ///
    /// The cell is the view-state slot: a content+identity match costs
    /// a compare (the per-frame fast path), a change compiles once and
    /// interns. A FAILED compile (only the empty input) keeps the LAST
    /// GOOD program applied — never a panic in a render loop, never a
    /// silent reset: the cell's diagnostics channel carries the reason
    /// ([`ClassCell::diagnostics`]), per-class misses included.
    ///
    /// Colour tokens without a table stay the typed boundary: the
    /// themeless compile lowers what it can and reports the token
    /// miss — pair with [`TwExt::tw_dyn_theme`] for the themed lane.
    fn tw_dyn(self, cell: &ClassCell, classes: &str) -> Self {
        self.tw_dyn_env(cell, classes, env())
    }

    /// [`TwExt::tw_dyn`] under an explicit probe environment — the
    /// `tw_env` pairing of the dynamic lane.
    fn tw_dyn_env(self, cell: &ClassCell, classes: &str, env: Env) -> Self {
        apply_dynamic(self, cell, classes, &env, None)
    }

    /// The dynamic lane's themed entrance: the runtime token table
    /// lowers colour tokens and `/opacity` modifiers exactly as
    /// [`tw!`]'s baked table does — the differential pin rides the
    /// tests (`tests/dyn.rs`).
    fn tw_dyn_theme(self, cell: &ClassCell, classes: &str, tokens: &TokenTable) -> Self {
        self.tw_dyn_theme_env(cell, classes, tokens, env())
    }

    /// [`TwExt::tw_dyn_theme`] under an explicit probe environment —
    /// the themed dynamic lane's full form.
    fn tw_dyn_theme_env(
        self,
        cell: &ClassCell,
        classes: &str,
        tokens: &TokenTable,
        env: Env,
    ) -> Self {
        apply_dynamic(self, cell, classes, &env, Some(tokens))
    }
}

impl<T: Styled> TwExt for T {}

/// Apply a static program's interaction layers through GPUI's native
/// `hover`/`focus`/... closures. Use on a stateful element:
///
/// ```ignore
/// div().id("save").tw(tw!("...")).tw_state(tw!("hover:bg-primary/90"))
/// ```
pub trait TwStateExt: StatefulInteractiveElement + Styled + Sized {
    /// Wire every state layer in `program` to its GPUI interaction slot.
    fn tw_state(self, program: StaticProgram) -> Self {
        let env = env();
        // The base+env walk stays sourceless: the interaction closures
        // are GPUI's own state, the element-own attribute bits ride the
        // `tw_probes` walk instead. The base walk's ring, gradient, grid,
        // shadow and filter states seed every closure — `hover:ring-4`
        // composes with the base's `ring-red-500` colour,
        // `hover:from-blue-500` with the base's `from-red-500` stop,
        // `hover:shadow-md` with the base's `shadow-lg` bands the way
        // CSS's vars persist across state layers, and `hover:grayscale-25`
        // with the base's recorded matrix chain.
        let mut ring = RingState::default();
        let mut grad = GradState::default();
        let mut grid = GridState::default();
        let mut shadow = ShadowState::default();
        let mut filter = FilterState::default();
        let mut this = apply_matching_polyfills(
            self,
            &program,
            &env,
            None,
            None,
            None,
            &mut ring,
            &mut grad,
            &mut grid,
            &mut shadow,
            &mut filter,
        );
        this = this.hover(|style| {
            apply_state_polyfills(style, &program, is_hover, ring, grad, grid, shadow, filter)
        });
        this = this.focus(|style| {
            apply_state_polyfills(style, &program, is_focus, ring, grad, grid, shadow, filter)
        });
        this = this.focus_visible(|style| {
            apply_state_polyfills(
                style,
                &program,
                is_focus_visible,
                ring,
                grad,
                grid,
                shadow,
                filter,
            )
        });
        this = this.active(|style| {
            apply_state_polyfills(style, &program, is_active, ring, grad, grid, shadow, filter)
        });
        this
    }
}

impl<T: StatefulInteractiveElement + Styled> TwStateExt for T {}

/// The motion lane (plan §3.7's GPUI half, the `with_animation`
/// boundary): apply a program's `animate-*` longhand through GPUI's
/// own animation element. The frozen static program cannot carry the
/// two inputs `with_animation` needs — the element id and the animator
/// closure — so they are the HOST's arguments; the lane contributes
/// the [`gpui::Animation`] itself, built from the frozen row text (the
/// v4 shorthand `spin 1s linear infinite`), under the same condition
/// semantics the base walk runs ([`animation_keyword`], the cascade's
/// last matching `Animation` declaration).
///
/// The programs come from the dynamic lane or a host compile: the
/// `tw!` macro's capability gate keeps refusing motion classes at
/// compile time (the static emit's honest refusal stands — plan §3.7),
/// while [`ClassCell`] compiles the same strings at runtime with no
/// gate. `animate-none` — or a program with no matching animation —
/// wraps the element statically: a zero-duration no-op animation, the
/// host's animator never called, no per-frame scheduling.
pub trait TwAnimExt: AnimationExt + Styled + Sized + 'static {
    /// Apply the program's winning animation under the current `env()`.
    /// The `animator` is the visual mapping GPUI calls per frame with
    /// the EASED delta — element-specific (rotate/scale/translate have
    /// no `Styled` slot), and [`pulse_animator`] is the built-in
    /// opacity preset.
    fn tw_anim(
        self,
        program: StaticProgram,
        id: impl Into<ElementId>,
        animator: impl Fn(Self, f32) -> Self + 'static,
    ) -> AnimationElement<Self> {
        self.tw_anim_env(program, env(), id, animator)
    }

    /// [`TwAnimExt::tw_anim`] under an explicit probe environment —
    /// the `tw_env` pairing of the motion lane.
    fn tw_anim_env(
        self,
        program: StaticProgram,
        env: Env,
        id: impl Into<ElementId>,
        animator: impl Fn(Self, f32) -> Self + 'static,
    ) -> AnimationElement<Self> {
        let animation = animation_keyword(&program, &env)
            .and_then(animation_for)
            .unwrap_or_else(|| Animation::new(std::time::Duration::ZERO));
        self.with_animation(id, animation, animator)
    }

    /// The motion lane's dynamic entrance (plan §3.6's surface (3)
    /// pairing): compile `classes` through [`ClassCell`] — the full V1
    /// pipeline, no GPUI capability gate — and apply the winning
    /// animation. A FAILED compile keeps the cell's LAST GOOD program
    /// (the honest per-frame degradation [`TwExt::tw_dyn`] runs).
    fn tw_anim_dyn(
        self,
        cell: &ClassCell,
        classes: &str,
        id: impl Into<ElementId>,
        animator: impl Fn(Self, f32) -> Self + 'static,
    ) -> AnimationElement<Self> {
        self.tw_anim_theme_dyn(cell, classes, None, id, animator)
    }

    /// [`TwAnimExt::tw_anim_dyn`] with a runtime token table — the
    /// themed dynamic lane's full form (`tw_dyn_theme`'s pairing).
    fn tw_anim_theme_dyn(
        self,
        cell: &ClassCell,
        classes: &str,
        tokens: Option<&TokenTable>,
        id: impl Into<ElementId>,
        animator: impl Fn(Self, f32) -> Self + 'static,
    ) -> AnimationElement<Self> {
        let compiled: Result<StaticProgram, CompileError> = match tokens {
            Some(tokens) => cell.get_theme(classes, tokens, &Config::default()),
            None => cell.get(classes, &Config::default()),
        };
        let program = match compiled {
            Ok(program) => program,
            Err(_) => cell.program(),
        };
        self.tw_anim_env(program, env(), id, animator)
    }
}

impl<T: AnimationExt + Styled + 'static> TwAnimExt for T {}

// ---------------------------------------------------------------------------
// Condition matching
// ---------------------------------------------------------------------------

fn is_hover(cond: Cond) -> bool {
    matches!(cond, Cond::Hover | Cond::GroupHover | Cond::PeerHover)
}

fn is_focus(cond: Cond) -> bool {
    matches!(cond, Cond::Focus | Cond::GroupFocus | Cond::PeerFocus)
}

fn is_focus_visible(cond: Cond) -> bool {
    matches!(cond, Cond::FocusVisible)
}

fn is_active(cond: Cond) -> bool {
    matches!(cond, Cond::Active)
}

fn env_matches(cond: Cond, env: &Env) -> bool {
    match cond {
        Cond::Base => true,
        Cond::Dark => env.dark,
        Cond::Hover => env.hover,
        Cond::Focus => env.focus,
        Cond::FocusVisible => env.focus_visible,
        Cond::FocusWithin => env.focus_within,
        Cond::Active => env.active,
        Cond::Disabled => env.disabled,
        Cond::Visited => env.visited,
        Cond::Checked => env.checked,
        Cond::PlaceholderShown => env.placeholder_shown,
        // The validation states are element state: the env answers (the
        // interp's `decide` semantics shared verbatim — one semantics,
        // three walks; `invalid:` on the constraint fact, `valid:` its
        // complement, the `user-` pair gating on the interaction fact
        // the HTML spec requires; the unset bits are the quiet misses a
        // selector that does not match is).
        Cond::Valid => !env.validation_invalid,
        Cond::Invalid => env.validation_invalid,
        Cond::UserValid => env.user_validated && !env.validation_invalid,
        Cond::UserInvalid => env.user_validated && env.validation_invalid,
        // The element-state family is element state: the env answers
        // (the interp's `decide` semantics shared verbatim — one
        // semantics, three walks; the target/indeterminate/autofill
        // layers on their ONE fact each, `read-only:` the editability
        // fact's complement, the `valid:` arm's own reading; the
        // unset bits are the quiet misses a selector that does not
        // match is).
        Cond::Target => env.url_target,
        Cond::Indeterminate => env.indeterminate,
        Cond::Autofill => env.autofilled,
        Cond::ReadOnly => !env.read_write,
        // The widget-state family is element state: the env answers
        // (the interp's `decide` semantics shared verbatim — one
        // semantics, three walks; the open/inert/default layers on
        // their ONE affirmative fact each, the `target:` arm's own
        // reading; the unset bits are the quiet misses a selector that
        // does not match is).
        Cond::Open => env.opened,
        Cond::Inert => env.inert,
        Cond::Default => env.form_default,
        // The form-state family is element state: the env answers (the
        // interp's `decide` semantics shared verbatim — one semantics,
        // three walks; the matches-neither pairs gate on their
        // membership fact (`form_associated`, `range_bounded`) beside
        // the pair's shared state bit (`required_field`, `range_in`),
        // the unset membership the quiet miss a selector that does not
        // match is).
        Cond::Enabled => env.form_associated && !env.disabled,
        Cond::Required => env.form_associated && env.required_field,
        Cond::Optional => env.form_associated && !env.required_field,
        Cond::InRange => env.range_bounded && env.range_in,
        Cond::OutOfRange => env.range_bounded && !env.range_in,
        // The pseudo-element tier styles a DIFFERENT render target, not
        // element state — the env never answers a pseudo layer (the
        // CSS/interp arms carry the targets' own cascades).
        Cond::Selection
        | Cond::Placeholder
        | Cond::Before
        | Cond::After
        | Cond::Marker
        | Cond::FirstLetter
        | Cond::FirstLine
        | Cond::FileSelector
        | Cond::Backdrop
        | Cond::DetailsContent => false,
        Cond::GroupHover => env.group_hover,
        Cond::GroupFocus => env.group_focus,
        // The composition tier answers from the env like the landed
        // pair (the interp's `decide` semantics shared verbatim — the
        // composed state one relation up, the unset bits the quiet
        // misses a selector that does not match is).
        Cond::GroupFocusVisible => env.group_focus_visible,
        Cond::GroupFocusWithin => env.group_focus_within,
        Cond::GroupActive => env.group_active,
        Cond::GroupDisabled => env.group_disabled,
        Cond::GroupVisited => env.group_visited,
        Cond::GroupChecked => env.group_checked,
        Cond::PeerHover => env.peer_hover,
        Cond::PeerFocus => env.peer_focus,
        Cond::PeerFocusVisible => env.peer_focus_visible,
        Cond::PeerFocusWithin => env.peer_focus_within,
        Cond::PeerActive => env.peer_active,
        Cond::PeerDisabled => env.peer_disabled,
        Cond::PeerVisited => env.peer_visited,
        Cond::PeerChecked => env.peer_checked,
        // The attribute shapes route through `cond_matches`'s probe
        // arms (the env alone has no answer for them); the guard keeps
        // the match total so a new `Cond` variant fails to compile here
        // instead of silently skipping.
        Cond::Data { .. } | Cond::Aria { .. } | Cond::Has(_) => {
            unreachable!("cond_matches routes the attribute shapes to the probe arms")
        }
        Cond::GroupHas(_) | Cond::PeerHas(_) => {
            unreachable!("cond_matches routes the attribute shapes to the probe arms")
        }
        // The structural shape routes through `cond_matches`'s seam arm
        // the same way (the facts are not frame state — the env alone
        // has no answer for them).
        Cond::Structural(_) => {
            unreachable!("cond_matches routes the structural shapes to the seam arm")
        }
        // The media-feature preferences are frame state: the env
        // answers (the interp's `decide` semantics shared verbatim —
        // one semantics, three walks; the unset bit is the quiet miss a
        // media query that does not hold is).
        Cond::Media(pref) => match pref {
            aeolus_ir::Pref::MotionSafe => !env.reduced_motion,
            aeolus_ir::Pref::MotionReduce => env.reduced_motion,
            aeolus_ir::Pref::ContrastMore => env.more_contrast,
            aeolus_ir::Pref::ContrastLess => env.less_contrast,
            aeolus_ir::Pref::ForcedColors => env.forced_colors,
            aeolus_ir::Pref::Print => env.print_media,
            aeolus_ir::Pref::PointerFine => env.pointer_fine,
            aeolus_ir::Pref::PointerCoarse => env.pointer_coarse,
            aeolus_ir::Pref::PointerNone => env.pointer_none,
            aeolus_ir::Pref::AnyPointerFine => env.any_pointer_fine,
            aeolus_ir::Pref::AnyPointerCoarse => env.any_pointer_coarse,
            aeolus_ir::Pref::AnyPointerNone => env.any_pointer_none,
            aeolus_ir::Pref::Portrait => env.portrait,
            aeolus_ir::Pref::Landscape => !env.portrait,
            aeolus_ir::Pref::InvertedColors => env.inverted_colors,
            aeolus_ir::Pref::NoScript => env.scripting_none,
        },
        // The starting-style at-rule is frame state: the env answers
        // (the interp's `decide` semantics shared verbatim — the unset
        // bit is the quiet miss a frame the element is not in is).
        Cond::Starting => env.starting_style,
        // The direction variants are frame state: the env answers (the
        // interp's `decide` semantics shared verbatim — `rtl:` on the
        // fact, `ltr:` its complement; the unset bit is the quiet miss
        // a selector that does not match is).
        Cond::Dir(dir) => match dir {
            aeolus_ir::Direction::Ltr => !env.direction_rtl,
            aeolus_ir::Direction::Rtl => env.direction_rtl,
        },
        // The supports shape routes through `cond_matches`'s seam arm
        // the same way (the render target's capability set is not frame
        // state — the env alone has no answer for it).
        Cond::Supports(_) => {
            unreachable!("cond_matches routes the supports shapes to the seam arm")
        }
        Cond::Bp(bp) | Cond::Min(bp) => env.viewport_rem >= bp.rem(),
        Cond::Max(bp) => env.viewport_rem < bp.rem(),
        // The container queries: the unnamed form answers from the
        // nearest eligible container the Env carries; the NAMED form
        // routes through `cond_matches`'s seam arm (that container's
        // width is not frame state — the env alone has no answer for
        // it), the guard keeps the match total so a new `Cond` variant
        // fails to compile here instead of silently skipping.
        Cond::Container { bp, name: None } => env.container_rem.is_some_and(|rem| rem >= bp.rem()),
        Cond::MaxContainer { bp, name: None } => {
            env.container_rem.is_some_and(|rem| rem < bp.rem())
        }
        Cond::Container { name: Some(_), .. } | Cond::MaxContainer { name: Some(_), .. } => {
            unreachable!("cond_matches routes the named container shapes to the seam arm")
        }
    }
}

// ---------------------------------------------------------------------------
// The walk
// ---------------------------------------------------------------------------

/// The one condition test the walk applies: the element-own attribute
/// probes answer through the host's probe source, the named container
/// queries through the host's bounds source (plan §3.5's `@container`
/// row), everything else through the env.
fn cond_matches(
    cond: Cond,
    env: &Env,
    probes: Option<&dyn AttrProbe>,
    containers: Option<&dyn ContainerProbe>,
    structs: Option<&dyn StructProbe>,
) -> bool {
    match cond {
        // The element-own probes answer through the seam — the interp's
        // `decide` semantics applied at the host: a presence spelling
        // fires on any value, a value spelling on equality, a miss is
        // CSS-quiet. With no source the layers stay quiet (the interp
        // reports `AttributeUnqueried` notes; this walk has no note
        // channel, so the selector-miss skip stands).
        Cond::Data { name, value } => match probes {
            Some(source) => match source.probe(AttrKind::Data, name) {
                Some(answer) => value.is_none_or(|folded| answer == folded),
                None => false,
            },
            None => false,
        },
        Cond::Aria { name, value } => match probes {
            Some(source) => match source.probe(AttrKind::Aria, name) {
                Some(answer) => value.is_none_or(|folded| answer == folded),
                None => false,
            },
            None => false,
        },
        // The descendant shapes (`has-`/`group-has-`/`peer-has-`) are
        // NOT element-own attribute bits — the plan §3.5 row keeps them
        // the diagnosed boundary (the interp reports them as notes), so
        // the walk skips them with or without a source.
        Cond::Has(_) | Cond::GroupHas(_) | Cond::PeerHas(_) => false,
        // The supports conditions answer through the interp arm's
        // SupportsProbe seam — the render target's capability set is
        // not frame state (the Env cannot answer it), and this walk
        // consults no supports source: the layers stay quiet (the
        // interp reports `SupportsUnqueried`; this walk has no note
        // channel, so the selector-miss skip stands).
        Cond::Supports(_) => false,
        // The structural probes answer through the DOM-position seam
        // (the interp's `structural_fires` verdict, shared verbatim —
        // one semantics, three walks). With no source the layers stay
        // quiet (the interp reports `StructuralUnqueried`; this walk
        // has no note channel, so the selector-miss skip stands).
        Cond::Structural(test) => match structs {
            Some(source) => structural_fires(test, source.structure()),
            None => false,
        },
        // The NAMED container queries answer through the bounds seam
        // (the interp's `decide` semantics shared verbatim — one
        // semantics, three walks): a hit fires by the same
        // AT/strictly-below boundary rules the viewport breakpoints
        // obey, a container the source does not know is a CSS-quiet
        // miss. With no source the layers stay quiet (the interp
        // reports `ContainerUnqueried`; this walk has no note channel,
        // so the selector-miss skip stands). The unnamed forms never
        // reach this arm — `env_matches` answers them from the Env.
        Cond::Container {
            bp,
            name: Some(name),
        } => match containers {
            Some(source) => source.container(name).is_some_and(|rem| rem >= bp.rem()),
            None => false,
        },
        Cond::MaxContainer {
            bp,
            name: Some(name),
        } => match containers {
            Some(source) => source.container(name).is_some_and(|rem| rem < bp.rem()),
            None => false,
        },
        other => env_matches(other, env),
    }
}

fn apply_matching<S: Styled>(
    style: S,
    program: &StaticProgram,
    env: &Env,
    probes: Option<&dyn AttrProbe>,
    containers: Option<&dyn ContainerProbe>,
    structs: Option<&dyn StructProbe>,
) -> S {
    apply_matching_polyfills(
        style,
        program,
        env,
        probes,
        containers,
        structs,
        &mut RingState::default(),
        &mut GradState::default(),
        &mut GridState::default(),
        &mut ShadowState::default(),
        &mut FilterState::default(),
    )
}

/// [`apply_matching`] with the polyfill accumulators AND the grid
/// placement state threaded through — the shared walk body, so the base
/// walk ([`TwExt::tw`]) and the [`TwStateExt`] closures compose the
/// SAME `--tw-ring-*`, `--tw-gradient-*`, grid-placement, `shadow-*`
/// and filter-chain state and the interaction closures inherit the
/// base's bands, stops, placement, shadow bands and matrix chain
/// instead of resetting them.
fn apply_matching_polyfills<S: Styled>(
    mut style: S,
    program: &StaticProgram,
    env: &Env,
    probes: Option<&dyn AttrProbe>,
    containers: Option<&dyn ContainerProbe>,
    structs: Option<&dyn StructProbe>,
    ring: &mut RingState,
    grad: &mut GradState,
    grid: &mut GridState,
    shadow: &mut ShadowState,
    filter: &mut FilterState,
) -> S {
    for (layer_index, layer) in program.layers.iter().enumerate() {
        if !cond_matches(layer.cond, env, probes, containers, structs) {
            continue;
        }
        for (decl_index, decl) in layer.decls.iter().enumerate() {
            let keyword = program.keyword(layer_index, decl_index).unwrap_or("");
            // The polyfill families ride their accumulators: the four
            // `--tw-ring-*` longhands compose into ONE box-shadow list
            // and the gradient longhands into ONE linear gradient,
            // each applied after the walk (the composed surfaces a
            // `shadow()`/`bg()` call replaces wholesale), and the text
            // colour feeds the ring's `currentColor` default.
            if is_ring_prop(decl.prop) {
                ring.apply(decl.prop, decl.val, keyword);
                continue;
            }
            if is_grad_prop(decl.prop) {
                grad.apply(decl.prop, decl.val, keyword);
                continue;
            }
            // The grid-placement longhands ride their accumulator: the
            // span families are PAIR-shaped (see [`GridState`]), so
            // they compose after the walk like the other accumulated
            // surfaces.
            if is_grid_prop(decl.prop) {
                grid.apply(decl.prop, decl.val);
                continue;
            }
            // The shadow longhands ride their accumulator the same
            // way: the `shadow-*` presets and the `shadow-{color}`
            // recolour compose into ONE box-shadow list after the
            // walk (the v4 band list, the [`ShadowState`] machinery).
            if is_shadow_prop(decl.prop) {
                shadow.apply(decl.prop, decl.val, keyword);
                continue;
            }
            // The filter longhands ride THEIR accumulator (the §3.7
            // row's polyfill half): the matrix chain records, the
            // drop-shadow bands parse, the spatial/backdrop halves
            // land in the state's documented no-op arms.
            if is_filter_prop(decl.prop) {
                filter.apply(decl.prop, decl.val, keyword);
                continue;
            }
            // The colour slots the matrix chain recolours record
            // beside the other states' reads (the chain re-applies
            // them filtered after the walk).
            if matches!(decl.prop, Prop::Background | Prop::BorderColor) {
                filter.apply(decl.prop, decl.val, keyword);
            }
            if decl.prop == Prop::Color {
                ring.apply(decl.prop, decl.val, keyword);
                shadow.apply(decl.prop, decl.val, keyword);
                filter.apply(decl.prop, decl.val, keyword);
            }
            style = apply_decl(style, decl.prop, decl.val, keyword);
        }
    }
    apply_polyfills(style, ring, grad, grid, shadow, filter)
}

/// The four `--tw-ring-*` longhands the polyfill owns (the registry's
/// ring rows lower into exactly these; `ring-inset` lowers into
/// `Prop::BoxShadow` and stays the reported boundary — its bare
/// `inset` text is outside the shadow tier's closed-world preset
/// forms, so the emit gate keeps refusing it).
fn is_ring_prop(prop: Prop) -> bool {
    matches!(
        prop,
        Prop::RingWidth | Prop::RingColor | Prop::RingOffsetWidth | Prop::RingOffsetColor
    )
}

/// The shadow tier's longhands (plan §3.7's shadow row): the composed
/// preset band list (`Prop::BoxShadow`) and the `shadow-{color}`
/// recolour (`Prop::ShadowColor`) — the [`ShadowState`] machinery.
fn is_shadow_prop(prop: Prop) -> bool {
    matches!(prop, Prop::BoxShadow | Prop::ShadowColor)
}

/// The gradient family the §3.7 lane owns: the direction row's
/// `BackgroundImage` text plus the `--tw-gradient-*` stop and position
/// longhands (`GradientVia` rides too — the accumulator records it and
/// composes NOTHING, never a flattened three-stop misreading).
fn is_grad_prop(prop: Prop) -> bool {
    matches!(
        prop,
        Prop::BackgroundImage
            | Prop::GradientFrom
            | Prop::GradientVia
            | Prop::GradientTo
            | Prop::GradientFromPosition
            | Prop::GradientViaPosition
            | Prop::GradientToPosition
    )
}

/// The grid-placement longhands the placement state owns (the registry's
/// start/end/span rows lower into exactly these; the auto-track and
/// auto-flow longhands carry no GPUI slot and stay the reported
/// boundary).
fn is_grid_prop(prop: Prop) -> bool {
    matches!(
        prop,
        Prop::GridColumnStart | Prop::GridColumnEnd | Prop::GridRowStart | Prop::GridRowEnd
    )
}

/// The filter family's longhands (the §3.7 filter row): the nine
/// polyfillable forms (the seven colour-matrix matrices, the
/// `drop-shadow-*` presets, the composed `filter` root) ride the
/// [`FilterState`] machinery; the spatial half (`blur-*`) and the
/// `backdrop-*` mirrors land in the state's documented no-op arms (the
/// walk's skip — the static gate refuses them typed).
fn is_filter_prop(prop: Prop) -> bool {
    matches!(
        prop,
        Prop::FilterBlur
            | Prop::FilterBrightness
            | Prop::FilterContrast
            | Prop::FilterGrayscale
            | Prop::FilterHueRotate
            | Prop::FilterInvert
            | Prop::FilterSaturate
            | Prop::FilterSepia
            | Prop::FilterDropShadow
            | Prop::Filter
            | Prop::BackdropBlur
            | Prop::BackdropBrightness
            | Prop::BackdropContrast
            | Prop::BackdropGrayscale
            | Prop::BackdropHueRotate
            | Prop::BackdropInvert
            | Prop::BackdropOpacity
            | Prop::BackdropSaturate
            | Prop::BackdropSepia
            | Prop::BackdropFilter
    )
}

/// One axis of a frozen program's placements — the per-longhand
/// cascade winners the walk recorded (a later layer's value overwrites
/// an earlier layer's on the same longhand, the cascade sort's order).
#[derive(Clone, Copy, Debug, Default)]
struct AxisPlacement {
    start: Option<f32>,
    end: Option<f32>,
}

/// The grid-placement accumulator (the `RingState`/`GradState`
/// precedent): the four start/end longhands ride ONE state because the
/// span families are PAIR-shaped — the registry's `col-span`/`row-span`
/// rows fan ONE class onto the equal (start, end) pair, and GPUI's grid
/// carries that reading as the `Span` placement. Mapping the longhands
/// per-declaration would read "start 3, end 3" — a degenerate
/// zero-width range, never Tailwind's span 3. The compose rules:
///
/// * an equal POSITIVE start/end pair composes GPUI's `Span` form (the
///   registry's span fingerprint — two DIFFERENT classes spelling the
///   same equal pair read the same way, the pair shape is the span's);
/// * every other recorded value maps 1:1 onto the line slots
///   (`col_start`/`col_end`, `row_start`/`row_end` — negative lines
///   included, the from-end form the registry's negative flag accepts);
/// * an axis the program never mentions stays `Auto` — no accidental
///   placement.
#[derive(Clone, Copy, Debug, Default)]
struct GridState {
    column: AxisPlacement,
    row: AxisPlacement,
}

impl GridState {
    /// Record one placement declaration (non-numeric spellings are the
    /// registry rows' own boundary — the rows carry `num()`/`arb()`
    /// value kinds, so the fold produces `Val::Num`s; anything else
    /// never reaches the walk's frozen form).
    fn apply(&mut self, prop: Prop, val: Val) {
        let Val::Num(n) = val else {
            return;
        };
        match prop {
            Prop::GridColumnStart => self.column.start = Some(n),
            Prop::GridColumnEnd => self.column.end = Some(n),
            Prop::GridRowStart => self.row.start = Some(n),
            Prop::GridRowEnd => self.row.end = Some(n),
            _ => {}
        }
    }
}

/// The polyfills' single application point: the composed v4 shadow list
/// set in one `shadow()` call, the composed gradient in one `bg()` call
/// — or nothing (an empty state never touches the refinement). The grid
/// placement composes beside them (an axis the program never mentioned
/// never touches the refinement), and the shadow tier's band list
/// composes into the SAME box-shadow call as the ring's — the ring's
/// bands first, the preset's bands after, v4's own list order (`box-
/// shadow: var(--tw-ring-offset-shadow), var(--tw-ring-shadow),
/// var(--tw-shadow)`; the first layer paints on top, so the ring sits
/// over the preset's inner edge exactly as the v4 list reads).
///
/// The filter tier's chain composes beside them (the §3.7 row's
/// polyfill half): the recorded colour slots re-applied through the
/// matrices (the walk applied them inline unfiltered — the chain's
/// re-application is the cascade's last write, BEFORE the gradient
/// overrides the solid slot GPUI keeps), the composed ring/shadow band
/// colours recoloured the same way, and the parsed drop-shadow bands
/// appended after the preset bands (v4's canonical order puts the
/// drop-shadow var last — the matrices never recolour its colour).
/// An empty state composes nothing, exactly the pre-filter walk.
fn apply_polyfills<S: Styled>(
    style: S,
    ring: &RingState,
    grad: &GradState,
    grid: &GridState,
    shadow: &ShadowState,
    filter: &FilterState,
) -> S {
    // The matrix chain's recolour: identity until a live chain with a
    // matrix amount lands (`has_matrix`), then the seven spec matrices
    // in v4's var-canonical order — the drop-shadow-only chain
    // recolours nothing.
    let chain = filter.has_matrix();
    let shade = |color: Hsla| filter.shade(color).unwrap_or(color);
    let mut style = style;
    if chain {
        if let Some(background) = filter.background() {
            style = style.bg(shade(Hsla::from(background)));
        }
        if let Some(text) = filter.text() {
            style = style.text_color(shade(Hsla::from(text)));
        }
        if let Some(border) = filter.border() {
            style = style.border_color(shade(Hsla::from(border)));
        }
    }
    let mut shadows = ring.shadows();
    shadows.extend(shadow.shadows());
    if chain {
        for band in &mut shadows {
            band.color = shade(band.color);
        }
    }
    shadows.extend(filter.drop_shadows().unwrap_or_default());
    let mut style = if shadows.is_empty() {
        style
    } else {
        style.shadow(shadows)
    };
    if let Some(background) = grad.background_with(shade) {
        style = style.bg(background);
    }
    apply_grid(style, grid)
}

/// The grid placement's application point: the composed axis ranges
/// through GPUI's own placement builders — the span fingerprint as one
/// `col_span`/`row_span` call, the lines as the `col_start`/`col_end`/
/// `row_start`/`row_end` pairs.
fn apply_grid<S: Styled>(mut style: S, grid: &GridState) -> S {
    match (grid.column.start, grid.column.end) {
        (Some(start), Some(end)) if start == end && start > 0.0 => {
            style = style.col_span(start as u16);
        }
        (Some(start), Some(end)) => {
            style = style.col_start(start as i16);
            style = style.col_end(end as i16);
        }
        (Some(start), None) => style = style.col_start(start as i16),
        (None, Some(end)) => style = style.col_end(end as i16),
        (None, None) => {}
    }
    match (grid.row.start, grid.row.end) {
        (Some(start), Some(end)) if start == end && start > 0.0 => {
            style = style.row_span(start as u16);
        }
        (Some(start), Some(end)) => {
            style = style.row_start(start as i16);
            style = style.row_end(end as i16);
        }
        (Some(start), None) => style = style.row_start(start as i16),
        (None, Some(end)) => style = style.row_end(end as i16),
        (None, None) => {}
    }
    style
}
/// The dynamic lane's apply: the compiled program (or, on a failed
/// compile, the cell's LAST GOOD program — the honest per-frame
/// degradation) through the same walk the static programs take.
fn apply_dynamic<S: Styled>(
    style: S,
    cell: &ClassCell,
    classes: &str,
    env: &Env,
    tokens: Option<&TokenTable>,
) -> S {
    let compiled: Result<StaticProgram, CompileError> = match tokens {
        Some(tokens) => cell.get_theme(classes, tokens, &Config::default()),
        None => cell.get(classes, &Config::default()),
    };
    let program = match compiled {
        Ok(program) => program,
        Err(_) => cell.program(),
    };
    apply_matching(style, &program, env, None, None, None)
}

/// The interaction-closure half of the walk: the layers the predicate
/// selects (the closure's own state), composed over the base walk's
/// ring, gradient, grid-placement, shadow and filter states.
fn apply_state_polyfills<S: Styled>(
    style: S,
    program: &StaticProgram,
    pred: fn(Cond) -> bool,
    mut ring: RingState,
    mut grad: GradState,
    mut grid: GridState,
    mut shadow: ShadowState,
    mut filter: FilterState,
) -> S {
    let mut style = style;
    for (layer_index, layer) in program.layers.iter().enumerate() {
        if !pred(layer.cond) {
            continue;
        }
        for (decl_index, decl) in layer.decls.iter().enumerate() {
            let keyword = program.keyword(layer_index, decl_index).unwrap_or("");
            if is_ring_prop(decl.prop) {
                ring.apply(decl.prop, decl.val, keyword);
                continue;
            }
            if is_grad_prop(decl.prop) {
                grad.apply(decl.prop, decl.val, keyword);
                continue;
            }
            if is_grid_prop(decl.prop) {
                grid.apply(decl.prop, decl.val);
                continue;
            }
            if is_shadow_prop(decl.prop) {
                shadow.apply(decl.prop, decl.val, keyword);
                continue;
            }
            if is_filter_prop(decl.prop) {
                filter.apply(decl.prop, decl.val, keyword);
                continue;
            }
            if matches!(decl.prop, Prop::Background | Prop::BorderColor) {
                filter.apply(decl.prop, decl.val, keyword);
            }
            if decl.prop == Prop::Color {
                ring.apply(decl.prop, decl.val, keyword);
                shadow.apply(decl.prop, decl.val, keyword);
                filter.apply(decl.prop, decl.val, keyword);
            }
            style = apply_decl(style, decl.prop, decl.val, keyword);
        }
    }
    apply_polyfills(style, &ring, &grad, &grid, &shadow, &filter)
}

// ---------------------------------------------------------------------------
// Declaration mapping
// ---------------------------------------------------------------------------

fn apply_decl<S: Styled>(style: S, prop: Prop, val: Val, keyword: &str) -> S {
    match prop {
        Prop::Display => display(style, keyword),
        Prop::Position => position(style, keyword),
        Prop::Visibility => visibility(style, keyword),
        Prop::FlexDirection => flex_direction(style, keyword),
        Prop::FlexWrap => flex_wrap(style, keyword),
        Prop::JustifyContent => justify(style, keyword),
        Prop::JustifyItems | Prop::JustifySelf => justify(style, keyword),
        Prop::AlignItems => align_items(style, keyword),
        Prop::AlignContent => align_content(style, keyword),
        Prop::AlignSelf => align_self(style, keyword),
        Prop::FlexGrow => with_num(style, val, Styled::flex_grow),
        Prop::FlexShrink => with_num(style, val, Styled::flex_shrink),
        // The basis family: the pinned GPUI surface carries `flex_basis`
        // (the §3.7 grid row's "native subset" finished for flex
        // sizing) — the spacing rem, the fraction percentage and the
        // arbitrary percentage all land on the one `Length` slot.
        Prop::FlexBasis => with_length(style, val, keyword, Styled::flex_basis),
        Prop::GridTemplateColumns => match val {
            Val::Num(n) => style.grid_cols(n as u16),
            _ => style,
        },
        Prop::GridTemplateRows => match val {
            Val::Num(n) => style.grid_rows(n as u16),
            _ => style,
        },
        Prop::GapRow => with_definite(style, val, keyword, Styled::gap_y),
        Prop::GapCol => with_definite(style, val, keyword, Styled::gap_x),
        Prop::Width => with_length(style, val, keyword, Styled::w),
        Prop::Height => with_length(style, val, keyword, Styled::h),
        Prop::MinWidth => with_length(style, val, keyword, Styled::min_w),
        Prop::MinHeight => with_length(style, val, keyword, Styled::min_h),
        Prop::MaxWidth => with_length(style, val, keyword, Styled::max_w),
        Prop::MaxHeight => with_length(style, val, keyword, Styled::max_h),
        Prop::AspectRatio => with_num(style, val, Styled::aspect_ratio),
        Prop::Top => with_length(style, val, keyword, Styled::top),
        Prop::Right => with_length(style, val, keyword, Styled::right),
        Prop::Bottom => with_length(style, val, keyword, Styled::bottom),
        Prop::Left => with_length(style, val, keyword, Styled::left),
        Prop::PaddingTop => with_definite(style, val, keyword, Styled::pt),
        Prop::PaddingRight => with_definite(style, val, keyword, Styled::pr),
        Prop::PaddingBottom => with_definite(style, val, keyword, Styled::pb),
        Prop::PaddingLeft => with_definite(style, val, keyword, Styled::pl),
        Prop::MarginTop => with_length(style, val, keyword, Styled::mt),
        Prop::MarginRight => with_length(style, val, keyword, Styled::mr),
        Prop::MarginBottom => with_length(style, val, keyword, Styled::mb),
        Prop::MarginLeft => with_length(style, val, keyword, Styled::ml),
        Prop::BorderTopWidth => with_absolute(style, val, keyword, Styled::border_t),
        Prop::BorderRightWidth => with_absolute(style, val, keyword, Styled::border_r),
        Prop::BorderBottomWidth => with_absolute(style, val, keyword, Styled::border_b),
        Prop::BorderLeftWidth => with_absolute(style, val, keyword, Styled::border_l),
        Prop::BorderColor => with_color(style, val, Styled::border_color),
        Prop::BorderStyle => border_style(style, keyword),
        Prop::BorderTopLeftRadius => with_absolute(style, val, keyword, Styled::rounded_tl),
        Prop::BorderTopRightRadius => with_absolute(style, val, keyword, Styled::rounded_tr),
        Prop::BorderBottomRightRadius => with_absolute(style, val, keyword, Styled::rounded_br),
        Prop::BorderBottomLeftRadius => with_absolute(style, val, keyword, Styled::rounded_bl),
        Prop::Background => with_color(style, val, Styled::bg),
        Prop::Color => with_color(style, val, Styled::text_color),
        Prop::FontSize => with_absolute(style, val, keyword, Styled::text_size),
        Prop::LineHeight => with_definite(style, val, keyword, Styled::line_height),
        Prop::FontWeight => match keyword.parse::<f32>() {
            Ok(weight) => style.font_weight(FontWeight::from(weight)),
            Err(_) => style,
        },
        Prop::FontFamily => font_family(style, keyword),
        Prop::FontStyle => match keyword {
            "italic" => style.italic(),
            "normal" | "not-italic" => style.not_italic(),
            _ => style,
        },
        Prop::TextAlign => text_align(style, keyword),
        Prop::TextDecorationLine => text_decoration(style, keyword),
        // The decoration family's colour/thickness/style longhands (the
        // §3.7 typography surface's decoration half) — the helpers carry
        // the per-leg semantics (the pinned methods' get-or-insert on the
        // underline leg, the v4-true only-if-present patch on the
        // strikethrough leg).
        Prop::TextDecorationColor => decoration_color(style, val),
        Prop::TextDecorationThickness => decoration_thickness(style, val),
        Prop::TextDecorationStyle => decoration_style(style, keyword),
        Prop::WhiteSpace => match keyword {
            "nowrap" | "pre" | "pre-line" | "pre-wrap" | "break-spaces" => {
                style.whitespace_nowrap()
            }
            _ => style.whitespace_normal(),
        },
        Prop::TextOverflow => match keyword {
            "ellipsis" | "truncate" => style.text_ellipsis(),
            _ => style,
        },
        Prop::OverflowX => overflow_x(style, keyword),
        Prop::OverflowY => overflow_y(style, keyword),
        Prop::Opacity => with_num(style, val, |style, n| {
            style.opacity(if n > 1.0 { n / 100.0 } else { n })
        }),
        Prop::Cursor => cursor(style, keyword),
        _ => style,
    }
}

fn display<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "none" => style.hidden(),
        "flex" | "inline-flex" => style.flex(),
        "grid" | "inline-grid" => style.grid(),
        _ => style.block(),
    }
}

fn position<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "absolute" => style.absolute(),
        _ => style.relative(),
    }
}

fn visibility<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "hidden" | "collapse" => style.invisible(),
        _ => style.visible(),
    }
}

fn flex_direction<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "column" => style.flex_col(),
        "column-reverse" => style.flex_col_reverse(),
        "row-reverse" => style.flex_row_reverse(),
        _ => style.flex_row(),
    }
}

fn flex_wrap<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "wrap" => style.flex_wrap(),
        "wrap-reverse" => style.flex_wrap_reverse(),
        _ => style.flex_nowrap(),
    }
}

fn justify<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "center" => style.justify_center(),
        "flex-end" | "end" => style.justify_end(),
        "space-between" => style.justify_between(),
        "space-around" => style.justify_around(),
        "space-evenly" => style.justify_evenly(),
        _ => style.justify_start(),
    }
}

fn align_items<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "center" => style.items_center(),
        "flex-end" | "end" => style.items_end(),
        "baseline" => style.items_baseline(),
        "stretch" => style.items_stretch(),
        _ => style.items_start(),
    }
}

fn align_content<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "center" => style.content_center(),
        "flex-end" | "end" => style.content_end(),
        "space-between" => style.content_between(),
        "space-around" => style.content_around(),
        "space-evenly" => style.content_evenly(),
        "stretch" => style.content_stretch(),
        _ => style.content_start(),
    }
}

fn align_self<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "center" => style.self_center(),
        "flex-start" | "start" => style.self_start(),
        "flex-end" | "end" => style.self_end(),
        "baseline" => style.self_baseline(),
        "stretch" => style.self_stretch(),
        _ => style.self_start(),
    }
}

fn border_style<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "dashed" => style.border_dashed(),
        _ => style,
    }
}

fn text_align<S: Styled>(style: S, keyword: &str) -> S {
    // ONE mapping: the pub helper's closed set IS the walk's own arm —
    // the parallel match the old private arm kept could drift from the
    // pinned spelling set.
    style.text_align(text_align_value(keyword))
}

fn text_decoration<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "underline" => style.underline(),
        "line-through" => style.line_through(),
        "none" | "no-underline" => style.text_decoration_none(),
        _ => style,
    }
}

/// `decoration-{color}` — `text-decoration-color`: the colour the
/// composed decoration lines render. The underline leg rides the pinned
/// `Styled::text_decoration_color` method (its own get-or-insert
/// semantics — GPUI's reading of the class); the strikethrough leg has
/// no pinned method, so the walk patches it V4-TRUE — only when a
/// line-through already composes (the colour alone never creates one).
fn decoration_color<S: Styled>(style: S, val: Val) -> S {
    let Some(color) = color(val) else {
        return style;
    };
    let style = style.text_decoration_color(color);
    let mut style = style;
    if let Some(strikethrough) = style.text_style().strikethrough.as_mut() {
        strikethrough.color = Some(color);
    }
    style
}

/// `decoration-{thickness}` — `text-decoration-thickness`: the
/// underline leg's absolute-pixel slot (the pinned
/// `text_decoration_0..8` methods' own get-or-insert semantics,
/// generalized to the whole integer scale — the bare integers are v4's
/// own `isPositiveInteger` grammar); the strikethrough leg patches only
/// when a line-through already composes. The keyword and rem/percent
/// forms never reach the walk — the emit gates refuse them (the
/// absolute-px slot's typed boundary).
fn decoration_thickness<S: Styled>(style: S, val: Val) -> S {
    let Some(thickness) = (match val {
        Val::Px(n) | Val::Num(n) => Some(px(n)),
        _ => None,
    }) else {
        return style;
    };
    let mut style = style;
    let underline = style
        .text_style()
        .underline
        .get_or_insert_with(Default::default);
    underline.thickness = thickness;
    if let Some(strikethrough) = style.text_style().strikethrough.as_mut() {
        strikethrough.thickness = thickness;
    }
    style
}

/// `decoration-{style}` — `text-decoration-style`: the pinned
/// `text_decoration_solid`/`text_decoration_wavy` methods (the two
/// forms GPUI's single-line surface expresses); the double/dotted/dashed
/// keywords never reach the walk — the emit gates refuse them.
fn decoration_style<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "wavy" => style.text_decoration_wavy(),
        "solid" => style.text_decoration_solid(),
        _ => style,
    }
}

fn overflow_x<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "hidden" | "clip" | "scroll" | "auto" => style.overflow_x_hidden(),
        _ => style,
    }
}

fn overflow_y<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "hidden" | "clip" | "scroll" | "auto" => style.overflow_y_hidden(),
        _ => style,
    }
}

fn font_family<S: Styled>(style: S, keyword: &str) -> S {
    if keyword.contains("mono") {
        style.font_family("monospace")
    } else if keyword.contains("serif") {
        style.font_family("serif")
    } else {
        style.font_family("sans-serif")
    }
}

fn cursor<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "pointer" => style.cursor_pointer(),
        "text" => style.cursor_text(),
        "move" => style.cursor_move(),
        "not-allowed" => style.cursor_not_allowed(),
        "crosshair" => style.cursor_crosshair(),
        _ => style,
    }
}

// ---------------------------------------------------------------------------
// Value helpers
// ---------------------------------------------------------------------------

fn with_num<S: Styled>(style: S, val: Val, f: impl FnOnce(S, f32) -> S) -> S {
    match val.as_num() {
        Some(n) => f(style, n),
        None => style,
    }
}

fn with_definite<S: Styled>(
    style: S,
    val: Val,
    keyword: &str,
    f: impl FnOnce(S, DefiniteLength) -> S,
) -> S {
    match definite(val, keyword) {
        Some(length) => f(style, length),
        None => style,
    }
}

fn with_length<S: Styled>(style: S, val: Val, keyword: &str, f: impl FnOnce(S, Length) -> S) -> S {
    match length(val, keyword) {
        Some(length) => f(style, length),
        None => style,
    }
}

fn with_absolute<S: Styled>(
    style: S,
    val: Val,
    keyword: &str,
    f: impl FnOnce(S, AbsoluteLength) -> S,
) -> S {
    match absolute(val, keyword) {
        Some(length) => f(style, length),
        None => style,
    }
}

fn with_color<S: Styled>(style: S, val: Val, f: impl FnOnce(S, Hsla) -> S) -> S {
    match color(val) {
        Some(color) => f(style, color),
        None => style,
    }
}

/// The row text's own length form: the plain px/rem spellings the row
/// tables carry beside the `Val::Kw` slots (`border`'s bare `1px`, the
/// radius scale's `0.375rem`, the composite text sizes' `0.875rem`
/// primary). The CALC texts (the composite line-height pair) are the
/// CSS arm's own output — the walk does not evaluate expressions and
/// returns `None` (never a guessed value).
fn row_length(keyword: &str) -> Option<Val> {
    if let Some(n) = keyword.strip_suffix("px") {
        n.trim().parse::<f32>().ok().map(Val::Px)
    } else if let Some(n) = keyword.strip_suffix("rem") {
        n.trim().parse::<f32>().ok().map(Val::Rem)
    } else if let Some(n) = keyword.strip_suffix('%') {
        n.trim().parse::<f32>().ok().map(Val::Pct)
    } else {
        None
    }
}

fn definite(val: Val, keyword: &str) -> Option<DefiniteLength> {
    match val {
        Val::Rem(n) => Some(rems(n).into()),
        Val::Px(n) => Some(px(n).into()),
        Val::Pct(n) => Some(relative(n / 100.0)),
        // The keyword slot's row text (the seam the frozen program
        // carries: the slot indexes the row's own table, the text rides
        // beside it).
        Val::Kw(_) => match row_length(keyword)? {
            Val::Rem(n) => Some(rems(n).into()),
            Val::Px(n) => Some(px(n).into()),
            Val::Pct(n) => Some(relative(n / 100.0)),
            _ => None,
        },
        _ => None,
    }
}

fn length(val: Val, keyword: &str) -> Option<Length> {
    match val {
        Val::Rem(n) => Some(rems(n).into()),
        Val::Px(n) => Some(px(n).into()),
        Val::Pct(n) => Some(relative(n / 100.0).into()),
        Val::Kw(_) => match row_length(keyword)? {
            Val::Rem(n) => Some(rems(n).into()),
            Val::Px(n) => Some(px(n).into()),
            Val::Pct(n) => Some(relative(n / 100.0).into()),
            _ => None,
        },
        _ => None,
    }
}

fn absolute(val: Val, keyword: &str) -> Option<AbsoluteLength> {
    match val {
        Val::Rem(n) => Some(rems(n).into()),
        Val::Px(n) => Some(px(n).into()),
        // The bare-integer px scales (v4's own reading: `border-2` is
        // 2px) — the number form the row lowers.
        Val::Num(n) => Some(px(n).into()),
        Val::Kw(_) => match row_length(keyword)? {
            Val::Rem(n) => Some(rems(n).into()),
            Val::Px(n) => Some(px(n).into()),
            _ => None,
        },
        _ => None,
    }
}

fn color(val: Val) -> Option<Hsla> {
    match val {
        Val::Rgba([r, g, b, a]) => Some(rgba(u32::from_be_bytes([r, g, b, a])).into()),
        _ => None,
    }
}

/// The GPUI mapping for a bare `TextAlign` value — the walk's own
/// `Prop::TextAlign` arm reads through it (one grammar, the suites pin
/// the spelling set beside it). GPUI's [`TextAlign`] carries LEFT /
/// CENTER / RIGHT: the registry's `start`/`end` logical directions fold
/// to the LTR readings, and `justify` — no GPUI slot — rides the
/// documented LEFT default (the platform's own closed set, pinned).
pub fn text_align_value(keyword: &str) -> TextAlign {
    match keyword {
        "center" => TextAlign::Center,
        "right" | "end" => TextAlign::Right,
        _ => TextAlign::Left,
    }
}

// ---------------------------------------------------------------------------
// The frame environment (P4)
// ---------------------------------------------------------------------------

/// Refresh the probe environment from the window: dark mode and the
/// viewport width in rem. Call once per render; [`AeolusRoot`] does it
/// for you.
pub fn begin_frame(window: &gpui::Window) {
    let rem = window.rem_size();
    let viewport_rem = if rem > px(0.0) {
        window.viewport_size().width / rem
    } else {
        0.0
    };
    let dark = matches!(
        window.appearance(),
        gpui::WindowAppearance::Dark | gpui::WindowAppearance::VibrantDark
    );
    let mut current = env();
    current.dark = dark;
    current.viewport_rem = viewport_rem;
    set_env(current);
}

/// A wrapper view that refreshes [`Env`] once per render and then renders
/// the wrapped entity — the zero-argument `.tw()` ergonomics.
pub struct AeolusRoot<V: gpui::Render> {
    child: gpui::Entity<V>,
}

impl<V: gpui::Render> AeolusRoot<V> {
    /// Wrap a child view.
    pub const fn new(child: gpui::Entity<V>) -> Self {
        Self { child }
    }
}

impl<V: gpui::Render> gpui::Render for AeolusRoot<V> {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        begin_frame(window);
        self.child.clone().into_any_element()
    }
}
