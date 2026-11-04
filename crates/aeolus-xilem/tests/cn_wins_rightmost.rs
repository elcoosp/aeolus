//! Proof test (c): `cn()` conflict resolution — rightmost wins — then the
//! resolver, then the lowering, agree on the final padding.

use aeolus_core::{Resolver, cn, shadcn_default};
use aeolus_xilem::layer_get;
use masonry::layout::Length;
use masonry::properties::Padding;

#[test]
fn cn_wins_rightmost() {
    // tw_merge semantics: `px-4` (later) knocks out `px-2` (earlier).
    let merged = cn(&["px-2", "px-4"]);
    assert_eq!(merged, "px-4");

    // The merged string resolves to horizontal padding 16px (`px-4` =
    // 4 × 0.25rem = 1rem = 16px at 16px/rem); vertical edges unset (0).
    let theme = shadcn_default().unwrap();
    let resolver = Resolver::new(&theme);
    let resolution = resolver.resolve(&merged).unwrap();
    let p = resolution.plan.base.padding.expect("px-4 sets padding");
    assert_eq!(
        (p.top, p.right, p.bottom, p.left),
        (0.0, 16.0, 0.0, 16.0),
        "rightmost px-4 wins: 16px horizontal, 0 vertical"
    );

    // And the lowering carries exactly that into masonry's Padding property
    // (which has per-side fields — the mapping is exact).
    let (stack, report) = aeolus_xilem::lower(&resolution.plan);
    assert!(report.is_empty(), "px-4 must fully map: {report:?}");
    let padding = layer_get::<Padding>(&stack, 0).expect("padding in base layer");
    assert_eq!(padding.left, Length::px(16.0));
    assert_eq!(padding.right, Length::px(16.0));
    assert_eq!(padding.top, Length::ZERO);
    assert_eq!(padding.bottom, Length::ZERO);
}
