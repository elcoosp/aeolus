//! The §4 code-size knob's microbenchmarks (divan, stable — the
//! workspace bench idiom): the per-expansion cost the `aeolus.toml`
//! profile read adds to every macro entry, beside the dispatch itself.
//!
//! The knob's hot path is COMPILE-time (the expansion walk reads the
//! calling crate's manifest directory once per invocation), so the
//! number this bench exists for is the discovery+parse cost — the
//! missing-file case (the overwhelmingly common project) and the
//! speed-file case — plus the dispatch's branch over the two landed
//! emit faces. The faces' own walk costs stay the `gate`/`outlined`
//! batteries' numbers.
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the
//! bench environment runs them.
use std::path::PathBuf;
use std::sync::OnceLock;

use aeolus_codegen_gpui::{
    Profile, emit_static_program_for, emit_static_program_outlined, profile_from_manifest,
};
use aeolus_compile::{Config, compile, keyword_alignment, keyword_texts};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The manifest-directory fixtures: one with no `aeolus.toml` (the
/// overwhelmingly common project — the default answers) and one with
/// the `speed` key written.
fn fixtures() -> (&'static PathBuf, &'static PathBuf) {
    static MISSING: OnceLock<PathBuf> = OnceLock::new();
    static SPEED: OnceLock<PathBuf> = OnceLock::new();
    (
        MISSING.get_or_init(|| {
            let dir = std::env::temp_dir().join(format!(
                "aeolus-codegen-gpui-bench-profile-missing-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("the missing fixture creates");
            dir
        }),
        SPEED.get_or_init(|| {
            let dir = std::env::temp_dir().join(format!(
                "aeolus-codegen-gpui-bench-profile-speed-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("the speed fixture creates");
            std::fs::write(dir.join("aeolus.toml"), "profile = \"speed\"\n")
                .expect("the speed fixture writes");
            dir
        }),
    )
}

fn compiled_button() -> &'static (aeolus_ir::Program, Vec<&'static str>) {
    static BUTTON: OnceLock<(aeolus_ir::Program, Vec<&'static str>)> = OnceLock::new();
    BUTTON.get_or_init(|| {
        let literal = "inline-flex items-center justify-center gap-2 rounded-md bg-primary p-4";
        let compiled = compile(literal, &Config::default()).expect("the walk is total");
        let annotations = keyword_texts(literal, &Config::default()).expect("the texts read");
        let keywords = keyword_alignment(&compiled, &annotations);
        (compiled.program, keywords)
    })
}

/// The knob's per-expansion read: no `aeolus.toml` in the manifest
/// directory — the default answers without a file read hit.
#[divan::bench]
fn profile_missing_file(bencher: Bencher) {
    let (missing, _) = fixtures();
    bencher.bench(|| profile_from_manifest(missing).expect("the default reads"));
}

/// The knob's per-expansion read: the `speed` key present — one small
/// file read + the one-key parse.
#[divan::bench]
fn profile_speed_file(bencher: Bencher) {
    let (_, speed) = fixtures();
    bencher.bench(|| {
        assert_eq!(
            profile_from_manifest(speed).expect("the speed profile reads"),
            Profile::Speed,
            "the fixture profile"
        );
    });
}

/// The dispatch branch itself, beside the outlined face's direct call
/// (the difference is the one `match` arm — the shipped expansion's
/// own shape).
#[divan::bench]
fn dispatch_size(bencher: Bencher) {
    let (program, keywords) = compiled_button();
    bencher.bench(|| emit_static_program_for(Profile::Size, program, keywords).to_string());
}

#[divan::bench]
fn dispatch_outlined_direct(bencher: Bencher) {
    let (program, keywords) = compiled_button();
    bencher.bench(|| emit_static_program_outlined(program, keywords).to_string());
}
