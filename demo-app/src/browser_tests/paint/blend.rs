//! Tests for `demo_blend`'s own BlendMode dropdown.

use crate::browser_tests::{container, document};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `demo_blend` retains its own `feBlend` primitive's `SvgNode` the same way `demo_filter`'s blur/shadow do, and
/// updates its `mode` attribute live from a `foreign_html::select_dropdown` rather than a radio group — this
/// panel is that helper's first caller. Source extraction alone cannot prove the dropdown actually reaches
/// `feBlend`'s own `mode` attribute, that its own sixteen options stay in `BlendMode`'s declared order, or that
/// the live caption below the blended circle tracks the current selection rather than staying at its build-time
/// default. Only a real browser, with a real `<select>` dispatching a real `change` event, can prove that.
///
/// This test also pins the static wiring around `feBlend` — `feFlood`'s own `result`, `feBlend`'s own `in2`/
/// `result`, and the final `feComposite`'s own `in`/`in2`/`operator` — that none of the mode-switching assertions
/// above touch. Without those, a regression dropping or miswiring the final `composite(In)` step could pass every
/// other assertion here while silently breaking the alpha-clipping behaviour the panel's own figcaption teaches.
#[wasm_bindgen_test]
fn demo_blend_dropdown_updates_feblend_mode_and_caption() {
    container("demo-blend");
    crate::paint::demo_blend::demo().expect("demo_blend::demo should build without error");

    let root = document().get_element_by_id("demo-blend").expect("container exists");

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

    // --- one filter drives the blended circle; its own feBlend mode starts at this demo's own default ---
    let blend = find_el("#demo-blend-filter feBlend");
    assert_eq!(
        blend.get_attribute("mode").as_deref(),
        Some("multiply"),
        "Multiply is this demo's own initial default"
    );
    assert_eq!(blend.get_attribute("in").as_deref(), Some("SourceGraphic"));

    // --- the rest of the chain around feBlend is exactly what the panel's own figcaption teaches: flood, then
    // blend, then a final composite(In) clipping the tinted result back to the source's own alpha coverage. None
    // of the mode-switching assertions above touch these three elements, so they cannot catch a regression that
    // drops or miswires the flood or the final composite while every mode/caption assertion still passes. ---
    let flood = find_el("#demo-blend-filter feFlood");
    assert_eq!(
        flood.get_attribute("result").as_deref(),
        Some("tint"),
        "feBlend's own in2 (checked below) reads this same result name"
    );

    assert_eq!(
        blend.get_attribute("in2").as_deref(),
        Some("tint"),
        "feBlend blends SourceGraphic against the flood's own tint, not some other input"
    );
    assert_eq!(
        blend.get_attribute("result").as_deref(),
        Some("tinted"),
        "the final composite (checked below) reads this same result name"
    );

    let composite = find_el("#demo-blend-filter feComposite");
    assert_eq!(
        composite.get_attribute("in").as_deref(),
        Some("tinted"),
        "the final step composites feBlend's own tinted result, not SourceGraphic directly"
    );
    assert_eq!(
        composite.get_attribute("in2").as_deref(),
        Some("SourceGraphic"),
        "clipping back to the source's own alpha coverage is the whole point of this step"
    );
    assert_eq!(
        composite.get_attribute("operator").as_deref(),
        Some("in"),
        "operator=\"in\" is what actually clips the opaque flood back to the circle's own transparent corners \
         — any other operator would leave the flood colour leaking through them, the exact mistake the panel's \
         own figcaption warns about"
    );

    let select = root
        .query_selector("select[aria-label='feBlend blend mode']")
        .expect("query select")
        .expect("no select matching aria-label")
        .dyn_into::<web_sys::HtmlSelectElement>()
        .expect("select is an HtmlSelectElement");

    // select_dropdown wraps its own <select> in a native <label>, not a plain <div>, so the browser's own
    // HTMLSelectElement.labels associates them without any hand-written for/id pair to keep in sync. This is the
    // one thing source extraction cannot prove: that the wrapping element is actually a <label> the DOM recognises
    // as this select's own label, not merely a sibling <div> that happens to sit next to it visually.
    let associated_labels = select.labels();
    assert_eq!(
        associated_labels.length(),
        1,
        "the select should have exactly one native label associated with it"
    );
    let associated_label = associated_labels
        .item(0)
        .expect("first associated label")
        .dyn_into::<web_sys::Element>()
        .expect("Element");
    assert_eq!(associated_label.tag_name(), "LABEL");
    // The label's own text_content would also pick up every <option>'s own text (they are still descendants of
    // the <select>, hence of the wrapping <label>, even though only one renders at a time), so this checks the
    // dedicated caption element inside it rather than the label's own full text_content.
    assert_eq!(
        associated_label
            .query_selector(".demo-slider-label")
            .expect("query caption")
            .and_then(|el| el.text_content())
            .as_deref(),
        Some("blend mode"),
        "the associated label should contain select_dropdown's own visible caption"
    );

    // The dropdown's own initial selection already matches feBlend's own initial mode, both driven by the same
    // DEFAULT_MODE constant, before any interaction happens.
    assert_eq!(
        select.value(),
        "1",
        "index 1 is Multiply in BLEND_MODE_OPTIONS's own declared order"
    );

    // All sixteen BlendMode options are present, in the library's own declaration order — not the subset of three
    // this demo's earlier fixed-circle layout showed. Every entry is checked, not just a few spot checks: a
    // regression that duplicated or swapped two unchecked intermediate entries would otherwise still pass.
    let options = select.query_selector_all("option").expect("query options");
    assert_eq!(options.length(), 16, "BlendMode has sixteen members");
    let option_text = |index: u32| -> String {
        options
            .item(index)
            .expect("option item")
            .dyn_into::<web_sys::Element>()
            .expect("Element")
            .text_content()
            .expect("option text")
    };
    let expected_labels = [
        "Normal", "Multiply", "Screen", "Darken", "Lighten", "Overlay", "Color Dodge", "Color Burn", "Hard Light",
        "Soft Light", "Difference", "Exclusion", "Hue", "Saturation", "Color", "Luminosity",
    ];
    for (index, &expected_label) in expected_labels.iter().enumerate() {
        assert_eq!(
            option_text(index as u32),
            expected_label,
            "option {index} should be {expected_label:?}"
        );
    }

    let caption = find_text("mode: Multiply");

    let dispatch_change = |value: &str| {
        select.set_value(value);
        let event = web_sys::Event::new("change").expect("create change event");
        select.dispatch_event(&event).expect("dispatch change");
    };

    // --- selecting "Color Dodge" (index 6) updates both feBlend's own mode and the live caption below it ---
    dispatch_change("6");
    assert_eq!(blend.get_attribute("mode").as_deref(), Some("color-dodge"));
    assert_eq!(
        caption.text_content().as_deref(),
        Some("mode: Color Dodge"),
        "the caption should track the dropdown's own current selection, not stay at its build-time default"
    );

    // --- selecting "Luminosity" (index 15, the last option) does the same ---
    dispatch_change("15");
    assert_eq!(blend.get_attribute("mode").as_deref(), Some("luminosity"));
    assert_eq!(caption.text_content().as_deref(), Some("mode: Luminosity"));

    // --- the original circle stays a plain, unfiltered comparison, untouched by every dropdown selection above ---
    let circles = root.query_selector_all("circle").expect("query circles");
    assert_eq!(circles.length(), 2, "one original circle and one blended circle");
    let original = circles
        .item(0)
        .expect("first circle")
        .dyn_into::<web_sys::Element>()
        .expect("Element");
    assert!(
        original.get_attribute("filter").is_none(),
        "the original circle carries no filter"
    );

    let blended_circle = circles
        .item(1)
        .expect("second circle")
        .dyn_into::<web_sys::Element>()
        .expect("Element");
    assert_eq!(
        blended_circle.get_attribute("filter").as_deref(),
        Some("url(#demo-blend-filter)")
    );
}
