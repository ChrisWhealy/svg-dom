//! Tests for `demo_filter`'s own four sliders: blur, dx, dy, and drop-shadow blur.

use crate::browser_tests::{container, document};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Unlike `demo_linear_gradient`/`demo_radial_gradient`, `demo_filter` needs no `select_el` escape hatch:
/// `gaussian_blur`/`drop_shadow` both return a live `SvgNode`, which the demo retains directly and updates via
/// `set_attr`. Source extraction cannot prove any slider still reaches its own target attribute through that
/// retained node, or that the two live captions still track them.
/// It also cannot prove the four controls stay independent of one another.
///
/// It also cannot prove either filter's own region is actually sized against the sliders' own documented
/// extremes, not just the SVG default: this test pins both regions' `x`/`y`/`width`/`height` (and `filterUnits`),
/// pins each slider's own `min`/`max` to the same bounds those regions are sized against, and finally drives
/// every slider to its documented maximum simultaneously to confirm neither region shrinks at that point. That
/// proves the DOM state matches the intended worst case; it cannot by itself prove no pixel is actually clipped
/// at that combination, which only a rendered, visually inspected browser can show.
/// Only a real browser can prove any of the above.
#[wasm_bindgen_test]
fn demo_filter_sliders_update_blur_and_drop_shadow_independently() {
    container("demo-filter");
    crate::paint::demo_filter::demo().expect("demo_filter::demo should build without error");

    let root = document().get_element_by_id("demo-filter").expect("container exists");

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

    // No id distinguishes either live caption from any other <text>, so both are found by their own initial
    // content, the same way other tests in this file find theirs.
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

    // --- blur circle: the slider drives #demo-filter-blur's feGaussianBlur stdDeviation ---
    let blur = find_el("#demo-filter-blur feGaussianBlur");
    assert_eq!(
        blur.get_attribute("stdDeviation").as_deref(),
        Some("3"),
        "3 is this demo's own initial default"
    );

    let blur_slider = find_slider("input[aria-label='gaussian blur standard deviation']");
    // Set at construction time, before any interaction, so a screen reader announces the real starting value.
    assert_eq!(blur_slider.get_attribute("aria-valuetext").as_deref(), Some("stdDeviation 3"));
    // Pins the slider's own reachable range to demo_filter.rs's MIN_BLUR/MAX_BLUR — the same bounds the blur
    // filter's own region margin below is sized against. If a future edit widened this range without widening
    // that margin to match, this assertion is what would catch the mismatch, not the region check alone.
    assert_eq!(blur_slider.min(), "0");
    assert_eq!(blur_slider.max(), "20");

    let blur_caption = find_text("stdDeviation: 3");

    // The circle's own filter region must stay wide enough for MAX_BLUR (20), not the SVG default: margin =
    // BLUR_SPREAD(3) * MAX_BLUR(20) / (2*CIRCLE_R(45)) = 60/90, as an objectBoundingBox fraction.
    let blur_filter = find_el("#demo-filter-blur");
    assert!(
        blur_filter.get_attribute("filterUnits").is_none(),
        "the blur filter should keep the SVG default objectBoundingBox filterUnits"
    );
    let parse_region_attr = |el: &web_sys::Element, attr: &str| -> f64 {
        el.get_attribute(attr)
            .unwrap_or_else(|| panic!("missing {attr}"))
            .parse()
            .unwrap_or_else(|_| panic!("{attr} is not numeric"))
    };
    let expected_circle_margin = 60.0_f64 / 90.0;
    assert!(
        (parse_region_attr(&blur_filter, "x") + expected_circle_margin).abs() < 1e-6,
        "blur filter region x should be -margin"
    );
    assert!(
        (parse_region_attr(&blur_filter, "y") + expected_circle_margin).abs() < 1e-6,
        "blur filter region y should be -margin"
    );
    assert!(
        (parse_region_attr(&blur_filter, "width") - (1.0 + 2.0 * expected_circle_margin)).abs() < 1e-6,
        "blur filter region width should be 1 + 2*margin"
    );
    assert!(
        (parse_region_attr(&blur_filter, "height") - (1.0 + 2.0 * expected_circle_margin)).abs() < 1e-6,
        "blur filter region height should be 1 + 2*margin"
    );

    dispatch_input(&blur_slider, "12");
    assert_eq!(
        blur.get_attribute("stdDeviation").as_deref(),
        Some("12"),
        "moving the slider should update stdDeviation"
    );
    assert_eq!(blur_slider.get_attribute("aria-valuetext").as_deref(), Some("stdDeviation 12"));
    assert_eq!(blur_caption.text_content().as_deref(), Some("stdDeviation: 12"));

    // --- drop-shadow banner: dx, dy, and stdDeviation each have their own slider on #demo-filter-shadow's
    // feDropShadow, and none of them affect the blur circle above ---
    let shadow = find_el("#demo-filter-shadow feDropShadow");
    assert_eq!(
        shadow.get_attribute("stdDeviation").as_deref(),
        Some("4"),
        "4 is this demo's own initial default"
    );
    assert_eq!(
        shadow.get_attribute("dx").as_deref(),
        Some("6"),
        "6 is this demo's own initial default"
    );
    assert_eq!(
        shadow.get_attribute("dy").as_deref(),
        Some("6"),
        "6 is this demo's own initial default"
    );
    assert_eq!(shadow.get_attribute("flood-color").as_deref(), Some("crimson"));
    assert_eq!(shadow.get_attribute("flood-opacity").as_deref(), Some("0.85"));

    // The banner's own filter region uses filterUnits="userSpaceOnUse" with an absolute region computed from
    // this demo's own SHADOW_BOX_X/Y/W/H layout constants, widened by margin = BLUR_SPREAD(3) *
    // MAX_SHADOW_BLUR(20) + MAX_OFFSET(10) = 70 on every side, wide enough for the worst-case blur/offset
    // combination the sliders below allow — not the SVG default, and not an objectBoundingBox guess against the
    // text's own unmeasured bbox.
    let shadow_filter = find_el("#demo-filter-shadow");
    assert_eq!(shadow_filter.get_attribute("filterUnits").as_deref(), Some("userSpaceOnUse"));
    assert_eq!(
        shadow_filter.get_attribute("x").as_deref(),
        Some("230"),
        "300 (SHADOW_BOX_X) - 70 (margin)"
    );
    assert_eq!(
        shadow_filter.get_attribute("y").as_deref(),
        Some("8"),
        "78 (SHADOW_BOX_Y) - 70 (margin)"
    );
    assert_eq!(
        shadow_filter.get_attribute("width").as_deref(),
        Some("420"),
        "280 (SHADOW_BOX_W) + 2*70 (margin)"
    );
    assert_eq!(
        shadow_filter.get_attribute("height").as_deref(),
        Some("200"),
        "60 (SHADOW_BOX_H) + 2*70 (margin)"
    );

    let dx_slider = find_slider("input[aria-label='drop shadow dx offset']");
    let dy_slider = find_slider("input[aria-label='drop shadow dy offset']");
    let stddev_slider = find_slider("input[aria-label='drop shadow standard deviation']");
    assert_eq!(dx_slider.get_attribute("aria-valuetext").as_deref(), Some("dx 6"));
    assert_eq!(dy_slider.get_attribute("aria-valuetext").as_deref(), Some("dy 6"));
    assert_eq!(stddev_slider.get_attribute("aria-valuetext").as_deref(), Some("stdDeviation 4"));
    assert_eq!(
        dy_slider.get_attribute("aria-orientation").as_deref(),
        Some("vertical"),
        "a rotated <input type=range> stays a horizontal slider to assistive technology without this"
    );
    // Pins each slider's own reachable range to demo_filter.rs's MIN_OFFSET/MAX_OFFSET/MIN_SHADOW_BLUR/
    // MAX_SHADOW_BLUR — the same bounds the shadow filter's own region margin above is sized against, for the
    // same reason the blur slider's own min/max are pinned above.
    assert_eq!(dx_slider.min(), "-10");
    assert_eq!(dx_slider.max(), "10");
    assert_eq!(dy_slider.min(), "-10");
    assert_eq!(dy_slider.max(), "10");
    assert_eq!(stddev_slider.min(), "0");
    assert_eq!(stddev_slider.max(), "20");

    let shadow_caption = find_text("dx 6 · dy 6 · stdDeviation 4");

    dispatch_input(&dx_slider, "-8");
    assert_eq!(
        shadow.get_attribute("dx").as_deref(),
        Some("-8"),
        "moving dx should update the shadow's own dx"
    );
    assert_eq!(dx_slider.get_attribute("aria-valuetext").as_deref(), Some("dx -8"));
    assert_eq!(shadow_caption.text_content().as_deref(), Some("dx -8 · dy 6 · stdDeviation 4"));
    assert_eq!(
        shadow.get_attribute("dy").as_deref(),
        Some("6"),
        "dy must stay put while only dx moves"
    );
    assert_eq!(
        shadow.get_attribute("stdDeviation").as_deref(),
        Some("4"),
        "stdDeviation must stay put while only dx moves"
    );

    dispatch_input(&dy_slider, "9");
    assert_eq!(
        shadow.get_attribute("dy").as_deref(),
        Some("9"),
        "moving dy should update the shadow's own dy"
    );
    assert_eq!(dy_slider.get_attribute("aria-valuetext").as_deref(), Some("dy 9"));
    assert_eq!(shadow_caption.text_content().as_deref(), Some("dx -8 · dy 9 · stdDeviation 4"));
    assert_eq!(
        shadow.get_attribute("dx").as_deref(),
        Some("-8"),
        "dx must stay put while only dy moves"
    );

    // The dy slider's own keydown handler remaps ArrowUp/ArrowDown to match the visual "up is smaller" scale, the
    // same reason and mechanism `demo_radial_gradient`'s own fy slider needs it. A synthetic keydown dispatch
    // never triggers a browser's native default action in the first place, so this exercises only the demo's own
    // handler, not any native fallback behaviour.
    let dispatch_keydown = |slider: &web_sys::HtmlInputElement, key: &str| {
        let init = web_sys::KeyboardEventInit::new();
        init.set_key(key);
        let event =
            web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).expect("create keydown event");
        slider.dispatch_event(&event).expect("dispatch keydown");
    };

    dispatch_keydown(&dy_slider, "ArrowUp");
    assert_eq!(
        dy_slider.value(),
        "8",
        "ArrowUp should decrement, matching the visual up-is-smaller scale"
    );
    assert_eq!(shadow.get_attribute("dy").as_deref(), Some("8"));
    assert_eq!(dy_slider.get_attribute("aria-valuetext").as_deref(), Some("dy 8"));

    dispatch_keydown(&dy_slider, "ArrowDown");
    assert_eq!(
        dy_slider.value(),
        "9",
        "ArrowDown should increment, matching the visual down-is-larger scale"
    );
    assert_eq!(shadow.get_attribute("dy").as_deref(), Some("9"));
    assert_eq!(dy_slider.get_attribute("aria-valuetext").as_deref(), Some("dy 9"));

    dispatch_input(&stddev_slider, "15");
    assert_eq!(
        shadow.get_attribute("stdDeviation").as_deref(),
        Some("15"),
        "moving the shadow's own blur slider should update its stdDeviation"
    );
    assert_eq!(
        stddev_slider.get_attribute("aria-valuetext").as_deref(),
        Some("stdDeviation 15")
    );
    assert_eq!(shadow_caption.text_content().as_deref(), Some("dx -8 · dy 9 · stdDeviation 15"));
    assert_eq!(
        shadow.get_attribute("dx").as_deref(),
        Some("-8"),
        "dx must stay put while only stdDeviation moves"
    );
    assert_eq!(
        shadow.get_attribute("dy").as_deref(),
        Some("9"),
        "dy must stay put while only stdDeviation moves"
    );

    // --- the blur circle and the drop-shadow banner never touch one another's filter ---
    assert_eq!(
        blur.get_attribute("stdDeviation").as_deref(),
        Some("12"),
        "the blur circle's own stdDeviation must stay at its own last value, untouched by any shadow slider"
    );
    assert_eq!(blur_caption.text_content().as_deref(), Some("stdDeviation: 12"));

    // --- every slider at its documented maximum simultaneously: the combination the filter regions above are
    // sized for, per demo_filter.rs's own BLUR_SPREAD/MAX_ margin comments ---
    // This proves the DOM state — the attribute values a browser's own renderer reads to paint the effect —
    // matches the documented worst case, and that neither region is recomputed (and so silently narrowed) as a
    // slider's live value changes rather than staying fixed at the build-time worst case. It cannot by itself
    // prove no pixel is actually clipped at that combination; only a rendered, visually inspected browser can
    // show that (confirmed separately, manually, during this demo's own development).
    dispatch_input(&blur_slider, "20");
    dispatch_input(&dx_slider, "10");
    dispatch_input(&dy_slider, "10");
    dispatch_input(&stddev_slider, "20");
    assert_eq!(blur.get_attribute("stdDeviation").as_deref(), Some("20"));
    assert_eq!(shadow.get_attribute("dx").as_deref(), Some("10"));
    assert_eq!(shadow.get_attribute("dy").as_deref(), Some("10"));
    assert_eq!(shadow.get_attribute("stdDeviation").as_deref(), Some("20"));
    assert!(
        (parse_region_attr(&blur_filter, "x") + expected_circle_margin).abs() < 1e-6,
        "the blur filter's own region must stay fixed at its build-time worst-case size, not shrink at the live value"
    );
    assert!((parse_region_attr(&blur_filter, "y") + expected_circle_margin).abs() < 1e-6);
    assert!((parse_region_attr(&blur_filter, "width") - (1.0 + 2.0 * expected_circle_margin)).abs() < 1e-6);
    assert!((parse_region_attr(&blur_filter, "height") - (1.0 + 2.0 * expected_circle_margin)).abs() < 1e-6);
    assert_eq!(
        shadow_filter.get_attribute("x").as_deref(),
        Some("230"),
        "the shadow filter's own region must stay fixed at its build-time worst-case size too"
    );
    assert_eq!(shadow_filter.get_attribute("y").as_deref(), Some("8"));
    assert_eq!(shadow_filter.get_attribute("width").as_deref(), Some("420"));
    assert_eq!(shadow_filter.get_attribute("height").as_deref(), Some("200"));

    let banner = find_el("text[font-weight='bold']");
    assert_eq!(banner.text_content().as_deref(), Some("DROP SHADOW"));
    assert_eq!(banner.get_attribute("filter").as_deref(), Some("url(#demo-filter-shadow)"));
}
