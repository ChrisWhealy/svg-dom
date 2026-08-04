//! Browser tests for the interactive HTML controls built inside `<foreignObject>`s.
//!
//! Each source module gets its own test file here.
//! `texts.rs` and `structure.rs` each hold one category's own tests.
//! `paint/` holds one file per demo, since `paint` itself already splits one file per demo.
//! `radio_group.rs` tests `foreign_html::radio_group` directly, not through a demo panel.
//! Three demo panels share that one helper: `demo_text`'s two groups, `demo_image`'s preserveAspectRatio group,
//! and `demo_radial_gradient`'s spreadMethod group.
//! A single direct test covers the helper's own behaviour once, instead of duplicating it across those three
//! panels.
//!
//! These tests exist for a specific reason.
//! `unit_tests::every_registered_demo_has_extractable_source` only proves one thing: every registered demo
//! function's Rust source is extractable.
//! It builds nothing and dispatches no events.
//! So it cannot catch a regression where selecting "Middle" stopped updating `text-anchor`.
//! It also cannot catch a regression where the startOffset slider's `max` started exceeding the guide arc's
//! real `total_length()` again.
//! Only a real browser can prove either case.
//! It needs a real DOM and a real wasm-bindgen event listener actually firing.
//!
//! Run via `wasm-pack test --headless --firefox demo-app` (or `--chrome`).
//! The package path must come *after* the flags.
//! `wasm-pack test demo-app --headless --firefox` puts the path first instead.
//! That form fails with "Must specify at least one of --node, --chrome, --firefox, or --safari".
//! This was confirmed empirically.
//! `wasm-pack build` behaves differently: `test`'s own `[PATH_AND_EXTRA_OPTIONS]...` argument swallows anything
//! placed after it, instead of parsing it as flags.
//! This is otherwise the same invocation shape the root `svg-dom` crate's own `tests/*.rs` use.
//! It is just scoped to this crate's directory instead of the repo root.
//! This module compiles as an ordinary `#[cfg(test)]` module.
//! It is not gated on `target_arch = "wasm32"`.
//! `#[wasm_bindgen_test]` itself already degrades safely on a native target.
//! It compiles cleanly there but registers zero tests.
//! This was confirmed empirically against the root crate's own `tests/svg_root.rs`.
//! So `cargo test -p svg-dom-demo` continues to exercise only `unit_tests`'s native tests.
//! It stays unaffected by this module's presence.
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);

mod paint;
mod radio_group;
mod structure;
mod texts;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn document() -> web_sys::Document {
    web_sys::window().expect("no window").document().expect("no document")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Creates a fresh `<div id="{id}">` appended to `<body>`, for `SvgRoot::create_in(id, ...)` to attach to.
/// `tests/common.rs::div` plays this same role for the root crate's own browser tests.
/// `demo_text`/`demo_text_path` hard-code their own container id internally (`"demo-text"`/`"demo-text-path"`).
/// So, unlike the root crate's tests, this cannot use a fresh id per test.
/// Each demo function is instead exercised by exactly one test.
/// That avoids the duplicate-id ambiguity a second call would create in `get_element_by_id`.
fn container(id: &str) -> web_sys::Element {
    let el = document().create_element("div").expect("create container div");
    el.set_id(id);
    document()
        .body()
        .expect("no body")
        .append_child(&el)
        .expect("append container to body");
    el
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Finds the `<input type="radio">` labelled `option_label` inside whichever `<fieldset>` has a `<legend>`
/// reading `group_legend`.
/// `option_label` is the label's own text content, for example `"middle"`.
/// `group_legend` is for example `"text-anchor"`.
/// This is the exact structure `demo_text`'s two radio groups both build.
/// Finding it this way, rather than by position, is itself part of what this test verifies.
/// It only succeeds if the fieldset/legend grouping and each radio's `<label>` association are both actually
/// present, not just the radios themselves.
fn find_radio(root: &web_sys::Element, group_legend: &str, option_label: &str) -> web_sys::HtmlInputElement {
    let fieldsets = root.query_selector_all("fieldset").expect("query fieldsets");
    for i in 0..fieldsets.length() {
        let fieldset = fieldsets
            .item(i)
            .expect("fieldset item")
            .dyn_into::<web_sys::Element>()
            .expect("fieldset is an Element");
        let legend_text = fieldset
            .query_selector("legend")
            .expect("query legend")
            .and_then(|l| l.text_content());
        if legend_text.as_deref() != Some(group_legend) {
            continue;
        }

        let labels = fieldset.query_selector_all("label").expect("query labels");
        for j in 0..labels.length() {
            let label = labels
                .item(j)
                .expect("label item")
                .dyn_into::<web_sys::Element>()
                .expect("label is an Element");
            if label.text_content().as_deref().map(str::trim) == Some(option_label) {
                return label
                    .query_selector("input")
                    .expect("query input")
                    .expect("radio input inside label")
                    .dyn_into::<web_sys::HtmlInputElement>()
                    .expect("input is an HtmlInputElement");
            }
        }
    }
    panic!("no radio found for group {group_legend:?}, option {option_label:?}");
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Selects a radio the way a real click would.
/// It sets `checked` first, then dispatches `change`.
/// A synthetic `dispatch_event` does not perform the browser's own default action, toggling `checked`, the way
/// an actual click does.
/// So that has to be done by hand.
/// The slider tests apply this same principle to `value`.
fn select_radio(radio: &web_sys::HtmlInputElement) {
    radio.set_checked(true);
    let event = web_sys::Event::new("change").expect("create change event");
    radio.dispatch_event(&event).expect("dispatch change");
}
