//! Tests for `demo_morphology`'s own radius slider, shared by its Erode, Dilate, and bold-outline columns.

use crate::browser_tests::{container, document};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `morphology` returns its own primitive's `SvgNode` directly, so `demo_morphology` retains each one the same way
/// `demo_turbulence`'s sliders do.
///
/// A real browser test is needed to prove:
/// 1) The slider actually reaches all three of its own retained nodes together, not just some of them.
/// 2) Radius `0` actually reaches all three filters, so each primitive itself becomes a pass-through, the identity case
///    `SvgFilter::morphology`'s own doc comment describes.
/// 3) That pass-through makes the Erode and Dilate columns match the unfiltered original exactly.
///    The bold-outline column does not match quite as exactly.
///    Its own merge step places a black `SourceAlpha` layer underneath the original graphic.
///    `feMerge`'s own source-over compositing darkens the antialiased edge pixels that layer sits under, even though
///    morphology itself contributes nothing there.
///    This test cannot see that difference. It only checks the `radius` attribute and the caption text, neither of
///    which carries pixel-level information.
/// 4) The slider's own tick marks actually land under the native thumb's own centre at every position, not just at the
///    track's own raw fractional position.
/// 5) The original column's own text stays untouched by the slider, unlike the other three columns.
/// 6) The bold-outline column's own filter graph actually has the shape that produces this exact outline effect.
///    `in="SourceAlpha"`, `result="thickened"`, and the merge's own two `feMergeNode`s reading `thickened` then
///    `SourceGraphic`, in that order, are each load-bearing on their own.
///    Changing any one of them would compile and run without error.
///    Swapping `in` to `SourceGraphic` would still dilate the silhouette, but in the glyphs' own steel-blue colour, not
///    black. Reversing the merge order would place the black dilated layer on top, obscuring the glyphs instead of
///    merely fringing them. Neither mistake looks like no outline as each produces a plausible, but wrong result.
#[wasm_bindgen_test]
fn demo_morphology_radius_slider_updates_erode_dilate_and_outline_together() -> Result<(), String> {
    container("demo-morphology");
    crate::paint::demo_morphology::demo()
        .map_err(|e| format!("demo_morphology::demo should build without error: {e:?}"))?;

    let root = document()
        .get_element_by_id("demo-morphology")
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

    // The Erode and Dilate filters, at this demo's own default
    let erode = find_el("#morphology-erode feMorphology")?;
    if erode.get_attribute("operator").as_deref() != Some("erode") {
        return Err(format!(
            "expected operator \"erode\", got {:?}",
            erode.get_attribute("operator")
        ));
    }
    if erode.get_attribute("radius").as_deref() != Some("1.2") {
        return Err(format!(
            "1.2 is this demo's own original fixed radius, kept as the slider's own initial default, got {:?}",
            erode.get_attribute("radius")
        ));
    }

    let dilate_filter = find_el("#morphology-dilate")?;
    if dilate_filter.get_attribute("x").as_deref() != Some("-0.5") {
        return Err(format!(
            "widen_filter_region's own fixed margin, needed so the slider's own larger radii do not clip, got {:?}",
            dilate_filter.get_attribute("x")
        ));
    }
    if dilate_filter.get_attribute("y").as_deref() != Some("-0.5") {
        return Err(format!("expected y \"-0.5\", got {:?}", dilate_filter.get_attribute("y")));
    }
    if dilate_filter.get_attribute("width").as_deref() != Some("2") {
        return Err(format!("expected width \"2\", got {:?}", dilate_filter.get_attribute("width")));
    }
    if dilate_filter.get_attribute("height").as_deref() != Some("2") {
        return Err(format!(
            "expected height \"2\", got {:?}",
            dilate_filter.get_attribute("height")
        ));
    }

    let dilate = find_el("#morphology-dilate feMorphology")?;
    if dilate.get_attribute("operator").as_deref() != Some("dilate") {
        return Err(format!(
            "expected operator \"dilate\", got {:?}",
            dilate.get_attribute("operator")
        ));
    }
    if dilate.get_attribute("radius").as_deref() != Some("1.2") {
        return Err(format!("expected radius \"1.2\", got {:?}", dilate.get_attribute("radius")));
    }

    let radius_slider = find_slider("input[aria-label='morphology radius']")?;
    if radius_slider.get_attribute("min").as_deref() != Some("0") {
        return Err(format!("expected min \"0\", got {:?}", radius_slider.get_attribute("min")));
    }
    if radius_slider.get_attribute("max").as_deref() != Some("40") {
        return Err(format!("expected max \"40\", got {:?}", radius_slider.get_attribute("max")));
    }
    if radius_slider.value() != "12" {
        return Err(format!("expected value \"12\", got {:?}", radius_slider.value()));
    }
    if radius_slider.get_attribute("aria-valuetext").as_deref() != Some("1.2") {
        return Err(format!(
            "expected aria-valuetext \"1.2\", got {:?}",
            radius_slider.get_attribute("aria-valuetext")
        ));
    }

    let erode_caption = find_text("Erode(1.2)")?;
    let dilate_caption = find_text("Dilate(1.2)")?;

    // The slider's own five tick marks sit under the native thumb's own actual centre at each position, not the track's
    // own bare fractional position — see paint/mod.rs's own SLIDER_THUMB_RADIUS_PX doc comment for why a raw percentage
    // would be wrong here.
    let tick_container = radius_slider
        .closest(".demo-slider-container")
        .map_err(|e| format!("query closest container: {e:?}"))?
        .ok_or_else(|| "radius_slider has a .demo-slider-container ancestor".to_owned())?;
    let tick_styles: Vec<String> = {
        let marks = tick_container
            .query_selector_all(".demo-tick-mark")
            .map_err(|e| format!("query tick marks: {e:?}"))?;
        let mut styles = Vec::new();
        for i in 0..marks.length() {
            let el = marks
                .item(i)
                .ok_or_else(|| "mark item".to_owned())?
                .dyn_into::<web_sys::Element>()
                .map_err(|_| "expected an Element".to_owned())?;
            styles.push(el.get_attribute("style").unwrap_or_default());
        }
        styles
    };
    let expected_tick_styles = vec![
        "left:calc(0.00% + 8.00px);".to_owned(),
        "left:calc(25.00% + 4.00px);".to_owned(),
        "left:calc(50.00% + 0.00px);".to_owned(),
        "left:calc(75.00% - 4.00px);".to_owned(),
        "left:calc(100.00% - 8.00px);".to_owned(),
    ];
    if tick_styles != expected_tick_styles {
        return Err(format!("unexpected tick styles: {tick_styles:?}"));
    }

    // The bold-outline column shares the same slider too: its own dilate radius moves together with the direct
    // Erode/Dilate columns, not independently of them
    let outline_filter = find_el("#morphology-outline")?;
    if outline_filter.get_attribute("x").as_deref() != Some("-0.5") {
        return Err(format!(
            "the bold-outline column needs the same widened region as the direct Dilate column, for the same \
             reason, got {:?}",
            outline_filter.get_attribute("x")
        ));
    }
    let outline = find_el("#morphology-outline feMorphology")?;
    if outline.get_attribute("operator").as_deref() != Some("dilate") {
        return Err(format!(
            "expected operator \"dilate\", got {:?}",
            outline.get_attribute("operator")
        ));
    }
    if outline.get_attribute("radius").as_deref() != Some("1.2") {
        return Err(format!("expected radius \"1.2\", got {:?}", outline.get_attribute("radius")));
    }
    if outline.get_attribute("in").as_deref() != Some("SourceAlpha") {
        return Err(format!(
            "the bold outline dilates the source's own alpha silhouette, not its full graphic, got {:?}",
            outline.get_attribute("in")
        ));
    }
    if outline.get_attribute("result").as_deref() != Some("thickened") {
        return Err(format!(
            "the merge below reads this primitive's own output by this name, got {:?}",
            outline.get_attribute("result")
        ));
    }

    // The merge's own two feMergeNode children must read in this exact order:
    // 1) the dilated fringe, then
    // 2) the original graphic on top of it.
    //
    // Reversing this order would place the black dilated silhouette over top of the original, obscuring or darkening
    // the steel-blue glyphs instead of leaving them on top of the fringe.
    let merge_node_inputs: Vec<Option<String>> = {
        let nodes = root
            .query_selector_all("#morphology-outline feMerge feMergeNode")
            .map_err(|e| format!("query feMergeNode elements: {e:?}"))?;
        let mut inputs = Vec::new();
        for i in 0..nodes.length() {
            let el = nodes
                .item(i)
                .ok_or_else(|| "feMergeNode item".to_owned())?
                .dyn_into::<web_sys::Element>()
                .map_err(|_| "expected an Element".to_owned())?;
            inputs.push(el.get_attribute("in"));
        }
        inputs
    };
    let expected_merge_node_inputs = vec![Some("thickened".to_owned()), Some("SourceGraphic".to_owned())];
    if merge_node_inputs != expected_merge_node_inputs {
        return Err(format!("unexpected feMergeNode inputs: {merge_node_inputs:?}"));
    }

    let outline_caption = find_text("bold outline (dilate 1.2 + merge)")?;

    // `SourceAlpha` has zero-valued colour channels, so the bold-outline column's own exposed fringe is plain black.
    // A white backing rectangle keeps that fringe visible against this gallery's own dark canvas — the same reason
    // `demo_color_matrix`'s own LuminanceToAlpha rectangle needs one. It must stay unfiltered and sit before the
    // outlined text in document order, or SVG's own paint order would put it on top instead of underneath.
    let rects = root
        .query_selector_all("rect")
        .map_err(|e| format!("query rect elements: {e:?}"))?;
    if rects.length() != 1 {
        return Err(format!(
            "the bold-outline column's own white backing rectangle is this demo's only rect, got {} rects",
            rects.length()
        ));
    }
    let backing = rects
        .item(0)
        .ok_or_else(|| "backing rect".to_owned())?
        .dyn_into::<web_sys::Element>()
        .map_err(|_| "expected an Element".to_owned())?;
    if backing.get_attribute("fill").as_deref() != Some("#ffffee") {
        return Err(format!("expected fill \"#ffffee\", got {:?}", backing.get_attribute("fill")));
    }
    if backing.get_attribute("filter").is_some() {
        return Err("the backing rectangle itself must stay unfiltered".to_owned());
    }
    let outlined_text = find_el("text[filter='url(#morphology-outline)']")?;
    let position = backing.compare_document_position(&outlined_text);
    if position & web_sys::Node::DOCUMENT_POSITION_FOLLOWING == 0 {
        return Err(
            "the backing rectangle must be drawn before the outlined text, so SVG's own paint order keeps it \
             underneath rather than on top"
                .to_owned(),
        );
    }

    // Moving the slider to zero disables all three primitives, not a maximally-thinned, maximally-thickened, or
    // maximally-fringed result.
    //
    // The Erode and Dilate columns then read exactly like an unfiltered `SourceGraphic` pass-through.
    // The bold-outline column does not read quite as exactly. Its own merge still darkens antialiased edge pixels
    // slightly, from the black `SourceAlpha` layer underneath, even with morphology itself inert.
    //
    // This only asserts the `radius` attribute and the caption text below, neither of which can see that
    // pixel-level difference.
    //
    // See this file's own module doc comment, point 3, for the full explanation.
    dispatch_input(&radius_slider, "0")?;
    if erode.get_attribute("radius").as_deref() != Some("0") {
        return Err(format!("expected radius \"0\", got {:?}", erode.get_attribute("radius")));
    }
    if dilate.get_attribute("radius").as_deref() != Some("0") {
        return Err(format!("expected radius \"0\", got {:?}", dilate.get_attribute("radius")));
    }
    if outline.get_attribute("radius").as_deref() != Some("0") {
        return Err(format!("expected radius \"0\", got {:?}", outline.get_attribute("radius")));
    }
    if radius_slider.get_attribute("aria-valuetext").as_deref() != Some("0") {
        return Err(format!(
            "expected aria-valuetext \"0\", got {:?}",
            radius_slider.get_attribute("aria-valuetext")
        ));
    }
    if erode_caption.text_content().as_deref() != Some("Erode(0)") {
        return Err(format!("expected caption \"Erode(0)\", got {:?}", erode_caption.text_content()));
    }
    if dilate_caption.text_content().as_deref() != Some("Dilate(0)") {
        return Err(format!(
            "expected caption \"Dilate(0)\", got {:?}",
            dilate_caption.text_content()
        ));
    }
    if outline_caption.text_content().as_deref() != Some("bold outline (dilate 0 + merge)") {
        return Err(format!(
            "expected caption \"bold outline (dilate 0 + merge)\", got {:?}",
            outline_caption.text_content()
        ));
    }

    // An intermediate value exercises the `on_input` handler's own division, not just the two ends of the
    // slider's own range.
    //
    // `0` (dispatched above) and `40` (dispatched below) are both exact multiples of `10`.
    // Neither can tell a working division from one accidentally rounded, truncated, or removed.
    // A broken integer division would still read `0` and `4` from those two values, the same as a correct one.
    // `13` divides to `1.3` instead, a value no such mistake could produce by accident.
    dispatch_input(&radius_slider, "13")?;
    if erode.get_attribute("radius").as_deref() != Some("1.3") {
        return Err(format!("expected radius \"1.3\", got {:?}", erode.get_attribute("radius")));
    }
    if dilate.get_attribute("radius").as_deref() != Some("1.3") {
        return Err(format!("expected radius \"1.3\", got {:?}", dilate.get_attribute("radius")));
    }
    if outline.get_attribute("radius").as_deref() != Some("1.3") {
        return Err(format!("expected radius \"1.3\", got {:?}", outline.get_attribute("radius")));
    }
    if radius_slider.get_attribute("aria-valuetext").as_deref() != Some("1.3") {
        return Err(format!(
            "expected aria-valuetext \"1.3\", got {:?}",
            radius_slider.get_attribute("aria-valuetext")
        ));
    }
    if erode_caption.text_content().as_deref() != Some("Erode(1.3)") {
        return Err(format!(
            "expected caption \"Erode(1.3)\", got {:?}",
            erode_caption.text_content()
        ));
    }
    if dilate_caption.text_content().as_deref() != Some("Dilate(1.3)") {
        return Err(format!(
            "expected caption \"Dilate(1.3)\", got {:?}",
            dilate_caption.text_content()
        ));
    }
    if outline_caption.text_content().as_deref() != Some("bold outline (dilate 1.3 + merge)") {
        return Err(format!(
            "expected caption \"bold outline (dilate 1.3 + merge)\", got {:?}",
            outline_caption.text_content()
        ));
    }

    // Moving the slider to its documented maximum updates every filter and caption together
    dispatch_input(&radius_slider, "40")?;
    if erode.get_attribute("radius").as_deref() != Some("4") {
        return Err(format!("expected radius \"4\", got {:?}", erode.get_attribute("radius")));
    }
    if dilate.get_attribute("radius").as_deref() != Some("4") {
        return Err(format!("expected radius \"4\", got {:?}", dilate.get_attribute("radius")));
    }
    if outline.get_attribute("radius").as_deref() != Some("4") {
        return Err(format!("expected radius \"4\", got {:?}", outline.get_attribute("radius")));
    }
    if radius_slider.get_attribute("aria-valuetext").as_deref() != Some("4") {
        return Err(format!(
            "expected aria-valuetext \"4\", got {:?}",
            radius_slider.get_attribute("aria-valuetext")
        ));
    }
    if erode_caption.text_content().as_deref() != Some("Erode(4)") {
        return Err(format!("expected caption \"Erode(4)\", got {:?}", erode_caption.text_content()));
    }
    if dilate_caption.text_content().as_deref() != Some("Dilate(4)") {
        return Err(format!(
            "expected caption \"Dilate(4)\", got {:?}",
            dilate_caption.text_content()
        ));
    }
    if outline_caption.text_content().as_deref() != Some("bold outline (dilate 4 + merge)") {
        return Err(format!(
            "expected caption \"bold outline (dilate 4 + merge)\", got {:?}",
            outline_caption.text_content()
        ));
    }
    if dilate_filter.get_attribute("width").as_deref() != Some("2") {
        return Err(format!(
            "the widened region should stay fixed, not shrink at the slider's own maximum, got {:?}",
            dilate_filter.get_attribute("width")
        ));
    }
    if dilate_filter.get_attribute("height").as_deref() != Some("2") {
        return Err(format!(
            "expected height \"2\", got {:?}",
            dilate_filter.get_attribute("height")
        ));
    }
    if outline_filter.get_attribute("width").as_deref() != Some("2") {
        return Err(format!(
            "the bold-outline column's own widened region should stay fixed too, got {:?}",
            outline_filter.get_attribute("width")
        ));
    }
    if outline_filter.get_attribute("height").as_deref() != Some("2") {
        return Err(format!(
            "expected height \"2\", got {:?}",
            outline_filter.get_attribute("height")
        ));
    }

    // The original column stays a plain, unfiltered comparison, untouched by every control above
    //
    // All four columns share the literal text "MORPH", so this checks the absence of a `filter` attribute
    // directly, rather than assuming the original column is the first "MORPH" text in document order.
    let morph_texts = root
        .query_selector_all("text")
        .map_err(|e| format!("query text elements: {e:?}"))?;
    let mut unfiltered_morph_texts: Vec<web_sys::Element> = Vec::new();
    for i in 0..morph_texts.length() {
        let el = morph_texts
            .item(i)
            .ok_or_else(|| "text item".to_owned())?
            .dyn_into::<web_sys::Element>()
            .map_err(|_| "expected an Element".to_owned())?;
        if el.text_content().as_deref() == Some("MORPH") && el.get_attribute("filter").is_none() {
            unfiltered_morph_texts.push(el);
        }
    }
    if unfiltered_morph_texts.len() != 1 {
        return Err(format!(
            "exactly one of the four MORPH texts (the original column's own) should carry no filter, got {}",
            unfiltered_morph_texts.len()
        ));
    }

    find_text("original")?;
    Ok(())
}
