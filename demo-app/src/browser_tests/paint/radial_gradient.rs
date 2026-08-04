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
fn demo_radial_gradient_controls_update_stops_spread_and_focal_point() {
    container("demo-radial-gradient");
    crate::paint::demo_radial_gradient::demo().expect("demo_radial_gradient::demo should build without error");

    let root = document().get_element_by_id("demo-radial-gradient").expect("container exists");

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

    // --- centred: the slider shifts #demo-rg-c's middle stop along the radius ---
    let c_stop = find_el("#demo-rg-c stop:nth-child(2)");
    assert_eq!(c_stop.get_attribute("offset").as_deref(), Some("0.5"));

    let c_slider = find_slider("input[aria-label='centred gradient stop 2']");
    assert_eq!(c_slider.get_attribute("aria-valuetext").as_deref(), Some("50%"));
    dispatch_input(&c_slider, "70");
    assert_eq!(
        c_stop.get_attribute("offset").as_deref(),
        Some("0.70"),
        "moving the centred slider should update the middle stop's offset"
    );
    assert_eq!(c_slider.get_attribute("aria-valuetext").as_deref(), Some("70%"));

    // --- spreadMethod: radio buttons choose #demo-rg-r's spreadMethod ---
    let r_gradient = find_el("#demo-rg-r");
    assert_eq!(
        r_gradient.get_attribute("spreadMethod").as_deref(),
        Some("reflect"),
        "reflect is this demo's own initial default"
    );

    let pad = find_radio(&root, "spreadMethod", "pad");
    select_radio(&pad);
    assert_eq!(r_gradient.get_attribute("spreadMethod").as_deref(), Some("pad"));

    let repeat = find_radio(&root, "spreadMethod", "repeat");
    select_radio(&repeat);
    assert_eq!(r_gradient.get_attribute("spreadMethod").as_deref(), Some("repeat"));

    let reflect = find_radio(&root, "spreadMethod", "reflect");
    select_radio(&reflect);
    assert_eq!(r_gradient.get_attribute("spreadMethod").as_deref(), Some("reflect"));
    assert!(reflect.checked());
    assert!(!pad.checked(), "selecting reflect should clear pad");
    assert!(!repeat.checked(), "selecting reflect should clear repeat");

    // --- off-centre focal: fx (horizontal) and fy (vertical) sliders move #demo-rg-f's focal point ---
    let f_gradient = find_el("#demo-rg-f");
    assert_eq!(f_gradient.get_attribute("fx").as_deref(), Some("0.25"));
    assert_eq!(f_gradient.get_attribute("fy").as_deref(), Some("0.25"));

    // The focal marker reads its own position straight from the rectangle's live x/y/width/height, rather than
    // this test hard-coding the demo's own layout constants, the same reason `marker_view_box_slider_...` above
    // captures the line's own initial attributes instead of recomputing them.
    let focal_rect = find_el("rect");
    let rect_x: f64 = focal_rect.get_attribute("x").expect("rect x").parse().expect("numeric x");
    let rect_y: f64 = focal_rect.get_attribute("y").expect("rect y").parse().expect("numeric y");
    let rect_w: f64 = focal_rect
        .get_attribute("width")
        .expect("rect width")
        .parse()
        .expect("numeric width");
    let rect_h: f64 = focal_rect
        .get_attribute("height")
        .expect("rect height")
        .parse()
        .expect("numeric height");

    // `fill="none"` is what distinguishes the marker from the two gradient-filled circles elsewhere on this panel.
    let focal_marker = find_el("circle[fill='none']");
    let marker_cx =
        |el: &web_sys::Element| -> f64 { el.get_attribute("cx").expect("marker cx").parse().expect("numeric cx") };
    let marker_cy =
        |el: &web_sys::Element| -> f64 { el.get_attribute("cy").expect("marker cy").parse().expect("numeric cy") };

    assert!((marker_cx(&focal_marker) - (rect_x + 0.25 * rect_w)).abs() < 0.1);
    assert!((marker_cy(&focal_marker) - (rect_y + 0.25 * rect_h)).abs() < 0.1);

    let fx_slider = find_slider("input[aria-label='off-centre focal gradient fx']");
    assert_eq!(fx_slider.get_attribute("aria-valuetext").as_deref(), Some("25%"));
    dispatch_input(&fx_slider, "60");
    assert_eq!(
        f_gradient.get_attribute("fx").as_deref(),
        Some("0.60"),
        "moving the fx slider should update the gradient's fx"
    );
    assert_eq!(fx_slider.get_attribute("aria-valuetext").as_deref(), Some("60%"));
    assert!(
        (marker_cx(&focal_marker) - (rect_x + 0.60 * rect_w)).abs() < 0.1,
        "the focal marker's cx should follow the fx slider"
    );
    assert!(
        (marker_cy(&focal_marker) - (rect_y + 0.25 * rect_h)).abs() < 0.1,
        "the focal marker's cy should stay put while only fx moves"
    );

    let fy_slider = find_slider("input[aria-label='off-centre focal gradient fy']");
    assert_eq!(fy_slider.get_attribute("aria-valuetext").as_deref(), Some("25%"));
    assert_eq!(
        fy_slider.get_attribute("aria-orientation").as_deref(),
        Some("vertical"),
        "a rotated <input type=range> stays a horizontal slider to assistive technology without this"
    );
    // build_v_slider's own inline `style="width:..."` (its pre-rotation length) must track this caller's own
    // track length, not the `.demo-slider-vertical` CSS class's own residual value. Without this assertion, a
    // regression that dropped the inline style would silently fall back to that class's own width and every
    // other assertion here would still pass, since none of them checks the track's own physical length.
    let expected_fy_width = format!("width:{rect_h}px");
    assert_eq!(
        fy_slider.get_attribute("style").as_deref(),
        Some(expected_fy_width.as_str()),
        "the fy track's own inline width should match the focal rectangle's real height, not a hard-coded value"
    );
    dispatch_input(&fy_slider, "60");
    assert_eq!(
        f_gradient.get_attribute("fy").as_deref(),
        Some("0.60"),
        "moving the fy slider should update the gradient's fy"
    );
    assert_eq!(fy_slider.get_attribute("aria-valuetext").as_deref(), Some("60%"));
    assert!(
        (marker_cy(&focal_marker) - (rect_y + 0.60 * rect_h)).abs() < 0.1,
        "the focal marker's cy should follow the fy slider"
    );
    assert!(
        (marker_cx(&focal_marker) - (rect_x + 0.60 * rect_w)).abs() < 0.1,
        "the focal marker's cx should stay put while only fy moves"
    );

    // The fy slider's own keydown handler remaps ArrowUp/ArrowDown to match the visual "up is smaller" scale, the
    // same reason and mechanism `demo_linear_gradient`'s own vertical slider needs it. A synthetic keydown
    // dispatch never triggers a browser's native default action in the first place, so this exercises only the
    // demo's own handler, not any native fallback behaviour.
    let dispatch_keydown = |slider: &web_sys::HtmlInputElement, key: &str| {
        let init = web_sys::KeyboardEventInit::new();
        init.set_key(key);
        let event =
            web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).expect("create keydown event");
        slider.dispatch_event(&event).expect("dispatch keydown");
    };

    dispatch_keydown(&fy_slider, "ArrowUp");
    assert_eq!(
        fy_slider.value(),
        "59",
        "ArrowUp should decrement, matching the visual up-is-smaller scale"
    );
    assert_eq!(f_gradient.get_attribute("fy").as_deref(), Some("0.59"));
    assert_eq!(fy_slider.get_attribute("aria-valuetext").as_deref(), Some("59%"));

    dispatch_keydown(&fy_slider, "ArrowDown");
    assert_eq!(
        fy_slider.value(),
        "60",
        "ArrowDown should increment, matching the visual down-is-larger scale"
    );
    assert_eq!(f_gradient.get_attribute("fy").as_deref(), Some("0.60"));
    assert_eq!(fy_slider.get_attribute("aria-valuetext").as_deref(), Some("60%"));

    // --- ellipse metallic sheen: untouched by every control above ---
    let e_stop_1 = find_el("#demo-rg-e stop:nth-child(1)");
    let e_stop_2 = find_el("#demo-rg-e stop:nth-child(2)");
    let e_stop_3 = find_el("#demo-rg-e stop:nth-child(3)");
    assert_eq!(e_stop_1.get_attribute("offset").as_deref(), Some("0"));
    assert_eq!(e_stop_1.get_attribute("stop-color").as_deref(), Some("white"));
    assert_eq!(e_stop_2.get_attribute("offset").as_deref(), Some("0.4"));
    assert_eq!(e_stop_2.get_attribute("stop-color").as_deref(), Some("mediumseagreen"));
    assert_eq!(e_stop_3.get_attribute("offset").as_deref(), Some("1"));
    assert_eq!(e_stop_3.get_attribute("stop-color").as_deref(), Some("#003d1f"));
}
