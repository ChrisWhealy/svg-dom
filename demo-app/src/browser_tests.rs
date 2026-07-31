//! Browser tests for the interactive HTML controls built inside `<foreignObject>`s.
//! `texts` contributes the text-anchor/dominant-baseline radio groups (`demo_text`) and the startOffset slider
//! (`demo_text_path`). `structure` contributes the marker viewBox zoom slider (`demo_marker_view_box`) and the
//! preserveAspectRatio radio group (`demo_image`).
//! `paint` contributes the linearGradient demo's five sliders (`demo_linear_gradient`).
//!
//! One test below builds `foreign_html::radio_group` directly, not through a demo panel.
//! Three panels now share this helper: `demo_text`'s two groups, and `demo_image`'s preserveAspectRatio group.
//! A single direct test covers the helper's own behaviour once, instead of duplicating it per panel.
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
use std::{cell::RefCell, rc::Rc};

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

    // Neither target <text> element carries an id (see texts/demo_text.rs), so they are told apart by their own static
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[wasm_bindgen_test]
fn marker_view_box_slider_updates_marker_and_readout_without_moving_the_line() {
    container("demo-marker-view-box");
    super::structure::demo_marker_view_box::demo().expect("demo_marker_view_box::demo should build without error");

    let root = document().get_element_by_id("demo-marker-view-box").expect("container exists");

    let marker = root
        .query_selector("marker#arrow-zoom")
        .expect("query marker")
        .expect("arrow-zoom marker present");

    let line = root.query_selector("line").expect("query line").expect("line present");
    // Captured before any interaction, not hard-coded, so this test does not need to know the demo's own layout
    // constants (PAD_Y etc.) to prove the line never moves.
    let initial_x1 = line.get_attribute("x1");
    let initial_y1 = line.get_attribute("y1");
    let initial_x2 = line.get_attribute("x2");
    let initial_y2 = line.get_attribute("y2");

    // Neither the target <text> nor the slider carries an id, so both are found the same way the other tests in
    // this file find their targets: by content/aria-label, not DOM position.
    let readout = {
        let texts = root.query_selector_all("text").expect("query text elements");
        let mut found = None;
        for i in 0..texts.length() {
            let el = texts
                .item(i)
                .expect("text item")
                .dyn_into::<web_sys::Element>()
                .expect("Element");
            if el.text_content().as_deref() == Some("viewBox 100 x 70") {
                found = Some(el);
                break;
            }
        }
        found.expect("no <text> element with initial content \"viewBox 100 x 70\"")
    };

    let slider = root
        .query_selector("input[aria-label='marker viewBox zoom']")
        .expect("query slider")
        .expect("slider with the expected aria-label present")
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect("slider is an HtmlInputElement");

    // Both start at the marker's own default (the full, unclipped triangle), asserted before any interaction so a
    // later mismatch can only be attributed to the slider itself, not to the initial state. aria-valuetext is
    // checked here too: it is set once, separately from the readout text, at construction time, so a regression
    // there (e.g. a literal, unformatted string) would not otherwise be caught until the first `input` event.
    assert_eq!(marker.get_attribute("viewBox").as_deref(), Some("0 0 100 70"));
    assert_eq!(marker.get_attribute("refX").as_deref(), Some("100"));
    assert_eq!(marker.get_attribute("refY").as_deref(), Some("35"));
    assert_eq!(slider.get_attribute("aria-valuetext").as_deref(), Some("viewBox 100 x 70"));

    let dispatch_input = |value: &str| {
        slider.set_value(value);
        let event = web_sys::Event::new("input").expect("create input event");
        slider.dispatch_event(&event).expect("dispatch input");
    };

    dispatch_input("50");
    assert_eq!(
        marker.get_attribute("viewBox").as_deref(),
        Some("0 0 50 35"),
        "at +50% the viewBox should shrink to half its base size"
    );
    assert_eq!(
        marker.get_attribute("refX").as_deref(),
        Some("50"),
        "refX should track the new viewBox's width"
    );
    assert_eq!(
        marker.get_attribute("refY").as_deref(),
        Some("17.5"),
        "refY should track half the new viewBox's height"
    );
    assert_eq!(readout.text_content().as_deref(), Some("viewBox 50 x 35"));
    assert_eq!(slider.get_attribute("aria-valuetext").as_deref(), Some("viewBox 50 x 35"));

    dispatch_input("-50");
    assert_eq!(
        marker.get_attribute("viewBox").as_deref(),
        Some("0 0 150 105"),
        "at -50% the viewBox should grow to 1.5x its base size"
    );
    assert_eq!(marker.get_attribute("refX").as_deref(), Some("150"));
    assert_eq!(marker.get_attribute("refY").as_deref(), Some("52.5"));
    assert_eq!(readout.text_content().as_deref(), Some("viewBox 150 x 105"));
    assert_eq!(slider.get_attribute("aria-valuetext").as_deref(), Some("viewBox 150 x 105"));

    // Neither viewBox nor refX/refY on a marker has any way to reach the <line> that references it via
    // marker-end: the shaft must keep its own length and position throughout, at both extremes of the slider.
    assert_eq!(line.get_attribute("x1"), initial_x1);
    assert_eq!(line.get_attribute("y1"), initial_y1);
    assert_eq!(line.get_attribute("x2"), initial_x2);
    assert_eq!(line.get_attribute("y2"), initial_y2);
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[wasm_bindgen_test]
fn demo_image_radio_group_updates_preserve_aspect_ratio() {
    container("demo-image");
    super::structure::demo_image::demo().expect("demo_image::demo should build without error");

    let root = document().get_element_by_id("demo-image").expect("container exists");

    // Only the interactive image carries a preserveAspectRatio attribute. The set_href swap image never sets one.
    // This attribute is what tells the two <image> elements apart.
    let image = root
        .query_selector("image[preserveAspectRatio]")
        .expect("query image")
        .expect("interactive image present");

    // Starts at the demo's own default, asserted before any interaction. A later mismatch can then only come
    // from the radio click itself, not from the initial state.
    assert_eq!(image.get_attribute("preserveAspectRatio").as_deref(), Some("xMidYMid meet"));

    let none = find_radio(&root, "preserveAspectRatio", "none");
    select_radio(&none);
    assert_eq!(
        image.get_attribute("preserveAspectRatio").as_deref(),
        Some("none"),
        "selecting none should update preserveAspectRatio"
    );

    let slice = find_radio(&root, "preserveAspectRatio", "slice");
    select_radio(&slice);
    assert_eq!(
        image.get_attribute("preserveAspectRatio").as_deref(),
        Some("xMidYMid slice"),
        "selecting slice should update preserveAspectRatio"
    );

    let meet = find_radio(&root, "preserveAspectRatio", "meet");
    select_radio(&meet);
    assert_eq!(
        image.get_attribute("preserveAspectRatio").as_deref(),
        Some("xMidYMid meet"),
        "selecting meet should update preserveAspectRatio"
    );

    // The radio group must only ever touch the interactive image. This check confirms the swap-demo image
    // still carries no preserveAspectRatio attribute of its own.
    let swap_image = root
        .query_selector("image:not([preserveAspectRatio])")
        .expect("query swap image")
        .expect("swap image present");
    assert!(swap_image.get_attribute("preserveAspectRatio").is_none());
}

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
        super::foreign_html::radio_group(&document(), "demo-legend", "radio-group-test", &OPTIONS, "a", move |value| {
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `demo_linear_gradient` writes to raw `<stop>` and gradient elements via `select_el`, not through a typed
/// `SvgLinearGradient` handle.
/// Source extraction cannot prove those CSS selectors still resolve.
/// It also cannot prove the two spectrum sliders still keep their middle stops ordered.
/// Only a real browser can prove either.
///
/// The spectrum's mutual constraint gets two explicit boundary checks, not just one.
/// Each slider's own `min`/`max` attribute enforces the constraint, not a JavaScript clamp reacting after the
/// fact. This test checks those attributes directly, not only the values they produce.
/// The second boundary check pushes stop 3 against stop 2's own updated value, not its original one, to prove
/// the live attribute tracks the other slider, not a value fixed at construction time.
///
/// Every slider's `aria-valuetext` is also checked before any interaction.
/// Each one must already report its real starting value there, not only after the first input event.
#[wasm_bindgen_test]
fn demo_linear_gradient_sliders_update_stops_and_respect_ordering() {
    container("demo-linear-gradient");
    super::paint::demo_linear_gradient::demo().expect("demo_linear_gradient::demo should build without error");

    let root = document().get_element_by_id("demo-linear-gradient").expect("container exists");

    let dispatch_input = |slider: &web_sys::HtmlInputElement, value: &str| {
        slider.set_value(value);
        let event = web_sys::Event::new("input").expect("create input event");
        slider.dispatch_event(&event).expect("dispatch input");
    };

    let find_el = |selector: &str| -> web_sys::Element {
        root.query_selector(selector)
            .unwrap_or_else(|_| panic!("invalid selector {selector:?}"))
            .unwrap_or_else(|| panic!("no element matching {selector:?}"))
    };

    let find_slider = |aria_label_selector: &str| -> web_sys::HtmlInputElement {
        root.query_selector(aria_label_selector)
            .expect("query slider")
            .unwrap_or_else(|| panic!("no slider matching {aria_label_selector:?}"))
            .dyn_into::<web_sys::HtmlInputElement>()
            .expect("slider is an HtmlInputElement")
    };

    // --- horizontal: the slider shifts #demo-lg-h's second stop ---
    let h_stop = find_el("#demo-lg-h stop:nth-child(2)");
    assert_eq!(h_stop.get_attribute("offset").as_deref(), Some("1"));

    let h_slider = find_slider("input[aria-label='horizontal gradient stop 2']");
    // Set at construction time, before any interaction, so a screen reader announces the real starting value.
    assert_eq!(h_slider.get_attribute("aria-valuetext").as_deref(), Some("100%"));
    dispatch_input(&h_slider, "40");
    assert_eq!(
        h_stop.get_attribute("offset").as_deref(),
        Some("0.40"),
        "moving the horizontal slider should update the second stop's offset"
    );

    // --- vertical: the slider shifts #demo-lg-v's second stop ---
    let v_stop = find_el("#demo-lg-v stop:nth-child(2)");
    assert_eq!(v_stop.get_attribute("offset").as_deref(), Some("1"));

    let v_slider = find_slider("input[aria-label=\"shift the vertical gradient's second stop\"]");
    assert_eq!(v_slider.get_attribute("aria-valuetext").as_deref(), Some("100%"));
    dispatch_input(&v_slider, "25");
    assert_eq!(
        v_stop.get_attribute("offset").as_deref(),
        Some("0.25"),
        "moving the vertical slider should update the second stop's offset"
    );

    // --- diagonal: the slider rotates #demo-lg-d and updates the visible readout ---
    let d_gradient = find_el("#demo-lg-d");
    assert_eq!(
        d_gradient.get_attribute("gradientTransform").as_deref(),
        Some("rotate(45, 0.5, 0.5)")
    );

    // The readout text starts at "rotate 45°", the demo's own initial caption. No id distinguishes it from any
    // other <text>, so it is found by that starting content, the same way other tests in this file find theirs.
    let rotate_readout = {
        let texts = root.query_selector_all("text").expect("query text elements");
        let mut found = None;
        for i in 0..texts.length() {
            let el = texts
                .item(i)
                .expect("text item")
                .dyn_into::<web_sys::Element>()
                .expect("Element");
            if el.text_content().as_deref() == Some("rotate 45°") {
                found = Some(el);
                break;
            }
        }
        found.expect("no <text> element with initial content \"rotate 45°\"")
    };

    let rotate_slider = find_slider("input[aria-label='diagonal gradient rotation']");
    // The slider's own raw value starts at 0 (a relative displacement), but the rendered gradient and the visible
    // readout both start at the absolute 45° base. aria-valuetext must report that same absolute angle from
    // construction, not the raw 0, and not merely from the first input event onward.
    assert_eq!(rotate_slider.value(), "0");
    assert_eq!(rotate_slider.get_attribute("aria-valuetext").as_deref(), Some("rotate 45°"));
    dispatch_input(&rotate_slider, "-30");
    assert_eq!(
        d_gradient.get_attribute("gradientTransform").as_deref(),
        Some("rotate(15, 0.5, 0.5)"),
        "rotating -30 from the 45° base should give 15°"
    );
    assert_eq!(rotate_readout.text_content().as_deref(), Some("rotate 15°"));
    assert_eq!(rotate_slider.get_attribute("aria-valuetext").as_deref(), Some("rotate 15°"));

    // --- 4-stop spectrum: the two middle stops stay ordered ---
    let s2_stop = find_el("#demo-lg-s stop:nth-child(2)");
    let s3_stop = find_el("#demo-lg-s stop:nth-child(3)");
    assert_eq!(s2_stop.get_attribute("offset").as_deref(), Some("0.35"));
    assert_eq!(s3_stop.get_attribute("offset").as_deref(), Some("0.65"));

    let s2_slider = find_slider("input[aria-label='spectrum gradient stop 2']");
    let s3_slider = find_slider("input[aria-label='spectrum gradient stop 3']");
    assert_eq!(s2_slider.get_attribute("aria-valuetext").as_deref(), Some("35%"));
    assert_eq!(s3_slider.get_attribute("aria-valuetext").as_deref(), Some("65%"));

    // Each slider's own min/max attribute already exposes the live constraint at construction, before either
    // slider has fired an input event of its own. Stop 2's absolute min (1) and stop 3's absolute max (99) never
    // change; only the shared boundary between the two stops does.
    assert_eq!(s2_slider.min(), "1", "stop 2's absolute lower bound never changes");
    assert_eq!(
        s2_slider.max(),
        "64",
        "stop 2's live upper bound should track stop 3's value minus one"
    );
    assert_eq!(
        s3_slider.min(),
        "36",
        "stop 3's live lower bound should track stop 2's value plus one"
    );
    assert_eq!(s3_slider.max(), "99", "stop 3's absolute upper bound never changes");

    // Push stop 2 past stop 3's current value (65). The native max attribute above already stops it at 64,
    // before this demo's own `on_input` handler ever runs.
    dispatch_input(&s2_slider, "70");
    assert_eq!(
        s2_slider.value(),
        "64",
        "the browser's own max attribute should stop stop 2 at one point below stop 3"
    );
    assert_eq!(s2_stop.get_attribute("offset").as_deref(), Some("0.640"));
    assert_eq!(
        s3_stop.get_attribute("offset").as_deref(),
        Some("0.65"),
        "stop 3 must stay put while stop 2 moves"
    );
    assert_eq!(
        s3_slider.min(),
        "65",
        "stop 3's live lower bound should follow stop 2's new value"
    );

    // Push stop 3 down past stop 2's new current value (64), not its original one (35). The native min
    // attribute, already updated above, stops it at 65 — proving the constraint tracks a live value, not one
    // fixed when the sliders were built.
    dispatch_input(&s3_slider, "50");
    assert_eq!(
        s3_slider.value(),
        "65",
        "the browser's own min attribute should stop stop 3 at one point above stop 2"
    );
    assert_eq!(
        s2_stop.get_attribute("offset").as_deref(),
        Some("0.640"),
        "stop 2 must stay put while stop 3 moves"
    );
    assert_eq!(s3_stop.get_attribute("offset").as_deref(), Some("0.650"));
    assert_eq!(
        s2_slider.max(),
        "64",
        "stop 2's live upper bound should still track stop 3's value minus one"
    );

    // The fixed outer stops never move, however far the middle two are dragged.
    let s1_stop = find_el("#demo-lg-s stop:nth-child(1)");
    let s4_stop = find_el("#demo-lg-s stop:nth-child(4)");
    assert_eq!(s1_stop.get_attribute("offset").as_deref(), Some("0"));
    assert_eq!(s4_stop.get_attribute("offset").as_deref(), Some("1"));

    // --- gradient stroke: untouched by every slider above ---
    let stroke_stop_1 = find_el("#demo-lg-stroke stop:nth-child(1)");
    let stroke_stop_2 = find_el("#demo-lg-stroke stop:nth-child(2)");
    assert_eq!(stroke_stop_1.get_attribute("offset").as_deref(), Some("0"));
    assert_eq!(stroke_stop_1.get_attribute("stop-color").as_deref(), Some("mediumseagreen"));
    assert_eq!(stroke_stop_2.get_attribute("offset").as_deref(), Some("1"));
    assert_eq!(stroke_stop_2.get_attribute("stop-color").as_deref(), Some("coral"));
}
