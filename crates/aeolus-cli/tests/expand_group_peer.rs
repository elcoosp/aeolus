//! The composition tier's CLI battery (the expand face's flag pins):
//! each `--group-*`/`--peer-*` fact flag asserts its frame fact and
//! the fired layer lands in the effective declarations; the unset
//! fact stays the quiet miss. Written FIRST (the RED spec): the suite
//! fails while the flag surface carries no composition arms.

use aeolus_cli::run;

/// The expand face's effective-declaration lines for one run.
fn expand_lines(args: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let owned: Vec<String> = args.iter().map(std::string::ToString::to_string).collect();
    let code = run(owned, &mut out, &mut err);
    assert_eq!(code, 0, "expand exits clean: {err:?}");
    String::from_utf8(out)
        .expect("stdout is utf-8")
        .lines()
        .map(str::to_string)
        .collect()
}

#[test]
fn the_group_composition_flag_asserts_its_fact() {
    let lines = expand_lines(&["expand", "--group-active", "p-4 group-active:p-8"]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("padding-top = 2rem"),
        "the group-active layer answers the asserted fact: {joined}"
    );
}

#[test]
fn the_peer_composition_flag_asserts_its_fact() {
    let lines = expand_lines(&["expand", "--peer-disabled", "p-4 peer-disabled:p-2"]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("padding-top = 0.5rem"),
        "the peer-disabled layer answers the asserted fact: {joined}"
    );
}

#[test]
fn the_unset_facts_stay_the_quiet_miss() {
    let lines = expand_lines(&["expand", "p-4 group-active:p-8 peer-disabled:p-2"]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("padding-top = 1rem"),
        "the base declaration answers; the composition layers wait: {joined}"
    );
    assert!(
        !joined.contains("padding-top = 2rem") && !joined.contains("padding-top = 0.5rem"),
        "no composition layer fired on the resting face: {joined}"
    );
}

#[test]
fn the_founding_composition_flags_assert_their_facts() {
    // The four founding members join the tier: hover/focus on both
    // relations — the facts the composition family's own first four
    // conds ride (the `--group-active` precedent).
    for (flag, class, expected) in [
        ("--group-hover", "p-4 group-hover:p-8", "padding-top = 2rem"),
        ("--group-focus", "p-4 group-focus:p-8", "padding-top = 2rem"),
        ("--peer-hover", "p-4 peer-hover:p-2", "padding-top = 0.5rem"),
        ("--peer-focus", "p-4 peer-focus:p-2", "padding-top = 0.5rem"),
    ] {
        let lines = expand_lines(&["expand", flag, class]);
        let joined = lines.join("\n");
        assert!(
            joined.contains(expected),
            "{flag} answers the asserted fact: {joined}"
        );
    }
}

#[test]
fn the_founding_flags_unset_stay_the_quiet_miss() {
    let lines = expand_lines(&["expand", "p-4 group-hover:p-8 peer-focus:p-2"]);
    let joined = lines.join("\n");
    assert!(
        joined.contains("padding-top = 1rem"),
        "the base declaration answers; the founding layers wait: {joined}"
    );
    assert!(
        !joined.contains("padding-top = 2rem") && !joined.contains("padding-top = 0.5rem"),
        "no founding layer fired on the resting face: {joined}"
    );
}

#[test]
fn every_composition_flag_parses() {
    // The closed flag set: each spelling parses (the fold's own set).
    let flags = [
        "--group-focus-visible",
        "--group-focus-within",
        "--group-active",
        "--group-disabled",
        "--group-visited",
        "--group-checked",
        "--peer-focus-visible",
        "--peer-focus-within",
        "--peer-active",
        "--peer-disabled",
        "--peer-visited",
        "--peer-checked",
        "--group-hover",
        "--group-focus",
        "--peer-hover",
        "--peer-focus",
    ];
    for flag in flags {
        let lines = expand_lines(&["expand", flag, "p-4"]);
        assert!(
            lines.iter().any(|l| l.contains("padding-top = 1rem")),
            "{flag} parses and the base answers: {lines:?}"
        );
    }
}
