//! The §4 perf contract's outlined emit face: the byte-equal runtime
//! behavior pin — the outlined face's frozen layers + keywords carry
//! the IDENTICAL content the reference face freezes (the same decls,
//! conds and keyword texts; only the expression shape differs: one
//! `StaticProgramBuilder::assemble` call over `static` slices instead
//! of the reference face's inline const block). The reference face
//! stays the differential emit.
use aeolus_codegen_gpui::{emit_static_program, emit_static_program_outlined};
use aeolus_compile::{Config, compile_theme, keyword_alignment, keyword_texts};
use aeolus_theme::{ThemeSource, TokenTable};

fn tokens_of(
    emit: fn(&aeolus_ir::Program, &[&str]) -> proc_macro2::TokenStream,
    literal: &str,
) -> String {
    let source = ThemeSource::shadcn().expect("the baked source parses");
    let table = TokenTable::baked(&source).expect("the baked table builds");
    let compiled = compile_theme(literal, &table, &Config::default()).unwrap();
    let annotations = keyword_texts(literal, &Config::default()).unwrap();
    let keywords = keyword_alignment(&compiled, &annotations);
    emit(&compiled.program, &keywords).to_string()
}

/// The frozen content: every `Prop::…`/`Cond::…`/`Val::…` variant and
/// string literal, in order — the data both faces freeze (the
/// expression shape is the only difference).
fn frozen_content(tokens: &str) -> Vec<String> {
    let text: String = tokens.chars().filter(|c| !c.is_whitespace()).collect();
    let mut out = Vec::new();
    for pattern in [
        "Prop::",
        "Cond::",
        "Val::",
        "Structural::",
        "Bp::",
        "Pref::",
        "Direction::",
    ] {
        let mut search = 0usize;
        while let Some(pos) = text[search..].find(pattern) {
            let start = search + pos;
            let end = text[start..]
                .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                .map(|i| start + i)
                .unwrap_or(text.len());
            out.push(text[start..end].to_string());
            search = end.max(start + 1);
        }
    }
    let mut rest = text.as_str();
    while let Some(open) = rest.find('"') {
        let after = &rest[open + 1..];
        match after.find('"') {
            Some(close) => {
                out.push(after[..close].to_string());
                rest = &after[close + 1..];
            }
            None => break,
        }
    }
    out
}

#[test]
fn the_outlined_face_is_byte_equal_to_the_reference_face() {
    for literal in [
        "p-4 bg-primary text-sm font-bold",
        "hover:bg-red-500/90 focus:ring-2 focus:ring-red-500 bg-red-500",
        "grid grid-cols-3 gap-4 h-32 ring-2 ring-red-500",
        "bg-linear-to-r from-red-500 to-blue-500",
        "group-hover/menu:bg-red-500",
    ] {
        let reference = frozen_content(&tokens_of(emit_static_program, literal));
        let outlined = frozen_content(&tokens_of(emit_static_program_outlined, literal));
        assert!(!reference.is_empty(), "{literal}: the reference froze decls");
        assert_eq!(
            reference, outlined,
            "{literal}: the outlined face freezes the same program content"
        );
    }
}
