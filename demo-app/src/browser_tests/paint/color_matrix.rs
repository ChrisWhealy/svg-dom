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
fn demo_color_matrix_controls_update_saturate_hue_and_matrix_type_independently() -> Result<(), String> {
    container("demo-color-matrix");
    crate::paint::demo_color_matrix::demo()
        .map_err(|e| format!("demo_color_matrix::demo should build without error: {e:?}"))?;

    let root = document()
        .get_element_by_id("demo-color-matrix")
        .ok_or_else(|| "container exists".to_owned())?;

    const SEPIA_VALUES: &str = "0.393 0.769 0.189 0 0 0.349 0.686 0.168 0 0 0.272 0.534 0.131 0 0 0 0 0 1 0";

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

    // --- saturate: at this demo's own default ---
    let saturate = find_el("#cm-filter-saturate feColorMatrix")?;
    if saturate.get_attribute("type").as_deref() != Some("saturate") {
        return Err(format!("expected type \"saturate\", got {:?}", saturate.get_attribute("type")));
    }
    if saturate.get_attribute("values").as_deref() != Some("0") {
        return Err(format!(
            "0.0 is this demo's own initial default, got {:?}",
            saturate.get_attribute("values")
        ));
    }

    let saturate_slider = find_slider("input[aria-label='colour matrix saturate']")?;
    if saturate_slider.get_attribute("min").as_deref() != Some("0") {
        return Err(format!(
            "expected saturate_slider min \"0\", got {:?}",
            saturate_slider.get_attribute("min")
        ));
    }
    if saturate_slider.get_attribute("max").as_deref() != Some("200") {
        return Err(format!(
            "200% (2.0) demonstrates real oversaturation, not just the 1.0 identity endpoint, got {:?}",
            saturate_slider.get_attribute("max")
        ));
    }
    if saturate_slider.value() != "0" {
        return Err(format!(
            "expected saturate_slider value \"0\", got {:?}",
            saturate_slider.value()
        ));
    }
    if saturate_slider.get_attribute("aria-valuetext").as_deref() != Some("0.0") {
        return Err(format!(
            "expected saturate_slider aria-valuetext \"0.0\", got {:?}",
            saturate_slider.get_attribute("aria-valuetext")
        ));
    }

    let saturate_caption = find_text("Saturate(0.0)")?;

    // --- hue rotate: at this demo's own default ---
    let hue = find_el("#cm-filter-hue feColorMatrix")?;
    if hue.get_attribute("type").as_deref() != Some("hueRotate") {
        return Err(format!("expected type \"hueRotate\", got {:?}", hue.get_attribute("type")));
    }
    if hue.get_attribute("values").as_deref() != Some("180") {
        return Err(format!(
            "180 is this demo's own initial default, got {:?}",
            hue.get_attribute("values")
        ));
    }

    let hue_slider = find_slider("input[aria-label='colour matrix hue rotate']")?;
    if hue_slider.get_attribute("min").as_deref() != Some("0") {
        return Err(format!(
            "expected hue_slider min \"0\", got {:?}",
            hue_slider.get_attribute("min")
        ));
    }
    if hue_slider.get_attribute("max").as_deref() != Some("360") {
        return Err(format!(
            "expected hue_slider max \"360\", got {:?}",
            hue_slider.get_attribute("max")
        ));
    }
    if hue_slider.value() != "180" {
        return Err(format!("expected hue_slider value \"180\", got {:?}", hue_slider.value()));
    }
    if hue_slider.get_attribute("aria-valuetext").as_deref() != Some("180 degrees") {
        return Err(format!(
            "the raw slider value alone does not carry its own unit, got {:?}",
            hue_slider.get_attribute("aria-valuetext")
        ));
    }

    let hue_caption = find_text("HueRotate(180)")?;

    // --- matrix: at this demo's own default (sepia) ---
    let matrix = find_el("#cm-filter-matrix feColorMatrix")?;
    if matrix.get_attribute("type").as_deref() != Some("matrix") {
        return Err(format!("expected type \"matrix\", got {:?}", matrix.get_attribute("type")));
    }
    if matrix.get_attribute("values").as_deref() != Some(SEPIA_VALUES) {
        return Err(format!(
            "the sepia coefficients are this demo's own initial default, got {:?}",
            matrix.get_attribute("values")
        ));
    }

    let matrix_caption = find_text("Matrix (sepia)")?;
    let sepia = find_radio(&root, "matrix type", "sepia")?;
    let luminance = find_radio(&root, "matrix type", "luminance")?;
    if !sepia.checked() {
        return Err("sepia is this demo's own initial default".to_owned());
    }
    if luminance.checked() {
        return Err("luminance should not start checked".to_owned());
    }

    // --- an intermediate saturate position keeps its own full two-decimal precision, not just one decimal
    // place. The slider moves in 1% steps, so 101 distinct positions exist between 0 and 100. A fixed
    // one-decimal format would collapse those down to only 11 displayed values: 0.24 and 0.25 would both read
    // as "0.2". This pins the caption and aria-valuetext against an exact two-decimal value (0.25) and against
    // a value whose own trailing zero should be stripped (0.7, not 0.70), not just the two endpoints, where a
    // one-decimal format happens to already look correct. ---
    dispatch_input(&saturate_slider, "25")?;
    if saturate.get_attribute("values").as_deref() != Some("0.25") {
        return Err(format!("expected values \"0.25\", got {:?}", saturate.get_attribute("values")));
    }
    if saturate_slider.get_attribute("aria-valuetext").as_deref() != Some("0.25") {
        return Err(format!(
            "expected aria-valuetext \"0.25\", got {:?}",
            saturate_slider.get_attribute("aria-valuetext")
        ));
    }
    if saturate_caption.text_content().as_deref() != Some("Saturate(0.25)") {
        return Err(format!(
            "expected caption \"Saturate(0.25)\", got {:?}",
            saturate_caption.text_content()
        ));
    }

    dispatch_input(&saturate_slider, "70")?;
    if saturate.get_attribute("values").as_deref() != Some("0.7") {
        return Err(format!("expected values \"0.7\", got {:?}", saturate.get_attribute("values")));
    }
    if saturate_slider.get_attribute("aria-valuetext").as_deref() != Some("0.7") {
        return Err(format!(
            "expected aria-valuetext \"0.7\", got {:?}",
            saturate_slider.get_attribute("aria-valuetext")
        ));
    }
    if saturate_caption.text_content().as_deref() != Some("Saturate(0.7)") {
        return Err(format!(
            "expected caption \"Saturate(0.7)\", got {:?}",
            saturate_caption.text_content()
        ));
    }

    // --- raw 100 is the identity point (1.0), not the slider's own maximum ---
    dispatch_input(&saturate_slider, "100")?;
    if saturate.get_attribute("values").as_deref() != Some("1") {
        return Err(format!("expected values \"1\", got {:?}", saturate.get_attribute("values")));
    }
    if saturate_slider.get_attribute("aria-valuetext").as_deref() != Some("1.0") {
        return Err(format!(
            "expected aria-valuetext \"1.0\", got {:?}",
            saturate_slider.get_attribute("aria-valuetext")
        ));
    }
    if saturate_caption.text_content().as_deref() != Some("Saturate(1.0)") {
        return Err(format!(
            "expected caption \"Saturate(1.0)\", got {:?}",
            saturate_caption.text_content()
        ));
    }

    // --- moving saturate to its documented maximum demonstrates real oversaturation. The SVG default range
    // would stop at the identity point above; this slider's own extended range goes further. ---
    dispatch_input(&saturate_slider, "200")?;
    if saturate.get_attribute("values").as_deref() != Some("2") {
        return Err(format!("expected values \"2\", got {:?}", saturate.get_attribute("values")));
    }
    if saturate_slider.get_attribute("aria-valuetext").as_deref() != Some("2.0") {
        return Err(format!(
            "expected aria-valuetext \"2.0\", got {:?}",
            saturate_slider.get_attribute("aria-valuetext")
        ));
    }
    if saturate_caption.text_content().as_deref() != Some("Saturate(2.0)") {
        return Err(format!(
            "expected caption \"Saturate(2.0)\", got {:?}",
            saturate_caption.text_content()
        ));
    }
    if hue.get_attribute("values").as_deref() != Some("180") {
        return Err(format!(
            "moving the saturate slider should not touch hue rotate, got {:?}",
            hue.get_attribute("values")
        ));
    }
    if matrix.get_attribute("values").as_deref() != Some(SEPIA_VALUES) {
        return Err(format!(
            "moving the saturate slider should not touch the matrix type, got {:?}",
            matrix.get_attribute("values")
        ));
    }

    // --- moving hue rotate updates only the hue channel, caption, and aria-valuetext ---
    dispatch_input(&hue_slider, "45")?;
    if hue.get_attribute("values").as_deref() != Some("45") {
        return Err(format!("expected values \"45\", got {:?}", hue.get_attribute("values")));
    }
    if hue_caption.text_content().as_deref() != Some("HueRotate(45)") {
        return Err(format!(
            "expected caption \"HueRotate(45)\", got {:?}",
            hue_caption.text_content()
        ));
    }
    if hue_slider.get_attribute("aria-valuetext").as_deref() != Some("45 degrees") {
        return Err(format!(
            "expected aria-valuetext \"45 degrees\", got {:?}",
            hue_slider.get_attribute("aria-valuetext")
        ));
    }
    if saturate.get_attribute("values").as_deref() != Some("2") {
        return Err(format!(
            "moving the hue rotate slider should not touch saturate, which stays at its own last value, got {:?}",
            saturate.get_attribute("values")
        ));
    }

    // --- selecting luminance clears the matrix's own values attribute and updates its type and caption ---
    select_radio(&luminance)?;
    if matrix.get_attribute("type").as_deref() != Some("luminanceToAlpha") {
        return Err(format!(
            "expected type \"luminanceToAlpha\", got {:?}",
            matrix.get_attribute("type")
        ));
    }
    if matrix.get_attribute("values").is_some() {
        return Err("luminanceToAlpha needs no values attribute at all".to_owned());
    }
    if matrix_caption.text_content().as_deref() != Some("LuminanceToAlpha") {
        return Err(format!(
            "expected caption \"LuminanceToAlpha\", got {:?}",
            matrix_caption.text_content()
        ));
    }
    if !luminance.checked() {
        return Err("luminance should be checked".to_owned());
    }
    if sepia.checked() {
        return Err("selecting luminance should clear sepia".to_owned());
    }

    // --- selecting sepia again restores the exact same values text construction produced ---
    select_radio(&sepia)?;
    if matrix.get_attribute("type").as_deref() != Some("matrix") {
        return Err(format!("expected type \"matrix\", got {:?}", matrix.get_attribute("type")));
    }
    if matrix.get_attribute("values").as_deref() != Some(SEPIA_VALUES) {
        return Err(format!(
            "toggling back to sepia should restore identical values text, not a differently formatted \
             equivalent, got {:?}",
            matrix.get_attribute("values")
        ));
    }
    if matrix_caption.text_content().as_deref() != Some("Matrix (sepia)") {
        return Err(format!(
            "expected caption \"Matrix (sepia)\", got {:?}",
            matrix_caption.text_content()
        ));
    }
    if !sepia.checked() {
        return Err("sepia should be checked".to_owned());
    }
    if luminance.checked() {
        return Err("selecting sepia should clear luminance".to_owned());
    }

    if saturate.get_attribute("values").as_deref() != Some("2") {
        return Err(format!(
            "toggling the matrix radio group should not touch saturate, got {:?}",
            saturate.get_attribute("values")
        ));
    }
    if hue.get_attribute("values").as_deref() != Some("45") {
        return Err(format!(
            "toggling the matrix radio group should not touch hue rotate, got {:?}",
            hue.get_attribute("values")
        ));
    }

    // --- the original rectangle stays a plain, unfiltered comparison, untouched by every control above ---
    let rects = root.query_selector_all("rect").map_err(|e| format!("query rects: {e:?}"))?;
    if rects.length() != 5 {
        return Err(format!(
            "one original rectangle, one per control, and the matrix rectangle's own white backing rectangle, \
             got {}",
            rects.length()
        ));
    }
    let element_at = |index: u32| -> Result<web_sys::Element, String> {
        rects
            .item(index)
            .ok_or_else(|| format!("no rect at index {index}"))?
            .dyn_into::<web_sys::Element>()
            .map_err(|_| "expected an Element".to_owned())
    };
    let original = element_at(0)?;
    if original.get_attribute("filter").is_some() {
        return Err("the original rectangle carries no filter".to_owned());
    }

    // --- LuminanceToAlpha zeroes colour, leaving alpha as its only visible signal. Blending near-transparent
    // black into this gallery's own near-black canvas would crush that signal, so a plain white rectangle sits
    // behind the matrix rectangle. It must be drawn before that rectangle, in document order, or SVG's own
    // paint order would put it on top instead of underneath. ---
    let backing = element_at(3)?;
    if backing.get_attribute("fill").as_deref() != Some("white") {
        return Err(format!(
            "the backing rectangle should give LuminanceToAlpha's own alpha signal something to blend against, \
             got {:?}",
            backing.get_attribute("fill")
        ));
    }
    if backing.get_attribute("filter").is_some() {
        return Err(
            "the backing rectangle itself must stay unfiltered, or it would hide behind its own colour transform"
                .to_owned(),
        );
    }

    let matrix_rect = element_at(4)?;
    if matrix_rect.get_attribute("filter").as_deref() != Some("url(#cm-filter-matrix)") {
        return Err(format!(
            "the matrix rectangle should be the last rect, painted on top of its own backing rectangle, got {:?}",
            matrix_rect.get_attribute("filter")
        ));
    }
    Ok(())
}
