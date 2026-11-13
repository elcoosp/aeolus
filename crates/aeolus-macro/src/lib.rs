//! `tw!` — the zero-runtime class front door (architecture plan §3.1
//! emit (a)/(b), P3).
//!
//! The macro runs the V1 compiler at expansion time and emits the
//! program's STATIC form — the IR layers plus the row-keyword texts the
//! runtime backend maps natively. The input string is never emitted
//! back: F1 (`tw!` discarded its compile-time result) is closed here.
//!
//! ```ignore
//! // A static program value, applied with `aeolus_gpui::TwExt::tw`:
//! div().tw(tw!("p-4 bg-primary hover:bg-primary/90"))
//!
//! // The same seam merges static class lists at compile time:
//! div().tw(cn!("p-4", "px-8 rounded-full"))
//!
//! // A CSS-first theme file (the same one a web v4 project uses):
//! let theme = theme!("themes/app.css");
//! ```
//!
//! The macro compiles against the embedded shadcn palette by default so
//! colour tokens bake to literals (T0/T1). A host that needs a custom
//! theme compiles at runtime through [`aeolus_compile::compile_theme`]
//! and its `DynCache` — the documented dynamic lane.

//! The macro grammar is a hand-written token walk (the mission's syn-free
//! constraint); the pedantic lints it trips are excepted in the crate
//! manifest's `[lints.clippy]` table, with the rationale — the workspace
//! convention (root `Cargo.toml`) keeps every allow beside its reason.

use proc_macro::{Delimiter, TokenStream, TokenTree};
use std::collections::HashMap;
use std::fmt::Write as _;

use aeolus_compile::cn as merge_cn;
use aeolus_compile::{
    ClassKeywords, Compiled, Config, compile_theme, keyword_alignment, keyword_texts,
};
use aeolus_ir::Cond;
use aeolus_theme::{ThemeSource, TokenTable};

/// Compile a class string at build time into an `aeolus_rt::StaticProgram`.
#[proc_macro]
pub fn tw(input: TokenStream) -> TokenStream {
    let literal = match extract_single_string(&input) {
        Ok(s) => s,
        Err(msg) => return compile_error(&input, &msg),
    };
    let (compiled, classes) = match compile_literal(&literal) {
        Ok(pair) => pair,
        Err(msg) => return compile_error(&input, &msg),
    };
    let keywords = keyword_alignment(&compiled, &classes);
    aeolus_codegen_gpui::emit_static_program(&compiled.program, &keywords).into()
}

/// Compile-time `cn()`: compile each static list, merge at IR level, and
/// emit the merged static program.
#[proc_macro]
pub fn cn(input: TokenStream) -> TokenStream {
    let strings = match extract_string_list(input.clone()) {
        Ok(s) => s,
        Err(msg) => return compile_error(&input, &msg),
    };
    let mut compiled = Vec::new();
    let mut classes: Vec<ClassKeywords> = Vec::new();
    for string in &strings {
        match compile_literal(string) {
            Ok((c, k)) => {
                compiled.push(c);
                classes.extend(k);
            }
            Err(msg) => return compile_error(&input, &msg),
        }
    }
    let refs: Vec<&Compiled> = compiled.iter().collect();
    let merged = match merge_cn(&refs) {
        Ok(merged) => merged,
        Err(err) => return compile_error(&input, &format!("cn!: {err}")),
    };
    let keywords = last_keywords(&merged, &classes);
    aeolus_codegen_gpui::emit_static_program(&merged.program, &keywords).into()
}

/// The compatibility alias kept for the pre-1.0 spelling (`tw_cn!`).
#[proc_macro]
pub fn tw_cn(input: TokenStream) -> TokenStream {
    cn(input)
}

