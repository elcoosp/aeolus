use aeolus_gpui::prelude::*;
use gpui::rems;

#[test]
fn tw_compiles_to_a_static_program() {
    let program: StaticProgram = tw!("p-4 bg-primary text-sm font-bold");
    let base = &program.layers[0];
    assert!(!base.decls.is_empty());
    assert!(program.layers.iter().any(|layer| !layer.decls.is_empty()));
}

#[test]
fn tw_applies_to_style_refinement() {
    let program = tw!("p-4 bg-primary");
    let style = gpui::StyleRefinement::default().tw(program);
    assert!(style.padding.top.is_some());
    assert_eq!(style.padding.top.unwrap(), rems(1.0).into());
}

#[test]
fn tw_state_compiles() {
    let program = tw!("hover:bg-primary/90 focus:ring-2");
    assert!(program.layers.iter().any(|layer| !layer.decls.is_empty()));
}

#[test]
fn cn_merges_at_compile_time() {
    let program = cn!("p-4", "px-8");
    let base = &program.layers[0];
    let width_decls = base
        .decls
        .iter()
        .filter(|decl| decl.prop == aeolus_ir::Prop::PaddingLeft)
        .count();
    assert_eq!(width_decls, 1);
    let style = gpui::StyleRefinement::default().tw(program);
    assert_eq!(style.padding.left, Some(rems(2.0).into()));
    assert_eq!(style.padding.top, Some(rems(1.0).into()));
}

#[test]
fn unsupported_keyword_value_is_not_a_program() {
    let program = tw!("contents");
    let style = gpui::StyleRefinement::default().tw(program);
    assert!(style.display.is_some());
}

variants! { pub Button {
    base:"inline-flex items-center gap-2 rounded-md font-medium",
    variant: {
        default:"bg-primary text-primary-foreground",
        destructive:"bg-destructive text-white",
    },
    size: { default:"h-9 px-4 py-2", sm:"h-8 px-3"},
} }

#[test]
fn variants_compile_a_closed_world_table() {
    let program = Button::style(ButtonVariant::Default, ButtonSize::Sm);
    let style = gpui::StyleRefinement::default().tw(program);
    assert_eq!(style.padding.left, Some(rems(0.75).into()));
    assert!(style.background.is_some());
}
