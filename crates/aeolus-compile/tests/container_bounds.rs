//! The named-container BOUNDS seam (plan §3.5's `@container` row, the
//! half the named-container fold left open): a NAMED container query
//! (`@md/card:`) needs THAT container's width, and one
//! [`Env::container_rem`] slot cannot answer per name — the fold's own
//! tests pin the named layers reporting
//! [`InterpNote::ContainerUnqueried`] under the plain [`interp`] forms.
//!
//! This suite pins the seam that closes that boundary at the pipeline
//! level: the [`ContainerProbe`] trait (one question — what width does
//! the container NAME carry?), the built-in [`ContainerState`] static
//! table (the [`AttrState`](aeolus_compile::AttrState) precedent), and
//! the `*_containers` entry points the `*_probes` convention carries.
//! The CSS semantics the pins encode:
//!
//! * a min form (`@md/card:`) fires AT and above the named container's
//!   width, a max form (`@max-lg/sidebar:`) strictly below it — the
//!   same boundary semantics the viewport breakpoints obey;
//! * a container the source does NOT know is a QUIET miss (a CSS
//!   selector that does not match), never an exclusion note — the
//!   sourceless forms keep the reported exclusion (the honest boundary
//!   where the host gave no answer at all);
//! * two named containers answer independently (the per-name collapse
//!   the fold deleted stays deleted at resolution time);
//! * the unnamed forms keep answering from [`Env::container_rem`]
//!   beside the seam — the seam only takes over the NAMED question;
//! * the seams compose: attribute probes and the container bounds ride
//!   ONE walk (the DynCache loop's "both seams" shape), and the themed
//!   forms resolve token values through the same seam family.
//!
//! Written FIRST (the RED spec): the suite fails while the named
//! container layers have no seam to ask.

use aeolus_compile::{
    Config, ContainerBound, ContainerProbe, ContainerState, Env, InterpNote, compile,
    compile_theme, interp, interp_containers, interp_probes_containers, interp_theme_containers,
};
use aeolus_ir::{Cond, Prop, Val};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The value one longhand resolved to.
fn resolved(interp: &aeolus_compile::Interped, prop: Prop) -> Val {
    interp
        .decls
        .iter()
        .find(|d| d.prop == prop)
        .unwrap_or_else(|| panic!("{prop:?} is in the resolved declarations"))
        .val
}

/// The note one cond produced, if any.
fn note_for<'a>(interp: &'a aeolus_compile::Interped, cond: &'a Cond) -> Option<&'a InterpNote> {
    interp.notes.iter().find(|note| match note {
        InterpNote::PseudoElement { cond: c, .. }
        | InterpNote::ContainerUnqueried { cond: c, .. }
        | InterpNote::AttributeUnqueried { cond: c, .. } => c == cond,
    })
}

/// The runtime-tier table over the shadcn source.
fn runtime_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

// ---------------------------------------------------------------------------
// The min form: AT and above the named container's width
// ---------------------------------------------------------------------------

#[test]
fn the_seam_answers_the_named_min_form_at_and_above() {
    let compiled = compile("@md/card:w-64", &Config::default()).expect("named compiles");
    // Exactly AT md's 48rem: the min form fires (the CSS `>=` boundary).
    let bounds = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let hot = interp_containers(
        &compiled.program,
        &Env::default(),
        &ContainerState::new(&bounds),
    )
    .expect("the walk resolves");
    assert_eq!(
        resolved(&hot, Prop::Width),
        Val::Rem(16.0),
        "w-64 lowers to 16rem and the named layer fires at the boundary"
    );
    // Strictly below: the layer stays quiet — and QUIET, not excluded.
    let bounds = [ContainerBound {
        name: "card",
        rem: 47.9,
    }];
    let cold = interp_containers(
        &compiled.program,
        &Env::default(),
        &ContainerState::new(&bounds),
    )
    .expect("the walk resolves");
    assert!(
        cold.decls.iter().all(|d| d.prop != Prop::Width),
        "below the named container's width the layer does not fire"
    );
    assert!(
        note_for(
            &cold,
            &Cond::Container {
                bp: aeolus_ir::Bp::Md,
                name: Some("card"),
            }
        )
        .is_none(),
        "a source-answered miss is CSS-quiet, never an exclusion note"
    );
}

