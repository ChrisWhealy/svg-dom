//! Tests for `structure`'s own interactive controls: `demo_marker_view_box`'s zoom slider, and `demo_image`'s
//! preserveAspectRatio radio group.

use crate::browser_tests::{container, document, find_radio, select_radio};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[wasm_bindgen_test]
fn marker_view_box_slider_updates_marker_and_readout_without_moving_the_line() -> Result<(), String> {
    container("demo-marker-view-box");
    crate::structure::demo_marker_view_box::demo()
        .map_err(|e| format!("demo_marker_view_box::demo should build without error: {e:?}"))?;

    let root = document()
        .get_element_by_id("demo-marker-view-box")
        .ok_or_else(|| "container exists".to_owned())?;

    let marker = root
        .query_selector("marker#arrow-zoom")
        .map_err(|e| format!("query marker: {e:?}"))?
        .ok_or_else(|| "arrow-zoom marker present".to_owned())?;

    let line = root
        .query_selector("line")
        .map_err(|e| format!("query line: {e:?}"))?
        .ok_or_else(|| "line present".to_owned())?;
    // Captured before any interaction, not hard-coded, so this test does not need to know the demo's own layout
    // constants (PAD_Y etc.) to prove the line never moves.
    let initial_x1 = line.get_attribute("x1");
    let initial_y1 = line.get_attribute("y1");
    let initial_x2 = line.get_attribute("x2");
    let initial_y2 = line.get_attribute("y2");

    // Neither the target <text> nor the slider carries an id, so both are found the same way the other tests in
    // this file find their targets: by content/aria-label, not DOM position.
    let readout = {
        let texts = root
            .query_selector_all("text")
            .map_err(|e| format!("query text elements: {e:?}"))?;
        let mut found = None;
        for i in 0..texts.length() {
            let el = texts
                .item(i)
                .ok_or_else(|| "text item".to_owned())?
                .dyn_into::<web_sys::Element>()
                .map_err(|_| "expected an Element".to_owned())?;
            if el.text_content().as_deref() == Some("viewBox 100 x 70") {
                found = Some(el);
                break;
            }
        }
        found.ok_or_else(|| "no <text> element with initial content \"viewBox 100 x 70\"".to_owned())?
    };

    let slider = root
        .query_selector("input[aria-label='marker viewBox zoom']")
        .map_err(|e| format!("query slider: {e:?}"))?
        .ok_or_else(|| "slider with the expected aria-label present".to_owned())?
        .dyn_into::<web_sys::HtmlInputElement>()
        .map_err(|_| "slider is an HtmlInputElement".to_owned())?;

    // Both start at the marker's own default (the full, unclipped triangle), asserted before any interaction so a
    // later mismatch can only be attributed to the slider itself, not to the initial state. aria-valuetext is
    // checked here too: it is set once, separately from the readout text, at construction time, so a regression
    // there (e.g. a literal, unformatted string) would not otherwise be caught until the first `input` event.
    if marker.get_attribute("viewBox").as_deref() != Some("0 0 100 70") {
        return Err(format!(
            "expected initial viewBox \"0 0 100 70\", got {:?}",
            marker.get_attribute("viewBox")
        ));
    }
    if marker.get_attribute("refX").as_deref() != Some("100") {
        return Err(format!("expected initial refX \"100\", got {:?}", marker.get_attribute("refX")));
    }
    if marker.get_attribute("refY").as_deref() != Some("35") {
        return Err(format!("expected initial refY \"35\", got {:?}", marker.get_attribute("refY")));
    }
    if slider.get_attribute("aria-valuetext").as_deref() != Some("viewBox 100 x 70") {
        return Err(format!(
            "expected initial aria-valuetext \"viewBox 100 x 70\", got {:?}",
            slider.get_attribute("aria-valuetext")
        ));
    }

    let dispatch_input = |value: &str| -> Result<(), String> {
        slider.set_value(value);
        let event = web_sys::Event::new("input").map_err(|e| format!("create input event: {e:?}"))?;
        slider.dispatch_event(&event).map_err(|e| format!("dispatch input: {e:?}"))?;
        Ok(())
    };

    dispatch_input("50")?;
    if marker.get_attribute("viewBox").as_deref() != Some("0 0 50 35") {
        return Err(format!(
            "at +50% the viewBox should shrink to half its base size, got {:?}",
            marker.get_attribute("viewBox")
        ));
    }
    if marker.get_attribute("refX").as_deref() != Some("50") {
        return Err(format!(
            "refX should track the new viewBox's width, got {:?}",
            marker.get_attribute("refX")
        ));
    }
    if marker.get_attribute("refY").as_deref() != Some("17.5") {
        return Err(format!(
            "refY should track half the new viewBox's height, got {:?}",
            marker.get_attribute("refY")
        ));
    }
    if readout.text_content().as_deref() != Some("viewBox 50 x 35") {
        return Err(format!(
            "expected readout \"viewBox 50 x 35\", got {:?}",
            readout.text_content()
        ));
    }
    if slider.get_attribute("aria-valuetext").as_deref() != Some("viewBox 50 x 35") {
        return Err(format!(
            "expected aria-valuetext \"viewBox 50 x 35\", got {:?}",
            slider.get_attribute("aria-valuetext")
        ));
    }

    dispatch_input("-50")?;
    if marker.get_attribute("viewBox").as_deref() != Some("0 0 150 105") {
        return Err(format!(
            "at -50% the viewBox should grow to 1.5x its base size, got {:?}",
            marker.get_attribute("viewBox")
        ));
    }
    if marker.get_attribute("refX").as_deref() != Some("150") {
        return Err(format!("expected refX \"150\", got {:?}", marker.get_attribute("refX")));
    }
    if marker.get_attribute("refY").as_deref() != Some("52.5") {
        return Err(format!("expected refY \"52.5\", got {:?}", marker.get_attribute("refY")));
    }
    if readout.text_content().as_deref() != Some("viewBox 150 x 105") {
        return Err(format!(
            "expected readout \"viewBox 150 x 105\", got {:?}",
            readout.text_content()
        ));
    }
    if slider.get_attribute("aria-valuetext").as_deref() != Some("viewBox 150 x 105") {
        return Err(format!(
            "expected aria-valuetext \"viewBox 150 x 105\", got {:?}",
            slider.get_attribute("aria-valuetext")
        ));
    }

    // Neither viewBox nor refX/refY on a marker has any way to reach the <line> that references it via
    // marker-end: the shaft must keep its own length and position throughout, at both extremes of the slider.
    if line.get_attribute("x1") != initial_x1 {
        return Err("the line's x1 must not move".to_owned());
    }
    if line.get_attribute("y1") != initial_y1 {
        return Err("the line's y1 must not move".to_owned());
    }
    if line.get_attribute("x2") != initial_x2 {
        return Err("the line's x2 must not move".to_owned());
    }
    if line.get_attribute("y2") != initial_y2 {
        return Err("the line's y2 must not move".to_owned());
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[wasm_bindgen_test]
fn demo_image_radio_group_updates_preserve_aspect_ratio() -> Result<(), String> {
    container("demo-image");
    crate::structure::demo_image::demo().map_err(|e| format!("demo_image::demo should build without error: {e:?}"))?;

    let root = document()
        .get_element_by_id("demo-image")
        .ok_or_else(|| "container exists".to_owned())?;

    // Only the interactive image carries a preserveAspectRatio attribute. The set_href swap image never sets one.
    // This attribute is what tells the two <image> elements apart.
    let image = root
        .query_selector("image[preserveAspectRatio]")
        .map_err(|e| format!("query image: {e:?}"))?
        .ok_or_else(|| "interactive image present".to_owned())?;

    // Starts at the demo's own default, asserted before any interaction. A later mismatch can then only come
    // from the radio click itself, not from the initial state.
    if image.get_attribute("preserveAspectRatio").as_deref() != Some("xMidYMid meet") {
        return Err(format!(
            "expected initial preserveAspectRatio \"xMidYMid meet\", got {:?}",
            image.get_attribute("preserveAspectRatio")
        ));
    }

    let none = find_radio(&root, "preserveAspectRatio", "none")?;
    select_radio(&none)?;
    if image.get_attribute("preserveAspectRatio").as_deref() != Some("none") {
        return Err(format!(
            "selecting none should update preserveAspectRatio, got {:?}",
            image.get_attribute("preserveAspectRatio")
        ));
    }

    let slice = find_radio(&root, "preserveAspectRatio", "slice")?;
    select_radio(&slice)?;
    if image.get_attribute("preserveAspectRatio").as_deref() != Some("xMidYMid slice") {
        return Err(format!(
            "selecting slice should update preserveAspectRatio, got {:?}",
            image.get_attribute("preserveAspectRatio")
        ));
    }

    let meet = find_radio(&root, "preserveAspectRatio", "meet")?;
    select_radio(&meet)?;
    if image.get_attribute("preserveAspectRatio").as_deref() != Some("xMidYMid meet") {
        return Err(format!(
            "selecting meet should update preserveAspectRatio, got {:?}",
            image.get_attribute("preserveAspectRatio")
        ));
    }

    // The radio group must only ever touch the interactive image. This check confirms the swap-demo image
    // still carries no preserveAspectRatio attribute of its own.
    let swap_image = root
        .query_selector("image:not([preserveAspectRatio])")
        .map_err(|e| format!("query swap image: {e:?}"))?
        .ok_or_else(|| "swap image present".to_owned())?;
    if swap_image.get_attribute("preserveAspectRatio").is_some() {
        return Err("the swap image must not gain a preserveAspectRatio attribute".to_owned());
    }
    Ok(())
}
