//! Tests for `structure`'s own interactive controls: `demo_marker_view_box`'s zoom slider, and `demo_image`'s
//! preserveAspectRatio radio group.

use crate::browser_tests::{container, document, find_radio, select_radio};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[wasm_bindgen_test]
fn marker_view_box_slider_updates_marker_and_readout_without_moving_the_line() {
    container("demo-marker-view-box");
    crate::structure::demo_marker_view_box::demo().expect("demo_marker_view_box::demo should build without error");

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
    crate::structure::demo_image::demo().expect("demo_image::demo should build without error");

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