/// Parse a CSS-first theme file at the host (`@theme`/`:root`/`.dark`,
/// the same file a web Tailwind v4 project uses).
///
/// One argument: the light source. Two arguments: light + dark override.
#[proc_macro]
pub fn theme(input: TokenStream) -> TokenStream {
    let strings = match extract_string_list(input.clone()) {
        Ok(s) => s,
        Err(msg) => return compile_error(&input, &msg),
    };
    match strings.as_slice() {
        [light] => {
            let light = format!("include_str!({light:?})");
            let light: TokenStream = light.parse().expect("include_str path");
            quote_ts(&format!(
                "::aeolus_rt::theme_from_css({light}, ::core::option::Option::None)"
            ))
        }
        [light, dark] => {
            let light = format!("include_str!({light:?})");
            let dark = format!("include_str!({dark:?})");
            quote_ts(&format!(
                "::aeolus_rt::theme_from_css({light}, ::core::option::Option::Some({dark}))"
            ))
        }
        _ => compile_error(&input, "theme!: expected one or two string literal paths"),
    }
}

// ---------------------------------------------------------------------------
// The compile walk
// ---------------------------------------------------------------------------

fn compile_literal(literal: &str) -> Result<(Compiled, Vec<ClassKeywords>), String> {
    let source = ThemeSource::shadcn().map_err(|err| format!("tw!: theme: {err}"))?;
    let table = TokenTable::baked(&source).map_err(|err| format!("tw!: theme: {err}"))?;
    let config = Config::default();
    let compiled = compile_theme(literal, &table, &config).map_err(|err| format!("tw!: {err}"))?;
    if !compiled.diagnostics.is_empty() {
        let mut msg = String::new();
        for diagnostic in &compiled.diagnostics {
            let _ = write!(msg, "tw!: {diagnostic}; ");
        }
        return Err(msg);
    }
    let classes = keyword_texts(literal, &config).map_err(|err| format!("tw!: {err}"))?;
    Ok((compiled, classes))
}

/// The merged form: one text per merged declaration, keyed by the LAST
/// source occurrence (rightmost wins, the `cn()` contract).
fn last_keywords(compiled: &Compiled, classes: &[ClassKeywords]) -> Vec<&'static str> {
    let mut last: HashMap<(Cond, aeolus_ir::Prop), &'static str> = HashMap::new();
    for class in classes {
        for &(prop, text) in &class.keywords {
            last.insert((class.cond, prop), text);
        }
    }
    let mut out = Vec::new();
    for layer in compiled.program.layers {
        for decl in layer.decls {
            out.push(last.get(&(layer.cond, decl.prop)).copied().unwrap_or(""));
        }
    }
    out
}

fn quote_ts(text: &str) -> TokenStream {
    text.parse()
        .unwrap_or_else(|_| compile_error_msg("macro: generated tokens did not parse"))
}

// ---------------------------------------------------------------------------
// Token parsing (syn-free, the mission's own constraint)
// ---------------------------------------------------------------------------

fn extract_single_string(input: &TokenStream) -> Result<String, String> {
    let mut strings = extract_string_list(input.clone())?;
    if strings.len() != 1 {
        return Err("tw!: expected exactly one string literal".into());
    }
    Ok(strings.remove(0))
}

fn extract_string_list(input: TokenStream) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut pending: Option<String> = None;
    for token in input {
        match token {
            TokenTree::Literal(lit) => {
                let s = lit.to_string();
                if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                    let unquoted = unescape(&s[1..s.len() - 1]);
                    match pending.as_mut() {
                        Some(prev) => prev.push_str(&unquoted),
                        None => pending = Some(unquoted),
                    }
                } else {
                    return Err(format!("tw!: unsupported literal {s} (expected a string)"));
                }
            }
            TokenTree::Punct(p) if p.as_char() == ',' => {
                if let Some(s) = pending.take() {
                    out.push(s);
                }
            }
            TokenTree::Group(g) if g.delimiter() == Delimiter::None => {
                let inner = extract_string_list(g.stream())?;
                out.extend(inner);
            }
            other => {
                return Err(format!(
                    "tw!: unexpected token {other} (expected a string literal)"
                ));
            }
        }
    }
    if let Some(s) = pending.take() {
        out.push(s);
    }
    Ok(out)
}

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some('0') => out.push('\0'),
            // Rust's escaped newline: `\` then a line break, the
            // following indentation swallowed. Class strings in the wild
            // wrap this way.
            Some('\n' | '\r') => {
                while chars.peek().is_some_and(|next| next.is_whitespace()) {
                    let _ = chars.next();
                }
            }
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

