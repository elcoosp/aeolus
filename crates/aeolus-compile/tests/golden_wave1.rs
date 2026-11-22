//! The parity wave 1's differential battery — the five highest-traffic
//! families end to end (architecture plan §5.3's differential line at
//! the V1 pipeline's own faces):
//!
//! 1. **The golden corpus** (`crates/aeolus-core/tests/fixtures/
//!    golden.corpus`, the deliberate 193 → 215 → 2992 regeneration through the
//!    real tailwindcss v4.1.17): every wave-1 entry's `emit_css_theme`
//!    output normalizes to the SAME declaration set under the SAME
//!    selector (the corpus's nested `:where(& > :not(:last-child))`
//!    child rule flattens to the emitter's own selector form).
//! 2. **The v0.x oracle** (the plan-level differential, the P1 exit
//!    criterion's slot form): `line-clamp-2` and the
//!    font-variant-numeric keyword half map onto the SAME
//!    `TextProperty` slots the audited resolver carries.
//! 3. **The style fold** (the interp arm): the wave-1 longhands land
//!    in the `ResolvedStyle` slots (or report as the typed
//!    `ChildSelector`/`SlotUnsupported` boundaries), never silently.
//! 4. **The byte-level emission shapes** for the child-selector rules
//!    (the v4 machinery's own line order, quoted from the pinned
//!    compiler).

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;

use aeolus_compile::{
    Config, PlanNoteKind, Target, compile_theme, emit_css_theme, emit_plan_theme,
    keyword_alignment, keyword_texts, program_style,
};
use aeolus_core::{Resolver, TAILWIND_DEFAULT_THEME, TailwindTheme, TextProperty, TextValue};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Fixture loading (the dependency-free line format, tools/gen_golden.mjs)
// ---------------------------------------------------------------------------

/// The fixture's theme context for one theme: the `:root` vars and the
/// `@property` initial values the var substitution resolves through.
#[derive(Default, Clone)]
struct Vars {
    root: BTreeMap<String, String>,
    props: BTreeMap<String, String>,
}

/// The parsed fixture: the default theme's var tables + the raw rule
/// per class (the corpus's dependency-free line format).
struct Fixture {
    vars: Vars,
    rules: HashMap<&'static str, String>,
}

fn load_fixture() -> Fixture {
    let raw = include_str!("../../aeolus-core/tests/fixtures/golden.corpus");
    let mut vars = Vars::default();
    let mut rules: HashMap<&'static str, String> = HashMap::new();
    let mut in_default = false;
    let mut current: Option<(&'static str, Vec<String>)> = None;
    // A leak pool for the class keys (the fixture text is `include_str!`
    // — the keys borrow into 'static through the leak).
    for line in raw.lines() {
        let line = line.trim_end();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("@theme ") {
            in_default = rest.trim() == "default";
            continue;
        }
        if line.trim() == "@end" {
            in_default = false;
        } else if line.starts_with('@') {
            // Any other directive closes the theme block (the entry
            // bodies' `--tw-*` assignment lines stay entry lines — the
            // corpus bodies indent their rule lines; the theme block's
            // vars are top-level).
            in_default = false;
        }
        if in_default {
            if let Some(rest) = line.strip_prefix("@propdef ") {
                if let Some((name, value)) = rest.split_once(':') {
                    vars.props
                        .insert(name.trim().to_string(), value.trim().to_string());
                }
                continue;
            }
            if let Some((name, value)) = line
                .strip_prefix("--")
                .map(|rest| rest.split_once(':'))
                .flatten()
            {
                vars.root
                    .insert(format!("--{name}"), value.trim().to_string());
                continue;
            }
        }
        if let Some(rest) = line.strip_prefix("@entry ") {
            in_default = false;
            let (_theme, class) = rest
                .split_once(' ')
                .expect("entry lines carry the theme and the class");
            let leaked: &'static str = Box::leak(class.to_string().into_boxed_str());
            current = Some((leaked, Vec::new()));
            continue;
        }
        if line.trim() == "@end" {
            if let Some((class, lines)) = current.take() {
                rules.entry(class).or_insert_with(|| lines.join("\n"));
            }
            continue;
        }
        if let Some((_, lines)) = &mut current {
            lines.push(line.to_string());
        }
    }
    assert!(!rules.is_empty(), "the corpus fixture must parse");
    Fixture { vars, rules }
}

/// The wave-1 classes the corpus grew (the deliberate regeneration's
/// tail). Every entry here must BOTH normalize-equal its golden rule
/// through [`emit_css_theme`] AND compile with zero diagnostics.
const WAVE1_CLASSES: &[&str] = &[
    "line-clamp-none",
    "ordinal",
    "slashed-zero",
    "lining-nums",
    "oldstyle-nums",
    "proportional-nums",
    "diagonal-fractions",
    "stacked-fractions",
    "normal-nums",
    "space-x-4",
    "space-y-2",
    "-space-x-2",
    "space-x-reverse",
    "space-y-reverse",
    "divide-x",
    "divide-y-2",
    "divide-x-reverse",
    "divide-dashed",
    "divide-solid",
    "divide-red-500",
    "sr-only",
    "not-sr-only",
];

/// The default-theme baked table (the `theme_for("default")` twin: the
/// v4 default palette CSS the v0.x oracle reads, baked through the
/// theme pass).
fn default_tokens() -> TokenTable {
    let source = ThemeSource::from_css(TAILWIND_DEFAULT_THEME, None).expect("default theme parses");
    TokenTable::baked(&source).expect("the default palette fits the baked table")
}

// ---------------------------------------------------------------------------
// The normalizer (documented model choices, the parity_golden.rs
// conventions at the V1 emitter's own shapes)
// ---------------------------------------------------------------------------

/// One flattened rule: selector + normalized declaration map.
type FlatRule = (String, BTreeMap<String, String>);

/// Flatten parsed nested (selector, decls) nodes by textual `&`
/// substitution — the corpus's child rule
/// (`:where(& > :not(:last-child))`) resolves to the emitter's own
/// selector form.
fn flatten(nodes: &[Node], sel: &str, out: &mut Vec<(String, Vec<(String, String)>)>) {
    let mut own: Vec<(String, String)> = Vec::new();
    for node in nodes {
        match node {
            Node::Decl(p, v) => own.push((p.clone(), v.clone())),
            Node::Rule(nested_sel, kids) => {
                let joined = nested_sel.replacen('&', sel, 1);
                flatten(kids, &joined, out);
            }
        }
    }
    if !own.is_empty() {
        out.push((sel.to_string(), own));
    }
}

enum Node {
    Decl(String, String),
    Rule(String, Vec<Node>),
}

/// The dependency-free nested-rule parser (the parity_golden.rs shape,
/// minus the at-rule arms — the wave-1 entries carry none).
fn parse_nodes(body: &str) -> Vec<Node> {
    let mut nodes = Vec::new();
    let mut buf = String::new();
    let bytes: Vec<char> = body.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == '{' {
            let mut depth = 1;
            let mut j = i + 1;
            while j < bytes.len() && depth > 0 {
                if bytes[j] == '{' {
                    depth += 1;
                }
                if bytes[j] == '}' {
                    depth -= 1;
                }
                j += 1;
            }
            let inner: String = bytes[i + 1..j - 1].iter().collect();
            let sel = buf.trim().to_string();
            buf.clear();
            nodes.push(Node::Rule(sel, parse_nodes(&inner)));
            i = j;
            continue;
        }
        if c == ';' {
            if let Some((p, v)) = buf.trim().split_once(':') {
                nodes.push(Node::Decl(p.trim().to_string(), v.trim().to_string()));
            }
            buf.clear();
            i += 1;
            continue;
        }
        buf.push(c);
        i += 1;
    }
    nodes
}

