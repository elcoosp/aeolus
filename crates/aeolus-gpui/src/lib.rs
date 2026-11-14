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
//! [`Env`]'s nearest-container slot, and the element-bounds WRAPPER
//! (who MEASURES the widths) stays the host's half — the walk consults
//! the seam's answers, it never measures.
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

use std::cell::RefCell;

use aeolus_compile::{CompileError, Config, Env, structural_fires};
use aeolus_ir::{Cond, Prop, Val};
use aeolus_rt::{ClassCell, StaticProgram, TokenTable};
use gpui::{
    AbsoluteLength, Animation, AnimationElement, AnimationExt, DefiniteLength, ElementId,
    FontWeight, Hsla, IntoElement, Length, StatefulInteractiveElement, Styled, TextAlign, px,
    relative, rems, rgba,
};

mod grad;
mod motion;
mod ring;
pub use grad::GradState;
pub use motion::{
    AnimSpec, Easing, animation, animation_for, animation_keyword, parse_animation, pulse_animator,
};
pub use ring::RingState;

pub use aeolus_macro::{cn, theme, tw, tw_cn, variants};
pub use aeolus_rt::{Env as Environment, StaticProgram as Program, theme_from_css};
// The attribute-bits seam (plan §3.5): the element's own `data-`/`aria-`
// bits ride the host state surface, so the prelude carries the probe
// trait, the built-in bit table and the bit/kind constructors.
pub use aeolus_rt::{AttrBit, AttrKind, AttrProbe, AttrState, StructFacts, StructProbe};
// The container-bounds seam (plan §3.5's `@container` row): the NAMED
// container queries (`@md/card:`) answer through the host's
// [`ContainerProbe`] source, so the prelude carries the trait, the
// built-in static table and its bound row.
pub use aeolus_rt::{ContainerBound, ContainerProbe, ContainerState};

/// The V1 consumption surface: macros, the program type and the two
/// extension traits.
pub mod prelude {
    pub use crate::{
        AeolusRoot, GradState, TwAnimExt, TwExt, TwStateExt, begin_frame, cn, set_env, theme,
        theme_from_css, tw, tw_cn, variants,
    };
    pub use aeolus_compile::Env;
    pub use aeolus_rt::{
        AttrBit, AttrKind, AttrProbe, AttrState, ClassCell, ContainerBound, ContainerProbe,
        ContainerState, StaticProgram, StructFacts, StructProbe,
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
    /// element-bounds WRAPPER — who MEASURES the widths — stays the
    /// host's half: this walk consults the answers, it never measures.
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
        // `tw_probes` walk instead. The base walk's ring and gradient
        // states seed every closure — `hover:ring-4` composes with the
        // base's `ring-red-500` colour and `hover:from-blue-500` with
        // the base's `from-red-500` stop the way CSS's vars persist
        // across state layers.
        let mut ring = RingState::default();
        let mut grad = GradState::default();
        let mut this =
            apply_matching_polyfills(self, &program, &env, None, None, None, &mut ring, &mut grad);
        this = this.hover(|style| apply_state_polyfills(style, &program, is_hover, ring, grad));
        this = this.focus(|style| apply_state_polyfills(style, &program, is_focus, ring, grad));
        this = this.focus_visible(|style| {
            apply_state_polyfills(style, &program, is_focus_visible, ring, grad)
        });
        this = this.active(|style| apply_state_polyfills(style, &program, is_active, ring, grad));
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
        Cond::Selection | Cond::Placeholder => false,
        Cond::GroupHover => env.group_hover,
        Cond::GroupFocus => env.group_focus,
        Cond::PeerHover => env.peer_hover,
        Cond::PeerFocus => env.peer_focus,
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
    )
}

/// [`apply_matching`] with BOTH polyfill accumulators threaded through
/// — the shared walk body, so the base walk ([`TwExt::tw`]) and the
/// [`TwStateExt`] closures compose the SAME `--tw-ring-*` and
/// `--tw-gradient-*` state and the interaction closures inherit the
/// base's bands and stops instead of resetting them.
fn apply_matching_polyfills<S: Styled>(
    mut style: S,
    program: &StaticProgram,
    env: &Env,
    probes: Option<&dyn AttrProbe>,
    containers: Option<&dyn ContainerProbe>,
    structs: Option<&dyn StructProbe>,
    ring: &mut RingState,
    grad: &mut GradState,
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
            if decl.prop == Prop::Color {
                ring.apply(decl.prop, decl.val, keyword);
            }
            style = apply_decl(style, decl.prop, decl.val, keyword);
        }
    }
    apply_polyfills(style, ring, grad)
}

