//! The dynamic lane's GPUI half (architecture plan §3.6's surface (3)):
//! [`TwExt::tw_dyn`] and its `_env`/`_theme` pairings ride the
//! [`aeolus_rt::ClassCell`] view-state cell — "compiled when the string
//! changes, a hash compare per frame otherwise" — and apply through the
//! SAME walk the static programs take.
//!
//! The pins:
//! * THE DIFFERENTIAL: the cell's themed program is byte-equal to the
//!   macro's frozen program for the same string (the plan's own
//!   capability-split rule — full v4 coverage here is the same compiler
//!   as static, no smaller matcher);
//! * `tw_dyn` applies the compiled program to a `StyleRefinement`;
//! * a failed compile degrades to the last good program — the honest
//!   per-frame contract (never panic in a render loop, never a silent
//!   no-op: the cell's diagnostics carry the reason);
//! * `tw_dyn_theme` resolves colour tokens through the runtime table;
//! * `tw_dyn_env` lifts the environment layers (`dark:`).

use aeolus_compile::Config;
use aeolus_gpui::prelude::*;
use aeolus_rt::{ThemeSource, TokenTable};
use gpui::rems;

fn baked_shadcn() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the T0/T1 bake succeeds")
}

/// THE DIFFERENTIAL PIN: the cell's compiled program equals the macro's
/// frozen program — layers AND keyword row texts — for the same string
/// under the same baked table.
#[test]
fn cell_program_equals_the_macro_program() {
    let table = baked_shadcn();
    let cell = ClassCell::new();
    let cell_program = cell
        .get_theme(
            "p-4 bg-primary text-sm hover:bg-primary",
            &table,
            &Config::default(),
        )
        .expect("the demo string lowers clean on the baked source");
    let macro_program: StaticProgram = tw!("p-4 bg-primary text-sm hover:bg-primary");
    assert_eq!(cell_program, macro_program);
}

/// `tw_dyn` compiles the string through the cell and applies it — the
/// slow lane's one-call entrance.
#[test]
fn tw_dyn_applies_the_compiled_program() {
    let cell = ClassCell::new();
    let style = gpui::StyleRefinement::default().tw_dyn(&cell, "p-4");
    assert_eq!(style.padding.top, Some(rems(1.0).into()));
    assert_eq!(style.padding.left, Some(rems(1.0).into()));
}

/// A themed string through `tw_dyn_theme`: the runtime table resolves
/// the token to the baked literal the style refinement consumes.
#[test]
fn tw_dyn_theme_resolves_tokens() {
    let table = baked_shadcn();
    let cell = ClassCell::new();
    let style = gpui::StyleRefinement::default().tw_dyn_theme(&cell, "bg-primary", &table);
    assert!(style.background.is_some());
    assert!(cell.diagnostics().is_empty());
}

/// The themeless lane keeps the token boundary honest: the string
/// lowers what it can and the miss rides the cell's diagnostics —
/// never a silent drop.
#[test]
fn tw_dyn_reports_the_token_boundary() {
    let cell = ClassCell::new();
    let style = gpui::StyleRefinement::default().tw_dyn(&cell, "bg-primary");
    assert!(style.background.is_none(), "no theme table, no colour");
    assert_eq!(cell.diagnostics().len(), 1, "the typed boundary reports");
}

/// The honest per-frame degradation: a failed compile (empty input)
/// keeps the LAST GOOD program applied — never a panic, never a silent
/// reset.
#[test]
fn tw_dyn_degrades_to_the_last_good_program() {
    let cell = ClassCell::new();
    let good = gpui::StyleRefinement::default().tw_dyn(&cell, "p-4");
    let degraded = gpui::StyleRefinement::default().tw_dyn(&cell, "   ");
    assert_eq!(degraded.padding.top, good.padding.top);
    assert_eq!(degraded.padding.left, good.padding.left);
}

/// `tw_dyn_env` lifts the environment layers the walk matches — the
/// `dark:` tier fires under a dark probe.
#[test]
fn tw_dyn_env_lifts_the_dark_layer() {
    let cell = ClassCell::new();
    let env = Env {
        dark: true,
        ..Env::default()
    };
    let style = gpui::StyleRefinement::default().tw_dyn_env(&cell, "dark:p-8", env);
    assert_eq!(style.padding.top, Some(rems(2.0).into()));
    let resting = gpui::StyleRefinement::default().tw_dyn(&cell, "dark:p-8");
    assert_eq!(resting.padding.top, None);
}
