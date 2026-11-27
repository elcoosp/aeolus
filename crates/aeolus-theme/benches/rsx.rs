//! The `rsx.toml` import's microbenchmarks (divan, stable — the
//! workspace bench idiom). The import is a once-per-theme cold path
//! (a host imports its sheet at startup or bake time), so the arms
//! measure the parse+map walk over the demo sheet and a synthetic
//! wide sheet — the shape the migration story hands over.
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (the theme-pass method note applies).

use aeolus_theme::import_rsx;
use divan::Bencher;

fn main() {
    divan::main();
}

/// The gpui-rsx demo sheet (`gpui-rsx/demo/rsx.toml`): every section
/// but `[radius]`/`[spacing]`, seven colours, the note channel riding.
const DEMO: &str = "# Semantic colors resolve through Kit's component theme at render time.
[options]
dynamic-runtime = \"rsx_runtime\"
preset = \"gpui-component\"

[colors]
demo-canvas = \"#111a16\"
demo-surface = \"#19241f\"
demo-raised = \"#24352b\"
demo-border = \"#2b3b33\"
demo-text = \"#edf3ef\"
demo-muted = \"#bac8bf\"
demo-accent = \"#8bddc5\"

[text-size]
body = \"14px\"

[line-height]
body = \"20px\"

[opacity]
muted = \"60%\"

[shadow]
card = { expr = \"vec![BoxShadow::new(px(0.0), px(1.0), hsla(0.0, 0.0, 0.0, 0.2))]\" }
";

/// The import over the demo sheet: the parse, the namespace map and
/// the note channel in one walk.
#[divan::bench]
fn import_demo_sheet(bencher: Bencher) {
    bencher.bench(|| import_rsx(DEMO).expect("the demo sheet imports"));
}

/// A synthetic wide sheet (256 colours, every mapped section) — the
/// parse+map walk's shape at the size a real design system hands over.
#[divan::bench]
fn import_wide_sheet(bencher: Bencher) {
    let mut sheet = String::from("[options]\nspacing-scale = \"tailwind\"\n\n[colors]\n");
    for index in 0..256 {
        let _ = std::fmt::write(
            &mut sheet,
            format_args!("token-{index:03} = \"#{index:02x}aa{index:02x}\"\n"),
        );
    }
    sheet.push_str("\n[radius]\n");
    for index in 0..8 {
        let _ = std::fmt::write(&mut sheet, format_args!("r-{index} = \"{index}px\"\n"));
    }
    sheet.push_str("\n[text-size]\n");
    for index in 0..8 {
        let _ = std::fmt::write(&mut sheet, format_args!("t-{index} = \"{index}px\"\n"));
    }
    sheet.push_str("\n[line-height]\n");
    for index in 0..8 {
        let _ = std::fmt::write(
            &mut sheet,
            format_args!("t-{index} = \"{}px\"\n", index * 4),
        );
    }
    bencher.bench(move || import_rsx(&sheet).expect("the wide sheet imports"));
}