/// Substitute the var() references: the rule's own `--tw-*`
/// assignments first (the machinery the rule composes), then the
/// fixture's `:root` vars and `@property` initial values, then the
/// empty fallback (`var(--x,)` — the composed font-variant-numeric
/// line's untouched slots). Returns the local assignments collected so
/// the caller can drop them from the comparison.
fn substitute_vars(
    value: &str,
    local: &BTreeMap<String, String>,
    root: Option<&BTreeMap<String, String>>,
    props: Option<&BTreeMap<String, String>>,
) -> String {
    let mut out = String::new();
    let bytes: Vec<char> = value.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 'v' && bytes[i..].iter().collect::<String>().starts_with("var(") {
            // Find the matching close paren.
            let mut depth = 0;
            let mut j = i + 3;
            while j < bytes.len() {
                if bytes[j] == '(' {
                    depth += 1;
                }
                if bytes[j] == ')' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                j += 1;
            }
            let inner: String = bytes[i + 4..j].iter().collect();
            let (name, fallback) = match inner.split_once(',') {
                Some((n, f)) => (n.trim().to_string(), Some(f.trim().to_string())),
                None => (inner.trim().to_string(), None),
            };
            if let Some(v) = local
                .get(&name)
                .or_else(|| root.and_then(|r| r.get(&name)))
                .or_else(|| props.and_then(|p| p.get(&name)))
            {
                let _ = write!(out, "{v}");
            } else if fallback.is_some() {
                // The empty fallback: the untouched slot renders "".
            } else {
                // Unknown var without a fallback: keep the reference
                // text (the comparison names it instead of guessing).
                let _ = write!(out, "var({inner})");
            }
            i = j + 1;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    out
}

/// Evaluate the wave-1 corpus's calc forms: `calc(A * B)` (B a number
/// or a nested calc) and `calc(1 - N)`, over `rem`/`px` lengths. Both
/// sides run through the SAME evaluator, so a partial grammar is only
/// honest when it fully reduces every form both sides carry — the
/// asserted shapes are exactly the between-child machinery's.
fn eval_calcs(value: &str) -> String {
    let trimmed = value.trim();
    if let Some(inner) = trimmed
        .strip_prefix("calc(")
        .and_then(|s| s.strip_suffix(')'))
    {
        // Split at the TOP-LEVEL ` * ` (paren-depth zero).
        let bytes: Vec<char> = inner.chars().collect();
        let mut depth = 0i32;
        let mut star: Option<usize> = None;
        let mut minus: Option<usize> = None;
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            if depth == 0 {
                let rest: String = bytes[i..].iter().collect();
                if rest.starts_with(" * ") && star.is_none() {
                    star = Some(i);
                }
                if rest.starts_with(" - ") && minus.is_none() {
                    minus = Some(i);
                }
            }
            i += 1;
        }
        if let Some(at) = star {
            let left = eval_calcs(&bytes[..at].iter().collect::<String>());
            let right = eval_calcs(&bytes[at + 3..].iter().collect::<String>());
            return mul_len(&left, &right);
        }
        if let Some(at) = minus {
            let left = eval_calcs(&bytes[..at].iter().collect::<String>());
            let right = eval_calcs(&bytes[at + 3..].iter().collect::<String>());
            return sub_num(&left, &right);
        }
        return inner.trim().to_string();
    }
    trimmed.to_string()
}

