//! The §4 code-size knob: the project-level `aeolus.toml` profile the
//! macro reads at expansion. The plan line this battery pins —
//! docs/AEOLUS_V1_ARCHITECTURE.md §4:
//!
//! > Code-size knob in `aeolus.toml`: `profile = "speed" | "size"`
//! > (inline vs one outlined `fn` per distinct program).
//!
//! The knob maps the plan's two shapes onto the crate's two LANDED
//! emit faces: `speed` = the inline reference emit
//! ([`emit_static_program`], the call site's direct
//! `StaticProgram::new` construction), `size` = the outlined emit
//! ([`emit_static_program_outlined`], one `assemble` call over
//! `static` slices — the ICF-friendly shape §4's binary-growth row
//! names, and the face the macro has shipped since the outlined-face
//! perf switch). The DEFAULT (no `aeolus.toml`, no `profile` key)
//! preserves that shipped behavior exactly.
//!
//! RED-first: the profile seam (`Profile`, `profile_from_manifest`,
//! `emit_static_program_for`) is this battery's capture.
use std::fs;
use std::path::{Path, PathBuf};

use aeolus_codegen_gpui::{
    Profile, ProfileError, emit_static_program, emit_static_program_for,
    emit_static_program_outlined, profile_from_manifest,
};
use aeolus_compile::{Config, compile, keyword_alignment, keyword_texts};

