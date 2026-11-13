//! The IR-level overlay seam's contracts (architecture plan §3.6's
//! surface (2) note — "User overrides go through `cn!` which resolves
//! at compile time when static, or by `Program::overlay` (mask + copy)
//! when not" — and §3.3's "Overlay/`cn()` at IR level = mask algebra +
//! copy"): the composition of two programs the pipeline's `cn()`
//! delegate-folds through, pinned end to end.
//!
//! Written FIRST (the RED spec): the suite fails until `overlay`
//! exists.

use aeolus_compile::{Compiled, Config, cn, compile, overlay};
use aeolus_ir::{Cond, Decl, Layer, Program, Prop, Val};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compile a class string, dropping the (demo-clean) diagnostics.
fn compiled(classes: &str) -> Compiled {
    let out = compile(classes, &Config::default()).expect("non-empty input compiles");
    assert!(
        out.diagnostics.is_empty(),
        "{classes:?} lowers clean: {:?}",
        out.diagnostics
    );
    out
}

/// The declarations one program's BASE layer carries, as (prop, val)
/// pairs — the read side the pins assert through.
fn base_decls(program: &Program) -> Vec<(Prop, Val)> {
    program.layers[0]
        .decls
        .iter()
        .map(|decl| (decl.prop, decl.val))
        .collect()
}

/// A hand-frozen program (the macro's own shape): one base layer, the
/// slice leaked — the static form a `tw!`-style freeze hands the host.
fn frozen(cond: Cond, decls: Vec<Decl>) -> Program {
    let decls: &'static [Decl] = Box::leak(decls.into_boxed_slice());
    let layers: &'static [Layer] = Box::leak(vec![Layer::new(cond, decls)].into_boxed_slice());
    Program::new(layers)
}

fn decl(prop: Prop, val: Val) -> Decl {
    Decl { prop, val }
}

// ---------------------------------------------------------------------------
// The composition algebra
// ---------------------------------------------------------------------------

#[test]
fn overlay_last_wins_per_longhand() {
    // `p-4 ⊕ px-8`: the four-longhand cascade — the override's
    // left/right win, the base's top/bottom stand. The overlap the
    // composition removes is exactly `base_conflicts()`'s bits.
    let base = compiled("p-4");
    let over = compiled("px-8");
    let merged = overlay(&base.program, &over.program);
    let decls = base_decls(&merged);
    let pt = decls
        .iter()
        .find(|(prop, _)| *prop == Prop::PaddingTop)
        .expect("padding-top present");
    let pl = decls
        .iter()
        .find(|(prop, _)| *prop == Prop::PaddingLeft)
        .expect("padding-left present");
    // The spacing scale: 4 units = 1rem, 8 horizontal units = 2rem.
    assert_eq!(pt.1, Val::Rem(1.0));
    assert_eq!(pl.1, Val::Rem(2.0));
    assert!(
        merged.base_conflicts().is_empty(),
        "the merge's own conflict-freedom contract"
    );
}

#[test]
fn overlay_folds_same_condition_layers() {
    // Both sides carry base layers: ONE base layer out. Disjoint
    // conditions stay separate layers.
    let base = compiled("p-4 rounded-md");
    let over = compiled("px-8");
    let merged = overlay(&base.program, &over.program);
    assert_eq!(merged.layers.len(), 1, "the base layers folded");
    let double = overlay(&compiled("p-4").program, &compiled("hover:p-8").program);
    assert!(
        double.layers.len() == 2,
        "disjoint conditions stay separate: {:?}",
        double.layers.iter().map(|l| l.cond).collect::<Vec<_>>()
    );
}

#[test]
fn overlay_resorts_by_specificity() {
    // The result is specificity-sorted regardless of the composition
    // order — a hover program overlaid ONTO and the base overlaid
    // UNDER produce the same layer order (stable, source order breaks
    // ties).
    let base = compiled("p-4").program;
    let hover = compiled("hover:p-8").program;
    let a = overlay(&base, &hover);
    let b = overlay(&hover, &base);
    assert!(a.is_sorted());
    assert!(b.is_sorted());
    assert_eq!(
        a.layers.iter().map(|l| l.cond).collect::<Vec<_>>(),
        b.layers.iter().map(|l| l.cond).collect::<Vec<_>>(),
        "the sort, not the composition order, decides"
    );
}

