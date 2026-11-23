//! The §4 code-size knob's headless read-out: the `aeolus.toml`
//! project profile the macros read at expansion
//! (`profile = "speed" | "size"` — the inline reference emit vs the
//! outlined emit), the typed refusals a mis-set knob produces, and
//! the two faces' expression shapes for one compiled program.
//!
//! Run: `cargo run -p aeolus-codegen-gpui --example profile`
//!
//! The knob is a COMPILE-time surface: the macro reads the calling
//! crate's manifest directory once per expansion (the `theme:` bake's
//! `CARGO_MANIFEST_DIR` anchor). This example drives the same public
//! seam the macro calls — [`profile_from_manifest`] over fixture
//! directories, then [`emit_static_program_for`] — so the file
//! contract's every state is observable headless.
use std::fs;
use std::path::PathBuf;

use aeolus_codegen_gpui::{Profile, emit_static_program_for, profile_from_manifest};
use aeolus_compile::{Config, compile, keyword_alignment, keyword_texts};

/// One sandbox per contract state; each gets its own directory so the
/// reads are independent.
struct Fixture {
    dir: PathBuf,
}

impl Fixture {
    fn new(name: &str, contents: Option<&str>) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "aeolus-codegen-gpui-example-profile-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("the fixture directory creates");
        if let Some(contents) = contents {
            fs::write(dir.join("aeolus.toml"), contents).expect("the fixture writes");
        }
        Self { dir }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn main() {
    println!("the §4 knob: profile = \"speed\" | \"size\" in the project's aeolus.toml");
    println!("(the macro reads <CARGO_MANIFEST_DIR>/aeolus.toml once per expansion)\n");

    let missing = Fixture::new("missing", None);
    let speed = Fixture::new("speed", Some("profile = \"speed\"\n"));
    let size = Fixture::new("size", Some("profile = \"size\"\n"));
    let invalid = Fixture::new("invalid", Some("profile = \"fast\"\n"));

    for (label, dir) in [
        ("no aeolus.toml (the common project)", missing.dir.clone()),
        ("profile = \"speed\"", speed.dir.clone()),
        ("profile = \"size\"", size.dir.clone()),
    ] {
        let profile = profile_from_manifest(&dir).expect("the fixture reads");
        println!("{label:>38} -> {profile:?}");
    }

    println!("\nthe typed refusal a mis-set knob produces:");
    match profile_from_manifest(&invalid.dir) {
        Ok(profile) => println!("unexpectedly read: {profile:?}"),
        Err(err) => println!("  {err}"),
    }

    println!("\nthe dispatch over one compiled program (\"p-4 hover:bg-primary/90\"):");
    let compiled = compile("p-4 hover:bg-primary/90", &Config::default()).expect("compiles");
    let annotations =
        keyword_texts("p-4 hover:bg-primary/90", &Config::default()).expect("the texts read");
    let keywords = keyword_alignment(&compiled, &annotations);
    for profile in [Profile::Speed, Profile::Size] {
        let tokens = emit_static_program_for(profile, &compiled.program, &keywords).to_string();
        let shape = if tokens.contains("StaticProgramBuilder") {
            "one assemble call over static slices (the outlined face)"
        } else {
            "direct StaticProgram::new over inline consts (the reference face)"
        };
        println!(
            "  {profile:?} -> {} bytes, {shape}",
            tokens.split_whitespace().count()
        );
    }
    println!(
        "\nthe faces' runtime answers are byte-identical (tests/outlined_face.rs pins \
         the frozen content) — the knob selects the EXPANSION shape; the default is \
         the shipped outlined face."
    );
}
