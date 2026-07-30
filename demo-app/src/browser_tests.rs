//! Browser tests for the interactive HTML controls `texts.rs` builds inside `<foreignObject>`s (the
//! text-anchor/dominant-baseline radio groups in `demo_text`, and the startOffset slider in `demo_text_path`).
//!
//! These exist because `unit_tests::every_registered_demo_has_extractable_source` only proves every registered
//! demo function's Rust *source* is extractable — it builds nothing and dispatches no events, so it cannot catch
//! a regression where, say, selecting "Middle" stopped updating `text-anchor`, or the startOffset slider's `max`
//! started exceeding the guide arc's real `total_length()` again. Only a real browser, with a real DOM and a real
//! wasm-bindgen event listener actually firing, can prove that.
//!
//! Run via `wasm-pack test --headless --firefox demo-app` (or `--chrome`) — note the package path comes *after*
//! the flags: `wasm-pack test demo-app --headless --firefox` (path first) fails with "Must specify at least one
//! of --node, --chrome, --firefox, or --safari", confirmed empirically — unlike `wasm-pack build`, `test`'s own
//! `[PATH_AND_EXTRA_OPTIONS]...` argument swallows anything placed after it instead of parsing it as flags. This
//! is otherwise the same invocation shape the root `svg-dom` crate's own `tests/*.rs` use, just scoped to this
//! crate's directory instead of the repo root. Compiled as an
//! ordinary `#[cfg(test)]` module (not gated on `target_arch = "wasm32"`) because `#[wasm_bindgen_test]` itself
//! already degrades safely on a native target — it compiles cleanly but registers zero tests there (confirmed
//! empirically against the root crate's own `tests/svg_root.rs`), so `cargo test -p svg-dom-demo` continues to
//! exercise only `unit_tests`'s native tests, unaffected by this module's presence.
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn document() -> web_sys::Document {
    web_sys::window().expect("no window").document().expect("no document")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Creates a fresh `<div id="{id}">` appended to `<body>`, for `SvgRoot::create_in(id, ...)` to attach to — the
/// same role `tests/common.rs::div` plays for the root crate's own browser tests. `demo_text`/`demo_text_path`
/// hard-code their own container id internally (`"demo-text"`/`"demo-text-path"`), so unlike the root crate's
/// tests, this cannot use a fresh id per test; each demo function is instead exercised by exactly one test below,
/// avoiding the duplicate-id ambiguity a second call would create in `get_element_by_id`.
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
/// Finds the `<input type="radio">` labelled `option_label` (the label's own text content, e.g. `"middle"`)
/// inside whichever `<fieldset>` has a `<legend>` reading `group_legend` (e.g. `"text-anchor"`) — the exact
/// structure `demo_text`'s two radio groups both build. Finding it this way, rather than by position, is itself
/// part of what this test verifies: it only succeeds if the fieldset/legend grouping and each radio's `<label>`
/// association are both actually present, not just the radios themselves.
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
/// Selects a radio the way a real click would: sets `checked` first, then dispatches `change`. A synthetic
/// `dispatch_event` does not perform the browser's own default action (toggling `checked`) the way an actual
/// click does, so that has to be done by hand — the slider test below applies the same principle to `value`.
fn select_radio(radio: &web_sys::HtmlInputElement) {
    radio.set_checked(true);
    let event = web_sys::Event::new("change").expect("create change event");
    radio.dispatch_event(&event).expect("dispatch change");
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[wasm_bindgen_test]
fn demo_text_radio_groups_update_their_target_attributes() {
    container("demo-text");
    super::texts::demo_text::demo().expect("demo_text::demo should build without error");

    let root = document().get_element_by_id("demo-text").expect("container exists");

    // Neither target <text> element carries an id (see texts.rs), so they are told apart by their own static
    // text content — which the interactive controls never touch, only their text-anchor/dominant-baseline
    // attributes — rather than by DOM position.
    let find_text = |content: &str| -> web_sys::Element {
        let texts = root.query_selector_all("text").expect("query text elements");
        for i in 0..texts.length() {
            let el = texts
                .item(i)
                .expect("text item")
                .dyn_into::<web_sys::Element>()
                .expect("Element");
            if el.text_content().as_deref() == Some(content) {
                return el;
            }
        }
        panic!("no <text> element with content {content:?}");
    };

    let anchor_text = find_text("sample text");
    let baseline_text = find_text("baseline");

    // Both start at the library's own default, exactly as demo_text sets them up — asserted before any
    // interaction so a later mismatch can only be attributed to the radio click itself, not to the initial state.
    assert_eq!(anchor_text.get_attribute("text-anchor").as_deref(), Some("start"));
    assert_eq!(baseline_text.get_attribute("dominant-baseline").as_deref(), Some("alphabetic"));

    let middle = find_radio(&root, "text-anchor", "middle");
    select_radio(&middle);
    assert_eq!(
        anchor_text.get_attribute("text-anchor").as_deref(),
        Some("middle"),
        "selecting Middle should update text-anchor"
    );

    let hanging = find_radio(&root, "dominant-baseline", "hanging");
    select_radio(&hanging);
    assert_eq!(
        baseline_text.get_attribute("dominant-baseline").as_deref(),
        Some("hanging"),
        "selecting Hanging should update dominant-baseline"
    );

    // Checking the SVG attributes alone does not prove the two <input name="..."> groups are independent: a browser
    // only fires `change` on the radio that becomes newly checked, not on one a same-name group silently unchecks.
    // So if a regression merged the two `name` values, selecting `hanging` would silently uncheck `middle` without ever
    // calling `set_text_anchor` again, leaving `anchor_text`'s attribute at "middle" by accident rather than by proof
    // — the assertions above would pass either way. Inspecting the inputs' own `checked`/`name` state is what actually
    // pins down group independence.
    assert!(
        middle.checked(),
        "middle should still be checked after selecting hanging in the other group"
    );
    assert!(hanging.checked());
    assert_ne!(middle.name(), hanging.name(), "the two radio groups must not share a name");
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[wasm_bindgen_test]
fn start_offset_slider_updates_position_colour_text_and_stays_within_the_path_length() {
    container("demo-text-path");
    super::texts::demo_text_path::demo().expect("demo_text_path::demo should build without error");

    let root = document().get_element_by_id("demo-text-path").expect("container exists");

    let guide = root
        .query_selector("#demo-tp-offset-arc")
        .expect("query guide arc")
        .expect("guide arc present")
        .dyn_into::<web_sys::SvgGeometryElement>()
        .expect("guide arc is an SvgGeometryElement");
    let real_length = f64::from(guide.get_total_length());

    // The aria-label doubles as the locator here and as the accessible-name assertion: this query only succeeds
    // if the slider actually has one.
    let slider = root
        .query_selector("input[aria-label='textPath startOffset']")
        .expect("query slider")
        .expect("slider with the expected aria-label present")
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect("slider is an HtmlInputElement");

    let slider_max: f64 = slider.max().parse().expect("slider max is numeric");
    assert!(
        slider_max <= real_length,
        "slider max ({slider_max}) must never exceed the guide's real total_length() ({real_length})"
    );

    // `demo_text_path` builds *two* <textPath> elements — the sine-wave one above this section, and this one —
    // so a bare "textPath" selector would silently grab the wrong one (document order, not creation-site
    // proximity). Its `href` is the one thing that actually distinguishes it.
    let offset_path = root
        .query_selector("textPath[href='#demo-tp-offset-arc']")
        .expect("query textPath")
        .expect("offset textPath present");
    assert_eq!(
        offset_path.get_attribute("fill").as_deref(),
        Some("white"),
        "home position starts white"
    );
    assert_eq!(offset_path.text_content().as_deref(), Some("Offset 0"));

    // Moving the slider: set the DOM property, then dispatch — the `input` listener reads `.value()` directly
    // rather than inspecting the event, exactly like `select_radio` above relies on for `checked`.
    let dispatch_input = |value: &str| {
        slider.set_value(value);
        let event = web_sys::Event::new("input").expect("create input event");
        slider.dispatch_event(&event).expect("dispatch input");
    };

    dispatch_input("50");
    assert_eq!(offset_path.get_attribute("startOffset").as_deref(), Some("50"));
    assert_eq!(
        offset_path.get_attribute("fill").as_deref(),
        Some("coral"),
        "away from home should read orange"
    );
    assert_eq!(offset_path.text_content().as_deref(), Some("Offset 50"));
    assert_eq!(
        slider.get_attribute("aria-valuetext").as_deref(),
        Some("Offset 50"),
        "aria-valuetext should mirror the same text sighted users see on the curve"
    );

    dispatch_input("0");
    assert_eq!(
        offset_path.get_attribute("fill").as_deref(),
        Some("white"),
        "back at home position should read white again"
    );
}
