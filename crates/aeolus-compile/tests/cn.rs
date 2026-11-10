//! The IR-level `cn()` merge contracts, pinned end to end (architecture
//! plan §3.2 "hosts the `cn()` engine at IR level" + §3.3 "mask algebra
//! + copy").
//!
//! Five families:
//! 1. **The merge** — same-condition layers merge by `Cond` equality,
//!    declarations concatenate left-then-right, and the per-longhand
//!    LAST occurrence wins (the tw_merge "rightmost class wins"
//!    contract, carried by the longhand IR — no group table needed at
//!    this level, the fan-outs ARE the groups).
//! 2. **The differential contract** — merging separately compiled
//!    programs resolves to exactly what compiling the concatenated
//!    string resolves to, and the removed declarations are precisely
//!    the overlap `Program::base_conflicts()` names on the raw
//!    concatenation.
//! 3. **Conditions** — disjoint conditions stay separate layers,
//!    specificity-sorted; equal conditions fold into one layer.
//! 4. **Honest reports** — diagnostics concatenate in input order with
//!    their original spans (the inputs are separate strings; the merge
//!    never offsets or drops them).
//! 5. **Algebra** — the merge is idempotent and associative, and the
//!    theme tier's token values merge with the rightmost alpha winning.
//!
//! Written FIRST (the RED spec): the suite fails against a tree without
//! `cn()` and passes only once the merge satisfies every pin below.

use aeolus_compile::{Config, DiagnosticKind, cn, compile, compile_theme};
use aeolus_ir::{Bp, Cond, Decl, Prop, Val};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The base layer's declarations (layer 0 of a compiled program).
fn base(program: &aeolus_ir::Program) -> &[Decl] {
    program.layers.first().map_or(&[], |l| l.decls)
}

/// The front-to-back read of a program: one slot per (condition,
/// longhand), the LAST declaration per slot winning — the read a
/// backend performs. This is the equivalence the differential pins are
/// stated over: two programs are merge-equivalent when their resolved
/// slots are equal.
fn slots(program: &aeolus_ir::Program) -> Vec<(Cond, Prop, Val)> {
    let mut out = Vec::new();
    for layer in program.layers {
        for decl in layer.decls {
            let slot = (layer.cond, decl.prop);
            match out.iter_mut().find(|(c, p, _)| (*c, *p) == slot) {
                Some((_, _, val)) => *val = decl.val,
                None => out.push((layer.cond, decl.prop, decl.val)),
            }
        }
    }
    out
}

const PADDING_X_MASK: aeolus_ir::PropMask =
    aeolus_ir::PropMask::of(&[Prop::PaddingRight, Prop::PaddingLeft]);

// ---------------------------------------------------------------------------
// 1. The merge
// ---------------------------------------------------------------------------

#[test]
fn cn_conflict_free_inputs_pass_through() {
    let a = compile("p-4", &Config::default()).expect("p-4 compiles");
    let b = compile("hover:p-1", &Config::default()).expect("hover:p-1 compiles");
    let merged = cn(&[&a, &b]).expect("two conflict-free programs merge");
    assert!(merged.diagnostics.is_empty());
    assert_eq!(
        merged.program.layers.len(),
        2,
        "base + hover stay two layers"
    );
    let expected = [slots(&a.program), slots(&b.program)].concat();
    assert_eq!(
        slots(&merged.program),
        expected,
        "the merge is the slot union, in order"
    );
    // The pass-through is the normalization too: a single input comes
    // back with the same resolved slots.
    let single = cn(&[&a]).expect("one program merges");
    assert_eq!(slots(&single.program), slots(&a.program));
}

#[test]
fn cn_base_overlap_dedupes_rightmost_wins() {
    // The v0.x cn story, at the IR level: `p-4` fans out the four
    // padding longhands at 1rem, `px-8` re-sets left/right at 2rem.
    // The merged base layer carries FOUR declarations, not six, and
    // the left/right values are the rightmost program's.
    let a = compile("p-4", &Config::default()).expect("p-4 compiles");
    let b = compile("px-8", &Config::default()).expect("px-8 compiles");
    let merged = cn(&[&a, &b]).expect("merge");
    let decls = base(&merged.program);
    assert_eq!(
        decls.len(),
        4,
        "the overlap is merged away, not appended: {decls:?}"
    );
    let value_of = |prop: Prop| decls.iter().find(|d| d.prop == prop).map(|d| d.val);
    assert_eq!(
        value_of(Prop::PaddingTop),
        Some(Val::Rem(1.0)),
        "top/bottom keep p-4"
    );
    assert_eq!(
        value_of(Prop::PaddingBottom),
        Some(Val::Rem(1.0)),
        "top/bottom keep p-4"
    );
    assert_eq!(
        value_of(Prop::PaddingLeft),
        Some(Val::Rem(2.0)),
        "left/right ride the rightmost program (rightmost wins)"
    );
    assert_eq!(
        value_of(Prop::PaddingRight),
        Some(Val::Rem(2.0)),
        "rightmost wins"
    );
}

