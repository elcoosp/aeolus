//! The parity path (plan §4.2): `StyleBook::resolve_at` + a probe the app
//! drives. The probe answers group/peer/data/aria/container questions the
//! Catalog closures cannot, and the core evaluator flattens the whole plan
//! — compound variants included — before the Iced lowering runs.

use aeolus_core::eval::{Probe, Rel};
use aeolus_core::{Resolver, Test, shadcn_default};
use aeolus_iced::{IcedProbe, StyleBook};

fn resolver() -> Resolver<'static> {
    let theme = shadcn_default().unwrap();
    Resolver::new(Box::leak(Box::new(theme)))
}

fn solid(bg: &iced_core::Background) -> iced_core::Color {
    match *bg {
        iced_core::Background::Color(c) => c,
        iced_core::Background::Gradient(g) => {
            panic!("expected a solid color, got gradient {g:?}")
        }
    }
}

fn rgba(c: iced_core::Color) -> [u8; 4] {
    [
        (c.r * 255.0).round() as u8,
        (c.g * 255.0).round() as u8,
        (c.b * 255.0).round() as u8,
        (c.a * 255.0).round() as u8,
    ]
}

/// A probe that answers data attributes the test drives.
#[derive(Default)]
struct DataProbe {
    state_open: bool,
    inner: IcedProbe,
}

impl Probe for DataProbe {
    fn test(&self, t: &Test) -> bool {
        match t {
            Test::Data(name, value) => {
                name.as_ref() == "state" && value.as_deref() == Some("open") && self.state_open
            }
            _ => self.inner.test(t),
        }
    }

    fn related(&self, rel: Rel, label: Option<&str>, t: &Test) -> bool {
        self.inner.related(rel, label, t)
    }

    fn media(&self, m: aeolus_core::Media) -> bool {
        self.inner.media(m)
    }

    fn viewport_width(&self) -> Option<f32> {
        self.inner.viewport_width()
    }

    fn container_width(&self, name: Option<&str>) -> Option<f32> {
        self.inner.container_width(name)
    }
}

#[test]
fn data_attribute_rules_fire_through_the_probe() {
    let resolver = resolver();
    let r = resolver
        .resolve("bg-primary data-[state=open]:bg-black")
        .unwrap();
    let (book, report) = StyleBook::new(&r.plan);
    // The rule is REPORTED at lower time (not native)...
    assert!(report.dropped.iter().any(|(w, _)| w.starts_with("rules")));
    // ...and still resolves through the parity path.
    let closed = book.resolve_at(&DataProbe::default());
    assert_eq!(
        rgba(solid(closed.container.background.as_ref().unwrap())),
        [23, 23, 23, 255],
        "data-[state=open] inactive: primary"
    );
    let opened = book.resolve_at(&DataProbe {
        state_open: true,
        ..DataProbe::default()
    });
    assert_eq!(
        rgba(solid(opened.container.background.as_ref().unwrap())),
        [0, 0, 0, 255],
        "data-[state=open] firing: black"
    );
}

#[test]
fn hover_via_probe_matches_the_native_button_path() {
    let resolver = resolver();
    let r = resolver.resolve("bg-primary hover:bg-red-500").unwrap();
    let (book, report) = StyleBook::new(&r.plan);
    assert!(report.is_empty(), "hover lowers natively: {report:?}");

    let probed = book.resolve_at(&IcedProbe {
        hovered: true,
        ..IcedProbe::default()
    });
    let native = book.button_style(
        iced_widget::button::Status::Hovered,
        &iced_core::Theme::Light,
    );
    assert_eq!(
        rgba(solid(probed.container.background.as_ref().unwrap())),
        rgba(solid(native.background.as_ref().unwrap())),
        "the probe path and the Status path must agree"
    );
}

#[test]
fn responsive_resolves_when_the_probe_knows_the_width() {
    let resolver = resolver();
    let r = resolver.resolve("bg-black sm:bg-red-500").unwrap();
    let (book, _report) = StyleBook::new(&r.plan);

    let narrow = book.resolve_at(&IcedProbe {
        width: Some(300.0),
        ..IcedProbe::default()
    });
    assert_eq!(
        rgba(solid(narrow.container.background.as_ref().unwrap())),
        [0, 0, 0, 255]
    );

    let wide = book.resolve_at(&IcedProbe {
        width: Some(700.0), // ≥ sm 640px
        ..IcedProbe::default()
    });
    assert_eq!(
        rgba(solid(wide.container.background.as_ref().unwrap())),
        aeolus_core::color::Color::oklch(0.637, 0.237, 25.331, 1.0).to_rgba8(),
        "sm:bg-red-500 fires at ≥640px"
    );
}

#[test]
fn dark_media_flips_with_the_probe() {
    let resolver = resolver();
    let r = resolver.resolve("bg-primary dark:bg-black").unwrap();
    let (book, _report) = StyleBook::new(&r.plan);

    let light = book.resolve_at(&IcedProbe::default());
    assert_eq!(
        rgba(solid(light.container.background.as_ref().unwrap())),
        [23, 23, 23, 255]
    );

    let dark = book.resolve_at(&IcedProbe {
        dark: true,
        ..IcedProbe::default()
    });
    assert_eq!(
        rgba(solid(dark.container.background.as_ref().unwrap())),
        [0, 0, 0, 255]
    );
}