/// The four `--tw-ring-*` longhands the polyfill owns (the registry's
/// ring rows lower into exactly these; `ring-inset` lowers into
/// `Prop::BoxShadow` and stays the reported boundary).
fn is_ring_prop(prop: Prop) -> bool {
    matches!(
        prop,
        Prop::RingWidth | Prop::RingColor | Prop::RingOffsetWidth | Prop::RingOffsetColor
    )
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

/// The polyfills' single application point: the composed v4 shadow list
/// set in one `shadow()` call, the composed gradient in one `bg()` call
/// — or nothing (an empty state never touches the refinement).
fn apply_polyfills<S: Styled>(style: S, ring: &RingState, grad: &GradState) -> S {
    let mut style = apply_ring(style, ring);
    if let Some(background) = grad.background() {
        style = style.bg(background);
    }
    style
}

/// The ring polyfill's application point: the composed v4 list in one
/// `shadow()` call, or nothing (an empty state never touches the
/// refinement — `ring-0` leaves no shadow layers behind).
fn apply_ring<S: Styled>(style: S, ring: &RingState) -> S {
    let shadows = ring.shadows();
    if shadows.is_empty() {
        return style;
    }
    style.shadow(shadows)
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
/// ring and gradient states.
fn apply_state_polyfills<S: Styled>(
    style: S,
    program: &StaticProgram,
    pred: fn(Cond) -> bool,
    mut ring: RingState,
    mut grad: GradState,
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
            if decl.prop == Prop::Color {
                ring.apply(decl.prop, decl.val, keyword);
            }
            style = apply_decl(style, decl.prop, decl.val, keyword);
        }
    }
    apply_polyfills(style, &ring, &grad)
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
        Prop::GridTemplateColumns => match val {
            Val::Num(n) => style.grid_cols(n as u16),
            _ => style,
        },
        Prop::GridTemplateRows => match val {
            Val::Num(n) => style.grid_rows(n as u16),
            _ => style,
        },
        Prop::GapRow => with_definite(style, val, Styled::gap_y),
        Prop::GapCol => with_definite(style, val, Styled::gap_x),
        Prop::Width => with_length(style, val, Styled::w),
        Prop::Height => with_length(style, val, Styled::h),
        Prop::MinWidth => with_length(style, val, Styled::min_w),
        Prop::MinHeight => with_length(style, val, Styled::min_h),
        Prop::MaxWidth => with_length(style, val, Styled::max_w),
        Prop::MaxHeight => with_length(style, val, Styled::max_h),
        Prop::AspectRatio => with_num(style, val, Styled::aspect_ratio),
        Prop::Top => with_length(style, val, Styled::top),
        Prop::Right => with_length(style, val, Styled::right),
        Prop::Bottom => with_length(style, val, Styled::bottom),
        Prop::Left => with_length(style, val, Styled::left),
        Prop::PaddingTop => with_definite(style, val, Styled::pt),
        Prop::PaddingRight => with_definite(style, val, Styled::pr),
        Prop::PaddingBottom => with_definite(style, val, Styled::pb),
        Prop::PaddingLeft => with_definite(style, val, Styled::pl),
        Prop::MarginTop => with_length(style, val, Styled::mt),
        Prop::MarginRight => with_length(style, val, Styled::mr),
        Prop::MarginBottom => with_length(style, val, Styled::mb),
        Prop::MarginLeft => with_length(style, val, Styled::ml),
        Prop::BorderTopWidth => with_absolute(style, val, Styled::border_t),
        Prop::BorderRightWidth => with_absolute(style, val, Styled::border_r),
        Prop::BorderBottomWidth => with_absolute(style, val, Styled::border_b),
        Prop::BorderLeftWidth => with_absolute(style, val, Styled::border_l),
        Prop::BorderColor => with_color(style, val, Styled::border_color),
        Prop::BorderStyle => border_style(style, keyword),
        Prop::BorderTopLeftRadius => with_absolute(style, val, Styled::rounded_tl),
        Prop::BorderTopRightRadius => with_absolute(style, val, Styled::rounded_tr),
        Prop::BorderBottomRightRadius => with_absolute(style, val, Styled::rounded_br),
        Prop::BorderBottomLeftRadius => with_absolute(style, val, Styled::rounded_bl),
        Prop::Background => with_color(style, val, Styled::bg),
        Prop::Color => with_color(style, val, Styled::text_color),
        Prop::FontSize => with_absolute(style, val, Styled::text_size),
        Prop::LineHeight => with_definite(style, val, Styled::line_height),
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
    match keyword {
        "center" => style.text_center(),
        "right" | "end" => style.text_right(),
        _ => style.text_left(),
    }
}

fn text_decoration<S: Styled>(style: S, keyword: &str) -> S {
    match keyword {
        "underline" => style.underline(),
        "line-through" => style.line_through(),
        "none" | "no-underline" => style.text_decoration_none(),
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

fn with_definite<S: Styled>(style: S, val: Val, f: impl FnOnce(S, DefiniteLength) -> S) -> S {
    match definite(val) {
        Some(length) => f(style, length),
        None => style,
    }
}

fn with_length<S: Styled>(style: S, val: Val, f: impl FnOnce(S, Length) -> S) -> S {
    match length(val) {
        Some(length) => f(style, length),
        None => style,
    }
}

fn with_absolute<S: Styled>(style: S, val: Val, f: impl FnOnce(S, AbsoluteLength) -> S) -> S {
    match absolute(val) {
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

fn definite(val: Val) -> Option<DefiniteLength> {
    match val {
        Val::Rem(n) => Some(rems(n).into()),
        Val::Px(n) => Some(px(n).into()),
        Val::Pct(n) => Some(relative(n / 100.0)),
        _ => None,
    }
}

fn length(val: Val) -> Option<Length> {
    match val {
        Val::Rem(n) => Some(rems(n).into()),
        Val::Px(n) => Some(px(n).into()),
        Val::Pct(n) => Some(relative(n / 100.0).into()),
        _ => None,
    }
}

fn absolute(val: Val) -> Option<AbsoluteLength> {
    match val {
        Val::Rem(n) => Some(rems(n).into()),
        Val::Px(n) => Some(px(n).into()),
        _ => None,
    }
}

fn color(val: Val) -> Option<Hsla> {
    match val {
        Val::Rgba([r, g, b, a]) => Some(rgba(u32::from_be_bytes([r, g, b, a])).into()),
        _ => None,
    }
}

/// The GPUI mapping for a bare `TextAlign` value, exposed for tests.
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
