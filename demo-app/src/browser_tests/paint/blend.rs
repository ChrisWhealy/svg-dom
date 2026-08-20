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
fn demo_blend_dropdown_updates_feblend_mode_and_caption() -> Result<(), String> {
    container("demo-blend");
    crate::paint::demo_blend::demo().map_err(|e| format!("demo_blend::demo should build without error: {e:?}"))?;

    let root = document()
        .get_element_by_id("demo-blend")
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

    // --- one filter drives the blended circle; its own feBlend mode starts at this demo's own default ---
    let blend = find_el("#demo-blend-filter feBlend")?;
    if blend.get_attribute("mode").as_deref() != Some("multiply") {
        return Err(format!(
            "Multiply is this demo's own initial default, got {:?}",
            blend.get_attribute("mode")
        ));
    }
    if blend.get_attribute("in").as_deref() != Some("SourceGraphic") {
        return Err(format!("expected in=\"SourceGraphic\", got {:?}", blend.get_attribute("in")));
    }

    // --- the rest of the chain around feBlend is exactly what the panel's own figcaption teaches: flood, then
    // blend, then a final composite(In) clipping the tinted result back to the source's own alpha coverage. None
    // of the mode-switching assertions above touch these three elements, so they cannot catch a regression that
    // drops or miswires the flood or the final composite while every mode/caption assertion still passes. ---
    let flood = find_el("#demo-blend-filter feFlood")?;
    if flood.get_attribute("result").as_deref() != Some("tint") {
        return Err(format!(
            "feBlend's own in2 (checked below) reads this same result name, got {:?}",
            flood.get_attribute("result")
        ));
    }

    if blend.get_attribute("in2").as_deref() != Some("tint") {
        return Err(format!(
            "feBlend blends SourceGraphic against the flood's own tint, not some other input, got {:?}",
            blend.get_attribute("in2")
        ));
    }
    if blend.get_attribute("result").as_deref() != Some("tinted") {
        return Err(format!(
            "the final composite (checked below) reads this same result name, got {:?}",
            blend.get_attribute("result")
        ));
    }

    let composite = find_el("#demo-blend-filter feComposite")?;
    if composite.get_attribute("in").as_deref() != Some("tinted") {
        return Err(format!(
            "the final step composites feBlend's own tinted result, not SourceGraphic directly, got {:?}",
            composite.get_attribute("in")
        ));
    }
    if composite.get_attribute("in2").as_deref() != Some("SourceGraphic") {
        return Err(format!(
            "clipping back to the source's own alpha coverage is the whole point of this step, got {:?}",
            composite.get_attribute("in2")
        ));
    }
    if composite.get_attribute("operator").as_deref() != Some("in") {
        return Err(format!(
            "operator=\"in\" is what actually clips the opaque flood back to the circle's own transparent corners \
             — any other operator would leave the flood colour leaking through them, the exact mistake the panel's \
             own figcaption warns about, got {:?}",
            composite.get_attribute("operator")
        ));
    }

    let select = root
        .query_selector("select[aria-label='feBlend blend mode']")
        .map_err(|e| format!("query select: {e:?}"))?
        .ok_or_else(|| "no select matching aria-label".to_owned())?
        .dyn_into::<web_sys::HtmlSelectElement>()
        .map_err(|_| "select is an HtmlSelectElement".to_owned())?;

    // select_dropdown wraps its own <select> in a native <label>, not a plain <div>, so the browser's own
    // HTMLSelectElement.labels associates them without any hand-written for/id pair to keep in sync. This is the
    // one thing source extraction cannot prove: that the wrapping element is actually a <label> the DOM recognises
    // as this select's own label, not merely a sibling <div> that happens to sit next to it visually.
    let associated_labels = select.labels();
    if associated_labels.length() != 1 {
        return Err(format!(
            "the select should have exactly one native label associated with it, got {}",
            associated_labels.length()
        ));
    }
    let associated_label = associated_labels
        .item(0)
        .ok_or_else(|| "first associated label".to_owned())?
        .dyn_into::<web_sys::Element>()
        .map_err(|_| "expected an Element".to_owned())?;
    if associated_label.tag_name() != "LABEL" {
        return Err(format!("expected tag name \"LABEL\", got {:?}", associated_label.tag_name()));
    }
    // The label's own text_content would also pick up every <option>'s own text (they are still descendants of
    // the <select>, hence of the wrapping <label>, even though only one renders at a time), so this checks the
    // dedicated caption element inside it rather than the label's own full text_content.
    let caption_text = associated_label
        .query_selector(".demo-slider-label")
        .map_err(|e| format!("query caption: {e:?}"))?
        .and_then(|el| el.text_content());
    if caption_text.as_deref() != Some("blend mode") {
        return Err(format!(
            "the associated label should contain select_dropdown's own visible caption, got {caption_text:?}"
        ));
    }

    // The dropdown's own initial selection already matches feBlend's own initial mode, both driven by the same
    // DEFAULT_MODE constant, before any interaction happens.
    if select.value() != "1" {
        return Err(format!(
            "index 1 is Multiply in BLEND_MODE_OPTIONS's own declared order, got {:?}",
            select.value()
        ));
    }

    // All sixteen BlendMode options are present, in the library's own declaration order — not the subset of three
    // this demo's earlier fixed-circle layout showed. Every entry is checked, not just a few spot checks: a
    // regression that duplicated or swapped two unchecked intermediate entries would otherwise still pass.
    let options = select
        .query_selector_all("option")
        .map_err(|e| format!("query options: {e:?}"))?;
    if options.length() != 16 {
        return Err(format!("BlendMode has sixteen members, got {}", options.length()));
    }
    let option_text = |index: u32| -> Result<String, String> {
        options
            .item(index)
            .ok_or_else(|| "option item".to_owned())?
            .dyn_into::<web_sys::Element>()
            .map_err(|_| "expected an Element".to_owned())?
            .text_content()
            .ok_or_else(|| "option text".to_owned())
    };
    let expected_labels = [
        "Normal", "Multiply", "Screen", "Darken", "Lighten", "Overlay", "Color Dodge", "Color Burn", "Hard Light",
        "Soft Light", "Difference", "Exclusion", "Hue", "Saturation", "Color", "Luminosity",
    ];
    for (index, &expected_label) in expected_labels.iter().enumerate() {
        let actual = option_text(index as u32)?;
        if actual != expected_label {
            return Err(format!("option {index} should be {expected_label:?}, got {actual:?}"));
        }
    }

    let caption = find_text("mode: Multiply")?;

    let dispatch_change = |value: &str| -> Result<(), String> {
        select.set_value(value);
        let event = web_sys::Event::new("change").map_err(|e| format!("create change event: {e:?}"))?;
        select.dispatch_event(&event).map_err(|e| format!("dispatch change: {e:?}"))?;
        Ok(())
    };

    // --- selecting "Color Dodge" (index 6) updates both feBlend's own mode and the live caption below it ---
    dispatch_change("6")?;
    if blend.get_attribute("mode").as_deref() != Some("color-dodge") {
        return Err(format!("expected mode \"color-dodge\", got {:?}", blend.get_attribute("mode")));
    }
    if caption.text_content().as_deref() != Some("mode: Color Dodge") {
        return Err(format!(
            "the caption should track the dropdown's own current selection, not stay at its build-time default, \
             got {:?}",
            caption.text_content()
        ));
    }

    // --- selecting "Luminosity" (index 15, the last option) does the same ---
    dispatch_change("15")?;
    if blend.get_attribute("mode").as_deref() != Some("luminosity") {
        return Err(format!("expected mode \"luminosity\", got {:?}", blend.get_attribute("mode")));
    }
    if caption.text_content().as_deref() != Some("mode: Luminosity") {
        return Err(format!(
            "expected caption \"mode: Luminosity\", got {:?}",
            caption.text_content()
        ));
    }

    // --- the original circle stays a plain, unfiltered comparison, untouched by every dropdown selection above ---
    let circles = root.query_selector_all("circle").map_err(|e| format!("query circles: {e:?}"))?;
    if circles.length() != 2 {
        return Err(format!("one original circle and one blended circle, got {}", circles.length()));
    }
    let original = circles
        .item(0)
        .ok_or_else(|| "first circle".to_owned())?
        .dyn_into::<web_sys::Element>()
        .map_err(|_| "expected an Element".to_owned())?;
    if original.get_attribute("filter").is_some() {
        return Err("the original circle carries no filter".to_owned());
    }

    let blended_circle = circles
        .item(1)
        .ok_or_else(|| "second circle".to_owned())?
        .dyn_into::<web_sys::Element>()
        .map_err(|_| "expected an Element".to_owned())?;
    if blended_circle.get_attribute("filter").as_deref() != Some("url(#demo-blend-filter)") {
        return Err(format!(
            "expected filter=\"url(#demo-blend-filter)\", got {:?}",
            blended_circle.get_attribute("filter")
        ));
    }
    Ok(())
}
