//! Consumer demonstration: the composition tier's CLI face — the
//! `expand --group-*`/`--peer-*` flags assert the parent-relative
//! frame facts headless (a `.group` ancestor / `.peer` sibling in the
//! composed state): `group-active:p-8` fires only while
//! `--group-active` rides, the unset fact the QUIET miss a frame the
//! composition does not hold in (the media facts' own reading).
//!
//! ```sh
//! cargo run -p aeolus-cli --example expand_group_peer
//! ```

fn main() {
    let mut args = vec![
        "expand".to_string(),
        "--group-active".to_string(),
        "--peer-disabled".to_string(),
        "p-4 group-active:p-8 peer-disabled:p-2".to_string(),
    ];
    let mut out = std::io::stdout();
    let mut err = std::io::stderr();
    let code = aeolus_cli::run(args.clone(), &mut out, &mut err);
    assert_eq!(code, 0, "the fired facts exit clean");
    println!();
    println!("── the resting face: no composition facts asserted ──");
    args.truncate(1);

    args.push("p-4 group-active:p-8 peer-disabled:p-2".to_string());
    let code = aeolus_cli::run(args, &mut std::io::stdout(), &mut std::io::stderr());
    assert_eq!(code, 0, "the quiet miss exits clean too");
}
