//! The interpreter-arm contracts (architecture plan §3.1 emit (c) — the
//! `interp` half the `Target` enum named but never armed): a compiled
//! [`Program`] resolves to its EFFECTIVE declarations under a probe
//! [`Env`] and a [`TokenSource`], per the cascade the IR sorts.
//!
//! The pins, in the order the crate docs promise them:
//!
//! * the cascade is a front-to-back read: a fired layer wins per
//!   longhand over every earlier layer, within a layer the later
//!   declaration wins (`p-4 px-8` → four paddings, right values), and
//!   the resolved declarations come out in `Prop` cascade order;
//! * probes answer from the `Env` only — element states, the group/peer
//!   relations, the viewport width (min forms fire AT the boundary, max
//!   forms BELOW it), the container context (absent = the layer never
//!   matches, CSS semantics, reported as a note);
//! * `::selection`/`::placeholder` layers are separate RENDER TARGETS,
//!   never the element's own cascade — excluded and reported;
//! * theme dynamism resolves through the [`TokenSource`] SEAM: the
//!   built-in table multiplies the active half (one load + one
//!   multiply), a host source wires in and answers for itself, and the
//!   theme-less interp refuses token values with the typed boundary;
//! * every failure carries the stable code + help contract, and the
//!   walk itself is deterministic and pure.
//!
//! Written FIRST (the RED spec): the suite fails against the 0.13.0
//! seam and passes only once the interpreter satisfies every pin below.

use std::cell::Cell;

use aeolus_compile::{
    Config, DynCache, Env, InterpError, InterpNote, Interped, TokenSource, compile_theme, interp,
    interp_theme,
};
use aeolus_ir::{Bp, Cond, Decl, Layer, Program, Prop, Val};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The runtime-tier table over the shadcn light+dark source.
fn runtime_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

/// The baked-tier table over the same source.
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::baked(&source).expect("default-seeded source builds")
}

/// The value one longhand resolved to.
fn resolved(interp: &Interped, prop: Prop) -> Val {
    interp
        .decls
        .iter()
        .find(|d| d.prop == prop)
        .unwrap_or_else(|| panic!("{prop:?} is in the resolved declarations"))
        .val
}

/// The resolved declarations are in the IR's Prop cascade order — the
/// deterministic shape backends read front to back.
fn assert_cascade_order(interp: &Interped) {
    assert!(
        interp.decls.windows(2).all(|w| w[0].prop <= w[1].prop),
        "resolved declarations are Prop-sorted: {:?}",
        interp.decls
    );
}

/// The note one cond produced, if any.
fn note_for(interp: &Interped, cond: Cond) -> Option<&InterpNote> {
    interp.notes.iter().find(|note| match note {
        InterpNote::PseudoElement { cond: c, .. }
        | InterpNote::ContainerUnqueried { cond: c, .. }
        | InterpNote::AttributeUnqueried { cond: c, .. } => *c == cond,
    })
}

// ---------------------------------------------------------------------------
// 1. The cascade: per-longhand last-wins, per-edge dominance
// ---------------------------------------------------------------------------

#[test]
fn base_duplicates_resolve_per_longhand_last_wins() {
    // `p-4 px-8` sorts into ONE base rule: top/bottom carry p-4's 1rem,
    // left/right carry px-8's 2rem — the same read `Program::base_conflicts`
    // documents (a stable cascade-ordered read keeps the LAST declaration).
    let compiled = compile_theme("p-4 px-8", &runtime_table(), &Config::default())
        .expect("the string compiles");
    let interp = interp(&compiled.program, &Env::default()).expect("the base resolves");
    assert_eq!(interp.decls.len(), 4, "the four padding longhands");
    assert_eq!(resolved(&interp, Prop::PaddingTop), Val::Rem(1.0));
    assert_eq!(resolved(&interp, Prop::PaddingBottom), Val::Rem(1.0));
    assert_eq!(resolved(&interp, Prop::PaddingLeft), Val::Rem(2.0));
    assert_eq!(resolved(&interp, Prop::PaddingRight), Val::Rem(2.0));
    assert_cascade_order(&interp);
    assert!(interp.notes.is_empty(), "no exclusions on a plain base");
}