#[test]
fn cn_normalizes_a_single_overlapping_input() {
    // The same string compiled as ONE input still normalizes: the merge
    // is total, not a two-program special case. `compile("p-4 px-8")`
    // emits the raw cascade (six base decls, the documented v0.11.0
    // contract); `cn` on it merges the six down to four.
    let compiled = compile("p-4 px-8", &Config::default()).expect("compiles");
    assert_eq!(
        base(&compiled.program).len(),
        6,
        "the raw cascade keeps both fan-outs"
    );
    let merged = cn(&[&compiled]).expect("one input merges");
    assert_eq!(base(&merged.program).len(), 4, "the overlap dedupes");
    assert!(
        merged.program.base_conflicts().is_empty(),
        "no conflicts survive the merge"
    );
}

// ---------------------------------------------------------------------------
// 2. The differential contract
// ---------------------------------------------------------------------------

#[test]
fn cn_removes_exactly_the_base_conflict_bits() {
    // On the RAW concatenation the overlap is PADDING_X (the bits a
    // later same-condition group re-sets — pinned by the compile suite);
    // the merge removes exactly those declarations.
    let concat = compile("p-4 px-8", &Config::default()).expect("compiles");
    assert_eq!(concat.program.base_conflicts(), PADDING_X_MASK);
    let a = compile("p-4", &Config::default()).expect("p-4 compiles");
    let b = compile("px-8", &Config::default()).expect("px-8 compiles");
    let merged = cn(&[&a, &b]).expect("merge");
    assert!(
        merged.program.base_conflicts().is_empty(),
        "the merge leaves the conflict bits empty"
    );
    assert_eq!(
        merged.program.touched(),
        concat.program.touched(),
        "same longhand surface"
    );
}

#[test]
fn cn_is_value_equivalent_to_compiling_the_concatenation() {
    // THE differential contract: merging separately compiled programs
    // resolves to exactly what compiling the concatenated string
    // resolves to (the front-to-back read per (condition, longhand)).
    let a = compile("p-4 px-2", &Config::default()).expect("a compiles");
    let b = compile("hover:p-1", &Config::default()).expect("b compiles");
    let merged = cn(&[&a, &b]).expect("merge");
    let concat = compile("p-4 px-2 hover:p-1", &Config::default()).expect("concat compiles");
    assert_eq!(
        slots(&merged.program),
        slots(&concat.program),
        "cn(a, b) resolves like compile(a + \" \" + b)"
    );
    // The raw forms differ only by the dedupe the merge owns: the
    // concatenated program keeps the duplicate cascade (6 base decls),
    // the merged one does not (4).
    assert_eq!(base(&concat.program).len(), 6);
    assert_eq!(base(&merged.program).len(), 4);
}

// ---------------------------------------------------------------------------
// 3. Conditions
// ---------------------------------------------------------------------------

#[test]
fn cn_conditional_layers_merge_by_cond_equality() {
    let a = compile("hover:p-4", &Config::default()).expect("a compiles");
    let b = compile("hover:px-8", &Config::default()).expect("b compiles");
    let merged = cn(&[&a, &b]).expect("merge");
    assert_eq!(
        merged.program.layers.len(),
        1,
        "equal conditions fold into ONE layer"
    );
    let layer = &merged.program.layers[0];
    assert_eq!(layer.cond, Cond::Hover);
    assert_eq!(
        layer.decls.len(),
        4,
        "hover:p-4 ⊕ hover:px-8 is the four-longhand cascade"
    );
}

#[test]
fn cn_disjoint_conditions_stay_separate_and_specificity_sorted() {
    // Probe states rank below media queries (cond.rs: probe 2 < media 4),
    // so the hover layer sorts before the md layer in BOTH inputs' merge.
    let a = compile("hover:p-4", &Config::default()).expect("a compiles");
    let b = compile("md:px-8", &Config::default()).expect("b compiles");
    let merged = cn(&[&a, &b]).expect("merge");
    let conds: Vec<Cond> = merged.program.layers.iter().map(|l| l.cond).collect();
    assert_eq!(
        conds,
        vec![Cond::Hover, Cond::Bp(Bp::Md)],
        "specificity order, stable"
    );
    assert!(merged.program.is_sorted());
}