fn compile_error(input: &TokenStream, msg: &str) -> TokenStream {
    let text = format!("compile_error!({msg:?})");
    let _ = input;
    text.parse()
        .unwrap_or_else(|_| compile_error_msg("tw!: could not render the error"))
}

fn compile_error_msg(msg: &str) -> TokenStream {
    format!("compile_error!({msg:?})")
        .parse()
        .expect("compile_error! is valid Rust")
}

// ---------------------------------------------------------------------------
// `variants!`: compile-time component variant tables
// ---------------------------------------------------------------------------

struct VariantTable {
    visibility: String,
    name: String,
    fields: Vec<VariantField>,
}

struct VariantField {
    field: String,
    classes: VariantClasses,
}

enum VariantClasses {
    Base(String),
    Groups(Vec<(String, String)>),
    /// `compound: [ ... ]` — the cva compound-variant entries (plan §9's
    /// open question, decided): extra classes applied to every combo
    /// that matches ALL of an entry's constraints, resolved at EXPANSION
    /// time (the entries fold into the cartesian match arms the render
    /// already emits — zero runtime cost beyond the dispatch).
    Compound(Vec<CompoundEntry>),
}

/// One `compound:` entry: the group-key constraints (ALL must match the
/// selection) and the class list they ride.
struct CompoundEntry {
    constraints: Vec<(String, String)>,
    classes: String,
}

/// Compile a closed-world variant table (the shadcn/cva surface):
///
/// ```ignore
/// variants! { pub Button {
///     base: "inline-flex items-center gap-2 rounded-md font-medium",
///     variant: { default: "bg-primary text-primary-foreground",
///                destructive: "bg-destructive text-white" },
///     size: { default: "h-9 px-4 py-2", sm: "h-8 px-3" },
/// } }
/// ```
///
/// Generates one enum per group, a `style(...)` function returning the
/// merged `aeolus_rt::StaticProgram`, and a `Default` enum variant
/// (the key named `default`, else the first key).
#[proc_macro]
pub fn variants(input: TokenStream) -> TokenStream {
    let table = match parse_variants(input.clone()) {
        Ok(table) => table,
        Err(msg) => return compile_error(&input, &msg),
    };
    match render_variants(&table) {
        Ok(tokens) => tokens,
        Err(msg) => compile_error(&input, &msg),
    }
}

fn parse_variants(input: TokenStream) -> Result<VariantTable, String> {
    let mut tokens = input.into_iter().peekable();
    let mut visibility = String::new();
    if matches!(tokens.peek(), Some(TokenTree::Ident(id)) if id.to_string() == "pub") {
        let _ = tokens.next();
        visibility.push_str("pub ");
        if let Some(TokenTree::Group(group)) = tokens.peek() {
            if group.delimiter() == Delimiter::Parenthesis {
                let mut text = String::from("pub(");
                for token in group.stream() {
                    text.push_str(&token.to_string());
                }
                text.push_str(") ");
                let _ = tokens.next();
                visibility = text;
            }
        }
    }
    let name = match tokens.next() {
        Some(TokenTree::Ident(id)) => id.to_string(),
        _ => return Err("variants!: expected a component type name".into()),
    };
    let body = match tokens.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => group.stream(),
        _ => return Err("variants!: expected `{ ... }` after the type name".into()),
    };

    let entries = split_top_level(body, ',');
    let mut fields = Vec::new();
    let mut pending_field: Option<String> = None;
    for entry in entries {
        let parts = split_first_colon(entry);
        let Some((field, rest)) = parts else {
            continue;
        };
        let field = field.trim().to_string();
        let rest = rest.trim();
        if rest.is_empty() {
            pending_field = Some(field);
            continue;
        }
        let classes = match rest.parse::<TokenStream>() {
            Ok(stream) => parse_variant_classes(stream, &field)?,
            Err(_) => return Err(format!("variants!: `{field}` has an invalid value")),
        };
        fields.push(VariantField { field, classes });
        pending_field = None;
    }
    if fields.is_empty() {
        return Err("variants!: the table has no fields".into());
    }
    if let Some(field) = pending_field {
        return Err(format!("variants!: `{field}` has no value"));
    }
    Ok(VariantTable {
        visibility,
        name,
        fields,
    })
}

