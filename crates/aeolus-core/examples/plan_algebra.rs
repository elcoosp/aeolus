//! Consumer example: plan algebra — compose a component's resolved plan
//! with per-instance overrides, read the repaint-vs-relayout lanes the
//! diff reports, and persist the composed plan as the canonical snapshot
//! artifact.
//!
//! ```sh
//! cargo run -p aeolus-core --example plan_algebra
//! ```
//!
//! `StylePlan::overlay` is the plan-level mirror of `cn()`: later wins
//! per slot, layers merge by condition, and the composition is
//! diff-integrated so backends keep their repaint-only vs relayout lanes.

use aeolus_core::{Resolver, StylePlan, diff, shadcn_default};

fn main() {
    let theme = shadcn_default().expect("the embedded palette parses");
    let resolver = Resolver::new(&theme);

    // A component's cached plan (e.g. the shadcn Button's base) ...
    let base = resolver
        .resolve("inline-flex items-center rounded-md bg-primary px-4 py-2 text-sm")
        .expect("the base resolves")
        .plan;
    // ... plus one instance's overrides, with no theme reference needed:
    let overrides = resolver
        .resolve("px-8 rounded-full uppercase")
        .expect("the overrides resolve")
        .plan;

    let merged = base.overlay(&overrides);
    println!("base      : inline-flex items-center rounded-md bg-primary px-4 py-2 text-sm");
    println!("overrides : px-8 rounded-full uppercase");
    println!("merged    : overlay later-wins-per-slot, layers keep their conditions\n");

    // The composition is diff-integrated: the diff of base → merged is
    // EXACTLY the override's slots, still lane-tagged.
    let d = diff(&base.base, &merged.base);
    println!("diff ({} changed slot(s)):", d.changed.len());
    for slot in &d.changed {
        println!("  {slot:?} ({:?})", slot.lane());
    }
    println!(
        "lane: {} — a repaint-only diff restyles without re-layout",
        if d.needs_layout() {
            "relayout"
        } else {
            "repaint-only"
        }
    );

    // Overlay is idempotent (composing the same overrides again is a no-op).
    let twice = merged.overlay(&overrides);
    assert_eq!(twice, merged, "overlay is idempotent");

    // And the composed plan is an artifact: canonical JSON in, strict
    // re-parse out, equal plan back.
    let json = merged.to_snapshot_json();
    let back = StylePlan::from_snapshot_json(&json).expect("canonical JSON re-parses");
    assert_eq!(back, merged, "the artifact preserves the composed plan");
    println!(
        "\nmerged plan → {} bytes of canonical JSON → strict re-parse OK",
        json.len()
    );
}
