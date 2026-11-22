//! Consumer demonstration: the themed bake (architecture plan §3.4's
//! T0/T1 tiers at EXPANSION time). The `theme:` argument reads a host
//! CSS sheet relative to this crate's `CARGO_MANIFEST_DIR`, bakes it
//! through `ThemeSource::from_app_css` + `TokenTable::baked` and
//! carries an `include_str!` rebuild anchor — Cargo re-runs rustc when
//! the CSS changes. The no-arg form stays the embedded shadcn bake,
//! byte-identical. Headless: the expansion is a plain `StaticProgram`
//! value; the example inspects its layers.
//!
//! ```sh
//! cargo run -p aeolus-macro --example themed_program
//! ```

use aeolus_ir::Val;
use aeolus_macro::{cn_static, tw, tw_static, variants_static};

// The closed-world table on the NEUTRAL lane: no GPUI audit rides, the
// expansion is the plain static program every backend consumes.
variants_static! {
    pub Badge {
        base: "inline-flex items-center rounded-md px-2 py-0.5 text-xs font-medium",
        variant: { default: "bg-primary text-primary-foreground", outline: "border border-accent/50 text-accent" },
    }
}

fn main() {
    // The themed bake: `examples/fixtures/theme.css` is the example's own sheet —
    // `@theme` tokens plus a `.dark` block, the single file a web v4
    // project uses. `--color-primary` carries a dark override, so the
    // token bakes to a `ModePair` const pair (T1); `border-accent/50`
    // premultiplies the light literal (T0). The colour suffixes are the
    // registry's closed table — the sheet OVERRIDES those names' values
    // (a brand-new token name is the registry wave's boundary).
    let program = tw_static!(theme: "examples/fixtures/theme.css", "bg-primary text-primary-foreground border-accent/50 p-4");
    println!("── the themed bake (expansion-time, T0/T1) ──");
    println!("  layers: {}", program.layers.len());
    for layer in program.program().layers {
        for decl in layer.decls {
            println!("  {:?} -> {:?}", decl.prop, decl.val);
        }
    }
    let pair = program
        .program()
        .layers
        .iter()
        .flat_map(|l| l.decls.iter())
        .find(|d| matches!(d.val, Val::ModePair(_)));
    println!(
        "  dark pair present: {} (the plan §3.4 T1 shape)",
        pair.is_some()
    );

    // The GPUI-gated entry takes the same theme argument: the same
    // bake, plus the audit gate (a capability GPUI cannot express stays
    // a compile error here — the neutral lane's freeze is the host's
    // choice).
    let gated = tw!(theme: "examples/fixtures/theme.css", "bg-primary text-primary-foreground p-4");
    println!("── the gated themed bake (tw!, GPUI audit riding) ──");
    println!("  layers: {}", gated.layers.len());

    // The compile-time merge takes the theme argument once, at the
    // front of the list.
    let merged = cn_static!(theme: "examples/fixtures/theme.css", "p-4", "px-8");
    println!("── the merged themed bake (cn_static!) ──");
    println!("  layers: {}", merged.layers.len());

    // The variants table: every arm bakes through the same table.
    let badge = Badge::style(BadgeVariant::Outline);
    println!("── the neutral variants table ──");
    println!("  outline badge layers: {}", badge.layers.len());
    println!(
        "  (no runtime tokens ride: every colour is a literal or a const pair — \
         the `theme!` macro stays the T2 runtime handle)"
    );
}
