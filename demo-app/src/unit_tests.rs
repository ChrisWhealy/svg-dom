use super::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `demo_fn_source`'s textual extraction out of `DEMO_SRC` is a separate mechanism from `DEMO_PANELS` itself —
/// compilation only proves `$module::$func` in `demo_gallery!` resolves to a real function, not that
/// `demo_fn_source` can find its source text. A function that compiles fine but breaks the extraction (made `pub`,
/// marked `async`, moved into a file `DEMO_SRC` doesn't `include_str!`, or otherwise reformatted) would still pass
/// `cargo build` and `demo-server`'s startup `validate`, and only surface as `append_source_frame` silently
/// skipping the source frame — see that function's own doc comment for why it fails silently rather than erroring.
/// This test is what actually catches that class of drift, on every `cargo test` run rather than only when someone
/// happens to look at a panel in a browser.
#[test]
fn every_registered_demo_has_extractable_source() {
    for (_, _, fn_name) in DEMO_PANELS {
        assert!(demo_fn_source(fn_name).is_some(), "source not found for {fn_name}");
    }
}
