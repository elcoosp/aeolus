//! Micro-debt: the `theme_from_css` runtime helper gains its first
//! pins. The crate-level helper is the `theme!` macro's runtime half
//! (T2, the explicit slow lane the plan allows for user-editable
//! themes) — a host passes the SAME `@theme`/`:root`/`.dark` CSS a web
//! Tailwind v4 project uses and receives the
//! [`ThemeSource`](aeolus_rt::ThemeSource) the dynamic lane's compiles
//! consume. The helper had no consumer and no coverage anywhere in the
//! workspace (the example read it, no test pinned it): the parse
//! contract and the typed error half are pinned here, through the
//! theme pass's own audited reader.

use aeolus_compile::Config;
use aeolus_rt::{ClassCell, theme_from_css};

/// A tiny synthetic source: one light-only token (`brand`), one token
/// with a dark value (`surface`) — the theme tests' own controlled
/// input, through the helper instead of `ThemeSource::from_css`.
fn mini_source() -> aeolus_rt::ThemeSource {
    theme_from_css(
        ":root { --color-primary: #336699; --color-surface: #eeeeee; }",
        Some(".dark { --color-surface: #111111; }"),
    )
    .expect("mini source parses")
}

#[test]
fn the_runtime_helper_parses_the_web_css_shapes() {
    let source = mini_source();
    // The tiers ride the parsed source whole: the light palette answers
    // the token, the dark block is the source's dark variant.
    assert!(source.dark().is_some(), "the .dark block rides");
    assert!(
        source.light().color("primary").is_some(),
        "the light token answers"
    );

    // The dynamic lane consumes what the helper parsed: the cell
    // compiles a themed class through the runtime table built from the
    // source (the slow lane's own shape — theme tokens in, the typed
    // boundary out when a token is missing).
    let table = aeolus_rt::TokenTable::baked(&source).expect("the baked table builds");
    let cell = ClassCell::new();
    let program = cell
        .get_theme("bg-primary", &table, &Config::default())
        .expect("the parsed source's token compiles");
    assert!(
        program
            .layers
            .iter()
            .any(|layer| layer.mask.contains(aeolus_ir::Prop::Background)),
        "bg-primary lowered through the helper's source"
    );
}

#[test]
fn the_error_half_stays_typed() {
    // A `var()` that never resolves is a parse failure — the helper
    // forwards the theme pass's typed error, never a panic.
    let err = theme_from_css(":root { --color-broken: calc(var(--never) + 1px); }", None)
        .expect_err("the unresolved var is a parse failure");
    assert!(
        matches!(err, aeolus_rt::ThemeError::Parse { .. }),
        "the typed error: {err:?}"
    );
    assert!(err.to_string().contains("theme CSS"), "{err}");
}
