//! Tests for `foreign_html::radio_group` itself, called directly rather than through a demo panel.

use crate::browser_tests::{container, document, find_radio, select_radio};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen_test::wasm_bindgen_test;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// This test calls `foreign_html::radio_group` directly, with no demo panel and no SVG target attribute.
// Three demos build a radio group through this one shared function.
// A single direct test at this level covers all three, instead of testing the same mechanics three times.
#[wasm_bindgen_test]
fn radio_group_checks_default_clears_prior_selection_and_calls_on_select() {
    let root = container("foreign-html-radio-group");

    const OPTIONS: [(&str, &str); 2] = [("a", "Alpha"), ("b", "Beta")];

    // `on_select` only records the values it receives here. No SVG attribute is involved, unlike every other
    // test in this file. That keeps this test focused on radio_group's own contract, not on any one caller.
    let selected: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = selected.clone();
    let fieldset =
        crate::foreign_html::radio_group(&document(), "demo-legend", "radio-group-test", &OPTIONS, "a", move |value| {
            recorder.borrow_mut().push(value)
        })
        .expect("radio_group should build without error");
    root.append_child(&fieldset).expect("append fieldset to container");

    let alpha = find_radio(&root, "demo-legend", "Alpha");
    let beta = find_radio(&root, "demo-legend", "Beta");

    // The default value starts checked. Asserted before any interaction, so a later mismatch can only come
    // from the selection itself, not from construction.
    assert!(alpha.checked(), "the default option should start checked");
    assert!(!beta.checked());
    assert!(selected.borrow().is_empty(), "on_select should not fire before any selection");

    select_radio(&beta);
    assert!(beta.checked());
    assert!(!alpha.checked(), "selecting Beta should clear Alpha");
    assert_eq!(selected.borrow().as_slice(), ["b"], "on_select should receive Beta's own value");
}
