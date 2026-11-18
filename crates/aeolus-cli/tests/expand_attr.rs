//! The attribute-probe families' CLI battery (the expand face's flag
//! pins): each `--data name[=value]` / `--aria name[=value]` fact
//! asserts one attribute bit the interp's `AttrProbe` seam answers the
//! `data-*`/`aria-*` layers from — a hit FIRES the layer, a
//! source-answered miss is the CSS-quiet skip (a selector that does
//! not match), and the sourceless forms stay the reported boundary
//! (the `AttributeUnqueried` note). Written FIRST (the RED spec): the
//! suite fails while the flag surface carries no attribute arms.

use aeolus_cli::run;

/// The expand face's effective-declaration lines for one run (exit 0
/// expected — the honest-refusal pins drive `run` directly).
fn expand_lines(args: &[&str]) -> Vec<String> {
    let (code, lines, err) = expand_raw(args);
    assert_eq!(code, 0, "expand exits clean: {err:?}");
    lines
}

/// The expand face's raw run: `(exit code, stdout lines, stderr text)`
/// — the honest-refusal pins read all three channels.
fn expand_raw(args: &[&str]) -> (i32, Vec<String>, String) {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let owned: Vec<String> = args.iter().map(std::string::ToString::to_string).collect();
    let code = run(owned, &mut out, &mut err);
    let lines = String::from_utf8(out)
        .expect("stdout is utf-8")
        .lines()
        .map(str::to_string)
        .collect();
    let err = String::from_utf8(err).expect("stderr is utf-8");
    (code, lines, err)
}

#[test]
fn the_presence_fact_fires_the_bare_data_layer() {
    // The bare `data-state:` spelling is the presence test: the fact
    // table answers the attribute exists, the layer fires.
    let lines = expand_lines(&["expand", "--data", "state=open", "p-4", "data-state:p-8"]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("padding-top = 2rem"),
        "the presence fact answers the bare data layer: {joined}"
    );
}

#[test]
fn the_value_fact_fires_the_bracketed_layer() {
    // The bracketed spelling is the value test: the fact's value
    // matches the folded payload, the layer fires.
    let lines = expand_lines(&[
        "expand",
        "--data",
        "state=open",
        "p-4",
        "data-[state=open]:p-8",
    ]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("padding-top = 2rem"),
        "the value fact answers the bracketed layer: {joined}"
    );
}

#[test]
fn the_source_answered_miss_is_the_quiet_skip() {
    // A source rides (the fact table) and answers "no": a CSS selector
    // that does not match — the layer never fires and NO note rides
    // (the sourceless boundary is the reported one, not this).
    let (code, lines, err) = expand_raw(&[
        "expand",
        "--data",
        "state=other",
        "p-4",
        "data-[state=open]:p-8",
    ]);
    assert_eq!(code, 0, "the answered miss still exits clean: {err:?}");
    let joined = lines.join("\n");
    assert!(
        joined.contains("padding-top = 1rem"),
        "the base declaration answers; the value layer waits: {joined}"
    );
    assert!(
        !joined.contains("note:"),
        "an answered miss is CSS-quiet, never a reported exclusion: {joined}"
    );
}

#[test]
fn the_presence_fact_with_another_name_misses() {
    // The fact table carries a DIFFERENT attribute: the probed one is
    // absent, the layer stays the quiet miss.
    let lines = expand_lines(&["expand", "--data", "other=open", "p-4", "data-state:p-8"]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("padding-top = 1rem"),
        "the probed attribute is absent, the layer waits: {joined}"
    );
}

#[test]
fn the_bare_aria_fact_asserts_the_true_reading() {
    // The bare `aria-*:` spelling names the "true" value (the fold's
    // own reading — the corpus renders `[aria-busy="true"]`), so the
    // bare CLI fact asserts the bit WITH "true" and the layer fires.
    let lines = expand_lines(&["expand", "--aria", "busy", "p-4", "aria-busy:p-8"]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("padding-top = 2rem"),
        "the bare aria fact answers the bare spelling: {joined}"
    );
}

#[test]
fn the_aria_value_spelling_splits_the_family() {
    // The bracketed aria spelling compares values: the matching fact
    // fires, the mismatching one stays the quiet miss.
    let lines = expand_lines(&["expand", "--aria", "sort=asc", "p-4", "aria-[sort=asc]:p-8"]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("padding-top = 2rem"),
        "the matching aria value fires: {joined}"
    );
    let lines = expand_lines(&[
        "expand",
        "--aria",
        "sort=desc",
        "p-4",
        "aria-[sort=asc]:p-8",
    ]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("padding-top = 1rem"),
        "the mismatching aria value stays the quiet miss: {joined}"
    );
    assert!(
        !joined.contains("note:"),
        "an answered miss is CSS-quiet: {joined}"
    );
}

