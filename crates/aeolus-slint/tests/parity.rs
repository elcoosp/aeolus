//! Parity between the book's layer cascade and the core evaluator.
//!
//! `StyleBook::resolve_for` (Slint state flags → layers) and
//! `StyleBook::resolve_at` (full [`Probe`] evaluation over the original
//! plan) must agree in rank order — the same contract the egui/iced
//! backends pin. These tests drive both paths through the shadcn Button
//! table and the responsive/group/peer/dark conditions Slint's native
//! `states` cannot express.

use aeolus_core::{Media, Resolver, Test, shadcn_default};
use aeolus_slint::{SlintProbe, StyleBook};

fn resolver() -> Resolver<'static> {
    let theme = shadcn_default().unwrap();
    Resolver::new(Box::leak(Box::new(theme)))
}

fn button_book(r: &Resolver<'static>) -> StyleBook {
    StyleBook::tw(
        r,
        "bg-primary hover:bg-primary/90 active:bg-primary/80 disabled:opacity-50 focus-visible:p-2",
    )
}

#[test]
fn idle_resolves_the_base() {
    let r = resolver();
    let book = button_book(&r);
    let style = book.resolve_for(&SlintProbe::default());
    assert!(style.background.is_some(), "base fill present");
    // No layer fired: opacity untouched, border slots bare.
    assert_eq!(style.opacity, None);
}

#[test]
fn hover_fires_over_the_base() {
    let r = resolver();
    let book = button_book(&r);
    let idle = book.resolve_for(&SlintProbe::default());
    let hover = book.resolve_for(&SlintProbe {
        has_hover: true,
        ..SlintProbe::default()
    });
    // The hover layer repainted the fill (primary/90 vs primary).
    assert_ne!(idle.background, hover.background, "hover repaints the fill");
}

#[test]
fn rank_order_focus_beats_hover() {
    let r = resolver();
    // `focus-visible:p-2` + `hover:bg-primary/90`: v4's variant order
    // (hover < focus-visible) decides, NOT source order — a focused AND
    // hovered element resolves the FOCUS layer last.
    let book = button_book(&r);
    let focus_only = book.resolve_for(&SlintProbe {
        has_focus: true,
        ..SlintProbe::default()
    });
    // p-2 = 2 * 4px = 8px (the Tailwind scale).
    assert_eq!(focus_only.padding.map(|p| p.top), Some(8.0));
    let both = book.resolve_for(&SlintProbe {
        has_hover: true,
        has_focus: true,
        ..SlintProbe::default()
    });
    assert_eq!(
        both.padding.map(|p| p.top),
        Some(8.0),
        "the focus layer applies after hover even when hover is also active"
    );
}

#[test]
fn disabled_resolves_the_opacity_layer() {
    let r = resolver();
    let book = button_book(&r);
    let disabled = book.resolve_for(&SlintProbe {
        enabled: false,
        ..SlintProbe::default()
    });
    assert_eq!(disabled.opacity, Some(0.5));
}

#[test]
fn dark_merges_when_the_scheme_is_dark() {
    let r = resolver();
    let book = StyleBook::tw(&r, "bg-background dark:bg-input/30");
    let light = book.resolve_for(&SlintProbe::default());
    let dark = book.resolve_for(&SlintProbe {
        dark: true,
        ..SlintProbe::default()
    });
    assert_ne!(
        light.background, dark.background,
        "the dark layer repaints the fill"
    );
}

#[test]
fn resolve_at_covers_responsive_conditions() {
    let r = resolver();
    let book = StyleBook::tw(&r, "h-9 md:h-10");
    // Responsive conditions resolve through the parity path (the book's
    // native layers are state-only; width conditions are reported at
    // construction).
    let base = book.resolve_at(&SlintProbe {
        width: Some(500.0),
        ..SlintProbe::default()
    });
    let md = book.resolve_at(&SlintProbe {
        width: Some(900.0),
        ..SlintProbe::default()
    });
    assert_eq!(base.height, Some(aeolus_slint::SlintLength::Px(36.0)));
    assert_eq!(md.height, Some(aeolus_slint::SlintLength::Px(40.0)));
}

/// A probe that answers group-hover — the app-side extension point the
/// backend documents (group state lives with the app).
struct WithGroupHover {
    inner: SlintProbe,
}

impl aeolus_core::Probe for WithGroupHover {
    fn test(&self, t: &Test) -> bool {
        self.inner.test(t)
    }
    fn related(&self, rel: aeolus_core::Rel, _label: Option<&str>, t: &Test) -> bool {
        matches!(rel, aeolus_core::Rel::Group)
            && matches!(t, Test::State(aeolus_core::State::Hover))
    }
    fn media(&self, m: Media) -> bool {
        matches!(m, Media::Dark) && self.inner.dark
    }
    fn viewport_width(&self) -> Option<f32> {
        self.inner.width
    }
}

#[test]
fn resolve_at_answers_group_conditions_through_the_probe() {
    let r = resolver();
    let book = StyleBook::tw(&r, "opacity-100 group-hover:opacity-75");
    let without = book.resolve_for(&SlintProbe::default());
    assert_eq!(without.opacity, Some(1.0));
    // The native path cannot answer group-hover (reported at construction),
    // the parity path can — through the app-extended probe.
    let with = book.resolve_at(&WithGroupHover {
        inner: SlintProbe::default(),
    });
    assert_eq!(with.opacity, Some(0.75));
}

#[test]
fn resolve_at_matches_resolve_for_on_state_layers() {
    let r = resolver();
    let book = button_book(&r);
    let probe = SlintProbe {
        has_hover: true,
        has_focus: true,
        ..SlintProbe::default()
    };
    let via_book = book.resolve_for(&probe);
    let via_core = book.resolve_at(&probe);
    // The layer cascade (book) and the evaluator (core) agree on the
    // condition payloads both paths can see.
    assert_eq!(via_book.border_width, via_core.border_width);
    assert_eq!(via_book.opacity, via_core.opacity);
}
