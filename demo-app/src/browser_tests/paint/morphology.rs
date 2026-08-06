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
/// 6) The bold-outline column's own filter graph actually has the shape that produces an outline at all.
///    `in="SourceAlpha"`, `result="thickened"`, and the merge's own two `feMergeNode`s reading `thickened` then
///    `SourceGraphic`, in that order, are each load-bearing on their own. Changing any one of them would compile and
///    run without error, but would silently stop producing a bold outline.
#[wasm_bindgen_test]
fn demo_morphology_radius_slider_updates_erode_dilate_and_outline_together() {
    container("demo-morphology");
    crate::paint::demo_morphology::demo().expect("demo_morphology::demo should build without error");

    let root = document().get_element_by_id("demo-morphology").expect("container exists");

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

    // --- the Erode and Dilate filters, at this demo's own default ---
    let erode = find_el("#morphology-erode feMorphology");
    assert_eq!(erode.get_attribute("operator").as_deref(), Some("erode"));
    assert_eq!(
        erode.get_attribute("radius").as_deref(),
        Some("1.2"),
        "1.2 is this demo's own original fixed radius, kept as the slider's own initial default"
    );

    let dilate_filter = find_el("#morphology-dilate");
    assert_eq!(
        dilate_filter.get_attribute("x").as_deref(),
        Some("-0.5"),
        "widen_filter_region's own fixed margin, needed so the slider's own larger radii do not clip"
    );
    assert_eq!(dilate_filter.get_attribute("y").as_deref(), Some("-0.5"));
    assert_eq!(dilate_filter.get_attribute("width").as_deref(), Some("2"));
    assert_eq!(dilate_filter.get_attribute("height").as_deref(), Some("2"));

    let dilate = find_el("#morphology-dilate feMorphology");
    assert_eq!(dilate.get_attribute("operator").as_deref(), Some("dilate"));
    assert_eq!(dilate.get_attribute("radius").as_deref(), Some("1.2"));

    let radius_slider = find_slider("input[aria-label='morphology radius']");
    assert_eq!(radius_slider.get_attribute("min").as_deref(), Some("0"));
    assert_eq!(radius_slider.get_attribute("max").as_deref(), Some("40"));
    assert_eq!(radius_slider.value(), "12");
    assert_eq!(radius_slider.get_attribute("aria-valuetext").as_deref(), Some("1.2"));

    let erode_caption = find_text("Erode(1.2)");
    let dilate_caption = find_text("Dilate(1.2)");

    // --- the slider's own five tick marks sit under the native thumb's own actual centre at each position, not
    // the track's own bare fractional position — see paint/mod.rs's own SLIDER_THUMB_RADIUS_PX doc comment for
    // why a raw percentage would be wrong here. ---
    let tick_container = radius_slider
        .closest(".demo-slider-container")
        .expect("query closest container")
        .expect("radius_slider has a .demo-slider-container ancestor");
    let tick_styles: Vec<String> = {
        let marks = tick_container.query_selector_all(".demo-tick-mark").expect("query tick marks");
        (0..marks.length())
            .map(|i| {
                marks
                    .item(i)
                    .expect("mark item")
                    .dyn_into::<web_sys::Element>()
                    .expect("Element")
                    .get_attribute("style")
                    .unwrap_or_default()
            })
            .collect()
    };
    assert_eq!(
        tick_styles,
        vec![
            "left:calc(0.00% + 8.00px);",
            "left:calc(25.00% + 4.00px);",
            "left:calc(50.00% + 0.00px);",
            "left:calc(75.00% - 4.00px);",
            "left:calc(100.00% - 8.00px);",
        ]
    );

    // --- the bold-outline column shares the same slider too: its own dilate radius moves together with the
    // direct Erode/Dilate columns, not independently of them ---
    let outline_filter = find_el("#morphology-outline");
    assert_eq!(
        outline_filter.get_attribute("x").as_deref(),
        Some("-0.5"),
        "the bold-outline column needs the same widened region as the direct Dilate column, for the same reason"
    );
    let outline = find_el("#morphology-outline feMorphology");
    assert_eq!(outline.get_attribute("operator").as_deref(), Some("dilate"));
    assert_eq!(outline.get_attribute("radius").as_deref(), Some("1.2"));
    assert_eq!(
        outline.get_attribute("in").as_deref(),
        Some("SourceAlpha"),
        "the bold outline dilates the source's own alpha silhouette, not its full graphic"
    );
    assert_eq!(
        outline.get_attribute("result").as_deref(),
        Some("thickened"),
        "the merge below reads this primitive's own output by this name"
    );

    // The merge's own two feMergeNode children must read in this exact order: the dilated fringe first, then the
    // original graphic on top of it. Reversed, the original graphic would sit underneath its own fringe instead of over
    // it, hiding the fringe rather than surrounding the glyphs with it.
    let merge_node_inputs: Vec<Option<String>> = {
        let nodes = root
            .query_selector_all("#morphology-outline feMerge feMergeNode")
            .expect("query feMergeNode elements");
        (0..nodes.length())
            .map(|i| {
                nodes
                    .item(i)
                    .expect("feMergeNode item")
                    .dyn_into::<web_sys::Element>()
                    .expect("Element")
                    .get_attribute("in")
            })
            .collect()
    };
    assert_eq!(
        merge_node_inputs,
        vec![Some("thickened".to_owned()), Some("SourceGraphic".to_owned())]
    );

    let outline_caption = find_text("bold outline (dilate 1.2 + merge)");

    // --- `SourceAlpha` has zero-valued colour channels, so the bold-outline column's own exposed fringe is
    // plain black. A white backing rectangle keeps that fringe visible against this gallery's own dark canvas —
    // the same reason `demo_color_matrix`'s own LuminanceToAlpha rectangle needs one. It must stay unfiltered
    // and sit before the outlined text in document order, or SVG's own paint order would put it on top instead
    // of underneath. ---
    let rects = root.query_selector_all("rect").expect("query rect elements");
    assert_eq!(
        rects.length(),
        1,
        "the bold-outline column's own white backing rectangle is this demo's only rect"
    );
    let backing = rects
        .item(0)
        .expect("backing rect")
        .dyn_into::<web_sys::Element>()
        .expect("Element");
    assert_eq!(backing.get_attribute("fill").as_deref(), Some("#ffffee"));
    assert!(
        backing.get_attribute("filter").is_none(),
        "the backing rectangle itself must stay unfiltered"
    );
    let outlined_text = find_el("text[filter='url(#morphology-outline)']");
    let position = backing.compare_document_position(&outlined_text);
    assert!(
        position & web_sys::Node::DOCUMENT_POSITION_FOLLOWING != 0,
        "the backing rectangle must be drawn before the outlined text, so SVG's own paint order keeps it \
         underneath rather than on top"
    );

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
    dispatch_input(&radius_slider, "0");
    assert_eq!(erode.get_attribute("radius").as_deref(), Some("0"));
    assert_eq!(dilate.get_attribute("radius").as_deref(), Some("0"));
    assert_eq!(outline.get_attribute("radius").as_deref(), Some("0"));
    assert_eq!(radius_slider.get_attribute("aria-valuetext").as_deref(), Some("0"));
    assert_eq!(erode_caption.text_content().as_deref(), Some("Erode(0)"));
    assert_eq!(dilate_caption.text_content().as_deref(), Some("Dilate(0)"));
    assert_eq!(
        outline_caption.text_content().as_deref(),
        Some("bold outline (dilate 0 + merge)")
    );

    // --- moving the slider to its documented maximum updates every filter and caption together ---
    dispatch_input(&radius_slider, "40");
    assert_eq!(erode.get_attribute("radius").as_deref(), Some("4"));
    assert_eq!(dilate.get_attribute("radius").as_deref(), Some("4"));
    assert_eq!(outline.get_attribute("radius").as_deref(), Some("4"));
    assert_eq!(radius_slider.get_attribute("aria-valuetext").as_deref(), Some("4"));
    assert_eq!(erode_caption.text_content().as_deref(), Some("Erode(4)"));
    assert_eq!(dilate_caption.text_content().as_deref(), Some("Dilate(4)"));
    assert_eq!(
        outline_caption.text_content().as_deref(),
        Some("bold outline (dilate 4 + merge)")
    );
    assert_eq!(
        dilate_filter.get_attribute("width").as_deref(),
        Some("2"),
        "the widened region should stay fixed, not shrink at the slider's own maximum"
    );
    assert_eq!(dilate_filter.get_attribute("height").as_deref(), Some("2"));
    assert_eq!(
        outline_filter.get_attribute("width").as_deref(),
        Some("2"),
        "the bold-outline column's own widened region should stay fixed too"
    );
    assert_eq!(outline_filter.get_attribute("height").as_deref(), Some("2"));

    // --- the original column stays a plain, unfiltered comparison, untouched by every control above ---
    //
    // All four columns share the literal text "MORPH", so this checks the absence of a `filter` attribute
    // directly, rather than assuming the original column is the first "MORPH" text in document order.
    let morph_texts = root.query_selector_all("text").expect("query text elements");
    let unfiltered_morph_texts: Vec<web_sys::Element> = (0..morph_texts.length())
        .map(|i| {
            morph_texts
                .item(i)
                .expect("text item")
                .dyn_into::<web_sys::Element>()
                .expect("Element")
        })
        .filter(|el| el.text_content().as_deref() == Some("MORPH") && el.get_attribute("filter").is_none())
        .collect();
    assert_eq!(
        unfiltered_morph_texts.len(),
        1,
        "exactly one of the four MORPH texts (the original column's own) should carry no filter"
    );

    find_text("original");
}
