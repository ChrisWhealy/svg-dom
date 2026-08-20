//! Tests for `demo_radial_gradient`'s own slider, radio group, and slider pair.

use crate::browser_tests::{container, document, find_radio, select_radio};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `demo_radial_gradient` writes to raw `<stop>` and gradient elements via `select_el`, the same escape-hatch
/// reason `demo_linear_gradient`'s own doc comment gives.
/// Source extraction cannot prove any of these live updates still work.
/// It also cannot prove the fy slider's own keyboard remap still matches its visual "up is smaller" scale.
/// Only a real browser can prove either.
#[wasm_bindgen_test]
fn demo_radial_gradient_controls_update_stops_spread_and_focal_point() -> Result<(), String> {
    container("demo-radial-gradient");
    crate::paint::demo_radial_gradient::demo()
        .map_err(|e| format!("demo_radial_gradient::demo should build without error: {e:?}"))?;

    let root = document()
        .get_element_by_id("demo-radial-gradient")
        .ok_or_else(|| "container exists".to_owned())?;

    let dispatch_input = |slider: &web_sys::HtmlInputElement, value: &str| -> Result<(), String> {
        slider.set_value(value);
        let event = web_sys::Event::new("input").map_err(|e| format!("create input event: {e:?}"))?;
        slider.dispatch_event(&event).map_err(|e| format!("dispatch input: {e:?}"))?;
        Ok(())
    };

    let find_el = |selector: &str| -> Result<web_sys::Element, String> {
        root.query_selector(selector)
            .map_err(|e| format!("invalid selector {selector:?}: {e:?}"))?
            .ok_or_else(|| format!("no element matching {selector:?}"))
    };

    let find_slider = |aria_label_selector: &str| -> Result<web_sys::HtmlInputElement, String> {
        root.query_selector(aria_label_selector)
            .map_err(|e| format!("query slider: {e:?}"))?
            .ok_or_else(|| format!("no slider matching {aria_label_selector:?}"))?
            .dyn_into::<web_sys::HtmlInputElement>()
            .map_err(|_| "slider is an HtmlInputElement".to_owned())
    };

    // --- centred: the slider shifts #demo-rg-c's middle stop along the radius ---
    let c_stop = find_el("#demo-rg-c stop:nth-child(2)")?;
    if c_stop.get_attribute("offset").as_deref() != Some("0.5") {
        return Err(format!(
            "expected initial offset \"0.5\", got {:?}",
            c_stop.get_attribute("offset")
        ));
    }

    let c_slider = find_slider("input[aria-label='centred gradient stop 2']")?;
    if c_slider.get_attribute("aria-valuetext").as_deref() != Some("50%") {
        return Err(format!(
            "expected initial aria-valuetext \"50%\", got {:?}",
            c_slider.get_attribute("aria-valuetext")
        ));
    }
    dispatch_input(&c_slider, "70")?;
    if c_stop.get_attribute("offset").as_deref() != Some("0.70") {
        return Err(format!(
            "moving the centred slider should update the middle stop's offset, got {:?}",
            c_stop.get_attribute("offset")
        ));
    }
    if c_slider.get_attribute("aria-valuetext").as_deref() != Some("70%") {
        return Err(format!(
            "expected aria-valuetext \"70%\", got {:?}",
            c_slider.get_attribute("aria-valuetext")
        ));
    }

    // --- spreadMethod: radio buttons choose #demo-rg-r's spreadMethod ---
    let r_gradient = find_el("#demo-rg-r")?;
    if r_gradient.get_attribute("spreadMethod").as_deref() != Some("reflect") {
        return Err(format!(
            "reflect is this demo's own initial default, got {:?}",
            r_gradient.get_attribute("spreadMethod")
        ));
    }

    let pad = find_radio(&root, "spreadMethod", "pad")?;
    select_radio(&pad)?;
    if r_gradient.get_attribute("spreadMethod").as_deref() != Some("pad") {
        return Err(format!(
            "expected spreadMethod \"pad\", got {:?}",
            r_gradient.get_attribute("spreadMethod")
        ));
    }

    let repeat = find_radio(&root, "spreadMethod", "repeat")?;
    select_radio(&repeat)?;
    if r_gradient.get_attribute("spreadMethod").as_deref() != Some("repeat") {
        return Err(format!(
            "expected spreadMethod \"repeat\", got {:?}",
            r_gradient.get_attribute("spreadMethod")
        ));
    }

    let reflect = find_radio(&root, "spreadMethod", "reflect")?;
    select_radio(&reflect)?;
    if r_gradient.get_attribute("spreadMethod").as_deref() != Some("reflect") {
        return Err(format!(
            "expected spreadMethod \"reflect\", got {:?}",
            r_gradient.get_attribute("spreadMethod")
        ));
    }
    if !reflect.checked() {
        return Err("reflect should be checked".to_owned());
    }
    if pad.checked() {
        return Err("selecting reflect should clear pad".to_owned());
    }
    if repeat.checked() {
        return Err("selecting reflect should clear repeat".to_owned());
    }

    // --- off-centre focal: fx (horizontal) and fy (vertical) sliders move #demo-rg-f's focal point ---
    let f_gradient = find_el("#demo-rg-f")?;
    if f_gradient.get_attribute("fx").as_deref() != Some("0.25") {
        return Err(format!(
            "expected initial fx \"0.25\", got {:?}",
            f_gradient.get_attribute("fx")
        ));
    }
    if f_gradient.get_attribute("fy").as_deref() != Some("0.25") {
        return Err(format!(
            "expected initial fy \"0.25\", got {:?}",
            f_gradient.get_attribute("fy")
        ));
    }

    // The focal marker reads its own position straight from the rectangle's live x/y/width/height, rather than
    // this test hard-coding the demo's own layout constants, the same reason `marker_view_box_slider_...` above
    // captures the line's own initial attributes instead of recomputing them.
    let focal_rect = find_el("rect")?;
    let rect_x: f64 = focal_rect
        .get_attribute("x")
        .ok_or_else(|| "rect x".to_owned())?
        .parse()
        .map_err(|e| format!("numeric x: {e:?}"))?;
    let rect_y: f64 = focal_rect
        .get_attribute("y")
        .ok_or_else(|| "rect y".to_owned())?
        .parse()
        .map_err(|e| format!("numeric y: {e:?}"))?;
    let rect_w: f64 = focal_rect
        .get_attribute("width")
        .ok_or_else(|| "rect width".to_owned())?
        .parse()
        .map_err(|e| format!("numeric width: {e:?}"))?;
    let rect_h: f64 = focal_rect
        .get_attribute("height")
        .ok_or_else(|| "rect height".to_owned())?
        .parse()
        .map_err(|e| format!("numeric height: {e:?}"))?;

    // `fill="none"` is what distinguishes the marker from the two gradient-filled circles elsewhere on this panel.
    let focal_marker = find_el("circle[fill='none']")?;
    let marker_cx = |el: &web_sys::Element| -> Result<f64, String> {
        el.get_attribute("cx")
            .ok_or_else(|| "marker cx".to_owned())?
            .parse()
            .map_err(|e| format!("numeric cx: {e:?}"))
    };
    let marker_cy = |el: &web_sys::Element| -> Result<f64, String> {
        el.get_attribute("cy")
            .ok_or_else(|| "marker cy".to_owned())?
            .parse()
            .map_err(|e| format!("numeric cy: {e:?}"))
    };

    if (marker_cx(&focal_marker)? - (rect_x + 0.25 * rect_w)).abs() >= 0.1 {
        return Err("expected focal marker cx to match the 25% focal point".to_owned());
    }
    if (marker_cy(&focal_marker)? - (rect_y + 0.25 * rect_h)).abs() >= 0.1 {
        return Err("expected focal marker cy to match the 25% focal point".to_owned());
    }

    let fx_slider = find_slider("input[aria-label='off-centre focal gradient fx']")?;
    if fx_slider.get_attribute("aria-valuetext").as_deref() != Some("25%") {
        return Err(format!(
            "expected initial aria-valuetext \"25%\", got {:?}",
            fx_slider.get_attribute("aria-valuetext")
        ));
    }
    dispatch_input(&fx_slider, "60")?;
    if f_gradient.get_attribute("fx").as_deref() != Some("0.60") {
        return Err(format!(
            "moving the fx slider should update the gradient's fx, got {:?}",
            f_gradient.get_attribute("fx")
        ));
    }
    if fx_slider.get_attribute("aria-valuetext").as_deref() != Some("60%") {
        return Err(format!(
            "expected aria-valuetext \"60%\", got {:?}",
            fx_slider.get_attribute("aria-valuetext")
        ));
    }
    if (marker_cx(&focal_marker)? - (rect_x + 0.60 * rect_w)).abs() >= 0.1 {
        return Err("the focal marker's cx should follow the fx slider".to_owned());
    }
    if (marker_cy(&focal_marker)? - (rect_y + 0.25 * rect_h)).abs() >= 0.1 {
        return Err("the focal marker's cy should stay put while only fx moves".to_owned());
    }

    let fy_slider = find_slider("input[aria-label='off-centre focal gradient fy']")?;
    if fy_slider.get_attribute("aria-valuetext").as_deref() != Some("25%") {
        return Err(format!(
            "expected initial aria-valuetext \"25%\", got {:?}",
            fy_slider.get_attribute("aria-valuetext")
        ));
    }
    if fy_slider.get_attribute("aria-orientation").as_deref() != Some("vertical") {
        return Err(format!(
            "a rotated <input type=range> stays a horizontal slider to assistive technology without this, got {:?}",
            fy_slider.get_attribute("aria-orientation")
        ));
    }
    // build_v_slider's own inline `style="width:..."` (its pre-rotation length) must track this caller's own
    // track length, not the `.demo-slider-vertical` CSS class's own residual value. Without this assertion, a
    // regression that dropped the inline style would silently fall back to that class's own width and every
    // other assertion here would still pass, since none of them checks the track's own physical length.
    let expected_fy_width = format!("width:{rect_h}px");
    if fy_slider.get_attribute("style").as_deref() != Some(expected_fy_width.as_str()) {
        return Err(format!(
            "the fy track's own inline width should match the focal rectangle's real height, not a hard-coded \
             value, got {:?}",
            fy_slider.get_attribute("style")
        ));
    }
    dispatch_input(&fy_slider, "60")?;
    if f_gradient.get_attribute("fy").as_deref() != Some("0.60") {
        return Err(format!(
            "moving the fy slider should update the gradient's fy, got {:?}",
            f_gradient.get_attribute("fy")
        ));
    }
    if fy_slider.get_attribute("aria-valuetext").as_deref() != Some("60%") {
        return Err(format!(
            "expected aria-valuetext \"60%\", got {:?}",
            fy_slider.get_attribute("aria-valuetext")
        ));
    }
    if (marker_cy(&focal_marker)? - (rect_y + 0.60 * rect_h)).abs() >= 0.1 {
        return Err("the focal marker's cy should follow the fy slider".to_owned());
    }
    if (marker_cx(&focal_marker)? - (rect_x + 0.60 * rect_w)).abs() >= 0.1 {
        return Err("the focal marker's cx should stay put while only fy moves".to_owned());
    }

    // The fy slider's own keydown handler remaps ArrowUp/ArrowDown to match the visual "up is smaller" scale, the
    // same reason and mechanism `demo_linear_gradient`'s own vertical slider needs it. A synthetic keydown
    // dispatch never triggers a browser's native default action in the first place, so this exercises only the
    // demo's own handler, not any native fallback behaviour.
    let dispatch_keydown = |slider: &web_sys::HtmlInputElement, key: &str| -> Result<(), String> {
        let init = web_sys::KeyboardEventInit::new();
        init.set_key(key);
        let event = web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
            .map_err(|e| format!("create keydown event: {e:?}"))?;
        slider.dispatch_event(&event).map_err(|e| format!("dispatch keydown: {e:?}"))?;
        Ok(())
    };

    dispatch_keydown(&fy_slider, "ArrowUp")?;
    if fy_slider.value() != "59" {
        return Err(format!(
            "ArrowUp should decrement, matching the visual up-is-smaller scale, got {:?}",
            fy_slider.value()
        ));
    }
    if f_gradient.get_attribute("fy").as_deref() != Some("0.59") {
        return Err(format!("expected fy \"0.59\", got {:?}", f_gradient.get_attribute("fy")));
    }
    if fy_slider.get_attribute("aria-valuetext").as_deref() != Some("59%") {
        return Err(format!(
            "expected aria-valuetext \"59%\", got {:?}",
            fy_slider.get_attribute("aria-valuetext")
        ));
    }

    dispatch_keydown(&fy_slider, "ArrowDown")?;
    if fy_slider.value() != "60" {
        return Err(format!(
            "ArrowDown should increment, matching the visual down-is-larger scale, got {:?}",
            fy_slider.value()
        ));
    }
    if f_gradient.get_attribute("fy").as_deref() != Some("0.60") {
        return Err(format!("expected fy \"0.60\", got {:?}", f_gradient.get_attribute("fy")));
    }
    if fy_slider.get_attribute("aria-valuetext").as_deref() != Some("60%") {
        return Err(format!(
            "expected aria-valuetext \"60%\", got {:?}",
            fy_slider.get_attribute("aria-valuetext")
        ));
    }

    // --- ellipse metallic sheen: untouched by every control above ---
    let e_stop_1 = find_el("#demo-rg-e stop:nth-child(1)")?;
    let e_stop_2 = find_el("#demo-rg-e stop:nth-child(2)")?;
    let e_stop_3 = find_el("#demo-rg-e stop:nth-child(3)")?;
    if e_stop_1.get_attribute("offset").as_deref() != Some("0") {
        return Err(format!(
            "expected e_stop_1 offset \"0\", got {:?}",
            e_stop_1.get_attribute("offset")
        ));
    }
    if e_stop_1.get_attribute("stop-color").as_deref() != Some("white") {
        return Err(format!(
            "expected e_stop_1 stop-color \"white\", got {:?}",
            e_stop_1.get_attribute("stop-color")
        ));
    }
    if e_stop_2.get_attribute("offset").as_deref() != Some("0.4") {
        return Err(format!(
            "expected e_stop_2 offset \"0.4\", got {:?}",
            e_stop_2.get_attribute("offset")
        ));
    }
    if e_stop_2.get_attribute("stop-color").as_deref() != Some("mediumseagreen") {
        return Err(format!(
            "expected e_stop_2 stop-color \"mediumseagreen\", got {:?}",
            e_stop_2.get_attribute("stop-color")
        ));
    }
    if e_stop_3.get_attribute("offset").as_deref() != Some("1") {
        return Err(format!(
            "expected e_stop_3 offset \"1\", got {:?}",
            e_stop_3.get_attribute("offset")
        ));
    }
    if e_stop_3.get_attribute("stop-color").as_deref() != Some("#003d1f") {
        return Err(format!(
            "expected e_stop_3 stop-color \"#003d1f\", got {:?}",
            e_stop_3.get_attribute("stop-color")
        ));
    }
    Ok(())
}