fn parse_variant_classes(rest: TokenStream, field: &str) -> Result<VariantClasses, String> {
    let tokens: Vec<TokenTree> = rest.clone().into_iter().collect();
    // The compound LIST: a bracket group of brace entries, each naming
    // group-key constraints plus the `class` list they ride.
    if field == "compound" {
        if let Some(TokenTree::Group(group)) = tokens.first() {
            if group.delimiter() == Delimiter::Bracket {
                return parse_compound_entries(group.stream(), field);
            }
        }
        return Err(format!(
            "variants!: `{field}` expects a `[ {{ ... }} ]` list of entries"
        ));
    }
    if let Some(TokenTree::Group(group)) = tokens.first() {
        if group.delimiter() == Delimiter::Brace {
            let mut groups = Vec::new();
            let mut pending: Option<String> = None;
            for entry in split_top_level(group.stream(), ',') {
                let parts = split_first_colon(entry);
                let Some((key, value)) = parts else {
                    continue;
                };
                let key = key.trim().to_string();
                let value = value.trim();
                if value.is_empty() {
                    pending = Some(key);
                    continue;
                }
                let stream: TokenStream = value
                    .parse()
                    .map_err(|_| format!("variants!: `{field}.{key}` is not a string"))?;
                let text = extract_single_string(&stream)?;
                groups.push((key, text));
                pending = None;
            }
            if let Some(key) = pending {
                return Err(format!("variants!: `{field}.{key}` has no class list"));
            }
            if groups.is_empty() {
                return Err(format!("variants!: `{field}` group is empty"));
            }
            return Ok(VariantClasses::Groups(groups));
        }
    }
    if let Some(TokenTree::Literal(_)) = tokens.first() {
        return Ok(VariantClasses::Base(extract_single_string(&rest)?));
    }
    Err(format!(
        "variants!: `{field}` expects a string or `{{ ... }}`"
    ))
}

/// Parse a `compound: [ ... ]` list: each brace entry carries
/// `group: key` constraint pairs plus the `class` list they apply to.
fn parse_compound_entries(stream: TokenStream, field: &str) -> Result<VariantClasses, String> {
    let mut entries = Vec::new();
    for entry in split_top_level(stream, ',') {
        let tokens: Vec<TokenTree> = entry.into_iter().collect();
        let Some(TokenTree::Group(group)) = tokens.first() else {
            continue;
        };
        if group.delimiter() != Delimiter::Brace {
            return Err(format!(
                "variants!: `{field}` entries are brace groups (`{{ variant: key, class: \"…\" }}`)"
            ));
        }
        let mut constraints = Vec::new();
        let mut classes: Option<String> = None;
        for pair in split_top_level(group.stream(), ',') {
            let Some((key, value)) = split_first_colon(pair) else {
                continue;
            };
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            if key == "class" {
                let text = extract_single_string(
                    &value
                        .parse::<TokenStream>()
                        .map_err(|_| format!("variants!: `{field}.class` is not a string"))?,
                )?;
                classes = Some(text);
            } else {
                constraints.push((key, value));
            }
        }
        if constraints.is_empty() {
            return Err(format!(
                "variants!: `{field}` entry constrains nothing (name at least one group key)"
            ));
        }
        let Some(classes) = classes else {
            return Err(format!("variants!: `{field}` entry has no class list"));
        };
        entries.push(CompoundEntry {
            constraints,
            classes,
        });
    }
    if entries.is_empty() {
        return Err(format!("variants!: `{field}` list is empty"));
    }
    Ok(VariantClasses::Compound(entries))
}

