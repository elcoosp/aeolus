//! Consumer demonstration: the named-container fold (plan §3.5's
//! `@container` row, the v0.9 oracle's `@md/name` form). The container
//! variant's NAME rides the typed [`Cond`] surface — the named and
//! unnamed containers are DIFFERENT conditions (two different
//! containers are two different cascade slots, never one shared
//! condition), the CSS arm renders the v4 named query, the plan bridge
//! fills the v0.x oracle's own `Atom::Container.name`, and the interp
//! answers the unnamed question from the Env while the named one
//! reports (`ContainerUnqueried` — the Env carries the nearest
//! container only).
//!
//! ```sh
//! cargo run -p aeolus-compile --example named_containers
//! ```

use aeolus_compile::{Config, Env, InterpNote, compile, emit_css, emit_plan, interp};
use aeolus_core::{Resolver, TAILWIND_DEFAULT_THEME, TailwindTheme};
use aeolus_ir::{Bp, Cond};

fn main() {
    let config = Config::default();

    println!("── the fold: named and unnamed are different conditions ──");
    let named = compile("@md/card:flex", &config).expect("named compiles");
    let unnamed = compile("@md:flex", &config).expect("unnamed compiles");
    println!("  @md/card:flex → {:?}", named.program.layers[0].cond);
    println!("  @md:flex      → {:?}", unnamed.program.layers[0].cond);
    assert_ne!(named.program.layers[0].cond, unnamed.program.layers[0].cond);
    let max = compile("@max-lg/sidebar:hidden", &config).expect("max compiles");
    println!(
        "  @max-lg/sidebar:hidden → {:?}",
        max.program.layers[0].cond
    );
    assert_eq!(
        max.program.layers[0].cond,
        Cond::MaxContainer {
            bp: Bp::Lg,
            name: Some("sidebar"),
        }
    );

    println!("── the CSS arm: the v4 named query ──");
    let sheet = emit_css("@md/card:w-64 @md:w-full", &config).expect("css emits");
    println!("{sheet}");

    println!("── the plan bridge: the v0.x oracle's own name field ──");
    let sheet = emit_plan("@md/card:w-64", &config).expect("plan emits");
    let theme = TailwindTheme::from_theme_css(TAILWIND_DEFAULT_THEME).expect("theme");
    let oracle = Resolver::new(&theme)
        .resolve("@md/card:w-64")
        .expect("oracle resolves");
    let named_atom = |plan: &aeolus_core::StylePlan| {
        plan.rules
            .iter()
            .flat_map(|rule| rule.when.0.iter())
            .find_map(|atom| match atom {
                aeolus_core::cond::Atom::Container { name, .. } => Some(name.clone()),
                _ => None,
            })
    };
    println!(
        "  bridge: Atom::Container.name = {:?}; oracle: {:?}",
        named_atom(&sheet.plan),
        named_atom(&oracle.plan)
    );

    println!("── the interp: unnamed answers, named reports ──");
    let compiled = compile("@md:flex @md/card:w-64", &config).expect("compiles");
    let env = Env {
        container_rem: Some(64.0),
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("interps");
    for note in &interped.notes {
        if let InterpNote::ContainerUnqueried { cond, .. } = note {
            println!(
                "  note: the {cond:?} layer never matched (the Env carries the nearest container only)"
            );
        }
    }
    println!(
        "  the unnamed layer resolved {} declarations",
        interped.decls.len()
    );
}
