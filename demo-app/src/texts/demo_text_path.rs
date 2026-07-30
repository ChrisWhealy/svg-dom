use super::sine_wave_path;
use crate::{
    DemoClosure, H, W, caption,
    colours::*,
    dom_err,
    foreign_html::{foreign_object_document, xhtml},
    keep_demo_closure, keep_demo_node,
};

use svg_dom::{
    Error, PathDef, PathDefAbsolute, SvgRoot,
    root::utils::{Point, Size},
};
use wasm_bindgen::{JsCast, prelude::*};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// textPath — gluing text to a curve
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-text-path", Size::new(W, H))?;

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // text follows a sine wave
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // The guide wave lives in <defs> which means it is never rendered unless it is first referenced by `href` — which
    // is the usual way to use a textPath: the geometry is simply a positioning aid for the text.
    //
    // Four periods of the sine wave are defined, but the text only covers part of the path's full length.
    let wave_d = sine_wave_path(20.0, 90.0, 1000.0, 40.0, 4.0);
    let defs = svg.build_defs(|d| {
        d.path(&wave_d)?.set_attr("id", "demo-tp-wave")?;
        Ok(())
    })?;
    let _ = defs; // held only to construct the wave; the id reference below is what matters

    let curve = svg.text(Point::origin(), "")?;
    let curve_path = curve.text_path("#demo-tp-wave", "Now riding a sine wave, up and down")?;
    curve_path.set_fill(PLAIN_TEXT)?;
    curve_path.set_font_size(14.0)?;

    caption(&svg, 200.0, "textPath — follows a sine wave")?;

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Interactive startOffset
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // The dashed guide arc is drawn directly on the canvas rather than being defined as a reusable object in <defs>, so
    // the effect of `set_start_offset` is visible. A single `<textPath>` element reads "Offset <n>" and slides along
    // the arc as the native HTML `<input type="range">` (housed in a `<foreignObject>`) is moved. The slider's own
    // `max` is the arc's `total_length()`, truncated to a whole user unit so it can never let the offset run past the
    // path's end.
    //
    // Unlike the two radio-driven demos above, this control is not built via a shared helper: it is the only slider
    // with its own tick-mark row (a `structure::demo_marker_view_box` slider exists too, but has no ticks), so
    // there is no second call site for a `slider_group` abstraction to actually serve. It still reuses `xhtml`/
    // `foreign_object_document` from `foreign_html` for the parts that genuinely are shared (creating XHTML
    // elements, finding the `<foreignObject>`'s owner document), and `demo/style.css`'s `.demo-slider*`/
    // `.demo-tick-*`/`.demo-endpoint-labels` classes for the parts of its styling that would otherwise repeat the
    // dark-theme palette inline in Rust.
    let guide = svg.path_from_defs(&[
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(430.0, 130.0))),
        PathDef::Abs(PathDefAbsolute::QuadraticBezierTo(
            Point::new(600.0, 45.0),
            Point::new(770.0, 130.0),
        )),
    ])?;
    guide.set_attr("id", "demo-tp-offset-arc")?;
    guide.set_fill(NONE)?;
    guide.set_stroke(GUIDE)?;
    guide.set_attr("stroke-dasharray", "4 3")?;

    // Measured once, right after the (already-live) guide has been created. `guide` is unconditionally a <path>, which
    // always implements `SVGGeometryElement` (see total_length's own doc comment for exactly which element types do
    // not), so `None` here would mean the browser's own getTotalLength() failed on an element that must support it —
    // which would represent a broken demo invariant rather than a normal runtime condition; hence an explicit
    // construction error rather than an `unwrap_or(0.0)` that would otherwise leave the slider silently stuck at a max
    // of zero.
    let offset_max = guide
        .total_length()
        .ok_or_else(|| Error::Dom("demo-tp-offset-arc has no measurable path length".into()))?;
    // Floored, not rounded: see this section's own doc comment above for why.
    let max_rounded = offset_max.floor() as i64;

    let offset_text = svg.text(Point::origin(), "")?;
    let offset_path = offset_text.text_path("#demo-tp-offset-arc", "Offset 0")?;

    offset_path.set_fill(WHITE)?;
    offset_path.set_font_size(16.0)?;

    // The slider sits above the curve's peak (a quadratic Bézier through the (600, 45) control point never actually
    // reaches it — the true peak is the curve's own y-midpoint, well below y=45 — so there is clear headroom here).
    let slider_fo = svg.foreign_object(Point::new(430.0, 6.0), Size::new(340.0, 50.0))?;
    let slider_document = foreign_object_document(&slider_fo)?;

    let slider_container = xhtml(&slider_document, "div")?;
    slider_container
        .set_attribute("class", "demo-slider-container")
        .map_err(dom_err)?;

    let slider = xhtml(&slider_document, "input")?
        .dyn_into::<web_sys::HtmlInputElement>()
        .map_err(|_| Error::Dom("createElement(\"input\") did not return an HtmlInputElement".into()))?;
    slider.set_type("range");
    slider.set_min("0");
    slider.set_max(&max_rounded.to_string());
    slider.set_step("1");
    slider.set_value("0");
    slider.set_attribute("class", "demo-slider").map_err(dom_err)?;

    // No visible <label> — the nearby "Offset <n>" SVG text already carries this information for sighted users, but SVG
    // text content is not programmatically associated with an HTML control the way <label> and aria-labelledby are, so
    // an explicit aria-label is needed to give this slider an accessible name.
    slider.set_attribute("aria-label", "textPath startOffset").map_err(dom_err)?;
    // aria-valuetext starts matching the slider's own initial value/text content below, and is kept in sync by every
    // `input` event (see that handler below for why this is the right way to expose the current offset to assistive
    // technology, rather than create a separate self-announcing live region (`aria-live`) on top of a control that
    // already has its own accessible value semantics).
    slider.set_attribute("aria-valuetext", "Offset 0").map_err(dom_err)?;

    // Tick marks every 25 units plus one at the end, hand-drawn using a `<span class="demo-tick-mark">` per tick, and a
    // `position:absolute` at `left: {value / max * 100}%`, rather than via a <datalist> linked through the slider's
    // `list` attribute.
    //
    // A <datalist>'s tick marks are only actually rendered by Chrome/Edge. Firefox and Safari apply the snap-to-value
    // behaviour but do not draw any visible marks at all, so on those browsers the ticks this demo asks for would not
    // be rendered. The hand-drawn marks do not depend on that browser-specific rendering gap.
    //
    // When CI invokes `browser-tests` in .github/workflows/ci.yml, it runs this file's interaction tests in Firefox and
    // Chrome. These tests dispatch DOM events and check attribute/value state, so they can confirm the slider still
    // works correctly in both browsers. They do not however, take screenshots or compare computed layout, so they
    // cannot confirm that there is pixel-perfect alignment between different browsers.
    //
    // No test coverage is provided for WebKit/Safari, because it needs a macOS runner which unfortunately lacks reliable
    // headless support.
    const TICK_STEP: i64 = 25;
    let ticks_row = xhtml(&slider_document, "div")?;
    ticks_row.set_attribute("class", "demo-tick-row").map_err(dom_err)?;

    let add_tick = |value: i64| -> Result<(), Error> {
        let percent = if max_rounded == 0 { 0.0 } else { value as f64 / max_rounded as f64 * 100.0 };
        let mark = xhtml(&slider_document, "span")?;
        mark.set_attribute("class", "demo-tick-mark").map_err(dom_err)?;
        // The one genuinely dynamic property per mark; everything else about its appearance lives in
        // demo/style.css's `.demo-tick-mark` rule.
        mark.set_attribute("style", &format!("left:{percent:.2}%;")).map_err(dom_err)?;
        ticks_row.append_child(&mark).map_err(dom_err)?;
        Ok(())
    };

    let mut tick = 0;
    while tick < max_rounded {
        add_tick(tick)?;
        tick += TICK_STEP;
    }
    add_tick(max_rounded)?;

    let endpoint_labels = xhtml(&slider_document, "div")?;
    endpoint_labels
        .set_attribute("class", "demo-endpoint-labels")
        .map_err(dom_err)?;

    let label_zero = xhtml(&slider_document, "span")?;
    label_zero.set_text_content(Some("0"));

    let label_max = xhtml(&slider_document, "span")?;
    label_max.set_text_content(Some(&max_rounded.to_string()));
    endpoint_labels.append_child(&label_zero).map_err(dom_err)?;
    endpoint_labels.append_child(&label_max).map_err(dom_err)?;

    // The closure reads the slider's value directly (a clone of the same `HtmlInputElement`, not the event's target).
    // While the slider is being dragged, the `input` event that fires continuously, rather than `change` that fires
    // only once the user lets go.
    let slider_value = slider.clone();
    let target = offset_path.clone();
    let mut label_buf = String::new();
    let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
        let offset: f64 = slider_value.value_as_number();
        let _ = target.set_start_offset(offset);
        // When in the home position (offset 0), the text color is white; anywhere else along the path, is orange.
        let _ = target.set_fill(if offset == 0.0 { WHITE } else { CORAL });
        let _ = target.set_text_fmt(&mut label_buf, format_args!("Offset {offset:.0}"));
        // Keeps the accessible value in step with the same text sighted users see on the curve, rather than
        // leaving assistive technology to announce just the bare number a native range input reports by default.
        let _ = slider_value.set_attribute("aria-valuetext", &label_buf);
    });
    slider
        .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
        .map_err(dom_err)?;
    keep_demo_closure(on_input);

    slider_container.append_child(&slider).map_err(dom_err)?;
    slider_container.append_child(&ticks_row).map_err(dom_err)?;
    slider_container.append_child(&endpoint_labels).map_err(dom_err)?;
    slider_fo.as_element().append_child(&slider_container).map_err(dom_err)?;
    keep_demo_node(slider_fo);

    caption(&svg, 600.0, "startOffset — drag the slider")?;

    Ok(())
}