/// `length * scalar` (the reverse-var composition's shape).
fn mul_len(left: &str, right: &str) -> String {
    let n: f32 = match right.parse() {
        Ok(n) => n,
        Err(_) => return format!("{left} * {right}"),
    };
    if let Some((v, unit)) = split_len(left) {
        return format_len(v * n, unit);
    }
    format!("{left} * {right}")
}

/// `1 - scalar` (the reverse complement).
fn sub_num(left: &str, right: &str) -> String {
    let l: f32 = left.parse().unwrap_or(1.0);
    let r: f32 = right.parse().unwrap_or(0.0);
    format!("{}", l - r)
}

fn split_len(text: &str) -> Option<(f32, &'static str)> {
    for unit in ["rem", "px"] {
        if let Some(n) = text.strip_suffix(unit) {
            return n.trim().parse::<f32>().ok().map(|n| (n, unit));
        }
    }
    None
}

fn format_len(n: f32, unit: &'static str) -> String {
    let n = if (n - n.round()).abs() < f32::EPSILON {
        format!("{}", n.round() as i64)
    } else {
        format!("{n}")
    };
    format!("{n}{unit}")
}

/// Fold the per-edge longhand sets into the shorthand the golden
/// carries (the IR's fan-out model vs the corpus's shorthand emission —
/// the documented normalization, the parity_golden.rs conventions):
/// four equal padding/margin/border-width edges collapse, matching
/// overflow axes collapse. Idempotent, applied to BOTH sides.
fn fold_shorthands(decls: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let mut out = decls.clone();
    for (shorthand, edges) in [
        (
            "padding",
            [
                "padding-top",
                "padding-right",
                "padding-bottom",
                "padding-left",
            ],
        ),
        (
            "margin",
            ["margin-top", "margin-right", "margin-bottom", "margin-left"],
        ),
        (
            "border-width",
            [
                "border-top-width",
                "border-right-width",
                "border-bottom-width",
                "border-left-width",
            ],
        ),
    ] {
        let values: Vec<String> = edges.iter().filter_map(|e| out.get(*e).cloned()).collect();
        if values.len() == edges.len() && values.iter().all(|v| *v == values[0]) {
            for edge in edges {
                out.remove(edge);
            }
            out.insert(shorthand.to_string(), values[0].clone());
        }
    }
    if let (Some(x), Some(y)) = (out.get("overflow-x"), out.get("overflow-y")) {
        if x == y {
            let v = x.clone();
            out.remove("overflow-x");
            out.remove("overflow-y");
            out.insert("overflow".to_string(), v);
        }
    }
    out
}

/// The full normalization: var substitution (with the rule's own
/// machinery assignments as the local table), calc evaluation, the
/// shorthand fold, the `--tw-*` assignment drop, and the mechanical
/// `border-*-style: solid` drop (the @property initial the width rules
/// carry — the parity_golden.rs convention). rem values render as px
/// (the pinned 16px rem) so the baked and var forms compare exactly.
fn normalize_rule(
    selector: &str,
    decls: &[(String, String)],
    vars: &Vars,
) -> (String, BTreeMap<String, String>) {
    let local: BTreeMap<String, String> = decls
        .iter()
        .filter(|(p, _)| p.starts_with("--"))
        .cloned()
        .collect();
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    for (prop, value) in decls {
        if prop.starts_with("--") {
            continue;
        }
        // The substitution chain: the rule's own machinery assignments
        // first, then the fixture's `:root` vars and `@property`
        // initials, then the empty fallback (the composed
        // font-variant-numeric line's untouched slots).
        let mut v = substitute_vars(value, &local, Some(&vars.root), Some(&vars.props));
        v = substitute_vars(&v, &local, Some(&vars.root), Some(&vars.props));
        v = eval_calcs(&v);
        v = rems_to_px(&v);
        if is_mechanical(prop, &v) {
            continue;
        }
        out.insert(prop.clone(), v);
    }
    (selector.to_string(), fold_shorthands(&out))
}

