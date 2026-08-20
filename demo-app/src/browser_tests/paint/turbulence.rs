//! Tests for `demo_turbulence`'s own baseFrequency slider, shared by both noise rectangles, and its own
//! displacement scale, x-channel, and y-channel sliders.

use crate::browser_tests::{container, document};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `turbulence` and `displacement_map` each return their own primitive's `SvgNode` directly, so `demo_turbulence`
/// retains each one the same way `demo_filter`'s sliders do. Source extraction cannot prove:
/// 1) Any slider actually reaches its own retained node.
/// 2) The frequency slider updates both noise filters together, not just one of them.
/// 3) The displacement filter's own fixed region stays exactly as widened, unaffected by the scale slider's own live
///    value.
/// 4) The x-channel and y-channel sliders actually reach `xChannelSelector`/`yChannelSelector`, with the correct
///    one-letter keyword at every one of their own four positions, not just a sample of them — `CHANNELS[index]`
///    and `Channel::selector_str` translate a numeric HTML value into a categorical SVG keyword, so a wrong
///    mapping at just one position (Blue, say) would not show up from checking only the other three.
/// 5) Two sliders can actually reach the single-diagonal state (`Alpha` for both) that `SvgFilter::displacement_map`'s
///    own doc comment warns about, not just some other, unconstrained combination.
/// 6) All four controls, and the original circle, stay independent of one another.
/// 7) The scale slider actually reaches `scale="0"`, the value `demo/panels/panel-turbulence.html` prominently
///    claims restores a perfect geometric circle.
///
/// Only a real browser can prove any of that. Even a real browser running this file cannot prove point 7's own
/// rendered-pixel half of the claim, since `wasm-bindgen-test`'s WebDriver-run tests have no access to rasterised
/// output — see `cdp-integration-test/tests/turbulence_scale_zero_render.rs` for that half.
#[wasm_bindgen_test]
fn demo_turbulence_sliders_update_frequency_and_displacement_scale_independently() -> Result<(), String> {
    container("demo-turbulence");
    crate::paint::demo_turbulence::demo()
        .map_err(|e| format!("demo_turbulence::demo should build without error: {e:?}"))?;

    let root = document()
        .get_element_by_id("demo-turbulence")
        .ok_or_else(|| "container exists".to_owned())?;

    let find_el = |selector: &str| -> Result<web_sys::Element, String> {
        root.query_selector(selector)
            .map_err(|e| format!("invalid selector {selector:?}: {e:?}"))?
            .ok_or_else(|| format!("no element matching {selector:?}"))
    };

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

    let find_slider = |aria_label_selector: &str| -> Result<web_sys::HtmlInputElement, String> {
        root.query_selector(aria_label_selector)
            .map_err(|e| format!("query slider: {e:?}"))?
            .ok_or_else(|| format!("no slider matching {aria_label_selector:?}"))?
            .dyn_into::<web_sys::HtmlInputElement>()
            .map_err(|_| "slider is an HtmlInputElement".to_owned())
    };

    let dispatch_input = |slider: &web_sys::HtmlInputElement, value: &str| -> Result<(), String> {
        slider.set_value(value);
        let event = web_sys::Event::new("input").map_err(|e| format!("create input event: {e:?}"))?;
        slider.dispatch_event(&event).map_err(|e| format!("dispatch input: {e:?}"))?;
        Ok(())
    };

    // --- both noise filters share the same slider-driven baseFrequency, at this demo's own default ---
    let fractal = find_el("#turbulence-fractal feTurbulence")?;
    if fractal.get_attribute("type").as_deref() != Some("fractalNoise") {
        return Err(format!(
            "expected type \"fractalNoise\", got {:?}",
            fractal.get_attribute("type")
        ));
    }
    if fractal.get_attribute("baseFrequency").as_deref() != Some("0.015") {
        return Err(format!(
            "0.015 is this demo's own initial default, got {:?}",
            fractal.get_attribute("baseFrequency")
        ));
    }
    if fractal.get_attribute("numOctaves").as_deref() != Some("4") {
        return Err(format!(
            "expected numOctaves \"4\", got {:?}",
            fractal.get_attribute("numOctaves")
        ));
    }
    if fractal.get_attribute("seed").as_deref() != Some("3") {
        return Err(format!("expected seed \"3\", got {:?}", fractal.get_attribute("seed")));
    }

    let marbled = find_el("#turbulence-marbled feTurbulence")?;
    if marbled.get_attribute("type").as_deref() != Some("turbulence") {
        return Err(format!("expected type \"turbulence\", got {:?}", marbled.get_attribute("type")));
    }
    if marbled.get_attribute("baseFrequency").as_deref() != Some("0.015") {
        return Err(format!(
            "expected baseFrequency \"0.015\", got {:?}",
            marbled.get_attribute("baseFrequency")
        ));
    }

    let frequency_slider = find_slider("input[aria-label='turbulence base frequency']")?;
    if frequency_slider.get_attribute("min").as_deref() != Some("0") {
        return Err(format!("expected min \"0\", got {:?}", frequency_slider.get_attribute("min")));
    }
    if frequency_slider.get_attribute("max").as_deref() != Some("50") {
        return Err(format!("expected max \"50\", got {:?}", frequency_slider.get_attribute("max")));
    }
    if frequency_slider.value() != "15" {
        return Err(format!("expected value \"15\", got {:?}", frequency_slider.value()));
    }
    if frequency_slider.get_attribute("aria-valuetext").as_deref() != Some("0.015") {
        return Err(format!(
            "expected aria-valuetext \"0.015\", got {:?}",
            frequency_slider.get_attribute("aria-valuetext")
        ));
    }

    let fractal_caption = find_text("FractalNoise (0.015)")?;
    let marbled_caption = find_text("Turbulence (0.015)")?;

    // --- the displacement filter, at this demo's own default ---
    let displace_filter = find_el("#turbulence-displace")?;
    if displace_filter.get_attribute("x").as_deref() != Some("-0.5") {
        return Err(format!(
            "widen_filter_region's own fixed margin, got {:?}",
            displace_filter.get_attribute("x")
        ));
    }
    if displace_filter.get_attribute("y").as_deref() != Some("-0.5") {
        return Err(format!("expected y \"-0.5\", got {:?}", displace_filter.get_attribute("y")));
    }
    if displace_filter.get_attribute("width").as_deref() != Some("2") {
        return Err(format!(
            "expected width \"2\", got {:?}",
            displace_filter.get_attribute("width")
        ));
    }
    if displace_filter.get_attribute("height").as_deref() != Some("2") {
        return Err(format!(
            "expected height \"2\", got {:?}",
            displace_filter.get_attribute("height")
        ));
    }

    let displace_noise = find_el("#turbulence-displace feTurbulence")?;
    if displace_noise.get_attribute("baseFrequency").as_deref() != Some("0.02") {
        return Err(format!(
            "fixed, not driven by either slider, got {:?}",
            displace_noise.get_attribute("baseFrequency")
        ));
    }
    if displace_noise.get_attribute("result").as_deref() != Some("noise") {
        return Err(format!(
            "expected result \"noise\", got {:?}",
            displace_noise.get_attribute("result")
        ));
    }

    let displace_map = find_el("#turbulence-displace feDisplacementMap")?;
    if displace_map.get_attribute("in2").as_deref() != Some("noise") {
        return Err(format!("expected in2 \"noise\", got {:?}", displace_map.get_attribute("in2")));
    }
    if displace_map.get_attribute("in").as_deref() != Some("SourceGraphic") {
        return Err(format!(
            "expected in \"SourceGraphic\", got {:?}",
            displace_map.get_attribute("in")
        ));
    }
    if displace_map.get_attribute("xChannelSelector").as_deref() != Some("R") {
        return Err(format!(
            "expected xChannelSelector \"R\", got {:?}",
            displace_map.get_attribute("xChannelSelector")
        ));
    }
    if displace_map.get_attribute("yChannelSelector").as_deref() != Some("G") {
        return Err(format!(
            "expected yChannelSelector \"G\", got {:?}",
            displace_map.get_attribute("yChannelSelector")
        ));
    }
    if displace_map.get_attribute("scale").as_deref() != Some("24") {
        return Err(format!(
            "24 is this demo's own initial default, got {:?}",
            displace_map.get_attribute("scale")
        ));
    }

    let scale_slider = find_slider("input[aria-label='displacement map scale']")?;
    if scale_slider.get_attribute("min").as_deref() != Some("0") {
        return Err(format!("expected min \"0\", got {:?}", scale_slider.get_attribute("min")));
    }
    if scale_slider.get_attribute("max").as_deref() != Some("60") {
        return Err(format!("expected max \"60\", got {:?}", scale_slider.get_attribute("max")));
    }
    if scale_slider.value() != "24" {
        return Err(format!("expected value \"24\", got {:?}", scale_slider.value()));
    }

    // --- x channel and y channel each pick one of four Channel variants, by a 4-position slider, at this
    // demo's own default (Red for x, Green for y) ---
    let x_slider = find_slider("input[aria-label='displacement map x channel']")?;
    if x_slider.get_attribute("min").as_deref() != Some("0") {
        return Err(format!("expected min \"0\", got {:?}", x_slider.get_attribute("min")));
    }
    if x_slider.get_attribute("max").as_deref() != Some("3") {
        return Err(format!("expected max \"3\", got {:?}", x_slider.get_attribute("max")));
    }
    if x_slider.value() != "0" {
        return Err(format!("expected value \"0\", got {:?}", x_slider.value()));
    }
    if x_slider.get_attribute("aria-valuetext").as_deref() != Some("Red") {
        return Err(format!(
            "expected aria-valuetext \"Red\", got {:?}",
            x_slider.get_attribute("aria-valuetext")
        ));
    }

    // Unlike every other build_h_slider call in this demo, the x-channel slider labels all four of its own
    // positions, not just its two ends: R/G/B/A are categorical, so a bare tick mark at the two middle positions
    // would leave them anonymous. Source extraction cannot prove those four labels actually reach the DOM in the
    // right order.
    let x_container = x_slider
        .closest(".demo-slider-container")
        .map_err(|e| format!("query closest container: {e:?}"))?
        .ok_or_else(|| "x_slider has a .demo-slider-container ancestor".to_owned())?;
    let x_endpoint_labels = x_container
        .query_selector(".demo-endpoint-labels")
        .map_err(|e| format!("query endpoint labels: {e:?}"))?
        .ok_or_else(|| "endpoint-labels row present".to_owned())?;
    let x_label_texts: Vec<String> = {
        let spans = x_endpoint_labels
            .query_selector_all("span")
            .map_err(|e| format!("query endpoint spans: {e:?}"))?;
        let mut texts = Vec::new();
        for i in 0..spans.length() {
            texts.push(
                spans
                    .item(i)
                    .ok_or_else(|| "span item".to_owned())?
                    .text_content()
                    .unwrap_or_default(),
            );
        }
        texts
    };
    let expected_x_label_texts = vec!["R".to_owned(), "G".to_owned(), "B".to_owned(), "A".to_owned()];
    if x_label_texts != expected_x_label_texts {
        return Err(format!("unexpected x-channel endpoint labels: {x_label_texts:?}"));
    }

    let y_slider = find_slider("input[aria-label='displacement map y channel']")?;
    if y_slider.get_attribute("min").as_deref() != Some("0") {
        return Err(format!("expected min \"0\", got {:?}", y_slider.get_attribute("min")));
    }
    if y_slider.get_attribute("max").as_deref() != Some("3") {
        return Err(format!("expected max \"3\", got {:?}", y_slider.get_attribute("max")));
    }
    if y_slider.value() != "1" {
        return Err(format!("expected value \"1\", got {:?}", y_slider.value()));
    }
    if y_slider.get_attribute("aria-valuetext").as_deref() != Some("Green") {
        return Err(format!(
            "expected aria-valuetext \"Green\", got {:?}",
            y_slider.get_attribute("aria-valuetext")
        ));
    }
    if y_slider.get_attribute("aria-orientation").as_deref() != Some("vertical") {
        return Err(format!(
            "a rotated <input type=range> stays a horizontal slider to assistive technology without this, got {:?}",
            y_slider.get_attribute("aria-orientation")
        ));
    }

    // The native thumb's own centre is inset from the track's bare top/bottom edge by its own radius, so the
    // Red/Alpha tick labels sit that same distance in from the track's own raw fractional position (156/240),
    // not at the track's own bare top/bottom (144/244) a naive fractional placement would use. Source extraction
    // cannot prove either of these labels' own `y` attribute actually reaches the DOM shifted, rather than at the
    // unshifted position that would sit above/below the thumb's own centre instead of through it.
    let red_label = find_text("Red")?;
    if red_label.get_attribute("y").as_deref() != Some("156") {
        return Err(format!("expected y \"156\", got {:?}", red_label.get_attribute("y")));
    }
    let alpha_label = find_text("Alpha")?;
    if alpha_label.get_attribute("y").as_deref() != Some("240") {
        return Err(format!("expected y \"240\", got {:?}", alpha_label.get_attribute("y")));
    }

    let distorted_caption = find_text("scale 24 · x Red · y Green")?;

    // --- moving the frequency slider to its documented minimum and maximum updates both noise filters,
    // their captions, and aria-valuetext together ---
    dispatch_input(&frequency_slider, "0")?;
    if fractal.get_attribute("baseFrequency").as_deref() != Some("0") {
        return Err(format!(
            "expected baseFrequency \"0\", got {:?}",
            fractal.get_attribute("baseFrequency")
        ));
    }
    if marbled.get_attribute("baseFrequency").as_deref() != Some("0") {
        return Err(format!(
            "expected baseFrequency \"0\", got {:?}",
            marbled.get_attribute("baseFrequency")
        ));
    }
    if frequency_slider.get_attribute("aria-valuetext").as_deref() != Some("0") {
        return Err(format!(
            "expected aria-valuetext \"0\", got {:?}",
            frequency_slider.get_attribute("aria-valuetext")
        ));
    }
    if fractal_caption.text_content().as_deref() != Some("FractalNoise (0)") {
        return Err(format!(
            "expected caption \"FractalNoise (0)\", got {:?}",
            fractal_caption.text_content()
        ));
    }
    if marbled_caption.text_content().as_deref() != Some("Turbulence (0)") {
        return Err(format!(
            "expected caption \"Turbulence (0)\", got {:?}",
            marbled_caption.text_content()
        ));
    }

    dispatch_input(&frequency_slider, "50")?;
    if fractal.get_attribute("baseFrequency").as_deref() != Some("0.05") {
        return Err(format!(
            "expected baseFrequency \"0.05\", got {:?}",
            fractal.get_attribute("baseFrequency")
        ));
    }
    if marbled.get_attribute("baseFrequency").as_deref() != Some("0.05") {
        return Err(format!(
            "expected baseFrequency \"0.05\", got {:?}",
            marbled.get_attribute("baseFrequency")
        ));
    }
    if frequency_slider.get_attribute("aria-valuetext").as_deref() != Some("0.05") {
        return Err(format!(
            "expected aria-valuetext \"0.05\", got {:?}",
            frequency_slider.get_attribute("aria-valuetext")
        ));
    }
    if fractal_caption.text_content().as_deref() != Some("FractalNoise (0.05)") {
        return Err(format!(
            "expected caption \"FractalNoise (0.05)\", got {:?}",
            fractal_caption.text_content()
        ));
    }
    if marbled_caption.text_content().as_deref() != Some("Turbulence (0.05)") {
        return Err(format!(
            "expected caption \"Turbulence (0.05)\", got {:?}",
            marbled_caption.text_content()
        ));
    }
    if displace_map.get_attribute("scale").as_deref() != Some("24") {
        return Err(format!(
            "moving the frequency slider should not touch the displacement scale, got {:?}",
            displace_map.get_attribute("scale")
        ));
    }

    // Moving the scale slider to zero updates only the displacement map and caption. `demo/panels/panel- turbulence.html`
    // prominently claims scale 0 restores a perfect geometric circle, so this state should be checked explicitly rather
    // than skipping straight to the maximum below. This DOM-level assertion can only prove `scale="0"` reaches the
    // attribute, not that the circle actually renders as a perfect circle at that value.  That test is performed by
    // `cdp-integration-test/tests/turbulence_scale_zero_render.rs`.
    dispatch_input(&scale_slider, "0")?;
    if displace_map.get_attribute("scale").as_deref() != Some("0") {
        return Err(format!("expected scale \"0\", got {:?}", displace_map.get_attribute("scale")));
    }
    if distorted_caption.text_content().as_deref() != Some("scale 0 · x Red · y Green") {
        return Err(format!(
            "expected caption \"scale 0 · x Red · y Green\", got {:?}",
            distorted_caption.text_content()
        ));
    }
    if displace_map.get_attribute("xChannelSelector").as_deref() != Some("R") {
        return Err(format!(
            "moving the scale slider should not touch either channel selector, got {:?}",
            displace_map.get_attribute("xChannelSelector")
        ));
    }
    if displace_map.get_attribute("yChannelSelector").as_deref() != Some("G") {
        return Err(format!(
            "expected yChannelSelector \"G\", got {:?}",
            displace_map.get_attribute("yChannelSelector")
        ));
    }
    if displace_noise.get_attribute("baseFrequency").as_deref() != Some("0.02") {
        return Err(format!(
            "moving the scale slider should not touch the displacement noise's own fixed frequency, got {:?}",
            displace_noise.get_attribute("baseFrequency")
        ));
    }

    // --- moving the scale slider to its documented maximum updates only the displacement map and caption ---
    dispatch_input(&scale_slider, "60")?;
    if displace_map.get_attribute("scale").as_deref() != Some("60") {
        return Err(format!("expected scale \"60\", got {:?}", displace_map.get_attribute("scale")));
    }
    if distorted_caption.text_content().as_deref() != Some("scale 60 · x Red · y Green") {
        return Err(format!(
            "expected caption \"scale 60 · x Red · y Green\", got {:?}",
            distorted_caption.text_content()
        ));
    }
    if fractal.get_attribute("baseFrequency").as_deref() != Some("0.05") {
        return Err(format!(
            "moving the scale slider should not touch either noise filter, got {:?}",
            fractal.get_attribute("baseFrequency")
        ));
    }
    if marbled.get_attribute("baseFrequency").as_deref() != Some("0.05") {
        return Err(format!(
            "expected baseFrequency \"0.05\", got {:?}",
            marbled.get_attribute("baseFrequency")
        ));
    }
    if displace_filter.get_attribute("width").as_deref() != Some("2") {
        return Err(format!(
            "the region should stay fixed, not shrink at the scale slider's own maximum, got {:?}",
            displace_filter.get_attribute("width")
        ));
    }
    if displace_filter.get_attribute("height").as_deref() != Some("2") {
        return Err(format!(
            "expected height \"2\", got {:?}",
            displace_filter.get_attribute("height")
        ));
    }
    if displace_map.get_attribute("xChannelSelector").as_deref() != Some("R") {
        return Err(format!(
            "moving the scale slider should not touch either channel selector, got {:?}",
            displace_map.get_attribute("xChannelSelector")
        ));
    }
    if displace_map.get_attribute("yChannelSelector").as_deref() != Some("G") {
        return Err(format!(
            "expected yChannelSelector \"G\", got {:?}",
            displace_map.get_attribute("yChannelSelector")
        ));
    }

    // index, one-letter keyword, and full name for each of the four `Channel` variants, in `CHANNELS`'s own
    // declared order — the complete `CHANNELS[index]`/`Channel::selector_str` mapping the x-channel and
    // y-channel sliders both translate their own numeric value through.
    const CHANNEL_STEPS: [(&str, &str, &str); 4] =
        [("0", "R", "Red"), ("1", "G", "Green"), ("2", "B", "Blue"), ("3", "A", "Alpha")];

    // --- driving the x-channel slider through all four of its own positions in order proves the complete
    // mapping, not just the two points (its own default, Red, and Alpha) a single dispatch would touch. Blue in
    // particular is otherwise never reached by either slider anywhere in this file. ---
    for (index, keyword, name) in CHANNEL_STEPS {
        dispatch_input(&x_slider, index)?;
        if displace_map.get_attribute("xChannelSelector").as_deref() != Some(keyword) {
            return Err(format!(
                "expected xChannelSelector {keyword:?}, got {:?}",
                displace_map.get_attribute("xChannelSelector")
            ));
        }
        if x_slider.get_attribute("aria-valuetext").as_deref() != Some(name) {
            return Err(format!(
                "expected aria-valuetext {name:?}, got {:?}",
                x_slider.get_attribute("aria-valuetext")
            ));
        }
        let expected_caption = format!("scale 60 · x {name} · y Green");
        if distorted_caption.text_content().as_deref() != Some(expected_caption.as_str()) {
            return Err(format!(
                "expected caption {expected_caption:?}, got {:?}",
                distorted_caption.text_content()
            ));
        }
    }
    if displace_map.get_attribute("yChannelSelector").as_deref() != Some("G") {
        return Err(format!(
            "sweeping the x-channel slider through every position should not touch y, got {:?}",
            displace_map.get_attribute("yChannelSelector")
        ));
    }
    if displace_map.get_attribute("scale").as_deref() != Some("60") {
        return Err(format!(
            "sweeping the x-channel slider through every position should not touch scale, got {:?}",
            displace_map.get_attribute("scale")
        ));
    }

    // --- driving the y-channel slider through all four of its own positions the same way proves the same
    // complete mapping for y. The x-channel slider's own loop above leaves it at Alpha, its own final position,
    // so this loop's own last iteration (Alpha) reproduces the single-diagonal constraint
    // SvgFilter::displacement_map's own doc comment warns about: this demo's own sliders can reach that exact
    // state, not just avoid it. ---
    for (index, keyword, name) in CHANNEL_STEPS {
        dispatch_input(&y_slider, index)?;
        if displace_map.get_attribute("yChannelSelector").as_deref() != Some(keyword) {
            return Err(format!(
                "expected yChannelSelector {keyword:?}, got {:?}",
                displace_map.get_attribute("yChannelSelector")
            ));
        }
        if y_slider.get_attribute("aria-valuetext").as_deref() != Some(name) {
            return Err(format!(
                "expected aria-valuetext {name:?}, got {:?}",
                y_slider.get_attribute("aria-valuetext")
            ));
        }
        let expected_caption = format!("scale 60 · x Alpha · y {name}");
        if distorted_caption.text_content().as_deref() != Some(expected_caption.as_str()) {
            return Err(format!(
                "expected caption {expected_caption:?}, got {:?}",
                distorted_caption.text_content()
            ));
        }
    }
    if displace_map.get_attribute("xChannelSelector").as_deref() != Some("A") {
        return Err(format!(
            "sweeping the y-channel slider through every position should not touch x, which stays at its own \
             last value, got {:?}",
            displace_map.get_attribute("xChannelSelector")
        ));
    }
    if displace_map.get_attribute("scale").as_deref() != Some("60") {
        return Err(format!(
            "sweeping the y-channel slider through every position should not touch scale, got {:?}",
            displace_map.get_attribute("scale")
        ));
    }

    // --- the original circle stays a plain, unfiltered comparison, untouched by every control above ---
    let circles = root.query_selector_all("circle").map_err(|e| format!("query circles: {e:?}"))?;
    if circles.length() != 2 {
        return Err(format!(
            "one original circle and one distorted circle, got {}",
            circles.length()
        ));
    }
    let original = circles
        .item(0)
        .ok_or_else(|| "first circle".to_owned())?
        .dyn_into::<web_sys::Element>()
        .map_err(|_| "expected an Element".to_owned())?;
    if original.get_attribute("filter").is_some() {
        return Err("the original circle carries no filter".to_owned());
    }

    let distorted = circles
        .item(1)
        .ok_or_else(|| "second circle".to_owned())?
        .dyn_into::<web_sys::Element>()
        .map_err(|_| "expected an Element".to_owned())?;
    if distorted.get_attribute("filter").as_deref() != Some("url(#turbulence-displace)") {
        return Err(format!(
            "expected filter \"url(#turbulence-displace)\", got {:?}",
            distorted.get_attribute("filter")
        ));
    }
    Ok(())
}