fn split_top_level(stream: TokenStream, separator: char) -> Vec<TokenStream> {
    let mut out = Vec::new();
    let mut current = Vec::new();
    for token in stream {
        match &token {
            TokenTree::Punct(punct) if punct.as_char() == separator => {
                if !current.is_empty() {
                    out.push(current.drain(..).collect());
                }
            }
            _ => current.push(token),
        }
    }
    if !current.is_empty() {
        out.push(current.into_iter().collect());
    }
    out
}

fn split_first_colon(stream: TokenStream) -> Option<(String, String)> {
    let mut left = String::new();
    let mut right = String::new();
    let mut seen = false;
    for token in stream {
        if !seen {
            if let TokenTree::Punct(punct) = &token {
                if punct.as_char() == ':' {
                    seen = true;
                    continue;
                }
            }
            left.push_str(&token.to_string());
        } else {
            right.push_str(&token.to_string());
            if let TokenTree::Punct(punct) = &token {
                if punct.as_char() == ',' {
                    right.pop();
                }
            }
        }
    }
    seen.then_some((left, right))
}

fn pascal_case(text: &str) -> String {
    let mut out = String::new();
    for part in text.split(|c: char| !c.is_alphanumeric()) {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.extend(chars);
        }
    }
    if out.is_empty() {
        out.push_str("Variant");
    }
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, 'V');
    }
    out
}

