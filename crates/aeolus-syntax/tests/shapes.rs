//! Candidate-shape tests: the slices, spans and error positions the
//! diagnostics surfaces build on. These pin the LEXER's own contract
//! (the differential suite pins the grammar agreement with the oracle).

use aeolus_syntax::{LexErrorKind, parse_candidate, parse_class_string};

#[test]
fn candidate_slices_borrow_the_raw_token() {
    let raw = "dark:group-hover/menu:bg-primary/90!";
    let c = parse_candidate(raw).expect("parses");
    assert_eq!(c.raw, raw);
    assert_eq!(c.span, (0, raw.len()));
    // Variant segments are raw slices.
    assert_eq!(c.variants, vec!["dark", "group-hover/menu"]);
    // Body strips variants, sign, modifier and brackets.
    assert_eq!(c.body, "bg-primary");
    // The modifier is RAW (no opacity interpretation here).
    assert_eq!(c.modifier, Some("90"));
    assert!(c.important);
    assert!(!c.negative);
    assert!(c.arbitrary.is_none());
    // Every borrowed slice points into the same buffer.
    let base = c.raw.as_ptr();
    for v in &c.variants {
        let ptr = v.as_ptr();
        assert!(
            ptr >= base && (unsafe { ptr.add(v.len()) } <= unsafe { base.add(raw.len()) }),
            "variant slice {v:?} must alias the input"
        );
    }
}

#[test]
fn arbitrary_carry_hint_form_and_property_flag() {
    let c = parse_candidate("w-[length:200px]").expect("parses");
    let a = c.arbitrary.expect("arbitrary");
    assert_eq!(a.hint, Some("length"));
    assert_eq!(a.value, "200px");
    assert_eq!(a.form, aeolus_syntax::Form::Bracket);
    assert!(!a.is_property);

    let c = parse_candidate("bg-(--my-var)").expect("parses");
    let a = c.arbitrary.expect("arbitrary");
    assert_eq!(a.form, aeolus_syntax::Form::Paren);
    assert_eq!(a.value, "--my-var");
    assert_eq!(c.body, "bg");

    // A bare property bracket has no utility head — the oracle rejects
    // it (per-class honest failure); the property FLAG is testable on a
    // headed form.
    let e = parse_candidate("[mask-type:luminance]").expect_err("no utility head");
    assert_eq!(e.kind, LexErrorKind::EmptyHead);
    let c = parse_candidate("bg-[mask-type:luminance]").expect("parses");
    let a = c.arbitrary.expect("arbitrary");
    assert!(a.is_property, "unknown ':' hint = property form");

    let c = parse_candidate("text-[color:var(--brand)]").expect("parses");
    let a = c.arbitrary.expect("arbitrary");
    assert_eq!(a.hint, Some("color"));
    assert!(!a.is_property);
}

#[test]
fn negation_and_importance_take_every_spelling() {
    let c = parse_candidate("-mt-4").expect("parses");
    assert!(c.negative);
    assert_eq!(c.body, "mt-4");

    let c = parse_candidate("!p-4").expect("parses");
    assert!(c.important);
    let c = parse_candidate("p-4!").expect("parses");
    assert!(c.important);
    assert_eq!(c.body, "p-4");
}

#[test]
fn errors_carry_spans_and_stable_kinds() {
    let e = parse_candidate(":p-4").expect_err("empty variant");
    assert_eq!(e.kind, LexErrorKind::EmptyVariant);

    let e = parse_candidate("hover:").expect_err("empty head");
    assert_eq!(e.kind, LexErrorKind::EmptyHead);

    let e = parse_candidate("p--4").expect_err("double dash");
    assert_eq!(e.kind, LexErrorKind::EmptyElement);

    let e = parse_candidate("w-[").expect_err("unbalanced");
    assert_eq!(e.kind, LexErrorKind::UnbalancedGroup);

    let e = parse_candidate("w-[]").expect_err("empty arbitrary");
    assert_eq!(e.kind, LexErrorKind::EmptyArbitrary);

    let e = parse_candidate("w-[4px]x").expect_err("text after group");
    assert_eq!(e.kind, LexErrorKind::TextAfterGroup);

    // Messages mirror the oracle's wording family.
    assert_eq!(e.message(), "unexpected text after arbitrary group");
    // Display includes the span.
    assert!(e.to_string().contains("0.."));
}

#[test]
fn per_class_failures_never_take_down_the_string() {
    let input = "p-4 w-[ not-a-utility hover:p-4";
    let (tokens, errors) = parse_class_string(input);
    // `not-a-utility` LEXES (dash-split elements, no registry here) —
    // classification is the registry pipeline's job, not the lexer's.
    assert_eq!(tokens.len(), 3, "p-4, not-a-utility, hover:p-4 survive");
    assert_eq!(tokens[0].body, "p-4");
    assert_eq!(tokens[1].body, "not-a-utility");
    assert_eq!(tokens[2].body, "p-4");
    assert_eq!(tokens[2].variants, vec!["hover"]);
    assert_eq!(errors.len(), 1, "w-[ reports");
    assert!(errors[0].0.starts_with("w-["), "the raw rides the error");
}

#[test]
fn variant_prefix_is_the_merge_key() {
    let c = parse_candidate("dark:hover:p-4").expect("parses");
    assert_eq!(c.variant_prefix(), "dark:hover");
    let c = parse_candidate("p-4").expect("parses");
    assert_eq!(c.variant_prefix(), "");
    // Grouped labels and arbitrary selectors stay verbatim.
    let c = parse_candidate("group-hover/menu:[&:focus]:p-4").expect("parses");
    assert_eq!(c.variant_prefix(), "group-hover/menu:[&:focus]");
}

#[test]
fn span_offsets_skip_whitespace_exactly() {
    let input = "a b";
    let (tokens, _) = parse_class_string(input);
    assert_eq!(tokens[0].span, (0, 1));
    assert_eq!(tokens[1].span, (2, 3));
    // Multi-space, tabs, newlines: spans always land on the raw bytes.
    let input = "\n\n  md:p-6\t\tbg-none  ";
    let (tokens, _) = parse_class_string(input);
    assert_eq!(tokens.len(), 2);
    assert_eq!(&input[tokens[0].span.0..tokens[0].span.1], "md:p-6");
    assert_eq!(&input[tokens[1].span.0..tokens[1].span.1], "bg-none");
}