#[test]
fn fired_layers_evict_base_per_longhand_only_where_they_fire() {
    // The per-edge story at IR level: `hover:px-8` wins left/right under
    // hover and leaves top/bottom to the base; without hover the base
    // stands alone.
    let compiled = compile_theme("p-4 hover:px-8", &runtime_table(), &Config::default())
        .expect("the string compiles");
    let resting = interp(&compiled.program, &Env::default()).expect("resolves");
    assert_eq!(resolved(&resting, Prop::PaddingLeft), Val::Rem(1.0));
    let hovered = interp(
        &compiled.program,
        &Env {
            hover: true,
            ..Env::default()
        },
    )
    .expect("resolves");
    assert_eq!(resolved(&hovered, Prop::PaddingLeft), Val::Rem(2.0));
    assert_eq!(
        resolved(&hovered, Prop::PaddingTop),
        Val::Rem(1.0),
        "the hover layer sets only its own longhands — the per-edge mask story"
    );
    assert_cascade_order(&hovered);
}

#[test]
fn interp_is_deterministic_and_pure() {
    let compiled = compile_theme(
        "p-4 hover:bg-primary md:px-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("the string compiles");
    let env = Env {
        hover: true,
        viewport_rem: 64.0,
        ..Env::default()
    };
    let first = interp_theme(&compiled.program, &runtime_table(), &env).expect("resolves");
    let second = interp_theme(&compiled.program, &runtime_table(), &env).expect("resolves");
    assert_eq!(first, second, "same program + env, same answer");
    // And the PROGRAM is untouched by the walk (the interp borrows it):
    assert_eq!(compiled.program.layers.len(), 3);
}

// ---------------------------------------------------------------------------
// 2. Probes answer from the Env
// ---------------------------------------------------------------------------

#[test]
fn element_states_fire_only_under_their_probe() {
    let compiled = compile_theme(
        "p-4 focus:px-8 disabled:py-2 active:p-0",
        &runtime_table(),
        &Config::default(),
    )
    .expect("the string compiles");
    let resting = interp(&compiled.program, &Env::default()).expect("resolves");
    assert_eq!(resolved(&resting, Prop::PaddingLeft), Val::Rem(1.0));
    let focused = interp(
        &compiled.program,
        &Env {
            focus: true,
            ..Env::default()
        },
    )
    .expect("resolves");
    assert_eq!(resolved(&focused, Prop::PaddingLeft), Val::Rem(2.0));
    assert_eq!(resolved(&focused, Prop::PaddingTop), Val::Rem(1.0));
    let disabled = interp(
        &compiled.program,
        &Env {
            disabled: true,
            ..Env::default()
        },
    )
    .expect("resolves");
    assert_eq!(resolved(&disabled, Prop::PaddingTop), Val::Rem(0.5));
    let active = interp(
        &compiled.program,
        &Env {
            active: true,
            ..Env::default()
        },
    )
    .expect("resolves");
    assert_eq!(resolved(&active, Prop::PaddingTop), Val::Rem(0.0));
}

#[test]
fn group_and_peer_probes_answer_from_the_env() {
    let compiled = compile_theme(
        "p-4 group-hover:p-0 peer-focus:p-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("the string compiles");
    let group = interp(
        &compiled.program,
        &Env {
            group_hover: true,
            ..Env::default()
        },
    )
    .expect("resolves");
    assert_eq!(resolved(&group, Prop::PaddingTop), Val::Rem(0.0));
    let peer = interp(
        &compiled.program,
        &Env {
            peer_focus: true,
            ..Env::default()
        },
    )
    .expect("resolves");
    assert_eq!(resolved(&peer, Prop::PaddingTop), Val::Rem(2.0));
}

#[test]
fn min_breakpoints_fire_at_and_above_their_rem() {
    let compiled = compile_theme("sm:p-0 md:p-2 lg:p-4", &runtime_table(), &Config::default())
        .expect("the string compiles");
    // 0rem viewport: no breakpoint fires — and with no base rule in the
    // string, the element has NO resolved declarations at all.
    let narrow = interp(&compiled.program, &Env::default()).expect("resolves");
    assert!(
        narrow.decls.is_empty(),
        "no base rule and no breakpoint fired: nothing to resolve"
    );
    // AT the boundary fires (min-width semantics); equal specificity
    // keeps source order, so the LAST fired layer wins the read.
    let at_sm = interp(
        &compiled.program,
        &Env {
            viewport_rem: Bp::Sm.rem(),
            ..Env::default()
        },
    )
    .expect("resolves");
    assert_eq!(resolved(&at_sm, Prop::PaddingTop), Val::Rem(0.0));
    let at_md = interp(
        &compiled.program,
        &Env {
            viewport_rem: Bp::Md.rem(),
            ..Env::default()
        },
    )
    .expect("resolves");
    assert_eq!(resolved(&at_md, Prop::PaddingTop), Val::Rem(0.5));
    let at_lg = interp(
        &compiled.program,
        &Env {
            viewport_rem: Bp::Lg.rem(),
            ..Env::default()
        },
    )
    .expect("resolves");
    assert_eq!(resolved(&at_lg, Prop::PaddingTop), Val::Rem(1.0));
}

#[test]
fn max_breakpoints_fire_below_their_rem() {
    let compiled = compile_theme("max-md:p-0 p-4", &runtime_table(), &Config::default())
        .expect("the string compiles");
    let below = interp(
        &compiled.program,
        &Env {
            viewport_rem: 47.999_9,
            ..Env::default()
        },
    )
    .expect("resolves");
    assert_eq!(
        resolved(&below, Prop::PaddingTop),
        Val::Rem(0.0),
        "`max-md` is `width < 48rem`: just below the boundary it fires"
    );
    let at = interp(
        &compiled.program,
        &Env {
            viewport_rem: Bp::Md.rem(),
            ..Env::default()
        },
    )
    .expect("resolves");
    assert_eq!(
        resolved(&at, Prop::PaddingTop),
        Val::Rem(1.0),
        "AT the boundary the max form stops firing (CSS `width < Nrem`)"
    );
}

#[test]
fn explicit_min_form_fires_like_its_breakpoint() {
    let compiled = compile_theme("min-md:p-0", &runtime_table(), &Config::default())
        .expect("the string compiles");
    let fired = interp(
        &compiled.program,
        &Env {
            viewport_rem: Bp::Md.rem(),
            ..Env::default()
        },
    )
    .expect("resolves");
    assert_eq!(resolved(&fired, Prop::PaddingTop), Val::Rem(0.0));
}

#[test]
fn container_layers_need_a_container_context() {
    let compiled = compile_theme("@md:p-0 @max-md:p-8", &runtime_table(), &Config::default())
        .expect("the string compiles");
    // No container in the Env: neither layer fires, and the interp
    // REPORTS both (CSS: no eligible container never matches — honest,
    // but never silent).
    let orphan = interp(&compiled.program, &Env::default()).expect("resolves");
    assert!(
        orphan.decls.is_empty(),
        "no base rule, no container context"
    );
    assert!(
        note_for(
            &orphan,
            Cond::Container {
                bp: Bp::Md,
                name: None
            }
        )
        .is_some()
    );
    assert!(
        note_for(
            &orphan,
            Cond::MaxContainer {
                bp: Bp::Md,
                name: None
            }
        )
        .is_some()
    );
    // A container width answers the queries with the same min/max
    // boundary semantics as the viewport forms.
    let at = interp(
        &compiled.program,
        &Env {
            container_rem: Some(Bp::Md.rem()),
            ..Env::default()
        },
    )
    .expect("resolves");
    assert!(
        note_for(
            &at,
            Cond::Container {
                bp: Bp::Md,
                name: None
            }
        )
        .is_none()
    );
    assert_eq!(resolved(&at, Prop::PaddingTop), Val::Rem(0.0));
    let below = interp(
        &compiled.program,
        &Env {
            container_rem: Some(47.0),
            ..Env::default()
        },
    )
    .expect("resolves");
    assert_eq!(resolved(&below, Prop::PaddingTop), Val::Rem(2.0));
}

#[test]
fn pseudo_element_layers_are_render_targets_not_element_state() {
    // `::selection`/`::placeholder` style a DIFFERENT target: the interp
    // resolves the ELEMENT's own cascade, so these layers are excluded
    // and reported — never silently folded in, never an error.
    let compiled = compile_theme(
        "p-4 selection:bg-red-500 placeholder:text-white",
        &runtime_table(),
        &Config::default(),
    )
    .expect("the string compiles");
    let interp = interp(&compiled.program, &Env::default()).expect("resolves");
    assert_eq!(
        resolved(&interp, Prop::PaddingTop),
        Val::Rem(1.0),
        "the element's own base resolves"
    );
    let selection = note_for(&interp, Cond::Selection).expect("the selection layer is reported");
    let placeholder =
        note_for(&interp, Cond::Placeholder).expect("the placeholder layer is reported");
    for note in [selection, placeholder] {
        assert!(
            matches!(note, InterpNote::PseudoElement { .. }),
            "the exclusion rides the pseudo-element note: {note:?}"
        );
    }
    assert!(
        !interp
            .decls
            .iter()
            .any(|d| d.prop == Prop::Background && d.val.is_token()),
        "the selection layer's colour never leaks into the element's cascade"
    );
}

// ---------------------------------------------------------------------------
// 3. Theme dynamism: the TokenSource seam
// ---------------------------------------------------------------------------

#[test]
fn baked_pairs_resolve_by_the_env_mode() {
    let table = baked_table();
    let compiled = compile_theme("bg-primary", &table, &Config::default()).expect("compiles");
    let slot = match compiled.program.layers[0].decls[0].val {
        Val::ModePair(slot) => slot,
        other => panic!("the baked tier lowers a ModePair, got {other:?}"),
    };
    let (light, dark) = table.pair_at(slot).expect("the slot indexes the table");
    let day = interp_theme(&compiled.program, &table, &Env::default()).expect("resolves");
    assert_eq!(resolved(&day, Prop::Background), Val::Rgba(light));
    let night = interp_theme(
        &compiled.program,
        &table,
        &Env {
            dark: true,
            ..Env::default()
        },
    )
    .expect("resolves");
    assert_eq!(resolved(&night, Prop::Background), Val::Rgba(dark));
}

#[test]
fn runtime_tokens_resolve_through_one_load_and_one_multiply() {
    let table = runtime_table();
    let compiled = compile_theme("bg-primary/50", &table, &Config::default()).expect("compiles");
    let id = match compiled.program.layers[0].decls[0].val {
        Val::Tok { id, alpha } => {
            assert_eq!(alpha, 0.5, "the modifier rides the token");
            id
        }
        other => panic!("the runtime tier lowers a Tok, got {other:?}"),
    };
    let interp = interp_theme(&compiled.program, &table, &Env::default()).expect("resolves");
    let resolved_val = resolved(&interp, Prop::Background);
    assert!(
        matches!(resolved_val, Val::Rgba(_)),
        "the interp hands the backend a CONCRETE colour, not a slot: {resolved_val:?}"
    );
    assert_eq!(
        resolved_val,
        Val::Rgba(
            table
                .resolve_rgba(id, 0.5, false)
                .expect("the audited primitive is the oracle")
        ),
        "the built-in source resolves through the theme pass's one-load-one-multiply"
    );
}

#[test]
fn dark_mode_multiplies_the_dark_half() {
    let table = runtime_table();
    let compiled = compile_theme("bg-primary", &table, &Config::default()).expect("compiles");
    let day = interp_theme(&compiled.program, &table, &Env::default()).expect("resolves");
    let night = interp_theme(
        &compiled.program,
        &table,
        &Env {
            dark: true,
            ..Env::default()
        },
    )
    .expect("resolves");
    assert_ne!(
        resolved(&day, Prop::Background),
        resolved(&night, Prop::Background),
        "the mode picks the half the multiply rides"
    );
}

#[test]
fn themeless_interp_refuses_token_values_with_the_typed_boundary() {
    // The theme forms are the theme pass's surface: interp WITHOUT a
    // source refuses them per longhand, the same typed boundary the
    // theme-less compile walk reports (never a silent passthrough, never
    // a truncation).
    let table = runtime_table();
    let compiled = compile_theme("bg-primary p-4", &table, &Config::default()).expect("compiles");
    let err = interp(&compiled.program, &Env::default())
        .expect_err("a token value without a source is the typed boundary");
    assert!(
        matches!(
            err,
            InterpError::ThemeRequired {
                prop: Prop::Background
            }
        ),
        "the error names the longhand that carried the token: {err:?}"
    );
    assert_eq!(err.code(), "aeolus::compile::interp_theme_required");
    assert!(!err.help().is_empty());
    // The baked tier's ModePair form refuses through the same boundary.
    let baked = compile_theme("bg-primary", &baked_table(), &Config::default()).expect("compiles");
    assert!(matches!(
        interp(&baked.program, &Env::default()),
        Err(InterpError::ThemeRequired { .. })
    ));
}

/// A host source that answers every question with a fixed colour and
/// records which questions it was asked — the seam is REAL: the interp
/// consumes the host's answers, not the theme crate's.
struct FixedSource {
    asked_dark: Cell<usize>,
    asked_pair: Cell<usize>,
}

impl FixedSource {
    fn new() -> Self {
        Self {
            asked_dark: Cell::new(0),
            asked_pair: Cell::new(0),
        }
    }
}

impl TokenSource for FixedSource {
    fn resolve(&self, _id: u16, _alpha: f32, dark: bool) -> Result<Val, InterpError> {
        if dark {
            self.asked_dark.set(self.asked_dark.get() + 1);
        }
        Ok(Val::Rgba([10, 20, 30, 40]))
    }

    fn resolve_pair(&self, _slot: u16, _dark: bool) -> Result<Val, InterpError> {
        self.asked_pair.set(self.asked_pair.get() + 1);
        Ok(Val::Rgba([1, 2, 3, 4]))
    }
}

#[test]
fn host_token_sources_wire_through_the_seam() {
    let source = FixedSource::new();
    // Only the dark layer carries a token, so under a dark env the mode
    // rode exactly one token question.
    let compiled = compile_theme(
        "p-4 dark:bg-primary/50",
        &runtime_table(),
        &Config::default(),
    )
    .expect("compiles");
    let night = interp_theme(
        &compiled.program,
        &source,
        &Env {
            dark: true,
            ..Env::default()
        },
    )
    .expect("the host source resolves");
    assert_eq!(
        resolved(&night, Prop::Background),
        Val::Rgba([10, 20, 30, 40]),
        "the interp consumed the HOST's answer"
    );
    assert_eq!(
        source.asked_dark.get(),
        1,
        "the mode rode the token question"
    );
    // A baked program routes the pair question to the pair method:
    let baked = compile_theme("bg-primary", &baked_table(), &Config::default()).expect("compiles");
    interp_theme(&baked.program, &source, &Env::default()).expect("resolves");
    assert_eq!(
        source.asked_pair.get(),
        1,
        "ModePair values ask the pair method"
    );
}

/// A host source that refuses everything — a host's failures are the
/// interp's failures, surfaced typed, never second-guessed.
struct Failing;

impl TokenSource for Failing {
    fn resolve(&self, _id: u16, _alpha: f32, _dark: bool) -> Result<Val, InterpError> {
        Err(InterpError::UnknownToken { id: 7 })
    }

    fn resolve_pair(&self, _slot: u16, _dark: bool) -> Result<Val, InterpError> {
        Err(InterpError::UnknownPair { slot: 3 })
    }
}

#[test]
fn source_failures_surface_typed() {
    let runtime =
        compile_theme("bg-primary", &runtime_table(), &Config::default()).expect("compiles");
    assert_eq!(
        interp_theme(&runtime.program, &Failing, &Env::default()).expect_err("fails"),
        InterpError::UnknownToken { id: 7 }
    );
    let baked = compile_theme("bg-primary", &baked_table(), &Config::default()).expect("compiles");
    assert_eq!(
        interp_theme(&baked.program, &Failing, &Env::default()).expect_err("fails"),
        InterpError::UnknownPair { slot: 3 }
    );
}

/// A one-layer base program over leaked decls — the IR's own documented
/// dynamic-form storage ("one interned `Box<[Decl]>` shared by
/// reference"), which is what lets a test hand-build a program the
/// pipeline would never emit.
fn program_of(decls: &[Decl]) -> Program {
    let decls: &'static [Decl] = Box::leak(decls.to_vec().into_boxed_slice());
    let layers: &'static [Layer] =
        Box::leak(vec![Layer::new(Cond::Base, decls)].into_boxed_slice());
    Program::new(layers)
}

#[test]
fn the_builtin_source_maps_the_theme_pass_refusals() {
    // Hand-built programs (the IR is const-constructible) can carry ids
    // and alphas the pipeline would never emit — the built-in source
    // maps the theme pass's typed refusals onto the interp's own set.
    let table = runtime_table();
    let unknown = program_of(&[Decl {
        prop: Prop::Background,
        val: Val::Tok {
            id: 9_999,
            alpha: 1.0,
        },
    }]);
    assert!(matches!(
        interp_theme(&unknown, &table, &Env::default()),
        Err(InterpError::UnknownToken { id: 9_999 })
    ));
    let bad_alpha = program_of(&[Decl {
        prop: Prop::Background,
        val: Val::Tok {
            id: 0,
            alpha: f32::NAN,
        },
    }]);
    assert!(matches!(
        interp_theme(&bad_alpha, &table, &Env::default()),
        Err(InterpError::AlphaNotFinite { .. })
    ));
    let baked = baked_table();
    let primary = baked.id("primary").expect("primary is a shadcn token");
    let alpha_on_pair = program_of(&[Decl {
        prop: Prop::Background,
        val: Val::Tok {
            id: primary,
            alpha: 0.5,
        },
    }]);
    assert!(matches!(
        interp_theme(&alpha_on_pair, &baked, &Env::default()),
        Err(InterpError::AlphaOnPair { id }) if id == primary
    ));
    let unknown_pair = program_of(&[Decl {
        prop: Prop::Background,
        val: Val::ModePair(9_999),
    }]);
    assert!(matches!(
        interp_theme(&unknown_pair, &baked, &Env::default()),
        Err(InterpError::UnknownPair { slot: 9_999 })
    ));
}

#[test]
fn interp_errors_carry_stable_codes_and_help() {
    let errors = [
        InterpError::ThemeRequired {
            prop: Prop::Background,
        },
        InterpError::UnknownToken { id: 1 },
        InterpError::UnknownPair { slot: 2 },
        InterpError::AlphaOnPair { id: 3 },
        InterpError::AlphaNotFinite { alpha: f32::NAN },
    ];
    for err in &errors {
        assert!(
            err.code().starts_with("aeolus::compile::interp_"),
            "{err:?}"
        );
        assert!(!err.help().is_empty(), "{err:?} carries help");
        assert!(!err.to_string().is_empty(), "{err:?} renders");
    }
}

#[cfg(feature = "miette")]
#[test]
fn interp_errors_carry_miette_code_and_help() {
    use miette::Diagnostic;
    let err = InterpError::ThemeRequired {
        prop: Prop::Background,
    };
    // The structured impl exposes the SAME strings the always-on
    // code()/help() methods carry.
    assert_eq!(
        Diagnostic::code(&err).map(|c| c.to_string()),
        Some("aeolus::compile::interp_theme_required".to_string())
    );
    assert!(Diagnostic::help(&err).is_some());
}

// ---------------------------------------------------------------------------
// 4. Composition: the cache + the interp (the per-frame loop)
// ---------------------------------------------------------------------------

#[test]
fn cached_programs_interp_identically_across_envs() {
    // The intended consumption loop: compile ONCE (the DynCache dedupes
    // the interned slices), interp per frame with a fresh Env.
    let cache = DynCache::new();
    let config = Config::default();
    let compiled = cache
        .compile_theme("p-4 hover:px-8 dark:bg-primary", &runtime_table(), &config)
        .expect("compiles");
    let layers_ptr = compiled.program.layers.as_ptr();
    let resting = interp(&compiled.program, &Env::default()).expect("resolves");
    let hovered = interp(
        &compiled.program,
        &Env {
            hover: true,
            ..Env::default()
        },
    )
    .expect("resolves");
    assert_eq!(resolved(&resting, Prop::PaddingLeft), Val::Rem(1.0));
    assert_eq!(resolved(&hovered, Prop::PaddingLeft), Val::Rem(2.0));
    assert_eq!(
        compiled.program.layers.as_ptr(),
        layers_ptr,
        "the interp borrowed, never re-compiled: the cached slices stand"
    );
}

// ---------------------------------------------------------------------------
// 5. The pure-IR surface
// ---------------------------------------------------------------------------

#[test]
fn an_empty_program_resolves_empty() {
    let interp = interp(&Program::new(&[]), &Env::default()).expect("an empty program is total");
    assert!(interp.decls.is_empty());
    assert!(interp.notes.is_empty());
    assert!(interp.touched().is_empty());
}

#[test]
fn touched_mask_covers_the_resolved_longhands() {
    let compiled =
        compile_theme("p-4 px-8 flex-col", &runtime_table(), &Config::default()).expect("compiles");
    let interp = interp(&compiled.program, &Env::default()).expect("resolves");
    let touched = interp.touched();
    for prop in [
        Prop::PaddingTop,
        Prop::PaddingRight,
        Prop::PaddingBottom,
        Prop::PaddingLeft,
    ] {
        assert!(touched.contains(prop), "{prop:?} is touched");
    }
    for decl in &interp.decls {
        assert!(
            touched.contains(decl.prop),
            "the mask mirrors the resolved declarations: {decl:?}"
        );
    }
}

#[test]
fn keyword_values_pass_through_as_the_ir_carries_them() {
    // `Val::Kw` is row-relative on purpose (the codegen crate owns the
    // names); the interp's contract is to resolve THEME dynamism, not
    // keyword names — keywords pass through untouched.
    let compiled =
        compile_theme("flex-col", &runtime_table(), &Config::default()).expect("compiles");
    let interp = interp(&compiled.program, &Env::default()).expect("resolves");
    assert!(
        interp.decls.iter().any(|d| matches!(d.val, Val::Kw(_))),
        "the keyword value rides through: {:?}",
        interp.decls
    );
}
