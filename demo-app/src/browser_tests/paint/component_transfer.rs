//! Tests for `demo_component_transfer`'s own three sliders: gamma exponent, discrete step count, and alpha slope.

use crate::browser_tests::{container, document};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `component_transfer` returns only the `<feComponentTransfer>` container's own `SvgNode`.
/// It does not return a handle to any `<feFuncX>` child.
/// `gaussian_blur`, `drop_shadow`, and `blend` each return their own primitive node instead.
/// So `demo_component_transfer`'s own three sliders reach their target elements by CSS selector, not by a
/// retained handle.
///
/// Source extraction cannot prove that this wiring works.
/// It cannot prove the gamma and alpha sliders' own scaled values reach the correct fractional attribute.
/// It cannot prove the discrete slider's own `tableValues` stays evenly spaced at every step count.
/// It cannot prove that text matches a fresh build's own text.
/// It cannot prove the three controls stay independent of each other and of the original rectangle.
/// Only a real browser can prove any of that.
#[wasm_bindgen_test]
fn demo_component_transfer_sliders_update_gamma_discrete_and_alpha_independently() {
    container("demo-component-transfer");
    crate::paint::demo_component_transfer::demo().expect("demo_component_transfer::demo should build without error");

    let root = document()
        .get_element_by_id("demo-component-transfer")
        .expect("container exists");

    let find_el = |selector: &str| -> web_sys::Element {
        root.query_selector(selector)
            .unwrap_or_else(|_| panic!("invalid selector {selector:?}"))
            .unwrap_or_else(|| panic!("no element matching {selector:?}"))
    };

    let find_all = |selector: &str| -> Vec<web_sys::Element> {
        let list = root.query_selector_all(selector).expect("query elements");
        (0..list.length())
            .map(|i| list.item(i).expect("item").dyn_into::<web_sys::Element>().expect("Element"))
            .collect()
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

    // --- gamma: three <feFuncR/G/B>, all sharing the slider's own exponent, at this demo's own default ---
    let gamma_funcs = find_all(
        "#component-transfer-gamma feFuncR, #component-transfer-gamma feFuncG, #component-transfer-gamma feFuncB",
    );
    assert_eq!(gamma_funcs.len(), 3, "gamma drives all three colour channels");
    for func in &gamma_funcs {
        assert_eq!(func.get_attribute("type").as_deref(), Some("gamma"));
        assert_eq!(
            func.get_attribute("exponent").as_deref(),
            Some("2.2"),
            "2.2 is this demo's own initial default"
        );
        assert_eq!(func.get_attribute("amplitude").as_deref(), Some("1"));
        assert_eq!(func.get_attribute("offset").as_deref(), Some("0"));
    }

    let gamma_slider = find_slider("input[aria-label='component transfer gamma exponent']");
    assert_eq!(
        gamma_slider.get_attribute("min").as_deref(),
        Some("2"),
        "2 (0.2) is this slider's own documented minimum"
    );
    assert_eq!(
        gamma_slider.get_attribute("max").as_deref(),
        Some("50"),
        "50 (5.0) is this slider's own documented maximum"
    );
    assert_eq!(gamma_slider.value(), "22");
    assert_eq!(gamma_slider.get_attribute("aria-valuetext").as_deref(), Some("2.2"));

    let gamma_caption = find_text("Gamma(2.2)");

    // --- discrete: three <feFuncR/G/B>, all sharing the slider's own step count, at this demo's own default ---
    let discrete_funcs = find_all(
        "#component-transfer-discrete feFuncR, #component-transfer-discrete feFuncG, \
         #component-transfer-discrete feFuncB",
    );
    assert_eq!(discrete_funcs.len(), 3, "discrete drives all three colour channels");
    for func in &discrete_funcs {
        assert_eq!(func.get_attribute("type").as_deref(), Some("discrete"));
        assert_eq!(
            func.get_attribute("tableValues").as_deref(),
            Some("0 0.333 0.667 1"),
            "4 evenly spaced steps is this demo's own initial default"
        );
    }

    let discrete_slider = find_slider("input[aria-label='component transfer discrete step count']");
    assert_eq!(
        discrete_slider.get_attribute("min").as_deref(),
        Some("2"),
        "2 is this slider's own documented minimum"
    );
    assert_eq!(
        discrete_slider.get_attribute("max").as_deref(),
        Some("8"),
        "8 is this slider's own documented maximum"
    );
    assert_eq!(discrete_slider.value(), "4");

    let discrete_caption = find_text("Discrete(4-step)");

    // --- alpha: a single <feFuncA>, untouched by either channel above, at this demo's own default ---
    let alpha_func = find_el("#component-transfer-alpha feFuncA");
    assert_eq!(alpha_func.get_attribute("type").as_deref(), Some("linear"));
    assert_eq!(
        alpha_func.get_attribute("slope").as_deref(),
        Some("0.4"),
        "0.4 is this demo's own initial default"
    );
    assert_eq!(
        alpha_func.get_attribute("intercept").as_deref(),
        Some("0"),
        "intercept stays fixed at 0.0. Only slope is exposed to the slider."
    );

    let alpha_slider = find_slider("input[aria-label='component transfer alpha slope']");
    assert_eq!(alpha_slider.get_attribute("min").as_deref(), Some("0"));
    assert_eq!(alpha_slider.get_attribute("max").as_deref(), Some("100"));
    assert_eq!(alpha_slider.value(), "40");
    assert_eq!(alpha_slider.get_attribute("aria-valuetext").as_deref(), Some("0.4"));

    let alpha_caption = find_text("Alpha Linear(0.4)");

    // --- moving gamma to its documented minimum and maximum updates only the gamma channels and caption ---
    dispatch_input(&gamma_slider, "2"); // 0.2
    for func in &gamma_funcs {
        assert_eq!(func.get_attribute("exponent").as_deref(), Some("0.2"));
    }
    assert_eq!(gamma_slider.get_attribute("aria-valuetext").as_deref(), Some("0.2"));
    assert_eq!(gamma_caption.text_content().as_deref(), Some("Gamma(0.2)"));

    dispatch_input(&gamma_slider, "50"); // 5.0
    for func in &gamma_funcs {
        assert_eq!(
            func.get_attribute("exponent").as_deref(),
            Some("5"),
            "5.0 prints as a bare \"5\". This matches component_transfer's own construction-time Display formatting."
        );
    }
    assert_eq!(gamma_slider.get_attribute("aria-valuetext").as_deref(), Some("5.0"));
    assert_eq!(gamma_caption.text_content().as_deref(), Some("Gamma(5.0)"));

    for func in &discrete_funcs {
        assert_eq!(
            func.get_attribute("tableValues").as_deref(),
            Some("0 0.333 0.667 1"),
            "moving the gamma slider should not touch the discrete channels"
        );
    }
    assert_eq!(
        alpha_func.get_attribute("slope").as_deref(),
        Some("0.4"),
        "moving the gamma slider should not touch the alpha channel"
    );

    // --- moving discrete to its documented minimum and maximum updates only the discrete channels and caption ---
    dispatch_input(&discrete_slider, "2");
    for func in &discrete_funcs {
        assert_eq!(func.get_attribute("tableValues").as_deref(), Some("0 1"));
    }
    assert_eq!(discrete_caption.text_content().as_deref(), Some("Discrete(2-step)"));

    dispatch_input(&discrete_slider, "8");
    for func in &discrete_funcs {
        assert_eq!(
            func.get_attribute("tableValues").as_deref(),
            Some("0 0.143 0.286 0.429 0.571 0.714 0.857 1"),
            "8 evenly spaced steps, each rounded to 3 decimal places"
        );
    }
    assert_eq!(discrete_caption.text_content().as_deref(), Some("Discrete(8-step)"));

    for func in &gamma_funcs {
        assert_eq!(
            func.get_attribute("exponent").as_deref(),
            Some("5"),
            "moving the discrete slider should not touch the gamma channels. They stay at their own last value."
        );
    }
    assert_eq!(
        alpha_func.get_attribute("slope").as_deref(),
        Some("0.4"),
        "moving the discrete slider should not touch the alpha channel"
    );

    // --- moving alpha to its documented minimum and maximum updates only the alpha channel and caption ---
    dispatch_input(&alpha_slider, "0");
    assert_eq!(alpha_func.get_attribute("slope").as_deref(), Some("0"));
    assert_eq!(alpha_slider.get_attribute("aria-valuetext").as_deref(), Some("0.0"));
    assert_eq!(alpha_caption.text_content().as_deref(), Some("Alpha Linear(0.0)"));

    dispatch_input(&alpha_slider, "100");
    assert_eq!(alpha_func.get_attribute("slope").as_deref(), Some("1"));
    assert_eq!(alpha_slider.get_attribute("aria-valuetext").as_deref(), Some("1.0"));
    assert_eq!(alpha_caption.text_content().as_deref(), Some("Alpha Linear(1.0)"));

    for func in &gamma_funcs {
        assert_eq!(
            func.get_attribute("exponent").as_deref(),
            Some("5"),
            "moving the alpha slider should not touch the gamma channels. They stay at their own last value."
        );
    }
    for func in &discrete_funcs {
        assert_eq!(
            func.get_attribute("tableValues").as_deref(),
            Some("0 0.143 0.286 0.429 0.571 0.714 0.857 1"),
            "moving the alpha slider should not touch the discrete channels. They stay at their own last value."
        );
    }

    // --- the original rectangle stays a plain, unfiltered comparison, untouched by every slider above ---
    let rects = root.query_selector_all("rect").expect("query rects");
    assert_eq!(rects.length(), 4, "one original rectangle and one per slider");
    let original = rects
        .item(0)
        .expect("first rect")
        .dyn_into::<web_sys::Element>()
        .expect("Element");
    assert!(
        original.get_attribute("filter").is_none(),
        "the original rectangle carries no filter"
    );
}