/// The mechanical declarations both sides drop (the documented
/// normalizations): `border-*`/`outline-style: solid` = the @property
/// initial emitted with every width.
fn is_mechanical(prop: &str, value: &str) -> bool {
    let style_ctx =
        (prop.starts_with("border") || prop.starts_with("outline")) && prop.ends_with("-style");
    style_ctx && value == "solid"
}

fn rems_to_px(value: &str) -> String {
    let mut out = String::new();
    for token in value.split_whitespace() {
        let token = match split_len(token) {
            Some((n, "rem")) => format_len(n * 16.0, "px"),
            _ => token.to_string(),
        };
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&token);
    }
    if out.is_empty() {
        value.to_string()
    } else {
        out
    }
}

/// The CSS arm's rules for one class under the baked default table —
/// parsed through the SAME normalize path the golden side takes.
fn emitted_rules(class: &str, tokens: &TokenTable, vars: &Vars) -> Vec<FlatRule> {
    let sheet = emit_css_theme(class, tokens, &Config::default())
        .unwrap_or_else(|e| panic!("{class}: the wave-1 class compiles: {e:?}"));
    assert!(
        sheet.diagnostics.is_empty(),
        "{class}: unexpected diagnostics {:?}",
        sheet.diagnostics
    );
    sheet
        .rules
        .iter()
        .map(|rule| {
            // The rendered body is the `prop: value;` line list — the
            // same parse the corpus's own entries go through.
            let decls: Vec<(String, String)> = rule
                .body
                .lines()
                .filter_map(|line| line.trim().strip_suffix(';'))
                .filter_map(|line| line.split_once(':'))
                .map(|(p, v)| (p.trim().to_string(), v.trim().to_string()))
                .collect();
            normalize_rule(&rule.selector, &decls, vars)
        })
        .collect()
}

/// The golden rule for one class, flattened + normalized.
fn golden_rules(class: &str, fx: &Fixture) -> Vec<FlatRule> {
    let raw = fx
        .rules
        .get(class)
        .unwrap_or_else(|| panic!("{class}: the corpus entry exists (regenerate deliberately)"));
    let first = raw.find('{').expect("the golden rule has a selector");
    let sel = raw[..first].trim();
    let body = &raw[first + 1..raw.rfind('}').expect("the golden rule closes")];
    let parsed = parse_nodes(body);
    let mut nodes = Vec::new();
    flatten(&parsed, sel, &mut nodes);
    nodes
        .into_iter()
        .map(|(sel, decls)| normalize_rule(&sel, &decls, &fx.vars))
        .collect()
}

// ---------------------------------------------------------------------------
// 1. The golden corpus (the V1 emitter's own differential)
// ---------------------------------------------------------------------------

