//! Consumer example: resolve a class string into the typed IR — the
//! "what did my classes mean" entry point. No widget, no window: the
//! semantic layer is framework-agnostic.
//!
//! ```sh
//! cargo run -p aeolus-core --example resolve
//! ```
//!
//! Covers the three things a consumer usually asks for first: which slots
//! the classes set (base + conditional layers), the verification CSS the
//! emitter produces for them (v4's own declaration shapes), and the
//! canonical plan-snapshot JSON artifact with its strict re-parse.

use aeolus_core::{Resolver, StylePlan, format_rules, shadcn_default};

/// The shadcn Button default-variant class string — the same one the
/// benches resolve, so the shape consumers see matches the measured path.
const BUTTON: &str = "inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50";

fn main() {
    // Once at startup: the embedded shadcn light palette becomes the theme.
    // (`shadcn_dark()` layers the dark palette the same way.)
    let theme = shadcn_default().expect("the embedded palette parses");
    let resolver = Resolver::new(&theme);

    // Classes in, typed IR out. Every class a backend cannot express comes
    // back as a precise warning — never dropped silently.
    let resolution = resolver
        .resolve(BUTTON)
        .expect("the Button table resolves warning-free");

    println!("classes : {BUTTON}\n");
    let slots = resolution.plan.base.slots();
    println!("base slots ({}):", slots.len());
    for slot in &slots {
        println!("  - {slot:?}");
    }
    println!("\nconditional layers ({}):", resolution.plan.layers.len());
    for layer in &resolution.plan.layers {
        println!("  - {:?} → {:?}", layer.condition, layer.style.slots());
    }
    println!(
        "\nwarnings: {} (the no-silent-failures rule — drops are reported, not swallowed)",
        resolution.warnings.len()
    );

    // The verification emitter reproduces v4's own per-class CSS shapes
    // (a debugging/verification tool — backends still consume StylePlan).
    let css = resolver
        .to_css("p-4 bg-primary hover:bg-primary/90")
        .expect("the classes resolve");
    println!("\nverification CSS for `p-4 bg-primary hover:bg-primary/90`:");
    print!("{}", format_rules(&css));

    // The IR is also an artifact: canonical, byte-stable JSON with a
    // STRICT reader (unknown fields are errors, never defaults).
    let json = resolution.plan.to_snapshot_json();
    let reparsed = StylePlan::from_snapshot_json(&json).expect("the writer's own output re-parses");
    assert_eq!(reparsed, resolution.plan, "snapshot round-trip is exact");
    println!(
        "\nsnapshot artifact: {} bytes, schema v{}, strict round-trip OK",
        json.len(),
        aeolus_core::PLAN_SNAPSHOT_VERSION
    );
}
