//! Consumer demonstration: the floem walk's full probe surface (the
//! widest `lower_program_*` growth). A `tw!`-compiled [`StaticProgram`]
//! carries `@container`/`first:`/`has-*`/`supports-*` layers the
//! sourceless walk reports as unqueried — a host whose layout KNOWS the
//! answers threads its probe sources through
//! [`lower_program_probes_containers_struct_descendants_supports`] and
//! those layers FIRE in the lowered Floem [`Style`], the answered miss
//! the CSS-quiet skip. Headless by construction: the walk is pure.
//!
//! ```sh
//! cargo run -p aeolus-floem --example program_probes
//! ```

use floem::style::Background;

use aeolus_compile::{Config, compile_theme, keyword_alignment, keyword_texts};
use aeolus_floem::{lower_program, lower_program_probes_containers_struct_descendants_supports};
use aeolus_rt::{
    ContainerBound, ContainerState, DescendantFact, DescendantKind, DescendantState, Env,
    StaticProgram, StructFacts, SupportsState,
};
use aeolus_theme::{ThemeSource, TokenTable};

/// Compile a class string the way `tw!` does (the theme pass's baked
/// shadcn table) and freeze it into the static form the macro emits:
/// the interned layers plus the row texts the keyword slots index.
fn frozen(classes: &str) -> StaticProgram {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::baked(&source).expect("the baked shadcn table builds");
    let config = Config::default();
    let compiled = compile_theme(classes, &table, &config).expect("the class string lowers");
    assert!(
        compiled.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        compiled.diagnostics
    );
    let annotations = keyword_texts(classes, &config).expect("non-empty input annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    StaticProgram::new(
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    )
}

/// The solid fill the walked style carries, when one is set — the same
/// read the differential pins apply.
fn fill_of(style: &floem::style::Style) -> Option<String> {
    style.get(Background).map(|b| match b {
        floem::peniko::Brush::Solid(c) => {
            let rgba = c.components.map(|v| (v * 255.0).round() as u8);
            format!("#{:02x}{:02x}{:02x}", rgba[0], rgba[1], rgba[2])
        }
        _ => "(non-solid brush)".to_string(),
    })
}

fn main() {
    // One call site: the element's own classes beside one layer per
    // probe seam — exactly the shape a widget with a measured container,
    // a tracked subtree and a capability query carries.
    let classes = "p-4 bg-primary @md/card:bg-red-500 first:bg-accent \
                   has-focus:shadow-lg supports-[display:grid]:grid";
    let program = frozen(classes);

    println!("── the floem walk's full probe surface ──");
    println!("  layers walked: {}", program.layers.len());

    // Sourceless: every probe layer reports its typed boundary — the
    // honest shape the narrow entry keeps.
    let (resting, report) = lower_program(&program, &Env::default());
    println!("── sourceless (the narrow entry's shape) ──");
    println!("  background (bg-primary):   {:?}", fill_of(&resting));
    println!(
        "  walk report:               {} drop(s) — the unqueried container/structural/\
         descendant/supports layers named, never silent",
        report.dropped.len()
    );
    for (what, reason) in &report.dropped {
        println!("    · {what}: {reason}");
    }

    // The host's probe sources: the measured container answers above
    // the md boundary, the DOM-position facts answer `first:`, the
    // subtree fact answers `has-focus:`, the capability set answers the
    // `@supports` block.
    let containers = ContainerState::new(&[ContainerBound {
        name: "card",
        rem: 64.0,
    }]);
    let structs = StructFacts::new(0, 3);
    let descendants = DescendantState::new(&[DescendantFact {
        kind: DescendantKind::Element,
        fragment: "*:focus",
    }]);
    let supports = SupportsState::new(&["(display:grid)"]);

    let (wired, report) = lower_program_probes_containers_struct_descendants_supports(
        &program,
        &Env::default(),
        None,
        Some(&containers),
        Some(&structs),
        Some(&descendants),
        Some(&supports),
        None,
    );
    println!("── wired (the widest entry) ──");
    println!("  background (fired red-500): {:?}", fill_of(&wired));
    println!(
        "  walk report:               {} drop(s) — the answered layers fired, \
         whatever a backend slot cannot carry still names itself",
        report.dropped.len()
    );
    for (what, reason) in &report.dropped {
        println!("    · {what}: {reason}");
    }

    // The answered miss: the card BELOW the boundary — CSS-quiet, the
    // resting fill, no drop. A separate program carries ONLY the
    // container layer, so the miss is the whole delta.
    let narrow = ContainerState::new(&[ContainerBound {
        name: "card",
        rem: 32.0,
    }]);
    let only_container = frozen("bg-primary @md/card:bg-red-500");
    let (resting_only, _) = lower_program(&only_container, &Env::default());
    let (quiet, report) = lower_program_probes_containers_struct_descendants_supports(
        &only_container,
        &Env::default(),
        None,
        Some(&narrow),
        None,
        None,
        None,
        None,
    );
    println!("── the answered miss (the card below the boundary) ──");
    println!(
        "  background == resting:     {}",
        fill_of(&quiet) == fill_of(&resting_only)
    );
    println!(
        "  walk report:               {} drop(s) — an answered miss is quiet, never a drop",
        report.dropped.len()
    );
}
