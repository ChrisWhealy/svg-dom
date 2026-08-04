//! Tests for `demo_color_matrix`'s own Saturate slider, HueRotate slider, and Matrix/LuminanceToAlpha radio
//! group.

use crate::browser_tests::{container, document, find_radio, select_radio};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `color_matrix` returns the `<feColorMatrix>` primitive's own `SvgNode` directly, so
/// `demo_color_matrix` retains it the same way `demo_filter`'s sliders do.
/// Source extraction cannot prove any of the three controls actually reach that retained node.
/// It cannot prove the Matrix/LuminanceToAlpha toggle sets and clears the `values` attribute correctly in both
/// directions.
/// It cannot prove the sepia `values` text stays identical after a toggle away and back.
/// It cannot prove the three controls, and the original rectangle, stay independent of one another.
/// It also cannot prove the matrix rectangle's own white backing rectangle stays unfiltered and stays behind
/// it, in document order, so SVG's own paint order keeps it underneath rather than on top.
/// It also cannot prove the saturate slider's own caption and `aria-valuetext` keep their full two-decimal
/// precision at an intermediate position, not just at the two endpoints, where a coarser format would still
/// happen to look correct.
/// It also cannot prove the saturate slider's own extended range actually reaches real oversaturation (2.0),
/// not just the 1.0 identity point partway along it.
/// It also cannot prove the hue rotate slider's own `aria-valuetext` carries its own unit, both at construction
/// and after a later move, rather than exposing only the raw unitless number.
/// Only a real browser can prove any of that.
#[wasm_bindgen_test]
fn demo_color_matrix_controls_update_saturate_hue_and_matrix_type_independently() {
    container("demo-color-matrix");
    crate::paint::demo_color_matrix::demo().expect("demo_color_matrix::demo should build without error");

    let root = document().get_element_by_id("demo-color-matrix").expect("container exists");

    const SEPIA_VALUES: &str = "0.393 0.769 0.189 0 0 0.349 0.686 0.168 0 0 0.272 0.534 0.131 0 0 0 0 0 1 0";

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

    // --- saturate: at this demo's own default ---
    let saturate = find_el("#cm-filter-saturate feColorMatrix");
    assert_eq!(saturate.get_attribute("type").as_deref(), Some("saturate"));
    assert_eq!(
        saturate.get_attribute("values").as_deref(),
        Some("0"),
        "0.0 is this demo's own initial default"
    );

    let saturate_slider = find_slider("input[aria-label='colour matrix saturate']");
    assert_eq!(saturate_slider.get_attribute("min").as_deref(), Some("0"));
    assert_eq!(
        saturate_slider.get_attribute("max").as_deref(),
        Some("200"),
        "200% (2.0) demonstrates real oversaturation, not just the 1.0 identity endpoint"
    );
    assert_eq!(saturate_slider.value(), "0");
    assert_eq!(saturate_slider.get_attribute("aria-valuetext").as_deref(), Some("0.0"));

    let saturate_caption = find_text("Saturate(0.0)");

    // --- hue rotate: at this demo's own default ---
    let hue = find_el("#cm-filter-hue feColorMatrix");
    assert_eq!(hue.get_attribute("type").as_deref(), Some("hueRotate"));
    assert_eq!(
        hue.get_attribute("values").as_deref(),
        Some("180"),
        "180 is this demo's own initial default"
    );

    let hue_slider = find_slider("input[aria-label='colour matrix hue rotate']");
    assert_eq!(hue_slider.get_attribute("min").as_deref(), Some("0"));
    assert_eq!(hue_slider.get_attribute("max").as_deref(), Some("360"));
    assert_eq!(hue_slider.value(), "180");
    assert_eq!(
        hue_slider.get_attribute("aria-valuetext").as_deref(),
        Some("180 degrees"),
        "the raw slider value alone does not carry its own unit"
    );

    let hue_caption = find_text("HueRotate(180)");

    // --- matrix: at this demo's own default (sepia) ---
    let matrix = find_el("#cm-filter-matrix feColorMatrix");
    assert_eq!(matrix.get_attribute("type").as_deref(), Some("matrix"));
    assert_eq!(
        matrix.get_attribute("values").as_deref(),
        Some(SEPIA_VALUES),
        "the sepia coefficients are this demo's own initial default"
    );

    let matrix_caption = find_text("Matrix (sepia)");
    let sepia = find_radio(&root, "matrix type", "sepia");
    let luminance = find_radio(&root, "matrix type", "luminance");
    assert!(sepia.checked(), "sepia is this demo's own initial default");
    assert!(!luminance.checked());

    // --- an intermediate saturate position keeps its own full two-decimal precision, not just one decimal
    // place. The slider moves in 1% steps, so 101 distinct positions exist between 0 and 100. A fixed
    // one-decimal format would collapse those down to only 11 displayed values: 0.24 and 0.25 would both read
    // as "0.2". This pins the caption and aria-valuetext against an exact two-decimal value (0.25) and against
    // a value whose own trailing zero should be stripped (0.7, not 0.70), not just the two endpoints, where a
    // one-decimal format happens to already look correct. ---
    dispatch_input(&saturate_slider, "25");
    assert_eq!(saturate.get_attribute("values").as_deref(), Some("0.25"));
    assert_eq!(saturate_slider.get_attribute("aria-valuetext").as_deref(), Some("0.25"));
    assert_eq!(saturate_caption.text_content().as_deref(), Some("Saturate(0.25)"));

    dispatch_input(&saturate_slider, "70");
    assert_eq!(saturate.get_attribute("values").as_deref(), Some("0.7"));
    assert_eq!(saturate_slider.get_attribute("aria-valuetext").as_deref(), Some("0.7"));
    assert_eq!(saturate_caption.text_content().as_deref(), Some("Saturate(0.7)"));

    // --- raw 100 is the identity point (1.0), not the slider's own maximum ---
    dispatch_input(&saturate_slider, "100");
    assert_eq!(saturate.get_attribute("values").as_deref(), Some("1"));
    assert_eq!(saturate_slider.get_attribute("aria-valuetext").as_deref(), Some("1.0"));
    assert_eq!(saturate_caption.text_content().as_deref(), Some("Saturate(1.0)"));

    // --- moving saturate to its documented maximum demonstrates real oversaturation. The SVG default range
    // would stop at the identity point above; this slider's own extended range goes further. ---
    dispatch_input(&saturate_slider, "200");
    assert_eq!(saturate.get_attribute("values").as_deref(), Some("2"));
    assert_eq!(saturate_slider.get_attribute("aria-valuetext").as_deref(), Some("2.0"));
    assert_eq!(saturate_caption.text_content().as_deref(), Some("Saturate(2.0)"));
    assert_eq!(
        hue.get_attribute("values").as_deref(),
        Some("180"),
        "moving the saturate slider should not touch hue rotate"
    );
    assert_eq!(
        matrix.get_attribute("values").as_deref(),
        Some(SEPIA_VALUES),
        "moving the saturate slider should not touch the matrix type"
    );

    // --- moving hue rotate updates only the hue channel, caption, and aria-valuetext ---
    dispatch_input(&hue_slider, "45");
    assert_eq!(hue.get_attribute("values").as_deref(), Some("45"));
    assert_eq!(hue_caption.text_content().as_deref(), Some("HueRotate(45)"));
    assert_eq!(hue_slider.get_attribute("aria-valuetext").as_deref(), Some("45 degrees"));
    assert_eq!(
        saturate.get_attribute("values").as_deref(),
        Some("2"),
        "moving the hue rotate slider should not touch saturate, which stays at its own last value"
    );

    // --- selecting luminance clears the matrix's own values attribute and updates its type and caption ---
    select_radio(&luminance);
    assert_eq!(matrix.get_attribute("type").as_deref(), Some("luminanceToAlpha"));
    assert!(
        matrix.get_attribute("values").is_none(),
        "luminanceToAlpha needs no values attribute at all"
    );
    assert_eq!(matrix_caption.text_content().as_deref(), Some("LuminanceToAlpha"));
    assert!(luminance.checked());
    assert!(!sepia.checked(), "selecting luminance should clear sepia");

    // --- selecting sepia again restores the exact same values text construction produced ---
    select_radio(&sepia);
    assert_eq!(matrix.get_attribute("type").as_deref(), Some("matrix"));
    assert_eq!(
        matrix.get_attribute("values").as_deref(),
        Some(SEPIA_VALUES),
        "toggling back to sepia should restore identical values text, not a differently formatted equivalent"
    );
    assert_eq!(matrix_caption.text_content().as_deref(), Some("Matrix (sepia)"));
    assert!(sepia.checked());
    assert!(!luminance.checked(), "selecting sepia should clear luminance");

    assert_eq!(
        saturate.get_attribute("values").as_deref(),
        Some("2"),
        "toggling the matrix radio group should not touch saturate"
    );
    assert_eq!(
        hue.get_attribute("values").as_deref(),
        Some("45"),
        "toggling the matrix radio group should not touch hue rotate"
    );

    // --- the original rectangle stays a plain, unfiltered comparison, untouched by every control above ---
    let rects = root.query_selector_all("rect").expect("query rects");
    assert_eq!(
        rects.length(),
        5,
        "one original rectangle, one per control, and the matrix rectangle's own white backing rectangle"
    );
    let element_at = |index: u32| -> web_sys::Element {
        rects
            .item(index)
            .unwrap_or_else(|| panic!("no rect at index {index}"))
            .dyn_into::<web_sys::Element>()
            .expect("Element")
    };
    let original = element_at(0);
    assert!(
        original.get_attribute("filter").is_none(),
        "the original rectangle carries no filter"
    );

    // --- LuminanceToAlpha zeroes colour, leaving alpha as its only visible signal. Blending near-transparent
    // black into this gallery's own near-black canvas would crush that signal, so a plain white rectangle sits
    // behind the matrix rectangle. It must be drawn before that rectangle, in document order, or SVG's own
    // paint order would put it on top instead of underneath. ---
    let backing = element_at(3);
    assert_eq!(
        backing.get_attribute("fill").as_deref(),
        Some("white"),
        "the backing rectangle should give LuminanceToAlpha's own alpha signal something to blend against"
    );
    assert!(
        backing.get_attribute("filter").is_none(),
        "the backing rectangle itself must stay unfiltered, or it would hide behind its own colour transform"
    );

    let matrix_rect = element_at(4);
    assert_eq!(
        matrix_rect.get_attribute("filter").as_deref(),
        Some("url(#cm-filter-matrix)"),
        "the matrix rectangle should be the last rect, painted on top of its own backing rectangle"
    );
}
