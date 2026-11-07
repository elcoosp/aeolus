//! The parity path (plan §4.2): `StyleBook::resolve_at` + a probe the app
//! drives. In immediate mode the frame re-resolves everything anyway —
//! these tests pin that group/peer, data/aria and viewport conditions
//! answer through the same backend-agnostic evaluator the other backends
//! use, with egui-specific probes built from a real `Context`.

use aeolus_core::eval::{Probe, Rel};
use aeolus_core::{Resolver, Test, shadcn_default};
use aeolus_egui::{EguiProbe, StyleBook};

fn resolver() -> Resolver<'static> {
    let theme = shadcn_default().unwrap();
    Resolver::new(Box::leak(Box::new(theme)))
}

fn rgba(c: egui::Color32) -> [u8; 4] {
    let [r, g, b, a] = c.to_array();
    [r, g, b, a]
}

/// A probe that answers data attributes the test drives.
#[derive(Default)]
struct DataProbe {
    state_open: bool,
    inner: EguiProbe,
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
fn probe_from_ctx_answers_theme_and_width() {
    let ctx = egui::Context::default();
    ctx.set_theme(egui::Theme::Light);
    let probe = EguiProbe::from_ctx(&ctx);
    assert!(!probe.dark);
    assert!(probe.width.is_some_and(|w| w > 0.0));

    ctx.set_theme(egui::Theme::Dark);
    let probe = EguiProbe::from_ctx(&ctx);
    assert!(probe.dark);
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
    assert_eq!(rgba(closed.frame.fill), [23, 23, 23, 255]);

    let opened = book.resolve_at(&DataProbe {
        state_open: true,
        ..DataProbe::default()
    });
    assert_eq!(rgba(opened.frame.fill), [0, 0, 0, 255]);
}

#[test]
fn viewport_conditions_resolve_from_the_ctx_width() {
    let resolver = resolver();
    let r = resolver.resolve("bg-black sm:bg-red-500").unwrap();
    let (book, _report) = StyleBook::new(&r.plan);

    // A real Context reports its viewport width per frame.
    let ctx = egui::Context::default();
    let probe = EguiProbe::from_ctx(&ctx);
    // The default test viewport is small enough to stay below sm: 640px?
    // Drive BOTH sides deterministically instead.
    let narrow = book.resolve_at(&EguiProbe {
        width: Some(300.0),
        ..EguiProbe::from_ctx(&ctx)
    });
    assert_eq!(rgba(narrow.frame.fill), [0, 0, 0, 255]);

    let wide = book.resolve_at(&EguiProbe {
        width: Some(700.0),
        ..EguiProbe::from_ctx(&ctx)
    });
    assert_eq!(
        rgba(wide.frame.fill),
        aeolus_core::color::Color::oklch(0.637, 0.237, 25.331, 1.0).to_rgba8(),
        "sm:bg-red-500 fires at ≥640px"
    );
    let _ = probe;
}

#[test]
fn dark_media_flips_with_the_ctx_theme() {
    let resolver = resolver();
    let r = resolver.resolve("bg-primary dark:bg-black").unwrap();
    let (book, _report) = StyleBook::new(&r.plan);

    let ctx = egui::Context::default();
    ctx.set_theme(egui::Theme::Light);
    let light = book.resolve_at(&EguiProbe::from_ctx(&ctx));
    assert_eq!(rgba(light.frame.fill), [23, 23, 23, 255]);

    ctx.set_theme(egui::Theme::Dark);
    let dark = book.resolve_at(&EguiProbe::from_ctx(&ctx));
    assert_eq!(rgba(dark.frame.fill), [0, 0, 0, 255]);
}
