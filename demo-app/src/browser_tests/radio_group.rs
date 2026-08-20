//! Tests for `foreign_html::radio_group` itself, called directly rather than through a demo panel.

use crate::browser_tests::{container, document, find_radio, select_radio};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen_test::wasm_bindgen_test;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// This test calls `foreign_html::radio_group` directly, with no demo panel and no SVG target attribute.
// Three demos build a radio group through this one shared function.
// A single direct test at this level covers all three, instead of testing the same mechanics three times.
#[wasm_bindgen_test]
fn radio_group_checks_default_clears_prior_selection_and_calls_on_select() -> Result<(), String> {
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
        .map_err(|e| format!("radio_group should build without error: {e:?}"))?;
    root.append_child(&fieldset)
        .map_err(|e| format!("append fieldset to container: {e:?}"))?;

    let alpha = find_radio(&root, "demo-legend", "Alpha")?;
    let beta = find_radio(&root, "demo-legend", "Beta")?;

    // The default value starts checked. Asserted before any interaction, so a later mismatch can only come
    // from the selection itself, not from construction.
    if !alpha.checked() {
        return Err("the default option should start checked".to_owned());
    }
    if beta.checked() {
        return Err("Beta should not start checked".to_owned());
    }
    if !selected.borrow().is_empty() {
        return Err("on_select should not fire before any selection".to_owned());
    }

    select_radio(&beta)?;
    if !beta.checked() {
        return Err("Beta should be checked after selecting it".to_owned());
    }
    if alpha.checked() {
        return Err("selecting Beta should clear Alpha".to_owned());
    }
    if selected.borrow().as_slice() != ["b"] {
        return Err(format!(
            "on_select should receive Beta's own value, got {:?}",
            selected.borrow()
        ));
    }
    Ok(())
}