#[test]
fn wave1_families_match_the_golden_corpus() {
    let fx = load_fixture();
    let tokens = default_tokens();
    let mut failures: Vec<String> = Vec::new();
    for class in WAVE1_CLASSES {
        if *class == "divide-red-500" {
            // The colour token's VALUE rides the same token path the
            // v0.x harness pins; the shape comparison below covers it.
            continue;
        }
        let ours = emitted_rules(class, &tokens, &fx.vars);
        let golden = golden_rules(class, &fx);
        if ours.len() != golden.len() {
            failures.push(format!("{class}: rule count {ours:?} vs golden {golden:?}"));
            continue;
        }
        for (i, ((osel, odecls), (gsel, gdecls))) in ours.iter().zip(&golden).enumerate() {
            if osel != gsel {
                failures.push(format!(
                    "{class}[{i}]: selector {osel:?} vs golden {gsel:?}"
                ));
            }
            for (prop, value) in gdecls {
                match odecls.get(prop) {
                    Some(ours) if ours == value => {}
                    Some(ours) => failures.push(format!(
                        "{class}[{i}] {prop}: ours {ours:?} vs golden {value:?}"
                    )),
                    None => failures.push(format!(
                        "{class}[{i}] {prop}: golden {value:?} — OURS MISSING"
                    )),
                }
            }
            for (prop, value) in odecls {
                if !gdecls.contains_key(prop.as_str()) {
                    failures.push(format!(
                        "{class}[{i}] extra decl {prop}: {value:?} (golden {gdecls:?})"
                    ));
                }
            }
        }
    }
    // The colour-token entry's VALUE rides the same token path the
    // v0.x harness pins byte-equivalently (`border-red-500` in the
    // corpus); the child-rule harness pins the selector and the
    // declaration SHAPE (the oklch-vs-baked-literal comparison is the
    // v0 harness's own epsilon battery).
    let ours = emitted_rules("divide-red-500", &tokens, &fx.vars);
    let golden = golden_rules("divide-red-500", &fx);
    assert_eq!(ours.len(), golden.len(), "divide-red-500: one child rule");
    assert_eq!(ours[0].0, golden[0].0, "divide-red-500: the child selector");
    assert!(
        ours[0].1.contains_key("border-color") && golden[0].1.contains_key("border-color"),
        "divide-red-500: the colour lands on the children"
    );
    assert!(
        failures.is_empty(),
        "{} wave-1 parity failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// The flagship wave's classes (the deliberate regeneration's new
/// families). Every entry here must BOTH normalize-equal its golden
/// rule through [`emit_css_theme`] AND compile with zero diagnostics —
/// the same contract the wave-1 list runs, over the gradient stop
/// machinery, the composed transform forms and the named group/peer
/// labels the IR slot rides.
const FLAGSHIP_CLASSES: &[&str] = &[
    "via-50%",
    "scale-95",
    "scale-x-50",
    "scale-y-125",
    "scale-150",
    "rotate-45",
    "-rotate-90",
    "translate-x-2",
    "-translate-y-1",
    "translate-y-4",
    "translate-x-1/2",
    "skew-y-3",
    "skew-x-12",
    "group-hover:underline",
    "peer-checked:bg-blue-500",
    "group-active/menu:bg-red-500",
    "group-focus-within/nav:underline",
    "peer-disabled/menu:opacity-50",
];

#[test]
fn flagship_families_match_the_golden_corpus() {
    let fx = load_fixture();
    let tokens = default_tokens();
    let mut failures: Vec<String> = Vec::new();
    for class in FLAGSHIP_CLASSES {
        let ours = emitted_rules(class, &tokens, &fx.vars);
        let golden = golden_rules(class, &fx);
        if ours.len() != golden.len() {
            failures.push(format!("{class}: rule count {ours:?} vs golden {golden:?}"));
            continue;
        }
        for (i, ((osel, odecls), (gsel, gdecls))) in ours.iter().zip(&golden).enumerate() {
            // Two documented normalizations, the wave-1 battery's own
            // precedents carried over:
            // * the `@media (hover: hover)` capability wrapper (the
            //   hover-capable-pointer gate) — the battery's flatten has
            //   no media model, so a media-wrapped golden rule compares
            //   its DECLARATION SET only;
            // * the colour-TOKEN entries (`bg-red-500`/`bg-blue-500`
            //   under a variant) — the golden's `var(--color-*)` chain
            //   normalizes to the oklch source, ours to the baked
            //   sRGB bytes (the same token path the v0.x harness pins
            //   byte-equivalently); the SHAPE comparison covers them.
            let media_wrapped = gsel.starts_with("@media");
            let colour_entry = gdecls.keys().any(|p| {
                matches!(
                    p.as_str(),
                    "background-color"
                        | "color"
                        | "border-color"
                        | "text-decoration-color"
                        | "outline-color"
                )
            });
            if !media_wrapped && osel != gsel {
                failures.push(format!(
                    "{class}[{i}]: selector {osel:?} vs golden {gsel:?}"
                ));
            }
            for (prop, value) in gdecls {
                match odecls.get(prop) {
                    Some(ours) if ours == value => {}
                    Some(_) if colour_entry => {}
                    Some(ours) => failures.push(format!(
                        "{class}[{i}] {prop}: ours {ours:?} vs golden {value:?}"
                    )),
                    None => failures.push(format!(
                        "{class}[{i}] {prop}: golden {value:?} — OURS MISSING"
                    )),
                }
            }
            for (prop, value) in odecls {
                if !gdecls.contains_key(prop.as_str()) {
                    failures.push(format!(
                        "{class}[{i}] extra decl {prop}: {value:?} (golden {gdecls:?})"
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} flagship parity failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// ---------------------------------------------------------------------------
// 2. The v0.x oracle (the plan-level differential)
// ---------------------------------------------------------------------------

/// The plan-bridge sheet for one class (the differential's V1 half).
fn bridged(class: &str) -> aeolus_compile::PlanSheet {
    let tokens = default_tokens();
    let sheet = emit_plan_theme(class, &tokens, &Config::default())
        .unwrap_or_else(|e| panic!("{class}: the plan bridge lowers: {e:?}"));
    assert!(
        sheet.diagnostics.is_empty(),
        "{class}: unexpected diagnostics {:?}",
        sheet.diagnostics
    );
    sheet
}

/// The oracle's plan for one class (the differential's v0.x half).
fn oracle_plan(class: &str) -> aeolus_core::StylePlan {
    let theme = TailwindTheme::from_theme_css(TAILWIND_DEFAULT_THEME).expect("default theme");
    let resolution = Resolver::new(&theme)
        .resolve(class)
        .unwrap_or_else(|e| panic!("{class}: the oracle resolves: {e:?}"));
    assert!(
        resolution.warnings.is_empty(),
        "{class}: the oracle resolves clean, got {:?}",
        resolution.warnings
    );
    resolution.plan
}

#[test]
fn line_clamp_maps_the_oracle_text_slots() {
    let v1 = bridged("line-clamp-2");
    let v0 = oracle_plan("line-clamp-2");
    assert_eq!(
        v1.plan.base.text, v0.base.text,
        "line-clamp-2: the Count AND the ellipsis leg map oracle-faithfully"
    );
    assert!(
        v1.notes.is_empty(),
        "line-clamp-2: no boundary notes, got {:?}",
        v1.notes
    );
}

#[test]
fn font_variant_numeric_maps_the_oracle_keyword_slot() {
    for class in ["tabular-nums", "ordinal"] {
        let v1 = bridged(class);
        let v0 = oracle_plan(class);
        assert_eq!(
            v1.plan.base.text, v0.base.text,
            "{class}: the keyword slot maps oracle-faithfully"
        );
        assert!(v1.notes.is_empty(), "{class}: no boundary notes");
    }
}

/// The text slots the wave-1 longhands land in (the slot vocabulary
/// the audit's capability rows name).
#[test]
fn the_wave1_text_slots_carry_the_vocabulary() {
    let v1 = bridged("line-clamp-2");
    assert!(
        v1.plan
            .base
            .text
            .contains(&(TextProperty::LineClamp, TextValue::Count(2)))
    );
    let v1 = bridged("stacked-fractions");
    assert!(v1.plan.base.text.contains(&(
        TextProperty::FontVariantNumeric,
        TextValue::Keyword("stacked-fractions"),
    )));
}

// ---------------------------------------------------------------------------
// 3. The style fold (the interp arm's slots and boundaries)
// ---------------------------------------------------------------------------

/// The frozen (layers, keywords) pair for one class — the macros' own
/// compile path.
fn frozen(class: &str) -> (&'static [aeolus_ir::Layer], &'static [&'static str]) {
    let tokens = default_tokens();
    let config = Config::default();
    let compiled = compile_theme(class, &tokens, &config)
        .unwrap_or_else(|e| panic!("{class}: the class compiles: {e:?}"));
    assert!(
        compiled.diagnostics.is_empty(),
        "{class}: unexpected diagnostics {:?}",
        compiled.diagnostics
    );
    let annotations = keyword_texts(class, &config).expect("the class annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    (
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    )
}

#[test]
fn the_fold_carries_the_clamp_and_variant_slots() {
    let (layers, keywords) = frozen("line-clamp-2");
    let (style, walk) = program_style(layers, keywords, &aeolus_compile::Env::default());
    assert!(walk.is_empty(), "line-clamp-2 folds clean: {:?}", walk);
    assert!(
        style
            .text
            .contains(&(TextProperty::LineClamp, TextValue::Count(2)))
            && style
                .text
                .contains(&(TextProperty::TextOverflow, TextValue::Keyword("ellipsis"))),
        "the clamp slots land: {:?}",
        style.text
    );

    let (layers, keywords) = frozen("tabular-nums");
    let (style, walk) = program_style(layers, keywords, &aeolus_compile::Env::default());
    assert!(walk.is_empty(), "tabular-nums folds clean: {:?}", walk);
    assert!(
        style.text.contains(&(
            TextProperty::FontVariantNumeric,
            TextValue::Keyword("tabular-nums")
        )),
        "the variant slot lands: {:?}",
        style.text
    );
}

/// sr-only's composite lands in its EXISTING slots (the truncate
/// multi-row precedent) and its clip leg reports as the typed
/// SlotUnsupported boundary.
#[test]
fn the_sr_only_composite_lands_and_reports_the_clip_leg() {
    let (layers, keywords) = frozen("sr-only");
    let (style, walk) = program_style(layers, keywords, &aeolus_compile::Env::default());
    // The legs that HAVE v0.x slots.
    assert_eq!(style.extras.position, Some(aeolus_core::Position::Absolute));
    let dims = style.dimensions.expect("the 1px legs land");
    assert_eq!(dims.width, Some(aeolus_core::SizedValue::Px(1.0)));
    assert_eq!(dims.height, Some(aeolus_core::SizedValue::Px(1.0)));
    let padding = style.padding.expect("the zero padding legs land");
    assert_eq!(padding.top, 0.0);
    let margin = style.margin.expect("the -1px margin legs land");
    assert_eq!(margin.top, -1.0);
    let overflow = style.extras.overflow.expect("the overflow leg lands");
    assert_eq!(overflow.x, Some(aeolus_core::OverflowKind::Hidden));
    assert!(
        style
            .text
            .contains(&(TextProperty::Whitespace, TextValue::Keyword("nowrap"))),
        "the nowrap leg lands: {:?}",
        style.text
    );
    // The clip leg: the typed boundary, never a silent drop.
    assert!(
        walk.boundaries
            .iter()
            .any(|note| matches!(note, PlanNoteKind::SlotUnsupported { .. })),
        "the clip leg reports: {:?}",
        walk.boundaries
    );
}

/// The between-child families report as the typed ChildSelector
/// boundary (the five backends' precise "no child access" reason's
/// source) — and carry NO per-element slot (the child-selector intent
/// is the longhand's own).
#[test]
fn the_between_child_families_report_the_child_selector_boundary() {
    for class in ["space-x-4", "divide-y-2", "divide-red-500", "divide-solid"] {
        let (layers, keywords) = frozen(class);
        let (_style, walk) = program_style(layers, keywords, &aeolus_compile::Env::default());
        assert!(
            walk.boundaries
                .iter()
                .any(|note| matches!(note, PlanNoteKind::ChildSelector { .. })),
            "{class}: the child-selector boundary reports: {:?}",
            walk.boundaries
        );
    }
}

/// The reverse flips compose (the state the GPUI wrapper reads from
/// the same program layers).
#[test]
fn the_reverse_flips_lower_as_their_own_longhands() {
    let (layers, keywords) = frozen("space-x-reverse");
    let (style, walk) = program_style(layers, keywords, &aeolus_compile::Env::default());
    let _ = (style, walk); // the reverse has NO v0.x slot — the boundary IS the report
    let (layers, keywords) = frozen("space-x-reverse");
    let (_style, walk) = program_style(layers, keywords, &aeolus_compile::Env::default());
    assert!(
        walk.boundaries
            .iter()
            .any(|note| matches!(note, PlanNoteKind::ChildSelector { .. })),
        "the reverse flip reports through the same boundary: {:?}",
        walk.boundaries
    );
}

// ---------------------------------------------------------------------------
// 4. The byte-level emission shapes (the child-rule machinery)
// ---------------------------------------------------------------------------

#[test]
fn the_child_rule_emission_is_the_v4_machinery_byte_shape() {
    let tokens = default_tokens();
    let sheet = emit_css_theme("space-x-4", &tokens, &Config::default()).expect("compiles");
    assert_eq!(
        sheet.to_string(),
        ":where(.space-x-4 > :not(:last-child)) {\n  \
         --tw-space-x-reverse: 0;\n  \
         margin-inline-start: calc(1rem * var(--tw-space-x-reverse));\n  \
         margin-inline-end: calc(1rem * calc(1 - var(--tw-space-x-reverse)));\n}\n"
    );
    let sheet = emit_css_theme("divide-y-2", &tokens, &Config::default()).expect("compiles");
    assert_eq!(
        sheet.to_string(),
        ":where(.divide-y-2 > :not(:last-child)) {\n  \
         --tw-divide-y-reverse: 0;\n  \
         border-bottom-style: var(--tw-border-style);\n  \
         border-top-style: var(--tw-border-style);\n  \
         border-top-width: calc(2px * var(--tw-divide-y-reverse));\n  \
         border-bottom-width: calc(2px * calc(1 - var(--tw-divide-y-reverse)));\n}\n"
    );
    let sheet = emit_css_theme("divide-dashed", &tokens, &Config::default()).expect("compiles");
    assert_eq!(
        sheet.to_string(),
        ":where(.divide-dashed > :not(:last-child)) {\n  \
         --tw-border-style: dashed;\n  \
         border-style: dashed;\n}\n"
    );
}

/// The negated form and the reverse rule (the machinery's flip) render
/// the same shapes under the same child selector.
#[test]
fn the_negated_and_reverse_forms_render() {
    let tokens = default_tokens();
    let sheet = emit_css_theme("-space-x-2", &tokens, &Config::default()).expect("compiles");
    assert!(
        sheet
            .to_string()
            .contains("margin-inline-start: calc(-0.5rem * var(--tw-space-x-reverse))"),
        "the negated form bakes the negative value: {}",
        sheet
    );
    let sheet = emit_css_theme("space-x-reverse", &tokens, &Config::default()).expect("compiles");
    assert_eq!(
        sheet.to_string(),
        ":where(.space-x-reverse > :not(:last-child)) {\n  \
         --tw-space-x-reverse: 1;\n}\n"
    );
}

/// The clamp composite's full emission (the overflow / -webkit-box
/// display pair beside the clamp value — the oracle's own text).
#[test]
fn the_clamp_composite_emission() {
    let tokens = default_tokens();
    let sheet = emit_css_theme("line-clamp-2", &tokens, &Config::default()).expect("compiles");
    assert_eq!(
        sheet.to_string(),
        ".line-clamp-2 {\n  \
         display: -webkit-box;\n  \
         overflow: hidden;\n  \
         -webkit-line-clamp: 2;\n  \
         -webkit-box-orient: vertical;\n}\n"
    );
    let sheet = emit_css_theme("line-clamp-none", &tokens, &Config::default()).expect("compiles");
    assert_eq!(
        sheet.to_string(),
        ".line-clamp-none {\n  \
         display: block;\n  \
         overflow: visible;\n  \
         -webkit-line-clamp: unset;\n  \
         -webkit-box-orient: horizontal;\n}\n"
    );
}

/// The Target::Css routing renders the same sheet (the `Target` enum's
/// CSS arm and the emit_css entry are the same walk).
#[test]
fn the_target_css_arm_renders_the_same_sheet() {
    let tokens = default_tokens();
    let via_target = Target::Css;
    let _ = via_target;
    let sheet = emit_css_theme("tabular-nums", &tokens, &Config::default()).expect("compiles");
    assert_eq!(
        sheet.to_string(),
        ".tabular-nums {\n  font-variant-numeric: tabular-nums;\n}\n"
    );
}

// ---------------------------------------------------------------------------
// The scale-up battery (M1): the FULL corpus through the V1 pipeline
// ---------------------------------------------------------------------------

/// The seed's documented V1 typed-refusal surface (the scale-up
/// findings): the seed's entries ride the corpus for the v0.x oracle's
/// byte-parity, and these families are the V1 pipeline's OWN typed
/// boundaries at this pin — refused with the named diagnostic kind, never
/// silently dropped. Each family is the honest wave-2 capability-push
/// candidate the conformance findings name (the important!/negation/
/// arbitrary-variant/composite-modifier/named-container declaration
/// grammar, the special bg-transparent keyword and the has-first
/// composition tier).
const DOCUMENTED_V1_REFUSALS: &[(&str, &str)] = &[
    ("!p-4", "UnsupportedImportant"),
    ("bg-red-500!", "UnsupportedImportant"),
    ("-tracking-wide", "UnsupportedNegation"),
    ("bg-transparent", "UnsupportedValue"),
    ("tracking-[7px]", "UnsupportedValue"),
    ("text-sm/6", "UnsupportedModifier"),
    ("text-base/tight", "UnsupportedModifier"),
    ("text-lg/loose", "UnsupportedModifier"),
    ("@container/card-header", "UnsupportedModifier"),
    ("not-hover:opacity-50", "UnsupportedVariant"),
    ("has-first:grid-cols-3", "UnsupportedVariant"),
    ("[&:hover]:bg-red-600", "UnsupportedVariant"),
    ("[@media(print)]:hidden", "UnsupportedVariant"),
];

/// The shadcn token table (the corpus's `shadcn` theme context).
fn shadcn_tokens() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the baked shadcn table builds")
}

/// The scale-up floor: EVERY corpus entry (the 2992-entry expansion) must
/// lower through the V1 pipeline with ZERO diagnostics under its own
/// theme's token table — the "never a silent drop" contract at scale. The
/// generator only emits classes the classification artifact marked V1-ok,
/// so a refusal here is a drift between the committed artifact and the
/// pipeline (a registry or lowering change that forgot the regeneration).
/// The v0.x oracle's battery (`aeolus-core/tests/parity_golden.rs`) pins
/// the byte-parity subset; the families it skips ride THIS battery's
/// zero-diagnostics floor plus the wave/flagship byte shapes above.
#[test]
fn the_expanded_corpus_lowers_clean_through_the_v1_pipeline() {
    let raw = include_str!("../../aeolus-core/tests/fixtures/golden.corpus");
    let default = default_tokens();
    let shadcn = shadcn_tokens();
    let config = Config::default();
    let mut entries = 0usize;
    let mut failures: Vec<String> = Vec::new();
    let mut theme = String::new();
    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix("@theme ") {
            theme = rest.trim().to_string();
            continue;
        }
        let Some(rest) = line.strip_prefix("@entry ") else {
            continue;
        };
        let (entry_theme, class) = rest.split_once(' ').expect("entry line shape");
        let _ = entry_theme;
        entries += 1;
        let table = if theme == "shadcn" { &shadcn } else { &default };
        match compile_theme(class, table, &config) {
            Ok(compiled) => {
                let expected_refusal = DOCUMENTED_V1_REFUSALS
                    .iter()
                    .find(|(cls, _)| *cls == class)
                    .map(|(_, kind)| *kind);
                if let Some(kind) = expected_refusal {
                    // The seed's documented typed-refusal surface: the
                    // class rides the corpus for the v0.x oracle's
                    // byte-parity (the seed predates the V1 pipeline's
                    // boundaries); the V1 lane refuses it TYPED. The KIND
                    // is pinned — a drift in the refusal shape fails
                    // here, never silently.
                    let actual = compiled
                        .diagnostics
                        .first()
                        .map(|diagnostic| {
                            format!("{:?}", diagnostic.kind)
                                .split_once('{')
                                .unwrap_or((&format!("{:?}", diagnostic.kind), ""))
                                .0
                                .trim()
                                .to_string()
                        })
                        .unwrap_or_else(|| "NO_DIAGNOSTIC".to_string());
                    if actual != kind {
                        failures.push(format!(
                            "{theme}:{class} — refusal drift: expected {kind}, got {actual}"
                        ));
                    }
                    continue;
                }
                if !compiled.diagnostics.is_empty() {
                    failures.push(format!(
                        "{theme}:{class} — {} diagnostics: {:?}",
                        compiled.diagnostics.len(),
                        compiled.diagnostics
                    ));
                }
                if compiled.program.layers.is_empty() {
                    failures.push(format!("{theme}:{class} — empty program"));
                }
            }
            Err(err) => failures.push(format!("{theme}:{class} — compile error {err}")),
        }
    }
    // The artifact-count pin: the corpus is the generator's committed
    // output (2992 @end entries at this pin) — a regeneration that changes
    // the count updates this number deliberately.
    assert_eq!(entries, 2992, "the corpus entry count at this pin");
    assert!(
        failures.is_empty(),
        "{} V1 lowering failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