#[test]
fn the_seam_answers_the_named_max_form_strictly_below() {
    let compiled =
        compile("@max-lg/sidebar:hidden", &Config::default()).expect("named max compiles");
    // At exactly lg's 64rem the max form does NOT fire (strict `<`).
    let bounds = [ContainerBound {
        name: "sidebar",
        rem: 64.0,
    }];
    let at = interp_containers(
        &compiled.program,
        &Env::default(),
        &ContainerState::new(&bounds),
    )
    .expect("the walk resolves");
    assert!(
        at.decls.is_empty(),
        "at the boundary the max form stays quiet"
    );
    // Below it: `hidden` wins the Display longhand.
    let bounds = [ContainerBound {
        name: "sidebar",
        rem: 63.9,
    }];
    let below = interp_containers(
        &compiled.program,
        &Env::default(),
        &ContainerState::new(&bounds),
    )
    .expect("the walk resolves");
    assert!(
        below.decls.iter().any(|d| d.prop == Prop::Display),
        "below the boundary the named max layer fires"
    );
}

// ---------------------------------------------------------------------------
// The honest shapes: quiet miss vs reported exclusion
// ---------------------------------------------------------------------------

#[test]
fn an_unknown_container_is_a_quiet_miss_with_the_seam_wired() {
    let compiled = compile("@md/card:w-64", &Config::default()).expect("named compiles");
    // The source knows OTHER containers, not `card`: the selector does
    // not match — CSS-quiet, no note.
    let other = ContainerState::new(&[ContainerBound {
        name: "sidebar",
        rem: 30.0,
    }]);
    let interped =
        interp_containers(&compiled.program, &Env::default(), &other).expect("the walk resolves");
    assert!(
        interped.decls.is_empty(),
        "an unknown container never fires"
    );
    assert!(
        interped.notes.is_empty(),
        "a wired source answering None is a quiet miss, not an exclusion: {:?}",
        interped.notes
    );
}

#[test]
fn the_sourceless_forms_keep_the_reported_exclusion() {
    let compiled = compile("@md/card:w-64", &Config::default()).expect("named compiles");
    // The plain form carries no container source: the named layer is
    // REPORTED (the fold's own pin — the boundary the seam closes when
    // the host wires one).
    let interped = interp(&compiled.program, &Env::default()).expect("the walk resolves");
    let note = note_for(
        &interped,
        &Cond::Container {
            bp: aeolus_ir::Bp::Md,
            name: Some("card"),
        },
    )
    .expect("the sourceless named layer reports");
    assert!(
        matches!(note, InterpNote::ContainerUnqueried { .. }),
        "the sourceless form keeps the honest exclusion"
    );
}

// ---------------------------------------------------------------------------
// Per-name independence and the unnamed regression
// ---------------------------------------------------------------------------

#[test]
fn named_containers_answer_independently() {
    let compiled =
        compile("@md/card:text-sm @md/menu:text-lg", &Config::default()).expect("compiles");
    // The expected value is the PLAIN compile's own answer (the variant
    // must not change what `text-sm` lowers to — the differential pin
    // the value pass rides; the keyword slot is row-relative, so the
    // value identity is the pin, not the text).
    let plain = compile("text-sm", &Config::default()).expect("plain compiles");
    let plain_interped = interp(&plain.program, &Env::default()).expect("resolves");
    let expected = resolved(&plain_interped, Prop::FontSize);
    let state = ContainerState::new(&[
        ContainerBound {
            name: "card",
            rem: 50.0,
        },
        ContainerBound {
            name: "menu",
            rem: 10.0,
        },
    ]);
    let interped =
        interp_containers(&compiled.program, &Env::default(), &state).expect("the walk resolves");
    assert_eq!(
        resolved(&interped, Prop::FontSize),
        expected,
        "text-sm wins: card is wide enough, menu is not"
    );
}