/// A unique fixture directory per test (the tests run in parallel
/// threads; every battery owns its `aeolus.toml` sandbox).
struct Fixture {
    dir: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "aeolus-codegen-gpui-profile-{}-{name}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("the fixture directory creates");
        Self { dir }
    }

    fn path(&self) -> &Path {
        &self.dir
    }

    /// Write the crate-root `aeolus.toml` the knob reads.
    fn write(&self, contents: &str) {
        fs::write(self.dir.join("aeolus.toml"), contents).expect("the fixture aeolus.toml writes");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

/// One compiled program + its keyword annotations — the emit input
/// every dispatch pin rides.
fn compiled(literal: &str) -> (aeolus_ir::Program, Vec<&'static str>) {
    let compiled = compile(literal, &Config::default()).expect("the walk is total");
    let annotations = keyword_texts(literal, &Config::default()).expect("the texts read");
    let keywords = keyword_alignment(&compiled, &annotations);
    (compiled.program, keywords)
}

// ---------------------------------------------------------------------
// The file contract
// ---------------------------------------------------------------------

/// No `aeolus.toml` in the manifest directory — the overwhelmingly
/// common case — answers the DEFAULT profile, which is the face the
/// macro already ships (`Size`, the outlined emit): introducing the
/// knob must not silently change any existing consumer's expansion.
#[test]
fn missing_file_answers_the_default_size_profile() {
    let fixture = Fixture::new("missing");
    let profile = profile_from_manifest(fixture.path()).expect("the default reads");
    assert_eq!(profile, Profile::Size, "the shipped face is the default");
    assert_eq!(profile, Profile::default(), "Default is Size");
}

/// `profile = "speed"` selects the inline reference emit.
#[test]
fn speed_profile_reads_from_the_project_file() {
    let fixture = Fixture::new("speed");
    fixture.write("profile = \"speed\"\n");
    let profile = profile_from_manifest(fixture.path()).expect("the speed profile reads");
    assert_eq!(
        profile,
        Profile::Speed,
        "the speed key maps the reference face"
    );
}

/// `profile = "size"` selects the outlined emit explicitly.
#[test]
fn size_profile_reads_from_the_project_file() {
    let fixture = Fixture::new("size");
    fixture.write("profile = \"size\"\n");
    let profile = profile_from_manifest(fixture.path()).expect("the size profile reads");
    assert_eq!(
        profile,
        Profile::Size,
        "the size key maps the outlined face"
    );
}

/// A `profile` value the knob does not name is a TYPED refusal naming
/// the path and the value — never a silent guess (the honest-failure
/// contract every seam keeps).
#[test]
fn unknown_profile_value_is_a_typed_refusal() {
    let fixture = Fixture::new("invalid");
    fixture.write("profile = \"fast\"\n");
    let err = profile_from_manifest(fixture.path()).expect_err("the unknown value refuses");
    match &err {
        ProfileError::InvalidValue { path, value } => {
            assert_eq!(value, "fast", "the refused value carries");
            assert!(
                path.ends_with("aeolus.toml"),
                "the refusal names the file: {path:?}"
            );
        }
        ProfileError::Unreadable { .. } => {
            panic!("the invalid value is the InvalidValue kind: {err:?}")
        }
    }
    let text = err.to_string();
    assert!(
        text.contains("speed") && text.contains("size"),
        "the refusal names the accepted spellings: {text}"
    );
    assert!(
        text.contains("fast"),
        "the refusal echoes the value: {text}"
    );
}

/// Comments, section headers and unknown keys are ignored — the knob
/// parses the ONE key it owns, never the whole file.
#[test]
fn comments_sections_and_unknown_keys_are_ignored() {
    let fixture = Fixture::new("tolerant");
    fixture.write(
        "# the project's aeolus.toml\n\
         [tool.other]\n\
         spacing = \"px\"\n\
         \n\
         profile = \"speed\"\n",
    );
    let profile = profile_from_manifest(fixture.path()).expect("the tolerant file reads");
    assert_eq!(
        profile,
        Profile::Speed,
        "the profile key wins past the noise"
    );
}

/// The FIRST `profile` line answers (documented order, not a whim).
#[test]
fn the_first_profile_line_wins() {
    let fixture = Fixture::new("first");
    fixture.write("profile = \"speed\"\nprofile = \"size\"\n");
    let profile = profile_from_manifest(fixture.path()).expect("the first line reads");
    assert_eq!(profile, Profile::Speed, "the first profile line wins");
}

/// A `profile` key whose value is missing is the same typed refusal —
/// the operator named the knob; the parser never guesses.
#[test]
fn valueless_profile_key_is_a_typed_refusal() {
    let fixture = Fixture::new("valueless");
    fixture.write("profile =\n");
    let err = profile_from_manifest(fixture.path()).expect_err("the empty value refuses");
    assert!(
        matches!(err, ProfileError::InvalidValue { .. }),
        "the empty value is the InvalidValue kind: {err:?}"
    );
}

/// A present-but-unreadable `aeolus.toml` is a typed refusal, not a
/// silent default (the fixture uses a DIRECTORY at the file's path —
/// deterministic on every platform and uid).
#[test]
fn unreadable_file_is_a_typed_refusal() {
    let fixture = Fixture::new("unreadable");
    fs::create_dir(fixture.dir.join("aeolus.toml")).expect("the directory fixture creates");
    let err = profile_from_manifest(fixture.path()).expect_err("the directory refuses");
    match &err {
        ProfileError::Unreadable { path, .. } => {
            assert!(
                path.ends_with("aeolus.toml"),
                "the refusal names the file: {path:?}"
            );
        }
        ProfileError::InvalidValue { .. } => {
            panic!("the unreadable file is the Unreadable kind: {err:?}")
        }
    }
}

// ---------------------------------------------------------------------
// The dispatch contract
// ---------------------------------------------------------------------

/// The `size` dispatch is the outlined face BYTE FOR BYTE — the
/// knob's default must be indistinguishable from the shipped expansion.
#[test]
fn dispatch_size_matches_the_outlined_face_byte_for_byte() {
    let (program, keywords) = compiled("p-4 hover:bg-primary/90");
    let dispatched = emit_static_program_for(Profile::Size, &program, &keywords).to_string();
    let outlined = emit_static_program_outlined(&program, &keywords).to_string();
    assert_eq!(
        dispatched, outlined,
        "the size dispatch IS the outlined face"
    );
    assert!(
        dispatched.contains("StaticProgramBuilder :: assemble"),
        "the outlined shape rides the one assemble call: {dispatched}"
    );
}

/// The `speed` dispatch is the reference face BYTE FOR BYTE — the
/// plan's inline shape (the call site's direct construction).
#[test]
fn dispatch_speed_matches_the_reference_face_byte_for_byte() {
    let (program, keywords) = compiled("p-4 hover:bg-primary/90");
    let dispatched = emit_static_program_for(Profile::Speed, &program, &keywords).to_string();
    let reference = emit_static_program(&program, &keywords).to_string();
    assert_eq!(
        dispatched, reference,
        "the speed dispatch IS the reference face"
    );
    assert!(
        dispatched.contains("StaticProgram :: new"),
        "the inline shape constructs directly: {dispatched}"
    );
}

/// The two shapes DIFFER (the knob actually switches the expansion)
/// while the faces' frozen CONTENT equality stays the existing
/// battery's pin (`tests/outlined_face.rs`).
#[test]
fn the_two_faces_render_different_shapes() {
    let (program, keywords) = compiled("flex flex-col");
    let speed = emit_static_program_for(Profile::Speed, &program, &keywords).to_string();
    let size = emit_static_program_for(Profile::Size, &program, &keywords).to_string();
    assert_ne!(speed, size, "the knob switches the expression shape");
}