#[test]
fn overlay_is_differential_with_cn() {
    // THE pin: the composition `cn()` delegate-folds through is the
    // same walk — overlay(a, b) == cn(&[a, b]).program, and the N-ary
    // fold groups associatively.
    let a = compiled("p-4 rounded-md gap-2");
    let b = compiled("px-8 rounded-full hover:p-8");
    let c = compiled("text-sm dark:p-8");
    let direct = cn(&[&a, &b, &c]).expect("non-empty inputs merge");
    let folded = overlay(&overlay(&a.program, &b.program), &c.program);
    assert_eq!(
        direct.program.layers, folded.layers,
        "cn is the diagnostic-carrying form of the same composition"
    );
    let pair = overlay(&a.program, &b.program);
    let via_cn = cn(&[&a, &b]).expect("non-empty inputs merge");
    assert_eq!(pair.layers, via_cn.program.layers);
}

#[test]
fn overlay_is_idempotent_and_associative() {
    // The algebra the plan promises (the merge's own RED-pinned laws,
    // now at the composition level).
    let a = compiled("p-4 rounded-md").program;
    let b = compiled("px-8").program;
    let c = compiled("hover:p-8").program;
    // Idempotence: a ⊕ a == a.
    assert_eq!(overlay(&a, &a).layers, a.layers);
    // Associativity: (a ⊕ b) ⊕ c == a ⊕ (b ⊕ c).
    let left = overlay(&overlay(&a, &b), &c);
    let right = overlay(&a, &overlay(&b, &c));
    assert_eq!(left.layers, right.layers);
}

#[test]
fn overlay_interns_each_composition() {
    // The interning story, honest: each composition call interns its
    // result into the leak pool — the SAME contract `cn` documents
    // ("one merge per distinct composition is the intended call
    // pattern"); the DynCache is the bounded dedupe over the COMPILE
    // surface, not the composition one. The pins assert the RESULT
    // equality (the algebra), not a pointer identity no cache provides.
    let a = compiled("p-4").program;
    let b = compiled("px-8").program;
    let first = overlay(&a, &b);
    let second = overlay(&a, &b);
    assert_eq!(first.layers, second.layers);
}

#[test]
fn overlay_handles_empty_programs() {
    // The total function: an empty side contributes nothing, two empty
    // sides compose to the empty program.
    let empty = Program::new(&[]);
    let some = compiled("p-4").program;
    assert_eq!(overlay(&empty, &some).layers, some.layers);
    assert_eq!(overlay(&some, &empty).layers, some.layers);
    assert_eq!(overlay(&empty, &empty).layers, empty.layers);
}

// ---------------------------------------------------------------------------
// The static ⊕ dynamic use case (§3.6's surface (2))
// ---------------------------------------------------------------------------

#[test]
fn overlay_composes_the_frozen_form_with_a_runtime_compile() {
    // The sketch's use case: a `tw!`-frozen static program (the
    // macro's &'static form) overlaid with a RUNTIME-compiled user
    // override — the composition the non-compile-time path exists
    // for. The result stays the IR's own interned form.
    let frozen_base = frozen(
        Cond::Base,
        vec![
            decl(Prop::PaddingTop, Val::Rem(1.0)),
            decl(Prop::PaddingRight, Val::Rem(1.0)),
            decl(Prop::PaddingBottom, Val::Rem(1.0)),
            decl(Prop::PaddingLeft, Val::Rem(1.0)),
            decl(Prop::Color, Val::Rgba([255, 255, 255, 255])),
        ],
    );
    let runtime_override = compiled("px-8").program;
    let merged = overlay(&frozen_base, &runtime_override);
    let decls = base_decls(&merged);
    assert_eq!(
        decls
            .iter()
            .find(|(p, _)| *p == Prop::PaddingLeft)
            .map(|(_, v)| *v),
        Some(Val::Rem(2.0)),
        "the runtime override wins its longhands"
    );
    assert_eq!(
        decls
            .iter()
            .find(|(p, _)| *p == Prop::Color)
            .map(|(_, v)| *v),
        Some(Val::Rgba([255, 255, 255, 255])),
        "the frozen base keeps the longhands the override never touched"
    );
}

#[test]
fn compiled_overlay_method_rides_the_seam() {
    // The ergonomic half: `Compiled::overlay` composes a bare program
    // onto a compile's program — the compile's diagnostics ride (the
    // override is a bare Program, it carries no reports of its own).
    let base = compiled("p-4");
    let over = compiled("px-8").program;
    let via_method = base.overlay(&over);
    let via_fn = overlay(&base.program, &over);
    assert_eq!(
        via_method.program.layers, via_fn.layers,
        "the same walk under both entry points"
    );
    assert!(via_method.diagnostics.is_empty());
    // And the honest case: the base's diagnostics ride the merge.
    let noisy = compile("p-4 not-a-utility", &Config::default()).expect("non-empty input compiles");
    assert_eq!(noisy.diagnostics.len(), 1);
    let merged = noisy.overlay(&over);
    assert_eq!(
        merged.diagnostics.len(),
        1,
        "the base's reports are never dropped by the composition"
    );
}
