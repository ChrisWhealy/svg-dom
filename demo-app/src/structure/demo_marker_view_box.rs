use crate::{
    DemoClosure, H, PAD_Y, W, caption,
    colours::*,
    dom_err,
    foreign_html::{foreign_object_document, xhtml},
    keep_demo_closure, keep_demo_node,
};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};
use wasm_bindgen::{JsCast, prelude::*};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgMarker::set_view_box — one shared triangle, a slider drives the viewBox window onto it
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    const BASE_W: f64 = 100.0;
    const BASE_H: f64 = 70.0;

    let svg = SvgRoot::create_in("demo-marker-view-box", Size::new(W, H))?;
    let defs = svg.defs()?;

    // The marker always renders the same polygon: a triangle from (0,0) to (100,35) to (0,70). Its markerWidth and
    // markerHeight stay fixed, maintaining the triangle's 10:7 aspect ratio. The slider below drives set_view_box()
    // directly, but it also has to move refX/refY in step (see that section's own comment for why) — this is not a
    // pure single-method demo, and the comments below say so explicitly.
    let triangle = [Point::new(0.0, 0.0), Point::new(100.0, 35.0), Point::new(0.0, 70.0)];
    let arrow = defs.build_marker("arrow-zoom", |m| {
        m.set_ref_x(BASE_W)?;
        m.set_ref_y(BASE_H / 2.0)?;
        m.set_marker_width(24.0)?;
        m.set_marker_height(16.8)?;
        m.set_view_box(0.0, 0.0, BASE_W, BASE_H)?;
        m.set_orient("auto")?;
        m.polygon(&triangle)?.set_fill(ACCENT_BLUE)?;
        Ok(())
    })?;

    let line = svg.line(Point::new(140.0, PAD_Y + 65.0), Point::new(650.0, PAD_Y + 65.0))?;
    line.set_stroke(ACCENT_BLUE)?;
    line.set_stroke_width(2.0)?;
    line.set_marker_end_ref(&arrow)?;

    let initial_readout = format!("viewBox {BASE_W} x {BASE_H}");
    let readout = svg.text(Point::new(140.0, PAD_Y + 45.0), &initial_readout)?;
    readout.set_fill(TEXT)?;
    readout.set_font_size(13.0)?;

    caption(
        &svg,
        W / 2.0,
        "one slider drives set_view_box() and its matching refX/refY — drag to zoom the arrowhead",
    )?;

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Interactive zoom slider
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Ranges from -50 to 50 (percent), zero at the centre. A positive value shrinks the viewBox, so it magnifies the
    // arrowhead and crops more of it away. A negative value enlarges the viewBox, so it zooms out. Width and height
    // always scale by the same factor, so the viewBox keeps its 10:7 aspect ratio at every position on the slider.
    //
    // Every slider event also moves refX/refY to (width, height / 2.0), the right-centre of the new viewBox. This is
    // not a side effect of set_view_box() itself: refX and refY are separate marker attributes, and this demo sets both
    // together so the arrowhead's tip stays aligned with the end of the line at every zoom level.
    //
    // refX and refY name a point in the marker's own coordinate space, not a fixed pixel offset, so as the viewBox's
    // width and height shrink or grow, the old refX/refY values would no longer point at the same place in the new
    // viewBox. Without moving them, set_view_box() alone would leave the arrowhead visibly drifting off the end of the
    // line as the slider moved — so in spite of the panel title, this demo shows viewBox and refX/refY working
    // together, not set_view_box() in isolation.
    //
    // Neither attribute reaches past the marker's own content, though. The <line> that references this marker via
    // marker-end keeps its own length and position no matter what the slider does.
    let slider_fo = svg.foreign_object(Point::new(140.0, 6.0), Size::new(400.0, 50.0))?;
    let slider_document = foreign_object_document(&slider_fo)?;

    let slider_container = xhtml(&slider_document, "div")?;
    slider_container
        .set_attribute("class", "demo-slider-container")
        .map_err(dom_err)?;

    let slider = xhtml(&slider_document, "input")?
        .dyn_into::<web_sys::HtmlInputElement>()
        .map_err(|_| Error::Dom("createElement(\"input\") did not return an HtmlInputElement".into()))?;
    slider.set_type("range");
    slider.set_min("-50");
    slider.set_max("50");
    slider.set_step("1");
    slider.set_value("0");
    slider.set_attribute("class", "demo-slider").map_err(dom_err)?;
    // No visible <label>: the "viewBox <w> x <h>" SVG text above already carries this for sighted users, but SVG
    // text content is not programmatically associated with an HTML control the way <label> is, so this needs its
    // own accessible name. aria-valuetext is kept in sync with the same text on every `input` event, below.
    slider.set_attribute("aria-label", "marker viewBox zoom").map_err(dom_err)?;
    slider.set_attribute("aria-valuetext", &initial_readout).map_err(dom_err)?;

    let endpoint_labels = xhtml(&slider_document, "div")?;
    endpoint_labels
        .set_attribute("class", "demo-endpoint-labels")
        .map_err(dom_err)?;

    for text in ["-50%", "0%", "+50%"] {
        let label = xhtml(&slider_document, "span")?;
        label.set_text_content(Some(text));
        endpoint_labels.append_child(&label).map_err(dom_err)?;
    }

    // `SvgMarker` is not `Clone` (unlike `SvgNode`), so `arrow` moves into this closure outright — there is only
    // ever one handle to it, and nothing after this point needs a second one.
    let slider_value = slider.clone();
    let readout_target = readout.clone();
    let mut label_buf = String::new();
    let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
        let percent: f64 = slider_value.value_as_number();
        let factor = 1.0 - percent / 100.0;
        let width = BASE_W * factor;
        let height = BASE_H * factor;

        let _ = arrow.set_ref_x(width);
        let _ = arrow.set_ref_y(height / 2.0);
        let _ = arrow.set_view_box(0.0, 0.0, width, height);
        let _ = readout_target.set_text_fmt(&mut label_buf, format_args!("viewBox {width:.0} x {height:.0}"));
        let _ = slider_value.set_attribute("aria-valuetext", &label_buf);
    });
    slider
        .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
        .map_err(dom_err)?;
    keep_demo_closure(on_input);

    slider_container.append_child(&slider).map_err(dom_err)?;
    slider_container.append_child(&endpoint_labels).map_err(dom_err)?;
    slider_fo.as_element().append_child(&slider_container).map_err(dom_err)?;
    keep_demo_node(slider_fo);

    Ok(())
}
