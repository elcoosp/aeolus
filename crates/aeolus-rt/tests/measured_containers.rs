//! The element-bounds half of the named-container seam (architecture
//! plan §3.5's `@container` row, the "who MEASURES the widths" half the
//! docs keep assigned to the host): [`aeolus_rt::MeasuredContainers`]
//! is the layout-phase hand-off the measuring wrapper publishes into
//! and every walk reads through the ONE [`aeolus_rt::ContainerProbe`]
//! question.
//!
//! The pins:
//! * a published width answers its name through the seam — the
//!   `ContainerProbe` impl is the handle, not a separate table;
//! * a re-publish overwrites (the resize story: the same name carries
//!   the NEW width, the walk's verdicts follow);
//! * a name never published is the CSS-quiet miss (`None`), never an
//!   exclusion;
//! * the handle clones share ONE state (the wrapper publishes through
//!   its clone, the descendant's clone reads the same answer);
//! * the measured answers drive the walk EXACTLY like the built-in
//!   `ContainerState` table does — the differential pin rides
//!   `interp_containers` beside the handle;
//! * the AT/strictly-below boundary rules are the viewport
//!   breakpoints' own, shared verbatim through the seam.

use aeolus_compile::{Env, compile, interp_containers};
use aeolus_ir::Val;
use aeolus_rt::{ContainerBound, ContainerProbe, ContainerState, MeasuredContainers};

#[test]
fn published_width_answers_through_the_seam() {
    let measured = MeasuredContainers::new();
    measured.publish("card", 48.0);
    assert_eq!(
        measured.container("card"),
        Some(48.0),
        "the published rem answers the ONE seam question"
    );
}

#[test]
fn republish_overwrites_the_same_name() {
    let measured = MeasuredContainers::new();
    measured.publish("sidebar", 20.0);
    measured.publish("sidebar", 65.0);
    assert_eq!(
        measured.container("sidebar"),
        Some(65.0),
        "the resize story: the same name carries the NEW width"
    );
}

#[test]
fn unpublished_name_is_the_quiet_miss() {
    let measured = MeasuredContainers::new();
    assert_eq!(
        measured.container("card"),
        None,
        "a name never published is the CSS selector that does not match"
    );
}

#[test]
fn clones_share_one_state() {
    let measured = MeasuredContainers::new();
    let writer = measured.clone();
    let reader = measured.clone();
    writer.publish("card", 48.0);
    assert_eq!(
        reader.container("card"),
        Some(48.0),
        "the wrapper's clone publishes, the descendant's clone reads the same answer"
    );
}

#[test]
fn measured_answers_drive_the_interp_exactly_like_the_built_in_table() {
    // The differential pin: the SAME published rows, answered through
    // the measured handle and through the borrowed static table, give
    // the SAME interp verdicts — the handle is the probe, not a second
    // semantics.
    let program_source = "@md/card:p-4 @max-lg/sidebar:hidden";
    let compiled = compile(program_source, &Default::default()).expect("compiles");
    let program = &compiled.program;

    let rows = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let built_in = ContainerState::new(&rows);
    let measured = MeasuredContainers::new();
    measured.publish("card", 48.0);

    let from_state = interp_containers(program, &Env::default(), &built_in).expect("interp");
    let from_measured = interp_containers(program, &Env::default(), &measured).expect("interp");
    assert_eq!(
        from_state.decls, from_measured.decls,
        "the measured handle answers exactly what the static table answers"
    );
    assert_eq!(from_state.notes, from_measured.notes);
}

#[test]
fn boundary_rules_shared_verbatim_through_the_seam() {
    // `md` is 48rem: AT the boundary fires (the min-width rule), one
    // pixel below misses — the viewport breakpoints' own semantics,
    // carried by the measured answers unchanged.
    let compiled = compile("@md/card:p-4", &Default::default()).expect("compiles");
    let program = &compiled.program;

    let at = MeasuredContainers::new();
    at.publish("card", 48.0);
    let fired = interp_containers(program, &Env::default(), &at).expect("interp");
    assert!(
        fired.decls.iter().any(|d| d.val == Val::Rem(1.0)),
        "AT the boundary fires: {:?}",
        fired.decls
    );

    let below = MeasuredContainers::new();
    below.publish("card", 47.9);
    let missed = interp_containers(program, &Env::default(), &below).expect("interp");
    assert!(
        missed.decls.is_empty(),
        "below the boundary is the quiet miss: {:?}",
        missed.decls
    );
}
