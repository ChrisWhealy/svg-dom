//! Tests for `texts`'s own interactive controls: `demo_text`'s text-anchor/dominant-baseline radio groups, and
//! `demo_text_path`'s startOffset slider.

use crate::browser_tests::{container, document, find_radio, select_radio};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[wasm_bindgen_test]
fn demo_text_radio_groups_update_their_target_attributes() -> Result<(), String> {
    container("demo-text");
    crate::texts::demo_text::demo().map_err(|e| format!("demo_text::demo should build without error: {e:?}"))?;

    let root = document()
        .get_element_by_id("demo-text")
        .ok_or_else(|| "container exists".to_owned())?;

    // Neither target <text> element carries an id (see texts/demo_text.rs), so they are told apart by their own static
    // text content — which the interactive controls never touch, only their text-anchor/dominant-baseline
    // attributes — rather than by DOM position.
    let find_text = |content: &str| -> Result<web_sys::Element, String> {
        let texts = root
            .query_selector_all("text")
            .map_err(|e| format!("query text elements: {e:?}"))?;
        for i in 0..texts.length() {
            let el = texts
                .item(i)
                .ok_or_else(|| "text item".to_owned())?
                .dyn_into::<web_sys::Element>()
                .map_err(|_| "expected an Element".to_owned())?;
            if el.text_content().as_deref() == Some(content) {
                return Ok(el);
            }
        }
        Err(format!("no <text> element with content {content:?}"))
    };

    let anchor_text = find_text("sample text")?;
    let baseline_text = find_text("baseline")?;

    // Both start at the library's own default, exactly as demo_text sets them up — asserted before any
    // interaction so a later mismatch can only be attributed to the radio click itself, not to the initial state.
    if anchor_text.get_attribute("text-anchor").as_deref() != Some("start") {
        return Err(format!(
            "expected initial text-anchor \"start\", got {:?}",
            anchor_text.get_attribute("text-anchor")
        ));
    }
    if baseline_text.get_attribute("dominant-baseline").as_deref() != Some("alphabetic") {
        return Err(format!(
            "expected initial dominant-baseline \"alphabetic\", got {:?}",
            baseline_text.get_attribute("dominant-baseline")
        ));
    }

    let middle = find_radio(&root, "text-anchor", "middle")?;
    select_radio(&middle)?;
    if anchor_text.get_attribute("text-anchor").as_deref() != Some("middle") {
        return Err(format!(
            "selecting Middle should update text-anchor, got {:?}",
            anchor_text.get_attribute("text-anchor")
        ));
    }

    let hanging = find_radio(&root, "dominant-baseline", "hanging")?;
    select_radio(&hanging)?;
    if baseline_text.get_attribute("dominant-baseline").as_deref() != Some("hanging") {
        return Err(format!(
            "selecting Hanging should update dominant-baseline, got {:?}",
            baseline_text.get_attribute("dominant-baseline")
        ));
    }

    // Checking the SVG attributes alone does not prove the two <input name="..."> groups are independent: a browser
    // only fires `change` on the radio that becomes newly checked, not on one a same-name group silently unchecks.
    // So if a regression merged the two `name` values, selecting `hanging` would silently uncheck `middle` without ever
    // calling `set_text_anchor` again, leaving `anchor_text`'s attribute at "middle" by accident rather than by proof
    // — the assertions above would pass either way. Inspecting the inputs' own `checked`/`name` state is what actually
    // pins down group independence.
    if !middle.checked() {
        return Err("middle should still be checked after selecting hanging in the other group".to_owned());
    }
    if !hanging.checked() {
        return Err("hanging should be checked".to_owned());
    }
    if middle.name() == hanging.name() {
        return Err("the two radio groups must not share a name".to_owned());
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[wasm_bindgen_test]
fn start_offset_slider_updates_position_colour_text_and_stays_within_the_path_length() -> Result<(), String> {
    container("demo-text-path");
    crate::texts::demo_text_path::demo()
        .map_err(|e| format!("demo_text_path::demo should build without error: {e:?}"))?;

    let root = document()
        .get_element_by_id("demo-text-path")
        .ok_or_else(|| "container exists".to_owned())?;

    let guide = root
        .query_selector("#demo-tp-offset-arc")
        .map_err(|e| format!("query guide arc: {e:?}"))?
        .ok_or_else(|| "guide arc present".to_owned())?
        .dyn_into::<web_sys::SvgGeometryElement>()
        .map_err(|_| "guide arc is an SvgGeometryElement".to_owned())?;
    let real_length = f64::from(guide.get_total_length());

    // The aria-label doubles as the locator here and as the accessible-name assertion: this query only succeeds
    // if the slider actually has one.
    let slider = root
        .query_selector("input[aria-label='textPath startOffset']")
        .map_err(|e| format!("query slider: {e:?}"))?
        .ok_or_else(|| "slider with the expected aria-label present".to_owned())?
        .dyn_into::<web_sys::HtmlInputElement>()
        .map_err(|_| "slider is an HtmlInputElement".to_owned())?;

    let slider_max: f64 = slider.max().parse().map_err(|e| format!("slider max is numeric: {e:?}"))?;
    if slider_max > real_length {
        return Err(format!(
            "slider max ({slider_max}) must never exceed the guide's real total_length() ({real_length})"
        ));
    }

    // `demo_text_path` builds *two* <textPath> elements — the sine-wave one above this section, and this one —
    // so a bare "textPath" selector would silently grab the wrong one (document order, not creation-site
    // proximity). Its `href` is the one thing that actually distinguishes it.
    let offset_path = root
        .query_selector("textPath[href='#demo-tp-offset-arc']")
        .map_err(|e| format!("query textPath: {e:?}"))?
        .ok_or_else(|| "offset textPath present".to_owned())?;
    if offset_path.get_attribute("fill").as_deref() != Some("white") {
        return Err(format!(
            "home position starts white, got {:?}",
            offset_path.get_attribute("fill")
        ));
    }
    if offset_path.text_content().as_deref() != Some("Offset 0") {
        return Err(format!(
            "expected text content \"Offset 0\", got {:?}",
            offset_path.text_content()
        ));
    }

    // Moving the slider: set the DOM property, then dispatch — the `input` listener reads `.value()` directly
    // rather than inspecting the event, exactly like `select_radio` above relies on for `checked`.
    let dispatch_input = |value: &str| -> Result<(), String> {
        slider.set_value(value);
        let event = web_sys::Event::new("input").map_err(|e| format!("create input event: {e:?}"))?;
        slider.dispatch_event(&event).map_err(|e| format!("dispatch input: {e:?}"))?;
        Ok(())
    };

    dispatch_input("50")?;
    if offset_path.get_attribute("startOffset").as_deref() != Some("50") {
        return Err(format!(
            "expected startOffset \"50\", got {:?}",
            offset_path.get_attribute("startOffset")
        ));
    }
    if offset_path.get_attribute("fill").as_deref() != Some("coral") {
        return Err(format!(
            "away from home should read orange, got {:?}",
            offset_path.get_attribute("fill")
        ));
    }
    if offset_path.text_content().as_deref() != Some("Offset 50") {
        return Err(format!(
            "expected text content \"Offset 50\", got {:?}",
            offset_path.text_content()
        ));
    }
    if slider.get_attribute("aria-valuetext").as_deref() != Some("Offset 50") {
        return Err(format!(
            "aria-valuetext should mirror the same text sighted users see on the curve, got {:?}",
            slider.get_attribute("aria-valuetext")
        ));
    }

    dispatch_input("0")?;
    if offset_path.get_attribute("fill").as_deref() != Some("white") {
        return Err(format!(
            "back at home position should read white again, got {:?}",
            offset_path.get_attribute("fill")
        ));
    }
    Ok(())
}