fn render_variants(table: &VariantTable) -> Result<TokenStream, String> {
    let mut output = String::new();
    let visibility = &table.visibility;
    let name = &table.name;
    let mut enum_defs = String::new();
    let mut signature = Vec::new();
    let mut group_fields = Vec::new();

    for field in &table.fields {
        if let VariantClasses::Groups(groups) = &field.classes {
            let enum_name = format!("{name}{}", pascal_case(&field.field));
            enum_defs.push_str(&format!(
                "#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]\n{visibility}enum {enum_name} {{\n"
            ));
            let default_key = groups
                .iter()
                .find(|(key, _)| key == "default")
                .map_or(0, |(key, _)| {
                    groups.iter().position(|(k, _)| k == key).unwrap_or(0)
                });
            for (index, (key, _)) in groups.iter().enumerate() {
                let variant = pascal_case(key);
                if index == default_key {
                    enum_defs.push_str("    #[default]\n");
                }
                enum_defs.push_str(&format!("    {variant},\n"));
            }
            enum_defs.push_str("}\n");
            signature.push(format!("{}: {enum_name}", field.field));
            group_fields.push((field.field.clone(), enum_name, groups.clone()));
        }
    }

    let base: String = table
        .fields
        .iter()
        .filter_map(|field| match &field.classes {
            VariantClasses::Base(classes) => Some(classes.as_str()),
            VariantClasses::Groups(_) | VariantClasses::Compound(_) => None,
        })
        .collect::<Vec<_>>()
        .join(" ");

    let params = signature.join(", ");
    let mut arms = String::new();
    if group_fields.is_empty() {
        // A compound entry constrains GROUPS; a table without groups has
        // nothing to constrain (the honest grammar error).
        let compounds: Vec<&CompoundEntry> = table
            .fields
            .iter()
            .filter_map(|field| match &field.classes {
                VariantClasses::Compound(entries) => Some(entries.iter().collect::<Vec<_>>()),
                _ => None,
            })
            .flatten()
            .collect();
        if let Some(entry) = compounds.first() {
            let _ = entry;
            return Err(
                "variants!: `compound` constrains a group the table does not declare".into(),
            );
        }
        let (program, _) = compile_literal(&base).map_err(|err| format!("variants!: {err}"))?;
        let keywords = keyword_alignment(&program, &[]);
        let tokens = aeolus_codegen_gpui::emit_static_program(&program.program, &keywords);
        output.push_str(&enum_defs);
        output.push_str(&format!(
            "{visibility}struct {name};\nimpl {name} {{\n    {visibility}fn style() -> ::aeolus_rt::StaticProgram {{\n        {tokens}\n    }}\n}}\n"
        ));
        return output
            .parse()
            .map_err(|_| "variants!: generated code did not parse".into());
    }

    // The compound entries, with every constraint validated against the
    // declared groups (an unknown group or key is a grammar error, not
    // a silent never-matching arm).
    let compounds: Vec<&CompoundEntry> = table
        .fields
        .iter()
        .filter_map(|field| match &field.classes {
            VariantClasses::Compound(entries) => Some(entries.iter().collect::<Vec<_>>()),
            _ => None,
        })
        .flatten()
        .collect();
    for entry in &compounds {
        for (group, key) in &entry.constraints {
            let declared = group_fields.iter().any(|(field, _, groups)| {
                field == group && groups.iter().any(|(group_key, _)| group_key == key)
            });
            if !declared {
                return Err(format!(
                    "variants!: `compound` constrains `{group}: {key}` — no such group key"
                ));
            }
        }
    }

    let combos = cartesian(&group_fields);
    for combo in combos {
        let mut pattern = Vec::new();
        let mut classes = base.clone();
        for (field, enum_name, groups) in &group_fields {
            let key = combo
                .iter()
                .find(|(field_name, _)| field_name == field)
                .map(|(_, key)| key.as_str())
                .unwrap_or_else(|| groups[0].0.as_str());
            let class = groups
                .iter()
                .find(|(group_key, _)| group_key == key)
                .map_or("", |(_, classes)| classes.as_str());
            if !classes.is_empty() {
                classes.push(' ');
            }
            classes.push_str(class);
            pattern.push(format!("{enum_name}::{}", pascal_case(key)));
        }
        // The compound entries whose constraints ALL match this combo
        // fold their class list in — cva's semantics, resolved at
        // expansion time (the arm already carries the whole class
        // string).
        for entry in &compounds {
            if entry
                .constraints
                .iter()
                .all(|(group, key)| combo.iter().any(|(f, k)| f == group && k == key))
            {
                if !classes.is_empty() {
                    classes.push(' ');
                }
                classes.push_str(&entry.classes);
            }
        }
        let (program, class_annotations) =
            compile_literal(&classes).map_err(|err| format!("variants!: {err}"))?;
        let keywords = keyword_alignment(&program, &class_annotations);
        let tokens = aeolus_codegen_gpui::emit_static_program(&program.program, &keywords);
        arms.push_str(&format!("        ({}) => {tokens},\n", pattern.join(", ")));
    }

    output.push_str(&enum_defs);
    output.push_str(&format!("{visibility}struct {name};\n"));
    output.push_str(&format!(
        "impl {name} {{\n    {visibility}fn style({params}) -> ::aeolus_rt::StaticProgram {{\n        match ("
    ));
    let names: Vec<&str> = group_fields
        .iter()
        .map(|(field, _, _)| field.as_str())
        .collect();
    output.push_str(&names.join(", "));
    output.push_str(") {\n");
    output.push_str(&arms);
    output.push_str("        }\n    }\n}\n");
    output
        .parse()
        .map_err(|_| "variants!: generated code did not parse".into())
}

fn cartesian(fields: &[(String, String, Vec<(String, String)>)]) -> Vec<Vec<(String, String)>> {
    let mut rows = vec![Vec::new()];
    for (field, _, groups) in fields {
        let mut next = Vec::new();
        for row in &rows {
            for (key, _) in groups {
                let mut extended = row.clone();
                extended.push((field.clone(), key.clone()));
                next.push(extended);
            }
        }
        rows = next;
    }
    rows
}