#[test]
fn the_sourceless_boundary_stays_the_reported_note() {
    // No attribute facts ride: the plain walk carries no AttrProbe
    // source, the layers stay the reported `AttributeUnqueried`
    // exclusion — never a silent drop.
    let lines = expand_lines(&["expand", "p-4", "data-state:p-8", "aria-busy:p-2"]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("padding-top = 1rem"),
        "the base declaration answers; the attribute layers wait: {joined}"
    );
    assert!(
        joined.contains("attribute/DOM probes"),
        "the sourceless boundary is the reported note: {joined}"
    );
}

#[test]
fn the_facts_ride_the_themed_walk() {
    // The themed dispatch doubles for the attribute seam too: the
    // token form resolves through the table, the fired layer lands
    // with its resolved colour (the hex render).
    let lines = expand_lines(&[
        "expand",
        "--theme",
        "shadcn",
        "--data",
        "state=open",
        "p-4",
        "data-state:bg-primary",
    ]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("background-color = #"),
        "the themed walk resolves the token on the fired layer: {joined}"
    );
}

#[test]
fn the_facts_compose_with_the_other_probe_families() {
    // The dispatch matrix composes: attribute facts AND descendant
    // facts ride ONE walk (the widest `*_probes_descendants` form),
    // both layers answer.
    let lines = expand_lines(&[
        "expand",
        "--data",
        "state=open",
        "--has",
        "element=*:focus",
        "p-4",
        "data-state:p-8",
        "has-focus:mt-2",
    ]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("padding-top = 2rem"),
        "the attribute layer answers: {joined}"
    );
    assert!(
        joined.contains("margin-top = 0.5rem"),
        "the descendant layer answers beside it: {joined}"
    );
}

#[test]
fn the_fact_table_composes_across_families() {
    // Repeatable facts build ONE table across both families: a data
    // row and an aria row each answer their own layer.
    let lines = expand_lines(&[
        "expand",
        "--data",
        "kind=card",
        "--aria",
        "busy",
        "p-4",
        "data-[kind=card]:mt-4",
        "aria-busy:mb-4",
    ]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("margin-top = 1rem"),
        "the data row answers its layer: {joined}"
    );
    assert!(
        joined.contains("margin-bottom = 1rem"),
        "the aria row answers its layer: {joined}"
    );
}

#[test]
fn the_json_shape_rides_the_facts() {
    let lines = expand_lines(&[
        "expand",
        "--json",
        "--data",
        "state=open",
        "p-4",
        "data-state:p-8",
    ]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("\"prop\":\"padding-top\",\"value\":\"2rem\""),
        "the JSON object carries the fired layer: {joined}"
    );
}

#[test]
fn every_attr_fact_spelling_parses() {
    // The closed fact grammar: bare presence and name=value across
    // both families parse and the base answers beside them.
    for flag in [
        ["--data", "state"],
        ["--data", "state=open"],
        ["--aria", "busy"],
        ["--aria", "busy=true"],
    ] {
        let lines = expand_lines(&["expand", flag[0], flag[1], "p-4"]);
        assert!(
            lines.iter().any(|l| l.contains("padding-top = 1rem")),
            "{} {} parses and the base answers: {lines:?}",
            flag[0],
            flag[1]
        );
    }
}

#[test]
fn the_honest_refusal_set_reports_usage_errors() {
    // The fold's own grammar, refused at the CLI face: an empty name,
    // an empty value, a non-attribute name character, a quote in the
    // value, a missing value — each a usage error (exit 2), never a
    // half-answered layer.
    for args in [
        vec!["expand", "--data", "=open", "p-4"],
        vec!["expand", "--data", "state=", "p-4"],
        vec!["expand", "--data", "st.ate=x", "p-4"],
        vec!["expand", "--data", "state=o\"pen", "p-4"],
        vec!["expand", "--aria"],
    ] {
        let (code, _lines, err) = expand_raw(&args);
        assert_eq!(code, 2, "the malformed fact is a usage error: {err:?}");
        assert!(
            err.contains("--data") || err.contains("--aria"),
            "the refusal names the flag: {err:?}"
        );
    }
}