// ---------------------------------------------------------------------------
// 4. Honest reports
// ---------------------------------------------------------------------------

#[test]
fn cn_diagnostics_concatenate_in_input_order() {
    // The merge never drops or offsets a report: left's diagnostics
    // come first with their own input's spans, then right's.
    let a = compile("not-a-utility p-4", &Config::default()).expect("a degrades, not fails");
    assert_eq!(a.diagnostics.len(), 1, "the unknown root is one report");
    let b = compile("has-data-[x]:p-4", &Config::default()).expect("b degrades, not fails");
    assert_eq!(b.diagnostics.len(), 1, "the untyped variant is one report");
    let merged = cn(&[&a, &b]).expect("merge");
    assert_eq!(merged.diagnostics.len(), 2, "both reports ride the merge");
    assert_eq!(merged.diagnostics[0].raw, "not-a-utility");
    assert_eq!(
        merged.diagnostics[0].span,
        (0, 13),
        "the span is the ORIGINAL input's bytes"
    );
    assert!(matches!(
        merged.diagnostics[1].kind,
        DiagnosticKind::UnsupportedVariant { .. }
    ));
}

// ---------------------------------------------------------------------------
// 5. Algebra
// ---------------------------------------------------------------------------

#[test]
fn cn_is_idempotent() {
    let x = compile("p-4 hover:p-1", &Config::default()).expect("x compiles");
    let merged = cn(&[&x, &x]).expect("merge");
    assert_eq!(
        merged, x,
        "cn(x, x) == x (conflict-free input, so normalization is a no-op)"
    );
}

#[test]
fn cn_is_associative() {
    // The pinned triple: (a ⊕ b) ⊕ c == a ⊕ (b ⊕ c) — the merge is
    // "concatenate in order, last per longhand wins", which groups
    // freely. The triple exercises both dimensions: base overlap AND a
    // conditional layer riding the rightmost input.
    let a = compile("p-4", &Config::default()).expect("a compiles");
    let b = compile("px-8", &Config::default()).expect("b compiles");
    let c = compile("p-2 hover:text-sm", &Config::default()).expect("c compiles");
    let ab = cn(&[&a, &b]).expect("a ⊕ b");
    let left = cn(&[&ab, &c]).expect("(a ⊕ b) ⊕ c");
    let bc = cn(&[&b, &c]).expect("b ⊕ c");
    let right = cn(&[&a, &bc]).expect("a ⊕ (b ⊕ c)");
    assert_eq!(left, right, "the merge is associative");
}

#[test]
fn cn_theme_tokens_merge_with_the_rightmost_alpha_winning() {
    // The theme tier's token values merge per longhand too: the same
    // colour token compiled twice, the rightmost alpha riding the Tok
    // slot (T2's one-multiply contract).
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let tokens = TokenTable::runtime(&source).expect("default-seeded source builds");
    let a = compile_theme("bg-primary", &tokens, &Config::default()).expect("a compiles");
    let b = compile_theme("bg-primary/50", &tokens, &Config::default()).expect("b compiles");
    let merged = cn(&[&a, &b]).expect("merge");
    let decls = base(&merged.program);
    assert_eq!(decls.len(), 1, "one background longhand survives");
    let id = tokens.id("primary").expect("primary is a token");
    assert_eq!(
        decls[0],
        Decl {
            prop: Prop::Background,
            val: Val::Tok { id, alpha: 0.5 }
        },
        "the rightmost program's alpha rides the token slot"
    );
}

#[test]
fn cn_result_is_sorted_and_mask_consistent() {
    let a = compile("hover:p-4", &Config::default()).expect("a compiles");
    let b = compile("p-4 px-8", &Config::default()).expect("b compiles");
    let merged = cn(&[&a, &b]).expect("merge");
    assert!(merged.program.is_sorted(), "specificity non-decreasing");
    for layer in merged.program.layers {
        let mut mask = aeolus_ir::PropMask::EMPTY;
        for decl in layer.decls {
            mask = mask.insert(decl.prop);
        }
        assert_eq!(layer.mask, mask, "the mask mirrors the merged declarations");
    }
}
