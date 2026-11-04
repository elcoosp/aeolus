//! Fuzz `TailwindTheme::from_theme_css`: arbitrary CSS must never panic the
//! theme parser; a successful parse must expose a consistent token map.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(theme) = aeolus_core::TailwindTheme::from_theme_css(s) {
            // A parsed theme must answer lookups without panicking.
            let _ = theme.spacing();
            let _ = theme.color("primary");
            let _ = theme.radius("md");
            let _ = theme.font_size("sm");
            let _ = theme.shadow("sm");
            // And a resolver must handle any class string against it.
            let resolver = aeolus_core::Resolver::new(&theme);
            let _ = resolver.resolve("p-4 bg-primary hover:bg-primary/90 rounded-md");
        }
    }
});
