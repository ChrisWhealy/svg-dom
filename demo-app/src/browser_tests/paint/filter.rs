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
fn demo_filter_sliders_update_blur_and_drop_shadow_independently() -> Result<(), String> {
    container("demo-filter");
    crate::paint::demo_filter::demo().map_err(|e| format!("demo_filter::demo should build without error: {e:?}"))?;

    let root = document()
        .get_element_by_id("demo-filter")
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

    // No id distinguishes either live caption from any other <text>, so both are found by their own initial
    // content, the same way other tests in this file find theirs.
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

    // --- blur circle: the slider drives #demo-filter-blur's feGaussianBlur stdDeviation ---
    let blur = find_el("#demo-filter-blur feGaussianBlur")?;
    if blur.get_attribute("stdDeviation").as_deref() != Some("3") {
        return Err(format!(
            "3 is this demo's own initial default, got {:?}",
            blur.get_attribute("stdDeviation")
        ));
    }

    let blur_slider = find_slider("input[aria-label='gaussian blur standard deviation']")?;
    // Set at construction time, before any interaction, so a screen reader announces the real starting value.
    if blur_slider.get_attribute("aria-valuetext").as_deref() != Some("stdDeviation 3") {
        return Err(format!(
            "expected aria-valuetext \"stdDeviation 3\", got {:?}",
            blur_slider.get_attribute("aria-valuetext")
        ));
    }
    // Pins the slider's own reachable range to demo_filter.rs's MIN_BLUR/MAX_BLUR — the same bounds the blur
    // filter's own region margin below is sized against. If a future edit widened this range without widening
    // that margin to match, this assertion is what would catch the mismatch, not the region check alone.
    if blur_slider.min() != "0" {
        return Err(format!("expected min \"0\", got {:?}", blur_slider.min()));
    }
    if blur_slider.max() != "20" {
        return Err(format!("expected max \"20\", got {:?}", blur_slider.max()));
    }

    let blur_caption = find_text("stdDeviation: 3")?;

    // The circle's own filter region must stay wide enough for MAX_BLUR (20), not the SVG default: margin =
    // BLUR_SPREAD(3) * MAX_BLUR(20) / (2*CIRCLE_R(45)) = 60/90, as an objectBoundingBox fraction.
    let blur_filter = find_el("#demo-filter-blur")?;
    if blur_filter.get_attribute("filterUnits").is_some() {
        return Err("the blur filter should keep the SVG default objectBoundingBox filterUnits".to_owned());
    }
    let parse_region_attr = |el: &web_sys::Element, attr: &str| -> Result<f64, String> {
        el.get_attribute(attr)
            .ok_or_else(|| format!("missing {attr}"))?
            .parse()
            .map_err(|e| format!("{attr} is not numeric: {e:?}"))
    };
    let expected_circle_margin = 60.0_f64 / 90.0;
    if (parse_region_attr(&blur_filter, "x")? + expected_circle_margin).abs() >= 1e-6 {
        return Err("blur filter region x should be -margin".to_owned());
    }
    if (parse_region_attr(&blur_filter, "y")? + expected_circle_margin).abs() >= 1e-6 {
        return Err("blur filter region y should be -margin".to_owned());
    }
    if (parse_region_attr(&blur_filter, "width")? - (1.0 + 2.0 * expected_circle_margin)).abs() >= 1e-6 {
        return Err("blur filter region width should be 1 + 2*margin".to_owned());
    }
    if (parse_region_attr(&blur_filter, "height")? - (1.0 + 2.0 * expected_circle_margin)).abs() >= 1e-6 {
        return Err("blur filter region height should be 1 + 2*margin".to_owned());
    }

    dispatch_input(&blur_slider, "12")?;
    if blur.get_attribute("stdDeviation").as_deref() != Some("12") {
        return Err(format!(
            "moving the slider should update stdDeviation, got {:?}",
            blur.get_attribute("stdDeviation")
        ));
    }
    if blur_slider.get_attribute("aria-valuetext").as_deref() != Some("stdDeviation 12") {
        return Err(format!(
            "expected aria-valuetext \"stdDeviation 12\", got {:?}",
            blur_slider.get_attribute("aria-valuetext")
        ));
    }
    if blur_caption.text_content().as_deref() != Some("stdDeviation: 12") {
        return Err(format!(
            "expected caption \"stdDeviation: 12\", got {:?}",
            blur_caption.text_content()
        ));
    }

    // --- drop-shadow banner: dx, dy, and stdDeviation each have their own slider on #demo-filter-shadow's
    // feDropShadow, and none of them affect the blur circle above ---
    let shadow = find_el("#demo-filter-shadow feDropShadow")?;
    if shadow.get_attribute("stdDeviation").as_deref() != Some("4") {
        return Err(format!(
            "4 is this demo's own initial default, got {:?}",
            shadow.get_attribute("stdDeviation")
        ));
    }
    if shadow.get_attribute("dx").as_deref() != Some("6") {
        return Err(format!(
            "6 is this demo's own initial default, got {:?}",
            shadow.get_attribute("dx")
        ));
    }
    if shadow.get_attribute("dy").as_deref() != Some("6") {
        return Err(format!(
            "6 is this demo's own initial default, got {:?}",
            shadow.get_attribute("dy")
        ));
    }
    if shadow.get_attribute("flood-color").as_deref() != Some("crimson") {
        return Err(format!(
            "expected flood-color \"crimson\", got {:?}",
            shadow.get_attribute("flood-color")
        ));
    }
    if shadow.get_attribute("flood-opacity").as_deref() != Some("0.85") {
        return Err(format!(
            "expected flood-opacity \"0.85\", got {:?}",
            shadow.get_attribute("flood-opacity")
        ));
    }

    // The banner's own filter region uses filterUnits="userSpaceOnUse" with an absolute region computed from
    // this demo's own SHADOW_BOX_X/Y/W/H layout constants, widened by margin = BLUR_SPREAD(3) *
    // MAX_SHADOW_BLUR(20) + MAX_OFFSET(10) = 70 on every side, wide enough for the worst-case blur/offset
    // combination the sliders below allow — not the SVG default, and not an objectBoundingBox guess against the
    // text's own unmeasured bbox.
    let shadow_filter = find_el("#demo-filter-shadow")?;
    if shadow_filter.get_attribute("filterUnits").as_deref() != Some("userSpaceOnUse") {
        return Err(format!(
            "expected filterUnits \"userSpaceOnUse\", got {:?}",
            shadow_filter.get_attribute("filterUnits")
        ));
    }
    if shadow_filter.get_attribute("x").as_deref() != Some("230") {
        return Err(format!(
            "300 (SHADOW_BOX_X) - 70 (margin), got {:?}",
            shadow_filter.get_attribute("x")
        ));
    }
    if shadow_filter.get_attribute("y").as_deref() != Some("8") {
        return Err(format!(
            "78 (SHADOW_BOX_Y) - 70 (margin), got {:?}",
            shadow_filter.get_attribute("y")
        ));
    }
    if shadow_filter.get_attribute("width").as_deref() != Some("420") {
        return Err(format!(
            "280 (SHADOW_BOX_W) + 2*70 (margin), got {:?}",
            shadow_filter.get_attribute("width")
        ));
    }
    if shadow_filter.get_attribute("height").as_deref() != Some("200") {
        return Err(format!(
            "60 (SHADOW_BOX_H) + 2*70 (margin), got {:?}",
            shadow_filter.get_attribute("height")
        ));
    }

    let dx_slider = find_slider("input[aria-label='drop shadow dx offset']")?;
    let dy_slider = find_slider("input[aria-label='drop shadow dy offset']")?;
    let stddev_slider = find_slider("input[aria-label='drop shadow standard deviation']")?;
    if dx_slider.get_attribute("aria-valuetext").as_deref() != Some("dx 6") {
        return Err(format!(
            "expected aria-valuetext \"dx 6\", got {:?}",
            dx_slider.get_attribute("aria-valuetext")
        ));
    }
    if dy_slider.get_attribute("aria-valuetext").as_deref() != Some("dy 6") {
        return Err(format!(
            "expected aria-valuetext \"dy 6\", got {:?}",
            dy_slider.get_attribute("aria-valuetext")
        ));
    }
    if stddev_slider.get_attribute("aria-valuetext").as_deref() != Some("stdDeviation 4") {
        return Err(format!(
            "expected aria-valuetext \"stdDeviation 4\", got {:?}",
            stddev_slider.get_attribute("aria-valuetext")
        ));
    }
    if dy_slider.get_attribute("aria-orientation").as_deref() != Some("vertical") {
        return Err(format!(
            "a rotated <input type=range> stays a horizontal slider to assistive technology without this, got {:?}",
            dy_slider.get_attribute("aria-orientation")
        ));
    }
    // Pins each slider's own reachable range to demo_filter.rs's MIN_OFFSET/MAX_OFFSET/MIN_SHADOW_BLUR/
    // MAX_SHADOW_BLUR — the same bounds the shadow filter's own region margin above is sized against, for the
    // same reason the blur slider's own min/max are pinned above.
    if dx_slider.min() != "-10" {
        return Err(format!("expected min \"-10\", got {:?}", dx_slider.min()));
    }
    if dx_slider.max() != "10" {
        return Err(format!("expected max \"10\", got {:?}", dx_slider.max()));
    }
    if dy_slider.min() != "-10" {
        return Err(format!("expected min \"-10\", got {:?}", dy_slider.min()));
    }
    if dy_slider.max() != "10" {
        return Err(format!("expected max \"10\", got {:?}", dy_slider.max()));
    }
    if stddev_slider.min() != "0" {
        return Err(format!("expected min \"0\", got {:?}", stddev_slider.min()));
    }
    if stddev_slider.max() != "20" {
        return Err(format!("expected max \"20\", got {:?}", stddev_slider.max()));
    }

    let shadow_caption = find_text("dx 6 · dy 6 · stdDeviation 4")?;

    dispatch_input(&dx_slider, "-8")?;
    if shadow.get_attribute("dx").as_deref() != Some("-8") {
        return Err(format!(
            "moving dx should update the shadow's own dx, got {:?}",
            shadow.get_attribute("dx")
        ));
    }
    if dx_slider.get_attribute("aria-valuetext").as_deref() != Some("dx -8") {
        return Err(format!(
            "expected aria-valuetext \"dx -8\", got {:?}",
            dx_slider.get_attribute("aria-valuetext")
        ));
    }
    if shadow_caption.text_content().as_deref() != Some("dx -8 · dy 6 · stdDeviation 4") {
        return Err(format!(
            "expected caption \"dx -8 · dy 6 · stdDeviation 4\", got {:?}",
            shadow_caption.text_content()
        ));
    }
    if shadow.get_attribute("dy").as_deref() != Some("6") {
        return Err(format!(
            "dy must stay put while only dx moves, got {:?}",
            shadow.get_attribute("dy")
        ));
    }
    if shadow.get_attribute("stdDeviation").as_deref() != Some("4") {
        return Err(format!(
            "stdDeviation must stay put while only dx moves, got {:?}",
            shadow.get_attribute("stdDeviation")
        ));
    }

    dispatch_input(&dy_slider, "9")?;
    if shadow.get_attribute("dy").as_deref() != Some("9") {
        return Err(format!(
            "moving dy should update the shadow's own dy, got {:?}",
            shadow.get_attribute("dy")
        ));
    }
    if dy_slider.get_attribute("aria-valuetext").as_deref() != Some("dy 9") {
        return Err(format!(
            "expected aria-valuetext \"dy 9\", got {:?}",
            dy_slider.get_attribute("aria-valuetext")
        ));
    }
    if shadow_caption.text_content().as_deref() != Some("dx -8 · dy 9 · stdDeviation 4") {
        return Err(format!(
            "expected caption \"dx -8 · dy 9 · stdDeviation 4\", got {:?}",
            shadow_caption.text_content()
        ));
    }
    if shadow.get_attribute("dx").as_deref() != Some("-8") {
        return Err(format!(
            "dx must stay put while only dy moves, got {:?}",
            shadow.get_attribute("dx")
        ));
    }

    // The dy slider's own keydown handler remaps ArrowUp/ArrowDown to match the visual "up is smaller" scale, the
    // same reason and mechanism `demo_radial_gradient`'s own fy slider needs it. A synthetic keydown dispatch
    // never triggers a browser's native default action in the first place, so this exercises only the demo's own
    // handler, not any native fallback behaviour.
    let dispatch_keydown = |slider: &web_sys::HtmlInputElement, key: &str| -> Result<(), String> {
        let init = web_sys::KeyboardEventInit::new();
        init.set_key(key);
        let event = web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
            .map_err(|e| format!("create keydown event: {e:?}"))?;
        slider.dispatch_event(&event).map_err(|e| format!("dispatch keydown: {e:?}"))?;
        Ok(())
    };

    dispatch_keydown(&dy_slider, "ArrowUp")?;
    if dy_slider.value() != "8" {
        return Err(format!(
            "ArrowUp should decrement, matching the visual up-is-smaller scale, got {:?}",
            dy_slider.value()
        ));
    }
    if shadow.get_attribute("dy").as_deref() != Some("8") {
        return Err(format!("expected dy \"8\", got {:?}", shadow.get_attribute("dy")));
    }
    if dy_slider.get_attribute("aria-valuetext").as_deref() != Some("dy 8") {
        return Err(format!(
            "expected aria-valuetext \"dy 8\", got {:?}",
            dy_slider.get_attribute("aria-valuetext")
        ));
    }

    dispatch_keydown(&dy_slider, "ArrowDown")?;
    if dy_slider.value() != "9" {
        return Err(format!(
            "ArrowDown should increment, matching the visual down-is-larger scale, got {:?}",
            dy_slider.value()
        ));
    }
    if shadow.get_attribute("dy").as_deref() != Some("9") {
        return Err(format!("expected dy \"9\", got {:?}", shadow.get_attribute("dy")));
    }
    if dy_slider.get_attribute("aria-valuetext").as_deref() != Some("dy 9") {
        return Err(format!(
            "expected aria-valuetext \"dy 9\", got {:?}",
            dy_slider.get_attribute("aria-valuetext")
        ));
    }

    dispatch_input(&stddev_slider, "15")?;
    if shadow.get_attribute("stdDeviation").as_deref() != Some("15") {
        return Err(format!(
            "moving the shadow's own blur slider should update its stdDeviation, got {:?}",
            shadow.get_attribute("stdDeviation")
        ));
    }
    if stddev_slider.get_attribute("aria-valuetext").as_deref() != Some("stdDeviation 15") {
        return Err(format!(
            "expected aria-valuetext \"stdDeviation 15\", got {:?}",
            stddev_slider.get_attribute("aria-valuetext")
        ));
    }
    if shadow_caption.text_content().as_deref() != Some("dx -8 · dy 9 · stdDeviation 15") {
        return Err(format!(
            "expected caption \"dx -8 · dy 9 · stdDeviation 15\", got {:?}",
            shadow_caption.text_content()
        ));
    }
    if shadow.get_attribute("dx").as_deref() != Some("-8") {
        return Err(format!(
            "dx must stay put while only stdDeviation moves, got {:?}",
            shadow.get_attribute("dx")
        ));
    }
    if shadow.get_attribute("dy").as_deref() != Some("9") {
        return Err(format!(
            "dy must stay put while only stdDeviation moves, got {:?}",
            shadow.get_attribute("dy")
        ));
    }

    // --- the blur circle and the drop-shadow banner never touch one another's filter ---
    if blur.get_attribute("stdDeviation").as_deref() != Some("12") {
        return Err(format!(
            "the blur circle's own stdDeviation must stay at its own last value, untouched by any shadow slider, \
             got {:?}",
            blur.get_attribute("stdDeviation")
        ));
    }
    if blur_caption.text_content().as_deref() != Some("stdDeviation: 12") {
        return Err(format!(
            "expected caption \"stdDeviation: 12\", got {:?}",
            blur_caption.text_content()
        ));
    }

    // --- every slider at its documented maximum simultaneously: the combination the filter regions above are
    // sized for, per demo_filter.rs's own BLUR_SPREAD/MAX_ margin comments ---
    // This proves the DOM state — the attribute values a browser's own renderer reads to paint the effect —
    // matches the documented worst case, and that neither region is recomputed (and so silently narrowed) as a
    // slider's live value changes rather than staying fixed at the build-time worst case. It cannot by itself
    // prove no pixel is actually clipped at that combination; only a rendered, visually inspected browser can
    // show that (confirmed separately, manually, during this demo's own development).
    dispatch_input(&blur_slider, "20")?;
    dispatch_input(&dx_slider, "10")?;
    dispatch_input(&dy_slider, "10")?;
    dispatch_input(&stddev_slider, "20")?;
    if blur.get_attribute("stdDeviation").as_deref() != Some("20") {
        return Err(format!(
            "expected stdDeviation \"20\", got {:?}",
            blur.get_attribute("stdDeviation")
        ));
    }
    if shadow.get_attribute("dx").as_deref() != Some("10") {
        return Err(format!("expected dx \"10\", got {:?}", shadow.get_attribute("dx")));
    }
    if shadow.get_attribute("dy").as_deref() != Some("10") {
        return Err(format!("expected dy \"10\", got {:?}", shadow.get_attribute("dy")));
    }
    if shadow.get_attribute("stdDeviation").as_deref() != Some("20") {
        return Err(format!(
            "expected stdDeviation \"20\", got {:?}",
            shadow.get_attribute("stdDeviation")
        ));
    }
    if (parse_region_attr(&blur_filter, "x")? + expected_circle_margin).abs() >= 1e-6 {
        return Err(
            "the blur filter's own region must stay fixed at its build-time worst-case size, not shrink at the \
             live value"
                .to_owned(),
        );
    }
    if (parse_region_attr(&blur_filter, "y")? + expected_circle_margin).abs() >= 1e-6 {
        return Err("blur filter region y should stay fixed too".to_owned());
    }
    if (parse_region_attr(&blur_filter, "width")? - (1.0 + 2.0 * expected_circle_margin)).abs() >= 1e-6 {
        return Err("blur filter region width should stay fixed too".to_owned());
    }
    if (parse_region_attr(&blur_filter, "height")? - (1.0 + 2.0 * expected_circle_margin)).abs() >= 1e-6 {
        return Err("blur filter region height should stay fixed too".to_owned());
    }
    if shadow_filter.get_attribute("x").as_deref() != Some("230") {
        return Err(format!(
            "the shadow filter's own region must stay fixed at its build-time worst-case size too, got {:?}",
            shadow_filter.get_attribute("x")
        ));
    }
    if shadow_filter.get_attribute("y").as_deref() != Some("8") {
        return Err(format!("expected y \"8\", got {:?}", shadow_filter.get_attribute("y")));
    }
    if shadow_filter.get_attribute("width").as_deref() != Some("420") {
        return Err(format!(
            "expected width \"420\", got {:?}",
            shadow_filter.get_attribute("width")
        ));
    }
    if shadow_filter.get_attribute("height").as_deref() != Some("200") {
        return Err(format!(
            "expected height \"200\", got {:?}",
            shadow_filter.get_attribute("height")
        ));
    }

    let banner = find_el("text[font-weight='bold']")?;
    if banner.text_content().as_deref() != Some("DROP SHADOW") {
        return Err(format!(
            "expected text content \"DROP SHADOW\", got {:?}",
            banner.text_content()
        ));
    }
    if banner.get_attribute("filter").as_deref() != Some("url(#demo-filter-shadow)") {
        return Err(format!(
            "expected filter \"url(#demo-filter-shadow)\", got {:?}",
            banner.get_attribute("filter")
        ));
    }
    Ok(())
}
