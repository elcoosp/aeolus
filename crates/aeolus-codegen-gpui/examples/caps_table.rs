//! Consumer demonstration: the generated capability table (the §3.7
//! generator slice's committed module) — the per-longhand verdicts a
//! host renders without re-running the emit gates. The rows ride
//! `Prop::ALL`'s own cascade order, the tiers are the emit gates' own
//! answers (`supports`/`polyfilled`), and a refused longhand carries
//! the typed boundary reason the gate answers — the docs page's
//! ceiling, readable in-process.
//!
//! ```sh
//! cargo run -p aeolus-codegen-gpui --example caps_table
//! ```

use aeolus_codegen::conformance::Tier;
use aeolus_codegen_gpui::{PROP_CAPABILITIES, PropCapability};

fn main() {
    let native = PROP_CAPABILITIES
        .iter()
        .filter(|row| row.tier == Tier::Native)
        .count();
    let polyfill = PROP_CAPABILITIES
        .iter()
        .filter(|row| row.tier == Tier::Polyfill)
        .count();
    let refused = PROP_CAPABILITIES
        .iter()
        .filter(|row| row.tier == Tier::Refused)
        .count();
    println!(
        "== the GPUI capability table ({} longhands: {native} native / {polyfill} polyfilled / {refused} refused) ==",
        PROP_CAPABILITIES.len(),
    );

    // The polyfilled tier: no native slot, the runtime composes it
    // (the ring family and the filter row's landed half).
    println!("\n── polyfilled (the runtime composes) ──");
    for PropCapability { prop, .. } in PROP_CAPABILITIES
        .iter()
        .filter(|row| row.tier == Tier::Polyfill)
    {
        println!("  {}", prop.name());
    }

    // The refused tier's typed boundaries: the reason the emit gate
    // answers — never a silent drop.
    println!("\n── refused (the typed boundaries) ──");
    for PropCapability { prop, reason, .. } in PROP_CAPABILITIES
        .iter()
        .filter(|row| row.tier == Tier::Refused)
        .take(6)
    {
        println!("  {:<22} {}", prop.name(), reason);
    }
    println!(
        "  … and {} more (the committed table carries every row)",
        refused - 6
    );
}
