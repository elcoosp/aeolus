//! Consumer demonstration: the `rsx.toml` import (architecture plan §5,
//! "Integrating with `gpui-rsx`", config-bridge bullet) — a gpui-rsx
//! theme file walks onto the SAME CSS-first surface every other theme
//! rides, and the unmappable entries answer on the typed note channel.
//!
//! ```sh
//! cargo run -p aeolus-theme --example rsx_import
//! ```
//!
//! The example imports the gpui-rsx demo sheet (verbatim from the
//! upstream repo's `demo/rsx.toml`), prints the emitted `@theme`
//! sheet, the config bridge's spacing answer and the notes, then
//! bakes the sheet into a `TokenTable` and resolves one imported
//! token — the migration's whole story, headless.

use aeolus_theme::{TokenTable, import_rsx};

fn main() {
    // The gpui-rsx demo sheet, verbatim (`gpui-rsx/demo/rsx.toml`).
    let sheet = "\
# Semantic colors resolve through Kit's component theme at render time.
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

    // 1. The import: the sheet, the scale, the notes.
    let import = import_rsx(sheet).expect("the demo sheet imports");
    println!("── the emitted @theme sheet ──────────────────────────────");
    println!("{}", import.css());
    println!("── the config bridge ─────────────────────────────────────");
    println!(
        "spacing-scale: {}{}",
        import.spacing_scale(),
        if import.spacing_scale().is_px() {
            "  (gpui-rsx's 1-px unit scale — the migration guide's scale row applies)"
        } else {
            "  (aeolus's own default scale — identical semantics)"
        }
    );
    println!("\n── the notes (what left, and why) ────────────────────────");
    for note in import.notes() {
        println!("{note}");
    }

    // 2. The round-trip: the imported sheet bakes and resolves.
    let source = import.theme_source().expect("the imported sheet bakes");
    let table = TokenTable::baked(&source).expect("the table builds");
    println!("\n── the round-trip ────────────────────────────────────────");
    for name in ["demo-canvas", "demo-accent"] {
        let id = table.id(name).expect("the imported token rides");
        let rgba = table
            .resolve_rgba(id, 1.0, false)
            .expect("the imported hex resolves");
        println!("{name} → id {id} → rgba {rgba:?}");
    }
}