#[test]
fn the_unnamed_forms_still_answer_from_the_env_beside_the_seam() {
    let compiled = compile("@md:flex", &Config::default()).expect("unnamed compiles");
    let env = Env {
        container_rem: Some(48.0),
        ..Env::default()
    };
    // The seam knows only `card`; the UNNAMED layer keeps asking the
    // Env's nearest-container slot — the seam takes over the NAMED
    // question only.
    let bounds = [ContainerBound {
        name: "card",
        rem: 1.0,
    }];
    let interped = interp_containers(&compiled.program, &env, &ContainerState::new(&bounds))
        .expect("the walk resolves");
    assert!(
        interped.decls.iter().any(|d| d.prop == Prop::Display),
        "the unnamed container layer still answers from Env::container_rem"
    );
    assert!(interped.notes.is_empty(), "the unnamed answer is clean");
}

// ---------------------------------------------------------------------------
// The seams compose: probes + containers on one walk, themed forms
// ---------------------------------------------------------------------------

#[test]
fn the_full_form_rides_both_seams_on_one_walk() {
    let compiled = compile(
        "data-[state=open]:underline @md/card:w-64",
        &Config::default(),
    )
    .expect("compiles");
    let bits = [aeolus_compile::AttrBit {
        kind: aeolus_compile::AttrKind::Data,
        name: "state",
        value: "open",
    }];
    let probes = aeolus_compile::AttrState::new(&bits);
    let bounds = ContainerState::new(&[
        ContainerBound {
            name: "card",
            rem: 48.0,
        },
        ContainerBound {
            name: "menu",
            rem: 10.0,
        },
    ]);
    let interped = interp_probes_containers(&compiled.program, &Env::default(), &probes, &bounds)
        .expect("the walk resolves");
    assert!(
        interped
            .decls
            .iter()
            .any(|d| d.prop == Prop::TextDecorationLine),
        "the attribute-probe layer fires through the probe seam"
    );
    assert_eq!(
        resolved(&interped, Prop::Width),
        Val::Rem(16.0),
        "the named-container layer fires through the bounds seam"
    );
    assert!(interped.notes.is_empty(), "both layers answered: no notes");
}

#[test]
fn the_themed_form_resolves_tokens_beside_the_bounds_seam() {
    let table = runtime_table();
    let compiled =
        compile_theme("@md/card:bg-primary", &table, &Config::default()).expect("theme compiles");
    let bounds = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let hot = interp_theme_containers(
        &compiled.program,
        &table,
        &Env::default(),
        &ContainerState::new(&bounds),
    )
    .expect("the walk resolves");
    let resolved_hot = resolved(&hot, Prop::Background);
    assert!(
        matches!(resolved_hot, Val::Rgba([..])),
        "the token resolves to concrete RGBA beside the seam: {resolved_hot:?}"
    );
    let bounds = [ContainerBound {
        name: "card",
        rem: 10.0,
    }];
    let cold = interp_theme_containers(
        &compiled.program,
        &table,
        &Env::default(),
        &ContainerState::new(&bounds),
    )
    .expect("the walk resolves");
    assert!(
        cold.decls.iter().all(|d| d.prop != Prop::Background),
        "below the named width the themed layer stays quiet"
    );
}

// ---------------------------------------------------------------------------
// The Compiled convenience forms
// ---------------------------------------------------------------------------

#[test]
fn the_compiled_convenience_form_carries_the_seam() {
    let compiled = compile("@md/card:w-64", &Config::default()).expect("named compiles");
    let bounds = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let interped = compiled
        .interp_containers(&Env::default(), &ContainerState::new(&bounds))
        .expect("the walk resolves");
    assert_eq!(
        resolved(&interped, Prop::Width),
        Val::Rem(16.0),
        "the Compiled form delegates to the same seam walk"
    );
}

// ---------------------------------------------------------------------------
// The trait is implementable by a host (the seam contract)
// ---------------------------------------------------------------------------

#[test]
fn a_host_source_answers_the_same_question() {
    /// A host's live container registry — the seam's ONE question,
    /// answered from the host's own state.
    struct HostBounds {
        rem: f32,
    }
    impl ContainerProbe for HostBounds {
        fn container(&self, name: &str) -> Option<f32> {
            (name == "card").then_some(self.rem)
        }
    }
    let compiled = compile("@md/card:w-64", &Config::default()).expect("named compiles");
    let host = HostBounds { rem: 50.0 };
    let interped =
        interp_containers(&compiled.program, &Env::default(), &host).expect("the walk resolves");
    assert_eq!(
        resolved(&interped, Prop::Width),
        Val::Rem(16.0),
        "a host source answers through the same trait object the built-in rides"
    );
}
