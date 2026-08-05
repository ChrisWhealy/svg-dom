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
/// 4) The x-channel and y-channel sliders actually reach `xChannelSelector`/`yChannelSelector`, each with the correct
///    one-letter keyword for the selected `Channel`.
/// 5) Two sliders can actually reach the single-diagonal state (`Alpha` for both) that `SvgFilter::displacement_map`'s
///    own doc comment warns about, not just some other, unconstrained combination.
/// 6) All four controls, and the original circle, stay independent of one another.
/// 7) The scale slider actually reaches `scale="0"`, the value `demo/panels/panel-turbulence.html` prominently
///    claims restores a perfect geometric circle.
///
/// Only a real browser can prove any of that. Even a real browser running this file cannot prove point 7's own
/// rendered-pixel half of the claim, since `wasm-bindgen-test`'s WebDriver-run tests have no access to rasterised
/// output — see `accessibility-tree-test/tests/turbulence_scale_zero_render.rs` for that half.
#[wasm_bindgen_test]
fn demo_turbulence_sliders_update_frequency_and_displacement_scale_independently() {
    container("demo-turbulence");
    crate::paint::demo_turbulence::demo().expect("demo_turbulence::demo should build without error");

    let root = document().get_element_by_id("demo-turbulence").expect("container exists");

    let find_el = |selector: &str| -> web_sys::Element {
        root.query_selector(selector)
            .unwrap_or_else(|_| panic!("invalid selector {selector:?}"))
            .unwrap_or_else(|| panic!("no element matching {selector:?}"))
    };

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

    let find_slider = |aria_label_selector: &str| -> web_sys::HtmlInputElement {
        root.query_selector(aria_label_selector)
            .expect("query slider")
            .unwrap_or_else(|| panic!("no slider matching {aria_label_selector:?}"))
            .dyn_into::<web_sys::HtmlInputElement>()
            .expect("slider is an HtmlInputElement")
    };

    let dispatch_input = |slider: &web_sys::HtmlInputElement, value: &str| {
        slider.set_value(value);
        let event = web_sys::Event::new("input").expect("create input event");
        slider.dispatch_event(&event).expect("dispatch input");
    };

    // --- both noise filters share the same slider-driven baseFrequency, at this demo's own default ---
    let fractal = find_el("#turbulence-fractal feTurbulence");
    assert_eq!(fractal.get_attribute("type").as_deref(), Some("fractalNoise"));
    assert_eq!(
        fractal.get_attribute("baseFrequency").as_deref(),
        Some("0.015"),
        "0.015 is this demo's own initial default"
    );
    assert_eq!(fractal.get_attribute("numOctaves").as_deref(), Some("4"));
    assert_eq!(fractal.get_attribute("seed").as_deref(), Some("3"));

    let marbled = find_el("#turbulence-marbled feTurbulence");
    assert_eq!(marbled.get_attribute("type").as_deref(), Some("turbulence"));
    assert_eq!(marbled.get_attribute("baseFrequency").as_deref(), Some("0.015"));

    let frequency_slider = find_slider("input[aria-label='turbulence base frequency']");
    assert_eq!(frequency_slider.get_attribute("min").as_deref(), Some("0"));
    assert_eq!(frequency_slider.get_attribute("max").as_deref(), Some("50"));
    assert_eq!(frequency_slider.value(), "15");
    assert_eq!(frequency_slider.get_attribute("aria-valuetext").as_deref(), Some("0.015"));

    let fractal_caption = find_text("FractalNoise (0.015)");
    let marbled_caption = find_text("Turbulence (0.015)");

    // --- the displacement filter, at this demo's own default ---
    let displace_filter = find_el("#turbulence-displace");
    assert_eq!(
        displace_filter.get_attribute("x").as_deref(),
        Some("-0.5"),
        "widen_filter_region's own fixed margin"
    );
    assert_eq!(displace_filter.get_attribute("y").as_deref(), Some("-0.5"));
    assert_eq!(displace_filter.get_attribute("width").as_deref(), Some("2"));
    assert_eq!(displace_filter.get_attribute("height").as_deref(), Some("2"));

    let displace_noise = find_el("#turbulence-displace feTurbulence");
    assert_eq!(
        displace_noise.get_attribute("baseFrequency").as_deref(),
        Some("0.02"),
        "fixed, not driven by either slider"
    );
    assert_eq!(displace_noise.get_attribute("result").as_deref(), Some("noise"));

    let displace_map = find_el("#turbulence-displace feDisplacementMap");
    assert_eq!(displace_map.get_attribute("in2").as_deref(), Some("noise"));
    assert_eq!(displace_map.get_attribute("in").as_deref(), Some("SourceGraphic"));
    assert_eq!(displace_map.get_attribute("xChannelSelector").as_deref(), Some("R"));
    assert_eq!(displace_map.get_attribute("yChannelSelector").as_deref(), Some("G"));
    assert_eq!(
        displace_map.get_attribute("scale").as_deref(),
        Some("24"),
        "24 is this demo's own initial default"
    );

    let scale_slider = find_slider("input[aria-label='displacement map scale']");
    assert_eq!(scale_slider.get_attribute("min").as_deref(), Some("0"));
    assert_eq!(scale_slider.get_attribute("max").as_deref(), Some("60"));
    assert_eq!(scale_slider.value(), "24");

    // --- x channel and y channel each pick one of four Channel variants, by a 4-position slider, at this
    // demo's own default (Red for x, Green for y) ---
    let x_slider = find_slider("input[aria-label='displacement map x channel']");
    assert_eq!(x_slider.get_attribute("min").as_deref(), Some("0"));
    assert_eq!(x_slider.get_attribute("max").as_deref(), Some("3"));
    assert_eq!(x_slider.value(), "0");
    assert_eq!(x_slider.get_attribute("aria-valuetext").as_deref(), Some("Red"));

    // Unlike every other build_h_slider call in this demo, the x-channel slider labels all four of its own
    // positions, not just its two ends: R/G/B/A are categorical, so a bare tick mark at the two middle positions
    // would leave them anonymous. Source extraction cannot prove those four labels actually reach the DOM in the
    // right order.
    let x_container = x_slider
        .closest(".demo-slider-container")
        .expect("query closest container")
        .expect("x_slider has a .demo-slider-container ancestor");
    let x_endpoint_labels = x_container
        .query_selector(".demo-endpoint-labels")
        .expect("query endpoint labels")
        .expect("endpoint-labels row present");
    let x_label_texts: Vec<String> = {
        let spans = x_endpoint_labels.query_selector_all("span").expect("query endpoint spans");
        (0..spans.length())
            .map(|i| spans.item(i).expect("span item").text_content().unwrap_or_default())
            .collect()
    };
    assert_eq!(x_label_texts, vec!["R", "G", "B", "A"]);

    let y_slider = find_slider("input[aria-label='displacement map y channel']");
    assert_eq!(y_slider.get_attribute("min").as_deref(), Some("0"));
    assert_eq!(y_slider.get_attribute("max").as_deref(), Some("3"));
    assert_eq!(y_slider.value(), "1");
    assert_eq!(y_slider.get_attribute("aria-valuetext").as_deref(), Some("Green"));
    assert_eq!(
        y_slider.get_attribute("aria-orientation").as_deref(),
        Some("vertical"),
        "a rotated <input type=range> stays a horizontal slider to assistive technology without this"
    );

    // The native thumb's own centre is inset from the track's bare top/bottom edge by its own radius, so the
    // Red/Alpha tick labels sit that same distance in from the track's own raw fractional position (156/240),
    // not at the track's own bare top/bottom (144/244) a naive fractional placement would use. Source extraction
    // cannot prove either of these labels' own `y` attribute actually reaches the DOM shifted, rather than at the
    // unshifted position that would sit above/below the thumb's own centre instead of through it.
    let red_label = find_text("Red");
    assert_eq!(red_label.get_attribute("y").as_deref(), Some("156"));
    let alpha_label = find_text("Alpha");
    assert_eq!(alpha_label.get_attribute("y").as_deref(), Some("240"));

    let distorted_caption = find_text("scale 24 · x Red · y Green");

    // --- moving the frequency slider to its documented minimum and maximum updates both noise filters,
    // their captions, and aria-valuetext together ---
    dispatch_input(&frequency_slider, "0");
    assert_eq!(fractal.get_attribute("baseFrequency").as_deref(), Some("0"));
    assert_eq!(marbled.get_attribute("baseFrequency").as_deref(), Some("0"));
    assert_eq!(frequency_slider.get_attribute("aria-valuetext").as_deref(), Some("0"));
    assert_eq!(fractal_caption.text_content().as_deref(), Some("FractalNoise (0)"));
    assert_eq!(marbled_caption.text_content().as_deref(), Some("Turbulence (0)"));

    dispatch_input(&frequency_slider, "50");
    assert_eq!(fractal.get_attribute("baseFrequency").as_deref(), Some("0.05"));
    assert_eq!(marbled.get_attribute("baseFrequency").as_deref(), Some("0.05"));
    assert_eq!(frequency_slider.get_attribute("aria-valuetext").as_deref(), Some("0.05"));
    assert_eq!(fractal_caption.text_content().as_deref(), Some("FractalNoise (0.05)"));
    assert_eq!(marbled_caption.text_content().as_deref(), Some("Turbulence (0.05)"));
    assert_eq!(
        displace_map.get_attribute("scale").as_deref(),
        Some("24"),
        "moving the frequency slider should not touch the displacement scale"
    );

    // Moving the scale slider to zero updates only the displacement map and caption. `demo/panels/panel- turbulence.html`
    // prominently claims scale 0 restores a perfect geometric circle, so this state should be checked explicitly rather
    // than skipping straight to the maximum below. This DOM-level assertion can only prove `scale="0"` reaches the
    // attribute, not that the circle actually renders as a perfect circle at that value.  That test is performed by
    // `accessibility-tree-test/tests/turbulence_scale_zero_render.rs`.
    dispatch_input(&scale_slider, "0");
    assert_eq!(displace_map.get_attribute("scale").as_deref(), Some("0"));
    assert_eq!(distorted_caption.text_content().as_deref(), Some("scale 0 · x Red · y Green"));
    assert_eq!(
        displace_map.get_attribute("xChannelSelector").as_deref(),
        Some("R"),
        "moving the scale slider should not touch either channel selector"
    );
    assert_eq!(displace_map.get_attribute("yChannelSelector").as_deref(), Some("G"));
    assert_eq!(
        displace_noise.get_attribute("baseFrequency").as_deref(),
        Some("0.02"),
        "moving the scale slider should not touch the displacement noise's own fixed frequency"
    );

    // --- moving the scale slider to its documented maximum updates only the displacement map and caption ---
    dispatch_input(&scale_slider, "60");
    assert_eq!(displace_map.get_attribute("scale").as_deref(), Some("60"));
    assert_eq!(distorted_caption.text_content().as_deref(), Some("scale 60 · x Red · y Green"));
    assert_eq!(
        fractal.get_attribute("baseFrequency").as_deref(),
        Some("0.05"),
        "moving the scale slider should not touch either noise filter"
    );
    assert_eq!(marbled.get_attribute("baseFrequency").as_deref(), Some("0.05"));
    assert_eq!(
        displace_filter.get_attribute("width").as_deref(),
        Some("2"),
        "the region should stay fixed, not shrink at the scale slider's own maximum"
    );
    assert_eq!(displace_filter.get_attribute("height").as_deref(), Some("2"));
    assert_eq!(
        displace_map.get_attribute("xChannelSelector").as_deref(),
        Some("R"),
        "moving the scale slider should not touch either channel selector"
    );
    assert_eq!(displace_map.get_attribute("yChannelSelector").as_deref(), Some("G"));

    // --- moving the x-channel slider to its documented maximum (Alpha) updates only xChannelSelector,
    // aria-valuetext, and the caption ---
    dispatch_input(&x_slider, "3");
    assert_eq!(displace_map.get_attribute("xChannelSelector").as_deref(), Some("A"));
    assert_eq!(x_slider.get_attribute("aria-valuetext").as_deref(), Some("Alpha"));
    assert_eq!(
        distorted_caption.text_content().as_deref(),
        Some("scale 60 · x Alpha · y Green")
    );
    assert_eq!(
        displace_map.get_attribute("yChannelSelector").as_deref(),
        Some("G"),
        "moving the x-channel slider should not touch y"
    );
    assert_eq!(
        displace_map.get_attribute("scale").as_deref(),
        Some("60"),
        "moving the x-channel slider should not touch scale"
    );

    // --- selecting Alpha for y too reproduces the single-diagonal constraint SvgFilter::displacement_map's own
    // doc comment warns about: this demo's own sliders can reach that exact state, not just avoid it ---
    dispatch_input(&y_slider, "3");
    assert_eq!(displace_map.get_attribute("yChannelSelector").as_deref(), Some("A"));
    assert_eq!(y_slider.get_attribute("aria-valuetext").as_deref(), Some("Alpha"));
    assert_eq!(
        distorted_caption.text_content().as_deref(),
        Some("scale 60 · x Alpha · y Alpha")
    );
    assert_eq!(
        displace_map.get_attribute("xChannelSelector").as_deref(),
        Some("A"),
        "moving the y-channel slider should not touch x, which stays at its own last value"
    );
    assert_eq!(
        displace_map.get_attribute("scale").as_deref(),
        Some("60"),
        "moving the y-channel slider should not touch scale"
    );

    // --- the original circle stays a plain, unfiltered comparison, untouched by every control above ---
    let circles = root.query_selector_all("circle").expect("query circles");
    assert_eq!(circles.length(), 2, "one original circle and one distorted circle");
    let original = circles
        .item(0)
        .expect("first circle")
        .dyn_into::<web_sys::Element>()
        .expect("Element");
    assert!(
        original.get_attribute("filter").is_none(),
        "the original circle carries no filter"
    );

    let distorted = circles
        .item(1)
        .expect("second circle")
        .dyn_into::<web_sys::Element>()
        .expect("Element");
    assert_eq!(distorted.get_attribute("filter").as_deref(), Some("url(#turbulence-displace)"));
}
