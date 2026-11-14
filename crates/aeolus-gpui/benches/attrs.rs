//! The attribute-bits host state seam's microbenchmarks (divan, stable
//! — the workspace bench idiom): the base walk over the same shadcn
//! Button string the compile-seam and dyn-cache benches key on, with
//! and without the element's attribute bits in hand.
//!
//! The walk is one pass over the interned layers either way — the seam
//! adds one probe lookup per `data-*:`/`aria-*:` layer, no allocation.
//! The three arms pin the cost split: the sourceless walk (attribute
//! layers skip), the bit-miss walk (the empty-table element) and the
//! bit-hit walk (the `aria-invalid` bit answers — the Button string's
//! own state layer firing).

use aeolus_gpui::prelude::*;
use divan::Bencher;

fn main() {
    divan::main();
}

/// The shadcn v4 Button table verbatim (base + default variant + default
/// size) minus the classes outside this slice's typed surface (the
/// arbitrary `[&_svg]` forms, `has-[>svg]:`, `transition-all` — the
/// macro's honest refusals). It carries its own `aria-invalid:` layers,
/// so the probe walk has work to do. (`tw!` compiles a literal, so the
/// program builds in one helper the three benches share.)
fn button_program() -> StaticProgram {
    tw!(
        "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2"
    )
}

/// The element's attribute bits: the one bit the Button string probes
/// (the `aria-invalid`"true"state a form host carries on a failed
/// submit).
fn button_bits() -> [AttrBit<'static>; 1] {
    [AttrBit {
        kind: AttrKind::Aria,
        name: "invalid",
        value: "true",
    }]
}

#[divan::bench]
fn tw_sourceless(bencher: Bencher) {
    let program = button_program();
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

#[divan::bench]
fn tw_probes_bit_miss(bencher: Bencher) {
    let program = button_program();
    let rows: [AttrBit<'static>; 0] = [];
    let state = AttrState::new(&rows);
    bencher.bench(|| gpui::StyleRefinement::default().tw_probes(program, &state));
}

#[divan::bench]
fn tw_probes_bit_hit(bencher: Bencher) {
    let program = button_program();
    let rows = button_bits();
    let state = AttrState::new(&rows);
    bencher.bench(|| gpui::StyleRefinement::default().tw_probes(program, &state));
}
